use serde::{Deserialize, Serialize};

/// Parameters for testing an sBPF program
#[derive(Debug, Clone, Deserialize)]
pub struct TestParams {
    /// The compiled sBPF binary
    pub binary: Vec<u8>,

    /// Mock accounts to use in testing
    #[serde(default)]
    pub accounts: Vec<AccountSpec>,

    /// Instruction data to pass to the program
    pub instruction_data: Option<String>,  // base64

    /// Signers for the transaction
    #[serde(default)]
    pub signers: Vec<String>,  // pubkey strings
}

/// Account specification for testing
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSpec {
    /// Account public key
    pub pubkey: String,

    /// Initial lamports balance
    #[serde(default)]
    pub lamports: u64,

    /// Account data (base64-encoded)
    pub data: Option<String>,

    /// Account owner program ID
    pub owner: Option<String>,

    /// Whether account is executable
    #[serde(default)]
    pub executable: bool,

    /// Whether this account is a signer
    #[serde(default)]
    pub is_signer: bool,

    /// Whether this account is writable
    #[serde(default)]
    pub is_writable: bool,
}

/// Result of testing an sBPF program
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    /// Whether the test succeeded
    pub success: bool,

    /// Return value from the program
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_value: Option<u64>,

    /// Compute units consumed
    pub compute_units: u64,

    /// Program logs
    pub logs: Vec<String>,

    /// Account changes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub account_changes: Vec<AccountChange>,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Description of an account change during execution
#[derive(Debug, Clone, Serialize)]
pub struct AccountChange {
    /// Account public key
    pub pubkey: String,

    /// Change in lamports (can be negative)
    pub lamports_delta: i64,

    /// Whether account data changed
    pub data_changed: bool,

    /// New data size if changed
    pub new_data_size: Option<usize>,
}

/// Metadata about a validated binary
#[derive(Debug, Clone, Serialize)]
pub struct BinaryMetadata {
    /// Size in bytes
    pub size_bytes: usize,

    /// Architecture (should be "BPF")
    pub architecture: String,

    /// Entry point address
    pub entrypoint: String,

    /// ELF sections found
    pub sections: Vec<String>,

    /// Validation errors (if any)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
}

/// Response from deploying a program locally
#[derive(Debug, Clone, Serialize)]
pub struct DeployResponse {
    /// Generated program ID
    pub program_id: String,

    /// Whether deployment succeeded
    pub deployed: bool,

    /// Size of deployed binary
    pub size_bytes: usize,
}
