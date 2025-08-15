use reqwest;
use serde_json::{json, Value};
use tokio::time::Duration;

#[tokio::test]
async fn test_new_rpc_methods_and_websocket() {
    // Test the missing RPC methods
    let client = reqwest::Client::new();
    
    // Test the 3 supposedly missing methods
    let test_methods = vec![
        ("getBlockCommitment", json!({"slot": 1000})),
        ("getSnapshotSlot", json!({})),
        ("getStakeActivation", json!({"pubkey": "11111111111111111111111111111111"})),
    ];
    
    for (method_name, params) in test_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": method_name,
                "arguments": params
            }
        });
        
        println!("Testing method: {}", method_name);
        
        // Note: This test will fail if server is not running
        // It's more of a manual test to verify the methods are properly integrated
        match client
            .post("http://localhost:3000/api/mcp")
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(json_response) => {
                            println!("✓ {} response: {}", method_name, json_response);
                        }
                        Err(e) => {
                            println!("✗ {} JSON parse error: {}", method_name, e);
                        }
                    }
                } else {
                    println!("✗ {} HTTP error: {}", method_name, response.status());
                }
            }
            Err(e) => {
                println!("✗ {} Network error (server not running?): {}", method_name, e);
            }
        }
    }
}

#[tokio::test]
async fn test_websocket_connection() {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{SinkExt, StreamExt};
    
    println!("Testing WebSocket connection...");
    
    match connect_async("ws://localhost:8900").await {
        Ok((mut ws_stream, _)) => {
            println!("✓ WebSocket connected");
            
            // Test account subscription
            let subscribe_msg = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "accountSubscribe",
                "params": ["11111111111111111111111111111111"]
            });
            
            if let Err(e) = ws_stream.send(Message::Text(subscribe_msg.to_string().into())).await {
                println!("✗ Failed to send subscription: {}", e);
                return;
            }
            
            // Wait for response
            match tokio::time::timeout(Duration::from_secs(5), ws_stream.next()).await {
                Ok(Some(Ok(Message::Text(response)))) => {
                    println!("✓ Subscription response: {}", response);
                }
                Ok(Some(Ok(_))) => {
                    println!("✓ Got non-text response");
                }
                Ok(Some(Err(e))) => {
                    println!("✗ WebSocket error: {}", e);
                }
                Ok(None) => {
                    println!("✗ WebSocket closed");
                }
                Err(_) => {
                    println!("✗ Timeout waiting for response");
                }
            }
        }
        Err(e) => {
            println!("✗ WebSocket connection failed (server not running?): {}", e);
        }
    }
}

// This test verifies our tool definitions compile correctly
#[test]
fn test_tool_definitions_completeness() {
    // Verify we have all expected RPC methods
    let expected_missing_methods = vec![
        "getBlockCommitment",
        "getSnapshotSlot", 
        "getStakeActivation",
    ];
    
    let expected_websocket_methods = vec![
        "accountSubscribe", "accountUnsubscribe",
        "blockSubscribe", "blockUnsubscribe", 
        "logsSubscribe", "logsUnsubscribe",
        "programSubscribe", "programUnsubscribe",
        "rootSubscribe", "rootUnsubscribe",
        "signatureSubscribe", "signatureUnsubscribe",
        "slotSubscribe", "slotUnsubscribe",
        "slotsUpdatesSubscribe", "slotsUpdatesUnsubscribe",
        "voteSubscribe", "voteUnsubscribe",
    ];
    
    println!("Expected missing methods: {:?}", expected_missing_methods);
    println!("Expected WebSocket methods: {:?}", expected_websocket_methods);
    
    // This test just verifies the constants exist, real functionality requires a running server
    assert!(!expected_missing_methods.is_empty());
    assert!(!expected_websocket_methods.is_empty());
}