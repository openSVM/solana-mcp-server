#!/bin/bash
# One-liner deployment script for local development
set -e

echo "🚀 Deploying Solana MCP Server locally..."

# Download latest release, configure Claude Desktop, and start server
curl -s https://api.github.com/repos/opensvm/solana-mcp-server/releases/latest | grep browser_download_url | grep $(uname -s | tr '[:upper:]' '[:lower:]') | cut -d '"' -f 4 | xargs curl -L -o solana-mcp-server && chmod +x solana-mcp-server && mkdir -p ~/.config/claude && echo '{"mcpServers":{"solana":{"command":"'$(pwd)'/solana-mcp-server","env":{"SOLANA_RPC_URL":"https://api.mainnet-beta.solana.com","SOLANA_COMMITMENT":"confirmed"}}}}' > ~/.config/claude/config.json && echo "✅ Local deployment complete! Server configured for Claude Desktop at $(pwd)/solana-mcp-server"