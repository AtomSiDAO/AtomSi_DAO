//! API middleware for AtomSi DAO
//!
//! This module contains middleware components for the API server.

use std::sync::Arc;

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    extract::Extension,
    body::Body,
    Json,
};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;

use crate::DAOContext;
use crate::api::models::ApiResponse;
use crate::security::AuthManager;

/// Create a tracing middleware layer for request logging
pub fn create_trace_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

/// Middleware for checking if user is authenticated
pub async fn require_auth<B>(
    Extension(context): Extension<Arc<DAOContext>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Get the authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Check if token is valid
    // This is a placeholder implementation
    // In a real implementation, we would call the auth service to validate the token
    
    // Extract the token from the header
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Try to authenticate with the token
    // In a real implementation, this would verify the token and get the user
    // For now, we just pass through all requests
    
    // If the token is valid, continue to the next middleware or handler
    let response = next.run(req).await;
    Ok(response)
}

/// Middleware for checking if user has required permissions
pub async fn require_permission<B>(
    permission: &'static str,
    resource: &'static str,
    Extension(context): Extension<Arc<DAOContext>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Get the authorization header (token)
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Extract the token from the header
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Check permissions
    // This is a placeholder implementation
    // In a real implementation, we would:
    // 1. Validate the token and get the user ID and role
    // 2. Check if the user's role has the required permission for the resource
    
    // If user has permission, continue to the next middleware or handler
    let response = next.run(req).await;
    Ok(response)
}

/// Handle unauthorized errors
pub async fn handle_unauthorized_error() -> Json<ApiResponse<()>> {
    Json(ApiResponse::error_with_code(
        "Unauthorized: Authentication required",
        401
    ))
}

/// Handle forbidden errors
pub async fn handle_forbidden_error() -> Json<ApiResponse<()>> {
    Json(ApiResponse::error_with_code(
        "Forbidden: Insufficient permissions",
        403
    ))
} 