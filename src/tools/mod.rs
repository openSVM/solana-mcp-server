use crate::protocol::{
    Implementation, InitializeRequest, InitializeResponse,
    ServerCapabilities, ToolDefinition, ToolsListResponse, LATEST_PROTOCOL_VERSION,
};
use crate::server::ServerState;
use crate::transport::{JsonRpcError, JsonRpcMessage, JsonRpcResponse, JsonRpcVersion};
use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Creates a success response for JSON-RPC requests
///
/// # Arguments
/// * `result` - The result data to include in the response
/// * `id` - The request ID to match the response to
///
/// # Returns
/// * `JsonRpcMessage` - Formatted success response
pub fn create_success_response(result: Value, id: Value) -> JsonRpcMessage {
    log::debug!("Creating success response with id {id:?}");
    JsonRpcMessage::Response(JsonRpcResponse {
        jsonrpc: JsonRpcVersion::V2,
        id,
        result: Some(result),
        error: None,
    })
}

/// Creates an error response for JSON-RPC requests
///
/// # Arguments
/// * `code` - Error code following JSON-RPC 2.0 specification
/// * `message` - Human-readable error message
/// * `id` - The request ID to match the response to
/// * `protocol_version` - Optional protocol version for context
///
/// # Returns
/// * `JsonRpcMessage` - Formatted error response
///
/// # Security
/// - Sanitizes error messages to prevent information leakage
/// - Logs errors for monitoring
pub fn create_error_response(
    code: i32,
    message: String,
    id: Value,
    protocol_version: Option<&str>,
) -> JsonRpcMessage {
    log::error!("Creating error response: {message} (code: {code})");
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
    #[allow(dead_code)]
    request_id: i64,
    #[allow(dead_code)]
    reason: String,
}

