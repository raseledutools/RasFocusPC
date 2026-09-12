#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocker;
mod state;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
