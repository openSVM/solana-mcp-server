use crate::SolanaMcpServer;
use anyhow::Result;
use mcp_sdk::{
    transport::JsonRpcRequest,
    types::{CallToolResponse, ToolResponseContent},
};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

#[tokio::test]
async fn test_server_e2e() -> Result<()> {
    // Create server
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    let server = SolanaMcpServer::new(client);

    // Test Slot Information
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 1,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_slot",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    let current_slot = if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                text.parse::<u64>().unwrap()
            }
            _ => panic!("Expected Text response"),
        }
    } else {
        panic!("Expected result");
    };

    // Test get_slot_leaders using current slot
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 2,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_slot_leaders",
            "arguments": {
                "start_slot": current_slot,
                "limit": 10
            },
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test System Information
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 3,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_health",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert_eq!(text, "ok");
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test get_version
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 4,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_version",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test get_genesis_hash
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 5,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_genesis_hash",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test Account Information
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 6,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_balance",
            "arguments": {
                "pubkey": "11111111111111111111111111111111"
            },
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert!(text.parse::<u64>().is_ok());
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test Epoch Information
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 7,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_epoch_info",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test Validator Information
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 8,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_vote_accounts",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test Token Information
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 9,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_token_accounts_by_owner",
            "arguments": {
                "owner": "11111111111111111111111111111111"
            },
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Text { text } => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected Text response"),
        }
    }

    // Test Resource Endpoints
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 10,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_supply",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await?;
    assert!(response.error.is_none());
    if let Some(result) = response.result {
        let response: CallToolResponse = serde_json::from_value(result)?;
        assert!(!response.content.is_empty());
        match &response.content[0] {
            ToolResponseContent::Resource { resource } => {
                assert_eq!(resource.uri.as_str(), "solana://supply");
                assert_eq!(resource.mime_type.as_ref().unwrap(), "application/json");
            }
            _ => panic!("Expected Resource response"),
        }
    }

    // Test Error Cases
    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 11,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "invalid_tool",
            "arguments": null,
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.is_err());

    let request = JsonRpcRequest {
        jsonrpc: Default::default(),
        id: 12,
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "get_balance",
            "arguments": {},
            "meta": null
        })),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.is_err());

    Ok(())
}
