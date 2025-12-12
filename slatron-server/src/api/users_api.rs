use crate::auth::hash_password;
use crate::models::{NewUser, User};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;
use serde::Deserialize;

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
    Extension(user): Extension<User>,
) -> Result<Json<Vec<User>>, StatusCode> {
    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::users::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = users
        .select(User::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_user(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::users;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let password_hash =
        hash_password(&req.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_user = NewUser {
        username: req.username,
        password_hash,
        role: req.role,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(user_id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::users::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Safeguard: If demoting an admin, ensure at least one other admin remains
    if let Some(new_role) = &req.role {
        if new_role != "admin" {
            let target_user = users
                .filter(id.eq(user_id))
                .select(User::as_select())
                .first::<User>(&mut conn)
                .optional()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if let Some(target) = target_user {
                if target.is_admin() {
                    let admin_count: i64 = users
                        .filter(role.eq("admin"))
                        .count()
                        .get_result(&mut conn)
                        .unwrap_or(0);

                    if admin_count <= 1 {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                }
            }
        }
    }

    // Build a tuple of updates
    if let Some(new_username) = &req.username {
        diesel::update(users.filter(id.eq(user_id)))
            .set(username.eq(new_username))
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(new_password) = &req.password {
        let new_hash =
            hash_password(new_password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        diesel::update(users.filter(id.eq(user_id)))
            .set(password_hash.eq(new_hash))
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(new_role) = &req.role {
        diesel::update(users.filter(id.eq(user_id)))
            .set(role.eq(new_role))
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Fetch and return the updated user
    let user = users
        .filter(id.eq(user_id))
        .select(User::as_select())
        .first(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(user_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::users::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Safeguard: If deleting an admin, ensure at least one other admin remains
    let target_user = users
        .filter(id.eq(user_id))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(target) = target_user {
        if target.is_admin() {
            let admin_count: i64 = users
                .filter(role.eq("admin"))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0);

            if admin_count <= 1 {
                // Cannot delete the last admin
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    diesel::delete(users.filter(id.eq(user_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
