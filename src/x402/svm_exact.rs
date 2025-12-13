//! x402 v2 SVM Exact Scheme Validation
//!
//! Implements validation for Solana (SVM) exact payment scheme.
//! Enforces strict requirements per the x402 v2 specification for SVM networks.

use crate::error::{McpError, McpResult};
use super::config::NetworkConfig;
use super::types::PaymentPayload;
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction::TokenInstruction;
use std::str::FromStr;

/// Validates SVM exact scheme payment payload
///
/// Enforces the following requirements:
/// 1. Strict instruction layout (CU limit, CU price, optional ATA create, TransferChecked)
/// 2. Facilitator fee payer must not appear in instruction accounts
/// 3. Facilitator fee payer must not be authority/source
/// 4. Compute unit price within bounds
/// 5. Destination ATA validates against payTo/asset
/// 6. Transfer amount equals PaymentRequirements.amount exactly
///
/// # Arguments
/// * `payload` - Payment payload to validate
/// * `network_config` - Network configuration with bounds
///
/// # Returns
/// * `McpResult<()>` - Ok if valid, Err with details if invalid
pub fn validate_svm_exact_payment(
    payload: &PaymentPayload,
    network_config: &NetworkConfig,
) -> McpResult<()> {
    // Validate network is Solana
    if !network_config.network.starts_with("solana:") {
        return Err(McpError::validation(
            "SVM exact validation only applies to Solana networks".to_string()
        ));
    }

    // Extract transaction data from payload
    let tx_data = payload.payload.get("transaction")
        .ok_or_else(|| McpError::validation("Missing transaction in payment payload".to_string()))?;

    // Parse transaction (base64 or base58 encoded)
    let tx_bytes = if let Some(tx_str) = tx_data.as_str() {
        decode_transaction(tx_str)?
    } else {
        return Err(McpError::validation("Invalid transaction format".to_string()));
    };

    // Deserialize transaction
    let transaction: solana_sdk::transaction::Transaction = bincode::deserialize(&tx_bytes)
        .map_err(|e| McpError::validation(format!("Failed to deserialize transaction: {}", e)))?;

    // Validate instruction layout
    validate_instruction_layout(&transaction)?;

    // Validate compute unit price bounds
    validate_compute_unit_price(&transaction, network_config)?;

    // Validate facilitator fee payer constraints
    validate_fee_payer_constraints(&transaction, payload)?;

    // Validate transfer amount
    validate_transfer_amount(&transaction, &payload.accepted.amount)?;

    // Validate destination account
    validate_destination_account(&transaction, &payload.accepted.pay_to, &payload.accepted.asset)?;

    Ok(())
}

/// Decodes transaction from base64 or base58 string
fn decode_transaction(tx_str: &str) -> McpResult<Vec<u8>> {
    // Try base64 first
    use base64::Engine;
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(tx_str) {
        return Ok(bytes);
    }

    // Try base58
    if let Ok(bytes) = bs58::decode(tx_str).into_vec() {
        return Ok(bytes);
    }

    Err(McpError::validation("Invalid transaction encoding".to_string()))
}

