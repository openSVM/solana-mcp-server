use crate::protocol::{
    Implementation, InitializeRequest, InitializeResponse, Resource, ResourcesListResponse,
    ServerCapabilities, ToolDefinition, ToolsListResponse, LATEST_PROTOCOL_VERSION,
};
use crate::server::ServerState;
use crate::transport::{JsonRpcError, JsonRpcMessage, JsonRpcResponse, JsonRpcVersion};
use crate::validation::{
    sanitize_for_logging, validate_network_id, validate_network_name, validate_rpc_url,
};
use crate::SvmNetwork;
use anyhow::Result;
use base64::Engine;
use reqwest;
use serde::Deserialize;
use serde_json::Value;
use solana_sdk::commitment_config::CommitmentConfig;

use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

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

pub async fn handle_cancelled(
    params: Option<Value>,
    id: Option<Value>,
    state: &ServerState,
) -> Result<JsonRpcMessage> {
    log::info!("Handling cancelled request");
    if let Some(params) = params {
        let _cancel_params: CancelledParams = serde_json::from_value(params)?;
        Ok(create_success_response(
            Value::Null,
            id.unwrap_or(Value::Null),
        ))
    } else {
        log::error!("Missing cancelled params");
        Ok(create_error_response(
            -32602,
            "Invalid params".to_string(),
            id.unwrap_or(Value::Null),
            Some(state.protocol_version.as_str()),
        ))
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
        // Additional Account Methods
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
            description: Some(
                "Returns identity and transaction information about a confirmed block".to_string(),
            ),
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
        ToolDefinition {
            name: "getAccountOwner".to_string(),
            description: Some("Returns the owner of an account".to_string()),
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
            name: "getTokenAccountsByMint".to_string(),
            description: Some("Returns all token accounts by token mint".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
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
                "required": ["mint"]
            }),
        },
        // Additional Block Methods
        ToolDefinition {
            name: "getSlotLeaders".to_string(),
            description: Some("Returns slot leaders for a given slot range".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "startSlot": {
                        "type": "integer",
                        "description": "Start slot"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Limit number of results"
                    }
                },
                "required": ["startSlot", "limit"]
            }),
        },
        ToolDefinition {
            name: "getBlockProduction".to_string(),
            description: Some("Returns recent block production information".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "identity": {
                        "type": "string",
                        "description": "Validator identity (base58 encoded)"
                    },
                    "firstSlot": {
                        "type": "integer",
                        "description": "First slot to query"
                    },
                    "lastSlot": {
                        "type": "integer",
                        "description": "Last slot to query"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getVoteAccounts".to_string(),
            description: Some("Returns account info and stake for all voting accounts".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    },
                    "votePubkey": {
                        "type": "string",
                        "description": "Vote account pubkey (base58 encoded)"
                    },
                    "keepUnstakedDelinquents": {
                        "type": "boolean",
                        "description": "Keep unstaked delinquents"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getLeaderSchedule".to_string(),
            description: Some("Returns the leader schedule for an epoch".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "slot": {
                        "type": "integer",
                        "description": "Slot to query (optional)"
                    },
                    "identity": {
                        "type": "string",
                        "description": "Validator identity (base58 encoded)"
                    }
                }
            }),
        },
        // Additional System Methods
        ToolDefinition {
            name: "getClusterNodes".to_string(),
            description: Some("Returns information about all cluster nodes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getEpochSchedule".to_string(),
            description: Some("Returns epoch schedule information".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getInflationGovernor".to_string(),
            description: Some("Returns current inflation governor".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getInflationRate".to_string(),
            description: Some("Returns specific inflation values for current epoch".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getInflationReward".to_string(),
            description: Some("Returns inflation reward for list of addresses".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "addresses": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Array of addresses (base58 encoded)"
                    },
                    "epoch": {
                        "type": "integer",
                        "description": "Epoch number"
                    }
                },
                "required": ["addresses"]
            }),
        },
        ToolDefinition {
            name: "getTransactionCount".to_string(),
            description: Some("Returns current Transaction count from ledger".to_string()),
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
            name: "requestAirdrop".to_string(),
            description: Some("Request an airdrop of lamports to a Pubkey".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkey": {
                        "type": "string",
                        "description": "Public key to receive airdrop (base58 encoded)"
                    },
                    "lamports": {
                        "type": "integer",
                        "description": "Amount in lamports"
                    }
                },
                "required": ["pubkey", "lamports"]
            }),
        },
        // Additional Transaction Methods
        ToolDefinition {
            name: "getBlockTime".to_string(),
            description: Some("Returns estimated production time of a block".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "slot": {
                        "type": "integer",
                        "description": "Slot number"
                    }
                },
                "required": ["slot"]
            }),
        },
        ToolDefinition {
            name: "getFeeForMessage".to_string(),
            description: Some("Get the fee for a message".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Encoded message"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["base58", "base64"],
                        "default": "base64"
                    }
                },
                "required": ["message"]
            }),
        },
        // Additional Token Methods
        ToolDefinition {
            name: "getTokenAccountsByDelegate".to_string(),
            description: Some("Returns all token accounts by approved delegate".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "delegate": {
                        "type": "string",
                        "description": "Delegate public key (base58 encoded)"
                    },
                    "mint": {
                        "type": "string",
                        "description": "Token mint (base58 encoded)"
                    },
                    "programId": {
                        "type": "string",
                        "description": "Token program ID (base58 encoded)"
                    }
                },
                "required": ["delegate"]
            }),
        },
        ToolDefinition {
            name: "getTokenLargestAccounts".to_string(),
            description: Some("Returns 20 largest accounts of a token type".to_string()),
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
        // Additional Block and Slot Methods
        ToolDefinition {
            name: "getBlocksWithLimit".to_string(),
            description: Some(
                "Returns a list of confirmed blocks starting at given slot".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "startSlot": {
                        "type": "integer",
                        "description": "Start slot"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of blocks to return"
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["startSlot", "limit"]
            }),
        },
        ToolDefinition {
            name: "getStakeMinimumDelegation".to_string(),
            description: Some("Returns stake minimum delegation".to_string()),
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
        // Additional complex transaction method
        ToolDefinition {
            name: "getTransactionWithConfig".to_string(),
            description: Some(
                "Returns transaction details with additional configuration".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "signature": {
                        "type": "string",
                        "description": "Transaction signature (base58 encoded)"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["json", "jsonParsed", "base58", "base64"]
                    },
                    "commitment": {
                        "type": "string",
                        "enum": ["processed", "confirmed", "finalized"]
                    },
                    "maxSupportedTransactionVersion": {
                        "type": "integer",
                        "description": "Maximum transaction version to return"
                    }
                },
                "required": ["signature"]
            }),
        },
        // New critical missing methods
        ToolDefinition {
            name: "isBlockhashValid".to_string(),
            description: Some("Check if a blockhash is still valid".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "blockhash": {
                        "type": "string",
                        "description": "Base58 encoded blockhash"
                    },
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"]
                    }
                },
                "required": ["blockhash"]
            }),
        },
        ToolDefinition {
            name: "getSlotLeader".to_string(),
            description: Some("Get the current slot leader".to_string()),
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
            name: "minimumLedgerSlot".to_string(), 
            description: Some("Get the minimum ledger slot available".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getMaxRetransmitSlot".to_string(), 
            description: Some("Get the max slot seen from retransmit stage".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getMaxShredInsertSlot".to_string(), 
            description: Some("Get the max slot seen from shred insert".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getHighestSnapshotSlot".to_string(), 
            description: Some("Get highest snapshot slot".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Deprecated methods for backward compatibility
        ToolDefinition {
            name: "getRecentBlockhash".to_string(), 
            description: Some("Get recent blockhash (deprecated)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getFees".to_string(), 
            description: Some("Get fees (deprecated)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getConfirmedBlock".to_string(), 
            description: Some("Get confirmed block (deprecated)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "slot": {
                        "type": "integer",
                        "description": "Slot number to query"
                    }
                },
                "required": ["slot"]
            }),
        },
        ToolDefinition {
            name: "getConfirmedTransaction".to_string(), 
            description: Some("Get confirmed transaction (deprecated)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "signature": {
                        "type": "string",
                        "description": "Transaction signature (base58 encoded)"
                    }
                },
                "required": ["signature"]
            }),
        },
        ToolDefinition {
            name: "getConfirmedBlocks".to_string(), 
            description: Some("Get confirmed blocks (deprecated)".to_string()),
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
                    }
                },
                "required": ["startSlot"]
            }),
        },
        ToolDefinition {
            name: "getConfirmedBlocksWithLimit".to_string(), 
            description: Some("Get confirmed blocks with limit (deprecated)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "startSlot": {
                        "type": "integer",
                        "description": "Start slot"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of blocks to return"
                    }
                },
                "required": ["startSlot", "limit"]
            }),
        },
        ToolDefinition {
            name: "getConfirmedSignaturesForAddress2".to_string(), 
            description: Some("Get confirmed signatures for address (deprecated)".to_string()),
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
                    }
                },
                "required": ["address"]
            }),
        },
        ToolDefinition {
            name: "getAccountInfoAndContext".to_string(),
            description: Some("Returns account information with context (slot info)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkey": {
                        "type": "string",
                        "description": "Account public key (base58 encoded)"
                    }
                },
                "required": ["pubkey"]
            }),
        },
        ToolDefinition {
            name: "getBalanceAndContext".to_string(),
            description: Some("Returns account balance with context (slot info)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkey": {
                        "type": "string",
                        "description": "Account public key (base58 encoded)"
                    }
                },
                "required": ["pubkey"]
            }),
        },
        ToolDefinition {
            name: "getMultipleAccountsAndContext".to_string(),
            description: Some("Returns multiple account information with context (slot info)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkeys": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Array of account public keys (base58 encoded)"
                    }
                },
                "required": ["pubkeys"]
            }),
        },
        ToolDefinition {
            name: "getProgramAccountsAndContext".to_string(),
            description: Some("Returns all accounts owned by program with context (slot info)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "program_id": {
                        "type": "string",
                        "description": "Program public key (base58 encoded)"
                    },
                    "filters": {
                        "type": "array",
                        "description": "Optional filters to apply",
                        "items": {
                            "type": "object"
                        }
                    }
                },
                "required": ["program_id"]
            }),
        },
        ToolDefinition {
            name: "getRecentPerformanceSamples".to_string(),
            description: Some("Returns recent performance samples from the cluster".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of samples to return"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getRecentPrioritizationFees".to_string(),
            description: Some("Returns recent prioritization fees".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "addresses": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Array of account addresses (base58 encoded)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getSignatureStatuses".to_string(),
            description: Some("Returns signature statuses for transaction signatures".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "signatures": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Array of transaction signatures (base58 encoded)"
                    },
                    "search_transaction_history": {
                        "type": "boolean",
                        "description": "Search transaction history (default: false)"
                    }
                },
                "required": ["signatures"]
            }),
        },
        // Manual RPC methods for missing functionality
        ToolDefinition {
            name: "getBlockCommitment".to_string(),
            description: Some("Get block commitment information for a specific slot".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "slot": {
                        "type": "integer",
                        "description": "Slot number to query"
                    }
                },
                "required": ["slot"]
            }),
        },
        ToolDefinition {
            name: "getSnapshotSlot".to_string(),
            description: Some("Get current snapshot slot".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getStakeActivation".to_string(),
            description: Some("Get stake activation information for a stake account".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pubkey": {
                        "type": "string",
                        "description": "Stake account public key (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"]
                    },
                    "epoch": {
                        "type": "integer",
                        "description": "Epoch number (optional)"
                    }
                },
                "required": ["pubkey"]
            }),
        },
        // WebSocket Subscription Methods
        ToolDefinition {
            name: "accountSubscribe".to_string(),
            description: Some("Subscribe to account changes".to_string()),
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
            name: "accountUnsubscribe".to_string(),
            description: Some("Unsubscribe from account changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "blockSubscribe".to_string(),
            description: Some("Subscribe to block changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "string",
                        "description": "Filter criteria ('all' or account address)",
                        "default": "all"
                    },
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["confirmed", "finalized"],
                        "default": "finalized"
                    },
                    "encoding": {
                        "type": "string",
                        "description": "Encoding format",
                        "enum": ["json", "jsonParsed", "base58", "base64"],
                        "default": "json"
                    },
                    "transactionDetails": {
                        "type": "string",
                        "description": "Level of transaction detail",
                        "enum": ["full", "accounts", "signatures", "none"],
                        "default": "full"
                    },
                    "showRewards": {
                        "type": "boolean",
                        "description": "Whether to populate rewards array",
                        "default": true
                    }
                }
            }),
        },
        ToolDefinition {
            name: "blockUnsubscribe".to_string(),
            description: Some("Unsubscribe from block changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "logsSubscribe".to_string(),
            description: Some("Subscribe to transaction logs".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "string",
                        "description": "Filter criteria ('all', 'allWithVotes', or account address)",
                        "default": "all"
                    },
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"],
                        "default": "finalized"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "logsUnsubscribe".to_string(),
            description: Some("Unsubscribe from transaction logs".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "programSubscribe".to_string(),
            description: Some("Subscribe to program account changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "program_id": {
                        "type": "string",
                        "description": "Program public key (base58 encoded)"
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
                    },
                    "filters": {
                        "type": "array",
                        "description": "Optional filters to apply",
                        "items": {
                            "type": "object"
                        }
                    }
                },
                "required": ["program_id"]
            }),
        },
        ToolDefinition {
            name: "programUnsubscribe".to_string(),
            description: Some("Unsubscribe from program account changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "rootSubscribe".to_string(),
            description: Some("Subscribe to root changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "rootUnsubscribe".to_string(),
            description: Some("Unsubscribe from root changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "signatureSubscribe".to_string(),
            description: Some("Subscribe to transaction signature confirmations".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "signature": {
                        "type": "string",
                        "description": "Transaction signature (base58 encoded)"
                    },
                    "commitment": {
                        "type": "string",
                        "description": "Commitment level",
                        "enum": ["processed", "confirmed", "finalized"]
                    },
                    "enableReceivedNotification": {
                        "type": "boolean",
                        "description": "Enable notifications when signature is received"
                    }
                },
                "required": ["signature"]
            }),
        },
        ToolDefinition {
            name: "signatureUnsubscribe".to_string(),
            description: Some("Unsubscribe from signature confirmations".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "slotSubscribe".to_string(),
            description: Some("Subscribe to slot changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "slotUnsubscribe".to_string(),
            description: Some("Unsubscribe from slot changes".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "slotsUpdatesSubscribe".to_string(),
            description: Some("Subscribe to slot update notifications".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "slotsUpdatesUnsubscribe".to_string(),
            description: Some("Unsubscribe from slot updates".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        ToolDefinition {
            name: "voteSubscribe".to_string(),
            description: Some("Subscribe to vote notifications".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "voteUnsubscribe".to_string(),
            description: Some("Unsubscribe from vote notifications".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "subscription_id": {
                        "type": "integer",
                        "description": "Subscription ID to cancel"
                    }
                },
                "required": ["subscription_id"]
            }),
        },
        // Network Management Methods
        ToolDefinition {
            name: "listSvmNetworks".to_string(),
            description: Some("List all available SVM networks from awesome-svm repository".to_string()),
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
                        "description": "Network name"
                    },
                    "rpcUrl": {
                        "type": "string",
                        "description": "RPC URL for the network"
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
                        "description": "Network identifier to disable"
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
                        "description": "New RPC URL for the network"
                    }
                },
                "required": ["networkId", "rpcUrl"]
            }),
        },
        ToolDefinition {
            name: "testSbpfProgram".to_string(),
            description: Some("Test an sBPF program locally without deploying to network".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "programBinary": {
                        "type": "string",
                        "description": "Base64-encoded sBPF program binary (ELF format)"
                    },
                    "accounts": {
                        "type": "array",
                        "description": "Accounts to pass to the program",
                        "items": {
                            "type": "object",
                            "properties": {
                                "pubkey": {
                                    "type": "string",
                                    "description": "Account public key (base58)"
                                },
                                "lamports": {
                                    "type": "number",
                                    "description": "Account lamports balance"
                                },
                                "data": {
                                    "type": "string",
                                    "description": "Account data (base64-encoded)"
                                },
                                "owner": {
                                    "type": "string",
                                    "description": "Account owner program ID (base58)"
                                },
                                "executable": {
                                    "type": "boolean",
                                    "description": "Whether the account is executable"
                                },
                                "isSigner": {
                                    "type": "boolean",
                                    "description": "Whether the account is a signer"
                                },
                                "isWritable": {
                                    "type": "boolean",
                                    "description": "Whether the account is writable"
                                }
                            },
                            "required": ["pubkey"]
                        }
                    },
                    "instructionData": {
                        "type": "string",
                        "description": "Instruction data (base64-encoded)"
                    },
                    "signers": {
                        "type": "array",
                        "description": "Signing keypairs",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["programBinary"]
            }),
        },
        ToolDefinition {
            name: "validateSbpfBinary".to_string(),
            description: Some("Validate an sBPF binary without execution".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "programBinary": {
                        "type": "string",
                        "description": "Base64-encoded sBPF program binary (ELF format)"
                    }
                },
                "required": ["programBinary"]
            }),
        },
        ToolDefinition {
            name: "deploySbpfProgramLocal".to_string(),
            description: Some("Deploy an sBPF program to local VM (returns program ID)".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "programBinary": {
                        "type": "string",
                        "description": "Base64-encoded sBPF program binary (ELF format)"
                    }
                },
                "required": ["programBinary"]
            }),
        },
        ToolDefinition {
            name: "prepareDevnetDeploy".to_string(),
            description: Some("Prepare sBPF program for Solana devnet deployment and get CLI instructions".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "programBinary": {
                        "type": "string",
                        "description": "Base64-encoded sBPF program binary (ELF format)"
                    },
                    "rpcUrl": {
                        "type": "string",
                        "description": "Optional custom RPC URL (defaults to devnet)"
                    }
                },
                "required": ["programBinary"]
            }),
        },
        ToolDefinition {
            name: "securityScanSbpfBinary".to_string(),
            description: Some("Perform comprehensive security scan on sBPF program to detect vulnerabilities and risks".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "programBinary": {
                        "type": "string",
                        "description": "Base64-encoded sBPF program binary (ELF format)"
                    }
                },
                "required": ["programBinary"]
            }),
        },
        ToolDefinition {
            name: "getSbpfReadme".to_string(),
            description: Some("Get README documentation for sBPF testing tools".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "getSbpfTutorial".to_string(),
            description: Some("Get step-by-step tutorial for testing sBPF programs".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "Tutorial topic: 'quickstart', 'validation', 'execution', or 'examples'",
                        "enum": ["quickstart", "validation", "execution", "examples"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getSbpfExamples".to_string(),
            description: Some("Get code examples for sBPF testing".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "description": "Programming language: 'curl', 'javascript', 'python', or 'rust'",
                        "enum": ["curl", "javascript", "python", "rust"]
                    }
                }
            }),
        },
        ToolDefinition {
            name: "getSbpfFaq".to_string(),
            description: Some("Get frequently asked questions and troubleshooting for sBPF testing".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ];

    let tools_len = tools.len();
    log::debug!("Returning {tools_len} tools");

    let response = ToolsListResponse {
        tools,
        next_cursor: None,
        meta: None,
    };

    Ok(create_success_response(
        serde_json::to_value(response).unwrap(),
        id.unwrap_or(Value::Null),
    ))
}

