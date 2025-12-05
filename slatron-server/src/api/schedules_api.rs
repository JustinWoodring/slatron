use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::{NewSchedule, NewScheduleBlock, Schedule, ScheduleBlock};
use crate::AppState;

#[derive(Deserialize)]
pub struct CollapsedScheduleQuery {
    pub node_id: i32,
    pub date: NaiveDate,
}

#[derive(Serialize)]
pub struct CollapsedScheduleResponse {
    pub blocks: Vec<CollapsedBlock>,
}

#[derive(Serialize, Clone)]
pub struct CollapsedBlock {
    pub start_time: String,
    pub duration_minutes: i32,
    pub content_id: Option<i32>,
    pub script_id: Option<i32>,
    pub priority: i32,
    pub schedule_name: String,
}

pub async fn list_schedules(
    State(state): State<AppState>,
) -> Result<Json<Vec<Schedule>>, StatusCode> {
    use crate::schema::schedules::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = schedules
        .select(Schedule::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_schedule(
    State(state): State<AppState>,
    Json(new_schedule): Json<NewSchedule>,
) -> Result<Json<Schedule>, StatusCode> {
    use crate::schema::schedules;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let schedule = diesel::insert_into(schedules::table)
        .values(&new_schedule)
        .returning(Schedule::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(schedule))
}

pub async fn update_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<i32>,
    Json(updates): Json<NewSchedule>,
) -> Result<Json<Schedule>, StatusCode> {
    use crate::schema::schedules::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let schedule = diesel::update(schedules.find(schedule_id))
        .set((
            name.eq(updates.name),
            description.eq(updates.description),
            schedule_type.eq(updates.schedule_type),
            priority.eq(updates.priority),
            is_active.eq(updates.is_active),
        ))
        .returning(Schedule::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(schedule))
}

pub async fn delete_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    use crate::schema::schedules::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(schedules.find(schedule_id))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_schedule_blocks(
    State(state): State<AppState>,
    Path(schedule_id): Path<i32>,
) -> Result<Json<Vec<ScheduleBlock>>, StatusCode> {
    use crate::schema::schedule_blocks::dsl;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let blocks = dsl::schedule_blocks
        .filter(dsl::schedule_id.eq(schedule_id))
        .select(ScheduleBlock::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(blocks))
}

pub async fn create_schedule_block(
    State(state): State<AppState>,
    Path(_schedule_id): Path<i32>,
    Json(new_block): Json<NewScheduleBlock>,
) -> Result<Json<ScheduleBlock>, StatusCode> {
    use crate::schema::schedule_blocks;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let block = diesel::insert_into(schedule_blocks::table)
        .values(&new_block)
        .returning(ScheduleBlock::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(block))
}

pub async fn update_schedule_block(
    State(state): State<AppState>,
    Path((_schedule_id, block_id)): Path<(i32, i32)>,
    Json(updates): Json<NewScheduleBlock>,
) -> Result<Json<ScheduleBlock>, StatusCode> {
    use crate::schema::schedule_blocks::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let block = diesel::update(schedule_blocks.find(block_id))
        .set((
            content_id.eq(updates.content_id),
            day_of_week.eq(updates.day_of_week),
            specific_date.eq(updates.specific_date),
            start_time.eq(updates.start_time),
            duration_minutes.eq(updates.duration_minutes),
            script_id.eq(updates.script_id),
        ))
        .returning(ScheduleBlock::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(block))
}

pub async fn delete_schedule_block(
    State(state): State<AppState>,
    Path((_schedule_id, block_id)): Path<(i32, i32)>,
) -> Result<StatusCode, StatusCode> {
    use crate::schema::schedule_blocks::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(schedule_blocks.find(block_id))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_collapsed_schedule(
    State(state): State<AppState>,
    Query(params): Query<CollapsedScheduleQuery>,
) -> Result<Json<CollapsedScheduleResponse>, StatusCode> {
    use crate::services::schedule_service;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let blocks = schedule_service::calculate_collapsed_schedule(&mut conn, params.node_id, params.date)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CollapsedScheduleResponse { blocks }))
}