/// Validates instruction layout is correct
///
/// Expected layout:
/// 1. ComputeBudgetInstruction::SetComputeUnitLimit
/// 2. ComputeBudgetInstruction::SetComputeUnitPrice
/// 3. (Optional) AssociatedTokenAccount::Create
/// 4. Token::TransferChecked
fn validate_instruction_layout(tx: &solana_sdk::transaction::Transaction) -> McpResult<()> {
    let instructions = &tx.message.instructions;
    let account_keys = &tx.message.account_keys;

    if instructions.is_empty() {
        return Err(McpError::validation("Transaction has no instructions".to_string()));
    }

    // At minimum, we need 3 instructions (CU limit, CU price, TransferChecked)
    if instructions.len() < 3 {
        return Err(McpError::validation(format!(
            "Invalid instruction count. Expected at least 3, got {}",
            instructions.len()
        )));
    }

    // Well-known program IDs
    let compute_budget_program = solana_sdk::compute_budget::ID;
    let token_program = spl_token::ID;
    let ata_program = spl_associated_token_account::ID;

    // Track which instructions we've found
    let mut has_cu_limit = false;
    let mut has_cu_price = false;
    let mut has_transfer_checked = false;
    let mut ata_create_index: Option<usize> = None;
    let mut transfer_index: Option<usize> = None;

    for (idx, instruction) in instructions.iter().enumerate() {
        let program_id = account_keys.get(instruction.program_id_index as usize)
            .ok_or_else(|| McpError::validation(format!("Invalid program_id_index at instruction {}", idx)))?;

        // Check compute budget instructions
        if program_id == &compute_budget_program {
            // Try to deserialize as compute budget instruction
            if !instruction.data.is_empty() {
                // First byte is the instruction discriminator
                match instruction.data[0] {
                    2 => { // SetComputeUnitLimit
                        has_cu_limit = true;
                        tracing::debug!("Found SetComputeUnitLimit at index {}", idx);
                    }
                    3 => { // SetComputeUnitPrice
                        has_cu_price = true;
                        tracing::debug!("Found SetComputeUnitPrice at index {}", idx);
                    }
                    _ => {}
                }
            }
        }
        // Check ATA program instructions
        else if program_id == &ata_program {
            // ATA Create instruction typically has discriminator 0
            if instruction.data.is_empty() || instruction.data[0] == 0 {
                ata_create_index = Some(idx);
                tracing::debug!("Found ATA Create at index {}", idx);
            }
        }
        // Check token program instructions
        else if program_id == &token_program {
            // Try to unpack token instruction
            if let Ok(token_ix) = TokenInstruction::unpack(&instruction.data) {
                if matches!(token_ix, TokenInstruction::TransferChecked { .. }) {
                    has_transfer_checked = true;
                    transfer_index = Some(idx);
                    tracing::debug!("Found TransferChecked at index {}", idx);
                }
            }
        }
    }

    // Validate required instructions are present
    if !has_cu_limit {
        return Err(McpError::validation(
            "Missing required SetComputeUnitLimit instruction".to_string()
        ));
    }

    if !has_cu_price {
        return Err(McpError::validation(
            "Missing required SetComputeUnitPrice instruction".to_string()
        ));
    }

    if !has_transfer_checked {
        return Err(McpError::validation(
            "Missing required TransferChecked instruction".to_string()
        ));
    }

    // Validate instruction ordering
    // TransferChecked must be last (or ATA create must be just before it)
    if let Some(transfer_idx) = transfer_index {
        let expected_last_idx = instructions.len() - 1;
        
        if let Some(ata_idx) = ata_create_index {
            // If ATA create exists, it should be right before TransferChecked
            if ata_idx + 1 != transfer_idx {
                return Err(McpError::validation(
                    "ATA Create must immediately precede TransferChecked instruction".to_string()
                ));
            }
            // And transfer should be last
            if transfer_idx != expected_last_idx {
                return Err(McpError::validation(
                    "TransferChecked must be the last instruction".to_string()
                ));
            }
        } else {
            // No ATA create, transfer should be last
            if transfer_idx != expected_last_idx {
                return Err(McpError::validation(
                    "TransferChecked must be the last instruction".to_string()
                ));
            }
        }
    }

    tracing::debug!("Instruction layout validation passed");

    Ok(())
}

/// Validates compute unit price is within bounds
fn validate_compute_unit_price(
    tx: &solana_sdk::transaction::Transaction,
    network_config: &NetworkConfig,
) -> McpResult<()> {
    // Extract compute unit price from transaction
    let compute_unit_price = extract_compute_unit_price(tx)?;

    if let (Some(min), Some(max)) = (
        network_config.min_compute_unit_price,
        network_config.max_compute_unit_price,
    ) {
        if compute_unit_price < min || compute_unit_price > max {
            return Err(McpError::validation(format!(
                "Compute unit price {} out of bounds [{}, {}]",
                compute_unit_price, min, max
            )));
        }
    }

    tracing::debug!(
        compute_unit_price = compute_unit_price,
        "Compute unit price validated"
    );

    Ok(())
}

/// Extracts compute unit price from transaction
fn extract_compute_unit_price(tx: &solana_sdk::transaction::Transaction) -> McpResult<u64> {
    let instructions = &tx.message.instructions;
    let account_keys = &tx.message.account_keys;
    let compute_budget_program = solana_sdk::compute_budget::ID;

    for instruction in instructions.iter() {
        let program_id = account_keys.get(instruction.program_id_index as usize)
            .ok_or_else(|| McpError::validation("Invalid program_id_index in instruction".to_string()))?;

        if program_id == &compute_budget_program {
            // SetComputeUnitPrice: discriminator (1 byte) = 3, followed by u64 price (8 bytes)
            if instruction.data.len() == 9 && instruction.data[0] == 3 {
                let price_bytes = &instruction.data[1..9];
                let price = u64::from_le_bytes(price_bytes.try_into().unwrap());
                tracing::debug!(compute_unit_price = price, "Extracted compute unit price");
                return Ok(price);
            }
        }
    }

    Err(McpError::validation(
        "No SetComputeUnitPrice instruction found in transaction".to_string()
    ))
}

