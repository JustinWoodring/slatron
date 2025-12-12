use crate::models::{ContentItem, NewContentItem, User};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;

pub async fn list_content(
    State(state): State<AppState>,
) -> Result<Json<Vec<ContentItem>>, StatusCode> {
    use crate::schema::content_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = content_items
        .select(ContentItem::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_content(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(new_item): Json<NewContentItem>,
) -> Result<Json<ContentItem>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::content_items;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let item = diesel::insert_into(content_items::table)
        .values(&new_item)
        .returning(ContentItem::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(item))
}

pub async fn update_content(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(item_id): Path<i32>,
    Json(updates): Json<NewContentItem>,
) -> Result<Json<ContentItem>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::content_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let item = diesel::update(content_items.filter(id.eq(item_id)))
        .set((
            title.eq(updates.title),
            description.eq(updates.description),
            content_type.eq(updates.content_type),
            content_path.eq(updates.content_path),
            adapter_id.eq(updates.adapter_id),
            duration_minutes.eq(updates.duration_minutes),
            tags.eq(updates.tags),
            node_accessibility.eq(updates.node_accessibility),
            transformer_scripts.eq(updates.transformer_scripts),
        ))
        .returning(ContentItem::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(item))
}

pub async fn delete_content(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(item_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::content_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(content_items.filter(id.eq(item_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
