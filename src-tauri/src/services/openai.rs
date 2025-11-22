use crate::constants::api::openai as openai_api;
use crate::models::TokenUsage;
use crate::services::ai_provider::{AiModel, PromptSender};
use crate::services::http_client::{RetriableClient, RetryIntent};
use crate::services::model_info::ModelInfoService;
use crate::services::provider_http::{ensure_success, log_status_if_error};
use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use serde::{Deserialize, Serialize};

pub struct OpenAIService {
    client: RetriableClient,
    api_key: String,
    model: String,
}

impl OpenAIService {
    pub fn new(api_key: String, model: String) -> Self {
        OpenAIService {
            client: RetriableClient::default(),
            api_key,
            model,
        }
    }

    pub(crate) async fn fetch_models_internal(api_key: &str) -> Result<Vec<AiModel>> {
        debug!("Fetching OpenAI models");

        let client = RetriableClient::default();
        let response = client
            .send_with_retry("openai.models", RetryIntent::Idempotent, |http| {
                let api_key = api_key.to_string();
                let client = http.clone();
                async move {
                    client
                        .get(openai_api::MODELS)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await
                }
            })
            .await?;

        let response = ensure_success("OpenAI models", response).await?;
        let models_response: OpenAiModelsResponse = response.json().await?;

        let total_count = models_response.data.len();
        debug!("Total models from API: {}", total_count);

        let mut models: Vec<AiModel> = models_response
            .data
            .into_iter()
            .filter(|m| {
                let is_useful = ModelInfoService::is_useful_chat_model(&m.id);
                if !is_useful {
                    debug!("Filtering out model: {}", m.id);
                }
                is_useful
            })
            .map(|m| ModelInfoService::create_ai_model(m.id))
            .collect();

        models.sort_by_key(|m| ModelInfoService::get_model_priority(&m.id));

        info!(
            "Successfully fetched {} OpenAI chat models (filtered from {} total)",
            models.len(),
            total_count
        );
        Ok(models)
    }

    pub(crate) async fn test_api_key_internal(api_key: &str) -> Result<bool> {
        debug!("Testing OpenAI API key");

        let client = RetriableClient::default();
        let response = client
            .send_with_retry("openai.validate_key", RetryIntent::Idempotent, |http| {
                let api_key = api_key.to_string();
                let client = http.clone();
                async move {
                    client
                        .get(openai_api::MODELS)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await
                }
            })
            .await?;

        let status = response.status();
        log_status_if_error("OpenAI API key", status);
        let is_valid = status.is_success();
        debug!("OpenAI API key test result: {}", is_valid);
        Ok(is_valid)
    }
}

#[async_trait]
impl PromptSender for OpenAIService {
    async fn send_prompt(&self, prompt: &str) -> Result<(String, Option<TokenUsage>)> {
        log::info!("Sending prompt to OpenAI (model: {})", self.model);
        log::debug!("Prompt length: {} characters", prompt.len());

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "user", "content": prompt }
            ],
            "max_tokens": 2000,
            "temperature": 0.2,
            "response_format": { "type": "json_object" }
        });

        let api_key = self.api_key.clone();
        let resp = self
            .client
            .send_with_retry(
                "openai.chat_completions",
                RetryIntent::NonIdempotent,
                |http| {
                    let body = body.clone();
                    let api_key = api_key.clone();
                    let client = http.clone();
                    async move {
                        client
                            .post(openai_api::CHAT_COMPLETIONS)
                            .bearer_auth(api_key)
                            .json(&body)
                            .send()
                            .await
                    }
                },
            )
            .await
            .map_err(|e| {
                log::error!("OpenAI HTTP request failed: {}", e);
                e
            })?;

        let status = resp.status();
        log::debug!("OpenAI response status: {}", status);

        let resp = ensure_success("OpenAI chat completions", resp).await?;
        let text = resp.text().await?;
        log::debug!("OpenAI response length: {} bytes", text.len());

        // Parse OpenAI response structure: choices[0].message.content + usage
        if let Ok(v) = serde_json::from_str::<OpenAIResponse>(&text) {
            if let Some(choice) = v.choices.first() {
                log::info!("Successfully parsed OpenAI response");

                let token_usage = v.usage.map(|usage| TokenUsage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                });

                if let Some(ref tokens) = token_usage {
                    log::debug!(
                        "Token usage - prompt: {}, completion: {}, total: {}",
                        tokens.prompt_tokens,
                        tokens.completion_tokens,
                        tokens.total_tokens
                    );
                }

                return Ok((choice.message.content.clone(), token_usage));
            }
        }

        log::error!("Failed to parse OpenAI response structure");
        Err(anyhow::anyhow!("Failed to parse OpenAI response: {}", text))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelInfo {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_openai_response_format() {
        // Test parsing OpenAI's actual response structure
        let openai_response = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "{\"word\":\"test\",\"phonetic\":\"/tɛst/\",\"meanings\":[{\"partOfSpeech\":\"noun\",\"definitions\":[\"A procedure\"]}],\"synonyms\":[],\"antonyms\":[],\"examples\":[]}"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            }
        }"#;

        let response: OpenAIResponse = serde_json::from_str(openai_response).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert!(response.choices[0].message.content.contains("test"));
    }
}
