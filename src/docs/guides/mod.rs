use std::collections::HashMap;

pub fn get_guide_docs() -> HashMap<String, String> {
    let mut docs = HashMap::new();
    
    docs.insert("programs".to_string(), include_str!("programs.md").to_string());
    docs.insert("development".to_string(), include_str!("development.md").to_string());
    docs.insert("deployment".to_string(), include_str!("deployment.md").to_string());
    
    docs
}
