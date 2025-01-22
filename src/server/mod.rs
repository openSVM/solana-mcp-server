use anyhow::Result;
use mcp_sdk::server::{Server, ServerConfig, ServerInfo};
use crate::rpc;

pub async fn start_server() -> Result<()> {
    let server = Server::new(
        ServerInfo {
            name: "solana-mcp-server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        ServerConfig::default(),
    );

    // Add server capabilities and handlers here
    
    Ok(())
}
