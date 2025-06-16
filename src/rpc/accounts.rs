use anyhow::Result;
use serde_json::Value;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::RpcFilterType,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

pub async fn get_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<Value> {
    let balance = client.get_balance(pubkey).await?;
    Ok(serde_json::json!({ "balance": balance }))
}

pub async fn get_account_info(client: &RpcClient, pubkey: &Pubkey) -> Result<Value> {
    let account = client.get_account(pubkey).await?;
    Ok(serde_json::json!({ "account": account }))
}

pub async fn get_account_info_with_config(
    client: &RpcClient,
    pubkey: &Pubkey,
    commitment: Option<CommitmentConfig>,
    encoding: Option<UiAccountEncoding>,
) -> Result<Value> {
    let config = RpcAccountInfoConfig {
        encoding,
        commitment,
        data_slice: None,
        min_context_slot: None,
    };
    let account = client.get_account_with_config(pubkey, config).await?;
    Ok(serde_json::json!({ "account": account }))
}

pub async fn get_multiple_accounts(client: &RpcClient, pubkeys: &[Pubkey]) -> Result<Value> {
    let accounts = client.get_multiple_accounts(pubkeys).await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_multiple_accounts_with_config(
    client: &RpcClient,
    pubkeys: &[Pubkey],
    commitment: Option<CommitmentConfig>,
    encoding: Option<UiAccountEncoding>,
) -> Result<Value> {
    let config = RpcAccountInfoConfig {
        encoding,
        commitment,
        data_slice: None,
        min_context_slot: None,
    };
    let accounts = client
        .get_multiple_accounts_with_config(pubkeys, config)
        .await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_program_accounts(client: &RpcClient, program_id: &Pubkey) -> Result<Value> {
    let accounts = client.get_program_accounts(program_id).await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_program_accounts_with_config(
    client: &RpcClient,
    program_id: &Pubkey,
    commitment: Option<CommitmentConfig>,
    encoding: Option<UiAccountEncoding>,
    filters: Vec<RpcFilterType>,
) -> Result<Value> {
    let config = RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding,
            commitment,
            data_slice: None,
            min_context_slot: None,
        },
        with_context: None,
    };
    let accounts = client
        .get_program_accounts_with_config(program_id, config)
        .await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_largest_accounts(
    client: &RpcClient,
    filter: Option<solana_client::rpc_config::RpcLargestAccountsFilter>,
) -> Result<Value> {
    let config = solana_client::rpc_config::RpcLargestAccountsConfig {
        commitment: None,
        filter,
    };
    let accounts = client.get_largest_accounts_with_config(config).await?;
    Ok(serde_json::json!({ "accounts": accounts }))
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
