use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::application::dto::{CreateRegistryEntryRequest, RegistryEntryResponse, UpdateRegistryEntryRequest};
use crate::infrastructure::auth::Claims;
use crate::presentation::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct GetByKeyQuery {
    pub key: String,
}

fn parse_user_id(claims: &Claims) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(&claims.sub).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Invalid user id in token" })))
    })
}

/// Returns Err if user has no access. Ok(None) = owner; Ok(Some(perm)) = shared.
async fn get_permission(
    state: &AppState,
    space_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, (StatusCode, Json<serde_json::Value>)> {
    state
        .space_service
        .get_user_permission(space_id, user_id)
        .await
        .map_err(|e| (StatusCode::FORBIDDEN, Json(json!({ "error": e.to_string() }))))
}

pub async fn create_entry(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRegistryEntryRequest>,
) -> Result<(StatusCode, Json<RegistryEntryResponse>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    let perm = get_permission(&state, req.space_id, user_id).await?;
    // readonly cannot write
    if perm.as_deref() == Some("readonly") {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "error": "Read-only access" }))));
    }
    match state.registry_service.create_entry(req).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(entry))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_entry(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<RegistryEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    let entry = state.registry_service.get_entry(id).await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() }))))?;
    // verify access to the entry's space
    get_permission(&state, entry.space_id, user_id).await?;
    Ok(Json(entry))
}

pub async fn get_entry_by_key(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(space_id): Path<Uuid>,
    Query(query): Query<GetByKeyQuery>,
) -> Result<Json<RegistryEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    get_permission(&state, space_id, user_id).await?;
    match state.registry_service.get_entry_by_key(space_id, &query.key).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn list_entries_by_space(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(space_id): Path<Uuid>,
) -> Result<Json<Vec<RegistryEntryResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    get_permission(&state, space_id, user_id).await?;
    match state.registry_service.list_entries_by_space(space_id).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn update_entry(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRegistryEntryRequest>,
) -> Result<Json<RegistryEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    let entry = state.registry_service.get_entry(id).await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() }))))?;
    let perm = get_permission(&state, entry.space_id, user_id).await?;
    if matches!(perm.as_deref(), Some("readonly") | Some("appendonly")) {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "error": "Write access required to update entries" }))));
    }
    match state.registry_service.update_entry(id, req).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn delete_entry(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    let entry = state.registry_service.get_entry(id).await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() }))))?;
    let perm = get_permission(&state, entry.space_id, user_id).await?;
    if matches!(perm.as_deref(), Some("readonly") | Some("appendonly")) {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "error": "Write access required to delete entries" }))));
    }
    match state.registry_service.delete_entry(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() })))),
    }
}

