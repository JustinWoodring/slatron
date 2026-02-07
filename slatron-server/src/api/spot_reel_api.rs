use crate::models::{
    ContentItem, NewContentItem, NewSpotReel, NewSpotReelItem, SpotReel, SpotReelItem,
    UpdateSpotReel, UpdateSpotReelItem, User,
};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Response types

#[derive(Serialize)]
pub struct SpotReelWithItems {
    #[serde(flatten)]
    pub reel: SpotReel,
    pub items: Vec<SpotReelItem>,
    pub content_item_id: Option<i32>,
}

#[derive(Serialize)]
pub struct SpotReelListEntry {
    #[serde(flatten)]
    pub reel: SpotReel,
    pub item_count: i64,
    pub total_duration_secs: i64,
    pub content_item_id: Option<i32>,
}

// Request types

#[derive(Deserialize)]
pub struct CreateSpotReelRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct AddSpotReelItemRequest {
    pub item_type: String,
    pub item_path: String,
    pub display_duration_secs: Option<i32>,
    pub title: Option<String>,
}

#[derive(Deserialize)]
pub struct ReorderItem {
    pub id: i32,
    pub position: i32,
}

#[derive(Deserialize)]
pub struct ReorderRequest {
    pub items: Vec<ReorderItem>,
}

// --- Spot Reel CRUD ---

pub async fn list_spot_reels(
    State(state): State<AppState>,
) -> Result<Json<Vec<SpotReelListEntry>>, StatusCode> {
    use crate::schema::content_items;
    use crate::schema::spot_reel_items;
    use crate::schema::spot_reels::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reels: Vec<SpotReel> = spot_reels
        .select(SpotReel::as_select())
        .order(title.asc())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut entries = Vec::new();
    for reel in reels {
        let reel_id_val = reel.id.unwrap_or(0);

        let item_count: i64 = spot_reel_items::table
            .filter(spot_reel_items::spot_reel_id.eq(reel_id_val))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let items: Vec<SpotReelItem> = spot_reel_items::table
            .filter(spot_reel_items::spot_reel_id.eq(reel_id_val))
            .select(SpotReelItem::as_select())
            .load(&mut conn)
            .unwrap_or_default();

        let total_duration_secs: i64 = items
            .iter()
            .map(|i| i.display_duration_secs as i64)
            .sum();

        // Find the associated content_item
        let content_item: Option<ContentItem> = content_items::table
            .filter(content_items::spot_reel_id.eq(Some(reel_id_val)))
            .select(ContentItem::as_select())
            .first(&mut conn)
            .ok();

        entries.push(SpotReelListEntry {
            reel,
            item_count,
            total_duration_secs,
            content_item_id: content_item.and_then(|c| c.id),
        });
    }

    Ok(Json(entries))
}

