//! Governance module for AtomSi DAO
//!
//! This module provides functionality for governance operations
//! including voting strategies, delegation, and vote counting.

mod strategies;

pub use strategies::{VoteWeight, VotingStrategy};

use crate::{
    blockchain::BlockchainAdapter,
    config::Config,
    core::{Database, DaoError, Result},
    proposals::{ProposalManager, ProposalVote},
};
use std::sync::Arc;

/// Re-export proposal vote type for convenience
pub use crate::proposals::ProposalVote as Vote;

/// Governance engine
pub struct GovernanceEngine {
    /// Configuration
    config: Arc<Config>,
    /// Blockchain adapter
    blockchain: Arc<dyn BlockchainAdapter>,
    /// Database
    database: Database,
    /// Token manager reference
    token_manager: Arc<crate::token::TokenManager>,
    /// Proposal manager reference
    proposal_manager: Arc<ProposalManager>,
    /// Voting strategy
    voting_strategy: Box<dyn VotingStrategy>,
}

impl GovernanceEngine {
    /// Create a new governance engine
    pub fn new(
        config: &Config,
        blockchain: impl BlockchainAdapter + 'static,
        database: Database,
        token_manager: &crate::token::TokenManager,
        proposal_manager: &ProposalManager,
    ) -> Result<Self> {
        // Create the default voting strategy based on configuration
        let voting_strategy: Box<dyn VotingStrategy> = match config.dao.governance_token.as_str() {
            // If the governance token is set to "Quadratic", use quadratic voting
            "Quadratic" => Box::new(strategies::QuadraticVoting::new()),
            // If the governance token is set to "Conviction", use conviction voting
            "Conviction" => Box::new(strategies::ConvictionVoting::new()),
            // Otherwise, use token-weighted voting
            _ => Box::new(strategies::TokenWeightedVoting::new()),
        };
        
        Ok(Self {
            config: Arc::new(config.clone()),
            blockchain: Arc::new(blockchain),
            database,
            token_manager: Arc::new(token_manager.clone()),
            proposal_manager: Arc::new(proposal_manager.clone()),
            voting_strategy,
        })
    }
    
    /// Set the voting strategy
    pub fn set_voting_strategy(&mut self, strategy: Box<dyn VotingStrategy>) {
        self.voting_strategy = strategy;
    }
    
    /// Get the voting weight for an address
    pub async fn get_voting_weight(&self, address: &str) -> Result<VoteWeight> {
        // Get the token balance for the address
        let balance = self
            .blockchain
            .balance(address)
            .await
            .map_err(|e| DaoError::BlockchainError(e))?;
        
        // Calculate the voting weight using the strategy
        self.voting_strategy.calculate_weight(address, balance)
    }
    
    /// Get the voting power for an address
    pub async fn get_voting_power(&self, address: &str) -> Result<u64> {
        // Get the voting weight
        let weight = self.get_voting_weight(address).await?;
        
        // Return the weight value
        Ok(weight.value)
    }
    
    /// Submit a vote
    pub async fn submit_vote(
        &self,
        proposal_id: &str,
        voter: &str,
        vote: Vote,
    ) -> Result<()> {
        // Submit the vote through the proposal manager
        self.proposal_manager.vote(proposal_id, voter, vote).await
    }
    
    /// Delegate voting power
    pub async fn delegate_voting_power(&self, delegator: &str, delegate: &str) -> Result<()> {
        // Check if the delegator has enough tokens
        let delegator_balance = self
            .blockchain
            .balance(delegator)
            .await
            .map_err(|e| DaoError::BlockchainError(e))?;
        
        if delegator_balance == 0 {
            return Err(DaoError::Unauthorized);
        }
        
        // Save the delegation in the database
        self.database
            .execute(
                "INSERT INTO delegations (delegator, delegate, amount, created_at) VALUES ($1, $2, $3, $4)",
                &[
                    &delegator,
                    &delegate,
                    &delegator_balance,
                    &chrono::Utc::now(),
                ],
            )
            .await?;
        
        Ok(())
    }
    
    /// Revoke delegation
    pub async fn revoke_delegation(&self, delegator: &str, delegate: &str) -> Result<()> {
        // Remove the delegation from the database
        self.database
            .execute(
                "DELETE FROM delegations WHERE delegator = $1 AND delegate = $2",
                &[&delegator, &delegate],
            )
            .await?;
        
        Ok(())
    }
    
    /// Get delegated voting power
    pub async fn get_delegated_voting_power(&self, delegate: &str) -> Result<u64> {
        // Query delegations for the delegate
        let rows = self
            .database
            .query(
                "SELECT SUM(amount) as total FROM delegations WHERE delegate = $1",
                &[&delegate],
            )
            .await?;
        
        // Extract the total delegated power
        let total = if rows.is_empty() {
            0
        } else {
            rows[0].get::<_, Option<i64>>("total").unwrap_or(0) as u64
        };
        
        Ok(total)
    }
    
    /// Process governance operations (e.g., update voting power, check proposal states)
    pub async fn process(&self) -> Result<()> {
        // Process proposals
        self.proposal_manager.process_proposals().await?;
        
        Ok(())
    }
} 