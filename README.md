# Solana MCP Server

A Model Context Protocol (MCP) server that provides comprehensive access to Solana blockchain data through Cline. This server implements a wide range of Solana RPC methods, making it easy to query blockchain information directly through natural language conversations.

## Features

The server provides 21 essential Solana RPC methods across different categories:

### Account & Balance Operations
- `get_sol_balance`: Get SOL balance for an address
- `get_token_balance`: Get SPL token balance
- `get_account_info`: Get account information
- `get_largest_accounts`: Get largest accounts on network

### Block & Transaction Information
- `get_slot`: Get current slot
- `get_block`: Get block information
- `get_block_time`: Get block production time
- `get_transaction`: Get transaction details
- `get_recent_blockhash`: Get recent blockhash

### Token Operations
- `get_token_accounts_by_owner`: Get token accounts by owner
- `get_token_accounts_by_delegate`: Get delegated token accounts
- `get_token_supply`: Get token supply information

### System Information
- `get_epoch_info`: Get current epoch information
- `get_version`: Get node version
- `get_health`: Get node health status
- `get_supply`: Get current supply
- `get_inflation_rate`: Get inflation rate
- `get_cluster_nodes`: Get cluster node information
- `get_minimum_balance_for_rent_exemption`: Get minimum rent-exempt balance

### Staking & Governance
- `get_vote_accounts`: Get vote accounts
- `get_leader_schedule`: Get leader schedule

## Quick Setup

One-line setup command for macOS:
```bash
mkdir -p ~/Library/Application\ Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/ && echo '{"mcpServers":{"solana":{"command":"cargo","args":["run"],"cwd":"'$(pwd)'","env":{"SOLANA_RPC_URL":"https://api.mainnet-beta.solana.com"}}}}' > ~/Library/Application\ Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json
```

For Linux:
```bash
mkdir -p ~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/ && echo '{"mcpServers":{"solana":{"command":"cargo","args":["run"],"cwd":"'$(pwd)'","env":{"SOLANA_RPC_URL":"https://api.mainnet-beta.solana.com"}}}}' > ~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json
```

For Windows (PowerShell):
```powershell
$settings = @{mcpServers=@{solana=@{command="cargo";args=@("run");cwd=$PWD.Path;env=@{SOLANA_RPC_URL="https://api.mainnet-beta.solana.com"}}}} | ConvertTo-Json -Depth 10; New-Item -ItemType Directory -Force "$env:APPDATA\Code\User\globalStorage\saoudrizwan.claude-dev\settings"; $settings | Out-File "$env:APPDATA\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json"
```

## Setup in Cline

1. Add the following configuration to your Cline MCP settings file (`~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` on macOS):

```json
{
  "mcpServers": {
    "solana": {
      "command": "cargo",
      "args": ["run"],
      "cwd": "/path/to/solana-mcp-server",
      "env": {
        "SOLANA_RPC_URL": "https://api.mainnet-beta.solana.com"  // Or your preferred RPC endpoint
      }
    }
  }
}
```

2. Restart Cline to load the new MCP server.

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
