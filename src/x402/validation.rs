//! x402 v2 Validation Functions
//!
//! Validation utilities for x402 protocol messages and parameters.

use crate::error::{McpError, McpResult};
use super::types::X402_VERSION;

/// Validates CAIP-2 network string format
///
/// CAIP-2 format: {namespace}:{reference}
/// Examples:
/// - solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp (Solana mainnet)
/// - eip155:1 (Ethereum mainnet)
/// - eip155:84532 (Base Sepolia)
///
/// # Arguments
/// * `network` - Network string to validate
///
/// # Returns
/// * `McpResult<()>` - Ok if valid, Err if invalid
pub fn validate_caip2_network(network: &str) -> McpResult<()> {
    let parts: Vec<&str> = network.split(':').collect();
    
    if parts.len() != 2 {
        return Err(McpError::validation(format!(
            "Invalid CAIP-2 network format '{}'. Expected format: namespace:reference",
            network
        )));
    }
    
    let namespace = parts[0];
    let reference = parts[1];
    
    if namespace.is_empty() || reference.is_empty() {
        return Err(McpError::validation(format!(
            "Invalid CAIP-2 network '{}'. Namespace and reference must not be empty",
            network
        )));
    }
    
    // Validate namespace contains only lowercase letters and numbers
    if !namespace.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        return Err(McpError::validation(format!(
            "Invalid CAIP-2 namespace '{}'. Must contain only lowercase letters and digits",
            namespace
        )));
    }
    
    Ok(())
}

/// Validates x402 protocol version
///
/// # Arguments
/// * `version` - Version number to validate
///
/// # Returns
/// * `McpResult<()>` - Ok if version is 2, Err otherwise
pub fn validate_x402_version(version: u32) -> McpResult<()> {
    if version != X402_VERSION {
        return Err(McpError::validation(format!(
            "Invalid x402 version {}. Expected version {}",
            version, X402_VERSION
        )));
    }
    Ok(())
}

/// Validates payment amount is a valid positive integer string
///
/// # Arguments
/// * `amount` - Amount string to validate
///
/// # Returns
/// * `McpResult<()>` - Ok if valid, Err otherwise
pub fn validate_payment_amount(amount: &str) -> McpResult<()> {
    if amount.is_empty() {
        return Err(McpError::validation("Payment amount cannot be empty".to_string()));
    }
    
    // Parse as u64 to ensure it's a valid positive integer
    amount.parse::<u64>().map_err(|_| {
        McpError::validation(format!(
            "Invalid payment amount '{}'. Must be a positive integer",
            amount
        ))
    })?;
    
    Ok(())
}

/// Validates timeout is within reasonable bounds
///
/// # Arguments
/// * `timeout_seconds` - Timeout value to validate
///
/// # Returns
/// * `McpResult<()>` - Ok if valid, Err otherwise
pub fn validate_timeout(timeout_seconds: u64) -> McpResult<()> {
    const MIN_TIMEOUT: u64 = 1;
    const MAX_TIMEOUT: u64 = 300; // 5 minutes max
    
    if timeout_seconds < MIN_TIMEOUT || timeout_seconds > MAX_TIMEOUT {
        return Err(McpError::validation(format!(
            "Invalid timeout {}. Must be between {} and {} seconds",
            timeout_seconds, MIN_TIMEOUT, MAX_TIMEOUT
        )));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_caip2_network_valid() {
        assert!(validate_caip2_network("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp").is_ok());
        assert!(validate_caip2_network("eip155:1").is_ok());
        assert!(validate_caip2_network("eip155:84532").is_ok());
        assert!(validate_caip2_network("cosmos:cosmoshub-4").is_ok());
    }

    #[test]
    fn test_validate_caip2_network_invalid() {
        assert!(validate_caip2_network("invalid").is_err());
        assert!(validate_caip2_network("").is_err());
        assert!(validate_caip2_network("namespace:").is_err());
        assert!(validate_caip2_network(":reference").is_err());
        assert!(validate_caip2_network("UPPERCASE:ref").is_err());
        assert!(validate_caip2_network("namespace:ref:extra").is_err());
    }

    #[test]
    fn test_validate_x402_version() {
        assert!(validate_x402_version(2).is_ok());
        assert!(validate_x402_version(1).is_err());
        assert!(validate_x402_version(3).is_err());
    }

    #[test]
    fn test_validate_payment_amount() {
        assert!(validate_payment_amount("1000000").is_ok());
        assert!(validate_payment_amount("0").is_ok());
        assert!(validate_payment_amount("999999999999999").is_ok());
        
        assert!(validate_payment_amount("").is_err());
        assert!(validate_payment_amount("abc").is_err());
        assert!(validate_payment_amount("-1000").is_err());
        assert!(validate_payment_amount("10.5").is_err());
    }

    #[test]
    fn test_validate_timeout() {
        assert!(validate_timeout(1).is_ok());
        assert!(validate_timeout(60).is_ok());
        assert!(validate_timeout(300).is_ok());
        
        assert!(validate_timeout(0).is_err());
        assert!(validate_timeout(301).is_err());
        assert!(validate_timeout(1000).is_err());
    }
}
