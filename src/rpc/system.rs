use crate::cache::with_cache;
use crate::error::{McpError, McpResult};
use crate::logging::{log_rpc_request_start, log_rpc_request_success, log_rpc_request_failure, new_request_id};
use serde_json::Value;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::RpcRequestAirdropConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    message::Message,
};
use std::sync::Arc;
use std::time::Instant;

/// Get node health status
pub async fn get_health(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getHealth";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_health().await {
        Ok(health) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "health": health });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("health status retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get node version information with caching support
pub async fn get_version_cached(
    client: &RpcClient,
    cache: &Arc<crate::cache::RpcCache>,
) -> McpResult<Value> {
    let method = "getVersion";
    let params = serde_json::json!({});
    
    with_cache(cache, method, &params, || {
        let client = client;
        async move {
            get_version(&client).await
        }
    }).await
}

/// Get node version information
pub async fn get_version(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getVersion";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_version().await {
        Ok(version) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "version": version });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("version info retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get node identity
pub async fn get_identity(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getIdentity";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_identity().await {
        Ok(identity) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "identity": identity });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("identity retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get cluster nodes information
pub async fn get_cluster_nodes(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getClusterNodes";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_cluster_nodes().await {
        Ok(nodes) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "nodes": nodes });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} cluster nodes retrieved", nodes.len())),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get current epoch information
pub async fn get_epoch_info(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getEpochInfo";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_epoch_info().await {
        Ok(epoch_info) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "epoch_info": epoch_info });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("epoch info retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get epoch schedule
pub async fn get_epoch_schedule(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getEpochSchedule";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_epoch_schedule().await {
        Ok(schedule) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "schedule": schedule });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("epoch schedule retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get inflation governor information
pub async fn get_inflation_governor(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getInflationGovernor";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_inflation_governor().await {
        Ok(governor) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "governor": governor });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("inflation governor retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get current inflation rate
pub async fn get_inflation_rate(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getInflationRate";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_inflation_rate().await {
        Ok(inflation) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "inflation_rate": inflation });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("inflation rate retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

/// Get inflation rewards for accounts
pub async fn get_inflation_reward(
    client: &RpcClient,
    addresses: &[Pubkey],
    epoch: Option<u64>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getInflationReward";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("addresses_count: {}, epoch: {:?}", addresses.len(), epoch)),
    );

    match client.get_inflation_reward(addresses, epoch).await {
        Ok(rewards) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "rewards": rewards });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("inflation rewards retrieved"),
                None,
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
            log_rpc_request_failure(
                request_id,
                method,
                error.error_type(),
                duration,
                Some(&error.to_log_value()),
                None,
            );
            
            Err(error)
        }
    }
}

pub async fn get_minimum_balance_for_rent_exemption(
    client: &RpcClient,
    data_len: usize,
) -> McpResult<Value> {
    let lamports = client.get_minimum_balance_for_rent_exemption(data_len).await?;
    Ok(serde_json::json!({ "lamports": lamports }))
}

pub async fn get_supply(client: &RpcClient) -> McpResult<Value> {
    let supply = client.supply().await?;
    Ok(serde_json::json!({ "supply": supply }))
}

pub async fn get_supply_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> McpResult<Value> {
    let supply = client.supply_with_commitment(commitment).await?;
    Ok(serde_json::json!({ "supply": supply }))
}

pub async fn request_airdrop(
    client: &RpcClient,
    pubkey: &Pubkey,
    lamports: u64,
) -> McpResult<Value> {
    use crate::log_rpc_call;
    
    let params_summary = format!("pubkey: {pubkey}, lamports: {lamports}");
    
    log_rpc_call!(
        "requestAirdrop",
        client,
        async {
            let signature = client.request_airdrop(pubkey, lamports).await?;
            Ok::<Value, crate::error::McpError>(serde_json::json!({ "signature": signature }))
        },
        &params_summary
    )
}

