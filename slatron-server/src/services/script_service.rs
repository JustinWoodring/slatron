use anyhow::Result;
use chrono::{Datelike, Local, Timelike};
use diesel::prelude::*;
use rhai::{Dynamic, Engine, Scope};
use std::sync::Arc;

use crate::models::{ContentItem, DjProfile, Schedule, ScheduleBlock};
use crate::AppState;

#[derive(Debug, Clone)]
pub struct ScriptExecutionConfig {
    pub script_id: i32,
    pub params: serde_json::Value,
}

#[derive(Clone)]
pub struct ScriptService {
    engine: Arc<Engine>,
}

impl ScriptService {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register global helpers available to all server scripts

        // 1. Time Helper (Legacy - uses Local machine time)
        engine.register_fn("get_local_time", || -> String {
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        });

        // 2. Advanced Time Helper: get_time(format, timezone)
        engine.register_fn("get_time", |fmt: &str, tz_str: &str| -> String {
            match tz_str.parse::<chrono_tz::Tz>() {
                Ok(tz) => {
                    let now = chrono::Utc::now().with_timezone(&tz);
                    now.format(fmt).to_string()
                }
                Err(_) => {
                    let now = chrono::Utc::now();
                    format!(
                        "Error: Invalid Timezone '{}' (Using UTC: {})",
                        tz_str,
                        now.format(fmt)
                    )
                }
            }
        });

        // 3. Formatted Date Time Helper (User requested)
        engine.register_fn("get_date_time", |fmt: &str| -> String {
            // Uses server local time
            Local::now().format(fmt).to_string()
        });

        // 4. HTTP Helper (Synchronous/Blocking)
        engine.register_fn("http_get", |url: &str| -> String {
            match reqwest::blocking::get(url) {
                Ok(resp) => resp
                    .text()
                    .unwrap_or_else(|e| format!("Error reading body: {}", e)),
                Err(e) => format!("Error fetching URL: {}", e),
            }
        });

        // 4. Log Helper
        engine.register_fn("log_info", |msg: &str| {
            tracing::info!("[SCRIPT] {}", msg);
        });

        // 5. XML Helper (Custom Parser for List Handling)
        engine.register_fn("parse_xml", |xml: &str| -> Dynamic {
            let v = ScriptService::parse_xml_to_value(xml);
            rhai::serde::to_dynamic(&v).unwrap_or(Dynamic::UNIT)
        });