pub async fn get_spot_reel(
    State(state): State<AppState>,
    Path(reel_id): Path<i32>,
) -> Result<Json<SpotReelWithItems>, StatusCode> {
    use crate::schema::content_items;
    use crate::schema::spot_reel_items;
    use crate::schema::spot_reels::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reel: SpotReel = spot_reels
        .filter(id.eq(reel_id))
        .first(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let items: Vec<SpotReelItem> = spot_reel_items::table
        .filter(spot_reel_items::spot_reel_id.eq(reel_id))
        .select(SpotReelItem::as_select())
        .order(spot_reel_items::position.asc())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let content_item: Option<ContentItem> = content_items::table
        .filter(content_items::spot_reel_id.eq(Some(reel_id)))
        .select(ContentItem::as_select())
        .first(&mut conn)
        .ok();

    Ok(Json(SpotReelWithItems {
        reel,
        items,
        content_item_id: content_item.and_then(|c| c.id),
    }))
}

pub async fn create_spot_reel(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<CreateSpotReelRequest>,
) -> Result<Json<SpotReelWithItems>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::content_items;
    use crate::schema::spot_reels;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 1. Create the spot reel
    let new_reel = NewSpotReel {
        title: req.title.clone(),
        description: req.description.clone(),
    };

    let reel: SpotReel = diesel::insert_into(spot_reels::table)
        .values(&new_reel)
        .returning(SpotReel::as_select())
        .get_result(&mut conn)
        .map_err(|e| {
            tracing::error!("Failed to create spot reel: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reel_id_val = reel.id.unwrap_or(0);

    // 2. Auto-create a content_item pointing to this reel
    let new_content = NewContentItem {
        title: req.title,
        description: req.description,
        content_type: "spot_reel".to_string(),
        content_path: format!("spot_reel://{}", reel_id_val),
        adapter_id: None,
        duration_minutes: None,
        tags: Some("spot_reel".to_string()),
        node_accessibility: None,
        transformer_scripts: None,
        is_dj_accessible: false,
        spot_reel_id: Some(reel_id_val),
    };

    let content: ContentItem = diesel::insert_into(content_items::table)
        .values(&new_content)
        .returning(ContentItem::as_select())
        .get_result(&mut conn)
        .map_err(|e| {
            tracing::error!("Failed to create content item for spot reel: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(SpotReelWithItems {
        reel,
        items: vec![],
        content_item_id: content.id,
    }))
}

pub async fn update_spot_reel(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(reel_id): Path<i32>,
    Json(updates): Json<UpdateSpotReel>,
) -> Result<Json<SpotReel>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::spot_reels::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reel = diesel::update(spot_reels.filter(id.eq(reel_id)))
        .set(&updates)
        .returning(SpotReel::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Also update the associated content_item title/description if changed
    if updates.title.is_some() || updates.description.is_some() {
        use crate::schema::content_items;
        use crate::models::UpdateContentItem;

        let content_update = UpdateContentItem {
            title: updates.title,
            description: updates.description,
            content_type: None,
            content_path: None,
            adapter_id: None,
            duration_minutes: None,
            tags: None,
            node_accessibility: None,
            transformer_scripts: None,
            is_dj_accessible: None,
            spot_reel_id: None,
        };

        let _ = diesel::update(
            content_items::table.filter(content_items::spot_reel_id.eq(Some(reel_id))),
        )
        .set(&content_update)
        .execute(&mut conn);
    }

    Ok(Json(reel))
}

pub async fn delete_spot_reel(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(reel_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::content_items;
    use crate::schema::spot_reels::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete the associated content_item first
    diesel::delete(content_items::table.filter(content_items::spot_reel_id.eq(Some(reel_id))))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete the spot reel (cascade will remove items)
    diesel::delete(spot_reels.filter(id.eq(reel_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Spot Reel Items ---

pub async fn add_item(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(reel_id): Path<i32>,
    Json(req): Json<AddSpotReelItemRequest>,
) -> Result<Json<SpotReelItem>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate item_type
    let valid_types = ["image", "video", "web"];
    if !valid_types.contains(&req.item_type.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    use crate::schema::spot_reel_items;
    use crate::schema::spot_reels::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify the reel exists
    let _reel: SpotReel = spot_reels
        .filter(id.eq(reel_id))
        .first(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Get the next position
    let max_position: Option<i32> = spot_reel_items::table
        .filter(spot_reel_items::spot_reel_id.eq(reel_id))
        .select(diesel::dsl::max(spot_reel_items::position))
        .first(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let next_position = max_position.map(|p| p + 1).unwrap_or(0);

    let new_item = NewSpotReelItem {
        spot_reel_id: reel_id,
        item_type: req.item_type,
        item_path: req.item_path,
        display_duration_secs: req.display_duration_secs.unwrap_or(10),
        position: next_position,
        title: req.title,
    };

    let item = diesel::insert_into(spot_reel_items::table)
        .values(&new_item)
        .returning(SpotReelItem::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((reel_id, item_id)): Path<(i32, i32)>,
    Json(updates): Json<UpdateSpotReelItem>,
) -> Result<Json<SpotReelItem>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate item_type if provided
    if let Some(ref it) = updates.item_type {
        let valid_types = ["image", "video", "web"];
        if !valid_types.contains(&it.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    use crate::schema::spot_reel_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let item = diesel::update(
        spot_reel_items
            .filter(id.eq(item_id))
            .filter(spot_reel_id.eq(reel_id)),
    )
    .set(&updates)
    .returning(SpotReelItem::as_select())
    .get_result(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(item))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((reel_id, item_id)): Path<(i32, i32)>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::spot_reel_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(
        spot_reel_items
            .filter(id.eq(item_id))
            .filter(spot_reel_id.eq(reel_id)),
    )
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn reorder_items(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(reel_id): Path<i32>,
    Json(req): Json<ReorderRequest>,
) -> Result<Json<Vec<SpotReelItem>>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }

    use crate::schema::spot_reel_items;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update each item's position
    for reorder in &req.items {
        diesel::update(
            spot_reel_items::table
                .filter(spot_reel_items::id.eq(reorder.id))
                .filter(spot_reel_items::spot_reel_id.eq(reel_id)),
        )
        .set(spot_reel_items::position.eq(reorder.position))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Return updated items
    let items: Vec<SpotReelItem> = spot_reel_items::table
        .filter(spot_reel_items::spot_reel_id.eq(reel_id))
        .select(SpotReelItem::as_select())
        .order(spot_reel_items::position.asc())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(items))
}
