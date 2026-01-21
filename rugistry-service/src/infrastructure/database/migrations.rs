use sqlx::{PgPool, postgres::PgPoolOptions, Executor};

pub async fn run_migrations(database_url: &str) -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    let migration_sql = include_str!("../../../migrations/001_initial_schema.sql");
    
    // Execute using execute_many which handles multiple statements
    pool.execute(migration_sql).await?;
    
    Ok(())
}

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
