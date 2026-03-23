use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::{RegistryEntry, Space};

pub type Result<T> = std::result::Result<T, RepositoryError>;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

#[derive(Debug, Clone)]
pub struct SpaceShare {
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub permission: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait SpaceRepository: Send + Sync {
    async fn create(&self, space: &Space) -> Result<Space>;
    async fn get_by_id(&self, id: Uuid) -> Result<Space>;
    async fn get_by_name(&self, name: &str) -> Result<Space>;
    async fn list(&self) -> Result<Vec<Space>>;
    /// Returns spaces accessible to user: owned + shared, with optional permission string (None = owner)
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<(Space, Option<String>)>>;
    async fn update(&self, space: &Space) -> Result<Space>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn add_share(&self, space_id: Uuid, user_id: Uuid, permission: &str) -> Result<()>;
    async fn remove_share(&self, space_id: Uuid, shared_user_id: Uuid) -> Result<()>;
    async fn get_shares(&self, space_id: Uuid) -> Result<Vec<SpaceShare>>;
    /// Returns None if user is owner, Some(permission) if shared, Err(NotFound) if no access
    async fn get_user_permission(&self, space_id: Uuid, user_id: Uuid) -> Result<Option<String>>;
    async fn get_owner_id(&self, space_id: Uuid) -> Result<Option<Uuid>>;
    async fn get_user_id_by_username(&self, username: &str) -> Result<Uuid>;
}

#[async_trait]
pub trait RegistryRepository: Send + Sync {
    async fn create(&self, entry: &RegistryEntry) -> Result<RegistryEntry>;
    async fn get_by_id(&self, id: Uuid) -> Result<RegistryEntry>;
    async fn get_by_key(&self, space_id: Uuid, key: &str) -> Result<RegistryEntry>;
    async fn list_by_space(&self, space_id: Uuid) -> Result<Vec<RegistryEntry>>;
    async fn update(&self, entry: &RegistryEntry) -> Result<RegistryEntry>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}
