/// Integration tests for validation functionality
use solana_mcp_server::validation::*;

#[test]
fn test_comprehensive_url_validation() {
    // Valid HTTPS URLs
    assert!(validate_rpc_url("https://api.opensvm.com").is_ok());
    assert!(validate_rpc_url("https://api.mainnet-beta.solana.com").is_ok());
    assert!(validate_rpc_url("https://rpc.ankr.com/solana").is_ok());

    // Invalid protocols
    assert!(validate_rpc_url("http://api.opensvm.com").is_err());
    assert!(validate_rpc_url("ftp://api.opensvm.com").is_err());
    assert!(validate_rpc_url("ws://api.opensvm.com").is_err());

    // Test various malformed URLs
    assert!(validate_rpc_url("not-a-url").is_err());
    assert!(validate_rpc_url("").is_err());
    assert!(validate_rpc_url("https://").is_err()); // empty host
                                                    // Note: "https:///path" is parsed as host="path" which is technically valid

    // URL too long
    let long_url = format!("https://example.com/{}", "x".repeat(2100));
    assert!(validate_rpc_url(&long_url).is_err());
}

#[test]
fn test_network_id_validation() {
    // Valid network IDs
    assert!(validate_network_id("solana-mainnet").is_ok());
    assert!(validate_network_id("test_network_123").is_ok());
    assert!(validate_network_id("devnet").is_ok());
    assert!(validate_network_id("custom-rpc-1").is_ok());

    // Invalid network IDs
    assert!(validate_network_id("").is_err());
    assert!(validate_network_id("network with spaces").is_err());
    assert!(validate_network_id("network@special").is_err());
    assert!(validate_network_id("network/slash").is_err());
    assert!(validate_network_id(&"x".repeat(65)).is_err());
}

#[test]
fn test_network_name_validation() {
    // Valid network names
    assert!(validate_network_name("Solana Mainnet").is_ok());
    assert!(validate_network_name("Test Network 123").is_ok());
    assert!(validate_network_name("My Custom RPC").is_ok());
    assert!(validate_network_name("Network (Production)").is_ok());

    // Invalid network names
    assert!(validate_network_name("").is_err());
    assert!(validate_network_name(&"x".repeat(129)).is_err());
    assert!(validate_network_name("Network\nwith\nlinebreaks").is_err());
    assert!(validate_network_name("Network\twith\ttabs").is_err());
}

#[test]
fn test_commitment_validation() {
    // Valid commitments
    assert!(validate_commitment("processed").is_ok());
    assert!(validate_commitment("confirmed").is_ok());
    assert!(validate_commitment("finalized").is_ok());

    // Invalid commitments
    assert!(validate_commitment("").is_err());
    assert!(validate_commitment("invalid").is_err());
    assert!(validate_commitment("PROCESSED").is_err());
    assert!(validate_commitment("recent").is_err());
}

#[test]
fn test_sanitize_for_logging() {
    // URLs should be sanitized to hide sensitive paths
    let sensitive_url =
        "https://api.opensvm.com/v1/accounts/abc123?encoding=json&commitment=confirmed";
    let sanitized = sanitize_for_logging(sensitive_url);
    assert_eq!(sanitized, "https://api.opensvm.com");

    // Long strings should be truncated
    let long_string = "x".repeat(150);
    let sanitized_long = sanitize_for_logging(&long_string);
    assert!(sanitized_long.len() <= 115);
    assert!(sanitized_long.ends_with("...[truncated]"));

    // Short strings should remain unchanged
    let short_string = "short text";
    let sanitized_short = sanitize_for_logging(short_string);
    assert_eq!(sanitized_short, short_string);
}

#[test]
fn test_security_edge_cases() {
    // Test various injection attempts
    assert!(validate_rpc_url("https://evil.com\x00.api.opensvm.com").is_err());
    assert!(validate_network_id("network\x00id").is_err());
    assert!(validate_network_name("name\x00with\x00nulls").is_err());

    // Test boundary conditions
    assert!(validate_network_id(&"x".repeat(64)).is_ok());
    assert!(validate_network_id(&"x".repeat(65)).is_err());
    assert!(validate_network_name(&"x".repeat(128)).is_ok());
    assert!(validate_network_name(&"x".repeat(129)).is_err());
}
