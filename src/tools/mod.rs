use anyhow::Result;
use crate::transport::{JsonRpcMessage, JsonRpcResponse, JsonRpcError, JsonRpcVersion};
use crate::protocol::{InitializeRequest, InitializeResponse, ServerCapabilities, Implementation, LATEST_PROTOCOL_VERSION, ToolDefinition, ToolsListResponse, Resource, ResourcesListResponse};
use url::Url;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub fn create_success_response(result: Value, id: u64) -> JsonRpcMessage {
    log::debug!("Creating success response with id {}", id);
    JsonRpcMessage::Response(JsonRpcResponse {
        jsonrpc: JsonRpcVersion::V2,
        id,
        result: Some(result),
        error: None,
    })
}

pub fn create_error_response(code: i32, message: String, id: u64, protocol_version: Option<&str>) -> JsonRpcMessage {
    log::error!("Creating error response: {} (code: {})", message, code);
    let error = JsonRpcError {
        code,
        message,
        data: protocol_version.map(|v| serde_json::json!({ "protocolVersion": v })),
    };
    
    JsonRpcMessage::Response(JsonRpcResponse {
        jsonrpc: JsonRpcVersion::V2,
        id,
        result: None,
        error: Some(error),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CancelledParams {
    request_id: i64,
    reason: String,
}

pub async fn handle_initialize(params: Option<Value>, id: Option<Value>, state: &ServerState) -> Result<JsonRpcMessage> {
    log::info!("Handling initialize request");
    if let Some(params) = params {
        let init_params = match serde_json::from_value::<InitializeRequest>(params.clone()) {
            Ok(params) => params,
            Err(e) => {
                log::error!("Failed to parse initialize params: {}", e);
                return Ok(create_error_response(
                    -32602,
                    "Invalid params: protocolVersion is required".to_string(),
                    id.and_then(|v| v.as_u64()).unwrap_or(0),
                    Some(state.protocol_version.as_str()),
                ));
            }
        };

        log::info!(
            "Initializing with protocol version: {}, client: {} v{}", 
            init_params.protocol_version,
            init_params.client_info.name,
            init_params.client_info.version
        );

        // Validate protocol version
        if init_params.protocol_version != state.protocol_version {
            log::error!(
                "Protocol version mismatch. Server: {}, Client: {}", 
                state.protocol_version, 
                init_params.protocol_version
            );
            return Ok(create_error_response(
                -32002,
                format!("Protocol version mismatch. Server: {}, Client: {}", 
                    state.protocol_version, init_params.protocol_version),
                id.and_then(|v| v.as_u64()).unwrap_or(0),
                Some(state.protocol_version.as_str()),
            ));
        }

        let response = InitializeResponse {
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
            server_info: Implementation {
                name: "solana-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ServerCapabilities {
                tools: {
                    let mut tools = HashMap::new();
                    tools.insert("getAccountInfo".to_string(), ToolDefinition {
                        name: "getAccountInfo".to_string(),
                        description: Some("Returns all information associated with the account".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "pubkey": {
                                    "type": "string",
                                    "description": "Account public key (base58 encoded)"
                                },
                                "commitment": {
                                    "type": "string",
                                    "description": "Commitment level",
                                    "enum": ["processed", "confirmed", "finalized"]
                                },
                                "encoding": {
                                    "type": "string",
                                    "description": "Encoding format",
                                    "enum": ["base58", "base64", "jsonParsed"]
                                }
                            },
                            "required": ["pubkey"]
                        }),
                    });
                    tools.insert("getBalance".to_string(), ToolDefinition {
                        name: "getBalance".to_string(),
                        description: Some("Returns the balance of the account".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "pubkey": {
                                    "type": "string",
                                    "description": "Account public key (base58 encoded)"
                                },
                                "commitment": {
                                    "type": "string",
                                    "description": "Commitment level",
                                    "enum": ["processed", "confirmed", "finalized"]
                                }
                            },
                            "required": ["pubkey"]
                        }),
                    });
                    tools.insert("getProgramAccounts".to_string(), ToolDefinition {
                        name: "getProgramAccounts".to_string(),
                        description: Some("Returns all accounts owned by the program".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "programId": {
                                    "type": "string",
                                    "description": "Program public key (base58 encoded)"
                                },
                                "config": {
                                    "type": "object",
                                    "description": "Configuration object",
                                    "properties": {
                                        "encoding": {
                                            "type": "string",
                                            "enum": ["base58", "base64", "jsonParsed"]
                                        },
                                        "commitment": {
                                            "type": "string",
                                            "enum": ["processed", "confirmed", "finalized"]
                                        }
                                    }
                                }
                            },
                            "required": ["programId"]
                        }),
                    });
                    tools.insert("getTransaction".to_string(), ToolDefinition {
                        name: "getTransaction".to_string(),
                        description: Some("Returns transaction details".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "signature": {
                                    "type": "string",
                                    "description": "Transaction signature (base58 encoded)"
                                },
                                "commitment": {
                                    "type": "string",
                                    "enum": ["processed", "confirmed", "finalized"]
                                }
                            },
                            "required": ["signature"]
                        }),
                    });
                    tools.insert("getHealth".to_string(), ToolDefinition {
                        name: "getHealth".to_string(),
                        description: Some("Returns the current health of the node".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });
                    tools.insert("getVersion".to_string(), ToolDefinition {
                        name: "getVersion".to_string(),
                        description: Some("Returns the current Solana version".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });
                    Some(tools)
                },
                resources: {
                    let mut resources = HashMap::new();
                    resources.insert("docs".to_string(), Resource {
                        name: "Documentation".to_string(),
                        description: Some("Solana API documentation".to_string()),
                        uri: Url::parse("https://docs.solana.com/developing/clients/jsonrpc-api").unwrap(),
                        mime_type: Some("text/html".to_string()),
                    });
                    Some(resources)
                },
                ..Default::default()
            },
        };

        log::info!("Server initialized successfully");
        Ok(create_success_response(
            serde_json::to_value(response).unwrap(),
            id.and_then(|v| v.as_u64()).unwrap_or(0),
        ))
    } else {
        log::error!("Missing initialization params");
        Ok(create_error_response(-32602, "Invalid params".to_string(), id.and_then(|v| v.as_u64()).unwrap_or(0), Some(state.protocol_version.as_str())))
    }
}