        Self {
            engine: Arc::new(engine),
        }
    }

    // Helper for robust XML parsing to JSON (Preserves lists)
    fn parse_xml_to_value(xml: &str) -> serde_json::Value {
        use quick_xml::events::Event;
        use quick_xml::reader::Reader;
        use serde_json::{Map, Value};

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        // Stack of (Tag Name, Map of Children)
        // We start with a dummy root to capture the top-level element
        let mut stack: Vec<(String, Map<String, Value>)> = Vec::new();
        stack.push(("ROOT".to_string(), Map::new()));

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    let mut map = Map::new();
                    // Capture attributes
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let val = String::from_utf8_lossy(&attr.value).into_owned();
                            let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                            map.insert(format!("@{}", key), Value::String(val));
                        }
                    }
                    stack.push((name, map));
                }
                Ok(Event::End(_)) => {
                    if stack.len() > 1 {
                        let (name, map) = stack.pop().unwrap();
                        let value = Value::Object(map);

                        if let Some((_, parent_map)) = stack.last_mut() {
                            if let Some(existing) = parent_map.get_mut(&name) {
                                if let Value::Array(arr) = existing {
                                    arr.push(value);
                                } else {
                                    let old = existing.clone();
                                    *existing = Value::Array(vec![old, value]);
                                }
                            } else {
                                parent_map.insert(name, value);
                            }
                        }
                    }
                }
                Ok(Event::Empty(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    let mut map = Map::new();
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let val = String::from_utf8_lossy(&attr.value).into_owned();
                            let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                            map.insert(format!("@{}", key), Value::String(val));
                        }
                    }
                    let value = Value::Object(map);

                    if let Some((_, parent_map)) = stack.last_mut() {
                        if let Some(existing) = parent_map.get_mut(&name) {
                            if let Value::Array(arr) = existing {
                                arr.push(value);
                            } else {
                                let old = existing.clone();
                                *existing = Value::Array(vec![old, value]);
                            }
                        } else {
                            parent_map.insert(name, value);
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().into_owned();
                    if !text.is_empty() {
                        if let Some((_, map)) = stack.last_mut() {
                            if let Some(Value::String(s)) = map.get_mut("$text") {
                                s.push_str(&text);
                            } else {
                                map.insert("$text".to_string(), Value::String(text));
                            }
                        }
                    }
                }
                Ok(Event::CData(e)) => {
                    let text = String::from_utf8_lossy(&e).into_owned();
                    if !text.is_empty() {
                        if let Some((_, map)) = stack.last_mut() {
                            if let Some(Value::String(s)) = map.get_mut("$text") {
                                s.push_str(&text);
                            } else {
                                map.insert("$text".to_string(), Value::String(text));
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => (),
            }
            buf.clear();
        }

        if let Some((_, root)) = stack.pop() {
            Value::Object(root)
        } else {
            Value::Null
        }
    }

    /// Helper to parse script configuration string (JSON or CSV)
    pub fn parse_config_string(config_str: &str) -> Vec<ScriptExecutionConfig> {
        if config_str.trim().starts_with('[') {
            #[derive(serde::Deserialize)]
            struct JsonScriptConfig {
                id: i32,
                params: Option<serde_json::Value>,
            }

            match serde_json::from_str::<Vec<JsonScriptConfig>>(config_str) {
                Ok(configs) => configs
                    .into_iter()
                    .map(|c| ScriptExecutionConfig {
                        script_id: c.id,
                        params: c.params.unwrap_or(serde_json::Value::Null),
                    })
                    .collect(),
                Err(e) => {
                    tracing::warn!("Failed to parse script JSON: {}. Falling back to empty.", e);
                    vec![]
                }
            }
        } else {
            // Legacy CSV parsing
            config_str
                .split(',')
                .filter_map(|s| s.trim().parse::<i32>().ok())
                .map(|id| ScriptExecutionConfig {
                    script_id: id,
                    params: serde_json::Value::Null,
                })
                .collect()
        }
    }

    /// Executed one or more context scripts in order.
    /// Accumulates their output into the final prompt string.
    pub fn run_context_scripts(
        &self,
        state: &AppState,
        script_configs: Vec<ScriptExecutionConfig>,
        dj_profile: &DjProfile,
        content_item: Option<&ContentItem>,
        server_tz_setting: String,
        schedule_info_arg: Option<serde_json::Value>, // { block: ..., upcoming: ... }
    ) -> Result<String> {
        use crate::schema::scripts::dsl::*;

        if script_configs.is_empty() {
            return Ok(String::new());
        }

        let mut conn = state.db.get()?;

        // Extract IDs for fetching
        let target_ids: Vec<i32> = script_configs.iter().map(|c| c.script_id).collect();

        // Fetch Scripts
        let context_scripts = scripts
            .filter(id.eq_any(target_ids))
            .load::<crate::models::Script>(&mut conn)?;

        if context_scripts.is_empty() {
            return Ok(String::new());
        }

        // Map configs for easy param lookup
        let config_map: std::collections::HashMap<i32, serde_json::Value> = script_configs
            .iter()
            .map(|c| (c.script_id, c.params.clone()))
            .collect();

        // Map scripts for lookup by ID
        let scripts_map: std::collections::HashMap<i32, crate::models::Script> = context_scripts
            .into_iter()
            .map(|s| (s.id.unwrap_or(0), s))
            .collect();

        // Convert Objects to Dynamic once
        let dj_dynamic: Dynamic = rhai::serde::to_dynamic(dj_profile)?;

        let item_dynamic: Dynamic = if let Some(item) = content_item {
            rhai::serde::to_dynamic(item)?
        } else {
            Dynamic::UNIT
        };

        // Resolve Schedule Info: Use arg if provided (Test Mode), else Fetch from DB (Live Mode)
        let schedule_dynamic = if let Some(info) = schedule_info_arg {
            rhai::serde::to_dynamic(info).unwrap_or(Dynamic::UNIT)
        } else {
            // Fetch active schedule block and upcoming context
            match fetch_schedule_context(&mut conn, dj_profile.id, &server_tz_setting) {
                Ok(info) => rhai::serde::to_dynamic(info).unwrap_or(Dynamic::UNIT),
                Err(e) => {
                    tracing::error!("Failed to fetch schedule context: {}", e);
                    Dynamic::UNIT
                }
            }
        };

        // Prepare Output
        let mut final_context = String::new();

        for config in script_configs {
            let script_id = config.script_id;
            let script = match scripts_map.get(&script_id) {
                Some(s) => s,
                None => continue,
            };
            let mut scope = Scope::new();

            // Standard Context
            scope.push("context", String::new()); // output accumulator for THIS script? or previous?
                                                  // Usually context scripts return a string which is appended.
                                                  // In some designs, 'context' variable holds previous context.
                                                  // Let's assume clear scope for now but inject global vars.

            scope.push("server_timezone", server_tz_setting.clone());

            // Inject Objects
            scope.push("dj", dj_dynamic.clone());
            scope.push("content_item", item_dynamic.clone());
            scope.push("schedule", schedule_dynamic.clone());

            // Inject Parameters
            if let Some(params_value) = config_map.get(&script.id.unwrap_or(0)) {
                let params_dynamic: Dynamic = rhai::serde::to_dynamic(params_value)?;
                scope.push("params", params_dynamic);
            } else {
                scope.push("params", Dynamic::UNIT);
            }

            // Compile & Run
            match self.engine.compile(&script.script_content) {
                Ok(ast) => {
                    // We expect the script to return a string or modify "context" variable?
                    // Previous implementation seemed to verify if it returns string.
                    // Or maybe it pushes to 'context' var?
                    // "scope.push("context", String::new())" suggests it might modify it.
                    // But `let ctx = scope.get_value...` in `test_script`.
                    // So let's capture the 'context' variable after run.

                    if let Err(e) = self.engine.run_ast_with_scope(&mut scope, &ast) {
                        tracing::error!("Error running context script {}: {}", script.name, e);
                    } else {
                        // Extract output from 'context' variable
                        let script_output =
                            scope.get_value::<String>("context").unwrap_or_default();
                        if !script_output.trim().is_empty() {
                            final_context.push_str(&script_output);
                            final_context.push('\n');
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error compiling context script {}: {}", script.name, e);
                }
            }
        }

        Ok(final_context)
    }

    /// Executes "transformer" scripts for a ContentItem.
    /// Injects `content_item`, `dj` (if available - actually DjProfile info might be needed logic?),
    /// and `params`.
    ///
    /// Transformer scripts typically modify the description or metadata string, or just return an analysis string.
    /// For now, let's assume they return a string context similar to above, or maybe they modify the item?
    /// The user request implies "user defined set of parameters" for both types.
    /// Let's assume transformers also output to a string buffer for now, to be appended to the track context.
    pub fn run_transformer_scripts(
        &self,
        state: &AppState,
        content_item: &ContentItem,
        scripts_config: Vec<ScriptExecutionConfig>,
    ) -> Result<String> {
        use crate::schema::global_settings::dsl as gs;
        use crate::schema::scripts::dsl::*;

        if scripts_config.is_empty() {
            // Default behavior or return empty?
            return Ok(String::new());
        }

        let mut conn = state
            .db
            .get()
            .map_err(|e| anyhow::anyhow!("DB Connection failed: {}", e))?;

        let server_tz_setting: String = gs::global_settings
            .filter(gs::key.eq("server_timezone"))
            .select(gs::value)
            .first(&mut conn)
            .optional()?
            .unwrap_or_else(|| "UTC".to_string());

        let target_ids: Vec<i32> = scripts_config.iter().map(|c| c.script_id).collect();
        let transformer_scripts = scripts
            .filter(id.eq_any(target_ids))
            .load::<crate::models::Script>(&mut conn)?;

        if transformer_scripts.is_empty() {
            return Ok(String::new());
        }

        let config_map: std::collections::HashMap<i32, serde_json::Value> = scripts_config
            .into_iter()
            .map(|c| (c.script_id, c.params))
            .collect();

        // Convert ContentItem to Dynamic
        let item_dynamic: Dynamic = rhai::serde::to_dynamic(content_item)?;

        let mut final_output = String::new();

        for script in transformer_scripts {
            let mut scope = Scope::new();

            // Push 'output' variable for the script to write to
            scope.push("output", String::new());
            // Allow reading existing description?
            scope.push(
                "description",
                content_item.description.clone().unwrap_or_default(),
            );

            // Standard Context
            scope.push("server_timezone", server_tz_setting.clone());

            // Inject Item
            scope.push("content_item", item_dynamic.clone());

            // Inject Parameters
            if let Some(params_value) = config_map.get(&script.id.unwrap_or(0)) {
                let params_dynamic: Dynamic = rhai::serde::to_dynamic(params_value)?;
                scope.push("params", params_dynamic);
            } else {
                scope.push("params", Dynamic::UNIT);
            }

            match self.engine.compile(&script.script_content) {
                Ok(ast) => {
                    if let Err(e) = self.engine.run_ast_with_scope(&mut scope, &ast) {
                        tracing::error!("Error running transformer script {}: {}", script.name, e);
                    } else {
                        let out = scope.get_value::<String>("output").unwrap_or_default();
                        if !out.is_empty() {
                            final_output.push_str(&out);
                            final_output.push('\n');
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to compile script {}: {}", script.name, e);
                }
            }
        }

        Ok(final_output)
    }
    /// Executes a specific entry point function (e.g. "on_load") in a script.
    /// Injects `content_item`, `dj` (optional), and `params` into the scope.
    pub fn call_entry_point(
        &self,
        state: &AppState,
        script_config: &ScriptExecutionConfig,
        content_item: &ContentItem,
        dj_profile: Option<&DjProfile>,
        entry_point: &str,
    ) -> Result<()> {
        use crate::schema::global_settings::dsl as gs;
        use crate::schema::scripts::dsl::*;

        let mut conn = state
            .db
            .get()
            .map_err(|e| anyhow::anyhow!("DB Connection failed: {}", e))?;

        // Fetch Script Content
        let script = scripts
            .filter(id.eq(script_config.script_id))
            .first::<crate::models::Script>(&mut conn)?;

        // Fetch Global Timezone
        let server_tz_setting: String = gs::global_settings
            .filter(gs::key.eq("server_timezone"))
            .select(gs::value)
            .first(&mut conn)
            .optional()?
            .unwrap_or_else(|| "UTC".to_string());

        let mut scope = Scope::new();

        // 1. Inject Standard Context
        scope.push("server_timezone", server_tz_setting);

        // 2. Inject Content Item
        let item_dynamic: Dynamic = rhai::serde::to_dynamic(content_item)?;
        scope.push("content_item", item_dynamic);

        // 3. Inject DJ Profile (if available)
        if let Some(dj) = dj_profile {
            let dj_dynamic: Dynamic = rhai::serde::to_dynamic(dj)?;
            scope.push("dj", dj_dynamic);
        }

        // 4. Inject Parameters
        let params_dynamic: Dynamic = rhai::serde::to_dynamic(&script_config.params)?;
        scope.push("params", params_dynamic);

        // 5. Compile & Call
        let ast = self.engine.compile(&script.script_content)?;

        // Check if function exists
        let has_fn = ast
            .iter_functions()
            .any(|f| f.name == entry_point && f.params.is_empty());

        if has_fn {
            // Call the function with no arguments (it uses scope)
            let _result: Dynamic = self.engine.call_fn(&mut scope, &ast, entry_point, ())?;
            tracing::info!("Executed hook '{}' for script {}", entry_point, script.name);
        } else {
            // It's acceptable for the hook to be missing
            tracing::debug!(
                "Hook '{}' not found in script {}, skipping.",
                entry_point,
                script.name
            );
        }

        Ok(())
    }

    /// Executed ad-hoc script for testing purposes.
    /// Injects mock data for `dj` and `content_item` to simulate real execution.
    pub fn test_script(
        &self,
        script_content: &str,
        script_type: &str,
        params: serde_json::Value,
    ) -> Result<String> {
        let mut scope = Scope::new();

        // 1. Standard Context
        // We don't have DB access conveniently here for global settings without passing state...
        // For testing, let's default to UTC or 'Local'.
        scope.push("server_timezone", "UTC".to_string());
        scope.push("context", String::new()); // For server_context scripts
        scope.push("output", String::new()); // For transformer scripts

        // 2. Inject Mock Parameters
        let params_dynamic: Dynamic = rhai::serde::to_dynamic(params)?;
        scope.push("params", params_dynamic);

        // 3. Inject Mock DJ Profile
        let mock_dj = crate::models::DjProfile {
            id: Some(999),
            name: "Test DJ".to_string(),
            personality_prompt: "You are a witty radio host.".to_string(),
            voice_config_json: "{}".to_string(),
            context_depth: 5,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            voice_provider_id: None,
            llm_provider_id: None,
            context_script_ids: None,
            talkativeness: 0.5,
        };
        let dj_dynamic: Dynamic = rhai::serde::to_dynamic(mock_dj)?;
        scope.push("dj", dj_dynamic);

        // 4. Inject Mock Content Item
        let mock_item = crate::models::ContentItem {
            id: Some(101),
            title: "Test Track Title".to_string(),
            description: Some("Current description text.".to_string()),
            content_type: "music".to_string(),
            content_path: "/path/to/test.mp3".to_string(),
            adapter_id: None,
            duration_minutes: Some(3), // i32
            tags: Some("test,mock".to_string()),
            node_accessibility: Some("public".to_string()),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            transformer_scripts: None,
            is_dj_accessible: true,
        };
        let item_dynamic: Dynamic = rhai::serde::to_dynamic(mock_item)?;
        scope.push("content_item", item_dynamic);

        // 5. Inject Mock Schedule
        let mock_schedule = serde_json::json!({
            "block": {
                "id": 1,
                "name": "Morning Drive",
                "start_time": "06:00:00",
                "end_time": "10:00:00",
                "dj_id": 999
            },
            "upcoming": [
                { "title": "Next Song Title", "content_type": "music" }
            ],
            "time_remaining_minutes": 45
        });
        let schedule_dynamic: Dynamic = rhai::serde::to_dynamic(mock_schedule)?;
        scope.push("schedule", schedule_dynamic);

        // 6. Compile & Run
        match self.engine.compile(script_content) {
            Ok(ast) => {
                match self.engine.run_ast_with_scope(&mut scope, &ast) {
                    Ok(_) => {
                        // Check result based on type behavior
                        if script_type == "server_context" {
                            let ctx = scope.get_value::<String>("context").unwrap_or_default();
                            return Ok(ctx);
                        } else if script_type == "transformer" {
                            let out = scope.get_value::<String>("output").unwrap_or_default();
                            return Ok(out);
                        } else if script_type == "content_loader" {
                            // Let's re-run with eval to capture return.
                            let result = self
                                .engine
                                .eval_ast_with_scope::<Dynamic>(&mut scope, &ast)
                                .map_err(|e| anyhow::anyhow!("Runtime Error: {}", e))?;
                            return Ok(rhai::serde::from_dynamic::<serde_json::Value>(&result)
                                .map(|v| v.to_string())
                                .unwrap_or_else(|_| result.to_string()));
                        }

                        Ok("Script executed successfully (No string output captured)".to_string())
                    }
                    Err(e) => Err(anyhow::anyhow!("Runtime Error: {}", e)),
                }
            }
            Err(e) => Err(anyhow::anyhow!("Compilation Error: {}", e)),
        }
    }
}

// Helper to fetch the active schedule block and upcoming context
fn fetch_schedule_context(
    conn: &mut crate::db::DbConnection,
    dj_profile_id: Option<i32>,
    tz_setting: &str,
) -> Result<serde_json::Value> {
    use crate::schema::{dj_profiles, schedule_blocks, schedules};

    let dj_id_val = match dj_profile_id {
        Some(id) => id,
        None => return Ok(serde_json::json!({})),
    };

    // Calculate current time/date
    let tz: chrono_tz::Tz = tz_setting.parse().unwrap_or(chrono_tz::UTC);
    let now = chrono::Utc::now().with_timezone(&tz);

    let current_date = now.date_naive();
    let current_time = now.time();
    let current_dow = current_date.weekday().num_days_from_monday() as i32;

    let yesterday_date = current_date.pred_opt().unwrap();
    let yesterday_dow = yesterday_date.weekday().num_days_from_monday() as i32;

    // --- 1. Active Block Logic ---
    let mut active_block_json = serde_json::Value::Null;
    let mut time_remaining = serde_json::Value::Null;

    let active_candidates: Vec<(ScheduleBlock, Schedule)> = schedule_blocks::table
        .inner_join(schedules::table)
        .filter(schedules::is_active.eq(true))
        .filter(
            (schedule_blocks::dj_id.eq(dj_id_val)).or(schedule_blocks::dj_id
                .is_null()
                .and(schedules::dj_id.eq(dj_id_val))),
        )
        .filter(
            (schedules::schedule_type
                .eq("weekly")
                .and(schedule_blocks::day_of_week.eq_any(vec![current_dow, yesterday_dow])))
            .or(schedules::schedule_type
                .eq("one_off")
                .and(schedule_blocks::specific_date.eq_any(vec![current_date, yesterday_date]))),
        )
        .select((ScheduleBlock::as_select(), Schedule::as_select()))
        .load(conn)?;

    for (block, schedule) in active_candidates {
        let is_match = match schedule.schedule_type.as_str() {
            "weekly" => {
                if block.day_of_week == Some(current_dow) {
                    is_time_match(block.start_time, block.duration_minutes, current_time, 0)
                } else if block.day_of_week == Some(yesterday_dow) {
                    is_time_match(block.start_time, block.duration_minutes, current_time, 1)
                } else {
                    false
                }
            }
            "one_off" => {
                if block.specific_date == Some(current_date) {
                    is_time_match(block.start_time, block.duration_minutes, current_time, 0)
                } else if block.specific_date == Some(yesterday_date) {
                    is_time_match(block.start_time, block.duration_minutes, current_time, 1)
                } else {
                    false
                }
            }
            _ => false,
        };

        if is_match {
            active_block_json = serde_json::json!({
                "id": block.id,
                "name": schedule.name,
                "start_time": block.start_time.format("%H:%M:%S").to_string(),
                "duration_minutes": block.duration_minutes,
                "dj_id": dj_id_val,
                "schedule_id": schedule.id,
            });

            let day_offset =
                if block.day_of_week == Some(yesterday_dow) || block.specific_date == Some(yesterday_date) {
                    1
                } else {
                    0
                };

            time_remaining = serde_json::json!(calculate_remaining_minutes(
                block.start_time,
                block.duration_minutes,
                current_time,
                day_offset
            ));
            break; // Found the active one
        }
    }

    // --- 2. Upcoming Blocks Logic ---
    // Fetch next 5 active blocks starting after current time today
    // We join with dj_profiles (left join) to get DJ Name if available.
    // Note: dj_profiles::dsl::id is nullable if we join it?
    // Actually, schedule_blocks.dj_id is nullable. schedule.dj_id is nullable.
    // Ideally we want the "effective DJ name".
    // Doing complex effective DJ resolution in SQL with Diesel left joins can be verbose.
    // We will just fetch (Block, Schedule) and then map DJ names in memory or simple join if possible.
    // Let's stick to (Block, Schedule) for now and maybe fetch DJ names separately if needed,
    // or just return DJ ID. The prompt implies "DJ might know about other stuff", so names are useful.
    // Let's try to left join `dj_profiles` on `schedule_blocks.dj_id`.

    // Simple query first: Upcoming blocks today.
    // Ignoring "one_off" vs "weekly" logic for future dates for simplicity - just "today".
    // Logic:
    // (Weekly & DOW=Today & Start > Now) OR (OneOff & Date=Today & Start > Now)
    // Order by StartTime ASC Limit 5.

    let upcoming_data: Vec<(ScheduleBlock, Schedule)> = schedule_blocks::table
        .inner_join(schedules::table)
        .filter(schedules::is_active.eq(true))
        .filter(schedule_blocks::start_time.gt(current_time))
        .filter(
            (schedules::schedule_type
                .eq("weekly")
                .and(schedule_blocks::day_of_week.eq(current_dow)))
            .or(schedules::schedule_type
                .eq("one_off")
                .and(schedule_blocks::specific_date.eq(current_date))),
        )
        .order(schedule_blocks::start_time.asc())
        .limit(5)
        .select((ScheduleBlock::as_select(), Schedule::as_select()))
        .load(conn)?;

    let mut upcoming_blocks_json = Vec::new();

    // Optimization: Collect DJ IDs to fetch names
    let mut dj_ids_to_fetch = Vec::new();
    for (b, s) in &upcoming_data {
        if let Some(did) = b.dj_id { dj_ids_to_fetch.push(did); }
        else if let Some(did) = s.dj_id { dj_ids_to_fetch.push(did); }
    }
    dj_ids_to_fetch.sort();
    dj_ids_to_fetch.dedup();

    let mut dj_map = std::collections::HashMap::new();
    if !dj_ids_to_fetch.is_empty() {
        let djs = dj_profiles::table
            .filter(dj_profiles::id.eq_any(dj_ids_to_fetch))
            .select((dj_profiles::id, dj_profiles::name))
            .load::<(Option<i32>, String)>(conn)?;
        for (id, name) in djs {
            if let Some(i) = id {
                dj_map.insert(i, name);
            }
        }
    }

    for (block, schedule) in upcoming_data {
        let effective_dj_id = block.dj_id.or(schedule.dj_id);
        let dj_name = effective_dj_id.and_then(|id| dj_map.get(&id).cloned());

        upcoming_blocks_json.push(serde_json::json!({
            "name": schedule.name,
            "start_time": block.start_time.format("%H:%M:%S").to_string(),
            "duration_minutes": block.duration_minutes,
            "dj_name": dj_name,
        }));
    }

    Ok(serde_json::json!({
        "block": active_block_json,
        "time_remaining_minutes": time_remaining,
        "upcoming_blocks": upcoming_blocks_json
    }))
}

fn is_time_match(
    start: chrono::NaiveTime,
    duration_min: i32,
    now: chrono::NaiveTime,
    day_offset: i32, // 0 = same day, 1 = start was yesterday
) -> bool {
    let start_secs = start.num_seconds_from_midnight();
    let now_secs = now.num_seconds_from_midnight();
    let duration_secs = (duration_min as u32) * 60;

    if day_offset == 0 {
        // Starts today.
        // Match if now >= start AND now < start + duration
        // Note: start + duration can exceed 24h (86400), but now_secs is always < 86400.
        // So we just check strictly.
        // If duration wraps midnight, it continues to next day (which is covered by day_offset=1 check tomorrow)
        // But for "Today", we only care if we are in the portion that is Today.
        // Wait, if start=23:00, dur=120 (2 hours), it ends 01:00 tomorrow.
        // If now=23:30, it matches.
        let end_secs = start_secs + duration_secs;
        now_secs >= start_secs && now_secs < end_secs
    } else {
        // Starts yesterday.
        // Match if now < (start + duration - 24h)
        // i.e. spilled over part.
        let end_secs = start_secs + duration_secs;
        if end_secs > 86400 {
            // It spills over
            let spill_secs = end_secs - 86400;
            now_secs < spill_secs
        } else {
            false
        }
    }
}

fn calculate_remaining_minutes(
    start: chrono::NaiveTime,
    duration_min: i32,
    now: chrono::NaiveTime,
    day_offset: i32,
) -> i32 {
    let start_secs = start.num_seconds_from_midnight();
    let now_secs = now.num_seconds_from_midnight();
    let duration_secs = (duration_min as u32) * 60;

    let end_secs_absolute = start_secs + duration_secs; // Relative to start day midnight
    let now_secs_absolute = if day_offset == 1 {
        now_secs + 86400
    } else {
        now_secs
    };

    if now_secs_absolute < end_secs_absolute {
        ((end_secs_absolute - now_secs_absolute) / 60) as i32
    } else {
        0
    }
}
