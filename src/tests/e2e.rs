use super::*;
use mcp_sdk::{
    transport::{
        StdioTransport, JsonRpcMessage, Transport, JsonRpcRequest, JsonRpcResponse,
        ReadResourceRequest, JsonRpcVersion,
    },
    types::{
        CallToolRequest, CallToolResponse, ToolResponseContent,
        ResourceContents,
    },
};
use serde_json::json;

#[tokio::test]
async fn test_handle_read_resource() {
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    let server = SolanaMcpServer::new(client);

    // Test supply resource
    let request = ReadResourceRequest {
        uri: "solana://supply".parse().unwrap(),
    };
    let result = server.handle_read_resource(request).await;
    assert!(result.is_ok());

    // Test inflation resource
    let request = ReadResourceRequest {
        uri: "solana://inflation".parse().unwrap(),
    };
    let result = server.handle_read_resource(request).await;
    assert!(result.is_ok());

    // Test invalid resource
    let request = ReadResourceRequest {
        uri: "solana://invalid".parse().unwrap(),
    };
    let result = server.handle_read_resource(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_tool_request() {
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    let server = SolanaMcpServer::new(client);

    // Test get_slot
    let request = CallToolRequest {
        name: "get_slot".to_string(),
        arguments: None,
    };
    let result = server.handle_tool_request(request).await;
    assert!(result.is_ok());

    // Test get_health
    let request = CallToolRequest {
        name: "get_health".to_string(),
        arguments: None,
    };
    let result = server.handle_tool_request(request).await;
    assert!(result.is_ok());

    // Test get_version
    let request = CallToolRequest {
        name: "get_version".to_string(),
        arguments: None,
    };
    let result = server.handle_tool_request(request).await;
    assert!(result.is_ok());

    // Test invalid tool
    let request = CallToolRequest {
        name: "invalid_tool".to_string(),
        arguments: None,
    };
    let result = server.handle_tool_request(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transport_implementation() {
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    let server = SolanaMcpServer::new(client);

    // Test send
    let message = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::Two,
        id: 1,
        method: "test".to_string(),
        params: Some(json!({})),
    });
    let result = server.send(&message).await;
    assert!(result.is_ok());

    // Test receive
    let result = server.receive().await;
    assert!(result.is_ok());

    // Test open
    let result = server.open().await;
    assert!(result.is_ok());

    // Test close
    let result = server.close().await;
    assert!(result.is_ok());
}
