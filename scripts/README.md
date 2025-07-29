# One-liner Deployment Scripts

This directory contains one-liner deployment scripts for the Solana MCP Server across different platforms.

## Quick Start

```bash
# Run the main script to see all options
./scripts/deploy.sh
```

## Available Scripts

### Local Development
```bash
./scripts/deploy-local.sh
```
- Downloads latest release binary
- Configures Claude Desktop integration
- No additional dependencies required

### Docker Container
```bash
./scripts/deploy-docker.sh
```
- Builds Docker image
- Runs container on port 8080
- Requires: Docker

### Docker Compose
```bash
./scripts/deploy-compose.sh
```
- Creates docker-compose.yml
- Deploys with health checks
- Requires: Docker, Docker Compose

### Kubernetes
```bash
./scripts/deploy-k8s.sh
```
- Creates deployment and service manifests
- Deploys 3 replicas with load balancer
- Requires: Docker, kubectl, Kubernetes cluster

### AWS Lambda
```bash
./scripts/deploy-lambda.sh
```
- Builds Lambda-compatible binary
- Creates function and API Gateway
- Requires: AWS CLI, cargo-lambda, valid AWS credentials

### Google Cloud Functions
```bash
./scripts/deploy-gcf.sh
```
- Builds for Cloud Functions runtime
- Deploys HTTP-triggered function
- Requires: gcloud CLI, valid GCP credentials

### Vercel Edge Functions
```bash
./scripts/deploy-vercel.sh
```
- Creates Vercel project structure
- Deploys to Vercel Edge runtime
- Requires: Vercel CLI, Node.js

## Prerequisites by Platform

| Platform | Requirements |
|----------|-------------|
| Local | curl, unzip |
| Docker | Docker |
| Docker Compose | Docker, docker-compose |
| Kubernetes | Docker, kubectl, K8s cluster |
| AWS Lambda | AWS CLI, cargo-lambda, AWS credentials |
| Google Cloud | gcloud CLI, GCP credentials |
| Vercel | Vercel CLI, Node.js |

## Environment Variables

All scripts use these default environment variables:
- `SOLANA_RPC_URL=https://api.mainnet-beta.solana.com`
- `SOLANA_COMMITMENT=confirmed`
- `RUST_LOG=info`

Modify the scripts to customize these values for your deployment.

## Troubleshooting

If a deployment fails:
1. Check that all required tools are installed
2. Verify credentials are configured (for cloud platforms)
3. Ensure network connectivity
4. Check platform-specific logs

## Support

See the main [DEPLOYMENT.md](../docs/deployment.md) for detailed deployment guides and troubleshooting.