pub async fn request_airdrop_with_config(
    client: &RpcClient,
    pubkey: &Pubkey,
    lamports: u64,
    commitment: Option<CommitmentConfig>,
    recent_blockhash: Option<String>,
) -> McpResult<Value> {
    let config = RpcRequestAirdropConfig {
        commitment,
        recent_blockhash,
    };
    let signature = client
        .request_airdrop_with_config(pubkey, lamports, config)
        .await?;
    Ok(serde_json::json!({ "signature": signature }))
}

pub async fn get_stake_minimum_delegation(client: &RpcClient) -> McpResult<Value> {
    let minimum = client.get_stake_minimum_delegation().await?;
    Ok(serde_json::json!({ "minimum": minimum }))
}

pub async fn get_stake_minimum_delegation_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> McpResult<Value> {
    let minimum = client.get_stake_minimum_delegation_with_commitment(commitment).await?;

    Ok(serde_json::json!({ "minimum": minimum }))
}

pub async fn get_transaction_count(client: &RpcClient) -> McpResult<Value> {
    let count = client.get_transaction_count().await?;
    Ok(serde_json::json!({ "count": count }))
}

pub async fn get_transaction_count_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> McpResult<Value> {
    let count = client.get_transaction_count_with_commitment(commitment).await?;

    Ok(serde_json::json!({ "count": count }))
}

pub async fn get_latest_blockhash(client: &RpcClient) -> McpResult<Value> {
    let blockhash = client.get_latest_blockhash().await?;
    Ok(serde_json::json!({
        "blockhash": blockhash.to_string()
    }))
}

pub async fn get_latest_blockhash_with_commitment(
    client: &RpcClient,
    commitment: CommitmentConfig,
) -> McpResult<Value> {
    let blockhash = client.get_latest_blockhash_with_commitment(commitment).await?.0;

    Ok(serde_json::json!({
        "blockhash": blockhash.to_string()
    }))
}

pub async fn get_fee_for_message(
    client: &RpcClient,
    message: &Message,
) -> McpResult<Value> {

    let fee = client.get_fee_for_message(message).await?;
    Ok(serde_json::json!({ "fee": fee }))
}

/// Check if a blockhash is still valid for submitting transactions
pub async fn is_blockhash_valid(
    client: &RpcClient,
    blockhash: &str,
    commitment: Option<CommitmentConfig>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "isBlockhashValid";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("blockhash: {blockhash}")),
    );

    let blockhash_obj = match blockhash.parse() {
        Ok(hash) => hash,
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::validation(format!("Invalid blockhash format: {e}"))
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

    match client.is_blockhash_valid(&blockhash_obj, commitment.unwrap_or_default()).await {
        Ok(is_valid) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "valid": is_valid });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("blockhash validity: {is_valid}")),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

/// Get the current slot leader
pub async fn get_slot_leader(
    client: &RpcClient,
    commitment: Option<CommitmentConfig>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getSlotLeader";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    // Get current slot first, then get slot leaders for that range
    let current_slot_result = client.get_slot_with_commitment(commitment.unwrap_or_default()).await;
    
    match current_slot_result {
        Ok(slot) => {
            // Get slot leaders for the current slot (with limit 1)
            match client.get_slot_leaders(slot, 1).await {
                Ok(leaders) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    let leader = leaders.first().map(|l| l.to_string()).unwrap_or_else(|| "unknown".to_string());
                    let result = serde_json::json!({ "leader": leader });
                    
                    log_rpc_request_success(
                        request_id,
                        method,
                        duration,
                        Some(&format!("slot leader for slot {slot}: {leader}")),
                        Some(&client.url()),
                    );
                    
                    Ok(result)
                }
                Err(e) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    let error = McpError::from(e)
                        .with_request_id(request_id)
                        .with_method(method)
                        .with_rpc_url(client.url());
                    
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
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

/// Get the minimum ledger slot available
pub async fn minimum_ledger_slot(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "minimumLedgerSlot";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.minimum_ledger_slot().await {
        Ok(slot) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "slot": slot });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("minimum ledger slot: {slot}")),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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
/// Get the max slot seen from retransmit stage
pub async fn get_max_retransmit_slot(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getMaxRetransmitSlot";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_max_retransmit_slot().await {
        Ok(slot) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "slot": slot });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("max retransmit slot retrieved"),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

/// Get the max slot seen from shred insert
pub async fn get_max_shred_insert_slot(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getMaxShredInsertSlot";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_max_shred_insert_slot().await {
        Ok(slot) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "slot": slot });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("max shred insert slot retrieved"),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

