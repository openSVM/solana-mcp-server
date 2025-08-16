use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;
use solana_mcp_server::{Config, start_websocket_server_task};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::OnceLock;

static WS_BENCHMARK_SERVER: OnceLock<u16> = OnceLock::new();

/// Setup shared WebSocket server for benchmarking
async fn setup_shared_websocket_benchmark_server() -> Result<(tokio::task::JoinHandle<()>, u16), Box<dyn std::error::Error + Send + Sync>> {
    let port = 9003;
    
    let mut config = Config::load().map_err(|e| format!("Failed to load config: {e}"))?;
    // Use mock RPC endpoint for benchmarks
    config.rpc_url = "http://localhost:8899".to_string();
    config.timeouts.websocket_connection_seconds = 5;
    config.timeouts.websocket_message_seconds = 5;
    
    let config_arc = Arc::new(config);
    
    let handle = start_websocket_server_task(port, config_arc);
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    WS_BENCHMARK_SERVER.set(port).ok();
    
    Ok((handle, port))
}

fn get_websocket_benchmark_server_port() -> u16 {
    if let Some(port) = WS_BENCHMARK_SERVER.get() {
        *port
    } else {
        let rt = Runtime::new().unwrap();
        let (_handle, port) = rt.block_on(async {
            setup_shared_websocket_benchmark_server().await.expect("Failed to setup shared WebSocket server")
        });
        port
    }
}

/// Helper to establish WebSocket connection (with connection caching)
async fn connect_websocket(port: u16) -> Result<(tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("ws://localhost:{port}");
    let (ws_stream, response) = connect_async(&url).await?;
    Ok((ws_stream, response))
}

/// Benchmark WebSocket connection establishment
fn bench_websocket_connection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let port = get_websocket_benchmark_server_port();
    
    c.bench_function("websocket_connection", |b| {
        b.to_async(&rt).iter(|| async {
            let result = connect_websocket(port).await;
            black_box(result)
        })
    });
}

/// Benchmark basic WebSocket subscription (simplified)
fn bench_websocket_subscriptions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let port = get_websocket_benchmark_server_port();
    
    let mut group = c.benchmark_group("websocket_subscriptions");
    
    // Only test one simple subscription method to avoid timeouts
    let subscription_methods = vec![
        ("slotSubscribe", json!({})),
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
                
                // Wait for response with timeout
                let response = tokio::time::timeout(
                    Duration::from_millis(1000),
                    ws_stream.next()
                ).await;
                
                black_box(response)
            })
        });
    }
    
    group.finish();
}

/// Benchmark WebSocket error handling (simplified)
fn bench_websocket_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let port = get_websocket_benchmark_server_port();
    
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
            
            // Wait for error response with timeout
            let response = tokio::time::timeout(
                Duration::from_millis(1000),
                ws_stream.next()
            ).await;
            
            black_box(response)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_websocket_connection,
    bench_websocket_subscriptions,
    bench_websocket_error_handling
);
criterion_main!(benches);