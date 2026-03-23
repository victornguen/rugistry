use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::application::dto::{
    CreateSpaceRequest, ShareSpaceRequest, SpaceResponse, SpaceShareResponse, UpdateSpaceRequest,
};
use crate::infrastructure::auth::Claims;
use crate::presentation::routes::AppState;

fn parse_user_id(claims: &Claims) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Invalid user id in token" })),
        )
    })
}

pub async fn create_space(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateSpaceRequest>,
) -> Result<(StatusCode, Json<SpaceResponse>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.create_space(req, user_id).await {
        Ok(space) => Ok((StatusCode::CREATED, Json(space))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_space(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<SpaceResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.get_space(id, user_id).await {
        Ok(space) => Ok(Json(space)),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_space_by_name(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(name): Path<String>,
) -> Result<Json<SpaceResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.get_space_by_name(&name, user_id).await {
        Ok(space) => Ok(Json(space)),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn list_spaces(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<SpaceResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.list_spaces(user_id).await {
        Ok(spaces) => Ok(Json(spaces)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn update_space(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSpaceRequest>,
) -> Result<Json<SpaceResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.update_space(id, req, user_id).await {
        Ok(space) => Ok(Json(space)),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn delete_space(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.delete_space(id, user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::FORBIDDEN, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn add_share(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(space_id): Path<Uuid>,
    Json(req): Json<ShareSpaceRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.add_share(space_id, req, user_id).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn remove_share(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((space_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.remove_share(space_id, target_user_id, user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_shares(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(space_id): Path<Uuid>,
) -> Result<Json<Vec<SpaceShareResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    match state.space_service.get_shares(space_id, user_id).await {
        Ok(shares) => Ok(Json(shares)),
        Err(e) => Err((StatusCode::FORBIDDEN, Json(json!({ "error": e.to_string() })))),
    }
}

