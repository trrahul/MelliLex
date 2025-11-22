//! Shortcut management for global lookup.
//!
//! Handles registration and unregistration of global shortcuts
//! and Windows mouse hook for Ctrl+Right-click.

use once_cell::sync::Lazy;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use super::capture_queue::enqueue_capture;

#[cfg(windows)]
use crate::services::mouse_hook::{start_ctrl_right_click_hook, CtrlRightClickHook};

#[cfg(windows)]
static CTRL_RIGHT_HOOK: Lazy<Mutex<Option<CtrlRightClickHook>>> = Lazy::new(|| Mutex::new(None));

/// Registers a global shortcut for word capture.
/// If the shortcut is already registered, it will be re-registered.
pub fn register(app: AppHandle, shortcut: &str) -> tauri::Result<()> {
    log::info!("[GlobalLookup] Registering shortcut: {}", shortcut);

    // Unregister first to avoid "already registered" errors (idempotent)
    let _ = app.global_shortcut().unregister(shortcut);

    app.global_shortcut()
        .on_shortcut(shortcut, move |app_handle, _, _| {
            enqueue_capture(app_handle.clone());
        })
        .map_err(|e| {
            log::error!("[GlobalLookup] Failed to register shortcut: {}", e);
            tauri::Error::PluginInitialization("tauri-plugin-global-shortcut".into(), e.to_string())
        })?;

    log::info!("[GlobalLookup] Shortcut registered successfully");
    Ok(())
}

/// Unregisters a global shortcut and stops the mouse hook.
pub fn unregister(app: AppHandle, shortcut: &str) -> tauri::Result<()> {
    log::info!("[GlobalLookup] Unregistering shortcut: {}", shortcut);

    app.global_shortcut().unregister(shortcut).map_err(|e| {
        log::error!("[GlobalLookup] Failed to unregister shortcut: {}", e);
        tauri::Error::PluginInitialization("tauri-plugin-global-shortcut".into(), e.to_string())
    })?;

    log::info!("[GlobalLookup] Shortcut unregistered successfully");

    #[cfg(windows)]
    {
        CTRL_RIGHT_HOOK
            .lock()
            .expect("mouse hook mutex poisoned")
            .take();
    }

    Ok(())
}

/// Starts the Windows Ctrl+Right-click mouse hook.
#[cfg(windows)]
pub fn start_mouse_hook(app: AppHandle) -> tauri::Result<()> {
    let app_handle = app.clone();
    let hook = start_ctrl_right_click_hook(move || {
        enqueue_capture(app_handle.clone());
    })
    .map_err(|e| tauri::Error::PluginInitialization("mouse-hook".into(), e.to_string()))?;
    
    *CTRL_RIGHT_HOOK.lock().expect("mouse hook mutex poisoned") = Some(hook);
    Ok(())
}

/// No-op on non-Windows platforms.
#[cfg(not(windows))]
pub fn start_mouse_hook(_app: AppHandle) -> tauri::Result<()> {
    Ok(())
}
