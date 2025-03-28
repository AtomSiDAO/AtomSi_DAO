//! Treasury module for AtomSi DAO
//!
//! This module provides functionality for managing DAO funds,
//! including multi-signature control, spending limits, and asset tracking.

use crate::{
    blockchain::BlockchainAdapter,
    config::Config,
    core::{Database, DaoError, Result},
    token::{TokenAmount, TokenManager},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Treasury transaction ID type
pub type TransactionId = String;

/// Treasury transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Pending approval
    Pending,
    /// Approved but not executed
    Approved,
    /// Executed
    Executed,
    /// Rejected
    Rejected,
    /// Failed
    Failed,
}

/// Treasury transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID
    pub id: TransactionId,
    /// Transaction description
    pub description: String,
    /// Recipient address
    pub to: String,
    /// Token symbol
    pub token: String,
    /// Amount
    pub amount: TokenAmount,
    /// Transaction status
    pub status: TransactionStatus,
    /// Required number of approvals
    pub required_approvals: u32,
    /// Current number of approvals
    pub current_approvals: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Execution timestamp
    pub executed_at: Option<DateTime<Utc>>,
    /// Signers who have approved
    pub approvers: Vec<String>,
    /// Blockchain transaction hash (if available)
    pub transaction_hash: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Transaction builder
pub struct TransactionBuilder {
    description: Option<String>,
    to: Option<String>,
    token: Option<String>,
    amount: Option<TokenAmount>,
    required_approvals: Option<u32>,
    metadata: serde_json::Value,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new() -> Self {
        Self {
            description: None,
            to: None,
            token: None,
            amount: None,
            required_approvals: None,
            metadata: serde_json::Value::Null,
        }
    }
    
    /// Set the transaction description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Set the recipient address
    pub fn to<S: Into<String>>(mut self, to: S) -> Self {
        self.to = Some(to.into());
        self
    }
    
    /// Set the token symbol
    pub fn token<S: Into<String>>(mut self, token: S) -> Self {
        self.token = Some(token.into());
        self
    }
    
    /// Set the transaction amount
    pub fn amount(mut self, amount: TokenAmount) -> Self {
        self.amount = Some(amount);
        self
    }
    
    /// Set the required number of approvals
    pub fn required_approvals(mut self, required_approvals: u32) -> Self {
        self.required_approvals = Some(required_approvals);
        self
    }
    
