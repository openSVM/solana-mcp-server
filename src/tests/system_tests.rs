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
async fn test_get_health() -> Result<()> {
    let server = setup().await;
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
    Ok(())
}

#[tokio::test]
async fn test_get_version() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_version".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_object());
            assert!(json["result"]["solana-core"].is_string());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_identity() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_identity".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_string());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_genesis_hash() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_genesis_hash".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_string());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_slot() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_slot".into(),
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
    Ok(())
}

#[tokio::test]
async fn test_get_slot_leader() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_slot_leader".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_string());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_slot_leaders() -> Result<()> {
    let server = setup().await;
    
    // First get current slot
    let slot_request = CallToolRequest {
        name: "get_slot".into(),
        arguments: None,
        meta: None,
    };
    let slot_response = server.handle_tool_request(slot_request).await?;
    let current_slot = match &slot_response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"].as_str().unwrap().parse::<u64>().unwrap()
        }
        _ => panic!("Expected Text response"),
    };

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
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_array());
            assert!(json["result"].as_array().unwrap().len() <= 10);
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_cluster_nodes() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_cluster_nodes".into(),
        arguments: None,
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
async fn test_get_vote_accounts() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_vote_accounts".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_object());
            assert!(json["result"]["current"].is_array());
            assert!(json["result"]["delinquent"].is_array());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_stake_minimum_delegation() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_stake_minimum_delegation".into(),
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
    Ok(())
}

#[tokio::test]
async fn test_get_supply() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_supply".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            let result = json.get("result").expect("Missing result field");
            assert!(result.is_object(), "Result should be an object");
            let value = result.get("value").expect("Missing value field");
            assert!(value.is_object(), "Value should be an object");
            assert!(value["total"].as_u64().is_some(), "Total should be a number");
            assert!(value["circulating"].as_u64().is_some(), "Circulating should be a number");
            assert!(value["nonCirculating"].as_u64().is_some(), "Non-circulating should be a number");
            assert!(value["nonCirculatingAccounts"].is_array(), "Non-circulating accounts should be an array");
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}
