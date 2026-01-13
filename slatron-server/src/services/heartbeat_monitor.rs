use crate::models::{AiProvider, ContentItem, DjProfile, Node};
use crate::websocket::{NodeCommand, ServerMessage};
use crate::AppState;
use chrono::Utc;
use diesel::prelude::*;
use std::time::Duration;
use tokio::time::interval;

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::sync::{Arc, Mutex};

struct GenerationGuard {
    node_id: i32,
    active_set: Arc<Mutex<HashSet<i32>>>,
}

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = self.active_set.lock() {
            set.remove(&self.node_id);
        }
    }
}

pub async fn run(state: AppState) {
    // Run every 10 seconds
    let mut tick = interval(Duration::from_secs(10));
    // Track last triggered Content ID per Node ID to prevent double triggers for same song
    // Map<NodeID, (ContentID, Timestamp)>
    let last_triggered_content: Arc<Mutex<HashMap<i32, (i32, Instant)>>> = Arc::new(Mutex::new(HashMap::new()));
    // Track known content on node to detect unloads
    // Track active processing tasks to prevent stacking
    let mut last_known_content: HashMap<i32, i32> = HashMap::new();
    let active_generations: Arc<Mutex<HashSet<i32>>> = Arc::new(Mutex::new(HashSet::new()));

    loop {
        tick.tick().await;

        if let Err(e) = check_heartbeats(&state).await {
            tracing::error!("Heartbeat monitor error: {}", e);
        }

        if let Err(e) = process_dj_logic(
            &state, 
            last_triggered_content.clone(), 
            &mut last_known_content,
            active_generations.clone()
        ).await {
            tracing::error!("DJ Logic error: {}", e);
        }
    }
}

async fn check_heartbeats(state: &AppState) -> Result<(), String> {
    use crate::schema::nodes::dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| "Database connection error".to_string())?;

    // Threshold: 30 seconds ago
    let threshold = Utc::now().naive_utc() - chrono::Duration::seconds(30);

    // Find nodes that are 'online' but haven't sent a heartbeat recently
    let offline_count = diesel::update(
        dsl::nodes
            .filter(dsl::status.eq("online"))
            .filter(dsl::last_heartbeat.lt(threshold)),
    )
    .set(dsl::status.eq("offline"))
    .execute(&mut conn)
    .map_err(|e| e.to_string())?;

    if offline_count > 0 {
        tracing::warn!("Marked {} unresponsive nodes as offline", offline_count);
    }

    Ok(())
}

