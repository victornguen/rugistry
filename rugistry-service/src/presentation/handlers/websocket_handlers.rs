use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::infrastructure::auth::{AuthConfig, Claims};
use crate::infrastructure::events::EventBus;
use crate::presentation::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    /// Filter by exact key match
    pub key: Option<String>,
    /// Filter by regex pattern
    pub pattern: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(space_id): Path<Uuid>,
    Query(query): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Response {
    let regex = query.pattern.and_then(|p| Regex::new(&p).ok());
    ws.on_upgrade(move |socket| {
        handle_socket(socket, state.event_bus, state.auth_config, space_id, query.key, regex)
    })
}

#[derive(Deserialize)]
struct WsAuth {
    token: String,
}

async fn handle_socket(
    socket: WebSocket,
    event_bus: Arc<EventBus>,
    auth_config: Arc<AuthConfig>,
    space_id: Uuid,
    filter_key: Option<String>,
    filter_pattern: Option<Regex>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Authenticate via the first message: client must send {"token": "<jwt>"}
    let authenticated = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            serde_json::from_str::<WsAuth>(&text)
                .map(|auth| {
                    decode::<Claims>(
                        &auth.token,
                        &DecodingKey::from_secret(auth_config.jwt_secret.as_bytes()),
                        &Validation::new(Algorithm::HS256),
                    )
                    .is_ok()
                })
                .unwrap_or(false)
        }
        _ => false,
    };

    if !authenticated {
        let _ = sender.send(Message::Close(None)).await;
        return;
    }

    let mut rx = event_bus.subscribe();

    // Task for sending events to the client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Filter by space_id first
            if event.space_id != space_id {
                continue;
            }

            // Apply optional key filtering
            let should_send = if let Some(ref key_filter) = filter_key {
                // Exact key match
                event.key.as_ref().map_or(false, |k| k == key_filter)
            } else if let Some(ref pattern) = filter_pattern {
                // Regex pattern match
                event.key.as_ref().map_or(false, |k| pattern.is_match(k))
            } else {
                // No filter, send all events for this space
                true
            };

            if should_send {
                if let Ok(json) = serde_json::to_string(&event) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Task for receiving messages from client (close detection)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
