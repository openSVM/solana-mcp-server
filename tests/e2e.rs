use std::time::Duration;
use serde_json::{json, Value};
use solana_mcp_server::{Config, ServerState, start_mcp_server_task};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Comprehensive end-to-end tests for the MCP JSON-RPC API
/// 
/// These tests start an actual HTTP server and make real HTTP requests
/// to test the complete MCP protocol implementation

const TEST_PORT: u16 = 8888;
const TEST_SERVER_URL: &str = "http://localhost:8888";

/// Test setup helper that starts the MCP HTTP server
async fn setup_test_server() -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration
    let config = Config::load().map_err(|e| format!("Failed to load config: {}", e))?;
    
    // Create server state
    let server_state = ServerState::new(config);
    let state = Arc::new(RwLock::new(server_state));
    
    // Start HTTP server with MCP API
    let handle = start_mcp_server_task(TEST_PORT, state);
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    Ok(handle)
}

/// Helper function to make HTTP requests to the MCP API
async fn make_mcp_request(request: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/mcp", TEST_SERVER_URL))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
        
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let json_response: Value = response.json().await?;
    Ok(json_response)
}

/// Test 1: Basic server connectivity and health check
#[tokio::test]
async fn test_basic_connectivity() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Test health endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health", TEST_SERVER_URL))
        .send()
        .await
        .expect("Failed to connect to health endpoint");
        
    assert!(response.status().is_success());
    
    let health_json: Value = response.json().await.expect("Failed to parse health response");
    assert_eq!(health_json["status"], "ok");
    assert_eq!(health_json["service"], "solana-mcp-server");
    assert!(health_json["capabilities"]["tools"].as_bool().unwrap_or(false));
}

/// Test 2: MCP Initialize Protocol
#[tokio::test]
async fn test_mcp_initialize_protocol() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Test initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "e2e-test-client",
                "version": "1.0.0"
            }
        }
    });
    
    let response = make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Validate response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert_eq!(result["serverInfo"]["name"], "solana-mcp-server");
    assert!(result["capabilities"]["tools"].is_object());
}

/// Test 3: Tools List Endpoint
#[tokio::test] 
async fn test_tools_list() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // First initialize
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Now test tools/list
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    
    let response = make_mcp_request(tools_request).await.expect("Failed to get tools list");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    
    let tools = response["result"]["tools"].as_array().expect("Tools should be an array");
    assert!(!tools.is_empty(), "Should have at least one tool");
    
    // Verify some expected tools exist
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
        
    assert!(tool_names.contains(&"getBalance"));
    assert!(tool_names.contains(&"getAccountInfo"));
    assert!(tool_names.contains(&"getHealth"));
    assert!(tool_names.contains(&"getVersion"));
}

/// Test 4: Tool Execution - getHealth
#[tokio::test]
async fn test_tool_execution_get_health() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Execute getHealth tool
    let tool_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "getHealth",
            "arguments": {}
        }
    });
    
    let response = make_mcp_request(tool_request).await.expect("Failed to call getHealth tool");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"].is_object());
}

/// Test 5: Tool Execution - getBalance with System Program
#[tokio::test] 
async fn test_tool_execution_get_balance() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Execute getBalance tool for System Program
    let tool_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "getBalance",
            "arguments": {
                "pubkey": "11111111111111111111111111111112"
            }
        }
    });
    
    let response = make_mcp_request(tool_request).await.expect("Failed to call getBalance tool");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"].is_object());
}

/// Test 6: Error Handling - Invalid JSON-RPC
#[tokio::test]
async fn test_error_handling_invalid_jsonrpc() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Send invalid JSON-RPC request (missing jsonrpc field)
    let invalid_request = json!({
        "id": 1,
        "method": "initialize"
    });
    
    let response = make_mcp_request(invalid_request).await.expect("Should get error response");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], Value::Null);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32600);
    assert!(response["error"]["message"].as_str().unwrap().contains("jsonrpc"));
}

