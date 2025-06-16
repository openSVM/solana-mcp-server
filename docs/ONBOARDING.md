# Developer Onboarding Guide

Welcome to the Solana MCP Server project! This guide will help you get up and running as a contributor quickly and confidently. Whether you're new to Rust, Solana, or MCP (Model Context Protocol), this step-by-step walkthrough will guide you through the entire development lifecycle.

## 🚀 Quick Start

If you're already familiar with Rust development, here's the fast track:

```bash
# Clone and setup
git clone https://github.com/opensvm/solana-mcp-server.git
cd solana-mcp-server

# Build and test
cargo build
cargo test

# Run the server
cargo run
```

For a detailed walkthrough, continue reading below.

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Prerequisites](#prerequisites)
3. [Environment Setup](#environment-setup)
4. [Building the Project](#building-the-project)
5. [Running Tests](#running-tests)
6. [Running the Server](#running-the-server)
7. [Debugging](#debugging)
8. [Deployment](#deployment)
9. [Contributing](#contributing)
10. [Troubleshooting](#troubleshooting)
11. [Resources](#resources)

## 🎯 Project Overview

The Solana MCP Server is a **Model Context Protocol (MCP) implementation** that provides AI assistants (like Claude) with comprehensive access to Solana blockchain data. It acts as a bridge between AI systems and the Solana ecosystem.

### Key Components

- **MCP Server**: Implements the Model Context Protocol specification
- **RPC Methods**: 40+ Solana RPC methods for blockchain data access
- **Multi-Network Support**: Mainnet, Devnet, Testnet, and Eclipse networks
- **Flexible Deployment**: Local, Docker, serverless, and Kubernetes options

### Architecture at a Glance

```
AI Assistant (Claude) → MCP Protocol → Solana MCP Server → Solana RPC → Blockchain
```

For detailed architecture information, see [docs/ARCHITECTURE.md](./ARCHITECTURE.md).

## 📚 Prerequisites

Before starting, ensure you have:

- **Basic familiarity with**:
  - Command line/terminal usage
  - Git version control
  - Basic programming concepts
- **Helpful but not required**:
  - Rust programming language
  - Blockchain/Solana concepts
  - JSON-RPC protocols

## 🛠️ Environment Setup

### 1. Install Rust

The project requires Rust 1.75 or later.

```bash
# Install Rust via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install Development Tools

```bash
# Install essential tools
cargo install cargo-watch    # For auto-rebuilding during development
cargo install cargo-expand   # For macro debugging (optional)

# For better debugging experience
rustup component add rust-src
rustup component add llvm-tools-preview
```

### 3. IDE Setup

**VS Code (Recommended)**:
```bash
# Install VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension serayuzgur.crates
code --install-extension vadimcn.vscode-lldb  # For debugging
```

**Other IDEs**:
- **IntelliJ IDEA**: Install the Rust plugin
- **Vim/Neovim**: Use rust-analyzer LSP
- **Emacs**: Use rust-mode with lsp-mode

### 4. Clone the Repository

```bash
git clone https://github.com/opensvm/solana-mcp-server.git
cd solana-mcp-server
```

### 5. Verify Environment

```bash
# Check Rust version
rustc --version
# Should show: rustc 1.75.0 or later

# Check project structure
ls -la
# Should show: Cargo.toml, src/, docs/, tests/, etc.
```

## 🔨 Building the Project

### 1. Understanding the Build System

The project uses **Cargo**, Rust's build system and package manager. Key files:

- `Cargo.toml`: Project configuration and dependencies
- `Cargo.lock`: Dependency versions (committed to Git)
- `src/`: Source code directory
- `tests/`: Integration tests

### 2. Building for Development

```bash
# Build in debug mode (faster compilation, slower execution)
cargo build

# Build with optimizations (slower compilation, faster execution)
cargo build --release

# Check code without building (fastest)
cargo check
```

### 3. Understanding Build Output

```bash
# After building, you'll see:
ls -la target/debug/
# solana-mcp-server    # The main executable
# deps/                # Compiled dependencies
# build/               # Build artifacts
```

### 4. Continuous Building

For development, use `cargo-watch` to automatically rebuild on file changes:

```bash
# Auto-rebuild on changes
cargo watch -x build

# Auto-rebuild and run tests
cargo watch -x test

# Auto-rebuild and run the server
cargo watch -x run
```

## 🧪 Running Tests

### 1. Test Structure

The project has several types of tests:

- **Unit Tests**: In `src/` files using `#[cfg(test)]`
- **Integration Tests**: In `tests/` directory
- **Documentation Tests**: In doc comments (using `///`)

### 2. Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run tests in specific file
cargo test --test validation
```

### 3. Understanding Test Output

```bash
# Successful test run shows:
running 9 tests
test protocol::tests::test_server_capabilities ... ok
test validation::tests::test_validate_commitment ... ok
# ... more tests ...
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 4. Writing Tests

When contributing, add tests for new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(my_function(5), 10);
    }
}
```

## 🚀 Running the Server

### 1. Configuration

The server can be configured via:

- **Environment variables** (recommended for development)
- **Configuration file** (`config.json`)
- **Command line arguments**

### 2. Basic Local Run

```bash
# Run with default configuration
cargo run

# Run with custom RPC endpoint
SOLANA_RPC_URL=https://api.devnet.solana.com cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run with specific configuration file
cargo run -- --config config.json
```

### 3. Environment Variables

Key environment variables for development:

```bash
# Set Solana RPC endpoint
export SOLANA_RPC_URL="https://api.devnet.solana.com"

# Set commitment level (processed|confirmed|finalized)
export SOLANA_COMMITMENT="confirmed"

# Enable debug logging
export RUST_LOG="debug"

# Set protocol version
export SOLANA_PROTOCOL_VERSION="2024-11-05"
```

### 4. Verifying the Server

Once running, the server will:

1. **Initialize**: Load configuration and validate settings
2. **Connect**: Establish RPC connections to Solana networks
3. **Listen**: Wait for MCP protocol connections
4. **Log**: Output status information

Look for log messages like:
```
[INFO] Solana MCP Server starting
[INFO] Connected to Solana RPC: https://api.devnet.solana.com
[INFO] Server ready to accept connections
```

## 🐛 Debugging

### 1. Logging

Use environment variables to control logging:

```bash
# Debug level logging
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=solana_mcp_server=debug cargo run

# Multiple modules
RUST_LOG=solana_mcp_server=debug,reqwest=info cargo run
```

### 2. Debugging Tools

**LLDB (recommended for macOS/Linux)**:
```bash
# Install LLDB
rustup component add llvm-tools-preview

# Debug with LLDB
rust-lldb target/debug/solana-mcp-server
# (lldb) run
# (lldb) bt  # backtrace
# (lldb) p variable_name  # print variable
```

**GDB (Linux)**:
```bash
# Debug with GDB
rust-gdb target/debug/solana-mcp-server
# (gdb) run
# (gdb) bt  # backtrace
# (gdb) p variable_name  # print variable
```

### 3. IDE Debugging

**VS Code**: Use the CodeLLDB extension:
1. Install the `vadimcn.vscode-lldb` extension
2. Add debug configuration in `.vscode/launch.json`
3. Set breakpoints and press F5

### 4. Common Debugging Scenarios

**Connection Issues**:
```bash
# Test RPC connectivity
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  https://api.devnet.solana.com

# Check network connectivity
ping api.devnet.solana.com
```

**Performance Issues**:
```bash
# Profile with flamegraph
cargo install flamegraph
sudo flamegraph target/debug/solana-mcp-server

# Memory usage
valgrind --tool=massif target/debug/solana-mcp-server
```

## 🚢 Deployment

### 1. Deployment Options

The project supports multiple deployment methods:

- **Local Binary**: Direct execution
- **Docker**: Containerized deployment
- **Kubernetes**: Orchestrated containers
- **Serverless**: AWS Lambda, Google Cloud Functions, Vercel

### 2. Local Deployment

```bash
# Build release version
cargo build --release

# Run release version
./target/release/solana-mcp-server

# Or use the deployment script
./scripts/deploy-local.sh
```

### 3. Docker Deployment

```bash
# Build Docker image
docker build -t solana-mcp-server .

# Run container
docker run -p 8080:8080 \
  -e SOLANA_RPC_URL=https://api.mainnet-beta.solana.com \
  solana-mcp-server

# Or use the deployment script
./scripts/deploy-docker.sh
```

### 4. Other Deployment Options

See the [Deployment Guide](./DEPLOYMENT.md) for detailed instructions on:

- **Kubernetes**: `./scripts/deploy-k8s.sh`
- **AWS Lambda**: `./scripts/deploy-lambda.sh`
- **Google Cloud Functions**: `./scripts/deploy-gcf.sh`
- **Vercel**: `./scripts/deploy-vercel.sh`

## 🤝 Contributing

### 1. Development Workflow

```bash
# 1. Create a feature branch
git checkout -b feature/my-new-feature

# 2. Make changes and test
cargo test
cargo build

# 3. Commit changes
git add .
git commit -m "Add new feature: description"

# 4. Push and create PR
git push origin feature/my-new-feature
```

### 2. Code Style

The project follows standard Rust conventions:

```bash
# Format code
cargo fmt

# Check for common issues
cargo clippy

# Check for security issues
cargo audit  # (requires: cargo install cargo-audit)
```

### 3. Submitting Changes

1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Test** thoroughly
5. **Submit** a pull request

### 4. PR Checklist

Before submitting a PR:

- [ ] Code builds without warnings
- [ ] All tests pass
- [ ] New tests added for new functionality
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] Commit messages are descriptive

## 🔧 Troubleshooting

### Common Issues

**Build Fails with "linker not found"**:
```bash
# Install build tools
# Ubuntu/Debian:
sudo apt install build-essential

# macOS:
xcode-select --install

# Windows:
# Install Visual Studio Build Tools
```

**Tests Fail with Network Errors**:
```bash
# Check internet connection
ping api.devnet.solana.com

# Use local test validator (requires Solana CLI)
solana-test-validator

# Set test RPC to local validator
export SOLANA_RPC_URL="http://localhost:8899"
```

**Cargo Build is Slow**:
```bash
# Use cargo-cache to clean old builds
cargo install cargo-cache
cargo cache --autoclean

# Use more parallel jobs
cargo build -j 8
```

**Clippy Warnings**:
```bash
# Current codebase has some clippy warnings that are non-critical
# You can run clippy to see them:
cargo clippy

# To ignore existing warnings while working:
cargo clippy --all-targets --all-features
```

**IDE Not Recognizing Code**:
```bash
# Regenerate IDE files
cargo clean
cargo build

# Update rust-analyzer
rustup update
```

### Getting Help

1. **Check existing issues**: [GitHub Issues](https://github.com/opensvm/solana-mcp-server/issues)
2. **Search documentation**: Use the search in GitHub
3. **Ask questions**: Open a new issue with the "question" label
4. **Join discussions**: [GitHub Discussions](https://github.com/opensvm/solana-mcp-server/discussions)

## 📖 Resources

### Documentation

- [Architecture Overview](./ARCHITECTURE.md) - System design and components
- [API Reference](./API_REFERENCE.md) - Complete RPC method documentation
- [Configuration Guide](./CONFIGURATION.md) - Configuration options and management
- [Deployment Guide](./DEPLOYMENT.md) - Production deployment instructions
- [Examples](./EXAMPLES.md) - Practical usage examples

### Rust Learning Resources

- [The Rust Programming Language](https://doc.rust-lang.org/book/) - Official Rust book
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by examples
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises
- [Cargo Book](https://doc.rust-lang.org/cargo/) - Cargo documentation

### Solana Resources

- [Solana Documentation](https://docs.solana.com/) - Official Solana docs
- [Solana Cookbook](https://solanacookbook.com/) - Developer recipes
- [Solana RPC API](https://docs.solana.com/api/http) - RPC method reference

### Model Context Protocol

- [MCP Specification](https://modelcontextprotocol.io/docs/specification) - Protocol specification
- [MCP SDK](https://github.com/modelcontextprotocol/sdk) - Official SDK
- [MCP Examples](https://github.com/modelcontextprotocol/examples) - Example implementations

## 🎉 Next Steps

Congratulations! You now have a solid foundation for contributing to the Solana MCP Server. Here's what you can do next:

1. **Explore the codebase**: Start with `src/main.rs` and `src/lib.rs`
2. **Try making small changes**: Add a log message or modify a test
3. **Read the detailed docs**: Dive deeper into [ARCHITECTURE.md](./ARCHITECTURE.md)
4. **Pick up an issue**: Look for "good first issue" labels
5. **Join the community**: Participate in discussions and help others

Welcome to the team! 🚀

---

*This guide is maintained by the community. If you find issues or have suggestions for improvements, please [open an issue](https://github.com/opensvm/solana-mcp-server/issues/new) or submit a PR.*