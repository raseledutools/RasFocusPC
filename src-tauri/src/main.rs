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
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.show().unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
