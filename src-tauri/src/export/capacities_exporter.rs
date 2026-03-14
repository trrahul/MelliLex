use crate::errors::AppError;
use serde::Serialize;
use std::time::Duration;

/// Single Responsibility: Export markdown to Capacities app via API
pub struct CapacitiesExporter;

impl CapacitiesExporter {
    pub async fn export(
        api_token: &str,
        space_id: &str,
        markdown: &str,
        no_timestamp: bool,
    ) -> Result<(), AppError> {
        if api_token.trim().is_empty() {
            return Err(AppError::validation("Capacities API token is required"));
        }

        #[derive(Serialize)]
        struct CapacitiesPayload<'a> {
            #[serde(rename = "spaceId")]
            space_id: &'a str,
            #[serde(rename = "mdText")]
            md_text: &'a str,
            origin: &'static str,
            #[serde(rename = "noTimeStamp")]
            no_time_stamp: bool,
        }

        let payload = CapacitiesPayload {
            space_id,
            md_text: markdown,
            origin: "commandPalette",
            no_time_stamp: no_timestamp,
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Network(format!("Failed to build HTTP client: {}", e)))?;

        log::info!("[CapacitiesExporter] Sending request to Capacities API");

        let response = client
            .post("https://api.capacities.io/save-to-daily-note")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_token))
            .json(&payload)
            .send()
            .await
            .map_err(|err| {
                AppError::Network(format!("Failed to connect to Capacities API: {}", err))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            log::error!(
                "[CapacitiesExporter] Capacities API returned error status: {}",
                status
            );

            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            log::error!("[CapacitiesExporter] Error response: {}", error_text);

            return Err(AppError::Network(format!(
                "Capacities API error ({}): {}",
                status, error_text
            )));
        }

        log::info!("[CapacitiesExporter] Successfully exported to Capacities");
        Ok(())
    }
}
