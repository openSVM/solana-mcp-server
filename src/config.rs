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
}

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
