//! Ethereum blockchain adapter for AtomSi DAO
//!
//! This module provides functionality for interacting with Ethereum and Ethereum-compatible blockchains.

use async_trait::async_trait;
use ethers::prelude::{
    Address, abi::parse_abi, ContractCall, Http, LocalWallet, Middleware, Provider, 
    SignerMiddleware, TransactionRequest, H160, H256, U256, Wallet, abigen
};
use ethers::utils::hex;
use ethers::signers::{Signer, Signature, LocalAccount};
use std::str::FromStr;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::blockchain::{BlockchainInterface, RawTransaction, TransactionData};

/// Ethereum blockchain adapter for interacting with Ethereum and EVM-compatible chains
pub struct EthereumAdapter {
    provider: Provider<Http>,
}

impl EthereumAdapter {
    /// Create a new Ethereum adapter
    pub fn new(rpc_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| Error::BlockchainError(format!("Failed to connect to Ethereum node: {}", e)))?;
        
        Ok(Self { provider })
    }
    
    /// Create a provider with signer
    fn with_signer(&self, private_key: &str) -> Result<SignerMiddleware<Provider<Http>, LocalWallet>> {
        let wallet = LocalWallet::from_str(private_key)
            .map_err(|e| Error::BlockchainError(format!("Invalid private key: {}", e)))?;
        
        let chain_id = self
            .provider
            .get_chainid()
            .blocking_send()
            .map_err(|e| Error::BlockchainError(format!("Failed to get chain ID: {}", e)))?;
        
        let signer = wallet.with_chain_id(chain_id.as_u64());
        
        Ok(SignerMiddleware::new(self.provider.clone(), signer))
    }
    
    /// Parse an Ethereum address
    fn parse_address(address: &str) -> Result<H160> {
        Address::from_str(address)
            .map_err(|e| Error::BlockchainError(format!("Invalid Ethereum address: {}", e)))
    }
    
    /// Parse a transaction hash
    fn parse_hash(hash: &str) -> Result<H256> {
        H256::from_str(hash)
            .map_err(|e| Error::BlockchainError(format!("Invalid transaction hash: {}", e)))
    }
    
    /// Format a U256 value as a string
    fn format_u256(value: U256) -> String {
        value.to_string()
    }
}

#[async_trait]
impl BlockchainInterface for EthereumAdapter {
    async fn get_balance(&self, address: &str) -> Result<String> {
        let address = Self::parse_address(address)?;
        
        let balance = self
            .provider
            .get_balance(address, None)
            .await
            .map_err(|e| Error::BlockchainError(format!("Failed to get balance: {}", e)))?;
        
        Ok(Self::format_u256(balance))
    }
    
    async fn get_transaction(&self, tx_hash: &str) -> Result<TransactionData> {
        let hash = Self::parse_hash(tx_hash)?;
        
        // Get transaction details
        let tx = self
            .provider
            .get_transaction(hash)
            .await
            .map_err(|e| Error::BlockchainError(format!("Failed to get transaction: {}", e)))?;
        
        let tx = match tx {
            Some(tx) => tx,
            None => {
                return Err(Error::BlockchainError(format!(
                    "Transaction not found: {}",
                    tx_hash
                )))
            }
        };
        
        // Get transaction receipt for additional details
        let receipt = self
            .provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| Error::BlockchainError(format!("Failed to get transaction receipt: {}", e)))?;
        
        let (status, gas_used) = match receipt {
            Some(receipt) => (receipt.status, receipt.gas_used),
            None => (None, None),
        };
        
