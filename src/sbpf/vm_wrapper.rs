use super::errors::SbpfError;
use super::types::{AccountChange, DeployResponse, TestResult};
use litesvm::LiteSVM;
use solana_sdk::{
    account::Account as SdkAccount,
    instruction::AccountMeta,
    pubkey::Pubkey as SdkPubkey,
    signature::{Keypair as SdkKeypair, Signer as SdkSigner},
};
use solana_transaction::Transaction as LiteTransaction;
use solana_message::Message as LiteMessage;
use solana_keypair::Keypair as LiteKeypair;
use solana_signer::Signer as LiteSigner;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

// Type conversion helpers between solana-sdk and litesvm types
fn sdk_pubkey_to_litesvm(sdk: &SdkPubkey) -> solana_pubkey::Pubkey {
    solana_pubkey::Pubkey::new_from_array(sdk.to_bytes())
}

fn litesvm_pubkey_to_sdk(litesvm: &solana_pubkey::Pubkey) -> SdkPubkey {
    SdkPubkey::new_from_array(litesvm.to_bytes())
}

fn sdk_account_to_litesvm(sdk: &SdkAccount) -> solana_account::Account {
    solana_account::Account {
        lamports: sdk.lamports,
        data: sdk.data.clone(),
        owner: sdk_pubkey_to_litesvm(&sdk.owner),
        executable: sdk.executable,
        rent_epoch: sdk.rent_epoch,
    }
}

fn litesvm_account_to_sdk(litesvm: &solana_account::Account) -> SdkAccount {
    SdkAccount {
        lamports: litesvm.lamports,
        data: litesvm.data.clone(),
        owner: litesvm_pubkey_to_sdk(&litesvm.owner),
        executable: litesvm.executable,
        rent_epoch: litesvm.rent_epoch,
    }
}

/// Wrapper around liteSVM for local sBPF testing
pub struct SbpfVmWrapper {
    vm: Arc<Mutex<LiteSVM>>,
    deployed_programs: Arc<Mutex<HashMap<SdkPubkey, Vec<u8>>>>,
}

