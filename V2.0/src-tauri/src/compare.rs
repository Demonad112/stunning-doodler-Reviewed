//! Compare commands: scan both folders in parallel, stream progress over a channel, keep the
//! result in memory and hand it to the UI one folder level at a time.

use scan_core::{
    compare, scan, CancelToken, DiffRow, DiffSummary, DiffTree, ScanProgress, ScanState,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::ipc::Channel;
use tauri::State;

const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
/// Returned when a compare is cancelled; the UI matches on it to stay quiet.
const CANCELLED: &str = "cancelled";

/// The running compare's cancel token and the last finished result.
#[derive(Default)]
pub struct CompareState {
    cancel: Mutex<Option<CancelToken>>,
    result: Mutex<Option<DiffTree>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CompareEvent {
    Progress {
        left: ScanProgress,
        right: ScanProgress,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareResult {
    summary: DiffSummary,
    rows: Vec<DiffRow>,
    elapsed_ms: u64,
}

#[tauri::command]
pub async fn compare_start(
    state: State<'_, CompareState>,
    left: String,
    right: String,
    on_event: Channel<CompareEvent>,
) -> Result<CompareResult, String> {
    let left = PathBuf::from(left.trim());
    let right = PathBuf::from(right.trim());
    let cancel = CancelToken::new();
    if let Some(previous) = state
        .cancel
        .lock()
        .map_err(|err| err.to_string())?
        .replace(cancel.clone())
    {
        previous.cancel();
    }
    // Free the previous result before building the next one.
    state.result.lock().map_err(|err| err.to_string())?.take();

    let started = std::time::Instant::now();
    let job_cancel = cancel.clone();
    let diff =
        tauri::async_runtime::spawn_blocking(move || run(&left, &right, &job_cancel, &on_event))
            .await
            .map_err(|err| err.to_string())??;
    // A newer compare (or a cancel) arrived while this one was diffing.
    if cancel.is_cancelled() {
        return Err(CANCELLED.to_string());
    }

    let result = CompareResult {
        summary: diff.summary().clone(),
        rows: diff.children(0).unwrap_or_default(),
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    };
    *state.result.lock().map_err(|err| err.to_string())? = Some(diff);
    Ok(result)
}

fn run(
    left: &std::path::Path,
    right: &std::path::Path,
    cancel: &CancelToken,
    on_event: &Channel<CompareEvent>,
) -> Result<DiffTree, String> {
    let left_state = ScanState::new(cancel.clone());
    let right_state = ScanState::new(cancel.clone());
    let send_progress = || {
        // A closed channel (page left) is not an error; the scan still finishes.
        let _ = on_event.send(CompareEvent::Progress {
            left: left_state.progress(),
            right: right_state.progress(),
        });
    };

    let (left_tree, right_tree) = thread::scope(|scope| {
        let left_job = scope.spawn(|| scan(left, &left_state));
        let right_job = scope.spawn(|| scan(right, &right_state));
        while !(left_job.is_finished() && right_job.is_finished()) {
            send_progress();
            thread::sleep(PROGRESS_INTERVAL);
        }
        (left_job.join(), right_job.join())
    });
    send_progress();
    if cancel.is_cancelled() {
        return Err(CANCELLED.to_string());
    }

    let left_tree = left_tree
        .map_err(|_| "The source scan stopped unexpectedly.".to_string())?
        .map_err(|err| format!("Source: {err}"))?;
    let right_tree = right_tree
        .map_err(|_| "The destination scan stopped unexpectedly.".to_string())?
        .map_err(|err| format!("Destination: {err}"))?;
    Ok(compare(&left_tree, &right_tree))
}

/// Rows below one folder of the last compare.
#[tauri::command]
pub fn compare_children(state: State<'_, CompareState>, id: u32) -> Result<Vec<DiffRow>, String> {
    let result = state.result.lock().map_err(|err| err.to_string())?;
    result
        .as_ref()
        .and_then(|diff| diff.children(id))
        .ok_or_else(|| "That compare result is no longer available. Compare again.".to_string())
}

#[tauri::command]
pub fn compare_cancel(state: State<'_, CompareState>) -> Result<(), String> {
    if let Some(cancel) = state.cancel.lock().map_err(|err| err.to_string())?.as_ref() {
        cancel.cancel();
    }
    Ok(())
}
