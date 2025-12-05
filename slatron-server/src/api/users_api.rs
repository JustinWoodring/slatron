use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::Deserialize;
use crate::auth::hash_password;
use crate::models::{NewUser, User};
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, StatusCode> {
    use crate::schema::users::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = users
        .select(User::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    use crate::schema::users;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let password_hash = hash_password(&req.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_user = NewUser {
        username: req.username,
        password_hash,
        role: req.role,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    use crate::schema::users::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut query = diesel::update(users.find(user_id)).into_boxed();

    if let Some(new_username) = req.username {
        query = query.set(username.eq(new_username)).into_boxed();
    }

    if let Some(new_password) = req.password {
        let new_hash = hash_password(&new_password)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        query = query.set(password_hash.eq(new_hash)).into_boxed();
    }

    if let Some(new_role) = req.role {
        query = query.set(role.eq(new_role)).into_boxed();
    }

    let user = query
        .returning(User::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    use crate::schema::users::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(users.find(user_id))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
