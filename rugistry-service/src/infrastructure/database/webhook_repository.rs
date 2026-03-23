use async_trait::async_trait;
use chrono::DateTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Webhook;
use crate::domain::repositories::{RepositoryError, Result, WebhookRepository};

pub struct PostgresWebhookRepository {
    pool: PgPool,
}

impl PostgresWebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

type WebhookRow = (Uuid, Uuid, String, Option<String>, DateTime<chrono::Utc>, DateTime<chrono::Utc>);

fn row_to_webhook(r: WebhookRow) -> Webhook {
    Webhook { id: r.0, space_id: r.1, url: r.2, secret: r.3, created_at: r.4, updated_at: r.5 }
}

#[async_trait]
impl WebhookRepository for PostgresWebhookRepository {
    async fn create(&self, webhook: &Webhook) -> Result<Webhook> {
        let row = sqlx::query_as::<_, WebhookRow>(
            "INSERT INTO space_webhooks (id, space_id, url, secret, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, space_id, url, secret, created_at, updated_at",
        )
        .bind(webhook.id)
        .bind(webhook.space_id)
        .bind(&webhook.url)
        .bind(&webhook.secret)
        .bind(webhook.created_at)
        .bind(webhook.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(row_to_webhook(row))
    }

    async fn list_by_space(&self, space_id: Uuid) -> Result<Vec<Webhook>> {
        let rows = sqlx::query_as::<_, WebhookRow>(
            "SELECT id, space_id, url, secret, created_at, updated_at
             FROM space_webhooks WHERE space_id = $1 ORDER BY created_at",
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_webhook).collect())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Webhook> {
        let row = sqlx::query_as::<_, WebhookRow>(
            "SELECT id, space_id, url, secret, created_at, updated_at
             FROM space_webhooks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
        .ok_or_else(|| RepositoryError::NotFound(format!("Webhook {}", id)))?;
        Ok(row_to_webhook(row))
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM space_webhooks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
