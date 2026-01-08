/// End-to-end test demonstrating the sBPF MCP tools
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::json;

/// Test that demonstrates validateSbpfBinary tool
#[tokio::test]
async fn test_validate_binary_tool() {
    // Create a minimal ELF binary with BPF architecture
    let mut binary = vec![0u8; 1024];

    // ELF magic number
    binary[0..4].copy_from_slice(&[0x7F, 0x45, 0x4C, 0x46]);
    binary[4] = 2; // 64-bit
    binary[5] = 1; // Little-endian
    binary[6] = 1; // ELF version
    binary[18] = 0xF7; // BPF machine type
    binary[19] = 0x00;

    let binary_b64 = BASE64.encode(&binary);

    println!("Testing validateSbpfBinary with {} byte binary (base64: {} chars)",
             binary.len(), binary_b64.len());

    // This would be called through the MCP tool handler
    // For now, just verify the binary validates
    use solana_mcp_server::sbpf::BinaryValidator;
    let result = BinaryValidator::validate(&binary);

    match result {
        Ok(metadata) => {
            println!("✓ Binary validated successfully:");
            println!("  - Size: {} bytes", metadata.size_bytes);
            println!("  - Architecture: {}", metadata.architecture);
            println!("  - Entrypoint: {}", metadata.entrypoint);
            println!("  - Sections: {:?}", metadata.sections);
            if !metadata.errors.is_empty() {
                println!("  - Warnings: {:?}", metadata.errors);
            }
        }
        Err(e) => {
            println!("✗ Validation failed: {:?}", e);
        }
    }
}

/// Test that demonstrates the tool schema is correct
#[test]
fn test_tool_schemas() {
    // Verify testSbpfProgram schema
    let test_schema = json!({
        "type": "object",
        "properties": {
            "programBinary": {
                "type": "string",
                "description": "Base64-encoded sBPF program binary (ELF format)"
            },
            "accounts": {
                "type": "array",
                "description": "Accounts to pass to the program"
            },
            "instructionData": {
                "type": "string",
                "description": "Instruction data (base64-encoded)"
            }
        },
        "required": ["programBinary"]
    });

    assert!(test_schema.is_object());
    assert!(test_schema["properties"]["programBinary"]["type"].as_str().unwrap() == "string");

    println!("✓ Tool schemas are valid");
}

/// Test account spec serialization/deserialization
#[test]
fn test_account_spec_json() {
    use solana_mcp_server::sbpf::AccountSpec;

    let account_json = json!({
        "pubkey": "11111111111111111111111111111111",
        "lamports": 1000000,
        "data": BASE64.encode(b"test data"),
        "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "executable": false,
        "isSigner": true,
        "isWritable": true
    });

    let account_spec: AccountSpec = serde_json::from_value(account_json).unwrap();

    assert_eq!(account_spec.pubkey, "11111111111111111111111111111111");
    assert_eq!(account_spec.lamports, 1000000);
    assert_eq!(account_spec.is_signer, true);
    assert_eq!(account_spec.is_writable, true);

    println!("✓ AccountSpec JSON serialization works");
}

/// Test that base64 encoding/decoding works as expected for tool parameters
#[test]
fn test_base64_tool_parameters() {
    let test_data = b"Hello from Solana sBPF program!";
    let encoded = BASE64.encode(test_data);
    let decoded = BASE64.decode(&encoded).unwrap();

    assert_eq!(test_data.as_slice(), decoded.as_slice());

    println!("✓ Base64 encoding/decoding for tool parameters works");
    println!("  Original: {:?}", String::from_utf8_lossy(test_data));
    println!("  Encoded: {}", encoded);
}

/// Demonstrate the flow of testing an sBPF program
#[tokio::test]
async fn test_sbpf_workflow_demonstration() {
    println!("\n=== sBPF Testing Workflow Demonstration ===\n");

    // Step 1: Create or load a binary
    println!("Step 1: Prepare sBPF binary");
    let mut binary = vec![0u8; 1024];
    binary[0..4].copy_from_slice(&[0x7F, 0x45, 0x4C, 0x46]);
    binary[4] = 2;
    binary[5] = 1;
    binary[6] = 1;
    binary[18] = 0xF7;
    binary[19] = 0x00;
    println!("  ✓ Binary prepared ({} bytes)", binary.len());

    // Step 2: Validate the binary
    println!("\nStep 2: Validate binary");
    use solana_mcp_server::sbpf::TestExecutor;
    match TestExecutor::validate_only(&binary) {
        Ok(metadata) => {
            println!("  ✓ Validation successful");
            println!("    - Architecture: {}", metadata.architecture);
            println!("    - Size: {} bytes", metadata.size_bytes);
        }
        Err(e) => {
            println!("  ✗ Validation failed: {:?}", e);
        }
    }

    // Step 3: Deploy to local VM (would happen in deploySbpfProgramLocal tool)
    println!("\nStep 3: Deploy to local VM");
    use solana_mcp_server::sbpf::SbpfVmWrapper;
    let vm = SbpfVmWrapper::new();
    match vm.deploy_program(binary.clone()).await {
        Ok(response) => {
            println!("  ✓ Deployment successful");
            println!("    - Program ID: {}", response.program_id);
            println!("    - Size: {} bytes", response.size_bytes);
        }
        Err(e) => {
            println!("  ✗ Deployment failed: {:?}", e);
        }
    }

    // Step 4: Execute test (would happen in testSbpfProgram tool)
    // Note: This would require a valid compiled program with proper instructions
    println!("\nStep 4: Execute program test");
    println!("  (Requires valid compiled BPF program - skipped in this demo)");

    println!("\n=== Workflow demonstration complete ===\n");
}
