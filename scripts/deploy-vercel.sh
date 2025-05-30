#!/bin/bash
# One-liner deployment script for Vercel Edge Functions
set -e

echo "⚡ Deploying Solana MCP Server to Vercel Edge Functions..."

# Create Vercel project structure, install dependencies, and deploy
mkdir -p api && cat > api/solana-mcp.rs << 'EOF'
use vercel_runtime::{run, Body, Error, Request, Response};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let body = req.body();
    let payload: Value = serde_json::from_slice(body)?;
    
    let config = solana_mcp_server::Config::load().map_err(|e| {
        Error::from(format!("Config error: {}", e))
    })?;
    
    let state = std::sync::Arc::new(tokio::sync::RwLock::new(
        solana_mcp_server::server::ServerState::new(config)
    ));
    
    let response = solana_mcp_server::tools::handle_request(
        &payload.to_string(),
        state
    ).await.map_err(|e| Error::from(format!("Handler error: {}", e)))?;
    
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&response)?.into())?)
}
EOF
echo '{"functions": {"api/solana-mcp.rs": {"runtime": "vercel-rust@4.0.0"}}, "env": {"SOLANA_RPC_URL": "https://api.mainnet-beta.solana.com"}}' > vercel.json && npx vercel --prod && echo "✅ Vercel deployment complete! Check: npx vercel ls"