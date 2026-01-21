use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Space;
use crate::domain::repositories::{RepositoryError, Result, SpaceRepository};

pub struct PostgresSpaceRepository {
    pool: PgPool,
}

impl PostgresSpaceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SpaceRepository for PostgresSpaceRepository {
    async fn create(&self, space: &Space) -> Result<Space> {
        sqlx::query_as::<_, Space>(
            "INSERT INTO spaces (id, name, description, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, description, created_at, updated_at"
        )
        .bind(space.id)
        .bind(&space.name)
        .bind(&space.description)
        .bind(space.created_at)
        .bind(space.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Space> {
        sqlx::query_as::<_, Space>(
            "SELECT id, name, description, created_at, updated_at FROM spaces WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Space with id {} not found", id)),
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn get_by_name(&self, name: &str) -> Result<Space> {
        sqlx::query_as::<_, Space>(
            "SELECT id, name, description, created_at, updated_at FROM spaces WHERE name = $1"
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Space with name {} not found", name)),
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn list(&self) -> Result<Vec<Space>> {
        sqlx::query_as::<_, Space>(
            "SELECT id, name, description, created_at, updated_at FROM spaces ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn update(&self, space: &Space) -> Result<Space> {
        sqlx::query_as::<_, Space>(
            "UPDATE spaces SET name = $2, description = $3, updated_at = $4 
             WHERE id = $1
             RETURNING id, name, description, created_at, updated_at"
        )
        .bind(space.id)
        .bind(&space.name)
        .bind(&space.description)
        .bind(chrono::Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Space with id {} not found", space.id)),
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM spaces WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Space with id {} not found", id)));
        }

        Ok(())
    }
}

// Implement FromRow for Space
impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Space {
    fn from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Space {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
