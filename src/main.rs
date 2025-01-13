use anyhow::Result;
use log::info;
use mcp_sdk::{
    transport::StdioTransport,
    server::{Server, ServerOptions},
    protocol::ProtocolBuilder,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_mcp_server::SolanaMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    
    let client = RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    );
    
    let _server = SolanaMcpServer::new(client);
    let transport = StdioTransport::default();
    let protocol = ProtocolBuilder::new(transport);
    let options = ServerOptions::default();
    
    info!("Starting Solana MCP server...");
    Server::new(protocol, options).listen().await?;
    
    Ok(())
}
