use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::application::{
    dto::{CreateSpaceRequest, SpaceResponse, UpdateSpaceRequest},
    services::SpaceService,
};
use std::sync::Arc;

pub async fn create_space(
    State(service): State<Arc<SpaceService>>,
    Json(req): Json<CreateSpaceRequest>,
) -> Result<(StatusCode, Json<SpaceResponse>), (StatusCode, Json<serde_json::Value>)> {
    match service.create_space(req).await {
        Ok(space) => Ok((StatusCode::CREATED, Json(space))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn get_space(
    State(service): State<Arc<SpaceService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SpaceResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_space(id).await {
        Ok(space) => Ok(Json(space)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn get_space_by_name(
    State(service): State<Arc<SpaceService>>,
    Path(name): Path<String>,
) -> Result<Json<SpaceResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_space_by_name(&name).await {
        Ok(space) => Ok(Json(space)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn list_spaces(
    State(service): State<Arc<SpaceService>>,
) -> Result<Json<Vec<SpaceResponse>>, (StatusCode, Json<serde_json::Value>)> {
    match service.list_spaces().await {
        Ok(spaces) => Ok(Json(spaces)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_space(
    State(service): State<Arc<SpaceService>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSpaceRequest>,
) -> Result<Json<SpaceResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.update_space(id, req).await {
        Ok(space) => Ok(Json(space)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_space(
    State(service): State<Arc<SpaceService>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match service.delete_space(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}
