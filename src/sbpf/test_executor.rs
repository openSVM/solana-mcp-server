use super::binary_validator::BinaryValidator;
use super::errors::SbpfError;
use super::types::{TestParams, TestResult};
use super::vm_wrapper::{parse_pubkey, SbpfVmWrapper};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use std::collections::HashMap;

/// Executes sBPF program tests
pub struct TestExecutor {
    vm: SbpfVmWrapper,
}

impl TestExecutor {
    /// Create a new test executor
    pub fn new() -> Self {
        Self {
            vm: SbpfVmWrapper::new(),
        }
    }

    /// Execute a complete test from parameters
    pub async fn execute_test(&self, params: TestParams) -> Result<TestResult, SbpfError> {
        // 1. Validate binary
        let metadata = BinaryValidator::validate(&params.binary)?;
        log::info!(
            "Validated sBPF binary: {} bytes, {} sections",
            metadata.size_bytes,
            metadata.sections.len()
        );

        if !metadata.errors.is_empty() {
            log::warn!("Binary validation warnings: {:?}", metadata.errors);
        }

        // 2. Deploy program
        let deploy_result = self.vm.deploy_program(params.binary).await?;
        let program_id = parse_pubkey(&deploy_result.program_id)?;
        log::info!("Deployed program: {}", program_id);

        // 3. Setup accounts
        let mut initial_accounts = HashMap::new();
        let mut account_metas = Vec::new();

        for account_spec in params.accounts {
            let pubkey = parse_pubkey(&account_spec.pubkey)?;

            // Create account
            let mut account = Account {
                lamports: account_spec.lamports,
                data: vec![],
                owner: if let Some(owner_str) = &account_spec.owner {
                    parse_pubkey(owner_str)?
                } else {
                    solana_sdk::system_program::ID
                },
                executable: account_spec.executable,
                rent_epoch: 0,
            };

            // Decode and set account data if provided
            if let Some(data_b64) = &account_spec.data {
                account.data = BASE64
                    .decode(data_b64)
                    .map_err(|e| SbpfError::Base64Error(e))?;
            }

            // Store initial account state
            initial_accounts.insert(pubkey, account);

            // Create account meta for instruction
            account_metas.push(AccountMeta {
                pubkey,
                is_signer: account_spec.is_signer,
                is_writable: account_spec.is_writable,
            });
        }

        log::info!("Setup {} test accounts", initial_accounts.len());

        // 4. Decode instruction data
        let instruction_data = if let Some(data_b64) = params.instruction_data {
            BASE64
                .decode(&data_b64)
                .map_err(|e| SbpfError::Base64Error(e))?
        } else {
            vec![]
        };

        log::info!("Instruction data: {} bytes", instruction_data.len());

        // 5. Execute test
        let result = self
            .vm
            .test_program(&program_id, account_metas, instruction_data, initial_accounts)
            .await?;

        log::info!(
            "Test execution complete: success={}, compute_units={}, logs={}",
            result.success,
            result.compute_units,
            result.logs.len()
        );

        Ok(result)
    }

    /// Validate a binary without executing
    pub fn validate_only(binary: &[u8]) -> Result<super::types::BinaryMetadata, SbpfError> {
        BinaryValidator::validate(binary)
    }
}

impl Default for TestExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_only() {
        // Test with invalid binary
        let invalid = vec![0u8; 1024];
        let result = TestExecutor::validate_only(&invalid);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let _executor = TestExecutor::new();
        // Just verify it doesn't panic during creation
        assert!(true);
    }
}
