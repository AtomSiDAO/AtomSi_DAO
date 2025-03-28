//! Token module for AtomSi DAO
//!
//! This module provides functionality for managing DAO tokens,
//! including token creation, transfers, and staking.

mod types;

pub use types::{Token, TokenAmount, TokenId};

use crate::{
    blockchain::BlockchainAdapter,
    config::Config,
    core::{Database, DaoError, Result},
};
use std::sync::Arc;

/// Manager for token operations
#[derive(Clone)]
pub struct TokenManager {
    /// Configuration
    config: Arc<Config>,
    /// Blockchain adapter
    blockchain: Arc<dyn BlockchainAdapter>,
    /// Database
    database: Database,
}

impl TokenManager {
    /// Create a new token manager
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
    
    /// Create a new token
    pub async fn create_token(&self, name: &str, symbol: &str, initial_supply: u64) -> Result<Token> {
        // Check if the token already exists
        if self.token_exists(symbol).await? {
            return Err(DaoError::InvalidParameter(format!(
                "Token with symbol {} already exists",
                symbol
            )));
        }
        
        // Create a new token
        let token = Token {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            total_supply: initial_supply,
            decimals: 18,
            created_at: chrono::Utc::now(),
        };
        
        // Save the token to the database
        self.save_token(&token).await?;
        
        Ok(token)
    }
    
    /// Get a token by symbol
    pub async fn get_token(&self, symbol: &str) -> Result<Token> {
        // Load the token from the database
        let query = "SELECT * FROM tokens WHERE symbol = $1";
        let row = self
            .database
            .query_one(query, &[&symbol])
            .await
            .map_err(|e| match e {
                DaoError::DatabaseError(msg) if msg.contains("no rows in result set") => {
                    DaoError::InvalidParameter(format!("Token with symbol {} not found", symbol))
                }
                _ => e,
            })?;
        
        // Parse the token from the row
        let token: Token = serde_json::from_value(row.get("data"))
            .map_err(|e| DaoError::DatabaseError(format!("Failed to parse token: {}", e)))?;
        
        Ok(token)
    }
    
    /// Check if a token exists
    pub async fn token_exists(&self, symbol: &str) -> Result<bool> {
        // Query the database for the token
        let query = "SELECT 1 FROM tokens WHERE symbol = $1";
        let result = self.database.query_opt(query, &[&symbol]).await?;
        
        Ok(result.is_some())
    }
    
    /// Get the balance of a token for an address
    pub async fn get_balance(&self, symbol: &str, address: &str) -> Result<TokenAmount> {
        // If the symbol matches the governance token, get the balance from the blockchain
        if symbol == self.config.dao.governance_token {
            let balance = self
                .blockchain
                .balance(address)
                .await
                .map_err(|e| DaoError::BlockchainError(e))?;
            
            return Ok(balance);
        }
        
        // Otherwise, query the database
        let query = "SELECT balance FROM token_balances WHERE symbol = $1 AND address = $2";
        let result = self.database.query_opt(query, &[&symbol, &address]).await?;
        
        let balance = match result {
            Some(row) => row.get::<_, i64>("balance") as u64,
            None => 0,
        };
        
        Ok(balance)
    }
    
    /// Transfer tokens from one address to another
    pub async fn transfer(
        &self,
        symbol: &str,
        from: &str,
        to: &str,
        amount: TokenAmount,
    ) -> Result<()> {
        // Check if the token exists
        if !self.token_exists(symbol).await? {
            return Err(DaoError::InvalidParameter(format!(
                "Token with symbol {} not found",
                symbol
            )));
        }
        
        // Check if the sender has enough balance
        let sender_balance = self.get_balance(symbol, from).await?;
        if sender_balance < amount {
            return Err(DaoError::InvalidParameter(
                "Insufficient balance".to_string(),
            ));
        }
        
        // If the symbol matches the governance token, send a transaction on the blockchain
        if symbol == self.config.dao.governance_token {
            self.blockchain
                .send_transaction(to, amount)
                .await
                .map_err(|e| DaoError::BlockchainError(e))?;
            
            return Ok(());
        }
        
        // Otherwise, update the balances in the database
        self.database
            .transaction(|tx| {
                Box::pin(async move {
                    // Deduct from sender
                    tx.execute(
                        "UPDATE token_balances SET balance = balance - $1 WHERE symbol = $2 AND address = $3",
                        &[&(amount as i64), &symbol, &from],
                    )
                    .await
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to update balance: {}", e)))?;
                    
                    // Add to recipient
                    let updated = tx
                        .execute(
                            "UPDATE token_balances SET balance = balance + $1 WHERE symbol = $2 AND address = $3",
                            &[&(amount as i64), &symbol, &to],
                        )
                        .await
                        .map_err(|e| DaoError::DatabaseError(format!("Failed to update balance: {}", e)))?;
                    
                    // If recipient doesn't have a balance entry yet, create one
                    if updated == 0 {
                        tx.execute(
                            "INSERT INTO token_balances (symbol, address, balance) VALUES ($1, $2, $3)",
                            &[&symbol, &to, &(amount as i64)],
                        )
                        .await
                        .map_err(|e| DaoError::DatabaseError(format!("Failed to insert balance: {}", e)))?;
                    }
                    
                    // Record the transfer
                    tx.execute(
                        "INSERT INTO token_transfers (symbol, from_address, to_address, amount, timestamp) VALUES ($1, $2, $3, $4, $5)",
                        &[&symbol, &from, &to, &(amount as i64), &chrono::Utc::now()],
                    )
                    .await
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to record transfer: {}", e)))?;
                    
                    Ok(())
                })
            })
            .await?;
        
        Ok(())
    }
    
