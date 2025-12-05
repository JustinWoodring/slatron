use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::{NewScript, Script};
use crate::AppState;

#[derive(Serialize)]
pub struct ValidateScriptResponse {
    pub valid: bool,
    pub errors: Vec<String>,
}

pub async fn list_scripts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Script>>, StatusCode> {
    use crate::schema::scripts::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = scripts
        .select(Script::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_script(
    State(state): State<AppState>,
    Json(new_script): Json<NewScript>,
) -> Result<Json<Script>, StatusCode> {
    use crate::schema::scripts;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let script = diesel::insert_into(scripts::table)
        .values(&new_script)
        .returning(Script::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(script))
}

pub async fn update_script(
    State(state): State<AppState>,
    Path(script_id): Path<i32>,
    Json(updates): Json<NewScript>,
) -> Result<Json<Script>, StatusCode> {
    use crate::schema::scripts::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let script = diesel::update(scripts.find(script_id))
        .set((
            name.eq(updates.name),
            description.eq(updates.description),
            script_type.eq(updates.script_type),
            script_content.eq(updates.script_content),
            parameters_schema.eq(updates.parameters_schema),
        ))
        .returning(Script::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(script))
}

pub async fn delete_script(
    State(state): State<AppState>,
    Path(script_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    use crate::schema::scripts::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(scripts.find(script_id))
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
    Path(_script_id): Path<i32>,
    Json(req): Json<ValidateScriptRequest>,
) -> Result<Json<ValidateScriptResponse>, StatusCode> {
    use crate::rhai_engine;

    let errors = rhai_engine::validate_script(&req.script_content, &req.script_type);

    Ok(Json(ValidateScriptResponse {
        valid: errors.is_empty(),
        errors,
    }))
}
