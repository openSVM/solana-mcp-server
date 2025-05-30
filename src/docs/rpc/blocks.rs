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

    // getBlockProduction
    docs.insert("getBlockProduction".to_string(), RpcMethodDoc {
        description: "Returns recent block production information from the current or previous epoch".to_string(),
        request_params: vec![
            ParamDoc {
                name: "identity".to_string(),
                type_info: "string".to_string(),
                description: "Optional validator identity pubkey".to_string(),
                required: false,
            },
            ParamDoc {
                name: "range".to_string(),
                type_info: "object".to_string(),
                description: "Slot range to return block production for".to_string(),
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
                name: "byIdentity".to_string(),
                type_info: "object".to_string(),
                description: "Block production by validator identity".to_string(),
                required: true,
            },
            ParamDoc {
                name: "range".to_string(),
                type_info: "object".to_string(),
                description: "Slot range of the block production information".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get recent block production".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getBlockProduction"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "slot": 9887
    },
    "value": {
      "byIdentity": {
        "85iYT5RuzRTDgjyRa3cP8SYhM2j21fj7NhfJ3peu1DPr": {
          "leaderSlots": 1,
          "skippedSlots": 0
        }
      },
      "range": {
        "firstSlot": 0,
        "lastSlot": 9886
      }
    }
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
    
    // getSlotLeaders
    docs.insert("getSlotLeaders".to_string(), RpcMethodDoc {
        description: "Returns slot leaders for a given slot range".to_string(),
        request_params: vec![
            ParamDoc {
                name: "startSlot".to_string(),
                type_info: "number".to_string(),
                description: "Start slot".to_string(),
                required: true,
            },
            ParamDoc {
                name: "limit".to_string(),
                type_info: "number".to_string(),
                description: "Max number of slot leaders to return".to_string(),
                required: true,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "leaders".to_string(),
                type_info: "array".to_string(),
                description: "Array of validator identities (base-58 encoded strings)".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get slot leaders".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getSlotLeaders",
  "params": [100, 10]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": [
    "ChorusmmK7i1AxXeiTtQgQZhQNiXYU84ULeaYF1EH15n",
    "ChorusmmK7i1AxXeiTtQgQZhQNiXYU84ULeaYF1EH15n",
    "ChorusmmK7i1AxXeiTtQgQZhQNiXYU84ULeaYF1EH15n",
    "ChorusmmK7i1AxXeiTtQgQZhQNiXYU84ULeaYF1EH15n",
    "Awes4Tr6TX8JDzEhCZY2QVNimT6iD1zWHzf1vNyGvpLM",
    "Awes4Tr6TX8JDzEhCZY2QVNimT6iD1zWHzf1vNyGvpLM",
    "Awes4Tr6TX8JDzEhCZY2QVNimT6iD1zWHzf1vNyGvpLM",
    "Awes4Tr6TX8JDzEhCZY2QVNimT6iD1zWHzf1vNyGvpLM",
    "DWvDTSh3qfn88UoQTEKRV2JnLt5jtJAVoiCo3ivtMwXP",
    "DWvDTSh3qfn88UoQTEKRV2JnLt5jtJAVoiCo3ivtMwXP"
  ],
  "id": 1
}"#.to_string(),
            }
        ],
    });
    
    // getBlocks
    docs.insert("getBlocks".to_string(), RpcMethodDoc {
        description: "Returns a list of confirmed blocks between two slots".to_string(),
        request_params: vec![
            ParamDoc {
                name: "startSlot".to_string(),
                type_info: "number".to_string(),
                description: "Start slot (inclusive)".to_string(),
                required: true,
            },
            ParamDoc {
                name: "endSlot".to_string(),
                type_info: "number".to_string(),
                description: "End slot (inclusive, optional)".to_string(),
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
                name: "blocks".to_string(),
                type_info: "array".to_string(),
                description: "Array of block numbers in the slot range".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get blocks in a slot range".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getBlocks",
  "params": [5, 10]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": [5, 6, 7, 8, 9, 10],
  "id": 1
}"#.to_string(),
            }
        ],
    });
    
    // getBlocksWithLimit
    docs.insert("getBlocksWithLimit".to_string(), RpcMethodDoc {
        description: "Returns a list of confirmed blocks starting at the given slot with a limit on returned blocks".to_string(),
        request_params: vec![
            ParamDoc {
                name: "startSlot".to_string(),
                type_info: "number".to_string(),
                description: "Start slot (inclusive)".to_string(),
                required: true,
            },
            ParamDoc {
                name: "limit".to_string(),
                type_info: "number".to_string(),
                description: "Maximum number of blocks to return".to_string(),
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
                name: "blocks".to_string(),
                type_info: "array".to_string(),
                description: "Array of block numbers limited by the input parameter".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get a limited number of blocks starting from a slot".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getBlocksWithLimit",
  "params": [5, 3]
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": [5, 6, 7],
  "id": 1
}"#.to_string(),
            }
        ],
    });
    
    // getLeaderSchedule
    docs.insert("getLeaderSchedule".to_string(), RpcMethodDoc {
        description: "Returns the leader schedule for an epoch".to_string(),
        request_params: vec![
            ParamDoc {
                name: "slot".to_string(),
                type_info: "number".to_string(),
                description: "Fetch the leader schedule for the epoch that contains this slot (optional)".to_string(),
                required: false,
            },
            ParamDoc {
                name: "identity".to_string(),
                type_info: "string".to_string(),
                description: "Only return results for this validator identity (base-58 encoded string, optional)".to_string(),
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
                name: "schedule".to_string(),
                type_info: "object".to_string(),
                description: "Leader schedule by validator identity".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get leader schedule".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getLeaderSchedule"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "4Qkev8aNZcqFNSRhQzwyLMFSsi94jHqE8WNVTJzTP99F": [
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
    ]
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });
    
    // getVoteAccounts
    docs.insert("getVoteAccounts".to_string(), RpcMethodDoc {
        description: "Returns the account info and associated stake for all voting accounts in the current bank".to_string(),
        request_params: vec![
            ParamDoc {
                name: "commitment".to_string(),
                type_info: "string".to_string(),
                description: "Commitment level to use".to_string(),
                required: false,
            },
            ParamDoc {
                name: "votePubkey".to_string(),
                type_info: "string".to_string(),
                description: "Only return results for this vote account pubkey (base-58 encoded string)".to_string(),
                required: false,
            },
            ParamDoc {
                name: "keepUnstakedDelinquents".to_string(),
                type_info: "boolean".to_string(),
                description: "Do not filter out delinquent validators with no stake".to_string(),
                required: false,
            },
            ParamDoc {
                name: "delinquentSlotDistance".to_string(),
                type_info: "number".to_string(),
                description: "Specify the number of slots behind the tip to consider a validator delinquent".to_string(),
                required: false,
            },
        ],
        response_fields: vec![
            ParamDoc {
                name: "current".to_string(),
                type_info: "array".to_string(),
                description: "Active vote accounts info".to_string(),
                required: true,
            },
            ParamDoc {
                name: "delinquent".to_string(),
                type_info: "array".to_string(),
                description: "Delinquent vote accounts info".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get vote accounts".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getVoteAccounts"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": {
    "current": [
      {
        "commission": 0,
        "epochVoteAccount": true,
        "epochCredits": [[1, 1, 0]],
        "nodePubkey": "Node1dRYdUGYSamyNQJ5HHGroupWifnxXdXEZY4fVxbiQ",
        "lastVote": 12345,
        "activatedStake": 42,
        "votePubkey": "3ZT31jkAGhUaw8jsy4bTknwBMP8i4Eueh52By4zXcsVw"
      }
    ],
    "delinquent": []
  },
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getFirstAvailableBlock
    docs.insert("getFirstAvailableBlock".to_string(), RpcMethodDoc {
        description: "Returns the slot of the lowest confirmed block that has not been purged from the ledger".to_string(),
        request_params: vec![],
        response_fields: vec![
            ParamDoc {
                name: "slot".to_string(),
                type_info: "number".to_string(),
                description: "First available block slot".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get first available block".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getFirstAvailableBlock"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": 5,
  "id": 1
}"#.to_string(),
            }
        ],
    });

    // getGenesisHash
    docs.insert("getGenesisHash".to_string(), RpcMethodDoc {
        description: "Returns the genesis hash of the ledger".to_string(),
        request_params: vec![],
        response_fields: vec![
            ParamDoc {
                name: "hash".to_string(),
                type_info: "string".to_string(),
                description: "Genesis hash as base-58 encoded string".to_string(),
                required: true,
            },
        ],
        examples: vec![
            Example {
                description: "Get genesis hash".to_string(),
                request: r#"{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getGenesisHash"
}"#.to_string(),
                response: r#"{
  "jsonrpc": "2.0",
  "result": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d",
  "id": 1
}"#.to_string(),
            }
        ],
    });

    docs
}
