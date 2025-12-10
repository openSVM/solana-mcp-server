# Solana MCP Server

A Model Context Protocol (MCP) server that provides comprehensive access to Solana blockchain data through Cline. This server implements a wide range of Solana RPC methods, making it easy to query blockchain information directly through natural language conversations.

## 🚀 Usage Modes

The Solana MCP Server supports two modes of operation:

### 📡 Stdio Mode (Default)
For integration with Claude Desktop and other MCP clients:
```bash
solana-mcp-server stdio  # or just: solana-mcp-server
```

### 🌐 Web Service Mode  
For HTTP API access and integration with web applications:
```bash
# Run on default port 3000
solana-mcp-server web

# Run on custom port
solana-mcp-server web --port 8080
```

**Web Service Endpoints:**
- `POST /api/mcp` - MCP JSON-RPC API
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics

📖 **[Complete Web Service Documentation](./docs/web-service.md)**

## Quick Installation (One-liner)

🚀 **Install Solana MCP Server for Claude Desktop in one command:**

```bash
curl -fsSL https://raw.githubusercontent.com/opensvm/solana-mcp-server/main/scripts/install.sh | bash
```

This will:
- Download pre-built binaries (if available) or build from source
- Configure Claude Desktop automatically  
- Set up proper environment variables
- Back up existing configurations

**Alternative install methods:**

```bash
# Using wget
wget -qO- https://raw.githubusercontent.com/opensvm/solana-mcp-server/main/scripts/install.sh | bash

# Manual download and run
curl -fsSL https://raw.githubusercontent.com/opensvm/solana-mcp-server/main/scripts/install.sh -o install.sh
chmod +x install.sh
./install.sh
```

After installation, restart Claude Desktop and start querying Solana data directly!

## Manual Installation (Advanced)

### Using Pre-built Binaries

