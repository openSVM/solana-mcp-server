# API Reference

## Overview

The Solana MCP Server exposes 47 comprehensive RPC methods across 6 major categories, plus 4 network management methods. All methods support both single-network and multi-network execution modes.

## Network Management Methods

### `listSvmNetworks`
Lists all available SVM-compatible networks from the awesome-svm registry.

**Parameters:** None

**Response:**
```json
{
  "networks": [
    {
      "id": "solana-mainnet",
      "name": "Solana Mainnet",
      "rpc_url": "https://api.mainnet-beta.solana.com",
      "chain_id": "solana",
      "type": "mainnet"
    }
  ]
}
```

**Example:**
```
User: "Show me all available SVM networks"
Response: Lists networks from https://raw.githubusercontent.com/openSVM/awesome-svm/refs/heads/main/svm-networks.json
```

### `enableSvmNetwork`
Enables a network for multi-network queries.

**Parameters:**
- `network_id` (string): Network identifier to enable

**Example:**
```
User: "Enable the Eclipse mainnet network"
```

### `disableSvmNetwork`
Disables a network from multi-network queries.

**Parameters:**
- `network_id` (string): Network identifier to disable

### `setNetworkRpcUrl`
Override the RPC URL for a specific network.

**Parameters:**
- `network_id` (string): Network identifier
- `rpc_url` (string): Custom RPC endpoint URL

**Example:**
```
User: "Set a custom RPC URL for Solana mainnet"
```

## Account Methods

### `getAccountInfo`
Returns all information associated with the account of provided Pubkey.

**Parameters:**
- `pubkey` (string): Pubkey of account to query, as base-58 encoded string
- `commitment` (string, optional): Commitment level (processed|confirmed|finalized)
- `encoding` (string, optional): Data encoding (base58|base64|jsonParsed)

**Single Network Response:**
```json
{
  "context": { "slot": 123456 },
  "value": {
    "data": ["base64-encoded-data", "base64"],
    "executable": false,
    "lamports": 1000000000,
    "owner": "11111111111111111111111111111112",
    "rentEpoch": 361
  }
}
```

**Multi-Network Response:**
```json
{
  "solana-mainnet": {
    "context": { "slot": 123456 },
    "value": { /* account info */ }
  },
  "eclipse-mainnet": {
    "context": { "slot": 123450 },
    "value": { /* account info */ }
  }
}
```

### `getBalance`
Returns the SOL balance of an account.

**Parameters:**
- `pubkey` (string): Account Pubkey to query

**Response:**
```json
{
  "context": { "slot": 123456 },
  "value": 1000000000
}
```

### `getMultipleAccounts`
Returns account information for multiple Pubkeys.

**Parameters:**
- `pubkeys` (array of strings): List of Pubkeys to query
- `commitment` (string, optional): Commitment level

### `getProgramAccounts`
Returns all accounts owned by the provided program Pubkey.

**Parameters:**
- `programId` (string): Program Pubkey to query
- `commitment` (string, optional): Commitment level
- `filters` (array, optional): Filter criteria
- `encoding` (string, optional): Data encoding

### `getLargestAccounts`
Returns the 20 largest accounts by lamport balance.

**Parameters:**
- `commitment` (string, optional): Commitment level
- `filter` (string, optional): Filter by account type (circulating|nonCirculating)

### `getMinimumBalanceForRentExemption`
Returns minimum balance for rent exemption.

**Parameters:**
- `dataSize` (integer): Size of account data in bytes

**Response:**
```json
{
  "value": 890880
}
```

## Block Methods

### `getBlock`
Returns identity and transaction information about a confirmed block.

**Parameters:**
- `slot` (integer): Slot number to query
- `commitment` (string, optional): Commitment level
- `encoding` (string, optional): Transaction encoding
- `transactionDetails` (string, optional): Level of transaction detail
- `rewards` (boolean, optional): Include rewards

### `getBlocks`
Returns a list of confirmed blocks between two slots.

**Parameters:**
- `start_slot` (integer): Start slot (inclusive)
- `end_slot` (integer, optional): End slot (inclusive)

### `getBlocksWithLimit`
Returns a list of confirmed blocks starting at given slot.

**Parameters:**
- `start_slot` (integer): Start slot
- `limit` (integer): Maximum number of blocks to return

### `getBlockTime`
Returns estimated production time of a block.

