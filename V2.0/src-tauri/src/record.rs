//! Record commands: list the source, then copy it (Copy mode) or watch another tool copy it
//! (Watch mode), streaming progress over a channel. Runs are stored by `transfer-core`, so a
//! finished record can be reopened as a report.

use scan_core::{check_pair, clean_path};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tauri::State;
use transfer_core::prepare::PreflightReport;
use transfer_core::recovery::RecoveryResult;
use transfer_core::run::Selection;
use transfer_core::store::{self, RunStore};
use transfer_core::watch::{WatchControl, WatchOptions};
use transfer_core::{
    ConflictPolicy, ItemResult, ItemStatus, Phase, RunSummary, TransferMode, TransferProgress,
    TransferSettings, TransferSink, VerifyLevel,
};

const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
/// Not-copied rows sent to the report page; the counts always cover all of them.
const MAX_NOT_COPIED_ROWS: usize = 10_000;

/// The running record's cancel and finish switches. One record at a time.
// ponytail: one job slot; a job map (like DeepServer 1) if two records ever need to run at once.
#[derive(Default)]
pub struct RecordState(Mutex<Option<WatchControl>>);

impl RecordState {
    fn begin(&self) -> Result<WatchControl, String> {
        let mut job = self.0.lock().map_err(|err| err.to_string())?;
        if job.is_some() {
            return Err("A record is already running. Finish or cancel it first.".into());
        }
        let control = WatchControl::default();
        *job = Some(control.clone());
        Ok(control)
    }

    fn end(&self) {
        if let Ok(mut job) = self.0.lock() {
            job.take();
        }
    }

    fn control(&self) -> Option<WatchControl> {
        self.0.lock().ok().and_then(|job| job.clone())
    }
}

/// Sends progress at most every 100 ms, and always the last one.
struct ChannelSink<'a> {
    channel: &'a Channel<TransferProgress>,
    sent: Option<Instant>,
    pending: Option<TransferProgress>,
}

impl<'a> ChannelSink<'a> {
    fn new(channel: &'a Channel<TransferProgress>) -> Self {
        Self {
            channel,
            sent: None,
            pending: None,
        }
    }

    fn flush(&mut self) {
        if let Some(progress) = self.pending.take() {
            // A closed channel (page left) is not an error; the job carries on.
            let _ = self.channel.send(progress);
            self.sent = Some(Instant::now());
        }
    }
}

impl TransferSink for ChannelSink<'_> {
    fn progress(&mut self, progress: &TransferProgress) {
        self.pending = Some(progress.clone());
        let due = self
            .sent
            .is_none_or(|sent| sent.elapsed() >= PROGRESS_INTERVAL);
        if due || progress.phase == Phase::Done {
            self.flush();
        }
    }

    fn item(&mut self, _item: &ItemResult) {}
}

/// A stored run, as the report page shows it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDetails {
    summary: RunSummary,
    preflight: Option<PreflightReport>,
    /// The first [`MAX_NOT_COPIED_ROWS`] not-copied rows, folders and files.
    not_copied: Vec<ItemResult>,
    /// Where the run's files are kept.
    folder: String,
}

fn load(run_id: &str) -> Result<RunDetails, String> {
    let store = RunStore::open(&transfer_core::records_root(), run_id).map_err(to_text)?;
    let not_copied = store
        .latest_results()
        .map_err(to_text)?
        .into_values()
        .filter(|item| item.status == ItemStatus::NotCopied)
        .take(MAX_NOT_COPIED_ROWS)
        .collect();
    Ok(RunDetails {
        summary: store.summary().map_err(to_text)?,
        preflight: store.preflight().ok(),
        not_copied,
        folder: store.dir().display().to_string(),
    })
}

fn to_text(err: impl ToString) -> String {
    err.to_string()
}

/// Runs `work` on a blocking thread as the one record job.
async fn job<F>(state: &RecordState, work: F) -> Result<RunDetails, String>
where
    F: FnOnce(&WatchControl) -> Result<String, String> + Send + 'static,
{
    let control = state.begin()?;
    let outcome = tauri::async_runtime::spawn_blocking(move || work(&control)).await;
    state.end();
    let run_id = outcome.map_err(to_text)??;
    tauri::async_runtime::spawn_blocking(move || load(&run_id))
        .await
        .map_err(to_text)?
}

