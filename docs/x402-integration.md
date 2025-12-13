# x402 v2 Payment Protocol Integration

This document describes how to enable and use the x402 v2 payment protocol in the Solana MCP Server.

## Overview

The x402 v2 payment protocol enables monetization of MCP tool calls and resources. When enabled, the server can require payment before executing certain operations, with payments verified and settled through a facilitator service.

The implementation follows the canonical x402 v2 specification:
https://github.com/coinbase/x402/blob/ce5085245c55c1a76416e445403cc3e10169b2e4/specs/x402-specification-v2.md

## Web3 Developer Onboarding

This section helps Web3 developers understand how x402 integrates blockchain payments with MCP (Model Context Protocol).

### What is x402?

**x402** is a protocol for monetizing API access using blockchain payments. Think of it as "HTTP 402 Payment Required" but for Web3:

- **Traditional Web2**: HTTP 402 status code (rarely used) - client needs to pay via credit card/PayPal
- **Web3 x402**: Client pays with crypto (USDC, SOL, etc.) via blockchain transactions
- **Verification**: Payment is verified on-chain before the API executes the request

### Key Concepts for Web3 Developers

#### 1. **CAIP-2 Network Identifiers**

CAIP-2 is a standard for identifying blockchain networks in a human-readable format:

```
<namespace>:<reference>
```

**Examples:**
- Solana Mainnet: `solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp`
- Solana Devnet: `solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1`
- Ethereum Mainnet: `eip155:1`
- Polygon: `eip155:137`

**Why it matters:** Your server can support multiple chains, and clients specify which chain they're paying on.

#### 2. **SPL Token Payments (Solana)**

On Solana, payments use SPL tokens (like USDC):

- **Token Mint Address**: The on-chain program that controls the token (e.g., USDC: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`)
- **Token Account**: Each wallet has an Associated Token Account (ATA) for each token type
- **Transfer**: Payment moves tokens from payer's ATA to your ATA

**Decimals Matter:**
- USDC has 6 decimals: `1000000` units = 1 USDC
- SOL has 9 decimals: `1000000000` lamports = 1 SOL
- Always specify amounts in the smallest unit (like wei in Ethereum)

#### 3. **Compute Units and Gas**

Solana uses **Compute Units** instead of gas:

- **Compute Unit Limit**: Max compute budget for the transaction (like gas limit)
- **Compute Unit Price**: Price per compute unit in microlamports (like gas price)
- **Total Fee**: `compute_units_used × compute_unit_price` microlamports

**Why this matters for x402:**
- Prevents clients from setting extremely high prices (gas griefing)
- You configure min/max compute unit price bounds
- Example: `min: 1000, max: 50000` microlamports per CU

#### 4. **The Facilitator Service**

The facilitator is a trusted service that:

1. **Verifies** payment authorizations (simulates transaction without broadcasting)
2. **Settles** payments (broadcasts transaction to blockchain)
3. **Returns** transaction signatures and settlement receipts

**Why use a facilitator?**
- Your API server doesn't need to manage private keys
- Facilitator handles transaction complexity
- Separates payment logic from business logic
- Can batch multiple payments for efficiency

**Facilitator Endpoints:**
- `POST /verify` - Check if payment is valid (no blockchain interaction)
- `POST /settle` - Execute payment on blockchain
- `GET /supported` - List supported networks and payment schemes

#### 5. **Payment Flow for Web3 Developers**

**Step 1: Client Request (No Payment)**
```javascript
// Client makes request without payment
const response = await fetch('https://api.example.com/mcp', {
  method: 'POST',
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: {
      name: 'getBalance',
      arguments: { pubkey: 'Gh9Z...' }
    }
  })
});
```

**Step 2: Server Returns Payment Required**
```json
{
  "error": {
    "code": -40200,
    "message": "Payment required",
    "data": {
      "x402Version": 2,
      "accepts": [{
        "scheme": "exact",
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "amount": "10000",
        "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "payTo": "FeeRecipient...",
        "maxTimeoutSeconds": 60
      }]
    }
  }
}
```

**Step 3: Client Creates Payment Transaction**
```javascript
// Using @solana/web3.js
import { Connection, Transaction, SystemProgram } from '@solana/web3.js';
import { createTransferCheckedInstruction } from '@solana/spl-token';

// 1. Create transaction with required instructions
const transaction = new Transaction();

