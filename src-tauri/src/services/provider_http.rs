use anyhow::{anyhow, Result};
use log::error;
use reqwest::{Response, StatusCode};

pub async fn ensure_success(label: &str, response: Response) -> Result<Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    error!("{} API error: status {}, body: {}", label, status, body);
    Err(anyhow!("{} API error: {}", label, status))
}

pub fn log_status_if_error(label: &str, status: StatusCode) {
    if !status.is_success() {
        error!("{} API error: status {}", label, status);
    }
}
