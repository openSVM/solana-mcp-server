use std::collections::HashMap;
use crate::docs::{RpcMethodDoc, ParamDoc, Example};

pub fn get_system_method_docs() -> HashMap<String, RpcMethodDoc> {
    let mut docs = HashMap::new();

    // getHealth
    docs.insert("getHealth".to_string(), RpcMethodDoc {
        description: "Returns the current health of the node".to_string(),
        request_params: vec![],
        response_fields: vec![
            ParamDoc {
                name: "status".to_string(),
                type_info: "string".to_string(),
                description: "Node health status (ok, behind, unknown)".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get node health".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getHealth"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": "ok",
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getVersion
    docs.insert("getVersion".to_string(), RpcMethodDoc {
        description: "Returns the current Solana version running on the node".to_string(),
        request_params: vec![],
        response_fields: vec![
            ParamDoc {
                name: "solana-core".to_string(),
                type_info: "string".to_string(),
                description: "Software version of solana-core".to_string(),
                required: true,
            },
            ParamDoc {
                name: "feature-set".to_string(),
                type_info: "number".to_string(),
                description: "Unique identifier of the current software's feature set".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get version information".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getVersion"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "solana-core": "1.16.15",
    "feature-set": 2067660359
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getSlot
    docs.insert("getSlot".to_string(), RpcMethodDoc {
        description: "Returns the current slot the node is processing".to_string(),
        request_params: vec![
            ParamDoc {
                name: "commitment".to_string(),
                type_info: "string".to_string(),
                description: "Commitment level to use".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "slot".to_string(),
                type_info: "number".to_string(),
                description: "Current slot".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get current slot".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getSlot"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": 1234,
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getEpochInfo
    docs.insert("getEpochInfo".to_string(), RpcMethodDoc {
        description: "Returns information about the current epoch".to_string(),
        request_params: vec![
            ParamDoc {
                name: "commitment".to_string(),
                type_info: "string".to_string(),
                description: "Commitment level to use".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "absoluteSlot".to_string(),
                type_info: "number".to_string(),
                description: "The current slot".to_string(),
                required: true,
            },
            ParamDoc {
                name: "blockHeight".to_string(),
                type_info: "number".to_string(),
                description: "The current block height".to_string(),
                required: true,
            },
            ParamDoc {
                name: "epoch".to_string(),
                type_info: "number".to_string(),
                description: "The current epoch".to_string(),
                required: true,
            },
            ParamDoc {
                name: "slotIndex".to_string(),
                type_info: "number".to_string(),
                description: "The current slot relative to the start of the current epoch".to_string(),
                required: true,
            },
            ParamDoc {
                name: "slotsInEpoch".to_string(),
                type_info: "number".to_string(),
                description: "The number of slots in this epoch".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get current epoch information".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getEpochInfo"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "absoluteSlot": 166598,
    "blockHeight": 166500,
    "epoch": 27,
    "slotIndex": 2790,
    "slotsInEpoch": 8192,
    "transactionCount": 22661093
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    docs
}
