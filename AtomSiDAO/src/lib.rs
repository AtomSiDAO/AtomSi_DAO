//! AtomSi DAO Framework
//!
//! A comprehensive framework for building decentralized autonomous organizations.
//!
//! # Architecture
//!
//! The framework is organized into several core modules:
//!
//! - `blockchain`: Adapters for various blockchain networks
//! - `config`: Configuration management
//! - `crypto`: Cryptographic utilities
//! - `database`: Database connection and management
//! - `error`: Error types and handling
//! - `governance`: Governance mechanisms and voting
//! - `identity`: Member identity and reputation
//! - `proposal`: Proposal creation and management
//! - `security`: Authentication, authorization, and permissions
//! - `token`: Token management
//! - `treasury`: Treasury management
//! - `utils`: Utility functions and helpers
//! - `api`: API endpoints for interacting with the DAO
//!
//! # Example
//!
//! ```rust,no_run
//! use atomsi_dao::{
//!     config::ConfigManager,
//!     database::DatabaseManager,
//!     blockchain::BlockchainAdapter,
//! };
//!
//! async fn start_dao() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config_manager = ConfigManager::with_defaults("config.json");
//!     let config = config_manager.get_config();
//!     
//!     // Connect to database
//!     let db_manager = DatabaseManager::new(&config.database).await?;
//!     
//!     // Connect to blockchain
//!     let blockchain = BlockchainAdapter::new(&config.blockchain)?;
//!     
//!     // Initialize and run the DAO
//!     // ...
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]

// Re-export major modules
pub mod api;
pub mod blockchain;
pub mod config;
pub mod crypto;
pub mod database;
pub mod error;
pub mod governance;
pub mod identity;
pub mod proposal;
pub mod security;
pub mod token;
pub mod treasury;
pub mod utils;

// Re-export commonly used types
pub use error::{Error, Result};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Initialize the DAO framework with the given configuration file
pub async fn init(config_path: &str) -> Result<DAOContext> {
    // Load configuration
    let config_manager = config::ConfigManager::new(config_path)?;
    let config = config_manager.get_config();
    
    // Initialize database
    let db_manager = database::DatabaseManager::new(&config.database).await?;
    
    // Initialize blockchain adapter
    let blockchain = blockchain::BlockchainAdapter::new(&config.blockchain)?;
    
    // Create and return the DAO context
    Ok(DAOContext {
        config_manager,
        db_manager,
        blockchain,
    })
}

/// Main context for the DAO framework
pub struct DAOContext {
    pub config_manager: config::ConfigManager,
    pub db_manager: database::DatabaseManager,
    pub blockchain: blockchain::BlockchainAdapter,
}

impl DAOContext {
    /// Create a new governance manager
    pub fn governance_manager(&self) -> governance::GovernanceManager {
        governance::GovernanceManager::new(
            self.db_manager.clone(),
            self.blockchain.clone(),
        )
    }
    
    /// Create a new proposal manager
    pub fn proposal_manager(&self) -> proposal::ProposalManager {
        proposal::ProposalManager::new(
            self.db_manager.clone(),
            self.blockchain.clone(),
        )
    }
    
    /// Create a new treasury manager
    pub fn treasury_manager(&self) -> treasury::TreasuryManager {
        treasury::TreasuryManager::new(
            self.db_manager.clone(),
            self.blockchain.clone(),
        )
    }
    
    /// Create a new identity manager
    pub fn identity_manager(&self) -> identity::IdentityManager {
        identity::IdentityManager::new(
            self.db_manager.clone(),
        )
    }
    
    /// Create a new token manager
    pub fn token_manager(&self) -> token::TokenManager {
        token::TokenManager::new(
            self.db_manager.clone(),
            self.blockchain.clone(),
        )
    }
}

/// Shutdown the DAO framework gracefully
pub async fn shutdown(context: DAOContext) -> Result<()> {
    // Close database connections
    context.db_manager.close().await?;
    
    Ok(())
}

/// Initialize logging for the AtomSi DAO framework
pub fn init_logging() {
    env_logger::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
} 