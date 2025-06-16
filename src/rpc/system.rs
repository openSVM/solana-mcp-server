use anyhow::Result;
use serde_json::Value;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcRequestAirdropConfig};
use solana_sdk::{commitment_config::CommitmentConfig, message::Message, pubkey::Pubkey};

pub async fn get_health(client: &RpcClient) -> Result<Value> {
    let health = client.get_health().await?;
    Ok(serde_json::json!({ "health": health }))
}

pub async fn get_version(client: &RpcClient) -> Result<Value> {
    let version = client.get_version().await?;
    Ok(serde_json::json!({ "version": version }))
}

pub async fn get_identity(client: &RpcClient) -> Result<Value> {
    let identity = client.get_identity().await?;
    Ok(serde_json::json!({ "identity": identity }))
}

pub async fn get_cluster_nodes(client: &RpcClient) -> Result<Value> {
    let nodes = client.get_cluster_nodes().await?;
    Ok(serde_json::json!({ "nodes": nodes }))
}

pub async fn get_epoch_info(client: &RpcClient) -> Result<Value> {
    let epoch_info = client.get_epoch_info().await?;
    Ok(serde_json::json!({ "epoch_info": epoch_info }))
}

pub async fn get_epoch_schedule(client: &RpcClient) -> Result<Value> {
    let schedule = client.get_epoch_schedule().await?;
    Ok(serde_json::json!({ "schedule": schedule }))
}

pub async fn get_inflation_governor(client: &RpcClient) -> Result<Value> {
    let governor = client.get_inflation_governor().await?;
    Ok(serde_json::json!({ "governor": governor }))
}

pub async fn get_inflation_rate(client: &RpcClient) -> Result<Value> {
    let inflation = client.get_inflation_rate().await?;
    Ok(serde_json::json!({ "inflation_rate": inflation }))
}

pub async fn get_inflation_reward(
    client: &RpcClient,
    addresses: &[Pubkey],
    epoch: Option<u64>,
) -> Result<Value> {
    let rewards = client.get_inflation_reward(addresses, epoch).await?;
    Ok(serde_json::json!({ "rewards": rewards }))
}

pub async fn get_minimum_balance_for_rent_exemption(
    client: &RpcClient,
    data_len: usize,
) -> Result<Value> {
    let lamports = client
        .get_minimum_balance_for_rent_exemption(data_len)
        .await?;
    Ok(serde_json::json!({ "lamports": lamports }))
}

pub async fn get_supply(client: &RpcClient) -> Result<Value> {
    let supply = client.supply().await?;
    Ok(serde_json::json!({ "supply": supply }))
}

pub async fn get_supply_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let supply = client.supply_with_commitment(commitment).await?;
    Ok(serde_json::json!({ "supply": supply }))
}

pub async fn request_airdrop(client: &RpcClient, pubkey: &Pubkey, lamports: u64) -> Result<Value> {
    let signature = client.request_airdrop(pubkey, lamports).await?;
    Ok(serde_json::json!({ "signature": signature }))
}

pub async fn request_airdrop_with_config(
    client: &RpcClient,
    pubkey: &Pubkey,
    lamports: u64,
    commitment: Option<CommitmentConfig>,
    recent_blockhash: Option<String>,
) -> Result<Value> {
    let config = RpcRequestAirdropConfig {
        commitment,
        recent_blockhash,
    };
    let signature = client
        .request_airdrop_with_config(pubkey, lamports, config)
        .await?;
    Ok(serde_json::json!({ "signature": signature }))
}

pub async fn get_stake_minimum_delegation(client: &RpcClient) -> Result<Value> {
    let minimum = client.get_stake_minimum_delegation().await?;
    Ok(serde_json::json!({ "minimum": minimum }))
}

pub async fn get_stake_minimum_delegation_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let minimum = client
        .get_stake_minimum_delegation_with_commitment(commitment)
        .await?;
    Ok(serde_json::json!({ "minimum": minimum }))
}

pub async fn get_transaction_count(client: &RpcClient) -> Result<Value> {
    let count = client.get_transaction_count().await?;
    Ok(serde_json::json!({ "count": count }))
}

pub async fn get_transaction_count_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let count = client
        .get_transaction_count_with_commitment(commitment)
        .await?;
    Ok(serde_json::json!({ "count": count }))
}

pub async fn get_latest_blockhash(client: &RpcClient) -> Result<Value> {
    let blockhash = client.get_latest_blockhash().await?;
    Ok(serde_json::json!({
        "blockhash": blockhash.to_string()
    }))
}

pub async fn get_latest_blockhash_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let blockhash = client
        .get_latest_blockhash_with_commitment(commitment)
        .await?
        .0;
    Ok(serde_json::json!({
        "blockhash": blockhash.to_string()
    }))
}

pub async fn get_fee_for_message(client: &RpcClient, message: &Message) -> Result<Value> {
    let fee = client.get_fee_for_message(message).await?;
    Ok(serde_json::json!({ "fee": fee }))
}
