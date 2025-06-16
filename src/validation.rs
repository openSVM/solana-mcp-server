/// Validation module for input sanitization and security checks
use anyhow::{anyhow, Result};
use url::Url;

/// Validates that a URL is well-formed and uses HTTPS protocol
///
/// # Arguments
/// * `url_str` - The URL string to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, Err with description if invalid
///
/// # Security
/// - Enforces HTTPS protocol to prevent MITM attacks
/// - Validates URL structure to prevent injection
/// - Ensures no malformed URLs are accepted
pub fn validate_rpc_url(url_str: &str) -> Result<()> {
    // Parse the URL to ensure it's well-formed
    let url = Url::parse(url_str).map_err(|e| anyhow!("Invalid URL format: {}", e))?;

    // Enforce HTTPS for security
    if url.scheme() != "https" {
        return Err(anyhow!(
            "RPC URL must use HTTPS protocol for security. Got: {}",
            url.scheme()
        ));
    }

    // Ensure host is present
    if url.host().is_none() {
        return Err(anyhow!("RPC URL must have a valid host"));
    }

    // Additional security checks
    let host = url.host_str().unwrap();

    // Prevent localhost/internal addresses in production
    if is_internal_address(host) {
        log::warn!("Using internal/localhost address: {}", host);
    }

    // Basic format validation
    if url_str.len() > 2048 {
        return Err(anyhow!("URL too long (max 2048 characters)"));
    }

    Ok(())
}

/// Checks if an address is internal/localhost
fn is_internal_address(host: &str) -> bool {
    host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host.starts_with("192.168.")
        || host.starts_with("10.")
        || (host.starts_with("172.") && is_private_class_b(host))
}

/// Checks if an address is in the 172.16.0.0/12 private range
fn is_private_class_b(host: &str) -> bool {
    if let Some(second_octet) = host.split('.').nth(1) {
        if let Ok(octet) = second_octet.parse::<u8>() {
            return (16..=31).contains(&octet);
        }
    }
    false
}

/// Validates network ID format and content
///
/// # Arguments
/// * `network_id` - The network identifier to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, Err with description if invalid
pub fn validate_network_id(network_id: &str) -> Result<()> {
    if network_id.is_empty() {
        return Err(anyhow!("Network ID cannot be empty"));
    }

    if network_id.len() > 64 {
        return Err(anyhow!("Network ID too long (max 64 characters)"));
    }

    // Allow alphanumeric, hyphens, and underscores
    if !network_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "Network ID can only contain alphanumeric characters, hyphens, and underscores"
        ));
    }

    Ok(())
}

/// Validates network name format and content
///
/// # Arguments  
/// * `name` - The network name to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, Err with description if invalid
pub fn validate_network_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Network name cannot be empty"));
    }

    if name.len() > 128 {
        return Err(anyhow!("Network name too long (max 128 characters)"));
    }

    // More permissive for display names but still safe
    if name.chars().any(|c| c.is_control()) {
        return Err(anyhow!("Network name cannot contain control characters"));
    }

    Ok(())
}

/// Validates commitment level
///
/// # Arguments
/// * `commitment` - The commitment level to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, Err with description if invalid
pub fn validate_commitment(commitment: &str) -> Result<()> {
    match commitment {
        "processed" | "confirmed" | "finalized" => Ok(()),
        _ => Err(anyhow!(
            "Invalid commitment level. Must be 'processed', 'confirmed', or 'finalized'"
        )),
    }
}

/// Sanitizes a string for safe logging (removes sensitive information)
///
/// # Arguments
/// * `input` - The string to sanitize
///
/// # Returns
/// * `String` - Sanitized string safe for logging
pub fn sanitize_for_logging(input: &str) -> String {
    // For URLs, only show scheme and host, hide path/params
    if let Ok(url) = Url::parse(input) {
        if let Some(host) = url.host_str() {
            return format!("{}://{}", url.scheme(), host);
        }
    }

    // For other strings, truncate if too long
    if input.len() > 100 {
        format!("{}...[truncated]", &input[..100])
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_rpc_url_valid_https() {
        assert!(validate_rpc_url("https://api.opensvm.com").is_ok());
        assert!(validate_rpc_url("https://api.mainnet-beta.solana.com").is_ok());
    }

    #[test]
    fn test_validate_rpc_url_rejects_http() {
        assert!(validate_rpc_url("http://api.opensvm.com").is_err());
    }

    #[test]
    fn test_validate_rpc_url_rejects_invalid() {
        assert!(validate_rpc_url("not-a-url").is_err());
        assert!(validate_rpc_url("").is_err());
        assert!(validate_rpc_url("https://").is_err());
    }

    #[test]
    fn test_validate_network_id() {
        assert!(validate_network_id("solana-mainnet").is_ok());
        assert!(validate_network_id("test_network_123").is_ok());
        assert!(validate_network_id("").is_err());
        assert!(validate_network_id("network with spaces").is_err());
        assert!(validate_network_id(&"x".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_network_name() {
        assert!(validate_network_name("Solana Mainnet").is_ok());
        assert!(validate_network_name("Test Network 123").is_ok());
        assert!(validate_network_name("").is_err());
        assert!(validate_network_name(&"x".repeat(129)).is_err());
    }

    #[test]
    fn test_validate_commitment() {
        assert!(validate_commitment("processed").is_ok());
        assert!(validate_commitment("confirmed").is_ok());
        assert!(validate_commitment("finalized").is_ok());
        assert!(validate_commitment("invalid").is_err());
    }

    #[test]
    fn test_sanitize_for_logging() {
        let url = "https://api.opensvm.com/v1/accounts/abc123?encoding=json";
        let sanitized = sanitize_for_logging(url);
        assert_eq!(sanitized, "https://api.opensvm.com");

        let long_string = "x".repeat(150);
        let sanitized_long = sanitize_for_logging(&long_string);
        assert!(sanitized_long.len() <= 115); // 100 + "...[truncated]"
    }

    #[test]
    fn test_is_internal_address() {
        assert!(is_internal_address("localhost"));
        assert!(is_internal_address("127.0.0.1"));
        assert!(is_internal_address("192.168.1.1"));
        assert!(is_internal_address("10.0.0.1"));
        assert!(is_internal_address("172.16.0.1"));
        assert!(!is_internal_address("api.opensvm.com"));
        assert!(!is_internal_address("172.15.0.1"));
        assert!(!is_internal_address("172.32.0.1"));
    }
}
