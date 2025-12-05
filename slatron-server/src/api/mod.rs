pub mod auth_api;
pub mod content_api;
pub mod nodes_api;
pub mod permissions_api;
pub mod schedules_api;
pub mod scripts_api;
pub mod users_api;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use crate::AppState;
use crate::auth::middleware::auth_middleware;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Public auth endpoints
        .route("/auth/login", post(auth_api::login))
        .route("/auth/logout", post(auth_api::logout))
        // Protected routes
        .route("/schedules", get(schedules_api::list_schedules))
        .route("/schedules", post(schedules_api::create_schedule))
        .route("/schedules/:id", put(schedules_api::update_schedule))
        .route("/schedules/:id", delete(schedules_api::delete_schedule))
        .route("/schedules/:id/blocks", get(schedules_api::get_schedule_blocks))
        .route("/schedules/:id/blocks", post(schedules_api::create_schedule_block))
        .route("/schedules/:schedule_id/blocks/:block_id", put(schedules_api::update_schedule_block))
        .route("/schedules/:schedule_id/blocks/:block_id", delete(schedules_api::delete_schedule_block))
        .route("/schedules/collapsed", get(schedules_api::get_collapsed_schedule))
        // Content
        .route("/content", get(content_api::list_content))
        .route("/content", post(content_api::create_content))
        .route("/content/:id", put(content_api::update_content))
        .route("/content/:id", delete(content_api::delete_content))
        // Nodes
        .route("/nodes", get(nodes_api::list_nodes))
        .route("/nodes", post(nodes_api::create_node))
        .route("/nodes/:id", delete(nodes_api::delete_node))
        .route("/nodes/:id/command", post(nodes_api::send_command))
        // Scripts
        .route("/scripts", get(scripts_api::list_scripts))
        .route("/scripts", post(scripts_api::create_script))
        .route("/scripts/:id", put(scripts_api::update_script))
        .route("/scripts/:id", delete(scripts_api::delete_script))
        .route("/scripts/:id/validate", post(scripts_api::validate_script))
        // Users
        .route("/users", get(users_api::list_users))
        .route("/users", post(users_api::create_user))
        .route("/users/:id", put(users_api::update_user))
        .route("/users/:id", delete(users_api::delete_user))
        // Permissions
        .route("/permissions", get(permissions_api::list_permissions))
        .route("/permissions", post(permissions_api::create_permission))
        .route("/permissions/:id", delete(permissions_api::delete_permission))
}
