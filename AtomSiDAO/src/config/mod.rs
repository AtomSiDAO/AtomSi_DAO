//! Configuration module for AtomSi DAO
//!
//! This module provides configuration management for the DAO.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};

/// Configuration settings for the AtomSi DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General DAO settings
    pub dao: DaoConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Blockchain configuration
    pub blockchain: BlockchainConfig,
    
    /// API server configuration
    pub api: ApiConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Custom configuration values
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Configuration for the DAO itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoConfig {
    /// The name of the DAO
    pub name: String,
    
    /// Description of the DAO
    pub description: String,
    
    /// DAO version
    pub version: String,
    
    /// DAO website URL
    pub website_url: Option<String>,
    
    /// Default chain ID for this DAO
    pub default_chain_id: u64,
    
    /// Default token symbol used in the DAO
    pub default_token_symbol: String,
    
    /// Address of the token contract
    pub token_contract_address: String,
    
    /// List of admin addresses
    pub admin_addresses: Vec<String>,
}

/// Configuration for database connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type (postgres, mysql, sqlite)
    pub db_type: String,
    
    /// Database host
    pub host: String,
    
    /// Database port
    pub port: u16,
    
    /// Database name
    pub name: String,
    
    /// Database username
    pub username: String,
    
    /// Database password
    pub password: String,
    
    /// Connection pool size
    pub pool_size: u32,
    
    /// SQLite file path (only used when db_type is sqlite)
    pub sqlite_path: Option<String>,
}

/// Configuration for blockchain connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// Default RPC URL
    pub rpc_url: String,
    
    /// Chain ID
    pub chain_id: u64,
    
    /// Explorer URL
    pub explorer_url: String,
    
    /// Gas price strategy (static, standard, fast)
    pub gas_price_strategy: String,
    
    /// Static gas price in gwei (used when gas_price_strategy is static)
    pub static_gas_price_gwei: Option<u64>,
    
    /// List of supported chain configurations
    pub supported_chains: HashMap<String, ChainConfig>,
}

/// Configuration for a specific blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain name
    pub name: String,
    
    /// Chain ID
    pub chain_id: u64,
    
    /// RPC URL
    pub rpc_url: String,
    
    /// Explorer URL
    pub explorer_url: String,
    
    /// Native currency symbol
    pub currency_symbol: String,
    
    /// Block time in seconds
    pub block_time_seconds: u64,
    
    /// Contract addresses for this chain
    pub contract_addresses: HashMap<String, String>,
}

/// Configuration for the API server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Host to bind the API server to
    pub host: String,
    
    /// Port to bind the API server to
    pub port: u16,
    
    /// Base URL for the API
    pub base_url: String,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
    
    /// Enable API key authentication
    pub enable_api_key_auth: bool,
    
    /// API key (only used when enable_api_key_auth is true)
    pub api_key: Option<String>,
    
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    
    /// Rate limit requests per minute
    pub rate_limit_per_minute: Option<u32>,
}

/// Configuration for security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Secret key for signing JWTs
    pub jwt_secret: String,
    
    /// JWT expiration time in seconds
    pub jwt_expiration_seconds: u64,
    
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    
    /// Enable two-factor authentication
    pub enable_2fa: bool,
    
    /// Minimum password length
    pub min_password_length: u8,
    
    /// Number of required password character classes (lowercase, uppercase, numbers, symbols)
    pub required_password_character_classes: u8,
    
    /// Maximum login attempts before lockout
    pub max_login_attempts: u8,
    
    /// Login lockout duration in seconds
    pub login_lockout_seconds: u64,
}