// Add compute budget instructions
transaction.add(
  ComputeBudgetProgram.setComputeUnitLimit({ units: 200000 }),
  ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 5000 })
);

// Add token transfer instruction
transaction.add(
  createTransferCheckedInstruction(
    sourceATA,      // Your token account
    tokenMint,      // USDC mint
    destATA,        // Server's token account
    wallet.publicKey, // Your wallet
    10000,          // Amount (0.01 USDC)
    6               // Decimals
  )
);

// 2. Sign transaction
transaction.feePayer = wallet.publicKey;
transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
await wallet.signTransaction(transaction);

// 3. Serialize transaction
const serializedTx = transaction.serialize().toString('base64');
```

**Step 4: Client Retries with Payment**
```javascript
const retryResponse = await fetch('https://api.example.com/mcp', {
  method: 'POST',
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: {
      name: 'getBalance',
      arguments: { pubkey: 'Gh9Z...' },
      _meta: {
        payment: {
          x402Version: 2,
          accepted: {
            scheme: 'exact',
            network: 'solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp',
            amount: '10000',
            asset: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
            payTo: 'FeeRecipient...',
            maxTimeoutSeconds: 60
          },
          payload: {
            transaction: serializedTx,
            signature: transaction.signatures[0].signature.toString('base64')
          }
        }
      }
    }
  })
});
```

**Step 5: Server Verifies and Settles**
- Server sends payment to facilitator `/verify` endpoint
- Facilitator simulates transaction (checks balance, signature, etc.)
- If valid, facilitator broadcasts to blockchain via `/settle`
- Server executes the paid operation
- Returns result with settlement receipt

#### 6. **Security Considerations for Web3**

**Transaction Validation:**
- ✅ Signature verification: Proves the wallet owner authorized payment
- ✅ Amount validation: Ensures payment matches requirements exactly
- ✅ Destination validation: Confirms payment goes to correct address
- ✅ Timeout validation: Prevents replay attacks with old transactions
- ✅ Compute unit bounds: Prevents excessive gas price attacks

**Fee Payer Rules:**
- The facilitator pays blockchain fees (gas)
- Facilitator's wallet must NOT be the source of payment tokens
- This prevents the facilitator from paying itself
- Enforced in SVM exact scheme validation

**Replay Protection:**
- Use `maxTimeoutSeconds` to limit transaction validity
- Recent blockhash ensures transaction is current
- Signature is unique per transaction

#### 7. **Common Web3 Integration Patterns**

**Pattern 1: Wallet Integration**
```javascript
// Using Phantom wallet (Solana)
const connectWallet = async () => {
  if (window.solana) {
    await window.solana.connect();
    return window.solana;
  }
  throw new Error('Phantom wallet not installed');
};
```

**Pattern 2: Multi-Wallet Support**
```javascript
// Support multiple wallets
const wallets = {
  phantom: window.solana,
  solflare: window.solflare,
  // etc.
};
```

**Pattern 3: Token Account Creation**
```javascript
// Create ATA if it doesn't exist
import { getAssociatedTokenAddress, createAssociatedTokenAccountInstruction } from '@solana/spl-token';

const ata = await getAssociatedTokenAddress(mint, owner);
const ataInfo = await connection.getAccountInfo(ata);

if (!ataInfo) {
  // Add ATA creation instruction before transfer
  transaction.add(
    createAssociatedTokenAccountInstruction(
      payer, ata, owner, mint
    )
  );
}
```

**Pattern 4: Transaction Status Monitoring**
```javascript
// Monitor transaction confirmation
const signature = await connection.sendRawTransaction(transaction.serialize());
await connection.confirmTransaction(signature, 'confirmed');

// Check if transaction succeeded
const status = await connection.getSignatureStatus(signature);
if (status.value?.confirmationStatus === 'confirmed') {
  console.log('Payment confirmed!');
}
```

#### 8. **Testing Your Integration**

**Local Development:**
```bash
# 1. Use Solana devnet
# 2. Get devnet SOL from faucet
solana airdrop 2 <your-wallet-address> --url devnet