/// Lists the source, then copies it or watches the destination. Returns when the record ends:
/// copy finished, watch finished (by [`record_finish`] or once everything arrived), or cancel.
#[tauri::command]
pub async fn record_start(
    state: State<'_, RecordState>,
    source: String,
    destination: String,
    mode: TransferMode,
    verify: VerifyLevel,
    on_event: Channel<TransferProgress>,
) -> Result<RunDetails, String> {
    let source = clean_path(&source);
    let destination = clean_path(&destination);
    check_pair(&source, &destination)?;
    let settings = TransferSettings {
        source: source.display().to_string(),
        destination: destination.display().to_string(),
        mode,
        verify,
        conflict: ConflictPolicy::Skip,
    };
    job(&state, move |control| {
        let root = transfer_core::records_root();
        let mut sink = ChannelSink::new(&on_event);
        let (summary, preflight) =
            transfer_core::prepare::prepare(&root, settings, &control.cancel, &mut sink)
                .map_err(to_text)?;
        if mode == TransferMode::Copy {
            if let Some(error) = &preflight.write_error {
                return Err(format!(
                    "DeepServer can't write to the destination: {error}"
                ));
            }
            if !preflight.enough_space {
                return Err(format!(
                    "Not enough space at the destination: the copy needs {} bytes and {} are free.",
                    preflight.bytes_needed,
                    preflight.free_bytes.unwrap_or_default()
                ));
            }
        }
        let finished = match mode {
            TransferMode::Copy => {
                transfer_core::run::run_copy(&root, &summary.id, None, &control.cancel, &mut sink)
            }
            TransferMode::Watch => transfer_core::watch::watch(
                &root,
                &summary.id,
                &WatchOptions::default(),
                control,
                &mut sink,
            ),
        };
        sink.flush();
        finished.map(|summary| summary.id).map_err(to_text)
    })
    .await
}

/// Copies the not-copied files again: the given relative paths, or all of them.
#[tauri::command]
pub async fn record_retry(
    state: State<'_, RecordState>,
    run_id: String,
    paths: Vec<String>,
    on_event: Channel<TransferProgress>,
) -> Result<RunDetails, String> {
    job(&state, move |control| {
        let mut sink = ChannelSink::new(&on_event);
        let selection = Selection {
            paths,
            reason: None,
        };
        let finished = transfer_core::run::retry(
            &transfer_core::records_root(),
            &run_id,
            &selection,
            &control.cancel,
            &mut sink,
        );
        sink.flush();
        finished.map(|summary| summary.id).map_err(to_text)
    })
    .await
}

/// Watch mode: stop watching and give every file that has not arrived a verdict.
#[tauri::command]
pub fn record_finish(state: State<'_, RecordState>) -> bool {
    state
        .control()
        .map(|control| control.finish.cancel())
        .is_some()
}

/// Stops the running record. Copy mode lists the rest as Cancelled, so Retry continues it.
#[tauri::command]
pub fn record_cancel(state: State<'_, RecordState>) -> bool {
    state
        .control()
        .map(|control| control.cancel.cancel())
        .is_some()
}

/// Copies every not-copied file to `folder` (default: `<destination>_NotCopied`) with a list.
#[tauri::command]
pub async fn record_recover(
    state: State<'_, RecordState>,
    run_id: String,
    folder: Option<String>,
) -> Result<RecoveryResult, String> {
    let control = state.begin()?;
    let result = tauri::async_runtime::spawn_blocking(move || {
        let chosen = folder
            .filter(|folder| !folder.trim().is_empty())
            .map(|folder| clean_path(&folder));
        transfer_core::recovery::copy_to_recovery(
            &transfer_core::records_root(),
            &run_id,
            &Selection::default(),
            chosen.as_deref(),
            &control.cancel,
            &mut |_, _| {},
        )
        .map_err(to_text)
    })
    .await;
    state.end();
    result.map_err(to_text)?
}

#[tauri::command]
pub async fn record_load(run_id: String) -> Result<RunDetails, String> {
    tauri::async_runtime::spawn_blocking(move || load(&run_id))
        .await
        .map_err(to_text)?
}

/// Stored records, newest first.
#[tauri::command]
pub async fn record_list() -> Result<Vec<RunSummary>, String> {
    tauri::async_runtime::spawn_blocking(|| store::list_runs(&transfer_core::records_root()))
        .await
        .map_err(to_text)
}

/// Opens Explorer at a file or folder (a source file, the recovery folder).
#[tauri::command]
pub fn record_reveal(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    tauri_plugin_opener::reveal_item_in_dir(&path)
        .map_err(|err| format!("Could not open {} in Explorer: {err}", path.display()))
}

/// Startup clean-up of old records (older than 90 days, beyond the newest 200), at most once a
/// day, on a background thread.
pub fn prune_in_background() {
    std::thread::spawn(|| {
        let root = transfer_core::records_root();
        let now = transfer_core::now_ms();
        if store::prune_due(&root, now, store::STARTUP_PRUNE_EVERY_MS) {
            store::prune_runs(
                &root,
                store::KEEP_RUNS_DAYS,
                store::KEEP_RUNS_MAX,
                now,
                &|_| false,
            );
            store::mark_pruned(&root, now);
        }
    });
}
