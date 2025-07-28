#!/bin/bash
# One-liner deployment script for Kubernetes with autoscaling
set -e

echo "☸️ Deploying Solana MCP Server to Kubernetes with autoscaling..."

# Build and tag the image
echo "🔨 Building Docker image..."
docker build -t solana-mcp-server:latest .

# Deploy the application and autoscaling
echo "🚀 Deploying to Kubernetes..."
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/hpa.yaml

echo "⏳ Waiting for deployment to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/solana-mcp-server

echo "✅ Kubernetes deployment with autoscaling complete!"
echo ""
echo "📊 Check status:"
echo "  kubectl get pods,svc,hpa -l app=solana-mcp-server"
echo ""
echo "📈 Monitor autoscaling:"
echo "  kubectl get hpa solana-mcp-server-hpa --watch"
echo ""
echo "🔍 Check metrics:"
echo "  kubectl port-forward svc/solana-mcp-service 8080:8080"
echo "  curl http://localhost:8080/metrics"
echo ""
echo "🏥 Check health:"
echo "  curl http://localhost:8080/health"