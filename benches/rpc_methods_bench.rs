use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::{json, Value};
use solana_mcp_server::{Config, ServerState, start_mcp_server_task};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::runtime::Runtime;
use std::sync::OnceLock;

static RPC_BENCHMARK_SERVER: OnceLock<u16> = OnceLock::new();

/// Setup shared test server for RPC method benchmarking
async fn setup_shared_rpc_benchmark_server() -> Result<(tokio::task::JoinHandle<()>, u16), Box<dyn std::error::Error + Send + Sync>> {
    let port = 9002;
    
    let mut config = Config::load().map_err(|e| format!("Failed to load config: {e}"))?;
    // Use mock RPC endpoint for benchmarks to avoid external network calls
    config.rpc_url = "http://localhost:8899".to_string();
    config.timeouts.http_request_seconds = 1;
    
    let server_state = ServerState::new(config);
    let state = Arc::new(RwLock::new(server_state));
    
    let handle = start_mcp_server_task(port, state);
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    RPC_BENCHMARK_SERVER.set(port).ok();
    
    Ok((handle, port))
}

fn get_rpc_benchmark_server_port() -> u16 {
    if let Some(port) = RPC_BENCHMARK_SERVER.get() {
        *port
    } else {
        let rt = Runtime::new().unwrap();
        let (_handle, port) = rt.block_on(async {
            setup_shared_rpc_benchmark_server().await.expect("Failed to setup shared RPC server")
        });
        port
    }
}

async fn make_rpc_request(request: Value, port: u16) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    
    let client = CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    });
    
    let response = client
        .post(format!("http://localhost:{port}/api/mcp"))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
        
    let json: Value = response.json().await?;
    Ok(json)
}

/// Benchmark system RPC methods (reduced to fastest methods only)
fn bench_system_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let port = get_rpc_benchmark_server_port();
    
    // Initialize server (one time setup)
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
    
    // Only benchmark fast methods that don't require external network calls
    let system_methods = vec![
        "getVersion",
        "getGenesisHash",
        "getSlot",
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

/// Benchmark error handling performance (simplified)
fn bench_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let port = get_rpc_benchmark_server_port();
    
    // Initialize server (one time setup)
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
    
    group.finish();
}

criterion_group!(
    benches,
    bench_system_methods,
    bench_error_handling
);
criterion_main!(benches);