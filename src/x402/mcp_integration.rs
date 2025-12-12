//! x402 v2 MCP Transport Integration
//!
//! Defines how x402 payment protocol integrates with MCP transport layer.
//! Handles Payment Required responses and Payment Payload submission.

use super::config::X402Config;
use super::facilitator::FacilitatorClient;
use super::types::{PaymentPayload, PaymentRequired, PaymentRequirements, ResourceInfo, SettlementResponse};
use super::validation::{validate_x402_version, validate_payment_amount, validate_timeout};
use crate::error::{McpError, McpResult};
use crate::transport::{JsonRpcError, JsonRpcMessage, JsonRpcResponse, JsonRpcVersion};
use serde_json::Value;

/// x402-specific JSON-RPC error codes
pub const PAYMENT_REQUIRED_CODE: i32 = -40200;
pub const INVALID_PAYMENT_CODE: i32 = -40201;

/// Creates a Payment Required JSON-RPC error response
///
/// This response indicates that payment is required to access the resource.
/// The payment requirements are included in the error data field.
///
/// # Arguments
/// * `payment_required` - Payment requirements to include
/// * `request_id` - The JSON-RPC request ID
///
/// # Returns
/// * `JsonRpcMessage` - Formatted error response with payment requirements
pub fn create_payment_required_response(
    payment_required: PaymentRequired,
    request_id: Value,
) -> JsonRpcMessage {
    tracing::info!(
        x402_version = payment_required.x402_version,
        accepts_count = payment_required.accepts.len(),
        "Returning Payment Required response"
    );

    let error = JsonRpcError {
        code: PAYMENT_REQUIRED_CODE,
        message: payment_required.error.clone()
            .unwrap_or_else(|| "Payment required".to_string()),
        data: Some(serde_json::to_value(payment_required).unwrap()),
    };

    JsonRpcMessage::Response(JsonRpcResponse {
        jsonrpc: JsonRpcVersion::V2,
        id: request_id,
        result: None,
        error: Some(error),
    })
}

/// Creates an Invalid Payment JSON-RPC error response
///
/// This response indicates that the payment payload was invalid.
///
/// # Arguments
/// * `reason` - Reason why the payment was invalid
/// * `request_id` - The JSON-RPC request ID
///
/// # Returns
/// * `JsonRpcMessage` - Formatted error response
pub fn create_invalid_payment_response(
    reason: String,
    request_id: Value,
) -> JsonRpcMessage {
    tracing::warn!(
        reason = %reason,
        "Returning Invalid Payment response"
    );

    let error = JsonRpcError {
        code: INVALID_PAYMENT_CODE,
        message: format!("Invalid payment: {}", reason),
        data: None,
    };

    JsonRpcMessage::Response(JsonRpcResponse {
        jsonrpc: JsonRpcVersion::V2,
        id: request_id,
        result: None,
        error: Some(error),
    })
}

/// Extracts payment payload from MCP request metadata
///
/// The payment payload is expected in the _meta.payment field of the request.
///
/// # Arguments
/// * `meta` - The _meta object from the MCP request
///
/// # Returns
/// * `McpResult<Option<PaymentPayload>>` - Parsed payment payload if present
pub fn extract_payment_payload(meta: Option<&Value>) -> McpResult<Option<PaymentPayload>> {
    let Some(meta) = meta else {
        return Ok(None);
    };

    let Some(payment_data) = meta.get("payment") else {
        return Ok(None);
    };

    let payment_payload: PaymentPayload = serde_json::from_value(payment_data.clone())
        .map_err(|e| McpError::validation(format!("Invalid payment payload format: {}", e)))?;

    // Validate payment payload
    validate_x402_version(payment_payload.x402_version)?;
    validate_payment_amount(&payment_payload.accepted.amount)?;
    validate_timeout(payment_payload.accepted.max_timeout_seconds)?;

    Ok(Some(payment_payload))
}

