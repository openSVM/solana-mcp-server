#!/bin/bash
# One-liner deployment script for Docker
set -e

echo "🐳 Deploying Solana MCP Server with Docker..."

# Build and run Docker container with all necessary configuration
docker build -t solana-mcp-server . && docker run -d --name solana-mcp-server -p 8080:8080 -e SOLANA_RPC_URL=https://api.mainnet-beta.solana.com -e SOLANA_COMMITMENT=confirmed -e RUST_LOG=info --restart unless-stopped solana-mcp-server && echo "✅ Docker deployment complete! Server running on http://localhost:8080 - Check status: docker logs solana-mcp-server"