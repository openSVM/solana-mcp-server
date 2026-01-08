use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

#[tokio::main]
async fn main() {
    env_logger::init();

    println!("Creating RPC client...");
    let client = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    println!("Calling get_slot()...");
    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        client.get_slot()
    ).await {
        Ok(Ok(slot)) => println!("SUCCESS: Slot = {}", slot),
        Ok(Err(e)) => println!("RPC ERROR: {}", e),
        Err(_) => println!("TIMEOUT after 10 seconds"),
    }
}
