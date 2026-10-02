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
        .setup(|app| {
            if supports_mica() {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_effects(EffectsBuilder::new().effect(Effect::Mica).build())?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("error while running DeepServer");
}
