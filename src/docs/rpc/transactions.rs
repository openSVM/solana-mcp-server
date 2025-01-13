use std::collections::HashMap;
use crate::docs::{RpcMethodDoc, ParamDoc, Example};

pub fn get_transaction_method_docs() -> HashMap<String, RpcMethodDoc> {
    let mut docs = HashMap::new();

    // getTransaction
    docs.insert("getTransaction".to_string(), RpcMethodDoc {
        description: "Returns transaction details for a confirmed transaction".to_string(),
        request_params: vec![
            ParamDoc {
                name: "signature".to_string(),
                type_info: "string".to_string(),
                description: "Transaction signature as base-58 encoded string".to_string(),
                required: true,
            },
            ParamDoc {
                name: "config".to_string(),
                type_info: "object".to_string(),
                description: "Configuration object containing encoding, commitment and max supported transaction version".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "slot".to_string(),
                type_info: "number".to_string(),
                description: "The slot this transaction was processed in".to_string(),
                required: true,
            },
            ParamDoc {
                name: "transaction".to_string(),
                type_info: "object".to_string(),
                description: "Transaction object, either in JSON format or encoded binary data depending on encoding parameter".to_string(),
                required: true,
            },
            ParamDoc {
                name: "meta".to_string(),
                type_info: "object | null".to_string(),
                description: "Transaction status metadata object".to_string(),
                required: false,
            },
        ],
        examples: vec![
            Example {
                description: "Get transaction details".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTransaction",
  "params": [
    "2nBhEBYYvfaAe16UMNqRHre4YNSskvuYgx3M6E4JP1oDYvZEJHvoPzyUidNgNX5r9sTyN1J9UxtbCXy2rqYcuyuv",
    {"encoding": "json", "maxSupportedTransactionVersion": 0}
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "slot": 430,
    "transaction": {
      "message": {
        "accountKeys": ["3UVYmECPPMZSCqWKfENfuoTv51fTDTWicX9xmBD2euKe"],
        "header": {
          "numReadonlySignedAccounts": 0,
          "numReadonlyUnsignedAccounts": 1,
          "numRequiredSignatures": 1
        },
        "instructions": [
          {
            "accounts": [0, 1],
            "data": "3Bxs4h24hBtQy9rw",
            "programIdIndex": 2
          }
        ],
        "recentBlockhash": "mfcyqEXB3DnHXki6KjjmZck6YjmZLvpAByy2fj4nh6B"
      },
      "signatures": [
        "2nBhEBYYvfaAe16UMNqRHre4YNSskvuYgx3M6E4JP1oDYvZEJHvoPzyUidNgNX5r9sTyN1J9UxtbCXy2rqYcuyuv"
      ]
    },
    "meta": {
      "err": null,
      "fee": 5000,
      "postBalances": [499998932000, 26858640, 1],
      "preBalances": [499998937000, 26858640, 1],
      "status": { "Ok": null }
    }
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getSignaturesForAddress
    docs.insert("getSignaturesForAddress".to_string(), RpcMethodDoc {
        description: "Returns signatures for confirmed transactions that include the given address in their accountKeys list".to_string(),
        request_params: vec![
            ParamDoc {
                name: "address".to_string(),
                type_info: "string".to_string(),
                description: "Account address as base-58 encoded string".to_string(),
                required: true,
            },
            ParamDoc {
                name: "config".to_string(),
                type_info: "object".to_string(),
                description: "Configuration object containing limit, before, until parameters".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "signature".to_string(),
                type_info: "string".to_string(),
                description: "Transaction signature as base-58 encoded string".to_string(),
                required: true,
            },
            ParamDoc {
                name: "slot".to_string(),
                type_info: "number".to_string(),
                description: "The slot that contains the block with the transaction".to_string(),
                required: true,
            },
            ParamDoc {
                name: "err".to_string(),
                type_info: "object | null".to_string(),
                description: "Error if transaction failed, null if transaction succeeded".to_string(),
                required: true,
            },
            ParamDoc {
                name: "memo".to_string(),
                type_info: "string | null".to_string(),
                description: "Memo associated with the transaction, null if no memo is present".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get signatures for address".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getSignaturesForAddress",
  "params": [
    "Vote111111111111111111111111111111111111111",
    {
      "limit": 1
    }
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": [
    {
      "err": null,
      "memo": null,
      "signature": "5h6xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXFSDwt8GFXM7W5Ncn16wmqokgpiKRLuS83KUxyZyv2sUYv",
      "slot": 114
    }
  ],
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getSignatureStatuses
    docs.insert("getSignatureStatuses".to_string(), RpcMethodDoc {
        description: "Returns the statuses of a list of signatures".to_string(),
        request_params: vec![
            ParamDoc {
                name: "signatures".to_string(),
                type_info: "array".to_string(),
                description: "An array of transaction signatures to confirm as base-58 encoded strings".to_string(),
                required: true,
            },
            ParamDoc {
                name: "config".to_string(),
                type_info: "object".to_string(),
                description: "Configuration object for search parameters".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "slot".to_string(),
                type_info: "number".to_string(),
                description: "The slot the transaction was processed".to_string(),
                required: true,
            },
            ParamDoc {
                name: "confirmations".to_string(),
                type_info: "number | null".to_string(),
                description: "Number of blocks since signature confirmation, null if rooted".to_string(),
                required: true,
            },
            ParamDoc {
                name: "err".to_string(),
                type_info: "object | null".to_string(),
                description: "Error if transaction failed, null if transaction succeeded".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get signature statuses".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getSignatureStatuses",
  "params": [
    [
      "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW"
    ]
  ]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 82
    },
    "value": [
      {
        "slot": 48,
        "confirmations": null,
        "err": null,
        "status": {
          "Ok": null
        },
        "confirmationStatus": "finalized"
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
