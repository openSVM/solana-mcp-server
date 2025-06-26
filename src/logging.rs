use tracing::{info, warn, error, instrument, Span};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt,
    filter::EnvFilter,
};
use uuid::Uuid;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;

/// Metrics collection for monitoring RPC call outcomes
#[derive(Debug, Default)]
pub struct Metrics {
    /// Total number of RPC calls
    pub total_calls: AtomicU64,
    /// Number of successful RPC calls
    pub successful_calls: AtomicU64,
    /// Number of failed RPC calls by error type
    pub failed_calls_by_type: DashMap<String, u64>,
    /// Number of failed RPC calls by method
    pub failed_calls_by_method: DashMap<String, u64>,
    /// Duration histogram buckets (in milliseconds)
    /// Buckets: <10ms, 10-50ms, 50-100ms, 100-500ms, 500-1000ms, >1000ms
    pub duration_buckets: [AtomicU64; 6],
    /// Total duration sum for average calculation
    pub total_duration_ms: AtomicU64,
}

impl Metrics {
    /// Increment total calls counter
    pub fn increment_total_calls(&self) {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment successful calls counter and record duration
    pub fn increment_successful_calls(&self, duration_ms: u64) {
        self.successful_calls.fetch_add(1, Ordering::Relaxed);
        self.record_duration(duration_ms);
    }

    /// Record duration in appropriate histogram bucket
    fn record_duration(&self, duration_ms: u64) {
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        let bucket_index = match duration_ms {
            0..=9 => 0,      // <10ms
            10..=49 => 1,    // 10-50ms
            50..=99 => 2,    // 50-100ms
            100..=499 => 3,  // 100-500ms
            500..=999 => 4,  // 500-1000ms
            _ => 5,          // >1000ms
        };
        
        self.duration_buckets[bucket_index].fetch_add(1, Ordering::Relaxed);
    }

    /// Increment failed calls counter by error type and record duration
    pub fn increment_failed_calls(&self, error_type: &str, method: Option<&str>, duration_ms: u64) {
        // Increment by error type using dashmap for concurrent access
        self.failed_calls_by_type
            .entry(error_type.to_string())
            .and_modify(|e| *e += 1)
            .or_insert(1);
        
        // Increment by method if available
        if let Some(method) = method {
            self.failed_calls_by_method
                .entry(method.to_string())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
        
        // Record duration for failed requests too
        self.record_duration(duration_ms);
    }

    /// Get current metrics as JSON value
    pub fn to_json(&self) -> Value {
        // Convert DashMap to HashMap for JSON serialization
        let failed_by_type: std::collections::HashMap<String, u64> = self.failed_calls_by_type
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
            
        let failed_by_method: std::collections::HashMap<String, u64> = self.failed_calls_by_method
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();

        let total_calls = self.total_calls.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        let avg_duration = if total_calls > 0 { total_duration / total_calls } else { 0 };

        // Collect histogram data
        let histogram = [
            ("0-9ms", self.duration_buckets[0].load(Ordering::Relaxed)),
            ("10-49ms", self.duration_buckets[1].load(Ordering::Relaxed)),
            ("50-99ms", self.duration_buckets[2].load(Ordering::Relaxed)),
            ("100-499ms", self.duration_buckets[3].load(Ordering::Relaxed)),
            ("500-999ms", self.duration_buckets[4].load(Ordering::Relaxed)),
            ("1000ms+", self.duration_buckets[5].load(Ordering::Relaxed)),
        ];
        
        serde_json::json!({
            "total_calls": total_calls,
            "successful_calls": self.successful_calls.load(Ordering::Relaxed),
            "failed_calls_by_type": failed_by_type,
            "failed_calls_by_method": failed_by_method,
            "performance": {
                "avg_duration_ms": avg_duration,
                "total_duration_ms": total_duration,
                "duration_histogram": histogram
            }
        })
    }

    /// Reset all metrics (useful for testing)
    pub fn reset(&self) {
        self.total_calls.store(0, Ordering::Relaxed);
        self.successful_calls.store(0, Ordering::Relaxed);
        self.total_duration_ms.store(0, Ordering::Relaxed);
        for bucket in &self.duration_buckets {
            bucket.store(0, Ordering::Relaxed);
        }
        self.failed_calls_by_type.clear();
        self.failed_calls_by_method.clear();
    }
}

/// Global metrics instance
static METRICS: once_cell::sync::Lazy<Metrics> = once_cell::sync::Lazy::new(|| Metrics::default());

/// Initialize structured logging with tracing
/// 
/// Sets up JSON-formatted structured logging with the tracing crate.
/// Configures appropriate log levels and output format for production use.
/// 
/// # Arguments
/// * `level` - Optional log level filter (defaults to "info")
/// 
/// # Examples
/// ```
/// use solana_mcp_server::logging::init_logging;
/// 
/// // Initialize with default level (info)
/// init_logging(None);
/// 
/// // Initialize with debug level
/// init_logging(Some("debug"));
/// ```
pub fn init_logging(level: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level.unwrap_or("info")))?;

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .json()
                .with_current_span(false)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_target(true)
                .with_line_number(true)
                .with_file(true)
        )
        .try_init()?;

    info!("Structured logging initialized");
    Ok(())
}

