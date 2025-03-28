//! Voting strategies for governance
//!
//! This module provides different voting strategy implementations
//! for calculating vote weight based on token holdings.

use crate::core::{DaoError, Result};
use async_trait::async_trait;
use std::collections::HashMap;

/// Vote weight structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteWeight {
    /// Numerical weight value
    pub value: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Voting strategy trait
pub trait VotingStrategy: Send + Sync {
    /// Calculate the voting weight for an address
    fn calculate_weight(&self, address: &str, balance: u64) -> Result<VoteWeight>;
    
    /// Get the strategy name
    fn name(&self) -> &str;
    
    /// Get the strategy description
    fn description(&self) -> &str;
}

/// Token-weighted voting strategy
///
/// In this strategy, voting power is directly proportional to token holdings.
pub struct TokenWeightedVoting;

impl TokenWeightedVoting {
    /// Create a new token-weighted voting strategy
    pub fn new() -> Self {
        Self
    }
}

impl VotingStrategy for TokenWeightedVoting {
    fn calculate_weight(&self, _address: &str, balance: u64) -> Result<VoteWeight> {
        Ok(VoteWeight {
            value: balance,
            metadata: HashMap::new(),
        })
    }
    
    fn name(&self) -> &str {
        "Token Weighted Voting"
    }
    
    fn description(&self) -> &str {
        "Voting power is directly proportional to token holdings"
    }
}

/// Quadratic voting strategy
///
/// In this strategy, voting power is proportional to the square root of token holdings.
pub struct QuadraticVoting;

impl QuadraticVoting {
    /// Create a new quadratic voting strategy
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate the square root of a number, rounded down
    fn sqrt(x: u64) -> u64 {
        (x as f64).sqrt() as u64
    }
}

impl VotingStrategy for QuadraticVoting {
    fn calculate_weight(&self, _address: &str, balance: u64) -> Result<VoteWeight> {
        // Voting power = square root of token balance
        let weight = Self::sqrt(balance);
        
        let mut metadata = HashMap::new();
        metadata.insert("formula".to_string(), "sqrt(balance)".to_string());
        metadata.insert("original_balance".to_string(), balance.to_string());
        
        Ok(VoteWeight {
            value: weight,
            metadata,
        })
    }
    
    fn name(&self) -> &str {
        "Quadratic Voting"
    }
    
    fn description(&self) -> &str {
        "Voting power is proportional to the square root of token holdings"
    }
}

/// Conviction voting strategy
///
/// In this strategy, voting power increases over time as votes are held.
pub struct ConvictionVoting {
    /// Maximum conviction
    max_conviction: u64,
    /// Conviction increase per block
    conviction_per_block: u64,
}

impl ConvictionVoting {
    /// Create a new conviction voting strategy
    pub fn new() -> Self {
        Self {
            max_conviction: 10,
            conviction_per_block: 1,
        }
    }
    
    /// Create a new conviction voting strategy with custom parameters
    pub fn with_params(max_conviction: u64, conviction_per_block: u64) -> Self {
        Self {
            max_conviction,
            conviction_per_block,
        }
    }
}

impl VotingStrategy for ConvictionVoting {
    fn calculate_weight(&self, _address: &str, balance: u64) -> Result<VoteWeight> {
        // For the sake of this example, we'll just use the token balance
        // In a real implementation, we would track voting history and calculate
        // conviction based on how long tokens have been staked for voting
        
        // Normally this would depend on time, but for simplicity we'll just
        // use a fixed multiplier for this example
        let conviction = 5.min(self.max_conviction);
        let weight = balance * conviction;
        
        let mut metadata = HashMap::new();
        metadata.insert("conviction".to_string(), conviction.to_string());
        metadata.insert("formula".to_string(), "balance * conviction".to_string());
        metadata.insert("original_balance".to_string(), balance.to_string());
        
        Ok(VoteWeight {
            value: weight,
            metadata,
        })
    }
    
    fn name(&self) -> &str {
        "Conviction Voting"
    }
    
    fn description(&self) -> &str {
        "Voting power increases over time as votes are held"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_weighted_voting() {
        let strategy = TokenWeightedVoting::new();
        
        // Test with zero balance
        let weight = strategy.calculate_weight("0x1", 0).unwrap();
        assert_eq!(weight.value, 0);
        
        // Test with non-zero balance
        let weight = strategy.calculate_weight("0x1", 100).unwrap();
        assert_eq!(weight.value, 100);
    }
    
    #[test]
    fn test_quadratic_voting() {
        let strategy = QuadraticVoting::new();
        
        // Test with zero balance
        let weight = strategy.calculate_weight("0x1", 0).unwrap();
        assert_eq!(weight.value, 0);
        
        // Test with non-zero balance
        let weight = strategy.calculate_weight("0x1", 100).unwrap();
        assert_eq!(weight.value, 10); // sqrt(100) = 10
        
        // Test with large balance
        let weight = strategy.calculate_weight("0x1", 10000).unwrap();
        assert_eq!(weight.value, 100); // sqrt(10000) = 100
    }
    
    #[test]
    fn test_conviction_voting() {
        let strategy = ConvictionVoting::new();
        
        // Test with zero balance
        let weight = strategy.calculate_weight("0x1", 0).unwrap();
        assert_eq!(weight.value, 0);
        
        // Test with non-zero balance
        let weight = strategy.calculate_weight("0x1", 100).unwrap();
        assert_eq!(weight.value, 500); // 100 * 5 = 500
        
        // Test with custom parameters
        let strategy = ConvictionVoting::with_params(3, 2);
        let weight = strategy.calculate_weight("0x1", 100).unwrap();
        assert_eq!(weight.value, 300); // 100 * 3 = 300
    }
} 