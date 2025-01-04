use anyhow::Result;
use async_trait::async_trait;
use log::{error, info, warn};
use mcp_sdk::{
    transport::{StdioTransport, JsonRpcMessage, Transport, JsonRpcResponse, JsonRpcVersion},
    types::{
        CallToolRequest, CallToolResponse, ToolResponseContent,
        ResourceContents, ReadResourceRequest,
    },
    server::{Server, ServerOptions},
    protocol::ProtocolBuilder,
};
// ... (rest of the imports remain the same)

// ... (rest of the code remains the same until Transport impl)

#[async_trait]
impl Transport for SolanaMcpServer {
    async fn send<'life0, 'async_trait>(
        &'life0 self,
        _message: &'life0 JsonRpcMessage,
    ) -> Result<()>
    where
        'life0: 'async_trait,
    {
        Ok(())
    }

    async fn receive<'life0, 'async_trait>(
        &'life0 self,
    ) -> Result<JsonRpcMessage>
    where
        'life0: 'async_trait,
    {
        Ok(JsonRpcMessage::Response(JsonRpcResponse {
            jsonrpc: JsonRpcVersion::Version2,
            id: 0,
            result: Some(serde_json::Value::Null),
            error: None,
        }))
    }

    async fn open<'life0, 'async_trait>(
        &'life0 self,
    ) -> Result<()>
    where
        'life0: 'async_trait,
    {
        Ok(())
    }

    async fn close<'life0, 'async_trait>(
        &'life0 self,
    ) -> Result<()>
    where
        'life0: 'async_trait,
    {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    
    let client = RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    );
    
    let server = SolanaMcpServer::new(client);
    let transport = StdioTransport::default();
    let protocol = ProtocolBuilder::new(server);
    let options = ServerOptions::default();
    
    info!("Starting Solana MCP server...");
    Server::new(protocol, options).execute(transport).await?;
    
    Ok(())
}
