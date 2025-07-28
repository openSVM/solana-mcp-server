use crate::error::{McpError, McpResult};
use crate::logging::{log_rpc_request_start, log_rpc_request_success, log_rpc_request_failure, new_request_id};
use anyhow::Result;
use base64::Engine;
use serde_json::Value;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_config::{
        RpcSendTransactionConfig, RpcSimulateTransactionAccountsConfig,
        RpcSimulateTransactionConfig, RpcTransactionConfig,
    },
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use std::time::Instant;

pub async fn get_transaction(client: &RpcClient, signature: &Signature) -> Result<Value> {
    let tx = client
        .get_transaction(signature, UiTransactionEncoding::Json)
        .await?;
    Ok(serde_json::json!({ "transaction": tx }))
}

pub async fn get_transaction_with_config(
    client: &RpcClient,
    signature: &Signature,
    encoding: UiTransactionEncoding,
    commitment: Option<CommitmentConfig>,
    max_supported_transaction_version: Option<u8>,
) -> Result<Value> {
    let config = RpcTransactionConfig {
        encoding: Some(encoding),
        commitment,
        max_supported_transaction_version,
    };
    let tx = client
        .get_transaction_with_config(signature, config)
        .await?;
    Ok(serde_json::json!({ "transaction": tx }))
}

pub async fn get_signatures_for_address(
    client: &RpcClient,
    address: &Pubkey,
    before: Option<Signature>,
    until: Option<Signature>,
    limit: Option<u64>,
) -> Result<Value> {
    let config = GetConfirmedSignaturesForAddress2Config {
        before,
        until,
        limit: limit.map(|l| l as usize),
        commitment: None,
    };
    let signatures = client
        .get_signatures_for_address_with_config(address, config)
        .await?;
    Ok(serde_json::json!({ "signatures": signatures }))
}

pub async fn send_transaction(
    client: &RpcClient,
    transaction_data: &str,
    encoding: &str,
) -> Result<Value> {
    let wire_transaction = match encoding {
        "base58" => bs58::decode(transaction_data).into_vec()?,
        "base64" => base64::engine::general_purpose::STANDARD.decode(transaction_data)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid encoding. Must be base58 or base64"
            ))
        }
    };

    let tx: Transaction = bincode::deserialize(&wire_transaction)?;

    let signature = client
        .send_transaction_with_config(
            &tx,
            RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Finalized),
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
        )
        .await?;

    Ok(serde_json::json!({ "signature": signature }))
}

pub async fn send_transaction_with_config(
    client: &RpcClient,
    transaction_data: &str,
    encoding: &str,
    skip_preflight: bool,
    preflight_commitment: Option<CommitmentLevel>,
    max_retries: Option<usize>,
    min_context_slot: Option<u64>,
) -> Result<Value> {
    let wire_transaction = match encoding {
        "base58" => bs58::decode(transaction_data).into_vec()?,
        "base64" => base64::engine::general_purpose::STANDARD.decode(transaction_data)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid encoding. Must be base58 or base64"
            ))
        }
    };

    let tx: Transaction = bincode::deserialize(&wire_transaction)?;

    let signature = client
        .send_transaction_with_config(
            &tx,
            RpcSendTransactionConfig {
                skip_preflight,
                preflight_commitment,
                encoding: None,
                max_retries,
                min_context_slot,
            },
        )
        .await?;

    Ok(serde_json::json!({ "signature": signature }))
}

pub async fn simulate_transaction(
    client: &RpcClient,
    transaction_data: &str,
    encoding: &str,
) -> Result<Value> {
    let wire_transaction = match encoding {
        "base58" => bs58::decode(transaction_data).into_vec()?,
        "base64" => base64::engine::general_purpose::STANDARD.decode(transaction_data)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid encoding. Must be base58 or base64"
            ))
        }
    };

    let tx: Transaction = bincode::deserialize(&wire_transaction)?;

    let result = client.simulate_transaction(&tx).await?;
    Ok(serde_json::json!({ "result": result }))
}

