#!/bin/bash
set -e

echo "Building Solana MCP Server..."
cargo build --release

echo "Starting MCP JSON-RPC test..."

# Create a temporary directory for test files
TEST_DIR=$(mktemp -d)
echo "Test directory: $TEST_DIR"

# Start the server in the background
./target/release/solana-mcp-server stdio > "$TEST_DIR/server-output.log" 2> "$TEST_DIR/server-error.log" &
SERVER_PID=$!

echo "Started server with PID: $SERVER_PID"

# Give server time to start
sleep 2

# Function to send JSON-RPC request and capture response
send_request() {
    local request="$1"
    local description="$2"
    
    echo "Sending $description..."
    echo "$request" | timeout 5s nc -q 1 localhost 8080 > "$TEST_DIR/response.json" 2>/dev/null || {
        # Fallback: write to server stdin directly
        echo "$request" > /proc/$SERVER_PID/fd/0 2>/dev/null || {
            echo "Failed to send request: $description"
            return 1
        }
    }
    
    if [ -f "$TEST_DIR/response.json" ]; then
        echo "Response for $description:"
        cat "$TEST_DIR/response.json"
        echo ""
        
        # Validate JSON
        if jq . "$TEST_DIR/response.json" >/dev/null 2>&1; then
            echo "✅ Valid JSON response"
        else
            echo "❌ Invalid JSON response"
        fi
        echo "---"
    fi
}

# Test 1: Initialize request
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'

# Test 2: Tools list request  
TOOLS_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# Test 3: Tool call request
CALL_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"getHealth","arguments":{}}}'

# Send requests
send_request "$INIT_REQUEST" "initialize request"
send_request "$TOOLS_REQUEST" "tools/list request"
send_request "$CALL_REQUEST" "tools/call request"

# Cleanup
echo "Stopping server..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo "Server output:"
cat "$TEST_DIR/server-output.log" 2>/dev/null || echo "No server output"
echo ""
echo "Server errors:"  
cat "$TEST_DIR/server-error.log" 2>/dev/null || echo "No server errors"

# Cleanup temp directory
rm -rf "$TEST_DIR"

echo "Test completed"