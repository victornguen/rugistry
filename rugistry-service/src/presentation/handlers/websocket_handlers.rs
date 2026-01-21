use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::application::dto::ChangeNotification;
use crate::infrastructure::events::EventBus;

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
    State(event_bus): State<Arc<EventBus>>,
) -> Response {
    // Compile regex pattern if provided
    let regex = query.pattern.and_then(|p| Regex::new(&p).ok());
    
    ws.on_upgrade(move |socket| handle_socket(socket, event_bus, space_id, query.key, regex))
}

async fn handle_socket(
    socket: WebSocket,
    event_bus: Arc<EventBus>,
    space_id: Uuid,
    filter_key: Option<String>,
    filter_pattern: Option<Regex>,
) {
    let (mut sender, mut receiver) = socket.split();
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
                    if sender
                        .send(axum::extract::ws::Message::Text(json))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    // Task for receiving messages from client (ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Close(_) = msg {
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
