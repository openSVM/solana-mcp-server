use thiserror::Error;
use serde_json::Value;
use uuid::Uuid;

/// Comprehensive error types for the Solana MCP Server
/// 
/// This module defines a hierarchy of error types that provide
/// rich context for debugging and monitoring while maintaining
/// security by avoiding sensitive data exposure.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum McpError {
    /// Client-side errors (invalid input, malformed requests)
    #[error("Client error: {message}")]
    Client {
        message: String,
        request_id: Option<Uuid>,
        method: Option<String>,
    },

    /// Server-side errors (internal failures, service unavailable)
    #[error("Server error: {message}")]
    Server {
        message: String,
        request_id: Option<Uuid>,
        method: Option<String>,
        source_message: Option<String>, // Store source error as string for Clone/PartialEq
    },

    /// RPC-specific errors (Solana client failures)
    #[error("RPC error: {message}")]
    Rpc {
        message: String,
        request_id: Option<Uuid>,
        method: Option<String>,
        rpc_url: Option<String>,
        source_message: Option<String>, // Store source error as string for Clone/PartialEq
    },

    /// Validation errors (invalid parameters, security checks)
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        request_id: Option<Uuid>,
        method: Option<String>,
        parameter: Option<String>,
    },

    /// Network errors (connectivity issues, timeouts)
    #[error("Network error: {message}")]
    Network {
        message: String,
        request_id: Option<Uuid>,
        method: Option<String>,
        endpoint: Option<String>,
    },

    /// Authentication/Authorization errors
    #[error("Auth error: {message}")]
    Auth {
        message: String,
        request_id: Option<Uuid>,
        method: Option<String>,
    },
}

impl McpError {
    /// Creates a client error with context
    pub fn client(message: impl Into<String>) -> Self {
        Self::Client {
            message: message.into(),
            request_id: None,
            method: None,
        }
    }

    /// Creates a server error with context
    pub fn server(message: impl Into<String>) -> Self {
        Self::Server {
            message: message.into(),
            request_id: None,
            method: None,
            source_message: None,
        }
    }

    /// Creates an RPC error with context
    pub fn rpc(message: impl Into<String>) -> Self {
        Self::Rpc {
            message: message.into(),
            request_id: None,
            method: None,
            rpc_url: None,
            source_message: None,
        }
    }

    /// Creates a validation error with context
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            request_id: None,
            method: None,
            parameter: None,
        }
    }

    /// Creates a network error with context
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            request_id: None,
            method: None,
            endpoint: None,
        }
    }

    /// Creates an auth error with context
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
            request_id: None,
            method: None,
        }
    }

    /// Adds request ID context to the error
    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        match &mut self {
            McpError::Client { request_id: ref mut id, .. } => *id = Some(request_id),
            McpError::Server { request_id: ref mut id, .. } => *id = Some(request_id),
            McpError::Rpc { request_id: ref mut id, .. } => *id = Some(request_id),
            McpError::Validation { request_id: ref mut id, .. } => *id = Some(request_id),
            McpError::Network { request_id: ref mut id, .. } => *id = Some(request_id),
            McpError::Auth { request_id: ref mut id, .. } => *id = Some(request_id),
        }
        self
    }

    /// Adds method context to the error
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        let method = method.into();
        match &mut self {
            McpError::Client { method: ref mut m, .. } => *m = Some(method),
            McpError::Server { method: ref mut m, .. } => *m = Some(method),
            McpError::Rpc { method: ref mut m, .. } => *m = Some(method),
            McpError::Validation { method: ref mut m, .. } => *m = Some(method),
            McpError::Network { method: ref mut m, .. } => *m = Some(method),
            McpError::Auth { method: ref mut m, .. } => *m = Some(method),
        }
        self
    }

    /// Adds parameter context to validation errors
    pub fn with_parameter(mut self, parameter: impl Into<String>) -> Self {
        if let McpError::Validation { parameter: ref mut p, .. } = &mut self {
            *p = Some(parameter.into());
        }
        self
    }

    /// Adds RPC URL context to RPC errors
    pub fn with_rpc_url(mut self, rpc_url: impl Into<String>) -> Self {
        if let McpError::Rpc { rpc_url: ref mut url, .. } = &mut self {
            *url = Some(rpc_url.into());
        }
        self
    }

    /// Adds endpoint context to network errors
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        if let McpError::Network { endpoint: ref mut e, .. } = &mut self {
            *e = Some(endpoint.into());
        }
        self
    }

    /// Adds source error context
    pub fn with_source(mut self, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        let source_message = source.to_string();
        match &mut self {
            McpError::Server { source_message: ref mut s, .. } => *s = Some(source_message),
            McpError::Rpc { source_message: ref mut s, .. } => *s = Some(source_message),
            _ => {}, // Other error types don't have source fields
        }
        self
    }

    /// Returns the JSON-RPC error code for this error type
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            McpError::Client { .. } => -32602, // Invalid params
            McpError::Validation { .. } => -32602, // Invalid params
            McpError::Auth { .. } => -32601, // Method not found (for security)
            McpError::Server { .. } => -32603, // Internal error
            McpError::Rpc { .. } => -32603, // Internal error
            McpError::Network { .. } => -32603, // Internal error
        }
    }

    /// Returns a safe error message for client responses (no sensitive info)
    pub fn safe_message(&self) -> String {
        match self {
            McpError::Client { message, .. } => message.clone(),
            McpError::Validation { message, .. } => message.clone(),
            McpError::Auth { .. } => "Authentication required".to_string(),
            McpError::Server { .. } => "Internal server error".to_string(),
            McpError::Rpc { .. } => "RPC service temporarily unavailable".to_string(),
            McpError::Network { .. } => "Network service temporarily unavailable".to_string(),
        }
    }

    /// Returns the request ID if available
    pub fn request_id(&self) -> Option<Uuid> {
        match self {
            McpError::Client { request_id, .. } => *request_id,
            McpError::Server { request_id, .. } => *request_id,
            McpError::Rpc { request_id, .. } => *request_id,
            McpError::Validation { request_id, .. } => *request_id,
            McpError::Network { request_id, .. } => *request_id,
            McpError::Auth { request_id, .. } => *request_id,
        }
    }

    /// Returns the method name if available
    pub fn method(&self) -> Option<&str> {
        match self {
            McpError::Client { method, .. } => method.as_deref(),
            McpError::Server { method, .. } => method.as_deref(),
            McpError::Rpc { method, .. } => method.as_deref(),
            McpError::Validation { method, .. } => method.as_deref(),
            McpError::Network { method, .. } => method.as_deref(),
            McpError::Auth { method, .. } => method.as_deref(),
        }
    }

    /// Converts to a JSON value for structured logging
    pub fn to_log_value(&self) -> Value {
        let mut log_data = serde_json::Map::new();
        
        log_data.insert("error_type".to_string(), Value::String(self.error_type().to_string()));
        log_data.insert("message".to_string(), Value::String(self.to_string()));
        
        if let Some(request_id) = self.request_id() {
            log_data.insert("request_id".to_string(), Value::String(request_id.to_string()));
        }
        
        if let Some(method) = self.method() {
            log_data.insert("method".to_string(), Value::String(method.to_string()));
        }

        match self {
            McpError::Validation { parameter, .. } => {
                if let Some(param) = parameter {
                    log_data.insert("parameter".to_string(), Value::String(param.clone()));
                }
            },
            McpError::Rpc { rpc_url, source_message, .. } => {
                if let Some(url) = rpc_url {
                    // Sanitize URL for logging
                    let sanitized = crate::validation::sanitize_for_logging(url);
                    log_data.insert("rpc_url".to_string(), Value::String(sanitized));
                }
                if let Some(source_msg) = source_message {
                    log_data.insert("source_error".to_string(), Value::String(source_msg.clone()));
                }
            },
            McpError::Network { endpoint, .. } => {
                if let Some(ep) = endpoint {
                    let sanitized = crate::validation::sanitize_for_logging(ep);
                    log_data.insert("endpoint".to_string(), Value::String(sanitized));
                }
            },
            McpError::Server { source_message, .. } => {
                if let Some(source_msg) = source_message {
                    log_data.insert("source_error".to_string(), Value::String(source_msg.clone()));
                }
            },
            _ => {}
        }

        Value::Object(log_data)
    }

    /// Returns the error type as a string for categorization
    pub fn error_type(&self) -> &'static str {
        match self {
            McpError::Client { .. } => "client",
            McpError::Server { .. } => "server", 
            McpError::Rpc { .. } => "rpc",
            McpError::Validation { .. } => "validation",
            McpError::Network { .. } => "network",
            McpError::Auth { .. } => "auth",
        }
    }
}

