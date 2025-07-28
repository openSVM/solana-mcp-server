use anyhow::{Context, Result};
use serde_json::{json, Value};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

/// Get block commitment information for a specific slot
/// This method returns commitment information for a given slot
pub async fn get_block_commitment(
    client: &RpcClient,
    slot: u64,
) -> Result<Value> {
    let method = "getBlockCommitment";
    
    // Make the RPC call manually since it's not in the client
    let params = json!([slot]);
    
    match client.send::<Value>(
        solana_client::rpc_request::RpcRequest::Custom { method },
        params,
    ).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into()),
    }
}

/// Get the current snapshot slot
/// This method returns the slot of the current snapshot
pub async fn get_snapshot_slot(
    client: &RpcClient,
) -> Result<Value> {
    let method = "getSnapshotSlot";
        
    // Make the RPC call manually since it's not in the client
    let params = json!([]);
    
    match client.send::<Value>(
        solana_client::rpc_request::RpcRequest::Custom { method },
        params,
    ).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into()),
    }
}

/// Get stake activation information for a given stake account
/// Returns the stake activation state for a stake account
pub async fn get_stake_activation(
    client: &RpcClient,
    pubkey: &str,
    commitment: Option<CommitmentConfig>,
) -> Result<Value> {
    let method = "getStakeActivation";
    
    // Validate pubkey format
    let stake_pubkey = pubkey.parse::<solana_sdk::pubkey::Pubkey>()
        .with_context(|| format!("Invalid stake account pubkey: {}", pubkey))?;
    
    // Build params
    let mut params = vec![json!(stake_pubkey.to_string())];
    
    // Add optional configuration
    if let Some(config) = commitment {
        let config_obj = json!({
            "commitment": config.commitment.to_string()
        });
        params.push(config_obj);
    }
    
    // Make the RPC call manually since it's not in the client
    match client.send::<Value>(
        solana_client::rpc_request::RpcRequest::Custom { method },
        json!(params),
    ).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into()),
    }
}