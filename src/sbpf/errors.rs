use thiserror::Error;

#[derive(Error, Debug)]
pub enum SbpfError {
    #[error("Invalid binary: {0}")]
    InvalidBinary(String),

    #[error("Binary too large: {size} bytes (max: {max})")]
    BinaryTooLarge { size: usize, max: usize },

    #[error("Binary too small: {size} bytes (min: 64)")]
    BinaryTooSmall { size: usize },

    #[error("Not a valid ELF file")]
    NotElfFile,

    #[error("Not BPF architecture (found: {0})")]
    NotBpfArchitecture(u16),

    #[error("Deployment failed: {0}")]
    DeploymentError(String),

    #[error("Execution failed: {0}")]
    ExecutionError(String),

    #[error("Account error: {0}")]
    AccountError(String),

    #[error("Invalid parameter: {parameter}: {reason}")]
    InvalidParameter { parameter: String, reason: String },

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Pubkey parse error: {0}")]
    PubkeyParseError(String),

    #[error("liteSVM error: {0}")]
    LiteSvmError(String),
}

// Conversion to our MCP error type
impl From<SbpfError> for crate::error::McpError {
    fn from(err: SbpfError) -> Self {
        use crate::error::McpError;

        match err {
            SbpfError::InvalidBinary(msg) => {
                McpError::validation(msg)
                    .with_parameter("programBinary")
            }
            SbpfError::NotElfFile => {
                McpError::validation("Not a valid ELF file".to_string())
                    .with_parameter("programBinary")
            }
            SbpfError::NotBpfArchitecture(arch) => {
                McpError::validation(format!("Not BPF architecture (found: 0x{:x})", arch))
                    .with_parameter("programBinary")
            }
            SbpfError::BinaryTooLarge { size, max } => {
                McpError::validation(format!("Binary too large: {} > {}", size, max))
                    .with_parameter("programBinary")
            }
            SbpfError::BinaryTooSmall { size } => {
                McpError::validation(format!("Binary too small: {} bytes", size))
                    .with_parameter("programBinary")
            }
            SbpfError::InvalidParameter { parameter, reason } => {
                McpError::validation(reason)
                    .with_parameter(&parameter)
            }
            SbpfError::Base64Error(e) => {
                McpError::validation(format!("Base64 decode error: {}", e))
                    .with_parameter("programBinary")
            }
            SbpfError::PubkeyParseError(msg) => {
                McpError::validation(msg)
                    .with_parameter("pubkey")
            }
            SbpfError::DeploymentError(msg) => {
                McpError::server(format!("Deployment failed: {}", msg))
            }
            SbpfError::ExecutionError(msg) => {
                McpError::server(format!("Execution failed: {}", msg))
            }
            SbpfError::AccountError(msg) => {
                McpError::server(format!("Account error: {}", msg))
            }
            SbpfError::LiteSvmError(msg) => {
                McpError::server(format!("VM error: {}", msg))
            }
        }
    }
}
