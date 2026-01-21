use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{
    ChangeEventType, ChangeNotification, CreateRegistryEntryRequest, RegistryEntryResponse,
    UpdateRegistryEntryRequest,
};
use crate::domain::{entities::RegistryEntry, repositories::RegistryRepository};
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
        let entry = RegistryEntry::new(
            req.space_id,
            req.key.clone(),
            req.value,
            req.value_type,
            req.description,
        );
        
        let created = self.repository.create(&entry).await?;
        let response = entry_to_response(created);
        
        // Notify subscribers with full entry data
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
        if let Some(value) = req.value {
            entry.value = value;
        }
        if let Some(value_type) = req.value_type {
            entry.value_type = value_type;
        }
        if req.description.is_some() {
            entry.description = req.description;
        }
        
        entry.updated_at = chrono::Utc::now();
        let updated = self.repository.update(&entry).await?;
        let response = entry_to_response(updated);
        
        // Notify subscribers with full entry data
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
        
        // Notify subscribers
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
