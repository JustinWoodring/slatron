use crate::models::{NewSchedule, NewScheduleBlock, Schedule, ScheduleBlock, UpdateSchedule, User};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{NaiveDate, Timelike};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub schedule_id: i32,
    pub dj_id: Option<i32>,
    pub dj_name: Option<String>,
}

pub async fn list_schedules(
    State(state): State<AppState>,
) -> Result<Json<Vec<Schedule>>, StatusCode> {
    use crate::schema::schedules::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = schedules
        .select(Schedule::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_schedule(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(new_schedule): Json<NewSchedule>,
) -> Result<Json<Schedule>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::schedules;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let schedule = diesel::insert_into(schedules::table)
        .values(&new_schedule)
        .returning(Schedule::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(schedule))
}

pub async fn update_schedule(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(schedule_id): Path<i32>,
    Json(updates): Json<UpdateSchedule>,
) -> Result<Json<Schedule>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::node_schedules;
    use crate::schema::schedules::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Enforce activation logic: A schedule cannot be active if it isn't assigned to a node.
    if let Some(true) = updates.is_active {
        let count: i64 = node_schedules::table
            .filter(node_schedules::schedule_id.eq(schedule_id))
            .count()
            .get_result(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if count == 0 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let schedule = diesel::update(schedules.filter(id.eq(schedule_id)))
        .set(&updates)
        .returning(Schedule::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(schedule))
}

pub async fn delete_schedule(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(schedule_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::schedules::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(schedules.filter(id.eq(schedule_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_schedule_blocks(
    State(state): State<AppState>,
    Path(schedule_id): Path<i32>,
) -> Result<Json<Vec<ScheduleBlock>>, StatusCode> {
    use crate::schema::schedule_blocks::dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let blocks = dsl::schedule_blocks
        .filter(dsl::schedule_id.eq(schedule_id))
        .select(ScheduleBlock::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(blocks))
}

// Helper to check for overlaps
fn check_overlap(
    conn: &mut SqliteConnection,
    sched_id: i32,
    day: Option<i32>,
    date: Option<NaiveDate>, // Added date parameter
    start: chrono::NaiveTime,
    duration_mins: i32,
    exclude_block_id: Option<i32>,
) -> Result<bool, diesel::result::Error> {
    use crate::schema::schedule_blocks::dsl::*;

    let mut query = schedule_blocks
        .filter(schedule_id.eq(sched_id))
        .into_boxed();

    if let Some(d) = day {
        query = query.filter(day_of_week.eq(d));
    } else if let Some(dt) = date {
        query = query.filter(specific_date.eq(dt));
    } else {
        // If neither, fallback or logic needs refinement?
        // For now, if neither is set (shouldn't happen in strict mode), maybe return false?
        // But let's assume one is always set.
    }

    let blocks = query
        .select(ScheduleBlock::as_select())
        .load::<ScheduleBlock>(conn)?;

    let new_start_mins = start.num_seconds_from_midnight() as i32 / 60;
    let new_end_mins = new_start_mins + duration_mins;

    for b in blocks {
        if let Some(id_to_exclude) = exclude_block_id {
            if b.id == Some(id_to_exclude) {
                continue;
            }
        }

        let b_start_mins = b.start_time.num_seconds_from_midnight() as i32 / 60;
        let b_end_mins = b_start_mins + b.duration_minutes;

        // Check intersection
        if new_start_mins < b_end_mins && new_end_mins > b_start_mins {
            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn create_schedule_block(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(_s_id): Path<i32>,
    Json(new_block): Json<NewScheduleBlock>,
) -> Result<Json<ScheduleBlock>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::schedule_blocks;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate duration and end of day
    if new_block.duration_minutes <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let start_mins = new_block.start_time.num_seconds_from_midnight() as i32 / 60;
    if start_mins + new_block.duration_minutes > 1440 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check overlap
    let has_overlap = check_overlap(
        &mut conn,
        new_block.schedule_id,
        new_block.day_of_week,
        new_block.specific_date,
        new_block.start_time,
        new_block.duration_minutes,
        None,
    )
    .map_err(|e| {
        tracing::error!("Overlap check failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if has_overlap {
        return Err(StatusCode::CONFLICT);
    }

    let block = diesel::insert_into(schedule_blocks::table)
        .values(&new_block)
        .returning(ScheduleBlock::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(block))
}

pub async fn update_schedule_block(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((_schedule_id, block_id)): Path<(i32, i32)>,
    Json(updates): Json<NewScheduleBlock>,
) -> Result<Json<ScheduleBlock>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::schedule_blocks::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate duration and end of day
    if updates.duration_minutes <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let start_mins = updates.start_time.num_seconds_from_midnight() as i32 / 60;
    if start_mins + updates.duration_minutes > 1440 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check overlap
    let has_overlap = check_overlap(
        &mut conn,
        updates.schedule_id,
        updates.day_of_week,
        updates.specific_date,
        updates.start_time,
        updates.duration_minutes,
        Some(block_id),
    )
    .map_err(|e| {
        tracing::error!("Overlap check failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if has_overlap {
        return Err(StatusCode::CONFLICT);
    }

    let block = diesel::update(schedule_blocks.filter(id.eq(block_id)))
        .set((
            content_id.eq(updates.content_id),
            day_of_week.eq(updates.day_of_week),
            specific_date.eq(updates.specific_date),
            start_time.eq(updates.start_time),
            duration_minutes.eq(updates.duration_minutes),
            script_id.eq(updates.script_id),
            dj_id.eq(updates.dj_id), // Added missing field
        ))
        .returning(ScheduleBlock::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(block))
}

pub async fn delete_schedule_block(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((_schedule_id, block_id)): Path<(i32, i32)>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::schedule_blocks::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(schedule_blocks.filter(id.eq(block_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_collapsed_schedule(
    State(state): State<AppState>,
    Query(params): Query<CollapsedScheduleQuery>,
) -> Result<Json<CollapsedScheduleResponse>, StatusCode> {
    use crate::services::schedule_service;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let blocks = schedule_service::calculate_collapsed_schedule(
        &mut conn,
        params.node_id,
        params.date,
        None,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CollapsedScheduleResponse { blocks }))
}
