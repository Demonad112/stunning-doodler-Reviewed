//! Disk Cleanup commands: scan one folder or drive with streamed progress, keep the tree in
//! memory, hand the UI one folder level at a time, and recycle or delete items from it.

use cleanup_core::quick::{self, Places, QuickId, QuickWin};
use cleanup_core::{Drive, Overview, Protected, Row};
use report_core::{
    Body, CleanReport, CleanedItem, FileItem, FolderItem, JobInfo, Removed, Saved, TypeItem,
    UsageReport,
};
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
    /// The scan's saved report; deletes are added to it.
    report: Mutex<Option<Saved>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    path: String,
    overview: Overview,
    rows: Vec<Row>,
    elapsed_ms: u64,
    report_id: Option<String>,
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
    job: JobInfo,
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

    let overview = cleanup_core::overview(&tree);
    let rows = cleanup_core::rows(&tree, 0);
    let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let report = usage_report(&path, &tree, &overview, &rows, elapsed_ms);
    let saved = report_core::save(&report_core::reports_root(), job, Body::DiskUsage(report)).ok();
    let result = ScanResult {
        path: path.display().to_string(),
        overview,
        rows,
        elapsed_ms,
        report_id: saved.as_ref().map(|saved| saved.id.clone()),
    };
    *lock(&state.tree)? = Some(tree);
    *lock(&state.report)? = saved;
    Ok(result)
}

/// Lines kept per list in a saved disk usage report.
const REPORT_ROWS: usize = 25;

fn usage_report(
    path: &std::path::Path,
    tree: &Tree,
    overview: &Overview,
    rows: &[Row],
    elapsed_ms: u64,
) -> UsageReport {
    let root = &overview.root;
    let drive = cleanup_core::drives().into_iter().find(|drive| {
        drive
            .path
            .trim_end_matches('\\')
            .eq_ignore_ascii_case(path.display().to_string().trim_end_matches(['\\', '/']))
    });
    UsageReport {
        path: path.display().to_string(),
        size: root.size,
        files: root.files,
        dirs: root.dirs,
        unreadable: root.errors,
        cloud_files: root.cloud_files,
        cloud_bytes: root.cloud_bytes,
        drive_total: drive.as_ref().map(|drive| drive.total),
        drive_free: drive.as_ref().map(|drive| drive.free),
        elapsed_ms,
        top_folders: rows
            .iter()
            .take(REPORT_ROWS)
            .map(|row| FolderItem {
                name: tree.path(row.id).display().to_string(),
                size: row.size,
                files: row.files,
            })
            .collect(),
        largest_files: overview
            .largest_files
            .iter()
            .take(REPORT_ROWS)
            .map(|file| FileItem {
                path: tree.path(file.id).display().to_string(),
                size: file.size,
            })
            .collect(),
        types: overview
            .file_types
            .iter()
            .take(REPORT_ROWS)
            .map(|kind| TypeItem {
                extension: kind.extension.clone(),
                size: kind.size,
                files: kind.files,
            })
            .collect(),
        removed: Vec::new(),
    }
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
    let (path, root, size) = with_tree(&state, |tree| {
        if id == 0 || !tree.is_live(id) {
            return Err("That item is already gone. Scan again to refresh.".to_string());
        }
        let node = tree.node(id);
        Ok((
            tree.path(id),
            tree.path(0),
            node.size.saturating_sub(node.cloud_bytes),
        ))
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
        if let Some(saved) = lock(&state.report)?.as_mut() {
            if let Body::DiskUsage(report) = &mut saved.body {
                report.removed.push(Removed {
                    path: path.display().to_string(),
                    size,
                    permanent,
                });
                // The delete happened; a report that can't be updated isn't worth an error.
                let _ = report_core::write(&report_core::reports_root(), saved);
            }
        }
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

/// The quick cleanup cards, measured now.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickList {
    /// Running as administrator: the system cleanups can run.
    elevated: bool,
    items: Vec<QuickWin>,
}

#[tauri::command]
pub async fn cleanup_quick_list() -> Result<QuickList, String> {
    tauri::async_runtime::spawn_blocking(|| QuickList {
        elevated: cleanup_core::is_elevated(),
        items: quick::list(&Places::for_this_pc()),
    })
    .await
    .map_err(|err| err.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickResult {
    report: CleanReport,
    report_id: Option<String>,
}

/// Runs the chosen quick cleanups one after another and saves what they freed as a report.
#[tauri::command]
pub async fn cleanup_quick_clean(ids: Vec<QuickId>, job: JobInfo) -> Result<QuickResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let places = Places::for_this_pc();
        let elevated = cleanup_core::is_elevated();
        let items = ids
            .into_iter()
            .map(|id| {
                if id.needs_admin() && !elevated {
                    return CleanedItem {
                        title: id.title().into(),
                        error: Some("needs administrator".into()),
                        ..CleanedItem::default()
                    };
                }
                match quick::clean(&places, id) {
                    Ok(cleaned) => CleanedItem {
                        title: id.title().into(),
                        freed: cleaned.freed,
                        removed_files: cleaned.removed,
                        skipped_files: cleaned.skipped,
                        error: None,
                    },
                    Err(error) => CleanedItem {
                        title: id.title().into(),
                        error: Some(error),
                        ..CleanedItem::default()
                    },
                }
            })
            .collect();
        let report = CleanReport { items };
        let report_id = report_core::save(
            &report_core::reports_root(),
            job,
            Body::Cleanup(report.clone()),
        )
        .map(|saved| saved.id)
        .ok();
        QuickResult { report, report_id }
    })
    .await
    .map_err(|err| err.to_string())
}

/// Starts DeepServer again as administrator (Windows asks first), then closes this copy.
#[tauri::command]
pub fn app_restart_admin(
    app: tauri::AppHandle,
    record: tauri::State<'_, crate::record::RecordState>,
) -> Result<(), String> {
    if record.is_running() {
        return Err("A record is running. Finish or cancel it before restarting.".into());
    }
    run_as_admin()?;
    app.exit(0);
    Ok(())
}

#[cfg(windows)]
fn run_as_admin() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: *mut std::ffi::c_void,
            verb: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show: i32,
        ) -> isize;
    }
    const SW_SHOWNORMAL: i32 = 1;
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let wide = |text: &std::ffi::OsStr| -> Vec<u16> { text.encode_wide().chain([0]).collect() };
    let verb = wide("runas".as_ref());
    let file = wide(exe.as_os_str());
    // SAFETY: null-terminated strings that outlive the call; null window, parameters and folder.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    // Values up to 32 are errors, including "the user said no".
    if result <= 32 {
        return Err("DeepServer was not restarted as administrator.".into());
    }
    Ok(())
}

#[cfg(not(windows))]
fn run_as_admin() -> Result<(), String> {
    Err("Only on Windows.".into())
}
