//! x402 v2 Core Types
//!
//! Strongly-typed representations of x402 v2 protocol messages.
//! These types are transport-agnostic and scheme-agnostic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// x402 protocol version constant
pub const X402_VERSION: u32 = 2;

/// Resource information describing the protected resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceInfo {
    /// URL of the protected resource
    pub url: String,
    /// Human-readable description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the expected response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Payment requirements for a specific payment method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: String,
    /// Blockchain network identifier in CAIP-2 format (e.g., "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp")
    pub network: String,
    /// Required payment amount in atomic token units (string to prevent precision loss)
    pub amount: String,
    /// Token contract address or mint address
    pub asset: String,
    /// Recipient wallet address or role constant
    pub pay_to: String,
    /// Maximum time allowed for payment completion in seconds
    pub max_timeout_seconds: u64,
    /// Scheme-specific additional information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// Protocol extensions (optional functionality)
pub type Extensions = HashMap<String, ExtensionData>;

/// Extension data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtensionData {
    /// Extension-specific data
    pub info: serde_json::Value,
    /// JSON Schema defining the expected structure
    pub schema: serde_json::Value,
}

/// Payment Required response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequired {
    /// Protocol version (must be 2)
    pub x402_version: u32,
    /// Human-readable error message explaining why payment is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Information about the protected resource
    pub resource: ResourceInfo,
    /// Array of acceptable payment methods
    pub accepts: Vec<PaymentRequirements>,
    /// Protocol extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Payment Payload submitted by client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayload {
    /// Protocol version (must be 2)
    pub x402_version: u32,
    /// Information about the resource being accessed (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<ResourceInfo>,
    /// Payment method chosen from the accepts array
    pub accepted: PaymentRequirements,
    /// Scheme-specific payment data (e.g., signature, authorization)
    pub payload: serde_json::Value,
    /// Protocol extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Settlement Response from facilitator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SettlementResponse {
    /// Indicates whether settlement was successful
    pub success: bool,
    /// Error reason if settlement failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
    /// Address of the payer's wallet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
    /// Blockchain transaction hash (empty string if failed)
    pub transaction: String,
    /// Blockchain network identifier in CAIP-2 format
    pub network: String,
}

/// Verify Response from facilitator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    /// Indicates whether the payment authorization is valid
    pub is_valid: bool,
    /// Reason for invalidity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
    /// Address of the payer's wallet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_required_serialization() {
        let payment_required = PaymentRequired {
            x402_version: 2,
            error: Some("Payment required".to_string()),
            resource: ResourceInfo {
                url: "https://api.example.com/data".to_string(),
                description: Some("Premium data".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            accepts: vec![PaymentRequirements {
                scheme: "exact".to_string(),
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                amount: "1000000".to_string(),
                asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                pay_to: "FeeRecipient123456789".to_string(),
                max_timeout_seconds: 60,
                extra: None,
            }],
            extensions: None,
        };

        let json = serde_json::to_string(&payment_required).unwrap();
        let deserialized: PaymentRequired = serde_json::from_str(&json).unwrap();
        assert_eq!(payment_required, deserialized);
        assert_eq!(deserialized.x402_version, 2);
    }

    #[test]
    fn test_payment_payload_serialization() {
        let mut extra = HashMap::new();
        extra.insert("test".to_string(), serde_json::json!("value"));
        
        let payment_payload = PaymentPayload {
            x402_version: 2,
            resource: None,
            accepted: PaymentRequirements {
                scheme: "exact".to_string(),
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                amount: "1000000".to_string(),
                asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                pay_to: "FeeRecipient123456789".to_string(),
                max_timeout_seconds: 60,
                extra: Some(extra),
            },
            payload: serde_json::json!({
                "signature": "test_signature",
                "transaction": "test_tx"
            }),
            extensions: None,
        };

        let json = serde_json::to_string(&payment_payload).unwrap();
        let deserialized: PaymentPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payment_payload, deserialized);
    }

    #[test]
    fn test_settlement_response() {
        let response = SettlementResponse {
            success: true,
            error_reason: None,
            payer: Some("payer_address".to_string()),
            transaction: "tx_hash_123".to_string(),
            network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SettlementResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_verify_response() {
        let response = VerifyResponse {
            is_valid: true,
            invalid_reason: None,
            payer: Some("payer_address".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: VerifyResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}
