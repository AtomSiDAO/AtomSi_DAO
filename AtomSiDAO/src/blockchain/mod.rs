//! Blockchain module for AtomSi DAO
//!
//! This module provides adapters for interacting with various blockchain networks.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::prelude::{Address, TransactionReceipt, H256, U256};
use serde::{Deserialize, Serialize};

use crate::config::BlockchainConfig;
use crate::crypto::CryptoError;
use crate::error::{Error, Result};

pub mod ethereum;
pub mod providers;

/// Transaction data for blockchain transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    /// Transaction hash
    pub hash: String,
    
    /// From address
    pub from: String,
    
    /// To address
    pub to: Option<String>,
    
    /// Value sent in the transaction (in wei)
    pub value: String,
    
    /// Gas used by the transaction
    pub gas_used: Option<u64>,
    
    /// Gas price (in wei)
    pub gas_price: Option<String>,
    
    /// Block number where the transaction was included
    pub block_number: Option<u64>,
    
    /// Transaction status (true for success, false for failure)
    pub status: Option<bool>,
    
    /// Custom transaction data
    pub data: Option<String>,
}

/// Interface for blockchain adapters
#[async_trait]
pub trait BlockchainInterface: Send + Sync {
    /// Get the balance of an address
    async fn get_balance(&self, address: &str) -> Result<String>;
    
    /// Get transaction details
    async fn get_transaction(&self, tx_hash: &str) -> Result<TransactionData>;
    
    /// Send a transaction
    async fn send_transaction(&self, transaction: &RawTransaction) -> Result<String>;
    
    /// Sign a message with a private key
    fn sign_message(&self, message: &str, private_key: &str) -> Result<String>;
    
    /// Verify a signature
    fn verify_signature(&self, message: &str, signature: &str, address: &str) -> Result<bool>;
    
    /// Call a contract method without sending a transaction
    async fn call_contract(&self, contract_address: &str, method_signature: &str, args: &[String]) -> Result<String>;
    
    /// Execute a contract transaction
    async fn execute_contract_transaction(
        &self,
        contract_address: &str,
        method_signature: &str,
        args: &[String],
        private_key: &str,
    ) -> Result<String>;
    
    /// Get the current block number
    async fn get_block_number(&self) -> Result<u64>;
    
    /// Get the chain ID
    async fn get_chain_id(&self) -> Result<u64>;
}

/// Raw transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    /// From address
    pub from: String,
    
    /// To address
    pub to: String,
    
    /// Value to send (in wei)
    pub value: String,
    
    /// Transaction data
    pub data: Option<String>,
    
    /// Gas limit
    pub gas_limit: Option<u64>,
    
    /// Gas price (in wei)
    pub gas_price: Option<String>,
    
    /// Nonce
    pub nonce: Option<u64>,
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash
    pub transaction_hash: String,
    
    /// Block number
    pub block_number: u64,
    
    /// Gas used
    pub gas_used: u64,
    
    /// Status (1 for success, 0 for failure)
    pub status: u8,
    
    /// Logs
    pub logs: Vec<Log>,
}

/// Log entry from a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    /// Contract address
    pub address: String,
    
    /// Topics (indexed fields)
    pub topics: Vec<String>,
    
    /// Data (non-indexed fields)
    pub data: String,
}

/// Blockchain adapter for connecting to different chains
#[derive(Clone)]
pub struct BlockchainAdapter {
    config: BlockchainConfig,
    adapters: HashMap<u64, Arc<dyn BlockchainInterface>>,
    default_chain_id: u64,
}

impl BlockchainAdapter {
    /// Create a new blockchain adapter
    pub fn new(config: &BlockchainConfig) -> Result<Self> {
        let mut adapters = HashMap::new();
        
        // Create adapter for the main chain
        let adapter = ethereum::EthereumAdapter::new(&config.rpc_url)?;
        adapters.insert(config.chain_id, Arc::new(adapter));
        
        // Create adapters for supported chains
        for (_, chain_config) in &config.supported_chains {
            if !adapters.contains_key(&chain_config.chain_id) {
                let adapter = ethereum::EthereumAdapter::new(&chain_config.rpc_url)?;
                adapters.insert(chain_config.chain_id, Arc::new(adapter));
            }
        }
        
        Ok(Self {
            config: config.clone(),
            adapters,
            default_chain_id: config.chain_id,
        })
    }
    
    /// Get adapter for a specific chain
    pub fn get_adapter(&self, chain_id: u64) -> Result<Arc<dyn BlockchainInterface>> {
        self.adapters
            .get(&chain_id)
            .cloned()
            .ok_or_else(|| Error::BlockchainError(format!("Adapter not found for chain ID: {}", chain_id)))
    }
    
    /// Get the default adapter
    pub fn get_default_adapter(&self) -> Result<Arc<dyn BlockchainInterface>> {
        self.get_adapter(self.default_chain_id)
    }
    
    /// Get the balance of an address on a specific chain
    pub async fn get_balance(&self, address: &str, chain_id: Option<u64>) -> Result<String> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.get_balance(address).await
    }
    
    /// Get transaction details from a specific chain
    pub async fn get_transaction(&self, tx_hash: &str, chain_id: Option<u64>) -> Result<TransactionData> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.get_transaction(tx_hash).await
    }
    
    /// Send a transaction to a specific chain
    pub async fn send_transaction(&self, transaction: &RawTransaction, chain_id: Option<u64>) -> Result<String> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.send_transaction(transaction).await
    }
    
    /// Sign a message with a private key
    pub fn sign_message(&self, message: &str, private_key: &str, chain_id: Option<u64>) -> Result<String> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.sign_message(message, private_key)
    }
    
    /// Verify a signature
    pub fn verify_signature(&self, message: &str, signature: &str, address: &str, chain_id: Option<u64>) -> Result<bool> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.verify_signature(message, signature, address)
    }
    
    /// Call a contract method without sending a transaction
    pub async fn call_contract(
        &self,
        contract_address: &str,
        method_signature: &str,
        args: &[String],
        chain_id: Option<u64>,
    ) -> Result<String> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.call_contract(contract_address, method_signature, args).await
    }
    
    /// Execute a contract transaction
    pub async fn execute_contract_transaction(
        &self,
        contract_address: &str,
        method_signature: &str,
        args: &[String],
        private_key: &str,
        chain_id: Option<u64>,
    ) -> Result<String> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter
            .execute_contract_transaction(contract_address, method_signature, args, private_key)
            .await
    }
    
    /// Get the current block number from a specific chain
    pub async fn get_block_number(&self, chain_id: Option<u64>) -> Result<u64> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.get_block_number().await
    }
    
    /// Get the chain ID from a specific adapter
    pub async fn get_chain_id(&self, chain_id: Option<u64>) -> Result<u64> {
        let chain_id = chain_id.unwrap_or(self.default_chain_id);
        let adapter = self.get_adapter(chain_id)?;
        adapter.get_chain_id().await
    }
} 