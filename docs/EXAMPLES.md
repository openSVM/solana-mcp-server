# Examples and Use Cases

## Overview

This document provides practical examples of using the Solana MCP Server in various scenarios, from basic queries to advanced multi-network operations.

## Basic Account Operations

### Check Account Balance

**Natural Language:**
```
User: "What's the SOL balance of address Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr?"
```

**Direct API Call:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "getBalance",
    "arguments": {
      "pubkey": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "The account has a balance of 2.5 SOL (2,500,000,000 lamports)"
    }
  ]
}
```

### Get Account Information

**Natural Language:**
```
User: "Show me detailed information about account 4fYNw3dojWmQ4dXtSGE9epjRGy9zgEyakSEKvaNdZ54"
```

**Response includes:**
- Account balance
- Owner program
- Data size
- Executable status
- Rent epoch

## Token Operations

### Check Token Balance

**Natural Language:**
```
User: "What's the USDC balance in token account 7UX2i7SucgLMQcfZ75s3VXmZZY4YRUyJN9X1Rtf7ckM?"
```

**Multi-step process:**
1. Server calls `getTokenAccountBalance`
2. Identifies token mint and decimals
3. Returns human-readable balance

### Find Token Accounts by Owner

**Natural Language:**
```
User: "Show me all USDC token accounts owned by 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
```

**Server performs:**
1. Calls `getTokenAccountsByOwner`
2. Filters for USDC mint
3. Returns formatted list with balances

## Block and Transaction Queries

### Current Network Status

**Natural Language:**
```
User: "What's the current slot number and latest block hash?"
```

**Server responds with:**
- Current slot from `getSlot`
- Latest blockhash from `getLatestBlockhash`
- Current epoch information

### Transaction History

**Natural Language:**
```
User: "Show me the last 5 transactions for address ABC123..."
```

**Process:**
1. `getSignaturesForAddress` with limit=5
2. For each signature, calls `getTransaction`
3. Formats transaction details

## Multi-Network Operations

### Network Discovery and Setup

**Step 1: List Available Networks**
```
User: "Show me all available SVM networks"
```

**Response shows:**
- Solana Mainnet/Devnet
- Eclipse networks
- Other SVM-compatible chains
- Custom network options

**Step 2: Enable Networks**
```
User: "Enable Eclipse mainnet for queries"
```

**Server performs:**
```json
{
  "method": "enableSvmNetwork",
  "params": {
    "network_id": "eclipse-mainnet"
  }
}
```

### Cross-Network Comparisons

**Balance Comparison:**
```
User: "Check my SOL balance on all enabled networks"
```

**With networks: solana-mainnet, eclipse-mainnet enabled:**
```json
{
  "solana-mainnet": {
    "context": {"slot": 123456},
    "value": 5000000000
  },
  "eclipse-mainnet": {
    "context": {"slot": 98765},
    "value": 2500000000
  }
}
```

**Formatted Response:**
```
Your SOL balance across networks:
• Solana Mainnet: 5.0 SOL
• Eclipse Mainnet: 2.5 SOL
Total: 7.5 SOL equivalent
```

### Network-Specific Queries

**Custom RPC Configuration:**
```
User: "Set a custom RPC URL for development testing"
```

**Server prompts for:**
- Network identifier
- RPC URL
- Enable/disable status

## Advanced Use Cases

### DeFi Portfolio Analysis

**Natural Language:**
```
User: "Analyze the DeFi portfolio for wallet 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
```

**Multi-step Analysis:**
1. Get all token accounts owned by wallet
2. For each token, get current supply and balance
3. Calculate portfolio distribution
4. Check for staking accounts
5. Analyze recent transaction patterns

### Validator Performance Monitoring

**Natural Language:**
```
User: "Show me block production stats for validator 7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2GRps"
```

**Server queries:**
1. `getBlockProduction` for validator
2. `getVoteAccounts` for stake information
3. `getLeaderSchedule` for upcoming slots

### Cross-Chain Token Tracking

**Example: USDC across Networks**
```
User: "Compare USDC supply and largest holders across all enabled networks"
```

**Process:**
1. Identify USDC mint addresses per network
2. Call `getTokenSupply` on each network
3. Call `getTokenLargestAccounts` on each
4. Aggregate and compare results

## Development Workflows

### Smart Contract Testing

**Devnet Account Setup:**
```
User: "Request an airdrop of 10 SOL to my test account on devnet"
```

**Server performs:**
```json
{
  "method": "requestAirdrop",
  "params": {
    "pubkey": "test-account-address",
    "lamports": 10000000000
  }
}
```

### Transaction Simulation

**Pre-flight Testing:**
```
User: "Simulate this transaction before sending it"
```

**Server uses:**
- `simulateTransaction` for dry-run
- Returns success/failure and logs
- Shows account changes and fees

## Integration Examples

### Claude Desktop Integration

**Setup in `~/.config/claude/config.json`:**
```json
{
  "mcpServers": {
    "solana": {
      "command": "/path/to/solana-mcp-server",
      "env": {
        "SOLANA_RPC_URL": "https://api.mainnet-beta.solana.com",
        "SOLANA_COMMITMENT": "confirmed"
      }
    }
  }
}
```

### Web Application Integration

**HTTP Endpoint Usage:**
```javascript
// Query account balance via HTTP
const response = await fetch('http://localhost:8080/rpc', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    method: 'tools/call',
    params: {
      name: 'getBalance',
      arguments: {
        pubkey: 'account-address'
      }
    }
  })
});

const result = await response.json();
```

### Serverless Function Integration

**AWS Lambda Handler:**
```javascript
export const handler = async (event) => {
  const query = event.body;
  
  // Process through MCP server
  const response = await mcpServer.handleRequest(query);
  
  return {
    statusCode: 200,
    body: JSON.stringify(response),
    headers: {
      'Content-Type': 'application/json',
    },
  };
};
```

## Error Handling Examples

### Network Connectivity Issues

**When RPC is down:**
```json
{
  "error": {
    "code": -32603,
    "message": "RPC endpoint unavailable",
    "data": {
      "network": "solana-mainnet",
      "url": "https://api.mainnet-beta.solana.com",
      "fallback_available": true
    }
  }
}
```

### Invalid Parameters

**Invalid address format:**
```json
{
  "error": {
    "code": -32602,
    "message": "Invalid pubkey format",
    "data": {
      "provided": "invalid-address",
      "expected": "base58-encoded string"
    }
  }
}
```

## Performance Examples

### Batch Operations

**Multiple Account Queries:**
```
User: "Check balances for these 10 accounts: [list of addresses]"
```

**Server optimizes with:**
- `getMultipleAccounts` for batch processing
- Parallel requests across networks
- Response aggregation

### Caching Scenarios

**Repeated Network Info Queries:**
- Network list cached for 1 hour
- Block height cached for 30 seconds
- Account info cached for configurable duration

## Monitoring and Debugging

### Health Check Examples

**Server Health:**
```bash
curl http://localhost:8080/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "enabled_networks": ["solana-mainnet", "eclipse-mainnet"],
  "rpc_status": {
    "solana-mainnet": "connected",
    "eclipse-mainnet": "connected"
  }
}
```

### Performance Metrics

**Request Timing:**
```
2024-01-15 10:30:00 [INFO] Request processed in 150ms
  - Network routing: 5ms
  - RPC calls: 120ms (parallel)
  - Response formatting: 25ms
```

This comprehensive examples guide demonstrates the full capabilities of the Solana MCP Server across various use cases and integration scenarios.