mod elevenlabs_adapter;
mod gemini_adapter;
mod orpheus_adapter;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;

use crate::models::AiProvider;

pub use elevenlabs_adapter::ElevenLabsTtsAdapter;
pub use gemini_adapter::GeminiTtsAdapter;
pub use orpheus_adapter::OrpheusTtsAdapter;

/// Configuration for TTS generation
#[derive(Debug, Clone)]
pub struct TtsConfig {
    pub voice_name: Option<String>,
    pub emotion: Option<String>,
    pub speech_modifier: Option<String>,
}

/// Trait for TTS provider implementations
#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn generate_speech(
        &self,
        text: &str,
        config: TtsConfig,
        output_dir: &PathBuf,
    ) -> Result<PathBuf>;
}

/// Main TTS service that routes to appropriate provider
pub struct TtsService {
    client: Client,
}

impl Default for TtsService {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn generate_speech(
        &self,
        text: &str,
        voice_name: Option<String>,
        emotion: Option<String>,
        speech_modifier: Option<String>,
        output_dir: &PathBuf,
        provider: &AiProvider,
    ) -> Result<PathBuf> {
        let config = TtsConfig {
            voice_name,
            emotion,
            speech_modifier,
        };

        let adapter: Box<dyn TtsProvider> = self.get_provider(provider)?;
        adapter.generate_speech(text, config, output_dir).await
    }

    fn get_provider(&self, provider: &AiProvider) -> Result<Box<dyn TtsProvider>> {
        match provider.provider_type.as_str() {
            "orpheus" => Ok(Box::new(OrpheusTtsAdapter::new(
                self.client.clone(),
                provider,
            )?)),
            "gemini" | "google" | "gemini-tts" => Ok(Box::new(GeminiTtsAdapter::new(
                self.client.clone(),
                provider,
            )?)),
            "elevenlabs" => Ok(Box::new(ElevenLabsTtsAdapter::new(
                self.client.clone(),
                provider,
            )?)),
            _ => Err(anyhow!(
                "Unsupported TTS provider type: {}",
                provider.provider_type
            )),
        }
    }
}
