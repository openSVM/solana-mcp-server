# MCP JSON-RPC API Specification

This document describes the Model Context Protocol (MCP) JSON-RPC API implementation for the Solana MCP Server.

## Overview

The Solana MCP Server implements the full MCP JSON-RPC 2.0 specification, providing a standards-compliant interface for AI clients to interact with the Solana blockchain.

## API Endpoints

### HTTP Web Service Mode

When running in web service mode, the server exposes the following endpoints:

- `POST /api/mcp` - MCP JSON-RPC 2.0 API endpoint
- `GET /health` - Health check and capability information
- `GET /metrics` - Prometheus metrics (Prometheus format)

## MCP JSON-RPC 2.0 Specification

All MCP requests and responses follow the JSON-RPC 2.0 specification with MCP-specific extensions.

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "methodName",
  "params": {
    // Method-specific parameters
  }
}
```

### Response Format

#### Success Response
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    // Method-specific result data
  }
}
```

#### Error Response
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": {
      "protocolVersion": "2024-11-05"
    }
  }
}
```

## Content Types

The MCP specification supports multiple content types with optional annotations:

### Text Content
```json
{
  "type": "text",
  "text": "Content text here",
  "annotations": {
    "audience": ["user", "assistant"],
    "priority": 0.8,
    "lastModified": "2024-01-15T10:00:00Z"
  }
}
```

### Image Content
```json
{
  "type": "image",
  "data": "base64-encoded-image-data",
  "mimeType": "image/png",
  "annotations": {
    "audience": ["user"],
    "priority": 1.0
  }
}
```

### Resource Content
```json
{
  "type": "resource",
  "resource": {
    "uri": "https://example.com/resource",
    "mimeType": "application/json"
  },
  "annotations": {
    "priority": 0.5
  }
}
```

## Annotations

Annotations provide metadata about content objects:

- `audience`: Array of intended recipients (`["user", "assistant"]`)
- `priority`: Importance level (0.0 = least important, 1.0 = most important)
- `lastModified`: ISO 8601 timestamp of last modification

## Error Codes

Standard JSON-RPC 2.0 error codes are used:

- `-32700`: Parse error
- `-32600`: Invalid Request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `-32002`: Server not initialized (MCP-specific)

## Health Check Response

The `/health` endpoint returns detailed server information:

```json
{
  "status": "ok",
  "service": "solana-mcp-server",
  "version": "1.0.2",
  "protocol": "2024-11-05",
  "capabilities": {
    "tools": true,
    "resources": true,
    "prompts": false,
    "sampling": false
  }
}
```

## Headers

### Request Headers
- `Content-Type: application/json` (required)
- `Accept: application/json` (recommended)

### Response Headers  
- `Content-Type: application/json`
- `X-MCP-Version: 2024-11-05` (protocol version)
- `Cache-Control: no-cache`

## Client Usage Examples

### Python Example
```python
import requests
import json

# Initialize MCP session
init_request = {
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
}

response = requests.post(
    "http://localhost:3000/api/mcp",
    headers={"Content-Type": "application/json"},
    json=init_request
)

print(response.json())
```

### JavaScript Example
```javascript
const mcpRequest = {
  jsonrpc: "2.0",
  id: 2,
  method: "tools/call",
  params: {
    name: "getBalance",
    arguments: {
      pubkey: "11111111111111111111111111111112"
    }
  }
};

fetch("http://localhost:3000/api/mcp", {
  method: "POST",
  headers: {
    "Content-Type": "application/json"
  },
  body: JSON.stringify(mcpRequest)
})
.then(response => response.json())
.then(data => console.log(data));
```

## Validation

The server performs strict validation on all requests:

1. **JSON-RPC 2.0 compliance**: Validates `jsonrpc`, `method`, and `id` fields
2. **Content-Type validation**: Ensures `application/json` content type
3. **Parameter validation**: Validates method-specific parameters
4. **Protocol version compatibility**: Checks MCP protocol version

## Security Considerations

- All HTTP responses include appropriate caching headers
- Request validation prevents malformed JSON-RPC requests
- Parameter sanitization prevents injection attacks
- Network detection for proper metrics labeling
- Rate limiting should be implemented at the reverse proxy level

## Compatibility

This implementation follows:
- JSON-RPC 2.0 specification
- MCP Protocol version 2024-11-05
- HTTP/1.1 and HTTP/2 standards
- OpenAPI 3.0 compatible (documentation available separately)

The server maintains full backward compatibility with existing stdio transport clients while providing modern HTTP JSON-RPC capabilities for web-based integrations.