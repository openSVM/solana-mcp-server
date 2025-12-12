//! x402 v2 Payment Protocol Support
//!
//! This module implements the x402 v2 payment protocol for monetizing MCP tool calls
//! and resources. It provides types, validation, and facilitator integration for
//! payment flows.
//!
//! The implementation follows the canonical x402 v2 specification:
//! https://github.com/coinbase/x402/blob/ce5085245c55c1a76416e445403cc3e10169b2e4/specs/x402-specification-v2.md

pub mod types;
pub mod config;
pub mod facilitator;
pub mod svm_exact;
pub mod validation;
pub mod mcp_integration;

pub use types::{
    PaymentRequired, PaymentPayload, PaymentRequirements, ResourceInfo,
    SettlementResponse, VerifyResponse, Extensions,
};
pub use config::X402Config;
pub use facilitator::FacilitatorClient;
pub use validation::{validate_caip2_network, validate_x402_version};
pub use mcp_integration::{
    create_payment_required_response, create_invalid_payment_response,
    extract_payment_payload, process_payment, build_payment_requirements,
    PAYMENT_REQUIRED_CODE, INVALID_PAYMENT_CODE,
};
