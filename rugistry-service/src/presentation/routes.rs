use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::application::services::{RegistryService, SpaceService};
use crate::infrastructure::events::EventBus;
use crate::presentation::handlers;

pub fn create_router(
    space_service: Arc<SpaceService>,
    registry_service: Arc<RegistryService>,
    event_bus: Arc<EventBus>,
) -> Router {
    // Create separate routers for different services
    let space_routes = Router::new()
        .route("/api/spaces", post(handlers::create_space))
        .route("/api/spaces", get(handlers::list_spaces))
        .route("/api/spaces/:id", get(handlers::get_space))
        .route("/api/spaces/name/:name", get(handlers::get_space_by_name))
        .route("/api/spaces/:id", put(handlers::update_space))
        .route("/api/spaces/:id", delete(handlers::delete_space))
        .with_state(space_service);

    let registry_routes = Router::new()
        .route("/api/entries", post(handlers::create_entry))
        .route("/api/entries/:id", get(handlers::get_entry))
        .route("/api/entries/:id", put(handlers::update_entry))
        .route("/api/entries/:id", delete(handlers::delete_entry))
        .route("/api/spaces/:space_id/entries", get(handlers::list_entries_by_space))
        .route("/api/spaces/:space_id/entries/key", get(handlers::get_entry_by_key))
        .with_state(registry_service);

    let ws_routes = Router::new()
        .route("/api/ws/:space_id", get(handlers::ws_handler))
        .with_state(event_bus);

    // Merge all routes
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(space_routes)
        .merge(registry_routes)
        .merge(ws_routes)
}
