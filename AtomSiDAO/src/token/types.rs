//! Types for the token module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Token ID type
pub type TokenId = String;

/// Token amount type
pub type TokenAmount = u64;

/// Token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token ID
    pub id: TokenId,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Total supply
    pub total_supply: TokenAmount,
    /// Decimals
    pub decimals: u8,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Token balance structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Token symbol
    pub symbol: String,
    /// Owner address
    pub address: String,
    /// Balance amount
    pub balance: TokenAmount,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Token transfer structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    /// Transfer ID
    pub id: String,
    /// Token symbol
    pub symbol: String,
    /// Sender address
    pub from_address: String,
    /// Recipient address
    pub to_address: String,
    /// Transfer amount
    pub amount: TokenAmount,
    /// Transfer timestamp
    pub timestamp: DateTime<Utc>,
    /// Transaction hash (if available)
    pub transaction_hash: Option<String>,
}

/// Token event type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenEventType {
    /// Token minting
    Mint,
    /// Token burning
    Burn,
    /// Token transfer
    Transfer,
    /// Token approval
    Approval,
}

/// Token event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEvent {
    /// Event ID
    pub id: String,
    /// Token symbol
    pub symbol: String,
    /// Event type
    pub event_type: TokenEventType,
    /// Related address
    pub address: String,
    /// Event amount
    pub amount: TokenAmount,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Transaction hash (if available)
    pub transaction_hash: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
} 