        Ok(TransactionData {
            hash: format!("{:#x}", hash),
            from: format!("{:#x}", tx.from),
            to: tx.to.map(|addr| format!("{:#x}", addr)),
            value: Self::format_u256(tx.value),
            gas_used: gas_used.map(|g| g.as_u64()),
            gas_price: tx.gas_price.map(Self::format_u256),
            block_number: tx.block_number.map(|b| b.as_u64()),
            status: status.map(|s| s.as_u64() == 1),
            data: tx.input.0.is_empty()
                .then(|| None)
                .unwrap_or_else(|| Some(format!("0x{}", hex::encode(&tx.input.0)))),
        })
    }
    
    async fn send_transaction(&self, transaction: &RawTransaction) -> Result<String> {
        // Need a private key to send transactions
        if !transaction.from.starts_with("0x") {
            // Assume it's a private key
            let signer = self.with_signer(&transaction.from)?;
            
            let to_address = Self::parse_address(&transaction.to)?;
            let value = U256::from_dec_str(&transaction.value)
                .map_err(|e| Error::BlockchainError(format!("Invalid value: {}", e)))?;
            
            let mut tx_request = TransactionRequest::new()
                .to(to_address)
                .value(value);
            
            // Add optional fields
            if let Some(data) = &transaction.data {
                let data = hex::decode(&data.trim_start_matches("0x"))
                    .map_err(|e| Error::BlockchainError(format!("Invalid data: {}", e)))?;
                tx_request = tx_request.data(data);
            }
            
            if let Some(gas_limit) = transaction.gas_limit {
                tx_request = tx_request.gas(gas_limit);
            }
            
            if let Some(gas_price) = &transaction.gas_price {
                let gas_price = U256::from_dec_str(gas_price)
                    .map_err(|e| Error::BlockchainError(format!("Invalid gas price: {}", e)))?;
                tx_request = tx_request.gas_price(gas_price);
            }
            
            if let Some(nonce) = transaction.nonce {
                tx_request = tx_request.nonce(nonce);
            }
            
            // Send the transaction
            let pending_tx = signer
                .send_transaction(tx_request, None)
                .await
                .map_err(|e| Error::BlockchainError(format!("Failed to send transaction: {}", e)))?;
            
            Ok(format!("{:#x}", pending_tx.tx_hash()))
        } else {
            // Cannot send transaction without a private key
            Err(Error::BlockchainError(
                "Cannot send transaction without a private key".to_string(),
            ))
        }
    }
    
    fn sign_message(&self, message: &str, private_key: &str) -> Result<String> {
        let wallet = LocalWallet::from_str(private_key)
            .map_err(|e| Error::BlockchainError(format!("Invalid private key: {}", e)))?;
        
        let signature = wallet
            .sign_message(message)
            .blocking_send()
            .map_err(|e| Error::BlockchainError(format!("Failed to sign message: {}", e)))?;
        
        Ok(signature.to_string())
    }
    
    fn verify_signature(&self, message: &str, signature: &str, address: &str) -> Result<bool> {
        let signature = Signature::from_str(signature)
            .map_err(|e| Error::BlockchainError(format!("Invalid signature: {}", e)))?;
        
        let address = Self::parse_address(address)?;
        
        let recovered = signature
            .recover(message)
            .map_err(|e| Error::BlockchainError(format!("Failed to recover address: {}", e)))?;
        
        Ok(recovered == address)
    }
    
    async fn call_contract(&self, contract_address: &str, method_signature: &str, args: &[String]) -> Result<String> {
        let address = Self::parse_address(contract_address)?;
        
        // Parse the method signature to get the function selector
        let function = parse_abi(&[method_signature])
            .map_err(|e| Error::BlockchainError(format!("Invalid method signature: {}", e)))?
            .functions
            .values()
            .next()
            .ok_or_else(|| Error::BlockchainError("No functions found in ABI".to_string()))?
            .clone();
        
        // Parse the arguments
        let mut encoded_args = Vec::new();
        
        for (i, arg) in args.iter().enumerate() {
            if i >= function.inputs.len() {
                return Err(Error::BlockchainError(format!("Too many arguments provided")));
            }
            
            let param_type = &function.inputs[i].kind;
            
            // This is a simplified version - in a real implementation, you would need to
            // properly encode each argument based on its type
            encoded_args.push(arg.clone());
        }
        
        // Create a call data string (simplified version)
        let data = format!(
            "0x{}{}",
            function.short_signature(),
            encoded_args.join("")
        );
        
        // Call the contract
        let result = self
            .provider
            .call(
                &TransactionRequest::new()
                    .to(address)
                    .data(data),
                None,
            )
            .await
            .map_err(|e| Error::BlockchainError(format!("Contract call failed: {}", e)))?;
        
        Ok(format!("0x{}", hex::encode(result.as_ref())))
    }
    
    async fn execute_contract_transaction(
        &self,
        contract_address: &str,
        method_signature: &str,
        args: &[String],
        private_key: &str,
    ) -> Result<String> {
        let signer = self.with_signer(private_key)?;
        let address = Self::parse_address(contract_address)?;
        
        // Parse the method signature to get the function selector
        let function = parse_abi(&[method_signature])
            .map_err(|e| Error::BlockchainError(format!("Invalid method signature: {}", e)))?
            .functions
            .values()
            .next()
            .ok_or_else(|| Error::BlockchainError("No functions found in ABI".to_string()))?
            .clone();
        
        // Parse the arguments
        let mut encoded_args = Vec::new();
        
        for (i, arg) in args.iter().enumerate() {
            if i >= function.inputs.len() {
                return Err(Error::BlockchainError(format!("Too many arguments provided")));
            }
            
            let param_type = &function.inputs[i].kind;
            
            // This is a simplified version - in a real implementation, you would need to
            // properly encode each argument based on its type
            encoded_args.push(arg.clone());
        }
        
        // Create a call data string (simplified version)
        let data = format!(
            "0x{}{}",
            function.short_signature(),
            encoded_args.join("")
        );
        
        // Send the transaction
        let pending_tx = signer
            .send_transaction(
                TransactionRequest::new()
                    .to(address)
                    .data(data),
                None,
            )
            .await
            .map_err(|e| Error::BlockchainError(format!("Failed to send transaction: {}", e)))?;
        
        Ok(format!("{:#x}", pending_tx.tx_hash()))
    }
    
    async fn get_block_number(&self) -> Result<u64> {
        let block_number = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| Error::BlockchainError(format!("Failed to get block number: {}", e)))?;
        
        Ok(block_number.as_u64())
    }
    
    async fn get_chain_id(&self) -> Result<u64> {
        let chain_id = self
            .provider
            .get_chainid()
            .await
            .map_err(|e| Error::BlockchainError(format!("Failed to get chain ID: {}", e)))?;
        
        Ok(chain_id.as_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::utils::Anvil;
    use std::time::Duration;
    
    async fn setup_test_environment() -> (EthereumAdapter, LocalWallet) {
        // Start an Anvil instance (local Ethereum node for testing)
        let anvil = Anvil::new().spawn();
        
        // Create adapter
        let adapter = EthereumAdapter::new(&anvil.endpoint()).unwrap();
        
        // Create wallet from one of the test accounts
        let wallet = LocalWallet::from_str(&anvil.keys()[0]).unwrap();
        
        (adapter, wallet)
    }
    
    #[tokio::test]
    async fn test_get_balance() {
        let (adapter, wallet) = setup_test_environment().await;
        let address = format!("{:#x}", wallet.address());
        
        let balance = adapter.get_balance(&address).await.unwrap();
        
        // Anvil gives 10000 ETH to test accounts by default
        assert_eq!(balance, "10000000000000000000000");
    }
    
    #[tokio::test]
    async fn test_sign_and_verify_message() {
        let (adapter, wallet) = setup_test_environment().await;
        let private_key = wallet.signer().to_bytes().to_vec();
        let private_key_hex = format!("0x{}", hex::encode(private_key));
        let address = format!("{:#x}", wallet.address());
        
        let message = "Hello, AtomSi DAO!";
        
        // Sign the message
        let signature = adapter.sign_message(message, &private_key_hex).unwrap();
        
        // Verify the signature
        let is_valid = adapter.verify_signature(message, &signature, &address).unwrap();
        
        assert!(is_valid);
        
        // Test invalid signature
        let wrong_address = "0x1234567890123456789012345678901234567890";
        let is_valid = adapter.verify_signature(message, &signature, wrong_address).unwrap();
        
        assert!(!is_valid);
    }
    
    #[tokio::test]
    async fn test_get_chain_id() {
        let (adapter, _) = setup_test_environment().await;
        
        let chain_id = adapter.get_chain_id().await.unwrap();
        
        // Anvil uses chain ID 31337 by default
        assert_eq!(chain_id, 31337);
    }
    
    #[tokio::test]
    async fn test_get_block_number() {
        let (adapter, _) = setup_test_environment().await;
        
        let block_number = adapter.get_block_number().await.unwrap();
        
        // Should be at least 0
        assert!(block_number >= 0);
    }
} 