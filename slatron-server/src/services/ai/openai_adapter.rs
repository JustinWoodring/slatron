use super::LlmProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::AiProvider;

#[derive(Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

pub struct OpenAiAdapter {
    client: Client,
    endpoint_url: String,
    model: String,
    api_key: String,
}

impl OpenAiAdapter {
    pub fn new(client: Client, provider: &AiProvider) -> Result<Self> {
        let endpoint_url = provider
            .endpoint_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());

        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        let api_key = provider.api_key.clone().unwrap_or_default();

        Ok(Self {
            client,
            endpoint_url,
            model,
            api_key,
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAiAdapter {
    async fn generate_completion(&self, prompt: &str) -> Result<String> {
        let request = OpenAiChatRequest {
            model: self.model.clone(),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let res = self
            .client
            .post(&self.endpoint_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let raw_text = res.text().await?;

        // Parse JSON while stripping comments
        let stripped = json_comments::StripComments::new(raw_text.as_bytes());
        let response_body: OpenAiChatResponse = serde_json::from_reader(stripped)
            .map_err(|e| anyhow!("Failed to parse OpenAI JSON: {}. Content: {}", e, raw_text))?;

        response_body
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("No choices in OpenAI response"))
    }
}
