use super::binary_validator::BinaryValidator;
use super::errors::SbpfError;
use super::types::DevnetDeployResponse;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

const DEFAULT_DEVNET_URL: &str = "https://api.devnet.solana.com";

/// Provides deployment guidance and validation for Solana devnet
pub struct DevnetDeployer {
    rpc_url: String,
}

impl DevnetDeployer {
    /// Create a new devnet deployer
    pub fn new(rpc_url: Option<String>) -> Self {
        Self {
            rpc_url: rpc_url.unwrap_or_else(|| DEFAULT_DEVNET_URL.to_string()),
        }
    }

    /// Prepare a program for devnet deployment
    /// Returns validation results and deployment instructions
    pub async fn prepare_deployment(
        &self,
        binary: Vec<u8>,
    ) -> Result<DevnetDeployResponse, SbpfError> {
        // 1. Validate binary first
        let metadata = BinaryValidator::validate(&binary)?;
        log::info!(
            "Validated binary for devnet deployment: {} bytes, {} sections",
            metadata.size_bytes,
            metadata.sections.len()
        );

        if !metadata.errors.is_empty() {
            log::warn!("Binary validation warnings: {:?}", metadata.errors);
        }

        // 2. Generate deployment instructions
        let temp_file_path = "/tmp/program_deploy.so";
        let encoded_binary = BASE64.encode(&binary);

        let cli_commands = format!(
            r#"# Solana CLI Deployment Instructions

## Step 1: Save your program binary
echo '{encoded}' | base64 -d > {temp_file}

## Step 2: Configure Solana CLI for devnet
solana config set --url {rpc_url}

## Step 3: Create or use existing keypair
# Create new: solana-keygen new -o ~/.config/solana/devnet-deploy-keypair.json
# Or use existing keypair file

## Step 4: Request devnet airdrop (if needed)
solana airdrop 2

## Step 5: Deploy your program
solana program deploy {temp_file}

# This will output your program ID which you can use to interact with the program
"#,
            encoded = if binary.len() > 100_000 {
                "<binary-too-large-for-inline>".to_string()
            } else {
                encoded_binary
            },
            temp_file = temp_file_path,
            rpc_url = self.rpc_url
        );

        Ok(DevnetDeployResponse {
            program_id: "Will be generated during deployment".to_string(),
            signature: "N/A - Use CLI deployment".to_string(),
            deployed: false,
            size_bytes: binary.len(),
            network: "devnet".to_string(),
            rpc_url: self.rpc_url.clone(),
            cli_commands: Some(cli_commands),
            binary_valid: true,
            validation_notes: if metadata.errors.is_empty() {
                None
            } else {
                Some(metadata.errors.join("; "))
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployer_creation() {
        let deployer = DevnetDeployer::new(None);
        assert_eq!(deployer.rpc_url, DEFAULT_DEVNET_URL);

        let custom_url = "https://api.testnet.solana.com".to_string();
        let deployer2 = DevnetDeployer::new(Some(custom_url.clone()));
        assert_eq!(deployer2.rpc_url, custom_url);
    }
}
