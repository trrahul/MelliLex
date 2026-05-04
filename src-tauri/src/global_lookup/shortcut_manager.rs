//! Mouse hook management for global lookup.
//!
//! Handles registration of the Windows Ctrl+Right-click mouse hook
//! for triggering word capture from any application.

use once_cell::sync::Lazy;
use std::sync::Mutex;
use tauri::AppHandle;

use super::capture_queue::enqueue_capture;

#[cfg(windows)]
use crate::services::mouse_hook::{start_ctrl_right_click_hook, CtrlRightClickHook};

#[cfg(windows)]
static CTRL_RIGHT_HOOK: Lazy<Mutex<Option<CtrlRightClickHook>>> = Lazy::new(|| Mutex::new(None));

/// Starts the Windows Ctrl+Right-click mouse hook.
#[cfg(windows)]
pub fn start_mouse_hook(app: AppHandle) -> tauri::Result<()> {
    log::info!("[GlobalLookup] Starting Ctrl+Right-click mouse hook");
    let app_handle = app.clone();
    let hook = start_ctrl_right_click_hook(move || {
        enqueue_capture(app_handle.clone());
    })
    .map_err(|e| tauri::Error::PluginInitialization("mouse-hook".into(), e.to_string()))?;
    
    *CTRL_RIGHT_HOOK.lock().expect("mouse hook mutex poisoned") = Some(hook);
    log::info!("[GlobalLookup] Mouse hook started successfully");
    Ok(())
}

/// Stops the Windows Ctrl+Right-click mouse hook.
#[cfg(windows)]
pub fn stop_mouse_hook(_app: AppHandle) -> tauri::Result<()> {
    log::info!("[GlobalLookup] Stopping mouse hook");
    CTRL_RIGHT_HOOK
        .lock()
        .expect("mouse hook mutex poisoned")
        .take();
    Ok(())
}

/// No-op on non-Windows platforms.
#[cfg(not(windows))]
pub fn start_mouse_hook(_app: AppHandle) -> tauri::Result<()> {
    Ok(())
}

/// No-op on non-Windows platforms.
#[cfg(not(windows))]
pub fn stop_mouse_hook(_app: AppHandle) -> tauri::Result<()> {
    Ok(())
}