/// Get highest snapshot slot  
pub async fn get_highest_snapshot_slot(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getHighestSnapshotSlot";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    match client.get_highest_snapshot_slot().await {
        Ok(snapshot_slot_info) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ 
                "full": snapshot_slot_info.full,
                "incremental": snapshot_slot_info.incremental
            });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("highest snapshot slot retrieved"),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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














/// Get recent blockhash (deprecated version of getLatestBlockhash) 
pub async fn get_recent_blockhash(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getRecentBlockhash";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    // Use the same underlying method as getLatestBlockhash
    match client.get_latest_blockhash().await {
        Ok(blockhash) => {
            let duration = start_time.elapsed().as_millis() as u64;
            // Return in the deprecated format for compatibility
            let result = serde_json::json!({
                "context": { "slot": 0 }, // Note: slot info not available in this deprecated method 
                "value": {
                    "blockhash": blockhash.to_string(),
                    "feeCalculator": {
                        "lamportsPerSignature": 5000 // Default fee, deprecated anyway
                    }
                }
            });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("recent blockhash retrieved (deprecated)"),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

/// Get fees (deprecated method)
pub async fn get_fees(client: &RpcClient) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getFees";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        None,
    );

    // Use the getLatestBlockhash method as basis for deprecated getFees
    match client.get_latest_blockhash().await {
        Ok(blockhash) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({
                "context": { "slot": 0 },
                "value": {
                    "blockhash": blockhash.to_string(),
                    "feeCalculator": {
                        "lamportsPerSignature": 5000 // Default fee for deprecated method
                    },
                    "lastValidSlot": 0,
                    "lastValidBlockHeight": 0
                }
            });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("fees retrieved (deprecated)"),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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
/// Get recent performance samples
pub async fn get_recent_performance_samples(
    client: &RpcClient,
    limit: Option<usize>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getRecentPerformanceSamples";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("limit: {limit:?}")),
    );

    match client.get_recent_performance_samples(limit).await {
        Ok(samples) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "samples": samples });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} performance samples retrieved", samples.len())),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

/// Get recent prioritization fees
pub async fn get_recent_prioritization_fees(
    client: &RpcClient,
    addresses: Option<Vec<String>>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getRecentPrioritizationFees";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("addresses: {addresses:?}")),
    );

    // Convert string addresses to Pubkeys if provided
    let pubkeys: Option<Vec<solana_sdk::pubkey::Pubkey>> = if let Some(addrs) = addresses {
        let parsed_keys: Result<Vec<_>, _> = addrs.iter()
            .map(|addr| addr.parse::<solana_sdk::pubkey::Pubkey>())
            .collect();
        match parsed_keys {
            Ok(keys) => Some(keys),
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                let error = McpError::InvalidParameter(format!("Invalid pubkey: {e}"))
                    .with_request_id(request_id)
                    .with_method(method)
                    .with_rpc_url(client.url());
                
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
        }
    } else {
        None
    };

    match client.get_recent_prioritization_fees(&pubkeys.unwrap_or_default()).await {
        Ok(fees) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "fees": fees });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} prioritization fees retrieved", fees.len())),
                Some(&client.url()),
            );
            
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let error = McpError::from(e)
                .with_request_id(request_id)
                .with_method(method)
                .with_rpc_url(client.url());
            
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