# 3. Use devnet USDC or create test token
# 4. Configure server for devnet:
{
  "x402": {
    "networks": {
      "solana-devnet": {
        "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        ...
      }
    }
  }
}
```

**Mock Facilitator:**
```javascript
// For testing without blockchain
// Mock facilitator always returns success
const mockFacilitator = {
  verify: () => ({ isValid: true, payer: 'MockWallet' }),
  settle: () => ({ 
    success: true, 
    transaction: 'MockTxHash',
    network: 'solana:devnet'
  })
};
```

#### 9. **Debugging Payment Issues**

**Common Issues:**

1. **"Insufficient funds"**
   - Check wallet balance: `solana balance <address>`
   - Ensure enough tokens + SOL for fees

2. **"Invalid signature"**
   - Verify wallet signed the transaction
   - Check transaction serialization is correct

3. **"Timeout exceeded"**
   - Transaction took too long to create
   - Generate fresh recent blockhash
   - Reduce `maxTimeoutSeconds` if too long

4. **"Compute unit price out of bounds"**
   - Check server's min/max compute unit price settings
   - Adjust your transaction's compute unit price

5. **"Settlement failed"**
   - Check blockchain explorer for transaction
   - Verify facilitator has SOL for fees
   - Ensure ATA exists for destination

**Useful Tools:**
- Solana Explorer: https://explorer.solana.com/
- Solscan: https://solscan.io/
- Web3.js Console: `node` REPL with @solana/web3.js

#### 10. **Production Checklist**

Before going to production:

- [ ] Test on devnet with real transactions
- [ ] Configure mainnet network IDs correctly
- [ ] Set appropriate compute unit price bounds
- [ ] Implement proper error handling in client
- [ ] Add transaction confirmation waiting
- [ ] Set up monitoring/alerting for failed payments
- [ ] Document pricing for users
- [ ] Test with multiple wallets (Phantom, Solflare, etc.)
- [ ] Implement graceful fallback for payment failures
- [ ] Consider offering free tier or trial credits

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

## Use Cases

This section provides 10+ detailed use cases demonstrating how to use x402 payment protocol in various scenarios.

### Use Case 1: Premium Data API Access

**Scenario:** A data provider wants to charge 0.01 USDC per balance check request.

**Setup:**
```json
{
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://facilitator.example.com",
    "networks": {
      "solana-mainnet": {
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "assets": [{
          "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "name": "USDC",
          "decimals": 6
        }],
        "pay_to": "DataProviderWalletAddress",
        "min_compute_unit_price": 1000,
        "max_compute_unit_price": 50000
      }
    }
  }
}
```

**Usage:**
1. Client calls `getBalance` without payment
2. Server returns Payment Required with amount "10000" (0.01 USDC with 6 decimals)
3. Client creates signed transaction and includes in `_meta.payment`
4. Server verifies and settles payment
5. Server executes getBalance and returns result with settlement receipt

### Use Case 2: Rate-Limited Free Tier with Paid Overflow

**Scenario:** Allow 100 free requests per hour, then require payment for additional requests.

**Implementation Strategy:**
- Track request counts per client (not implemented in current version)
- After limit exceeded, return Payment Required error
- Configure small payment amount (e.g., 0.001 USDC per request)

**Configuration:**
```json
{
  "x402": {
    "enabled": true,
    "networks": {
      "solana-mainnet": {
        "assets": [{
          "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "name": "USDC",
          "decimals": 6
        }],
        "pay_to": "ServiceProviderAddress"
      }
    }
  }
}
```

**Payment Required Response:**
```json
{
  "error": {
    "code": -40200,
    "data": {
      "x402Version": 2,
      "error": "Rate limit exceeded. Payment required for additional requests.",
      "resource": {
        "url": "mcp://tool/getBalance"
      },
      "accepts": [{
        "scheme": "exact",
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "amount": "1000",
        "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "payTo": "ServiceProviderAddress",
        "maxTimeoutSeconds": 60
      }]
    }
  }
}
```

### Use Case 3: Multi-Network Support

**Scenario:** Support payments on both Solana mainnet and devnet for testing.

**Configuration:**
```json
{
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://facilitator.example.com",
    "networks": {
      "solana-mainnet": {
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "assets": [{
          "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "name": "USDC",
          "decimals": 6
        }],
        "pay_to": "MainnetRecipient",
        "min_compute_unit_price": 1000,
        "max_compute_unit_price": 100000
      },
      "solana-devnet": {
        "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        "assets": [{
          "address": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr",
          "name": "DevUSDC",
          "decimals": 6
        }],
        "pay_to": "DevnetRecipient",
        "min_compute_unit_price": 100,
        "max_compute_unit_price": 10000
      }
    }
  }
}
```

**Usage:**
- Clients can choose which network to pay on
- Devnet for testing with lower compute unit prices
- Mainnet for production with real tokens

### Use Case 4: Multiple Token Support

**Scenario:** Accept payments in USDC, USDT, or SOL (wrapped).

**Configuration:**
```json
{
  "x402": {
    "networks": {
      "solana-mainnet": {
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "assets": [
          {
            "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "name": "USDC",
            "decimals": 6
          },
          {
            "address": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            "name": "USDT",
            "decimals": 6
          },
          {
            "address": "So11111111111111111111111111111111111111112",
            "name": "Wrapped SOL",
            "decimals": 9
          }
        ],
        "pay_to": "MultiTokenRecipient"
      }
    }
  }
}
```

**Payment Required Response:**
Server returns all accepted payment methods, client chooses one:
```json
{
  "accepts": [
    {
      "scheme": "exact",
      "amount": "10000",
      "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "extra": {"name": "USDC"}
    },
    {
      "scheme": "exact",
      "amount": "10000",
      "asset": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
      "extra": {"name": "USDT"}
    },
    {
      "scheme": "exact",
      "amount": "100000000",
      "asset": "So11111111111111111111111111111111111111112",
      "extra": {"name": "Wrapped SOL"}
    }
  ]
}
```

### Use Case 5: Tiered Pricing by Tool

**Scenario:** Different tools have different prices.

**Strategy:**
- Configure base payment amounts per tool (implementation-dependent)
- Return appropriate amount in Payment Required response

**Example Pricing:**
- `getBalance`: 0.001 USDC (1000 units)
- `getTransaction`: 0.005 USDC (5000 units)
- `getProgramAccounts`: 0.01 USDC (10000 units)

**Payment Required for Expensive Operation:**
```json
{
  "error": {
    "code": -40200,
    "data": {
      "x402Version": 2,
      "error": "Payment required for getProgramAccounts",
      "resource": {
        "url": "mcp://tool/getProgramAccounts",
        "description": "Expensive operation requiring 0.01 USDC"
      },
      "accepts": [{
        "scheme": "exact",
        "amount": "10000",
        "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "payTo": "ProviderAddress",
        "maxTimeoutSeconds": 60
      }]
    }
  }
}
```

### Use Case 6: Testing with Mock Facilitator

**Scenario:** Test payment integration without real blockchain transactions.

**Setup Mock Facilitator:**
```bash
# Run local mock facilitator for testing
npm install -g @x402/mock-facilitator
x402-mock-facilitator --port 3001
```

**Configuration:**
```json
{
  "x402": {
    "enabled": true,
    "facilitator_base_url": "http://localhost:3001",
    "request_timeout_seconds": 10,
    "max_retries": 1,
    "networks": {
      "solana-devnet": {
        "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        "assets": [{
          "address": "TestTokenMint",
          "name": "TestUSDC",
          "decimals": 6
        }],
        "pay_to": "TestRecipient"
      }
    }
  }
}
```

**Testing Flow:**
1. Mock facilitator always returns `isValid: true` for /verify
2. Mock facilitator returns mock transaction hash for /settle
3. Test payment flow without actual blockchain interaction
4. Verify error handling and retry logic

### Use Case 7: Handling Payment Verification Failures

**Scenario:** Client submits payment but verification fails.

**Common Failure Reasons:**
1. Insufficient balance
2. Invalid signature
3. Expired authorization
4. Amount mismatch

**Example Invalid Payment Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -40201,
    "message": "Invalid payment: insufficient balance"
  }
}
```

