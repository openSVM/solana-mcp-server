pub mod accounts;
pub mod blocks;
pub mod system;
pub mod tokens;
pub mod transactions;

use std::collections::HashMap;
use crate::docs::RpcMethodDoc;

pub fn get_rpc_method_docs() -> HashMap<String, RpcMethodDoc> {
    let mut docs = HashMap::new();
    
    // Combine docs from all RPC modules
    docs.extend(accounts::get_account_method_docs());
    docs.extend(blocks::get_block_method_docs());
    docs.extend(system::get_system_method_docs());
    docs.extend(tokens::get_token_method_docs());
    docs.extend(transactions::get_transaction_method_docs());
    
    docs
}
