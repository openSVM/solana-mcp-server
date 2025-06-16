use anyhow::Result;
use serde_json::Value;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use spl_token::id as spl_token_program_id;

pub async fn get_token_accounts_by_owner(client: &RpcClient, owner: &Pubkey) -> Result<Value> {
    let accounts = client
        .get_token_accounts_by_owner(
            owner,
            TokenAccountsFilter::ProgramId(spl_token_program_id()),
        )
        .await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_token_accounts_by_owner_with_commitment(
    client: &RpcClient,
    owner: &Pubkey,
    filter: TokenAccountsFilter,
    _commitment: CommitmentConfig,
    _encoding: Option<UiAccountEncoding>,
) -> Result<Value> {
    let accounts = client.get_token_accounts_by_owner(owner, filter).await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_token_accounts_by_delegate(
    client: &RpcClient,
    delegate: &Pubkey,
    filter: TokenAccountsFilter,
) -> Result<Value> {
    let accounts = client
        .get_token_accounts_by_delegate(delegate, filter)
        .await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_token_accounts_by_delegate_with_commitment(
    client: &RpcClient,
    delegate: &Pubkey,
    filter: TokenAccountsFilter,
    _commitment: CommitmentConfig,
    _encoding: Option<UiAccountEncoding>,
) -> Result<Value> {
    let accounts = client
        .get_token_accounts_by_delegate(delegate, filter)
        .await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_token_supply(client: &RpcClient, mint: &Pubkey) -> Result<Value> {
    let supply = client.get_token_supply(mint).await?;
    Ok(serde_json::json!({ "supply": supply }))
}

pub async fn get_token_supply_with_commitment(
    client: &RpcClient,
    mint: &Pubkey,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let supply = client
        .get_token_supply_with_commitment(mint, commitment)
        .await?;
    Ok(serde_json::json!({ "supply": supply }))
}

pub async fn get_token_largest_accounts(client: &RpcClient, mint: &Pubkey) -> Result<Value> {
    let accounts = client.get_token_largest_accounts(mint).await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_token_largest_accounts_with_commitment(
    client: &RpcClient,
    mint: &Pubkey,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let accounts = client
        .get_token_largest_accounts_with_commitment(mint, commitment)
        .await?;
    Ok(serde_json::json!({ "accounts": accounts }))
}

pub async fn get_token_account_balance(client: &RpcClient, account: &Pubkey) -> Result<Value> {
    let balance = client.get_token_account_balance(account).await?;
    Ok(serde_json::json!({ "balance": balance }))
}

pub async fn get_token_account_balance_with_commitment(
    client: &RpcClient,
    account: &Pubkey,
    commitment: CommitmentConfig,
) -> Result<Value> {
    let balance = client
        .get_token_account_balance_with_commitment(account, commitment)
        .await?;
    Ok(serde_json::json!({ "balance": balance }))
}
