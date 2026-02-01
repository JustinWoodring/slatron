mod anthropic_adapter;
mod gemini_adapter;
mod ollama_adapter;
mod openai_adapter;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;

use crate::models::AiProvider;

pub use anthropic_adapter::AnthropicAdapter;
pub use gemini_adapter::GeminiLlmAdapter;
pub use ollama_adapter::OllamaAdapter;
pub use openai_adapter::OpenAiAdapter;

/// Trait for LLM provider implementations
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate_completion(&self, prompt: &str) -> Result<String>;
}

/// Main AI service that routes to appropriate LLM provider
pub struct AiService {
    client: Client,
}

impl AiService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn generate_completion(&self, prompt: &str, provider: &AiProvider) -> Result<String> {
        let adapter: Box<dyn LlmProvider> = self.get_provider(provider)?;
        adapter.generate_completion(prompt).await
    }

    fn get_provider(&self, provider: &AiProvider) -> Result<Box<dyn LlmProvider>> {
        match provider.provider_type.as_str() {
            "ollama" => Ok(Box::new(OllamaAdapter::new(self.client.clone(), provider)?)),
            "openai" | "lmstudio" | "custom_llm" => {
                Ok(Box::new(OpenAiAdapter::new(self.client.clone(), provider)?))
            }
            "gemini" | "google" => Ok(Box::new(GeminiLlmAdapter::new(
                self.client.clone(),
                provider,
            )?)),
            "anthropic" => Ok(Box::new(AnthropicAdapter::new(
                self.client.clone(),
                provider,
            )?)),
            _ => Err(anyhow!(
                "Unsupported LLM provider type: {}",
                provider.provider_type
            )),
        }
    }
}
