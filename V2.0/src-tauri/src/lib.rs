mod compare;
mod record;

use serde::Serialize;
use tauri::window::{Effect, EffectsBuilder};
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    machine: String,
    /// True when the window got the Windows 11 Mica backdrop; the UI then drops its opaque
    /// background so the backdrop shows through.
    mica: bool,
}

/// Mica exists from Windows 11 (build 22000). Windows 10 and Server 2019 keep the solid
/// background, which matters because the window is created transparent.
fn supports_mica() -> bool {
    #[cfg(windows)]
    {
        windows_version::OsVersion::current().build >= 22000
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[tauri::command]
fn app_info(app: tauri::AppHandle) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        machine: std::env::var("COMPUTERNAME").unwrap_or_default(),
        mica: supports_mica(),
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(compare::CompareState::default())
        .manage(record::RecordState::default())
        .setup(|app| {
            record::prune_in_background();
            if supports_mica() {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_effects(EffectsBuilder::new().effect(Effect::Mica).build())?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            compare::compare_start,
            compare::compare_children,
            compare::compare_cancel,
            compare::compare_path,
            compare::compare_reveal,
            compare::compare_missing,
            record::record_start,
            record::record_retry,
            record::record_finish,
            record::record_cancel,
            record::record_recover,
            record::record_load,
            record::record_list,
            record::record_reveal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DeepServer");
}
