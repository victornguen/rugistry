use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::application::services::{RegistryService, SpaceService};
use crate::infrastructure::{auth::{auth_middleware, AuthConfig}, events::EventBus};
use crate::presentation::handlers;

// Combined application state
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub space_service: Arc<SpaceService>,
    pub registry_service: Arc<RegistryService>,
    pub event_bus: Arc<EventBus>,
    pub auth_config: Arc<AuthConfig>,
}

pub fn create_router(
    pool: PgPool,
    space_service: Arc<SpaceService>,
    registry_service: Arc<RegistryService>,
    event_bus: Arc<EventBus>,
    auth_config: Arc<AuthConfig>,
) -> Router {
    let state = AppState {
        pool,
        space_service,
        registry_service,
        event_bus: event_bus.clone(),
        auth_config: auth_config.clone(),
    };

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/v1/auth/register", post(handlers::auth_handlers::register))
        .route("/api/v1/auth/login", post(handlers::auth_handlers::login));

    // Protected space routes (require authentication)
    let space_routes = Router::new()
        .route("/api/v1/spaces", post(handlers::create_space))
        .route("/api/v1/spaces", get(handlers::list_spaces))
        .route("/api/v1/spaces/:id", get(handlers::get_space))
        .route("/api/v1/spaces/name/:name", get(handlers::get_space_by_name))
        .route("/api/v1/spaces/:id", put(handlers::update_space))
        .route("/api/v1/spaces/:id", delete(handlers::delete_space))
        .route("/api/v1/spaces/:id/shares", post(handlers::add_share))
        .route("/api/v1/spaces/:id/shares", get(handlers::get_shares))
        .route("/api/v1/spaces/:id/shares/:user_id", delete(handlers::remove_share))
        .route("/api/v1/users/search", get(handlers::search_users))
        .route_layer(middleware::from_fn_with_state(
            state.auth_config.clone(),
            auth_middleware,
        ));

    // Protected registry routes (require authentication)
    let registry_routes = Router::new()
        .route("/api/v1/entries", post(handlers::create_entry))
        .route("/api/v1/entries/:id", get(handlers::get_entry))
        .route("/api/v1/entries/:id", put(handlers::update_entry))
        .route("/api/v1/entries/:id", delete(handlers::delete_entry))
        .route("/api/v1/spaces/:space_id/entries", get(handlers::list_entries_by_space))
        .route("/api/v1/spaces/:space_id/entries/key", get(handlers::get_entry_by_key))
        .route_layer(middleware::from_fn_with_state(
            state.auth_config.clone(),
            auth_middleware,
        ));

    // WebSocket routes (require authentication)
    let ws_routes = Router::new()
        .route("/api/v1/ws/:space_id", get(handlers::ws_handler))
        .route_layer(middleware::from_fn_with_state(
            state.auth_config.clone(),
            auth_middleware,
        ));

    // Merge all routes
    Router::new()
        .merge(public_routes)
        .merge(space_routes)
        .merge(registry_routes)
        .merge(ws_routes)
        .with_state(state)
}
