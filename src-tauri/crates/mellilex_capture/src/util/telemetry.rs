use serde_json::Value;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum TelemetryOutcome {
    Success,
    NoData,
    Error,
}

impl fmt::Display for TelemetryOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryOutcome::Success => write!(f, "success"),
            TelemetryOutcome::NoData => write!(f, "no_data"),
            TelemetryOutcome::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug)]
pub struct TelemetryEvent<'a> {
    pub strategy: &'a str,
    pub outcome: TelemetryOutcome,
    pub duration: Duration,
    pub metadata: Value,
}

pub trait TelemetrySink: Send + Sync {
    fn emit(&self, event: TelemetryEvent<'_>);
}

#[derive(Clone)]
pub struct TelemetryHandle(pub Arc<dyn TelemetrySink>);

impl TelemetryHandle {
    pub fn emit(&self, event: TelemetryEvent<'_>) {
        self.0.emit(event);
    }
}

impl Default for TelemetryHandle {
    fn default() -> Self {
        Self(Arc::new(TracingTelemetrySink))
    }
}

struct TracingTelemetrySink;

impl TelemetrySink for TracingTelemetrySink {
    fn emit(&self, event: TelemetryEvent<'_>) {
        tracing::info!(
            target: "mellilex_capture::telemetry",
            strategy = event.strategy,
            outcome = %event.outcome,
            duration_ms = event.duration.as_millis(),
            metadata = %event.metadata,
            "capture telemetry event"
        );
    }
}