**Parameters:**
- `slot` (integer): Block slot to query

**Response:**
```json
{
  "value": 1627852800
}
```

### `getBlockHeight`
Returns current block height.

**Parameters:** None

**Response:**
```json
{
  "value": 123456789
}
```

### `getBlockCommitment`
Returns commitment for particular block.

**Parameters:**
- `slot` (integer): Block slot to query

### `getBlockProduction`
Returns recent block production information.

**Parameters:**
- `commitment` (string, optional): Commitment level
- `range` (object, optional): Slot range to query
- `identity` (string, optional): Validator identity pubkey

### `getSlot`
Returns the current slot the node is processing.

**Parameters:**
- `commitment` (string, optional): Commitment level

**Response:**
```json
{
  "value": 123456789
}
```

### `getSlotLeaders`
Returns slot leaders for a given slot range.

**Parameters:**
- `startSlot` (integer): Start slot
- `limit` (integer): Number of leaders to return

### `getFirstAvailableBlock`
Returns the lowest confirmed block still available.

**Parameters:** None

**Response:**
```json
{
  "value": 123000000
}
```

### `getGenesisHash`
Returns the genesis hash of the ledger.

**Parameters:** None

**Response:**
```json
{
  "value": "genesis-hash-string"
}
```

## System Methods

### `getHealth`
Returns current health status of the node.

**Parameters:** None

**Response:**
```json
"ok"
```

### `getVersion`
Returns current Solana version running on the node.

**Parameters:** None

**Response:**
```json
{
  "solana-core": "1.16.0",
  "feature-set": 123456789
}
```

### `getIdentity`
Returns identity pubkey for the current node.

**Parameters:** None

### `getClusterNodes`
Returns information about all cluster nodes.

**Parameters:** None

### `getLeaderSchedule`
Returns the leader schedule for an epoch.

**Parameters:**
- `slot` (integer, optional): Slot number to query
- `commitment` (string, optional): Commitment level
- `identity` (string, optional): Validator identity

### `getVoteAccounts`
Returns account info and stake for all voting accounts.

**Parameters:**
- `commitment` (string, optional): Commitment level
- `votePubkey` (string, optional): Specific vote account
- `keepUnstakedDelinquents` (boolean, optional): Include unstaked delinquents
- `delinquentSlotDistance` (integer, optional): Delinquent slot distance

## Epoch and Inflation Methods

### `getEpochInfo`
Returns information about the current epoch.

**Parameters:**
- `commitment` (string, optional): Commitment level

**Response:**
```json
{
  "absoluteSlot": 123456789,
  "blockHeight": 123456789,
  "epoch": 300,
  "slotIndex": 123456,
  "slotsInEpoch": 432000,
  "transactionCount": 987654321
}
```

### `getEpochSchedule`
Returns epoch schedule information.

**Parameters:** None

### `getInflationGovernor`
Returns current inflation governor.

**Parameters:**
- `commitment` (string, optional): Commitment level

### `getInflationRate`
Returns specific inflation values for current epoch.

**Parameters:** None

### `getInflationReward`
Returns inflation reward for list of addresses.

**Parameters:**
- `addresses` (array of strings): List of addresses to query
- `epoch` (integer, optional): Epoch to query
- `commitment` (string, optional): Commitment level

## Token Methods

### `getTokenAccountBalance`
Returns token balance of an SPL Token account.

**Parameters:**
- `accountAddress` (string): Token account to query
- `commitment` (string, optional): Commitment level

**Response:**
```json
{
  "context": { "slot": 123456 },
  "value": {
    "amount": "1000000000",
    "decimals": 9,
    "uiAmount": 1.0,
    "uiAmountString": "1"
  }
}
```

### `getTokenAccountsByDelegate`
Returns all token accounts by approved delegate.

**Parameters:**
- `delegateAddress` (string): Delegate address
- `mint` (string, optional): Specific token mint
- `programId` (string, optional): Token program ID
- `commitment` (string, optional): Commitment level

### `getTokenAccountsByOwner`
Returns all token accounts by token owner.

**Parameters:**
- `ownerAddress` (string): Owner address
- `mint` (string, optional): Specific token mint
- `programId` (string, optional): Token program ID

### `getTokenLargestAccounts`
Returns 20 largest accounts of a token type.

