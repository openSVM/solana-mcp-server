use std::io::{self, BufRead, BufReader, Write};
use mcp_sdk::transport::{Transport, JsonRpcMessage};
use anyhow::Result;
use std::sync::Mutex;

pub struct CustomStdioTransport {
    reader: Mutex<BufReader<io::Stdin>>,
    writer: Mutex<io::Stdout>,
}

impl CustomStdioTransport {
    pub fn new() -> Self {
        Self {
            reader: Mutex::new(BufReader::new(io::stdin())),
            writer: Mutex::new(io::stdout()),
        }
    }
}

impl Transport for CustomStdioTransport {
    fn send(&self, message: &JsonRpcMessage) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        writeln!(self.writer.lock().unwrap(), "{}", json)?;
        self.writer.lock().unwrap().flush()?;
        Ok(())
    }

    fn receive(&self) -> Result<JsonRpcMessage> {
        let mut line = String::new();
        self.reader.lock().unwrap().read_line(&mut line)?;
        let message = serde_json::from_str(&line)?;
        Ok(message)
    }

    fn open(&self) -> Result<()> {
        Ok(())
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }
}
