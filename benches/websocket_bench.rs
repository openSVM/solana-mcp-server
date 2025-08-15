use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;
use solana_mcp_server::{Config, start_websocket_server_task};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};

/// Setup WebSocket server for benchmarking
async fn setup_websocket_benchmark_server() -> Result<(tokio::task::JoinHandle<()>, u16), Box<dyn std::error::Error + Send + Sync>> {
    let port = 9003;
    
    let config = Config::load().map_err(|e| format!("Failed to load config: {e}"))?;
    let config_arc = Arc::new(config);
    
    let handle = start_websocket_server_task(port, config_arc);
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    Ok((handle, port))
}

/// Helper to establish WebSocket connection
async fn connect_websocket(port: u16) -> Result<(tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("ws://localhost:{port}");
    let (ws_stream, response) = connect_async(&url).await?;
    Ok((ws_stream, response))
}

/// Benchmark WebSocket connection establishment
fn bench_websocket_connection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_websocket_benchmark_server().await.expect("Failed to setup WebSocket server")
    });
    
    c.bench_function("websocket_connection", |b| {
        b.to_async(&rt).iter(|| async {
            let result = connect_websocket(port).await;
            black_box(result)
        })
    });
}

/// Benchmark WebSocket subscription methods
fn bench_websocket_subscriptions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_websocket_benchmark_server().await.expect("Failed to setup WebSocket server")
    });
    
    let mut group = c.benchmark_group("websocket_subscriptions");
    
    let subscription_methods = vec![
        ("accountSubscribe", json!({"pubkey": "11111111111111111111111111111112", "encoding": "base64"})),
        ("slotSubscribe", json!({})),
        ("rootSubscribe", json!({})),
        ("blockSubscribe", json!({"filter": "all", "encoding": "json"})),
        ("programSubscribe", json!({"pubkey": "11111111111111111111111111111112", "encoding": "base64"})),
        ("signatureSubscribe", json!({"signature": "5VERv8NMvQEK24H6JY9qrE4m8W8PUaH9wQxmTnneJbUY3v8j7JY5xJmwXxDWVqsR6YL1bCRjgWnPGc8LxrXZtCbU", "commitment": "finalized"})),
        ("logsSubscribe", json!({"filter": "all"})),
        ("voteSubscribe", json!({})),
        ("slotsUpdatesSubscribe", json!({})),
    ];
    
    for (method_name, params) in subscription_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method_name,
            "params": params
        });
        
        group.bench_with_input(BenchmarkId::new("subscribe", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let (mut ws_stream, _) = connect_websocket(port).await.expect("Failed to connect");
                
                // Send subscription request
                let message = Message::Text(req.to_string().into());
                ws_stream.send(message).await.expect("Failed to send message");
                
                // Wait for response
                if let Some(response) = ws_stream.next().await {
                    black_box(response)
                } else {
                    panic!("No response received")
                }
            })
        });
    }
    
    group.finish();
}

/// Benchmark WebSocket unsubscribe methods
fn bench_websocket_unsubscribe(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_websocket_benchmark_server().await.expect("Failed to setup WebSocket server")
    });
    
    let mut group = c.benchmark_group("websocket_unsubscribe");
    
    let unsubscribe_methods = vec![
        "accountUnsubscribe",
        "slotUnsubscribe", 
        "rootUnsubscribe",
        "blockUnsubscribe",
        "programUnsubscribe",
        "signatureUnsubscribe",
        "logsUnsubscribe",
        "voteUnsubscribe",
        "slotsUpdatesUnsubscribe",
    ];
    
    for method_name in unsubscribe_methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": method_name,
            "params": [1] // Fake subscription ID
        });
        
        group.bench_with_input(BenchmarkId::new("unsubscribe", method_name), &request, |b, req| {
            b.to_async(&rt).iter(|| async {
                let (mut ws_stream, _) = connect_websocket(port).await.expect("Failed to connect");
                
                // Send unsubscribe request
                let message = Message::Text(req.to_string().into());
                ws_stream.send(message).await.expect("Failed to send message");
                
                // Wait for response
                if let Some(response) = ws_stream.next().await {
                    black_box(response)
                } else {
                    panic!("No response received")
                }
            })
        });
    }
    
    group.finish();
}

