use anyhow::Result;
use std::io::{self, BufRead, Write};

pub async fn start_server() -> Result<()> {
    eprintln!("Solana MCP server ready - {} v{}", 
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => continue,
            Ok(_) => {
                if !line.trim().is_empty() {
                    let response = crate::tools::handle_request(&line).await?;
                    writeln!(stdout, "{}", serde_json::to_string_pretty(&response)?)?;
                    stdout.flush()?;
                }
            }
            Err(e) => eprintln!("Error reading input: {}", e),
        }
    }
}
