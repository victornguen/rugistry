use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a registry entry with key-value pair in a specific space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: Uuid,
    pub space_id: Uuid,
    pub key: String,
    pub value: String,
    pub value_type: ValueType,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Json,
}

impl RegistryEntry {
    pub fn new(
        space_id: Uuid,
        key: String,
        value: String,
        value_type: ValueType,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            space_id,
            key,
            value,
            value_type,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}
