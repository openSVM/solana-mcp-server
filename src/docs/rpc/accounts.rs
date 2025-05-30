use std::collections::HashMap;
use crate::docs::{RpcMethodDoc, ParamDoc, Example};

pub fn get_account_method_docs() -> HashMap<String, RpcMethodDoc> {
    let mut docs = HashMap::new();

    // getAccountInfo
    docs.insert("getAccountInfo".to_string(), RpcMethodDoc {
        description: "Returns all information associated with the account of provided Pubkey".to_string(),
        request_params: vec![
            ParamDoc {
                name: "pubkey".to_string(),
                type_info: "string".to_string(),
                description: "Pubkey of account to query, as base-58 encoded string".to_string(),
                required: true,
            },
            ParamDoc {
                name: "commitment".to_string(),
                type_info: "string".to_string(),
                description: "Commitment level to use: processed, confirmed, or finalized".to_string(),
                required: false,
            },
            ParamDoc {
                name: "encoding".to_string(),
                type_info: "string".to_string(),
                description: "Encoding format: base58, base64, or jsonParsed".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "context".to_string(),
                type_info: "object".to_string(),
                description: "Response context including slot".to_string(),
                required: true,
            },
            ParamDoc {
                name: "value".to_string(),
                type_info: "object | null".to_string(),
                description: "Account information or null if not found".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get account info with base58 encoding".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getAccountInfo",
  "params": [
    "vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg",
    {"encoding": "base58"}
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 1
    },
    "value": {
      "data": ["", "base58"],
      "executable": false,
      "lamports": 1000000000,
      "owner": "11111111111111111111111111111111",
      "rentEpoch": 2
    }
  },
  "id": 1
}"#.to_string(),
            },
        ],
    });

    // getBalance
    docs.insert("getBalance".to_string(), RpcMethodDoc {
        description: "Returns the balance of the account of provided Pubkey".to_string(),
        request_params: vec![
            ParamDoc {
                name: "pubkey".to_string(),
                type_info: "string".to_string(),
                description: "Pubkey of account to query, as base-58 encoded string".to_string(),
                required: true,
            },
            ParamDoc {
                name: "commitment".to_string(),
                type_info: "string".to_string(),
                description: "Commitment level to use: processed, confirmed, or finalized".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "context".to_string(),
                type_info: "object".to_string(),
                description: "Response context including slot".to_string(),
                required: true,
            },
            ParamDoc {
                name: "value".to_string(),
                type_info: "number".to_string(),
                description: "Balance in lamports".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get account balance".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getBalance",
  "params": ["83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri"]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 1
    },
    "value": 0
  },
  "id": 1
}"#.to_string(),
            },
        ],
    });

    // getProgramAccounts
    docs.insert("getProgramAccounts".to_string(), RpcMethodDoc {
        description: "Returns all accounts owned by the provided program Pubkey".to_string(),
        request_params: vec![
            ParamDoc {
                name: "programId".to_string(),
                type_info: "string".to_string(),
                description: "Public key of the program to query, as base-58 encoded string".to_string(),
                required: true,
            },
            ParamDoc {
                name: "config".to_string(),
                type_info: "object".to_string(),
                description: "Configuration object containing filters and encoding options".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "pubkey".to_string(),
                type_info: "string".to_string(),
                description: "Account public key".to_string(),
                required: true,
            },
            ParamDoc {
                name: "account".to_string(),
                type_info: "object".to_string(),
                description: "Account information".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get all token accounts for a specific mint".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getProgramAccounts",
  "params": [
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    {
      "encoding": "jsonParsed",
      "filters": [
        {
          "dataSize": 165
        },
        {
          "memcmp": {
            "offset": 0,
            "bytes": "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM"
          }
        }
      ]
    }
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": [
    {
      "account": {
        "data": {
          "parsed": {
            "info": {
              "isNative": false,
              "mint": "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM",
              "owner": "vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg",
              "state": "initialized",
              "tokenAmount": {
                "amount": "100000",
                "decimals": 6,
                "uiAmount": 0.1,
                "uiAmountString": "0.1"
              }
            },
            "type": "account"
          },
          "program": "spl-token",
          "space": 165
        },
        "executable": false,
        "lamports": 2039280,
        "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "rentEpoch": 313
      },
      "pubkey": "7EYnhQoR9YM3N7UoaKRoA44Uy8JeaZV3qyouov87awMs"
    }
  ],
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getLargestAccounts
    docs.insert("getLargestAccounts".to_string(), RpcMethodDoc {
        description: "Returns the 20 largest accounts by lamport balance".to_string(),
        request_params: vec![
            ParamDoc {
                name: "filter".to_string(),
                type_info: "string".to_string(),
                description: "Filter results by account type (circulating|nonCirculating)".to_string(),
                required: false,
            },
            ParamDoc {
                name: "commitment".to_string(),
                type_info: "string".to_string(),
                description: "Commitment level to use".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "context".to_string(),
                type_info: "object".to_string(),
                description: "Response context including slot".to_string(),
                required: true,
            },
            ParamDoc {
                name: "value".to_string(),
                type_info: "array".to_string(),
                description: "Array of account information objects".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get largest accounts".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getLargestAccounts"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 1
    },
    "value": [
      {
        "lamports": 999999999,
        "address": "99P8ZgtJYe1buSK8JXkvpLh8xPsCFuLYhz9hQFNw93WJ"
      },
      {
        "lamports": 42,
        "address": "uPwWLo16MVehpyWqsLkK3Ka8nLowWvAHbBChqv2FZeL"
      }
    ]
  },
  "id": 1
}"#.to_string(),
            },
        ],
    });

    // getMultipleAccounts
    docs.insert("getMultipleAccounts".to_string(), RpcMethodDoc {
        description: "Returns the account information for a list of Pubkeys".to_string(),
        request_params: vec![
            ParamDoc {
                name: "pubkeys".to_string(),
                type_info: "array".to_string(),
                description: "List of Pubkeys to query, as base-58 encoded strings".to_string(),
                required: true,
            },
            ParamDoc {
                name: "config".to_string(),
                type_info: "object".to_string(),
                description: "Configuration object containing commitment and encoding options".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "context".to_string(),
                type_info: "object".to_string(),
                description: "Response context including slot".to_string(),
                required: true,
            },
            ParamDoc {
                name: "value".to_string(),
                type_info: "array".to_string(),
                description: "Array of account information objects or nulls for accounts not found".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get multiple accounts".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getMultipleAccounts",
  "params": [
    [
      "vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg",
      "4fYNw3dojWmQ4dXtSGE9epjRGy9pFSx62YypT7avPYvA"
    ],
    {"encoding": "base58"}
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 1
    },
    "value": [
      {
        "data": ["", "base58"],
        "executable": false,
        "lamports": 1000000000,
        "owner": "11111111111111111111111111111111",
        "rentEpoch": 2
      },
      {
        "data": ["", "base58"],
        "executable": false,
        "lamports": 5000000,
        "owner": "11111111111111111111111111111111",
        "rentEpoch": 2
      }
    ]
  },
  "id": 1
}"#.to_string(),
            },
        ],
    });

    // getMinimumBalanceForRentExemption
    docs.insert("getMinimumBalanceForRentExemption".to_string(), RpcMethodDoc {
        description: "Returns the minimum balance required to make an account rent exempt".to_string(),
        request_params: vec![
            ParamDoc {
                name: "dataSize".to_string(),
                type_info: "number".to_string(),
                description: "Size of account data in bytes".to_string(),
                required: true,
            },
            ParamDoc {
                name: "commitment".to_string(),
                type_info: "string".to_string(),
                description: "Commitment level to use".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "lamports".to_string(),
                type_info: "number".to_string(),
                description: "Minimum lamports required for rent exemption".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get minimum balance for rent exemption".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getMinimumBalanceForRentExemption",
  "params": [50]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": 1238880,
  "id": 1
}"#.to_string(),
            },
        ],
    });

    docs
}
