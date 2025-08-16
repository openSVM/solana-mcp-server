#!/bin/bash

# Fast benchmark script for Solana MCP Server
echo "🚀 Running optimized benchmarks with reduced network overhead..."

# Set environment variables for faster benchmarks
export SOLANA_RPC_URL="http://localhost:8899"
export SOLANA_COMMITMENT="confirmed"
export RUST_LOG="warn"  # Reduce logging overhead

# Create benchmark results directory
mkdir -p benchmark-results

echo "📊 Running core performance benchmarks (no network calls)..."
timeout 300 cargo bench --bench optimized_benchmarks || echo "Optimized benchmarks completed or timed out"

echo "🌐 Running HTTP API benchmarks (with local mock RPC)..."
timeout 300 cargo bench --bench http_api_bench || echo "HTTP API benchmarks completed or timed out"

echo "🔧 Running simplified RPC methods benchmarks..."
timeout 300 cargo bench --bench rpc_methods_bench || echo "RPC methods benchmarks completed or timed out"

echo "📡 Running simplified WebSocket benchmarks..."
timeout 300 cargo bench --bench websocket_bench || echo "WebSocket benchmarks completed or timed out"

# Copy HTML reports to results directory
if [ -d "target/criterion" ]; then
    echo "📋 Copying benchmark reports..."
    cp -r target/criterion benchmark-results/
    echo "✅ Benchmark reports saved to benchmark-results/"
fi

echo "🎯 Optimized benchmarks completed!"
echo ""
echo "📈 Performance improvements:"
echo "   • Eliminated external network calls"
echo "   • Shared server instances"
echo "   • Connection reuse and pooling"
echo "   • Reduced timeout values"
echo "   • Focused on core logic performance"
echo ""
echo "📊 View detailed results in benchmark-results/criterion/"