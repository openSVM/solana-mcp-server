use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::transport::{Transport, JsonRpcMessage, JsonRpcNotification, JsonRpcVersion};
use crate::{Config, CustomStdioTransport};

pub struct ServerState {
    pub rpc_client: RpcClient,
    pub svm_clients: HashMap<String, RpcClient>,
    pub config: Config,
    pub initialized: bool,
    pub protocol_version: String,
}

impl ServerState {
    pub fn new(config: Config) -> Self {
        let commitment = match config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::default(),
        };
        
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            commitment.clone(),
        );

        // Create RPC clients for enabled SVM networks
        let mut svm_clients = HashMap::new();
        for (network_id, network) in &config.svm_networks {
            if network.enabled {
                let client = RpcClient::new_with_commitment(
                    network.rpc_url.clone(),
                    commitment.clone(),
                );
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

    pub fn update_config(&mut self, new_config: Config) {
        let commitment = match new_config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::default(),
        };

        // Update main RPC client if URL changed
        if self.config.rpc_url != new_config.rpc_url {
            self.rpc_client = RpcClient::new_with_commitment(
                new_config.rpc_url.clone(),
                commitment.clone(),
            );
        }

        // Update SVM clients
        self.svm_clients.clear();
        for (network_id, network) in &new_config.svm_networks {
            if network.enabled {
                let client = RpcClient::new_with_commitment(
                    network.rpc_url.clone(),
                    commitment.clone(),
                );
                self.svm_clients.insert(network_id.clone(), client);
            }
        }

        self.config = new_config;
    }

    pub fn get_enabled_networks(&self) -> Vec<&str> {
        self.config.svm_networks
            .iter()
            .filter(|(_, network)| network.enabled)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

pub async fn start_server() -> Result<()> {
    log::info!("Starting Solana MCP server...");

    let config = Config::load()?;
    log::info!("Loaded config: RPC URL: {}, Protocol Version: {}", config.rpc_url, config.protocol_version);
    
    let state = Arc::new(RwLock::new(ServerState::new(config.clone())));
    
    let transport = CustomStdioTransport::new();
    transport.open()?;
    log::info!("Opened stdio transport");

    // Send initial protocol version notification
    log::info!("Sending protocol version notification: {}", config.protocol_version);
    transport.send(&JsonRpcMessage::Notification(JsonRpcNotification {
        jsonrpc: JsonRpcVersion::V2,
        method: "protocol".to_string(),
        params: Some(serde_json::json!({
            "version": config.protocol_version.clone()
        })),
    }))?;

    // Start message loop
    log::info!("Starting message loop");
    loop {
        match transport.receive() {
            Ok(message) => {
                let message_str = serde_json::to_string(&message)?;
                log::debug!("Received message: {}", message_str);
                let response = crate::tools::handle_request(&message_str, state.clone()).await?;
                log::debug!("Sending response: {}", serde_json::to_string(&response)?);
                transport.send(&response)?;
            }
            Err(e) => {
                if e.to_string().contains("Connection closed") {
                    log::info!("Client disconnected");
                    break;
                }
                log::error!("Error receiving message: {}", e);
            }
        }
    }

    log::info!("Closing transport");
    transport.close()?;
    Ok(())
}
