use anyhow::Result;
use tokio::sync::mpsc;

pub async fn start_server() -> Result<()> {
    eprintln!("Solana MCP server ready - {} v{}", 
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Spawn blocking task for stdin
    tokio::task::spawn_blocking(move || {
        let stdin = std::io::stdin();
        let mut buffer = String::new();
        loop {
            buffer.clear();
            match stdin.read_line(&mut buffer) {
                Ok(0) => std::thread::sleep(std::time::Duration::from_millis(100)),
                Ok(_) => {
                    if !buffer.trim().is_empty() {
                        if tx.blocking_send(buffer.clone()).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => eprintln!("Error reading input: {}", e),
            }
        }
    });

    // Process messages in async context
    while let Some(line) = rx.recv().await {
        let response = crate::tools::handle_request(&line).await?;
        println!("{}", serde_json::to_string_pretty(&response)?);
    }

    Ok(())
}
