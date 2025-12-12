pub mod jwt;
pub mod middleware;

use crate::db::DbConnection;
use crate::models::User;
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub role: String,
}

pub fn hash_password(password: &str) -> Result<String> {
    Ok(hash(password, DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    Ok(verify(password, hash)?)
}

pub fn authenticate_user(conn: &mut DbConnection, username: &str, password: &str) -> Result<User> {
    use crate::schema::users::dsl;

    let user = dsl::users
        .filter(dsl::username.eq(username))
        .select(User::as_select())
        .first::<User>(conn)?;

    if verify_password(password, &user.password_hash)? {
        Ok(user)
    } else {
        Err(anyhow::anyhow!("Invalid credentials"))
    }
}
