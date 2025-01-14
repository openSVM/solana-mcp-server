use crate::SolanaMcpServer;
use anyhow::Result;
use mcp_sdk::types::{CallToolRequest, ToolResponseContent};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
    pubkey::Pubkey,
    system_instruction,
    transaction::Transaction,
    signature::{Keypair, Signer},
    hash::Hash,
};
use std::{str::FromStr, collections::HashMap};
use base64::{Engine as _, engine::general_purpose::STANDARD as base64};

async fn setup() -> SolanaMcpServer {
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    SolanaMcpServer::new(client)
}

#[tokio::test]
async fn test_get_transaction_count() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_transaction_count".into(),
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
async fn test_get_signatures_for_address() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_signatures_for_address".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("address".to_string(), json!("11111111111111111111111111111111"));
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
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_signature_statuses() -> Result<()> {
    let server = setup().await;
    // First get some valid signatures
    let request = CallToolRequest {
        name: "get_signatures_for_address".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("address".to_string(), json!("11111111111111111111111111111111"));
            args.insert("limit".to_string(), json!(1));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    let signatures = match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"].as_array().unwrap().iter()
                .map(|sig| sig["signature"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        }
        _ => panic!("Expected Text response"),
    };

    // Now test get_signature_statuses
    let request = CallToolRequest {
        name: "get_signature_statuses".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("signatures".to_string(), json!(signatures));
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
            assert!(json["result"]["value"].is_array());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_simulate_transaction() -> Result<()> {
    let server = setup().await;
    
    // Create a test transaction
    let payer = Keypair::new();
    let to = Pubkey::new_unique();
    let instruction = system_instruction::transfer(&payer.pubkey(), &to, 1000);
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    
    // Get latest blockhash
    let blockhash_request = CallToolRequest {
        name: "get_latest_blockhash".into(),
        arguments: None,
        meta: None,
    };
    let blockhash_response = server.handle_tool_request(blockhash_request).await?;
    let blockhash = match &blockhash_response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"].as_str().unwrap().to_string()
        }
        _ => panic!("Expected Text response"),
    };
    let blockhash = Hash::from_str(&blockhash)?;
    
    let tx = Transaction::new(&[&payer], message, blockhash);
    let tx_bytes = bincode::serialize(&tx)?;
    let tx_b64 = base64.encode(&tx_bytes);

    let request = CallToolRequest {
        name: "simulate_transaction".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("transaction".to_string(), json!(tx_b64));
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
            assert!(json["result"]["err"].is_null()); // Should succeed since it's just a simulation
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_fee_for_message() -> Result<()> {
    let server = setup().await;
    
    // Get latest blockhash first
    let blockhash_request = CallToolRequest {
        name: "get_latest_blockhash".into(),
        arguments: None,
        meta: None,
    };
    let blockhash_response = server.handle_tool_request(blockhash_request).await?;
    let blockhash = match &blockhash_response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"].as_str().unwrap().to_string()
        }
        _ => panic!("Expected Text response"),
    };
    let blockhash = Hash::from_str(&blockhash)?;

    // Create a test message
    let payer = Keypair::new();
    let to = Pubkey::new_unique();
    let instruction = system_instruction::transfer(&payer.pubkey(), &to, 1000);
    let message = Message::new_with_blockhash(
        &[instruction],
        Some(&payer.pubkey()),
        &blockhash
    );
    let message_bytes = bincode::serialize(&message)?;
    let message_b64 = base64.encode(&message_bytes);

    let request = CallToolRequest {
        name: "get_fee_for_message".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("message".to_string(), json!(message_b64));
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
async fn test_get_latest_blockhash() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_latest_blockhash".into(),
        arguments: None,
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_string());
            // Verify it's a valid hash string
            assert!(Hash::from_str(json["result"].as_str().unwrap()).is_ok());
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_is_blockhash_valid() -> Result<()> {
    let server = setup().await;
    
    // Get latest blockhash first
    let blockhash_request = CallToolRequest {
        name: "get_latest_blockhash".into(),
        arguments: None,
        meta: None,
    };
    let blockhash_response = server.handle_tool_request(blockhash_request).await?;
    let blockhash = match &blockhash_response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            json["result"].as_str().unwrap().to_string()
        }
        _ => panic!("Expected Text response"),
    };

    let request = CallToolRequest {
        name: "is_blockhash_valid".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("blockhash".to_string(), json!(blockhash));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            assert!(json["result"].is_boolean());
            // Don't assert the actual value since it may expire quickly
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_invalid_transaction() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "simulate_transaction".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("transaction".to_string(), json!("invalid_transaction"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());
    Ok(())
}
