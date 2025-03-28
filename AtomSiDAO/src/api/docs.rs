//! API documentation module for AtomSi DAO
//!
//! This module provides OpenAPI/Swagger documentation for the API.

use std::sync::Arc;

use axum::{
    routing::get,
    Router,
    response::{Html, IntoResponse},
    extract::Extension,
    http::StatusCode,
};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::DAOContext;

/// Generate OpenAPI documentation
pub fn create_docs_router() -> Router {
    // Create the OpenAPI definition
    #[derive(OpenApi)]
    #[openapi(
        paths(
            // Auth routes
            crate::api::routes::auth::login,
            crate::api::routes::auth::logout,
            crate::api::routes::auth::check_session,
            
            // Governance routes
            crate::api::routes::governance::get_proposals,
            crate::api::routes::governance::get_proposal,
            crate::api::routes::governance::create_proposal,
            crate::api::routes::governance::vote_on_proposal,
            
            // Treasury routes
            crate::api::routes::treasury::get_transactions,
            crate::api::routes::treasury::get_transaction,
            crate::api::routes::treasury::create_transaction,
            crate::api::routes::treasury::approve_transaction,
            crate::api::routes::treasury::get_balances,
            
            // Identity routes
            crate::api::routes::identity::get_members,
            crate::api::routes::identity::get_member,
            crate::api::routes::identity::get_activities,
            
            // Token routes
            crate::api::routes::token::get_tokens,
            crate::api::routes::token::get_token,
            crate::api::routes::token::get_token_balances,
            crate::api::routes::token::transfer_tokens,
        ),
        components(
            schemas(
                // Auth models
                crate::api::models::LoginRequest,
                crate::api::models::LoginResponse,
                crate::api::models::LogoutRequest,
                
                // Common models
                crate::api::models::ApiResponse<crate::api::models::PaginatedResponse<crate::api::models::MemberResponse>>,
                crate::api::models::PaginatedResponse<crate::api::models::MemberResponse>,
                crate::api::models::PaginationMeta,
                crate::api::models::PaginationParams,
                
                // Member models
                crate::api::models::MemberResponse,
                crate::api::models::ActivityResponse,
                
                // Proposal models
                crate::api::models::CreateProposalRequest,
                crate::api::models::ProposalResponse,
                crate::api::models::VoteRequest,
                crate::api::models::VoteResponse,
                
                // Treasury models
                crate::api::models::CreateTransactionRequest,
                crate::api::models::TransactionResponse,
                crate::api::models::TokenBalanceResponse,
                
                // Token models
                crate::api::models::TokenResponse,
                crate::api::models::TokenTransferRequest,
                crate::api::models::TokenTransferResponse,
            )
        ),
        tags(
            (name = "Auth", description = "Authentication endpoints"),
            (name = "Governance", description = "Governance and proposal endpoints"),
            (name = "Treasury", description = "Treasury transaction endpoints"),
            (name = "Identity", description = "Member identity endpoints"),
            (name = "Token", description = "Token management endpoints"),
        ),
        info(
            title = "AtomSi DAO API",
            version = env!("CARGO_PKG_VERSION"),
            description = "API for AtomSi DAO - a decentralized autonomous organization framework",
            license(
                name = "MIT",
                url = "https://github.com/atomsidao/atomsidao/blob/main/LICENSE"
            ),
            contact(
                name = "AtomSi DAO Team",
                url = "https://github.com/atomsidao/atomsidao",
                email = "contact@atomsidao.org"
            ),
        ),
        external_docs(
            url = "https://github.com/atomsidao/atomsidao/blob/main/README.md",
            description = "AtomSi DAO Documentation"
        )
    )]
    struct ApiDoc;
    
    // Create Swagger UI with the OpenAPI definition
    let swagger = SwaggerUi::new("/docs/swagger-ui")
        .url("/docs/openapi.json", ApiDoc::openapi());
    
    // Create the documentation router
    Router::new()
        .route("/", get(serve_docs_index))
        .merge(swagger)
}

/// Serve the API documentation index page
async fn serve_docs_index() -> impl IntoResponse {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>AtomSi DAO API Documentation</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
                line-height: 1.6;
                color: #333;
                max-width: 800px;
                margin: 0 auto;
                padding: 1rem;
                background-color: #f9f9f9;
            }
            h1 {
                color: #2c3e50;
                border-bottom: 2px solid #eee;
                padding-bottom: 0.5rem;
            }
            a {
                color: #3498db;
                text-decoration: none;
            }
            a:hover {
                text-decoration: underline;
            }
            .container {
                background-color: white;
                padding: 2rem;
                border-radius: 5px;
                box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            }
            .footer {
                margin-top: 2rem;
                font-size: 0.9rem;
                color: #7f8c8d;
                text-align: center;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>AtomSi DAO API Documentation</h1>
            <p>Welcome to the AtomSi DAO API documentation. This page provides resources to understand and interact with the AtomSi DAO API.</p>
            
            <h2>API Documentation Resources</h2>
            <ul>
                <li><a href="/docs/swagger-ui">Swagger UI</a> - Interactive API documentation</li>
                <li><a href="/docs/openapi.json">OpenAPI Specification</a> - Raw OpenAPI JSON specification</li>
            </ul>
            
            <h2>API Overview</h2>
            <p>The AtomSi DAO API provides endpoints to interact with the following components:</p>
            <ul>
                <li><strong>Authentication</strong> - Login, logout, and session management</li>
                <li><strong>Governance</strong> - Proposal creation, retrieval, and voting</li>
                <li><strong>Treasury</strong> - Transaction management and balance retrieval</li>
                <li><strong>Identity</strong> - Member information and activity management</li>
                <li><strong>Token</strong> - Token management and transfers</li>
            </ul>
        </div>
        
        <div class="footer">
            <p>AtomSi DAO - A Decentralized Autonomous Organization Framework</p>
            <p>Version: "#" + env!("CARGO_PKG_VERSION") + "#"</p>
        </div>
    </body>
    </html>
    "#;
    
    Html(html)
} 