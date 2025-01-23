use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde_json::json;
use mcp_sdk::transport::{Transport, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcVersion, JsonRpcError};
use std::sync::mpsc::{self, Sender, Receiver};
use anyhow::Result;

struct TestTransport {
    tx: Sender<serde_json::Value>,
    rx: Arc<Mutex<Receiver<serde_json::Value>>>,
}

impl TestTransport {
    fn new() -> (Self, Self) {
        let (client_tx, server_rx) = mpsc::channel();
        let (server_tx, client_rx) = mpsc::channel();
        
        let client = Self {
            tx: client_tx,
            rx: Arc::new(Mutex::new(client_rx)),
        };
        
        let server = Self {
            tx: server_tx,
            rx: Arc::new(Mutex::new(server_rx)),
        };
        
        (client, server)
    }
}

impl Transport for TestTransport {
    fn send(&self, message: &JsonRpcMessage) -> Result<()> {
        let json = match message {
            JsonRpcMessage::Request(req) => {
                json!({
                    "jsonrpc": JsonRpcVersion::V2,
                    "id": req.id,
                    "method": req.method,
                    "params": req.params
                })
            },
            JsonRpcMessage::Response(resp) => {
                json!({
                    "jsonrpc": JsonRpcVersion::V2,
                    "id": resp.id,
                    "result": resp.result,
                    "error": resp.error
                })
            },
            JsonRpcMessage::Notification(_) => {
                json!({})
            }
        };

        self.tx.send(json).unwrap();
        Ok(())
    }

    fn receive(&self) -> Result<JsonRpcMessage> {
        let value = match self.rx.lock().unwrap().recv_timeout(Duration::from_secs(5)) {
            Ok(value) => value,
            Err(_) => panic!("Failed to receive message within timeout"),
        };

        if value.get("error").is_some() {
            Ok(JsonRpcMessage::Response(JsonRpcResponse {
                jsonrpc: JsonRpcVersion::default(),
                id: value["id"].as_u64().unwrap(),
                result: None,
                error: Some(JsonRpcError {
                    code: value["error"]["code"].as_i64().unwrap() as i32,
                    message: value["error"]["message"].as_str().unwrap().to_string(),
                    data: value["error"].get("data").cloned(),
                }),
            }))
        } else if value.get("result").is_some() {
            Ok(JsonRpcMessage::Response(JsonRpcResponse {
                jsonrpc: JsonRpcVersion::default(),
                id: value["id"].as_u64().unwrap(),
                result: Some(value["result"].clone()),
                error: None,
            }))
        } else {
            Ok(JsonRpcMessage::Request(JsonRpcRequest {
                jsonrpc: JsonRpcVersion::default(),
                id: value["id"].as_u64().unwrap(),
                method: value["method"].as_str().unwrap().to_string(),
                params: value.get("params").cloned(),
            }))
        }
    }

    fn open(&self) -> Result<()> {
        Ok(())
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }
}