/// Validates facilitator fee payer constraints
fn validate_fee_payer_constraints(
    tx: &solana_sdk::transaction::Transaction,
    _payload: &PaymentPayload,
) -> McpResult<()> {
    let account_keys = &tx.message.account_keys;
    let fee_payer = account_keys.first()
        .ok_or_else(|| McpError::validation("Transaction has no fee payer".to_string()))?;

    let token_program = spl_token::ID;

    // Find TransferChecked instruction
    for instruction in tx.message.instructions.iter() {
        let program_id = account_keys.get(instruction.program_id_index as usize)
            .ok_or_else(|| McpError::validation("Invalid program_id_index".to_string()))?;

        if program_id == &token_program {
            if let Ok(token_ix) = TokenInstruction::unpack(&instruction.data) {
                if matches!(token_ix, TokenInstruction::TransferChecked { .. }) {
                
                // Get accounts involved in TransferChecked
                // TransferChecked accounts: [source, mint, destination, authority, ...signers]
                let transfer_accounts = &instruction.accounts;
                
                if transfer_accounts.len() < 4 {
                    return Err(McpError::validation(
                        "TransferChecked instruction has insufficient accounts".to_string()
                    ));
                }

                // Get the actual account keys from indices
                let source_idx = transfer_accounts[0] as usize;
                let authority_idx = transfer_accounts[3] as usize;

                let source_key = account_keys.get(source_idx)
                    .ok_or_else(|| McpError::validation("Invalid source account index".to_string()))?;
                let authority_key = account_keys.get(authority_idx)
                    .ok_or_else(|| McpError::validation("Invalid authority account index".to_string()))?;

                // Fee payer must not be the source
                if fee_payer == source_key {
                    return Err(McpError::validation(
                        "Fee payer cannot be the source of the transfer".to_string()
                    ));
                }

                // Fee payer must not be the authority
                if fee_payer == authority_key {
                    return Err(McpError::validation(
                        "Fee payer cannot be the authority of the transfer".to_string()
                    ));
                }

                // Fee payer should not appear in any TransferChecked accounts
                for &account_idx in transfer_accounts.iter() {
                    let account_key = account_keys.get(account_idx as usize)
                        .ok_or_else(|| McpError::validation("Invalid account index in TransferChecked".to_string()))?;
                    
                    if fee_payer == account_key {
                        return Err(McpError::validation(
                            "Fee payer must not appear in TransferChecked instruction accounts".to_string()
                        ));
                    }
                }

                    tracing::debug!(
                        fee_payer = %fee_payer,
                        "Fee payer constraints validated successfully"
                    );

                    return Ok(());
                }
            }
        }
    }

    Err(McpError::validation(
        "No TransferChecked instruction found for fee payer validation".to_string()
    ))
}

/// Validates transfer amount matches requirements
fn validate_transfer_amount(
    tx: &solana_sdk::transaction::Transaction,
    required_amount: &str,
) -> McpResult<()> {
    let required: u64 = required_amount.parse()
        .map_err(|e| McpError::validation(format!("Invalid required amount: {}", e)))?;

    let account_keys = &tx.message.account_keys;
    let token_program = spl_token::ID;

    // Find TransferChecked instruction and extract amount
    for instruction in tx.message.instructions.iter() {
        let program_id = account_keys.get(instruction.program_id_index as usize)
            .ok_or_else(|| McpError::validation("Invalid program_id_index".to_string()))?;

        if program_id == &token_program {
            if let Ok(TokenInstruction::TransferChecked { amount, decimals: _ }) = 
                TokenInstruction::unpack(&instruction.data) {
                
                if amount != required {
                    return Err(McpError::validation(format!(
                        "Transfer amount mismatch. Required: {}, Got: {}",
                        required, amount
                    )));
                }

                tracing::debug!(
                    required_amount = required,
                    actual_amount = amount,
                    "Transfer amount validated successfully"
                );

                return Ok(());
            }
        }
    }

    Err(McpError::validation(
        "No TransferChecked instruction found for amount validation".to_string()
    ))
}

