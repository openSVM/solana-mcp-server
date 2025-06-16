use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_solana_operations() {
    // Connect to Solana devnet
    // Use Solana's devnet endpoint
    // Use official devnet with longer timeout
    let rpc_url = "https://api.opensvm.com".to_string();
    let timeout = std::time::Duration::from_secs(60);
    let commitment = CommitmentConfig::finalized();
    let client = RpcClient::new_with_timeout_and_commitment(rpc_url.clone(), timeout, commitment);

    println!("\nTesting health check:");
    match client.get_health().await {
        Ok(health) => println!("Health status: {:?}", health),
        Err(err) => {
            println!("Error details: {:?}", err);
            panic!("Health check failed: {}", err);
        }
    }

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
    let system_program_id = "11111111111111111111111111111111"
        .parse::<Pubkey>()
        .unwrap();
    let account = client.get_account(&system_program_id).await.unwrap();
    println!("System Program Account:");
    println!("  Owner: {}", account.owner);
    println!("  Lamports: {}", account.lamports);
    println!("  Executable: {}", account.executable);

    // Get recent confirmed signatures first
    println!("\nTesting recent transactions:");
    let signatures = client
        .get_signatures_for_address(&system_program_id)
        .await
        .unwrap();
    println!("Recent transactions for System Program:");
    for sig in signatures.iter().take(3) {
        println!("  Signature: {}", sig.signature);
        println!("  Slot: {}", sig.slot);
        if let Some(err) = &sig.err {
            println!("  Error: {:?}", err);
        }
    }

    // Test creating a new keypair and getting its info
    println!("\nTesting keypair operations:");
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    println!("Generated keypair with pubkey: {}", pubkey);

    // Get account info (should be empty/not found)
    match client.get_account(&pubkey).await {
        Ok(account) => println!("Account exists with {} lamports", account.lamports),
        Err(e) => println!("Account not found as expected: {}", e),
    }

    // Get minimum rent
    println!("\nTesting rent calculation:");
    let rent = client
        .get_minimum_balance_for_rent_exemption(0)
        .await
        .unwrap();
    println!("Minimum balance for rent exemption: {} lamports", rent);

    // Get recent block
    println!("\nTesting block info:");
    let slot = client.get_slot().await.unwrap();
    println!("Current slot: {}", slot);

    // Get block production
    println!("\nTesting block production:");
    let production = client.get_block_production().await.unwrap();
    println!("Block production: {:?}", production);

    // Get cluster nodes
    println!("\nTesting cluster info:");
    let nodes = client.get_cluster_nodes().await.unwrap();
    println!("Found {} cluster nodes", nodes.len());
    for node in nodes.iter().take(3) {
        let version = node.version.as_ref().map_or("unknown", |v| v.as_str());
        println!("  {}: {}", node.pubkey, version);
    }
}
