use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::{json, Value};
use solana_mcp_server::tools;
use std::time::Duration;

/// Mock Solana RPC client for benchmarking without network calls
struct MockRpcClient {
    url: String,
}

impl MockRpcClient {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    // Mock responses for common methods
    async fn get_health(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("ok".to_string())
    }

    async fn get_version(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(json!({
            "solana-core": "1.18.0",
            "feature-set": 2891131721
        }))
    }

    async fn get_balance(
        &self,
        _pubkey: &str,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(1000000000) // 1 SOL in lamports
    }
}

/// Benchmark core JSON-RPC parsing without network overhead
fn bench_json_rpc_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_rpc_parsing");

    let sample_requests = vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "getHealth",
                "arguments": {}
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "getBalance",
                "arguments": {"pubkey": "11111111111111111111111111111112"}
            }
        }),
    ];

    for (i, request) in sample_requests.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("parse_request", i),
            request,
            |b, req| {
                b.iter(|| {
                    let json_str = black_box(req.to_string());
                    let parsed: Value = black_box(serde_json::from_str(&json_str).unwrap());
                    black_box(parsed)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark JSON serialization performance
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    let sample_responses = vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "serverInfo": {"name": "solana-mcp-server", "version": "1.1.0"}
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "tools": (0..90).map(|i| json!({
                    "name": format!("method_{}", i),
                    "description": "Sample RPC method",
                    "inputSchema": {}
                })).collect::<Vec<_>>()
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "content": [{
                    "type": "text",
                    "text": "Health: ok"
                }]
            }
        }),
    ];

    for (i, response) in sample_responses.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("serialize_response", i),
            response,
            |b, resp| {
                b.iter(|| {
                    let serialized = black_box(serde_json::to_string(resp).unwrap());
                    black_box(serialized)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark MCP protocol method routing
fn bench_mcp_method_routing(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcp_method_routing");

    let methods = vec![
        "initialize",
        "tools/list",
        "tools/call",
        "notifications/initialized",
        "notifications/cancelled",
    ];

    for method in methods {
        group.bench_function(method, |b| {
            b.iter(|| {
                let request = black_box(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": method
                }));
                
                // Simulate method routing logic
                let method_name = request["method"].as_str().unwrap();
                let is_tools_call = method_name == "tools/call";
                let is_notification = method_name.starts_with("notifications/");
                let requires_params = method_name == "initialize" || is_tools_call;
                
                black_box((is_tools_call, is_notification, requires_params))
            })
        });
    }

    group.finish();
}

/// Benchmark tool name lookup performance
fn bench_tool_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_lookup");

    // Sample of common Solana RPC methods
    let tool_names = vec![
        "getHealth",
        "getVersion",
        "getBalance",
        "getAccountInfo",
        "sendTransaction",
        "getBlock",
        "getSlot",
        "getEpochInfo",
        "getTokenAccountBalance",
        "getConfirmedBlock",
    ];

    // Simulate a tool registry lookup
    let tool_registry: std::collections::HashMap<String, bool> = tool_names
        .iter()
        .map(|name| (name.to_string(), true))
        .collect();

    for tool_name in &tool_names {
        group.bench_function(tool_name, |b| {
            b.iter(|| {
                let name = black_box(*tool_name);
                let found = black_box(tool_registry.contains_key(name));
                black_box(found)
            })
        });
    }

    group.finish();
}

/// Benchmark parameter validation
fn bench_parameter_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("parameter_validation");

    let test_cases = vec![
        ("pubkey_validation", json!({"pubkey": "11111111111111111111111111111112"})),
        ("commitment_validation", json!({"commitment": "confirmed"})),
        ("encoding_validation", json!({"encoding": "base64"})),
        ("complex_params", json!({
            "pubkey": "11111111111111111111111111111112",
            "commitment": "finalized",
            "encoding": "jsonParsed",
            "dataSlice": {"offset": 0, "length": 100}
        })),
    ];

    for (name, params) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let p = black_box(&params);
                
                // Simulate parameter validation
                let has_pubkey = p.get("pubkey").is_some();
                let has_commitment = p.get("commitment").is_some();
                let has_encoding = p.get("encoding").is_some();
                let param_count = p.as_object().map(|o| o.len()).unwrap_or(0);
                
                black_box((has_pubkey, has_commitment, has_encoding, param_count))
            })
        });
    }

    group.finish();
}

/// Benchmark response formatting
fn bench_response_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_formatting");

    let sample_data = vec![
        ("simple_text", "Health: ok"),
        ("balance_result", "Balance: 1000000000 lamports (1.0 SOL)"),
        ("account_info", "Account found: owner=11111111111111111111111111111112, lamports=1000000000, data_length=0"),
        ("error_response", "Error: Invalid public key format"),
    ];

    for (name, data) in sample_data {
        group.bench_function(name, |b| {
            b.iter(|| {
                let text = black_box(data);
                let response = black_box(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": text
                        }]
                    }
                }));
                black_box(response)
            })
        });
    }

    group.finish();
}

/// Benchmark concurrent request handling simulation
fn bench_concurrent_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_simulation");
    
    // Configure smaller sample sizes for concurrent tests
    group.sample_size(50);

    for concurrency in [1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::new("request_processing", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    let tasks: Vec<_> = (0..concurrency)
                        .map(|i| {
                            // Simulate processing a request
                            let request = black_box(json!({
                                "jsonrpc": "2.0",
                                "id": i,
                                "method": "tools/call",
                                "params": {
                                    "name": "getHealth",
                                    "arguments": {}
                                }
                            }));
                            
                            // Simulate request processing time
                            let method = request["params"]["name"].as_str().unwrap();
                            let response = json!({
                                "jsonrpc": "2.0",
                                "id": i,
                                "result": {
                                    "content": [{
                                        "type": "text",
                                        "text": format!("{}: ok", method)
                                    }]
                                }
                            });
                            
                            black_box(response)
                        })
                        .collect();
                    
                    black_box(tasks)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    group.bench_function("string_allocation", |b| {
        b.iter(|| {
            let requests: Vec<String> = (0..100)
                .map(|i| {
                    black_box(format!(
                        r#"{{"jsonrpc":"2.0","id":{},"method":"tools/call","params":{{"name":"getHealth","arguments":{{}}}}}}"#,
                        i
                    ))
                })
                .collect();
            black_box(requests)
        })
    });

    group.bench_function("json_allocation", |b| {
        b.iter(|| {
            let requests: Vec<Value> = (0..100)
                .map(|i| {
                    black_box(json!({
                        "jsonrpc": "2.0",
                        "id": i,
                        "method": "tools/call",
                        "params": {
                            "name": "getHealth",
                            "arguments": {}
                        }
                    }))
                })
                .collect();
            black_box(requests)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_rpc_parsing,
    bench_json_serialization,
    bench_mcp_method_routing,
    bench_tool_lookup,
    bench_parameter_validation,
    bench_response_formatting,
    bench_concurrent_simulation,
    bench_memory_allocation
);
criterion_main!(benches);