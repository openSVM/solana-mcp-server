use crate::error::{McpError, McpResult};
use crate::logging::{log_rpc_request_start, log_rpc_request_success, log_rpc_request_failure, new_request_id};
use serde_json::Value;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::RpcFilterType,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::time::Instant;

/// Get account balance for a given public key
pub async fn get_balance(client: &RpcClient, pubkey: &Pubkey) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getBalance";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkey: {}", pubkey)),
    );

    match client.get_balance(pubkey).await {
        Ok(balance) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "balance": balance });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("balance retrieved"),
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

/// Get account information for a given public key
pub async fn get_account_info(client: &RpcClient, pubkey: &Pubkey) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getAccountInfo";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkey: {}", pubkey)),
    );

    match client.get_account(pubkey).await {
        Ok(account) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "account": account });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("account info retrieved"),
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

/// Get account information with configuration options
pub async fn get_account_info_with_config(
    client: &RpcClient,
    pubkey: &Pubkey,
    commitment: Option<CommitmentConfig>,
    encoding: Option<UiAccountEncoding>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getAccountInfoWithConfig";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkey: {}, commitment: {:?}, encoding: {:?}", pubkey, commitment, encoding)),
    );

    let config = RpcAccountInfoConfig {
        encoding,
        commitment,
        data_slice: None,
        min_context_slot: None,
    };

    match client.get_account_with_config(pubkey, config).await {
        Ok(account) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "account": account });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("account info with config retrieved"),
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

/// Get multiple accounts information
pub async fn get_multiple_accounts(client: &RpcClient, pubkeys: &[Pubkey]) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getMultipleAccounts";
    
    if pubkeys.is_empty() {
        let error = McpError::validation("At least one pubkey is required")
            .with_request_id(request_id)
            .with_method(method)
            .with_parameter("pubkeys");
        
        return Err(error);
    }
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkeys_count: {}", pubkeys.len())),
    );

    match client.get_multiple_accounts(pubkeys).await {
        Ok(accounts) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "accounts": accounts });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} accounts retrieved", pubkeys.len())),
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

/// Get multiple accounts with configuration options
pub async fn get_multiple_accounts_with_config(
    client: &RpcClient,
    pubkeys: &[Pubkey],
    commitment: Option<CommitmentConfig>,
    encoding: Option<UiAccountEncoding>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getMultipleAccountsWithConfig";
    
    if pubkeys.is_empty() {
        let error = McpError::validation("At least one pubkey is required")
            .with_request_id(request_id)
            .with_method(method)
            .with_parameter("pubkeys");
        
        return Err(error);
    }
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkeys_count: {}, commitment: {:?}, encoding: {:?}", 
                     pubkeys.len(), commitment, encoding)),
    );

    let config = RpcAccountInfoConfig {
        encoding,
        commitment,
        data_slice: None,
        min_context_slot: None,
    };
    match client.get_multiple_accounts_with_config(pubkeys, config).await {
        Ok(accounts) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "accounts": accounts });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} accounts with config retrieved", pubkeys.len())),
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

/// Get all accounts owned by a program
pub async fn get_program_accounts(client: &RpcClient, program_id: &Pubkey) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getProgramAccounts";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("program_id: {}", program_id)),
    );

    match client.get_program_accounts(program_id).await {
        Ok(accounts) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "accounts": accounts });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} program accounts retrieved", accounts.len())),
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

/// Get program accounts with configuration and filters
pub async fn get_program_accounts_with_config(
    client: &RpcClient,
    program_id: &Pubkey,
    commitment: Option<CommitmentConfig>,
    encoding: Option<UiAccountEncoding>,
    filters: Vec<RpcFilterType>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getProgramAccountsWithConfig";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("program_id: {}, filters_count: {}, commitment: {:?}, encoding: {:?}", 
                     program_id, filters.len(), commitment, encoding)),
    );

    let config = RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding,
            commitment,
            data_slice: None,
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None, // Use default sorting behavior
    };
    match client.get_program_accounts_with_config(program_id, config).await {
        Ok(accounts) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "accounts": accounts });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} filtered program accounts retrieved", accounts.len())),
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

/// Get the largest accounts by balance
pub async fn get_largest_accounts(
    client: &RpcClient,
    filter: Option<solana_client::rpc_config::RpcLargestAccountsFilter>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getLargestAccounts";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("filter: {:?}", filter)),
    );

    let config = solana_client::rpc_config::RpcLargestAccountsConfig {
        commitment: None,
        filter,
        sort_results: None, // Use default sorting behavior
    };

    match client.get_largest_accounts_with_config(config).await {
        Ok(accounts) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "accounts": accounts });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("largest accounts retrieved"),
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

/// Get minimum balance required for rent exemption
pub async fn get_minimum_balance_for_rent_exemption(
    client: &RpcClient,
    data_len: usize,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getMinimumBalanceForRentExemption";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("data_len: {}", data_len)),
    );

    match client.get_minimum_balance_for_rent_exemption(data_len).await {
        Ok(lamports) => {
            let duration = start_time.elapsed().as_millis() as u64;
            let result = serde_json::json!({ "lamports": lamports });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some("minimum balance calculated"),
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

/// Get account info with context (slot information)
pub async fn get_account_info_and_context(
    client: &RpcClient,
    pubkey: &Pubkey,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getAccountInfoAndContext";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkey: {}", pubkey)),
    );

    match client.get_account_with_commitment(pubkey, CommitmentConfig::confirmed()).await {
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
                Some("account info with context retrieved"),
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

/// Get account balance with context (slot information)  
pub async fn get_balance_and_context(
    client: &RpcClient,
    pubkey: &Pubkey,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getBalanceAndContext";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkey: {}", pubkey)),
    );

    match client.get_balance_with_commitment(pubkey, CommitmentConfig::confirmed()).await {
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
                Some("balance with context retrieved"),
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

/// Get multiple accounts with context (slot information)
pub async fn get_multiple_accounts_and_context(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getMultipleAccountsAndContext";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("pubkeys: {} accounts", pubkeys.len())),
    );

    match client.get_multiple_accounts_with_commitment(pubkeys, CommitmentConfig::confirmed()).await {
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
                Some(&format!("{} accounts with context retrieved", pubkeys.len())),
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

/// Get program accounts with context (slot information)
pub async fn get_program_accounts_and_context(
    client: &RpcClient,
    program_id: &Pubkey,
    config: Option<RpcProgramAccountsConfig>,
) -> McpResult<Value> {
    let request_id = new_request_id();
    let start_time = Instant::now();
    let method = "getProgramAccountsAndContext";
    
    log_rpc_request_start(
        request_id,
        method,
        Some(&client.url()),
        Some(&format!("program_id: {}", program_id)),
    );

    let default_config = RpcProgramAccountsConfig {
        filters: None,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            data_slice: None,
            min_context_slot: None,
        },
        with_context: Some(true),
        sort_results: None,
    };

    let final_config = config.unwrap_or(default_config);

    match client.get_program_accounts_with_config(program_id, final_config).await {
        Ok(accounts) => {
            let duration = start_time.elapsed().as_millis() as u64;
            
            let result = serde_json::json!({
                "accounts": accounts
            });
            
            log_rpc_request_success(
                request_id,
                method,
                duration,
                Some(&format!("{} program accounts with context retrieved", accounts.len())),
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
