#!/bin/bash
# One-liner deployment script for Kubernetes
set -e

echo "☸️ Deploying Solana MCP Server to Kubernetes..."

# Create Kubernetes deployment and service, then apply
cat > k8s-deployment.yaml << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: solana-mcp-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: solana-mcp-server
  template:
    metadata:
      labels:
        app: solana-mcp-server
    spec:
      containers:
      - name: solana-mcp-server
        image: solana-mcp-server:latest
        ports:
        - containerPort: 8080
        env:
        - name: SOLANA_RPC_URL
          value: "https://api.mainnet-beta.solana.com"
        - name: SOLANA_COMMITMENT
          value: "confirmed"
        resources:
          requests:
            memory: "64Mi"
            cpu: "250m"
          limits:
            memory: "128Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: solana-mcp-service
spec:
  selector:
    app: solana-mcp-server
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
EOF
docker build -t solana-mcp-server:latest . && kubectl apply -f k8s-deployment.yaml && echo "✅ Kubernetes deployment complete! Check status: kubectl get pods,svc -l app=solana-mcp-server"