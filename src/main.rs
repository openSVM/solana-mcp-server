use anyhow::{Result, Context};
use clap::Parser;
use std::path::PathBuf;
use std::{fs, io::Write};

#[derive(Debug, serde::Deserialize)]
struct Config {
    url: String,
    apikey: String,
    #[serde(default)]
    iam_agent: bool,
    #[serde(default)]
    tools: Vec<String>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(value_name = "CONFIG", required = false)]
    config: Option<PathBuf>,
}

mod transport;

use mcp_sdk::{
    protocol::ProtocolBuilder,
    server::{Server, ServerOptions},
    types::{CallToolRequest, ServerCapabilities, ResourceCapabilities, ToolsListResponse, Tool},
};
use transport::CustomStdioTransport;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_mcp_server::SolanaMcpServer;
use std::sync::Arc;
use tokio::runtime::Handle;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Default config values
    let config = if let Some(config_path) = args.config {
        // Load and parse config file
        let config_str = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
        serde_json::from_str(&config_str)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?
    } else {
        Config {
            url: "https://rpc.opensvm.com".to_string(),
            apikey: "".to_string(),
            iam_agent: false,
            tools: vec![],
        }
    };

    let rpc_url = config.url;

    let client = RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    );

    let solana_server = Arc::new(SolanaMcpServer::new(client));
    let transport = CustomStdioTransport::new();
    
    let runtime = Handle::current();

    // API key validation function
    let api_key = config.apikey.clone();
    let validate_api_key = move |request: &serde_json::Value| -> Result<()> {
        let auth_header = request.get("auth")
            .and_then(|a| a.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing auth header"))?;
        
        if auth_header != api_key {
            return Err(anyhow::anyhow!("Invalid API key"));
        }
        Ok(())
    };

    // Define all available tools
    let all_tools = vec![
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
        }
    ];

    let enabled_tools = if config.tools.is_empty() {
        all_tools
    } else {
        all_tools
            .into_iter()
            .filter(|tool| config.tools.contains(&tool.name))
            .collect()
    };

    // Create server with capabilities
    let server = Server::new(
        ProtocolBuilder::new(transport)
            .request_handler("initialize", {
                let server = solana_server.clone();
                let validate = validate_api_key.clone();
                move |request: serde_json::Value| -> Result<serde_json::Value> {
                    eprintln!("Received initialize request: {}", serde_json::to_string(&request)?);
                    let id = request.get("id").and_then(|id| Some(id.clone()));
                    
                    // Validate JSON-RPC request
                    if request.get("jsonrpc") != Some(&json!("2.0")) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32600,
                                "message": "Invalid Request: jsonrpc version must be 2.0"
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        println!("{}", serde_json::to_string(&response)?);
                        std::io::stdout().flush()?;
                        return Ok(response);
                    }

                    // Validate method
                    if request.get("method") != Some(&json!("initialize")) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32601,
                                "message": "Method not found"
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        println!("{}", serde_json::to_string(&response)?);
                        std::io::stdout().flush()?;
                        return Ok(response);
                    }

                    // Validate API key
                    if let Err(e) = validate(&request) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32602,
                                "message": e.to_string()
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        println!("{}", serde_json::to_string(&response)?);
                        std::io::stdout().flush()?;
                        return Ok(response);
                    }

                    let tools = server.list_tools()?;
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "server": {
                                "name": "solana-mcp",
                                "version": "0.1.2",
                                "vendor": "OpenSVM"
                            },
                            "protocol": {
                                "name": "mcp",
                                "version": "0.1.0"
                            },
                            "capabilities": {
                                "tools": tools,
                                "resources": {
                                    "subscribe": false,
                                    "list_changed": false
                                }
                            }
                        }
                    });
                    eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                    println!("{}", serde_json::to_string(&response)?);
                    std::io::stdout().flush()?;
                    Ok(response)
                }
            })
            // List Tools Handler
            .request_handler("tools/list", {
                let server = solana_server.clone();
                let validate = validate_api_key.clone();
                move |request: serde_json::Value| -> Result<serde_json::Value> {
                    eprintln!("Received tools/list request: {}", serde_json::to_string(&request)?);
                    let id = request.get("id").and_then(|id| Some(id.clone()));
                    
                    if request.get("jsonrpc") != Some(&json!("2.0")) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32600,
                                "message": "Invalid Request: jsonrpc version must be 2.0"
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        return Ok(response);
                    }

                    if request.get("method") != Some(&json!("tools/list")) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32601,
                                "message": "Method not found"
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        return Ok(response);
                    }

                    // Validate API key
                    if let Err(e) = validate(&request) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32602,
                                "message": e.to_string()
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        return Ok(response);
                    }

                    let response = server.list_tools()?;
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": response
                    });
                    eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                    println!("{}", serde_json::to_string(&response)?);
                    std::io::stdout().flush()?;
                    Ok(response)
                }
            })
            // Call Tool Handler
            .request_handler("tools/call", {
                let server = solana_server.clone();
                let validate = validate_api_key.clone();
                move |request: serde_json::Value| -> Result<serde_json::Value> {
                    eprintln!("Received tools/call request: {}", serde_json::to_string(&request)?);
                    let id = request.get("id").and_then(|id| Some(id.clone()));
                    
                    if request.get("jsonrpc") != Some(&json!("2.0")) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32600,
                                "message": "Invalid Request: jsonrpc version must be 2.0"
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        return Ok(response);
                    }

                    if request.get("method") != Some(&json!("tools/call")) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32601,
                                "message": "Method not found"
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        return Ok(response);
                    }

                    // Validate API key
                    if let Err(e) = validate(&request) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32602,
                                "message": e.to_string()
                            }
                        });
                        eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                        return Ok(response);
                    }

                    let original_request = request.clone();
                    let tool_request = match serde_json::from_value::<CallToolRequest>(request) {
                        Ok(req) => req,
                        Err(e) => {
                            let response = json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32700,
                                    "message": format!("Parse error: {}", e),
                                    "data": {
                                        "request": original_request
                                    }
                                }
                            });
                            eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                            return Ok(response);
                        }
                    };
                    let server = server.clone();
                    let handle = runtime.clone();
                    let result = handle.block_on(server.handle_tool_request(tool_request));
                    let response = match result {
                        Ok(response) => {
                            json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": response
                            })
                        },
                        Err(e) => {
                            json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32602,
                                    "message": e.to_string(),
                                    "data": {
                                        "request": original_request
                                    }
                                }
                            })
                        },
                    };
                    eprintln!("Sending response: {}", serde_json::to_string(&response)?);
                    println!("{}", serde_json::to_string(&response)?);
                    std::io::stdout().flush()?;
                    Ok(response)
                }
            }),
        ServerOptions::default()
            .name("solana-mcp")
            .version("0.1.2")
            .capabilities(ServerCapabilities {
                tools: Some(json!(ToolsListResponse {
                    tools: enabled_tools,
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

    eprintln!("Initializing Solana MCP server...");
    eprintln!("Server ready to accept requests");
    std::io::stderr().flush()?;
    
    // Handle initialization
    server.listen().await?;

    Ok(())
}
