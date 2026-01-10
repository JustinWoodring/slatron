use crate::auth::jwt::verify_token;
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

#[allow(dead_code)]
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    use crate::models::User;
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];
    let claims = verify_token(token, state.config.jwt.secret.as_bytes())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Fetch user from DB to ensure validity and get full details
    // Fetch user from DB
    let mut conn = match state.db.get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Auth Middleware - Failed to get DB connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let user = match users
        .filter(id.eq(claims.user_id))
        .select(User::as_select())
        .first::<User>(&mut conn)
    {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(
                "Auth Middleware - Failed to fetch user {}: {}",
                claims.user_id,
                e
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Attach user (and claims) to request extensions
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}
