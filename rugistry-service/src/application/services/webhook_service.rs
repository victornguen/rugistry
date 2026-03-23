use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{ChangeNotification, CreateWebhookRequest, WebhookResponse};
use crate::domain::entities::Webhook;
use crate::domain::repositories::WebhookRepository;

pub struct WebhookService {
    repository: Arc<dyn WebhookRepository>,
}

impl WebhookService {
    pub fn new(repository: Arc<dyn WebhookRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_webhook(
        &self,
        space_id: Uuid,
        req: CreateWebhookRequest,
    ) -> anyhow::Result<WebhookResponse> {
        let webhook = Webhook::new(space_id, req.url, req.secret);
        let created = self.repository.create(&webhook).await?;
        Ok(to_response(created))
    }

    pub async fn list_webhooks(&self, space_id: Uuid) -> anyhow::Result<Vec<WebhookResponse>> {
        let webhooks = self.repository.list_by_space(space_id).await?;
        Ok(webhooks.into_iter().map(to_response).collect())
    }

    pub async fn delete_webhook(&self, id: Uuid, space_id: Uuid) -> anyhow::Result<()> {
        let webhook = self.repository.get_by_id(id).await?;
        if webhook.space_id != space_id {
            anyhow::bail!("Webhook not found");
        }
        self.repository.delete(id).await?;
        Ok(())
    }

    /// Called by the background EventBus subscriber to deliver events to all registered webhooks.
    pub async fn fire_webhooks(&self, notification: &ChangeNotification) {
        let webhooks = match self.repository.list_by_space(notification.space_id).await {
            Ok(w) => w,
            Err(_) => return,
        };
        if webhooks.is_empty() {
            return;
        }

        let payload = match serde_json::to_string(notification) {
            Ok(p) => p,
            Err(_) => return,
        };

        let client = reqwest::Client::new();

        for webhook in webhooks {
            let client = client.clone();
            let url = webhook.url.clone();
            let payload = payload.clone();
            let secret = webhook.secret.clone();

            tokio::spawn(async move {
                let mut req = client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .header("X-Rugistry-Event", "change");

                if let Some(s) = secret {
                    req = req.header("X-Webhook-Secret", s);
                }

                let _ = req
                    .body(payload)
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await;
            });
        }
    }
}

fn to_response(w: Webhook) -> WebhookResponse {
    WebhookResponse {
        id: w.id.to_string(),
        space_id: w.space_id.to_string(),
        url: w.url,
        has_secret: w.secret.is_some(),
        created_at: w.created_at.to_rfc3339(),
    }
}
