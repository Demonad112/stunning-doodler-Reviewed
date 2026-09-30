//! Tauri commands for the Transfer Monitor (see `transfer-core`).
//!
//! Long work (copy, retry, watch) runs on a blocking thread registered in [`TransferJobs`],
//! so it can be cancelled. The command returns at once; progress arrives as events:
//! - `transfer://progress`: [`TransferProgress`], at most every 100 ms
//! - `transfer://items`: `{ runId, items }`, item outcomes batched every 100 ms
//! - `transfer://finished`: `{ runId, summary, error }` when the job ends

use serde::{Deserialize, Serialize};
use shared_types::{AppErrorCode, AppErrorPayload};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};
use transfer_core::prepare::PreflightReport;
use transfer_core::recovery::RecoveryResult;
use transfer_core::run::Selection;
use transfer_core::store::RunStore;
use transfer_core::watch::{WatchControl, WatchOptions};
use transfer_core::{
    ItemResult, ItemStatus, RunSummary, TransferError, TransferProgress, TransferSettings,
    TransferSink,
};

const PROGRESS_EVENT: &str = "transfer://progress";
const ITEMS_EVENT: &str = "transfer://items";
const FINISHED_EVENT: &str = "transfer://finished";
const EMIT_EVERY: Duration = Duration::from_millis(100);
/// Copied files sent with a loaded run for the live feed.
const RECENT_ITEMS: usize = 200;

/// Cancel (and, for Watch, finish) switches of the jobs that are running, by run or request id.
#[derive(Default)]
pub struct TransferJobs(Mutex<HashMap<String, WatchControl>>);

impl TransferJobs {
    fn begin(&self, key: &str) -> Result<WatchControl, AppErrorPayload> {
        let mut jobs = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if jobs.contains_key(key) {
            return Err(error("This transfer is already running."));
        }
        let control = WatchControl::default();
        jobs.insert(key.to_owned(), control.clone());
        Ok(control)
    }

    fn end(&self, key: &str) {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(key);
    }

    fn get(&self, key: &str) -> Option<WatchControl> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(key)
            .cloned()
    }

    fn running(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

fn error(message: impl Into<String>) -> AppErrorPayload {
    AppErrorPayload::new(AppErrorCode::Unknown, "error.app.unknown.title", message)
}

fn map_error(err: TransferError) -> AppErrorPayload {
    match err {
        TransferError::InvalidPath(message) => AppErrorPayload::new(
            AppErrorCode::FileNotFound,
            "error.app.unknown.title",
            message,
        ),
        other => error(other.to_string()),
    }
}

async fn run_blocking<T, F>(work: F) -> Result<T, AppErrorPayload>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppErrorPayload> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|join| error(format!("transfer task failed: {join}")))?
}

fn root() -> PathBuf {
    transfer_core::transfers_root()
}

/// Forwards progress and items to the window, throttled.
struct EventSink {
    app: AppHandle,
    run_id: String,
    items: Vec<ItemResult>,
    items_sent: Instant,
    progress: Option<TransferProgress>,
    progress_sent: Instant,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemsPayload<'a> {
    run_id: &'a str,
    items: &'a [ItemResult],
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FinishedPayload {
    run_id: String,
    summary: Option<RunSummary>,
    error: Option<String>,
}

impl EventSink {
    fn new(app: AppHandle, run_id: &str) -> Self {
        let long_ago = Instant::now()
            .checked_sub(EMIT_EVERY)
            .unwrap_or_else(Instant::now);
        Self {
            app,
            run_id: run_id.to_owned(),
            items: Vec::new(),
            items_sent: long_ago,
            progress: None,
            progress_sent: long_ago,
        }
    }

    fn flush(&mut self) {
        if !self.items.is_empty() {
            let _ = self.app.emit(
                ITEMS_EVENT,
                ItemsPayload {
                    run_id: &self.run_id,
                    items: &self.items,
                },
            );
            self.items.clear();
        }
        self.items_sent = Instant::now();
        if let Some(progress) = self.progress.take() {
            let _ = self.app.emit(PROGRESS_EVENT, &progress);
        }
        self.progress_sent = Instant::now();
    }
}

impl TransferSink for EventSink {
    fn progress(&mut self, progress: &TransferProgress) {
        self.progress = Some(progress.clone());
        if self.progress_sent.elapsed() >= EMIT_EVERY {
            self.flush();
        }
    }

