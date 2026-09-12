#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocker;
mod state;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

/// Check if the current process has admin privileges on Windows.
#[cfg(target_os = "windows")]
fn is_elevated() -> bool {
    use std::mem;
    use std::ptr;
    // Use the Windows API via a raw call to check token elevation.
    unsafe {
        let mut token: *mut std::ffi::c_void = ptr::null_mut();
        // OpenProcessToken
        let process = windows_sys::Win32::System::Threading::GetCurrentProcess();
        if windows_sys::Win32::Security::OpenProcessToken(
            process,
            windows_sys::Win32::Security::TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return false;
        }
        let mut elevation = windows_sys::Win32::Security::TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size: u32 = mem::size_of::<windows_sys::Win32::Security::TOKEN_ELEVATION>() as u32;
        let result = windows_sys::Win32::Security::GetTokenInformation(
            token,
            windows_sys::Win32::Security::TokenElevation,
            &mut elevation as *mut _ as *mut std::ffi::c_void,
            size,
            &mut size,
        );
        windows_sys::Win32::Foundation::CloseHandle(token);
        result != 0 && elevation.TokenIsElevated != 0
    }
}

/// Re-launch the current executable with "runas" (UAC prompt) then exit.
#[cfg(target_os = "windows")]
fn relaunch_as_admin() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let exe = std::env::current_exe().expect("Cannot get exe path");
    let exe_wide: Vec<u16> = OsStr::new(exe.to_str().unwrap())
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let operation: Vec<u16> = OsStr::new("runas\0").encode_wide().collect();
    unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            0,
            operation.as_ptr(),
            exe_wide.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
        );
    }
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
