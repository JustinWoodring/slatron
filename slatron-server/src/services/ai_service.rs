use crate::models::{AiProvider, DjProfile, NewDjMemory};
use crate::AppState;
use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct DjResponse {
    pub text: String,
    pub emotion: Option<String>,
    pub memory_importance: Option<i32>,
    pub new_memory: Option<String>,
    pub next_track_id: Option<i32>,
}

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
        match provider.provider_type.as_str() {
            "ollama" => self.generate_ollama(prompt, provider).await,
            "openai" | "lmstudio" | "custom_llm" => {
                self.generate_openai_format(prompt, provider).await
            }
            "gemini" | "google" => self.generate_gemini_llm(prompt, provider).await,
            "anthropic" => self.generate_anthropic(prompt, provider).await,
            _ => Err(anyhow!(
                "Unsupported provider type: {}",
                provider.provider_type
            )),
        }
    }

    async fn generate_ollama(&self, prompt: &str, provider: &AiProvider) -> Result<String> {
        let endpoint = provider
            .endpoint_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434/api/generate".to_string());

        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "llama3".to_string());

        let request = OllamaRequest {
            model,
            prompt: prompt.to_string(),
            stream: false,
        };

        let res = self.client.post(&endpoint).json(&request).send().await?;

        if !res.status().is_success() {
            return Err(anyhow!("Ollama API error: {}", res.status()));
        }

        let response_body: OllamaResponse = res.json().await?;
        Ok(response_body.response)
    }

    async fn generate_openai_format(&self, prompt: &str, provider: &AiProvider) -> Result<String> {
        let endpoint = provider
            .endpoint_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());

        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        let api_key = provider.api_key.clone().unwrap_or_default();

        let request = OpenAiChatRequest {
            model,
            messages: vec![OpenAiMessage {
                role: "user".to_string(), // Or system + user if we want
                content: prompt.to_string(),
            }],
        };

        let res = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
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

    async fn generate_anthropic(&self, prompt: &str, provider: &AiProvider) -> Result<String> {
        // Anthropic API implementation
        let endpoint = "https://api.anthropic.com/v1/messages";
        let api_key = provider.api_key.clone().unwrap_or_default();
        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "claude-3-opus-20240229".to_string());

        let payload = serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "messages": [{ "role": "user", "content": prompt }]
        });

        let res = self
            .client
            .post(endpoint)
            .header("x-api-key", api_key)
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

    async fn generate_gemini_llm(&self, prompt: &str, provider: &AiProvider) -> Result<String> {
        let api_key = provider.api_key.clone().unwrap_or_default();
        let model = provider
            .model_name
            .clone()
            .unwrap_or_else(|| "gemini-pro".to_string());

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );

        let payload = serde_json::json!({
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

    pub async fn generate_dj_dialogue(
        &self,
        state: &AppState,
        profile: &DjProfile,
        context: &str,
        provider: &AiProvider,
        candidate_tracks: Option<&[crate::models::ContentItem]>,
    ) -> Result<DjResponse> {
        // 1. Fetch relevant memories
        let mut conn = state.db.get()?;
        use crate::schema::dj_memories::dsl::*;

        // Core Memories (High Importance)
        let core_memories = dj_memories
            .filter(dj_id.eq(profile.id.unwrap()))
            .filter(importance_score.ge(8))
            .order(created_at.desc())
            .limit(5)
            .load::<crate::models::DjMemory>(&mut conn)?;

        // Recent Memories (Short Term)
        let recent_memories = dj_memories
            .filter(dj_id.eq(profile.id.unwrap()))
            .order(created_at.desc())
            .limit(5)
            .load::<crate::models::DjMemory>(&mut conn)?;

        // Combine and format
        // Use a HashSet or similar to deduplicate by ID if needed, but simple overlap is acceptable for now.
        // Or better: Filter recent to exclude IDs present in core.
        // For simplicity, let's just show them in two sections.

        let mut memory_text = String::new();

        if !core_memories.is_empty() {
            memory_text.push_str("Core Memories (Permanent Personality Context):\n");
            for m in &core_memories {
                memory_text.push_str(&format!("- {}\n", m.content));
            }
            memory_text.push('\n');
        }

        memory_text.push_str("Recent Memories (Immediate Context):\n");
        if recent_memories.is_empty() {
            memory_text.push_str("- No recent events.\n");
        } else {
            for m in &recent_memories {
                memory_text.push_str(&format!("- {}\n", m.content));
            }
        }

        // 2. Run Context Transformers (Server-side Scripts)
        // Since scripts may use blocking HTTP, spawn_blocking is safer if we want to be correct,
        // though strictly speaking we are likely in a request handler where blocking a thread is bad but not fatal for low volume.
        // But let's do it properly.
        // 2. Run Context Transformers (Server-side Scripts)
        let script_service = state.script_service.clone();
        let state_clone_for_script = state.clone();

        let script_ids_str = profile.context_script_ids.clone().unwrap_or_default();
        let profile_clone = profile.clone();

        let script_context = tokio::task::spawn_blocking(move || {
            let scripts_config =
                crate::services::script_service::ScriptService::parse_config_string(
                    &script_ids_str,
                );
            script_service.run_context_scripts(
                &state_clone_for_script,
                &profile_clone,
                scripts_config,
            )
        })
        .await??;

        if !script_context.is_empty() {
            tracing::info!(
                "--- SCRIPT CONTEXT GENERATED ---\n{}\n-------------------------------",
                script_context
            );
        }

        let combined_context = if script_context.is_empty() {
            context.to_string()
        } else {
            format!("{}\n[System Context]: {}", context, script_context)
        };

        let mut track_selection_prompt = String::new();
        if let Some(tracks) = candidate_tracks {
            if !tracks.is_empty() {
                track_selection_prompt.push_str("\nAvailable Tracks to Pick From:\n");
                for t in tracks {
                    track_selection_prompt.push_str(&format!(
                        "- ID {}: {} (Type: {}, Desc: {})\n",
                        t.id.unwrap_or_default(),
                        t.title,
                        t.content_type,
                        t.description.as_deref().unwrap_or("")
                    ));
                }
                track_selection_prompt.push_str("\nINSTRUCTION: You MUST pick one track ID from the list above for the 'next_track_id' field. Choose the one that best fits your current mood/persona.\n");
            }
        }

        let full_prompt = format!(
            "You are a radio DJ.
Personality: {}
Recent Memories:
{}
Context: {}
{}

Generate a short break (1-3 sentences) suitable for a TTS engine.
IMPORTANT: You MUST use behavior tags to express emotion and pacing. Available tags: <laugh>, <sigh>, <breath>, <cough>, <clear_throat>.
Incorporate these naturally into the dialogue to make it feel ALIVE and human-like.
Output MUST be valid JSON with the following fields:
- text: The spoken words (including tags).
- emotion: A single adjective describing the voice style (e.g. 'excited', 'sad', 'scared', 'whispering', 'shouting'). If neutral, use null.
- memory_importance: Integer 1-10. How important is this event to remember? Be STRICT. 
  - 1-3: Routine chatter, song intros, generic comments. (DO NOT SAVE)
  - 4-7: Specific opinions, new user interactions, mild context shifts.
  - 8-10: Major events, distinct personality shifts, recurring jokes established, or critical information.
- new_memory: A short summary string of what just happened.
  - CRITICAL RULES FOR MEMORIES:
    1. **NO REPETITION**: Check 'Recent Memories'. If this event is similar to a recent one, return NULL.
    2. **HIGH ENTROPY**: Only save memories that add NEW, UNIQUE context. Do not save generic statements like \"DJ played a song\".
    3. **SPECIFICITY**: Be specific. Instead of \"DJ liked the song\", say \"DJ raved about the bassline in Track 88\".
  - If importance <= 5 or redundant, set this to null.
- next_track_id: Integer ID of the track you selected (if available tracks were provided).

Example JSON:
{{
  \"text\": \"That was... intense! Let's cool down with something smoother.\",
  \"emotion\": \"relaxed\",
  \"memory_importance\": 3,
  \"new_memory\": null,
  \"next_track_id\": 123
}}",
            profile.personality_prompt, memory_text, combined_context, track_selection_prompt
        );

        tracing::info!(
            "--- FULL SYSTEM PROMPT ---\n{}\n--------------------------",
            full_prompt
        );

        let json_str = self.generate_completion(&full_prompt, provider).await?;

        // Attempt to parse JSON. If it fails, try to strip markdown code blocks
        let clean_json = json_str
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```");

        // Parse JSON with comment stripping support
        let stripped = json_comments::StripComments::new(clean_json.as_bytes());
        let response: DjResponse = serde_json::from_reader(stripped)
            .map_err(|e| anyhow!("Failed to parse DJ JSON: {}. Content: {}", e, clean_json))?;

        // Save memory if important
        if response.memory_importance.unwrap_or(0) > 5 {
            if let Some(mem_content) = &response.new_memory {
                let new_mem = NewDjMemory {
                    dj_id: profile.id.unwrap(),
                    memory_type: "general".into(), // default
                    content: mem_content.clone(),
                    importance_score: response.memory_importance.unwrap_or(0),
                    happened_at: Utc::now().naive_utc(),
                };

                diesel::insert_into(dj_memories)
                    .values(&new_mem)
                    .execute(&mut conn)?;
            }
        }

        Ok(response)
    }
}
