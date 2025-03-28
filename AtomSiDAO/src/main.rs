use atomsi_dao::{config, core::Dao, init_logging};
use log::{info, warn};
use std::process;

#[tokio::main]
async fn main() {
    // Initialize logging
    init_logging();
    
    info!("Starting AtomSi DAO v{}", atomsi_dao::VERSION);
    
    // Load configuration
    let config = match config::load() {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            warn!("Failed to load configuration: {}", e);
            warn!("Using default configuration");
            config::default()
        }
    };
    
    // Initialize blockchain connection
    let blockchain = match init_blockchain(&config).await {
        Ok(blockchain) => {
            info!("Connected to blockchain network: {}", config.blockchain.network);
            blockchain
        }
        Err(e) => {
            eprintln!("ERROR: Failed to connect to blockchain: {}", e);
            process::exit(1);
        }
    };
    
    // Initialize database connection
    let db = match init_database(&config).await {
        Ok(db) => {
            info!("Connected to database");
            db
        }
        Err(e) => {
            eprintln!("ERROR: Failed to connect to database: {}", e);
            process::exit(1);
        }
    };
    
    // Initialize the DAO
    let dao = match Dao::new(config, blockchain, db) {
        Ok(dao) => {
            info!("DAO initialized successfully");
            dao
        }
        Err(e) => {
            eprintln!("ERROR: Failed to initialize DAO: {}", e);
            process::exit(1);
        }
    };
    
    // Start the API server
    if let Err(e) = run_api_server(&dao).await {
        eprintln!("ERROR: API server failed: {}", e);
        process::exit(1);
    }
}

async fn init_blockchain(config: &config::Config) -> Result<impl atomsi_dao::blockchain::BlockchainAdapter, String> {
    // This is a placeholder - in a real implementation, we would initialize the
    // appropriate blockchain adapter based on the configuration
    
    #[cfg(feature = "ethereum")]
    {
        use atomsi_dao::blockchain::ethereum::EthereumAdapter;
        Ok(EthereumAdapter::new(&config.blockchain).await.map_err(|e| e.to_string())?)
    }
    
    #[cfg(all(not(feature = "ethereum"), feature = "solana"))]
    {
        use atomsi_dao::blockchain::solana::SolanaAdapter;
        Ok(SolanaAdapter::new(&config.blockchain).await.map_err(|e| e.to_string())?)
    }
    
    #[cfg(all(not(feature = "ethereum"), not(feature = "solana"), feature = "polkadot"))]
    {
        use atomsi_dao::blockchain::polkadot::PolkadotAdapter;
        Ok(PolkadotAdapter::new(&config.blockchain).await.map_err(|e| e.to_string())?)
    }
    
    #[cfg(all(not(feature = "ethereum"), not(feature = "solana"), not(feature = "polkadot")))]
    {
        Err("No blockchain adapter enabled. Enable at least one blockchain feature.".to_string())
    }
}

async fn init_database(config: &config::Config) -> Result<atomsi_dao::core::Database, String> {
    // This is a placeholder - in a real implementation, we would initialize the
    // database connection based on the configuration
    atomsi_dao::core::Database::connect(&config.database)
        .await
        .map_err(|e| e.to_string())
}

async fn run_api_server(dao: &Dao) -> Result<(), String> {
    use axum::{
        routing::{get, post},
        Router,
    };
    use std::net::SocketAddr;
    
    info!("Starting API server on {}", dao.config().api.address);
    
    // Build our application with routes
    let app = Router::new()
        .route("/", get(|| async { "AtomSi DAO API" }))
        .route("/health", get(|| async { "OK" }))
        .route("/version", get(|| async { atomsi_dao::VERSION }))
        // Add more routes here
        ;
    
    // Parse the socket address
    let addr: SocketAddr = dao.config().api.address.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;
    
    // Run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| format!("Server error: {}", e))?;
    
    Ok(())
} 