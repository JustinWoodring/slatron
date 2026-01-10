use crate::models::{
    AiProvider, DjMemory, DjProfile, NewAiProvider, NewDjMemory, NewDjProfile, UpdateDjMemory,
};
use crate::schema::{ai_providers, dj_memories, dj_profiles};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;

// DJ Profile Handlers

pub async fn list_djs(
    State(state): State<AppState>,
) -> Result<Json<Vec<DjProfile>>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let results = dj_profiles::table
        .load::<DjProfile>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(results))
}

pub async fn create_dj(
    State(state): State<AppState>,
    Json(payload): Json<NewDjProfile>,
) -> Result<Json<DjProfile>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = diesel::insert_into(dj_profiles::table)
        .values(&payload)
        .returning(DjProfile::as_returning())
        .get_result(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

pub async fn get_dj(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<DjProfile>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = dj_profiles::table
        .filter(dj_profiles::id.eq(id))
        .first::<DjProfile>(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "DJ Profile not found".to_string()))?;

    Ok(Json(result))
}

pub async fn delete_dj(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    diesel::delete(dj_profiles::table.filter(dj_profiles::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::dj_profiles)]
pub struct UpdateDjProfile {
    pub name: Option<String>,
    pub personality_prompt: Option<String>,
    pub voice_config_json: Option<String>,
    pub context_depth: Option<i32>,
    pub voice_provider_id: Option<Option<i32>>, // Handle explicit null
    pub llm_provider_id: Option<Option<i32>>,
    pub context_script_ids: Option<Option<String>>,
    pub talkativeness: Option<f32>,
}

pub async fn update_dj(
    State(state): State<AppState>,
    Path(dj_id): Path<i32>,
    Json(payload): Json<UpdateDjProfile>,
) -> Result<Json<DjProfile>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    use crate::schema::dj_profiles::dsl::*;

    let target = dj_profiles.filter(id.eq(dj_id));

    let result = diesel::update(target)
        .set((&payload, updated_at.eq(chrono::Utc::now().naive_utc())))
        .returning(DjProfile::as_returning())
        .get_result(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

// AI Provider Handlers

pub async fn list_ai_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<AiProvider>>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let results = ai_providers::table
        .load::<AiProvider>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(results))
}

pub async fn create_ai_provider(
    State(state): State<AppState>,
    Json(payload): Json<NewAiProvider>,
) -> Result<Json<AiProvider>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = diesel::insert_into(ai_providers::table)
        .values(&payload)
        .returning(AiProvider::as_returning())
        .get_result(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct UpdateAiProvider {
    pub name: Option<String>,
    pub provider_type: Option<String>,
    pub api_key: Option<String>,
    pub endpoint_url: Option<String>,
    pub model_name: Option<String>,
    pub is_active: Option<bool>,
    pub provider_category: Option<String>,
}

pub async fn update_ai_provider(
    State(state): State<AppState>,
    Path(provider_id): Path<i32>,
    Json(payload): Json<UpdateAiProvider>,
) -> Result<Json<AiProvider>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    use crate::schema::ai_providers::dsl::*;

    let target = ai_providers.filter(id.eq(provider_id));

    let result = diesel::update(target)
        .set((
            payload.name.map(|v| name.eq(v)),
            payload.provider_type.map(|v| provider_type.eq(v)),
            payload.api_key.map(|v| api_key.eq(v)),
            payload.endpoint_url.map(|v| endpoint_url.eq(v)),
            payload.model_name.map(|v| model_name.eq(v)),
            payload.is_active.map(|v| is_active.eq(v)),
            payload.provider_category.map(|v| provider_category.eq(v)),
            updated_at.eq(chrono::Utc::now().naive_utc()),
        ))
        .returning(AiProvider::as_returning())
        .get_result(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

pub async fn delete_ai_provider(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    diesel::delete(ai_providers::table.filter(ai_providers::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// DJ Memory Handlers

pub async fn get_dj_memories(
    State(state): State<AppState>,
    Path(dj_id): Path<i32>,
) -> Result<Json<Vec<DjMemory>>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let results = dj_memories::table
        .filter(dj_memories::dj_id.eq(dj_id))
        .order(dj_memories::created_at.desc())
        .load::<DjMemory>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(results))
}

pub async fn create_dj_memory(
    State(state): State<AppState>,
    Path(dj_id): Path<i32>,
    Json(mut payload): Json<NewDjMemory>,
) -> Result<Json<DjMemory>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Ensure dj_id matches path
    payload.dj_id = dj_id;

    let result = diesel::insert_into(dj_memories::table)
        .values(&payload)
        .returning(DjMemory::as_returning())
        .get_result(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

pub async fn update_dj_memory(
    State(state): State<AppState>,
    Path(memory_id): Path<i32>,
    Json(payload): Json<UpdateDjMemory>,
) -> Result<Json<DjMemory>, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let target = dj_memories::table.filter(dj_memories::id.eq(memory_id));

    let result = diesel::update(target)
        .set(&payload)
        .returning(DjMemory::as_returning())
        .get_result(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

pub async fn delete_dj_memory(
    State(state): State<AppState>,
    Path(memory_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    diesel::delete(dj_memories::table.filter(dj_memories::id.eq(memory_id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
