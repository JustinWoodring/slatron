use super::LlmProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::models::AiProvider;

pub struct AnthropicAdapter {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicAdapter {
    pub fn new(client: Client, provider: &AiProvider) -> Result<Self> {
        let api_key = provider
            .api_key
            .clone()
            .ok_or_else(|| anyhow!("Anthropic provider API key is missing"))?;

        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "claude-3-opus-20240229".to_string());

        Ok(Self {
            client,
            api_key,
            model,
        })
    }
}

#[async_trait]
impl LlmProvider for AnthropicAdapter {
    async fn generate_completion(&self, prompt: &str) -> Result<String> {
        let endpoint = "https://api.anthropic.com/v1/messages";

        let payload = json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": [{ "role": "user", "content": prompt }]
        });

        let res = self
            .client
            .post(endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow!("Anthropic API error: {}", error_text));
        }

        let response_json: serde_json::Value = res.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract content from Anthropic response"))?;

        Ok(content.to_string())
    }
}
