use super::LlmProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::AiProvider;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct OllamaAdapter {
    client: Client,
    endpoint_url: String,
    model: String,
}

impl OllamaAdapter {
    pub fn new(client: Client, provider: &AiProvider) -> Result<Self> {
        let endpoint_url = provider
            .endpoint_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434/api/generate".to_string());

        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "llama3".to_string());

        Ok(Self {
            client,
            endpoint_url,
            model,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaAdapter {
    async fn generate_completion(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let res = self
            .client
            .post(&self.endpoint_url)
            .json(&request)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Ollama API error: {}", res.status()));
        }

        let response_body: OllamaResponse = res.json().await?;
        Ok(response_body.response)
    }
}
