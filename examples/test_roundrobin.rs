use solana_mcp_server::{Config, server::ServerState};

#[tokio::main]
async fn main() {
    let config = Config::load().expect("Failed to load config");
    let state = ServerState::new(config);

    println!("RPC pool size: {}", state.rpc_clients.len());

    // Test round-robin by calling get_next_rpc_client multiple times
    for i in 0..10 {
        let client = state.get_next_rpc_client();
        let url = client.url();
        println!("Request {}: {}", i + 1, url);
    }
}
