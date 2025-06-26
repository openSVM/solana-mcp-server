use crate::transport::{JsonRpcMessage, JsonRpcNotification, JsonRpcVersion, Transport};
use crate::validation::sanitize_for_logging;
use crate::{Config, CustomStdioTransport};
use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Server state containing RPC clients and configuration
///
/// Manages the main Solana RPC client, additional SVM network clients,
/// and server configuration. Thread-safe through Arc<RwLock<>> wrapper.
pub struct ServerState {
    /// Primary Solana RPC client
    pub rpc_client: RpcClient,
    /// Map of enabled SVM network clients by network ID
    pub svm_clients: HashMap<String, RpcClient>,
    /// Current server configuration
    pub config: Config,
    /// Whether the server has been initialized
    pub initialized: bool,
    /// Protocol version being used
    pub protocol_version: String,
}

impl ServerState {
    /// Creates a new ServerState with the given configuration
    ///
    /// # Arguments
    /// * `config` - Validated configuration to use
    ///
    /// # Returns
    /// * `Self` - New ServerState instance
    ///
    /// # Security
    /// - Only creates RPC clients for enabled networks
    /// - Uses validated configuration with HTTPS enforcement
    pub fn new(config: Config) -> Self {
        let commitment = Self::parse_commitment(&config.commitment);

        log::info!(
            "Creating RPC client for: {}",
            sanitize_for_logging(&config.rpc_url)
        );
        let rpc_client = RpcClient::new_with_commitment(config.rpc_url.clone(), commitment.clone());

        // Create RPC clients for enabled SVM networks
        let mut svm_clients = HashMap::new();
        for (network_id, network) in &config.svm_networks {
            if network.enabled {
                log::info!(
                    "Creating SVM client for network '{}': {}",
                    network_id,
                    sanitize_for_logging(&network.rpc_url)
                );
                let client =
                    RpcClient::new_with_commitment(network.rpc_url.clone(), commitment.clone());
                svm_clients.insert(network_id.clone(), client);
            }
        }

        Self {
            rpc_client,
            svm_clients,
            protocol_version: config.protocol_version.clone(),
            config,
            initialized: false,
        }
    }

    /// Updates the server configuration and recreates clients as needed
    ///
    /// # Arguments
    /// * `new_config` - New validated configuration
    ///
    /// # Security
    /// - Validates new configuration before applying
    /// - Recreates clients with new URLs securely
    pub fn update_config(&mut self, new_config: Config) {
        let commitment = Self::parse_commitment(&new_config.commitment);

        // Update main RPC client if URL changed
        if self.config.rpc_url != new_config.rpc_url {
            log::info!(
                "Updating main RPC client to: {}",
                sanitize_for_logging(&new_config.rpc_url)
            );
            self.rpc_client =
                RpcClient::new_with_commitment(new_config.rpc_url.clone(), commitment.clone());
        }

        // Update SVM clients
        self.svm_clients.clear();
        for (network_id, network) in &new_config.svm_networks {
            if network.enabled {
                log::info!(
                    "Creating/updating SVM client for network '{}': {}",
                    network_id,
                    sanitize_for_logging(&network.rpc_url)
                );
                let client =
                    RpcClient::new_with_commitment(network.rpc_url.clone(), commitment.clone());
                self.svm_clients.insert(network_id.clone(), client);
            }
        }

        self.config = new_config;
    }

    /// Gets list of enabled network IDs
    ///
    /// # Returns
    /// * `Vec<&str>` - List of enabled network identifiers
    pub fn get_enabled_networks(&self) -> Vec<&str> {
        self.config
            .svm_networks
            .iter()
            .filter(|(_, network)| network.enabled)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Parses commitment string into CommitmentConfig
    ///
    /// # Arguments
    /// * `commitment_str` - String representation of commitment level
    ///
    /// # Returns
    /// * `CommitmentConfig` - Parsed commitment configuration
    fn parse_commitment(commitment_str: &str) -> CommitmentConfig {
        match commitment_str {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => {
                log::warn!(
                    "Invalid commitment '{}', using default (finalized)",
                    commitment_str
                );
                CommitmentConfig::finalized()
            }
        }
    }
}

/// Starts the Solana MCP server with stdio transport
///
/// Initializes the server with configuration validation, sets up transport,
/// sends protocol negotiation, and starts the main message loop.
///
/// # Returns
/// * `Result<()>` - Ok if server shuts down cleanly, Err on critical errors
///
/// # Security
/// - Validates configuration before starting
/// - Uses secure transport with proper error handling
/// - Implements graceful shutdown on connection close
pub async fn start_server() -> Result<()> {
    log::info!("Starting Solana MCP server...");

    // Load and validate configuration
    let config = Config::load().map_err(|e| {
        log::error!("Failed to load configuration: {}", e);
        e
    })?;

    log::info!(
        "Loaded config: RPC URL: {}, Protocol Version: {}",
        sanitize_for_logging(&config.rpc_url),
        config.protocol_version
    );

    let state = Arc::new(RwLock::new(ServerState::new(config.clone())));

    let transport = CustomStdioTransport::new();
    transport.open().map_err(|e| {
        log::error!("Failed to open transport: {}", e);
        e
    })?;
    log::info!("Opened stdio transport");

    // Send initial protocol version notification
    log::info!(
        "Sending protocol version notification: {}",
        config.protocol_version
    );
    transport
        .send(&JsonRpcMessage::Notification(JsonRpcNotification {
            jsonrpc: JsonRpcVersion::V2,
            method: "protocol".to_string(),
            params: Some(serde_json::json!({
                "version": config.protocol_version.clone()
            })),
        }))
        .map_err(|e| {
            log::error!("Failed to send protocol notification: {}", e);
            e
        })?;

    // Start message loop with proper error handling
    log::info!("Starting message loop");
    loop {
        match transport.receive() {
            Ok(message) => {
                // Handle message without logging sensitive content
                log::debug!("Received message of type: {}", get_message_type(&message));

                match handle_message(message, state.clone()).await {
                    Ok(response) => {
                        log::debug!("Sending response");
                        if let Err(e) = transport.send(&response) {
                            log::error!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Error handling message: {}", e);
                        // Continue processing other messages
                    }
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Connection closed") || error_msg.contains("EOF") {
                    log::info!("Client disconnected gracefully");
                    break;
                } else {
                    log::error!("Error receiving message: {}", e);
                    // For non-connection errors, continue trying
                }
            }
        }
    }

    log::info!("Closing transport");
    if let Err(e) = transport.close() {
        log::warn!("Error closing transport: {}", e);
    }

    log::info!("Solana MCP server stopped");
    Ok(())
}

/// Handles a received message and returns appropriate response
async fn handle_message(
    message: JsonRpcMessage,
    state: Arc<RwLock<ServerState>>,
) -> Result<JsonRpcMessage> {
    let message_str = serde_json::to_string(&message)?;
    crate::tools::handle_request(&message_str, state).await
}

/// Gets the message type for safe logging
fn get_message_type(message: &JsonRpcMessage) -> &'static str {
    match message {
        JsonRpcMessage::Request(_) => "request",
        JsonRpcMessage::Response(_) => "response",
        JsonRpcMessage::Notification(_) => "notification",
    }
}
