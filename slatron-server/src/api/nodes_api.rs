use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::{NewNode, Node};
use crate::AppState;

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
    pub action: String,
    pub position_secs: Option<f64>,
    pub content_id: Option<i32>,
}

pub async fn list_nodes(
    State(state): State<AppState>,
) -> Result<Json<Vec<Node>>, StatusCode> {
    use crate::schema::nodes::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = nodes
        .select(Node::as_select())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_node(
    State(state): State<AppState>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<CreateNodeResponse>, StatusCode> {
    use crate::schema::nodes;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        .returning(Node::as_returning())
        .get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateNodeResponse {
        node,
        secret_key,
    }))
}

pub async fn delete_node(
    State(state): State<AppState>,
    Path(node_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    use crate::schema::nodes::dsl::*;

    let mut conn = state.db.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(nodes.find(node_id))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn send_command(
    State(_state): State<AppState>,
    Path(_node_id): Path<i32>,
    Json(_command): Json<NodeCommand>,
) -> Result<StatusCode, StatusCode> {
    // This will send commands via WebSocket to the node
    // Implementation will be in the WebSocket module
    // For now, just acknowledge the command
    Ok(StatusCode::ACCEPTED)
}
