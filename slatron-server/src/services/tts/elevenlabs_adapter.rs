use super::{TtsConfig, TtsProvider};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;

use crate::models::AiProvider;

pub struct ElevenLabsTtsAdapter {
    _client: Client,
    _api_key: Option<String>,
}

impl ElevenLabsTtsAdapter {
    pub fn new(client: Client, provider: &AiProvider) -> Result<Self> {
        Ok(Self {
            _client: client,
            _api_key: provider.api_key.clone(),
        })
    }
}

#[async_trait]
impl TtsProvider for ElevenLabsTtsAdapter {
    async fn generate_speech(
        &self,
        _text: &str,
        _config: TtsConfig,
        _output_dir: &PathBuf,
    ) -> Result<PathBuf> {
        Err(anyhow!(
            "ElevenLabs TTS support is not yet implemented. Coming soon!"
        ))
    }
}
