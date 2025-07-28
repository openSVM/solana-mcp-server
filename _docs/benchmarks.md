---
layout: docs
title: "Benchmark Tests"
description: "Comprehensive performance benchmarks and testing results"
order: 20
category: architecture
---

# Benchmark Tests

This directory contains comprehensive performance benchmarks for the Solana MCP Server.

## Overview

The benchmarks measure performance across three key areas:

### 1. HTTP API Benchmarks (`http_api_bench.rs`)
- **MCP Protocol Performance**: Initialization, tools list, protocol compliance
- **RPC Tool Calls**: Individual method call latency and throughput
- **Concurrent Requests**: Multi-client performance under load
- **Endpoint Performance**: Health and metrics endpoint response times

### 2. RPC Methods Benchmarks (`rpc_methods_bench.rs`)
- **System Methods**: Core blockchain queries (getHealth, getVersion, etc.)
- **Account Methods**: Balance and account information retrieval
- **Block/Transaction Methods**: Blockchain data access performance
- **Token Methods**: SPL token operations
- **Error Handling**: Invalid request processing efficiency

### 3. WebSocket Benchmarks (`websocket_bench.rs`)
- **Connection Management**: WebSocket establishment and teardown
- **Subscription Operations**: Real-time data subscription performance
- **Message Throughput**: High-frequency message handling
- **Concurrent Connections**: Multi-client WebSocket performance
- **Error Recovery**: Invalid request and connection error handling

## Running Benchmarks

### Local Execution

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench http_api_bench
cargo bench --bench rpc_methods_bench  
cargo bench --bench websocket_bench

# Generate HTML reports
cargo bench -- --output-format html
```

### Quick Test
```bash
# Test benchmark compilation and basic execution
./test-benchmarks.sh
```

## GitHub Actions Integration

The benchmarks are automatically executed via GitHub Actions:

- **Triggers**: Push to main/develop, PRs, daily schedule, manual dispatch
- **Platforms**: Ubuntu latest with full Rust toolchain
- **Artifacts**: HTML reports, detailed metrics, performance analysis
- **PR Integration**: Automatic benchmark result comments

### Workflow Features

1. **Comprehensive Execution**: All three benchmark suites
2. **HTML Report Generation**: Interactive charts and detailed analysis
3. **Artifact Storage**: 30-day retention of benchmark results
4. **Performance Analysis**: Regression detection and recommendations
5. **PR Comments**: Automatic benchmark summary on pull requests

## Benchmark Results

### Artifacts Generated

- `benchmark-reports-{run-id}`: Summary and analysis files
- `criterion-detailed-reports-{run-id}`: Interactive HTML reports
- `performance-comparison-{run-id}`: PR performance comparison

### Report Structure

```
benchmark-results/
├── README.md                      # Benchmark overview
├── benchmark-summary.txt          # Text summary
├── system-info.txt               # System information
├── performance-analysis.md       # Performance analysis
├── http-api-criterion-reports/   # HTTP API detailed reports
├── rpc-methods-criterion-reports/ # RPC methods detailed reports
└── websocket-criterion-reports/  # WebSocket detailed reports
```

## Performance Targets

### Response Time Targets
- Simple RPC calls: < 50ms
- Account queries: < 100ms  
- Block/transaction queries: < 200ms
- WebSocket connections: < 100ms

### Throughput Targets
- Concurrent HTTP requests: > 100 req/s
- WebSocket connections: > 50 concurrent
- Message throughput: > 1000 msg/s

## Benchmark Configuration

### Test Environment
- **Ports**: 9001-9003 (dedicated benchmark ports)
- **Duration**: Criterion default with HTML output
- **Concurrency**: 1, 5, 10, 20 concurrent clients tested
- **Network**: Local loopback for consistent results

### Error Handling
- Network timeouts handled gracefully
- Invalid parameter testing included  
- Connection failure recovery tested
- Real Solana RPC integration (may timeout in CI)

## Development

### Adding New Benchmarks

1. **Identify Performance-Critical Code**: Focus on hot paths
2. **Create Benchmark Function**: Follow Criterion patterns
3. **Add to Benchmark Group**: Include in appropriate suite
4. **Update Documentation**: Document new benchmark purpose
5. **Test Locally**: Verify benchmark executes successfully

### Best Practices

- Use `black_box()` to prevent compiler optimizations
- Test realistic scenarios and data sizes
- Include both success and error path benchmarks
- Use separate ports to avoid test conflicts
- Document performance expectations and targets

## Troubleshooting

### Common Issues

1. **Port Conflicts**: Benchmarks use dedicated ports 9001-9003
2. **Network Timeouts**: Some tests make real Solana RPC calls
3. **Resource Limits**: Large concurrent tests may hit system limits
4. **Build Dependencies**: Requires OpenSSL and standard build tools

### CI-Specific Considerations

- Ubuntu system dependencies installed automatically
- Benchmarks continue on error for partial results
- HTML reports generated even if some benchmarks fail
- Network restrictions may affect external RPC calls

## Future Enhancements

- **Historical Tracking**: Compare benchmark results over time  
- **Regression Alerts**: Automated alerts for performance degradation
- **Load Testing**: Extended duration and stress testing
- **Memory Profiling**: Memory usage and leak detection
- **Real Network Testing**: Against actual Solana clusters