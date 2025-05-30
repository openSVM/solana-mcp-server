#!/bin/bash
# One-liner deployment script for AWS Lambda
set -e

echo "⚡ Deploying Solana MCP Server to AWS Lambda..."

# Install dependencies, build Lambda package, and deploy with AWS CLI
cargo install cargo-lambda && cargo lambda build --release && cp target/lambda/solana-mcp-server/bootstrap . && zip lambda-deployment.zip bootstrap && aws lambda create-function --function-name solana-mcp-server --runtime provided.al2 --role arn:aws:iam::$(aws sts get-caller-identity --query Account --output text):role/lambda-execution-role --handler bootstrap --zip-file fileb://lambda-deployment.zip --timeout 30 --memory-size 256 --environment Variables='{SOLANA_RPC_URL=https://api.mainnet-beta.solana.com,RUST_LOG=info}' && aws apigatewayv2 create-api --name solana-mcp-api --protocol-type HTTP --target arn:aws:lambda:$(aws configure get region):$(aws sts get-caller-identity --query Account --output text):function:solana-mcp-server && echo "✅ AWS Lambda deployment complete! Function: solana-mcp-server"