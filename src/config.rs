use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::{env, fs, collections::HashMap};
use crate::protocol::LATEST_PROTOCOL_VERSION;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SvmNetwork {
    pub name: String,
    pub rpc_url: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub commitment: String,
    pub protocol_version: String,
    #[serde(default)]
    pub svm_networks: HashMap<String, SvmNetwork>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try to load from config file first
        if let Ok(content) = fs::read_to_string("config.json") {
            let config: Config = serde_json::from_str(&content)
                .context("Failed to parse config.json")?;
            return Ok(config);
        }

        // Fall back to environment variables
        let rpc_url = env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.opensvm.com".to_string());
            
        let commitment = env::var("SOLANA_COMMITMENT")
            .unwrap_or_else(|_| "confirmed".to_string());

        let protocol_version = env::var("SOLANA_PROTOCOL_VERSION")
            .unwrap_or_else(|_| LATEST_PROTOCOL_VERSION.to_string());

        Ok(Config {
            rpc_url,
            commitment,
            protocol_version,
            svm_networks: HashMap::new(),
        })
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write("config.json", content)
            .context("Failed to write config.json")?;
        Ok(())
    }
}