    fn item(&mut self, item: &ItemResult) {
        self.items.push(item.clone());
        if self.items_sent.elapsed() >= EMIT_EVERY {
            self.flush();
        }
    }
}

/// Runs `work` in the background as the job `key`, then emits `transfer://finished`.
fn spawn_job<F>(
    app: AppHandle,
    jobs: &TransferJobs,
    key: &str,
    work: F,
) -> Result<(), AppErrorPayload>
where
    F: FnOnce(&WatchControl, &mut EventSink) -> transfer_core::Result<RunSummary> + Send + 'static,
{
    let control = jobs.begin(key)?;
    let key = key.to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        let mut sink = EventSink::new(app.clone(), &key);
        let outcome = work(&control, &mut sink);
        sink.flush();
        use tauri::Manager;
        app.state::<TransferJobs>().end(&key);
        let (summary, error) = match outcome {
            Ok(summary) => (Some(summary), None),
            Err(err) => (None, Some(err.to_string())),
        };
        let _ = app.emit(
            FINISHED_EVENT,
            FinishedPayload {
                run_id: key,
                summary,
                error,
            },
        );
    });
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResponse {
    pub summary: RunSummary,
    pub preflight: PreflightReport,
}

/// Lists the source and runs the pre-flight check. `request_id` lets the UI cancel it.
#[tauri::command]
pub async fn transfer_prepare(
    app: AppHandle,
    jobs: State<'_, TransferJobs>,
    request_id: String,
    settings: TransferSettings,
) -> Result<PrepareResponse, AppErrorPayload> {
    let control = jobs.begin(&request_id)?;
    let key = request_id.clone();
    let result = run_blocking(move || {
        let mut sink = EventSink::new(app, &key);
        let result = transfer_core::prepare::prepare(&root(), settings, &control.cancel, &mut sink);
        sink.flush();
        result
            .map(|(summary, preflight)| PrepareResponse { summary, preflight })
            .map_err(map_error)
    })
    .await;
    jobs.end(&request_id);
    result
}

