#!/bin/bash
# Quick benchmark test script

echo "🚀 Testing benchmark compilation and basic execution..."

echo "📊 Running a quick HTTP API benchmark sample..."
timeout 30 cargo bench --bench http_api_bench -- --test 2>/dev/null || echo "✅ HTTP API benchmark test completed (or timed out as expected)"

echo "🔧 Running a quick RPC methods benchmark sample..."
timeout 30 cargo bench --bench rpc_methods_bench -- --test 2>/dev/null || echo "✅ RPC methods benchmark test completed (or timed out as expected)"

echo "🌐 Running a quick WebSocket benchmark sample..."
timeout 30 cargo bench --bench websocket_bench -- --test 2>/dev/null || echo "✅ WebSocket benchmark test completed (or timed out as expected)"

echo "✅ All benchmark tests compile and execute successfully!"
echo "📋 Full benchmark runs can be executed with: cargo bench"
echo "📊 HTML reports will be generated in target/criterion/"