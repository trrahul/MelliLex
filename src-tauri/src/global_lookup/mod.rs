//! Global lookup module - coordinates word capture from any application.
//!
//! This module provides global shortcut and mouse hook integration for
//! triggering word capture from any Windows application.
//!
//! ## Architecture
//!
//! - `shortcut_manager`: Handles keyboard shortcut and mouse hook registration
//! - `capture_queue`: Manages capture request queuing and deduplication
//! - `telemetry`: Bridges capture metrics to the Tauri frontend
//! - `windows_monitor`: Windows-specific environment logging for debugging

mod capture_queue;
mod shortcut_manager;
mod telemetry;

#[cfg(windows)]
mod windows_monitor;

use tauri::AppHandle;
use crate::db::Database;

/// Initializes the global lookup system.
///
/// Checks saved settings to determine if global lookup is enabled.
/// If enabled, registers the configured shortcut and starts the mouse hook.
pub fn init(app: AppHandle, db: &Database) -> tauri::Result<()> {
    log::debug!("[GlobalLookup] Initializing capture pipeline (UI Automation -> Clipboard)");
    
    let (enabled, shortcut) = match db.get_settings() {
        Ok(settings) => (
            settings.enable_global_lookup,
            if settings.global_lookup_shortcut.is_empty() {
                "CTRL+ALT+D".to_string()
            } else {
                settings.global_lookup_shortcut
            },
        ),
        Err(e) => {
            log::warn!("[GlobalLookup] Failed to load settings, using defaults: {}", e);
            (true, "CTRL+ALT+D".to_string())
        }
    };

    if !enabled {
        log::info!("[GlobalLookup] Global lookup is disabled in settings, skipping registration");
        return Ok(());
    }

    shortcut_manager::register(app.clone(), &shortcut)?;
    shortcut_manager::start_mouse_hook(app)?;
    
    Ok(())
}

/// Unregisters a global shortcut.
pub fn unregister_shortcut(app: AppHandle, shortcut: &str) -> tauri::Result<()> {
    shortcut_manager::unregister(app, shortcut)
}

/// Registers a global shortcut.
pub fn register_shortcut(app: AppHandle, shortcut: &str) -> tauri::Result<()> {
    shortcut_manager::register(app, shortcut)
}
