use crate::auth::{authenticate_user, jwt::create_token, LoginRequest, LoginResponse, UserInfo};
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

async fn check_rate_limit(
    attempts_lock: &Arc<RwLock<HashMap<String, (u32, SystemTime)>>>,
    username: &str,
) -> Result<(), StatusCode> {
    const MAX_ATTEMPTS: u32 = 5;
    const WINDOW_DURATION: Duration = Duration::from_secs(15 * 60);
    const CLEANUP_THRESHOLD: usize = 1000;

    // Check for cleanup need (Read Lock)
    let needs_cleanup = {
        let attempts = attempts_lock.read().await;
        attempts.len() > CLEANUP_THRESHOLD
    };

    if needs_cleanup {
        let mut attempts = attempts_lock.write().await;
        // Double check len in case another thread cleaned up
        if attempts.len() > CLEANUP_THRESHOLD {
            // Retain only entries that are within the window
            attempts.retain(|_, (_, time)| {
                time.elapsed().unwrap_or(Duration::ZERO) < WINDOW_DURATION
            });
        }
    }

    // Rate Limiting (Write Lock)
    let mut attempts = attempts_lock.write().await;
    let entry = attempts
        .entry(username.to_string())
        .or_insert((0, SystemTime::now()));

    // Reset window if expired
    if entry.1.elapsed().unwrap_or(Duration::ZERO) > WINDOW_DURATION {
        *entry = (0, SystemTime::now());
    }

    if entry.0 >= MAX_ATTEMPTS {
        tracing::warn!("Login rate limit exceeded for user: {}", username);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    entry.0 += 1;
    Ok(())
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Rate Limiting with cleanup
    check_rate_limit(&state.login_attempts, &payload.username).await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_basic() {
        let attempts = Arc::new(RwLock::new(HashMap::new()));
        let username = "testuser";

        // 5 allowed attempts
        for _ in 0..5 {
            assert!(check_rate_limit(&attempts, username).await.is_ok());
        }

        // 6th attempt fails
        assert_eq!(
            check_rate_limit(&attempts, username).await,
            Err(StatusCode::TOO_MANY_REQUESTS)
        );
    }

    #[tokio::test]
    async fn test_rate_limit_cleanup() {
        let attempts = Arc::new(RwLock::new(HashMap::new()));

        // Fill map with 1001 entries
        // We simulate that they are old by manually inserting them
        {
            let mut map = attempts.write().await;
            let old_time = SystemTime::now().checked_sub(Duration::from_secs(20 * 60)).unwrap(); // 20 mins ago

            for i in 0..1005 {
                map.insert(format!("user{}", i), (1, old_time));
            }
        }

        // Verify size
        assert_eq!(attempts.read().await.len(), 1005);

        // Trigger cleanup by making a request
        // This request is for a new user, so it should succeed AND trigger cleanup
        assert!(check_rate_limit(&attempts, "newuser").await.is_ok());

        // Verify cleanup happened
        // All 1005 old entries should be gone, only "newuser" remains
        assert_eq!(attempts.read().await.len(), 1);
        assert!(attempts.read().await.contains_key("newuser"));
    }
}
