use thiserror::Error;

/// Domain errors returned by the capture engine.
#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("word capture is not supported on this platform")]
    UnsupportedPlatform,
    #[error("word capture timed out before any strategy succeeded")]
    Timeout,
    #[error("no capture strategies were configured")]
    NoStrategiesConfigured,
    #[error("capture strategies executed but yielded no data")]
    NoCaptureData,
    #[error("capture failed: {message}")]
    Internal { message: String },
}

impl CaptureError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}
