use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_blockchain_operations() {
    // Connect to Solana devnet
    let rpc_url = "https://api.opensvm.com".to_string();
    let client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());
    
    // Configure RPC client to support latest transaction versions
    let config = solana_client::rpc_config::RpcBlockConfig {
        encoding: None,
        transaction_details: None,
        rewards: None,
        commitment: None,
        max_supported_transaction_version: Some(0),
    };

    println!("\nTesting health check:");
    let health = client.get_health().await.unwrap();
    println!("Health status: {:?}", health);

    println!("\nTesting version info:");
    let version = client.get_version().await.unwrap();
    println!("Version info: {:?}", version);

    println!("\nTesting latest blockhash:");
    let blockhash = client.get_latest_blockhash().await.unwrap();
    println!("Latest blockhash: {:?}", blockhash);

    println!("\nTesting transaction count:");
    let count = client.get_transaction_count().await.unwrap();
    println!("Transaction count: {}", count);

    // Get info about the System Program
    println!("\nTesting account info for System Program:");
    let system_program_id = "11111111111111111111111111111111".parse::<Pubkey>().unwrap();
    let account = client.get_account(&system_program_id).await.unwrap();
    println!("System Program Account:");
    println!("  Owner: {}", account.owner);
    println!("  Lamports: {}", account.lamports);
    println!("  Executable: {}", account.executable);

    // Get recent confirmed signatures first
    println!("\nTesting recent transactions:");
    let signatures = client.get_signatures_for_address(&system_program_id).await.unwrap();
    println!("Recent transactions for System Program:");
    for sig in signatures.iter().take(3) {
        println!("  Signature: {}", sig.signature);
        println!("  Slot: {}", sig.slot);
        if let Some(err) = &sig.err {
            println!("  Error: {:?}", err);
        }
    }

    // Get block data using a slot we know exists from recent transactions
    if let Some(first_sig) = signatures.first() {
        println!("\nTesting block data for slot {}:", first_sig.slot);
        let block = client.get_block_with_config(first_sig.slot, config).await;
        match block {
            Ok(block) => println!("Block data: {:#?}", block),
            Err(e) => println!("Could not fetch block: {}", e),
        }
    }
}
