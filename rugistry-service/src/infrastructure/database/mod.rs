pub mod migrations;
pub mod registry_repository;
pub mod space_repository;

use sqlx::PgPool;

pub async fn init_database(database_url: &str) -> anyhow::Result<PgPool> {
    // Run migrations
    migrations::run_migrations(database_url).await?;
    
    // Create connection pool
    let pool = migrations::create_pool(database_url).await?;
    
    Ok(pool)
}
