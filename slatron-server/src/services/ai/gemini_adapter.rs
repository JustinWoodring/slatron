use super::LlmProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::models::AiProvider;

pub struct GeminiLlmAdapter {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiLlmAdapter {
    pub fn new(client: Client, provider: &AiProvider) -> Result<Self> {
        let api_key = provider
            .api_key
            .clone()
            .ok_or_else(|| anyhow!("Gemini provider API key is missing"))?;

        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "gemini-pro".to_string());

        Ok(Self {
            client,
            api_key,
            model,
        })
    }
}

#[async_trait]
impl LlmProvider for GeminiLlmAdapter {
    async fn generate_completion(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let payload = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }]
        });

        let res = self.client.post(&url).json(&payload).send().await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let response_json: serde_json::Value = res.json().await?;
        let content = response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract content from Gemini response"))?;

        Ok(content.to_string())
    }
}
