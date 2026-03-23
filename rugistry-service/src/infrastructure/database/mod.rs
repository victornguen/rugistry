pub mod migrations;
pub mod registry_repository;
pub mod space_repository;
pub mod webhook_repository;

use sqlx::PgPool;

pub async fn init_database(database_url: &str) -> anyhow::Result<PgPool> {
    migrations::create_pool(database_url).await
}
