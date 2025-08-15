#!/usr/bin/env node

const { spawn } = require('child_process');

console.log('Testing MCP Server JSON-RPC responses...');

// Start the server
const server = spawn('./target/release/solana-mcp-server', ['stdio'], {
  env: {
    ...process.env,
    SOLANA_RPC_URL: 'https://api.devnet.solana.com',
    RUST_LOG: 'info'
  }
});

let responses = [];
let requestId = 1;

// Capture server output
server.stdout.on('data', (data) => {
  const lines = data.toString().split('\n').filter(line => line.trim());
  
  for (const line of lines) {
    try {
      const message = JSON.parse(line);
      if (message.jsonrpc === '2.0') {
        console.log(`\n📨 Server Response:`, JSON.stringify(message, null, 2));
        responses.push(message);
      }
    } catch (e) {
      // Not JSON-RPC, might be logs
      if (!line.includes('"timestamp"')) {
        console.log('📄 Server message:', line);
      }
    }
  }
});

server.stderr.on('data', (data) => {
  console.log('❌ Server stderr:', data.toString());
});

// Function to send a request
function sendRequest(method, params, description) {
  return new Promise((resolve) => {
    const request = {
      jsonrpc: '2.0',
      id: requestId++,
      method,
      params
    };
    
    console.log(`\n🚀 Sending ${description}:`, JSON.stringify(request, null, 2));
    server.stdin.write(JSON.stringify(request) + '\n');
    
    // Wait a bit for response
    setTimeout(resolve, 2000);
  });
}

// Test sequence
async function runTests() {
  // Wait for server to start
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  try {
    // Test 1: Initialize
    await sendRequest('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: {
        name: 'test-client',
        version: '1.0.0'
      }
    }, 'Initialize Request');
    
    // Test 2: List tools
    await sendRequest('tools/list', {}, 'Tools List Request');
    
    // Test 3: Call tool
    await sendRequest('tools/call', {
      name: 'getHealth',
      arguments: {}
    }, 'Tool Call Request');
    
    // Analyze results
    console.log('\n📊 Test Results Summary:');
    console.log(`Total responses received: ${responses.length}`);
    
    responses.forEach((response, i) => {
      console.log(`\nResponse ${i + 1}:`);
      if (response.error) {
        console.log(`  ❌ Error: ${response.error.message} (code: ${response.error.code})`);
      } else if (response.result) {
        console.log(`  ✅ Success: ${typeof response.result} result`);
      } else {
        console.log(`  ℹ️  Notification: ${response.method || 'unknown'}`);
      }
    });
    
    // Check for schema issues
    console.log('\n🔍 Schema Validation Check:');
    responses.forEach((response, i) => {
      const issues = [];
      
      if (!response.jsonrpc || response.jsonrpc !== '2.0') {
        issues.push('Missing or invalid jsonrpc field');
      }
      
      if (response.id === undefined && response.method === undefined) {
        issues.push('Missing both id and method fields');
      }
      
      if (response.result === undefined && response.error === undefined && response.method === undefined) {
        issues.push('Missing result, error, and method fields');
      }
      
      if (issues.length > 0) {
        console.log(`  Response ${i + 1} issues: ${issues.join(', ')}`);
      } else {
        console.log(`  Response ${i + 1}: ✅ Schema valid`);
      }
    });
    
  } catch (error) {
    console.error('Test error:', error);
  } finally {
    server.kill();
    process.exit(0);
  }
}

// Start tests
runTests().catch(console.error);

// Handle server exit
server.on('close', (code) => {
  console.log(`\nServer exited with code ${code}`);
});