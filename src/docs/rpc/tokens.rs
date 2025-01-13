use std::collections::HashMap;
use crate::docs::{RpcMethodDoc, ParamDoc, Example};

pub fn get_token_method_docs() -> HashMap<String, RpcMethodDoc> {
    let mut docs = HashMap::new();

    // getTokenAccountBalance
    docs.insert("getTokenAccountBalance".to_string(), RpcMethodDoc {
        description: "Returns the token balance of an SPL Token account".to_string(),
        request_params: vec![
            ParamDoc {
                name: "accountAddress".to_string(),
                type_info: "string".to_string(),
                description: "Public key of token account to query, as base-58 encoded string".to_string(),
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
                name: "amount".to_string(),
                type_info: "string".to_string(),
                description: "Raw token amount".to_string(),
                required: true,
            },
            ParamDoc {
                name: "decimals".to_string(),
                type_info: "number".to_string(),
                description: "Number of decimal places".to_string(),
                required: true,
            },
            ParamDoc {
                name: "uiAmount".to_string(),
                type_info: "number | null".to_string(),
                description: "Token amount as a float".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get token account balance".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTokenAccountBalance",
  "params": ["7fUAJdStEuGbc3sM84cKRL6yYaaSstyLSU4ve5oovLS7"]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 1114
    },
    "value": {
      "amount": "9999999910",
      "decimals": 6,
      "uiAmount": 9999.99991,
      "uiAmountString": "9999.99991"
    }
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getTokenSupply
    docs.insert("getTokenSupply".to_string(), RpcMethodDoc {
        description: "Returns the total supply of an SPL Token type".to_string(),
        request_params: vec![
            ParamDoc {
                name: "mint".to_string(),
                type_info: "string".to_string(),
                description: "Pubkey of token Mint to query as base-58 encoded string".to_string(),
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
                name: "amount".to_string(),
                type_info: "string".to_string(),
                description: "Raw total token supply without decimals".to_string(),
                required: true,
            },
            ParamDoc {
                name: "decimals".to_string(),
                type_info: "number".to_string(),
                description: "Number of base 10 digits to the right of the decimal place".to_string(),
                required: true,
            },
            ParamDoc {
                name: "uiAmount".to_string(),
                type_info: "number".to_string(),
                description: "Total token supply using mint-prescribed decimals".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get token supply".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTokenSupply",
  "params": [
    "3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E"
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 1114
    },
    "value": {
      "amount": "100000",
      "decimals": 2,
      "uiAmount": 1000,
      "uiAmountString": "1000"
    }
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getTokenLargestAccounts
    docs.insert("getTokenLargestAccounts".to_string(), RpcMethodDoc {
        description: "Returns the 20 largest accounts of a particular SPL Token type".to_string(),
        request_params: vec![
            ParamDoc {
                name: "mint".to_string(),
                type_info: "string".to_string(),
                description: "Pubkey of token Mint to query as base-58 encoded string".to_string(),
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
                name: "address".to_string(),
                type_info: "string".to_string(),
                description: "Address of the token account".to_string(),
                required: true,
            },
            ParamDoc {
                name: "amount".to_string(),
                type_info: "string".to_string(),
                description: "Raw token account balance without decimals".to_string(),
                required: true,
            },
            ParamDoc {
                name: "uiAmount".to_string(),
                type_info: "number".to_string(),
                description: "Token account balance using mint-prescribed decimals".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get largest token accounts".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTokenLargestAccounts",
  "params": [
    "3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E"
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 1114
    },
    "value": [
      {
        "address": "FYjHNoFtSQ5uijKrZFyYAxvEr87hsKXkXcxkcmkBAf4r",
        "amount": "771",
        "decimals": 2,
        "uiAmount": 7.71,
        "uiAmountString": "7.71"
      }
    ]
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    docs
}
