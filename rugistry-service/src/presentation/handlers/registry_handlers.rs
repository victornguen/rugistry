use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::application::{
    dto::{CreateRegistryEntryRequest, RegistryEntryResponse, UpdateRegistryEntryRequest},
    services::RegistryService,
};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct GetByKeyQuery {
    pub key: String,
}

pub async fn create_entry(
    State(service): State<Arc<RegistryService>>,
    Json(req): Json<CreateRegistryEntryRequest>,
) -> Result<(StatusCode, Json<RegistryEntryResponse>), (StatusCode, Json<serde_json::Value>)> {
    match service.create_entry(req).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(entry))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn get_entry(
    State(service): State<Arc<RegistryService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<RegistryEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_entry(id).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn get_entry_by_key(
    State(service): State<Arc<RegistryService>>,
    Path(space_id): Path<Uuid>,
    Query(query): Query<GetByKeyQuery>,
) -> Result<Json<RegistryEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_entry_by_key(space_id, &query.key).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn list_entries_by_space(
    State(service): State<Arc<RegistryService>>,
    Path(space_id): Path<Uuid>,
) -> Result<Json<Vec<RegistryEntryResponse>>, (StatusCode, Json<serde_json::Value>)> {
    match service.list_entries_by_space(space_id).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_entry(
    State(service): State<Arc<RegistryService>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRegistryEntryRequest>,
) -> Result<Json<RegistryEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.update_entry(id, req).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_entry(
    State(service): State<Arc<RegistryService>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match service.delete_entry(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}
