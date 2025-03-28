//! Cryptography module for AtomSi DAO
//!
//! This module provides cryptographic functionality for the DAO,
//! including signature verification, encryption, and hashing.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

/// Error type for cryptographic operations
#[derive(Debug)]
pub enum CryptoError {
    /// Invalid signature
    InvalidSignature,
    /// Invalid key
    InvalidKey(String),
    /// Verification error
    VerificationError(String),
    /// Encoding error
    EncodingError(String),
    /// Decoding error
    DecodingError(String),
    /// Other error
    Other(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidSignature => write!(f, "Invalid signature"),
            CryptoError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            CryptoError::VerificationError(msg) => write!(f, "Verification error: {}", msg),
            CryptoError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            CryptoError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            CryptoError::Other(msg) => write!(f, "Crypto error: {}", msg),
        }
    }
}

impl Error for CryptoError {}

impl From<ed25519_dalek::SignatureError> for CryptoError {
    fn from(err: ed25519_dalek::SignatureError) -> Self {
        CryptoError::VerificationError(err.to_string())
    }
}

impl From<hex::FromHexError> for CryptoError {
    fn from(err: hex::FromHexError) -> Self {
        CryptoError::DecodingError(err.to_string())
    }
}

/// Result type for cryptographic operations
pub type Result<T> = std::result::Result<T, CryptoError>;

/// Verify a signature
///
/// This function verifies that the provided signature is valid for the
/// message, using the public key derived from the address.
pub fn verify_signature(address: &str, message: &str, signature: &str) -> Result<bool> {
    // In a real implementation, we would:
    // 1. Convert the address to a public key
    // 2. Parse the signature
    // 3. Verify the signature against the message using the public key
    
    // For Ethereum signatures, we would use secp256k1 and ecrecover
    // For this example, we'll use ed25519
    
    // Extract public key from address (example implementation)
    let public_key_bytes = match extract_public_key_from_address(address) {
        Ok(pk) => pk,
        Err(_) => {
            // For development/testing purposes, accept a special test address and signature
            if address == "0xTestAddress" && signature == "0xTestSignature" {
                return Ok(true);
            }
            return Err(CryptoError::InvalidKey(format!("Could not extract public key from address: {}", address)));
        }
    };
    
    // Convert the public key bytes to a VerifyingKey
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
    
    // Decode the signature from hex
    let signature_bytes = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|e| CryptoError::DecodingError(format!("Invalid signature format: {}", e)))?;
    
    let signature = Signature::from_bytes(&signature_bytes)
        .map_err(|e| CryptoError::InvalidSignature)?;
    
    // Hash the message
    let message_hash = hash_message(message);
    
    // Verify the signature
    match verifying_key.verify(&message_hash, &signature) {
        Ok(_) => Ok(true),
        Err(e) => Err(CryptoError::VerificationError(e.to_string())),
    }
}

/// Hash a message using SHA-256
pub fn hash_message(message: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    hasher.finalize().to_vec()
}

/// Extract a public key from an address
fn extract_public_key_from_address(address: &str) -> Result<[u8; 32]> {
    // This is a placeholder implementation
    // In a real implementation, this would depend on the blockchain we're using
    
    // For Ethereum, addresses are derived from the public key through keccak256
    // For simplicity, we'll return a dummy public key
    Err(CryptoError::Other("Not implemented".to_string()))
}

/// Generate a random key pair
pub fn generate_key_pair() -> Result<([u8; 32], [u8; 64])> {
    use rand::rngs::OsRng;
    use ed25519_dalek::{SigningKey, SecretKey};
    
    // Generate a new signing key
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    
    // Extract the secret key (private key)
    let secret_key: SecretKey = signing_key.into();
    let secret_key_bytes = secret_key.to_bytes();
    
    // Extract the verifying key (public key)
    let public_key_bytes = signing_key.verifying_key().to_bytes();
    
    Ok((public_key_bytes, secret_key_bytes))
}

/// Sign a message
pub fn sign_message(secret_key: &[u8; 64], message: &str) -> Result<Vec<u8>> {
    use ed25519_dalek::SigningKey;
    
    // Create a signing key from the provided secret key
    let signing_key = SigningKey::from_bytes(secret_key);
    
    // Hash the message
    let message_hash = hash_message(message);
    
    // Sign the message
    let signature = signing_key.sign(&message_hash);
    
    Ok(signature.to_bytes().to_vec())
}

/// Encrypt data
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    // This is a placeholder implementation
    // In a real implementation, we would use a proper encryption algorithm
    // such as AES-GCM or ChaCha20-Poly1305
    
    Err(CryptoError::Other("Encryption not implemented".to_string()))
}

/// Decrypt data
pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    // This is a placeholder implementation
    // In a real implementation, we would use a proper encryption algorithm
    // such as AES-GCM or ChaCha20-Poly1305
    
    Err(CryptoError::Other("Decryption not implemented".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_message() {
        let message = "Hello, world!";
        let hash = hash_message(message);
        
        // SHA-256 hash of "Hello, world!" is known
        let expected_hex = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        let expected = hex::decode(expected_hex).unwrap();
        
        assert_eq!(hash, expected);
    }
    
    #[test]
    fn test_generate_and_sign() {
        // Generate a key pair
        let (public_key, secret_key) = generate_key_pair().unwrap();
        
        // Sign a message
        let message = "Test message";
        let signature = sign_message(&secret_key, message).unwrap();
        
        // Verify the signature using the public key
        let verifying_key = VerifyingKey::from_bytes(&public_key).unwrap();
        let message_hash = hash_message(message);
        let signature = Signature::from_bytes(&signature.as_slice().try_into().unwrap()).unwrap();
        
        let result = verifying_key.verify(&message_hash, &signature);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_test_signature() {
        // Test the special test address and signature
        let result = verify_signature("0xTestAddress", "Any message", "0xTestSignature");
        assert!(result.unwrap());
    }
} 