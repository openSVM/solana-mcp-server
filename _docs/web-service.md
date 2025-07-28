---
layout: docs
title: "Web Service Mode"
description: "HTTP API mode with JSON-RPC endpoints, health checks, and metrics integration"
order: 11
category: deployment
---

# Solana MCP Server - Web Service Mode

The Solana MCP Server supports running as an HTTP web service, providing full MCP JSON-RPC 2.0 API compliance for web-based integrations.

## Overview

When running in web service mode, the server provides:
- **Full MCP JSON-RPC 2.0 compliance** following the official specification  
- **Proper content type handling** with annotations support
- **Standards-compliant error responses** with protocol versioning
- **Health checks** with capability information
- **Prometheus metrics** integration

## Running as Web Service

### Basic Usage

```bash
# Run on default port 3000
solana-mcp-server web

# Run on custom port  
solana-mcp-server web --port 8000
```

### Available Endpoints

When running in web service mode, the server provides:

#### POST /api/mcp
- **Purpose**: MCP JSON-RPC 2.0 API endpoint
- **Content-Type**: `application/json` (required)
- **Description**: Accepts MCP JSON-RPC requests following the 2024-11-05 specification
- **Features**: Full protocol validation, proper error handling, content annotations

#### GET /health
- **Purpose**: Health check and capability information
- **Response**: Detailed server status including protocol version and capabilities
- **Description**: Returns comprehensive server health and MCP capability information

#### GET /metrics
- **Purpose**: Prometheus metrics endpoint
- **Content-Type**: `text/plain; version=0.0.4`
- **Description**: Exposes Prometheus-format metrics for monitoring

## API Usage Examples

### Initialize MCP Session

```bash
curl -X POST http://localhost:3000/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "my-client",
        "version": "1.0.0"
      }
    }
  }'
```

### List Available Tools

```bash
curl -X POST http://localhost:3000/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
  }'
```

### Call a Tool (Get Account Balance)

```bash
curl -X POST http://localhost:3000/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "getBalance",
      "arguments": {
        "pubkey": "11111111111111111111111111111112"
      }
    }
  }'
```

## Differences from Stdio Mode

### Initialization
- In stdio mode, the server waits for an `initialize` request
- In web service mode, the server is automatically initialized and ready to accept requests

### Session Management
- Stdio mode maintains a single persistent session
- Web service mode handles stateless HTTP requests (each request is independent)

### Error Handling
- Stdio mode can terminate on critical errors
- Web service mode returns HTTP error codes and continues serving requests

## Integration Examples

### Using with curl

```bash
#!/bin/bash
# Simple script to get account info via web API

ACCOUNT_PUBKEY="$1"
SERVER_URL="http://localhost:3000/api/mcp"

curl -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"tools/call\",
    \"params\": {
      \"name\": \"getAccountInfo\",
      \"arguments\": {
        \"pubkey\": \"$ACCOUNT_PUBKEY\"
      }
    }
  }" | jq .
```

### Using with Python

```python
import requests
import json

class SolanaMcpClient:
    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url
        self.api_url = f"{base_url}/api/mcp"
        self.request_id = 0
    
    def call_tool(self, tool_name, arguments=None):
        self.request_id += 1
        payload = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments or {}
            }
        }
        
        response = requests.post(
            self.api_url,
            headers={"Content-Type": "application/json"},
            json=payload
        )
        
        return response.json()
    
    def get_balance(self, pubkey):
        return self.call_tool("getBalance", {"pubkey": pubkey})
    
    def get_health(self):
        response = requests.get(f"{self.base_url}/health")
        return response.json()

# Usage
client = SolanaMcpClient()
balance = client.get_balance("11111111111111111111111111111112")
print(json.dumps(balance, indent=2))
```

### Using with JavaScript/Node.js

```javascript
class SolanaMcpClient {
    constructor(baseUrl = 'http://localhost:3000') {
        this.baseUrl = baseUrl;
        this.apiUrl = `${baseUrl}/api/mcp`;
        this.requestId = 0;
    }
    
    async callTool(toolName, arguments = {}) {
        this.requestId++;
        const payload = {
            jsonrpc: '2.0',
            id: this.requestId,
            method: 'tools/call',
            params: {
                name: toolName,
                arguments: arguments
            }
        };
        
        const response = await fetch(this.apiUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(payload)
        });
        
        return await response.json();
    }
    
    async getBalance(pubkey) {
        return await this.callTool('getBalance', { pubkey });
    }
    
    async getHealth() {
        const response = await fetch(`${this.baseUrl}/health`);
        return await response.json();
    }
}

// Usage
const client = new SolanaMcpClient();
client.getBalance('11111111111111111111111111111112')
    .then(result => console.log(JSON.stringify(result, null, 2)));
```

## Monitoring and Observability

### Health Checks
Use the `/health` endpoint for liveness and readiness probes:

```bash
# Simple health check
curl -f http://localhost:3000/health

# In Kubernetes
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10
```

### Metrics Collection
The `/metrics` endpoint provides Prometheus-compatible metrics:

```bash
# View metrics
curl http://localhost:3000/metrics

# Prometheus scrape config
scrape_configs:
  - job_name: 'solana-mcp-server'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

## Production Deployment

### Docker Compose Example

```yaml
version: '3.8'
services:
  solana-mcp-server:
    image: solana-mcp-server:latest
    command: ["web", "--port", "3000"]
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
    volumes:
      - ./config.json:/app/config.json:ro
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Kubernetes Deployment

```yaml
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
        args: ["web", "--port", "3000"]
        ports:
        - containerPort: 3000
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"

---
apiVersion: v1
kind: Service
metadata:
  name: solana-mcp-server-service
spec:
  selector:
    app: solana-mcp-server
  ports:
  - port: 80
    targetPort: 3000
  type: LoadBalancer
```

## Security Considerations

- The web service mode exposes the MCP server over HTTP
- Consider implementing authentication/authorization for production use
- Use HTTPS in production environments
- Configure appropriate CORS headers if needed for browser access
- Monitor and rate-limit API usage to prevent abuse

## Limitations

- Web service mode does not support streaming or persistent connections
- Each HTTP request is independent (no session state)
- Large responses may be subject to HTTP timeout limits
- No built-in authentication (implement at reverse proxy level)