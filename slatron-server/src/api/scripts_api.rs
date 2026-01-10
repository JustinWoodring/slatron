use crate::models::{NewScript, Script, User};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ValidateScriptResponse {
    pub valid: bool,
    pub errors: Vec<String>,
}

pub async fn list_scripts(State(state): State<AppState>) -> Result<Json<Vec<Script>>, StatusCode> {
    use crate::schema::scripts::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = scripts
        .select(Script::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_script(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(new_script): Json<NewScript>,
) -> Result<Json<Script>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::scripts;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if new_script.script_type != "transformer"
        && new_script.script_type != "content_loader"
        && new_script.script_type != "server_context"
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let script = diesel::insert_into(scripts::table)
        .values(&new_script)
        .returning(Script::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(script))
}

pub async fn update_script(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(script_id): Path<i32>,
    Json(updates): Json<NewScript>,
) -> Result<Json<Script>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::scripts::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if updates.script_type != "transformer"
        && updates.script_type != "content_loader"
        && updates.script_type != "server_context"
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let script = diesel::update(scripts.filter(id.eq(script_id)))
        .set((
            name.eq(updates.name),
            description.eq(updates.description),
            script_type.eq(updates.script_type),
            script_content.eq(updates.script_content),
            parameters_schema.eq(updates.parameters_schema),
        ))
        .returning(Script::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(script))
}

pub async fn delete_script(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(script_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::scripts::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(scripts.filter(id.eq(script_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ValidateScriptRequest {
    pub script_content: String,
    pub script_type: String,
}

pub async fn validate_script(
    State(_state): State<AppState>,
    Extension(user): Extension<User>,
    Path(_script_id): Path<i32>,
    Json(req): Json<ValidateScriptRequest>,
) -> Result<Json<ValidateScriptResponse>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::rhai_engine;

    let errors = rhai_engine::validate_script(&req.script_content, &req.script_type);

    Ok(Json(ValidateScriptResponse {
        valid: errors.is_empty(),
        errors,
    }))
}

#[derive(Deserialize)]
pub struct ExecuteScriptRequest {
    pub params: serde_json::Value, // Dynamic JSON params
}

#[derive(Serialize)]
pub struct ExecuteScriptResponse {
    pub success: bool,
    pub result: Option<String>,
    pub mpv_commands: Vec<String>,
    pub error: Option<String>,
}

pub async fn execute_script(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(script_id): Path<i32>,
    Json(req): Json<ExecuteScriptRequest>,
) -> Result<Json<ExecuteScriptResponse>, StatusCode> {
    tracing::info!("Request to execute script ID: {}", script_id);

    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::rhai_engine;
    use crate::schema::global_settings::dsl::global_settings;
    use crate::schema::scripts::dsl::*;

    let mut conn = state.db.get().map_err(|e| {
        tracing::error!("Failed to get DB connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let script = scripts
        .filter(id.eq(script_id))
        .select(Script::as_select())
        .first(&mut conn)
        .map_err(|e| {
            tracing::error!("Failed to fetch script {}: {}", script_id, e);
            StatusCode::NOT_FOUND
        })?;

    // Fetch Global Settings
    let settings_list = global_settings
        .load::<crate::models::GlobalSetting>(&mut conn)
        .map_err(|e| {
            tracing::error!("Failed to fetch global settings: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut settings_map = std::collections::HashMap::new();
    for s in settings_list {
        settings_map.insert(s.key, s.value);
    }

    // Convert serde_json::Value to rhai::Map
    let mut rhai_params = rhai::Map::new();
    if let serde_json::Value::Object(map) = req.params {
        for (k, v) in map {
            rhai_params.insert(k.into(), rhai::serde::to_dynamic(v).unwrap_or_default());
        }
    }

    // Verify script content
    tracing::info!("Script Content Preview: {:.100}...", script.script_content);

    match rhai_engine::execute_script(
        &script.script_content,
        &script.script_type,
        rhai_params,
        settings_map,
    ) {
        Ok((result, commands)) => {
            // Convert to valid JSON string instead of Rhai debug string
            let result_json = rhai::serde::from_dynamic::<serde_json::Value>(&result)
                .map(|v| v.to_string())
                .unwrap_or_else(|_| result.to_string());

            tracing::info!("Script execution successful. Result: {}", result_json);
            Ok(Json(ExecuteScriptResponse {
                success: true,
                result: Some(result_json),
                mpv_commands: commands,
                error: None,
            }))
        }
        Err(e) => Ok(Json(ExecuteScriptResponse {
            success: false,
            result: None,
            mpv_commands: vec![],
            error: Some(e),
        })),
    }
}
