#!/bin/bash
# One-liner deployment script for Google Cloud Functions
set -e

echo "☁️ Deploying Solana MCP Server to Google Cloud Functions..."

# Build for Cloud Functions, create function.yaml, and deploy
cargo build --release --target x86_64-unknown-linux-gnu && cat > function.yaml << 'EOF'
specVersion: v1alpha1
kind: CloudFunction
metadata:
  name: solana-mcp-server
spec:
  runtime: provided
  source:
    sourceArchiveUrl: gs://your-bucket/function-source.zip
  httpsTrigger: {}
  environmentVariables:
    SOLANA_RPC_URL: https://api.mainnet-beta.solana.com
    RUST_LOG: info
EOF
zip -r function-source.zip target/release/solana-mcp-server function.yaml && gsutil cp function-source.zip gs://$(gcloud config get-value project)-functions/ && gcloud functions deploy solana-mcp-server --source=gs://$(gcloud config get-value project)-functions/function-source.zip --trigger-http --runtime=provided --memory=256MB --timeout=30s --set-env-vars SOLANA_RPC_URL=https://api.mainnet-beta.solana.com,RUST_LOG=info && echo "✅ Google Cloud Functions deployment complete! Check: gcloud functions describe solana-mcp-server"