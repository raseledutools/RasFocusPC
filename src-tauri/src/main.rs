#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocker;
mod state;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

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
            "-WindowStyle", "Hidden",
            "-Command",
            &format!("Start-Process -FilePath '{}' -Verb RunAs", exe_str),
        ])
        .spawn();
    std::process::exit(0);
}

/// Open a URL in the system default browser (Edge, Chrome, Firefox, etc.)
#[tauri::command]
fn open_browser_window(url: String) -> Result<(), String> {
    let safe_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.clone()
    } else {
        format!("https://{}", url)
    };

    // Use Windows ShellExecute to open in system default browser
    // This is the most reliable approach - no WebView2 CSP issues
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &safe_url])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("xdg-open")
            .arg(&safe_url)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    Ok(())
}

/// Check GitHub releases for a newer version.
/// Returns: { available: bool, version?: string, download_url?: string }
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

    if latest_tag.is_empty() {
        return Ok(serde_json::json!({ "available": false }));
    }

    if latest_tag == current_version {
        return Ok(serde_json::json!({ "available": false }));
    }

    let download_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|a| {
                a["name"].as_str().map(|n| n.ends_with(".exe")).unwrap_or(false)
            })
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .unwrap_or("")
        .to_string();

    Ok(serde_json::json!({
        "available": true,
        "version": latest_tag,
        "download_url": download_url,
        "installed": false
    }))
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
