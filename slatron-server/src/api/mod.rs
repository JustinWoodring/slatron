pub mod auth_api;
pub mod content_api;
pub mod dj_api;
pub mod nodes_api;
pub mod permissions_api;
pub mod schedules_api;
pub mod scripts_api;
pub mod settings_api;
pub mod users_api;

use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn routes(state: AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        // Protected routes
        .route("/schedules", get(schedules_api::list_schedules))
        .route("/schedules", post(schedules_api::create_schedule))
        .route("/schedules/:id", put(schedules_api::update_schedule))
        .route("/schedules/:id", delete(schedules_api::delete_schedule))
        .route(
            "/schedules/:id/blocks",
            get(schedules_api::get_schedule_blocks),
        )
        .route(
            "/schedules/:id/blocks",
            post(schedules_api::create_schedule_block),
        )
        .route(
            "/schedules/:schedule_id/blocks/:block_id",
            put(schedules_api::update_schedule_block),
        )
        .route(
            "/schedules/:schedule_id/blocks/:block_id",
            delete(schedules_api::delete_schedule_block),
        )
        .route(
            "/schedules/collapsed",
            get(schedules_api::get_collapsed_schedule),
        )
        // Content
        .route("/content", get(content_api::list_content))
        .route("/content", post(content_api::create_content))
        .route("/content/:id", put(content_api::update_content))
        .route("/content/:id", delete(content_api::delete_content))
        // Nodes
        .route("/nodes", get(nodes_api::list_nodes))
        .route("/nodes", post(nodes_api::create_node))
        .route(
            "/nodes/:id",
            delete(nodes_api::delete_node).put(nodes_api::update_node),
        )
        .route("/nodes/:id/command", post(nodes_api::send_command))
        .route(
            "/nodes/:id/schedules",
            put(nodes_api::update_node_schedules),
        )
        .route("/nodes/:id/logs", get(nodes_api::get_node_logs))
        // Scripts
        .route("/scripts", get(scripts_api::list_scripts))
        .route("/scripts", post(scripts_api::create_script))
        .route("/scripts/:id", put(scripts_api::update_script))
        .route("/scripts/:id", delete(scripts_api::delete_script))
        .route("/scripts/:id/validate", post(scripts_api::validate_script))
        .route("/scripts/:id/execute", post(scripts_api::execute_script))
        // Users
        .route("/users", get(users_api::list_users))
        .route("/users", post(users_api::create_user))
        .route("/users/:id", put(users_api::update_user))
        .route("/users/:id", delete(users_api::delete_user))
        // Permissions
        .route("/permissions", get(permissions_api::list_permissions))
        .route("/permissions", post(permissions_api::create_permission))
        .route(
            "/permissions/:id",
            delete(permissions_api::delete_permission),
        )
        // Settings
        // Settings
        .route("/settings/:key", put(settings_api::update_setting))
        // DJ / AI Routes
        .route("/djs", get(dj_api::list_djs))
        .route("/djs", post(dj_api::create_dj))
        .route("/djs/:id", get(dj_api::get_dj).put(dj_api::update_dj))
        .route("/djs/:id", delete(dj_api::delete_dj))
        .route("/ai-providers", get(dj_api::list_ai_providers))
        .route("/ai-providers", post(dj_api::create_ai_provider))
        .route("/ai-providers/:id", put(dj_api::update_ai_provider))
        .route("/ai-providers/:id", delete(dj_api::delete_ai_provider))
        // DJ Memories
        .route("/djs/:id/memories", get(dj_api::get_dj_memories))
        .route("/djs/:id/memories", post(dj_api::create_dj_memory))
        .route("/memories/:id", put(dj_api::update_dj_memory))
        .route("/memories/:id", delete(dj_api::delete_dj_memory))
        .route_layer(middleware::from_fn_with_state(
            state,
            crate::auth::middleware::auth_middleware,
        ));

    Router::new()
        // Public auth endpoints
        .route("/auth/login", post(auth_api::login))
        .route("/auth/logout", post(auth_api::logout))
        .route("/nodes/:id/schedule", get(nodes_api::get_node_schedule))
        .route("/settings", get(settings_api::list_settings))
        .route(
            "/system/capabilities",
            get(settings_api::get_system_capabilities),
        )
        .merge(protected_routes)
}
