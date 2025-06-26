#!/bin/bash
# One-liner deployment script for Docker Compose
set -e

echo "🐳 Deploying Solana MCP Server with Docker Compose..."

# Create docker-compose.yml and deploy with load balancer
cat > docker-compose.yml << 'EOF'
version: '3.8'
services:
  solana-mcp:
    build: .
    ports:
      - "8080:8080"
    environment:
      - SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
      - SOLANA_COMMITMENT=confirmed
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
EOF
docker-compose up -d && echo "✅ Docker Compose deployment complete! Server running on http://localhost:8080 - Check status: docker-compose logs -f"