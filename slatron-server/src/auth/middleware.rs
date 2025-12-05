use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::auth::jwt::{verify_token, Claims};
use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];
    let claims = verify_token(token, &state.config.jwt.secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Attach claims to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

pub fn extract_claims(headers: &HeaderMap, secret: &str) -> Option<Claims> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;

    if !auth_header.starts_with("Bearer ") {
        return None;
    }

    let token = &auth_header[7..];
    verify_token(token, secret).ok()
}
