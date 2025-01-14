use crate::SolanaMcpServer;
use anyhow::Result;
use mcp_sdk::types::{CallToolRequest, ToolResponseContent};
use serde_json::json;
use std::collections::HashMap;
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
    let request = CallToolRequest {
        name: "get_slot".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    let current_slot = match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"].as_str().unwrap().parse::<u64>().unwrap()
        }
        _ => panic!("Expected Text response"),
    };

    // Test get_slot_leaders using current slot
    let request = CallToolRequest {
        name: "get_slot_leaders".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("start_slot".to_string(), json!(current_slot));
            args.insert("limit".to_string(), json!(10));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            assert!(!text.is_empty());
        }
        _ => panic!("Expected Text response"),
    }

    // Test System Information
    let request = CallToolRequest {
        name: "get_health".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert_eq!(json["result"], "ok");
        }
        _ => panic!("Expected Text response"),
    }

    // Test Account Information
    let request = CallToolRequest {
        name: "get_balance".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("pubkey".to_string(), json!("11111111111111111111111111111111"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].as_str().unwrap().parse::<u64>().is_ok());
        }
        _ => panic!("Expected Text response"),
    }

    // Test Block Information
    let request = CallToolRequest {
        name: "get_block_height".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].as_str().unwrap().parse::<u64>().is_ok());
        }
        _ => panic!("Expected Text response"),
    }

    // Test Account Information - Extended
    let request = CallToolRequest {
        name: "get_account_info".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("pubkey".to_string(), json!("11111111111111111111111111111111"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());

    // Test System Information - Extended
    let request = CallToolRequest {
        name: "get_version".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());

    let request = CallToolRequest {
        name: "get_identity".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());

    // Test Epoch and Inflation
    let request = CallToolRequest {
        name: "get_epoch_info".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());

    let request = CallToolRequest {
        name: "get_inflation_rate".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());

    // Test Token Information
    let request = CallToolRequest {
        name: "get_token_accounts_by_owner".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("ownerAddress".to_string(), json!("11111111111111111111111111111111"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());

    // Test Error Cases
    let request = CallToolRequest {
        name: "invalid_tool".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());

    let request = CallToolRequest {
        name: "get_balance".into(),
        arguments: Some(HashMap::new()),
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());

    Ok(())
}
