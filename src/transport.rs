use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
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
#[serde(rename_all = "camelCase")]
pub struct JsonRpcRequest {
    pub jsonrpc: JsonRpcVersion,
    pub id: Value, // JSON-RPC 2.0 allows string, number, or null
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcResponse {
    pub jsonrpc: JsonRpcVersion,
    pub id: Value, // JSON-RPC 2.0 allows string, number, or null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcNotification {
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

impl JsonRpcMessage {
    pub fn is_success(&self) -> bool {
        match self {
            JsonRpcMessage::Response(resp) => resp.error.is_none(),
            _ => false,
        }
    }
}

pub trait Transport {
    fn send(&self, message: &JsonRpcMessage) -> Result<()>;
    fn send_raw(&self, json: &str) -> Result<()>;
    fn receive(&self) -> Result<JsonRpcMessage>;
    fn open(&self) -> Result<()>;
    fn close(&self) -> Result<()>;
}

pub struct CustomStdioTransport {
    reader: Mutex<BufReader<io::Stdin>>,
    writer: Mutex<io::Stdout>,
}

impl Default for CustomStdioTransport {
    fn default() -> Self {
        Self::new()
    }
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
    fn send_raw(&self, json: &str) -> Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| io::Error::other("Failed to acquire writer lock"))?;
        let json = json.trim();
        writeln!(writer, "{json}")?;
        writer.flush()?;
        Ok(())
    }

    fn send(&self, message: &JsonRpcMessage) -> Result<()> {
        log::debug!("Sending message: {}", serde_json::to_string(message)?);
        let mut writer = self.writer.lock().map_err(|_| {
            let err = io::Error::other("Failed to acquire writer lock");
            log::error!("Transport error: {err}");
            err
        })?;
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buf);
        message.serialize(&mut ser)?;
        writer.write_all(&buf)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }

    fn receive(&self) -> Result<JsonRpcMessage> {
        let mut line = String::new();
        let mut reader = self.reader.lock().map_err(|_| {
            let err = io::Error::other("Failed to acquire reader lock");
            log::error!("Transport error: {err}");
            err
        })?;

        match reader.read_line(&mut line) {
            Ok(0) => {
                let err = io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed");
                log::info!("Transport connection closed");
                Err(err.into())
            }
            Ok(_) => {
                if line.trim().is_empty() {
                    let err = io::Error::new(io::ErrorKind::InvalidData, "Empty message received");
                    log::error!("Transport error: {err}");
                    return Err(err.into());
                }
                log::debug!("Received raw message: {}", line.trim());
                let message = serde_json::from_str(&line)?;
                Ok(message)
            }
            Err(e) => {
                log::error!("Transport error: {e}");
                Err(e.into())
            }
        }
    }

    fn open(&self) -> Result<()> {
        log::info!("Opening stdio transport");
        Ok(())
    }

    fn close(&self) -> Result<()> {
        log::info!("Closing stdio transport");
        Ok(())
    }
}
