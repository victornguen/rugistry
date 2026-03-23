use sqlx::{PgPool, postgres::PgPoolOptions, Executor};

pub async fn run_migrations(database_url: &str) -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    let migration_sql_1 = include_str!("../../../migrations/001_initial_schema.sql");
    let migration_sql_2 = include_str!("../../../migrations/002_add_password_auth.sql");
    let migration_sql_3 = include_str!("../../../migrations/003_space_ownership.sql");
    
    // Execute migrations in order
    pool.execute(migration_sql_1).await?;
    pool.execute(migration_sql_2).await?;
    pool.execute(migration_sql_3).await?;
    
    Ok(())
}

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
