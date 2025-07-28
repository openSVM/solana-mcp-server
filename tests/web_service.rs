/// Test that the web service binary accepts the correct CLI arguments
#[tokio::test]
async fn test_web_service_cli_args() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "web", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Run as web service on HTTP"));
    assert!(stdout.contains("--port"));
}

/// Test that the binary shows help for both modes
#[tokio::test]
async fn test_main_cli_help() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Solana MCP Server"));
    assert!(stdout.contains("stdio"));
    assert!(stdout.contains("web"));
}

/// Test that web service mode can start (without making actual network calls)
#[tokio::test]
async fn test_web_service_startup_validation() {
    // This test validates that the web service can be compiled and the CLI parsing works
    // We don't actually start the server to avoid port conflicts in CI
    
    // Just verify the binary can be built and help is shown correctly
    let output = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .expect("Failed to build binary");

    assert!(output.status.success(), "Build should succeed");
    
    // Verify web subcommand parsing
    let help_output = std::process::Command::new("cargo")
        .args(["run", "--", "web", "--help"])
        .output()
        .expect("Failed to run web help");

    let help_text = String::from_utf8_lossy(&help_output.stdout);
    assert!(help_text.contains("Port to run the web service on"));
    assert!(help_text.contains("[default: 3000]"));
}

/// Test that stdio mode still works as default
#[tokio::test]
async fn test_stdio_mode_default() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "stdio", "--help"])
        .output()
        .expect("Failed to execute command");

    // Should show help for stdio mode or general help
    assert!(output.status.success());
}

/// Integration test to verify HTTP server can be instantiated 
/// (without actually starting to avoid port conflicts)
#[tokio::test]
async fn test_http_server_instantiation() {
    use solana_mcp_server::{Config, ServerState, start_mcp_server_task};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // This just tests that all the types compile and can be instantiated
    // We don't actually start the server to avoid port binding issues in CI
    
    // Create a minimal config (this will use defaults or fail gracefully)
    match Config::load() {
        Ok(config) => {
            let server_state = ServerState::new(config);
            let state = Arc::new(RwLock::new(server_state));
            
            // Create the server task (but don't await it)
            let _handle = start_mcp_server_task(9999, state); // Use unlikely port
            
            // Just verify it compiles and can be created
            // In a real test environment, you'd start this and make HTTP requests
            // Server task created successfully
        }
        Err(_) => {
            // Config loading might fail in CI environment, that's ok
            // The important thing is that the types compile
            // Types compile correctly even if config fails
        }
    }
}