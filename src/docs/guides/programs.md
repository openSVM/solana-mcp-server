# Writing Solana Programs

This guide covers the fundamentals of writing programs (smart contracts) for the Solana blockchain.

## Getting Started

### Prerequisites
- Rust installed (latest stable version)
- Solana CLI tools
- Development environment setup
- Basic understanding of blockchain concepts

### Project Setup
1. Create new project:
```bash
cargo new my-solana-program --lib
```

2. Configure Cargo.toml:
```toml
[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
solana-program = "1.16"
borsh = "0.10"
thiserror = "1.0"
```

## Program Structure

### Entry Point
```rust
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    // Program logic here
    Ok(())
}
```

### Instruction Processing
1. Define Instructions:
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub enum ProgramInstruction {
    Initialize,
    Update { data: u64 },
    Close,
}
```

2. Process Instructions:
```rust
match ProgramInstruction::try_from_slice(instruction_data)? {
    ProgramInstruction::Initialize => {
        // Handle initialization
    }
    ProgramInstruction::Update { data } => {
        // Handle update
    }
    ProgramInstruction::Close => {
        // Handle close
    }
}
```

### Account Management
1. Account Validation:
```rust
fn validate_accounts(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    // Check account ownership
    if accounts[0].owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify signers
    if !accounts[1].is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    Ok(())
}
```

2. State Management:
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProgramState {
    pub admin: Pubkey,
    pub data: u64,
}

impl ProgramState {
    pub fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
        let data = account.try_borrow_data()?;
        let state = Self::try_from_slice(&data)?;
        Ok(state)
    }
    
    pub fn save(&self, account: &AccountInfo) -> ProgramResult {
        let mut data = account.try_borrow_mut_data()?;
        self.serialize(&mut *data)?;
        Ok(())
    }
}
```

## Error Handling

```rust
#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    #[error("Not rent exempt")]
    NotRentExempt,
    
    #[error("Account not initialized")]
    UninitializedAccount,
}
```

## Testing

1. Unit Tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initialize() {
        // Test initialization logic
    }
    
    #[test]
    fn test_update() {
        // Test update logic
    }
}
```

2. Program Tests:
```rust
#[cfg(test)]
mod program_test {
    use solana_program_test::*;
    use solana_sdk::{signature::Signer, transaction::Transaction};
    
    #[tokio::test]
    async fn test_full_flow() {
        let program_id = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) = 
            ProgramTest::new("program_name", program_id, None)
                .start()
                .await;
                
        // Test program flow
    }
}
```

## Best Practices

1. Security
- Always validate account ownership
- Check all account permissions
- Verify required signers
- Handle all error cases
- Protect against reentrancy

2. Performance
- Minimize account lookups
- Batch operations when possible
- Use efficient data structures
- Optimize compute usage

3. Development
- Use proper error handling
- Write comprehensive tests
- Document your code
- Follow Rust best practices

## Deployment

1. Build Program:
```bash
cargo build-bpf
```

2. Deploy:
```bash
solana program deploy target/deploy/program_name.so
```

3. Verify:
```bash
solana program show <PROGRAM_ID>
```

## Resources
- Solana Documentation
- Rust Documentation
- Program Examples
- Development Tools