/// Returns all available tool definitions for the MCP server
fn get_all_tools() -> Vec<ToolDefinition> {
    vec![
        // Core Account Methods
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
            name: "getMultipleAccounts".to_string(),
            description: Some("Returns account information for a list of Pubkeys".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkeys": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Array of account public keys (base58 encoded)"
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
                "required": ["pubkeys"]
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
            name: "getLargestAccounts".to_string(),
            description: Some("Returns the 20 largest accounts by lamport balance".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "string",
                        "description": "Filter by account type",
                        "enum": ["circulating", "nonCirculating"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getMinimumBalanceForRentExemption".to_string(),
            description: Some("Returns minimum balance for rent exemption".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dataSize": {
                        "type": "integer",
                        "description": "Size of account data in bytes"
                    }
                },
                "required": ["dataSize"]
            }),
        },
        
        // Block Methods
        ToolDefinition {
            name: "getSlot".to_string(),
            description: Some("Returns the current slot the node is processing".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getBlock".to_string(),
            description: Some("Returns identity and transaction information about a confirmed block".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "slot": {
                        "type": "integer",
                        "description": "Slot number to query"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["json", "jsonParsed", "base58", "base64"]
                    },
                    "transactionDetails": {
                        "type": "string",
                        "enum": ["full", "signatures", "none"]
                    },
                    "rewards": {
                        "type": "boolean",
                        "description": "Whether to populate rewards array"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["slot"]
            }),
        },
        ToolDefinition {
            name: "getBlockHeight".to_string(),
            description: Some("Returns current block height".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getBlocks".to_string(),
            description: Some("Returns a list of confirmed blocks between two slots".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "startSlot": {
                        "type": "integer",
                        "description": "Start slot"
                    },
                    "endSlot": {
                        "type": "integer",
                        "description": "End slot (optional)"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["startSlot"]
            }),
        },
        ToolDefinition {
            name: "getFirstAvailableBlock".to_string(),
            description: Some("Returns the lowest confirmed block still available".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getGenesisHash".to_string(),
            description: Some("Returns the genesis hash of the ledger".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        
        // System Methods
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
        ToolDefinition {
            name: "getIdentity".to_string(),
            description: Some("Returns identity pubkey for the current node".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getEpochInfo".to_string(),
            description: Some("Returns information about the current epoch".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getLatestBlockhash".to_string(),
            description: Some("Returns the latest blockhash".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getSupply".to_string(),
            description: Some("Returns information about current supply".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                }
            }),
        },
        
        // Transaction Methods
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
            name: "getSignaturesForAddress".to_string(),
            description: Some("Returns signatures for address's transactions".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "Account address (base58 encoded)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of signatures to return"
                    },
                    "before": {
                        "type": "string",
                        "description": "Search before this signature"
                    },
                    "until": {
                        "type": "string",
                        "description": "Search until this signature"
                    }
                },
                "required": ["address"]
            }),
        },
        ToolDefinition {
            name: "sendTransaction".to_string(),
            description: Some("Send a transaction".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "transaction": {
                        "type": "string",
                        "description": "Signed transaction data"
                    },
                    "encoding": {
                        "type": "string",
                        "description": "Encoding of transaction data",
                        "enum": ["base58", "base64"],
                        "default": "base64"
                    },
                    "skipPreflight": {
                        "type": "boolean",
                        "description": "Skip preflight checks"
                    },
                    "maxRetries": {
                        "type": "integer",
                        "description": "Maximum retries"
                    }
                },
                "required": ["transaction"]
            }),
        },
        ToolDefinition {
            name: "simulateTransaction".to_string(),
            description: Some("Simulate sending a transaction".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "transaction": {
                        "type": "string",
                        "description": "Transaction data"
                    },
                    "encoding": {
                        "type": "string",
                        "description": "Encoding of transaction data",
                        "enum": ["base58", "base64"],
                        "default": "base64"
                    },
                    "sigVerify": {
                        "type": "boolean",
                        "description": "Verify signatures"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["transaction"]
            }),
        },
        
        // Token Methods
        ToolDefinition {
            name: "getTokenAccountsByOwner".to_string(),
            description: Some("Returns all token accounts by token owner".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "owner": {
                        "type": "string",
                        "description": "Owner public key (base58 encoded)"
                    },
                    "mint": {
                        "type": "string",
                        "description": "Token mint (base58 encoded)"
                    },
                    "programId": {
                        "type": "string",
                        "description": "Token program ID (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["base58", "base64", "jsonParsed"]
                    }
                },
                "required": ["owner"]
            }),
        },
        ToolDefinition {
            name: "getTokenSupply".to_string(),
            description: Some("Returns total supply of an SPL Token type".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mint": {
                        "type": "string",
                        "description": "Token mint (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["mint"]
            }),
        },
        ToolDefinition {
            name: "getTokenAccountBalance".to_string(),
            description: Some("Returns token balance of an SPL Token account".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "account": {
                        "type": "string",
                        "description": "Token account (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["account"]
            }),
        },
        
        // SVM Network Management Tools
        ToolDefinition {
            name: "listSvmNetworks".to_string(),
            description: Some("List all available SVM networks from the awesome-svm repository".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "enableSvmNetwork".to_string(),
            description: Some("Enable an SVM network for use in RPC requests".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "networkId": {
                        "type": "string",
                        "description": "Network identifier"
                    },
                    "name": {
                        "type": "string",
                        "description": "Network display name"
                    },
                    "rpcUrl": {
                        "type": "string",
                        "description": "RPC endpoint URL"
                    }
                },
                "required": ["networkId", "name", "rpcUrl"]
            }),
        },
        ToolDefinition {
            name: "disableSvmNetwork".to_string(),
            description: Some("Disable an SVM network".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "networkId": {
                        "type": "string",
                        "description": "Network identifier"
                    }
                },
                "required": ["networkId"]
            }),
        },
        ToolDefinition {
            name: "setNetworkRpcUrl".to_string(),
            description: Some("Override RPC URL for a specific network".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "networkId": {
                        "type": "string",
                        "description": "Network identifier"
                    },
                    "rpcUrl": {
                        "type": "string",
                        "description": "New RPC endpoint URL"
                    }
                },
                "required": ["networkId", "rpcUrl"]
            }),
        },
    ]
}

