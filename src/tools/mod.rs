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
                {
                    "name": "get_account",
                    "description": "Get account information",
                    "parameters": ["pubkey"]
                },
                {
                    "name": "get_balance",
                    "description": "Get account balance",
                    "parameters": ["pubkey"]
                },
                {
                    "name": "get_block",
                    "description": "Get block information",
                    "parameters": ["slot"]
                },
                {
                    "name": "get_transaction",
                    "description": "Get transaction details",
                    "parameters": ["signature"]
                },
                {
                    "name": "get_token_account",
                    "description": "Get token account information",
                    "parameters": ["pubkey"]
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
