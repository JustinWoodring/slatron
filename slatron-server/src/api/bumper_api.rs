use crate::models::{
    Bumper, BumperBack, NewBumper, NewBumperBack, UpdateBumper, UpdateBumperBack, User,
};
use crate::AppState;
use axum::extract::Multipart;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

#[derive(Serialize)]
pub struct RenderResponse {
    pub success: bool,
    pub rendered_path: Option<String>,
    pub duration_ms: Option<i32>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct RenderAllResponse {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[derive(Deserialize)]
pub struct FetchBackRequest {
    pub url: String,
    pub name: String,
}

pub async fn list_bumpers(State(state): State<AppState>) -> Result<Json<Vec<Bumper>>, StatusCode> {
    use crate::schema::bumpers::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = bumpers
        .select(Bumper::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn get_bumper(
    State(state): State<AppState>,
    Path(bumper_id): Path<i32>,
) -> Result<Json<Bumper>, StatusCode> {
    use crate::schema::bumpers::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bumper = bumpers
        .filter(id.eq(bumper_id))
        .first(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(bumper))
}

pub async fn create_bumper(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(new_bumper): Json<NewBumper>,
) -> Result<Json<Bumper>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumpers;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate bumper_type
    let valid_types = [
        "station_ident",
        "transition",
        "show_opener",
        "lower_third",
        "custom",
    ];
    if !valid_types.contains(&new_bumper.bumper_type.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let bumper = diesel::insert_into(bumpers::table)
        .values(&new_bumper)
        .returning(Bumper::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(bumper))
}

pub async fn update_bumper(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(bumper_id): Path<i32>,
    Json(updates): Json<UpdateBumper>,
) -> Result<Json<Bumper>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumpers::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate bumper_type if provided
    if let Some(ref bt) = updates.bumper_type {
        let valid_types = [
            "station_ident",
            "transition",
            "show_opener",
            "lower_third",
            "custom",
        ];
        if !valid_types.contains(&bt.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let bumper = diesel::update(bumpers.filter(id.eq(bumper_id)))
        .set(&updates)
        .returning(Bumper::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(bumper))
}

pub async fn delete_bumper(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(bumper_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumpers::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if bumper is builtin
    let bumper: Bumper = bumpers
        .filter(id.eq(bumper_id))
        .first(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if bumper.is_builtin && !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    diesel::delete(bumpers.filter(id.eq(bumper_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn render_bumper(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(bumper_id): Path<i32>,
) -> Result<Json<RenderResponse>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.bumper_service.render_template(bumper_id) {
        Ok(_) => {
            // Fetch updated bumper to get rendered_path and duration_ms
            use crate::schema::bumpers::dsl::*;
            let mut conn = state
                .db
                .get()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let bumper: Bumper = bumpers
                .filter(id.eq(bumper_id))
                .first(&mut conn)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(RenderResponse {
                success: true,
                rendered_path: bumper.rendered_path,
                duration_ms: bumper.duration_ms,
                error: None,
            }))
        }
        Err(e) => Ok(Json(RenderResponse {
            success: false,
            rendered_path: None,
            duration_ms: None,
            error: Some(e.to_string()),
        })),
    }
}

pub async fn render_all_bumpers(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<RenderAllResponse>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumpers::dsl::*;
    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template_bumpers: Vec<Bumper> = bumpers
        .filter(is_template.eq(true))
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = template_bumpers.len();
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for bumper in template_bumpers {
        if let Some(bumper_id) = bumper.id {
            match state.bumper_service.render_template(bumper_id) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{}: {}", bumper.name, e));
                }
            }
        }
    }

    Ok(Json(RenderAllResponse {
        total,
        successful,
        failed,
        errors,
    }))
}

// Bumper Back API endpoints

pub async fn list_bumper_backs(
    State(state): State<AppState>,
) -> Result<Json<Vec<BumperBack>>, StatusCode> {
    use crate::schema::bumper_backs::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = bumper_backs
        .select(BumperBack::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn get_bumper_back(
    State(state): State<AppState>,
    Path(back_id): Path<i32>,
) -> Result<Json<BumperBack>, StatusCode> {
    use crate::schema::bumper_backs::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let back = bumper_backs
        .filter(id.eq(back_id))
        .first(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(back))
}

pub async fn create_bumper_back(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(new_back): Json<NewBumperBack>,
) -> Result<Json<BumperBack>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumper_backs;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let back = diesel::insert_into(bumper_backs::table)
        .values(&new_back)
        .returning(BumperBack::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(back))
}

pub async fn update_bumper_back(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(back_id): Path<i32>,
    Json(updates): Json<UpdateBumperBack>,
) -> Result<Json<BumperBack>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumper_backs::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let back = diesel::update(bumper_backs.filter(id.eq(back_id)))
        .set(&updates)
        .returning(BumperBack::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(back))
}

pub async fn delete_bumper_back(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(back_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumper_backs::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if bumper back is builtin
    let back: BumperBack = bumper_backs
        .filter(id.eq(back_id))
        .first(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if back.is_builtin && !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if any bumpers reference this back
    use crate::schema::bumpers;
    let referencing_count: i64 = bumpers::table
        .filter(bumpers::bumper_back_id.eq(Some(back_id)))
        .count()
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if referencing_count > 0 {
        // Cannot delete if bumpers reference it
        return Err(StatusCode::CONFLICT);
    }

    diesel::delete(bumper_backs.filter(id.eq(back_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn render_bumper_back(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(back_id): Path<i32>,
) -> Result<Json<RenderResponse>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.bumper_service.render_bumper_back(back_id) {
        Ok(_) => {
            // Fetch updated bumper back to get rendered path and duration
            use crate::schema::bumper_backs::dsl::*;
            let mut conn = state
                .db
                .get()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let back: BumperBack = bumper_backs
                .filter(id.eq(back_id))
                .first(&mut conn)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(RenderResponse {
                success: true,
                rendered_path: Some(back.file_path),
                duration_ms: back.duration_ms,
                error: None,
            }))
        }
        Err(e) => Ok(Json(RenderResponse {
            success: false,
            rendered_path: None,
            duration_ms: None,
            error: Some(e.to_string()),
        })),
    }
}

pub async fn render_all_bumper_backs(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<RenderAllResponse>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::bumper_backs::dsl::*;
    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let backs: Vec<BumperBack> = bumper_backs
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = backs
        .iter()
        .filter(|b| b.file_path.ends_with(".mlt"))
        .count();
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for back in backs {
        if back.file_path.ends_with(".mlt") {
            if let Some(back_id) = back.id {
                match state.bumper_service.render_bumper_back(back_id) {
                    Ok(_) => successful += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", back.name, e));
                    }
                }
            }
        }
    }

    Ok(Json(RenderAllResponse {
        total,
        successful,
        failed,
        errors,
    }))
}

pub async fn fetch_bumper_back(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<FetchBackRequest>,
) -> Result<Json<BumperBack>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    // 1. Download file
    let file_path = state
        .bumper_service
        .download_remote_file(&req.url)
        .map_err(|e| {
            tracing::error!("Failed to download bumper back: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 2. Get duration
    let duration_ms = state
        .bumper_service
        .get_duration_ms_public(file_path.to_str().unwrap_or(""))
        .map_err(|e| {
            tracing::error!("Failed to get duration: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 3. Create DB entry
    use crate::schema::bumper_backs;
    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_back = NewBumperBack {
        name: req.name,
        description: Some(format!("Fetched from {}", req.url)),
        file_path: file_path.to_string_lossy().to_string(),
        duration_ms: Some(duration_ms),
        is_builtin: false,
    };

    let back = diesel::insert_into(bumper_backs::table)
        .values(&new_back)
        .returning(BumperBack::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(back))
}

pub async fn upload_bumper_back(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<Json<BumperBack>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut name = String::new();
    let mut temp_file_path: Option<std::path::PathBuf> = None;
    let mut original_filename = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Multipart error: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "name" {
            name = field.text().await.map_err(|e| {
                tracing::error!("Failed to read name field: {}", e);
                StatusCode::BAD_REQUEST
            })?;
        } else if field_name == "file" {
            original_filename = field.file_name().unwrap_or("upload.mp4").to_string();

            // Stream to temp file
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("upload_{}", Uuid::new_v4()));
            let mut file = std::fs::File::create(&temp_path).map_err(|e| {
                tracing::error!("Failed to create temp file at {:?}: {}", temp_path, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let data = field.bytes().await.map_err(|e| {
                tracing::error!("Failed to read file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            file.write_all(&data).map_err(|e| {
                tracing::error!("Failed to write to temp file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Validate file type
            let file_type = infer::get_from_path(&temp_path)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or_else(|| {
                    let _ = std::fs::remove_file(&temp_path);
                    StatusCode::BAD_REQUEST
                })?;

            if !file_type.mime_type().starts_with("video/") {
                let _ = std::fs::remove_file(&temp_path);
                return Err(StatusCode::BAD_REQUEST);
            }

            // Whitelist specific extensions
            let extension = std::path::Path::new(&original_filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let allowed_extensions = ["mp4", "webm", "mov", "avi", "mkv"];
            if !allowed_extensions.contains(&extension.as_str()) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(StatusCode::BAD_REQUEST);
            }

            temp_file_path = Some(temp_path);
        }
    }

    if name.is_empty() || temp_file_path.is_none() {
        // If we have a temp file but failed for other reasons (missing name), clean it up
        if let Some(path) = temp_file_path {
            let _ = std::fs::remove_file(path);
        }
        tracing::error!("Missing name or file in upload request");
        return Err(StatusCode::BAD_REQUEST);
    }

    let temp_path = temp_file_path.unwrap();

    // 1. Process uploaded file (move to static)
    let file_path_res = state
        .bumper_service
        .process_uploaded_file(&temp_path, &original_filename);

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    let file_path = file_path_res.map_err(|e| {
        tracing::error!("Failed to process upload: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 2. Get duration
    let duration_ms = state
        .bumper_service
        .get_duration_ms_public(file_path.to_str().unwrap_or(""))
        .map_err(|e| {
            tracing::error!("Failed to get duration: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 3. Create DB entry
    use crate::schema::bumper_backs;
    let mut conn = state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_back = NewBumperBack {
        name,
        description: Some("Uploaded file".to_string()),
        file_path: file_path.to_string_lossy().to_string(),
        duration_ms: Some(duration_ms),
        is_builtin: false,
    };

    let back = diesel::insert_into(bumper_backs::table)
        .values(&new_back)
        .returning(BumperBack::as_select())
        .get_result(&mut conn)
        .map_err(|e| {
            tracing::error!("Database insert error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(back))
}
