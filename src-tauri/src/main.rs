#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocker;
mod state;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

/// Check if the current process has admin privileges on Windows.
/// `net session` exits 0 only when the caller is an Administrator.
#[cfg(target_os = "windows")]
fn is_elevated() -> bool {
    std::process::Command::new("net")
        .args(["session"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Re-launch the current executable with UAC elevation via PowerShell, then exit.
#[cfg(target_os = "windows")]
fn relaunch_as_admin() {
    let exe = std::env::current_exe().expect("Cannot get exe path");
    let exe_str = exe.to_string_lossy();
    // PowerShell Start-Process with -Verb RunAs triggers the UAC prompt.
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-WindowStyle", "Hidden",
            "-Command",
            &format!("Start-Process -FilePath '{}' -Verb RunAs", exe_str),
        ])
        .spawn();
    std::process::exit(0);
}

fn main() {
    // On Windows: auto-elevate if not already running as admin.
    #[cfg(target_os = "windows")]
    if !is_elevated() {
        relaunch_as_admin();
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            blocker::get_blocked_sites,
            blocker::add_site,
            blocker::remove_site,
            blocker::toggle_blocking,
            blocker::get_blocking_status,
            blocker::apply_preset,
            blocker::get_presets,
            blocker::get_schedule,
            blocker::save_schedule,
            check_for_update,
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.show().unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Called from the frontend "Check for Updates" button.
/// Returns: { available: bool, version: String, notes: String }
#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app
        .updater()
        .map_err(|e| format!("Updater init failed: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.body.clone().unwrap_or_default();
            // Download & install in background then ask to restart
            update
                .download_and_install(|_, _| {}, || {})
                .await
                .map_err(|e| format!("Install failed: {}", e))?;
            Ok(serde_json::json!({
                "available": true,
                "version": version,
                "notes": notes,
                "installed": true
            }))
        }
        Ok(None) => Ok(serde_json::json!({ "available": false })),
        Err(e) => Err(format!("Update check failed: {}", e)),
    }
}
