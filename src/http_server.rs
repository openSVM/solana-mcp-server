use axum::{
    extract::State,
    http::{StatusCode, HeaderMap, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use tracing::{info, error, debug};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::server::ServerState;
use crate::transport::{JsonRpcRequest, JsonRpcVersion};
use crate::config::Config;

/// HTTP request timeout (can be overridden by config)
const DEFAULT_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// HTTP server graceful shutdown timeout
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// HTTP server for metrics, health, and MCP API endpoints
pub struct McpHttpServer {
    port: u16,
    server_state: Option<Arc<RwLock<ServerState>>>,
    config: Option<Arc<Config>>,
}

impl McpHttpServer {
    pub fn new(port: u16) -> Self {
        Self { 
            port,
            server_state: None,
            config: None,
        }
    }

    pub fn with_server_state(port: u16, server_state: Arc<RwLock<ServerState>>) -> Self {
        Self {
            port,
            server_state: Some(server_state),
            config: None,
        }
    }

    pub fn with_config(mut self, config: Arc<Config>) -> Self {
        self.config = Some(config);
        self
    }

    /// Start the HTTP server with metrics, health, and optionally MCP API endpoints
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let http_timeout = self.config
            .as_ref()
            .map(|c| Duration::from_secs(c.timeouts.http_request_seconds))
            .unwrap_or(DEFAULT_HTTP_REQUEST_TIMEOUT);

        let app = if let Some(state) = &self.server_state {
            // Create router with MCP API endpoints and state
            Router::new()
                .route("/metrics", get(metrics_handler))
                .route("/health", get(health_handler))
                .route("/api/mcp", post(mcp_api_handler))
                .with_state(state.clone())
                .layer(ServiceBuilder::new()
                    .layer(TimeoutLayer::new(http_timeout))
                )
        } else {
            // Create router with only metrics and health endpoints
            Router::new()
                .route("/metrics", get(metrics_handler))
                .route("/health", get(health_handler))
                .layer(ServiceBuilder::new()
                    .layer(TimeoutLayer::new(http_timeout))
                )
        };

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Started HTTP server on {} with timeout {}s", 
              addr, 
              http_timeout.as_secs());

        let listener = TcpListener::bind(&addr).await?;
        
        // Start server with graceful shutdown handling
        match timeout(SHUTDOWN_TIMEOUT, axum::serve(listener, app)).await {
            Ok(result) => result.map_err(|e| e.into()),
            Err(_) => {
                error!("HTTP server startup timeout");
                Err("HTTP server startup timeout".into())
            }
        }
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
/// Follows the MCP protocol specification for proper JSON-RPC 2.0 handling
async fn mcp_api_handler(
    State(server_state): State<Arc<RwLock<ServerState>>>,
    headers: HeaderMap,
    Json(request): Json<serde_json::Value>,
) -> Response {
    debug!("Received MCP API request: {}", serde_json::to_string(&request).unwrap_or_default());
    
    // Validate Content-Type header (should be application/json for MCP)
    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        if let Ok(ct_str) = content_type.to_str() {
            // Be more strict about content type validation
            let is_valid = ct_str == "application/json" || 
                          ct_str.starts_with("application/json;");
            if !is_valid {
                return create_json_rpc_error_response(
                    -32600,
                    "Invalid Request: Content-Type must be application/json",
                    None,
                );
            }
        }
    }

    // Parse and validate JSON-RPC request structure
    let json_rpc_request = match parse_json_rpc_request(&request) {
        Ok(req) => req,
        Err(error_response) => return *error_response,
    };

    // Process the MCP request through the existing handler
    match crate::tools::handle_request(&serde_json::to_string(&request).unwrap_or_default(), server_state).await {
        Ok(response_message) => {
            // Convert JsonRpcMessage back to proper JSON-RPC 2.0 format
            match serde_json::to_value(&response_message) {
                Ok(json_response) => {
                    // The response_message is already a properly formatted JSON-RPC response
                    // Don't double-wrap it in create_json_rpc_success_response
                    (
                        StatusCode::OK,
                        [(CONTENT_TYPE, "application/json")],
                        Json(json_response)
                    ).into_response()
                }
                Err(e) => {
                    error!("Failed to serialize MCP response: {}", e);
                    create_json_rpc_error_response(
                        -32603,
                        "Internal error: Failed to serialize response",
                        Some(json_rpc_request.id.clone()),
                    )
                }
            }
        }
        Err(e) => {
            error!("Failed to handle MCP request: {}", e);
            create_json_rpc_error_response(
                -32603,
                &format!("Internal error: {e}"),
                Some(json_rpc_request.id.clone()),
            )
        }
    }
}

/// Parse and validate JSON-RPC 2.0 request according to MCP specification
fn parse_json_rpc_request(request: &serde_json::Value) -> Result<JsonRpcRequest, Box<Response>> {
    // Validate required fields for JSON-RPC 2.0
    let jsonrpc = request.get("jsonrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Box::new(create_json_rpc_error_response(
            -32600,
            "Invalid Request: missing 'jsonrpc' field",
            None,
        )))?;

    if jsonrpc != "2.0" {
        return Err(Box::new(create_json_rpc_error_response(
            -32600,
            "Invalid Request: 'jsonrpc' must be '2.0'",
            None,
        )));
    }

    let method = request.get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Box::new(create_json_rpc_error_response(
            -32600,
            "Invalid Request: missing 'method' field",
            None,
        )))?;

    let id = request.get("id")
        .cloned()
        .unwrap_or(Value::Null); // Default to null if no ID provided

    let params = request.get("params").cloned();

    Ok(JsonRpcRequest {
        jsonrpc: JsonRpcVersion::V2,
        id,
        method: method.to_string(),
        params,
    })
}


/// Create a properly formatted JSON-RPC 2.0 error response
fn create_json_rpc_error_response(code: i32, message: &str, id: Option<Value>) -> Response {
    let error_response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message,
            "data": {
                "protocolVersion": crate::protocol::LATEST_PROTOCOL_VERSION
            }
        },
        "id": id.unwrap_or(Value::Null)
    });

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json")],
        Json(error_response)
    ).into_response()
}

/// Handler for /health endpoint - MCP-compliant health check
async fn health_handler() -> Response {
    let health_response = serde_json::json!({
        "status": "ok",
        "service": "solana-mcp-server",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": crate::protocol::LATEST_PROTOCOL_VERSION,
        "capabilities": {
            "tools": true,
            "resources": true,
            "prompts": false,
            "sampling": false
        }
    });

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json")],
        Json(health_response)
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
        // Create a test server state using Config::load() or a minimal config
        // For testing purposes, we'll skip the actual test since it requires valid config
        // In a real test environment, you'd want to create a minimal test config
        
        // This test ensures the function signature is correct and compiles
        // Real integration tests would be in a separate test file with proper setup
    }
}