/// Configuration manager for handling configuration
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: String,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: &str) -> Result<Self> {
        let config = if Path::new(config_path).exists() {
            Self::load_from_file(config_path)?
        } else {
            return Err(Error::ConfigError(format!(
                "Configuration file not found: {}",
                config_path
            )));
        };
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path: config_path.to_string(),
        })
    }
    
    /// Create a new configuration manager with default configuration
    pub fn with_defaults(config_path: &str) -> Self {
        let config = Self::create_default_config();
        
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: config_path.to_string(),
        }
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> Config {
        self.config.read().unwrap().clone()
    }
    
    /// Update the configuration
    pub fn update_config(&self, config: Config) -> Result<()> {
        let mut config_lock = self.config.write().unwrap();
        *config_lock = config;
        
        self.save_to_file(&self.config_path)?;
        
        Ok(())
    }
    
    /// Save the configuration to a file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let config = self.config.read().unwrap();
        let config_json = serde_json::to_string_pretty(&*config)
            .map_err(|e| Error::ConfigError(format!("Failed to serialize config: {}", e)))?;
        
        let mut file = File::create(path)
            .map_err(|e| Error::ConfigError(format!("Failed to create config file: {}", e)))?;
        
        file.write_all(config_json.as_bytes())
            .map_err(|e| Error::ConfigError(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Load the configuration from a file
    fn load_from_file(path: &str) -> Result<Config> {
        let mut file = File::open(path)
            .map_err(|e| Error::ConfigError(format!("Failed to open config file: {}", e)))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::ConfigError(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = serde_json::from_str(&contents)
            .map_err(|e| Error::ConfigError(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
    }
    
    /// Create a default configuration
    fn create_default_config() -> Config {
        Config {
            dao: DaoConfig {
                name: "AtomSi DAO".to_string(),
                description: "A decentralized autonomous organization".to_string(),
                version: "0.1.0".to_string(),
                website_url: Some("https://atomsi-dao.example.com".to_string()),
                default_chain_id: 1, // Ethereum mainnet
                default_token_symbol: "ATOM".to_string(),
                token_contract_address: "0x0000000000000000000000000000000000000000".to_string(),
                admin_addresses: vec!["0x0000000000000000000000000000000000000000".to_string()],
            },
            database: DatabaseConfig {
                db_type: "sqlite".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                name: "atomsi_dao".to_string(),
                username: "user".to_string(),
                password: "password".to_string(),
                pool_size: 10,
                sqlite_path: Some("./data/atomsi_dao.db".to_string()),
            },
            blockchain: BlockchainConfig {
                rpc_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
                chain_id: 1,
                explorer_url: "https://etherscan.io".to_string(),
                gas_price_strategy: "standard".to_string(),
                static_gas_price_gwei: Some(50),
                supported_chains: {
                    let mut chains = HashMap::new();
                    chains.insert(
                        "ethereum".to_string(),
                        ChainConfig {
                            name: "Ethereum Mainnet".to_string(),
                            chain_id: 1,
                            rpc_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
                            explorer_url: "https://etherscan.io".to_string(),
                            currency_symbol: "ETH".to_string(),
                            block_time_seconds: 15,
                            contract_addresses: {
                                let mut addresses = HashMap::new();
                                addresses.insert(
                                    "token".to_string(),
                                    "0x0000000000000000000000000000000000000000".to_string(),
                                );
                                addresses
                            },
                        },
                    );
                    chains
                },
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                base_url: "/api/v1".to_string(),
                enable_cors: true,
                cors_allowed_origins: vec!["*".to_string()],
                enable_api_key_auth: false,
                api_key: None,
                enable_rate_limiting: true,
                rate_limit_per_minute: Some(60),
            },
            security: SecurityConfig {
                jwt_secret: "change_this_to_a_secure_random_string".to_string(),
                jwt_expiration_seconds: 86400, // 24 hours
                session_timeout_seconds: 3600, // 1 hour
                enable_2fa: false,
                min_password_length: 8,
                required_password_character_classes: 3,
                max_login_attempts: 5,
                login_lockout_seconds: 300, // 5 minutes
            },
            custom: HashMap::new(),
        }
    }
    
    /// Get a specific section of the configuration
    pub fn get_section<T: serde::de::DeserializeOwned>(&self, section: &str) -> Result<T> {
        let config = self.config.read().unwrap();
        
        let value = match section {
            "dao" => serde_json::to_value(&config.dao),
            "database" => serde_json::to_value(&config.database),
            "blockchain" => serde_json::to_value(&config.blockchain),
            "api" => serde_json::to_value(&config.api),
            "security" => serde_json::to_value(&config.security),
            _ => {
                if let Some(value) = config.custom.get(section) {
                    Ok(value.clone())
                } else {
                    return Err(Error::ConfigError(format!("Section not found: {}", section)));
                }
            }
        }
        .map_err(|e| Error::ConfigError(format!("Failed to serialize section: {}", e)))?;
        
        let section_config: T = serde_json::from_value(value)
            .map_err(|e| Error::ConfigError(format!("Failed to deserialize section: {}", e)))?;
        
        Ok(section_config)
    }
    
    /// Set a custom configuration value
    pub fn set_custom_value(&self, key: &str, value: serde_json::Value) -> Result<()> {
        let mut config = self.config.write().unwrap();
        config.custom.insert(key.to_string(), value);
        
        drop(config);
        self.save_to_file(&self.config_path)?;
        
        Ok(())
    }
    
    /// Get a custom configuration value
    pub fn get_custom_value(&self, key: &str) -> Option<serde_json::Value> {
        let config = self.config.read().unwrap();
        config.custom.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_create_default_config() {
        let config = ConfigManager::create_default_config();
        
        assert_eq!(config.dao.name, "AtomSi DAO");
        assert_eq!(config.blockchain.chain_id, 1);
        assert_eq!(config.api.port, 8080);
        assert_eq!(config.security.min_password_length, 8);
    }
    
    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        let config_path_str = config_path.to_str().unwrap();
        
        // Create a new config and save it
        let mut config = ConfigManager::create_default_config();
        config.dao.name = "Test DAO".to_string();
        
        let manager = ConfigManager {
            config: Arc::new(RwLock::new(config.clone())),
            config_path: config_path_str.to_string(),
        };
        
        manager.save_to_file(config_path_str).unwrap();
        
        // Load the config and verify it
        let loaded_config = ConfigManager::load_from_file(config_path_str).unwrap();
        
        assert_eq!(loaded_config.dao.name, "Test DAO");
        assert_eq!(loaded_config.api.port, 8080);
    }
    
    #[test]
    fn test_update_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        let config_path_str = config_path.to_str().unwrap();
        
        // Create a config manager with default config
        let manager = ConfigManager::with_defaults(config_path_str);
        
        // Update the config
        let mut config = manager.get_config();
        config.dao.name = "Updated DAO".to_string();
        config.api.port = 9090;
        
        manager.update_config(config).unwrap();
        
        // Verify the update
        let updated_config = manager.get_config();
        assert_eq!(updated_config.dao.name, "Updated DAO");
        assert_eq!(updated_config.api.port, 9090);
    }
    
    #[test]
    fn test_get_section() {
        let manager = ConfigManager::with_defaults("test_config.json");
        
        // Get the DAO section
        let dao_section: DaoConfig = manager.get_section("dao").unwrap();
        assert_eq!(dao_section.name, "AtomSi DAO");
        
        // Get the API section
        let api_section: ApiConfig = manager.get_section("api").unwrap();
        assert_eq!(api_section.port, 8080);
    }
    
    #[test]
    fn test_custom_values() {
        let manager = ConfigManager::with_defaults("test_config.json");
        
        // Set a custom value
        let value = serde_json::json!({
            "key1": "value1",
            "key2": 42
        });
        
        manager.set_custom_value("custom_section", value.clone()).unwrap();
        
        // Get the custom value
        let retrieved_value = manager.get_custom_value("custom_section").unwrap();
        assert_eq!(retrieved_value, value);
        
        // Verify it's in the config
        let config = manager.get_config();
        assert_eq!(config.custom.get("custom_section").unwrap(), &value);
    }
} 