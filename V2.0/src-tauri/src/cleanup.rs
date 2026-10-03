//! Disk Cleanup commands: scan one folder or drive with streamed progress, keep the tree in
//! memory, hand the UI one folder level at a time, and recycle or delete items from it.

use cleanup_core::{Drive, Overview, Protected, Row};
use scan_core::{clean_path, scan, CancelToken, NodeId, ScanProgress, ScanState, Tree};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tauri::State;

const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
/// Returned when a scan is cancelled; the UI matches on it to stay quiet.
const CANCELLED: &str = "cancelled";
const NO_RESULT: &str = "That scan is no longer available. Scan again.";

#[derive(Default)]
pub struct CleanupState {
    cancel: Mutex<Option<CancelToken>>,
    tree: Mutex<Option<Tree>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    path: String,
    overview: Overview,
    rows: Vec<Row>,
    elapsed_ms: u64,
}

#[tauri::command]
pub async fn cleanup_drives() -> Result<Vec<Drive>, String> {
    tauri::async_runtime::spawn_blocking(cleanup_core::drives)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn cleanup_scan(
    state: State<'_, CleanupState>,
    path: String,
    on_progress: Channel<ScanProgress>,
) -> Result<ScanResult, String> {
    let path = clean_path(&path);
    if path.as_os_str().is_empty() {
        return Err("Pick a drive or folder to scan.".into());
    }
    let cancel = CancelToken::new();
    if let Some(previous) = lock(&state.cancel)?.replace(cancel.clone()) {
        previous.cancel();
    }
    // Free the previous tree before building the next one.
    lock(&state.tree)?.take();

    let started = Instant::now();
    let job_cancel = cancel.clone();
    let job_path = path.clone();
    let tree = tauri::async_runtime::spawn_blocking(move || {
        let scan_state = ScanState::new(job_cancel);
        thread::scope(|scope| {
            let job = scope.spawn(|| scan(&job_path, &scan_state));
            while !job.is_finished() {
                // A closed channel (page left) is not an error; the scan still finishes.
                let _ = on_progress.send(scan_state.progress());
                thread::sleep(PROGRESS_INTERVAL);
            }
            let _ = on_progress.send(scan_state.progress());
            job.join()
        })
    })
    .await
    .map_err(|err| err.to_string())?
    .map_err(|_| "The scan stopped unexpectedly.".to_string())?;
    if cancel.is_cancelled() {
        return Err(CANCELLED.into());
    }
    let tree = tree.map_err(|err| err.to_string())?;

    let result = ScanResult {
        path: path.display().to_string(),
        overview: cleanup_core::overview(&tree),
        rows: cleanup_core::rows(&tree, 0),
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    };
    *lock(&state.tree)? = Some(tree);
    Ok(result)
}

#[tauri::command]
pub fn cleanup_cancel(state: State<'_, CleanupState>) -> Result<(), String> {
    if let Some(cancel) = lock(&state.cancel)?.as_ref() {
        cancel.cancel();
    }
    Ok(())
}

/// What is directly inside one folder of the last scan.
#[tauri::command]
pub fn cleanup_children(state: State<'_, CleanupState>, id: NodeId) -> Result<Vec<Row>, String> {
    with_tree(&state, |tree| Ok(cleanup_core::rows(tree, id)))
}

#[tauri::command]
pub fn cleanup_path(state: State<'_, CleanupState>, id: NodeId) -> Result<String, String> {
    item_path(&state, id).map(|path| path.display().to_string())
}

/// Opens Explorer with the item selected.
#[tauri::command]
pub fn cleanup_reveal(state: State<'_, CleanupState>, id: NodeId) -> Result<(), String> {
    let path = item_path(&state, id)?;
    tauri_plugin_opener::reveal_item_in_dir(&path)
        .map_err(|err| format!("Could not open {} in Explorer: {err}", path.display()))
}

/// Recycles (or, when `permanent`, deletes) one item, then drops it from the scan. Returns the
/// updated totals and lists.
#[tauri::command]
pub async fn cleanup_delete(
    state: State<'_, CleanupState>,
    id: NodeId,
    permanent: bool,
) -> Result<Overview, String> {
    let (path, root) = with_tree(&state, |tree| {
        if id == 0 || !tree.is_live(id) {
            return Err("That item is already gone. Scan again to refresh.".to_string());
        }
        Ok((tree.path(id), tree.path(0)))
    })?;
    Protected::for_this_pc().check(&path, &root)?;

    let job_path = path.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        if permanent {
            cleanup_core::delete_permanently(&job_path)
        } else {
            cleanup_core::recycle(&job_path)
        }
    })
    .await
    .map_err(|err| err.to_string())?;
    // Even a failed delete can take part of a folder; only a fully gone item leaves the tree.
    let gone = std::fs::symlink_metadata(&path).is_err();

    let mut guard = lock(&state.tree)?;
    let tree = guard.as_mut().ok_or(NO_RESULT)?;
    // A new scan may have replaced the tree while Windows was deleting.
    if gone && tree.is_live(id) && tree.path(id) == path {
        tree.remove(id);
    }
    if !gone {
        outcome?;
        return Err(format!(
            "{} is still there. Some files in it may be open in another program.",
            path.display()
        ));
    }
    Ok(cleanup_core::overview(tree))
}

fn lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    mutex.lock().map_err(|err| err.to_string())
}

fn with_tree<R>(
    state: &CleanupState,
    read: impl FnOnce(&Tree) -> Result<R, String>,
) -> Result<R, String> {
    let guard = lock(&state.tree)?;
    read(guard.as_ref().ok_or(NO_RESULT)?)
}

fn item_path(state: &CleanupState, id: NodeId) -> Result<PathBuf, String> {
    with_tree(state, |tree| {
        if tree.is_live(id) {
            Ok(tree.path(id))
        } else {
            Err("That item is no longer there.".into())
        }
    })
}
