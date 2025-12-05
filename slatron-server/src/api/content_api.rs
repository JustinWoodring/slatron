use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use crate::models::{ContentItem, NewContentItem};
use crate::AppState;

pub async fn list_content(
    State(state): State<AppState>,
) -> Result<Json<Vec<ContentItem>>, StatusCode> {
    use crate::schema::content_items::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = content_items
        .select(ContentItem::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_content(
    State(state): State<AppState>,
    Json(new_item): Json<NewContentItem>,
) -> Result<Json<ContentItem>, StatusCode> {
    use crate::schema::content_items;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let item = diesel::insert_into(content_items::table)
        .values(&new_item)
        .returning(ContentItem::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(item))
}

pub async fn update_content(
    State(state): State<AppState>,
    Path(item_id): Path<i32>,
    Json(updates): Json<NewContentItem>,
) -> Result<Json<ContentItem>, StatusCode> {
    use crate::schema::content_items::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let item = diesel::update(content_items.find(item_id))
        .set((
            title.eq(updates.title),
            description.eq(updates.description),
            content_type.eq(updates.content_type),
            content_path.eq(updates.content_path),
            adapter_id.eq(updates.adapter_id),
            duration_minutes.eq(updates.duration_minutes),
            tags.eq(updates.tags),
            node_accessibility.eq(updates.node_accessibility),
        ))
        .returning(ContentItem::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(item))
}

pub async fn delete_content(
    State(state): State<AppState>,
    Path(item_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    use crate::schema::content_items::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(content_items.find(item_id))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
