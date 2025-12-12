//! x402 v2 Configuration
//!
//! Configuration structures for x402 payment protocol integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{McpError, McpResult};
use super::validation::validate_caip2_network;

/// Supported asset configuration for a network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetConfig {
    /// Asset identifier (e.g., token mint address, contract address)
    pub address: String,
    /// Human-readable asset name (e.g., "USDC")
    pub name: String,
    /// Number of decimals
    pub decimals: u8,
}

/// Network-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkConfig {
    /// CAIP-2 network identifier
    pub network: String,
    /// Supported assets on this network
    pub assets: Vec<AssetConfig>,
    /// Payment recipient address for this network
    pub pay_to: String,
    /// Minimum compute unit price (for SVM networks only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_compute_unit_price: Option<u64>,
    /// Maximum compute unit price (for SVM networks only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_compute_unit_price: Option<u64>,
}

/// x402 protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct X402Config {
    /// Enable x402 payment protocol (default: false)
    #[serde(default)]
    pub enabled: bool,
    /// Facilitator base URL for payment verification and settlement
    pub facilitator_base_url: String,
    /// Request timeout in seconds (default: 30)
    #[serde(default = "default_request_timeout")]
    pub request_timeout_seconds: u64,
    /// Maximum retry attempts (default: 3)
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Supported networks and assets
    pub networks: HashMap<String, NetworkConfig>,
}

fn default_request_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

impl Default for X402Config {
    fn default() -> Self {
        Self {
            enabled: false,
            facilitator_base_url: String::new(),
            request_timeout_seconds: default_request_timeout(),
            max_retries: default_max_retries(),
            networks: HashMap::new(),
        }
    }
}

impl X402Config {
    /// Validates the x402 configuration
    ///
    /// # Returns
    /// * `McpResult<()>` - Ok if valid, Err with details if invalid
    pub fn validate(&self) -> McpResult<()> {
        if !self.enabled {
            // If disabled, no need to validate
            return Ok(());
        }

        // Validate facilitator URL
        if self.facilitator_base_url.is_empty() {
            return Err(McpError::validation(
                "x402 facilitator_base_url is required when enabled".to_string()
            ));
        }

        // Validate URL format
        let url = url::Url::parse(&self.facilitator_base_url).map_err(|e| {
            McpError::validation(format!(
                "Invalid facilitator_base_url '{}': {}",
                self.facilitator_base_url, e
            ))
        })?;

        // Ensure HTTPS for security
        if url.scheme() != "https" && url.scheme() != "http" {
            return Err(McpError::validation(format!(
                "Facilitator URL must use http or https scheme, got '{}'",
                url.scheme()
            )));
        }

        // Validate timeout
        if self.request_timeout_seconds == 0 || self.request_timeout_seconds > 300 {
            return Err(McpError::validation(format!(
                "request_timeout_seconds must be between 1 and 300, got {}",
                self.request_timeout_seconds
            )));
        }

        // Validate max retries
        if self.max_retries > 10 {
            return Err(McpError::validation(format!(
                "max_retries must be <= 10, got {}",
                self.max_retries
            )));
        }

        // Validate at least one network is configured
        if self.networks.is_empty() {
            return Err(McpError::validation(
                "At least one network must be configured when x402 is enabled".to_string()
            ));
        }

        // Validate each network configuration
        for (network_id, network_config) in &self.networks {
            validate_caip2_network(&network_config.network).map_err(|e| {
                McpError::validation(format!(
                    "Invalid network '{}': {}",
                    network_id, e
                ))
            })?;

            if network_config.assets.is_empty() {
                return Err(McpError::validation(format!(
                    "Network '{}' must have at least one asset configured",
                    network_id
                )));
            }

            if network_config.pay_to.is_empty() {
                return Err(McpError::validation(format!(
                    "Network '{}' must have pay_to address configured",
                    network_id
                )));
            }

            // Validate SVM-specific bounds
            if network_config.network.starts_with("solana:") {
                if let (Some(min), Some(max)) = (
                    network_config.min_compute_unit_price,
                    network_config.max_compute_unit_price,
                ) {
                    if min > max {
                        return Err(McpError::validation(format!(
                            "Network '{}': min_compute_unit_price ({}) must be <= max_compute_unit_price ({})",
                            network_id, min, max
                        )));
                    }
                }
            }

            // Validate asset configurations
            for asset in &network_config.assets {
                if asset.address.is_empty() {
                    return Err(McpError::validation(format!(
                        "Asset address cannot be empty in network '{}'",
                        network_id
                    )));
                }
                if asset.name.is_empty() {
                    return Err(McpError::validation(format!(
                        "Asset name cannot be empty in network '{}'",
                        network_id
                    )));
                }
            }
        }

        Ok(())
    }

    /// Returns a network configuration by network ID
    pub fn get_network(&self, network_id: &str) -> Option<&NetworkConfig> {
        self.networks.get(network_id)
    }

    /// Returns an asset configuration by network ID and asset address
    pub fn get_asset(&self, network_id: &str, asset_address: &str) -> Option<&AssetConfig> {
        self.get_network(network_id)?
            .assets
            .iter()
            .find(|a| a.address == asset_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = X402Config::default();
        assert!(!config.enabled);
        assert_eq!(config.request_timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_disabled_config_validation() {
        let config = X402Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_enabled_config_requires_facilitator_url() {
        let mut config = X402Config::default();
        config.enabled = true;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_config() {
        let mut networks = HashMap::new();
        networks.insert(
            "solana-mainnet".to_string(),
            NetworkConfig {
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                assets: vec![AssetConfig {
                    address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    name: "USDC".to_string(),
                    decimals: 6,
                }],
                pay_to: "FeeRecipient123456789".to_string(),
                min_compute_unit_price: Some(1000),
                max_compute_unit_price: Some(10000),
            },
        );

        let config = X402Config {
            enabled: true,
            facilitator_base_url: "https://facilitator.example.com".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            networks,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_compute_unit_price_bounds() {
        let mut networks = HashMap::new();
        networks.insert(
            "solana-mainnet".to_string(),
            NetworkConfig {
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                assets: vec![AssetConfig {
                    address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    name: "USDC".to_string(),
                    decimals: 6,
                }],
                pay_to: "FeeRecipient123456789".to_string(),
                min_compute_unit_price: Some(10000),
                max_compute_unit_price: Some(1000), // min > max
            },
        );

        let config = X402Config {
            enabled: true,
            facilitator_base_url: "https://facilitator.example.com".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            networks,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_get_network() {
        let mut networks = HashMap::new();
        networks.insert(
            "solana-mainnet".to_string(),
            NetworkConfig {
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                assets: vec![],
                pay_to: "FeeRecipient123456789".to_string(),
                min_compute_unit_price: None,
                max_compute_unit_price: None,
            },
        );

        let config = X402Config {
            enabled: true,
            facilitator_base_url: "https://facilitator.example.com".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            networks,
        };

        assert!(config.get_network("solana-mainnet").is_some());
        assert!(config.get_network("nonexistent").is_none());
    }
}
