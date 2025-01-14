# Solana MCP Server

A Model Context Protocol (MCP) server that provides comprehensive access to Solana blockchain data through Cline. This server implements a wide range of Solana RPC methods, making it easy to query blockchain information directly through natural language conversations.

## Install in Claude Desktop

```bash
TEMP_DIR=$(mktemp -d) && cd "$TEMP_DIR" && git clone https://github.com/opensvm/solana-mcp-server.git . && cargo build --release && CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/claude" && mkdir -p "$CONFIG_DIR" && echo "{\"mcpServers\":{\"solana\":{\"command\":\"$PWD/target/release/solana-mcp-server\",\"env\":{\"SOLANA_RPC_URL\":\"https://api.mainnet-beta.solana.com\"}}}}" > "$CONFIG_DIR/config.json" || { rm -rf "$TEMP_DIR"; exit 1; }
```

## Features

The server provides essential Solana RPC methods across different categories:

### Slot Information
- `get_slot`: Get current slot number
- `get_slot_leaders`: Get slot leaders for a specified range

### Block Information
- `get_block`: Get block information for a specific slot
- `get_block_height`: Get current block height

### Account Information
- `get_balance`: Get SOL balance for an address
- `get_account_info`: Get detailed account information

### Transaction Information
- `get_transaction`: Get transaction details by signature

### System Information
- `get_health`: Get node health status
- `get_version`: Get node version information
- `get_identity`: Get node identity

### Epoch and Inflation
- `get_epoch_info`: Get current epoch information
- `get_inflation_rate`: Get current inflation rate

### Token Information
- `get_token_accounts_by_owner`: Get token accounts owned by an address

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
