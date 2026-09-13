#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocker;
mod state;

use state::AppState;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

// ─── Admin check ─────────────────────────────────────────────────────────────

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

#[cfg(target_os = "windows")]
fn relaunch_as_admin() {
    let exe = std::env::current_exe().expect("Cannot get exe path");
    let exe_str = exe.to_string_lossy();
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &format!("Start-Process -FilePath '{}' -Verb RunAs", exe_str),
        ])
        .spawn();
    std::process::exit(0);
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Open a URL in the system default browser.
#[tauri::command]
fn open_browser_window(url: String) -> Result<(), String> {
    let safe_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.clone()
    } else {
        format!("https://{}", url)
    };

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &safe_url])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    std::process::Command::new("xdg-open")
        .arg(&safe_url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    Ok(())
}

/// Check GitHub releases for a newer version.
/// Returns { available, version?, download_url? }
#[tauri::command]
async fn check_for_update() -> Result<serde_json::Value, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::builder()
        .user_agent("RasFocusPC-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/raseledutools/RasFocusPC/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if resp.status() == 404 {
        return Ok(serde_json::json!({ "available": false }));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let latest_tag = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();

    if latest_tag.is_empty() || latest_tag == current_version {
        return Ok(serde_json::json!({ "available": false }));
    }

    let download_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|a| {
                a["name"]
                    .as_str()
                    .map(|n| n.ends_with(".exe"))
                    .unwrap_or(false)
            })
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .unwrap_or("")
        .to_string();

    Ok(serde_json::json!({
        "available": true,
        "version": latest_tag,
        "download_url": download_url
    }))
}

/// Download the new installer to %TEMP% and run it, then quit this app.
/// The installer (NSIS) will replace the running app automatically.
#[tauri::command]
async fn download_and_install_update(
    download_url: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use std::io::Write;

    let client = reqwest::Client::builder()
        .user_agent("RasFocusPC-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Download error: HTTP {}", resp.status()));
    }

    // Save to %TEMP%\RasFocusPC-update.exe
    let tmp_path = std::env::temp_dir().join("RasFocusPC-update.exe");
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Read failed: {}", e))?;

    {
        let mut file =
            std::fs::File::create(&tmp_path).map_err(|e| format!("Cannot create file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Write failed: {}", e))?;
    }

    // Launch installer silently (NSIS /S flag) then exit
    // The installer will close the running app via the uninstaller and replace it
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args([
                "/C",
                "start",
                "",
                tmp_path.to_str().unwrap_or(""),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Cannot launch installer: {}", e))?;
    }

    // Give installer a moment to start, then quit
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    app.exit(0);

    Ok(())
}

/// Show the main window (called from tray menu or frontend).
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Quit the app completely (called from tray menu or frontend).
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    #[cfg(target_os = "windows")]
    if !is_elevated() {
        relaunch_as_admin();
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_tray::init())
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
            check_for_update,
            download_and_install_update,
            show_window,
            quit_app,
        ])
        .setup(|app| {
            // ── Tray icon & menu ──────────────────────────────────────────
            let show_item = MenuItem::with_id(app, "show", "Show RasFocus PC", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("RasFocus PC")
                .menu(&menu)
                .menu_on_left_click(false)   // left-click = show window
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Left-click on tray icon → show window
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Hide to tray on close (don't destroy window) ──────────────
            let win = app.get_webview_window("main").unwrap();
            win.on_window_event(|event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    // The window handle is not directly accessible here via closure
                    // We handle hide via the prevent_close; the window just hides.
                    // We call hide via a separate listener below.
                }
            });

            // Separate listener that actually hides the window
            let win2 = app.get_webview_window("main").unwrap();
            let win2_clone = win2.clone();
            win2.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    let _ = win2_clone.hide();
                }
            });

            win.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
