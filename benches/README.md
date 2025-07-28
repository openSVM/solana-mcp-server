# Solana MCP Server Benchmarks

This directory contains performance benchmarks for the Solana MCP Server.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Test benchmarks compile
cargo check --benches

# Quick test execution
./test-benchmarks.sh
```

## Benchmark Suites

- **`http_api_bench.rs`** - HTTP JSON-RPC API performance
- **`rpc_methods_bench.rs`** - Individual RPC method performance  
- **`websocket_bench.rs`** - WebSocket subscription performance

## GitHub Actions

Benchmarks run automatically on:
- Push to main/develop branches
- Pull requests  
- Daily schedule (2 AM UTC)
- Manual workflow dispatch

Results are saved as artifacts with interactive HTML reports.

## Documentation

See [`docs/benchmarks.md`](../docs/benchmarks.md) for detailed documentation.

## Results

Benchmark results include:
- Interactive HTML reports via Criterion
- Performance comparison analysis
- System information and metrics
- Regression detection and recommendations