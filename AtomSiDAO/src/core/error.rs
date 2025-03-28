//! Error types for the core module

use thiserror::Error;

/// Error type for DAO operations
#[derive(Debug, Error)]
pub enum DaoError {
    /// Missing configuration
    #[error("Missing configuration")]
    MissingConfig,
    
    /// Missing blockchain adapter
    #[error("Missing blockchain adapter")]
    MissingBlockchainAdapter,
    
    /// Missing database
    #[error("Missing database")]
    MissingDatabase,
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    
    /// Blockchain error
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Governance error
    #[error("Governance error: {0}")]
    GovernanceError(String),
    
    /// Token error
    #[error("Token error: {0}")]
    TokenError(String),
    
    /// Treasury error
    #[error("Treasury error: {0}")]
    TreasuryError(String),
    
    /// Proposal error
    #[error("Proposal error: {0}")]
    ProposalError(String),
    
    /// Identity error
    #[error("Identity error: {0}")]
    IdentityError(String),
    
    /// Unauthorized operation
    #[error("Unauthorized operation")]
    Unauthorized,
    
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    /// External service error
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<sqlx::Error> for DaoError {
    fn from(error: sqlx::Error) -> Self {
        Self::DatabaseError(error.to_string())
    }
}

impl<E: std::error::Error> From<deadpool_postgres::PoolError<E>> for DaoError {
    fn from(error: deadpool_postgres::PoolError<E>) -> Self {
        Self::DatabaseError(error.to_string())
    }
}

/// Result type for DAO operations
pub type Result<T> = std::result::Result<T, DaoError>; 