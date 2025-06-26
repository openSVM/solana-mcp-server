use anyhow::Result;
use solana_mcp_server::{init_logging, start_server};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    if let Err(e) = init_logging(Some("info")) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }
    
    tracing::info!("Starting Solana MCP server...");
    start_server().await
}
