use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::{json, Value};
use solana_mcp_server::{Config, ServerState, start_mcp_server_task};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::runtime::Runtime;

/// Setup test server for RPC method benchmarking
async fn setup_rpc_benchmark_server() -> Result<(tokio::task::JoinHandle<()>, u16), Box<dyn std::error::Error + Send + Sync>> {
    let port = 9002;
    
    let config = Config::load().map_err(|e| format!("Failed to load config: {e}"))?;
    let server_state = ServerState::new(config);
    let state = Arc::new(RwLock::new(server_state));
    
    let handle = start_mcp_server_task(port, state);
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    Ok((handle, port))
}

async fn make_rpc_request(request: Value, port: u16) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://localhost:{port}/api/mcp"))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
        
    let json: Value = response.json().await?;
    Ok(json)
}

/// Benchmark system RPC methods
fn bench_system_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_rpc_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize server
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "rpc-benchmark", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_rpc_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let mut group = c.benchmark_group("system_methods");
    
    let system_methods = vec![
        "getHealth",
        "getVersion", 
        "getGenesisHash",
        "getSlot",
        "getBlockHeight",
        "getEpochInfo",
        "getIdentity",
        "getClusterNodes",
        "minimumLedgerSlot",
        "getHighestSnapshotSlot",
    ];
    
    for method_name in system_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": method_name,
                "arguments": {}
            }
        });
        
        group.bench_with_input(BenchmarkId::new("system", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let result = make_rpc_request(black_box(req.clone()), port).await;
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// Benchmark account-related RPC methods
fn bench_account_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_rpc_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize server
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "rpc-benchmark", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_rpc_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let mut group = c.benchmark_group("account_methods");
    
    let test_pubkey = "11111111111111111111111111111112"; // System program
    
    let account_methods = vec![
        ("getBalance", json!({"pubkey": test_pubkey})),
        ("getAccountInfo", json!({"pubkey": test_pubkey})),
        ("getBalanceAndContext", json!({"pubkey": test_pubkey})),
        ("getAccountInfoAndContext", json!({"pubkey": test_pubkey})),
        ("getMultipleAccounts", json!({"pubkeys": [test_pubkey]})),
        ("getMultipleAccountsAndContext", json!({"pubkeys": [test_pubkey]})),
    ];
    
    for (method_name, params) in account_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": method_name,
                "arguments": params
            }
        });
        
        group.bench_with_input(BenchmarkId::new("account", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let result = make_rpc_request(black_box(req.clone()), port).await;
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// Benchmark block and transaction methods
fn bench_block_transaction_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_rpc_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize server
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "rpc-benchmark", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_rpc_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let mut group = c.benchmark_group("block_transaction_methods");
    
    let block_tx_methods = vec![
        ("getLatestBlockhash", json!({})),
        ("getFeeForMessage", json!({"message": "AQABAgIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEBAgAABQEAAAAAAAAA"})),
        ("isBlockhashValid", json!({"blockhash": "EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N"})),
        ("getRecentBlockhash", json!({})), // Deprecated but still supported
        ("getFees", json!({})), // Deprecated but still supported
        ("getRecentPerformanceSamples", json!({})),
        ("getRecentPrioritizationFees", json!({})),
    ];
    
    for (method_name, params) in block_tx_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": method_name,
                "arguments": params
            }
        });
        
        group.bench_with_input(BenchmarkId::new("block_tx", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let result = make_rpc_request(black_box(req.clone()), port).await;
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// Benchmark token-related methods
fn bench_token_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_rpc_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize server
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "rpc-benchmark", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_rpc_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let mut group = c.benchmark_group("token_methods");
    
    let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // SPL Token program
    
    let token_methods = vec![
        ("getTokenAccountBalance", json!({"pubkey": "11111111111111111111111111111112"})),
        ("getTokenSupply", json!({"pubkey": "11111111111111111111111111111112"})),
        ("getTokenAccountsByOwner", json!({"pubkey": "11111111111111111111111111111112", "mint": token_program})),
        ("getTokenAccountsByDelegate", json!({"pubkey": "11111111111111111111111111111112", "mint": token_program})),
    ];
    
    for (method_name, params) in token_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": method_name,
                "arguments": params
            }
        });
        
        group.bench_with_input(BenchmarkId::new("token", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let result = make_rpc_request(black_box(req.clone()), port).await;
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_rpc_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize server
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "rpc-benchmark", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_rpc_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let mut group = c.benchmark_group("error_handling");
    
    // Test invalid method names
    let invalid_method_request = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "nonExistentMethod",
            "arguments": {}
        }
    });
    
    group.bench_function("invalid_method", |b| {
        b.to_async(&rt).iter(|| async {
            let result = make_rpc_request(black_box(invalid_method_request.clone()), port).await;
            black_box(result)
        })
    });
    
    // Test invalid parameters
    let invalid_params_request = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "getBalance",
            "arguments": {"invalid_param": "value"}
        }
    });
    
    group.bench_function("invalid_params", |b| {
        b.to_async(&rt).iter(|| async {
            let result = make_rpc_request(black_box(invalid_params_request.clone()), port).await;
            black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_system_methods,
    bench_account_methods,
    bench_block_transaction_methods,
    bench_token_methods,
    bench_error_handling
);
criterion_main!(benches);