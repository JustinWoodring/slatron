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

        // 3. HTTP Helper (Synchronous/Blocking)
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

        Self {
            engine: Arc::new(engine),
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

    /// Executes "server_context" scripts for a DJ Profile.
    /// Injects `dj` (profile data) and `params` (per-script).
    pub fn run_context_scripts(
        &self,
        state: &AppState,
        dj_profile: &DjProfile,
        scripts_config: Vec<ScriptExecutionConfig>,
    ) -> Result<String> {
        use crate::schema::global_settings::dsl as gs;
        use crate::schema::scripts::dsl::*;

        if scripts_config.is_empty() {
            return Ok(String::new());
        }

        let mut conn = state
            .db
            .get()
            .map_err(|e| anyhow::anyhow!("DB Connection failed: {}", e))?;

        // Fetch Global Timezone Setting
        let server_tz_setting: String = gs::global_settings
            .filter(gs::key.eq("server_timezone"))
            .select(gs::value)
            .first(&mut conn)
            .optional()?
            .unwrap_or_else(|| "UTC".to_string());

        // Extract IDs for fetching
        let target_ids: Vec<i32> = scripts_config.iter().map(|c| c.script_id).collect();

        // Fetch Scripts
        let context_scripts = scripts
            .filter(id.eq_any(target_ids))
            .load::<crate::models::Script>(&mut conn)?;

        if context_scripts.is_empty() {
            return Ok(String::new());
        }

        // Map configs for easy param lookup
        let config_map: std::collections::HashMap<i32, serde_json::Value> = scripts_config
            .into_iter()
            .map(|c| (c.script_id, c.params))
            .collect();

        // Convert DjProfile to Dynamic Map for Rhai
        let dj_dynamic: Dynamic = rhai::serde::to_dynamic(dj_profile)?;

        // Prepare Output
        let mut final_context = String::new();

        // Run Scripts (in order of IDs usually, or we should re-order by input list order?
        // DB returns unsorted usually or by ID. Let's rely on default sort for now or simpler logic.
        // Actually, users might expect order to matter.
        // Let's re-order `context_scripts` to match `target_ids` order if possible,
        // but for now let's just iterate fetched scripts.

        for script in context_scripts {
            let mut scope = Scope::new();

            // Standard Context
            scope.push("context", String::new());
            scope.push("server_timezone", server_tz_setting.clone());

            // Inject User Data
            scope.push("dj", dj_dynamic.clone());

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
                    if let Err(e) = self.engine.run_ast_with_scope(&mut scope, &ast) {
                        tracing::error!("Error running context script {}: {}", script.name, e);
                        // Append error to context to inform AI? Or just log?
                        // Just log for now.
                    } else {
                        // Extract output
                        let script_output =
                            scope.get_value::<String>("context").unwrap_or_default();
                        if !script_output.is_empty() {
                            final_context
                                .push_str(&format!("\n[{}]\n{}\n", script.name, script_output));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to compile script {}: {}", script.name, e);
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
}
