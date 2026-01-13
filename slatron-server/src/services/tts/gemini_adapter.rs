use super::{TtsConfig, TtsProvider};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

use crate::models::AiProvider;

pub struct GeminiTtsAdapter {
    client: Client,
    api_key: String,
}

impl GeminiTtsAdapter {
    pub fn new(client: Client, provider: &AiProvider) -> Result<Self> {
        let api_key = provider
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("Gemini TTS provider API key is missing"))?
            .clone();

        Ok(Self { client, api_key })
    }
}

#[async_trait]
impl TtsProvider for GeminiTtsAdapter {
    async fn generate_speech(
        &self,
        text: &str,
        config: TtsConfig,
        output_dir: &PathBuf,
    ) -> Result<PathBuf> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-tts:generateContent?key={}",
            self.api_key
        );

        let voice = config.voice_name.unwrap_or_else(|| "Aoede".to_string());

        // Construct style instructions
        let mut style_parts = Vec::new();
        if let Some(emo) = config.emotion {
            style_parts.push(format!("{} voice", emo));
        }
        if let Some(mod_str) = config.speech_modifier {
            if !mod_str.trim().is_empty() {
                style_parts.push(mod_str);
            }
        }

        let final_text = if style_parts.is_empty() {
            text.to_string()
        } else {
            format!("Say with a {}: {}", style_parts.join(", "), text)
        };

        let payload = json!({
            "contents": [{
                "parts": [{ "text": final_text }]
            }],
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": {
                            "voiceName": voice
                        }
                    }
                }
            }
        });

        let res = self.client.post(&url).json(&payload).send().await?;

        if !res.status().is_success() {
            let err_text = res.text().await?;
            return Err(anyhow!("Gemini TTS API error: {}", err_text));
        }

        let response_json: serde_json::Value = res.json().await?;

        // Extract base64 audio
        let audio_base64 = response_json["candidates"][0]["content"]["parts"][0]["inlineData"]
            ["data"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract audio data from response"))?;

        // Decode base64
        let audio_data = general_purpose::STANDARD.decode(audio_base64)?;

        // Ensure output directory exists
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        // Save raw PCM
        let pcm_filename = format!("{}.pcm", Uuid::new_v4());
        let pcm_path = output_dir.join(&pcm_filename);
        {
            let mut file = std::fs::File::create(&pcm_path)?;
            file.write_all(&audio_data)?;
        }

        // Convert to WAV using ffmpeg
        let wav_filename = format!("{}.wav", Uuid::new_v4());
        let wav_path = output_dir.join(&wav_filename);

        tracing::info!("Converting {:?} to {:?}", pcm_path, wav_path);

        let status = Command::new("ffmpeg")
            .arg("-y") // Overwrite output
            .arg("-f")
            .arg("s16le")
            .arg("-ar")
            .arg("24000")
            .arg("-ac")
            .arg("1")
            .arg("-i")
            .arg(&pcm_path)
            .arg(&wav_path)
            .status()?;

        // Clean up PCM
        let _ = std::fs::remove_file(pcm_path);

        if status.success() {
            Ok(wav_path)
        } else {
            Err(anyhow!("ffmpeg conversion failed"))
        }
    }
}
