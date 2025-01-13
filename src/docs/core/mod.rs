use std::collections::HashMap;

pub fn get_core_docs() -> HashMap<String, String> {
    let mut docs = HashMap::new();
    
    docs.insert("transactions".to_string(), include_str!("transactions.md").to_string());
    docs.insert("accounts".to_string(), include_str!("accounts.md").to_string());
    docs.insert("programs".to_string(), include_str!("programs.md").to_string());
    
    docs
}