/// Benchmark WebSocket message throughput
fn bench_websocket_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_websocket_benchmark_server().await.expect("Failed to setup WebSocket server")
    });
    
    let mut group = c.benchmark_group("websocket_throughput");
    
    for message_count in [1, 5, 10, 25].iter() {
        group.bench_with_input(BenchmarkId::new("messages", message_count), message_count, |b, &count| {
            b.to_async(&rt).iter(|| async {
                let (mut ws_stream, _) = connect_websocket(port).await.expect("Failed to connect");
                
                let request = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "slotSubscribe",
                    "params": {}
                });
                
                for i in 0..count {
                    let mut req = request.clone();
                    req["id"] = json!(i + 1);
                    let message = Message::Text(req.to_string().into());
                    ws_stream.send(message).await.expect("Failed to send message");
                }
                
                // Read all responses
                let mut responses = Vec::new();
                for _ in 0..count {
                    if let Some(response) = ws_stream.next().await {
                        responses.push(response);
                    }
                }
                
                black_box(responses)
            })
        });
    }
    
    group.finish();
}

/// Benchmark concurrent WebSocket connections
fn bench_concurrent_connections(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_websocket_benchmark_server().await.expect("Failed to setup WebSocket server")
    });
    
    let mut group = c.benchmark_group("concurrent_connections");
    
    for connection_count in [1, 3, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::new("connections", connection_count), connection_count, |b, &count| {
            b.to_async(&rt).iter(|| async {
                let tasks: Vec<_> = (0..count)
                    .map(|i| {
                        tokio::spawn(async move {
                            let (mut ws_stream, _) = connect_websocket(port).await.expect("Failed to connect");
                            
                            let request = json!({
                                "jsonrpc": "2.0",
                                "id": i + 1,
                                "method": "slotSubscribe",
                                "params": {}
                            });
                            
                            let message = Message::Text(request.to_string().into());
                            ws_stream.send(message).await.expect("Failed to send message");
                            
                            // Wait for response
                            if let Some(response) = ws_stream.next().await {
                                response
                            } else {
                                panic!("No response received")
                            }
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

/// Benchmark WebSocket error handling
fn bench_websocket_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (_handle, port) = rt.block_on(async {
        setup_websocket_benchmark_server().await.expect("Failed to setup WebSocket server")
    });
    
    let mut group = c.benchmark_group("websocket_error_handling");
    
    // Test invalid method
    let invalid_method_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "invalidSubscribe",
        "params": {}
    });
    
    group.bench_function("invalid_method", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut ws_stream, _) = connect_websocket(port).await.expect("Failed to connect");
            
            let message = Message::Text(invalid_method_request.to_string().into());
            ws_stream.send(message).await.expect("Failed to send message");
            
            // Wait for error response
            if let Some(response) = ws_stream.next().await {
                black_box(response)
            } else {
                panic!("No response received")
            }
        })
    });
    
    // Test invalid JSON
    group.bench_function("invalid_json", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut ws_stream, _) = connect_websocket(port).await.expect("Failed to connect");
            
            let message = Message::Text("{invalid json".to_string().into());
            ws_stream.send(message).await.expect("Failed to send message");
            
            // Wait for error response
            if let Some(response) = ws_stream.next().await {
                black_box(response)
            } else {
                panic!("No response received")
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_websocket_connection,
    bench_websocket_subscriptions,
    bench_websocket_unsubscribe,
    bench_websocket_throughput,
    bench_concurrent_connections,
    bench_websocket_error_handling
);
criterion_main!(benches);