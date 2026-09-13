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

/// Open a new browser window pointing at the given URL.
#[tauri::command]
fn open_browser_window(app: tauri::AppHandle, url: String) -> Result<(), String> {
    // Validate URL
    let safe_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.clone()
    } else {
        format!("https://{}", url)
    };

    // Use a unique label based on timestamp to allow multiple windows
    let label = format!("browser_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());

    tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::External(safe_url.parse().map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title("RasFocus Browser")
    .inner_size(1200.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| format!("Failed to open browser window: {}", e))?;

    Ok(())
}

fn main() {
    #[cfg(target_os = "windows")]
    if !is_elevated() {
        relaunch_as_admin();
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            open_browser_window,
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.show().unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