**Parameters:**
- `mint` (string): Token mint to query
- `commitment` (string, optional): Commitment level

### `getTokenSupply`
Returns total supply of an SPL Token type.

**Parameters:**
- `mint` (string): Token mint to query
- `commitment` (string, optional): Commitment level

## Transaction Methods

### `getTransaction`
Returns transaction details for confirmed transaction.

**Parameters:**
- `signature` (string): Transaction signature
- `commitment` (string, optional): Commitment level
- `encoding` (string, optional): Transaction encoding
- `maxSupportedTransactionVersion` (integer, optional): Max transaction version

### `getSignaturesForAddress`
Returns signatures for address's transactions.

**Parameters:**
- `address` (string): Account address
- `limit` (integer, optional): Maximum signatures to return
- `before` (string, optional): Start searching backwards from this signature
- `until` (string, optional): Search until this signature
- `commitment` (string, optional): Commitment level

### `getSignatureStatuses`
Returns statuses of a list of signatures.

**Parameters:**
- `signatures` (array of strings): Transaction signatures
- `searchTransactionHistory` (boolean, optional): Search full transaction history

### `getTransactionCount`
Returns current Transaction count from ledger.

**Parameters:**
- `commitment` (string, optional): Commitment level

**Response:**
```json
{
  "value": 987654321
}
```

### `simulateTransaction`
Simulate sending a transaction.

**Parameters:**
- `transaction` (string): Encoded transaction
- `commitment` (string, optional): Commitment level
- `encoding` (string, optional): Transaction encoding
- `replaceRecentBlockhash` (boolean, optional): Replace recent blockhash
- `accounts` (object, optional): Account configurations

### `sendTransaction`
Send a transaction to the network.

**Parameters:**
- `transaction` (string): Signed encoded transaction
- `encoding` (string, optional): Transaction encoding
- `skipPreflight` (boolean, optional): Skip preflight checks
- `preflightCommitment` (string, optional): Preflight commitment level
- `maxRetries` (integer, optional): Maximum retry attempts

## Other Methods

### `getFeeForMessage`
Get the fee for a message.

**Parameters:**
- `message` (string): Base64 encoded message
- `commitment` (string, optional): Commitment level

### `getLatestBlockhash`
Returns the latest blockhash.

**Parameters:**
- `commitment` (string, optional): Commitment level

**Response:**
```json
{
  "context": { "slot": 123456 },
  "value": {
    "blockhash": "blockhash-string",
    "lastValidBlockHeight": 123456
  }
}
```

### `isBlockhashValid`
Returns whether a blockhash is still valid.

**Parameters:**
- `blockhash` (string): Blockhash to validate
- `commitment` (string, optional): Commitment level

### `getStakeMinimumDelegation`
Returns stake minimum delegation.

**Parameters:**
- `commitment` (string, optional): Commitment level

### `getSupply`
Returns information about current supply.

**Parameters:**
- `commitment` (string, optional): Commitment level
- `excludeNonCirculatingAccountsList` (boolean, optional): Exclude non-circulating accounts

### `requestAirdrop`
Request an airdrop of lamports to a Pubkey (devnet/testnet only).

**Parameters:**
- `pubkey` (string): Recipient address
- `lamports` (integer): Amount to airdrop
- `commitment` (string, optional): Commitment level

## Error Handling

### Common Error Codes

- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `-32000`: Server error

### Error Response Format
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "protocolVersion": "2024-11-05"
    }
  }
}
```

## Rate Limiting

The server implements rate limiting to prevent abuse:

- **Default Limit**: 100 requests per minute per client
- **Burst Limit**: 10 requests per second
- **Multi-Network Impact**: Each enabled network counts as one request

## Usage Examples

### Natural Language Queries

```
User: "What's the SOL balance of address Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr?"
Assistant: [Calls getBalance with the provided address]

User: "Show me the current slot number on all enabled networks"
Assistant: [Calls getSlot on all enabled SVM networks]

User: "Get the token supply for USDC"
Assistant: [Calls getTokenSupply with USDC mint address]
```

### Direct API Calls

```bash
# Single network call
echo '{"method": "getBalance", "params": {"pubkey": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"}}' | \
  solana-mcp-server

# Multi-network setup
echo '{"method": "enableSvmNetwork", "params": {"network_id": "eclipse-mainnet"}}' | \
  solana-mcp-server
```