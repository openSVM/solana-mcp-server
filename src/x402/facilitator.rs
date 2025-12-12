//! x402 v2 Facilitator Client
//!
//! HTTP client for interacting with x402 facilitator endpoints:
//! - POST /verify - Verify payment authorization
//! - POST /settle - Execute payment settlement
//! - GET /supported - Query supported networks/schemes

use super::config::X402Config;
use super::types::{PaymentPayload, PaymentRequirements, SettlementResponse, VerifyResponse};
use crate::error::{McpError, McpResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Request body for /verify and /settle endpoints
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentRequest {
    payment_payload: PaymentPayload,
    payment_requirements: PaymentRequirements,
}

/// Response from /supported endpoint
#[derive(Debug, Deserialize)]
pub struct SupportedResponse {
    pub kinds: Vec<SupportedKind>,
    pub extensions: Vec<String>,
    #[serde(default)]
    pub signers: std::collections::HashMap<String, Vec<String>>,
}

/// Supported payment kind
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedKind {
    pub x402_version: u32,
    pub scheme: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

/// Facilitator client for x402 operations
pub struct FacilitatorClient {
    client: Client,
    base_url: String,
    max_retries: u32,
}

impl FacilitatorClient {
    /// Creates a new facilitator client
    ///
    /// # Arguments
    /// * `config` - x402 configuration
    ///
    /// # Returns
    /// * `McpResult<Self>` - New client instance
    pub fn new(config: &X402Config) -> McpResult<Self> {
        let timeout = Duration::from_secs(config.request_timeout_seconds);
        
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| McpError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: config.facilitator_base_url.clone(),
            max_retries: config.max_retries,
        })
    }

    /// Verifies a payment authorization without settlement
    ///
    /// # Arguments
    /// * `payment_payload` - Payment payload from client
    /// * `payment_requirements` - Original payment requirements
    ///
    /// # Returns
    /// * `McpResult<VerifyResponse>` - Verification result
    pub async fn verify(
        &self,
        payment_payload: &PaymentPayload,
        payment_requirements: &PaymentRequirements,
    ) -> McpResult<VerifyResponse> {
        let trace_id = Uuid::new_v4();
        
        tracing::info!(
            trace_id = %trace_id,
            network = %payment_requirements.network,
            scheme = %payment_requirements.scheme,
            "Verifying payment authorization"
        );

        let request = PaymentRequest {
            payment_payload: payment_payload.clone(),
            payment_requirements: payment_requirements.clone(),
        };

        let url = format!("{}/verify", self.base_url);
        
        self.execute_with_retry(&url, &request, &trace_id, "verify").await
    }

    /// Settles a payment by broadcasting to blockchain
    ///
    /// # Arguments
    /// * `payment_payload` - Payment payload from client
    /// * `payment_requirements` - Original payment requirements
    ///
    /// # Returns
    /// * `McpResult<SettlementResponse>` - Settlement result
    pub async fn settle(
        &self,
        payment_payload: &PaymentPayload,
        payment_requirements: &PaymentRequirements,
    ) -> McpResult<SettlementResponse> {
        let trace_id = Uuid::new_v4();
        
        tracing::info!(
            trace_id = %trace_id,
            network = %payment_requirements.network,
            scheme = %payment_requirements.scheme,
            "Settling payment"
        );

        let request = PaymentRequest {
            payment_payload: payment_payload.clone(),
            payment_requirements: payment_requirements.clone(),
        };

        let url = format!("{}/settle", self.base_url);
        
        self.execute_with_retry(&url, &request, &trace_id, "settle").await
    }

    /// Queries supported networks and schemes
    ///
    /// # Returns
    /// * `McpResult<SupportedResponse>` - List of supported configurations
    pub async fn get_supported(&self) -> McpResult<SupportedResponse> {
        let trace_id = Uuid::new_v4();
        
        tracing::info!(
            trace_id = %trace_id,
            "Querying supported networks"
        );

        let url = format!("{}/supported", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("X-Trace-ID", trace_id.to_string())
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    trace_id = %trace_id,
                    error = %e,
                    "Failed to query supported networks"
                );
                McpError::network(format!("Facilitator request failed: {}", e))
                    .with_endpoint(&url)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            
            tracing::error!(
                trace_id = %trace_id,
                status = %status,
                body = %body,
                "Facilitator returned error"
            );
            
            return Err(McpError::server(format!(
                "Facilitator error: {} - {}",
                status, body
            )));
        }

        let result: SupportedResponse = response.json().await.map_err(|e| {
            tracing::error!(
                trace_id = %trace_id,
                error = %e,
                "Failed to parse supported response"
            );
            McpError::server(format!("Failed to parse facilitator response: {}", e))
        })?;

        tracing::info!(
            trace_id = %trace_id,
            kinds_count = result.kinds.len(),
            "Successfully retrieved supported networks"
        );

        Ok(result)
    }

    /// Executes a request with exponential backoff retry
    async fn execute_with_retry<T, R>(
        &self,
        url: &str,
        request: &R,
        trace_id: &Uuid,
        operation: &str,
    ) -> McpResult<T>
    where
        T: serde::de::DeserializeOwned,
        R: Serialize,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt <= self.max_retries {
            if attempt > 0 {
                // Exponential backoff with jitter
                let base_delay = 100 * (2_u64.pow(attempt - 1));
                let jitter = rand::random::<u64>() % 100;
                let delay = Duration::from_millis(base_delay + jitter);
                
                tracing::debug!(
                    trace_id = %trace_id,
                    attempt = attempt,
                    delay_ms = delay.as_millis(),
                    "Retrying after delay"
                );
                
                tokio::time::sleep(delay).await;
            }

            tracing::debug!(
                trace_id = %trace_id,
                attempt = attempt + 1,
                max_retries = self.max_retries + 1,
                "Executing {} request",
                operation
            );

            match self.execute_once(url, request, trace_id).await {
                Ok(result) => {
                    tracing::info!(
                        trace_id = %trace_id,
                        attempt = attempt + 1,
                        "Request succeeded"
                    );
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!(
                        trace_id = %trace_id,
                        attempt = attempt + 1,
                        error = %e,
                        "Request failed"
                    );
                    last_error = Some(e);
                    attempt += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            McpError::server(format!("Request failed after {} attempts", self.max_retries + 1))
        }))
    }

    /// Executes a single request attempt
    async fn execute_once<T, R>(
        &self,
        url: &str,
        request: &R,
        trace_id: &Uuid,
    ) -> McpResult<T>
    where
        T: serde::de::DeserializeOwned,
        R: Serialize,
    {
        let response = self.client
            .post(url)
            .header("X-Trace-ID", trace_id.to_string())
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                McpError::network(format!("Facilitator request failed: {}", e))
                    .with_endpoint(url)
            })?;

        let status = response.status();
        
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            
            return Err(McpError::server(format!(
                "Facilitator error: {} - {}",
                status, body
            )));
        }

        let result: T = response.json().await.map_err(|e| {
            McpError::server(format!("Failed to parse facilitator response: {}", e))
        })?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_facilitator_client_creation() {
        let mut networks = HashMap::new();
        networks.insert(
            "test".to_string(),
            super::super::config::NetworkConfig {
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                assets: vec![],
                pay_to: "test".to_string(),
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

        let client = FacilitatorClient::new(&config);
        assert!(client.is_ok());
    }
}
