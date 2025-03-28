//! WebSocket module for AtomSi DAO
//!
//! This module provides WebSocket functionality for real-time updates.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use axum::{
    extract::{ws::{WebSocket, Message}, WebSocketUpgrade, Extension, Path, Query},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use futures::{stream::StreamExt, SinkExt};
use tracing::{info, error, debug};

use crate::DAOContext;
use crate::error::Result;

/// Maximum number of messages to buffer in broadcast channel
const MAX_BROADCAST_BUFFER: usize = 1000;

/// WebSocket close timeout (seconds)
const WS_CLOSE_TIMEOUT: u64 = 5;

/// Event types that can be broadcast through WebSockets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Proposal events
    ProposalCreated,
    ProposalUpdated,
    ProposalVoted,
    
    /// Treasury events
    TransactionCreated,
    TransactionApproved,
    TransactionExecuted,
    
    /// Member events
    MemberRegistered,
    MemberUpdated,
    
    /// Activity events
    ActivityRecorded,
}

/// WebSocket event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketEvent {
    /// Event type
    pub event_type: EventType,
    /// Timestamp of the event (ISO 8601 format)
    pub timestamp: String,
    /// Event data
    pub data: serde_json::Value,
}

/// WebSocket connection query parameters
#[derive(Debug, Deserialize)]
pub struct WebSocketParams {
    /// Optional auth token
    token: Option<String>,
    /// Optional event filters (comma-separated)
    events: Option<String>,
}

/// Client information
struct Client {
    /// Client ID
    id: String,
    /// Connection established time
    connected_at: Instant,
    /// Last activity time
    last_active: Instant,
    /// Member ID if authenticated
    member_id: Option<String>,
    /// Subscribed event types
    subscribed_events: Vec<EventType>,
}

/// WebSocket manager for handling connections and broadcasts
pub struct WebSocketManager {
    /// Broadcast sender for events
    event_sender: broadcast::Sender<WebSocketEvent>,
    /// Connected clients
    clients: Arc<Mutex<HashMap<String, Client>>>,
    /// DAO context
    context: Arc<DAOContext>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new(context: Arc<DAOContext>) -> Self {
        let (tx, _) = broadcast::channel(MAX_BROADCAST_BUFFER);
        Self {
            event_sender: tx,
            clients: Arc::new(Mutex::new(HashMap::new())),
            context,
        }
    }
    
    /// Get a new broadcast sender
    pub fn get_sender(&self) -> broadcast::Sender<WebSocketEvent> {
        self.event_sender.clone()
    }
    
    /// Broadcast an event to all clients
    pub fn broadcast_event(&self, event: WebSocketEvent) -> Result<usize> {
        let recipients = self.event_sender.send(event).map_err(|e| {
            error!("Failed to broadcast event: {}", e);
            crate::error::Error::InternalError("Failed to broadcast event".to_string())
        })?;
        Ok(recipients)
    }
    
    /// Create a new event for broadcasting
    pub fn create_event(event_type: EventType, data: serde_json::Value) -> WebSocketEvent {
        let now = chrono::Utc::now();
        WebSocketEvent {
            event_type,
            timestamp: now.to_rfc3339(),
            data,
        }
    }
    
    /// Register a new client
    fn register_client(&self, client_id: String, subscribed_events: Vec<EventType>, member_id: Option<String>) {
        let now = Instant::now();
        let client = Client {
            id: client_id.clone(),
            connected_at: now,
            last_active: now,
            member_id,
            subscribed_events,
        };
        
        let mut clients = self.clients.lock().unwrap();
        clients.insert(client_id, client);
    }
    
    /// Remove a client
    fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        if clients.remove(client_id).is_some() {
            debug!("Client disconnected: {}", client_id);
        }
    }
    
    /// Update client's last activity time
    fn update_client_activity(&self, client_id: &str) {
        let mut clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get_mut(client_id) {
            client.last_active = Instant::now();
        }
    }
    
    /// Check if event is subscribed by client
    fn is_event_subscribed(&self, client_id: &str, event_type: &EventType) -> bool {
        let clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get(client_id) {
            if client.subscribed_events.is_empty() {
                // If no specific subscriptions, subscribe to all events
                return true;
            }
            client.subscribed_events.iter().any(|e| std::mem::discriminant(e) == std::mem::discriminant(event_type))
        } else {
            false
        }
    }
    
    /// Get connection stats
    pub fn get_connection_stats(&self) -> (usize, HashMap<String, usize>) {
        let clients = self.clients.lock().unwrap();
        let total = clients.len();
        
        // Count clients by member_id
        let mut member_counts = HashMap::new();
        for client in clients.values() {
            if let Some(member_id) = &client.member_id {
                *member_counts.entry(member_id.clone()).or_insert(0) += 1;
            }
        }
        
        (total, member_counts)
    }
}

