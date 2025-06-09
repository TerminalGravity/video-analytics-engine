use axum::extract::ws::{Message, WebSocket};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{Alert, AnalyticsEvent, InferenceResult, StreamStatus, User},
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    // Client -> Server
    Subscribe {
        stream_id: Option<Uuid>,
        event_types: Vec<String>,
    },
    Unsubscribe {
        stream_id: Option<Uuid>,
    },
    Ping,

    // Server -> Client
    StreamStatusUpdate {
        stream_id: Uuid,
        status: StreamStatus,
    },
    NewInferenceResult {
        result: InferenceResult,
    },
    NewAlert {
        alert: Alert,
    },
    NewAnalyticsEvent {
        event: AnalyticsEvent,
    },
    Pong,
    Error {
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub user_id: Uuid,
    pub subscriptions: Vec<WebSocketSubscription>,
}

#[derive(Debug, Clone)]
pub struct WebSocketSubscription {
    pub stream_id: Option<Uuid>,
    pub event_types: Vec<String>,
}

pub type WebSocketSessions = Arc<Mutex<HashMap<Uuid, broadcast::Sender<WebSocketMessage>>>>;

pub async fn handle_socket(socket: WebSocket, state: AppState) {
    let session_id = Uuid::new_v4();
    let (mut sender, mut receiver) = socket.split();
    
    // Create a broadcast channel for this session
    let (tx, mut rx) = broadcast::channel::<WebSocketMessage>(100);
    
    // Store the session (in a real implementation, you'd want to associate this with a user)
    // For now, we'll use a simple approach
    
    tracing::info!("WebSocket session started: {}", session_id);

    // Spawn a task to handle outgoing messages
    let tx_clone = tx.clone();
    let outgoing_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json_msg = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    tracing::error!("Failed to serialize WebSocket message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json_msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = handle_text_message(&text, &tx_clone, &state).await {
                        tracing::error!("Error handling WebSocket message: {}", e);
                        let error_msg = WebSocketMessage::Error {
                            message: "Failed to process message".to_string(),
                        };
                        let _ = tx_clone.send(error_msg);
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket session closed: {}", session_id);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    // Echo ping as pong (axum handles this automatically, but we log it)
                    tracing::trace!("Received ping from session: {}", session_id);
                }
                Ok(Message::Pong(_)) => {
                    tracing::trace!("Received pong from session: {}", session_id);
                }
                Ok(Message::Binary(_)) => {
                    tracing::warn!("Received binary message (not supported)");
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = outgoing_task => {
            tracing::info!("Outgoing task completed for session: {}", session_id);
        }
        _ = incoming_task => {
            tracing::info!("Incoming task completed for session: {}", session_id);
        }
    }

    tracing::info!("WebSocket session ended: {}", session_id);
}

async fn handle_text_message(
    text: &str,
    tx: &broadcast::Sender<WebSocketMessage>,
    state: &AppState,
) -> Result<(), AppError> {
    let message: WebSocketMessage = serde_json::from_str(text)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON: {}", e)))?;

    match message {
        WebSocketMessage::Subscribe { stream_id, event_types } => {
            tracing::info!("Client subscribed to stream {:?} for events: {:?}", stream_id, event_types);
            
            // In a real implementation, you'd store this subscription
            // and use it to filter which messages to send to this client
            
            // Send confirmation (optional)
            // let _ = tx.send(WebSocketMessage::Pong);
        }
        
        WebSocketMessage::Unsubscribe { stream_id } => {
            tracing::info!("Client unsubscribed from stream {:?}", stream_id);
            
            // Remove subscription in a real implementation
        }
        
        WebSocketMessage::Ping => {
            let _ = tx.send(WebSocketMessage::Pong);
        }
        
        _ => {
            return Err(AppError::BadRequest("Unsupported message type".to_string()));
        }
    }

    Ok(())
}

// These functions would be called from other services to broadcast updates
pub async fn broadcast_stream_status_update(
    sessions: &WebSocketSessions,
    stream_id: Uuid,
    status: StreamStatus,
) {
    let message = WebSocketMessage::StreamStatusUpdate { stream_id, status };
    broadcast_to_all_sessions(sessions, message).await;
}

pub async fn broadcast_new_inference_result(
    sessions: &WebSocketSessions,
    result: InferenceResult,
) {
    let message = WebSocketMessage::NewInferenceResult { result };
    broadcast_to_all_sessions(sessions, message).await;
}

pub async fn broadcast_new_alert(
    sessions: &WebSocketSessions,
    alert: Alert,
) {
    let message = WebSocketMessage::NewAlert { alert };
    broadcast_to_all_sessions(sessions, message).await;
}

pub async fn broadcast_new_analytics_event(
    sessions: &WebSocketSessions,
    event: AnalyticsEvent,
) {
    let message = WebSocketMessage::NewAnalyticsEvent { event };
    broadcast_to_all_sessions(sessions, message).await;
}

async fn broadcast_to_all_sessions(
    sessions: &WebSocketSessions,
    message: WebSocketMessage,
) {
    let sessions = sessions.lock().await;
    for (session_id, tx) in sessions.iter() {
        if let Err(e) = tx.send(message.clone()) {
            tracing::warn!("Failed to send message to session {}: {}", session_id, e);
        }
    }
}

// Helper function to broadcast to specific stream subscribers
pub async fn broadcast_to_stream_subscribers(
    sessions: &WebSocketSessions,
    stream_id: Uuid,
    message: WebSocketMessage,
) {
    // In a real implementation, you'd filter sessions based on their subscriptions
    // For now, we'll broadcast to all sessions
    broadcast_to_all_sessions(sessions, message).await;
} 