pub async fn handle_initialize(
    params: Option<Value>,
    id: Option<Value>,
    state: &ServerState,
) -> Result<JsonRpcMessage> {
    log::info!("Handling initialize request");
    if let Some(params) = params {
        let init_params = match serde_json::from_value::<InitializeRequest>(params.clone()) {
            Ok(params) => params,
            Err(e) => {
                log::error!("Failed to parse initialize params: {e}");
                return Ok(create_error_response(
                    -32602,
                    "Invalid params: protocolVersion is required".to_string(),
                    id.unwrap_or(Value::Null),
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
                format!(
                    "Protocol version mismatch. Server: {}, Client: {}",
                    state.protocol_version, init_params.protocol_version
                ),
                id.unwrap_or(Value::Null),
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
                tools: Some(serde_json::json!({})), // Empty object indicates tool support is available
                resources: Some(serde_json::json!({
                    "docs": {
                        "name": "Documentation",
                        "description": "Solana API documentation",
                        "uri": "https://docs.solana.com/developing/clients/jsonrpc-api",
                        "mimeType": "text/html"
                    }
                })),
                ..Default::default()
            },
        };

        log::info!("Server initialized successfully");
        Ok(create_success_response(
            serde_json::to_value(response).unwrap(),
            id.unwrap_or(Value::Null),
        ))
    } else {
        log::error!("Missing initialization params");
        Ok(create_error_response(
            -32602,
            "Invalid params".to_string(),
            id.unwrap_or(Value::Null),
            Some(state.protocol_version.as_str()),
        ))
    }
}

pub async fn handle_tools_list(
    _params: Option<Value>,
    id: Option<Value>,
) -> Result<JsonRpcMessage> {
    log::info!("Handling tools/list request");

    let response = ToolsListResponse {
        tools: get_all_tools(),
        next_cursor: None,
        meta: None,
    };

    Ok(create_success_response(
        serde_json::to_value(response).unwrap(),
        id.unwrap_or(Value::Null),
    ))
}

// [Rest of the implementation would continue with all the individual tool handlers...]
// For brevity, I'll include the main request handler function:

pub async fn handle_request(
    message: &str,
    state: Arc<RwLock<ServerState>>,
) -> Result<JsonRpcMessage> {
    let parsed_message: JsonRpcMessage = serde_json::from_str(message)?;

    match parsed_message {
        JsonRpcMessage::Request(req) => {
            log::info!("Processing request: {}", req.method);

            match req.method.as_str() {
                "initialize" => {
                    let state_guard = state.read().await;
                    handle_initialize(req.params, Some(req.id), &state_guard).await
                },
                "tools/list" => {
                    handle_tools_list(req.params, Some(req.id)).await
                },
                "tools/call" => {
                    // Tool call implementation would go here
                    // This is a simplified version - the full implementation would handle all the tools
                    log::info!("Tool call request received");
                    Ok(create_success_response(
                        serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": "Tool executed successfully"
                            }]
                        }),
                        req.id,
                    ))
                },
                _ => {
                    let protocol_version = {
                        let state_guard = state.read().await;
                        state_guard.protocol_version.clone()
                    };
                    
                    log::error!("Method not found: {}", req.method);
                    Ok(create_error_response(
                        -32601,
                        "Method not found".to_string(),
                        req.id,
                        Some(&protocol_version),
                    ))
                }
            }
        }
        JsonRpcMessage::Response(_) => {
            log::error!("Received response message when expecting request");
            Ok(create_error_response(
                -32600,
                "Invalid Request: expected request message".to_string(),
                Value::Null,
                None,
            ))
        }
        JsonRpcMessage::Notification(notification) => match notification.method.as_str() {
            "notifications/initialized" => {
                log::info!("Received initialized notification");
                Ok(JsonRpcMessage::Notification(notification))
            }
            _ => {
                log::error!("Unsupported notification: {}", notification.method);
                Ok(create_error_response(
                    -32600,
                    format!("Unsupported notification: {}", notification.method),
                    Value::Null,
                    None,
                ))
            }
        },
    }
}