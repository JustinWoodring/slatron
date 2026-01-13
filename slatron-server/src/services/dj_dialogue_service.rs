use crate::models::{AiProvider, ContentItem, DjMemory, DjProfile, NewDjMemory};
use crate::services::ai::AiService;
use crate::services::script_service::ScriptService;
use crate::AppState;
use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct DjResponse {
    pub text: String,
    pub emotion: Option<String>,
    pub memory_importance: Option<i32>,
    pub new_memory: Option<String>,
    pub next_track_id: Option<i32>,
}

pub struct DjDialogueService {
    ai_service: Arc<AiService>,
    script_service: Arc<ScriptService>,
}

impl DjDialogueService {
    pub fn new(ai_service: Arc<AiService>, script_service: Arc<ScriptService>) -> Self {
        Self {
            ai_service,
            script_service,
        }
    }

    pub async fn generate_dialogue(
        &self,
        state: &AppState,
        profile: &DjProfile,
        context: &str,
        provider: &AiProvider,
        candidate_tracks: Option<&[ContentItem]>,
        schedule_info: Option<serde_json::Value>,
    ) -> Result<DjResponse> {
        let memories = self.fetch_memories(state, profile).await?;
        let memory_text = self.format_memories(&memories.0, &memories.1);
        let script_context = self
            .run_context_scripts(state, profile, candidate_tracks, schedule_info)
            .await?;
        let combined_context = self.combine_context(context, &script_context);
        let track_selection_prompt = self.build_track_selection_prompt(candidate_tracks);
        let full_prompt = self.build_full_prompt(
            profile,
            &memory_text,
            &combined_context,
            &track_selection_prompt,
        );

        let response = self
            .generate_and_parse_with_retry(&full_prompt, provider, 3)
            .await?;

        self.save_memory_if_important(state, profile, &response)
            .await?;

        Ok(response)
    }

    async fn fetch_memories(
        &self,
        state: &AppState,
        profile: &DjProfile,
    ) -> Result<(Vec<DjMemory>, Vec<DjMemory>)> {
        let mut conn = state.db.get()?;
        use crate::schema::dj_memories::dsl::*;

        let profile_id = profile.id.unwrap();

        // Core Memories (High Importance)
        let core_memories = dj_memories
            .filter(dj_id.eq(profile_id))
            .filter(importance_score.ge(8))
            .order(created_at.desc())
            .limit(5)
            .load::<DjMemory>(&mut conn)?;

        // Recent Memories (Short Term)
        let recent_memories = dj_memories
            .filter(dj_id.eq(profile_id))
            .order(created_at.desc())
            .limit(5)
            .load::<DjMemory>(&mut conn)?;

        Ok((core_memories, recent_memories))
    }

    fn format_memories(&self, core_memories: &[DjMemory], recent_memories: &[DjMemory]) -> String {
        let mut memory_text = String::new();

        if !core_memories.is_empty() {
            memory_text.push_str("Core Memories (Permanent Personality Context):\n");
            for m in core_memories {
                memory_text.push_str(&format!("- {}\n", m.content));
            }
            memory_text.push('\n');
        }

        memory_text.push_str("Recent Memories (Immediate Context):\n");
        if recent_memories.is_empty() {
            memory_text.push_str("- No recent events.\n");
        } else {
            for m in recent_memories {
                memory_text.push_str(&format!("- {}\n", m.content));
            }
        }

        memory_text
    }

    async fn run_context_scripts(
        &self,
        state: &AppState,
        profile: &DjProfile,
        candidate_tracks: Option<&[ContentItem]>,
        schedule_info: Option<serde_json::Value>,
    ) -> Result<String> {
        let script_service = self.script_service.clone();
        let state_clone = state.clone();
        let script_ids_str = profile.context_script_ids.clone().unwrap_or_default();
        let profile_clone = profile.clone();
        let track_option = candidate_tracks.and_then(|t| t.first()).cloned();
        let schedule_info_clone = schedule_info.clone();

        let script_context = tokio::task::spawn_blocking(move || {
            // Fetch Timezone (Synchronously for blocking task)
            let mut conn = state_clone.db.get().map_err(|e| anyhow::anyhow!(e))?;
            use crate::schema::global_settings::dsl as gs;
            let tz: String = gs::global_settings
                .filter(gs::key.eq("server_timezone"))
                .select(gs::value)
                .first(&mut conn)
                .optional()
                .unwrap_or(None)
                .unwrap_or_else(|| "UTC".to_string());

            let scripts_config = ScriptService::parse_config_string(&script_ids_str);

            script_service.run_context_scripts(
                &state_clone,
                scripts_config,
                &profile_clone,
                track_option.as_ref(),
                tz,
                schedule_info_clone,
            )
        })
        .await??;

        if !script_context.is_empty() {
            tracing::info!(
                "--- SCRIPT CONTEXT GENERATED ---\n{}\n-------------------------------",
                script_context
            );
        }

        Ok(script_context)
    }

