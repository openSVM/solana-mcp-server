//! x402 v2 Integration Tests
//!
//! These tests demonstrate the x402 payment protocol integration
//! with MCP transport layer.

#[cfg(feature = "x402")]
mod x402_tests {
    use solana_mcp_server::x402::{
        build_payment_requirements, create_payment_required_response,
        create_invalid_payment_response, extract_payment_payload,
        PaymentPayload, PaymentRequirements, ResourceInfo, X402Config,
        PAYMENT_REQUIRED_CODE, INVALID_PAYMENT_CODE,
    };
    use std::collections::HashMap;

    fn create_test_config() -> X402Config {
        let mut networks = HashMap::new();
        networks.insert(
            "solana-mainnet".to_string(),
            solana_mcp_server::x402::config::NetworkConfig {
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                assets: vec![solana_mcp_server::x402::config::AssetConfig {
                    address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    name: "USDC".to_string(),
                    decimals: 6,
                }],
                pay_to: "FeeRecipient123456789".to_string(),
                min_compute_unit_price: Some(1000),
                max_compute_unit_price: Some(10000),
            },
        );

        X402Config {
            enabled: true,
            facilitator_base_url: "https://facilitator.example.com".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            networks,
        }
    }

    #[test]
    fn test_payment_required_response() {
        let config = create_test_config();
        let payment_required = build_payment_requirements("getBalance", &config, "solana-mainnet")
            .expect("Failed to build payment requirements");

        assert_eq!(payment_required.x402_version, 2);
        assert!(!payment_required.accepts.is_empty());
        assert_eq!(payment_required.accepts[0].scheme, "exact");
        assert_eq!(
            payment_required.accepts[0].network,
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
        );

        let response = create_payment_required_response(payment_required, serde_json::json!(1));

        match response {
            solana_mcp_server::transport::JsonRpcMessage::Response(r) => {
                assert!(r.error.is_some());
                let error = r.error.unwrap();
                assert_eq!(error.code, PAYMENT_REQUIRED_CODE);
                assert!(error.data.is_some());
            }
            _ => panic!("Expected Response message"),
        }
    }

    #[test]
    fn test_invalid_payment_response() {
        let response = create_invalid_payment_response(
            "Invalid signature".to_string(),
            serde_json::json!(1),
        );

        match response {
            solana_mcp_server::transport::JsonRpcMessage::Response(r) => {
                assert!(r.error.is_some());
                let error = r.error.unwrap();
                assert_eq!(error.code, INVALID_PAYMENT_CODE);
                assert!(error.message.contains("Invalid signature"));
            }
            _ => panic!("Expected Response message"),
        }
    }

    #[test]
    fn test_extract_payment_payload_from_meta() {
        let payment_payload = PaymentPayload {
            x402_version: 2,
            resource: Some(ResourceInfo {
                url: "mcp://tool/getBalance".to_string(),
                description: Some("Test".to_string()),
                mime_type: Some("application/json".to_string()),
            }),
            accepted: PaymentRequirements {
                scheme: "exact".to_string(),
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                amount: "1000000".to_string(),
                asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                pay_to: "FeeRecipient123".to_string(),
                max_timeout_seconds: 60,
                extra: None,
            },
            payload: serde_json::json!({
                "transaction": "test_tx"
            }),
            extensions: None,
        };

        let meta = serde_json::json!({
            "payment": serde_json::to_value(&payment_payload).unwrap()
        });

        let extracted = extract_payment_payload(Some(&meta))
            .expect("Failed to extract payment payload");

        assert!(extracted.is_some());
        let extracted = extracted.unwrap();
        assert_eq!(extracted.x402_version, 2);
        assert_eq!(extracted.accepted.scheme, "exact");
    }

    #[test]
    fn test_extract_payment_payload_invalid_version() {
        let meta = serde_json::json!({
            "payment": {
                "x402Version": 1, // Invalid version
                "accepted": {
                    "scheme": "exact",
                    "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
                    "amount": "1000000",
                    "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "payTo": "FeeRecipient123",
                    "maxTimeoutSeconds": 60
                },
                "payload": {}
            }
        });

        let result = extract_payment_payload(Some(&meta));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_payment_payload_invalid_amount() {
        let meta = serde_json::json!({
            "payment": {
                "x402Version": 2,
                "accepted": {
                    "scheme": "exact",
                    "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
                    "amount": "invalid", // Invalid amount
                    "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "payTo": "FeeRecipient123",
                    "maxTimeoutSeconds": 60
                },
                "payload": {}
            }
        });

        let result = extract_payment_payload(Some(&meta));
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_flow_simulation() {
        // Step 1: Client makes request without payment
        // Server checks and determines payment is required

        let config = create_test_config();
        
        // Step 2: Build payment requirements
        let payment_required = build_payment_requirements("getBalance", &config, "solana-mainnet")
            .expect("Failed to build payment requirements");

        assert_eq!(payment_required.x402_version, 2);
        assert!(!payment_required.accepts.is_empty());

        // Step 3: Client receives payment requirements and creates payment
        let client_payment = PaymentPayload {
            x402_version: 2,
            resource: Some(payment_required.resource.clone()),
            accepted: payment_required.accepts[0].clone(),
            payload: serde_json::json!({
                "transaction": "base64_encoded_tx",
                "signature": "tx_signature"
            }),
            extensions: None,
        };

        // Step 4: Client includes payment in _meta field
        let meta = serde_json::json!({
            "payment": serde_json::to_value(&client_payment).unwrap()
        });

        // Step 5: Server extracts and validates payment
        let extracted = extract_payment_payload(Some(&meta))
            .expect("Failed to extract payment");
        
        assert!(extracted.is_some());
        let extracted = extracted.unwrap();
        assert_eq!(extracted.x402_version, 2);
        assert_eq!(extracted.accepted.amount, "1000000");

        // Step 6: Server would verify and settle with facilitator
        // (This would require actual facilitator integration)
    }
}

#[cfg(not(feature = "x402"))]
mod without_x402 {
    #[test]
    fn test_x402_disabled() {
        // When x402 feature is not enabled, these modules should not exist
        // This test just ensures the code compiles without the feature
        assert!(true);
    }
}
