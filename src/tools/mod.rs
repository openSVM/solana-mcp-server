use anyhow::Result;
use crate::transport::{JsonRpcMessage, JsonRpcResponse, JsonRpcError, JsonRpcVersion};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn create_success_response(result: Value, id: u64) -> JsonRpcMessage {
    JsonRpcMessage::Response(JsonRpcResponse {
        jsonrpc: JsonRpcVersion::V2,
        id,
        result: Some(result),
        error: None,
    })
}

pub fn create_error_response(code: i32, message: String, id: u64) -> JsonRpcMessage {
    JsonRpcMessage::Response(JsonRpcResponse {
        jsonrpc: JsonRpcVersion::V2,
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    protocol_version: String,
    capabilities: Value,
    client_info: ClientInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientInfo {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CancelledParams {
    request_id: i64,
    reason: String,
}

pub async fn handle_initialize(params: Option<Value>, id: Option<Value>) -> Result<JsonRpcMessage> {
    if let Some(params) = params {
        let _init_params: InitializeParams = serde_json::from_value(params)?;
        Ok(create_success_response(
            serde_json::json!({
                "serverInfo": {
                    "name": "solana-mcp-server",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {
                        // Account methods
                        "getAccountInfo": {
                            "description": "Returns all information associated with the account",
                            "parameters": {
                                "pubkey": "Account public key (base58 encoded)",
                                "commitment": "Commitment level (optional)",
                                "encoding": "Encoding format (optional)"
                            }
                        },
                        "getBalance": {
                            "description": "Returns the balance of the account",
                            "parameters": {
                                "pubkey": "Account public key (base58 encoded)",
                                "commitment": "Commitment level (optional)"
                            }
                        },
                        "getProgramAccounts": {
                            "description": "Returns all accounts owned by the program",
                            "parameters": {
                                "programId": "Program public key (base58 encoded)",
                                "config": "Configuration object (optional)"
                            }
                        },
                        // Transaction methods
                        "getTransaction": {
                            "description": "Returns transaction details",
                            "parameters": {
                                "signature": "Transaction signature (base58 encoded)",
                                "config": "Configuration object (optional)"
                            }
                        },
                        "getSignaturesForAddress": {
                            "description": "Returns signatures for transactions involving an address",
                            "parameters": {
                                "address": "Account address (base58 encoded)",
                                "config": "Configuration object (optional)"
                            }
                        },
                        "getSignatureStatuses": {
                            "description": "Returns the statuses of a list of signatures",
                            "parameters": {
                                "signatures": "Array of transaction signatures",
                                "config": "Configuration object (optional)"
                            }
                        },
                        // Token methods
                        "getTokenAccountBalance": {
                            "description": "Returns the token balance of an SPL Token account",
                            "parameters": {
                                "accountAddress": "Token account address (base58 encoded)",
                                "commitment": "Commitment level (optional)"
                            }
                        },
                        "getTokenSupply": {
                            "description": "Returns the total supply of an SPL Token type",
                            "parameters": {
                                "mint": "Token mint address (base58 encoded)",
                                "commitment": "Commitment level (optional)"
                            }
                        },
                        "getTokenLargestAccounts": {
                            "description": "Returns the 20 largest accounts of a particular SPL Token type",
                            "parameters": {
                                "mint": "Token mint address (base58 encoded)",
                                "commitment": "Commitment level (optional)"
                            }
                        },
                        // Block methods
                        "getBlock": {
                            "description": "Returns information about a confirmed block",
                            "parameters": {
                                "slot": "Slot number",
                                "config": "Configuration object (optional)"
                            }
                        },
                        "getBlockHeight": {
                            "description": "Returns the current block height",
                            "parameters": {
                                "commitment": "Commitment level (optional)"
                            }
                        },
                        // System methods
                        "getHealth": {
                            "description": "Returns the current health of the node",
                            "parameters": {}
                        },
                        "getVersion": {
                            "description": "Returns the current Solana version",
                            "parameters": {}
                        },
                        "getSlot": {
                            "description": "Returns the current slot",
                            "parameters": {
                                "commitment": "Commitment level (optional)"
                            }
                        },
                        "getEpochInfo": {
                            "description": "Returns information about the current epoch",
                            "parameters": {
                                "commitment": "Commitment level (optional)"
                            }
                        }
                    },
                    "resources": {
                        "solana_docs": {
                            "description": "Solana documentation",
                            "uri": "solana://docs/core"
                        },
                        "rpc_docs": {
                            "description": "RPC API documentation",
                            "uri": "solana://docs/rpc"
                        }
                    }
                }
            }),
            id.and_then(|v| v.as_u64()).unwrap_or(0),
        ))
    } else {
        Ok(create_error_response(-32602, "Invalid params".to_string(), id.and_then(|v| v.as_u64()).unwrap_or(0)))
    }
}

pub async fn handle_cancelled(params: Option<Value>, id: Option<Value>) -> Result<JsonRpcMessage> {
    if let Some(params) = params {
        let _cancel_params: CancelledParams = serde_json::from_value(params)?;
        Ok(create_success_response(
            Value::Null,
            id.and_then(|v| v.as_u64()).unwrap_or(0)
        ))
    } else {
        Ok(create_error_response(-32602, "Invalid params".to_string(), id.and_then(|v| v.as_u64()).unwrap_or(0)))
    }
}

pub async fn handle_tools_list(id: Option<Value>) -> Result<JsonRpcMessage> {
    Ok(create_success_response(
        serde_json::json!({
            "tools": [
                {
                    "name": "getAccountInfo",
                    "description": "Returns all information associated with the account",
                    "parameters": {
                        "pubkey": "Account public key (base58 encoded)"
                    }
                },
                {
                    "name": "getBalance",
                    "description": "Returns the balance of the account",
                    "parameters": {
                        "pubkey": "Account public key (base58 encoded)"
                    }
                }
            ],
            "resources": [
                {
                    "name": "solana_docs",
                    "description": "Solana documentation",
                    "uri": "solana://docs/core"
                },
                {
                    "name": "rpc_docs",
                    "description": "RPC API documentation",
                    "uri": "solana://docs/rpc"
                }
            ]
        }),
        id.and_then(|v| v.as_u64()).unwrap_or(0),
    ))
}

pub async fn handle_request(request: &str) -> Result<JsonRpcMessage> {
    let message: JsonRpcMessage = serde_json::from_str(request).map_err(|_| {
        anyhow::anyhow!("Invalid JSON-RPC request")
    })?;

    match message {
        JsonRpcMessage::Request(req) => {
            if req.jsonrpc != JsonRpcVersion::V2 {
                return Ok(create_error_response(
                    -32600,
                    "Invalid Request: jsonrpc version must be 2.0".to_string(),
                    req.id,
                ));
            }

            match req.method.as_str() {
                "initialize" => handle_initialize(req.params, Some(serde_json::Value::Number(req.id.into()))).await,
                "cancelled" => handle_cancelled(req.params, Some(serde_json::Value::Number(req.id.into()))).await,
                "tools/list" => handle_tools_list(Some(serde_json::Value::Number(req.id.into()))).await,
                _ => Ok(create_error_response(
                    -32601,
                    "Method not found".to_string(),
                    req.id,
                )),
            }
        },
        JsonRpcMessage::Response(_) => {
            Ok(create_error_response(
                -32600,
                "Invalid Request: expected request message".to_string(),
                0,
            ))
        },
        JsonRpcMessage::Notification(_) => {
            Ok(create_error_response(
                -32600,
                "Invalid Request: notifications not supported".to_string(),
                0,
            ))
        },
    }
}
