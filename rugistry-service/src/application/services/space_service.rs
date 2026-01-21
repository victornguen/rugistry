use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{CreateSpaceRequest, SpaceResponse, UpdateSpaceRequest};
use crate::domain::{entities::Space, repositories::SpaceRepository};

pub struct SpaceService {
    repository: Arc<dyn SpaceRepository>,
}

impl SpaceService {
    pub fn new(repository: Arc<dyn SpaceRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_space(&self, req: CreateSpaceRequest) -> anyhow::Result<SpaceResponse> {
        let space = Space::new(req.name, req.description);
        let created = self.repository.create(&space).await?;
        Ok(space_to_response(created))
    }

    pub async fn get_space(&self, id: Uuid) -> anyhow::Result<SpaceResponse> {
        let space = self.repository.get_by_id(id).await?;
        Ok(space_to_response(space))
    }

    pub async fn get_space_by_name(&self, name: &str) -> anyhow::Result<SpaceResponse> {
        let space = self.repository.get_by_name(name).await?;
        Ok(space_to_response(space))
    }

    pub async fn list_spaces(&self) -> anyhow::Result<Vec<SpaceResponse>> {
        let spaces = self.repository.list().await?;
        Ok(spaces.into_iter().map(space_to_response).collect())
    }

    pub async fn update_space(
        &self,
        id: Uuid,
        req: UpdateSpaceRequest,
    ) -> anyhow::Result<SpaceResponse> {
        let mut space = self.repository.get_by_id(id).await?;
        
        if let Some(name) = req.name {
            space.name = name;
        }
        if req.description.is_some() {
            space.description = req.description;
        }
        
        space.updated_at = chrono::Utc::now();
        let updated = self.repository.update(&space).await?;
        Ok(space_to_response(updated))
    }

    pub async fn delete_space(&self, id: Uuid) -> anyhow::Result<()> {
        self.repository.delete(id).await?;
        Ok(())
    }
}

fn space_to_response(space: Space) -> SpaceResponse {
    SpaceResponse {
        id: space.id,
        name: space.name,
        description: space.description,
        created_at: space.created_at.to_rfc3339(),
        updated_at: space.updated_at.to_rfc3339(),
    }
}
