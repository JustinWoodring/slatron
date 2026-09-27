use crate::auth::permissions::{Action, Grants, RES_CONTENT, WILDCARD_ID};
use crate::models::{ContentItem, NewContentItem, UpdateContentItem, User};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;

pub async fn list_content(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<ContentItem>>, StatusCode> {
    use crate::schema::content_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let grants = Grants::for_user(&mut conn, &user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut results = content_items
        .select(ContentItem::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(ids) = grants.visible_ids(&user.role, RES_CONTENT) {
        results.retain(|item| item.id.map(|cid| ids.contains(&cid)).unwrap_or(false));
    }

    Ok(Json(results))
}

pub async fn create_content(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(new_item): Json<NewContentItem>,
) -> Result<Json<ContentItem>, StatusCode> {
    use crate::schema::content_items;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let grants = Grants::for_user(&mut conn, &user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // Creating requires type-wide write: editors have it by role, viewers via
    // a wildcard grant.
    grants.ensure(&user.role, Action::Write, RES_CONTENT, WILDCARD_ID)?;

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
    Json(updates): Json<UpdateContentItem>,
) -> Result<Json<ContentItem>, StatusCode> {
    use crate::schema::content_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let grants = Grants::for_user(&mut conn, &user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    grants.ensure(&user.role, Action::Write, RES_CONTENT, item_id)?;

    let item = diesel::update(content_items.filter(id.eq(item_id)))
        .set(&updates)
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
    use crate::schema::content_items::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let grants = Grants::for_user(&mut conn, &user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    grants.ensure(&user.role, Action::Write, RES_CONTENT, item_id)?;

    diesel::delete(content_items.filter(id.eq(item_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
