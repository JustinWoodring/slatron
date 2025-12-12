use crate::auth::{authenticate_user, jwt::create_token, LoginRequest, LoginResponse, UserInfo};
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = authenticate_user(&mut conn, &payload.username, &payload.password)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

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
