use crate::models::TokenUsage;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiModel {
    pub id: String,
    pub name: String,
}

#[async_trait]
pub trait PromptSender: Send + Sync {
    async fn send_prompt(&self, prompt: &str) -> Result<(String, Option<TokenUsage>)>;
}
