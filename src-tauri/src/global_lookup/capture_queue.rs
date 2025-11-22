//! Capture queue management for global lookup.
//!
//! Handles queuing and deduplication of capture requests to prevent
//! multiple concurrent captures when shortcuts are triggered rapidly.

use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::async_runtime;
use tauri::{AppHandle, Emitter};

use crate::services::mellilex_capture_adapter::MellilexCaptureAdapter;
use crate::window_controls::show_main_window;
use mellilex_capture::{ScreenPoint, TelemetryHandle};

use super::telemetry::GlobalLookupTelemetrySink;
use mellilex_capture::CaptureRequest;
#[cfg(windows)]
use super::windows_monitor;

pub const GLOBAL_LOOKUP_EVENT: &str = "global-lookup-triggered";

static LOOKUP_RUNTIME: Lazy<async_runtime::RuntimeHandle> = Lazy::new(async_runtime::handle);
static CAPTURE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static PENDING_TRIGGER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Get the current cursor position.
#[cfg(windows)]
fn get_cursor_position() -> Option<ScreenPoint> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    
    unsafe {
        let mut point = POINT::default();
        if GetCursorPos(&mut point).is_ok() {
            Some(ScreenPoint { x: point.x, y: point.y })
        } else {
            None
        }
    }
}

/// Payload emitted when a global lookup is triggered.
#[derive(Serialize, Clone)]
pub struct GlobalLookupTriggerPayload {
    pub source: &'static str,
    pub word: Option<String>,
}

/// Enqueues a capture request, deduplicating rapid triggers.
///
/// If a capture is already in progress, the request is queued and will be
/// processed after the current capture completes.
pub fn enqueue_capture(app_handle: AppHandle) {
    if CAPTURE_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        spawn_capture_task(app_handle);
    } else {
        let queued = PENDING_TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
        log::debug!(
            "[GlobalLookup] Capture already running; queued additional request (pending={})",
            queued
        );
    }
}

/// Spawns the async capture task.
fn spawn_capture_task(app_handle: AppHandle) {
    LOOKUP_RUNTIME.spawn(async move {
        log::debug!("[GlobalLookup] Shortcut triggered, capturing word");

        #[cfg(windows)]
        windows_monitor::log_environment_snapshot();

        // Capture cursor position and detect Kindle BEFORE showing the window
        #[cfg(windows)]
        let (use_ocr, cursor_pos) = {
            let is_kindle = windows_monitor::is_kindle_under_cursor();
            let cursor = get_cursor_position();
            (is_kindle, cursor)
        };
        #[cfg(not(windows))]
        let (use_ocr, cursor_pos): (bool, Option<mellilex_capture::ScreenPoint>) = (false, None);

        if use_ocr {
            log::info!("[GlobalLookup] Kindle detected under cursor; preferring OCR");
        }
        if let Some(ref pos) = cursor_pos {
            log::info!("[GlobalLookup] Captured cursor position: ({}, {})", pos.x, pos.y);
        }

        if let Err(err) = show_main_window(&app_handle) {
            log::error!(
                "[GlobalLookup] Failed to show window before capture: {}",
                err
            );
        }

        let telemetry_handle = TelemetryHandle(Arc::new(GlobalLookupTelemetrySink::new(
            app_handle.clone(),
        )));

        let capture_adapter = if use_ocr {
            MellilexCaptureAdapter::with_telemetry_and_ocr(telemetry_handle)
        } else {
            MellilexCaptureAdapter::with_telemetry(telemetry_handle)
        };

        let mut request = CaptureRequest::default();
        if use_ocr {
            request.prefer_ocr = true;
        }
        request.cursor = cursor_pos;

        let word = match capture_adapter.capture_with_request(request).await {
            Ok(Some(text)) => {
                log::info!("[GlobalLookup] Successfully captured: '{}'", text);
                Some(text)
            }
            Ok(None) => {
                log::warn!("[GlobalLookup] No word captured (no text available)");
                None
            }
            Err(e) => {
                log::error!("[GlobalLookup] Capture manager failed: {}", e);
                None
            }
        };

        if let Err(err) = app_handle.emit(
            GLOBAL_LOOKUP_EVENT,
            GlobalLookupTriggerPayload {
                source: "global-lookup-shortcut",
                word,
            },
        ) {
            log::error!("[GlobalLookup] Failed to emit trigger event: {}", err);
        }

        CAPTURE_IN_PROGRESS.store(false, Ordering::SeqCst);

        // Process any queued requests using atomic decrement to avoid TOCTOU race
        // Only recurse if we successfully decremented from a positive value
        loop {
            let current = PENDING_TRIGGER_COUNT.load(Ordering::SeqCst);
            if current == 0 {
                break;
            }
            // Try to decrement; if another thread changed it, retry
            if PENDING_TRIGGER_COUNT
                .compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                log::debug!(
                    "[GlobalLookup] Processing queued request (remaining={})",
                    current - 1
                );
                enqueue_capture(app_handle);
                break;
            }
        }
    });
}
