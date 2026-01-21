use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::application::dto::ChangeNotification;
use crate::infrastructure::events::EventBus;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(event_bus): State<Arc<EventBus>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, event_bus))
}

async fn handle_socket(socket: WebSocket, event_bus: Arc<EventBus>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = event_bus.subscribe();

    // Task for sending events to the client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
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