1. Go to the [Releases](https://github.com/opensvm/solana-mcp-server/releases) page
2. Download the appropriate binary for your system:
   - Linux: `solana-mcp-server-linux-amd64`
   - macOS Intel: `solana-mcp-server-macos-amd64`
   - macOS Apple Silicon: `solana-mcp-server-macos-arm64`
   - Windows: `solana-mcp-server-windows-amd64.exe`
3. Make the binary executable (Linux/macOS):
   ```bash
   chmod +x solana-mcp-server-*
   ```
4. Configure Claude Desktop:
   ```bash
   CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/claude"
   mkdir -p "$CONFIG_DIR"
   echo "{\"mcpServers\":{\"solana\":{\"command\":\"$PWD/solana-mcp-server-*\",\"env\":{\"SOLANA_RPC_URL\":\"https://api.mainnet-beta.solana.com\"}}}}" > "$CONFIG_DIR/config.json"
   ```

### Building from Source

```bash
git clone https://github.com/opensvm/solana-mcp-server.git
cd solana-mcp-server
cargo build --release
```

Then configure Claude Desktop with the path to `target/release/solana-mcp-server`.

## Quick Deployment

🚀 **One-liner deployment scripts for all platforms:**

```bash
# Local development
./scripts/deploy-local.sh

# Docker container
./scripts/deploy-docker.sh

# Kubernetes with autoscaling
./scripts/deploy-k8s.sh

# AWS Lambda
./scripts/deploy-lambda.sh

# Google Cloud Functions  
./scripts/deploy-gcf.sh

# Vercel Edge Functions
./scripts/deploy-vercel.sh
```

See [`scripts/README.md`](scripts/README.md) for detailed usage and requirements for each deployment option.

## ⚡ Autoscaling and Monitoring

The Solana MCP Server supports dynamic scaling to handle variable load efficiently:

### Features
- **Prometheus metrics** exposed at `/metrics` endpoint
- **Kubernetes HPA** with CPU, memory, and custom metrics
- **Docker scaling** guidelines and automation scripts
- **Health checks** at `/health` endpoint
- **MCP JSON-RPC API** for web service integration
- **Automated RPC caching** with configurable TTL for improved performance

### RPC Caching

The server includes an intelligent caching layer for RPC responses to reduce latency and improve performance:

- **TTL-based caching**: Configurable time-to-live per method
- **Method-specific TTLs**: Different cache durations for different data types
- **Prometheus metrics**: Track cache hit/miss rates
- **Thread-safe**: Concurrent access using DashMap
- **Size limits**: Automatic eviction when capacity is reached

See **[Caching Documentation](./docs/caching.md)** for configuration and usage details.

### Web Service API

The server now supports both traditional stdio transport and HTTP JSON-RPC mode:

```bash
# Run as stdio transport (default)
solana-mcp-server stdio

# Run as web service
solana-mcp-server web --port 3000
```

**API Endpoints:**
- `POST /api/mcp` - Full MCP JSON-RPC 2.0 API
- `GET /health` - Health check with capability information  
- `GET /metrics` - Prometheus metrics

**[📚 Complete MCP JSON-RPC API Documentation](./docs/mcp-json-rpc-api.md)**

### Metrics Exposed
- `solana_mcp_rpc_requests_total` - Total RPC requests by method and network
- `solana_mcp_rpc_request_duration_seconds` - Request latency histogram
- `solana_mcp_rpc_requests_failed_total` - Failed requests by error type
- `solana_mcp_cache_hits_total` - Cache hits by method
- `solana_mcp_cache_misses_total` - Cache misses by method
- `solana_mcp_cache_size` - Current cache size
- Standard resource metrics (CPU, memory)

### Quick Start with Autoscaling

```bash
# Deploy with Kubernetes autoscaling
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/hpa.yaml

# Check autoscaling status
kubectl get hpa solana-mcp-server-hpa --watch

# Access metrics
kubectl port-forward svc/solana-mcp-service 8080:8080
curl http://localhost:8080/metrics
```

📊 **[Complete Autoscaling Documentation](./docs/metrics.md)** | 🐳 **[Docker Scaling Guide](./docs/docker-scaling.md)**

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
  - Input: Optional `filter` (string) - Filter by account type (circulating|nonCirculating)
  - Returns: Array of accounts with their balances

- `getMinimumBalanceForRentExemption`: Returns minimum balance for rent exemption
  - Input: `dataSize` (integer) - Size of account data in bytes
  - Returns: Minimum lamports required for rent exemption

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
  - Input: Optional `identity` (string) - Validator identity, `range` (object)
  - Returns: Block production stats

- `getSlot`: Returns the current slot the node is processing
  - Input: Optional `commitment` (string) - Commitment level
  - Returns: Current slot

- `getSlotLeaders`: Returns slot leaders for a given slot range
  - Input: `startSlot` (integer), `limit` (integer)
  - Returns: Array of validator identity pubkeys

- `getFirstAvailableBlock`: Returns the lowest confirmed block still available
  - Input: None
  - Returns: First available block slot

- `getGenesisHash`: Returns the genesis hash of the ledger
  - Input: None
  - Returns: Genesis hash as string

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

- `getLeaderSchedule`: Returns the leader schedule for an epoch
  - Input: Optional `slot` (integer), `identity` (string)
  - Returns: Leader schedule by validator identity

- `getVoteAccounts`: Returns account info and stake for all voting accounts
  - Input: Optional `votePubkey` (string), configuration parameters
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

### Basic Queries
- "What's the SOL balance of address Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr?"
- "Show me the current slot number"
- "Get information about the latest block"
- "What's the current inflation rate?"
- "Show me the token accounts owned by address ..."

### Multi-Network Queries
- "List all available SVM networks"
- "Enable Eclipse mainnet for queries"
- "Check SOL balance on all enabled networks"
- "Compare transaction counts across networks"

### Advanced Operations
- "Show me the largest USDC token accounts"
- "Get the leader schedule for the current epoch"
- "Find all accounts owned by the SPL Token program"
- "Check the block production stats for a validator"

## Security

This project undergoes regular security audits using `cargo audit`. Our CI/CD pipeline automatically scans for vulnerabilities and generates reports.

### Current Security Status
- ✅ **Active monitoring**: Weekly automated security scans
- ✅ **Dependency updates**: Regular updates to latest secure versions
- ⚠️ **Known acceptable risks**: Some vulnerabilities exist in deep Solana ecosystem dependencies
- 📋 **Full audit reports**: Available as CI artifacts and in `docs/security-audit.md`

For detailed security information, vulnerability assessments, and risk analysis, see:

📋 **[Security Audit Documentation](./docs/security-audit.md)**

## Documentation

For comprehensive documentation including architecture, deployment guides, and complete API reference, see:

📚 **[Complete Documentation](./docs/README.md)**

🚀 **[Developer Onboarding Guide](./docs/onboarding.md)** - **Start here if you're new to the project!**

- [🏗️ Architecture Overview](./docs/architecture.md) - Server internals and design
- [🚀 Deployment Guide](./docs/deployment.md) - Local, serverless, and endpoint deployment
- [📖 API Reference](./docs/api-reference.md) - Complete method documentation
- [⚙️ Configuration Guide](./docs/configuration.md) - Configuration options and management
- [💾 Caching Guide](./docs/caching.md) - RPC response caching configuration and usage

## Environment Variables

- `SOLANA_RPC_URL`: (Optional) The Solana RPC endpoint to use. Defaults to "https://api.mainnet-beta.solana.com" if not specified.
- `SOLANA_COMMITMENT`: (Optional) Commitment level (processed|confirmed|finalized). Defaults to "confirmed".
- `SOLANA_PROTOCOL_VERSION`: (Optional) MCP protocol version. Defaults to latest.

For cache configuration, see `config.json` or the **[Caching Documentation](./docs/caching.md)**.

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
