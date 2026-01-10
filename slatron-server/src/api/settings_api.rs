use crate::models::{GlobalSetting, NewGlobalSetting, User};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;

pub async fn list_settings(
    State(state): State<AppState>,
) -> Result<Json<Vec<GlobalSetting>>, StatusCode> {
    use crate::schema::global_settings::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = global_settings
        .load::<GlobalSetting>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn update_setting(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(setting_key): Path<String>,
    Json(payload): Json<NewGlobalSetting>,
) -> Result<Json<GlobalSetting>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::global_settings::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if exists
    let exists = global_settings
        .filter(key.eq(&setting_key))
        .first::<GlobalSetting>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let setting = if exists.is_some() {
        diesel::update(global_settings.filter(key.eq(&setting_key)))
            .set((
                value.eq(payload.value),
                description.eq(payload.description),
                updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .returning(GlobalSetting::as_select())
            .get_result(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        diesel::insert_into(global_settings)
            .values(&payload)
            .returning(GlobalSetting::as_select())
            .get_result(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(setting))
}

#[derive(serde::Serialize)]
pub struct SystemCapabilities {
    pub orpheus_enabled: bool,
}

pub async fn get_system_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<SystemCapabilities>, StatusCode> {
    Ok(Json(SystemCapabilities {
        orpheus_enabled: cfg!(feature = "ml-support"),
    }))
}
