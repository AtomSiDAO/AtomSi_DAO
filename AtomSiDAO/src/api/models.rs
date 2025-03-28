//! API models for AtomSi DAO
//!
//! This module contains request and response models for API endpoints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// General models

/// API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    /// Success status
    pub success: bool,
    /// Response data (if success is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error message (if success is false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful API response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    /// Create an error API response
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

/// Pagination parameters for list requests
#[derive(Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: usize,
    /// Items per page
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_page() -> usize {
    1
}

fn default_limit() -> usize {
    20
}

/// Pagination metadata
#[derive(Serialize)]
pub struct PaginationMeta {
    /// Current page
    pub page: usize,
    /// Items per page
    pub limit: usize,
    /// Total items
    pub total: usize,
    /// Total pages
    pub total_pages: usize,
}

/// Paginated response
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    /// Items
    pub items: Vec<T>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

// Auth models

/// Login request
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Ethereum address
    pub address: String,
    /// Signature of the message
    pub signature: String,
    /// Message that was signed
    pub message: String,
}

/// Login response
#[derive(Serialize)]
pub struct LoginResponse {
    /// JWT token for authentication
    pub token: String,
    /// Member information
    pub member: MemberResponse,
}

// Member models

/// Member response
#[derive(Serialize)]
pub struct MemberResponse {
    /// Member ID
    pub id: String,
    /// Ethereum address
    pub address: String,
    /// Name
    pub name: String,
    /// Role
    pub role: String,
    /// Status
    pub status: String,
    /// Reputation
    pub reputation: i32,
    /// Joined timestamp
    pub joined_at: u64,
    /// Last active timestamp
    pub last_active_at: u64,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Member activity response
#[derive(Serialize)]
pub struct ActivityResponse {
    /// Activity ID
    pub id: String,
    /// Member ID
    pub member_id: String,
    /// Activity type
    pub activity_type: String,
    /// Related object ID
    pub related_id: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// Description
    pub description: Option<String>,
    /// Reputation change
    pub reputation_change: i32,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

// Proposal models

/// Create proposal request
#[derive(Deserialize)]
pub struct CreateProposalRequest {
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Proposal type
    pub proposal_type: String,
    /// Voting start time (optional, immediate if not provided)
    pub voting_starts_at: Option<u64>,
    /// Voting duration in seconds
    pub voting_duration: u64,
    /// Execution data for on-chain actions
    pub execution_data: Option<serde_json::Value>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Proposal response
#[derive(Serialize)]
pub struct ProposalResponse {
    /// Proposal ID
    pub id: String,
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Proposer ID
    pub proposer_id: String,
    /// Proposer information
    pub proposer: MemberResponse,
    /// Proposal type
    pub proposal_type: String,
    /// Proposal status
    pub status: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Voting start timestamp
    pub voting_starts_at: Option<u64>,
    /// Voting end timestamp
    pub voting_ends_at: Option<u64>,
    /// Execution timestamp
    pub executed_at: Option<u64>,
    /// Execution data
    pub execution_data: Option<serde_json::Value>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    /// Vote counts
    pub vote_counts: VoteCounts,
}

/// Vote counts
#[derive(Serialize)]
pub struct VoteCounts {
    /// For votes
    pub for_votes: u64,
    /// Against votes
    pub against_votes: u64,
    /// Abstain votes
    pub abstain_votes: u64,
    /// Total votes
    pub total: u64,
}

/// Vote request
#[derive(Deserialize)]
pub struct VoteRequest {
    /// Vote choice
    pub choice: String,
    /// Vote weight
    pub weight: Option<u64>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Vote response
#[derive(Serialize)]
pub struct VoteResponse {
    /// Vote ID
    pub id: String,
    /// Proposal ID
    pub proposal_id: String,
    /// Voter ID
    pub voter_id: String,
    /// Voter information
    pub voter: MemberResponse,
    /// Vote choice
    pub choice: String,
    /// Vote weight
    pub weight: u64,
    /// Vote timestamp
    pub voted_at: u64,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

// Treasury models

/// Create transaction request
#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    /// Transaction description
    pub description: String,
    /// Recipient address
    pub recipient_address: String,
    /// Token symbol
    pub token_symbol: String,
    /// Amount
    pub amount: String,
    /// Related proposal ID (optional)
    pub related_proposal_id: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Transaction response
#[derive(Serialize)]
pub struct TransactionResponse {
    /// Transaction ID
    pub id: String,
    /// Transaction description
    pub description: String,
    /// Recipient address
    pub recipient_address: String,
    /// Token symbol
    pub token_symbol: String,
    /// Amount
    pub amount: String,
    /// Transaction status
    pub status: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Execution timestamp
    pub executed_at: Option<u64>,
    /// Required approvals
    pub required_approvals: u32,
    /// Current approvals
    pub current_approvals: u32,
    /// Related proposal ID
    pub related_proposal_id: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    /// Approvers
    pub approvers: Vec<MemberResponse>,
}

/// Treasury balance response
#[derive(Serialize)]
pub struct TreasuryBalanceResponse {
    /// Token symbol
    pub token_symbol: String,
    /// Token name
    pub token_name: String,
    /// Balance
    pub balance: String,
    /// Formatted balance with symbol
    pub formatted_balance: String,
    /// Token details
    pub token: Option<TokenResponse>,
}

// Token models

/// Token response
#[derive(Serialize)]
pub struct TokenResponse {
    /// Token ID
    pub id: String,
    /// Token symbol
    pub token_symbol: String,
    /// Token name
    pub token_name: String,
    /// Token type
    pub token_type: String,
    /// Decimals
    pub decimals: u8,
    /// Total supply
    pub total_supply: Option<String>,
    /// Contract address
    pub contract_address: Option<String>,
    /// Chain ID
    pub chain_id: Option<u64>,
    /// Creation timestamp
    pub created_at: u64,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Token balance response
#[derive(Serialize)]
pub struct TokenBalanceResponse {
    /// Balance ID
    pub id: String,
    /// Token ID
    pub token_id: String,
    /// Token information
    pub token: TokenResponse,
    /// Member ID
    pub member_id: String,
    /// Member information
    pub member: Option<MemberResponse>,
    /// Balance
    pub balance: String,
    /// Formatted balance with symbol
    pub formatted_balance: String,
    /// Last update timestamp
    pub last_updated: u64,
}

/// Token transfer request
#[derive(Deserialize)]
pub struct TokenTransferRequest {
    /// Token ID
    pub token_id: String,
    /// Recipient ID or address
    pub to: String,
    /// Amount
    pub amount: String,
    /// Description
    pub description: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Token transfer response
#[derive(Serialize)]
pub struct TokenTransferResponse {
    /// Transfer ID
    pub id: String,
    /// Token ID
    pub token_id: String,
    /// Token information
    pub token: TokenResponse,
    /// Sender member ID
    pub from_member_id: String,
    /// Sender information
    pub from_member: Option<MemberResponse>,
    /// Recipient member ID
    pub to_member_id: String,
    /// Recipient information
    pub to_member: Option<MemberResponse>,
    /// Amount
    pub amount: String,
    /// Formatted amount with symbol
    pub formatted_amount: String,
    /// Transfer timestamp
    pub timestamp: u64,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
} 