/// Handles the tools/call MCP method to execute a specific tool
pub async fn handle_tools_call(
    params: Option<Value>,
    id: Option<Value>,
    state: Arc<RwLock<ServerState>>,
) -> Result<JsonRpcMessage> {
    log::info!("Handling tools/call request");
    
    let params = params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
    
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing tool name parameter"))?;
        
    let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
    
    log::info!("Executing tool: {tool_name}");
    
    // Execute the specific tool based on the tool name
    let result = match tool_name {
        "getHealth" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_health(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Health check failed: {}", e))
        }
        "getVersion" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_version(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Version check failed: {}", e))
        }
        "getBalance" => {
            let pubkey_str = arguments
                .get("pubkey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
            let pubkey = Pubkey::try_from(pubkey_str)?;
            
            let state_guard = state.read().await;
            crate::rpc::accounts::get_balance(state_guard.get_next_rpc_client(), &pubkey).await
                .map_err(|e| anyhow::anyhow!("Get balance failed: {}", e))
        }
        "getAccountInfo" => {
            let pubkey_str = arguments
                .get("pubkey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
            let pubkey = Pubkey::try_from(pubkey_str)?;
            
            let state_guard = state.read().await;
            crate::rpc::accounts::get_account_info(state_guard.get_next_rpc_client(), &pubkey).await
                .map_err(|e| anyhow::anyhow!("Get account info failed: {}", e))
        }
        "getAccountOwner" => {
            let pubkey_str = arguments
                .get("pubkey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
            let pubkey = Pubkey::try_from(pubkey_str)?;
            
            let state_guard = state.read().await;
            let account_info = crate::rpc::accounts::get_account_info(state_guard.get_next_rpc_client(), &pubkey).await
                .map_err(|e| anyhow::anyhow!("Get account info failed: {}", e))?;
            
            // Extract owner from account info
            Ok(serde_json::json!({
                "owner": account_info.get("owner").unwrap_or(&serde_json::Value::Null)
            }))
        }
        "getMultipleAccounts" => {
            let pubkeys_array = arguments
                .get("pubkeys")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkeys parameter"))?;

            let mut pubkeys = Vec::new();
            for pubkey_val in pubkeys_array {
                let pubkey_str = pubkey_val
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid pubkey in array"))?;
                pubkeys.push(Pubkey::try_from(pubkey_str)?);
            }

            let state_guard = state.read().await;
            crate::rpc::accounts::get_multiple_accounts(state_guard.get_next_rpc_client(), &pubkeys).await
                .map_err(|e| anyhow::anyhow!("Get multiple accounts failed: {}", e))
        }
        "getSlot" => {
            log::info!("getSlot: About to acquire state lock");
            let state_guard = state.read().await;
            log::info!("getSlot: State lock acquired");
            let client = state_guard.get_next_rpc_client();
            log::info!("getSlot: Got RPC client, about to call get_slot");
            crate::rpc::blocks::get_slot(client).await
        }
        "getTransactionCount" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_transaction_count(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get transaction count failed: {}", e))
        }
        "getLatestBlockhash" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_latest_blockhash(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get latest blockhash failed: {}", e))
        }
        "getEpochInfo" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_epoch_info(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get epoch info failed: {}", e))
        }
        "getClusterNodes" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_cluster_nodes(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get cluster nodes failed: {}", e))
        }
        // New critical missing methods
        "isBlockhashValid" => {
            let blockhash = arguments.get("blockhash")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing blockhash parameter"))?;
            let commitment = arguments.get("commitment")
                .and_then(|v| v.as_str())
                .map(|c| match c {
                    "processed" => CommitmentConfig::processed(),
                    "confirmed" => CommitmentConfig::confirmed(),
                    "finalized" => CommitmentConfig::finalized(),
                    _ => CommitmentConfig::finalized(),
                });

            let state_guard = state.read().await;
            crate::rpc::system::is_blockhash_valid(state_guard.get_next_rpc_client(), blockhash, commitment).await
                .map_err(|e| anyhow::anyhow!("Check blockhash validity failed: {}", e))
        }
        "getSlotLeader" => {
            let commitment = arguments.get("commitment")
                .and_then(|v| v.as_str())
                .map(|c| match c {
                    "processed" => CommitmentConfig::processed(),
                    "confirmed" => CommitmentConfig::confirmed(),
                    "finalized" => CommitmentConfig::finalized(),
                    _ => CommitmentConfig::finalized(),
                });

            let state_guard = state.read().await;
            crate::rpc::system::get_slot_leader(state_guard.get_next_rpc_client(), commitment).await
                .map_err(|e| anyhow::anyhow!("Get slot leader failed: {}", e))
        }
        "minimumLedgerSlot" => {
            let state_guard = state.read().await;
            crate::rpc::system::minimum_ledger_slot(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get minimum ledger slot failed: {}", e))
        }
        "getMaxRetransmitSlot" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_max_retransmit_slot(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get max retransmit slot failed: {}", e))
        }
        "getMaxShredInsertSlot" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_max_shred_insert_slot(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get max shred insert slot failed: {}", e))
        }
        "getHighestSnapshotSlot" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_highest_snapshot_slot(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get highest snapshot slot failed: {}", e))
        }
        // Deprecated methods
        "getRecentBlockhash" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_recent_blockhash(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get recent blockhash failed: {}", e))
        }
        "getFees" => {
            let state_guard = state.read().await;
            crate::rpc::system::get_fees(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get fees failed: {}", e))
        }
        "getConfirmedBlock" => {
            let state_guard = state.read().await;
            let slot = arguments.get("slot").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing slot parameter"))?;
            crate::rpc::blocks::get_confirmed_block(state_guard.get_next_rpc_client(), slot).await
                .map_err(|e| anyhow::anyhow!("Get confirmed block failed: {}", e))
        }
        "getConfirmedTransaction" => {
            let state_guard = state.read().await;
            let signature_str = arguments.get("signature").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing signature parameter"))?;
            let signature = signature_str.parse()?;
            crate::rpc::transactions::get_confirmed_transaction(state_guard.get_next_rpc_client(), &signature).await
                .map_err(|e| anyhow::anyhow!("Get confirmed transaction failed: {}", e))
        }
        "getConfirmedBlocks" => {
            let state_guard = state.read().await;
            let start_slot = arguments.get("startSlot").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing startSlot parameter"))?;
            let end_slot = arguments.get("endSlot").and_then(|v| v.as_u64());
            crate::rpc::blocks::get_confirmed_blocks(state_guard.get_next_rpc_client(), start_slot, end_slot).await
                .map_err(|e| anyhow::anyhow!("Get confirmed blocks failed: {}", e))
        }
        "getConfirmedBlocksWithLimit" => {
            let state_guard = state.read().await;
            let start_slot = arguments.get("startSlot").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing startSlot parameter"))?;
            let limit = arguments.get("limit").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing limit parameter"))? as usize;
            crate::rpc::blocks::get_confirmed_blocks_with_limit(state_guard.get_next_rpc_client(), start_slot, limit).await
                .map_err(|e| anyhow::anyhow!("Get confirmed blocks with limit failed: {}", e))
        }
        "getConfirmedSignaturesForAddress2" => {
            let state_guard = state.read().await;
            let address_str = arguments.get("address").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing address parameter"))?;
            let address = Pubkey::try_from(address_str)?;
            let limit = arguments.get("limit").and_then(|v| v.as_u64());
            crate::rpc::transactions::get_confirmed_signatures_for_address_2(state_guard.get_next_rpc_client(), &address, None, None, limit).await
                .map_err(|e| anyhow::anyhow!("Get confirmed signatures for address failed: {}", e))
        }
        "getAccountInfoAndContext" => {
            let state_guard = state.read().await;
            let pubkey: String = arguments.get("pubkey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?
                .to_string();
            
            let parsed_pubkey = pubkey.parse::<solana_sdk::pubkey::Pubkey>()
                .map_err(|e| anyhow::anyhow!("Invalid pubkey: {}", e))?;
            
            crate::rpc::accounts::get_account_info_and_context(state_guard.get_next_rpc_client(), &parsed_pubkey)
                .await
                .map_err(|e| anyhow::anyhow!("Get account info with context failed: {}", e))
        }
        "getBalanceAndContext" => {
            let state_guard = state.read().await;
            let pubkey: String = arguments.get("pubkey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?
                .to_string();
            
            let parsed_pubkey = pubkey.parse::<solana_sdk::pubkey::Pubkey>()
                .map_err(|e| anyhow::anyhow!("Invalid pubkey: {}", e))?;
            
            crate::rpc::accounts::get_balance_and_context(state_guard.get_next_rpc_client(), &parsed_pubkey)
                .await
                .map_err(|e| anyhow::anyhow!("Get balance with context failed: {}", e))
        }
        "getMultipleAccountsAndContext" => {
            let state_guard = state.read().await;
            let pubkeys: Vec<String> = arguments.get("pubkeys")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .ok_or_else(|| anyhow::anyhow!("Missing or invalid pubkeys parameter"))?;
            
            let parsed_pubkeys: Result<Vec<_>, _> = pubkeys.iter()
                .map(|key| key.parse::<solana_sdk::pubkey::Pubkey>())
                .collect();
            
            let parsed_pubkeys = parsed_pubkeys
                .map_err(|e| anyhow::anyhow!("Invalid pubkey: {}", e))?;
            
            crate::rpc::accounts::get_multiple_accounts_and_context(state_guard.get_next_rpc_client(), &parsed_pubkeys)
                .await
                .map_err(|e| anyhow::anyhow!("Get multiple accounts with context failed: {}", e))
        }
        "getProgramAccountsAndContext" => {
            let state_guard = state.read().await;
            let program_id: String = arguments.get("program_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing program_id parameter"))?
                .to_string();
            
            let parsed_program_id = program_id.parse::<solana_sdk::pubkey::Pubkey>()
                .map_err(|e| anyhow::anyhow!("Invalid program_id: {}", e))?;
            
            crate::rpc::accounts::get_program_accounts_and_context(state_guard.get_next_rpc_client(), &parsed_program_id, None)
                .await
                .map_err(|e| anyhow::anyhow!("Get program accounts with context failed: {}", e))
        }
        "getRecentPerformanceSamples" => {
            let state_guard = state.read().await;
            let limit = arguments.get("limit")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            
            crate::rpc::system::get_recent_performance_samples(state_guard.get_next_rpc_client(), limit)
                .await
                .map_err(|e| anyhow::anyhow!("Get recent performance samples failed: {}", e))
        }
        "getRecentPrioritizationFees" => {
            let state_guard = state.read().await;
            let addresses: Option<Vec<String>> = arguments.get("addresses")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            
            crate::rpc::system::get_recent_prioritization_fees(state_guard.get_next_rpc_client(), addresses)
                .await
                .map_err(|e| anyhow::anyhow!("Get recent prioritization fees failed: {}", e))
        }
        "getStakeActivation" => {
            let state_guard = state.read().await;
            let pubkey: String = arguments.get("pubkey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?
                .to_string();
            
            let commitment = arguments.get("commitment")
                .and_then(|v| v.as_str())
                .and_then(|s| match s {
                    "processed" => Some(solana_sdk::commitment_config::CommitmentConfig::processed()),
                    "confirmed" => Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
                    "finalized" => Some(solana_sdk::commitment_config::CommitmentConfig::finalized()),
                    _ => None,
                });
            
            crate::rpc::missing_methods::get_stake_activation(state_guard.get_next_rpc_client(), &pubkey, commitment)
                .await
                .map_err(|e| anyhow::anyhow!("Get stake activation failed: {}", e))
        }
        "getSignatureStatuses" => {
            let signatures_array = arguments
                .get("signatures")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing signatures parameter"))?;

            let mut signatures = Vec::new();
            for sig_val in signatures_array {
                let sig_str = sig_val
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid signature in array"))?;
                signatures.push(sig_str.parse()?);
            }

            let search_transaction_history = arguments
                .get("search_transaction_history")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let state_guard = state.read().await;
            crate::rpc::transactions::get_signature_statuses(state_guard.get_next_rpc_client(), &signatures, Some(search_transaction_history)).await
                .map_err(|e| anyhow::anyhow!("Get signature statuses failed: {}", e))
        }
        // Manual RPC methods for missing functionality
        "getBlockCommitment" => {
            let slot = arguments.get("slot").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing slot parameter"))?;

            let state_guard = state.read().await;
            crate::rpc::missing_methods::get_block_commitment(state_guard.get_next_rpc_client(), slot).await
                .map_err(|e| anyhow::anyhow!("Get block commitment failed: {}", e))
        }
        "getSnapshotSlot" => {
            let state_guard = state.read().await;
            crate::rpc::missing_methods::get_snapshot_slot(state_guard.get_next_rpc_client()).await
                .map_err(|e| anyhow::anyhow!("Get snapshot slot failed: {}", e))
        }
        // WebSocket subscription methods  
        "accountSubscribe" => {
            let pubkey_str = arguments.get("pubkey").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
            let _pubkey = pubkey_str.parse::<solana_sdk::pubkey::Pubkey>()?;
            let _commitment = arguments.get("commitment").and_then(|v| v.as_str());
            let _encoding = arguments.get("encoding").and_then(|v| v.as_str());

            // WebSocket subscription - return a subscription ID
            Ok(serde_json::json!({
                "subscription_id": 1,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "accountUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false, 
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "blockSubscribe" => {
            let _filter = arguments.get("filter").and_then(|v| v.as_str()).unwrap_or("all");
            let _commitment = arguments.get("commitment").and_then(|v| v.as_str());
            let _encoding = arguments.get("encoding").and_then(|v| v.as_str());
            let _transaction_details = arguments.get("transactionDetails").and_then(|v| v.as_str());
            let _show_rewards = arguments.get("showRewards").and_then(|v| v.as_bool());

            Ok(serde_json::json!({
                "subscription_id": 2,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "blockUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "logsSubscribe" => {
            let _filter = arguments.get("filter").and_then(|v| v.as_str()).unwrap_or("all");
            let _commitment = arguments.get("commitment").and_then(|v| v.as_str());

            Ok(serde_json::json!({
                "subscription_id": 3,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "logsUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "programSubscribe" => {
            let program_id_str = arguments.get("program_id").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing program_id parameter"))?;
            let _program_id = program_id_str.parse::<solana_sdk::pubkey::Pubkey>()?;
            let _commitment = arguments.get("commitment").and_then(|v| v.as_str());
            let _encoding = arguments.get("encoding").and_then(|v| v.as_str());
            let _filters = arguments.get("filters");

            Ok(serde_json::json!({
                "subscription_id": 4,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "programUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "rootSubscribe" => {
            Ok(serde_json::json!({
                "subscription_id": 5,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "rootUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "signatureSubscribe" => {
            let signature_str = arguments.get("signature").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing signature parameter"))?;
            let _signature = signature_str.parse::<solana_sdk::signature::Signature>()?;
            let _commitment = arguments.get("commitment").and_then(|v| v.as_str());
            let _enable_received_notification = arguments.get("enableReceivedNotification").and_then(|v| v.as_bool());

            Ok(serde_json::json!({
                "subscription_id": 6,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "signatureUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "slotSubscribe" => {
            Ok(serde_json::json!({
                "subscription_id": 7,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "slotUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "slotsUpdatesSubscribe" => {
            Ok(serde_json::json!({
                "subscription_id": 8,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "slotsUpdatesUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "voteSubscribe" => {
            Ok(serde_json::json!({
                "subscription_id": 9,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        "voteUnsubscribe" => {
            let _subscription_id = arguments.get("subscription_id").and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;

            Ok(serde_json::json!({
                "success": false,
                "status": "WebSocket subscriptions require WebSocket connection mode. Use 'solana-mcp-server websocket --port 8900' to enable real-time subscriptions."
            }))
        }
        // Network Management Methods
        "listSvmNetworks" => {
            crate::tools::list_svm_networks().await
                .map_err(|e| anyhow::anyhow!("List SVM networks failed: {}", e))
        }
        "enableSvmNetwork" => {
            let network_id = arguments.get("networkId").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing networkId parameter"))?;
            let name = arguments.get("name").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;
            let rpc_url = arguments.get("rpcUrl").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing rpcUrl parameter"))?;

            crate::tools::enable_svm_network(state.clone(), network_id, name, rpc_url).await
                .map_err(|e| anyhow::anyhow!("Enable SVM network failed: {}", e))
        }
        "disableSvmNetwork" => {
            let network_id = arguments.get("networkId").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing networkId parameter"))?;

            crate::tools::disable_svm_network(state.clone(), network_id).await
                .map_err(|e| anyhow::anyhow!("Disable SVM network failed: {}", e))
        }
        "setNetworkRpcUrl" => {
            let network_id = arguments.get("networkId").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing networkId parameter"))?;
            let rpc_url = arguments.get("rpcUrl").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing rpcUrl parameter"))?;

            crate::tools::set_network_rpc_url(state.clone(), network_id, rpc_url).await
                .map_err(|e| anyhow::anyhow!("Set network RPC URL failed: {}", e))
        }
        "getTokenAccountsByMint" => {
            let mint_str = arguments
                .get("mint")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing mint parameter"))?;
            let mint = Pubkey::try_from(mint_str)?;

            let state_guard = state.read().await;
            crate::rpc::tokens::get_token_accounts_by_mint(state_guard.get_next_rpc_client(), &mint).await
                .map_err(|e| anyhow::anyhow!("Get token accounts by mint failed: {}", e))
        }
        "testSbpfProgram" => {
            let binary_b64 = arguments
                .get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing programBinary parameter"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)
                .map_err(|e| anyhow::anyhow!("Invalid base64: {}", e))?;

            let accounts = arguments
                .get("accounts")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|v| serde_json::from_value::<crate::sbpf::AccountSpec>(v.clone()))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid accounts: {}", e))?
                .unwrap_or_default();

            let instruction_data = arguments
                .get("instructionData")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let signers = arguments
                .get("signers")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let test_params = crate::sbpf::TestParams {
                binary,
                accounts,
                instruction_data,
                signers,
            };

            let executor = crate::sbpf::TestExecutor::new();
            executor.execute_test(test_params).await
                .map(|result| serde_json::to_value(result).unwrap())
                .map_err(|e| anyhow::anyhow!("Test execution failed: {}", e))
        }
        "validateSbpfBinary" => {
            let binary_b64 = arguments
                .get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing programBinary parameter"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)
                .map_err(|e| anyhow::anyhow!("Invalid base64: {}", e))?;

            crate::sbpf::TestExecutor::validate_only(&binary)
                .map(|metadata| serde_json::to_value(metadata).unwrap())
                .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))
        }
        "deploySbpfProgramLocal" => {
            let binary_b64 = arguments
                .get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing programBinary parameter"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)
                .map_err(|e| anyhow::anyhow!("Invalid base64: {}", e))?;

            let vm = crate::sbpf::SbpfVmWrapper::new();
            vm.deploy_program(binary).await
                .map(|response| serde_json::to_value(response).unwrap())
                .map_err(|e| anyhow::anyhow!("Deployment failed: {}", e))
        }
        "prepareDevnetDeploy" => {
            let binary_b64 = arguments
                .get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing programBinary parameter"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)
                .map_err(|e| anyhow::anyhow!("Invalid base64: {}", e))?;

            let rpc_url = arguments
                .get("rpcUrl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let deployer = crate::sbpf::DevnetDeployer::new(rpc_url);
            deployer.prepare_deployment(binary).await
                .map(|response| serde_json::to_value(response).unwrap())
                .map_err(|e| anyhow::anyhow!("Devnet deployment preparation failed: {}", e))
        }
        "securityScanSbpfBinary" => {
            let binary_b64 = arguments
                .get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing programBinary parameter"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)
                .map_err(|e| anyhow::anyhow!("Invalid base64: {}", e))?;

            crate::sbpf::SecurityScanner::scan(&binary)
                .map(|result| serde_json::to_value(result).unwrap())
                .map_err(|e| anyhow::anyhow!("Security scan failed: {}", e))
        }
        "getSbpfReadme" => {
            Ok(serde_json::json!({
                "title": "sBPF Testing Tools - README",
                "description": "Local testing for Solana programs using liteSVM",
                "overview": "The sBPF testing tools allow you to validate and test Solana programs locally without deploying to devnet/testnet/mainnet. This speeds up development cycles and reduces costs.",
                "features": [
                    "Binary validation - Check ELF format and architecture",
                    "Local deployment - Deploy programs to liteSVM",
                    "Program execution - Test with mock accounts and instruction data",
                    "No network required - Everything runs locally",
                    "Fast iteration - Test changes immediately"
                ],
                "tools": {
                    "validateSbpfBinary": "Validates program binary format and structure",
                    "deploySbpfProgramLocal": "Deploys program to local VM and returns program ID",
                    "testSbpfProgram": "Executes program with test accounts and captures logs"
                },
                "requirements": {
                    "binary_format": "ELF format with eBPF architecture (0x107)",
                    "encoding": "Base64-encoded binary data",
                    "size_limits": "64 bytes minimum, 512MB maximum"
                },
                "getting_started": "Use getSbpfTutorial with topic='quickstart' for step-by-step guide"
            }))
        }
        "getSbpfTutorial" => {
            let topic = arguments
                .get("topic")
                .and_then(|v| v.as_str())
                .unwrap_or("quickstart");

            let tutorial = match topic {
                "quickstart" => serde_json::json!({
                    "title": "sBPF Testing - Quick Start",
                    "steps": [
                        {
                            "step": 1,
                            "title": "Compile your Solana program",
                            "description": "Use cargo build-sbf to compile your program to eBPF",
                            "command": "cargo build-sbf",
                            "output": "Binary will be in target/deploy/<program_name>.so"
                        },
                        {
                            "step": 2,
                            "title": "Encode binary to base64",
                            "description": "Convert the .so file to base64 for API transmission",
                            "command": "base64 -w 0 target/deploy/<program_name>.so > program.b64"
                        },
                        {
                            "step": 3,
                            "title": "Validate the binary",
                            "description": "Check that your program is properly formatted",
                            "tool": "validateSbpfBinary",
                            "example": {
                                "jsonrpc": "2.0",
                                "method": "tools/call",
                                "params": {
                                    "name": "validateSbpfBinary",
                                    "arguments": {
                                        "programBinary": "<base64-encoded-binary>"
                                    }
                                },
                                "id": 1
                            }
                        },
                        {
                            "step": 4,
                            "title": "Deploy to local VM",
                            "description": "Get a program ID for testing",
                            "tool": "deploySbpfProgramLocal",
                            "note": "Returns a program ID that can be used in tests"
                        },
                        {
                            "step": 5,
                            "title": "Test program execution",
                            "description": "Run your program with test accounts",
                            "tool": "testSbpfProgram",
                            "note": "Provide accounts, instruction data, and signers"
                        }
                    ],
                    "next": "Use topic='validation' or 'execution' for detailed guides"
                }),
                "validation" => serde_json::json!({
                    "title": "Binary Validation Guide",
                    "description": "Understanding sBPF binary validation",
                    "what_is_validated": [
                        "ELF magic number (0x7F 0x45 0x4C 0x46)",
                        "Architecture type (0x107 for eBPF)",
                        "Binary size (64 bytes to 512MB)",
                        "Section presence (.text required)",
                        "ELF structure integrity"
                    ],
                    "common_errors": {
                        "NotElfFile": "Binary doesn't start with ELF magic number",
                        "NotBpfArchitecture": "Wrong architecture - must be eBPF (0x107)",
                        "BinaryTooSmall": "Binary is less than 64 bytes",
                        "BinaryTooLarge": "Binary exceeds 512MB limit",
                        "InvalidBinary": "ELF structure is malformed"
                    },
                    "success_output": {
                        "architecture": "BPF",
                        "entrypoint": "0x168",
                        "sections": [".text", ".rodata", ".data.rel.ro"],
                        "size_bytes": 21376
                    },
                    "tips": [
                        "Compile with: cargo build-sbf",
                        "Use release builds for accurate size",
                        "Check for .text section in output"
                    ]
                }),
                "execution" => serde_json::json!({
                    "title": "Program Execution Guide",
                    "description": "Testing sBPF programs with testSbpfProgram",
                    "parameters": {
                        "programBinary": {
                            "type": "string",
                            "required": true,
                            "description": "Base64-encoded eBPF binary"
                        },
                        "accounts": {
                            "type": "array",
                            "required": false,
                            "description": "Array of account specifications",
                            "fields": {
                                "pubkey": "Account public key (base58)",
                                "lamports": "Initial SOL balance",
                                "data": "Base64-encoded account data",
                                "owner": "Program that owns the account",
                                "executable": "Whether account is executable",
                                "isSigner": "Whether account signed transaction",
                                "isWritable": "Whether account can be modified"
                            }
                        },
                        "instructionData": {
                            "type": "string",
                            "required": false,
                            "description": "Base64-encoded instruction data"
                        }
                    },
                    "response": {
                        "success": "Boolean indicating execution success",
                        "return_value": "Program return value (if any)",
                        "compute_units": "Compute units consumed",
                        "logs": "Array of program log messages",
                        "account_changes": "Changes to account states"
                    },
                    "example_account": {
                        "pubkey": "11111111111111111111111111111111",
                        "lamports": 1000000,
                        "data": "SGVsbG8=",
                        "owner": "11111111111111111111111111111111",
                        "executable": false,
                        "isSigner": true,
                        "isWritable": true
                    }
                }),
                "examples" => serde_json::json!({
                    "title": "Code Examples",
                    "description": "See getSbpfExamples tool for language-specific examples",
                    "available_languages": ["curl", "javascript", "python", "rust"],
                    "usage": "Call getSbpfExamples with language parameter"
                }),
                _ => serde_json::json!({
                    "error": "Unknown topic",
                    "available_topics": ["quickstart", "validation", "execution", "examples"]
                })
            };

            Ok(tutorial)
        }
        "getSbpfExamples" => {
            let language = arguments
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("curl");

            let examples = match language {
                "curl" => serde_json::json!({
                    "language": "bash",
                    "examples": [
                        {
                            "title": "Validate Binary",
                            "code": "curl -X POST https://solahaha.com/api/mcp \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\n    \"jsonrpc\": \"2.0\",\n    \"method\": \"tools/call\",\n    \"params\": {\n      \"name\": \"validateSbpfBinary\",\n      \"arguments\": {\n        \"programBinary\": \"'$(base64 -w 0 program.so)'\"\n      }\n    },\n    \"id\": 1\n  }'"
                        },
                        {
                            "title": "Test Program",
                            "code": "curl -X POST https://solahaha.com/api/mcp \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\n    \"jsonrpc\": \"2.0\",\n    \"method\": \"tools/call\",\n    \"params\": {\n      \"name\": \"testSbpfProgram\",\n      \"arguments\": {\n        \"programBinary\": \"<base64>\",\n        \"accounts\": [{\n          \"pubkey\": \"11111111111111111111111111111111\",\n          \"lamports\": 1000000,\n          \"isSigner\": true,\n          \"isWritable\": true\n        }],\n        \"instructionData\": \"'$(echo -n \"test\" | base64)'\"\n      }\n    },\n    \"id\": 1\n  }'"
                        }
                    ]
                }),
                "javascript" => serde_json::json!({
                    "language": "javascript",
                    "examples": [
                        {
                            "title": "Validate Binary (Node.js)",
                            "code": "const fs = require('fs');\nconst fetch = require('node-fetch');\n\nconst binary = fs.readFileSync('program.so');\nconst base64Binary = binary.toString('base64');\n\nconst response = await fetch('https://solahaha.com/api/mcp', {\n  method: 'POST',\n  headers: { 'Content-Type': 'application/json' },\n  body: JSON.stringify({\n    jsonrpc: '2.0',\n    method: 'tools/call',\n    params: {\n      name: 'validateSbpfBinary',\n      arguments: { programBinary: base64Binary }\n    },\n    id: 1\n  })\n});\n\nconst result = await response.json();\nconsole.log(result);"
                        }
                    ]
                }),
                "python" => serde_json::json!({
                    "language": "python",
                    "examples": [
                        {
                            "title": "Validate and Test Program",
                            "code": "import base64\nimport requests\n\n# Read and encode binary\nwith open('program.so', 'rb') as f:\n    binary = base64.b64encode(f.read()).decode('utf-8')\n\n# Validate\nresponse = requests.post('https://solahaha.com/api/mcp', json={\n    'jsonrpc': '2.0',\n    'method': 'tools/call',\n    'params': {\n        'name': 'validateSbpfBinary',\n        'arguments': {'programBinary': binary}\n    },\n    'id': 1\n})\n\nprint('Validation:', response.json())\n\n# Test execution\ntest_response = requests.post('https://solahaha.com/api/mcp', json={\n    'jsonrpc': '2.0',\n    'method': 'tools/call',\n    'params': {\n        'name': 'testSbpfProgram',\n        'arguments': {\n            'programBinary': binary,\n            'accounts': [{\n                'pubkey': '11111111111111111111111111111111',\n                'lamports': 1000000,\n                'isSigner': True,\n                'isWritable': True\n            }],\n            'instructionData': base64.b64encode(b'test').decode('utf-8')\n        }\n    },\n    'id': 2\n})\n\nprint('Test result:', test_response.json())"
                        }
                    ]
                }),
                "rust" => serde_json::json!({
                    "language": "rust",
                    "examples": [
                        {
                            "title": "Test Program with reqwest",
                            "code": "use base64::Engine;\nuse serde_json::json;\n\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let binary = std::fs::read(\"program.so\")?;\n    let base64_binary = base64::engine::general_purpose::STANDARD.encode(&binary);\n\n    let client = reqwest::Client::new();\n    let response = client\n        .post(\"https://solahaha.com/api/mcp\")\n        .json(&json!({\n            \"jsonrpc\": \"2.0\",\n            \"method\": \"tools/call\",\n            \"params\": {\n                \"name\": \"validateSbpfBinary\",\n                \"arguments\": {\n                    \"programBinary\": base64_binary\n                }\n            },\n            \"id\": 1\n        }))\n        .send()\n        .await?;\n\n    let result: serde_json::Value = response.json().await?;\n    println!(\"Result: {}\", serde_json::to_string_pretty(&result)?);\n    \n    Ok(())\n}"
                        }
                    ]
                }),
                _ => serde_json::json!({
                    "error": "Unknown language",
                    "available_languages": ["curl", "javascript", "python", "rust"]
                })
            };

            Ok(examples)
        }
        "getSbpfFaq" => {
            Ok(serde_json::json!({
                "title": "sBPF Testing - FAQ & Troubleshooting",
                "questions": [
                    {
                        "q": "Why does my program validation fail?",
                        "a": "Common causes:\n1. Not compiled with cargo build-sbf\n2. Wrong architecture (must be eBPF 0x107)\n3. Binary too small (<64 bytes) or too large (>512MB)\n4. Corrupted or incomplete file\n\nSolution: Ensure you compile with 'cargo build-sbf' and the binary is complete."
                    },
                    {
                        "q": "Why does program execution fail with 'InvalidProgramForExecution'?",
                        "a": "This is a known limitation of liteSVM v0.9. While the binary validates correctly (ELF format, eBPF architecture), liteSVM has stricter runtime requirements that some programs don't meet.\n\nValidation works for:\n- Checking binary format\n- Analyzing sections\n- Extracting metadata\n\nFull execution support is limited. Use validation and deployment tools for binary analysis. For full execution testing, deploy to devnet/testnet.\n\nFuture versions may support more program types."
                    },
                    {
                        "q": "What's the difference between the three core tools?",
                        "a": "validateSbpfBinary: Just checks if the binary is valid (format, architecture, sections). No execution, just analysis.\n\ndeploySbpfProgramLocal: Loads the program into liteSVM and returns a program ID. Validates + deploys.\n\ntestSbpfProgram: Attempts full execution with mock accounts. Validates + deploys + executes. May fail due to liteSVM limitations."
                    },
                    {
                        "q": "How do I encode my binary to base64?",
                        "a": "Linux/Mac: base64 -w 0 program.so > program.b64\n\nWindows: certutil -encode program.so program.b64.tmp && findstr /v CERTIFICATE program.b64.tmp > program.b64\n\nPython: base64.b64encode(open('program.so', 'rb').read()).decode('utf-8')\n\nNode.js: fs.readFileSync('program.so').toString('base64')"
                    },
                    {
                        "q": "What binary format is required?",
                        "a": "Requirements:\n- ELF format (starts with 0x7F 0x45 0x4C 0x46)\n- eBPF architecture (machine type 0x107 or 0xF7)\n- Size: 64 bytes to 512MB\n- Should have .text section\n- Must be compiled with cargo build-sbf\n\nThe .so file from target/deploy/ after cargo build-sbf is correct."
                    },
                    {
                        "q": "Can I test Anchor programs?",
                        "a": "Yes for validation! Anchor programs compile to eBPF and can be validated.\n\nValidation: ✅ Works - checks format and structure\nExecution: ⚠️ Limited - liteSVM may not support all Anchor features\n\nBest use: Validate Anchor binaries to check format and size before deploying to network."
                    },
                    {
                        "q": "Why is my account data not changing?",
                        "a": "If execution succeeds but accounts don't change:\n1. Check isWritable is true for accounts that should change\n2. Verify the program logic actually modifies the accounts\n3. Check compute units consumed - if 0, program may not have run\n4. Review program logs for errors\n\nNote: Due to liteSVM limitations, some state changes may not be captured correctly."
                    },
                    {
                        "q": "What's the best workflow for development?",
                        "a": "Recommended workflow:\n\n1. Write your Solana program\n2. cargo build-sbf\n3. Validate with validateSbpfBinary ← Quick format check\n4. Test locally if possible (may hit limitations)\n5. Deploy to devnet for full testing\n6. Repeat: modify → compile → validate → deploy\n\nUse validation to catch format errors before network deployment."
                    },
                    {
                        "q": "How much does testing cost?",
                        "a": "FREE! All sBPF testing is completely free:\n- No SOL required\n- No network fees\n- No RPC costs\n- Unlimited validations\n- No rate limits currently\n\nThis is why it's great for rapid iteration during development."
                    },
                    {
                        "q": "What are the limitations?",
                        "a": "Current limitations:\n\n1. Execution Support: liteSVM v0.9 has limited program execution support. Validation always works, execution may fail.\n\n2. No CPI: Cross-program invocations not fully supported\n\n3. No Syscalls: Some Solana syscalls may not work\n\n4. Limited State: Fresh VM each time, no persistent state\n\n5. Simplified Runtime: Not identical to real Solana runtime\n\nBest for: Binary validation, format checking, quick smoke tests\nNot for: Comprehensive integration testing, CPI testing, production validation"
                    },
                    {
                        "q": "How do I get help?",
                        "a": "Resources:\n1. getSbpfReadme - Overview and features\n2. getSbpfTutorial - Step-by-step guides\n3. getSbpfExamples - Working code samples\n4. getSbpfFaq - This FAQ (troubleshooting)\n\nGitHub: https://github.com/openSVM/solana-mcp-server/issues\nAPI: https://solahaha.com/api/mcp"
                    }
                ],
                "tips": [
                    "Always compile with cargo build-sbf, not regular cargo build",
                    "Validation is fast and free - use it before every deployment",
                    "If execution fails, use validation to at least check format",
                    "Base64 encode without line breaks (use -w 0 or -A flags)",
                    "Check the .text section exists in validation output",
                    "Deploy to devnet for comprehensive testing"
                ],
                "common_errors": {
                    "NotElfFile": "Binary doesn't start with ELF magic bytes. Wrong file?",
                    "NotBpfArchitecture": "Compiled for wrong architecture. Use cargo build-sbf",
                    "BinaryTooSmall": "File incomplete or wrong file selected",
                    "InvalidBinary": "ELF structure corrupted. Recompile program",
                    "Base64Error": "Binary not properly base64 encoded",
                    "InvalidProgramForExecution": "liteSVM limitation - validation works, execution limited"
                }
            }))
        }
        _ => {
            return Ok(create_error_response(
                -32601,
                format!("Tool not found: {tool_name}"),
                id.unwrap_or(Value::Null),
                None,
            ));
        }
    };
    
    match result {
        Ok(result_value) => Ok(create_success_response(result_value, id.unwrap_or(Value::Null))),
        Err(e) => {
            log::error!("Tool execution failed: {e}");
            Ok(create_error_response(
                -32603,
                format!("Tool execution failed: {e}"),
                id.unwrap_or(Value::Null),
                None,
            ))
        }
    }
}

use solana_sdk::pubkey::Pubkey;

// SVM Network Management Functions

/// Fetches the latest list of SVM networks from the awesome-svm repository
///
/// # Returns
/// * `Result<Value>` - JSON containing available SVM networks
///
/// # Security
/// - Uses HTTPS to fetch network list
/// - Does not cache data to ensure freshness
/// - Validates response format
async fn list_svm_networks() -> Result<Value> {
    let url =
        "https://raw.githubusercontent.com/openSVM/awesome-svm/refs/heads/main/svm-networks.json";
    log::info!("Fetching SVM networks from: {}", sanitize_for_logging(url));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch SVM networks: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch SVM networks: HTTP {}",
            response.status()
        ));
    }

    let networks: Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse SVM networks JSON: {}", e))?;

    log::info!("Successfully fetched SVM networks list");
    Ok(networks)
}

/// Enables an SVM network for use
///
/// # Arguments
/// * `state` - Server state to update
/// * `network_id` - Unique identifier for the network
/// * `name` - Human-readable name for the network
/// * `rpc_url` - RPC endpoint URL (must be HTTPS)
///
/// # Returns
/// * `Result<Value>` - Success/error response
///
/// # Security
/// - Validates network ID format
/// - Validates network name content
/// - Enforces HTTPS for RPC URL
/// - Saves configuration atomically
async fn enable_svm_network(
    state: Arc<RwLock<ServerState>>,
    network_id: &str,
    name: &str,
    rpc_url: &str,
) -> Result<Value> {
    // Validate inputs
    validate_network_id(network_id).map_err(|e| anyhow::anyhow!("Invalid network ID: {}", e))?;

    validate_network_name(name).map_err(|e| anyhow::anyhow!("Invalid network name: {}", e))?;

    validate_rpc_url(rpc_url).map_err(|e| anyhow::anyhow!("Invalid RPC URL: {}", e))?;

    log::info!(
        "Enabling SVM network '{}' ({}): {}",
        network_id,
        name,
        sanitize_for_logging(rpc_url)
    );

    let mut state_guard = state.write().await;

    let network = SvmNetwork {
        name: name.to_string(),
        rpc_url: rpc_url.to_string(),
        enabled: true,
    };

    let mut new_config = state_guard.config.clone();
    new_config
        .svm_networks
        .insert(network_id.to_string(), network);

    // Validate and save configuration
    new_config
        .save()
        .map_err(|e| anyhow::anyhow!("Failed to save configuration: {}", e))?;

    state_guard.update_config(new_config);

    log::info!("Successfully enabled network '{network_id}'");
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Network '{}' enabled successfully", network_id)
    }))
}

/// Disables an SVM network
///
/// # Arguments
/// * `state` - Server state to update
/// * `network_id` - Unique identifier for the network to disable
///
/// # Returns
/// * `Result<Value>` - Success/error response
async fn disable_svm_network(state: Arc<RwLock<ServerState>>, network_id: &str) -> Result<Value> {
    validate_network_id(network_id).map_err(|e| anyhow::anyhow!("Invalid network ID: {}", e))?;

    log::info!("Disabling SVM network '{network_id}'");

    let mut state_guard = state.write().await;

    let mut new_config = state_guard.config.clone();
    if let Some(network) = new_config.svm_networks.get_mut(network_id) {
        network.enabled = false;
    } else {
        return Ok(serde_json::json!({
            "success": false,
            "error": format!("Network '{}' not found", network_id)
        }));
    }

    new_config
        .save()
        .map_err(|e| anyhow::anyhow!("Failed to save configuration: {}", e))?;

    state_guard.update_config(new_config);

    log::info!("Successfully disabled network '{network_id}'");
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Network '{}' disabled successfully", network_id)
    }))
}

/// Sets or updates the RPC URL for an existing network
///
/// # Arguments
/// * `state` - Server state to update
/// * `network_id` - Unique identifier for the network
/// * `rpc_url` - New RPC endpoint URL (must be HTTPS)
///
/// # Returns
/// * `Result<Value>` - Success/error response
///
/// # Security
/// - Validates network ID format
/// - Enforces HTTPS for RPC URL
/// - Validates configuration before saving
async fn set_network_rpc_url(
    state: Arc<RwLock<ServerState>>,
    network_id: &str,
    rpc_url: &str,
) -> Result<Value> {
    validate_network_id(network_id).map_err(|e| anyhow::anyhow!("Invalid network ID: {}", e))?;

    validate_rpc_url(rpc_url).map_err(|e| anyhow::anyhow!("Invalid RPC URL: {}", e))?;

    log::info!(
        "Updating RPC URL for network '{}': {}",
        network_id,
        sanitize_for_logging(rpc_url)
    );

    let mut state_guard = state.write().await;

    let mut new_config = state_guard.config.clone();
    if let Some(network) = new_config.svm_networks.get_mut(network_id) {
        network.rpc_url = rpc_url.to_string();
    } else {
        return Ok(serde_json::json!({
            "success": false,
            "error": format!("Network '{}' not found", network_id)
        }));
    }

    new_config
        .save()
        .map_err(|e| anyhow::anyhow!("Failed to save configuration: {}", e))?;

    state_guard.update_config(new_config);

    log::info!("Successfully updated RPC URL for network '{network_id}'");
    Ok(serde_json::json!({
        "success": true,
        "message": format!("RPC URL for network '{}' updated successfully", network_id)
    }))
}

/// Main request handler for the MCP server
///
/// Parses incoming JSON-RPC requests and routes them to appropriate handlers.
/// Supports all Solana RPC methods plus custom network management functionality.
///
/// # Arguments
/// * `request` - JSON-RPC request string
/// * `state` - Shared server state containing configuration and RPC clients
///
/// # Returns
/// * `Result<JsonRpcMessage>` - JSON-RPC response or error
///
/// # Security
/// - Validates all input parameters
/// - Sanitizes logging output to prevent sensitive data exposure
/// - Enforces HTTPS for all network operations
pub async fn handle_request(
    request: &str,
    state: Arc<RwLock<ServerState>>,
) -> Result<JsonRpcMessage> {
    // Sanitize request for logging to avoid exposing sensitive data
    log::debug!("Received request: {}", sanitize_for_logging(request));
    let message: JsonRpcMessage = serde_json::from_str(request).map_err(|e| {
        log::error!("Failed to parse JSON-RPC request: {e}");
        anyhow::anyhow!("Invalid JSON-RPC request: {}", e)
    })?;

    match message {
        JsonRpcMessage::Request(req) => {
            // First, check protocol version and initialization state with a read lock
            let (protocol_version, initialized) = {
                let state_guard = state.read().await;
                (state_guard.protocol_version.clone(), state_guard.initialized)
            }; // Read lock is dropped here

            let protocol_version = Some(protocol_version.as_str());

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
            if !initialized && req.method.as_str() != "initialize" {
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
                    let state_guard = state.read().await;
                    let response = handle_initialize(
                        req.params,
                        Some(req.id.clone()),
                        &state_guard,
                    )
                    .await?;
                    drop(state_guard); // Drop read lock before acquiring write lock

                    if response.is_success() {
                        let mut state_guard = state.write().await;
                        state_guard.initialized = true;
                        log::info!("Server initialized successfully");
                    } else {
                        log::error!("Server initialization failed");
                    }
                    Ok(response)
                }
                "cancelled" => {
                    let state_guard = state.read().await;
                    handle_cancelled(
                        req.params,
                        Some(req.id.clone()),
                        &state_guard,
                    )
                    .await
                }
                "tools/list" => {
                    let state_guard = state.read().await;
                    handle_tools_list(Some(req.id.clone()), &state_guard)
                        .await
                }
                "tools/call" => {
                    handle_tools_call(req.params, Some(req.id.clone()), state.clone())
                        .await
                }

                "resources/templates/list" => {
                    log::info!("Handling resources/templates/list request");
                    let response = ResourcesListResponse {
                        resources: vec![],
                        next_cursor: None,
                        meta: None,
                    };

                    Ok(create_success_response(
                        serde_json::to_value(response).unwrap(),
                        req.id,
                    ))
                }
                "resources/list" => {
                    log::info!("Handling resources/list request");
                    let resources = vec![Resource {
                        uri: Url::parse("https://docs.solana.com/developing/clients/jsonrpc-api")
                            .unwrap(),
                        name: "Documentation".to_string(),
                        description: Some("Solana API documentation".to_string()),
                        mime_type: Some("text/html".to_string()),
                    }];

                    let response = ResourcesListResponse {
                        resources,
                        next_cursor: None,
                        meta: None,
                    };

                    Ok(create_success_response(
                        serde_json::to_value(response).unwrap(),
                        req.id,
                    ))
                }
                _ => {
                    log::error!("Method not found: {}", req.method);
                    Ok(create_error_response(
                        -32601,
                        "Method not found".to_string(),
                        req.id,
                        protocol_version,
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