/// Test 7: Error Handling - Wrong Protocol Version
#[tokio::test]
async fn test_error_handling_wrong_protocol_version() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Send initialize with wrong protocol version
    let wrong_version_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "1.0.0",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    let response = make_mcp_request(wrong_version_request).await.expect("Should get error response");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32002);
    assert!(response["error"]["message"].as_str().unwrap().contains("Protocol version mismatch"));
}

/// Test 8: Error Handling - Server Not Initialized
#[tokio::test]
async fn test_error_handling_not_initialized() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Try to call a tool without initializing first
    let tool_request = json!({
        "jsonrpc": "2.0", 
        "id": 1,
        "method": "tools/list"
    });
    
    let response = make_mcp_request(tool_request).await.expect("Should get error response");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32002);
    assert!(response["error"]["message"].as_str().unwrap().contains("Server not initialized"));
}

/// Test 9: Error Handling - Method Not Found
#[tokio::test]
async fn test_error_handling_method_not_found() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Try to call non-existent method
    let invalid_method_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "nonexistent/method"
    });
    
    let response = make_mcp_request(invalid_method_request).await.expect("Should get error response");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601);
    assert!(response["error"]["message"].as_str().unwrap().contains("Method not found"));
}

/// Test 10: Error Handling - Invalid Tool Parameters
#[tokio::test]
async fn test_error_handling_invalid_tool_params() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Call getBalance without required pubkey parameter
    let tool_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "getBalance",
            "arguments": {}
        }
    });
    
    let response = make_mcp_request(tool_request).await.expect("Should get error response");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["error"].is_object());
    // Should be internal error since parameter validation fails
    assert_eq!(response["error"]["code"], -32603);
}

/// Test 11: Content-Type Validation
#[tokio::test]
async fn test_content_type_validation() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    let client = reqwest::Client::new();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    // Send request with incorrect content type
    let response = client
        .post(&format!("{}/api/mcp", TEST_SERVER_URL))
        .header("Content-Type", "text/plain")
        .body(serde_json::to_string(&request).unwrap())
        .send()
        .await
        .expect("Failed to send request");
        
    // Should return an error response (200 OK but with JSON-RPC error)
    if response.status().is_success() {
        let json_response: Value = response.json().await.expect("Failed to parse response");
        assert!(json_response["error"].is_object());
        assert_eq!(json_response["error"]["code"], -32600);
        assert!(json_response["error"]["message"].as_str().unwrap().contains("Content-Type"));
    } else {
        // Server rejected the request due to content type - that's also valid behavior
        assert!(!response.status().is_success());
    }
}

/// Test 12: Metrics Endpoint
#[tokio::test]
async fn test_metrics_endpoint() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/metrics", TEST_SERVER_URL))
        .send()
        .await
        .expect("Failed to connect to metrics endpoint");
        
    let status = response.status();
    let _metrics_text = response.text().await.expect("Failed to get metrics text");
    
    // Verify the metrics endpoint responds successfully
    assert!(status.is_success());
}

/// Test 13: Resources List
#[tokio::test]
async fn test_resources_list() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Test resources/list
    let resources_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/list"
    });
    
    let response = make_mcp_request(resources_request).await.expect("Failed to get resources list");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"]["resources"].is_array());
}

/// Test 14: Complex Tool - Multiple Account Info
#[tokio::test]
async fn test_complex_tool_multiple_accounts() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Test getMultipleAccounts with System Program and SPL Token Program
    let tool_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "getMultipleAccounts",
            "arguments": {
                "pubkeys": [
                    "11111111111111111111111111111111",  // System Program
                    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"   // SPL Token Program
                ]
            }
        }
    });
    
    let response = make_mcp_request(tool_request).await.expect("Failed to call getMultipleAccounts tool");
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"].is_object());
}

