//! Global lookup module - coordinates word capture from any application.
//!
//! This module provides mouse hook integration for
//! triggering word capture from any Windows application.
//!
//! ## Architecture
//!
//! - `shortcut_manager`: Handles mouse hook registration
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
/// If enabled, starts the Ctrl+Right-click mouse hook.
pub fn init(app: AppHandle, db: &Database) -> tauri::Result<()> {
    log::debug!("[GlobalLookup] Initializing capture pipeline (UI Automation + OCR)");
    
    let enabled = match db.get_settings() {
        Ok(settings) => settings.enable_global_lookup,
        Err(e) => {
            log::warn!("[GlobalLookup] Failed to load settings, using defaults: {}", e);
            true
        }
    };

    if !enabled {
        log::info!("[GlobalLookup] Global lookup is disabled in settings, skipping registration");
        return Ok(());
    }

    shortcut_manager::start_mouse_hook(app)?;
    
    Ok(())
}

/// Stops the mouse hook.
pub fn disable(app: AppHandle) -> tauri::Result<()> {
    shortcut_manager::stop_mouse_hook(app)
}

/// Starts the mouse hook.
pub fn enable(app: AppHandle) -> tauri::Result<()> {
    shortcut_manager::start_mouse_hook(app)
}
