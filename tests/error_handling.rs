/// Integration tests for error handling and logging functionality
use solana_mcp_server::{
    error::{McpError, McpResult},
    logging::{init_logging, get_metrics},
};
use uuid::Uuid;

#[tokio::test]
async fn test_error_handling_integration() {
    // Initialize logging for test
    let _ = init_logging(Some("debug"));
    
    // Reset metrics for clean test
    get_metrics().reset();
    
    // Test successful operation logging
    let result = test_successful_operation().await;
    assert!(result.is_ok());
    
    // Test error operation logging
    let result = test_error_operation().await;
    assert!(result.is_err());
    
    // Verify metrics were updated
    let metrics = get_metrics().to_json();
    assert!(metrics["total_calls"].as_u64().unwrap() >= 2);
    assert!(metrics["successful_calls"].as_u64().unwrap() >= 1);
    
    // Verify error categorization in metrics
    let failed_by_type = &metrics["failed_calls_by_type"];
    assert!(failed_by_type["validation"].as_u64().unwrap_or(0) >= 1);
}

async fn test_successful_operation() -> McpResult<()> {
    let request_id = Uuid::new_v4();
    
    // Simulate a successful operation
    solana_mcp_server::logging::log_rpc_request_start(
        request_id,
        "testOperation",
        Some("https://test.example.com"),
        Some("test parameters"),
    );
    
    // Simulate processing time
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    solana_mcp_server::logging::log_rpc_request_success(
        request_id,
        "testOperation",
        10,
        Some("test completed"),
    );
    
    Ok(())
}

async fn test_error_operation() -> McpResult<()> {
    let request_id = Uuid::new_v4();
    
    // Simulate an error operation
    solana_mcp_server::logging::log_rpc_request_start(
        request_id,
        "testErrorOperation",
        Some("https://test.example.com"),
        Some("test parameters"),
    );
    
    // Create a validation error
    let error = McpError::validation("Test validation error")
        .with_request_id(request_id)
        .with_method("testErrorOperation")
        .with_parameter("testParam");
    
    solana_mcp_server::logging::log_rpc_request_failure(
        request_id,
        "testErrorOperation",
        error.error_type(),
        5,
        Some(&error.to_log_value()),
    );
    
    Err(error)
}

#[test]
fn test_error_message_safety() {
    // Test that server errors don't leak sensitive information
    let server_error = McpError::server("Database connection failed with password: secret123");
    assert_eq!(server_error.safe_message(), "Internal server error");
    
    // Test that validation errors are shown to help debugging
    let validation_error = McpError::validation("Invalid pubkey format");
    assert_eq!(validation_error.safe_message(), "Invalid pubkey format");
    
    // Test that auth errors return generic messages
    let auth_error = McpError::auth("Invalid API key: abc123");
    assert_eq!(auth_error.safe_message(), "Authentication required");
}

#[test]
fn test_error_json_rpc_codes() {
    assert_eq!(McpError::client("test").json_rpc_code(), -32602);
    assert_eq!(McpError::validation("test").json_rpc_code(), -32602);
    assert_eq!(McpError::auth("test").json_rpc_code(), -32601);
    assert_eq!(McpError::server("test").json_rpc_code(), -32603);
    assert_eq!(McpError::rpc("test").json_rpc_code(), -32603);
    assert_eq!(McpError::network("test").json_rpc_code(), -32603);
}

#[test]
fn test_error_context_chaining() {
    let request_id = Uuid::new_v4();
    
    let error = McpError::rpc("Connection timeout")
        .with_request_id(request_id)
        .with_method("getBalance")
        .with_rpc_url("https://api.mainnet-beta.solana.com");
    
    assert_eq!(error.request_id(), Some(request_id));
    assert_eq!(error.method(), Some("getBalance"));
    assert_eq!(error.error_type(), "rpc");
    
    // Test log value contains expected fields
    let log_value = error.to_log_value();
    assert!(log_value["error_type"].as_str().unwrap() == "rpc");
    assert!(log_value["request_id"].is_string());
    assert!(log_value["method"].as_str().unwrap() == "getBalance");
    assert!(log_value["rpc_url"].is_string());
}