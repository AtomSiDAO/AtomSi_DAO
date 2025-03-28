//! Error module for AtomSi DAO
//!
//! This module defines the error types used throughout the DAO framework.

use thiserror::Error;

/// Result type for DAO operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for DAO operations
#[derive(Debug, Error)]
pub enum Error {
    /// Database errors
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Blockchain errors
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    
    /// Proposal errors
    #[error("Proposal error: {0}")]
    ProposalError(String),
    
    /// Voting errors
    #[error("Voting error: {0}")]
    VotingError(String),
    
    /// Treasury errors
    #[error("Treasury error: {0}")]
    TreasuryError(String),
    
    /// Authentication errors
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// Authorization errors
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    /// Identity errors
    #[error("Identity error: {0}")]
    IdentityError(String),
    
    /// Token errors
    #[error("Token error: {0}")]
    TokenError(String),
    
    /// Cryptography errors
    #[error("Cryptography error: {0}")]
    CryptoError(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Not found errors
    #[error("Not found: {0}")]
    NotFoundError(String),
    
    /// Already exists errors
    #[error("Already exists: {0}")]
    AlreadyExistsError(String),
    
    /// Resource not found errors
    #[error("Resource not found: {0}")]
    ResourceNotFoundError(String),
    
    /// External service errors
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    /// Rate limit errors
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Internal errors
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// Unexpected errors
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(err: tokio_postgres::Error) -> Self {
        Error::DatabaseError(err.to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::DatabaseError(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::NetworkError(err.to_string())
    }
}

/// Convert any error type to our Error type
pub fn to_internal_error<E: std::fmt::Display>(err: E) -> Error {
    Error::InternalError(err.to_string())
}

/// Helper function to convert Option to Result with NotFoundError
pub fn not_found<T>(option: Option<T>, message: &str) -> Result<T> {
    option.ok_or_else(|| Error::NotFoundError(message.to_string()))
}

/// Helper function to check if a condition is true or return an error
pub fn ensure(condition: bool, error: Error) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(error)
    }
}

/// Helper function to validate a condition or return a validation error
pub fn validate(condition: bool, message: &str) -> Result<()> {
    ensure(condition, Error::ValidationError(message.to_string()))
}

/// Helper function to check permission or return an authorization error
pub fn authorize(has_permission: bool, message: &str) -> Result<()> {
    ensure(has_permission, Error::AuthorizationError(message.to_string()))
}

/// Helper function to check if a resource exists
pub fn check_exists<T>(option: Option<T>, resource_type: &str, identifier: &str) -> Result<T> {
    option.ok_or_else(|| Error::ResourceNotFoundError(format!("{} with identifier {} not found", resource_type, identifier)))
}

/// Helper function to check if a resource already exists
pub fn check_not_exists<T>(option: Option<T>, resource_type: &str, identifier: &str) -> Result<()> {
    if option.is_some() {
        Err(Error::AlreadyExistsError(format!("{} with identifier {} already exists", resource_type, identifier)))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: Error = io_error.into();
        
        match error {
            Error::IoError(_) => {}
            _ => panic!("Expected IoError"),
        }
    }
    
    #[test]
    fn test_not_found_helper() {
        let option: Option<i32> = Some(42);
        let result = not_found(option, "Value not found");
        assert_eq!(result.unwrap(), 42);
        
        let option: Option<i32> = None;
        let result = not_found(option, "Value not found");
        assert!(matches!(result, Err(Error::NotFoundError(_))));
    }
    
    #[test]
    fn test_ensure_helper() {
        let result = ensure(true, Error::ValidationError("Test error".to_string()));
        assert!(result.is_ok());
        
        let result = ensure(false, Error::ValidationError("Test error".to_string()));
        assert!(matches!(result, Err(Error::ValidationError(_))));
    }
    
    #[test]
    fn test_validate_helper() {
        let result = validate(true, "Validation failed");
        assert!(result.is_ok());
        
        let result = validate(false, "Validation failed");
        assert!(matches!(result, Err(Error::ValidationError(_))));
    }
    
    #[test]
    fn test_authorize_helper() {
        let result = authorize(true, "Not authorized");
        assert!(result.is_ok());
        
        let result = authorize(false, "Not authorized");
        assert!(matches!(result, Err(Error::AuthorizationError(_))));
    }
    
    #[test]
    fn test_check_exists_helper() {
        let option: Option<i32> = Some(42);
        let result = check_exists(option, "Item", "42");
        assert_eq!(result.unwrap(), 42);
        
        let option: Option<i32> = None;
        let result = check_exists(option, "Item", "42");
        assert!(matches!(result, Err(Error::ResourceNotFoundError(_))));
    }
    
    #[test]
    fn test_check_not_exists_helper() {
        let option: Option<i32> = Some(42);
        let result = check_not_exists(option, "Item", "42");
        assert!(matches!(result, Err(Error::AlreadyExistsError(_))));
        
        let option: Option<i32> = None;
        let result = check_not_exists(option, "Item", "42");
        assert!(result.is_ok());
    }
} 