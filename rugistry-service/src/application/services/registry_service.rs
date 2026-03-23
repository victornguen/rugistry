use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{
    ChangeEventType, ChangeNotification, CreateRegistryEntryRequest, RegistryEntryResponse,
    UpdateRegistryEntryRequest,
};
use crate::domain::entities::{RegistryEntry, ValueType};
use crate::domain::repositories::RegistryRepository;
use crate::infrastructure::events::EventBus;

pub struct RegistryService {
    repository: Arc<dyn RegistryRepository>,
    event_bus: Arc<EventBus>,
}

impl RegistryService {
    pub fn new(repository: Arc<dyn RegistryRepository>, event_bus: Arc<EventBus>) -> Self {
        Self {
            repository,
            event_bus,
        }
    }

    pub async fn create_entry(
        &self,
        req: CreateRegistryEntryRequest,
    ) -> anyhow::Result<RegistryEntryResponse> {
        validate_value(&req.value, &req.value_type)?;

        let entry = RegistryEntry::new(
            req.space_id,
            req.key.clone(),
            req.value,
            req.value_type,
            req.description,
        );

        let created = self.repository.create(&entry).await?;
        let response = entry_to_response(created);

        self.event_bus.publish(ChangeNotification {
            event_type: ChangeEventType::Created,
            space_id: response.space_id,
            entry_id: Some(response.id),
            key: Some(response.key.clone()),
            entry: Some(response.clone()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        Ok(response)
    }

    pub async fn get_entry(&self, id: Uuid) -> anyhow::Result<RegistryEntryResponse> {
        let entry = self.repository.get_by_id(id).await?;
        Ok(entry_to_response(entry))
    }

    pub async fn get_entry_by_key(
        &self,
        space_id: Uuid,
        key: &str,
    ) -> anyhow::Result<RegistryEntryResponse> {
        let entry = self.repository.get_by_key(space_id, key).await?;
        Ok(entry_to_response(entry))
    }

    pub async fn list_entries_by_space(
        &self,
        space_id: Uuid,
    ) -> anyhow::Result<Vec<RegistryEntryResponse>> {
        let entries = self.repository.list_by_space(space_id).await?;
        Ok(entries.into_iter().map(entry_to_response).collect())
    }

    pub async fn update_entry(
        &self,
        id: Uuid,
        req: UpdateRegistryEntryRequest,
    ) -> anyhow::Result<RegistryEntryResponse> {
        let mut entry = self.repository.get_by_id(id).await?;

        if let Some(key) = req.key {
            entry.key = key;
        }
        if let Some(value_type) = req.value_type {
            entry.value_type = value_type;
        }
        if let Some(value) = req.value {
            validate_value(&value, &entry.value_type)?;
            entry.value = value;
        }
        if req.description.is_some() {
            entry.description = req.description;
        }

        entry.updated_at = chrono::Utc::now();
        let updated = self.repository.update(&entry).await?;
        let response = entry_to_response(updated);

        self.event_bus.publish(ChangeNotification {
            event_type: ChangeEventType::Updated,
            space_id: response.space_id,
            entry_id: Some(response.id),
            key: Some(response.key.clone()),
            entry: Some(response.clone()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        Ok(response)
    }

    pub async fn delete_entry(&self, id: Uuid) -> anyhow::Result<()> {
        let entry = self.repository.get_by_id(id).await?;
        self.repository.delete(id).await?;

        self.event_bus.publish(ChangeNotification {
            event_type: ChangeEventType::Deleted,
            space_id: entry.space_id,
            entry_id: Some(id),
            key: Some(entry.key),
            entry: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        Ok(())
    }
}

pub fn validate_value(value: &str, value_type: &ValueType) -> anyhow::Result<()> {
    match value_type {
        ValueType::Json => {
            serde_json::from_str::<serde_json::Value>(value)
                .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;
        }
        ValueType::List => {
            let parsed = serde_json::from_str::<serde_json::Value>(value)
                .map_err(|e| anyhow::anyhow!("Invalid list (must be JSON array): {}", e))?;
            if !parsed.is_array() {
                anyhow::bail!("List value must be a JSON array");
            }
        }
        ValueType::Toml => {
            value.parse::<toml::Value>()
                .map_err(|e| anyhow::anyhow!("Invalid TOML: {}", e))?;
        }
        ValueType::Yaml => {
            serde_yaml::from_str::<serde_yaml::Value>(value)
                .map_err(|e| anyhow::anyhow!("Invalid YAML: {}", e))?;
        }
        ValueType::Hocon => {
            hocon::HoconLoader::new()
                .load_str(value)
                .map_err(|e| anyhow::anyhow!("Invalid HOCON: {}", e))?
                .hocon()
                .map_err(|e| anyhow::anyhow!("Invalid HOCON: {}", e))?;
        }
        ValueType::Number => {
            value.parse::<f64>()
                .map_err(|_| anyhow::anyhow!("Invalid number: '{}'", value))?;
        }
        ValueType::Boolean => {
            if value != "true" && value != "false" {
                anyhow::bail!("Boolean value must be 'true' or 'false'");
            }
        }
        ValueType::String => {}
    }
    Ok(())
}

fn entry_to_response(entry: RegistryEntry) -> RegistryEntryResponse {
    RegistryEntryResponse {
        id: entry.id,
        space_id: entry.space_id,
        key: entry.key,
        value: entry.value,
        value_type: entry.value_type,
        description: entry.description,
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
    }
}