/// Convert anyhow errors to McpError
impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        McpError::server(err.to_string())
            .with_source(err.into())
    }
}

/// Convert solana client errors to McpError
impl From<solana_client::client_error::ClientError> for McpError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        use solana_client::client_error::ClientErrorKind;
        
        match err.kind() {
            ClientErrorKind::Io(_) => McpError::network(err.to_string()),
            ClientErrorKind::Reqwest(_) => McpError::network(err.to_string()),
            ClientErrorKind::RpcError(_) => McpError::rpc(err.to_string()),
            ClientErrorKind::SerdeJson(_) => McpError::server(err.to_string()),
            _ => McpError::server(err.to_string()),
        }.with_source(Box::new(err))
    }
}

/// Result type alias for MCP operations
pub type McpResult<T> = Result<T, McpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation_and_chaining() {
        let request_id = Uuid::new_v4();
        let error = McpError::validation("Invalid pubkey format")
            .with_request_id(request_id)
            .with_method("getBalance")
            .with_parameter("pubkey");

        assert_eq!(error.json_rpc_code(), -32602);
        assert_eq!(error.request_id(), Some(request_id));
        assert_eq!(error.method(), Some("getBalance"));
        assert_eq!(error.error_type(), "validation");
    }

    #[test]
    fn test_safe_message() {
        let server_error = McpError::server("Database connection failed with password: secret123");
        assert_eq!(server_error.safe_message(), "Internal server error");

        let validation_error = McpError::validation("Invalid pubkey format");
        assert_eq!(validation_error.safe_message(), "Invalid pubkey format");
    }

    #[test]
    fn test_log_value_serialization() {
        let request_id = Uuid::new_v4();
        let error = McpError::rpc("Connection timeout")
            .with_request_id(request_id)
            .with_method("getBalance")
            .with_rpc_url("https://api.mainnet-beta.solana.com");

        let log_value = error.to_log_value();
        assert!(log_value.get("error_type").is_some());
        assert!(log_value.get("request_id").is_some());
        assert!(log_value.get("method").is_some());
        assert!(log_value.get("rpc_url").is_some());
    }

    #[test]
    fn test_derived_traits() {
        let request_id = Uuid::new_v4();
        let error1 = McpError::validation("Invalid pubkey format")
            .with_request_id(request_id)
            .with_method("getBalance")
            .with_parameter("pubkey");

        // Test Clone
        let error2 = error1.clone();
        assert_eq!(error1.request_id(), error2.request_id());
        assert_eq!(error1.method(), error2.method());
        assert_eq!(error1.error_type(), error2.error_type());

        // Test PartialEq
        assert_eq!(error1, error2);

        // Test that different errors are not equal
        let error3 = McpError::client("Different error");
        assert_ne!(error1, error3);
    }
}