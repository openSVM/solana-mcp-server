use anyhow::Result;
use mcp_sdk::types::{CallToolRequest, CallToolResponse, ToolResponseContent, ToolsListResponse, Tool};
use serde_json::json;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_request::TokenAccountsFilter,
};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;
use std::{str::FromStr, sync::Arc, collections::HashMap};

pub struct SolanaMcpServer {
    client: Arc<RpcClient>,
    resources: HashMap<String, Resource>,
}

struct Resource {
    name: String,
    description: String,
    mime_type: String,
    text: String,
}

impl SolanaMcpServer {
    pub fn new(client: RpcClient) -> Self {
        let mut resources = HashMap::new();
        
        // Initialize documentation resources
        
        // Core Documentation
        resources.insert(
            "solana://docs/core/accounts".to_string(),
            Resource {
                name: "Solana Account Model".to_string(),
                description: "Documentation for Solana's account model and structure".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/core/accounts.md").to_string(),
            },
        );
        resources.insert(
            "solana://docs/core/programs".to_string(),
            Resource {
                name: "Solana Programs".to_string(),
                description: "Documentation for Solana program architecture and development".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/core/programs.md").to_string(),
            },
        );
        resources.insert(
            "solana://docs/core/transactions".to_string(),
            Resource {
                name: "Solana Transactions".to_string(),
                description: "Documentation for Solana transaction structure and lifecycle".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/core/transactions.md").to_string(),
            },
        );

        // Development Guides
        resources.insert(
            "solana://docs/guides/development".to_string(),
            Resource {
                name: "Development Guide".to_string(),
                description: "Guide for developing on Solana".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/guides/development.md").to_string(),
            },
        );
        resources.insert(
            "solana://docs/guides/deployment".to_string(),
            Resource {
                name: "Deployment Guide".to_string(),
                description: "Guide for deploying Solana programs".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/guides/deployment.md").to_string(),
            },
        );
        resources.insert(
            "solana://docs/guides/programs".to_string(),
            Resource {
                name: "Program Development Guide".to_string(),
                description: "Comprehensive guide for Solana program development".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/guides/programs.md").to_string(),
            },
        );

        // RPC API Documentation
        resources.insert(
            "solana://docs/rpc/accounts".to_string(),
            Resource {
                name: "Account RPC Methods".to_string(),
                description: "Documentation for account-related RPC methods".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/rpc/accounts.rs").to_string(),
            },
        );
        resources.insert(
            "solana://docs/rpc/blocks".to_string(),
            Resource {
                name: "Block RPC Methods".to_string(),
                description: "Documentation for block-related RPC methods".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/rpc/blocks.rs").to_string(),
            },
        );
        resources.insert(
            "solana://docs/rpc/system".to_string(),
            Resource {
                name: "System RPC Methods".to_string(),
                description: "Documentation for system-related RPC methods".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/rpc/system.rs").to_string(),
            },
        );
        resources.insert(
            "solana://docs/rpc/tokens".to_string(),
            Resource {
                name: "Token RPC Methods".to_string(),
                description: "Documentation for token-related RPC methods".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/rpc/tokens.rs").to_string(),
            },
        );
        resources.insert(
            "solana://docs/rpc/transactions".to_string(),
            Resource {
                name: "Transaction RPC Methods".to_string(),
                description: "Documentation for transaction-related RPC methods".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/rpc/transactions.rs").to_string(),
            },
        );
        resources.insert(
            "solana://docs/rpc-api".to_string(),
            Resource {
                name: "Solana RPC API Documentation".to_string(),
                description: "Documentation for the Solana RPC API endpoints and usage".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/rpc/mod.rs").to_string(),
            },
        );

        Self {
            client: Arc::new(client),
            resources,
        }
    }

