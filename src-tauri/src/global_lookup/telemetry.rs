//! Telemetry adapter for global lookup capture events.
//!
//! Bridges the mellilex_capture telemetry system to Tauri events,
//! allowing the frontend to receive capture metrics.

use serde_json::json;
use tauri::{AppHandle, Emitter};

use mellilex_capture::{TelemetryEvent, TelemetrySink};

pub const GLOBAL_LOOKUP_TELEMETRY_EVENT: &str = "global-lookup-capture-telemetry";

/// Telemetry sink that emits capture events to the Tauri frontend.
pub struct GlobalLookupTelemetrySink {
    app_handle: AppHandle,
}

impl GlobalLookupTelemetrySink {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl TelemetrySink for GlobalLookupTelemetrySink {
    fn emit(&self, event: TelemetryEvent<'_>) {
        let payload = json!({
            "strategy": event.strategy,
            "outcome": event.outcome.to_string(),
            "duration_ms": event.duration.as_millis(),
            "metadata": event.metadata,
        });

        if let Err(err) = self.app_handle.emit(GLOBAL_LOOKUP_TELEMETRY_EVENT, payload) {
            tracing::warn!("[GlobalLookup] Failed to emit telemetry event: {}", err);
        }
    }
}
