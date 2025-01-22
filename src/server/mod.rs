use anyhow::Result;
use crate::transport::{Transport, JsonRpcMessage};
use crate::CustomStdioTransport;
use serde_json::Value;

pub async fn start_server() -> Result<()> {
    eprintln!("Solana MCP server ready - {} v{}", 
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let transport = CustomStdioTransport::new();
    transport.open()?;

    loop {
        match transport.receive() {
            Ok(message) => {
                let message_str = serde_json::to_string(&message)?;
                let response = crate::tools::handle_request(&message_str).await?;
                transport.send(&response)?;
            }
            Err(e) => {
                if e.to_string().contains("Connection closed") {
                    eprintln!("Client disconnected");
                    break;
                }
                eprintln!("Error receiving message: {}", e);
            }
        }
    }

    transport.close()?;
    Ok(())
}
