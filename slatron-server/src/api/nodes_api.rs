use crate::models::{NewNode, Node, User};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateNodeRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateNodeResponse {
    pub node: Node,
    pub secret_key: String,
}

#[derive(Deserialize)]
pub struct NodeCommand {
    pub _action: String,
    pub _position_secs: Option<i32>,
    pub _content_id: Option<i32>,
}

#[derive(Serialize)]
pub struct EffectiveBlock {
    pub id: Option<i32>,
    pub schedule_id: i32,
    pub content_id: Option<i32>,
    pub specific_date: Option<chrono::NaiveDate>,
    pub start_time: chrono::NaiveTime,
    pub duration_minutes: i32,
    pub script_id: Option<i32>,
    pub source_schedule_name: String,
    pub dj_id: Option<i32>,
    pub dj_name: Option<String>,
}

#[derive(Serialize)]
pub struct NodeScheduleResponse {
    pub schedule: Option<crate::models::Schedule>,
    pub assigned_schedules: Vec<crate::models::Schedule>,
    pub blocks: Vec<EffectiveBlock>, // Changed from ScheduleBlock
    pub content: Vec<crate::models::ContentItem>,
    pub scripts: Vec<crate::models::Script>,
}

pub async fn list_nodes(State(state): State<AppState>) -> Result<Json<Vec<Node>>, StatusCode> {
    use crate::schema::nodes::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = nodes
        .select(Node::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_node(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<CreateNodeResponse>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::nodes;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate a secret key for the node
    let secret_key = Uuid::new_v4().to_string();

    let new_node = NewNode {
        name: req.name,
        secret_key: secret_key.clone(),
        ip_address: None,
        status: "offline".to_string(),
    };

    let node = diesel::insert_into(nodes::table)
        .values(&new_node)
        .returning(Node::as_select())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateNodeResponse { node, secret_key }))
}

#[derive(Deserialize)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
}

