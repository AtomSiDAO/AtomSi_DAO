//! Proposals module for AtomSi DAO
//!
//! This module provides functionality for creating, tracking, and executing
//! proposals within the DAO.

mod types;

pub use types::{Proposal, ProposalId, ProposalState, ProposalType, ProposalVote};

use crate::{
    blockchain::BlockchainAdapter,
    config::Config,
    core::{Database, DaoError, Result},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Proposal builder for creating new proposals
pub struct ProposalBuilder {
    title: Option<String>,
    description: Option<String>,
    proposal_type: Option<ProposalType>,
    proposer: Option<String>,
    metadata: serde_json::Value,
}

impl ProposalBuilder {
    /// Create a new proposal builder
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            proposal_type: None,
            proposer: None,
            metadata: serde_json::Value::Null,
        }
    }
    
    /// Set the proposal title
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }
    
    /// Set the proposal description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Set the proposal type
    pub fn proposal_type(mut self, proposal_type: ProposalType) -> Self {
        self.proposal_type = Some(proposal_type);
        self
    }
    
    /// Set the proposer address
    pub fn proposer<S: Into<String>>(mut self, proposer: S) -> Self {
        self.proposer = Some(proposer.into());
        self
    }
    
    /// Set additional metadata for the proposal
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Build the proposal
    pub fn build(self) -> Result<Proposal> {
        let title = self.title.ok_or_else(|| {
            DaoError::InvalidParameter("Proposal title is required".to_string())
        })?;
        
        let description = self.description.ok_or_else(|| {
            DaoError::InvalidParameter("Proposal description is required".to_string())
        })?;
        
        let proposal_type = self.proposal_type.ok_or_else(|| {
            DaoError::InvalidParameter("Proposal type is required".to_string())
        })?;
        
        let proposer = self.proposer.ok_or_else(|| {
            DaoError::InvalidParameter("Proposer is required".to_string())
        })?;
        
        Ok(Proposal {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            proposal_type,
            proposer,
            state: ProposalState::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            voting_starts_at: None,
            voting_ends_at: None,
            execution_date: None,
            metadata: self.metadata,
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
            votes: Vec::new(),
        })
    }
}

impl Default for ProposalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager for proposal operations
pub struct ProposalManager {
    config: Arc<Config>,
    blockchain: Arc<dyn BlockchainAdapter>,
    database: Database,
}