fn setup_mock_server() -> TestTransport {
    let (client_transport, server_transport) = TestTransport::new();
    
    thread::spawn(move || {
        loop {
            if let Ok(msg) = server_transport.rx.lock().unwrap().recv() {
                let id = msg["id"].as_u64().unwrap();
                let method = msg["method"].as_str().unwrap();
                let auth = msg["params"]["auth"].as_str().unwrap();

                match (method, auth) {
                    ("initialize", "test_key_123") => {
                        server_transport.tx.send(json!({
                            "jsonrpc": JsonRpcVersion::V2,
                            "id": id,
                            "result": {
                                "server": {
                                    "name": "solana-mcp",
                                    "version": "0.1.2"
                                },
                                "protocol": {
                                    "name": "mcp",
                                    "version": "0.1.0"
                                }
                            }
                        })).unwrap();
                    },
                    (_, "invalid_key") => {
                        server_transport.tx.send(json!({
                            "jsonrpc": JsonRpcVersion::V2,
                            "id": id,
                            "error": {
                                "code": -32601,
                                "message": "Invalid API key"
                            }
                        })).unwrap();
                    },
                    ("tools/call", "test_key_123") => {
                        let tool_name = msg["params"]["name"].as_str().unwrap();
                        match tool_name {
                            "get_slot" => {
                                server_transport.tx.send(json!({
                                    "jsonrpc": JsonRpcVersion::V2,
                                    "id": id,
                                    "result": 12345
                                })).unwrap();
                            },
                            _ => {
                                server_transport.tx.send(json!({
                                    "jsonrpc": JsonRpcVersion::V2,
                                    "id": id,
                                    "error": {
                                        "code": -32601,
                                        "message": "Tool not found"
                                    }
                                })).unwrap();
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    });

    client_transport
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_server_initialization() {
    let transport = setup_mock_server();

    let request = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        id: 1,
        method: "initialize".to_string(),
        params: Some(json!({
            "auth": "test_key_123"
        })),
    });

    transport.send(&request).unwrap();
    let response = transport.receive().unwrap();

    match response {
        JsonRpcMessage::Response(resp) => {
            assert_eq!(resp.jsonrpc, JsonRpcVersion::V2);
            assert_eq!(resp.id, 1);
            let result = resp.result.unwrap();
            assert!(result["server"]["name"].as_str().unwrap().contains("solana-mcp"));
            assert_eq!(result["protocol"]["name"].as_str().unwrap(), "mcp");
        },
        _ => panic!("Expected response message"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_invalid_api_key() {
    let transport = setup_mock_server();

    let request = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        id: 1,
        method: "tools/list".to_string(),
        params: Some(json!({
            "auth": "invalid_key"
        })),
    });

    transport.send(&request).unwrap();
    let response = transport.receive().unwrap();

    match response {
        JsonRpcMessage::Response(resp) => {
            assert_eq!(resp.jsonrpc, JsonRpcVersion::V2);
            assert_eq!(resp.id, 1);
            let error = resp.error.unwrap();
            assert_eq!(error.code, -32601);
            assert!(error.message.contains("Invalid API key"));
        },
        _ => panic!("Expected error message"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_tool_execution() {
    let transport = setup_mock_server();

    // First initialize
    let init_request = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        id: 1,
        method: "initialize".to_string(),
        params: Some(json!({
            "auth": "test_key_123"
        })),
    });

    transport.send(&init_request).unwrap();
    transport.receive().unwrap();

    // Then call tool
    let request = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        id: 2,
        method: "tools/call".to_string(),
        params: Some(json!({
            "auth": "test_key_123",
            "name": "get_slot",
            "arguments": {}
        })),
    });

    transport.send(&request).unwrap();
    let response = transport.receive().unwrap();

    match response {
        JsonRpcMessage::Response(resp) => {
            assert_eq!(resp.jsonrpc, JsonRpcVersion::V2);
            assert_eq!(resp.id, 2);
            assert!(resp.result.is_some());
        },
        _ => panic!("Expected response message"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_invalid_tool() {
    let transport = setup_mock_server();

    // First initialize
    let init_request = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        id: 1,
        method: "initialize".to_string(),
        params: Some(json!({
            "auth": "test_key_123"
        })),
    });

    transport.send(&init_request).unwrap();
    transport.receive().unwrap();

    // Then call invalid tool
    let request = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        id: 2,
        method: "tools/call".to_string(),
        params: Some(json!({
            "auth": "test_key_123",
            "name": "nonexistent_tool",
            "arguments": {}
        })),
    });

    transport.send(&request).unwrap();
    let response = transport.receive().unwrap();

    match response {
        JsonRpcMessage::Response(resp) => {
            assert_eq!(resp.jsonrpc, JsonRpcVersion::V2);
            assert_eq!(resp.id, 2);
            let error = resp.error.unwrap();
            assert_eq!(error.code, -32601);
            assert!(error.message.contains("Tool not found"));
        },
        _ => panic!("Expected error message"),
    }
}
