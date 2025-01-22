use anyhow::Result;
use clap::Parser;
use mcp_sdk::server::stdio::StdioServerTransport;
use solana_mcp_server::tools;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Config {
    #[arg(long)]
    iam_agent: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _config = Config::parse();
    let mut transport = StdioServerTransport::new();

    loop {
        if let Some(request) = transport.read_line().await? {
            let response = tools::handle_request(&request).await?;
            transport.write_line(&serde_json::to_string(&response)?).await?;
        }
    }
}
