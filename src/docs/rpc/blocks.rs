use std::collections::HashMap;
use crate::docs::{RpcMethodDoc, ParamDoc, Example};

pub fn get_block_method_docs() -> HashMap<String, RpcMethodDoc> {
    let mut docs = HashMap::new();

    // getBlock
    docs.insert("getBlock".to_string(), RpcMethodDoc {
        description: "Returns identity and transaction information about a confirmed block in the ledger".to_string(),
        request_params: vec![
            ParamDoc {
                name: "slot".to_string(),
                type_info: "number".to_string(),
                description: "Slot number".to_string(),
                required: true,
            },
            ParamDoc {
                name: "config".to_string(),
                type_info: "object".to_string(),
                description: "Configuration object for block data".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "blockhash".to_string(),
                type_info: "string".to_string(),
                description: "Blockhash of this block".to_string(),
                required: true,
            },
            ParamDoc {
                name: "previousBlockhash".to_string(),
                type_info: "string".to_string(),
                description: "Blockhash of this block's parent".to_string(),
                required: true,
            },
            ParamDoc {
                name: "parentSlot".to_string(),
                type_info: "number".to_string(),
                description: "Slot index of this block's parent".to_string(),
                required: true,
            },
            ParamDoc {
                name: "transactions".to_string(),
                type_info: "array".to_string(),
                description: "Array of transactions and status information".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get block information".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getBlock",
  "params": [430, {"encoding": "json", "maxSupportedTransactionVersion": 0}]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "blockhash": "3Eq21vXNB5s86c62bVuUfTeaMif1N2kUqRPBmGRJhyTA",
    "parentSlot": 429,
    "previousBlockhash": "mfcyqEXB3DnHXki6KjjmZck6YjmZLvpAByy2fj4nh6B",
    "transactions": [
      {
        "transaction": ["transaction_data_here"],
        "meta": {
          "fee": 5000,
          "postBalances": [499998932000, 26858640, 1],
          "preBalances": [499998937000, 26858640, 1],
          "status": { "Ok": null }
        }
      }
    ]
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getBlockHeight
    docs.insert("getBlockHeight".to_string(), RpcMethodDoc {
        description: "Returns the current block height of the node".to_string(),
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
                name: "blockHeight".to_string(),
                type_info: "number".to_string(),
                description: "Current block height".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get current block height".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getBlockHeight"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": 1233,
  "id": 1
}"#.to_string(),
            }
        ],
    });

    docs
}