    /// Set additional metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Build the transaction
    pub fn build(self) -> Result<Transaction> {
        let description = self.description.ok_or_else(|| {
            DaoError::InvalidParameter("Transaction description is required".to_string())
        })?;
        
        let to = self.to.ok_or_else(|| {
            DaoError::InvalidParameter("Recipient address is required".to_string())
        })?;
        
        let token = self.token.ok_or_else(|| {
            DaoError::InvalidParameter("Token symbol is required".to_string())
        })?;
        
        let amount = self.amount.ok_or_else(|| {
            DaoError::InvalidParameter("Transaction amount is required".to_string())
        })?;
        
        let required_approvals = self.required_approvals.unwrap_or(1);
        
        if required_approvals == 0 {
            return Err(DaoError::InvalidParameter(
                "Required approvals must be greater than 0".to_string(),
            ));
        }
        
        let now = Utc::now();
        
        Ok(Transaction {
            id: Uuid::new_v4().to_string(),
            description,
            to,
            token,
            amount,
            status: TransactionStatus::Pending,
            required_approvals,
            current_approvals: 0,
            created_at: now,
            updated_at: now,
            executed_at: None,
            approvers: Vec::new(),
            transaction_hash: None,
            metadata: self.metadata,
        })
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Treasury manager
pub struct TreasuryManager {
    /// Configuration
    config: Arc<Config>,
    /// Blockchain adapter
    blockchain: Arc<dyn BlockchainAdapter>,
    /// Database
    database: Database,
    /// Token manager reference
    token_manager: Arc<TokenManager>,
}

impl TreasuryManager {
    /// Create a new treasury manager
    pub fn new(
        config: &Config,
        blockchain: impl BlockchainAdapter + 'static,
        database: Database,
    ) -> Result<Self> {
        let token_manager = TokenManager::new(config, blockchain.clone(), database.clone())?;
        
        Ok(Self {
            config: Arc::new(config.clone()),
            blockchain: Arc::new(blockchain),
            database,
            token_manager: Arc::new(token_manager),
        })
    }
    
    /// Create a new transaction
    pub async fn create_transaction(&self, transaction: Transaction) -> Result<TransactionId> {
        // Check if the token exists
        if !self.token_manager.token_exists(&transaction.token).await? {
            return Err(DaoError::InvalidParameter(format!(
                "Token with symbol {} not found",
                transaction.token
            )));
        }
        
        // Check if the treasury has enough balance
        let treasury_address = self.get_treasury_address().await?;
        let treasury_balance = self
            .token_manager
            .get_balance(&transaction.token, &treasury_address)
            .await?;
        
        if treasury_balance < transaction.amount {
            return Err(DaoError::InvalidParameter(
                "Insufficient treasury balance".to_string(),
            ));
        }
        
        // Save the transaction to the database
        self.save_transaction(&transaction).await?;
        
        Ok(transaction.id)
    }
    
    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: &TransactionId) -> Result<Transaction> {
        // Load the transaction from the database
        let query = "SELECT * FROM treasury_transactions WHERE id = $1";
        let row = self.database.query_one(query, &[&id]).await?;
        
        // Parse the transaction from the row
        let transaction: Transaction = serde_json::from_value(row.get("data"))
            .map_err(|e| DaoError::DatabaseError(format!("Failed to parse transaction: {}", e)))?;
        
        Ok(transaction)
    }
    
    /// Get all transactions
    pub async fn get_transactions(
        &self,
        status: Option<TransactionStatus>,
    ) -> Result<Vec<Transaction>> {
        // Construct the query based on the status filter
        let (query, params) = match status {
            Some(status) => {
                let query = "SELECT * FROM treasury_transactions WHERE status = $1 ORDER BY created_at DESC";
                let status_str = serde_json::to_string(&status)
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize status: {}", e)))?;
                (query, vec![&status_str as &(dyn tokio_postgres::types::ToSql + Sync)])
            }
            None => {
                let query = "SELECT * FROM treasury_transactions ORDER BY created_at DESC";
                (query, Vec::new())
            }
        };
        
        // Load the transactions from the database
        let rows = self.database.query(query, &params).await?;
        
        // Parse the transactions from the rows
        let transactions = rows
            .into_iter()
            .map(|row| {
                serde_json::from_value(row.get("data"))
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to parse transaction: {}", e)))
            })
            .collect::<Result<Vec<Transaction>>>()?;
        
        Ok(transactions)
    }
    
    /// Approve a transaction
    pub async fn approve_transaction(
        &self,
        id: &TransactionId,
        approver: &str,
    ) -> Result<()> {
        // Load the transaction
        let mut transaction = self.get_transaction(id).await?;
        
        // Check if the transaction is in a pending state
        if transaction.status != TransactionStatus::Pending {
            return Err(DaoError::InvalidParameter(
                "Transaction is not in a pending state".to_string(),
            ));
        }
        
        // Check if the approver is a valid signer
        let signers = self.get_signers().await?;
        if !signers.contains(&approver.to_string()) {
            return Err(DaoError::Unauthorized);
        }
        
        // Check if the approver has already approved
        if transaction.approvers.contains(&approver.to_string()) {
            return Err(DaoError::InvalidParameter(
                "Approver has already approved this transaction".to_string(),
            ));
        }
        
        // Add the approver
        transaction.approvers.push(approver.to_string());
        transaction.current_approvals += 1;
        transaction.updated_at = Utc::now();
        
        // Check if the transaction has enough approvals
        if transaction.current_approvals >= transaction.required_approvals {
            transaction.status = TransactionStatus::Approved;
        }
        
        // Save the updated transaction
        self.save_transaction(&transaction).await?;
        
        // If the transaction is now approved, try to execute it
        if transaction.status == TransactionStatus::Approved {
            self.execute_transaction(id).await?;
        }
        
        Ok(())
    }
    
    /// Reject a transaction
    pub async fn reject_transaction(&self, id: &TransactionId, rejector: &str) -> Result<()> {
        // Load the transaction
        let mut transaction = self.get_transaction(id).await?;
        
        // Check if the transaction is in a pending state
        if transaction.status != TransactionStatus::Pending {
            return Err(DaoError::InvalidParameter(
                "Transaction is not in a pending state".to_string(),
            ));
        }
        
        // Check if the rejector is a valid signer
        let signers = self.get_signers().await?;
        if !signers.contains(&rejector.to_string()) {
            return Err(DaoError::Unauthorized);
        }
        
        // Update the transaction status
        transaction.status = TransactionStatus::Rejected;
        transaction.updated_at = Utc::now();
        
        // Save the updated transaction
        self.save_transaction(&transaction).await?;
        
        Ok(())
    }
    
    /// Execute a transaction
    pub async fn execute_transaction(&self, id: &TransactionId) -> Result<()> {
        // Load the transaction
        let mut transaction = self.get_transaction(id).await?;
        
        // Check if the transaction is approved
        if transaction.status != TransactionStatus::Approved {
            return Err(DaoError::InvalidParameter(
                "Transaction is not in an approved state".to_string(),
            ));
        }
        
        // Get the treasury address
        let treasury_address = self.get_treasury_address().await?;
        
        // Execute the transfer
        let result = self
            .token_manager
            .transfer(
                &transaction.token,
                &treasury_address,
                &transaction.to,
                transaction.amount,
            )
            .await;
        
        match result {
            Ok(_) => {
                // Update the transaction status
                transaction.status = TransactionStatus::Executed;
                transaction.executed_at = Some(Utc::now());
                transaction.updated_at = Utc::now();
                
                // Save the updated transaction
                self.save_transaction(&transaction).await?;
                
                Ok(())
            }
            Err(e) => {
                // Update the transaction status
                transaction.status = TransactionStatus::Failed;
                transaction.updated_at = Utc::now();
                transaction.metadata = serde_json::json!({
                    "error": e.to_string(),
                });
                
                // Save the updated transaction
                self.save_transaction(&transaction).await?;
                
                Err(e)
            }
        }
    }
    
    /// Get the treasury balance
    pub async fn get_balance(&self, token: &str) -> Result<TokenAmount> {
        let treasury_address = self.get_treasury_address().await?;
        self.token_manager
            .get_balance(token, &treasury_address)
            .await
    }
    
    /// Get all treasury balances
    pub async fn get_balances(&self) -> Result<HashMap<String, TokenAmount>> {
        let treasury_address = self.get_treasury_address().await?;
        
        // Get all tokens
        let query = "SELECT symbol FROM tokens";
        let rows = self.database.query(query, &[]).await?;
        
        let mut balances = HashMap::new();
        
        // Get balance for each token
        for row in rows {
            let symbol: String = row.get("symbol");
            let balance = self
                .token_manager
                .get_balance(&symbol, &treasury_address)
                .await?;
            
            balances.insert(symbol, balance);
        }
        
        Ok(balances)
    }
    
    // Private methods
    
    /// Get the treasury address
    async fn get_treasury_address(&self) -> Result<String> {
        // In a real implementation, this would be a multi-sig wallet address
        // controlled by the DAO signers
        
        // For this example, we'll just return a placeholder address
        // This would be derived from the governance parameters or stored in the database
        Ok("0xTreasury".to_string())
    }
    
    /// Get authorized signers
    async fn get_signers(&self) -> Result<Vec<String>> {
        // In a real implementation, these would be loaded from a governance contract
        // or from the database
        
        // For this example, we'll just return a placeholder list
        // This would be derived from the governance parameters
        let signers = (0..self.config.treasury.signers)
            .map(|i| format!("0xSigner{}", i))
            .collect();
        
        Ok(signers)
    }
    
    /// Save a transaction to the database
    async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        // Serialize the transaction
        let data = serde_json::to_value(transaction)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize transaction: {}", e)))?;
        
        // Check if the transaction already exists
        let exists = self
            .database
            .query_opt(
                "SELECT 1 FROM treasury_transactions WHERE id = $1",
                &[&transaction.id],
            )
            .await?
            .is_some();
        
        if exists {
            // Update the transaction
            self.database
                .execute(
                    "UPDATE treasury_transactions SET data = $1, status = $2, updated_at = $3 WHERE id = $4",
                    &[
                        &data,
                        &serde_json::to_string(&transaction.status).unwrap(),
                        &transaction.updated_at,
                        &transaction.id,
                    ],
                )
                .await?;
        } else {
            // Insert the transaction
            self.database
                .execute(
                    "INSERT INTO treasury_transactions (id, data, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
                    &[
                        &transaction.id,
                        &data,
                        &serde_json::to_string(&transaction.status).unwrap(),
                        &transaction.created_at,
                        &transaction.updated_at,
                    ],
                )
                .await?;
        }
        
        Ok(())
    }
} 