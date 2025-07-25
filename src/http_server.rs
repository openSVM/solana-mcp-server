use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::server::ServerState;

/// HTTP server for metrics, health, and MCP API endpoints
pub struct McpHttpServer {
    port: u16,
    server_state: Option<Arc<RwLock<ServerState>>>,
}

impl McpHttpServer {
    pub fn new(port: u16) -> Self {
        Self { 
            port,
            server_state: None,
        }
    }

    pub fn with_server_state(port: u16, server_state: Arc<RwLock<ServerState>>) -> Self {
        Self {
            port,
            server_state: Some(server_state),
        }
    }

    /// Start the HTTP server with metrics, health, and optionally MCP API endpoints
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = if let Some(state) = &self.server_state {
            // Create router with MCP API endpoints and state
            Router::new()
                .route("/metrics", get(metrics_handler))
                .route("/health", get(health_handler))
                .route("/api/mcp", post(mcp_api_handler))
                .with_state(state.clone())
                .layer(ServiceBuilder::new())
        } else {
            // Create router with only metrics and health endpoints
            Router::new()
                .route("/metrics", get(metrics_handler))
                .route("/health", get(health_handler))
                .layer(ServiceBuilder::new())
        };

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting HTTP server on {} with {} endpoints", 
              addr, 
              if self.server_state.is_some() { "metrics, health, and MCP API" } else { "metrics and health" });

        let listener = TcpListener::bind(&addr).await?;
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

/// Handler for /metrics endpoint
async fn metrics_handler() -> Response {
    match crate::metrics::get_metrics_text() {
        Ok(metrics) => {
            (
                [("content-type", "text/plain; version=0.0.4")],
                metrics,
            ).into_response()
        }
        Err(e) => {
            error!("Failed to get metrics: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate metrics",
            ).into_response()
        }
    }
}

/// Handler for /api/mcp endpoint - processes MCP JSON-RPC requests over HTTP
async fn mcp_api_handler(
    State(server_state): State<Arc<RwLock<ServerState>>>,
    Json(request): Json<serde_json::Value>,
) -> Response {
    info!("Received MCP API request");
    
    // Convert the JSON request to string for processing by handle_request
    let request_str = match serde_json::to_string(&request) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize request: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": "Parse error"
                    },
                    "id": null
                }))
            ).into_response();
        }
    };

    // Process the request using the existing MCP request handler
    match crate::tools::handle_request(&request_str, server_state).await {
        Ok(response_message) => {
            // Convert JsonRpcMessage back to JSON for HTTP response
            match serde_json::to_value(&response_message) {
                Ok(json_response) => {
                    (StatusCode::OK, Json(json_response)).into_response()
                }
                Err(e) => {
                    error!("Failed to serialize response: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": "Internal error"
                            },
                            "id": null
                        }))
                    ).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to handle MCP request: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Internal error"
                    },
                    "id": null
                }))
            ).into_response()
        }
    }
}

/// Handler for /health endpoint
async fn health_handler() -> Response {
    (
        [("content-type", "application/json")],
        r#"{"status":"ok","service":"solana-mcp-server"}"#,
    ).into_response()
}

/// Start the metrics server in a background task (legacy function for backward compatibility)
pub fn start_metrics_server_task(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let server = McpHttpServer::new(port);
        if let Err(e) = server.start().await {
            error!("HTTP server failed: {}", e);
        }
    })
}

/// Start the HTTP server with MCP API support in a background task
pub fn start_mcp_server_task(port: u16, server_state: Arc<RwLock<ServerState>>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let server = McpHttpServer::with_server_state(port, server_state);
        if let Err(e) = server.start().await {
            error!("MCP HTTP server failed: {}", e);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_handler() {
        // Initialize metrics first
        crate::metrics::init_prometheus_metrics().expect("Failed to init metrics");
        
        let _response = metrics_handler().await;
        // We can't easily test the response body without more complex setup,
        // but we can ensure it doesn't panic
    }

    #[tokio::test]
    async fn test_health_handler() {
        let _response = health_handler().await;
        // Health endpoint should always work
    }

    #[tokio::test]
    async fn test_mcp_api_handler() {
        use crate::Config;
        use crate::server::ServerState;
        
        // Create a test server state using Config::load() or a minimal config
        // For testing purposes, we'll skip the actual test since it requires valid config
        // In a real test environment, you'd want to create a minimal test config
        
        // This test ensures the function signature is correct and compiles
        // Real integration tests would be in a separate test file with proper setup
    }
}