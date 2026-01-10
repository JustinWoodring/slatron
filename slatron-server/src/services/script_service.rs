use anyhow::Result;
use chrono::Local;
use diesel::prelude::*;
use rhai::{Dynamic, Engine, Scope};
use std::sync::Arc;

use crate::models::{ContentItem, DjProfile};
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
            // TODO: Fetch active schedule block for this DJ from DB
            // For now, defaulting to UNIT until we implement the query logic or helper
            Dynamic::UNIT
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
