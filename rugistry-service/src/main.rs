mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use infrastructure::{
    auth::AuthConfig,
    database::{init_database, registry_repository::PostgresRegistryRepository, space_repository::PostgresSpaceRepository, webhook_repository::PostgresWebhookRepository},
    events::EventBus,
};
use application::services::{RegistryService, SpaceService, WebhookService};
use presentation::routes::create_router;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize auth config and fetch JWKS
    let auth_config = AuthConfig::from_env();
    tracing::info!("Authentication initialized");
    let auth_config = Arc::new(auth_config);

    // Get database URL
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    tracing::info!("Connecting to database...");
    let pool = init_database(&database_url).await?;
    tracing::info!("Database initialized successfully");

    // Create repositories
    let space_repo = Arc::new(PostgresSpaceRepository::new(pool.clone()));
    let registry_repo = Arc::new(PostgresRegistryRepository::new(pool.clone()));
    let webhook_repo = Arc::new(PostgresWebhookRepository::new(pool.clone()));

    // Create event bus
    let event_bus = Arc::new(EventBus::new(100));

    // Create services
    let space_service = Arc::new(SpaceService::new(space_repo));
    let registry_service = Arc::new(RegistryService::new(registry_repo, event_bus.clone()));
    let webhook_service = Arc::new(WebhookService::new(webhook_repo));

    // Spawn background task to deliver webhook events
    {
        let webhook_service = webhook_service.clone();
        let mut receiver = event_bus.subscribe();
        tokio::spawn(async move {
            while let Ok(notification) = receiver.recv().await {
                webhook_service.fire_webhooks(&notification).await;
            }
        });
    }

    // Create router with CORS
    let app = create_router(pool, space_service, registry_service, webhook_service, event_bus, auth_config)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Get host and port from environment or use defaults
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    
    tracing::info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
