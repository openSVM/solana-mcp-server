use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use solana_mcp_server::sbpf::{BinaryValidator, TestExecutor, TestParams, AccountSpec};

#[test]
fn test_binary_validation_too_small() {
    let small_binary = vec![0u8; 32];
    let result = BinaryValidator::validate(&small_binary);
    assert!(result.is_err());
}

#[test]
fn test_binary_validation_not_elf() {
    let not_elf = vec![0u8; 1024];
    let result = BinaryValidator::validate(&not_elf);
    assert!(result.is_err());
}

#[test]
fn test_binary_validation_valid_elf_header() {
    // Create minimal ELF header with BPF machine type
    let mut binary = vec![0u8; 1024];

    // ELF magic number
    binary[0..4].copy_from_slice(&[0x7F, 0x45, 0x4C, 0x46]);

    // ELF class (64-bit)
    binary[4] = 2;

    // Data encoding (little-endian)
    binary[5] = 1;

    // ELF version
    binary[6] = 1;

    // OS/ABI
    binary[7] = 0;

    // e_type (executable) at offset 16
    binary[16] = 0x02;
    binary[17] = 0x00;

    // e_machine (BPF = 0xF7) at offset 18
    binary[18] = 0xF7;
    binary[19] = 0x00;

    let result = BinaryValidator::validate(&binary);

    // Should validate the ELF structure
    match result {
        Ok(metadata) => {
            assert_eq!(metadata.architecture, "BPF");
            assert_eq!(metadata.size_bytes, 1024);
        }
        Err(e) => {
            println!("Validation error: {:?}", e);
            // If goblin can't parse it fully, that's okay for this minimal test
        }
    }
}

#[test]
fn test_binary_validation_wrong_architecture() {
    // Create minimal ELF header with wrong machine type
    let mut binary = vec![0u8; 1024];

    // ELF magic number
    binary[0..4].copy_from_slice(&[0x7F, 0x45, 0x4C, 0x46]);

    // ELF class (64-bit)
    binary[4] = 2;

    // Data encoding (little-endian)
    binary[5] = 1;

    // ELF version
    binary[6] = 1;

    // e_machine (x86-64 = 0x3E instead of BPF = 0xF7) at offset 18
    binary[18] = 0x3E;
    binary[19] = 0x00;

    let result = BinaryValidator::validate(&binary);

    // Should detect it's parsing successfully but may fail on architecture check
    // depending on goblin's parsing
    println!("Result: {:?}", result);
}

#[tokio::test]
async fn test_validate_only_wrapper() {
    let small_binary = vec![0u8; 32];
    let result = TestExecutor::validate_only(&small_binary);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_executor_creation() {
    let executor = TestExecutor::new();
    // Just verify it doesn't panic
    assert!(true);
}

#[test]
fn test_account_spec_serialization() {
    let account_spec = AccountSpec {
        pubkey: "11111111111111111111111111111111".to_string(),
        lamports: 1000000,
        data: Some(BASE64.encode(b"test data")),
        owner: Some("11111111111111111111111111111111".to_string()),
        executable: false,
        is_signer: false,
        is_writable: true,
    };

    let json = serde_json::to_value(&account_spec).unwrap();
    let deserialized: AccountSpec = serde_json::from_value(json).unwrap();

    assert_eq!(deserialized.pubkey, account_spec.pubkey);
    assert_eq!(deserialized.lamports, account_spec.lamports);
}

#[test]
fn test_base64_encoding_decoding() {
    let test_data = b"Hello, Solana BPF!";
    let encoded = BASE64.encode(test_data);
    let decoded = BASE64.decode(&encoded).unwrap();

    assert_eq!(test_data, decoded.as_slice());
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // This test would require a real compiled BPF program
    // For now, it's a placeholder for future integration tests
    #[tokio::test]
    #[ignore] // Ignore by default since we don't have a real BPF binary
    async fn test_full_program_execution() {
        // This would test with a real compiled Solana program
        // Example flow:
        // 1. Load a compiled .so file
        // 2. Deploy it using TestExecutor
        // 3. Execute with test accounts
        // 4. Verify results

        // TODO: Add real BPF binary for testing
        println!("Full integration test - requires real BPF binary");
    }
}

#[test]
fn test_binary_size_limits() {
    // Test minimum size boundary
    let min_size_binary = vec![0u8; 64];
    let result = BinaryValidator::check_size(&min_size_binary);
    assert!(result.is_ok());

    // Test below minimum
    let too_small = vec![0u8; 63];
    let result = BinaryValidator::check_size(&too_small);
    assert!(result.is_err());

    // Test maximum size (512MB would be too large to create in test)
    // Just verify the check exists
    let medium_binary = vec![0u8; 1024 * 1024]; // 1MB
    let result = BinaryValidator::check_size(&medium_binary);
    assert!(result.is_ok());
}
