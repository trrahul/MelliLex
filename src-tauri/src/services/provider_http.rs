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
    let detail = extract_error_message(&body).unwrap_or_else(|| status.to_string());
    Err(anyhow!("{}: {}", label, detail))
}

/// Best-effort extraction of a human-readable message from common provider
/// error payloads (OpenAI / Anthropic / Gemini all use `{"error":{"message":...}}`).
fn extract_error_message(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let err = value.get("error")?;
    if let Some(msg) = err.get("message").and_then(|v| v.as_str()) {
        return Some(msg.to_string());
    }
    err.as_str().map(|s| s.to_string())
}

pub fn log_status_if_error(label: &str, status: StatusCode) {
    if !status.is_success() {
        error!("{} API error: status {}", label, status);
    }
}
