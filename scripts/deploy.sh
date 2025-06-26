#!/bin/bash
# Main deployment script - choose your deployment option
set -e

echo "🚀 Solana MCP Server - One-liner Deployment Scripts"
echo "==================================================="
echo
echo "Available deployment options:"
echo "1. Local Development    - ./scripts/deploy-local.sh"
echo "2. Docker Container     - ./scripts/deploy-docker.sh"
echo "3. Docker Compose       - ./scripts/deploy-compose.sh"
echo "4. Kubernetes          - ./scripts/deploy-k8s.sh"
echo "5. AWS Lambda          - ./scripts/deploy-lambda.sh"
echo "6. Google Cloud Functions - ./scripts/deploy-gcf.sh"
echo "7. Vercel Edge Functions  - ./scripts/deploy-vercel.sh"
echo
echo "Usage: Choose and run any script directly, e.g.:"
echo "  bash scripts/deploy-local.sh"
echo "  bash scripts/deploy-docker.sh"
echo
echo "Each script is a complete one-liner deployment solution!"
echo "Make sure you have the required tools installed for your chosen platform."
echo