pub async fn simulate_transaction_with_config(
    client: &RpcClient,
    transaction_data: &str,
    encoding: &str,
    sig_verify: bool,
    commitment: Option<CommitmentConfig>,
    replace_recent_blockhash: bool,
    accounts_to_return: Option<Vec<Pubkey>>,
    min_context_slot: Option<u64>,
) -> Result<Value> {
    let wire_transaction = match encoding {
        "base58" => bs58::decode(transaction_data).into_vec()?,
        "base64" => base64::engine::general_purpose::STANDARD.decode(transaction_data)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid encoding. Must be base58 or base64"
            ))
        }
    };

    let tx: Transaction = bincode::deserialize(&wire_transaction)?;

    let accounts_config = accounts_to_return.map(|keys| RpcSimulateTransactionAccountsConfig {
        encoding: None,
        addresses: keys.iter().map(|key| key.to_string()).collect(),
    });

    let config = RpcSimulateTransactionConfig {
        sig_verify,
        commitment,
        encoding: None,
        replace_recent_blockhash,
        accounts: accounts_config,
        min_context_slot,
        inner_instructions: true,
    };

    let result = client.simulate_transaction_with_config(&tx, config).await?;
    Ok(serde_json::json!({ "result": result }))
}

pub async fn get_block_time(client: &RpcClient, slot: u64) -> Result<Value> {
    let timestamp = client.get_block_time(slot).await?;
    Ok(serde_json::json!({ "timestamp": timestamp }))
}

pub async fn get_minimum_ledger_slot(client: &RpcClient) -> Result<Value> {
    let slot = client.minimum_ledger_slot().await?;
    Ok(serde_json::json!({ "slot": slot }))
}

pub async fn get_first_available_block(client: &RpcClient) -> Result<Value> {
    let slot = client.get_first_available_block().await?;
    Ok(serde_json::json!({ "slot": slot }))
}

pub async fn get_fee_for_message(
    client: &RpcClient,
    transaction_data: &str,
    encoding: &str,
) -> Result<Value> {
    let wire_transaction = match encoding {
        "base58" => bs58::decode(transaction_data).into_vec()?,
        "base64" => base64::engine::general_purpose::STANDARD.decode(transaction_data)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid encoding. Must be base58 or base64"
            ))
        }
    };

    let tx: Transaction = bincode::deserialize(&wire_transaction)?;
    let fee = client.get_fee_for_message(&tx.message).await?;
    Ok(serde_json::json!({ "fee": fee }))
}
/// Get confirmed transaction (deprecated version of getTransaction)
pub async fn get_confirmed_transaction(client: &RpcClient, signature: &Signature) -> Result<Value> {
    // Use the same implementation as get_transaction
    get_transaction(client, signature).await
}

/// Get confirmed transaction with config (deprecated version of getTransaction)
pub async fn get_confirmed_transaction_with_config(
    client: &RpcClient,
    signature: &Signature,
    encoding: UiTransactionEncoding,
    commitment: Option<CommitmentConfig>,
    max_supported_transaction_version: Option<u8>,
) -> Result<Value> {
    // Use the same implementation as get_transaction_with_config
    get_transaction_with_config(client, signature, encoding, commitment, max_supported_transaction_version).await
}

/// Get confirmed signatures for address 2 (deprecated version of getSignaturesForAddress)
pub async fn get_confirmed_signatures_for_address_2(
    client: &RpcClient,
    address: &Pubkey,
    before: Option<Signature>,
    until: Option<Signature>,
    limit: Option<u64>,
) -> Result<Value> {
    // Use the same implementation as get_signatures_for_address
    get_signatures_for_address(client, address, before, until, limit).await
}

/// Get signature statuses for a list of transaction signatures
pub async fn get_signature_statuses(
    client: &RpcClient,
    signatures: &[String],
    search_transaction_history: Option<bool>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getSignatureStatuses";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("signatures: {} to check", signatures.len())),
    );

    // Parse signature strings to Signature objects
    let parsed_signatures: Result<Vec<_>, _> = signatures.iter()
        .map(|sig| sig.parse::<solana_sdk::signature::Signature>())
        .collect();

    let signature_objects = match parsed_signatures {
        Ok(sigs) => sigs,
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::InvalidParameter(format!("Invalid signature: {}", e))
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(&client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                Some(&client.url()),
            );
            
            return Err(error);
        }
    };

    let _config = solana_client::rpc_config::RpcSignatureStatusConfig {
        search_transaction_history: search_transaction_history.unwrap_or(false),
    };

    match client.get_signature_statuses_with_history(&signature_objects).await {
        Ok(response) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({
                "context": {
                    "slot": response.context.slot
                },
                "value": response.value
            });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} signature statuses retrieved", signatures.len())),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(&client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                Some(&client.url()),
            );
            
            Err(error)
        }
    }
}
