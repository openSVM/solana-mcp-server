use anyhow::Result;
use clap::{Parser, Subcommand};
use solana_mcp_server::{init_logging, start_server, start_mcp_server_task, start_websocket_server_task, Config, ServerState};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Parser)]
#[command(name = "solana-mcp-server")]
#[command(about = "Solana MCP Server - Run as stdio transport, web service, or WebSocket server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as stdio transport (default mode)
    Stdio,
    /// Run as web service on HTTP
    Web {
        /// Port to run the web service on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Run as WebSocket server for RPC subscriptions
    Websocket {
        /// Port to run the WebSocket server on
        #[arg(short, long, default_value = "8900")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command.unwrap_or(Commands::Stdio) {
        Commands::Stdio => {
            // For stdio mode, logging MUST go to stderr to avoid corrupting the JSON-RPC protocol on stdout
            if let Err(e) = init_logging(Some("info"), true) {
                eprintln!("Failed to initialize logging: {e}");
                std::process::exit(1);
            }
            tracing::info!("Starting Solana MCP server in stdio mode...");
            start_server().await
        }
        Commands::Web { port } => {
            // For web mode, logging can go to stdout since it doesn't interfere with HTTP protocol
            if let Err(e) = init_logging(Some("info"), false) {
                eprintln!("Failed to initialize logging: {e}");
                std::process::exit(1);
            }
            tracing::info!("Starting Solana MCP server in web service mode on port {}...", port);
            start_web_service(port).await
        }
        Commands::Websocket { port } => {
            // For WebSocket mode, logging can go to stdout since it doesn't interfere with WebSocket protocol
            if let Err(e) = init_logging(Some("info"), false) {
                eprintln!("Failed to initialize logging: {e}");
                std::process::exit(1);
            }
            tracing::info!("Starting Solana MCP server in WebSocket mode on port {}...", port);
            start_websocket_service(port).await
        }
    }
}

async fn start_web_service(port: u16) -> Result<()> {
    // Initialize Prometheus metrics
    solana_mcp_server::init_prometheus_metrics()
        .map_err(|e| anyhow::anyhow!("Failed to initialize Prometheus metrics: {}", e))?;

    // Load and validate configuration
    let config = Config::load().map_err(|e| {
        tracing::error!("Failed to load configuration: {}", e);
        e
    })?;

    tracing::info!(
        "Loaded config: RPC URL: {}, Protocol Version: {}",
        config.rpc_url, // Remove sanitization for now since validation is not accessible
        config.protocol_version
    );

    // Create server state
    let mut server_state = ServerState::new(config);
    server_state.initialized = true; // Auto-initialize for web service mode
    let state = Arc::new(RwLock::new(server_state));

    // Start the MCP HTTP server
    let server_handle = start_mcp_server_task(port, state);
    
    tracing::info!("MCP web service started on port {}", port);
    tracing::info!("Available endpoints:");
    tracing::info!("  POST /api/mcp - MCP JSON-RPC API");
    tracing::info!("  GET  /metrics - Prometheus metrics");
    tracing::info!("  GET  /health  - Health check");
    
    // Wait for the server to complete
    if let Err(e) = server_handle.await {
        tracing::error!("Web service error: {}", e);
        return Err(anyhow::anyhow!("Web service failed: {}", e));
    }
    
    Ok(())
}

async fn start_websocket_service(port: u16) -> Result<()> {
    // Initialize Prometheus metrics
    solana_mcp_server::init_prometheus_metrics()
        .map_err(|e| anyhow::anyhow!("Failed to initialize Prometheus metrics: {}", e))?;

    // Load and validate configuration
    let config = Arc::new(Config::load().map_err(|e| {
        tracing::error!("Failed to load configuration: {}", e);
        e
    })?);

    tracing::info!(
        "Loaded config: RPC URL: {}, Protocol Version: {}",
        config.rpc_url,
        config.protocol_version
    );

    // Start the WebSocket server
    let server_handle = start_websocket_server_task(port, config);
    
    tracing::info!("WebSocket server started on ws://0.0.0.0:{}", port);
    tracing::info!("Available subscription methods:");
    tracing::info!("  accountSubscribe/accountUnsubscribe");
    tracing::info!("  blockSubscribe/blockUnsubscribe");
    tracing::info!("  logsSubscribe/logsUnsubscribe");
    tracing::info!("  programSubscribe/programUnsubscribe");
    tracing::info!("  rootSubscribe/rootUnsubscribe");
    tracing::info!("  signatureSubscribe/signatureUnsubscribe");
    tracing::info!("  slotSubscribe/slotUnsubscribe");
    tracing::info!("  slotsUpdatesSubscribe/slotsUpdatesUnsubscribe");
    tracing::info!("  voteSubscribe/voteUnsubscribe");
    
    // Wait for the server to complete
    if let Err(e) = server_handle.await {
        tracing::error!("WebSocket server error: {}", e);
        return Err(anyhow::anyhow!("WebSocket server failed: {}", e));
    }
    
    Ok(())
}