/// Validates destination account matches payTo and asset
fn validate_destination_account(
    tx: &solana_sdk::transaction::Transaction,
    pay_to: &str,
    asset: &str,
) -> McpResult<()> {
    // Parse addresses
    let pay_to_pubkey = Pubkey::from_str(pay_to)
        .map_err(|e| McpError::validation(format!("Invalid payTo address: {}", e)))?;
    
    let asset_pubkey = Pubkey::from_str(asset)
        .map_err(|e| McpError::validation(format!("Invalid asset address: {}", e)))?;

    // Compute the expected Associated Token Account (ATA) address
    let expected_destination = spl_associated_token_account::get_associated_token_address(
        &pay_to_pubkey,
        &asset_pubkey,
    );

    let account_keys = &tx.message.account_keys;
    let token_program = spl_token::ID;

    // Find TransferChecked instruction and check destination
    for instruction in tx.message.instructions.iter() {
        let program_id = account_keys.get(instruction.program_id_index as usize)
            .ok_or_else(|| McpError::validation("Invalid program_id_index".to_string()))?;

        if program_id == &token_program {
            if let Ok(token_ix) = TokenInstruction::unpack(&instruction.data) {
                if matches!(token_ix, TokenInstruction::TransferChecked { .. }) {
                
                // TransferChecked accounts: [source, mint, destination, authority, ...signers]
                let transfer_accounts = &instruction.accounts;
                
                if transfer_accounts.len() < 3 {
                    return Err(McpError::validation(
                        "TransferChecked instruction has insufficient accounts".to_string()
                    ));
                }

                let destination_idx = transfer_accounts[2] as usize;
                let mint_idx = transfer_accounts[1] as usize;

                let destination_key = account_keys.get(destination_idx)
                    .ok_or_else(|| McpError::validation("Invalid destination account index".to_string()))?;
                let mint_key = account_keys.get(mint_idx)
                    .ok_or_else(|| McpError::validation("Invalid mint account index".to_string()))?;

                // Validate mint matches asset
                if mint_key != &asset_pubkey {
                    return Err(McpError::validation(format!(
                        "Mint address mismatch. Expected: {}, Got: {}",
                        asset_pubkey, mint_key
                    )));
                }

                // Validate destination is the correct ATA
                if destination_key != &expected_destination {
                    return Err(McpError::validation(format!(
                        "Destination ATA mismatch. Expected: {}, Got: {}",
                        expected_destination, destination_key
                    )));
                }

                    tracing::debug!(
                        pay_to = %pay_to_pubkey,
                        asset = %asset_pubkey,
                        expected_ata = %expected_destination,
                        actual_destination = %destination_key,
                        "Destination account validated successfully"
                    );

                    return Ok(());
                }
            }
        }
    }

    Err(McpError::validation(
        "No TransferChecked instruction found for destination validation".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{PaymentRequirements, ResourceInfo};

    #[test]
    fn test_non_solana_network_rejected() {
        let network_config = NetworkConfig {
            network: "eip155:1".to_string(),
            assets: vec![],
            pay_to: "0x123".to_string(),
            min_compute_unit_price: None,
            max_compute_unit_price: None,
        };

        let payload = PaymentPayload {
            x402_version: 2,
            resource: Some(ResourceInfo {
                url: "https://example.com".to_string(),
                description: None,
                mime_type: None,
            }),
            accepted: PaymentRequirements {
                scheme: "exact".to_string(),
                network: "eip155:1".to_string(),
                amount: "1000".to_string(),
                asset: "0xUSDC".to_string(),
                pay_to: "0x123".to_string(),
                max_timeout_seconds: 60,
                extra: None,
            },
            payload: serde_json::json!({}),
            extensions: None,
        };

        let result = validate_svm_exact_payment(&payload, &network_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_transaction_data() {
        let network_config = NetworkConfig {
            network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
            assets: vec![],
            pay_to: "test123".to_string(),
            min_compute_unit_price: Some(1000),
            max_compute_unit_price: Some(10000),
        };

        let payload = PaymentPayload {
            x402_version: 2,
            resource: None,
            accepted: PaymentRequirements {
                scheme: "exact".to_string(),
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                amount: "1000000".to_string(),
                asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                pay_to: "test123".to_string(),
                max_timeout_seconds: 60,
                extra: None,
            },
            payload: serde_json::json!({}),
            extensions: None,
        };

        let result = validate_svm_exact_payment(&payload, &network_config);
        assert!(result.is_err());
    }
}
