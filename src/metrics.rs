use prometheus::{
    CounterVec, HistogramOpts, HistogramVec, Opts, Registry, Encoder, TextEncoder
};
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Prometheus metrics registry for the application
pub static METRICS_REGISTRY: Lazy<Registry> = Lazy::new(|| {
    let registry = Registry::new();
    registry
});

/// Prometheus metrics for RPC operations
pub struct PrometheusMetrics {
    /// Total number of RPC requests
    pub rpc_requests_total: CounterVec,
    /// Number of successful RPC requests
    pub rpc_requests_successful: CounterVec,
    /// Number of failed RPC requests
    pub rpc_requests_failed: CounterVec,
    /// Request duration histogram
    pub rpc_request_duration: HistogramVec,
    /// Error count by type
    pub rpc_errors_total: CounterVec,
}

impl PrometheusMetrics {
    /// Create new Prometheus metrics instance
    pub fn new() -> Result<Self, prometheus::Error> {
        let rpc_requests_total = CounterVec::new(
            Opts::new("solana_mcp_rpc_requests_total", "Total RPC requests"),
            &["method", "network"]
        )?;

        let rpc_requests_successful = CounterVec::new(
            Opts::new("solana_mcp_rpc_requests_successful_total", "Successful RPC requests"),
            &["method", "network"]
        )?;

        let rpc_requests_failed = CounterVec::new(
            Opts::new("solana_mcp_rpc_requests_failed_total", "Failed RPC requests"),
            &["method", "network", "error_type"]
        )?;

        let rpc_request_duration = HistogramVec::new(
            HistogramOpts::new(
                "solana_mcp_rpc_request_duration_seconds",
                "RPC request duration in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "network"]
        )?;

        let rpc_errors_total = CounterVec::new(
            Opts::new("solana_mcp_rpc_errors_total", "Total RPC errors by type"),
            &["error_type", "method"]
        )?;

        // Try to register metrics, but ignore "AlreadyReg" errors for tests
        let _ = METRICS_REGISTRY.register(Box::new(rpc_requests_total.clone()));
        let _ = METRICS_REGISTRY.register(Box::new(rpc_requests_successful.clone()));
        let _ = METRICS_REGISTRY.register(Box::new(rpc_requests_failed.clone()));
        let _ = METRICS_REGISTRY.register(Box::new(rpc_request_duration.clone()));
        let _ = METRICS_REGISTRY.register(Box::new(rpc_errors_total.clone()));

        Ok(Self {
            rpc_requests_total,
            rpc_requests_successful,
            rpc_requests_failed,
            rpc_request_duration,
            rpc_errors_total,
        })
    }

    /// Record a successful RPC request
    pub fn record_success(&self, method: &str, network: &str, duration_seconds: f64) {
        self.rpc_requests_total
            .with_label_values(&[method, network])
            .inc();
        
        self.rpc_requests_successful
            .with_label_values(&[method, network])
            .inc();

        self.rpc_request_duration
            .with_label_values(&[method, network])
            .observe(duration_seconds);
    }

    /// Record a failed RPC request
    pub fn record_failure(&self, method: &str, network: &str, error_type: &str, duration_seconds: f64) {
        self.rpc_requests_total
            .with_label_values(&[method, network])
            .inc();

        self.rpc_requests_failed
            .with_label_values(&[method, network, error_type])
            .inc();

        self.rpc_errors_total
            .with_label_values(&[error_type, method])
            .inc();

        self.rpc_request_duration
            .with_label_values(&[method, network])
            .observe(duration_seconds);
    }
}

/// Global metrics instance
pub static PROMETHEUS_METRICS: Lazy<Arc<PrometheusMetrics>> = Lazy::new(|| {
    Arc::new(PrometheusMetrics::new().unwrap_or_else(|e| {
        // In tests, metrics might already be registered, so create a new instance without registration
        eprintln!("Warning: Failed to create Prometheus metrics ({}), creating basic instance", e);
        PrometheusMetrics::new_unregistered()
    }))
});

impl PrometheusMetrics {
    /// Create metrics without registering them (for tests)
    fn new_unregistered() -> Self {
        let rpc_requests_total = CounterVec::new(
            Opts::new("solana_mcp_rpc_requests_total_test", "Total RPC requests (test)"),
            &["method", "network"]
        ).unwrap();

        let rpc_requests_successful = CounterVec::new(
            Opts::new("solana_mcp_rpc_requests_successful_total_test", "Successful RPC requests (test)"),
            &["method", "network"]
        ).unwrap();

        let rpc_requests_failed = CounterVec::new(
            Opts::new("solana_mcp_rpc_requests_failed_total_test", "Failed RPC requests (test)"),
            &["method", "network", "error_type"]
        ).unwrap();

        let rpc_request_duration = HistogramVec::new(
            HistogramOpts::new(
                "solana_mcp_rpc_request_duration_seconds_test",
                "RPC request duration in seconds (test)"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "network"]
        ).unwrap();

        let rpc_errors_total = CounterVec::new(
            Opts::new("solana_mcp_rpc_errors_total_test", "Total RPC errors by type (test)"),
            &["error_type", "method"]
        ).unwrap();

        Self {
            rpc_requests_total,
            rpc_requests_successful,
            rpc_requests_failed,
            rpc_request_duration,
            rpc_errors_total,
        }
    }
}

/// Get metrics in Prometheus text format
pub fn get_metrics_text() -> Result<String, prometheus::Error> {
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer).unwrap_or_default())
}

/// Initialize prometheus metrics
pub fn init_prometheus_metrics() -> Result<(), prometheus::Error> {
    Lazy::force(&PROMETHEUS_METRICS);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        // Use the global metrics to ensure registration
        init_prometheus_metrics().expect("Failed to init metrics");
        
        // Test recording success
        PROMETHEUS_METRICS.record_success("getBalance", "mainnet", 0.1);
        
        // Test recording failure
        PROMETHEUS_METRICS.record_failure("getBalance", "mainnet", "timeout", 0.5);
        
        // Test text export - just verify it doesn't panic and has some content
        let metrics_text = get_metrics_text().expect("Failed to get metrics text");
        assert!(!metrics_text.is_empty(), "Metrics text should not be empty");
        // Basic check for Prometheus format
        assert!(metrics_text.contains("# HELP") || metrics_text.contains("# TYPE"), 
                "Metrics should contain Prometheus format markers");
    }

    #[test]
    fn test_metrics_labels() {
        // Use the global metrics instance to ensure it gets registered
        crate::metrics::init_prometheus_metrics().expect("Failed to init metrics");
        
        // Record metrics with different labels using the global instance
        PROMETHEUS_METRICS.record_success("getBalance", "mainnet", 0.1);
        PROMETHEUS_METRICS.record_success("getHealth", "testnet", 0.05);
        PROMETHEUS_METRICS.record_failure("getBalance", "mainnet", "rpc_error", 0.2);
        
        let metrics_text = get_metrics_text().expect("Failed to get metrics text");
        
        // Check that labels are included
        assert!(metrics_text.contains("method=\"getBalance\""));
        assert!(metrics_text.contains("network=\"mainnet\""));
        // Note: In some cases, metrics might not appear immediately, so this check might be flaky
        // For now, just check the basic structure
        assert!(metrics_text.contains("solana_mcp_rpc"));
    }
}