/// Log RPC request start with context
#[instrument(skip_all, fields(
    request_id = %request_id,
    method = %method,
    rpc_url = tracing::field::Empty,
    params_summary = tracing::field::Empty
))]
pub fn log_rpc_request_start(
    request_id: Uuid,
    method: &str,
    rpc_url: Option<&str>,
    params_summary: Option<&str>,
) {
    METRICS.increment_total_calls();
    
    let span = Span::current();
    
    if let Some(url) = rpc_url {
        let sanitized_url = crate::validation::sanitize_for_logging(url);
        span.record("rpc_url", &sanitized_url);
    }
    
    if let Some(summary) = params_summary {
        span.record("params_summary", summary);
    }
    
    info!("RPC request started");
}

/// Log RPC request success with context
#[instrument(skip_all, fields(
    request_id = %request_id,
    method = %method,
    duration_ms = tracing::field::Empty,
    result_summary = tracing::field::Empty
))]
pub fn log_rpc_request_success(
    request_id: Uuid,
    method: &str,
    duration_ms: u64,
    result_summary: Option<&str>,
) {
    METRICS.increment_successful_calls(duration_ms);
    
    let span = Span::current();
    span.record("duration_ms", duration_ms);
    
    if let Some(summary) = result_summary {
        span.record("result_summary", summary);
    }
    
    info!("RPC request completed successfully");
}

/// Log RPC request failure with context
#[instrument(skip_all, fields(
    request_id = %request_id,
    method = %method,
    error_type = %error_type,
    duration_ms = tracing::field::Empty,
    error_details = tracing::field::Empty
))]
pub fn log_rpc_request_failure(
    request_id: Uuid,
    method: &str,
    error_type: &str,
    duration_ms: u64,
    error_details: Option<&Value>,
) {
    METRICS.increment_failed_calls(error_type, Some(method), duration_ms);
    
    let span = Span::current();
    span.record("duration_ms", duration_ms);
    
    if let Some(details) = error_details {
        span.record("error_details", &details.to_string());
    }
    
    error!("RPC request failed");
}

/// Log server startup with context
#[instrument(fields(
    protocol_version = %protocol_version,
    rpc_url = tracing::field::Empty,
    svm_networks_count = networks_count
))]
pub fn log_server_startup(
    protocol_version: &str,
    rpc_url: &str,
    networks_count: usize,
) {
    let span = Span::current();
    let sanitized_url = crate::validation::sanitize_for_logging(rpc_url);
    span.record("rpc_url", &sanitized_url);
    
    info!("Solana MCP Server starting");
}

/// Log configuration changes with context
#[instrument(skip_all, fields(
    old_rpc_url = tracing::field::Empty,
    new_rpc_url = tracing::field::Empty,
    networks_changed = networks_changed
))]
pub fn log_config_change(
    old_rpc_url: Option<&str>,
    new_rpc_url: Option<&str>,
    networks_changed: bool,
) {
    let span = Span::current();
    
    if let Some(old_url) = old_rpc_url {
        let sanitized = crate::validation::sanitize_for_logging(old_url);
        span.record("old_rpc_url", &sanitized);
    }
    
    if let Some(new_url) = new_rpc_url {
        let sanitized = crate::validation::sanitize_for_logging(new_url);
        span.record("new_rpc_url", &sanitized);
    }
    
    info!("Configuration updated");
}

/// Log validation errors with context
#[instrument(skip_all, fields(
    request_id = %request_id,
    method = %method,
    parameter = %parameter,
    provided_value = tracing::field::Empty
))]
pub fn log_validation_error(
    request_id: Uuid,
    method: &str,
    parameter: &str,
    provided_value: Option<&str>,
    error_message: &str,
) {
    let span = Span::current();
    
    if let Some(value) = provided_value {
        // Sanitize the provided value to avoid logging sensitive data
        let sanitized = crate::validation::sanitize_for_logging(value);
        span.record("provided_value", &sanitized);
    }
    
    warn!("Validation error: {}", error_message);
}

