use mellilex_capture::{
    CaptureError, CaptureManager, CaptureManagerBuilder, CaptureRequest, TelemetryHandle,
};

pub struct MellilexCaptureAdapter {
    manager: CaptureManager,
}

impl MellilexCaptureAdapter {
    pub fn with_telemetry(telemetry: TelemetryHandle) -> Self {
        let manager = CaptureManagerBuilder::default()
            .with_default_strategies()
            .with_telemetry_sink(telemetry)
            .build();
        Self { manager }
    }

    pub fn with_telemetry_and_ocr(telemetry: TelemetryHandle) -> Self {
        #[cfg(windows)]
        let manager = CaptureManagerBuilder::default()
            .with_default_strategies()
            .with_ocr_strategy()
            .with_telemetry_sink(telemetry)
            .build();

        #[cfg(not(windows))]
        let manager = CaptureManagerBuilder::default()
            .with_default_strategies()
            .with_telemetry_sink(telemetry)
            .build();

        Self { manager }
    }

    pub async fn capture_with_request(
        &self,
        request: CaptureRequest,
    ) -> Result<Option<String>, CaptureError> {
        match self.manager.capture(&request).await {
            Ok(result) => Ok(Some(result.text)),
            Err(CaptureError::NoCaptureData) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
