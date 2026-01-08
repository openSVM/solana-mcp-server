use super::errors::SbpfError;
use super::types::BinaryMetadata;
use goblin::elf::Elf;

const MAX_BINARY_SIZE: usize = 512 * 1024 * 1024; // 512MB
const MIN_BINARY_SIZE: usize = 64;
const BPF_MACHINE_TYPE: u16 = 0x107; // eBPF (Solana uses extended BPF)
const BPF_CLASSIC_MACHINE_TYPE: u16 = 0xF7; // Classic BPF

pub struct BinaryValidator;

impl BinaryValidator {
    /// Validate an sBPF binary and extract metadata
    pub fn validate(data: &[u8]) -> Result<BinaryMetadata, SbpfError> {
        let mut errors = Vec::new();

        // 1. Size validation
        if data.len() < MIN_BINARY_SIZE {
            return Err(SbpfError::BinaryTooSmall { size: data.len() });
        }

        if data.len() > MAX_BINARY_SIZE {
            return Err(SbpfError::BinaryTooLarge {
                size: data.len(),
                max: MAX_BINARY_SIZE,
            });
        }

        // 2. ELF header validation
        if data.len() < 4 || &data[0..4] != &[0x7F, 0x45, 0x4C, 0x46] {
            return Err(SbpfError::NotElfFile);
        }

        // 3. Parse ELF
        let elf = match Elf::parse(data) {
            Ok(elf) => elf,
            Err(e) => {
                return Err(SbpfError::InvalidBinary(format!(
                    "ELF parse error: {}",
                    e
                )));
            }
        };

        // 4. Verify BPF architecture (accept both eBPF and classic BPF)
        if elf.header.e_machine != BPF_MACHINE_TYPE
            && elf.header.e_machine != BPF_CLASSIC_MACHINE_TYPE
        {
            return Err(SbpfError::NotBpfArchitecture(elf.header.e_machine));
        }

        // 5. Extract sections
        let sections: Vec<String> = elf
            .section_headers
            .iter()
            .filter_map(|sh| {
                elf.shdr_strtab
                    .get_at(sh.sh_name)
                    .map(|s| s.to_string())
            })
            .collect();

        // 6. Validate has required sections
        let has_text = sections.iter().any(|s| s == ".text");
        if !has_text {
            errors.push("Missing .text section".to_string());
        }

        // Return metadata
        Ok(BinaryMetadata {
            size_bytes: data.len(),
            architecture: "BPF".to_string(),
            entrypoint: format!("0x{:x}", elf.header.e_entry),
            sections,
            errors,
        })
    }

    /// Quick size check without full validation
    pub fn check_size(data: &[u8]) -> Result<(), SbpfError> {
        if data.len() < MIN_BINARY_SIZE {
            return Err(SbpfError::BinaryTooSmall { size: data.len() });
        }

        if data.len() > MAX_BINARY_SIZE {
            return Err(SbpfError::BinaryTooLarge {
                size: data.len(),
                max: MAX_BINARY_SIZE,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_too_small() {
        let data = vec![0u8; 32];
        let result = BinaryValidator::validate(&data);
        assert!(matches!(result, Err(SbpfError::BinaryTooSmall { .. })));
    }

    #[test]
    fn test_reject_not_elf() {
        let data = vec![0u8; 1024];
        let result = BinaryValidator::validate(&data);
        assert!(matches!(result, Err(SbpfError::NotElfFile)));
    }

    #[test]
    fn test_size_check() {
        assert!(BinaryValidator::check_size(&vec![0u8; 32]).is_err());
        assert!(BinaryValidator::check_size(&vec![0u8; 1024]).is_ok());
    }
}