/// Log network connectivity issues
#[instrument(skip_all, fields(
    request_id = %request_id,
    method = %method,
    endpoint = tracing::field::Empty,
    error_type = %error_type
))]
pub fn log_network_error(
    request_id: Uuid,
    method: &str,
    endpoint: &str,
    error_type: &str,
    error_message: &str,
) {
    let span = Span::current();
    let sanitized_endpoint = crate::validation::sanitize_for_logging(endpoint);
    span.record("endpoint", &sanitized_endpoint);
    
    error!("Network error: {}", error_message);
}

/// Log unexpected server errors with context
#[instrument(skip_all, fields(
    request_id = %request_id,
    method = %method,
    error_type = %error_type
))]
pub fn log_server_error(
    request_id: Uuid,
    method: &str,
    error_type: &str,
    error_message: &str,
) {
    error!("Server error: {}", error_message);
}

/// Create a new request ID for tracing
pub fn new_request_id() -> Uuid {
    Uuid::new_v4()
}

/// Get current metrics
pub fn get_metrics() -> &'static Metrics {
    &METRICS
}

/// Create a parameters summary for logging (sanitized)
pub fn create_params_summary(params: &Value) -> String {
    use crate::validation::sanitization::*;
    
    match params {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys()
                .take(MAX_OBJECT_KEYS_IN_SUMMARY)
                .map(|k| k.to_string())
                .collect();
            let summary = format!("params: [{}]", keys.join(", "));
            if map.len() > MAX_OBJECT_KEYS_IN_SUMMARY {
                format!("{}...({} more)", summary, map.len() - MAX_OBJECT_KEYS_IN_SUMMARY)
            } else {
                summary
            }
        },
        Value::Array(arr) => {
            format!("params: array[{}]", arr.len())
        },
        _ => "params: single_value".to_string(),
    }
}

/// Create a result summary for logging (sanitized)
pub fn create_result_summary(result: &Value) -> String {
    use crate::validation::sanitization::*;
    
    match result {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys()
                .take(MAX_OBJECT_KEYS_IN_SUMMARY)
                .map(|k| k.to_string())
                .collect();
            if map.len() > MAX_OBJECT_KEYS_IN_SUMMARY {
                format!("result: {{{},...({} more)}}", keys.join(", "), map.len() - MAX_OBJECT_KEYS_IN_SUMMARY)
            } else {
                format!("result: {{{}}}", keys.join(", "))
            }
        },
        Value::Array(arr) => {
            format!("result: array[{}]", arr.len())
        },
        Value::String(_) => "result: string".to_string(),
        Value::Number(_) => "result: number".to_string(),
        Value::Bool(_) => "result: boolean".to_string(),
        Value::Null => "result: null".to_string(),
    }
}

