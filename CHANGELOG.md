# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- x402 v2 payment protocol support (optional, feature-gated)
  - Core types for PaymentRequired, PaymentPayload, SettlementResponse, and VerifyResponse
  - CAIP-2 network validation for blockchain identifiers
  - Facilitator client with HTTP endpoints (/verify, /settle, /supported)
  - SVM exact scheme validation framework for Solana payments
  - MCP transport integration for payment flows
  - Comprehensive configuration with network and asset support
  - Retry logic with exponential backoff and jitter
  - Structured logging with trace/correlation IDs
  - Complete documentation in docs/x402-integration.md

### Changed
- Config struct now includes optional x402 configuration (feature-gated)
- Added `rand` dependency for retry jitter

## [1.1.1] - Previous Release

See git history for previous changes.
