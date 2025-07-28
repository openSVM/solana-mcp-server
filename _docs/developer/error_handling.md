---
layout: docs
title: "Error Handling and Logging Guidelines"
description: "Robust error handling and logging patterns for development and production"
order: 101
category: developer
---

# Error Handling and Logging Guidelines

## Overview

This document describes the robust error handling and logging patterns implemented in the Solana MCP Server.

## Error Handling Architecture

### Error Types

The server uses a centralized error hierarchy defined in `src/error.rs`:

- **`McpError::Client`** - Client-side errors (invalid input, malformed requests)
- **`McpError::Server`** - Server-side errors (internal failures, service unavailable)
- **`McpError::Rpc`** - RPC-specific errors (Solana client failures)
- **`McpError::Validation`** - Validation errors (invalid parameters, security checks)
- **`McpError::Network`** - Network errors (connectivity issues, timeouts)
- **`McpError::Auth`** - Authentication/Authorization errors

### Error Context

All errors include contextual information:
- Request ID for tracing
- Method name being executed
- Relevant parameters (sanitized)
- Source error information

### Error Usage Pattern

```rust
use crate::error::{McpError, McpResult};

pub async fn example_rpc_call(client: &RpcClient, param: &str) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "exampleCall";
    
    // Log request start
    log_rpc_request_start(request_id, method, Some(&client.url()), Some("param info"));

    match client.some_rpc_call(param).await {
        Ok(result) => {
            let duration = start_time.elapsed().as_millis() as u64;
            log_rpc_request_success(request_id, method, duration, Some("success message"));
            Ok(serde_json::json!({ "result": result }))
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(&client.url());
            
            log_rpc_request_failure(request_id, method, error.error_type(), duration, Some(&error.to_log_value()));
            Err(error)
        }
    }
}
```

## Logging Architecture

### Structured Logging

The server uses `tracing` with JSON output for structured logging:

- All logs include timestamps, levels, and contextual metadata
- Sensitive data is automatically sanitized
- Request IDs enable distributed tracing
- Performance metrics are captured

### Log Levels

- **ERROR** - RPC failures, server errors, critical issues
- **WARN** - Validation failures, recoverable errors
- **INFO** - Successful operations, server lifecycle events
- **DEBUG** - Detailed debugging information (testing only)

### Metrics Collection

The logging system automatically collects metrics:

- Total RPC calls
- Successful RPC calls
- Failed calls by error type
- Failed calls by method
- Request duration timing

Access metrics via `get_metrics().to_json()`.

## Security Considerations

### Data Sanitization

- URLs are sanitized to hide sensitive paths/parameters
- Long strings are truncated to prevent log flooding
- Parameter values are summarized, not logged in full
- Error messages distinguish between safe and internal errors

### Safe Error Messages

Client-facing error messages use `error.safe_message()` which:
- Shows validation errors to help with debugging
- Hides internal server errors to prevent information leakage
- Returns generic messages for auth errors

## Implementation Status

### Completed
- ✅ Error type hierarchy and context
- ✅ Structured logging with tracing
- ✅ Metrics collection
- ✅ Accounts RPC module
- ✅ System RPC module (partially)

### Remaining Work
- [ ] Complete system RPC module error handling
- [ ] Update blocks RPC module
- [ ] Update tokens RPC module
- [ ] Update transactions RPC module
- [ ] Integration tests for error scenarios

## Testing

Run error handling tests:
```bash
cargo test error::tests
cargo test logging::tests
```

## Configuration

Initialize logging in `main.rs`:
```rust
use solana_mcp_server::init_logging;

if let Err(e) = init_logging(Some("info")) {
    eprintln!("Failed to initialize logging: {}", e);
    std::process::exit(1);
}
```

Set log level via environment variable:
```bash
RUST_LOG=debug cargo run
```

## Best Practices

1. **Always use McpResult<T>** for RPC functions
2. **Include request IDs** for tracing correlation
3. **Log request start/success/failure** for all RPC calls
4. **Provide meaningful error context** using error builders
5. **Sanitize sensitive data** before logging
6. **Use appropriate error types** for categorization
7. **Test error handling paths** in integration tests