impl ProposalManager {
    /// Create a new proposal manager
    pub fn new(
        config: &Config,
        blockchain: impl BlockchainAdapter + 'static,
        database: Database,
    ) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config.clone()),
            blockchain: Arc::new(blockchain),
            database,
        })
    }
    
    /// Submit a proposal
    pub async fn submit_proposal(&self, proposal: Proposal) -> Result<ProposalId> {
        // Check if the proposer has enough tokens
        let proposer_balance = self
            .blockchain
            .balance(&proposal.proposer)
            .await
            .map_err(|e| DaoError::BlockchainError(e))?;
        
        if proposer_balance < self.config.governance.proposal_threshold {
            return Err(DaoError::Unauthorized);
        }
        
        // Check if the proposal is valid
        self.validate_proposal(&proposal)?;
        
        // Save the proposal to the database
        self.save_proposal(&proposal).await?;
        
        Ok(proposal.id)
    }
    
    /// Get a proposal by ID
    pub async fn get_proposal(&self, id: &ProposalId) -> Result<Proposal> {
        // Load the proposal from the database
        let query = "SELECT * FROM proposals WHERE id = $1";
        let row = self.database.query_one(query, &[&id]).await?;
        
        // Parse the proposal from the row
        let proposal: Proposal = serde_json::from_value(row.get("data"))
            .map_err(|e| DaoError::DatabaseError(format!("Failed to parse proposal: {}", e)))?;
        
        Ok(proposal)
    }
    
    /// Get all proposals
    pub async fn get_proposals(&self, state: Option<ProposalState>) -> Result<Vec<Proposal>> {
        // Construct the query based on the state filter
        let (query, params) = match state {
            Some(state) => {
                let query = "SELECT * FROM proposals WHERE state = $1 ORDER BY created_at DESC";
                let state_str = serde_json::to_string(&state)
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize state: {}", e)))?;
                (query, vec![&state_str as &(dyn tokio_postgres::types::ToSql + Sync)])
            }
            None => {
                let query = "SELECT * FROM proposals ORDER BY created_at DESC";
                (query, Vec::new())
            }
        };
        
        // Load the proposals from the database
        let rows = self.database.query(query, &params).await?;
        
        // Parse the proposals from the rows
        let proposals = rows
            .into_iter()
            .map(|row| {
                serde_json::from_value(row.get("data"))
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to parse proposal: {}", e)))
            })
            .collect::<Result<Vec<Proposal>>>()?;
        
        Ok(proposals)
    }
    
    /// Vote on a proposal
    pub async fn vote(
        &self,
        proposal_id: &ProposalId,
        voter: &str,
        vote: ProposalVote,
    ) -> Result<()> {
        // Load the proposal
        let mut proposal = self.get_proposal(proposal_id).await?;
        
        // Check if the proposal is in the voting state
        if proposal.state != ProposalState::Voting {
            return Err(DaoError::InvalidParameter(
                "Proposal is not in the voting state".to_string(),
            ));
        }
        
        // Check if the voting period is active
        let now = Utc::now();
        let voting_starts_at = proposal
            .voting_starts_at
            .ok_or_else(|| DaoError::InternalError("Voting start time not set".to_string()))?;
        let voting_ends_at = proposal
            .voting_ends_at
            .ok_or_else(|| DaoError::InternalError("Voting end time not set".to_string()))?;
        
        if now < voting_starts_at {
            return Err(DaoError::InvalidParameter(
                "Voting has not started yet".to_string(),
            ));
        }
        
        if now > voting_ends_at {
            return Err(DaoError::InvalidParameter("Voting has ended".to_string()));
        }
        
        // Check if the voter has already voted
        if proposal.votes.iter().any(|v| v.voter == voter) {
            return Err(DaoError::InvalidParameter(
                "Voter has already voted".to_string(),
            ));
        }
        
        // Get the voter's voting power
        let voting_power = self
            .blockchain
            .balance(voter)
            .await
            .map_err(|e| DaoError::BlockchainError(e))?;
        
        if voting_power == 0 {
            return Err(DaoError::Unauthorized);
        }
        
        // Update the vote counts
        match vote {
            ProposalVote::Yes => {
                proposal.yes_votes += voting_power;
            }
            ProposalVote::No => {
                proposal.no_votes += voting_power;
            }
            ProposalVote::Abstain => {
                proposal.abstain_votes += voting_power;
            }
        }
        
        // Record the vote
        proposal.votes.push(types::Vote {
            voter: voter.to_string(),
            vote,
            voting_power,
            timestamp: Utc::now(),
        });
        
        // Update the proposal in the database
        proposal.updated_at = Utc::now();
        self.save_proposal(&proposal).await?;
        
        Ok(())
    }
    
    /// Execute a proposal
    pub async fn execute_proposal(&self, proposal_id: &ProposalId) -> Result<()> {
        // Load the proposal
        let mut proposal = self.get_proposal(proposal_id).await?;
        
        // Check if the proposal is in the approved state
        if proposal.state != ProposalState::Approved {
            return Err(DaoError::InvalidParameter(
                "Proposal is not in the approved state".to_string(),
            ));
        }
        
        // Execute the proposal based on its type
        match proposal.proposal_type {
            ProposalType::Transfer { to, amount, token } => {
                // Execute the transfer
                self.blockchain
                    .send_transaction(&to, amount)
                    .await
                    .map_err(|e| DaoError::BlockchainError(e))?;
            }
            ProposalType::ContractCall {
                contract,
                function,
                args,
            } => {
                // Call the contract function
                self.blockchain
                    .call_contract(&contract, &function, &args)
                    .await
                    .map_err(|e| DaoError::BlockchainError(e))?;
            }
            ProposalType::ParameterChange { parameter, value } => {
                // Update the parameter
                // This would typically involve a governance contract call
                return Err(DaoError::NotSupported(
                    "Parameter changes not yet implemented".to_string(),
                ));
            }
            ProposalType::TextProposal { .. } => {
                // Text proposals don't require execution
            }
        }
        
        // Update the proposal state
        proposal.state = ProposalState::Executed;
        proposal.execution_date = Some(Utc::now());
        proposal.updated_at = Utc::now();
        
        // Save the updated proposal
        self.save_proposal(&proposal).await?;
        
        Ok(())
    }
    
    /// Process proposals (check voting periods, finalize votes, etc.)
    pub async fn process_proposals(&self) -> Result<()> {
        // Get all active proposals
        let mut proposals = self.get_proposals(Some(ProposalState::Voting)).await?;
        
        let now = Utc::now();
        
        for proposal in &mut proposals {
            // Check if the voting period has ended
            if let Some(voting_ends_at) = proposal.voting_ends_at {
                if now > voting_ends_at {
                    // Finalize the vote
                    self.finalize_vote(proposal).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Start the voting period for a proposal
    pub async fn start_voting(&self, proposal_id: &ProposalId) -> Result<()> {
        // Load the proposal
        let mut proposal = self.get_proposal(proposal_id).await?;
        
        // Check if the proposal is in the draft state
        if proposal.state != ProposalState::Draft {
            return Err(DaoError::InvalidParameter(
                "Proposal is not in the draft state".to_string(),
            ));
        }
        
        // Set the voting period
        let now = Utc::now();
        let voting_period = chrono::Duration::days(self.config.governance.voting_period_days as i64);
        
        proposal.state = ProposalState::Voting;
        proposal.voting_starts_at = Some(now);
        proposal.voting_ends_at = Some(now + voting_period);
        proposal.updated_at = now;
        
        // Save the updated proposal
        self.save_proposal(&proposal).await?;
        
        Ok(())
    }
    
    /// Cancel a proposal
    pub async fn cancel_proposal(&self, proposal_id: &ProposalId, canceller: &str) -> Result<()> {
        // Load the proposal
        let mut proposal = self.get_proposal(proposal_id).await?;
        
        // Check if the proposal is in a cancellable state
        if proposal.state != ProposalState::Draft && proposal.state != ProposalState::Voting {
            return Err(DaoError::InvalidParameter(
                "Proposal cannot be cancelled in its current state".to_string(),
            ));
        }
        
        // Check if the canceller is the proposer
        if proposal.proposer != canceller {
            return Err(DaoError::Unauthorized);
        }
        
        // Update the proposal state
        proposal.state = ProposalState::Cancelled;
        proposal.updated_at = Utc::now();
        
        // Save the updated proposal
        self.save_proposal(&proposal).await?;
        
        Ok(())
    }
    
    // Private methods
    
    /// Validate a proposal
    fn validate_proposal(&self, proposal: &Proposal) -> Result<()> {
        // Check the title
        if proposal.title.is_empty() {
            return Err(DaoError::InvalidParameter(
                "Proposal title cannot be empty".to_string(),
            ));
        }
        
        // Check the description
        if proposal.description.is_empty() {
            return Err(DaoError::InvalidParameter(
                "Proposal description cannot be empty".to_string(),
            ));
        }
        
        // Validate based on proposal type
        match &proposal.proposal_type {
            ProposalType::Transfer { to, amount, token } => {
                // Check if the address is valid
                if !self.blockchain.is_valid_address(to) {
                    return Err(DaoError::InvalidParameter(
                        "Invalid recipient address".to_string(),
                    ));
                }
                
                // Check if the amount is valid
                if *amount == 0 {
                    return Err(DaoError::InvalidParameter(
                        "Transfer amount must be greater than 0".to_string(),
                    ));
                }
                
                // Check if the token is valid
                if token.is_empty() {
                    return Err(DaoError::InvalidParameter(
                        "Token symbol cannot be empty".to_string(),
                    ));
                }
            }
            ProposalType::ContractCall {
                contract,
                function,
                args,
            } => {
                // Check if the contract address is valid
                if !self.blockchain.is_valid_address(contract) {
                    return Err(DaoError::InvalidParameter(
                        "Invalid contract address".to_string(),
                    ));
                }
                
                // Check if the function name is valid
                if function.is_empty() {
                    return Err(DaoError::InvalidParameter(
                        "Function name cannot be empty".to_string(),
                    ));
                }
            }
            ProposalType::ParameterChange { parameter, value } => {
                // Check if the parameter is valid
                if parameter.is_empty() {
                    return Err(DaoError::InvalidParameter(
                        "Parameter name cannot be empty".to_string(),
                    ));
                }
                
                // Check if the value is valid
                if value.is_null() {
                    return Err(DaoError::InvalidParameter(
                        "Parameter value cannot be null".to_string(),
                    ));
                }
            }
            ProposalType::TextProposal { .. } => {
                // Text proposals don't require additional validation
            }
        }
        
        Ok(())
    }
    
    /// Save a proposal to the database
    async fn save_proposal(&self, proposal: &Proposal) -> Result<()> {
        // Serialize the proposal
        let data = serde_json::to_value(proposal)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize proposal: {}", e)))?;
        
        // Check if the proposal already exists
        let exists = self
            .database
            .query_opt("SELECT 1 FROM proposals WHERE id = $1", &[&proposal.id])
            .await?
            .is_some();
        
        if exists {
            // Update the proposal
            self.database
                .execute(
                    "UPDATE proposals SET data = $1, state = $2, updated_at = $3 WHERE id = $4",
                    &[
                        &data,
                        &serde_json::to_string(&proposal.state).unwrap(),
                        &proposal.updated_at,
                        &proposal.id,
                    ],
                )
                .await?;
        } else {
            // Insert the proposal
            self.database
                .execute(
                    "INSERT INTO proposals (id, data, state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
                    &[
                        &proposal.id,
                        &data,
                        &serde_json::to_string(&proposal.state).unwrap(),
                        &proposal.created_at,
                        &proposal.updated_at,
                    ],
                )
                .await?;
        }
        
        Ok(())
    }
    
    /// Finalize the vote for a proposal
    async fn finalize_vote(&self, proposal: &mut Proposal) -> Result<()> {
        // Calculate the total votes
        let total_votes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
        
        // Calculate the quorum
        let quorum_threshold = (self.config.governance.quorum_percentage as u64 * total_votes) / 100;
        
        // Calculate the majority
        let majority_threshold =
            (self.config.governance.majority_percentage as u64 * (proposal.yes_votes + proposal.no_votes)) / 100;
        
        // Check if the proposal has reached quorum
        if total_votes < quorum_threshold {
            proposal.state = ProposalState::Rejected;
            proposal.updated_at = Utc::now();
            return self.save_proposal(proposal).await;
        }
        
        // Check if the proposal has reached majority
        if proposal.yes_votes >= majority_threshold {
            proposal.state = ProposalState::Approved;
        } else {
            proposal.state = ProposalState::Rejected;
        }
        
        proposal.updated_at = Utc::now();
        self.save_proposal(proposal).await
    }
} 