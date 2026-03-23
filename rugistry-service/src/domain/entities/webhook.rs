use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Webhook {
    pub id: Uuid,
    pub space_id: Uuid,
    pub url: String,
    pub secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Webhook {
    pub fn new(space_id: Uuid, url: String, secret: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            space_id,
            url,
            secret,
            created_at: now,
            updated_at: now,
        }
    }
}