/// Starts copying a prepared run (Copy mode).
#[tauri::command]
pub fn transfer_start(
    app: AppHandle,
    jobs: State<'_, TransferJobs>,
    run_id: String,
) -> Result<(), AppErrorPayload> {
    let id = run_id.clone();
    spawn_job(app, &jobs, &run_id, move |control, sink| {
        transfer_core::run::run_copy(&root(), &id, None, &control.cancel, sink)
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchRequest {
    /// Seconds a file's size must stay unchanged before it counts as arrived.
    pub stable_seconds: u64,
    /// Finish by itself after this many quiet seconds; 0 waits for Finish.
    pub quiet_seconds: u64,
}

/// Starts watching the destination of a prepared Watch run.
#[tauri::command]
pub fn transfer_watch(
    app: AppHandle,
    jobs: State<'_, TransferJobs>,
    run_id: String,
    request: WatchRequest,
) -> Result<(), AppErrorPayload> {
    let id = run_id.clone();
    let options = WatchOptions {
        stable_for: Duration::from_secs(request.stable_seconds.max(1)),
        quiet_period: (request.quiet_seconds > 0)
            .then(|| Duration::from_secs(request.quiet_seconds)),
        ..WatchOptions::default()
    };
    spawn_job(app, &jobs, &run_id, move |control, sink| {
        transfer_core::watch::watch(&root(), &id, &options, control, sink)
    })
}

/// Watch mode: stop watching and check every file that has not arrived.
#[tauri::command]
pub fn transfer_watch_finish(jobs: State<'_, TransferJobs>, run_id: String) -> bool {
    jobs.get(&run_id)
        .map(|control| control.finish.cancel())
        .is_some()
}

/// Stops a running prepare, copy, retry or watch. Returns false when nothing was running.
#[tauri::command]
pub fn transfer_cancel(jobs: State<'_, TransferJobs>, run_id: String) -> bool {
    jobs.get(&run_id)
        .map(|control| control.cancel.cancel())
        .is_some()
}

/// Copies the selected not-copied files again.
#[tauri::command]
pub fn transfer_retry(
    app: AppHandle,
    jobs: State<'_, TransferJobs>,
    run_id: String,
    selection: Selection,
) -> Result<(), AppErrorPayload> {
    let id = run_id.clone();
    spawn_job(app, &jobs, &run_id, move |control, sink| {
        transfer_core::run::retry(&root(), &id, &selection, &control.cancel, sink)
    })
}

/// Copies the selected not-copied files to a recovery folder (default: beside the destination).
#[tauri::command]
pub async fn transfer_copy_to_recovery(
    jobs: State<'_, TransferJobs>,
    run_id: String,
    selection: Selection,
    folder: Option<String>,
) -> Result<RecoveryResult, AppErrorPayload> {
    if jobs.running(&run_id) {
        return Err(error("Wait until the transfer has finished."));
    }
    run_blocking(move || {
        let chosen = folder
            .filter(|folder| !folder.trim().is_empty())
            .map(PathBuf::from);
        transfer_core::recovery::copy_to_recovery(
            &root(),
            &run_id,
            &selection,
            chosen.as_deref(),
            &job_core::CancellationToken::default(),
        )
        .map_err(map_error)
    })
    .await
}

/// Stored runs, newest first.
#[tauri::command]
pub async fn transfer_list_runs() -> Result<Vec<RunSummary>, AppErrorPayload> {
    run_blocking(|| Ok(transfer_core::store::list_runs(&root()))).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDetails {
    pub summary: RunSummary,
    pub preflight: Option<PreflightReport>,
    pub not_copied: Vec<ItemResult>,
    /// The most recently copied files, newest last.
    pub recent: Vec<ItemResult>,
    pub running: bool,
    pub folder: String,
}

/// A stored run with its not-copied files, for reopening it.
#[tauri::command]
pub async fn transfer_load_run(
    jobs: State<'_, TransferJobs>,
    run_id: String,
) -> Result<RunDetails, AppErrorPayload> {
    let running = jobs.running(&run_id);
    run_blocking(move || {
        let store = RunStore::open(&root(), &run_id).map_err(map_error)?;
        let summary = store.summary().map_err(map_error)?;
        let preflight = store.preflight().ok();
        let latest = store.latest_results().map_err(map_error)?;
        let mut recent: Vec<ItemResult> = latest
            .values()
            .filter(|item| matches!(item.status, ItemStatus::Copied | ItemStatus::Arrived))
            .cloned()
            .collect();
        recent.sort_by_key(|item| item.at_ms);
        let recent = recent.split_off(recent.len().saturating_sub(RECENT_ITEMS));
        let not_copied = latest
            .into_values()
            .filter(|item| item.status == ItemStatus::NotCopied)
            .collect();
        Ok(RunDetails {
            summary,
            preflight,
            not_copied,
            recent,
            running,
            folder: store.dir().display().to_string(),
        })
    })
    .await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportExport {
    pub format: String,
    pub output_path: String,
    pub bytes_written: u64,
}

/// Writes the run's report as `html`, `csv` or `json`. With `choose_path` a save dialog asks
/// where (`None` if cancelled); otherwise it goes into the run folder.
#[tauri::command]
pub async fn transfer_export_report(
    run_id: String,
    format: String,
    choose_path: bool,
) -> Result<Option<ReportExport>, AppErrorPayload> {
    run_blocking(move || {
        let report = transfer_core::report::build_report(&root(), &run_id).map_err(map_error)?;
        let (extension, content) = match format.as_str() {
            "csv" => ("csv", report_core::render_transfer_csv_report(&report)),
            "json" => (
                "json",
                report_core::render_json_report(&report_core::transfer_report_to_unified(&report))
                    .map_err(|err| error(err.to_string()))?,
            ),
            _ => ("html", report_core::render_transfer_html_report(&report)),
        };
        let file_name = format!("DeepServer transfer {run_id}.{extension}");
        let output = if choose_path {
            let start = Path::new(&report.destination)
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default();
            match rfd::FileDialog::new()
                .set_file_name(&file_name)
                .set_directory(start)
                .add_filter(extension.to_uppercase(), &[extension])
                .save_file()
            {
                Some(path) => path,
                None => return Ok(None),
            }
        } else {
            RunStore::open(&root(), &run_id)
                .map_err(map_error)?
                .dir()
                .join(&file_name)
        };
        // A BOM so Excel opens non-ASCII names in the CSV correctly.
        let bytes = if extension == "csv" {
            [b"\xEF\xBB\xBF".as_slice(), content.as_bytes()].concat()
        } else {
            content.into_bytes()
        };
        std::fs::write(&output, &bytes)
            .map_err(|err| error(format!("{}: {err}", output.display())))?;
        Ok(Some(ReportExport {
            format: extension.to_owned(),
            output_path: output.display().to_string(),
            bytes_written: bytes.len() as u64,
        }))
    })
    .await
}