**Client Recovery Strategy:**
1. Check wallet balance
2. Ensure sufficient funds for payment + gas
3. Generate new payment authorization
4. Retry request with updated payment

### Use Case 8: Compute Unit Price Bounds for Gas Abuse Prevention

**Scenario:** Prevent clients from submitting transactions with excessive compute unit prices.

**Configuration:**
```json
{
  "x402": {
    "networks": {
      "solana-mainnet": {
        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        "assets": [{
          "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "name": "USDC",
          "decimals": 6
        }],
        "pay_to": "RecipientAddress",
        "min_compute_unit_price": 1000,
        "max_compute_unit_price": 50000
      }
    }
  }
}
```

**Validation:**
- Server validates compute unit price is within [1000, 50000] microlamports
- Rejects transactions with prices outside bounds
- Returns Invalid Payment error with reason

**Example Rejection:**
```json
{
  "error": {
    "code": -40201,
    "message": "Invalid payment: Compute unit price 100000 out of bounds [1000, 50000]"
  }
}
```

### Use Case 9: Monitoring Payment Flow with Trace IDs

**Scenario:** Track payment requests across verification, settlement, and tool execution.

**How It Works:**
- Each payment request gets a unique UUID trace ID
- Trace ID logged at all stages: verify, settle, execute
- Use trace ID to correlate logs across services

