use anyhow::Result;
use mcp_sdk::types::{CallToolRequest, CallToolResponse, ToolResponseContent, ToolsListResponse, Tool};
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use std::{str::FromStr, sync::Arc, collections::HashMap};
use base64::{Engine as _, engine::general_purpose::STANDARD as base64};

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

        // RPC Optimization Guide
        resources.insert(
            "solana://docs/rpc-optimization".to_string(),
            Resource {
                name: "RPC Optimization Guide".to_string(),
                description: "Advanced optimization techniques for Solana RPC usage with performance metrics".to_string(),
                mime_type: "text/markdown".to_string(),
                text: include_str!("docs/guides/rpc_optimization.md").to_string(),
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
            // Account Methods
            Tool {
                name: "get_account_info".to_string(),
                description: Some("Returns all information associated with the account of provided Pubkey".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pubkey": {
                            "type": "string",
                            "description": "Pubkey of account to query, as base-58 encoded string"
                        }
                    },
                    "required": ["pubkey"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_multiple_accounts".to_string(),
                description: Some("Returns account information for a list of Pubkeys".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pubkeys": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "description": "Pubkey as base-58 encoded string"
                            }
                        }
                    },
                    "required": ["pubkeys"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_program_accounts".to_string(),
                description: Some("Returns all accounts owned by the provided program Pubkey".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "programId": {
                            "type": "string",
                            "description": "Pubkey of program to query, as base-58 encoded string"
                        }
                    },
                    "required": ["programId"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_balance".to_string(),
                description: Some("Returns the balance of the account of provided Pubkey".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pubkey": {
                            "type": "string",
                            "description": "Pubkey of account to query, as base-58 encoded string"
                        }
                    },
                    "required": ["pubkey"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_largest_accounts".to_string(),
                description: Some("Returns the 20 largest accounts, by lamport balance".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },

            // Block Methods
            Tool {
                name: "get_block".to_string(),
                description: Some("Returns identity and transaction information about a confirmed block".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "slot": {
                            "type": "integer",
                            "description": "Slot number"
                        }
                    },
                    "required": ["slot"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_blocks".to_string(),
                description: Some("Returns a list of confirmed blocks between two slots".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "start_slot": {
                            "type": "integer",
                            "description": "Start slot"
                        },
                        "end_slot": {
                            "type": "integer",
                            "description": "End slot"
                        }
                    },
                    "required": ["start_slot", "end_slot"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_blocks_with_limit".to_string(),
                description: Some("Returns a list of confirmed blocks starting at given slot".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "start_slot": {
                            "type": "integer",
                            "description": "Start slot"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of blocks to return"
                        }
                    },
                    "required": ["start_slot", "limit"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_block_time".to_string(),
                description: Some("Returns estimated production time of a block".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "slot": {
                            "type": "integer",
                            "description": "Slot number"
                        }
                    },
                    "required": ["slot"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_block_height".to_string(),
                description: Some("Returns the current block height".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_block_commitment".to_string(),
                description: Some("Returns commitment for particular block".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "slot": {
                            "type": "integer",
                            "description": "Slot number"
                        }
                    },
                    "required": ["slot"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_block_production".to_string(),
                description: Some("Returns recent block production information".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_first_available_block".to_string(),
                description: Some("Returns the slot of the lowest confirmed block that has not been purged from the ledger".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },

            // System Methods
            Tool {
                name: "get_health".to_string(),
                description: Some("Returns the current health of the node".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_version".to_string(),
                description: Some("Returns the current solana version running on the node".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_identity".to_string(),
                description: Some("Returns the identity pubkey for the current node".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_genesis_hash".to_string(),
                description: Some("Returns the genesis hash".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_slot".to_string(),
                description: Some("Returns the current slot the node is processing".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_slot_leader".to_string(),
                description: Some("Returns the current slot leader".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_slot_leaders".to_string(),
                description: Some("Returns the slot leaders for a slot range".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "start_slot": {
                            "type": "integer",
                            "description": "Start slot"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Number of slots to return"
                        }
                    },
                    "required": ["start_slot", "limit"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_cluster_nodes".to_string(),
                description: Some("Returns information about all the nodes participating in the cluster".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_vote_accounts".to_string(),
                description: Some("Returns the account info and associated stake for all the voting accounts".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },

            // Epoch and Inflation Methods
            Tool {
                name: "get_epoch_info".to_string(),
                description: Some("Returns information about the current epoch".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_epoch_schedule".to_string(),
                description: Some("Returns epoch schedule information".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_inflation_governor".to_string(),
                description: Some("Returns the current inflation governor".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_inflation_rate".to_string(),
                description: Some("Returns the specific inflation values for the current epoch".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_inflation_reward".to_string(),
                description: Some("Returns the inflation reward for a list of addresses for an epoch".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "addresses": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "description": "Pubkey as base-58 encoded string"
                            }
                        },
                        "epoch": {
                            "type": "integer",
                            "description": "Epoch for which to calculate rewards"
                        }
                    },
                    "required": ["addresses"],
                    "additionalProperties": false
                }),
            },

            // Token Methods
            Tool {
                name: "get_token_account_balance".to_string(),
                description: Some("Returns the token balance of an SPL Token account".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "accountAddress": {
                            "type": "string",
                            "description": "Pubkey of token account to query"
                        }
                    },
                    "required": ["accountAddress"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_token_accounts_by_delegate".to_string(),
                description: Some("Returns all SPL Token accounts by approved Delegate".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "delegateAddress": {
                            "type": "string",
                            "description": "Pubkey of delegate to query"
                        }
                    },
                    "required": ["delegateAddress"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_token_accounts_by_owner".to_string(),
                description: Some("Returns all SPL Token accounts by token owner".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "ownerAddress": {
                            "type": "string",
                            "description": "Pubkey of token account owner to query"
                        }
                    },
                    "required": ["ownerAddress"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_token_largest_accounts".to_string(),
                description: Some("Returns the 20 largest accounts of a particular SPL Token type".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "mint": {
                            "type": "string",
                            "description": "Pubkey of token mint to query"
                        }
                    },
                    "required": ["mint"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_token_supply".to_string(),
                description: Some("Returns the total supply of an SPL Token type".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "mint": {
                            "type": "string",
                            "description": "Pubkey of token mint to query"
                        }
                    },
                    "required": ["mint"],
                    "additionalProperties": false
                }),
            },

            // Transaction Methods
            Tool {
                name: "get_transaction".to_string(),
                description: Some("Returns transaction details for confirmed transaction".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "signature": {
                            "type": "string",
                            "description": "Transaction signature as base-58 encoded string"
                        }
                    },
                    "required": ["signature"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_signatures_for_address".to_string(),
                description: Some("Returns signatures for confirmed transactions that include the given address in their accountKeys list".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "string",
                            "description": "Account address as base-58 encoded string"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of signatures to return"
                        }
                    },
                    "required": ["address"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_signature_statuses".to_string(),
                description: Some("Returns the statuses of a list of signatures".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "signatures": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "description": "Transaction signature as base-58 encoded string"
                            }
                        }
                    },
                    "required": ["signatures"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_transaction_count".to_string(),
                description: Some("Returns the current Transaction count from the ledger".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "simulate_transaction".to_string(),
                description: Some("Simulate sending a transaction".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "transaction": {
                            "type": "string",
                            "description": "Transaction, as base-58 encoded string"
                        }
                    },
                    "required": ["transaction"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "send_transaction".to_string(),
                description: Some("Send a transaction".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "transaction": {
                            "type": "string",
                            "description": "Fully-signed transaction, as base-58 encoded string"
                        }
                    },
                    "required": ["transaction"],
                    "additionalProperties": false
                }),
            },

            // Other Methods
            Tool {
                name: "get_fee_for_message".to_string(),
                description: Some("Get the fee the network will charge for a message".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message, as base-58 encoded string"
                        }
                    },
                    "required": ["message"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_latest_blockhash".to_string(),
                description: Some("Returns the latest blockhash".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "is_blockhash_valid".to_string(),
                description: Some("Returns whether a blockhash is still valid or not".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "blockhash": {
                            "type": "string",
                            "description": "Blockhash to validate, as base-58 encoded string"
                        }
                    },
                    "required": ["blockhash"],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_stake_minimum_delegation".to_string(),
                description: Some("Returns the stake minimum delegation, in lamports".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "get_supply".to_string(),
                description: Some("Returns information about the current supply".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "request_airdrop".to_string(),
                description: Some("Requests an airdrop of lamports to a Pubkey".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pubkey": {
                            "type": "string",
                            "description": "Pubkey of account to receive lamports, as base-58 encoded string"
                        },
                        "lamports": {
                            "type": "integer",
                            "description": "Amount of lamports to airdrop"
                        }
                    },
                    "required": ["pubkey", "lamports"],
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
            "get_signatures_for_address" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let address_str = params.get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing address parameter"))?;
                let address = Pubkey::from_str(address_str)?;
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(1000);
                let signatures = self.client.get_signatures_for_address(&address).await?;
                let signatures = signatures.into_iter().take(limit as usize).collect::<Vec<_>>();
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": signatures
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_signature_statuses" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let signatures = params.get("signatures")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing signatures parameter"))?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.parse())
                    .collect::<Result<Vec<_>, _>>()?;
                let statuses = self.client.get_signature_statuses(&signatures).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": statuses
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_transaction_count" => {
                let count = self.client.get_transaction_count().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": count.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "simulate_transaction" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let tx_str = params.get("transaction")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing transaction parameter"))?;
                let tx_bytes = base64.decode(tx_str)?;
                let tx: Transaction = bincode::deserialize(&tx_bytes)?;
                let result = self.client.simulate_transaction(&tx).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": result
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "send_transaction" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let tx_str = params.get("transaction")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing transaction parameter"))?;
                let tx_bytes = base64.decode(tx_str)?;
                let tx: Transaction = bincode::deserialize(&tx_bytes)?;
                let signature = self.client.send_transaction(&tx).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": signature.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // Other Methods
            "get_fee_for_message" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let message_str = params.get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing message parameter"))?;
                let message_bytes = base64.decode(message_str)?;
                let message: Message = bincode::deserialize(&message_bytes)?;
                let fee = self.client.get_fee_for_message(&message).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": fee.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_latest_blockhash" => {
                let blockhash = self.client.get_latest_blockhash().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": blockhash.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "is_blockhash_valid" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let blockhash_str = params.get("blockhash")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing blockhash parameter"))?;
                let blockhash: Hash = blockhash_str.parse()?;
                let valid = self.client.is_blockhash_valid(&blockhash, CommitmentConfig::default()).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": valid
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_stake_minimum_delegation" => {
                let min_delegation = self.client.get_stake_minimum_delegation().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": min_delegation.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_supply" => {
                let supply = self.client.supply().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": supply
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "request_airdrop" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let pubkey_str = params.get("pubkey")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing pubkey parameter"))?;
                let lamports = params.get("lamports")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing lamports parameter"))?;
                let pubkey = Pubkey::from_str(pubkey_str)?;
                let signature = self.client.request_airdrop(&pubkey, lamports).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": signature.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // System Methods
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
            "get_genesis_hash" => {
                let genesis_hash = self.client.get_genesis_hash().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": genesis_hash.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_slot_leader" => {
                let leaders = self.client.get_slot_leaders(self.client.get_slot().await?, 1).await?;
                let leader = leaders.first().ok_or_else(|| anyhow::anyhow!("No slot leader found"))?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": leader.to_string()
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_cluster_nodes" => {
                let nodes = self.client.get_cluster_nodes().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": nodes
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_vote_accounts" => {
                let accounts = self.client.get_vote_accounts().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": accounts
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // Epoch Methods
            "get_epoch_info" => {
                let info = self.client.get_epoch_info().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": info
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_epoch_schedule" => {
                let schedule = self.client.get_epoch_schedule().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": schedule
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_inflation_governor" => {
                let governor = self.client.get_inflation_governor().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": governor
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_inflation_rate" => {
                let rate = self.client.get_inflation_rate().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": rate
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_inflation_reward" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let addresses = params.get("addresses")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing addresses parameter"))?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| Pubkey::from_str(s))
                    .collect::<Result<Vec<_>, _>>()?;
                let epoch = params.get("epoch").and_then(|v| v.as_u64());
                let rewards = self.client.get_inflation_reward(&addresses, epoch).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": rewards
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // Token Methods
            "get_token_account_balance" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let account = Pubkey::from_str(params.get("accountAddress")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing accountAddress parameter"))?)?;
                let balance = self.client.get_token_account_balance(&account).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": balance
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_token_accounts_by_delegate" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let delegate = Pubkey::from_str(params.get("delegateAddress")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing delegateAddress parameter"))?)?;
                use solana_client::rpc_request::TokenAccountsFilter;
                let accounts = self.client.get_token_accounts_by_delegate(&delegate, TokenAccountsFilter::ProgramId(spl_token::id())).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": accounts
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_token_accounts_by_owner" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let owner = Pubkey::from_str(params.get("ownerAddress")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing ownerAddress parameter"))?)?;
                use solana_client::rpc_request::TokenAccountsFilter;
                let accounts = self.client.get_token_accounts_by_owner(&owner, TokenAccountsFilter::ProgramId(spl_token::id())).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": accounts
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_token_largest_accounts" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let mint = Pubkey::from_str(params.get("mint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing mint parameter"))?)?;
                let accounts = self.client.get_token_largest_accounts(&mint).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": accounts
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_token_supply" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let mint = Pubkey::from_str(params.get("mint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing mint parameter"))?)?;
                let supply = self.client.get_token_supply(&mint).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": supply
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // Block Methods
            "get_blocks" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let start_slot = params.get("start_slot")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing start_slot parameter"))?;
                let end_slot = params.get("end_slot")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing end_slot parameter"))?;
                let blocks = self.client.get_blocks(start_slot, Some(end_slot)).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": blocks
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": blocks
                    }).to_string() }],
                    is_error: None,
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
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": time
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_block_production" => {
                let production = self.client.get_block_production().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": production
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_first_available_block" => {
                let block = self.client.get_first_available_block().await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": block
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            // Account Methods
            "get_multiple_accounts" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let pubkeys = params.get("pubkeys")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing pubkeys parameter"))?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| Pubkey::from_str(s))
                    .collect::<Result<Vec<_>, _>>()?;
                let accounts = self.client.get_multiple_accounts(&pubkeys).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": accounts
                    }).to_string() }],
                    is_error: None,
                    meta: None,
                })
            }
            "get_program_accounts" => {
                let params = request.arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                let program_id = Pubkey::from_str(params.get("programId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing programId parameter"))?)?;
                let accounts = self.client.get_program_accounts(&program_id).await?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text: json!({
                        "result": accounts
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
