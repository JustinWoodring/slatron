use crate::auth::permissions::RESOURCE_TYPES;
use crate::models::{NewPermission, Permission, User};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;

pub async fn list_permissions(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<Permission>>, StatusCode> {
    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::permissions::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = permissions
        .select(Permission::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_permission(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(new_perm): Json<NewPermission>,
) -> Result<Json<Permission>, StatusCode> {
    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    if !RESOURCE_TYPES.contains(&new_perm.resource_type.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !matches!(
        new_perm.permission_level.as_str(),
        "read" | "write" | "admin"
    ) {
        return Err(StatusCode::BAD_REQUEST);
    }
    use crate::schema::permissions;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let perm = diesel::insert_into(permissions::table)
        .values(&new_perm)
        .returning(Permission::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(perm))
}

pub async fn delete_permission(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(perm_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::permissions::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(permissions.filter(id.eq(perm_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
