//! WebSocket Example
//!
//! This example demonstrates how to use the WebSocket API for real-time updates.

use std::sync::Arc;
use std::time::Duration;

use atomsidao::api::{ApiServer, ApiConfig, websocket::EventType};
use atomsidao::DAOContext;
use atomsidao::database::{DatabaseAdapter, SqliteAdapter};
use atomsidao::blockchain::{BlockchainAdapter, EthereumAdapter};
use atomsidao::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Load configuration from environment variables or config file
    let config = Config::from_env()?;
    
    // Create database adapter (using in-memory SQLite for example)
    let db_adapter: Arc<dyn DatabaseAdapter> = {
        let sqlite = SqliteAdapter::new("sqlite::memory:").await?;
        Arc::new(sqlite)
    };
    
    // Create blockchain adapter
    let blockchain_adapter: Arc<dyn BlockchainAdapter> = {
        let ethereum = EthereumAdapter::new(&config.blockchain.rpc_url, None).await?;
        Arc::new(ethereum)
    };
    
    // Create DAO context
    let context = DAOContext::new(
        config.clone(),
        db_adapter.clone(),
        blockchain_adapter.clone(),
    );
    
    // Create API configuration
    let api_config = ApiConfig {
        bind_address: "127.0.0.1:3000".parse()?,
        enable_cors: true,
        enable_logging: true,
        enable_docs: true,
        enable_websockets: true,
    };
    
    // Create API server
    let api_server = Arc::new(ApiServer::new(api_config, Arc::new(context)));
    
    // Clone api_server for the event generator task
    let api_server_clone = api_server.clone();
    
    // Spawn API server
    let server_task = tokio::spawn(async move {
        println!("Starting API server on 127.0.0.1:3000");
        println!("WebSocket endpoint: ws://127.0.0.1:3000/ws");
        println!("API endpoint: http://127.0.0.1:3000/api");
        println!("Documentation: http://127.0.0.1:3000/docs");
        
        if let Err(e) = api_server.start().await {
            eprintln!("API server error: {}", e);
        }
    });
    
    // Spawn a task to generate events
    let event_generator = tokio::spawn(async move {
        // Wait for server to start
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Generate some events every few seconds
        for i in 1..=10 {
            println!("Generating event {}", i);
            
            // Alternate between different event types
            match i % 3 {
                0 => {
                    // Generate a proposal event
                    let proposal_data = serde_json::json!({
                        "proposal_id": format!("prop_{}", i),
                        "title": format!("Test Proposal {}", i),
                        "description": "This is a test proposal generated for WebSocket demo",
                        "proposer": "0x123456789abcdef",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    });
                    
                    api_server_clone.broadcast_ws_event(
                        EventType::ProposalCreated,
                        proposal_data,
                    ).unwrap();
                },
                1 => {
                    // Generate a transaction event
                    let tx_data = serde_json::json!({
                        "transaction_id": format!("tx_{}", i),
                        "description": "Treasury transaction for testing",
                        "amount": "1.5",
                        "token": "ETH",
                        "recipient": "0x987654321fedcba",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    });
                    
                    api_server_clone.broadcast_ws_event(
                        EventType::TransactionCreated,
                        tx_data,
                    ).unwrap();
                },
                _ => {
                    // Generate a member event
                    let member_data = serde_json::json!({
                        "member_id": format!("member_{}", i),
                        "name": format!("Test Member {}", i),
                        "address": format!("0x{:x}", i * 100),
                        "role": "Member",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    });
                    
                    api_server_clone.broadcast_ws_event(
                        EventType::MemberRegistered,
                        member_data,
                    ).unwrap();
                }
            }
            
            // Wait before generating the next event
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        
        println!("Event generation complete. Server will continue running.");
    });
    
    // Wait for tasks to complete (server will run indefinitely)
    event_generator.await?;
    server_task.await?;
    
    Ok(())
} 