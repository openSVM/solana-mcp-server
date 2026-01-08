# sBPF Testing Integration Plan: Using liteSVM

**Date:** 2026-01-08
**Status:** Design Document
**Target:** solana-mcp-server v1.2.0

## Executive Summary

Integrate **liteSVM** into solana-mcp-server to enable local sBPF binary testing without deploying to devnet/testnet.

**Key Decision:** Use liteSVM because:
- ✅ Lightweight, fast Solana VM
- ✅ Rust-native (easy integration)
- ✅ Well-maintained, public library
- ✅ Zero deployment overhead for users
- ✅ Perfect for quick validation workflow

---

## Architecture: liteSVM In-Process Integration

### Integration Strategy

**Option Selected:** **In-Process Library Integration**

**Why In-Process?**
1. **Zero IPC Overhead:** Direct function calls, no serialization
2. **Rust Native:** liteSVM is Rust, we're Rust - perfect match
3. **Simple:** Single dependency, no external processes
4. **Fast:** <10ms latency for most tests
5. **Reliable:** No process management complexity

**Alternative Rejected:** Subprocess/Docker
- Reason: Unnecessary complexity, slower, harder to debug

---

## 1. liteSVM Capabilities

### What liteSVM Provides

```rust
use litesvm::LiteSVM;

let mut vm = LiteSVM::new();

// Deploy program
vm.deploy_program(program_bytes)?;

// Process transaction
let result = vm.process_transaction(tx)?;

// Query accounts
let account = vm.get_account(&pubkey)?;
```

**Features:**
- ✅ Deploy programs
- ✅ Process transactions
- ✅ Account management
- ✅ Syscall emulation
- ✅ PDAs
- ✅ Rent calculation
- ✅ Multi-instruction transactions
- ✅ CPI support

**Perfect for:** Quick validation before deploying to network

---

## 2. Dependency Addition

**Cargo.toml:**
```toml
[dependencies]
# Existing dependencies...
solana-client = "~2.3"
solana-sdk = "~2.3"
solana-program = "~2.3"

# NEW: Local sBPF testing
litesvm = "0.3"                    # Lightweight Solana VM

# Binary parsing
goblin = "0.8"                     # ELF parser
```

**No Conflicts:** liteSVM uses same solana-sdk version we already have

---

## 3. New MCP Tools

### Tool 1: testSbpfProgram

**Description:** Test a compiled sBPF program locally

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "programBinary": {
      "type": "string",
      "description": "Base64-encoded sBPF binary (.so file)"
    },
    "accounts": {
      "type": "array",
      "description": "Mock accounts for testing",
      "items": {
        "type": "object",
        "properties": {
          "pubkey": { "type": "string" },
          "lamports": { "type": "integer" },
          "data": { "type": "string" },
          "owner": { "type": "string" },
          "executable": { "type": "boolean" }
        }
      }
    },
    "instructionData": {
      "type": "string",
      "description": "Base64-encoded instruction data to send to program"
    },
    "signers": {
      "type": "array",
      "description": "Array of signer pubkey strings",
      "items": { "type": "string" }
    }
  },
  "required": ["programBinary"]
}
```

**Response:**
```json
{
  "success": true,
  "transactionSignature": "...",
  "computeUnitsConsumed": 15234,
  "logs": [
    "Program log: Hello, Solana!",
    "Program returned success"
  ],
  "accountChanges": [
    {
      "pubkey": "...",
      "lamportsDelta": 1000,
      "dataChanged": true
    }
  ],
  "error": null
}
```

### Tool 2: validateSbpfBinary

**Description:** Validate sBPF binary structure without execution

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "programBinary": {
      "type": "string",
      "description": "Base64-encoded sBPF binary"
    }
  },
  "required": ["programBinary"]
}
```

**Response:**
```json
{
  "valid": true,
  "sizeBytes": 524288,
  "architecture": "BPF",
  "entrypoint": "0x1000",
  "sections": [
    ".text",
    ".rodata",
    ".data"
  ],
  "errors": []
}
```

