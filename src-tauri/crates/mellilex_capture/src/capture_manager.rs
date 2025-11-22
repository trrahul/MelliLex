use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(windows)]
use crate::strategy::uia::UiaCaptureStrategy;
#[cfg(windows)]
use crate::strategy::ocr::OcrCaptureStrategy;
use crate::util::telemetry::{TelemetryHandle, TelemetryOutcome};
use crate::{CaptureAttempt, CaptureError, CaptureRequest, CaptureResult, CaptureStrategy};
use tokio::time::timeout;

/// High-level orchestrator that coordinates registered strategies.
pub struct CaptureManager {
    strategies: Vec<Arc<dyn CaptureStrategy>>,
    telemetry: TelemetryHandle,
}

impl CaptureManager {
    pub fn new(strategies: Vec<Arc<dyn CaptureStrategy>>, telemetry: TelemetryHandle) -> Self {
        Self {
            strategies,
            telemetry,
        }
    }

    /// Convenience helper that builds a manager with the crate's default strategies.
    pub fn with_default_strategies() -> Self {
        CaptureManagerBuilder::default()
            .with_default_strategies()
            .build()
    }

    pub fn builder() -> CaptureManagerBuilder {
        CaptureManagerBuilder::default()
    }

    pub fn strategies(&self) -> &[Arc<dyn CaptureStrategy>] {
        &self.strategies
    }

    pub async fn capture(&self, request: &CaptureRequest) -> Result<CaptureResult, CaptureError> {
        if self.strategies.is_empty() {
            return Err(CaptureError::NoStrategiesConfigured);
        }

        let mut last_error: Option<CaptureError> = None;
        let mut had_no_data = false;

        let policy = CapturePolicy::from(request);
        let strategies = StrategyPrioritizer::order(&self.strategies, &policy);
        let telemetry = CaptureTelemetry::new(&self.telemetry);

        log::info!(
            "[CaptureManager] Starting capture with {} strategies, prefer_ocr={}",
            strategies.len(),
            policy.prefer_ocr
        );
        for (i, s) in strategies.iter().enumerate() {
            log::info!("[CaptureManager] Strategy {}: {} (ocr={})", i, s.name(), s.provides_ocr());
        }

        for strategy in strategies {
            if !strategy.is_supported() {
                log::info!("[CaptureManager] Skipping {} (not supported)", strategy.name());
                continue;
            }

            log::info!("[CaptureManager] Trying strategy: {}", strategy.name());
            let start = Instant::now();
            let capture_future = strategy.capture(request);
            match timeout(policy.timeout, capture_future).await {
                Ok(Ok(CaptureAttempt::Success(result))) => {
                    telemetry.success(strategy.name(), start.elapsed(), &result);
                    return Ok(result);
                }
                Ok(Ok(CaptureAttempt::NoData)) => {
                    telemetry.no_data(strategy.name(), start.elapsed());
                    had_no_data = true;
                    continue;
                }
                Ok(Err(err)) => {
                    telemetry.error(strategy.name(), start.elapsed(), &err.to_string());
                    last_error = Some(err);
                }
                Err(_) => {
                    telemetry.timeout(strategy.name(), start.elapsed(), policy.timeout);
                    last_error = Some(CaptureError::Timeout);
                }
            }
        }

        if let Some(err) = last_error {
            if had_no_data {
                Err(CaptureError::NoCaptureData)
            } else {
                Err(err)
            }
        } else {
            Err(CaptureError::NoCaptureData)
        }
    }
}

#[derive(Default)]
pub struct CaptureManagerBuilder {
    strategies: Vec<Arc<dyn CaptureStrategy>>,
    telemetry: TelemetryHandle,
}

impl CaptureManagerBuilder {
    pub fn with_strategy(mut self, strategy: Arc<dyn CaptureStrategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn with_default_strategies(self) -> Self {
        #[cfg(windows)]
        {
            use crate::strategy::clipboard::ClipboardCaptureStrategy;

            let builder = self.with_strategy(Arc::new(UiaCaptureStrategy::new()));
            builder.with_strategy(Arc::new(ClipboardCaptureStrategy::new()))
        }
        #[cfg(not(windows))]
        {
            self
        }
    }

    #[cfg(windows)]
    pub fn with_ocr_strategy(self) -> Self {
        self.with_strategy(Arc::new(OcrCaptureStrategy::new()))
    }

    pub fn with_telemetry_sink(mut self, telemetry: TelemetryHandle) -> Self {
        self.telemetry = telemetry;
        self
    }

    pub fn build(self) -> CaptureManager {
        CaptureManager::new(self.strategies, self.telemetry)
    }
}

struct CapturePolicy {
    timeout: Duration,
    prefer_ocr: bool,
}

impl From<&CaptureRequest> for CapturePolicy {
    fn from(request: &CaptureRequest) -> Self {
        Self {
            timeout: request.timeout,
            prefer_ocr: request.prefer_ocr,
        }
    }
}

struct StrategyPrioritizer;

impl StrategyPrioritizer {
    fn order(
        strategies: &[Arc<dyn CaptureStrategy>],
        policy: &CapturePolicy,
    ) -> Vec<Arc<dyn CaptureStrategy>> {
        if !policy.prefer_ocr {
            return strategies.to_vec();
        }

        let mut ocr = Vec::new();
        let mut others = Vec::new();

        for strategy in strategies {
            if strategy.provides_ocr() {
                ocr.push(strategy.clone());
            } else {
                others.push(strategy.clone());
            }
        }

        ocr.extend(others);
        ocr
    }
}

struct CaptureTelemetry<'a> {
    handle: &'a TelemetryHandle,
}

