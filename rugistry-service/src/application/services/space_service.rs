use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{
    CreateSpaceRequest, ShareSpaceRequest, SpaceResponse, SpaceShareResponse, UpdateSpaceRequest,
};
use crate::domain::{entities::Space, repositories::SpaceRepository};

pub struct SpaceService {
    repository: Arc<dyn SpaceRepository>,
}

impl SpaceService {
    pub fn new(repository: Arc<dyn SpaceRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_space(
        &self,
        req: CreateSpaceRequest,
        owner_id: Uuid,
    ) -> anyhow::Result<SpaceResponse> {
        let space = Space::new(req.name, req.description, Some(owner_id));
        let created = self.repository.create(&space).await?;
        Ok(space_to_response(created, None))
    }

    pub async fn get_space(&self, id: Uuid, user_id: Uuid) -> anyhow::Result<SpaceResponse> {
        let permission = self.repository.get_user_permission(id, user_id).await?;
        let space = self.repository.get_by_id(id).await?;
        Ok(space_to_response(space, permission))
    }

    pub async fn get_space_by_name(
        &self,
        name: &str,
        user_id: Uuid,
    ) -> anyhow::Result<SpaceResponse> {
        let space = self.repository.get_by_name(name).await?;
        let permission = self.repository.get_user_permission(space.id, user_id).await?;
        Ok(space_to_response(space, permission))
    }

    pub async fn list_spaces(&self, user_id: Uuid) -> anyhow::Result<Vec<SpaceResponse>> {
        let spaces = self.repository.list_for_user(user_id).await?;
        Ok(spaces
            .into_iter()
            .map(|(s, perm)| space_to_response(s, perm))
            .collect())
    }

    pub async fn update_space(
        &self,
        id: Uuid,
        req: UpdateSpaceRequest,
        user_id: Uuid,
    ) -> anyhow::Result<SpaceResponse> {
        self.require_owner(id, user_id).await?;
        let mut space = self.repository.get_by_id(id).await?;

        if let Some(name) = req.name {
            space.name = name;
        }
        if req.description.is_some() {
            space.description = req.description;
        }

        space.updated_at = chrono::Utc::now();
        let updated = self.repository.update(&space).await?;
        Ok(space_to_response(updated, None))
    }

    pub async fn delete_space(&self, id: Uuid, user_id: Uuid) -> anyhow::Result<()> {
        self.require_owner(id, user_id).await?;
        self.repository.delete(id).await?;
        Ok(())
    }

    pub async fn add_share(
        &self,
        space_id: Uuid,
        req: ShareSpaceRequest,
        requesting_user_id: Uuid,
    ) -> anyhow::Result<()> {
        self.require_owner(space_id, requesting_user_id).await?;

        let valid_perms = ["readonly", "write", "appendonly"];
        if !valid_perms.contains(&req.permission.as_str()) {
            anyhow::bail!("Invalid permission '{}'. Must be one of: readonly, write, appendonly", req.permission);
        }

        let target_user_id = self.repository.get_user_id_by_username(&req.username).await?;
        if target_user_id == requesting_user_id {
            anyhow::bail!("Cannot share a space with yourself");
        }

        self.repository
            .add_share(space_id, target_user_id, &req.permission)
            .await?;
        Ok(())
    }

    pub async fn remove_share(
        &self,
        space_id: Uuid,
        target_user_id: Uuid,
        requesting_user_id: Uuid,
    ) -> anyhow::Result<()> {
        self.require_owner(space_id, requesting_user_id).await?;
        self.repository.remove_share(space_id, target_user_id).await?;
        Ok(())
    }

    pub async fn get_shares(
        &self,
        space_id: Uuid,
        requesting_user_id: Uuid,
    ) -> anyhow::Result<Vec<SpaceShareResponse>> {
        self.require_owner(space_id, requesting_user_id).await?;
        let shares = self.repository.get_shares(space_id).await?;
        Ok(shares
            .into_iter()
            .map(|s| SpaceShareResponse {
                user_id: s.user_id.to_string(),
                username: s.username,
                permission: s.permission,
                created_at: s.created_at.to_rfc3339(),
            })
            .collect())
    }

    /// Returns the permission level. Err if no access, Ok(None) = owner.
    pub async fn get_user_permission(
        &self,
        space_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<Option<String>> {
        Ok(self.repository.get_user_permission(space_id, user_id).await?)
    }

    async fn require_owner(&self, space_id: Uuid, user_id: Uuid) -> anyhow::Result<()> {
        let owner_id = self.repository.get_owner_id(space_id).await?;
        match owner_id {
            // Legacy space (no owner) or current user is owner
            None | Some(_) if owner_id == Some(user_id) || owner_id.is_none() => Ok(()),
            _ => anyhow::bail!("Only the space owner can perform this action"),
        }
    }
}

fn space_to_response(space: Space, permission: Option<String>) -> SpaceResponse {
    SpaceResponse {
        id: space.id,
        name: space.name,
        description: space.description,
        owner_id: space.owner_id,
        permission,
        created_at: space.created_at.to_rfc3339(),
        updated_at: space.updated_at.to_rfc3339(),
    }
}

