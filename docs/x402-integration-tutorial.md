# x402 Integration Tutorial: Step-by-Step Guide

This tutorial walks you through integrating x402 v2 payment protocol into the Solana MCP Server, enabling you to monetize your MCP tool calls with blockchain payments.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Step 1: Enable x402 Feature](#step-1-enable-x402-feature)
4. [Step 2: Configure Your Server](#step-2-configure-your-server)
5. [Step 3: Set Up Facilitator Service](#step-3-set-up-facilitator-service)
6. [Step 4: Configure Payment Requirements](#step-4-configure-payment-requirements)
7. [Step 5: Implement Client-Side Integration](#step-5-implement-client-side-integration)
8. [Step 6: Test Your Integration](#step-6-test-your-integration)
9. [Step 7: Deploy to Production](#step-7-deploy-to-production)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.70+** installed (`rustc --version`)
- **Solana CLI** tools (`solana --version`)
- **A Solana wallet** with devnet SOL for testing
- **Basic understanding** of Solana transactions and SPL tokens
- **A facilitator service** URL (or use mock facilitator for testing)

## Quick Start

For the impatient, here's a 5-minute setup:

```bash
# 1. Clone the repo
git clone https://github.com/openSVM/solana-mcp-server.git
cd solana-mcp-server

# 2. Build with x402 feature
cargo build --release --features x402

# 3. Create config file
cat > config.json << 'EOF'
{
  "rpc_url": "https://api.devnet.solana.com",
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://facilitator.example.com",
    "networks": {
      "solana-devnet": {
        "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        "assets": [
          {
            "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "name": "USDC",
            "decimals": 6
          }
        ],
        "pay_to": "YOUR_RECIPIENT_WALLET_ADDRESS",
        "min_compute_unit_price": 1000,
        "max_compute_unit_price": 100000
      }
    }
  }
}
EOF

# 4. Run the server
./target/release/solana-mcp-server --config config.json
```

Now let's go through each step in detail.

---

## Step 1: Enable x402 Feature

### 1.1 Build with x402 Feature Flag

The x402 functionality is behind a feature flag (default **off**) to ensure zero impact on existing installations.

```bash
# Build with x402 enabled
cargo build --release --features x402

# Or for development
cargo build --features x402
```

### 1.2 Verify x402 is Available

Check that x402 is compiled in:

```bash
# The binary should be larger with x402 enabled
ls -lh target/release/solana-mcp-server

# Run with --help to see x402-related options
./target/release/solana-mcp-server --help
```

---

## Step 2: Configure Your Server

### 2.1 Create Configuration File

Create a `config.json` file with x402 configuration:

```json
{
  "rpc_url": "https://api.devnet.solana.com",
  "commitment": "confirmed",
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://facilitator.example.com",
    "timeout_seconds": 30,
    "max_retries": 3,
    "networks": {
      "solana-devnet": {
        "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        "assets": [
          {
            "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "name": "USDC",
            "decimals": 6
          }
        ],
        "pay_to": "FeeRecipient11111111111111111111111111111111",
        "min_compute_unit_price": 1000,
        "max_compute_unit_price": 100000
      }
    }
  }
}
```

### 2.2 Configuration Options Explained

| Option | Required | Description |
|--------|----------|-------------|
| `enabled` | Yes | Set to `true` to activate x402 |
| `facilitator_base_url` | Yes | Your facilitator service endpoint (HTTPS required) |
| `timeout_seconds` | No | HTTP request timeout (default: 30) |
| `max_retries` | No | Max retry attempts for facilitator calls (default: 3) |
| `networks` | Yes | Map of network configurations (see below) |

**Network Configuration:**

| Field | Required | Description |
|-------|----------|-------------|
| `network` | Yes | CAIP-2 network identifier (e.g., `solana:EtWT...`) |
| `assets` | Yes | Array of accepted SPL tokens |
| `pay_to` | Yes | Your wallet address to receive payments |
| `min_compute_unit_price` | Yes | Minimum gas price (prevents too-low fees) |
| `max_compute_unit_price` | Yes | Maximum gas price (prevents abuse) |

**Asset Configuration:**

| Field | Required | Description |
|-------|----------|-------------|
| `address` | Yes | SPL token mint address |
| `name` | Yes | Human-readable token name |
| `decimals` | Yes | Token decimals (e.g., 6 for USDC, 9 for SOL) |

### 2.3 Get CAIP-2 Network Identifiers

CAIP-2 format: `<namespace>:<reference>`

**Common Solana Networks:**

```
Mainnet:  solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp
Devnet:   solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1
Testnet:  solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z
```

The `reference` is the **genesis hash** of the network.

```bash
# Get genesis hash for current network
solana genesis-hash
```

### 2.4 Common Token Addresses

**Devnet:**
```json
{
  "USDC": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "USDT": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
}
```

**Mainnet:**
```json
{
  "USDC": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "USDT": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
  "SOL (Wrapped)": "So11111111111111111111111111111111111111112"
}
```

---

## Step 3: Set Up Facilitator Service

The facilitator service validates and settles payments. You can use an existing facilitator or run your own.

### 3.1 Use Mock Facilitator (Testing)

For local development, create a mock facilitator:

```javascript
// mock-facilitator.js
const express = require('express');
const app = express();
app.use(express.json());

// POST /verify - Always approve
app.post('/verify', (req, res) => {
  console.log('Verify request:', JSON.stringify(req.body, null, 2));
  res.json({
    valid: true,
    message: "Mock verification successful"
  });
});

// POST /settle - Always succeed
app.post('/settle', (req, res) => {
  console.log('Settle request:', JSON.stringify(req.body, null, 2));
  res.json({
    settled: true,
    transaction_id: "mock_tx_" + Date.now(),
    message: "Mock settlement successful"
  });
});

// GET /supported - Return your configured networks
app.get('/supported', (req, res) => {
  res.json({
    networks: [
      {
        network: "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        schemes: ["exact"],
        assets: ["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"]
      }
    ]
  });
});

app.listen(3001, () => {
  console.log('Mock facilitator running on http://localhost:3001');
});
```

Run it:

```bash
# Install express
npm install express

# Run mock facilitator
node mock-facilitator.js
```

Update your config:

```json
{
  "x402": {
    "facilitator_base_url": "http://localhost:3001",
    ...
  }
}
```

### 3.2 Production Facilitator Setup

For production, you'll need a real facilitator service that:

1. **Validates Solana transactions** against x402 v2 spec
2. **Checks payment amounts** match requirements
3. **Verifies ATA destinations** are correct
4. **Settles payments** on-chain
5. **Returns signed receipts**

Facilitator API Requirements:

**POST /verify**
- Input: `{ "x402Version": 2, "accepted": {...}, "payload": "..." }`
- Output: `{ "valid": true/false, "message": "..." }`

**POST /settle**
- Input: `{ "x402Version": 2, "accepted": {...}, "payload": "..." }`
- Output: `{ "settled": true/false, "transaction_id": "...", "receipt": {...} }`

**GET /supported**
- Output: List of supported networks, schemes, and assets

---

## Step 4: Configure Payment Requirements

### 4.1 Define Which Tools Require Payment

In your MCP server code, add payment requirements to specific tools:

```rust
use solana_mcp_server::x402::mcp_integration::{create_payment_required_response, PaymentRequirementsBuilder};

// Example: Make getBalance require payment
pub async fn handle_get_balance(address: String, payment: Option<PaymentPayload>) -> Result<Response> {
    // Check if payment is provided
    if payment.is_none() {
        // Return payment required error
        let requirements = PaymentRequirementsBuilder::new(
            "mcp://tool/getBalance",
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1", // Network
            "1000000", // Amount (1 USDC in smallest units)
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
        )
        .with_timeout(300) // 5 minutes
        .build();
        
        return create_payment_required_response(requirements);
    }
    
    // Verify and settle payment
    let config = load_x402_config()?;
    let facilitator = FacilitatorClient::new(&config.facilitator_base_url);
    
    // Verify payment
    let verify_result = facilitator.verify(payment.unwrap()).await?;
    if !verify_result.valid {
        return Err(Error::InvalidPayment(verify_result.message));
    }
    
    // Settle payment
    let settle_result = facilitator.settle(payment.unwrap()).await?;
    if !settle_result.settled {
        return Err(Error::SettlementFailed(settle_result.message));
    }
    
    // Process the actual request
    let balance = get_balance_from_blockchain(address).await?;
    
    Ok(Response {
        result: json!({ "balance": balance }),
        settlement: Some(settle_result),
    })
}
```

### 4.2 Set Pricing Strategy

**Fixed Pricing per Tool:**

```rust
fn get_tool_price(tool_name: &str) -> &str {
    match tool_name {
        "getBalance" => "1000000",        // 1 USDC
        "getTransaction" => "2000000",    // 2 USDC
        "getTokenAccounts" => "5000000",  // 5 USDC
        _ => "1000000"                     // Default: 1 USDC
    }
}
```

**Dynamic Pricing:**

```rust
fn calculate_price(tool_name: &str, params: &Value) -> String {
    // Base price
    let base = 1_000_000; // 1 USDC
    
    // Add complexity multiplier
    let multiplier = match tool_name {
        "getMultipleAccounts" => {
            let count = params["addresses"].as_array().unwrap().len();
            count as u64
        },
        _ => 1
    };
    
    (base * multiplier).to_string()
}
```

---

## Step 5: Implement Client-Side Integration

### 5.1 JavaScript/TypeScript Client

Create a client that handles payment flows:

```typescript
import { Connection, Keypair, Transaction, PublicKey } from '@solana/web3.js';
import {
  createTransferCheckedInstruction,
  getAssociatedTokenAddress,
} from '@solana/spl-token';

class MCPClient {
  private wallet: Keypair;
  private connection: Connection;
  private mcpEndpoint: string;

  constructor(walletKeypair: Keypair, rpcUrl: string, mcpEndpoint: string) {
    this.wallet = walletKeypair;
    this.connection = new Connection(rpcUrl, 'confirmed');
    this.mcpEndpoint = mcpEndpoint;
  }

  async callTool(toolName: string, params: any): Promise<any> {
    // Step 1: Try calling without payment
    let response = await this.makeRequest(toolName, params, null);
    
    // Step 2: Check if payment is required
    if (response.error && response.error.code === -40200) {
      console.log('Payment required:', response.error.data);
      
      // Step 3: Create and sign payment transaction
      const paymentData = response.error.data;
      const payment = await this.createPayment(paymentData);
      
      // Step 4: Retry with payment
      response = await this.makeRequest(toolName, params, payment);
    }
    
    if (response.error) {
      throw new Error(`MCP Error: ${response.error.message}`);
    }
    
    return response.result;
  }

  private async makeRequest(
    toolName: string,
    params: any,
    payment: any | null
  ): Promise<any> {
    const request: any = {
      jsonrpc: '2.0',
      id: Date.now(),
      method: `tools/call`,
      params: {
        name: toolName,
        arguments: params,
      },
    };
    
    // Add payment metadata if provided
    if (payment) {
      request.params._meta = { payment };
    }
    
    const response = await fetch(this.mcpEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    
    return response.json();
  }

  private async createPayment(paymentRequired: any): Promise<any> {
    // Extract payment requirements
    const { accepts } = paymentRequired;
    const requirement = accepts[0]; // Use first accepted payment method
    
    const {
      network,
      amount,
      asset: mintAddress,
      payTo,
      scheme,
    } = requirement;
    
    // Get token decimals (assuming USDC = 6)
    const decimals = 6;
    
    // Create payment transaction
    const transaction = new Transaction();
    
    // Add compute budget
    transaction.add(
      ComputeBudgetProgram.setComputeUnitLimit({
        units: 200_000,
      })
    );
    transaction.add(
      ComputeBudgetProgram.setComputeUnitPrice({
        microLamports: 10_000,
      })
    );
    
    // Get source ATA
    const mint = new PublicKey(mintAddress);
    const sourceATA = await getAssociatedTokenAddress(
      mint,
      this.wallet.publicKey
    );
    
    // Get destination ATA
    const destATA = await getAssociatedTokenAddress(
      mint,
      new PublicKey(payTo)
    );
    
    // Add transfer instruction
    transaction.add(
      createTransferCheckedInstruction(
        sourceATA,
        mint,
        destATA,
        this.wallet.publicKey,
        BigInt(amount),
        decimals
      )
    );
    
    // Get recent blockhash
    const { blockhash } = await this.connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = this.wallet.publicKey;
    
    // Sign transaction
    transaction.sign(this.wallet);
    
    // Serialize to base64
    const serialized = transaction.serialize().toString('base64');
    
    // Return payment payload
    return {
      x402Version: 2,
      accepted: requirement,
      payload: serialized,
    };
  }
}

// Usage example
async function main() {
  // Load wallet
  const wallet = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(process.env.WALLET_SECRET_KEY!))
  );
  
  // Create client
  const client = new MCPClient(
    wallet,
    'https://api.devnet.solana.com',
    'http://localhost:3000/api/mcp'
  );
  
  // Call a paid tool
  try {
    const balance = await client.callTool('getBalance', {
      address: 'DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK',
    });
    console.log('Balance:', balance);
  } catch (error) {
    console.error('Error:', error);
  }
}
```

### 5.2 Python Client

```python
import base64
import json
from solders.keypair import Keypair
from solders.transaction import Transaction
from solana.rpc.api import Client
import requests

class MCPClient:
    def __init__(self, wallet_path: str, rpc_url: str, mcp_endpoint: str):
        with open(wallet_path, 'r') as f:
            secret = json.load(f)
        self.wallet = Keypair.from_bytes(bytes(secret))
        self.connection = Client(rpc_url)
        self.mcp_endpoint = mcp_endpoint
    
    def call_tool(self, tool_name: str, params: dict) -> dict:
        # Try without payment
        response = self._make_request(tool_name, params, None)
        
        # Check if payment required
        if 'error' in response and response['error']['code'] == -40200:
            print('Payment required:', response['error']['data'])
            
            # Create payment
            payment_data = response['error']['data']
            payment = self._create_payment(payment_data)
            
            # Retry with payment
            response = self._make_request(tool_name, params, payment)
        
        if 'error' in response:
            raise Exception(f"MCP Error: {response['error']['message']}")
        
        return response['result']
    
    def _make_request(self, tool_name: str, params: dict, payment: dict) -> dict:
        request = {
            'jsonrpc': '2.0',
            'id': 1,
            'method': 'tools/call',
            'params': {
                'name': tool_name,
                'arguments': params,
            }
        }
        
        if payment:
            request['params']['_meta'] = {'payment': payment}
        
        response = requests.post(
            self.mcp_endpoint,
            json=request,
            headers={'Content-Type': 'application/json'}
        )
        
        return response.json()
    
    def _create_payment(self, payment_required: dict) -> dict:
        # Implement payment transaction creation
        # Similar to JavaScript example above
        pass

# Usage
client = MCPClient(
    wallet_path='~/.config/solana/id.json',
    rpc_url='https://api.devnet.solana.com',
    mcp_endpoint='http://localhost:3000/api/mcp'
)

balance = client.call_tool('getBalance', {
    'address': 'DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK'
})
print('Balance:', balance)
```

---

## Step 6: Test Your Integration

### 6.1 Unit Testing

Test payment flow with mock facilitator:

```bash
# Start mock facilitator
node mock-facilitator.js &

# Run server with test config
cargo run --features x402 -- --config test-config.json

# Run integration tests
cargo test --features x402 x402_integration
```

### 6.2 Manual Testing

**Test 1: Payment Required Response**

```bash
curl -X POST http://localhost:3000/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "getBalance",
      "arguments": {"address": "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK"}
    }
  }'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -40200,
    "message": "Payment Required",
    "data": {
      "x402Version": 2,
      "resource": {"url": "mcp://tool/getBalance"},
      "accepts": [...]
    }
  }
}
```

**Test 2: Successful Payment**

```bash
# Create a signed transaction (use your client code)
# Then submit with payment:

curl -X POST http://localhost:3000/api/mcp \
  -H "Content-Type": application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "getBalance",
      "arguments": {"address": "..."},
      "_meta": {
        "payment": {
          "x402Version": 2,
          "accepted": {...},
          "payload": "BASE64_SIGNED_TRANSACTION"
        }
      }
    }
  }'
```

### 6.3 Monitor Logs

Enable debug logging:

```bash
RUST_LOG=debug ./target/release/solana-mcp-server --config config.json
```

Look for:
- `[x402] Payment required for tool: getBalance`
- `[x402] Verifying payment with facilitator`
- `[x402] Payment verified successfully`
- `[x402] Settling payment with facilitator`
- `[x402] Payment settled: tx_id=...`

---

## Step 7: Deploy to Production

### 7.1 Production Checklist

- [ ] Use **mainnet** configuration with real USDC/USDT
- [ ] Set **strong compute unit price bounds** (prevent abuse)
- [ ] Use **HTTPS** for facilitator (required)
- [ ] Enable **structured logging** with trace IDs
- [ ] Configure **monitoring** and alerts
- [ ] Test **payment flows** thoroughly
- [ ] Set up **rate limiting** (prevent DoS)
- [ ] Document **pricing** for users
- [ ] Implement **refund policy**
- [ ] Add **terms of service**

### 7.2 Production Configuration

```json
{
  "rpc_url": "https://api.mainnet-beta.solana.com",
  "commitment": "confirmed",
  "x402": {
    "enabled": true,
    "facilitator_base_url": "https://facilitator.yourcompany.com",
    "timeout_seconds": 30,
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
        "pay_to": "YOUR_MAINNET_WALLET",
        "min_compute_unit_price": 5000,
        "max_compute_unit_price": 50000
      }
    }
  }
}
```

### 7.3 Deployment Methods

**Docker:**

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features x402

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/solana-mcp-server /usr/local/bin/
COPY config.json /etc/solana-mcp/config.json
CMD ["solana-mcp-server", "--config", "/etc/solana-mcp/config.json"]
```

```bash
docker build -t solana-mcp-server:x402 .
docker run -p 3000:3000 solana-mcp-server:x402
```

**Systemd Service:**

```ini
# /etc/systemd/system/solana-mcp.service
[Unit]
Description=Solana MCP Server with x402
After=network.target

[Service]
Type=simple
User=solana-mcp
ExecStart=/usr/local/bin/solana-mcp-server --config /etc/solana-mcp/config.json
Restart=on-failure
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable solana-mcp
sudo systemctl start solana-mcp
sudo systemctl status solana-mcp
```

---

## Troubleshooting

### Common Issues

#### 1. "Payment Required but no x402 config found"

**Cause:** x402 feature not enabled or config missing

**Solution:**
```bash
# Rebuild with x402 feature
cargo build --release --features x402

# Verify config has x402 section
cat config.json | jq .x402
```

#### 2. "Facilitator verification failed: connection timeout"

**Cause:** Facilitator service unreachable

**Solution:**
```bash
# Test facilitator connectivity
curl https://facilitator.example.com/supported

# Check firewall rules
# Increase timeout in config
```

#### 3. "Invalid payment: compute unit price out of bounds"

**Cause:** Transaction gas price too high/low

**Solution:**
```typescript
// Set appropriate compute unit price
transaction.add(
  ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 10_000, // Must be between min/max in config
  })
);
```

#### 4. "ATA validation failed: destination mismatch"

**Cause:** Incorrect destination ATA derivation

**Solution:**
```typescript
// Ensure you derive ATA correctly
const destATA = await getAssociatedTokenAddress(
  mint,
  new PublicKey(payTo), // Use payTo from payment requirements
  false // allowOwnerOffCurve = false
);
```

#### 5. "Settlement failed: insufficient funds"

**Cause:** Wallet doesn't have enough tokens

**Solution:**
```bash
# Check token balance
spl-token accounts

# Get devnet tokens
solana airdrop 2
spl-token create-account EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
# Get testnet USDC from faucet
```

### Debug Mode

Enable verbose logging:

```bash
RUST_LOG=solana_mcp_server=trace,x402=trace \
  ./target/release/solana-mcp-server --config config.json
```

### Validate Configuration

```bash
# Test config loading
cargo run --features x402 -- --config config.json --validate-only

# Check facilitator /supported endpoint
curl https://facilitator.example.com/supported | jq
```

---

## Next Steps

- **Read the full documentation:** `docs/x402-integration.md`
- **Review use cases:** Check 13 detailed scenarios in the docs
- **Join community:** Get help on Discord/GitHub
- **Monitor payments:** Set up analytics and dashboards
- **Iterate:** Start with one paid tool, expand gradually

## Additional Resources

- [x402 v2 Specification](https://github.com/coinbase/x402/blob/main/specs/x402-specification-v2.md)
- [Solana Web3.js Docs](https://solana-labs.github.io/solana-web3.js/)
- [SPL Token Program](https://spl.solana.com/token)
- [CAIP-2 Standard](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md)

---

**Questions?** Open an issue on GitHub or reach out to the community!