    pub fn list_tools(&self) -> Result<ToolsListResponse> {
        let tools = vec![
            Tool {
                name: "get_slot".to_string(),
                description: Some("Get current slot".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_slot_leaders".to_string(),
                description: Some("Get slot leaders".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "start_slot": {"type": "integer"},
                        "limit": {"type": "integer"}
                    },
                    "required": ["start_slot", "limit"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_block".to_string(),
                description: Some("Get block information".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "slot": {"type": "integer"}
                    },
                    "required": ["slot"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_block_height".to_string(),
                description: Some("Get current block height".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_balance".to_string(),
                description: Some("Get account balance".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pubkey": {"type": "string"}
                    },
                    "required": ["pubkey"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_account_info".to_string(),
                description: Some("Get detailed account information".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pubkey": {"type": "string"}
                    },
                    "required": ["pubkey"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_transaction".to_string(),
                description: Some("Get transaction details".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "signature": {"type": "string"}
                    },
                    "required": ["signature"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_health".to_string(),
                description: Some("Get node health status".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_version".to_string(),
                description: Some("Get node version information".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_identity".to_string(),
                description: Some("Get node identity".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_epoch_info".to_string(),
                description: Some("Get current epoch information".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_inflation_rate".to_string(),
                description: Some("Get current inflation rate".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_token_accounts_by_owner".to_string(),
                description: Some("Get token accounts owned by an address".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "owner": {"type": "string"}
                    },
                    "required": ["owner"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "process_sequential_thinking".to_string(),
                description: Some("Process a sequence of thinking steps with dependencies".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "steps": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string"},
                                    "description": {"type": "string"},
                                    "dependencies": {
                                        "type": "array",
                                        "items": {"type": "string"}
                                    }
                                },
                                "required": ["id", "description"]
                            }
                        }
                    },
                    "required": ["steps"],
                    "additionalProperties": false
                }),
            }
        ];

        Ok(ToolsListResponse {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    pub fn list_resources(&self) -> Result<serde_json::Value> {
        let resources = self.resources.iter().map(|(uri, resource)| {
            json!({
                "uri": uri,
                "name": resource.name,
                "description": resource.description,
                "mimeType": resource.mime_type
            })
        }).collect::<Vec<_>>();

        Ok(json!({
            "resources": resources
        }))
    }

    pub fn read_resource(&self, uri: &str) -> Result<serde_json::Value> {
        let resource = self.resources.get(uri)
            .ok_or_else(|| anyhow::anyhow!("Resource not found: {}", uri))?;

        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": resource.mime_type,
                "text": resource.text
            }]
        }))
    }

    pub async fn handle_tool_request(&self, request: CallToolRequest) -> Result<CallToolResponse> {
        match request.name.as_str() {
            // Slot Information
            "get_slot" => {
                let slot = self.client.get_slot().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": slot.to_string()
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": leaders
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": block
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_block_height" => {
                let height = self.client.get_block_height().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": height.to_string()
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": balance.to_string()
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": account
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": tx
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // System Information
            "get_health" => {
                self.client.get_health().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": "ok"
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_version" => {
                let version = self.client.get_version().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": version
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_identity" => {
                let identity = self.client.get_identity().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": identity.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // Epoch and Inflation
            "get_epoch_info" => {
                let epoch_info = self.client.get_epoch_info().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": epoch_info
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_inflation_rate" => {
                let inflation = self.client.get_inflation_rate().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": inflation
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": accounts
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "process_sequential_thinking" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let steps = params.get("steps")
                    .and_then(|s| s.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing steps parameter"))?;

                // Build dependency graph and process steps
                let mut completed_steps = Vec::new();
                let mut remaining_steps: Vec<_> = steps.iter().collect();

                while !remaining_steps.is_empty() {
                    let mut i = 0;
                    while i < remaining_steps.len() {
                        let step = &remaining_steps[i];
                        let step_id = step.get("id")
                            .and_then(|id| id.as_str())
                            .ok_or_else(|| anyhow::anyhow!("Step missing id"))?;
                        
                        // Check if dependencies are met
                        let dependencies = step.get("dependencies")
                            .and_then(|deps| deps.as_array())
                            .map(|deps| deps.iter()
                                .filter_map(|d| d.as_str())
                                .collect::<Vec<_>>())
                            .unwrap_or_default();

                        let deps_met = dependencies.iter()
                            .all(|dep| completed_steps.iter().any(|cs| cs == dep));

                        if deps_met {
                            completed_steps.push(step_id.to_string());
                            remaining_steps.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": {
                            "completed_steps": completed_steps
                        }
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            _ => anyhow::bail!("Tool not found"),
        }
    }
}

#[cfg(test)]
mod tests;
