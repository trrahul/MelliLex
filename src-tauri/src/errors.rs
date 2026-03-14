use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("AI provider error: {0}")]
    AiProvider(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider '{0}' is not configured")]
    ProviderNotConfigured(String),

    #[error("Provider '{0}' is not supported")]
    ProviderNotSupported(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Cache miss for: {0}")]
    CacheMiss(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Event emission failed: {0}")]
    EventEmission(String),

    #[error("Secret storage error: {0}")]
    SecretStorage(String),

    #[error("Lock acquisition failed: {0}")]
    LockPoisoned(String),

    #[error("File system error: {0}")]
    FileSystem(String),
}

// Conversions from external error types

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound("Query returned no rows".into())
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::AiProvider(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Parse(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileSystem(err.to_string())
    }
}

// Conversion to Tauri's InvokeError for command handlers
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

// Helper methods for common error scenarios
impl AppError {
    pub fn provider_not_configured(provider: impl Into<String>) -> Self {
        AppError::ProviderNotConfigured(provider.into())
    }

    pub fn provider_not_supported(provider: impl Into<String>) -> Self {
        AppError::ProviderNotSupported(provider.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        AppError::Validation(message.into())
    }

    pub fn not_found(item: impl Into<String>) -> Self {
        AppError::NotFound(item.into())
    }

    pub fn secret(message: impl Into<String>) -> Self {
        AppError::SecretStorage(message.into())
    }

    pub fn lock_poisoned(resource: impl Into<String>) -> Self {
        AppError::LockPoisoned(resource.into())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let err = AppError::ProviderNotConfigured("openai".into());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("ProviderNotConfigured"));
        assert!(json.contains("openai"));
    }

    #[test]
    fn test_helper_methods() {
        let err = AppError::provider_not_configured("anthropic");
        assert_eq!(err.to_string(), "Provider 'anthropic' is not configured");
    }

    #[test]
    fn test_from_rusqlite() {
        let db_err = rusqlite::Error::QueryReturnedNoRows;
        let app_err: AppError = db_err.into();
        assert!(matches!(app_err, AppError::NotFound(_)));
    }
}
