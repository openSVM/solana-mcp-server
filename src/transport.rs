use std::io::{self, BufRead, BufReader, Write};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

impl Default for JsonRpcVersion {
    fn default() -> Self {
        Self::V2
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: JsonRpcVersion,
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: JsonRpcVersion,
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

pub trait Transport {
    fn send(&self, message: &JsonRpcMessage) -> Result<()>;
    fn receive(&self) -> Result<JsonRpcMessage>;
    fn open(&self) -> Result<()>;
    fn close(&self) -> Result<()>;
}

pub struct CustomStdioTransport {
    reader: Mutex<BufReader<io::Stdin>>,
    writer: Mutex<io::Stdout>,
}

impl CustomStdioTransport {
    pub fn new() -> Self {
        Self {
            reader: Mutex::new(BufReader::new(io::stdin())),
            writer: Mutex::new(io::stdout()),
        }
    }
}

impl Transport for CustomStdioTransport {
    fn send(&self, message: &JsonRpcMessage) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        let mut writer = self.writer.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to acquire writer lock")
        })?;
        writeln!(writer, "{}", json)?;
        writer.flush()?;
        Ok(())
    }

    fn receive(&self) -> Result<JsonRpcMessage> {
        let mut line = String::new();
        let mut reader = self.reader.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to acquire reader lock")
        })?;
        
        match reader.read_line(&mut line) {
            Ok(0) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed").into()),
            Ok(_) => {
                if line.trim().is_empty() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty message received").into());
                }
                let message = serde_json::from_str(&line)?;
                Ok(message)
            },
            Err(e) => Err(e.into()),
        }
    }

    fn open(&self) -> Result<()> {
        Ok(())
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }
}