impl SbpfVmWrapper {
    /// Create a new VM instance
    pub fn new() -> Self {
        // Create VM without signature verification for testing
        let mut vm = LiteSVM::new();

        // Note: liteSVM 0.9 may have limited support for certain program types
        // If execution fails, the validation and deployment features still work

        Self {
            vm: Arc::new(Mutex::new(vm)),
            deployed_programs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Deploy a program to the local VM
    pub async fn deploy_program(&self, binary: Vec<u8>) -> Result<DeployResponse, SbpfError> {
        let mut vm = self.vm.lock().await;

        // Generate a program keypair
        let program_keypair = SdkKeypair::new();
        let program_id_sdk = program_keypair.pubkey();
        let program_id_litesvm = sdk_pubkey_to_litesvm(&program_id_sdk);

        log::info!("Deploying program with ID: {}", program_id_sdk);
        log::debug!("Program binary size: {} bytes", binary.len());

        // Deploy to liteSVM
        // Note: add_program should make the program executable and ready for invocation
        vm.add_program(program_id_litesvm, &binary);

        // Verify the program was added by checking if the account exists
        if let Some(account) = vm.get_account(&program_id_litesvm) {
            log::debug!(
                "Program account created: executable={}, owner={:?}, data_len={}",
                account.executable,
                account.owner,
                account.data.len()
            );
        } else {
            log::warn!("Program account not found after add_program");
        }

        // Store in our deployed programs map
        let size_bytes = binary.len();
        self.deployed_programs
            .lock()
            .await
            .insert(program_id_sdk, binary);

        log::info!("Program deployed successfully: {}", program_id_sdk);

        Ok(DeployResponse {
            program_id: program_id_sdk.to_string(),
            deployed: true,
            size_bytes,
        })
    }

    /// Test a program with given accounts and instruction data
    pub async fn test_program(
        &self,
        program_id: &SdkPubkey,
        account_metas: Vec<AccountMeta>,
        instruction_data: Vec<u8>,
        initial_accounts: HashMap<SdkPubkey, SdkAccount>,
    ) -> Result<TestResult, SbpfError> {
        let mut vm = self.vm.lock().await;

        // Setup accounts in VM
        let account_keys: Vec<SdkPubkey> = initial_accounts.keys().cloned().collect();
        for (pubkey, account) in initial_accounts.iter() {
            let litesvm_pubkey = sdk_pubkey_to_litesvm(pubkey);
            let litesvm_account = sdk_account_to_litesvm(account);
            vm.set_account(litesvm_pubkey, litesvm_account)
                .map_err(|e| SbpfError::AccountError(format!("{:?}", e)))?;
        }

        // Verify program exists before trying to execute
        let program_id_lite = sdk_pubkey_to_litesvm(program_id);
        if let Some(program_account) = vm.get_account(&program_id_lite) {
            log::debug!(
                "Found program account: executable={}, owner={:?}, data_len={}",
                program_account.executable,
                program_account.owner,
                program_account.data.len()
            );

            if !program_account.executable {
                log::warn!("Program account exists but executable=false");
                return Err(SbpfError::ExecutionError(
                    "Program account is not marked as executable".to_string()
                ));
            }
        } else {
            log::error!("Program account not found: {}", program_id);
            return Err(SbpfError::ExecutionError(
                format!("Program {} not found in VM", program_id)
            ));
        }

        log::info!("Creating instruction for program: {}", program_id);

        // Create instruction using litesvm types
        let account_metas_lite: Vec<solana_instruction::AccountMeta> = account_metas
            .iter()
            .map(|am| solana_instruction::AccountMeta {
                pubkey: sdk_pubkey_to_litesvm(&am.pubkey),
                is_signer: am.is_signer,
                is_writable: am.is_writable,
            })
            .collect();

        log::debug!("Instruction accounts: {}", account_metas_lite.len());

        let instruction = solana_instruction::Instruction {
            program_id: program_id_lite,
            accounts: account_metas_lite,
            data: instruction_data.clone(),
        };

        log::debug!("Instruction data length: {} bytes", instruction_data.len());

        // Create and sign transaction
        let payer = LiteKeypair::new();
        let payer_pubkey = payer.pubkey();

        // Airdrop to payer for fees
        vm.airdrop(&payer_pubkey, 1_000_000_000)
            .map_err(|e| SbpfError::LiteSvmError(format!("{:?}", e)))?;

        let recent_blockhash = vm.latest_blockhash();

        // Build message and transaction
        let message = LiteMessage::new_with_blockhash(
            &[instruction],
            Some(&payer_pubkey),
            &recent_blockhash,
        );

        let mut transaction = LiteTransaction::new_unsigned(message);
        transaction.sign(&[&payer], recent_blockhash);

        // Process transaction
        let result = vm
            .send_transaction(transaction)
            .map_err(|e| SbpfError::ExecutionError(format!("{:?}", e)))?;

        // Extract logs
        let logs: Vec<String> = result
            .logs
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Calculate account changes
        let mut account_changes = Vec::new();
        for pubkey in account_keys {
            let litesvm_pubkey = sdk_pubkey_to_litesvm(&pubkey);
            let new_account_litesvm = vm.get_account(&litesvm_pubkey);

            let old_account = initial_accounts.get(&pubkey);

            if let (Some(old), Some(new_lit)) = (old_account, new_account_litesvm.as_ref()) {
                let new = litesvm_account_to_sdk(new_lit);
                let lamports_delta = new.lamports as i64 - old.lamports as i64;
                let data_changed = old.data != new.data;
                let new_data_size = if data_changed {
                    Some(new.data.len())
                } else {
                    None
                };

                if lamports_delta != 0 || data_changed {
                    account_changes.push(AccountChange {
                        pubkey: pubkey.to_string(),
                        lamports_delta,
                        data_changed,
                        new_data_size,
                    });
                }
            }
        }

        // Transaction was successful if we got here (send_transaction returned Ok)
        let success = true;
        let error = None;

        // Extract compute units from metadata
        let compute_units = result.compute_units_consumed;

        // Extract return data if available
        let return_value = if result.return_data.data.len() >= 8 {
            Some(u64::from_le_bytes(
                result.return_data.data[..8].try_into().unwrap()
            ))
        } else {
            None
        };

        Ok(TestResult {
            success,
            return_value,
            compute_units,
            logs,
            account_changes,
            error,
        })
    }

    /// Get an account from the VM
    pub async fn get_account(&self, pubkey: &SdkPubkey) -> Option<SdkAccount> {
        let vm = self.vm.lock().await;
        let litesvm_pubkey = sdk_pubkey_to_litesvm(pubkey);
        vm.get_account(&litesvm_pubkey).map(|acc| litesvm_account_to_sdk(&acc))
    }

    /// Airdrop lamports to an account
    pub async fn airdrop(&self, pubkey: &SdkPubkey, lamports: u64) -> Result<(), SbpfError> {
        let mut vm = self.vm.lock().await;
        let litesvm_pubkey = sdk_pubkey_to_litesvm(pubkey);
        vm.airdrop(&litesvm_pubkey, lamports)
            .map_err(|e| SbpfError::LiteSvmError(format!("{:?}", e)))?;
        Ok(())
    }
}

impl Default for SbpfVmWrapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to parse pubkey from string
pub fn parse_pubkey(s: &str) -> Result<SdkPubkey, SbpfError> {
    SdkPubkey::from_str(s).map_err(|e| SbpfError::PubkeyParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vm_creation() {
        let vm = SbpfVmWrapper::new();
        // Just verify VM creation doesn't panic
        let blockhash = vm.vm.lock().await.latest_blockhash();
        // Blockhash should be non-zero (LiteSVM initializes it)
        assert!(blockhash.to_bytes() != [0u8; 32]);
    }

    #[test]
    fn test_pubkey_parse() {
        let valid = "11111111111111111111111111111111";
        assert!(parse_pubkey(valid).is_ok());

        let invalid = "not-a-pubkey";
        assert!(parse_pubkey(invalid).is_err());
    }
}
