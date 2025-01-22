use anyhow::Result;
use std::io::{self, BufRead, Write};

pub async fn start_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        
        if n == 0 {
            break;
        }

        let response = crate::tools::handle_request(&line).await?;
        writeln!(stdout, "{}", serde_json::to_string_pretty(&response)?)?;
        stdout.flush()?;
    }
    
    Ok(())
}
