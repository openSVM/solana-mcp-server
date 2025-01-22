use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    solana_mcp_server::server::start_server().await
}
