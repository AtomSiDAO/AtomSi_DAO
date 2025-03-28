//! API module for AtomSi DAO
//!
//! This module provides a RESTful API for interacting with the DAO.

use std::sync::Arc;
use std::net::SocketAddr;

use axum::{
    Router,
    routing::{get, post, put},
    extract::Extension,
    middleware,
    http::{Method, HeaderValue, StatusCode},
    response::IntoResponse,
};
use tower_http::cors::{CorsLayer, Any};
use tracing::info;

use crate::DAOContext;
use crate::api::models::ApiResponse;

pub mod models;
pub mod routes;
pub mod middleware as api_middleware;
pub mod docs;
pub mod websocket;

/// API server configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// The address to bind to
    pub bind_address: SocketAddr,
    /// Whether to enable CORS
    pub enable_cors: bool,
    /// Whether to enable request logging
    pub enable_logging: bool,
    /// Whether to enable API documentation
    pub enable_docs: bool,
    /// Whether to enable WebSockets
    pub enable_websockets: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:3000".parse().unwrap(),
            enable_cors: true,
            enable_logging: true,
            enable_docs: true,
            enable_websockets: true,
        }
    }
}

/// API server
pub struct ApiServer {
    /// API configuration
    config: ApiConfig,
    /// DAO context
    context: Arc<DAOContext>,
    /// WebSocket manager
    ws_manager: Option<Arc<websocket::WebSocketManager>>,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(config: ApiConfig, context: Arc<DAOContext>) -> Self {
        let ws_manager = if config.enable_websockets {
            Some(Arc::new(websocket::WebSocketManager::new(context.clone())))
        } else {
            None
        };
        
        Self {
            config,
            context,
            ws_manager,
        }
    }
    
    /// Get a reference to the WebSocket manager
    pub fn get_ws_manager(&self) -> Option<Arc<websocket::WebSocketManager>> {
        self.ws_manager.clone()
    }
    
    /// Start the API server
    pub async fn start(&self) -> anyhow::Result<()> {
        // Build the CORS layer if enabled
        let cors_layer = if self.config.enable_cors {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any)
                .allow_credentials(true)
        } else {
            CorsLayer::permissive()
        };
        
        // Build the base app with shared state
        let mut app = Router::new()
            .route("/health", get(health_check))
            .layer(Extension(self.context.clone()))
            .layer(cors_layer);
        
        // Add logging middleware if enabled
        if self.config.enable_logging {
            app = app.layer(api_middleware::create_trace_layer());
        }
        
        // Add WebSocket routes if enabled
        if let Some(ws_manager) = &self.ws_manager {
            app = app
                .route("/ws", get(websocket::handle_ws_upgrade))
                .route("/api/ws/info", get(websocket::get_ws_info))
                .layer(Extension(ws_manager.clone()));
        }
        
        // Add the API routes
        app = app.nest("/api", self.create_api_router());
        
        // Add the documentation routes if enabled
        if self.config.enable_docs {
            app = app.nest("/docs", docs::create_docs_router());
        }
        
        // Start the server
        info!("Starting API server on {}", self.config.bind_address);
        axum::Server::bind(&self.config.bind_address)
            .serve(app.into_make_service())
            .await?;
        
        Ok(())
    }
    
    /// Create the API router
    fn create_api_router(&self) -> Router {
        // Governance routes
        let governance_routes = Router::new()
            .route("/proposals", get(routes::governance::get_proposals)
                                   .post(routes::governance::create_proposal))
            .route("/proposals/:id", get(routes::governance::get_proposal))
            .route("/proposals/:id/vote", post(routes::governance::vote_on_proposal));
        
        // Treasury routes
        let treasury_routes = Router::new()
            .route("/transactions", get(routes::treasury::get_transactions)
                                      .post(routes::treasury::create_transaction))
            .route("/transactions/:id", get(routes::treasury::get_transaction))
            .route("/transactions/:id/approve", post(routes::treasury::approve_transaction))
            .route("/balances", get(routes::treasury::get_balances));
        
        // Identity routes
        let identity_routes = Router::new()
            .route("/members", get(routes::identity::get_members))
            .route("/members/:id", get(routes::identity::get_member))
            .route("/activities", get(routes::identity::get_activities));
        
        // Token routes
        let token_routes = Router::new()
            .route("/tokens", get(routes::token::get_tokens))
            .route("/tokens/:id", get(routes::token::get_token))
            .route("/tokens/:id/balances", get(routes::token::get_token_balances))
            .route("/transfer", post(routes::token::transfer_tokens));
        
        // Auth routes
        let auth_routes = Router::new()
            .route("/login", post(routes::auth::login))
            .route("/logout", post(routes::auth::logout))
            .route("/check-session", get(routes::auth::check_session));
        
        // Combine all routes into the API router
        // Protected routes require authentication
        let protected_routes = Router::new()
            .nest("/governance", governance_routes)
            .nest("/treasury", treasury_routes)
            .nest("/identity", identity_routes)
            .nest("/token", token_routes)
            .route_layer(middleware::from_fn_with_state(
                self.context.clone(),
                |context, req, next| async move {
                    api_middleware::require_auth(context, req, next).await
                },
            ));
        
        // Public routes don't require authentication
        let public_routes = Router::new()
            .nest("/auth", auth_routes);
        
        // Combine protected and public routes
        Router::new()
            .merge(protected_routes)
            .merge(public_routes)
            .fallback(handle_not_found)
    }
    
    /// Broadcast a WebSocket event
    pub fn broadcast_ws_event(
        &self,
        event_type: websocket::EventType,
        data: serde_json::Value,
    ) -> crate::error::Result<usize> {
        if let Some(ws_manager) = &self.ws_manager {
            let event = websocket::WebSocketManager::create_event(event_type, data);
            ws_manager.broadcast_event(event)
        } else {
            Ok(0) // WebSockets not enabled
        }
    }
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    let response = ApiResponse::success(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }));
    
    (StatusCode::OK, axum::Json(response))
}

/// Handle 404 Not Found
async fn handle_not_found() -> impl IntoResponse {
    let response = ApiResponse::<()>::error_with_code("Not Found", 404);
    (StatusCode::NOT_FOUND, axum::Json(response))
} 