/// Macro to reduce repetitive boilerplate around timing and logs for RPC calls
#[macro_export]
macro_rules! log_rpc_call {
    ($method:expr, $client:expr, $async_block:expr) => {{
        let request_id = $crate::logging::new_request_id();
        let start_time = std::time::Instant::now();
        
        $crate::logging::log_rpc_request_start(
            request_id,
            $method,
            Some(&$client.url()),
            None,
        );

        match $async_block.await {
            Ok(result) => {
                let duration = start_time.elapsed().as_millis() as u64;
                
                $crate::logging::log_rpc_request_success(
                    request_id,
                    $method,
                    duration,
                    Some("request completed"),
                );
                
                Ok(result)
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                let error = $crate::error::McpError::from(e)
                    .with_request_id(request_id)
                    .with_method($method)
                    .with_rpc_url(&$client.url());
                
                $crate::logging::log_rpc_request_failure(
                    request_id,
                    $method,
                    &error.error_type(),
                    duration,
                    Some(&error.to_log_value()),
                );
                
                Err(error)
            }
        }
    }};
    ($method:expr, $client:expr, $async_block:expr, $params:expr) => {{
        let request_id = $crate::logging::new_request_id();
        let start_time = std::time::Instant::now();
        
        $crate::logging::log_rpc_request_start(
            request_id,
            $method,
            Some(&$client.url()),
            Some($params),
        );

        match $async_block.await {
            Ok(result) => {
                let duration = start_time.elapsed().as_millis() as u64;
                
                $crate::logging::log_rpc_request_success(
                    request_id,
                    $method,
                    duration,
                    Some("request completed"),
                );
                
                Ok(result)
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                let error = $crate::error::McpError::from(e)
                    .with_request_id(request_id)
                    .with_method($method)
                    .with_rpc_url(&$client.url());
                
                $crate::logging::log_rpc_request_failure(
                    request_id,
                    $method,
                    &error.error_type(),
                    duration,
                    Some(&error.to_log_value()),
                );
                
                Err(error)
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_logging() {
        INIT.call_once(|| {
            let _ = init_logging(Some("debug"));
        });
    }

    #[test]
    fn test_metrics_operations() {
        let metrics = Metrics::default();
        
        // Test initial state
        assert_eq!(metrics.total_calls.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.successful_calls.load(Ordering::Relaxed), 0);
        
        // Test incrementing
        metrics.increment_total_calls();
        metrics.increment_successful_calls(150);
        metrics.increment_failed_calls("validation", Some("getBalance"), 200);
        
        assert_eq!(metrics.total_calls.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.successful_calls.load(Ordering::Relaxed), 1);
        
        // Test dashmap access
        assert_eq!(metrics.failed_calls_by_type.get("validation").map(|v| *v), Some(1));
        assert_eq!(metrics.failed_calls_by_method.get("getBalance").map(|v| *v), Some(1));
    }

    #[test]
    fn test_params_summary_creation() {
        let params = serde_json::json!({
            "pubkey": "test",
            "commitment": "finalized"
        });
        
        let summary = create_params_summary(&params);
        assert!(summary.contains("pubkey"));
        assert!(summary.contains("commitment"));
    }

    #[test]
    fn test_result_summary_creation() {
        let result = serde_json::json!({
            "balance": 1000000,
            "slot": 12345
        });
        
        let summary = create_result_summary(&result);
        assert!(summary.contains("balance"));
        assert!(summary.contains("slot"));
    }

    #[test]
    fn test_logging_functions() {
        init_test_logging();
        
        let request_id = new_request_id();
        
        // Test various logging functions
        log_rpc_request_start(
            request_id,
            "getBalance",
            Some("https://api.mainnet-beta.solana.com"),
            Some("pubkey provided")
        );
        
        log_rpc_request_success(
            request_id,
            "getBalance",
            150,
            Some("balance returned")
        );
        
        log_validation_error(
            request_id,
            "getBalance",
            "pubkey",
            Some("invalid-key"),
            "Invalid pubkey format"
        );
    }

    #[test]
    fn test_metrics_json_serialization() {
        let metrics = Metrics::default();
        metrics.increment_total_calls();
        metrics.increment_successful_calls(100);
        metrics.increment_failed_calls("rpc", Some("getBalance"), 250);
        
        let json = metrics.to_json();
        assert!(json.get("total_calls").is_some());
        assert!(json.get("successful_calls").is_some());
        assert!(json.get("failed_calls_by_type").is_some());
        assert!(json.get("failed_calls_by_method").is_some());
        assert!(json.get("performance").is_some());
        
        // Check performance metrics
        let performance = json.get("performance").unwrap();
        assert!(performance.get("avg_duration_ms").is_some());
        assert!(performance.get("duration_histogram").is_some());
    }

    #[test]
    fn test_performance_histogram() {
        let metrics = Metrics::default();
        
        // Test different duration buckets
        metrics.increment_total_calls(); metrics.increment_successful_calls(5);   // 0-9ms bucket
        metrics.increment_total_calls(); metrics.increment_successful_calls(25);  // 10-49ms bucket  
        metrics.increment_total_calls(); metrics.increment_successful_calls(75);  // 50-99ms bucket
        metrics.increment_total_calls(); metrics.increment_successful_calls(200); // 100-499ms bucket
        metrics.increment_total_calls(); metrics.increment_successful_calls(750); // 500-999ms bucket
        metrics.increment_total_calls(); metrics.increment_successful_calls(1500); // 1000ms+ bucket
        
        let json = metrics.to_json();
        let performance = json.get("performance").unwrap();
        let histogram = performance.get("duration_histogram").unwrap();
        
        // Verify each bucket has 1 entry
        let histogram_array = histogram.as_array().unwrap();
        assert_eq!(histogram_array.len(), 6);
        
        // Check that average is calculated correctly
        // (5+25+75+200+750+1500)/6 = 425
        let avg_duration = performance.get("avg_duration_ms").unwrap().as_u64().unwrap();
        assert_eq!(avg_duration, 425);
    }
}