/// Local sBPF binary testing module using liteSVM
///
/// Provides tools for testing compiled Solana programs locally
/// without deploying to devnet/testnet/mainnet.

pub mod binary_validator;
pub mod devnet_deployer;
pub mod errors;
pub mod test_executor;
pub mod types;
pub mod vm_wrapper;

// Re-export main types
pub use binary_validator::BinaryValidator;
pub use devnet_deployer::DevnetDeployer;
pub use errors::SbpfError;
pub use test_executor::TestExecutor;
pub use types::*;
pub use vm_wrapper::SbpfVmWrapper;
