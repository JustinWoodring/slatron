use crate::auth::{authenticate_user, jwt::create_token, LoginRequest, LoginResponse, UserInfo};
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use std::time::{Duration, SystemTime};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Rate Limiting: 5 attempts per 15 minutes per username
    {
        let mut attempts = state.login_attempts.write().await;
        let entry = attempts
            .entry(payload.username.clone())
            .or_insert((0, SystemTime::now()));

        if entry.1.elapsed().unwrap_or(Duration::ZERO) > Duration::from_secs(15 * 60) {
            *entry = (0, SystemTime::now());
        }

        if entry.0 >= 5 {
            tracing::warn!("Login rate limit exceeded for user: {}", payload.username);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        entry.0 += 1;
    }

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = authenticate_user(&mut conn, &payload.username, &payload.password)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Reset rate limit on successful login
    {
        let mut attempts = state.login_attempts.write().await;
        if let Some(entry) = attempts.get_mut(&payload.username) {
            *entry = (0, SystemTime::now());
        }
    }

    let token = create_token(
        user.id.expect("User ID missing"),
        &user.username,
        &user.role,
        &state.config.jwt.secret,
        state.config.jwt.expiration_hours,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user.id.expect("User ID missing"),
            username: user.username,
            role: user.role,
        },
    }))
}

pub async fn logout() -> Result<StatusCode, StatusCode> {
    // With JWT, logout is handled client-side by discarding the token
    Ok(StatusCode::OK)
}
