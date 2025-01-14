# Solana MCP Server

A Model Context Protocol (MCP) server that provides comprehensive access to Solana blockchain data through Cline. This server implements a wide range of Solana RPC methods, making it easy to query blockchain information directly through natural language conversations.

## Install in Claude Desktop

```bash
TEMP_DIR=$(mktemp -d) && cd "$TEMP_DIR" && git clone https://github.com/opensvm/solana-mcp-server.git . && cargo build --release && CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/claude" && mkdir -p "$CONFIG_DIR" && echo "{\"mcpServers\":{\"solana\":{\"command\":\"$PWD/target/release/solana-mcp-server\",\"env\":{\"SOLANA_RPC_URL\":\"https://api.mainnet-beta.solana.com\"}}}}" > "$CONFIG_DIR/config.json" || { rm -rf "$TEMP_DIR"; exit 1; }
```

## Available RPC Methods

### Account Methods
- `getAccountInfo`: Returns all information associated with the account of provided Pubkey
  - Input: `pubkey` (string) - Pubkey of account to query, as base-58 encoded string
  - Returns: Account data, balance, owner, and other metadata

- `getMultipleAccounts`: Returns account information for a list of Pubkeys
  - Input: `pubkeys` (array of strings) - List of Pubkeys to query
  - Returns: Array of account information

- `getProgramAccounts`: Returns all accounts owned by the provided program Pubkey
  - Input: `programId` (string) - Program Pubkey to query
  - Returns: Array of owned accounts with their data

- `getBalance`: Returns the SOL balance of an account
  - Input: `pubkey` (string) - Account Pubkey to query
  - Returns: Balance in lamports (1 SOL = 1,000,000,000 lamports)

- `getLargestAccounts`: Returns the 20 largest accounts by lamport balance
  - Input: None
  - Returns: Array of accounts with their balances

### Block Methods
- `getBlock`: Returns identity and transaction information about a confirmed block
  - Input: `slot` (integer) - Slot number to query
  - Returns: Block data including hash, parent, and transactions

- `getBlocks`: Returns a list of confirmed blocks between two slots
  - Input: `start_slot` (integer), `end_slot` (integer)
  - Returns: Array of block slots

- `getBlocksWithLimit`: Returns a list of confirmed blocks starting at given slot
  - Input: `start_slot` (integer), `limit` (integer)
  - Returns: Array of block slots up to limit

- `getBlockTime`: Returns estimated production time of a block
  - Input: `slot` (integer)
  - Returns: Unix timestamp (seconds since epoch)

- `getBlockHeight`: Returns current block height
  - Input: None
  - Returns: Current block height

- `getBlockCommitment`: Returns commitment for particular block
  - Input: `slot` (integer)
  - Returns: Block commitment information

- `getBlockProduction`: Returns recent block production information
  - Input: None
  - Returns: Block production stats

### System Methods
- `getHealth`: Returns current health status of the node
  - Input: None
  - Returns: "ok" if healthy, error otherwise

- `getVersion`: Returns current Solana version running on the node
  - Input: None
  - Returns: Version info including feature set

- `getIdentity`: Returns identity pubkey for the current node
  - Input: None
  - Returns: Node identity pubkey

- `getClusterNodes`: Returns information about all cluster nodes
  - Input: None
  - Returns: Array of node information

- `getVoteAccounts`: Returns account info and stake for all voting accounts
  - Input: None
  - Returns: Current and delinquent vote accounts

### Epoch and Inflation Methods
- `getEpochInfo`: Returns information about the current epoch
  - Input: None
  - Returns: Epoch info including slot and block height

- `getEpochSchedule`: Returns epoch schedule information
  - Input: None
  - Returns: Epoch schedule parameters

- `getInflationGovernor`: Returns current inflation governor
  - Input: None
  - Returns: Inflation parameters

- `getInflationRate`: Returns specific inflation values for current epoch
  - Input: None
  - Returns: Inflation rates

- `getInflationReward`: Returns inflation reward for list of addresses
  - Input: `addresses` (array of strings), optional `epoch` (integer)
  - Returns: Array of inflation rewards

### Token Methods
- `getTokenAccountBalance`: Returns token balance of an SPL Token account
  - Input: `accountAddress` (string) - Token account to query
  - Returns: Token amount with decimals

- `getTokenAccountsByDelegate`: Returns all token accounts by approved delegate
  - Input: `delegateAddress` (string)
  - Returns: Array of token accounts

- `getTokenAccountsByOwner`: Returns all token accounts by token owner
  - Input: `ownerAddress` (string)
  - Returns: Array of token accounts

- `getTokenLargestAccounts`: Returns 20 largest accounts of a token type
  - Input: `mint` (string) - Token mint to query
  - Returns: Array of largest token accounts

- `getTokenSupply`: Returns total supply of an SPL Token type
  - Input: `mint` (string) - Token mint to query
  - Returns: Total supply with decimals

### Transaction Methods
- `getTransaction`: Returns transaction details for confirmed transaction
  - Input: `signature` (string) - Transaction signature
  - Returns: Transaction info and status

- `getSignaturesForAddress`: Returns signatures for address's transactions
  - Input: `address` (string), optional `limit` (integer)
  - Returns: Array of transaction signatures

- `getSignatureStatuses`: Returns statuses of a list of signatures
  - Input: `signatures` (array of strings)
  - Returns: Array of transaction statuses

- `getTransactionCount`: Returns current Transaction count from ledger
  - Input: None
  - Returns: Transaction count

- `simulateTransaction`: Simulate sending a transaction
  - Input: `transaction` (string) - Encoded transaction
  - Returns: Simulation results

- `sendTransaction`: Send a transaction
  - Input: `transaction` (string) - Signed encoded transaction
  - Returns: Transaction signature

### Other Methods
- `getFeeForMessage`: Get the fee for a message
  - Input: `message` (string) - Encoded message
  - Returns: Fee in lamports

- `getLatestBlockhash`: Returns the latest blockhash
  - Input: None
  - Returns: Blockhash and last valid slot

- `isBlockhashValid`: Returns whether a blockhash is still valid
  - Input: `blockhash` (string)
  - Returns: Validity status

- `getStakeMinimumDelegation`: Returns stake minimum delegation
  - Input: None
  - Returns: Minimum stake delegation in lamports

- `getSupply`: Returns information about current supply
  - Input: None
  - Returns: Supply info including total and circulating

- `requestAirdrop`: Request an airdrop of lamports to a Pubkey
  - Input: `pubkey` (string), `lamports` (integer)
  - Returns: Transaction signature

## Usage Examples

Once configured, you can interact with the Solana blockchain through natural language in Cline. Here are some example queries:

- "What's the SOL balance of address Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr?"
- "Show me the current slot number"
- "Get information about the latest block"
- "What's the current inflation rate?"
- "Show me the token accounts owned by address ..."

## Environment Variables

- `SOLANA_RPC_URL`: (Optional) The Solana RPC endpoint to use. Defaults to "https://api.mainnet-beta.solana.com" if not specified.

## Development

### Prerequisites
- Rust and Cargo
- Solana CLI tools (optional, for testing)

### Building
```bash
cargo build
```

### Running
```bash
cargo run
```

## License

MIT License
