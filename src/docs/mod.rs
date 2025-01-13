pub mod rpc;
pub mod core;
pub mod guides;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcMethodDoc {
    pub description: String,
    pub request_params: Vec<ParamDoc>,
    pub response_fields: Vec<ParamDoc>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamDoc {
    pub name: String,
    pub type_info: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Example {
    pub description: String,
    pub request: String,
    pub response: String,
}

pub fn get_all_docs() -> HashMap<String, String> {
    let mut docs = HashMap::new();
    docs.extend(core::get_core_docs());
    docs.extend(guides::get_guide_docs());
    docs
}
