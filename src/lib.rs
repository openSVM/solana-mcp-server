//! Solana MCP Server
//! 
//! This crate provides a Model Context Protocol (MCP) server implementation for Solana RPC.
//! It exposes Solana RPC methods as MCP tools that can be used by MCP clients.

mod rpc;
mod server;
mod tools;

pub use server::SolanaMcpServer;

// Re-export commonly used types
pub use solana_client::nonblocking::rpc_client::RpcClient;
pub use solana_sdk::{pubkey::Pubkey, signature::Signature};
pub use anyhow::Result;
