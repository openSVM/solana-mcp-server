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
                .with_rpc_url(&client.url());
            
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
                .with_rpc_url(&client.url());
            
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
                .with_rpc_url(&client.url());
            
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
                .with_rpc_url(&client.url());
            
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
                .with_rpc_url(&client.url());
            
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
                .with_rpc_url(&client.url());
            
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
                .with_rpc_url(&client.url());
            
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
                .with_rpc_url(&client.url());
            
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
    
    let params_summary = format!("pubkey: {}, lamports: {}", pubkey, lamports);
    
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