pub async fn handle_cancelled(params: Option<Value>, id: Option<Value>, state: &ServerState) -> Result<JsonRpcMessage> {
    log::info!("Handling cancelled request");
    if let Some(params) = params {
        let cancel_params: CancelledParams = serde_json::from_value(params)?;
        log::info!("Request {} cancelled: {}", cancel_params.request_id, cancel_params.reason);
        Ok(create_success_response(
            Value::Null,
            id.and_then(|v| v.as_u64()).unwrap_or(0)
        ))
    } else {
        log::error!("Missing cancelled params");
        Ok(create_error_response(-32602, "Invalid params".to_string(), id.and_then(|v| v.as_u64()).unwrap_or(0), Some(state.protocol_version.as_str())))
    }
}

pub async fn handle_tools_list(id: Option<Value>, _state: &ServerState) -> Result<JsonRpcMessage> {
    log::info!("Handling tools/list request");
    let tools = vec![
        ToolDefinition {
            name: "getAccountInfo".to_string(),
            description: Some("Returns all information associated with the account".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkey": {
                        "type": "string",
                        "description": "Account public key (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"]
                    },
                    "encoding": {
                        "type": "string",
                        "description": "Encoding format",
                        "enum": ["base58", "base64", "jsonParsed"]
                    }
                },
                "required": ["pubkey"]
            }),
        },
        ToolDefinition {
            name: "getBalance".to_string(),
            description: Some("Returns the balance of the account".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkey": {
                        "type": "string",
                        "description": "Account public key (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["pubkey"]
            }),
        },
        ToolDefinition {
            name: "getProgramAccounts".to_string(),
            description: Some("Returns all accounts owned by the program".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "programId": {
                        "type": "string",
                        "description": "Program public key (base58 encoded)"
                    },
                    "config": {
                        "type": "object",
                        "description": "Configuration object",
                        "properties": {
                            "encoding": {
                                "type": "string",
                                "enum": ["base58", "base64", "jsonParsed"]
                            },
                            "commitment": {
                                "type": "string",
                                "enum": ["processed", "confirmed", "finalized"]
                            }
                        }
                    }
                },
                "required": ["programId"]
            }),
        },
        ToolDefinition {
            name: "getTransaction".to_string(),
            description: Some("Returns transaction details".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "signature": {
                        "type": "string",
                        "description": "Transaction signature (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["signature"]
            }),
        },
        ToolDefinition {
            name: "getHealth".to_string(),
            description: Some("Returns the current health of the node".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getVersion".to_string(),
            description: Some("Returns the current Solana version".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ];

    let tools_len = tools.len();
    log::debug!("Returning {} tools", tools_len);

    let response = ToolsListResponse {
        tools,
        next_cursor: None,
        meta: None,
    };

    Ok(create_success_response(
        serde_json::to_value(response).unwrap(),
        id.and_then(|v| v.as_u64()).unwrap_or(0),
    ))
}

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::server::ServerState;
use solana_sdk::pubkey::Pubkey;

pub async fn handle_request(request: &str, state: Arc<RwLock<ServerState>>) -> Result<JsonRpcMessage> {
    log::debug!("Received request: {}", request);
    let message: JsonRpcMessage = serde_json::from_str(request).map_err(|e| {
        log::error!("Failed to parse JSON-RPC request: {}", e);
        anyhow::anyhow!("Invalid JSON-RPC request: {}", e)
    })?;

    match message {
        JsonRpcMessage::Request(req) => {
            let mut state_guard = state.write().await;
            let protocol_version = Some(state_guard.protocol_version.as_str());

            if req.jsonrpc != JsonRpcVersion::V2 {
                log::error!("Invalid JSON-RPC version: {:?}", req.jsonrpc);
                return Ok(create_error_response(
                    -32600,
                    "Invalid Request: jsonrpc version must be 2.0".to_string(),
                    req.id,
                    protocol_version,
                ));
            }

            // Only allow initialize method if not initialized
            if !state_guard.initialized && req.method.as_str() != "initialize" {
                log::error!("Server not initialized, received method: {}", req.method);
                return Ok(create_error_response(
                    -32002,
                    "Server not initialized".to_string(),
                    req.id,
                    protocol_version,
                ));
            }

            log::info!("Handling method: {}", req.method);
            match req.method.as_str() {
                "initialize" => {
                    let response = handle_initialize(
                        req.params, 
                        Some(serde_json::Value::Number(req.id.into())),
                        &state_guard
                    ).await?;
                    
                    if response.is_success() {
                        state_guard.initialized = true;
                        log::info!("Server initialized successfully");
                    } else {
                        log::error!("Server initialization failed");
                    }
                    Ok(response)
                },
                "cancelled" => handle_cancelled(req.params, Some(serde_json::Value::Number(req.id.into())), &state_guard).await,
                "tools/list" => handle_tools_list(Some(serde_json::Value::Number(req.id.into())), &state_guard).await,
                
                // Account methods
                "getAccountInfo" => {
                    log::info!("Getting account info");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let pubkey_str = params.get("pubkey")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
                    let pubkey = Pubkey::try_from(pubkey_str)?;
                    
                    let state = state.read().await;
                    let result = crate::rpc::accounts::get_account_info(&state.rpc_client, &pubkey).await?;
                    Ok(create_success_response(result, req.id))
                },
                "getBalance" => {
                    log::info!("Getting balance");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let pubkey_str = params.get("pubkey")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
                    let pubkey = Pubkey::try_from(pubkey_str)?;
                    
                    let state = state.read().await;
                    let result = crate::rpc::accounts::get_balance(&state.rpc_client, &pubkey).await?;
                    Ok(create_success_response(result, req.id))
                },
                "getProgramAccounts" => {
                    log::info!("Getting program accounts");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let program_id_str = params.get("programId")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing programId parameter"))?;
                    let program_id = Pubkey::try_from(program_id_str)?;
                    
                    let state = state.read().await;
                    let result = crate::rpc::accounts::get_program_accounts(&state.rpc_client, &program_id).await?;
                    Ok(create_success_response(result, req.id))
                },
                
                "resources/templates/list" => {
                    log::info!("Handling resources/templates/list request");
                    let response = ResourcesListResponse {
                        resources: vec![],
                        next_cursor: None,
                        meta: None,
                    };

                    Ok(create_success_response(
                        serde_json::to_value(response).unwrap(),
                        req.id
                    ))
                },
                "resources/list" => {
                    log::info!("Handling resources/list request");
                    let resources = vec![
                        Resource {
                            uri: Url::parse("https://docs.solana.com/developing/clients/jsonrpc-api").unwrap(),
                            name: "Documentation".to_string(),
                            description: Some("Solana API documentation".to_string()),
                            mime_type: Some("text/html".to_string()),
                        }
                    ];

                    let response = ResourcesListResponse {
                        resources,
                        next_cursor: None,
                        meta: None,
                    };

                    Ok(create_success_response(
                        serde_json::to_value(response).unwrap(),
                        req.id
                    ))
                },
                _ => {
                    log::error!("Method not found: {}", req.method);
                    Ok(create_error_response(
                        -32601,
                        "Method not found".to_string(),
                        req.id,
                        protocol_version,
                    ))
                },
            }
        },
        JsonRpcMessage::Response(_) => {
            log::error!("Received response message when expecting request");
            Ok(create_error_response(
                -32600,
                "Invalid Request: expected request message".to_string(),
                0,
                None,
            ))
        },
        JsonRpcMessage::Notification(notification) => {
            match notification.method.as_str() {
                "notifications/initialized" => {
                    log::info!("Received initialized notification");
                    Ok(JsonRpcMessage::Notification(notification))
                },
                _ => {
                    log::error!("Unsupported notification: {}", notification.method);
                    Ok(create_error_response(
                        -32600,
                        format!("Unsupported notification: {}", notification.method),
                        0,
                        None,
                    ))
                }
            }
        },
    }
}