/// Processes a payment for a tool call
///
/// Verifies and settles the payment using the facilitator.
///
/// # Arguments
/// * `payment_payload` - The payment payload from the client
/// * `payment_requirements` - The original payment requirements
/// * `config` - x402 configuration
///
/// # Returns
/// * `McpResult<SettlementResponse>` - Settlement result from facilitator
pub async fn process_payment(
    payment_payload: &PaymentPayload,
    payment_requirements: &PaymentRequirements,
    config: &X402Config,
) -> McpResult<SettlementResponse> {
    let facilitator = FacilitatorClient::new(config)?;

    // First verify the payment
    tracing::info!("Verifying payment authorization");
    let verify_response = facilitator.verify(payment_payload, payment_requirements).await?;

    if !verify_response.is_valid {
        let reason = verify_response.invalid_reason
            .unwrap_or_else(|| "Payment verification failed".to_string());
        
        tracing::warn!(
            reason = %reason,
            payer = ?verify_response.payer,
            "Payment verification failed"
        );

        return Err(McpError::validation(format!("Payment verification failed: {}", reason)));
    }

    tracing::info!(
        payer = ?verify_response.payer,
        "Payment verified successfully, proceeding to settlement"
    );

    // Settle the payment
    let settlement_response = facilitator.settle(payment_payload, payment_requirements).await?;

    if !settlement_response.success {
        let reason = settlement_response.error_reason
            .clone()
            .unwrap_or_else(|| "Settlement failed".to_string());
        
        tracing::error!(
            reason = %reason,
            transaction = %settlement_response.transaction,
            "Payment settlement failed"
        );

        return Err(McpError::server(format!("Payment settlement failed: {}", reason)));
    }

    tracing::info!(
        transaction = %settlement_response.transaction,
        network = %settlement_response.network,
        payer = ?settlement_response.payer,
        "Payment settled successfully"
    );

    Ok(settlement_response)
}

/// Builds payment requirements for a tool call
///
/// # Arguments
/// * `tool_name` - Name of the tool being called
/// * `config` - x402 configuration
/// * `network_id` - Network identifier
///
/// # Returns
/// * `McpResult<PaymentRequired>` - Payment requirements for the tool
pub fn build_payment_requirements(
    tool_name: &str,
    config: &X402Config,
    network_id: &str,
) -> McpResult<PaymentRequired> {
    let network_config = config.get_network(network_id)
        .ok_or_else(|| McpError::validation(format!("Network '{}' not configured", network_id)))?;

    // Build payment requirements for each supported asset
    let mut accepts = Vec::new();
    for asset in &network_config.assets {
        accepts.push(PaymentRequirements {
            scheme: "exact".to_string(),
            network: network_config.network.clone(),
            amount: "1000000".to_string(), // Default amount - should be configurable per tool
            asset: asset.address.clone(),
            pay_to: network_config.pay_to.clone(),
            max_timeout_seconds: 60, // Default timeout - should be configurable
            extra: None,
        });
    }

    if accepts.is_empty() {
        return Err(McpError::validation(format!(
            "No assets configured for network '{}'",
            network_id
        )));
    }

    Ok(PaymentRequired {
        x402_version: 2,
        error: Some(format!("Payment required to call tool '{}'", tool_name)),
        resource: ResourceInfo {
            url: format!("mcp://tool/{}", tool_name),
            description: Some(format!("MCP tool call: {}", tool_name)),
            mime_type: Some("application/json".to_string()),
        },
        accepts,
        extensions: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_extract_payment_payload_none() {
        let result = extract_payment_payload(None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_payment_payload_no_payment_field() {
        let meta = serde_json::json!({
            "other": "data"
        });
        let result = extract_payment_payload(Some(&meta)).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_payment_requirements() {
        let mut networks = HashMap::new();
        networks.insert(
            "solana-mainnet".to_string(),
            super::super::config::NetworkConfig {
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                assets: vec![super::super::config::AssetConfig {
                    address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    name: "USDC".to_string(),
                    decimals: 6,
                }],
                pay_to: "FeeRecipient123".to_string(),
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

        let result = build_payment_requirements("getBalance", &config, "solana-mainnet").unwrap();
        assert_eq!(result.x402_version, 2);
        assert_eq!(result.accepts.len(), 1);
        assert_eq!(result.accepts[0].scheme, "exact");
        assert_eq!(result.accepts[0].network, "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp");
    }

    #[test]
    fn test_create_payment_required_response() {
        let payment_required = PaymentRequired {
            x402_version: 2,
            error: Some("Payment required".to_string()),
            resource: ResourceInfo {
                url: "mcp://tool/getBalance".to_string(),
                description: Some("MCP tool call".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            accepts: vec![],
            extensions: None,
        };

        let response = create_payment_required_response(payment_required, Value::from(1));
        
        match response {
            JsonRpcMessage::Response(r) => {
                assert!(r.error.is_some());
                assert_eq!(r.error.unwrap().code, PAYMENT_REQUIRED_CODE);
            }
            _ => panic!("Expected Response message"),
        }
    }

    #[test]
    fn test_create_invalid_payment_response() {
        let response = create_invalid_payment_response(
            "Invalid signature".to_string(),
            Value::from(1),
        );
        
        match response {
            JsonRpcMessage::Response(r) => {
                assert!(r.error.is_some());
                assert_eq!(r.error.unwrap().code, INVALID_PAYMENT_CODE);
            }
            _ => panic!("Expected Response message"),
        }
    }
}