    fn combine_context(&self, base_context: &str, script_context: &str) -> String {
        if script_context.is_empty() {
            base_context.to_string()
        } else {
            format!("{}\n[System Context]: {}", base_context, script_context)
        }
    }

    fn build_track_selection_prompt(&self, candidate_tracks: Option<&[ContentItem]>) -> String {
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

        track_selection_prompt
    }

    fn build_full_prompt(
        &self,
        profile: &DjProfile,
        memory_text: &str,
        context: &str,
        track_selection: &str,
    ) -> String {
        format!(
            "You are a radio DJ.
Personality: {}
Recent Memories:
{}
Context: {}
{}

Generate a short break (1-3 sentences) suitable for a TTS engine.

CRITICAL INSTRUCTION: Check 'Recent Memories' above.
- If you have recently discussed a specific news story, topic, fact, or anecdote, YOU MUST NOT MENTION IT AGAIN.
- If you have run out of new topics, focus on the music, the vibe, or the artist. Do not repeat old news.

IMPORTANT: You MUST use behavior tags to express emotion and pacing. Available tags: <giggle>, <laugh>, <chuckle>, <sigh>, <cough>, <sniffle>, <groan>, <yawn>, <gasp>.
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
            profile.personality_prompt, memory_text, context, track_selection
        )
    }

    async fn generate_and_parse_with_retry(
        &self,
        prompt: &str,
        provider: &AiProvider,
        max_attempts: u32,
    ) -> Result<DjResponse> {
        tracing::info!(
            "--- FULL SYSTEM PROMPT ---\n{}\n--------------------------",
            prompt
        );

        let mut current_prompt = prompt.to_string();

        for attempt in 1..=max_attempts {
            tracing::info!("Generating DJ Dialogue (Attempt {}/{})", attempt, max_attempts);

            let json_str = match self
                .ai_service
                .generate_completion(&current_prompt, provider)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("LLM Generation failed: {}", e);
                    if attempt >= max_attempts {
                        return Err(e);
                    }
                    continue;
                }
            };

            // Attempt to parse JSON. If it fails, try to strip markdown code blocks
            let clean_json = json_str
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```");

            // Parse JSON with comment stripping support
            let stripped = json_comments::StripComments::new(clean_json.as_bytes());
            match serde_json::from_reader::<_, DjResponse>(stripped) {
                Ok(response) => {
                    return Ok(response);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse DJ JSON: {}. Content: {}", e, clean_json);
                    if attempt >= max_attempts {
                        return Err(anyhow!(
                            "Failed to parse DJ JSON after {} attempts: {}. Content: {}",
                            max_attempts,
                            e,
                            clean_json
                        ));
                    }

                    // Add correction instruction for next attempt
                    current_prompt = format!("{}\n\nSYSTEM: Your previous response was invalid JSON. Error: {}. Please output ONLY valid JSON matching the schema.", current_prompt, e);
                }
            }
        }

        Err(anyhow!("Failed to generate DJ dialogue after retries"))
    }

    async fn save_memory_if_important(
        &self,
        state: &AppState,
        profile: &DjProfile,
        response: &DjResponse,
    ) -> Result<()> {
        if response.memory_importance.unwrap_or(0) > 5 {
            if let Some(mem_content) = &response.new_memory {
                let mut conn = state.db.get()?;
                use crate::schema::dj_memories::dsl::*;

                let new_mem = NewDjMemory {
                    dj_id: profile.id.unwrap(),
                    memory_type: "general".into(),
                    content: mem_content.clone(),
                    importance_score: response.memory_importance.unwrap_or(0),
                    happened_at: Utc::now().naive_utc(),
                };

                diesel::insert_into(dj_memories)
                    .values(&new_mem)
                    .execute(&mut conn)?;
            }
        }
        Ok(())
    }
}
