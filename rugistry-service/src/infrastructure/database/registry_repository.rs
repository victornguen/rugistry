use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{RegistryEntry, ValueType};
use crate::domain::repositories::{RegistryRepository, RepositoryError, Result};

pub struct PostgresRegistryRepository {
    pool: PgPool,
}

impl PostgresRegistryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RegistryRepository for PostgresRegistryRepository {
    async fn create(&self, entry: &RegistryEntry) -> Result<RegistryEntry> {
        let value_type_str = value_type_to_string(&entry.value_type);
        
        sqlx::query_as::<_, RegistryEntry>(
            "INSERT INTO registry_entries (id, space_id, key, value, value_type, description, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, space_id, key, value, value_type, description, created_at, updated_at"
        )
        .bind(entry.id)
        .bind(entry.space_id)
        .bind(&entry.key)
        .bind(&entry.value)
        .bind(value_type_str)
        .bind(&entry.description)
        .bind(entry.created_at)
        .bind(entry.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<RegistryEntry> {
        sqlx::query_as::<_, RegistryEntry>(
            "SELECT id, space_id, key, value, value_type, description, created_at, updated_at 
             FROM registry_entries WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Registry entry with id {} not found", id)),
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn get_by_key(&self, space_id: Uuid, key: &str) -> Result<RegistryEntry> {
        sqlx::query_as::<_, RegistryEntry>(
            "SELECT id, space_id, key, value, value_type, description, created_at, updated_at 
             FROM registry_entries WHERE space_id = $1 AND key = $2"
        )
        .bind(space_id)
        .bind(key)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                RepositoryError::NotFound(format!("Registry entry with key {} in space {} not found", key, space_id))
            }
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn list_by_space(&self, space_id: Uuid) -> Result<Vec<RegistryEntry>> {
        sqlx::query_as::<_, RegistryEntry>(
            "SELECT id, space_id, key, value, value_type, description, created_at, updated_at 
             FROM registry_entries WHERE space_id = $1 ORDER BY key"
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    async fn update(&self, entry: &RegistryEntry) -> Result<RegistryEntry> {
        let value_type_str = value_type_to_string(&entry.value_type);
        
        sqlx::query_as::<_, RegistryEntry>(
            "UPDATE registry_entries 
             SET key = $2, value = $3, value_type = $4, description = $5, updated_at = $6 
             WHERE id = $1
             RETURNING id, space_id, key, value, value_type, description, created_at, updated_at"
        )
        .bind(entry.id)
        .bind(&entry.key)
        .bind(&entry.value)
        .bind(value_type_str)
        .bind(&entry.description)
        .bind(chrono::Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(format!("Registry entry with id {} not found", entry.id)),
            _ => RepositoryError::DatabaseError(e.to_string()),
        })
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM registry_entries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Registry entry with id {} not found", id)));
        }

        Ok(())
    }
}

fn value_type_to_string(value_type: &ValueType) -> &str {
    match value_type {
        ValueType::String => "string",
        ValueType::Number => "number",
        ValueType::Boolean => "boolean",
        ValueType::Json => "json",
        ValueType::List => "list",
        ValueType::Hocon => "hocon",
        ValueType::Toml => "toml",
        ValueType::Yaml => "yaml",
    }
}

fn string_to_value_type(s: &str) -> ValueType {
    match s {
        "string" => ValueType::String,
        "number" => ValueType::Number,
        "boolean" => ValueType::Boolean,
        "json" => ValueType::Json,
        "list" => ValueType::List,
        "hocon" => ValueType::Hocon,
        "toml" => ValueType::Toml,
        "yaml" => ValueType::Yaml,
        _ => ValueType::String,
    }
}

// Implement FromRow for RegistryEntry
impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for RegistryEntry {
    fn from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
        use sqlx::Row;
        let value_type_str: String = row.try_get("value_type")?;
        Ok(RegistryEntry {
            id: row.try_get("id")?,
            space_id: row.try_get("space_id")?,
            key: row.try_get("key")?,
            value: row.try_get("value")?,
            value_type: string_to_value_type(&value_type_str),
            description: row.try_get("description")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
