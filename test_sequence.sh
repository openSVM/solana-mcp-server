#!/bin/bash

# Start the server and send both requests through the same pipe
(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"test-client","version":"1.0.0"},"capabilities":{}}}'; sleep 1; echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}') | ./target/release/solana-mcp-server
