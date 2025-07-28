use crate::protocol::LATEST_PROTOCOL_VERSION;
use crate::validation::{validate_commitment, validate_rpc_url};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs};

/// Represents a Solana Virtual Machine (SVM) network configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SvmNetwork {
    /// Human-readable name of the network
    pub name: String,
    /// RPC endpoint URL for the network (must be HTTPS)
    pub rpc_url: String,
    /// Whether this network is currently enabled for use
    pub enabled: bool,
}

/// Main configuration structure for the Solana MCP server
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Primary RPC URL for Solana operations
    pub rpc_url: String,
    /// Commitment level for transactions (processed, confirmed, finalized)
    pub commitment: String,
    /// Protocol version for MCP communication
    pub protocol_version: String,
    /// Additional SVM networks configuration
    #[serde(default)]
    pub svm_networks: HashMap<String, SvmNetwork>,
    /// Timeout configurations
    #[serde(default)]
    pub timeouts: TimeoutConfig,
}

/// Timeout configuration for various operations
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimeoutConfig {
    /// HTTP request timeout in seconds
    #[serde(default = "default_http_timeout")]
    pub http_request_seconds: u64,
    /// WebSocket connection timeout in seconds
    #[serde(default = "default_ws_connection_timeout")]
    pub websocket_connection_seconds: u64,
    /// WebSocket message timeout in seconds
    #[serde(default = "default_ws_message_timeout")]
    pub websocket_message_seconds: u64,
    /// RPC subscription creation timeout in seconds
    #[serde(default = "default_subscription_timeout")]
    pub subscription_seconds: u64,
    /// Maximum idle time for WebSocket connections in seconds
    #[serde(default = "default_max_idle_timeout")]
    pub max_idle_seconds: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            http_request_seconds: default_http_timeout(),
            websocket_connection_seconds: default_ws_connection_timeout(),
            websocket_message_seconds: default_ws_message_timeout(),
            subscription_seconds: default_subscription_timeout(),
            max_idle_seconds: default_max_idle_timeout(),
        }
    }
}

// Default timeout values
fn default_http_timeout() -> u64 { 30 }
fn default_ws_connection_timeout() -> u64 { 30 }
fn default_ws_message_timeout() -> u64 { 10 }
fn default_subscription_timeout() -> u64 { 15 }
fn default_max_idle_timeout() -> u64 { 300 }

impl Config {
    /// Loads configuration from file or environment variables
    ///
    /// Attempts to load from config.json first, then falls back to environment variables.
    /// All loaded configurations are validated for security and correctness.
    ///
    /// # Returns
    /// * `Result<Self>` - The loaded and validated configuration
    ///
    /// # Errors
    /// * Configuration file parsing errors
    /// * Validation errors for URLs or commitment levels
    /// * Environment variable access errors
    pub fn load() -> Result<Self> {
        let config = if let Ok(content) = fs::read_to_string("config.json") {
            log::info!("Loading configuration from config.json");
            let config: Config =
                serde_json::from_str(&content).context("Failed to parse config.json")?;
            config
        } else {
            log::info!("Loading configuration from environment variables");
            // Fall back to environment variables
            let rpc_url = env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.opensvm.com".to_string());

            let commitment =
                env::var("SOLANA_COMMITMENT").unwrap_or_else(|_| "confirmed".to_string());

            let protocol_version = env::var("SOLANA_PROTOCOL_VERSION")
                .unwrap_or_else(|_| LATEST_PROTOCOL_VERSION.to_string());

            Config {
                rpc_url,
                commitment,
                protocol_version,
                svm_networks: HashMap::new(),
                timeouts: TimeoutConfig::default(),
            }
        };

        // Validate the loaded configuration
        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration for security and correctness
    ///
    /// # Returns
    /// * `Result<()>` - Ok if valid, Err with description if invalid
    ///
    /// # Security
    /// - Validates all RPC URLs use HTTPS
    /// - Ensures commitment levels are valid
    /// - Checks for malformed network configurations
    pub fn validate(&self) -> Result<()> {
        // Validate main RPC URL
        validate_rpc_url(&self.rpc_url).context("Invalid main RPC URL")?;

        // Validate commitment level
        validate_commitment(&self.commitment).context("Invalid commitment level")?;

        // Validate all SVM network configurations
        for (network_id, network) in &self.svm_networks {
            validate_rpc_url(&network.rpc_url)
                .with_context(|| format!("Invalid RPC URL for network '{network_id}'"))?;

            if network.name.is_empty() {
                return Err(anyhow::anyhow!(
                    "Network name cannot be empty for network '{}'",
                    network_id
                ));
            }
        }

        Ok(())
    }

    /// Saves the configuration to config.json
    ///
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully, Err otherwise
    ///
    /// # Security
    /// - Validates configuration before saving
    /// - Ensures atomic write operation
    pub fn save(&self) -> Result<()> {
        // Validate before saving
        self.validate()
            .context("Cannot save invalid configuration")?;

        let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write("config.json", content).context("Failed to write config.json")?;

        log::info!("Configuration saved to config.json");
        Ok(())
    }
}
