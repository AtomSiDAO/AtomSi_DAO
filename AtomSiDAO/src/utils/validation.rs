//! Validation utilities for the AtomSi DAO
//!
//! This module provides validation functions for different types of data.

use regex::Regex;
use lazy_static::lazy_static;
use crate::utils::is_valid_hex;
use std::collections::HashMap;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    ).unwrap();
    
    static ref ETH_ADDRESS_REGEX: Regex = Regex::new(
        r"^0x[a-fA-F0-9]{40}$"
    ).unwrap();
    
    static ref URL_REGEX: Regex = Regex::new(
        r"^(https?://)?([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,6}(/[-a-zA-Z0-9@:%_\+.~#?&//=]*)?$"
    ).unwrap();
}

/// Error type for validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Error message
    pub message: String,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new<S: Into<String>>(field: S, message: S) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Validation result type
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Validate an email address
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_valid_email;
///
/// assert!(is_valid_email("user@example.com"));
/// assert!(!is_valid_email("invalid-email"));
/// ```
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

/// Validate an Ethereum address
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_valid_eth_address;
///
/// assert!(is_valid_eth_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e"));
/// assert!(!is_valid_eth_address("invalid-address"));
/// ```
pub fn is_valid_eth_address(address: &str) -> bool {
    ETH_ADDRESS_REGEX.is_match(address)
}

/// Validate a URL
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_valid_url;
///
/// assert!(is_valid_url("https://example.com"));
/// assert!(is_valid_url("http://sub.example.com/path"));
/// assert!(!is_valid_url("invalid-url"));
/// ```
pub fn is_valid_url(url: &str) -> bool {
    URL_REGEX.is_match(url)
}

/// Validate a percentage (0-100)
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_valid_percentage;
///
/// assert!(is_valid_percentage(50));
/// assert!(is_valid_percentage(0));
/// assert!(is_valid_percentage(100));
/// assert!(!is_valid_percentage(101));
/// assert!(!is_valid_percentage(-1));
/// ```
pub fn is_valid_percentage(percentage: i32) -> bool {
    percentage >= 0 && percentage <= 100
}

/// Validate that a string is not empty or contains only whitespace
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_not_empty;
///
/// assert!(is_not_empty("Hello"));
/// assert!(!is_not_empty(""));
/// assert!(!is_not_empty("  "));
/// ```
pub fn is_not_empty(text: &str) -> bool {
    !text.trim().is_empty()
}

/// Validate a string against a minimum and maximum length
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_valid_length;
///
/// assert!(is_valid_length("Hello", 1, 10));
/// assert!(!is_valid_length("Hello", 10, 20));
/// assert!(!is_valid_length("Hello", 0, 3));
/// ```
pub fn is_valid_length(text: &str, min: usize, max: usize) -> bool {
    let len = text.chars().count();
    len >= min && len <= max
}

/// Validate that a number is within a range (inclusive)
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_within_range;
///
/// assert!(is_within_range(5, 1, 10));
/// assert!(is_within_range(1, 1, 10));
/// assert!(is_within_range(10, 1, 10));
/// assert!(!is_within_range(0, 1, 10));
/// assert!(!is_within_range(11, 1, 10));
/// ```
pub fn is_within_range<T: PartialOrd>(value: T, min: T, max: T) -> bool {
    value >= min && value <= max
}

/// Check if a string contains only alphanumeric characters
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_alphanumeric;
///
/// assert!(is_alphanumeric("abc123"));
/// assert!(!is_alphanumeric("abc-123"));
/// ```
pub fn is_alphanumeric(text: &str) -> bool {
    text.chars().all(|c| c.is_alphanumeric())
}

/// Check if a string is a valid password (at least 8 characters, 
/// with at least one uppercase letter, one lowercase letter, one digit)
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::is_valid_password;
///
/// assert!(is_valid_password("Password123"));
/// assert!(!is_valid_password("password"));
/// assert!(!is_valid_password("PASSWORD"));
/// assert!(!is_valid_password("Pass"));
/// ```
pub fn is_valid_password(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }
    
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    
    has_uppercase && has_lowercase && has_digit
}

/// Validate an address format (e.g., Ethereum address)
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::validate_address;
///
/// assert!(validate_address("0x1234567890123456789012345678901234567890").is_ok());
/// assert!(validate_address("invalid").is_err());
/// ```
pub fn validate_address(address: &str) -> ValidationResult {
    if !is_valid_hex(address) {
        return Err(vec![ValidationError::new(
            "address",
            "Address must be a valid hex string starting with 0x",
        )]);
    }

    // Remove the 0x prefix
    let addr = &address[2..];
    
    // Check length (Ethereum addresses are 40 hex chars = 20 bytes)
    if addr.len() != 40 {
        return Err(vec![ValidationError::new(
            "address",
            "Address must be 20 bytes (40 hex characters) long",
        )]);
    }
    
    Ok(())
}

