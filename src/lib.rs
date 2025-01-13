use anyhow::Result;
use mcp_sdk::{
    transport::{JsonRpcRequest, JsonRpcResponse},
    types::{CallToolRequest, CallToolResponse, ResourceContents, ToolResponseContent},
};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_request::TokenAccountsFilter,
};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;
use std::{str::FromStr, sync::Arc};
use url::Url;

pub struct SolanaMcpServer {
    client: Arc<RpcClient>,
}

impl SolanaMcpServer {
    pub fn new(client: RpcClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    pub async fn handle_read_resource(&self, uri: Url) -> Result<ResourceContents> {
        match uri.as_str() {
            "solana://supply" => Ok(ResourceContents {
                uri,
                mime_type: Some("application/json".to_string()),
            }),
            "solana://inflation" => Ok(ResourceContents {
                uri,
                mime_type: Some("application/json".to_string()),
            }),
            "solana://validators" => Ok(ResourceContents {
                uri,
                mime_type: Some("application/json".to_string()),
            }),
            "solana://epoch" => Ok(ResourceContents {
                uri,
                mime_type: Some("application/json".to_string()),
            }),
            "solana://fees" => Ok(ResourceContents {
                uri,
                mime_type: Some("application/json".to_string()),
            }),
            "solana://stake" => Ok(ResourceContents {
                uri,
                mime_type: Some("application/json".to_string()),
            }),
            "solana://tokens" => Ok(ResourceContents {
                uri,
                mime_type: Some("application/json".to_string()),
            }),
            _ => anyhow::bail!("Resource not found"),
        }
    }

    pub async fn handle_tool_request(&self, request: CallToolRequest) -> Result<CallToolResponse> {
        match request.name.as_str() {
            // Slot Information
            "get_slot" => {
                let slot = self.client.get_slot().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: slot.to_string() }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_slot_leaders" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let start_slot = params.get("start_slot")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing start_slot parameter"))?;
                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing limit parameter"))?;
                let leaders = self.client.get_slot_leaders(start_slot, limit).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", leaders) }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Block Information
            "get_block" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let slot = params.get("slot")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing slot parameter"))?;
                let block = self.client.get_block(slot).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", block) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_block_height" => {
                let height = self.client.get_block_height().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: height.to_string() }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_block_production" => {
                let production = self.client.get_block_production().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", production) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_blocks" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let start_slot = params.get("start_slot")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing start_slot parameter"))?;
                let end_slot = params.get("end_slot")
                    .and_then(|v| v.as_u64());
                let blocks = self.client.get_blocks(start_slot, end_slot).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", blocks) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_blocks_with_limit" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let start_slot = params.get("start_slot")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing start_slot parameter"))?;
                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing limit parameter"))?;
                let blocks = self.client.get_blocks_with_limit(start_slot, limit.try_into()?).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", blocks) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_block_time" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let slot = params.get("slot")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing slot parameter"))?;
                let time = self.client.get_block_time(slot).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: time.to_string() }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Account Information
            "get_balance" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let pubkey_str = params.get("pubkey")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
                let pubkey = Pubkey::from_str(pubkey_str)?;
                let balance = self.client.get_balance(&pubkey).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: balance.to_string() }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_account_info" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let pubkey_str = params.get("pubkey")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
                let pubkey = Pubkey::from_str(pubkey_str)?;
                let account = self.client.get_account(&pubkey).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", account) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_multiple_accounts" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let pubkeys: Vec<Pubkey> = params.get("pubkeys")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing pubkeys parameter"))?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Pubkey::from_str(s).ok())
                    .collect();
                let accounts = self.client.get_multiple_accounts(&pubkeys).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", accounts) }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Transaction Information
            "get_transaction" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let signature_str = params.get("signature")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing signature parameter"))?;
                let signature = signature_str.parse()?;
                let tx = self.client.get_transaction(&signature, UiTransactionEncoding::Json).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", tx) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_signatures_for_address" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let address_str = params.get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing address parameter"))?;
                let address = Pubkey::from_str(address_str)?;
                let signatures = self.client.get_signatures_for_address(&address).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", signatures) }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // System Information
            "get_health" => {
                self.client.get_health().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: "ok".to_string() }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_version" => {
                let version = self.client.get_version().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", version) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_identity" => {
                let identity = self.client.get_identity().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: identity.to_string() }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_genesis_hash" => {
                let hash = self.client.get_genesis_hash().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: hash.to_string() }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Epoch and Inflation
            "get_epoch_info" => {
                let epoch_info = self.client.get_epoch_info().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", epoch_info) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_inflation_rate" => {
                let inflation = self.client.get_inflation_rate().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", inflation) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_inflation_governor" => {
                let governor = self.client.get_inflation_governor().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", governor) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_inflation_reward" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let addresses: Vec<Pubkey> = params.get("addresses")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing addresses parameter"))?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Pubkey::from_str(s).ok())
                    .collect();
                let epoch = params.get("epoch")
                    .and_then(|v| v.as_u64());
                let rewards = self.client.get_inflation_reward(&addresses, epoch).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", rewards) }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Token Information
            "get_token_accounts_by_owner" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let owner_str = params.get("owner")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing owner parameter"))?;
                let owner = Pubkey::from_str(owner_str)?;
                let accounts = self.client.get_token_accounts_by_owner(
                    &owner,
                    TokenAccountsFilter::ProgramId(spl_token::id()),
                ).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", accounts) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_token_largest_accounts" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let mint_str = params.get("mint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing mint parameter"))?;
                let mint = Pubkey::from_str(mint_str)?;
                let accounts = self.client.get_token_largest_accounts(&mint).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", accounts) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_token_supply" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let mint_str = params.get("mint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing mint parameter"))?;
                let mint = Pubkey::from_str(mint_str)?;
                let supply = self.client.get_token_supply(&mint).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", supply) }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Program Information
            "get_program_accounts" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let program_str = params.get("program")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing program parameter"))?;
                let program = Pubkey::from_str(program_str)?;
                let accounts = self.client.get_program_accounts(&program).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", accounts) }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Supply Information
            "get_supply" => {
                let uri = Url::parse("solana://supply")?;
                let resource = ResourceContents {
                    uri,
                    mime_type: Some("application/json".to_string()),
                };
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Resource { resource }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            // Validator Information
            "get_vote_accounts" => {
                let accounts = self.client.get_vote_accounts().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", accounts) }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            "get_cluster_nodes" => {
                let nodes = self.client.get_cluster_nodes().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: format!("{:?}", nodes) }],
                    is_error: Some(false),
                    meta: None,
                })
            }

            _ => anyhow::bail!("Tool not found"),
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        match request.method.as_str() {
            "call_tool" => {
                let params = request.params.ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                let tool_request: CallToolRequest = serde_json::from_value(params)?;
                let response = self.handle_tool_request(tool_request).await?;
                Ok(JsonRpcResponse {
                    jsonrpc: Default::default(),
                    id: request.id,
                    result: Some(serde_json::to_value(response)?),
                    error: None,
                })
            }
            _ => anyhow::bail!("Method not found"),
        }
    }
}

#[cfg(test)]
mod tests;
