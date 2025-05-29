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
    #[allow(dead_code)]
    request_id: i64,
    #[allow(dead_code)]
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

                    // Additional Account Methods
                    tools.insert("getMultipleAccounts".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getLargestAccounts".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getMinimumBalanceForRentExemption".to_string(), ToolDefinition {
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
                    });

                    // Block Methods
                    tools.insert("getSlot".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getBlock".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getBlockHeight".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getBlocks".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getFirstAvailableBlock".to_string(), ToolDefinition {
                        name: "getFirstAvailableBlock".to_string(),
                        description: Some("Returns the lowest confirmed block still available".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });

                    tools.insert("getGenesisHash".to_string(), ToolDefinition {
                        name: "getGenesisHash".to_string(),
                        description: Some("Returns the genesis hash of the ledger".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });

                    // System Methods
                    tools.insert("getIdentity".to_string(), ToolDefinition {
                        name: "getIdentity".to_string(),
                        description: Some("Returns identity pubkey for the current node".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });

                    tools.insert("getEpochInfo".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getLatestBlockhash".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getSupply".to_string(), ToolDefinition {
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
                    });

                    // Transaction Methods
                    tools.insert("getSignaturesForAddress".to_string(), ToolDefinition {
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
                    });

                    tools.insert("sendTransaction".to_string(), ToolDefinition {
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
                    });

                    tools.insert("simulateTransaction".to_string(), ToolDefinition {
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
                    });

                    // Token Methods
                    tools.insert("getTokenAccountsByOwner".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getTokenSupply".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getTokenAccountBalance".to_string(), ToolDefinition {
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
                    });

                    // Additional Block Methods
                    tools.insert("getSlotLeaders".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getBlockProduction".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getVoteAccounts".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getLeaderSchedule".to_string(), ToolDefinition {
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
                    });

                    // Additional System Methods
                    tools.insert("getClusterNodes".to_string(), ToolDefinition {
                        name: "getClusterNodes".to_string(),
                        description: Some("Returns information about all cluster nodes".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });

                    tools.insert("getEpochSchedule".to_string(), ToolDefinition {
                        name: "getEpochSchedule".to_string(),
                        description: Some("Returns epoch schedule information".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });

                    tools.insert("getInflationGovernor".to_string(), ToolDefinition {
                        name: "getInflationGovernor".to_string(),
                        description: Some("Returns current inflation governor".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });

                    tools.insert("getInflationRate".to_string(), ToolDefinition {
                        name: "getInflationRate".to_string(),
                        description: Some("Returns specific inflation values for current epoch".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    });

                    tools.insert("getInflationReward".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getTransactionCount".to_string(), ToolDefinition {
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
                    });

                    tools.insert("requestAirdrop".to_string(), ToolDefinition {
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
                    });

                    // Additional Transaction Methods
                    tools.insert("getBlockTime".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getFeeForMessage".to_string(), ToolDefinition {
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
                    });

                    // Additional Token Methods
                    tools.insert("getTokenAccountsByDelegate".to_string(), ToolDefinition {
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
                    });

                    tools.insert("getTokenLargestAccounts".to_string(), ToolDefinition {
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
                    });

                    // Additional Block and Slot Methods
                    tools.insert("getBlocksWithLimit".to_string(), ToolDefinition {
                        name: "getBlocksWithLimit".to_string(),
                        description: Some("Returns a list of confirmed blocks starting at given slot".to_string()),
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
                    });

                    tools.insert("getStakeMinimumDelegation".to_string(), ToolDefinition {
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
                    });

                    // Additional complex transaction method
                    tools.insert("getTransactionWithConfig".to_string(), ToolDefinition {
                        name: "getTransactionWithConfig".to_string(),
                        description: Some("Returns transaction details with additional configuration".to_string()),
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
        let _cancel_params: CancelledParams = serde_json::from_value(params)?;
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
            description: Some("Returns a list of confirmed blocks starting at given slot".to_string()),
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
            description: Some("Returns transaction details with additional configuration".to_string()),
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
                
                // Additional Account Methods
                "getMultipleAccounts" => {
                    log::info!("Getting multiple accounts");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let pubkeys_array = params.get("pubkeys")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| anyhow::anyhow!("Missing pubkeys parameter"))?;
                    
                    let mut pubkeys = Vec::new();
                    for pubkey_val in pubkeys_array {
                        let pubkey_str = pubkey_val.as_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid pubkey in array"))?;
                        pubkeys.push(Pubkey::try_from(pubkey_str)?);
                    }
                    
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => Some(solana_sdk::commitment_config::CommitmentConfig::processed()),
                            "confirmed" => Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
                            "finalized" => Some(solana_sdk::commitment_config::CommitmentConfig::finalized()),
                            _ => None,
                        };
                        let encoding = params.get("encoding").and_then(|v| v.as_str()).map(|e| match e {
                            "base58" => solana_account_decoder::UiAccountEncoding::Base58,
                            "base64" => solana_account_decoder::UiAccountEncoding::Base64,
                            "jsonParsed" => solana_account_decoder::UiAccountEncoding::JsonParsed,
                            _ => solana_account_decoder::UiAccountEncoding::Base64,
                        });
                        crate::rpc::accounts::get_multiple_accounts_with_config(&state.rpc_client, &pubkeys, commitment, encoding).await?
                    } else {
                        crate::rpc::accounts::get_multiple_accounts(&state.rpc_client, &pubkeys).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getLargestAccounts" => {
                    log::info!("Getting largest accounts");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let filter = params.get("filter").and_then(|v| v.as_str()).map(|f| match f {
                        "circulating" => solana_client::rpc_config::RpcLargestAccountsFilter::Circulating,
                        "nonCirculating" => solana_client::rpc_config::RpcLargestAccountsFilter::NonCirculating,
                        _ => solana_client::rpc_config::RpcLargestAccountsFilter::Circulating,
                    });
                    
                    let state = state.read().await;
                    let result = crate::rpc::accounts::get_largest_accounts(&state.rpc_client, filter).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getMinimumBalanceForRentExemption" => {
                    log::info!("Getting minimum balance for rent exemption");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let data_size = params.get("dataSize")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing dataSize parameter"))? as usize;
                    
                    let state = state.read().await;
                    let result = crate::rpc::accounts::get_minimum_balance_for_rent_exemption(&state.rpc_client, data_size).await?;
                    Ok(create_success_response(result, req.id))
                },

                // Block Methods
                "getSlot" => {
                    log::info!("Getting current slot");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::blocks::get_slot_with_commitment(&state.rpc_client, commitment).await?
                    } else {
                        crate::rpc::blocks::get_slot(&state.rpc_client).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getBlock" => {
                    log::info!("Getting block");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let slot = params.get("slot")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing slot parameter"))?;
                    
                    let state = state.read().await;
                    let result = if params.get("encoding").is_some() || params.get("transactionDetails").is_some() || 
                                   params.get("rewards").is_some() || params.get("commitment").is_some() {
                        let encoding = params.get("encoding").and_then(|v| v.as_str()).map(|e| match e {
                            "json" => solana_transaction_status::UiTransactionEncoding::Json,
                            "jsonParsed" => solana_transaction_status::UiTransactionEncoding::JsonParsed,
                            "base58" => solana_transaction_status::UiTransactionEncoding::Base58,
                            "base64" => solana_transaction_status::UiTransactionEncoding::Base64,
                            _ => solana_transaction_status::UiTransactionEncoding::Json,
                        });
                        let transaction_details = params.get("transactionDetails").and_then(|v| v.as_str()).map(|td| match td {
                            "full" => solana_transaction_status::TransactionDetails::Full,
                            "signatures" => solana_transaction_status::TransactionDetails::Signatures,
                            "none" => solana_transaction_status::TransactionDetails::None,
                            _ => solana_transaction_status::TransactionDetails::Full,
                        });
                        let rewards = params.get("rewards").and_then(|v| v.as_bool());
                        let commitment = params.get("commitment").and_then(|v| v.as_str()).map(|c| match c {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        });
                        crate::rpc::blocks::get_block_with_config(&state.rpc_client, slot, encoding, transaction_details, rewards, commitment).await?
                    } else {
                        crate::rpc::blocks::get_block(&state.rpc_client, slot).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getBlockHeight" => {
                    log::info!("Getting block height");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::blocks::get_block_height_with_commitment(&state.rpc_client, commitment).await?
                    } else {
                        crate::rpc::blocks::get_block_height(&state.rpc_client).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getBlocks" => {
                    log::info!("Getting blocks");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let start_slot = params.get("startSlot")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing startSlot parameter"))?;
                    let end_slot = params.get("endSlot").and_then(|v| v.as_u64());
                    
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::blocks::get_blocks_with_commitment(&state.rpc_client, start_slot, end_slot, commitment).await?
                    } else {
                        crate::rpc::blocks::get_blocks(&state.rpc_client, start_slot, end_slot).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getFirstAvailableBlock" => {
                    log::info!("Getting first available block");
                    let state = state.read().await;
                    let result = crate::rpc::blocks::get_first_available_block(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getGenesisHash" => {
                    log::info!("Getting genesis hash");
                    let state = state.read().await;
                    let result = crate::rpc::blocks::get_genesis_hash(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                // System Methods
                "getIdentity" => {
                    log::info!("Getting node identity");
                    let state = state.read().await;
                    let result = crate::rpc::system::get_identity(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getEpochInfo" => {
                    log::info!("Getting epoch info");
                    let _params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let state = state.read().await;
                    let result = crate::rpc::system::get_epoch_info(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getLatestBlockhash" => {
                    log::info!("Getting latest blockhash");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::system::get_latest_blockhash_with_commitment(&state.rpc_client, commitment).await?
                    } else {
                        crate::rpc::system::get_latest_blockhash(&state.rpc_client).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getSupply" => {
                    log::info!("Getting supply");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::system::get_supply_with_commitment(&state.rpc_client, commitment).await?
                    } else {
                        crate::rpc::system::get_supply(&state.rpc_client).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                // Transaction Methods
                "getSignaturesForAddress" => {
                    log::info!("Getting signatures for address");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let address_str = params.get("address")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing address parameter"))?;
                    let address = Pubkey::try_from(address_str)?;
                    
                    let before = params.get("before").and_then(|v| v.as_str()).and_then(|s| s.parse().ok());
                    let until = params.get("until").and_then(|v| v.as_str()).and_then(|s| s.parse().ok());
                    let limit = params.get("limit").and_then(|v| v.as_u64());
                    
                    let state = state.read().await;
                    let result = crate::rpc::transactions::get_signatures_for_address(&state.rpc_client, &address, before, until, limit).await?;
                    Ok(create_success_response(result, req.id))
                },

                "sendTransaction" => {
                    log::info!("Sending transaction");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let transaction_data = params.get("transaction")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing transaction parameter"))?;
                    let encoding = params.get("encoding").and_then(|v| v.as_str()).unwrap_or("base64");
                    
                    let state = state.read().await;
                    let result = if params.get("skipPreflight").is_some() || params.get("maxRetries").is_some() {
                        let skip_preflight = params.get("skipPreflight").and_then(|v| v.as_bool()).unwrap_or(false);
                        let max_retries = params.get("maxRetries").and_then(|v| v.as_u64()).map(|r| r as usize);
                        crate::rpc::transactions::send_transaction_with_config(
                            &state.rpc_client, 
                            transaction_data, 
                            encoding, 
                            skip_preflight, 
                            None, 
                            max_retries, 
                            None
                        ).await?
                    } else {
                        crate::rpc::transactions::send_transaction(&state.rpc_client, transaction_data, encoding).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "simulateTransaction" => {
                    log::info!("Simulating transaction");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let transaction_data = params.get("transaction")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing transaction parameter"))?;
                    let encoding = params.get("encoding").and_then(|v| v.as_str()).unwrap_or("base64");
                    
                    let state = state.read().await;
                    let result = if params.get("sigVerify").is_some() || params.get("commitment").is_some() {
                        let sig_verify = params.get("sigVerify").and_then(|v| v.as_bool()).unwrap_or(true);
                        let commitment = params.get("commitment").and_then(|v| v.as_str()).map(|c| match c {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        });
                        crate::rpc::transactions::simulate_transaction_with_config(
                            &state.rpc_client, 
                            transaction_data, 
                            encoding, 
                            sig_verify, 
                            commitment, 
                            false, 
                            None, 
                            None
                        ).await?
                    } else {
                        crate::rpc::transactions::simulate_transaction(&state.rpc_client, transaction_data, encoding).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                // Token Methods
                "getTokenAccountsByOwner" => {
                    log::info!("Getting token accounts by owner");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let owner_str = params.get("owner")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing owner parameter"))?;
                    let owner = Pubkey::try_from(owner_str)?;
                    
                    let state = state.read().await;
                    let result = crate::rpc::tokens::get_token_accounts_by_owner(&state.rpc_client, &owner).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getTokenSupply" => {
                    log::info!("Getting token supply");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let mint_str = params.get("mint")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing mint parameter"))?;
                    let mint = Pubkey::try_from(mint_str)?;
                    
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::tokens::get_token_supply_with_commitment(&state.rpc_client, &mint, commitment).await?
                    } else {
                        crate::rpc::tokens::get_token_supply(&state.rpc_client, &mint).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getTokenAccountBalance" => {
                    log::info!("Getting token account balance");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let account_str = params.get("account")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing account parameter"))?;
                    let account = Pubkey::try_from(account_str)?;
                    
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::tokens::get_token_account_balance_with_commitment(&state.rpc_client, &account, commitment).await?
                    } else {
                        crate::rpc::tokens::get_token_account_balance(&state.rpc_client, &account).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                // Additional Block Methods
                "getSlotLeaders" => {
                    log::info!("Getting slot leaders");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let start_slot = params.get("startSlot")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing startSlot parameter"))?;
                    let limit = params.get("limit")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing limit parameter"))?;
                    
                    let state = state.read().await;
                    let result = crate::rpc::blocks::get_slot_leaders(&state.rpc_client, start_slot, limit).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getBlockProduction" => {
                    log::info!("Getting block production");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let identity = params.get("identity").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let first_slot = params.get("firstSlot").and_then(|v| v.as_u64());
                    let last_slot = params.get("lastSlot").and_then(|v| v.as_u64());
                    
                    let state = state.read().await;
                    let result = crate::rpc::blocks::get_block_production(&state.rpc_client, identity, first_slot, last_slot).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getVoteAccounts" => {
                    log::info!("Getting vote accounts");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let commitment = params.get("commitment").and_then(|v| v.as_str()).map(|c| match c {
                        "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                        "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                        "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                    });
                    let vote_pubkey = params.get("votePubkey").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let keep_unstaked_delinquents = params.get("keepUnstakedDelinquents").and_then(|v| v.as_bool());
                    
                    let state = state.read().await;
                    let result = if commitment.is_some() || vote_pubkey.is_some() || keep_unstaked_delinquents.is_some() {
                        crate::rpc::blocks::get_vote_accounts_with_config(&state.rpc_client, commitment, vote_pubkey, keep_unstaked_delinquents, None).await?
                    } else {
                        crate::rpc::blocks::get_vote_accounts(&state.rpc_client).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getLeaderSchedule" => {
                    log::info!("Getting leader schedule");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let slot = params.get("slot").and_then(|v| v.as_u64());
                    let identity = params.get("identity").and_then(|v| v.as_str()).map(|s| s.to_string());
                    
                    let state = state.read().await;
                    let result = crate::rpc::blocks::get_leader_schedule(&state.rpc_client, slot, identity).await?;
                    Ok(create_success_response(result, req.id))
                },

                // Additional System Methods
                "getClusterNodes" => {
                    log::info!("Getting cluster nodes");
                    let state = state.read().await;
                    let result = crate::rpc::system::get_cluster_nodes(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getEpochSchedule" => {
                    log::info!("Getting epoch schedule");
                    let state = state.read().await;
                    let result = crate::rpc::system::get_epoch_schedule(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getInflationGovernor" => {
                    log::info!("Getting inflation governor");
                    let state = state.read().await;
                    let result = crate::rpc::system::get_inflation_governor(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getInflationRate" => {
                    log::info!("Getting inflation rate");
                    let state = state.read().await;
                    let result = crate::rpc::system::get_inflation_rate(&state.rpc_client).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getInflationReward" => {
                    log::info!("Getting inflation reward");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let addresses_array = params.get("addresses")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| anyhow::anyhow!("Missing addresses parameter"))?;
                    
                    let mut addresses = Vec::new();
                    for addr_val in addresses_array {
                        let addr_str = addr_val.as_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid address in array"))?;
                        addresses.push(Pubkey::try_from(addr_str)?);
                    }
                    
                    let epoch = params.get("epoch").and_then(|v| v.as_u64());
                    
                    let state = state.read().await;
                    let result = crate::rpc::system::get_inflation_reward(&state.rpc_client, &addresses, epoch).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getTransactionCount" => {
                    log::info!("Getting transaction count");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::system::get_transaction_count_with_commitment(&state.rpc_client, commitment).await?
                    } else {
                        crate::rpc::system::get_transaction_count(&state.rpc_client).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "requestAirdrop" => {
                    log::info!("Requesting airdrop");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let pubkey_str = params.get("pubkey")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
                    let pubkey = Pubkey::try_from(pubkey_str)?;
                    let lamports = params.get("lamports")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing lamports parameter"))?;
                    
                    let state = state.read().await;
                    let result = crate::rpc::system::request_airdrop(&state.rpc_client, &pubkey, lamports).await?;
                    Ok(create_success_response(result, req.id))
                },

                // Additional Transaction Methods
                "getBlockTime" => {
                    log::info!("Getting block time");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let slot = params.get("slot")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing slot parameter"))?;
                    
                    let state = state.read().await;
                    let result = crate::rpc::transactions::get_block_time(&state.rpc_client, slot).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getFeeForMessage" => {
                    log::info!("Getting fee for message");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let message_data = params.get("message")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing message parameter"))?;
                    let encoding = params.get("encoding").and_then(|v| v.as_str()).unwrap_or("base64");
                    
                    let state = state.read().await;
                    let result = crate::rpc::transactions::get_fee_for_message(&state.rpc_client, message_data, encoding).await?;
                    Ok(create_success_response(result, req.id))
                },

                // Additional Token Methods
                "getTokenAccountsByDelegate" => {
                    log::info!("Getting token accounts by delegate");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let delegate_str = params.get("delegate")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing delegate parameter"))?;
                    let delegate = Pubkey::try_from(delegate_str)?;
                    
                    let filter = if let Some(mint_str) = params.get("mint").and_then(|v| v.as_str()) {
                        let mint = Pubkey::try_from(mint_str)?;
                        solana_client::rpc_request::TokenAccountsFilter::Mint(mint)
                    } else if let Some(program_id_str) = params.get("programId").and_then(|v| v.as_str()) {
                        let program_id = Pubkey::try_from(program_id_str)?;
                        solana_client::rpc_request::TokenAccountsFilter::ProgramId(program_id)
                    } else {
                        solana_client::rpc_request::TokenAccountsFilter::ProgramId(spl_token::id())
                    };
                    
                    let state = state.read().await;
                    let result = crate::rpc::tokens::get_token_accounts_by_delegate(&state.rpc_client, &delegate, filter).await?;
                    Ok(create_success_response(result, req.id))
                },

                "getTokenLargestAccounts" => {
                    log::info!("Getting token largest accounts");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let mint_str = params.get("mint")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing mint parameter"))?;
                    let mint = Pubkey::try_from(mint_str)?;
                    
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::tokens::get_token_largest_accounts_with_commitment(&state.rpc_client, &mint, commitment).await?
                    } else {
                        crate::rpc::tokens::get_token_largest_accounts(&state.rpc_client, &mint).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                // Additional Block and Slot Methods
                "getBlocksWithLimit" => {
                    log::info!("Getting blocks with limit");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let start_slot = params.get("startSlot")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing startSlot parameter"))?;
                    let limit = params.get("limit")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow::anyhow!("Missing limit parameter"))? as usize;
                    
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::blocks::get_blocks_with_limit_and_commitment(&state.rpc_client, start_slot, limit, commitment).await?
                    } else {
                        crate::rpc::blocks::get_blocks_with_limit(&state.rpc_client, start_slot, limit).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getStakeMinimumDelegation" => {
                    log::info!("Getting stake minimum delegation");
                    let params = req.params.unwrap_or_else(|| serde_json::json!({}));
                    let state = state.read().await;
                    let result = if let Some(commitment_str) = params.get("commitment").and_then(|v| v.as_str()) {
                        let commitment = match commitment_str {
                            "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                            "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                            "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                            _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        };
                        crate::rpc::system::get_stake_minimum_delegation_with_commitment(&state.rpc_client, commitment).await?
                    } else {
                        crate::rpc::system::get_stake_minimum_delegation(&state.rpc_client).await?
                    };
                    Ok(create_success_response(result, req.id))
                },

                "getTransactionWithConfig" => {
                    log::info!("Getting transaction with config");
                    let params = req.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                    let signature_str = params.get("signature")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing signature parameter"))?;
                    let signature = signature_str.parse()?;
                    
                    let encoding = params.get("encoding").and_then(|v| v.as_str()).map(|e| match e {
                        "json" => solana_transaction_status::UiTransactionEncoding::Json,
                        "jsonParsed" => solana_transaction_status::UiTransactionEncoding::JsonParsed,
                        "base58" => solana_transaction_status::UiTransactionEncoding::Base58,
                        "base64" => solana_transaction_status::UiTransactionEncoding::Base64,
                        _ => solana_transaction_status::UiTransactionEncoding::Json,
                    }).unwrap_or(solana_transaction_status::UiTransactionEncoding::Json);
                    
                    let commitment = params.get("commitment").and_then(|v| v.as_str()).map(|c| match c {
                        "processed" => solana_sdk::commitment_config::CommitmentConfig::processed(),
                        "confirmed" => solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                        "finalized" => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                        _ => solana_sdk::commitment_config::CommitmentConfig::finalized(),
                    });
                    
                    let max_supported_transaction_version = params.get("maxSupportedTransactionVersion").and_then(|v| v.as_u64()).map(|v| v as u8);
                    
                    let state = state.read().await;
                    let result = crate::rpc::transactions::get_transaction_with_config(&state.rpc_client, &signature, encoding, commitment, max_supported_transaction_version).await?;
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
