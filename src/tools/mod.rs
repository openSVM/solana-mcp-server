use serde_json::json;
use mcp_sdk::types::Tool;

pub fn get_tools() -> Vec<Tool> {
    vec![
        // Slot & Block Methods
        Tool {
            name: "get_slot".to_string(),
            description: Some("Get current slot".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_slot_leaders".to_string(),
            description: Some("Get slot leaders".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_slot": {"type": "integer"},
                    "limit": {"type": "integer"}
                },
                "required": ["start_slot", "limit"]
            }),
        },
        Tool {
            name: "get_block".to_string(),
            description: Some("Get block information".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "slot": {"type": "integer"}
                },
                "required": ["slot"]
            }),
        },
        Tool {
            name: "get_block_height".to_string(),
            description: Some("Get current block height".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_block_production".to_string(),
            description: Some("Get block production information".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "identity": {"type": "string"},
                    "first_slot": {"type": "integer"},
                    "last_slot": {"type": "integer"}
                }
            }),
        },
        Tool {
            name: "get_blocks".to_string(),
            description: Some("Get confirmed blocks between two slots".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_slot": {"type": "integer"},
                    "end_slot": {"type": "integer"}
                },
                "required": ["start_slot"]
            }),
        },
        // Account Methods
        Tool {
            name: "get_balance".to_string(),
            description: Some("Get account balance".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pubkey": {"type": "string"}
                },
                "required": ["pubkey"]
            }),
        },
        Tool {
            name: "get_account_info".to_string(),
            description: Some("Get detailed account information".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pubkey": {"type": "string"}
                },
                "required": ["pubkey"]
            }),
        },
        Tool {
            name: "get_multiple_accounts".to_string(),
            description: Some("Get information for multiple accounts".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pubkeys": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["pubkeys"]
            }),
        },
        Tool {
            name: "get_program_accounts".to_string(),
            description: Some("Get all accounts owned by a program".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "program_id": {"type": "string"}
                },
                "required": ["program_id"]
            }),
        },
        // Transaction Methods
        Tool {
            name: "get_transaction".to_string(),
            description: Some("Get transaction details".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "signature": {"type": "string"}
                },
                "required": ["signature"]
            }),
        },
        Tool {
            name: "get_signatures_for_address".to_string(),
            description: Some("Get confirmed signatures for address".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "address": {"type": "string"},
                    "before": {"type": "string"},
                    "until": {"type": "string"},
                    "limit": {"type": "integer"}
                },
                "required": ["address"]
            }),
        },
        Tool {
            name: "send_transaction".to_string(),
            description: Some("Submit a signed transaction".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "transaction": {"type": "string"},
                    "encoding": {"type": "string", "enum": ["base58", "base64"]}
                },
                "required": ["transaction", "encoding"]
            }),
        },
        // System Info Methods
        Tool {
            name: "get_health".to_string(),
            description: Some("Get node health status".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_version".to_string(),
            description: Some("Get node version information".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_identity".to_string(),
            description: Some("Get node identity".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_cluster_nodes".to_string(),
            description: Some("Get information about all the nodes participating in the cluster".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        // Epoch & Inflation Methods
        Tool {
            name: "get_epoch_info".to_string(),
            description: Some("Get current epoch information".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_epoch_schedule".to_string(),
            description: Some("Get epoch schedule information".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_inflation_rate".to_string(),
            description: Some("Get current inflation rate".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_inflation_governor".to_string(),
            description: Some("Get inflation governor parameters".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        // Token Methods
        Tool {
            name: "get_token_accounts_by_owner".to_string(),
            description: Some("Get token accounts owned by an address".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": {"type": "string"}
                },
                "required": ["owner"]
            }),
        },
        Tool {
            name: "get_token_supply".to_string(),
            description: Some("Get total supply of a token".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mint": {"type": "string"}
                },
                "required": ["mint"]
            }),
        },
        Tool {
            name: "get_token_largest_accounts".to_string(),
            description: Some("Get token accounts with largest balances".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mint": {"type": "string"}
                },
                "required": ["mint"]
            }),
        },
    ]
}
