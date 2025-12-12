use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub user_id: i32,
    pub role: String,
    pub exp: i64, // expiration time
}

pub fn create_token(
    user_id: i32,
    username: &str,
    role: &str,
    secret: &str,
    expiration_hours: i64,
) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expiration_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: username.to_owned(),
        user_id,
        role: role.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("Failed to create token: {}", e))
}

#[allow(dead_code)]
pub fn verify_token(token_str: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token_str,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
