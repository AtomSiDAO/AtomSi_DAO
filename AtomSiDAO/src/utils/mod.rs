//! Utils module for AtomSi DAO
//!
//! This module provides utility functions and helpers for the DAO.

pub mod time;
pub mod validation;

/// Format an amount with token symbol
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::format_token_amount;
///
/// let amount = 1000000000000000000;
/// let formatted = format_token_amount(amount, "ETH", 18);
/// assert_eq!(formatted, "1.000000000000000000 ETH");
/// ```
pub fn format_token_amount(amount: u64, symbol: &str, decimals: u8) -> String {
    let factor = 10u64.pow(decimals as u32);
    let whole = amount / factor;
    let fractional = amount % factor;
    
    let fractional_str = format!("{:0width$}", fractional, width = decimals as usize);
    
    format!("{}.{} {}", whole, fractional_str, symbol)
}

/// Parse a token amount from a string
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::parse_token_amount;
///
/// let amount_str = "1.5 ETH";
/// let parsed = parse_token_amount(amount_str, 18);
/// assert_eq!(parsed, Ok(1500000000000000000));
/// ```
pub fn parse_token_amount(amount_str: &str, decimals: u8) -> Result<u64, String> {
    // Parse the string to extract the number
    let number_str = amount_str
        .split_whitespace()
        .next()
        .ok_or_else(|| "Invalid amount format".to_string())?;
    
    // Split by decimal point
    let parts: Vec<&str> = number_str.split('.').collect();
    if parts.len() > 2 {
        return Err("Invalid number format".to_string());
    }
    
    let whole_part = parts[0].parse::<u64>().map_err(|e| e.to_string())?;
    
    let factor = 10u64.pow(decimals as u32);
    let mut amount = whole_part * factor;
    
    if parts.len() == 2 {
        let fractional_str = parts[1];
        
        // Ensure the fractional part is not too long
        if fractional_str.len() > decimals as usize {
            return Err(format!(
                "Fractional part too long, maximum {} decimal places allowed",
                decimals
            ));
        }
        
        // Parse the fractional part
        let fractional_value = fractional_str.parse::<u64>().map_err(|e| e.to_string())?;
        
        // Add the fractional part
        let fractional_factor = 10u64.pow((decimals - fractional_str.len() as u8) as u32);
        amount += fractional_value * fractional_factor;
    }
    
    Ok(amount)
}

/// Truncate a string to a maximum length with ellipsis
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::truncate_string;
///
/// let text = "This is a very long text that should be truncated";
/// let truncated = truncate_string(text, 20);
/// assert_eq!(truncated, "This is a very long...");
/// ```
pub fn truncate_string(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length])
    }
}

/// Check if a string is a valid hex string
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::is_valid_hex;
///
/// assert!(is_valid_hex("0x1a2b3c4d5e6f"));
/// assert!(!is_valid_hex("0xGHIJKL"));
/// ```
pub fn is_valid_hex(hex: &str) -> bool {
    if !hex.starts_with("0x") {
        return false;
    }
    
    let hex_str = &hex[2..];
    hex_str.chars().all(|c| c.is_digit(16))
}

/// Convert a byte array to a hex string
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::bytes_to_hex;
///
/// let bytes = vec![0x1a, 0x2b, 0x3c, 0x4d];
/// assert_eq!(bytes_to_hex(&bytes), "0x1a2b3c4d");
/// ```
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let hex = hex::encode(bytes);
    format!("0x{}", hex)
}

/// Convert a hex string to a byte array
///
/// # Examples
///
/// ```
/// use atomsi_dao::utils::hex_to_bytes;
///
/// let hex = "0x1a2b3c4d";
/// assert_eq!(hex_to_bytes(hex), Ok(vec![0x1a, 0x2b, 0x3c, 0x4d]));
/// ```
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if !is_valid_hex(hex) {
        return Err("Invalid hex string".to_string());
    }
    
    let hex_str = &hex[2..];
    hex::decode(hex_str).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_token_amount() {
        // Test with 18 decimals (common for Ethereum tokens)
        assert_eq!(
            format_token_amount(1000000000000000000, "ETH", 18),
            "1.000000000000000000 ETH"
        );
        
        // Test with 6 decimals (common for stablecoins)
        assert_eq!(format_token_amount(1000000, "USDC", 6), "1.000000 USDC");
        
        // Test with 0 decimals
        assert_eq!(format_token_amount(100, "TOKEN", 0), "100. TOKEN");
    }
    
    #[test]
    fn test_parse_token_amount() {
        // Test with 18 decimals
        assert_eq!(parse_token_amount("1.5 ETH", 18), Ok(1500000000000000000));
        
        // Test with 6 decimals
        assert_eq!(parse_token_amount("1.5 USDC", 6), Ok(1500000));
        
        // Test with 0 decimals
        assert_eq!(parse_token_amount("100 TOKEN", 0), Ok(100));
        
        // Test error cases
        assert!(parse_token_amount("invalid", 18).is_err());
        assert!(parse_token_amount("1.1.1", 18).is_err());
    }
    
    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is a ...");
    }
    
    #[test]
    fn test_is_valid_hex() {
        assert!(is_valid_hex("0x1234567890abcdef"));
        assert!(!is_valid_hex("0xghijkl"));
        assert!(!is_valid_hex("1234567890abcdef"));
    }
    
    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(bytes_to_hex(&[0x12, 0x34, 0x56, 0x78]), "0x12345678");
    }
    
    #[test]
    fn test_hex_to_bytes() {
        assert_eq!(hex_to_bytes("0x12345678"), Ok(vec![0x12, 0x34, 0x56, 0x78]));
        assert!(hex_to_bytes("invalid").is_err());
    }
} 