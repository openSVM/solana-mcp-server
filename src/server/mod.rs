use anyhow::Result;
use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;
use crate::{rpc, tools::get_tools};

pub struct SolanaMcpServer {
    client: RpcClient,
}

impl SolanaMcpServer {
    pub fn new(client: RpcClient) -> Self {
        Self { client }
    }

    pub fn list_tools(&self) -> Result<Value> {
        Ok(serde_json::json!({
            "tools": get_tools()
        }))
    }

    pub async fn handle_tool_request(&self, request: mcp_sdk::types::CallToolRequest) -> Result<Value> {
        match request.name.as_str() {
            // Slot & Block Methods
            "get_slot" => rpc::blocks::get_slot(&self.client).await,
            "get_slot_leaders" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let start_slot = args.get("start_slot").and_then(|v| v.as_u64()).ok_or_else(|| anyhow::anyhow!("start_slot required"))?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).ok_or_else(|| anyhow::anyhow!("limit required"))?;
                rpc::blocks::get_slot_leaders(&self.client, start_slot, limit).await
            },
            "get_block" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let slot = args.get("slot").and_then(|v| v.as_u64()).ok_or_else(|| anyhow::anyhow!("slot required"))?;
                rpc::blocks::get_block(&self.client, slot).await
            },
            "get_block_height" => rpc::blocks::get_block_height(&self.client).await,
            "get_block_production" => {
                let args = request.arguments;
                let identity = args.as_ref().and_then(|args| args.get("identity").and_then(|v| v.as_str()).map(|s| s.to_string()));
                let first_slot = args.as_ref().and_then(|args| args.get("first_slot").and_then(|v| v.as_u64()));
                let last_slot = args.as_ref().and_then(|args| args.get("last_slot").and_then(|v| v.as_u64()));
                rpc::blocks::get_block_production(&self.client, identity, first_slot, last_slot).await
            },
            "get_blocks" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let start_slot = args.get("start_slot").and_then(|v| v.as_u64()).ok_or_else(|| anyhow::anyhow!("start_slot required"))?;
                let end_slot = args.get("end_slot").and_then(|v| v.as_u64());
                rpc::blocks::get_blocks(&self.client, start_slot, end_slot).await
            },
            // Account Methods
            "get_balance" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let pubkey = args.get("pubkey").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("pubkey required"))?;
                let pubkey = pubkey.parse()?;
                rpc::accounts::get_balance(&self.client, &pubkey).await
            },
            "get_account_info" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let pubkey = args.get("pubkey").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("pubkey required"))?;
                let pubkey = pubkey.parse()?;
                rpc::accounts::get_account_info(&self.client, &pubkey).await
            },
            "get_multiple_accounts" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let pubkeys = args.get("pubkeys")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("pubkeys array required"))?
                    .iter()
                    .map(|v| v.as_str().ok_or_else(|| anyhow::anyhow!("invalid pubkey")).and_then(|s| Ok(s.parse()?)))
                    .collect::<Result<Vec<_>>>()?;
                rpc::accounts::get_multiple_accounts(&self.client, &pubkeys).await
            },
            "get_program_accounts" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let program_id = args.get("program_id").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("program_id required"))?;
                let program_id = program_id.parse()?;
                rpc::accounts::get_program_accounts(&self.client, &program_id).await
            },
            // Transaction Methods
            "get_transaction" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let signature = args.get("signature").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("signature required"))?;
                let signature = signature.parse()?;
                rpc::transactions::get_transaction(&self.client, &signature).await
            },
            "get_signatures_for_address" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let address = args.get("address").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("address required"))?;
                let address = address.parse()?;
                let before = args.get("before").and_then(|v| v.as_str()).map(|s| s.parse()).transpose()?;
                let until = args.get("until").and_then(|v| v.as_str()).map(|s| s.parse()).transpose()?;
                let limit = args.get("limit").and_then(|v| v.as_u64());
                rpc::transactions::get_signatures_for_address(&self.client, &address, before, until, limit).await
            },
            "send_transaction" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let transaction = args.get("transaction").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("transaction required"))?;
                let encoding = args.get("encoding").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("encoding required"))?;
                rpc::transactions::send_transaction(&self.client, transaction, encoding).await
            },
            // System Info Methods
            "get_health" => rpc::system::get_health(&self.client).await,
            "get_version" => rpc::system::get_version(&self.client).await,
            "get_identity" => rpc::system::get_identity(&self.client).await,
            "get_cluster_nodes" => rpc::system::get_cluster_nodes(&self.client).await,
            "get_epoch_info" => rpc::system::get_epoch_info(&self.client).await,
            "get_epoch_schedule" => rpc::system::get_epoch_schedule(&self.client).await,
            "get_inflation_rate" => rpc::system::get_inflation_rate(&self.client).await,
            "get_inflation_governor" => rpc::system::get_inflation_governor(&self.client).await,
            // Token Methods
            "get_token_accounts_by_owner" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let owner = args.get("owner").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("owner required"))?;
                let owner = owner.parse()?;
                rpc::tokens::get_token_accounts_by_owner(&self.client, &owner).await
            },
            "get_token_supply" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let mint = args.get("mint").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("mint required"))?;
                let mint = mint.parse()?;
                rpc::tokens::get_token_supply(&self.client, &mint).await
            },
            "get_token_largest_accounts" => {
                let args = request.arguments.ok_or_else(|| anyhow::anyhow!("Invalid arguments"))?;
                let mint = args.get("mint").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("mint required"))?;
                let mint = mint.parse()?;
                rpc::tokens::get_token_largest_accounts(&self.client, &mint).await
            },
            _ => Err(anyhow::anyhow!("Tool not found: {}", request.name)),
        }
    }
}