### Tool 3: deploySbpfProgramLocal

**Description:** Deploy program to local VM and return program ID

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "programBinary": {
      "type": "string",
      "description": "Base64-encoded sBPF binary"
    },
    "programName": {
      "type": "string",
      "description": "Optional name for the deployed program"
    }
  },
  "required": ["programBinary"]
}
```

**Response:**
```json
{
  "programId": "7EqQdEUxfGLu...",
  "deployed": true,
  "sizeBytes": 524288
}
```

---

## 4. Module Structure

```
src/
├── sbpf/                          # NEW MODULE
│   ├── mod.rs                     # Public API
│   ├── vm_wrapper.rs              # liteSVM wrapper
│   ├── binary_validator.rs       # Binary validation
│   ├── test_executor.rs           # Test execution
│   ├── errors.rs                  # sBPF-specific errors
│   └── types.rs                   # Data structures
├── tools/mod.rs                   # Add new tools
└── config.rs                      # Add sbpf config
```

---

## 5. Implementation Details

### A. VM Wrapper (`src/sbpf/vm_wrapper.rs`)

```rust
use litesvm::LiteSVM;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::Instruction,
    account::Account,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SbpfVmWrapper {
    vm: Arc<Mutex<LiteSVM>>,
}

impl SbpfVmWrapper {
    pub fn new() -> Self {
        Self {
            vm: Arc::new(Mutex::new(LiteSVM::new())),
        }
    }

    pub async fn deploy_program(&self, binary: Vec<u8>) -> McpResult<Pubkey> {
        let mut vm = self.vm.lock().await;

        // Generate program keypair
        let program_keypair = Keypair::new();
        let program_id = program_keypair.pubkey();

        // Deploy program to VM
        vm.deploy_program(&binary)
            .map_err(|e| McpError::server(format!("Deploy failed: {}", e)))?;

        Ok(program_id)
    }

    pub async fn test_program(
        &self,
        program_id: &Pubkey,
        accounts: Vec<AccountMeta>,
        instruction_data: Vec<u8>,
        signers: Vec<Pubkey>,
    ) -> McpResult<TestResult> {
        let mut vm = self.vm.lock().await;

        // Create instruction
        let instruction = Instruction {
            program_id: *program_id,
            accounts,
            data: instruction_data,
        };

        // Create transaction
        let payer = Keypair::new(); // Mock payer
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            vm.get_latest_blockhash(),
        );

        // Process transaction
        let result = vm.process_transaction(transaction)
            .map_err(|e| McpError::server(format!("Execution failed: {}", e)))?;

        // Extract results
        Ok(TestResult {
            success: result.is_success(),
            compute_units: result.compute_units_consumed,
            logs: result.logs,
            account_changes: extract_account_changes(&result),
        })
    }

    pub async fn get_account(&self, pubkey: &Pubkey) -> McpResult<Option<Account>> {
        let vm = self.vm.lock().await;
        Ok(vm.get_account(pubkey))
    }

    pub async fn airdrop(&self, pubkey: &Pubkey, lamports: u64) -> McpResult<()> {
        let mut vm = self.vm.lock().await;
        vm.airdrop(pubkey, lamports)
            .map_err(|e| McpError::server(format!("Airdrop failed: {}", e)))?;
        Ok(())
    }
}
```

### B. Binary Validator (`src/sbpf/binary_validator.rs`)

```rust
use goblin::elf::Elf;

pub struct BinaryValidator;

