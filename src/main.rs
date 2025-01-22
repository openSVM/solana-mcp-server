use anyhow::Result;
use clap::Parser;
use solana_mcp_server::start_server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Config {
    #[arg(long)]
    iam_agent: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _config = Config::parse();
    start_server().await
}
