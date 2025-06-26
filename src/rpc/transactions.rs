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
