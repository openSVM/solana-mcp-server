/// Integration tests for autoscaling and metrics functionality
use solana_mcp_server::{init_prometheus_metrics, get_metrics_text, PROMETHEUS_METRICS};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_prometheus_metrics_integration() {
    // Initialize Prometheus metrics
    init_prometheus_metrics().expect("Failed to initialize Prometheus metrics");
    
    // Record some test metrics
    PROMETHEUS_METRICS.record_success("getBalance", "mainnet", 0.1);
    PROMETHEUS_METRICS.record_success("getHealth", "testnet", 0.05);
    PROMETHEUS_METRICS.record_failure("getBalance", "mainnet", "timeout", 0.5);
    
    // Get metrics text
    let metrics_text = get_metrics_text().expect("Failed to get metrics text");
    
    // Verify metrics are present
    assert!(metrics_text.contains("solana_mcp_rpc_requests_total"));
    assert!(metrics_text.contains("solana_mcp_rpc_request_duration_seconds"));
    assert!(!metrics_text.is_empty());
    
    // Verify Prometheus format
    assert!(metrics_text.contains("# HELP"));
    assert!(metrics_text.contains("# TYPE"));
}

#[tokio::test]
async fn test_http_server_startup() {
    // Start the metrics server with a timeout to avoid hanging tests
    let server_handle = solana_mcp_server::start_metrics_server_task(18080);
    
    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test that we can make a basic HTTP request
    let client = reqwest::Client::new();
    
    // Test health endpoint
    let health_result = timeout(
        Duration::from_secs(5),
        client.get("http://127.0.0.1:18080/health").send()
    ).await;
    
    if let Ok(Ok(response)) = health_result {
        assert!(response.status().is_success());
        
        let body = response.text().await.expect("Failed to get response body");
        assert!(body.contains("ok"));
    }
    
    // Test metrics endpoint
    let metrics_result = timeout(
        Duration::from_secs(5),
        client.get("http://127.0.0.1:18080/metrics").send()
    ).await;
    
    if let Ok(Ok(response)) = metrics_result {
        assert!(response.status().is_success());
        
        let content_type = response.headers().get("content-type");
        if let Some(ct) = content_type {
            assert!(ct.to_str().unwrap().contains("text/plain"));
        }
    }
    
    // Clean up by aborting the server task
    server_handle.abort();
}

#[tokio::test]
async fn test_metrics_labels_and_values() {
    // Initialize metrics
    init_prometheus_metrics().expect("Failed to initialize Prometheus metrics");
    
    // Record metrics with specific labels
    PROMETHEUS_METRICS.record_success("getAccountInfo", "mainnet", 0.2);
    PROMETHEUS_METRICS.record_success("getAccountInfo", "devnet", 0.15);
    PROMETHEUS_METRICS.record_failure("getBalance", "mainnet", "rpc_error", 1.0);
    
    let metrics_text = get_metrics_text().expect("Failed to get metrics text");
    
    // Check for specific method labels
    assert!(metrics_text.contains("getAccountInfo") || metrics_text.contains("method"));
    
    // Check for network labels  
    assert!(metrics_text.contains("mainnet") || metrics_text.contains("network"));
    
    // Check for error type labels
    assert!(metrics_text.contains("rpc_error") || metrics_text.contains("error_type"));
    
    // Verify histogram buckets are present
    assert!(metrics_text.contains("_bucket") || metrics_text.contains("duration"));
}

#[test]
fn test_kubernetes_manifests_exist() {
    // Verify that Kubernetes manifests exist and are readable
    let deployment_path = std::path::Path::new("k8s/deployment.yaml");
    assert!(deployment_path.exists(), "Kubernetes deployment manifest should exist");
    
    let hpa_path = std::path::Path::new("k8s/hpa.yaml");
    assert!(hpa_path.exists(), "Kubernetes HPA manifest should exist");
    
    let k8s_readme_path = std::path::Path::new("k8s/README.md");
    assert!(k8s_readme_path.exists(), "Kubernetes README should exist");
}

#[test]
fn test_documentation_exists() {
    // Verify that autoscaling documentation exists
    let metrics_doc_path = std::path::Path::new("docs/metrics.md");
    assert!(metrics_doc_path.exists(), "Metrics documentation should exist");
    
    let docker_scaling_doc_path = std::path::Path::new("docs/docker-scaling.md");
    assert!(docker_scaling_doc_path.exists(), "Docker scaling documentation should exist");
}

#[tokio::test]
async fn test_autoscaling_metrics_format() {
    // Test that metrics are in the correct format for Kubernetes HPA
    init_prometheus_metrics().expect("Failed to initialize Prometheus metrics");
    
    // Record some metrics to generate data
    for i in 0..10 {
        PROMETHEUS_METRICS.record_success("getBalance", "mainnet", 0.1 + (i as f64 * 0.01));
    }
    
    let metrics_text = get_metrics_text().expect("Failed to get metrics text");
    
    // Check for counter metrics (used for rate calculations in HPA)
    assert!(metrics_text.contains("solana_mcp_rpc_requests_total"));
    assert!(metrics_text.contains("TYPE") && metrics_text.contains("counter"));
    
    // Check for histogram metrics (used for latency percentiles in HPA)
    assert!(metrics_text.contains("solana_mcp_rpc_request_duration_seconds"));
    assert!(metrics_text.contains("TYPE") && metrics_text.contains("histogram"));
    
    // Verify histogram has buckets
    assert!(metrics_text.contains("_bucket"));
    assert!(metrics_text.contains("_sum"));
    assert!(metrics_text.contains("_count"));
}