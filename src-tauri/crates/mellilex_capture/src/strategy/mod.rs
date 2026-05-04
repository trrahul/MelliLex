use async_trait::async_trait;

use crate::{CaptureError, CaptureRequest, CaptureResult};

/// Outcome produced by a strategy invocation.
#[derive(Debug)]
pub enum CaptureAttempt {
    /// Strategy successfully produced a capture.
    Success(CaptureResult),
    /// Strategy was applicable but no text was available at the requested point.
    NoData,
}

/// Contract implemented by every capture strategy.
#[async_trait]
pub trait CaptureStrategy: Send + Sync {
    fn name(&self) -> &'static str;

    fn is_supported(&self) -> bool {
        true
    }

    async fn capture(&self, request: &CaptureRequest) -> Result<CaptureAttempt, CaptureError>;

    fn provides_ocr(&self) -> bool {
        false
    }
}

#[cfg(windows)]
pub mod ocr;
#[cfg(windows)]
pub mod uia;