**Example Log Output:**
```
INFO [trace_id=550e8400-e29b-41d4-a716-446655440000] Verifying payment authorization
INFO [trace_id=550e8400-e29b-41d4-a716-446655440000] Payment verified successfully
INFO [trace_id=550e8400-e29b-41d4-a716-446655440000] Settling payment
INFO [trace_id=550e8400-e29b-41d4-a716-446655440000] Payment settled: tx=5vR...abc
INFO [trace_id=550e8400-e29b-41d4-a716-446655440000] Executing tool: getBalance
```

**Usage:**
```bash
# Filter logs by trace ID
grep "550e8400-e29b-41d4-a716-446655440000" server.log

# Track payment flow end-to-end
# Useful for debugging failed payments
```

### Use Case 10: Retry Logic for Transient Failures

**Scenario:** Facilitator temporarily unavailable or network issues.

**Configuration:**
```json
{
  "x402": {
    "facilitator_base_url": "https://facilitator.example.com",
    "request_timeout_seconds": 30,
    "max_retries": 3
  }
}
```

**How It Works:**
1. First request fails (network timeout)
2. Wait 100ms + random jitter
3. Retry request
4. If fails again, wait 200ms + jitter
5. Retry request
6. If fails again, wait 400ms + jitter
7. Final retry
8. If all retries exhausted, return error to client

**Retry Timing:**
- Retry 1: 100ms + random(0-100ms)
- Retry 2: 200ms + random(0-100ms)
- Retry 3: 400ms + random(0-100ms)

**Example Error After Exhausted Retries:**
```json
{
  "error": {
    "code": -32603,
    "message": "Facilitator request failed after 3 attempts"
  }
}
```

### Use Case 11: Migration from Free to Paid API

**Scenario:** Gradually introduce payments without breaking existing clients.

**Phase 1 - Preparation:**
```json
{
  "x402": {
    "enabled": false  // Feature disabled, prepare infrastructure
  }
}
```

**Phase 2 - Testing:**
```json
{
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://staging-facilitator.example.com"
    // Test with staging facilitator
  }
}
```

**Phase 3 - Production:**
```json
{
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://facilitator.example.com"
    // Switch to production facilitator
  }
}
```

**Communication Strategy:**
- Announce payment requirement with 30-day notice
- Provide documentation and examples
- Offer free tier or credits for transition period

### Use Case 12: Handling Settlement Failures

**Scenario:** Payment verification succeeds but settlement fails.

**Possible Causes:**
1. Network congestion
2. Transaction simulation fails
3. Blockchain state changed between verify and settle

**Example Settlement Failure Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Payment settlement failed: transaction simulation failed"
  }
}
```

**Client Recovery:**
1. Check blockchain state
2. Verify account balances unchanged
3. Generate new payment authorization
4. Retry entire payment flow

**Server Behavior:**
- Log settlement failure with trace ID
- Do NOT execute paid operation
- Return error to client immediately
- No partial charges

### Use Case 13: Webhook Integration for Payment Events

**Scenario:** Notify external systems when payments are received.

**Note:** Webhook support not in current implementation, but shows integration pattern.

**Future Configuration:**
```json
{
  "x402": {
    "webhooks": {
      "payment_verified": "https://your-system.com/webhooks/payment-verified",
      "payment_settled": "https://your-system.com/webhooks/payment-settled",
      "payment_failed": "https://your-system.com/webhooks/payment-failed"
    }
  }
}
```

**Webhook Payload Example:**
```json
{
  "event": "payment_settled",
  "timestamp": "2025-12-12T15:00:00Z",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000",
  "payment": {
    "amount": "10000",
    "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "payer": "ClientWalletAddress",
    "transaction": "5vR...abc",
    "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
  },
  "tool": "getBalance"
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