impl<'a> CaptureTelemetry<'a> {
    fn new(handle: &'a TelemetryHandle) -> Self {
        Self { handle }
    }

    fn success(&self, strategy: &'static str, duration: Duration, result: &CaptureResult) {
        log::info!("[CaptureManager] Strategy '{}' succeeded", strategy);
        self.emit(
            strategy,
            TelemetryOutcome::Success,
            duration,
            json!({ "source": format!("{:?}", result.source) }),
        );
    }

    fn no_data(&self, strategy: &'static str, duration: Duration) {
        log::info!("[CaptureManager] Strategy '{}' returned no data", strategy);
        self.emit(strategy, TelemetryOutcome::NoData, duration, json!(null));
    }

    fn error(&self, strategy: &'static str, duration: Duration, message: &str) {
        log::warn!("[CaptureManager] Strategy '{}' failed: {}", strategy, message);
        self.emit(
            strategy,
            TelemetryOutcome::Error,
            duration,
            json!({ "error": message }),
        );
    }

    fn timeout(&self, strategy: &'static str, duration: Duration, limit: Duration) {
        let message = format!("timeout after {:?}", limit);
        self.error(strategy, duration, &message);
    }

    fn emit(
        &self,
        strategy: &'static str,
        outcome: TelemetryOutcome,
        duration: Duration,
        metadata: serde_json::Value,
    ) {
        self.handle.emit(crate::util::telemetry::TelemetryEvent {
            strategy,
            outcome,
            duration,
            metadata,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CaptureSource;
    use async_trait::async_trait;

    struct TestStrategy {
        name: &'static str,
        supported: bool,
        supports_ocr: bool,
        behavior: Box<dyn Fn() -> Result<CaptureAttempt, CaptureError> + Send + Sync>,
    }

    #[async_trait]
    impl CaptureStrategy for TestStrategy {
        fn name(&self) -> &'static str {
            self.name
        }

        fn is_supported(&self) -> bool {
            self.supported
        }

        async fn capture(&self, _request: &CaptureRequest) -> Result<CaptureAttempt, CaptureError> {
            (self.behavior)()
        }

        fn provides_ocr(&self) -> bool {
            self.supports_ocr
        }
    }

    fn success_strategy(name: &'static str, text: &'static str) -> Arc<dyn CaptureStrategy> {
        Arc::new(TestStrategy {
            name,
            supported: true,
            supports_ocr: false,
            behavior: Box::new(move || {
                let result = CaptureResult::new(text, CaptureSource::Unknown);
                Ok(CaptureAttempt::Success(result))
            }),
        })
    }

    fn no_data_strategy(name: &'static str) -> Arc<dyn CaptureStrategy> {
        Arc::new(TestStrategy {
            name,
            supported: true,
            supports_ocr: false,
            behavior: Box::new(|| Ok(CaptureAttempt::NoData)),
        })
    }

    fn failing_strategy(name: &'static str, message: &'static str) -> Arc<dyn CaptureStrategy> {
        Arc::new(TestStrategy {
            name,
            supported: true,
            supports_ocr: false,
            behavior: Box::new(move || Err(CaptureError::internal(message))),
        })
    }

    fn ocr_success_strategy(name: &'static str, text: &'static str) -> Arc<dyn CaptureStrategy> {
        Arc::new(TestStrategy {
            name,
            supported: true,
            supports_ocr: true,
            behavior: Box::new(move || {
                let result = CaptureResult::new(text, CaptureSource::Unknown);
                Ok(CaptureAttempt::Success(result))
            }),
        })
    }

    #[tokio::test]
    async fn returns_first_successful_strategy() {
        let manager = CaptureManager::builder()
            .with_strategy(no_data_strategy("no-data"))
            .with_strategy(success_strategy("success", "word"))
            .with_strategy(success_strategy("never-run", "other"))
            .build();

        let request = CaptureRequest::default();
        let result = manager.capture(&request).await.unwrap();
        assert_eq!(result.text, "word");
    }

    #[tokio::test]
    async fn returns_error_when_all_fail() {
        let manager = CaptureManager::builder()
            .with_strategy(failing_strategy("broken", "fail"))
            .build();

        let request = CaptureRequest::default();
        let err = manager.capture(&request).await.unwrap_err();
        assert!(matches!(err, CaptureError::Internal { .. }));
    }

    #[tokio::test]
    async fn reports_no_data_when_none_available() {
        let manager = CaptureManager::builder()
            .with_strategy(no_data_strategy("a"))
            .with_strategy(no_data_strategy("b"))
            .build();

        let request = CaptureRequest::default();
        let err = manager.capture(&request).await.unwrap_err();
        assert!(matches!(err, CaptureError::NoCaptureData));
    }

    #[test]
    fn default_strategies_builder_does_not_panic() {
        let _manager = CaptureManager::with_default_strategies();
    }

    #[tokio::test]
    async fn prioritizes_ocr_when_requested() {
        let manager = CaptureManager::builder()
            .with_strategy(success_strategy("text", "non-ocr"))
            .with_strategy(ocr_success_strategy("ocr", "ocr-word"))
            .build();

        let mut request = CaptureRequest::default();
        request.prefer_ocr = true;

        let result = manager.capture(&request).await.unwrap();
        assert_eq!(result.text, "ocr-word");
    }
}