/// Test 15: Concurrent Requests
#[tokio::test]
async fn test_concurrent_requests() {
    let _server_handle = setup_test_server().await.expect("Failed to start test server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0.0"}
        }
    });
    
    make_mcp_request(init_request).await.expect("Failed to initialize");
    
    // Make multiple concurrent requests
    let mut tasks = Vec::new();
    
    for i in 0..5 {
        let task = tokio::spawn(async move {
            let request = json!({
                "jsonrpc": "2.0",
                "id": i + 2,
                "method": "tools/call",
                "params": {
                "name": "getHealth",
                    "arguments": {}
                }
            });
            
            make_mcp_request(request).await
        });
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        let response = task.await.expect("Task panicked").expect("Request failed");
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
    }
}

// Legacy Solana operations test (kept for backward compatibility)
#[tokio::test]
async fn test_solana_operations_legacy() {
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signer::{keypair::Keypair, Signer},
    };

    // Connect to Solana devnet
    let rpc_url = "https://api.opensvm.com".to_string();
    let timeout = std::time::Duration::from_secs(60);
    let commitment = CommitmentConfig::finalized();
    let client = RpcClient::new_with_timeout_and_commitment(rpc_url.clone(), timeout, commitment);

    println!("\nTesting health check:");
    match client.get_health().await {
        Ok(health) => println!("Health status: {:?}", health),
        Err(err) => {
            println!("Error details: {:?}", err);
            // Don't panic in CI, just log the error
            println!("Health check failed: {}", err);
            return;
        }
    }

    println!("\nTesting version info:");
    let version = client.get_version().await.unwrap();
    println!("Version info: {:?}", version);

    println!("\nTesting latest blockhash:");
    let blockhash = client.get_latest_blockhash().await.unwrap();
    println!("Latest blockhash: {:?}", blockhash);

    println!("\nTesting transaction count:");
    let count = client.get_transaction_count().await.unwrap();
    println!("Transaction count: {}", count);

    // Get info about the System Program
    println!("\nTesting account info for System Program:");
    let system_program_id = "11111111111111111111111111111111"
        .parse::<Pubkey>()
        .unwrap();
    let account = client.get_account(&system_program_id).await.unwrap();
    println!("System Program Account:");
    println!("  Owner: {}", account.owner);
    println!("  Lamports: {}", account.lamports);
    println!("  Executable: {}", account.executable);

    // Get recent confirmed signatures first
    println!("\nTesting recent transactions:");
    let signatures = client
        .get_signatures_for_address(&system_program_id)
        .await
        .unwrap();
    println!("Recent transactions for System Program:");
    for sig in signatures.iter().take(3) {
        println!("  Signature: {}", sig.signature);
        println!("  Slot: {}", sig.slot);
        if let Some(err) = &sig.err {
            println!("  Error: {:?}", err);
        }
    }

    // Test creating a new keypair and getting its info
    println!("\nTesting keypair operations:");
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    println!("Generated keypair with pubkey: {}", pubkey);

    // Get account info (should be empty/not found)
    match client.get_account(&pubkey).await {
        Ok(account) => println!("Account exists with {} lamports", account.lamports),
        Err(e) => println!("Account not found as expected: {}", e),
    }

    // Get minimum rent
    println!("\nTesting rent calculation:");
    let rent = client
        .get_minimum_balance_for_rent_exemption(0)
        .await
        .unwrap();
    println!("Minimum balance for rent exemption: {} lamports", rent);

    // Get recent block
    println!("\nTesting block info:");
    let slot = client.get_slot().await.unwrap();
    println!("Current slot: {}", slot);

    // Get block production
    println!("\nTesting block production:");
    let production = client.get_block_production().await.unwrap();
    println!("Block production: {:?}", production);

    // Get cluster nodes
    println!("\nTesting cluster info:");
    let nodes = client.get_cluster_nodes().await.unwrap();
    println!("Found {} cluster nodes", nodes.len());
    for node in nodes.iter().take(3) {
        let version = node.version.as_ref().map_or("unknown", |v| v.as_str());
        println!("  {}: {}", node.pubkey, version);
    }
}
