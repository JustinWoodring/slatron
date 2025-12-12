use crate::AppState;
use chrono::Utc;
use diesel::prelude::*;
use std::time::Duration;
use tokio::time::interval;

pub async fn run(state: AppState) {
    let mut tick = interval(Duration::from_secs(30));

    loop {
        tick.tick().await;

        if let Err(e) = check_heartbeats(&state).await {
            tracing::error!("Heartbeat monitor error: {}", e);
        }
    }
}

async fn check_heartbeats(state: &AppState) -> Result<(), String> {
    use crate::schema::nodes::dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| "Database connection error".to_string())?;

    // Threshold: 30 seconds ago
    let threshold = Utc::now().naive_utc() - chrono::Duration::seconds(30);

    // Find nodes that are 'online' but haven't sent a heartbeat recently
    let offline_count = diesel::update(
        dsl::nodes
            .filter(dsl::status.eq("online"))
            .filter(dsl::last_heartbeat.lt(threshold)),
    )
    .set(dsl::status.eq("offline"))
    .execute(&mut conn)
    .map_err(|e| e.to_string())?;

    if offline_count > 0 {
        tracing::warn!("Marked {} unresponsive nodes as offline", offline_count);
    }

    Ok(())
}
