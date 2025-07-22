use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::{info, error};

/// HTTP server for metrics and health endpoints
pub struct MetricsServer {
    port: u16,
}

impl MetricsServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Start the metrics HTTP server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler))
            .layer(ServiceBuilder::new());

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting metrics server on {}", addr);

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

/// Handler for /health endpoint
async fn health_handler() -> Response {
    (
        [("content-type", "application/json")],
        r#"{"status":"ok","service":"solana-mcp-server"}"#,
    ).into_response()
}

/// Start the metrics server in a background task
pub fn start_metrics_server_task(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let server = MetricsServer::new(port);
        if let Err(e) = server.start().await {
            error!("Metrics server failed: {}", e);
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
}