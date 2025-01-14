use crate::SolanaMcpServer;
use anyhow::Result;
use mcp_sdk::types::{CallToolRequest, ToolResponseContent};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::collections::HashMap;

async fn setup() -> SolanaMcpServer {
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    SolanaMcpServer::new(client)
}

#[tokio::test]
async fn test_get_account_info() -> Result<()> {
    let server = setup().await;
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
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_object());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_balance() -> Result<()> {
    let server = setup().await;
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
    Ok(())
}

#[tokio::test]
async fn test_get_multiple_accounts() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_multiple_accounts".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("pubkeys".to_string(), json!(["11111111111111111111111111111111"]));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_array());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_program_accounts() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_program_accounts".into(),
        arguments: {
            let mut args = HashMap::new();
            // Use the Stake Program which should have accounts
            args.insert("programId".to_string(), json!("Stake11111111111111111111111111111111111111"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            // Just verify it's an array - may be empty depending on network state
            assert!(json["result"].is_array());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_invalid_account() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_balance".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("pubkey".to_string(), json!("invalid_address"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());
    Ok(())
}
