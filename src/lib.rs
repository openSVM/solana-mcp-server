use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use serde_json::Value;

pub struct SolanaMcpServer {
    client: RpcClient,
}

impl SolanaMcpServer {
    pub fn new(client: RpcClient) -> Self {
        Self { client }
    }

    pub fn list_tools(&self) -> Result<Value> {
        Ok(serde_json::json!([]))
    }

    pub async fn handle_tool_request(&self, request: mcp_sdk::types::CallToolRequest) -> Result<Value> {
        match request.name.as_str() {
            "get_slot" => {
                let slot = self.client.get_slot().await?;
                Ok(serde_json::json!({ "slot": slot }))
            },
            _ => Err(anyhow::anyhow!("Tool not found: {}", request.name)),
        }
    }
}