/// Validate a token symbol
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::validate_token_symbol;
///
/// assert!(validate_token_symbol("ETH").is_ok());
/// assert!(validate_token_symbol("").is_err());
/// ```
pub fn validate_token_symbol(symbol: &str) -> ValidationResult {
    let mut errors = Vec::new();
    
    if symbol.is_empty() {
        errors.push(ValidationError::new("symbol", "Token symbol cannot be empty"));
    }
    
    if symbol.len() > 10 {
        errors.push(ValidationError::new(
            "symbol",
            "Token symbol cannot be longer than 10 characters",
        ));
    }
    
    // Check that it only contains allowed characters
    if !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
        errors.push(ValidationError::new(
            "symbol",
            "Token symbol can only contain alphanumeric characters",
        ));
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a token amount
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::validate_token_amount;
///
/// assert!(validate_token_amount(100, 0).is_ok());
/// assert!(validate_token_amount(0, 0).is_err());
/// ```
pub fn validate_token_amount(amount: u64, min_amount: u64) -> ValidationResult {
    if amount <= min_amount {
        return Err(vec![ValidationError::new(
            "amount",
            format!("Amount must be greater than {}", min_amount),
        )]);
    }
    
    Ok(())
}

/// Validate required fields in a map
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::validate_required_fields;
/// use std::collections::HashMap;
///
/// let mut data = HashMap::new();
/// data.insert("name".to_string(), "John".to_string());
/// data.insert("age".to_string(), "30".to_string());
///
/// assert!(validate_required_fields(&data, &["name", "age"]).is_ok());
/// assert!(validate_required_fields(&data, &["name", "email"]).is_err());
/// ```
pub fn validate_required_fields(
    data: &HashMap<String, String>,
    required_fields: &[&str],
) -> ValidationResult {
    let mut errors = Vec::new();
    
    for field in required_fields {
        if !data.contains_key(*field) || data[*field].is_empty() {
            errors.push(ValidationError::new(
                *field,
                format!("Field '{}' is required", field),
            ));
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a string length
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::validate_string_length;
///
/// assert!(validate_string_length("test", 1, 10).is_ok());
/// assert!(validate_string_length("", 1, 10).is_err());
/// assert!(validate_string_length("this is too long", 1, 10).is_err());
/// ```
pub fn validate_string_length(value: &str, min_length: usize, max_length: usize) -> ValidationResult {
    let mut errors = Vec::new();
    
    if value.len() < min_length {
        errors.push(ValidationError::new(
            "length",
            format!("Value must be at least {} characters long", min_length),
        ));
    }
    
    if value.len() > max_length {
        errors.push(ValidationError::new(
            "length",
            format!("Value cannot be longer than {} characters", max_length),
        ));
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a numeric range
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::validation::validate_numeric_range;
///
/// assert!(validate_numeric_range(5, 1, 10).is_ok());
/// assert!(validate_numeric_range(0, 1, 10).is_err());
/// assert!(validate_numeric_range(11, 1, 10).is_err());
/// ```
pub fn validate_numeric_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
) -> ValidationResult {
    let mut errors = Vec::new();
    
    if value < min {
        errors.push(ValidationError::new(
            "range",
            format!("Value must be at least {}", min),
        ));
    }
    
    if value > max {
        errors.push(ValidationError::new(
            "range",
            format!("Value cannot be greater than {}", max),
        ));
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name@example.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("@example.com"));
    }
    
    #[test]
    fn test_is_valid_eth_address() {
        assert!(is_valid_eth_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e"));
        assert!(is_valid_eth_address("0x0000000000000000000000000000000000000000"));
        assert!(!is_valid_eth_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44"));  // too short
        assert!(!is_valid_eth_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44eX")); // too long
        assert!(!is_valid_eth_address("742d35Cc6634C0532925a3b844Bc454e4438f44e"));   // missing 0x
        assert!(!is_valid_eth_address("0xGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijkl")); // invalid chars
    }
    
    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("example.com"));
        assert!(is_valid_url("sub.example.com"));
        assert!(is_valid_url("example.com/path"));
        assert!(is_valid_url("example.com/path?query=value"));
        assert!(!is_valid_url("invalid-url"));
        assert!(!is_valid_url("http://"));
        assert!(!is_valid_url("http://.com"));
    }
    
    #[test]
    fn test_is_valid_percentage() {
        assert!(is_valid_percentage(0));
        assert!(is_valid_percentage(50));
        assert!(is_valid_percentage(100));
        assert!(!is_valid_percentage(-1));
        assert!(!is_valid_percentage(101));
    }
    
    #[test]
    fn test_is_not_empty() {
        assert!(is_not_empty("Hello"));
        assert!(is_not_empty(" Hello "));
        assert!(!is_not_empty(""));
        assert!(!is_not_empty("  "));
        assert!(!is_not_empty("\n\t"));
    }
    
    #[test]
    fn test_is_valid_length() {
        assert!(is_valid_length("Hello", 1, 10));
        assert!(is_valid_length("Hello", 5, 5));
        assert!(!is_valid_length("Hello", 10, 20));
        assert!(!is_valid_length("Hello", 0, 3));
    }
    
    #[test]
    fn test_is_within_range() {
        assert!(is_within_range(5, 1, 10));
        assert!(is_within_range(1, 1, 10));
        assert!(is_within_range(10, 1, 10));
        assert!(!is_within_range(0, 1, 10));
        assert!(!is_within_range(11, 1, 10));
        
        // Test with floats
        assert!(is_within_range(5.5, 1.0, 10.0));
        assert!(!is_within_range(10.1, 1.0, 10.0));
    }
    
    #[test]
    fn test_is_alphanumeric() {
        assert!(is_alphanumeric("abc123"));
        assert!(is_alphanumeric("ABC123"));
        assert!(!is_alphanumeric("abc-123"));
        assert!(!is_alphanumeric("abc 123"));
        assert!(!is_alphanumeric("abc!123"));
    }
    
    #[test]
    fn test_is_valid_password() {
        assert!(is_valid_password("Password123"));
        assert!(is_valid_password("P@ssw0rd"));
        assert!(!is_valid_password("password"));  // no uppercase
        assert!(!is_valid_password("PASSWORD"));  // no lowercase
        assert!(!is_valid_password("Password"));  // no digit
        assert!(!is_valid_password("Pass1"));     // too short
    }
    
    #[test]
    fn test_validate_address() {
        // Valid Ethereum address
        assert!(validate_address("0x1234567890123456789012345678901234567890").is_ok());
        
        // Invalid addresses
        assert!(validate_address("not-an-address").is_err());
        assert!(validate_address("0x123").is_err()); // Too short
        assert!(validate_address("0x123456789012345678901234567890123456789012").is_err()); // Too long
        assert!(validate_address("1234567890123456789012345678901234567890").is_err()); // Missing 0x
    }
    
    #[test]
    fn test_validate_token_symbol() {
        // Valid symbols
        assert!(validate_token_symbol("ETH").is_ok());
        assert!(validate_token_symbol("BTC").is_ok());
        assert!(validate_token_symbol("USDC").is_ok());
        
        // Invalid symbols
        assert!(validate_token_symbol("").is_err()); // Empty
        assert!(validate_token_symbol("TOOLONGTOKEN").is_err()); // Too long
        assert!(validate_token_symbol("ETH!").is_err()); // Contains special character
    }
    
    #[test]
    fn test_validate_token_amount() {
        // Valid amounts
        assert!(validate_token_amount(100, 0).is_ok());
        assert!(validate_token_amount(1, 0).is_ok());
        
        // Invalid amounts
        assert!(validate_token_amount(0, 0).is_err()); // Equal to min
        assert!(validate_token_amount(10, 100).is_err()); // Less than min
    }
    
    #[test]
    fn test_validate_required_fields() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John".to_string());
        data.insert("age".to_string(), "30".to_string());
        data.insert("empty".to_string(), "".to_string());
        
        // Valid
        assert!(validate_required_fields(&data, &["name", "age"]).is_ok());
        
        // Invalid
        assert!(validate_required_fields(&data, &["name", "email"]).is_err()); // Missing field
        assert!(validate_required_fields(&data, &["name", "empty"]).is_err()); // Empty field
    }
    
    #[test]
    fn test_validate_string_length() {
        // Valid
        assert!(validate_string_length("test", 1, 10).is_ok());
        assert!(validate_string_length("a", 1, 10).is_ok());
        assert!(validate_string_length("1234567890", 1, 10).is_ok());
        
        // Invalid
        assert!(validate_string_length("", 1, 10).is_err()); // Too short
        assert!(validate_string_length("12345678901", 1, 10).is_err()); // Too long
    }
    
    #[test]
    fn test_validate_numeric_range() {
        // Valid
        assert!(validate_numeric_range(5, 1, 10).is_ok());
        assert!(validate_numeric_range(1, 1, 10).is_ok());
        assert!(validate_numeric_range(10, 1, 10).is_ok());
        
        // Invalid
        assert!(validate_numeric_range(0, 1, 10).is_err()); // Too small
        assert!(validate_numeric_range(11, 1, 10).is_err()); // Too large
    }
} 