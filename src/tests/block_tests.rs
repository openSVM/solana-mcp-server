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
async fn test_get_block() -> Result<()> {
    let server = setup().await;
    
    // First get first available block
    let first_block_request = CallToolRequest {
        name: "get_first_available_block".into(),
        arguments: None,
        meta: None,
    };
    let first_block_response = server.handle_tool_request(first_block_request).await?;
    let first_block = match &first_block_response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"].as_u64().unwrap()
        }
        _ => panic!("Expected Text response"),
    };

    // Then get block info for first available block
    let request = CallToolRequest {
        name: "get_block".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("slot".to_string(), json!(first_block));
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
async fn test_get_blocks() -> Result<()> {
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
        name: "get_blocks".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("start_slot".to_string(), json!(current_slot - 2));
            args.insert("end_slot".to_string(), json!(current_slot - 1));
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
async fn test_get_blocks_with_limit() -> Result<()> {
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
        name: "get_blocks_with_limit".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("start_slot".to_string(), json!(current_slot - 2));
            args.insert("limit".to_string(), json!(5));
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
            assert!(json["result"].as_array().unwrap().len() <= 5);
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_block_time() -> Result<()> {
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
        name: "get_block_time".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("slot".to_string(), json!(current_slot - 1));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_number());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_block_height() -> Result<()> {
    let server = setup().await;
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
    Ok(())
}

#[tokio::test]
async fn test_get_block_production() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_block_production".into(),
        arguments: None,
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
async fn test_get_first_available_block() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_first_available_block".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].as_u64().is_some());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_invalid_block() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_block".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("slot".to_string(), json!(u64::MAX));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());
    Ok(())
}
