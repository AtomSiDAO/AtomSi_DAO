//! Core module for AtomSi DAO
//!
//! This module provides the main DAO implementation and core functionality.

mod database;
mod error;

pub use database::Database;
pub use error::{DaoError, Result};

use crate::{
    blockchain::BlockchainAdapter,
    config::Config,
    governance::GovernanceEngine,
    identity::IdentityManager,
    proposals::ProposalManager,
    token::TokenManager,
    treasury::TreasuryManager,
};

/// Main DAO structure
pub struct Dao {
    /// DAO configuration
    config: Config,
    
    /// Governance engine
    governance: GovernanceEngine,
    
    /// Token manager
    token_manager: TokenManager,
    
    /// Treasury manager
    treasury_manager: TreasuryManager,
    
    /// Proposal manager
    proposal_manager: ProposalManager,
    
    /// Identity manager
    identity_manager: IdentityManager,
}

impl Dao {
    /// Create a new DAO instance
    pub fn new(
        config: Config,
        blockchain: impl BlockchainAdapter + 'static,
        database: Database,
    ) -> Result<Self> {
        // Initialize managers
        let token_manager = TokenManager::new(&config, blockchain.clone(), database.clone())?;
        let identity_manager = IdentityManager::new(&config, blockchain.clone(), database.clone())?;
        let treasury_manager = TreasuryManager::new(&config, blockchain.clone(), database.clone())?;
        let proposal_manager = ProposalManager::new(&config, blockchain.clone(), database.clone())?;
        let governance = GovernanceEngine::new(
            &config,
            blockchain,
            database,
            &token_manager,
            &proposal_manager,
        )?;
        
        Ok(Self {
            config,
            governance,
            token_manager,
            treasury_manager,
            proposal_manager,
            identity_manager,
        })
    }
    
    /// Get the DAO configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Get the governance engine
    pub fn governance(&self) -> &GovernanceEngine {
        &self.governance
    }
    
    /// Get the token manager
    pub fn token_manager(&self) -> &TokenManager {
        &self.token_manager
    }
    
    /// Get the treasury manager
    pub fn treasury_manager(&self) -> &TreasuryManager {
        &self.treasury_manager
    }
    
    /// Get the proposal manager
    pub fn proposal_manager(&self) -> &ProposalManager {
        &self.proposal_manager
    }
    
    /// Get the identity manager
    pub fn identity_manager(&self) -> &IdentityManager {
        &self.identity_manager
    }
}

/// Builder for DAO instances
pub struct DaoBuilder {
    config: Option<Config>,
    blockchain: Option<Box<dyn BlockchainAdapter>>,
    database: Option<Database>,
}

impl DaoBuilder {
    /// Create a new DAO builder
    pub fn new() -> Self {
        Self {
            config: None,
            blockchain: None,
            database: None,
        }
    }
    
    /// Set the configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Set the blockchain adapter
    pub fn with_blockchain(mut self, blockchain: impl BlockchainAdapter + 'static) -> Self {
        self.blockchain = Some(Box::new(blockchain));
        self
    }
    
    /// Set the database
    pub fn with_database(mut self, database: Database) -> Self {
        self.database = Some(database);
        self
    }
    
    /// Build the DAO instance
    pub fn build(self) -> Result<Dao> {
        let config = self.config.ok_or(DaoError::MissingConfig)?;
        let blockchain = self.blockchain.ok_or(DaoError::MissingBlockchainAdapter)?;
        let database = self.database.ok_or(DaoError::MissingDatabase)?;
        
        Dao::new(config, blockchain, database)
    }
}

impl Default for DaoBuilder {
    fn default() -> Self {
        Self::new()
    }
} 