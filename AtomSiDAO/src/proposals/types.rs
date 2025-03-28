//! Types for the proposals module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Proposal ID type
pub type ProposalId = String;

/// Proposal state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalState {
    /// Proposal is in draft state
    Draft,
    /// Proposal is in voting state
    Voting,
    /// Proposal has been approved
    Approved,
    /// Proposal has been rejected
    Rejected,
    /// Proposal has been executed
    Executed,
    /// Proposal has been cancelled
    Cancelled,
}

/// Proposal type enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    /// Transfer funds
    Transfer {
        /// Recipient address
        to: String,
        /// Amount to transfer
        amount: u64,
        /// Token symbol
        token: String,
    },
    /// Call a contract function
    ContractCall {
        /// Contract address
        contract: String,
        /// Function name
        function: String,
        /// Function arguments
        args: Vec<serde_json::Value>,
    },
    /// Change a parameter
    ParameterChange {
        /// Parameter name
        parameter: String,
        /// Parameter value
        value: serde_json::Value,
    },
    /// Text proposal
    TextProposal {
        /// Additional metadata
        metadata: serde_json::Value,
    },
}

/// Proposal vote enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalVote {
    /// Vote in favor
    Yes,
    /// Vote against
    No,
    /// Abstain from voting
    Abstain,
}

/// Vote record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    /// Voter address
    pub voter: String,
    /// Vote choice
    pub vote: ProposalVote,
    /// Voting power
    pub voting_power: u64,
    /// Timestamp of the vote
    pub timestamp: DateTime<Utc>,
}

/// Proposal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Proposal ID
    pub id: ProposalId,
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Proposal type
    pub proposal_type: ProposalType,
    /// Proposer address
    pub proposer: String,
    /// Proposal state
    pub state: ProposalState,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Voting start timestamp
    pub voting_starts_at: Option<DateTime<Utc>>,
    /// Voting end timestamp
    pub voting_ends_at: Option<DateTime<Utc>>,
    /// Execution timestamp
    pub execution_date: Option<DateTime<Utc>>,
    /// Additional metadata
    pub metadata: serde_json::Value,
    /// Yes votes count (weighted)
    pub yes_votes: u64,
    /// No votes count (weighted)
    pub no_votes: u64,
    /// Abstain votes count (weighted)
    pub abstain_votes: u64,
    /// Individual votes
    pub votes: Vec<Vote>,
} 