async fn process_dj_logic(
    state: &AppState,
    last_triggered_content: Arc<Mutex<HashMap<i32, (i32, Instant)>>>,
    last_known_content: &mut HashMap<i32, i32>,
    active_generations: Arc<Mutex<HashSet<i32>>>,
) -> Result<(), anyhow::Error> {
    use crate::schema::ai_providers::dsl as ai_dsl;
    use crate::schema::content_items::dsl as c_dsl;
    use crate::schema::dj_profiles::dsl as dj_dsl;
    use crate::schema::nodes::dsl as n_dsl;
    use crate::schema::schedules::dsl as s_dsl;

    // 1. Get ALL online nodes (playing or idle)
    let mut conn = state.db.get().map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let nodes_online = n_dsl::nodes
        .filter(n_dsl::status.eq("online"))
        .load::<Node>(&mut conn)?;

    for node in nodes_online {
        let node_id = node.id.ok_or_else(|| anyhow::anyhow!("Node missing ID"))?;

        // 0. GENERATION INHIBITOR
        {
            if let Ok(set) = active_generations.lock() {
                if set.contains(&node_id) {
                    tracing::debug!("Node {} skipped: DJ generation currently in progress.", node_id);
                    continue;
                }
            }
        }
        
        let current_content_id_val = node.current_content_id.unwrap_or(0); // 0 for cold start

        // Detect Content Change (Unload)
        if let Some(prev_id) = last_known_content.get(&node_id) {
            if *prev_id != current_content_id_val && *prev_id != 0 {
                // Content changed! Run on_unload for prev_id
                let prev_content_opt = c_dsl::content_items
                    .filter(c_dsl::id.eq(*prev_id))
                    .first::<ContentItem>(&mut conn)
                    .optional()?;
                
                if let Some(prev_item) = prev_content_opt {
                     let t_scripts_str = prev_item.transformer_scripts.clone().unwrap_or_default();
                     let t_config = crate::services::script_service::ScriptService::parse_config_string(&t_scripts_str);
                     for cfg in &t_config {
                         if let Err(e) = state.script_service.call_entry_point(state, cfg, &prev_item, None, "on_unload") {
                            tracing::warn!("Failed to run on_unload for content {}: {}", prev_item.id.unwrap_or_default(), e);
                         }
                     }
                }
            }
        }
        last_known_content.insert(node_id, current_content_id_val);

        // Deduplication Check at Item Level
        // If we already triggered for this exact content ID on this node, skip.
        // Deduplication Check at Item Level
        // If we already triggered for this exact content ID on this node...
        if let Some((last_id, last_time)) = {
            let map = last_triggered_content.lock().unwrap();
            map.get(&node_id).cloned()
        } {
            if last_id == current_content_id_val {
                if current_content_id_val == 0 {
                    // Specific logic for Cold Start (0):
                    // If we triggered recently (< 45s), skip and let it load.
                    // If it's been > 45s and STILL 0, retry.
                    if last_time.elapsed().as_secs() < 45 {
                        tracing::debug!("Node {} Cold Start still loading (waited {}s)", node_id, last_time.elapsed().as_secs());
                        continue;
                    }
                    tracing::warn!("Node {} Cold Start stuck for >45s. Retrying...", node_id);
                } else {
                     // Normal Content: strict dedupe. We only support "Smart Schedule" once per song.
                     continue;
                }
            }
        }

        // Determine if we should trigger DJ logic
        // Case A: Song Ending (remaining < 65s) -> Early trigger
        // Case B: Cold Start (No content playing)

        let mut trigger_dj = false;
        let mut current_song_title = "Silence".to_string(); // Default for cold start
        let mut remaining_seconds = 0.0;

        if let Some(content_id_val) = node.current_content_id {
            if let Some(playback_pos) = node.playback_position_secs {
                // Fetch current content duration
                let content_item_opt = c_dsl::content_items
                    .filter(c_dsl::id.eq(content_id_val))
                    .first::<ContentItem>(&mut conn)
                    .optional()?;

                if let Some(item) = content_item_opt {
                    let duration_seconds = if let Some(dur) = node.playback_duration_secs {
                        dur
                    } else {
                        item.duration_minutes.unwrap_or(0) as f32 * 60.0
                    };

                    let remaining = duration_seconds - playback_pos;

                    tracing::debug!(
                        "DEBUG: Node {} Playing {}. Pos: {:.1}/{:.1}. Rem: {:.1}. Dedupe Last: {:?}",
                        node_id,
                        content_id_val,
                        playback_pos,
                        duration_seconds,
                        remaining,
                        last_triggered_content.lock().unwrap().get(&node_id)
                    );

                    if duration_seconds > 0.0 {
                        // Trigger if between 3s and 65s remaining (early window)
                        if remaining > 3.0 && remaining < 65.0 {
                            // Check if item allows DJ
                            if item.is_dj_accessible {
                                trigger_dj = true;
                                current_song_title = item.title.clone();
                                remaining_seconds = remaining;
                            }
                        }
                    }
                }
            }
        } else {
            // Case B: Cold Start / Idle
            tracing::debug!(
                "DEBUG: Node {} is Idle/Cold. Dedupe Last: {:?}",
                node_id,
                last_triggered_content.lock().unwrap().get(&node_id)
            );
            trigger_dj = true;
        }

        if !trigger_dj {
            continue;
        }

        // Logic triggered. Mark as handled immediately.
        {
            let mut map = last_triggered_content.lock().unwrap();
            map.insert(node_id, (current_content_id_val, Instant::now()));
        }

        tracing::debug!(
            "DEBUG: Node {} triggering DJ logic (ColdStart={})",
            node_id,
            node.current_content_id.is_none()
        );

        // --- Common DJ Logic Below ---

        // Try to find active schedule to find specific DJ
        // 1. Check for Node-Specific Schedule
        use crate::schema::node_schedules::dsl as ns_dsl;

        // Find all schedules assigned to this node, ordered by priority
        let node_schedules_all = ns_dsl::node_schedules
            .filter(ns_dsl::node_id.eq(node_id))
            .inner_join(s_dsl::schedules)
            .filter(s_dsl::is_active.eq(true))
            .order(ns_dsl::priority.desc())
            .select(crate::models::Schedule::as_select())
            .load::<crate::models::Schedule>(&mut conn)?;

        // Find all active global schedules as a fallback base
        let global_schedules = s_dsl::schedules
            .filter(s_dsl::is_active.eq(true))
            .order(s_dsl::priority.desc())
            .load::<crate::models::Schedule>(&mut conn)?;

        // Combine: Node Specific (Higher Priority) -> Global (Lower Priority)
        let mut candidate_map: HashMap<i32, crate::models::Schedule> = HashMap::new();
        // Insert globals first
        for s in global_schedules {
            if let Some(sid) = s.id { candidate_map.insert(sid, s); }
        }
        // Insert node schedules (overwriting if same ID)
        for s in node_schedules_all {
             if let Some(sid) = s.id { candidate_map.insert(sid, s); }
        }
        
        let mut active_schedules: Vec<crate::models::Schedule> = candidate_map.into_values().collect();
        // Sort by Priority Descending
        active_schedules.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut dj_profile_opt: Option<DjProfile> = None;
        let mut active_block_script: Option<String> = None;
        let mut active_block_info: Option<serde_json::Value> = None;

        let mut active_schedule: Option<crate::models::Schedule> = None;

        // Fetch Timezone
        use crate::schema::global_settings::dsl::{global_settings, key, value};
        use chrono::{Datelike, Timelike};
        use chrono_tz::Tz;

        let timezone_setting: Option<String> = global_settings
            .filter(key.eq("timezone"))
            .select(value)
            .first(&mut conn)
            .optional()
            .unwrap_or(None);

        let tz: Tz = timezone_setting
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(chrono_tz::UTC);
        
        let now_utc = Utc::now();
        let now_target = now_utc.with_timezone(&tz);
        let current_time = now_target.time();
        let current_dow = now_target.weekday().number_from_monday() as i32 - 1;
        let current_date = now_target.date_naive();
        
        // CASCADING SCHEDULE CHECK
        for schedule in active_schedules {
            if let Some(sched_id) = schedule.id {
                use crate::models::{ScheduleBlock, Script};
                use crate::schema::schedule_blocks::dsl as sb_dsl;
                use crate::schema::scripts::dsl as sc_dsl;

                let blocks = sb_dsl::schedule_blocks
                    .filter(sb_dsl::schedule_id.eq(sched_id))
                    .load::<ScheduleBlock>(&mut conn)
                    .unwrap_or_default();

                let active_block = blocks.iter().find(|b| {
                    let matches_day = if let Some(d) = b.specific_date {
                        d == current_date
                    } else if let Some(dow) = b.day_of_week {
                        dow == current_dow
                    } else {
                        false
                    };

                    if !matches_day {
                         return false;
                    }

                    let start = b.start_time;
                    let start_secs = start.num_seconds_from_midnight();
                    let end_secs = start_secs + (b.duration_minutes as u32 * 60);
                    let curr_secs = current_time.num_seconds_from_midnight();

                    curr_secs >= start_secs && curr_secs < end_secs
                });

                if let Some(block) = active_block {
                    // FOUND ACTIVE BLOCK! This schedule wins.
                    active_schedule = Some(schedule.clone());
                    
                    // Populate Block Info
                    let start = block.start_time;
                    let start_secs = start.num_seconds_from_midnight();
                    let end_secs = start_secs + (block.duration_minutes as u32 * 60);
                    let curr_secs = current_time.num_seconds_from_midnight();
                    let remaining_mins = (end_secs as i64 - curr_secs as i64) / 60;
                    
                    // Upcoming... 
                    let items: Vec<&ScheduleBlock> = blocks.iter()
                        .filter(|b| {
                            let matches_day = if let Some(d) = b.specific_date { d == current_date } 
                            else if let Some(dow) = b.day_of_week { dow == current_dow } else { false };
                            if !matches_day { return false; }
                            b.start_time > block.start_time
                        })
                        .collect::<Vec<&ScheduleBlock>>();
                    
                    let mut upcoming_refs = items;
                    upcoming_refs.sort_by_key(|b| b.start_time);
                    let upcoming_json: Vec<serde_json::Value> = upcoming_refs.into_iter().take(3).map(|b| {
                        serde_json::json!({
                           "title": format!("Block {}", b.id.unwrap_or(0)),
                           "start_time": b.start_time.to_string(),
                           "content_type": "block"
                        })
                    }).collect();

                    active_block_info = Some(serde_json::json!({
                        "block": {
                            "id": block.id,
                            "name": format!("Block {}", block.id.unwrap_or(0)), 
                            "start_time": block.start_time.to_string(),
                            "duration": block.duration_minutes,
                            "dj_id": block.dj_id
                        },
                        "time_remaining_minutes": remaining_mins,
                        "upcoming": upcoming_json
                    }));

                    // Block DJ Override
                    if let Some(blk_dj_id) = block.dj_id {
                         dj_profile_opt = dj_dsl::dj_profiles.filter(dj_dsl::id.eq(blk_dj_id)).first::<DjProfile>(&mut conn).optional()?;
                    }
                    // Block Script
                    if let Some(sid) = block.script_id {
                         if let Ok(script) = sc_dsl::scripts.filter(sc_dsl::id.eq(sid)).first::<Script>(&mut conn) {
                             active_block_script = Some(script.script_content);
                         }
                    }
                    
                    // Fallback to Schedule DJ if block didn't specify
                    if dj_profile_opt.is_none() {
                        if let Some(sched_dj_id) = schedule.dj_id {
                             dj_profile_opt = dj_dsl::dj_profiles.filter(dj_dsl::id.eq(sched_dj_id)).first::<DjProfile>(&mut conn).optional()?;
                        }
                    }
                    
                    break; // STOP here, we found our block.
                }
            }
        }

        // If after checking ALL active schedules we found nothing...
        if active_schedule.is_none() {
             // If Trigger was 'Cold Start' but NO active block matches time across ANY active schedule?
             // Then we shouldn't play anything. It's off-air time.
             if node.current_content_id.is_none() {
                tracing::info!("DEBUG: Cold start aborted - no active block found in any schedule.");
                continue;
            } else {
                // If we are currently playing content, but the block ended/no block matches:
                // We must STOP/PAUSE the node.
                tracing::info!("Node {} outside of active block. Stopping playback.", node_id);
                let nodes_map = state.connected_nodes.read().await;
                if let Some(tx) = nodes_map.get(&node_id) {
                    let cmd = NodeCommand::Pause; 
                    if let Err(e) = tx.send(ServerMessage::Command { command: cmd }) {
                        tracing::error!("Failed to send Stop/Pause: {}", e);
                    }
                }
                continue;
            }
        }

        // 3. Fallback to ANY active DJ
        if dj_profile_opt.is_none() {
            dj_profile_opt = dj_dsl::dj_profiles
                .first::<DjProfile>(&mut conn)
                .optional()?;
        }

        // Now we finalize the DJ choice
        if let Some(dj) = dj_profile_opt {
            let active_providers = ai_dsl::ai_providers
                .filter(ai_dsl::is_active.eq(true))
                .load::<AiProvider>(&mut conn)
                .unwrap_or_default();

            // Select LLM Provider
            let mut llm_provider = None;
            if let Some(pid) = dj.llm_provider_id {
                llm_provider = active_providers.iter().find(|p| p.id == Some(pid));
            }
            if llm_provider.is_none() {
                llm_provider = active_providers
                    .iter()
                    .find(|p| p.provider_type != "gemini-tts" && p.provider_type != "orpheus");
            }

            // Select TTS Provider
            let mut tts_provider = None;
            if let Some(pid) = dj.voice_provider_id {
                tts_provider = active_providers.iter().find(|p| p.id == Some(pid));
            }
            if tts_provider.is_none() {
                tts_provider = active_providers
                    .iter()
                    .find(|p| p.provider_type == "gemini-tts" || p.provider_type == "orpheus");
            }

            if let Some(provider) = llm_provider {
                // 4. Fetch Candidate Songs for Selection

                // Anti-Repeat: Get recent plays
                let recent_ids = {
                    let rp = state.recent_plays.read().await;
                    rp.iter().cloned().collect::<Vec<i32>>()
                };

                // Fetch larger pool for variety (200), ensuring RANDOM selection from DB
                let mut candidates = c_dsl::content_items
                    .filter(c_dsl::node_accessibility.eq("public"))
                    .filter(c_dsl::is_dj_accessible.eq(true))
                    // .filter(c_dsl::id.ne_all(&recent_ids)) // Diesel check might be complex with empty vec, do in memory
                    .order(diesel::dsl::sql::<diesel::sql_types::Integer>("RANDOM()"))
                    .limit(200) 
                    .load::<ContentItem>(&mut conn)
                    .unwrap_or_default();

                // Fallback
                if candidates.is_empty() {
                    candidates = c_dsl::content_items
                        .filter(c_dsl::node_accessibility.eq("public"))
                        .order(diesel::dsl::sql::<diesel::sql_types::Integer>("RANDOM()"))
                        .limit(50)
                        .load::<ContentItem>(&mut conn)
                        .unwrap_or_default();
                }

                // Filter out recently played and current song
                candidates.retain(|c| {
                    if let Some(cid) = c.id {
                         !recent_ids.contains(&cid) && cid != current_content_id_val
                    } else {
                        false
                    }
                });

                // Take top 20 (already randomized)
                candidates.truncate(20);

                if !candidates.is_empty() {
                    let candidates_clone = candidates.clone();
                    // Prepare owned values for spawn
                    let dj_clone = dj.clone();
                    let provider_clone = provider.clone();
                    let tts_provider_clone = tts_provider.cloned();
                    let state_clone = state.clone();
                    let _active_block_script_clone = active_block_script.clone();
                    
                    // Build Context String with Recent Plays
                    // We need to fetch titles for recent IDs to be useful for LLM
                    // For performance, we can just grab titles from the DB or look them up if cached.
                    // Doing a quick DB lookup for the last 5 titles
                    let recent_titles_str = if !recent_ids.is_empty() {
                         use crate::schema::content_items::dsl as c_dsl;
                         let last_5_ids = recent_ids.iter().take(5).cloned().collect::<Vec<i32>>();
                         let titles = c_dsl::content_items
                            .filter(c_dsl::id.eq_any(last_5_ids))
                            .select(c_dsl::title)
                            .load::<String>(&mut conn)
                            .unwrap_or_default();
                         
                         if !titles.is_empty() {
                             format!("Recently played tracks (Do not repeat these): {}", titles.join(", "))
                         } else {
                             String::new()
                         }
                    } else {
                        String::new()
                    };

                    let delay_ms = if remaining_seconds > 0.5 {
                        (remaining_seconds - 0.5) * 1000.0
                    } else {
                        0.0
                    } as u64;

                    let target_time =
                        tokio::time::Instant::now() + std::time::Duration::from_millis(delay_ms);

                    tracing::info!(
                        "Spawning DJ Generation Task. Remaining: {:.1}s. Injection Time: {:?} (Delay: {}ms)",
                        remaining_seconds,
                        target_time,
                        delay_ms
                    );

                    // Mark as active
                    {
                        if let Ok(mut set) = active_generations.lock() {
                            set.insert(node_id);
                        }
                    }
                    let active_generations_for_guard = active_generations.clone();
                    let last_triggered_content_clone = last_triggered_content.clone();

                    // Construct Schedule Info for context injection
                    let schedule_info = if let Some(block) = active_block_info {
                         // Only pass if we have block info
                         Some(block) 
                    } else {
                         None
                    };

                    tokio::spawn(async move {
                        // RAII Guard to remove from active set when task finishes/panics
                        let _guard = GenerationGuard {
                            node_id,
                            active_set: active_generations_for_guard
                        };

                        // 1. Generate Dialogue with Track Selection
                        let intro_prompt = if current_content_id_val == 0 {
                             "You are starting a set. Pick the first song from the list.".to_string()
                        } else {
                            format!(
                                "Current song '{}' is ending. Introduce the next song.",
                                current_song_title
                            )
                        };
                        
                        // Inject context
                        let full_prompt = if !recent_titles_str.is_empty() {
                            format!("{}\nContext: {}", intro_prompt, recent_titles_str)
                        } else {
                            intro_prompt
                        };

                        tracing::debug!("Prompt: {}", full_prompt);

                        let response = match state_clone
                            .dj_dialogue_service
                            .generate_dialogue(
                                &state_clone,
                                &dj_clone,
                                &full_prompt,
                                &provider_clone,
                                Some(&candidates_clone),
                                schedule_info, // Pass the info
                            )
                            .await
                        {
                            Ok(r) => r,
                            Err(e) => {
                                tracing::error!("Failed to generate dialogue: {}", e);
                                return;
                            }
                        };

                        // 2. Select Content
                        let selected_track_opt = response
                            .next_track_id
                            .and_then(|id| candidates_clone.iter().find(|c| c.id == Some(id)))
                            .or_else(|| {
                                tracing::warn!("LLM did not select a valid track ID. Falling back to random.");
                                candidates_clone.first()
                            });
                        
                        // 3. Prepare TTS (if speaking)
                        let mut tts_url: Option<String> = None;
                        if dj_clone.talkativeness >= 0.99 || rand::random::<f32>() <= dj_clone.talkativeness {
                             tracing::debug!("DJ Generated Dialogue: {}", response.text);
                             let output_dir = std::path::PathBuf::from("static/tts");
                             let (voice_name, speech_modifier) = if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&dj_clone.voice_config_json) {
                                 (
                                    cfg.get("voice_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                    cfg.get("speech_modifier").and_then(|v| v.as_str()).map(|s| s.to_string())
                                 )
                            } else { (None, None) };
                            
                            if let Some(prov) = tts_provider_clone {
                                match state_clone.tts_service.generate_speech(
                                    &response.text, voice_name, response.emotion, speech_modifier, &output_dir, &prov
                                ).await {
                                    Ok(path) => {
                                         let filename = path.file_name().unwrap().to_string_lossy();
                                         tts_url = Some(format!("http://{}:{}/tts/{}", state_clone.config.server.host, state_clone.config.server.port, filename));
                                    },
                                    Err(e) => tracing::error!("TTS Generation Failed: {}", e),
                                }
                            }
                        }

                        // 4. Wait for Sync
                        tokio::time::sleep_until(target_time).await;

                        // 5. Execute Transition
                        if let Some(track) = selected_track_opt {
                            let nodes_map = state_clone.connected_nodes.read().await;
                            if let Some(tx) = nodes_map.get(&node_id) {
                                let song_id = track.id.expect("Song ID missing");
                                
                                // History Tracking
                                {
                                    let mut rp = state_clone.recent_plays.write().await;
                                    rp.push_front(song_id);
                                    if rp.len() > 20 { rp.pop_back(); }
                                }

                                // Run on_load hook
                                let t_scripts_str = track.transformer_scripts.clone().unwrap_or_default();
                                let t_config = crate::services::script_service::ScriptService::parse_config_string(&t_scripts_str);
                                for cfg in &t_config {
                                     // Call on_load
                                     if let Err(e) = state_clone.script_service.call_entry_point(&state_clone, cfg, track, Some(&dj_clone), "on_load") {
                                        tracing::warn!("Failed to run on_load for content {}: {}", song_id, e);
                                     }
                                }

                                tracing::info!("Sending LoadContent for song {} (AI Selected: {})", song_id, response.next_track_id.is_some());
                                let load_cmd = NodeCommand::LoadContent {
                                    content_id: song_id,
                                    path: Some(track.content_path.clone()),
                                };
                                if let Err(e) = tx.send(ServerMessage::Command { command: load_cmd }) {
                                    tracing::error!("Failed to send LoadContent: {}", e);
                                } else {
                                    // SUCCESS: Update dedupe map to prevent immediate re-trigger
                                    // The node will briefly report "Idle" or "Loading" before "Playing"
                                    // update to (0, Now) so we hit the "Cold Start still loading" check in main loop
                                    if let Ok(mut map) = last_triggered_content_clone.lock() {
                                        map.insert(node_id, (0, Instant::now()));
                                    }
                                }
                                
                                // Inject TTS if available (plays over new track start)
                                if let Some(url) = tts_url {
                                     let inj_cmd = NodeCommand::InjectAudio { url, mix: true };
                                     let _ = tx.send(ServerMessage::Command { command: inj_cmd });
                                }
                            }
                        } else {
                             tracing::error!("No tracks available to queue!");
                        }
                    });
                }
            }
        }
    }

    Ok(())
}
