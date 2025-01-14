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

// Using USDC token on devnet for testing
const TEST_TOKEN_MINT: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";
const TEST_TOKEN_OWNER: &str = "HXtBm8XZbxaTt41uqaKhwUAa6Z1aPyvJdsZVENiWsetg";

#[tokio::test]
async fn test_get_token_supply() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_token_supply".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("mint".to_string(), json!(TEST_TOKEN_MINT));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            let result = json.get("result").expect("Missing result field");
            assert!(result.is_object(), "Result should be an object");
            assert!(result["amount"].is_string(), "Amount field should be a string");
            assert!(result["decimals"].is_number(), "Decimals field should be a number");
            assert!(result["amount"].as_str().unwrap().parse::<u64>().is_ok(), "Amount should be parseable as u64");
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_token_accounts_by_owner() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_token_accounts_by_owner".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("ownerAddress".to_string(), json!(TEST_TOKEN_OWNER));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            let result = json.get("result").expect("Missing result field");
            assert!(result.is_array(), "Result should be an array");
            // Each account should have pubkey and account fields
            if let Some(account) = result.as_array().unwrap().first() {
                assert!(account.get("pubkey").is_some(), "Account should have pubkey field");
                assert!(account.get("account").is_some(), "Account should have account field");
            }
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_token_accounts_by_delegate() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_token_accounts_by_delegate".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("delegateAddress".to_string(), json!(TEST_TOKEN_OWNER));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            let result = json.get("result").expect("Missing result field");
            assert!(result.is_array(), "Result should be an array");
            // Each account should have pubkey and account fields if any exist
            if let Some(account) = result.as_array().unwrap().first() {
                assert!(account.get("pubkey").is_some(), "Account should have pubkey field");
                assert!(account.get("account").is_some(), "Account should have account field");
            }
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_get_token_largest_accounts() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_token_largest_accounts".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("mint".to_string(), json!(TEST_TOKEN_MINT));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await?;
    assert!(!response.content.is_empty());
    match &response.content[0] {
        ToolResponseContent::Text { text } => {
            let json: serde_json::Value = serde_json::from_str(text)?;
            let result = json.get("result").expect("Missing result field");
            assert!(result.is_array(), "Result should be an array");
            // Each account should have address and amount fields
            if let Some(account) = result.as_array().unwrap().first() {
                assert!(account.get("address").is_some(), "Account should have address field");
                assert!(account.get("amount").is_some(), "Account should have amount field");
                assert!(account.get("decimals").is_some(), "Account should have decimals field");
            }
        }
        _ => panic!("Expected Text response"),
    }
    Ok(())
}

#[tokio::test]
async fn test_invalid_token_account() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_token_account_balance".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("accountAddress".to_string(), json!("invalid_address"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());
    Ok(())
}

#[tokio::test]
async fn test_invalid_token_mint() -> Result<()> {
    let server = setup().await;
    let request = CallToolRequest {
        name: "get_token_supply".into(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("mint".to_string(), json!("invalid_mint"));
            Some(args)
        },
        meta: None,
    };
    let response = server.handle_tool_request(request).await;
    assert!(response.is_err());
    Ok(())
}
