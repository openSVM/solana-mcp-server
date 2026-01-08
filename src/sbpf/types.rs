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

/// Response from deploying a program to devnet
#[derive(Debug, Clone, Serialize)]
pub struct DevnetDeployResponse {
    /// Deployed program ID (or placeholder if not yet deployed)
    pub program_id: String,

    /// Transaction signature (or N/A if using CLI)
    pub signature: String,

    /// Whether deployment succeeded
    pub deployed: bool,

    /// Size of deployed binary
    pub size_bytes: usize,

    /// Network (devnet/testnet/mainnet)
    pub network: String,

    /// RPC URL used
    pub rpc_url: String,

    /// CLI commands for manual deployment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_commands: Option<String>,

    /// Whether binary passed validation
    pub binary_valid: bool,

    /// Any validation warnings or notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_notes: Option<String>,
}

/// Parameters for devnet deployment
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevnetDeployParams {
    /// The compiled sBPF binary (base64-encoded)
    pub program_binary: String,

    /// Payer keypair (base64-encoded array of 64 bytes) - optional, will use airdrop if not provided
    pub payer_keypair: Option<String>,

    /// Custom RPC URL (defaults to devnet)
    pub rpc_url: Option<String>,
}
