#!/bin/bash
# One-liner installation script for Solana MCP Server
# Usage: curl -fsSL https://raw.githubusercontent.com/opensvm/solana-mcp-server/main/scripts/install.sh | bash
set -e

echo "🚀 Installing Solana MCP Server for Claude Desktop..."

# Function to detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$arch" in
        x86_64) arch="amd64" ;;
        arm64|aarch64) arch="arm64" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
    esac
    
    case "$os" in
        darwin) os="macos" ;;
        linux) os="linux" ;;
        *) echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac
    
    echo "${os}-${arch}"
}

# Function to get Claude config directory
get_claude_config_dir() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "$HOME/Library/Application Support/Claude"
    else
        echo "${XDG_CONFIG_HOME:-$HOME/.config}/claude"
    fi
}

# Try to download pre-built binary first, fall back to building from source
try_download_binary() {
    local platform=$(detect_platform)
    local binary_name="solana-mcp-server-${platform}"
    
    echo "Attempting to download pre-built binary for ${platform}..."
    
    # Try to download from releases
    local download_url="https://github.com/opensvm/solana-mcp-server/releases/latest/download/${binary_name}"
    if curl -sL "$download_url" -o solana-mcp-server; then
        # Check if download was successful (not a 404 page)
        if [[ $(wc -c < solana-mcp-server) -gt 1000 ]] && file solana-mcp-server | grep -q "executable"; then
            chmod +x solana-mcp-server
            echo "✅ Downloaded pre-built binary"
            return 0
        else
            echo "⚠️  Download failed or binary not available"
            rm -f solana-mcp-server
            return 1
        fi
    else
        echo "⚠️  Pre-built binary not available, building from source..."
        return 1
    fi
}

# Build from source
build_from_source() {
    echo "Building Solana MCP Server from source..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Clone and build
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    git clone https://github.com/opensvm/solana-mcp-server.git .
    cargo build --release
    
    # Copy binary to working directory
    cp target/release/solana-mcp-server "$OLDPWD/"
    cd "$OLDPWD"
    rm -rf "$TEMP_DIR"
    
    echo "✅ Built from source successfully"
}

# Main installation logic
main() {
    # Create working directory
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    cd "$INSTALL_DIR"
    
    # Try download first, then build if needed
    if ! try_download_binary; then
        build_from_source
    fi
    
    # Configure Claude Desktop
    local claude_config_dir=$(get_claude_config_dir)
    mkdir -p "$claude_config_dir"
    
    local config_file="$claude_config_dir/claude_desktop_config.json"
    local server_path="$INSTALL_DIR/solana-mcp-server"
    
    # Create or update Claude config
    if [[ -f "$config_file" ]]; then
        echo "⚠️  Existing Claude config found. Backing up..."
        cp "$config_file" "${config_file}.backup.$(date +%s)"
    fi
    
    cat > "$config_file" << EOF
{
  "mcpServers": {
    "solana": {
      "command": "$server_path",
      "env": {
        "SOLANA_RPC_URL": "https://api.mainnet-beta.solana.com",
        "SOLANA_COMMITMENT": "confirmed"
      }
    }
  }
}
EOF
    
    echo "✅ Installation complete!"
    echo ""
    echo "🎉 Solana MCP Server is now configured for Claude Desktop"
    echo "📁 Installed at: $server_path"
    echo "⚙️  Config file: $config_file"
    echo ""
    echo "🚀 To use:"
    echo "1. Restart Claude Desktop"
    echo "2. You can now query Solana blockchain data directly in Claude!"
    echo ""
    echo "💡 Example queries:"
    echo "   - 'What is the balance of account [pubkey]?'"
    echo "   - 'Show me the latest block information'"
    echo "   - 'Get transaction details for [signature]'"
    echo ""
    echo "📖 More info: https://github.com/opensvm/solana-mcp-server"
}

# Run main function
main "$@"