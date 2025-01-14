use anyhow::Result;
use log::info;
use mcp_sdk::{
    transport::StdioTransport,
    protocol::ProtocolBuilder,
    server::{Server, ServerOptions},
    types::{CallToolRequest, CallToolResponse, ToolResponseContent, ServerCapabilities, ResourceCapabilities, ToolsListResponse, Tool},
};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_mcp_server::SolanaMcpServer;
use std::sync::Arc;
use tokio::runtime::Handle;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let client = RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    );

    let solana_server = Arc::new(SolanaMcpServer::new(client));
    let transport = StdioTransport::default();

    info!("Starting Solana MCP server...");
    
    let runtime = Handle::current();

    // Create server with capabilities
    let server = Server::new(
        ProtocolBuilder::new(transport)
            .notification_handler("notifications/initialized", {
                move |params: serde_json::Value| {
                    info!("MCP server initialized with params: {:?}", params);
                    Ok(())
                }
            })
            // List Tools Handler
            .request_handler("tools/list", {
                let server = solana_server.clone();
                move |_: ()| -> Result<serde_json::Value> {
                    let response = server.list_tools()?;
                    Ok(serde_json::to_value(response)?)
                }
            })
            // List Resources Handler
            .request_handler("resources/list", {
                let server = solana_server.clone();
                move |_: ()| server.list_resources()
            })
            // Read Resource Handler
            .request_handler("resources/read", {
                let server = solana_server.clone();
                move |request: serde_json::Value| -> Result<serde_json::Value> {
                    let uri = request.get("params")
                        .and_then(|p| p.get("uri"))
                        .and_then(|u| u.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing uri parameter"))?;
                    server.read_resource(uri)
                }
            })
            // Call Tool Handler
            .request_handler("tools/call", {
                let server = solana_server.clone();
                move |request: CallToolRequest| {
                    let server = server.clone();
                    let handle = runtime.clone();
                    match handle.block_on(server.handle_tool_request(request)) {
                        Ok(response) => Ok(response),
                        Err(e) => Ok(CallToolResponse {
                            content: vec![ToolResponseContent::Text { text: json!({
                                "error": e.to_string()
                            }).to_string() }],
                            is_error: Some(true),
                            meta: None,
                        }),
                    }
                }
            }),
        ServerOptions::default()
            .capabilities(ServerCapabilities {
                tools: Some(json!(ToolsListResponse {
                    tools: vec![
                        Tool {
                            name: "get_slot".to_string(),
                            description: Some("Get current slot".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {},
                                "required": [],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_slot_leaders".to_string(),
                            description: Some("Get slot leaders".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {
                                    "start_slot": {"type": "integer"},
                                    "limit": {"type": "integer"}
                                },
                                "required": ["start_slot", "limit"],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_block".to_string(),
                            description: Some("Get block information".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {
                                    "slot": {"type": "integer"}
                                },
                                "required": ["slot"],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_block_height".to_string(),
                            description: Some("Get current block height".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {},
                                "required": [],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_balance".to_string(),
                            description: Some("Get account balance".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {
                                    "pubkey": {"type": "string"}
                                },
                                "required": ["pubkey"],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_account_info".to_string(),
                            description: Some("Get detailed account information".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {
                                    "pubkey": {"type": "string"}
                                },
                                "required": ["pubkey"],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_transaction".to_string(),
                            description: Some("Get transaction details".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {
                                    "signature": {"type": "string"}
                                },
                                "required": ["signature"],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_health".to_string(),
                            description: Some("Get node health status".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {},
                                "required": [],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_version".to_string(),
                            description: Some("Get node version information".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {},
                                "required": [],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_identity".to_string(),
                            description: Some("Get node identity".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {},
                                "required": [],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_epoch_info".to_string(),
                            description: Some("Get current epoch information".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {},
                                "required": [],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_inflation_rate".to_string(),
                            description: Some("Get current inflation rate".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {},
                                "required": [],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "get_token_accounts_by_owner".to_string(),
                            description: Some("Get token accounts owned by an address".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {
                                    "owner": {"type": "string"}
                                },
                                "required": ["owner"],
                                "additionalProperties": false
                            }),
                        },
                        Tool {
                            name: "process_sequential_thinking".to_string(),
                            description: Some("Process a sequence of thinking steps with dependencies".to_string()),
                            input_schema: json!({
                                "type": "object",
                                "properties": {
                                    "steps": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "id": {"type": "string"},
                                                "description": {"type": "string"},
                                                "dependencies": {
                                                    "type": "array",
                                                    "items": {"type": "string"}
                                                }
                                            },
                                            "required": ["id", "description"]
                                        }
                                    }
                                },
                                "required": ["steps"],
                                "additionalProperties": false
                            }),
                        }
                    ],
                    meta: None,
                    next_cursor: None,
                })),
                resources: Some(ResourceCapabilities {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                experimental: None,
                logging: None,
                prompts: None,
            }),
    );

    server.listen().await?;

    Ok(())
}
