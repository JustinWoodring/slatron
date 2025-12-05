use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use crate::models::{NewPermission, Permission};
use crate::AppState;

pub async fn list_permissions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Permission>>, StatusCode> {
    use crate::schema::permissions::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = permissions
        .select(Permission::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_permission(
    State(state): State<AppState>,
    Json(new_perm): Json<NewPermission>,
) -> Result<Json<Permission>, StatusCode> {
    use crate::schema::permissions;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let perm = diesel::insert_into(permissions::table)
        .values(&new_perm)
        .returning(Permission::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(perm))
}

pub async fn delete_permission(
    State(state): State<AppState>,
    Path(perm_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    use crate::schema::permissions::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(permissions.find(perm_id))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
