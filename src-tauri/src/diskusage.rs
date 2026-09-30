//! Tauri commands for the bundled disk-usage engine (see `diskusage-core`).

use diskusage_core::{ChangeRow, DiskUsageError, DriveInfo, SnapshotInfo};
use shared_types::{AppErrorCode, AppErrorPayload};
use std::path::{Path, PathBuf};

/// File name of the engine next to `DeepServer.exe`; Tauri's externalBin drops the target triple.
const ENGINE_FILE_NAME: &str = "deepserver-diskusage.exe";
/// Overrides where the engine is looked up (tests, portable layouts).
const ENGINE_ENV: &str = "DEEPSERVER_DISKUSAGE_EXE";

fn locate_engine() -> Result<PathBuf, AppErrorPayload> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os(ENGINE_ENV).filter(|value| !value.is_empty()) {
        candidates.push(PathBuf::from(path));
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        candidates.push(dir.join(ENGINE_FILE_NAME));
    }
    // `tauri dev` / cargo runs: use the engine built in the repo (native/diskusage/build).
    if cfg!(debug_assertions) {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../native/diskusage/build/altWinDirStat_x64.exe"),
        );
    }

    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            error(format!(
                "The disk-usage engine ({ENGINE_FILE_NAME}) was not found next to DeepServer. \
                 Reinstall DeepServer, or set {ENGINE_ENV}."
            ))
        })
}

fn error(message: impl Into<String>) -> AppErrorPayload {
    AppErrorPayload::new(AppErrorCode::Unknown, "error.app.unknown.title", message)
}

fn map_error(err: DiskUsageError) -> AppErrorPayload {
    match err {
        DiskUsageError::InvalidPath(path) => AppErrorPayload::new(
            AppErrorCode::FileNotFound,
            "error.app.unknown.title",
            format!("Not a folder or drive: {path}"),
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
        .map_err(|join| error(format!("disk-usage task failed: {join}")))?
}

/// Lettered drives (local, removable, network) with label and free/total space.
#[tauri::command]
pub async fn diskusage_list_drives() -> Result<Vec<DriveInfo>, AppErrorPayload> {
    run_blocking(|| Ok(diskusage_core::list_drives())).await
}

/// Opens the engine's treemap window on a folder or drive.
#[tauri::command]
pub fn diskusage_open(path: String) -> Result<(), AppErrorPayload> {
    let engine = locate_engine()?;
    diskusage_core::open_in_engine(&engine, &path).map_err(map_error)
}

/// Snapshots stored for a location, newest first.
#[tauri::command]
pub async fn diskusage_list_snapshots(path: String) -> Result<Vec<SnapshotInfo>, AppErrorPayload> {
    run_blocking(move || {
        diskusage_core::list_snapshots(&diskusage_core::history_root(), &path).map_err(map_error)
    })
    .await
}

/// Scans a folder or drive headlessly and stores a snapshot. Can take minutes on a large drive.
#[tauri::command]
pub async fn diskusage_snapshot(path: String) -> Result<SnapshotInfo, AppErrorPayload> {
    let engine = locate_engine()?;
    run_blocking(move || {
        diskusage_core::take_snapshot(&engine, &diskusage_core::history_root(), &path)
            .map_err(map_error)
    })
    .await
}

/// Folder changes between two snapshots; `all` lists every changed folder, not only significant ones.
#[tauri::command]
pub async fn diskusage_compare(
    baseline: String,
    current: String,
    all: bool,
) -> Result<Vec<ChangeRow>, AppErrorPayload> {
    let engine = locate_engine()?;
    run_blocking(move || {
        diskusage_core::compare_snapshots(&engine, Path::new(&baseline), Path::new(&current), all)
            .map_err(map_error)
    })
    .await
}
