use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::application::dto::{CreateWebhookRequest, WebhookResponse};
use crate::infrastructure::auth::Claims;
use crate::presentation::routes::AppState;

fn parse_user_id(claims: &Claims) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(&claims.sub).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Invalid user id in token" })))
    })
}

async fn require_owner(
    state: &AppState,
    space_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state.space_service.get_space(space_id, user_id).await {
        Ok(space) if space.permission.is_none() => Ok(()),
        Ok(_) => Err((StatusCode::FORBIDDEN, Json(json!({ "error": "Only the space owner can manage webhooks" })))),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn list_webhooks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(space_id): Path<Uuid>,
) -> Result<Json<Vec<WebhookResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    require_owner(&state, space_id, user_id).await?;
    match state.webhook_service.list_webhooks(space_id).await {
        Ok(webhooks) => Ok(Json(webhooks)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn create_webhook(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(space_id): Path<Uuid>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    require_owner(&state, space_id, user_id).await?;
    if req.url.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "URL is required" }))));
    }
    match state.webhook_service.create_webhook(space_id, req).await {
        Ok(webhook) => Ok((StatusCode::CREATED, Json(webhook))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((space_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims)?;
    require_owner(&state, space_id, user_id).await?;
    match state.webhook_service.delete_webhook(webhook_id, space_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() })))),
    }
}
