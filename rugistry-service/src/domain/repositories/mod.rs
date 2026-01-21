use async_trait::async_trait;
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
}

#[async_trait]
pub trait SpaceRepository: Send + Sync {
    async fn create(&self, space: &Space) -> Result<Space>;
    async fn get_by_id(&self, id: Uuid) -> Result<Space>;
    async fn get_by_name(&self, name: &str) -> Result<Space>;
    async fn list(&self) -> Result<Vec<Space>>;
    async fn update(&self, space: &Space) -> Result<Space>;
    async fn delete(&self, id: Uuid) -> Result<()>;
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