impl BinaryValidator {
    pub fn validate(data: &[u8]) -> McpResult<BinaryMetadata> {
        // 1. Size check
        if data.len() < 64 {
            return Err(McpError::validation("Binary too small"));
        }
        if data.len() > 512 * 1024 * 1024 {
            return Err(McpError::validation("Binary too large (>512MB)"));
        }

        // 2. ELF header validation
        if &data[0..4] != &[0x7F, 0x45, 0x4C, 0x46] {
            return Err(McpError::validation("Not a valid ELF binary"));
        }

        // 3. Parse ELF
        let elf = Elf::parse(data)
            .map_err(|e| McpError::validation(format!("Invalid ELF: {}", e)))?;

        // 4. Verify BPF architecture
        if elf.header.e_machine != 0xF7 {
            return Err(McpError::validation("Binary is not BPF architecture"));
        }

        // 5. Extract metadata
        Ok(BinaryMetadata {
            size_bytes: data.len(),
            architecture: "BPF".to_string(),
            entrypoint: format!("0x{:x}", elf.header.e_entry),
            sections: elf.section_headers.iter()
                .filter_map(|sh| elf.shdr_strtab.get_at(sh.sh_name))
                .map(|s| s.to_string())
                .collect(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct BinaryMetadata {
    pub size_bytes: usize,
    pub architecture: String,
    pub entrypoint: String,
    pub sections: Vec<String>,
}
```

### C. Test Executor (`src/sbpf/test_executor.rs`)

```rust
use super::vm_wrapper::SbpfVmWrapper;
use super::types::*;

pub struct TestExecutor {
    vm: SbpfVmWrapper,
}

impl TestExecutor {
    pub fn new() -> Self {
        Self {
            vm: SbpfVmWrapper::new(),
        }
    }

    pub async fn execute_test(&self, params: TestParams) -> McpResult<TestResult> {
        // 1. Validate binary
        let metadata = BinaryValidator::validate(&params.binary)?;
        log::info!("Validated binary: {} bytes", metadata.size_bytes);

        // 2. Deploy program
        let program_id = self.vm.deploy_program(params.binary).await?;
        log::info!("Deployed program: {}", program_id);

        // 3. Setup accounts
        for account_spec in params.accounts {
            let pubkey = Pubkey::from_str(&account_spec.pubkey)?;

            // Airdrop if needed
            if account_spec.lamports > 0 {
                self.vm.airdrop(&pubkey, account_spec.lamports).await?;
            }

            // Set account data if provided
            if let Some(data_b64) = account_spec.data {
                let data = base64::decode(data_b64)?;
                // Set account data via VM
                // (liteSVM provides this)
            }
        }

        // 4. Prepare instruction
        let instruction_data = if let Some(data_b64) = params.instruction_data {
            base64::decode(data_b64)?
        } else {
            vec![]
        };

        let account_metas: Vec<AccountMeta> = params.accounts
            .iter()
            .map(|spec| AccountMeta {
                pubkey: Pubkey::from_str(&spec.pubkey).unwrap(),
                is_signer: spec.is_signer.unwrap_or(false),
                is_writable: spec.is_writable.unwrap_or(false),
            })
            .collect();

        // 5. Execute
        let result = self.vm.test_program(
            &program_id,
            account_metas,
            instruction_data,
            params.signers.iter()
                .map(|s| Pubkey::from_str(s).unwrap())
                .collect(),
        ).await?;

        Ok(result)
    }
}
```

### D. Error Types (`src/sbpf/errors.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SbpfError {
    #[error("Invalid binary: {0}")]
    InvalidBinary(String),

    #[error("Deployment failed: {0}")]
    DeploymentError(String),

    #[error("Execution failed: {0}")]
    ExecutionError(String),

    #[error("Account error: {0}")]
    AccountError(String),

    #[error("Binary too large: {size} bytes (max: {max})")]
    BinaryTooLarge { size: usize, max: usize },
}

impl From<SbpfError> for McpError {
    fn from(err: SbpfError) -> Self {
        match err {
            SbpfError::InvalidBinary(msg) => McpError::validation(msg),
            SbpfError::BinaryTooLarge { size, max } => {
                McpError::validation(format!("Binary too large: {} > {}", size, max))
            }
            _ => McpError::server(err.to_string()),
        }
    }
}
```

---

## 6. Configuration

**File:** `src/config.rs`

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    // ... existing fields ...

    #[serde(default)]
    pub sbpf: SbpfConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SbpfConfig {
    /// Enable local sBPF testing
    pub enabled: bool,

    /// Maximum binary size (bytes)
    pub max_binary_size: usize,  // default: 512MB

    /// Default airdrop amount for test accounts
    pub default_airdrop_lamports: u64,  // default: 1_000_000_000 (1 SOL)
}

impl Default for SbpfConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_binary_size: 512 * 1024 * 1024,
            default_airdrop_lamports: 1_000_000_000,
        }
    }
}
```

**File:** `config.json`

```json
{
  "rpc_url": "https://api.mainnet-beta.solana.com",
  "commitment": "confirmed",

  "sbpf": {
    "enabled": true,
    "max_binary_size": 536870912,
    "default_airdrop_lamports": 1000000000
  }
}
```

---

## 7. Tool Integration

**File:** `src/tools/mod.rs`

### Register Tools

```rust
pub async fn handle_tools_list(...) -> Result<JsonRpcMessage> {
    let mut tools = vec![
        // ... existing tools ...
    ];

    // Add sBPF testing tools
    if state_guard.config.sbpf.enabled {
        tools.push(ToolDefinition {
            name: "testSbpfProgram".to_string(),
            description: Some("Test compiled sBPF program locally".to_string()),
            input_schema: sbpf_test_schema(),
        });

        tools.push(ToolDefinition {
            name: "validateSbpfBinary".to_string(),
            description: Some("Validate sBPF binary structure".to_string()),
            input_schema: sbpf_validate_schema(),
        });

        tools.push(ToolDefinition {
            name: "deploySbpfProgramLocal".to_string(),
            description: Some("Deploy program to local test VM".to_string()),
            input_schema: sbpf_deploy_schema(),
        });
    }

    Ok(/* ... */)
}
```

### Handle Tool Calls

```rust
pub async fn handle_tools_call(...) -> Result<JsonRpcMessage> {
    match tool_name {
        // ... existing tools ...

        "testSbpfProgram" => {
            let binary_b64 = arguments.get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing programBinary"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)?;

            let params = parse_test_params(arguments, binary)?;

            let executor = TestExecutor::new();
            let result = executor.execute_test(params).await?;

            Ok(serde_json::to_value(result)?)
        }

        "validateSbpfBinary" => {
            let binary_b64 = arguments.get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing programBinary"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)?;

            let metadata = BinaryValidator::validate(&binary)?;

            Ok(serde_json::to_value(metadata)?)
        }

        "deploySbpfProgramLocal" => {
            let binary_b64 = arguments.get("programBinary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing programBinary"))?;

            let binary = base64::engine::general_purpose::STANDARD
                .decode(binary_b64)?;

            let vm = SbpfVmWrapper::new();
            let program_id = vm.deploy_program(binary).await?;

            Ok(serde_json::json!({
                "programId": program_id.to_string(),
                "deployed": true
            }))
        }

        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}
```

---

## 8. HTTP Binary Upload (Optional Enhancement)

**File:** `src/http_server.rs`

```rust
use axum::{
    extract::{Multipart, State},
    routing::{post, get},
    Router,
};

impl McpHttpServer {
    pub async fn start(&self) -> Result<()> {
        let app = Router::new()
            // ... existing routes ...
            .route("/api/sbpf/upload", post(sbpf_upload_handler))
            .route("/api/sbpf/test/:file_id", post(sbpf_test_handler))
            // ...
    }
}

async fn sbpf_upload_handler(
    State(state): State<Arc<RwLock<ServerState>>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    let mut binary_data = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("binary") {
            binary_data = field.bytes().await?.to_vec();
        }
    }

    // Validate size
    if binary_data.len() > 512 * 1024 * 1024 {
        return Err(/* error */);
    }

    // Store with UUID
    let file_id = uuid::Uuid::new_v4().to_string();

    // Store in DashMap with 5-minute TTL
    let state_guard = state.write().await;
    state_guard.sbpf_upload_cache.insert(
        file_id.clone(),
        (binary_data, Instant::now()),
    );

    Ok(Json(UploadResponse {
        file_id,
        size_bytes: binary_data.len(),
    }))
}
```

---

## 9. Testing Strategy

### Unit Tests

**File:** `src/sbpf/test_executor.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_program() {
        let executor = TestExecutor::new();

        // Simple program that returns 42
        let binary = include_bytes!("../../fixtures/return42.so");

        let params = TestParams {
            binary: binary.to_vec(),
            accounts: vec![],
            instruction_data: None,
            signers: vec![],
        };

        let result = executor.execute_test(params).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_binary_validation() {
        let invalid_binary = b"not an elf file";
        let result = BinaryValidator::validate(invalid_binary);
        assert!(result.is_err());
    }
}
```

### Integration Tests

**File:** `tests/sbpf_integration.rs`

```rust
#[tokio::test]
async fn test_sbpf_tool_test_program() {
    let config = Config::load().unwrap();
    let state = Arc::new(RwLock::new(ServerState::new(config)));

    let binary = include_bytes!("fixtures/hello_world.so");
    let binary_b64 = base64::encode(binary);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "testSbpfProgram",
            "arguments": {
                "programBinary": binary_b64
            }
        }
    });

    let response = handle_request(request, state).await.unwrap();
    assert!(response["result"]["success"].as_bool().unwrap());
}
```

---

## 10. Implementation Checklist

### Phase 1: Core Integration (Week 1)
- [ ] Add litesvm to Cargo.toml
- [ ] Create src/sbpf/ module structure
- [ ] Implement vm_wrapper.rs
- [ ] Implement binary_validator.rs
- [ ] Implement test_executor.rs
- [ ] Add error types
- [ ] Write unit tests

### Phase 2: MCP Tools (Week 1-2)
- [ ] Add sbpf config to Config struct
- [ ] Register 3 new tools in handle_tools_list()
- [ ] Implement tool handlers in handle_tools_call()
- [ ] Add base64 encoding/decoding
- [ ] Write integration tests

### Phase 3: HTTP Upload (Week 2) - Optional
- [ ] Add /api/sbpf/upload endpoint
- [ ] Implement multipart form handling
- [ ] Add DashMap cache for uploaded files
- [ ] Add TTL cleanup task
- [ ] Write HTTP integration tests

### Phase 4: Production Hardening (Week 3)
- [ ] Add comprehensive error handling
- [ ] Add metrics (test duration, success rate)
- [ ] Performance benchmarking
- [ ] Security audit
- [ ] Documentation

---

## 11. Performance Expectations

| Metric | Target | Notes |
|--------|--------|-------|
| **Simple test** | <10ms | Direct function call, no IPC |
| **Complex test** | <50ms | Multi-instruction transaction |
| **Binary validation** | <5ms | ELF parsing only |
| **Deploy + test** | <20ms | Combined operation |
| **Memory overhead** | <50MB | Per VM instance |

---

## 12. Security Considerations

1. **Binary Size Limits:** Max 512MB to prevent DoS
2. **Input Validation:** Verify ELF format, reject malformed
3. **Compute Limits:** liteSVM has built-in compute unit limits
4. **Isolation:** Each test in separate VM instance
5. **Resource Cleanup:** Automatic memory cleanup after tests

---

## 13. Success Metrics

1. **Latency:** <20ms for 90% of tests
2. **Throughput:** Support 1000+ tests/minute
3. **Reliability:** 99.9% success rate on valid programs
4. **Adoption:** 30%+ of users test locally before deploying

---

## Conclusion

**Recommendation:** Implement liteSVM integration as in-process library.

**Key Benefits:**
- ✅ Fast (<20ms latency)
- ✅ Simple (single dependency)
- ✅ Reliable (no process management)
- ✅ Rust-native (perfect fit)
- ✅ Zero deployment overhead

**Timeline:** 2-3 weeks to production

**Next Steps:**
1. Add litesvm dependency
2. Implement src/sbpf/ module
3. Register MCP tools
4. Test and deploy

---

**Status:** Ready for implementation
**Priority:** HIGH (enables local testing workflow)
