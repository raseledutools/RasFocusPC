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

/// Open a URL inside the app itself as an embedded browser window.
/// Uses a real Chrome user-agent so YouTube, Google and other sites load normally.
#[tauri::command]
fn open_in_app_browser(app: tauri::AppHandle, url: String, title: String) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    let safe_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.clone()
    } else {
        format!("https://{}", url)
    };

    let win_label = format!(
        "browser_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_millis()
    );

    let display_title = if title.is_empty() {
        safe_url.clone()
    } else {
        title
    };

    // Chrome 124 user-agent — makes YouTube, Google Sign-In, and other
    // sites treat the WebView2 window as a real browser instead of blocking it.
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
              AppleWebKit/537.36 (KHTML, like Gecko) \
              Chrome/124.0.0.0 Safari/537.36";

    WebviewWindowBuilder::new(
        &app,
        &win_label,
        WebviewUrl::External(safe_url.parse().map_err(|e: url::ParseError| e.to_string())?),
    )
    .title(display_title)
    .inner_size(1280.0, 800.0)
    .min_inner_size(600.0, 400.0)
    .resizable(true)
    .center()
    .user_agent(ua)
    .build()
    .map_err(|e| format!("Failed to open browser window: {}", e))?;

    Ok(())
}

/// Convert colour temperature in Kelvin to (red, green, blue) multipliers 0.0–1.0.
/// Based on Tanner Helland's algorithm.
fn kelvin_to_rgb(k: i32) -> (f64, f64, f64) {
    if k >= 6500 {
        return (1.0, 1.0, 1.0); // reset / Pause mode
    }
    let t = k as f64 / 100.0;

    let r = if t <= 66.0 {
        1.0
    } else {
        (329.698727446 * (t - 60.0).powf(-0.1332047592) / 255.0).clamp(0.0, 1.0)
    };

    let g = if t <= 66.0 {
        (99.4708025861 * t.ln() - 161.1195681661) / 255.0
    } else {
        288.1221695283 * (t - 60.0).powf(-0.0755148492) / 255.0
    };
    let g = g.clamp(0.0, 1.0);

    let b = if t >= 66.0 {
        1.0
    } else if t <= 19.0 {
        0.0
    } else {
        ((138.5177312231 * (t - 10.0).ln() - 305.0447927307) / 255.0).clamp(0.0, 1.0)
    };

    (r, g, b)
}

/// Apply colour-temperature + brightness filter via SetDeviceGammaRamp (PowerShell P/Invoke).
/// temp_k : 1000–6500 K   brightness : 10–100
///
/// Uses a cached .ps1 helper in %TEMP% so Add-Type only compiles once per session.
/// Runs with wait() so the ramp is applied before we return.
#[tauri::command]
fn apply_display_filter(temp_k: i32, brightness: i32) -> Result<(), String> {
    let (rm, gm, bm) = kelvin_to_rgb(temp_k);
    let brt = (brightness as f64 / 100.0).clamp(0.1, 1.0);

    // Build 256-entry ramp values directly in Rust, pass as comma-separated strings.
    // This avoids per-call Add-Type C# compilation in PowerShell (slow + flaky).
    let red_vals: Vec<String> = (0u32..256)
        .map(|i| ((i * 256) as f64 * rm * brt).min(65535.0) as u32)
        .map(|v| v.to_string())
        .collect();
    let grn_vals: Vec<String> = (0u32..256)
        .map(|i| ((i * 256) as f64 * gm * brt).min(65535.0) as u32)
        .map(|v| v.to_string())
        .collect();
    let blu_vals: Vec<String> = (0u32..256)
        .map(|i| ((i * 256) as f64 * bm * brt).min(65535.0) as u32)
        .map(|v| v.to_string())
        .collect();

    let r_str = red_vals.join(",");
    let g_str = grn_vals.join(",");
    let b_str = blu_vals.join(",");

    let script = format!(
        r#"Add-Type -TypeDefinition @'
using System;using System.Runtime.InteropServices;
public class RasGamma{{
[DllImport("gdi32.dll")]public static extern bool SetDeviceGammaRamp(IntPtr h,ref RAMP r);
[DllImport("user32.dll")]public static extern IntPtr GetDC(IntPtr h);
[StructLayout(LayoutKind.Sequential)]
public struct RAMP{{
[MarshalAs(UnmanagedType.ByValArray,SizeConst=256)]public ushort[] Red;
[MarshalAs(UnmanagedType.ByValArray,SizeConst=256)]public ushort[] Green;
[MarshalAs(UnmanagedType.ByValArray,SizeConst=256)]public ushort[] Blue;
}}
}}
'@ -ErrorAction SilentlyContinue
$r=[int[]]@({r})
$g=[int[]]@({g})
$b=[int[]]@({b})
$ramp=New-Object RasGamma+RAMP
$ramp.Red=[System.Array]::ConvertAll($r,[converter[int,ushort]]{{param($x)[ushort]$x}})
$ramp.Green=[System.Array]::ConvertAll($g,[converter[int,ushort]]{{param($x)[ushort]$x}})
$ramp.Blue=[System.Array]::ConvertAll($b,[converter[int,ushort]]{{param($x)[ushort]$x}})
[RasGamma]::SetDeviceGammaRamp([RasGamma]::GetDC([IntPtr]::Zero),[ref]$ramp)"#,
        r = r_str, g = g_str, b = b_str
    );

    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("PowerShell failed: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("SetDeviceGammaRamp failed (exit {:?})", status.code()))
    }
}

/// Check GitHub releases for a newer version.
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

/// Download installer to %TEMP% and launch it, then quit.
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

    let tmp_path = std::env::temp_dir().join("RasFocusPC-update.exe");
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Read failed: {}", e))?;

    {
        let mut file = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("Cannot create file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Write failed: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", "", tmp_path.to_str().unwrap_or(""), "/S"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Cannot launch installer: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    app.exit(0);

    Ok(())
}

/// Show the main window.
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Quit the app completely.
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
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
            open_in_app_browser,
            apply_display_filter,
            check_for_update,
            download_and_install_update,
            show_window,
            quit_app,
        ])
        .setup(|app| {
            // ── Tray menu ────────────────────────────────────────────────
            let show_i = MenuItem::with_id(app, "show", "Show RasFocus PC", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu   = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("RasFocus PC")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Hide to tray on window close ─────────────────────────────
            let win = app.get_webview_window("main").unwrap();
            let win_hide = win.clone();
            win.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_hide.hide();
                }
            });

            win.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
