use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Space;
use crate::domain::repositories::{RepositoryError, Result, SpaceRepository, SpaceShare};

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
            "INSERT INTO spaces (id, name, description, owner_id, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, owner_id, created_at, updated_at"
        )
        .bind(space.id)
        .bind(&space.name)
        .bind(&space.description)
        .bind(space.owner_id)
        .bind(space.created_at)
        .bind(space.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Space> {
        sqlx::query_as::<_, Space>(
            "SELECT id, name, description, owner_id, created_at, updated_at FROM spaces WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Space {} not found", id)),
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn get_by_name(&self, name: &str) -> Result<Space> {
        sqlx::query_as::<_, Space>(
            "SELECT id, name, description, owner_id, created_at, updated_at FROM spaces WHERE name = $1"
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Space '{}' not found", name)),
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn list(&self) -> Result<Vec<Space>> {
        sqlx::query_as::<_, Space>(
            "SELECT id, name, description, owner_id, created_at, updated_at FROM spaces ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<(Space, Option<String>)>> {
        // Owned spaces (permission = NULL means owner)
        struct SpaceUserRow {
            id: Uuid,
            name: String,
            description: Option<String>,
            owner_id: Option<Uuid>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            permission: Option<String>,
        }
        impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for SpaceUserRow {
            fn from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
                use sqlx::Row;
                Ok(SpaceUserRow {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    owner_id: row.try_get("owner_id")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                    permission: row.try_get("permission")?,
                })
            }
        }

        let rows = sqlx::query_as::<_, SpaceUserRow>(
            "SELECT s.id, s.name, s.description, s.owner_id, s.created_at, s.updated_at,
                    NULL::VARCHAR AS permission
             FROM spaces s
             WHERE s.owner_id = $1
             UNION ALL
             SELECT s.id, s.name, s.description, s.owner_id, s.created_at, s.updated_at,
                    sh.permission
             FROM spaces s
             JOIN space_shares sh ON sh.space_id = s.id
             WHERE sh.user_id = $1
             ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| {
            let space = Space {
                id: r.id,
                name: r.name,
                description: r.description,
                owner_id: r.owner_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            (space, r.permission)
        }).collect())
    }

    async fn update(&self, space: &Space) -> Result<Space> {
        sqlx::query_as::<_, Space>(
            "UPDATE spaces SET name = $2, description = $3, updated_at = $4 
             WHERE id = $1
             RETURNING id, name, description, owner_id, created_at, updated_at"
        )
        .bind(space.id)
        .bind(&space.name)
        .bind(&space.description)
        .bind(chrono::Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Space {} not found", space.id)),
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
            return Err(RepositoryError::NotFound(format!("Space {} not found", id)));
        }
        Ok(())
    }

    async fn add_share(&self, space_id: Uuid, user_id: Uuid, permission: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO space_shares (space_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (space_id, user_id) DO UPDATE SET permission = $3"
        )
        .bind(space_id)
        .bind(user_id)
        .bind(permission)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn remove_share(&self, space_id: Uuid, shared_user_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM space_shares WHERE space_id = $1 AND user_id = $2"
        )
        .bind(space_id)
        .bind(shared_user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound("Share not found".to_string()));
        }
        Ok(())
    }

    async fn get_shares(&self, space_id: Uuid) -> Result<Vec<SpaceShare>> {
        struct ShareRow {
            space_id: Uuid,
            user_id: Uuid,
            username: String,
            permission: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }
        impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for ShareRow {
            fn from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
                use sqlx::Row;
                Ok(ShareRow {
                    space_id: row.try_get("space_id")?,
                    user_id: row.try_get("user_id")?,
                    username: row.try_get("username")?,
                    permission: row.try_get("permission")?,
                    created_at: row.try_get("created_at")?,
                })
            }
        }

        let rows = sqlx::query_as::<_, ShareRow>(
            "SELECT sh.space_id, sh.user_id, u.username, sh.permission, sh.created_at
             FROM space_shares sh
             JOIN users u ON u.id = sh.user_id
             WHERE sh.space_id = $1
             ORDER BY sh.created_at"
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| SpaceShare {
            space_id: r.space_id,
            user_id: r.user_id,
            username: r.username,
            permission: r.permission,
            created_at: r.created_at,
        }).collect())
    }

    async fn get_user_permission(&self, space_id: Uuid, user_id: Uuid) -> Result<Option<String>> {
        // Check ownership first
        let owner: Option<(Option<Uuid>,)> = sqlx::query_as("SELECT owner_id FROM spaces WHERE id = $1")
            .bind(space_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match owner {
            None => return Err(RepositoryError::NotFound(format!("Space {} not found", space_id))),
            Some((Some(owner_id),)) if owner_id == user_id => return Ok(None), // owner
            Some((None,)) => return Ok(None), // legacy space without owner — allow all
            _ => {}
        }

        // Check share
        let share: Option<(String,)> = sqlx::query_as(
            "SELECT permission FROM space_shares WHERE space_id = $1 AND user_id = $2"
        )
        .bind(space_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match share {
            Some((perm,)) => Ok(Some(perm)),
            None => Err(RepositoryError::PermissionDenied(
                format!("User has no access to space {}", space_id)
            )),
        }
    }

    async fn get_owner_id(&self, space_id: Uuid) -> Result<Option<Uuid>> {
        let row: Option<(Option<Uuid>,)> = sqlx::query_as("SELECT owner_id FROM spaces WHERE id = $1")
            .bind(space_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            None => Err(RepositoryError::NotFound(format!("Space {} not found", space_id))),
            Some((owner_id,)) => Ok(owner_id),
        }
    }

    async fn get_user_id_by_username(&self, username: &str) -> Result<Uuid> {
        let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        row.map(|(id,)| id)
            .ok_or_else(|| RepositoryError::NotFound(format!("User '{}' not found", username)))
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
            owner_id: row.try_get("owner_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

