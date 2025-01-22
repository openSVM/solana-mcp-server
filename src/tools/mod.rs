use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

pub async fn handle_tools_list() -> Result<JsonRpcResponse> {
    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::json!({
            "tools": [
                // Account methods
                {
                    "name": "get_account",
                    "description": "Get account information",
                    "parameters": {
                        "pubkey": "Account public key (base58 encoded)"
                    }
                },
                {
                    "name": "get_balance",
                    "description": "Get account balance in lamports",
                    "parameters": {
                        "pubkey": "Account public key (base58 encoded)"
                    }
                },
                {
                    "name": "get_program_accounts",
                    "description": "Get all accounts owned by a program",
                    "parameters": {
                        "program_id": "Program ID (base58 encoded)"
                    }
                },
                // Block methods
                {
                    "name": "get_block",
                    "description": "Get block information",
                    "parameters": {
                        "slot": "Block slot number"
                    }
                },
                {
                    "name": "get_block_height",
                    "description": "Get current block height",
                    "parameters": {}
                },
                {
                    "name": "get_block_production",
                    "description": "Get block production information",
                    "parameters": {
                        "identity": "Optional validator identity (base58 encoded)",
                        "first_slot": "Optional first slot",
                        "last_slot": "Optional last slot"
                    }
                },
                // System methods
                {
                    "name": "get_health",
                    "description": "Get node health status",
                    "parameters": {}
                },
                {
                    "name": "get_version",
                    "description": "Get node version information",
                    "parameters": {}
                },
                {
                    "name": "get_identity",
                    "description": "Get node identity pubkey",
                    "parameters": {}
                },
                {
                    "name": "get_inflation_rate",
                    "description": "Get current inflation rate",
                    "parameters": {}
                },
                // Token methods
                {
                    "name": "get_token_account",
                    "description": "Get token account information",
                    "parameters": {
                        "pubkey": "Token account public key (base58 encoded)"
                    }
                },
                {
                    "name": "get_token_balance",
                    "description": "Get token account balance",
                    "parameters": {
                        "pubkey": "Token account public key (base58 encoded)"
                    }
                },
                {
                    "name": "get_token_supply",
                    "description": "Get token total supply",
                    "parameters": {
                        "mint": "Token mint address (base58 encoded)"
                    }
                },
                {
                    "name": "get_token_largest_accounts",
                    "description": "Get largest token accounts",
                    "parameters": {
                        "mint": "Token mint address (base58 encoded)"
                    }
                },
                // Transaction methods
                {
                    "name": "get_transaction",
                    "description": "Get transaction details",
                    "parameters": {
                        "signature": "Transaction signature (base58 encoded)"
                    }
                },
                {
                    "name": "get_signatures_for_address",
                    "description": "Get signatures for transactions involving an address",
                    "parameters": {
                        "address": "Account address (base58 encoded)",
                        "before": "Optional signature to search before (base58 encoded)",
                        "until": "Optional signature to search until (base58 encoded)",
                        "limit": "Optional result limit"
                    }
                },
                {
                    "name": "send_transaction",
                    "description": "Submit a transaction",
                    "parameters": {
                        "transaction": "Signed transaction (base58 or base64 encoded)",
                        "encoding": "Transaction encoding (base58 or base64)"
                    }
                },
                {
                    "name": "simulate_transaction",
                    "description": "Simulate a transaction",
                    "parameters": {
                        "transaction": "Signed transaction (base58 or base64 encoded)",
                        "encoding": "Transaction encoding (base58 or base64)"
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
                },
                {
                    "name": "token_list",
                    "description": "List of known tokens",
                    "uri": "solana://tokens"
                },
                {
                    "name": "guides",
                    "description": "Development guides",
                    "uri": "solana://docs/guides"
                }
            ]
        })),
        error: None,
        id: None,
    })
}

pub async fn handle_request(request: &str) -> Result<JsonRpcResponse> {
    let request: JsonRpcRequest = match serde_json::from_str(request) {
        Ok(req) => req,
        Err(_) => {
            return Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32600,
                    message: "Invalid Request: jsonrpc version must be 2.0".to_string(),
                    data: None,
                }),
                id: None,
            });
        }
    };

    if request.jsonrpc != "2.0" {
        return Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid Request: jsonrpc version must be 2.0".to_string(),
                data: None,
            }),
            id: None,
        });
    }

    match request.method.as_str() {
        "tools/list" => handle_tools_list().await,
        _ => Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
            id: request.id,
        }),
    }
}
