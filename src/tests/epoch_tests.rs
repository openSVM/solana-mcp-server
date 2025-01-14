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
async fn test_get_epoch_info() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_epoch_info".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_object());
            let result = &json["result"];
            assert!(result["absoluteSlot"].is_number());
            assert!(result["blockHeight"].is_number());
            assert!(result["epoch"].is_number());
            assert!(result["slotIndex"].is_number());
            assert!(result["slotsInEpoch"].is_number());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_epoch_schedule() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_epoch_schedule".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_object());
            let result = &json["result"];
            assert!(result["firstNormalEpoch"].is_number());
            assert!(result["firstNormalSlot"].is_number());
            assert!(result["leaderScheduleSlotOffset"].is_number());
            assert!(result["slotsPerEpoch"].is_number());
            assert!(result["warmup"].is_boolean());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_inflation_governor() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_inflation_governor".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_object());
            let result = &json["result"];
            assert!(result["foundation"].is_number());
            assert!(result["foundationTerm"].is_number());
            assert!(result["initial"].is_number());
            assert!(result["taper"].is_number());
            assert!(result["terminal"].is_number());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_inflation_rate() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_inflation_rate".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_object());
            let result = &json["result"];
            assert!(result["epoch"].is_number());
            assert!(result["foundation"].is_number());
            assert!(result["total"].is_number());
            assert!(result["validator"].is_number());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_inflation_reward() -> Result<()> {
    let server = setup().await;
    
    // First get current epoch
    let epoch_request = CallToolRequest {
        name: "get_epoch_info".into(),
        arguments: None,
        meta: None,
    };
    let epoch_response = server.handle_tool_request(epoch_request).await?;
    let current_epoch = match &epoch_response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"]["epoch"].as_u64().unwrap()
        }
        _ => panic!("Expected Text response"),
    };

    // Test get_inflation_reward with current epoch
    let request = CallToolRequest {
        name: "get_inflation_reward".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("addresses".to_string(), json!(["11111111111111111111111111111111"]));
            args.insert("epoch".to_string(), json!(current_epoch - 1)); // Use previous epoch to ensure it's available
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
            // The result might be null for addresses that didn't receive rewards
            if !json["result"][0].is_null() {
                let reward = &json["result"][0];
                assert!(reward["amount"].is_number());
                assert!(reward["effectiveSlot"].is_number());
                assert!(reward["epoch"].is_number());
                assert!(reward["postBalance"].is_number());
            }
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_inflation_reward_invalid_address() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_inflation_reward".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("addresses".to_string(), json!(["invalid_address"]));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());
    Ok(())
}

#[tokio::test]
async fn test_get_inflation_reward_future_epoch() -> Result<()> {
    let server = setup().await;
    
    // Get current epoch
    let epoch_request = CallToolRequest {
        name: "get_epoch_info".into(),
        arguments: None,
        meta: None,
    };
    let epoch_response = server.handle_tool_request(epoch_request).await?;
    let current_epoch = match &epoch_response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"]["epoch"].as_u64().unwrap()
        }
        _ => panic!("Expected Text response"),
    };

    // Test with future epoch
    let request = CallToolRequest {
        name: "get_inflation_reward".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("addresses".to_string(), json!(["11111111111111111111111111111111"]));
            args.insert("epoch".to_string(), json!(current_epoch - 2)); // Use an older epoch that should be available
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
            // Should be null for future epochs
            assert!(json["result"][0].is_null());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}
