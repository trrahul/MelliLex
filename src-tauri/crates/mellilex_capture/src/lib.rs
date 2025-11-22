//! Public API surface for the MelliLex capture crate.
//!
//! This crate intentionally exposes only the abstractions required by the
//! backend so new strategies/platforms can be added without touching the Tauri
//! layer.

pub mod capture_manager;
pub mod context;
pub mod error;
pub mod strategy;
pub mod util;

pub use capture_manager::{CaptureManager, CaptureManagerBuilder};
pub use context::{CaptureRequest, CaptureResult, CaptureSource, ScreenPoint, ScreenRect};
pub use error::CaptureError;
#[cfg(windows)]
pub use strategy::ocr::OcrCaptureStrategy;
#[cfg(windows)]
pub use strategy::uia::UiaCaptureStrategy;
pub use strategy::{CaptureAttempt, CaptureStrategy};
pub use util::telemetry::{TelemetryEvent, TelemetryHandle, TelemetryOutcome, TelemetrySink};
