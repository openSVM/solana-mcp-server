use anyhow::Result;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging to stderr
    Builder::new()
        .filter_level(LevelFilter::Error)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    log::info!("Starting Solana MCP server...");
    solana_mcp_server::server::start_server().await
}
