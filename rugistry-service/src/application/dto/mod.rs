use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ValueType;

// Space DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSpaceRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSpaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    /// null = current user is owner; "readonly"/"write"/"appendonly" = shared access
    pub permission: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareSpaceRequest {
    pub username: String,
    pub permission: String, // "readonly", "write", "appendonly"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceShareResponse {
    pub user_id: String,
    pub username: String,
    pub permission: String,
    pub created_at: String,
}

// Registry Entry DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRegistryEntryRequest {
    pub space_id: Uuid,
    pub key: String,
    pub value: String,
    pub value_type: ValueType,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRegistryEntryRequest {
    pub key: Option<String>,
    pub value: Option<String>,
    pub value_type: Option<ValueType>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntryResponse {
    pub id: Uuid,
    pub space_id: Uuid,
    pub key: String,
    pub value: String,
    pub value_type: ValueType,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// Change notification for WebSocket/SSE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeNotification {
    pub event_type: ChangeEventType,
    pub space_id: Uuid,
    pub entry_id: Option<Uuid>,
    pub key: Option<String>,
    pub entry: Option<RegistryEntryResponse>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeEventType {
    Created,
    Updated,
    Deleted,
}

// Webhook DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub id: String,
    pub space_id: String,
    pub url: String,
    pub has_secret: bool,
    pub created_at: String,
}

