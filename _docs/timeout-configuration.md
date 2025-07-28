# Timeout Configuration

The Solana MCP Server provides comprehensive timeout configurations to prevent hanging operations and ensure reliable performance in production environments.

## Overview

All timeout settings are configurable through the `config.json` file or fall back to sensible defaults. The server includes timeouts for:

- HTTP API requests
- WebSocket connections and messages
- RPC subscription creation
- Connection idle timeout

## Configuration

### Using config.json

```json
{
  "rpc_url": "https://api.opensvm.com",
  "commitment": "confirmed",
  "protocol_version": "2024-11-05",
  "timeouts": {
    "http_request_seconds": 30,
    "websocket_connection_seconds": 30,
    "websocket_message_seconds": 10,
    "subscription_seconds": 15,
    "max_idle_seconds": 300
  }
}
```

### Timeout Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `http_request_seconds` | 30 | Maximum time for HTTP API requests |
| `websocket_connection_seconds` | 30 | WebSocket connection establishment timeout |
| `websocket_message_seconds` | 10 | Individual WebSocket message timeout |
| `subscription_seconds` | 15 | RPC subscription creation timeout |
| `max_idle_seconds` | 300 | Maximum idle time before closing connections |

### Environment Variables

If no `config.json` is provided, the server uses default timeout values. Individual timeout settings cannot currently be overridden via environment variables.

## GitHub Actions Timeouts

The CI/CD workflows have been updated with comprehensive timeout protection:

### Build Workflow (`build.yml`)
- **Overall job timeout**: 45 minutes
- **Build step**: 15 minutes
- **Dependency check**: 5 minutes  
- **Test execution**: 20 minutes

### Benchmark Workflow (`benchmark.yml`)
- **Overall job timeout**: 60 minutes
- **Individual benchmarks**: 15 minutes each
- **Command-level timeout**: 10 minutes via `timeout` command

### Audit Workflow (`audit.yml`)
- **Overall job timeout**: 15 minutes
- **cargo-audit installation**: 5 minutes
- **Dependency check**: 3 minutes
- **Audit execution**: 5 minutes

### Release Workflow (`release.yml`)
- **Overall job timeout**: 60 minutes (for cross-compilation)

## Server Implementation

### HTTP Server Timeouts

The HTTP server includes:
- Request-level timeouts via `tower-http::timeout::TimeoutLayer`
- Graceful shutdown timeout (10 seconds)
- Configurable per-request timeout

### WebSocket Server Timeouts

The WebSocket server provides:
- Connection establishment timeout
- Message send/receive timeouts
- Subscription creation timeout
- Idle connection cleanup
- Ping/pong heartbeat mechanism

### Error Handling

When timeouts occur:
- HTTP requests return `408 Request Timeout`
- WebSocket connections are gracefully closed
- Failed subscriptions return JSON-RPC error responses
- All timeout events are logged with appropriate severity

## Monitoring and Diagnostics

Timeout events are exposed through:
- **Prometheus metrics**: `solana_mcp_rpc_requests_failed_total{error_type="timeout"}`
- **Structured logging**: JSON-formatted timeout logs
- **Health endpoint**: Connection and timeout status

## Best Practices

### Development
- Use shorter timeouts (5-15 seconds) for development
- Enable debug logging to monitor timeout behavior
- Test with network delays and poor connectivity

### Production
- Use longer timeouts (30-60 seconds) for production stability
- Monitor timeout metrics for performance optimization
- Configure load balancer timeouts to exceed server timeouts

### High-Load Environments
- Increase `max_idle_seconds` for persistent connections
- Reduce `websocket_message_seconds` for responsive interaction
- Monitor connection pool utilization

## Troubleshooting

### Common Issues

**WebSocket connections timing out:**
```json
{
  "timeouts": {
    "websocket_connection_seconds": 60,
    "websocket_message_seconds": 20
  }
}
```

**HTTP requests timing out:**
```json
{
  "timeouts": {
    "http_request_seconds": 60
  }
}
```

**Subscription creation failing:**
```json
{
  "timeouts": {
    "subscription_seconds": 30
  }
}
```

### Debugging

Enable debug logging to see timeout behavior:
```bash
RUST_LOG=debug solana-mcp-server stdio
```

Check timeout metrics:
```bash
curl http://localhost:8080/metrics | grep timeout
```

## Examples

### Conservative Configuration (High Reliability)
```json
{
  "timeouts": {
    "http_request_seconds": 60,
    "websocket_connection_seconds": 45,
    "websocket_message_seconds": 20,
    "subscription_seconds": 30,
    "max_idle_seconds": 600
  }
}
```

### Aggressive Configuration (Low Latency)
```json
{
  "timeouts": {
    "http_request_seconds": 15,
    "websocket_connection_seconds": 10,
    "websocket_message_seconds": 5,
    "subscription_seconds": 10,
    "max_idle_seconds": 120
  }
}
```

### Load Balancer Integration
For systems behind load balancers, ensure server timeouts are less than load balancer timeouts:

```json
{
  "timeouts": {
    "http_request_seconds": 25
  }
}
```

*If load balancer timeout is 30 seconds*

This comprehensive timeout system ensures the Solana MCP Server never hangs indefinitely and provides predictable behavior under all network conditions.