    /// Mint new tokens
    pub async fn mint(
        &self,
        symbol: &str,
        to: &str,
        amount: TokenAmount,
    ) -> Result<()> {
        // Check if the token exists
        let mut token = self.get_token(symbol).await?;
        
        // Update the total supply
        token.total_supply += amount;
        
        // Save the updated token
        self.save_token(&token).await?;
        
        // Add to recipient balance
        let updated = self
            .database
            .execute(
                "UPDATE token_balances SET balance = balance + $1 WHERE symbol = $2 AND address = $3",
                &[&(amount as i64), &symbol, &to],
            )
            .await?;
        
        // If recipient doesn't have a balance entry yet, create one
        if updated == 0 {
            self.database
                .execute(
                    "INSERT INTO token_balances (symbol, address, balance) VALUES ($1, $2, $3)",
                    &[&symbol, &to, &(amount as i64)],
                )
                .await?;
        }
        
        // Record the mint
        self.database
            .execute(
                "INSERT INTO token_events (symbol, event_type, address, amount, timestamp) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &symbol,
                    &"mint",
                    &to,
                    &(amount as i64),
                    &chrono::Utc::now(),
                ],
            )
            .await?;
        
        Ok(())
    }
    
    /// Burn tokens
    pub async fn burn(
        &self,
        symbol: &str,
        from: &str,
        amount: TokenAmount,
    ) -> Result<()> {
        // Check if the token exists
        let mut token = self.get_token(symbol).await?;
        
        // Check if the sender has enough balance
        let sender_balance = self.get_balance(symbol, from).await?;
        if sender_balance < amount {
            return Err(DaoError::InvalidParameter(
                "Insufficient balance".to_string(),
            ));
        }
        
        // Update the total supply
        token.total_supply -= amount;
        
        // Save the updated token
        self.save_token(&token).await?;
        
        // Deduct from sender balance
        self.database
            .execute(
                "UPDATE token_balances SET balance = balance - $1 WHERE symbol = $2 AND address = $3",
                &[&(amount as i64), &symbol, &from],
            )
            .await?;
        
        // Record the burn
        self.database
            .execute(
                "INSERT INTO token_events (symbol, event_type, address, amount, timestamp) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &symbol,
                    &"burn",
                    &from,
                    &(amount as i64),
                    &chrono::Utc::now(),
                ],
            )
            .await?;
        
        Ok(())
    }
    
    // Private methods
    
    /// Save a token to the database
    async fn save_token(&self, token: &Token) -> Result<()> {
        // Serialize the token
        let data = serde_json::to_value(token)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize token: {}", e)))?;
        
        // Check if the token already exists
        let exists = self
            .database
            .query_opt(
                "SELECT 1 FROM tokens WHERE symbol = $1",
                &[&token.symbol],
            )
            .await?
            .is_some();
        
        if exists {
            // Update the token
            self.database
                .execute(
                    "UPDATE tokens SET data = $1 WHERE symbol = $2",
                    &[&data, &token.symbol],
                )
                .await?;
        } else {
            // Insert the token
            self.database
                .execute(
                    "INSERT INTO tokens (id, symbol, name, data, created_at) VALUES ($1, $2, $3, $4, $5)",
                    &[
                        &token.id,
                        &token.symbol,
                        &token.name,
                        &data,
                        &token.created_at,
                    ],
                )
                .await?;
        }
        
        Ok(())
    }
} 