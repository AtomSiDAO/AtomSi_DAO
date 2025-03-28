//! API Server Example
//!
//! This example demonstrates how to set up and start the AtomSi DAO API server.

use std::sync::Arc;
use std::net::SocketAddr;

use atomsidao::api::{ApiServer, ApiConfig};
use atomsidao::DAOContext;
use atomsidao::database::{DatabaseAdapter, PostgresAdapter, SqliteAdapter};
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
    
    // Create database adapter
    let db_adapter: Arc<dyn DatabaseAdapter> = match config.database.db_type.as_str() {
        "postgres" => {
            let postgres = PostgresAdapter::new(&config.database.connection_string).await?;
            Arc::new(postgres)
        },
        "sqlite" => {
            let sqlite = SqliteAdapter::new(&config.database.connection_string).await?;
            Arc::new(sqlite)
        },
        _ => {
            panic!("Unsupported database type: {}", config.database.db_type);
        }
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
        bind_address: config.api.host_port.parse::<SocketAddr>()?,
        enable_cors: config.api.enable_cors,
        enable_logging: true,
        enable_docs: true,
    };
    
    // Create and start API server
    let api_server = ApiServer::new(api_config, Arc::new(context));
    println!("Starting API server on {}", config.api.host_port);
    api_server.start().await?;
    
    Ok(())
} 