/// Handle WebSocket connection upgrade
pub async fn handle_ws_upgrade(
    ws: WebSocketUpgrade,
    Extension(ws_manager): Extension<Arc<WebSocketManager>>,
    Query(params): Query<WebSocketParams>,
) -> impl IntoResponse {
    // Authenticate the token if provided
    let member_id = if let Some(token) = params.token {
        // In a real implementation, validate the token and get member ID
        // For now, we'll just use the token as the member ID
        Some(token)
    } else {
        None
    };
    
    // Parse event subscriptions if provided
    let subscribed_events = if let Some(events) = params.events {
        events.split(',')
            .filter_map(|e| match e.trim() {
                "proposal_created" => Some(EventType::ProposalCreated),
                "proposal_updated" => Some(EventType::ProposalUpdated),
                "proposal_voted" => Some(EventType::ProposalVoted),
                "transaction_created" => Some(EventType::TransactionCreated),
                "transaction_approved" => Some(EventType::TransactionApproved),
                "transaction_executed" => Some(EventType::TransactionExecuted),
                "member_registered" => Some(EventType::MemberRegistered),
                "member_updated" => Some(EventType::MemberUpdated),
                "activity_recorded" => Some(EventType::ActivityRecorded),
                _ => None,
            })
            .collect()
    } else {
        Vec::new() // Empty means subscribe to all events
    };
    
    // Log the connection
    info!("WebSocket connection requested");
    
    // Upgrade the connection
    ws.on_upgrade(move |socket| handle_socket(socket, ws_manager, subscribed_events, member_id))
}

/// Handle WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    ws_manager: Arc<WebSocketManager>,
    subscribed_events: Vec<EventType>,
    member_id: Option<String>,
) {
    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    
    // Generate a unique client ID
    let client_id = uuid::Uuid::new_v4().to_string();
    
    // Register the client
    ws_manager.register_client(client_id.clone(), subscribed_events, member_id.clone());
    info!("WebSocket client connected: {}", client_id);
    
    // Subscribe to events
    let mut event_rx = ws_manager.get_sender().subscribe();
    
    // Send a welcome message
    let welcome_msg = serde_json::json!({
        "type": "welcome",
        "client_id": client_id,
        "message": "Connected to AtomSi DAO WebSocket",
        "authenticated": member_id.is_some(),
    });
    
    if let Err(e) = sender.send(Message::Text(welcome_msg.to_string())).await {
        error!("Error sending welcome message: {}", e);
        return;
    }
    
    // Spawn a task to forward events to the client
    let client_id_clone = client_id.clone();
    let ws_manager_clone = ws_manager.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            // Check if client is subscribed to this event
            if !ws_manager_clone.is_event_subscribed(&client_id_clone, &event.event_type) {
                continue;
            }
            
            // Serialize the event
            let event_json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            
            // Send the event
            if let Err(e) = sender.send(Message::Text(event_json)).await {
                error!("Error sending event to client {}: {}", client_id_clone, e);
                break;
            }
            
            // Update client activity
            ws_manager_clone.update_client_activity(&client_id_clone);
        }
    });
    
    // Handle messages from the client
    let client_id_clone = client_id.clone();
    let ws_manager_clone = ws_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!("Received text message from {}: {}", client_id_clone, text);
                    // Handle client message if needed
                    // For now, we'll just echo it back
                    if let Err(e) = sender.send(Message::Text(format!("Echo: {}", text))).await {
                        error!("Error echoing message: {}", e);
                        break;
                    }
                },
                Message::Binary(_) => {
                    // Ignore binary messages
                },
                Message::Ping(ping) => {
                    // Respond to ping with pong
                    if let Err(e) = sender.send(Message::Pong(ping)).await {
                        error!("Error sending pong: {}", e);
                        break;
                    }
                },
                Message::Pong(_) => {
                    // Ignore pong messages
                },
                Message::Close(_) => {
                    info!("Client requested close: {}", client_id_clone);
                    break;
                },
            }
            
            // Update client activity
            ws_manager_clone.update_client_activity(&client_id_clone);
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }
    
    // Gracefully close the connection
    let _ = sender.close().await;
    
    // Remove the client
    ws_manager.remove_client(&client_id);
    info!("WebSocket client disconnected: {}", client_id);
}

/// WebSocket information response
#[derive(Serialize)]
struct WebSocketInfoResponse {
    total_connections: usize,
    connections_by_member: HashMap<String, usize>,
    broadcast_channel_capacity: usize,
}

/// Get WebSocket connection information
pub async fn get_ws_info(
    Extension(ws_manager): Extension<Arc<WebSocketManager>>,
) -> impl IntoResponse {
    let (total, by_member) = ws_manager.get_connection_stats();
    
    let response = WebSocketInfoResponse {
        total_connections: total,
        connections_by_member: by_member,
        broadcast_channel_capacity: MAX_BROADCAST_BUFFER,
    };
    
    axum::Json(response)
} 