use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
#[cfg(feature = "ml-support")]
use ort::session::{builder::GraphOptimizationLevel, Session};
use reqwest::Client;
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

use crate::models::AiProvider;

pub struct TtsService {
    client: Client,
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
        match provider.provider_type.as_str() {
            "orpheus" => {
                self.generate_orpheus(text, voice_name, output_dir, provider)
                    .await
            }
            "gemini" | "google" | "gemini-tts" => {
                self.generate_gemini(
                    text,
                    voice_name,
                    emotion,
                    speech_modifier,
                    output_dir,
                    provider,
                )
                .await
            }
            "elevenlabs" => {
                // Placeholder for ElevenLabs support
                Err(anyhow!("ElevenLabs TTS support is not yet implemented"))
            }
            _ => Err(anyhow!(
                "Unsupported TTS provider type: {}",
                provider.provider_type
            )),
        }
    }

    #[allow(unused_variables)]
    async fn generate_orpheus(
        &self,
        text: &str,
        voice_name: Option<String>,
        output_dir: &PathBuf,
        provider: &AiProvider,
    ) -> Result<PathBuf> {
        #[cfg(feature = "ml-support")]
        {
            let lm_studio_url = provider
                .endpoint_url
                .as_deref()
                .unwrap_or("http://127.0.0.1:1234/v1/completions");

            // Use extracted model path
            let decoder_path_str = "data/snac.onnx";
            if !std::path::Path::new(decoder_path_str).exists() {
                return Err(anyhow!(
                    "SNAC model not found at {}. Ensure ml-support feature is active.",
                    decoder_path_str
                ));
            }
            let decoder_path = decoder_path_str;

            let voice = voice_name.unwrap_or_else(|| "tara".to_string());

            // 1. Format Prompt
            // Format: <|audio|>voice: text<|eot_id|>
            let sanitized_text = Self::sanitize_orpheus_text(text);
            let prompt = format!("<|audio|>{}: {}<|eot_id|>", voice, sanitized_text);

            let payload = json!({
                "model": "orpheus-3b-0.1-ft",
                "prompt": prompt,
                "max_tokens": 4096,
                "temperature": 0.6,
                "top_p": 0.9,
                "repeat_penalty": 1.1,
                "stream": true
            });

            println!("Requesting Orpheus TTS from LM Studio: {}", lm_studio_url);
            let mut res = self
                .client
                .post(lm_studio_url)
                .json(&payload)
                .send()
                .await?;

            if !res.status().is_success() {
                return Err(anyhow!("LM Studio API error: {}", res.status()));
            }

            // 2. Stream and Parse Tokens
            let mut collected_ids: Vec<i32> = Vec::new();
            let mut token_count = 0;
            let mut buffer = String::new();

            while let Some(chunk_res) = res.chunk().await? {
                let chunk_str = String::from_utf8_lossy(&chunk_res);
                buffer.push_str(&chunk_str);

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    if buffer.len() > pos + 1 {
                        buffer = buffer[pos + 1..].to_string();
                    } else {
                        buffer.clear();
                    }

                    let line = line.trim();
                    if line.starts_with("data: ") {
                        let data_str = &line[6..];
                        if data_str == "[DONE]" {
                            break;
                        }

                        if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(data_str) {
                            if let Some(choices) = json_data.get("choices") {
                                if let Some(choice) = choices.get(0) {
                                    if let Some(text_val) = choice.get("text") {
                                        if let Some(token_text) = text_val.as_str() {
                                            let trimmed = token_text.trim();
                                            if let Some(start) = trimmed.rfind("<custom_token_") {
                                                let substr = &trimmed[start..];
                                                if substr.ends_with('>') {
                                                    if let Ok(num) =
                                                        substr[14..substr.len() - 1].parse::<i32>()
                                                    {
                                                        let id =
                                                            num - 10 - ((token_count % 7) * 4096);
                                                        if id >= 0 {
                                                            collected_ids.push(id);
                                                            token_count += 1;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if collected_ids.is_empty() {
                return Err(anyhow!("No audio tokens received from Orpheus/LM Studio"));
            }

            // 5. Run SNAC Decoder (Native Rust via ONNX Runtime)
            let mut c0_vec: Vec<i32> = Vec::new();
            let mut c1_vec: Vec<i32> = Vec::new();
            let mut c2_vec: Vec<i32> = Vec::new();

            for chunk in collected_ids.chunks(7) {
                if chunk.len() < 7 {
                    continue;
                }
                c0_vec.push(chunk[0]);

                c1_vec.push(chunk[1]);
                c1_vec.push(chunk[4]);

                c2_vec.push(chunk[2]);
                c2_vec.push(chunk[3]);
                c2_vec.push(chunk[5]);
                c2_vec.push(chunk[6]);
            }

            // Create inputs using (shape, data) tuple to avoid ndarray version issues
            let v0 = ort::value::Value::from_array(([1, c0_vec.len()], c0_vec))?;
            let v1 = ort::value::Value::from_array(([1, c1_vec.len()], c1_vec))?;
            let v2 = ort::value::Value::from_array(([1, c2_vec.len()], c2_vec))?;

            println!("Loading SNAC ONNX model from: {}", decoder_path);
            let mut session = Session::builder()?
                .with_optimization_level(GraphOptimizationLevel::Level3)?
                .with_intra_threads(4)?
                .commit_from_file(decoder_path)?;

            let outputs = session.run(ort::inputs![
                "codes_0" => v0,
                "codes_1" => v1,
                "codes_2" => v2
            ])?;

            let audio_output = outputs["audio"].try_extract_tensor::<f32>()?;
            let (_, audio_data) = audio_output;

            let audio_i16: Vec<i16> = audio_data
                .iter()
                .map(|&x| (x * 32767.0).clamp(-32768.0, 32767.0) as i16)
                .collect();

            // Write WAV
            let wav_filename = format!("{}.wav", Uuid::new_v4());
            let wav_path = output_dir.join(&wav_filename);

            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: 24000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };

            println!(
                "Writing {} samples to WAV at {:?}",
                audio_i16.len(),
                wav_path
            );
            let mut writer = hound::WavWriter::create(&wav_path, spec)?;
            for sample in audio_i16 {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;

            return Ok(wav_path);
        }

        #[cfg(not(feature = "ml-support"))]
        {
            return Err(anyhow!(
                "Orpheus TTS requires 'ml-support' feature to be enabled"
            ));
        }
    }

    async fn generate_gemini(
        &self,
        text: &str,
        voice_name: Option<String>,
        emotion: Option<String>,
        speech_modifier: Option<String>,
        output_dir: &PathBuf,
        provider: &AiProvider,
    ) -> Result<PathBuf> {
        let api_key = provider
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("Provider API key is missing"))?;

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-tts:generateContent?key={}",
            api_key
        );

        let voice = voice_name.unwrap_or_else(|| "Aoede".to_string());

        // Construct style instructions
        let mut style_parts = Vec::new();
        if let Some(emo) = emotion {
            style_parts.push(format!("{} voice", emo));
        }
        if let Some(mod_str) = speech_modifier {
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
        let request = general_purpose::STANDARD.decode(audio_base64)?;

        // Ensure output directory exists (also for Gemini path)
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        // Save raw PCM
        let pcm_filename = format!("{}.pcm", Uuid::new_v4());
        let pcm_path = output_dir.join(&pcm_filename);
        {
            let mut file = std::fs::File::create(&pcm_path)?;
            file.write_all(&request)?;
        }

        // Convert to WAV using ffmpeg
        let wav_filename = format!("{}.wav", Uuid::new_v4());
        let wav_path = output_dir.join(&wav_filename);

        println!("Converting {:?} to {:?}", pcm_path, wav_path);

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

    #[allow(dead_code)]
    fn sanitize_orpheus_text(text: &str) -> String {
        let allowed_tags = [
            "giggle", "laugh", "chuckle", "sigh", "cough", "sniffle", "groan", "yawn", "gasp",
        ];

        let mut output = String::with_capacity(text.len());
        let mut tag_buffer = String::new();
        let mut in_tag = false;

        for c in text.chars() {
            if in_tag {
                if c == '>' {
                    // Tag Closed
                    if allowed_tags.contains(&tag_buffer.as_str()) {
                        output.push('<');
                        output.push_str(&tag_buffer);
                        output.push('>');
                    }
                    // Else: filter out (skip)

                    in_tag = false;
                    tag_buffer.clear();
                } else if c.is_whitespace() {
                    // Not a tag, treat as text
                    output.push('<');
                    output.push_str(&tag_buffer);
                    output.push(c);
                    in_tag = false;
                    tag_buffer.clear();
                } else {
                    tag_buffer.push(c);
                }
            } else {
                if c == '<' {
                    in_tag = true;
                } else {
                    output.push(c);
                }
            }
        }

        // Handle unclosed tag at end of string
        if in_tag {
            output.push('<');
            output.push_str(&tag_buffer);
        }

        output
    }
}