pub async fn delete_node(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(node_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::nodes::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(nodes.filter(id.eq(node_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_node(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(node_id): Path<i32>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<Node>, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::nodes::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let target = diesel::update(nodes.filter(id.eq(node_id)));

    if let Some(new_name) = &req.name {
        let updated_node = target
            .set(name.eq(new_name))
            .returning(Node::as_select())
            .get_result(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(updated_node))
    } else {
        // No changes, fetch and return
        let node = nodes
            .filter(id.eq(node_id))
            .select(Node::as_select())
            .first(&mut conn)
            .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(Json(node))
    }
}

pub async fn send_command(
    State(_state): State<AppState>,
    Extension(user): Extension<User>,
    Path(_node_id): Path<i32>,
    Json(_command): Json<NodeCommand>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    // This will send commands via WebSocket to the node
    // Implementation will be in the WebSocket module
    // For now, just acknowledge the command
    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize)]
pub struct UpdateNodeSchedulesRequest {
    pub schedule_ids: Vec<i32>,
}

pub async fn update_node_schedules(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(path_node_id): Path<i32>,
    Json(req): Json<UpdateNodeSchedulesRequest>,
) -> Result<StatusCode, StatusCode> {
    if !user.is_editor() {
        return Err(StatusCode::FORBIDDEN);
    }
    use crate::schema::node_schedules::dsl::*;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // 1. Delete all existing schedules for this node
        diesel::delete(node_schedules.filter(node_id.eq(path_node_id))).execute(conn)?;

        // 2. Insert new schedules with priority based on order
        // Index 0 (Top) gets highest priority.
        let count = req.schedule_ids.len();
        for (i, s_id) in req.schedule_ids.iter().enumerate() {
            let priority_val = (count - i) as i32;

            let new_assignment = crate::models::NewNodeSchedule {
                node_id: path_node_id,
                schedule_id: *s_id,
                priority: Some(priority_val),
            };

            diesel::insert_into(node_schedules)
                .values(&new_assignment)
                .execute(conn)?;
        }

        Ok(())
    })
    .map_err(|e| {
        tracing::error!("Failed to update node schedules: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}

pub async fn get_node_schedule(
    State(state): State<AppState>,
    Path(query_node_id): Path<i32>,
    headers: HeaderMap,
) -> Result<Json<NodeScheduleResponse>, StatusCode> {
    use crate::schema::content_items::dsl::{content_items, id as content_item_id};
    use crate::schema::schedules::dsl::{is_active, priority, schedules};
    use crate::schema::scripts::dsl::{id as script_id_col, scripts};
    use crate::services::schedule_service;

    let mut conn = state
        .db
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Auth Check
    let mut is_authorized = false;

    // 1. Check JWT (Editor/Admin access)
    if let Some(auth_header) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            if crate::auth::jwt::verify_token(token, state.config.jwt.secret.as_bytes()).is_ok() {
                is_authorized = true;
            }
        }
    }

    // 2. Check Node Secret (Node access)
    if !is_authorized {
        if let Some(secret_header) = headers.get("X-Node-Secret").and_then(|h| h.to_str().ok()) {
            use crate::schema::nodes::dsl::{
                id as node_id_col, nodes, secret_key as secret_key_col,
            };

            let node_secret: Option<String> = nodes
                .filter(node_id_col.eq(query_node_id))
                .select(secret_key_col)
                .first(&mut conn)
                .optional()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if let Some(stored_secret) = node_secret {
                // Constant time comparison would be better but standard string eq is acceptable here for now
                if stored_secret == secret_header {
                    is_authorized = true;
                }
            }
        }
    }

    if !is_authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 1. Fetch Global Timezone Setting
    use crate::schema::global_settings::dsl::{global_settings, key, value};
    let timezone_setting: Option<String> = global_settings
        .filter(key.eq("timezone"))
        .select(value)
        .first(&mut conn)
        .optional()
        .unwrap_or(None);

    // 2. Parse Timezone
    let tz: chrono_tz::Tz = timezone_setting
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(chrono_tz::UTC);

    // 3. Calculate Local Date for Facility
    let now_utc = chrono::Utc::now();
    let now_local = now_utc.with_timezone(&tz);
    let local_today = now_local.date_naive();

    // 4. Calculate Collapsed Schedule for TODAY (LOCAL)
    // The service now returns blocks in LOCAL time (HH:MM:SS) for the given date.
    let collapsed_blocks = schedule_service::calculate_collapsed_schedule(
        &mut conn,
        query_node_id,
        local_today,
        timezone_setting,
    )
    .map_err(|e| {
        tracing::error!("Failed to calculate collapsed schedule: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 5. Convert CollapsedBlocks to EffectiveBlocks for the response
    // We map the Local Time blocks back to absolute UTC time for the Node.
    let blocks: Vec<EffectiveBlock> = collapsed_blocks
        .iter()
        .enumerate()
        .map(|(idx, cb)| {
            // Parse Local Start Time
            let local_start =
                chrono::NaiveTime::parse_from_str(&cb.start_time, "%H:%M:%S").unwrap_or_default();

            // Construct Local DateTime
            let local_dt = local_today.and_time(local_start);

            // Convert to UTC
            // Note: This matches the "Facility Day" concept, but timestamps will be UTC.
            let utc_dt = local_dt.and_local_timezone(tz).unwrap().to_utc();

            EffectiveBlock {
                id: Some(idx as i32 + 1), // unique ID for frontend keys
                schedule_id: cb.schedule_id,
                content_id: cb.content_id,
                specific_date: Some(utc_dt.date_naive()), // UTC Date
                start_time: utc_dt.time(),                // UTC Time
                duration_minutes: cb.duration_minutes,
                script_id: cb.script_id,
                source_schedule_name: cb.schedule_name.clone(), // Populate from collapsed block
                dj_id: cb.dj_id,                                // Added mapping
                dj_name: cb.dj_name.clone(),
            }
        })
        .collect();

    // 3. Fetch Content Items referenced by the blocks
    let content_ids: Vec<i32> = blocks.iter().filter_map(|b| b.content_id).collect();

    let content_list = content_items
        .filter(content_item_id.eq_any(content_ids))
        .select(crate::models::ContentItem::as_select())
        .load::<crate::models::ContentItem>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Collect Script IDs (from blocks AND content transformers AND global settings)
    let mut script_ids: Vec<i32> = blocks.iter().filter_map(|b| b.script_id).collect();

    for item in &content_list {
        if let Some(transformers_json) = &item.transformer_scripts {
            if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(transformers_json) {
                for entry in entries {
                    if let Some(t_id) = entry.as_i64() {
                        script_ids.push(t_id as i32);
                    } else if let Some(obj) = entry.as_object() {
                        if let Some(id_val) = obj.get("id").or(obj.get("script_id")) {
                            if let Some(t_id) = id_val.as_i64() {
                                script_ids.push(t_id as i32);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fetch Global Active Scripts
    let global_script_json: Option<String> = global_settings
        .filter(key.eq("global_active_scripts"))
        .select(value)
        .first(&mut conn)
        .optional()
        .unwrap_or(None);

    if let Some(json_str) = global_script_json {
        if let Ok(names) = serde_json::from_str::<Vec<String>>(&json_str) {
            // Find IDs for these names
            use crate::schema::scripts::dsl::{name as script_name, scripts};
            let ids: Vec<Option<i32>> = scripts
                .filter(script_name.eq_any(names))
                .select(script_id_col)
                .load(&mut conn)
                .unwrap_or_default();

            for global_id in ids.into_iter().flatten() {
                script_ids.push(global_id);
            }
        }
    }

    script_ids.sort();
    script_ids.dedup();

    // 5. Fetch Scripts
    let fetched_scripts = scripts
        .filter(script_id_col.eq_any(script_ids))
        .select(crate::models::Script::as_select())
        .load::<crate::models::Script>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 6. determine "effective_schedule" metadata if needed (optional)
    // We can just pick the highest priority one to show name, or leave None.
    // The previous logic returned the "effective schedule" object.
    // We can try to fetch the highest priority *active* schedule just for metadata.
    use crate::schema::node_schedules::dsl::{node_id, node_schedules}; // Added import

    let primary_schedule: Option<crate::models::Schedule> = node_schedules
        .filter(node_id.eq(query_node_id))
        .inner_join(schedules)
        .select(crate::models::Schedule::as_select())
        .filter(is_active.eq(true))
        .order(priority.desc())
        .first(&mut conn)
        .optional()
        .unwrap_or(None);

    // 7. Get all assigned schedules for management UI
    // Sorted by effective priority (Override or Default) DESC
    // We reuse logic similar to schedule_service but returning just the schedules
    let assigned_data: Vec<(crate::models::Schedule, Option<i32>)> =
        crate::schema::node_schedules::dsl::node_schedules
            .inner_join(crate::schema::schedules::dsl::schedules)
            .filter(crate::schema::node_schedules::dsl::node_id.eq(query_node_id))
            .select((
                crate::models::Schedule::as_select(),
                crate::schema::node_schedules::dsl::priority,
            ))
            .load(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    struct EffectiveSchedule {
        schedule: crate::models::Schedule,
        effective_priority: i32,
    }

    let mut effective_schedules: Vec<EffectiveSchedule> = assigned_data
        .into_iter()
        .map(|(s, p_override)| {
            let eff = p_override.unwrap_or(s.priority);
            EffectiveSchedule {
                schedule: s,
                effective_priority: eff,
            }
        })
        .collect();

    effective_schedules.sort_by(|a, b| b.effective_priority.cmp(&a.effective_priority));

    let assigned_schedules_list: Vec<crate::models::Schedule> = effective_schedules
        .into_iter()
        .map(|es| es.schedule)
        .collect();

    Ok(Json(NodeScheduleResponse {
        schedule: primary_schedule,
        assigned_schedules: assigned_schedules_list,
        blocks,
        content: content_list,
        scripts: fetched_scripts,
    }))
}

pub async fn get_node_logs(
    State(state): State<AppState>,
    Path(query_node_id): Path<i32>,
) -> Result<Json<Vec<crate::models::LogEntry>>, StatusCode> {
    let logs_map = state.node_logs.read().await;

    if let Some(queue) = logs_map.get(&query_node_id) {
        Ok(Json(queue.iter().cloned().collect()))
    } else {
        Ok(Json(vec![]))
    }
}
