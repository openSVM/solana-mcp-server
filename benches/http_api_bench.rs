use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::{json, Value};
use solana_mcp_server::{Config, ServerState, start_mcp_server_task};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::runtime::Runtime;

/// Setup test server for benchmarking
async fn setup_benchmark_server() -> Result<(tokio::task::JoinHandle<()>, u16), Box<dyn std::error::Error + Send + Sync>> {
    // Use a fixed port for benchmarks to avoid conflicts
    let port = 9001;
    
    // Load configuration
    let config = Config::load().map_err(|e| format!("Failed to load config: {e}"))?;
    
    // Create server state
    let server_state = ServerState::new(config);
    let state = Arc::new(RwLock::new(server_state));
    
    // Start HTTP server with MCP API
    let handle = start_mcp_server_task(port, state);
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    Ok((handle, port))
}

/// Helper function to make HTTP requests for benchmarking
async fn make_benchmark_request(request: Value, port: u16) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
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

/// Benchmark MCP protocol initialization
fn bench_mcp_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup server once for all benchmark iterations
    let (_handle, port) = rt.block_on(async {
        setup_benchmark_server().await.expect("Failed to setup server")
    });
    
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "benchmark-client", "version": "1.0.0"}
        }
    });
    
    c.bench_function("mcp_initialize", |b| {
        b.to_async(&rt).iter(|| async {
            let result = make_benchmark_request(black_box(initialize_request.clone()), port).await;
            black_box(result)
        })
    });
}

/// Benchmark tools list retrieval
fn bench_tools_list(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize first
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "benchmark-client", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_benchmark_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    
    c.bench_function("tools_list", |b| {
        b.to_async(&rt).iter(|| async {
            let result = make_benchmark_request(black_box(tools_request.clone()), port).await;
            black_box(result)
        })
    });
}

/// Benchmark different RPC tool calls
fn bench_rpc_tool_calls(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize first
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "benchmark-client", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_benchmark_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let mut group = c.benchmark_group("rpc_tool_calls");
    
    // Benchmark simple methods
    let simple_methods = vec![
        ("getHealth", json!({})),
        ("getVersion", json!({})),
        ("getGenesisHash", json!({})),
        ("minimumLedgerSlot", json!({})),
    ];
    
    for (method_name, params) in simple_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": method_name,
                "arguments": params
            }
        });
        
        group.bench_with_input(BenchmarkId::new("simple", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let result = make_benchmark_request(black_box(req.clone()), port).await;
                black_box(result)
            })
        });
    }
    
    // Benchmark methods with parameters
    let param_methods = vec![
        ("getBalance", json!({"pubkey": "11111111111111111111111111111112"})),
        ("getAccountInfo", json!({"pubkey": "11111111111111111111111111111112"})),
        ("getSlot", json!("{}")),
    ];
    
    for (method_name, params) in param_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": method_name,
                "arguments": params
            }
        });
        
        group.bench_with_input(BenchmarkId::new("with_params", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let result = make_benchmark_request(black_box(req.clone()), port).await;
                black_box(result)
            })
        });
    }
    
    group.finish();
}

/// Benchmark concurrent requests
fn bench_concurrent_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_benchmark_server().await.expect("Failed to setup server")
    });
    
    // Initialize first
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "benchmark-client", "version": "1.0.0"}
        }
    });
    
    rt.block_on(async {
        make_benchmark_request(initialize_request, port).await.expect("Initialize failed");
    });
    
    let mut group = c.benchmark_group("concurrent_requests");
    
    for concurrency in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::new("getHealth", concurrency), concurrency, |b, &concurrency| {
            b.to_async(&rt).iter(|| async {
                let request = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "getHealth",
                        "arguments": {}
                    }
                });
                
                let tasks: Vec<_> = (0..concurrency)
                    .map(|_| {
                        let req = request.clone();
                        tokio::spawn(async move {
                            make_benchmark_request(req, port).await
                        })
                    })
                    .collect();
                
                let results = futures_util::future::join_all(tasks).await;
                black_box(results)
            })
        });
    }
    
    group.finish();
}

/// Benchmark health endpoint
fn bench_health_endpoint(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_benchmark_server().await.expect("Failed to setup server")
    });
    
    c.bench_function("health_endpoint", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client
                .get(format!("http://localhost:{port}/health"))
                .send()
                .await
                .expect("Health request failed");
            black_box(response.text().await)
        })
    });
}

/// Benchmark metrics endpoint
fn bench_metrics_endpoint(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_benchmark_server().await.expect("Failed to setup server")
    });
    
    c.bench_function("metrics_endpoint", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let response = client
                .get(format!("http://localhost:{port}/metrics"))
                .send()
                .await
                .expect("Metrics request failed");
            black_box(response.text().await)
        })
    });
}

criterion_group!(
    benches,
    bench_mcp_initialization,
    bench_tools_list,
    bench_rpc_tool_calls,
    bench_concurrent_requests,
    bench_health_endpoint,
    bench_metrics_endpoint
);
criterion_main!(benches);