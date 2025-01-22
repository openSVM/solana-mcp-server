use anyhow::Result;
use serde_json::Value;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{
        RpcBlockConfig, RpcBlockProductionConfig, RpcBlockProductionConfigRange,
        RpcGetVoteAccountsConfig, RpcLeaderScheduleConfig,
    },
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{TransactionDetails, UiTransactionEncoding};

pub async fn get_slot(client: &RpcClient) -> Result<Value> {
    let slot = client.get_slot().await?;
    Ok(serde_json::json!({ "slot": slot }))
}

pub async fn get_slot_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let slot = client.get_slot_with_commitment(commitment).await?;
    Ok(serde_json::json!({ "slot": slot }))
}

pub async fn get_slot_leaders(client: &RpcClient, start_slot: u64, limit: u64) -> Result<Value> {
    let leaders = client.get_slot_leaders(start_slot, limit).await?;
    Ok(serde_json::json!({ "leaders": leaders }))
}

pub async fn get_block(client: &RpcClient, slot: u64) -> Result<Value> {
    let block = client.get_block(slot).await?;
    Ok(serde_json::json!({ "block": block }))
}

pub async fn get_block_with_config(
    client: &RpcClient,
    slot: u64,
    encoding: Option<UiTransactionEncoding>,
    transaction_details: Option<TransactionDetails>,
    rewards: Option<bool>,
    commitment: Option<CommitmentConfig>,
) -> Result<Value> {
    let config = RpcBlockConfig {
        encoding,
        transaction_details,
        rewards,
        commitment,
        max_supported_transaction_version: None,
    };
    let block = client.get_block_with_config(slot, config).await?;
    Ok(serde_json::json!({ "block": block }))
}

pub async fn get_block_height(client: &RpcClient) -> Result<Value> {
    let height = client.get_block_height().await?;
    Ok(serde_json::json!({ "height": height }))
}

pub async fn get_block_height_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let height = client.get_block_height_with_commitment(commitment).await?;
    Ok(serde_json::json!({ "height": height }))
}

pub async fn get_block_production(
    client: &RpcClient,
    identity: Option<String>,
    first_slot: Option<u64>,
    last_slot: Option<u64>,
) -> Result<Value> {
    let config = RpcBlockProductionConfig {
        identity: identity.as_ref().map(|s| s.parse()).transpose()?,
        range: first_slot.map(|start| RpcBlockProductionConfigRange {
            first_slot: start,
            last_slot: Some(last_slot.unwrap_or(start + 10)),
        }),
        commitment: None,
    };
    let production = client.get_block_production_with_config(config).await?;
    Ok(serde_json::json!({ "production": production }))
}

pub async fn get_blocks(client: &RpcClient, start_slot: u64, end_slot: Option<u64>) -> Result<Value> {
    let blocks = client.get_blocks(start_slot, end_slot).await?;
    Ok(serde_json::json!({ "blocks": blocks }))
}

pub async fn get_blocks_with_commitment(
    client: &RpcClient,
    start_slot: u64,
    end_slot: Option<u64>,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let blocks = client.get_blocks_with_commitment(start_slot, end_slot, commitment).await?;
    Ok(serde_json::json!({ "blocks": blocks }))
}

pub async fn get_blocks_with_limit(
    client: &RpcClient,
    start_slot: u64,
    limit: usize,
) -> Result<Value> {
    let blocks = client.get_blocks_with_limit(start_slot, limit).await?;
    Ok(serde_json::json!({ "blocks": blocks }))
}

pub async fn get_blocks_with_limit_and_commitment(
    client: &RpcClient,
    start_slot: u64,
    limit: usize,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let blocks = client.get_blocks_with_limit_and_commitment(start_slot, limit, commitment).await?;
    Ok(serde_json::json!({ "blocks": blocks }))
}

pub async fn get_leader_schedule(
    client: &RpcClient,
    slot: Option<u64>,
    identity: Option<String>,
) -> Result<Value> {
    let config = RpcLeaderScheduleConfig {
        identity: identity.as_ref().map(|s| s.parse()).transpose()?,
        commitment: None,
    };
    let schedule = client.get_leader_schedule_with_config(slot, config).await?;
    Ok(serde_json::json!({ "schedule": schedule }))
}

pub async fn get_max_retransmit_slot(client: &RpcClient) -> Result<Value> {
    let slot = client.get_max_retransmit_slot().await?;
    Ok(serde_json::json!({ "slot": slot }))
}

pub async fn get_max_shred_insert_slot(client: &RpcClient) -> Result<Value> {
    let slot = client.get_max_shred_insert_slot().await?;
    Ok(serde_json::json!({ "slot": slot }))
}

pub async fn get_vote_accounts(client: &RpcClient) -> Result<Value> {
    let accounts = client.get_vote_accounts().await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_vote_accounts_with_config(
    client: &RpcClient,
    commitment: Option<CommitmentConfig>,
    vote_pubkey: Option<String>,
    keep_unstaked_delinquents: Option<bool>,
    delinquent_slot_distance: Option<u64>,
) -> Result<Value> {
    let config = RpcGetVoteAccountsConfig {
        commitment,
        vote_pubkey,
        keep_unstaked_delinquents,
        delinquent_slot_distance,
    };
    let accounts = client.get_vote_accounts_with_config(config).await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_first_available_block(client: &RpcClient) -> Result<Value> {
    let slot = client.get_first_available_block().await?;
    Ok(serde_json::json!({ "slot": slot }))
}

pub async fn get_genesis_hash(client: &RpcClient) -> Result<Value> {
    let hash = client.get_genesis_hash().await?;
    Ok(serde_json::json!({ "hash": hash }))
}
