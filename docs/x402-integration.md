# x402 v2 Payment Protocol Integration

This document describes how to enable and use the x402 v2 payment protocol in the Solana MCP Server.

## Overview

The x402 v2 payment protocol enables monetization of MCP tool calls and resources. When enabled, the server can require payment before executing certain operations, with payments verified and settled through a facilitator service.

The implementation follows the canonical x402 v2 specification:
https://github.com/coinbase/x402/blob/ce5085245c55c1a76416e445403cc3e10169b2e4/specs/x402-specification-v2.md

## Enabling x402

### 1. Build with x402 Feature

The x402 protocol support is behind a feature flag and disabled by default. To enable it:

```bash
cargo build --features x402
cargo run --features x402
```

### 2. Configure x402

Add the following configuration to your `config.json`:

```json
{
  "rpc_url": "https://api.mainnet-beta.solana.com",
  "commitment": "confirmed",
  "protocol_version": "2025-06-18",
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://facilitator.example.com",
    "request_timeout_seconds": 30,
    "max_retries": 3,
    "networks": {
      "solana-mainnet": {
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "assets": [
          {
            "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "name": "USDC",
            "decimals": 6
          }
        ],
        "pay_to": "YourFeeRecipientAddress",
        "min_compute_unit_price": 1000,
        "max_compute_unit_price": 100000
      }
    }
  }
}
```

### Configuration Fields

#### x402 Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | boolean | Yes | Enable/disable x402 protocol (default: false) |
| `facilitator_base_url` | string | Yes* | Base URL of the facilitator service (*required when enabled) |
| `request_timeout_seconds` | number | No | HTTP request timeout (default: 30) |
| `max_retries` | number | No | Maximum retry attempts (default: 3) |
| `networks` | object | Yes* | Supported networks and assets (*required when enabled) |

#### Network Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `network` | string | Yes | CAIP-2 network identifier (e.g., "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp") |
| `assets` | array | Yes | List of supported assets on this network |
| `pay_to` | string | Yes | Payment recipient address |
| `min_compute_unit_price` | number | No | Minimum compute unit price (SVM only) |
| `max_compute_unit_price` | number | No | Maximum compute unit price (SVM only) |

#### Asset Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | string | Yes | Asset identifier (e.g., token mint address) |
| `name` | string | Yes | Human-readable asset name |
| `decimals` | number | Yes | Number of decimal places |

## Payment Flow

### 1. Client Request (No Payment)

When a client makes a request to a protected tool without payment:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "getBalance",
    "arguments": {
      "pubkey": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"
    }
  }
}
```

### 2. Payment Required Response

The server returns a Payment Required error (code -40200) with payment requirements:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -40200,
    "message": "Payment required to call tool 'getBalance'",
    "data": {
      "x402Version": 2,
      "error": "Payment required to call tool 'getBalance'",
      "resource": {
        "url": "mcp://tool/getBalance",
        "description": "MCP tool call: getBalance",
        "mimeType": "application/json"
      },
      "accepts": [
        {
          "scheme": "exact",
          "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
          "amount": "1000000",
          "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "payTo": "YourFeeRecipientAddress",
          "maxTimeoutSeconds": 60
        }
      ]
    }
  }
}
```

### 3. Client Request with Payment

The client retries the request with payment information in the `_meta` field:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "getBalance",
    "arguments": {
      "pubkey": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"
    },
    "_meta": {
      "payment": {
        "x402Version": 2,
        "accepted": {
          "scheme": "exact",
          "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
          "amount": "1000000",
          "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "payTo": "YourFeeRecipientAddress",
          "maxTimeoutSeconds": 60
        },
        "payload": {
          "transaction": "base64_encoded_solana_transaction",
          "signature": "transaction_signature"
        }
      }
    }
  }
}
```

### 4. Success Response with Settlement

If payment is valid, the server executes the tool and includes settlement information:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Balance: 1.5 SOL"
      }
    ],
    "_meta": {
      "settlement": {
        "success": true,
        "transaction": "5vR...abc",
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "payer": "ClientWalletAddress"
      }
    }
  }
}
```

### 5. Invalid Payment Response

If payment is invalid, the server returns an Invalid Payment error (code -40201):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -40201,
    "message": "Invalid payment: insufficient funds"
  }
}
```

## Facilitator Endpoints

The facilitator service must implement these HTTP endpoints:

### POST /verify

Verifies a payment authorization without executing blockchain transaction.

**Request:**
```json
{
  "paymentPayload": { /* PaymentPayload object */ },
  "paymentRequirements": { /* PaymentRequirements object */ }
}
```

**Response:**
```json
{
  "isValid": true,
  "payer": "ClientWalletAddress"
}
```

### POST /settle

Executes payment settlement on the blockchain.

**Request:** Same as /verify

**Response:**
```json
{
  "success": true,
  "transaction": "5vR...abc",
  "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
  "payer": "ClientWalletAddress"
}
```

### GET /supported

Returns supported payment schemes and networks.

**Response:**
```json
{
  "kinds": [
    {
      "x402Version": 2,
      "scheme": "exact",
      "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
    }
  ],
  "extensions": [],
  "signers": {
    "solana:*": ["FacilitatorSignerAddress"]
  }
}
```

## SVM Exact Scheme

For Solana (SVM) networks using the "exact" scheme, the server validates:

1. **Strict Instruction Layout**
   - ComputeBudgetInstruction::SetComputeUnitLimit
   - ComputeBudgetInstruction::SetComputeUnitPrice
   - (Optional) AssociatedTokenAccount::Create
   - Token::TransferChecked

2. **Facilitator Fee Payer Constraints**
   - Fee payer must not appear in instruction accounts
   - Fee payer must not be transfer authority or source

3. **Compute Unit Price Bounds**
   - Must be within configured min/max range

4. **Destination Account Validation**
   - Destination ATA must match payTo/asset

5. **Transfer Amount**
   - Must exactly equal required amount

## Security Considerations

1. **HTTPS Required**: All facilitator URLs must use HTTPS
2. **Timeout Bounds**: Timeout must be between 1 and 300 seconds
3. **Retry with Jitter**: Exponential backoff with random jitter
4. **Structured Logging**: All operations logged with trace ID
5. **Input Validation**: All payment data validated before processing

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| -40200 | Payment Required | Payment is required to access the resource |
| -40201 | Invalid Payment | Payment payload is invalid or verification failed |

## Troubleshooting

### x402 Not Working

1. Verify feature flag is enabled: `cargo build --features x402`
2. Check `x402.enabled` is `true` in config.json
3. Verify facilitator URL is accessible
4. Check logs for detailed error messages

### Payment Verification Fails

1. Verify payment payload format matches specification
2. Check compute unit price is within bounds (SVM)
3. Verify asset and network are configured
4. Review facilitator logs

### Settlement Fails

1. Verify payer has sufficient balance
2. Check transaction instruction layout (SVM)
3. Verify fee payer constraints
4. Review blockchain explorer for transaction details

## Example: Complete Flow

See `tests/x402_integration.rs` for complete integration test examples demonstrating:
- Payment Required response
- Valid payment processing
- Invalid payment handling
- Settlement verification
