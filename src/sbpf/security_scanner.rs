use super::errors::SbpfError;
use goblin::elf::Elf;
use serde::{Deserialize, Serialize};

/// Security vulnerability severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// A detected security vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub severity: Severity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Security scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    pub binary_size: usize,
    pub vulnerabilities: Vec<Vulnerability>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub risk_score: u32, // 0-100, higher is worse
    pub passed: bool,
}

pub struct SecurityScanner;

impl SecurityScanner {
    /// Perform comprehensive security scan on sBPF binary
    pub fn scan(data: &[u8]) -> Result<SecurityScanResult, SbpfError> {
        // Parse ELF
        if data.len() < 4 || &data[0..4] != &[0x7F, 0x45, 0x4C, 0x46] {
            return Err(SbpfError::NotElfFile);
        }

        let elf = Elf::parse(data)
            .map_err(|e| SbpfError::InvalidBinary(format!("ELF parse error: {}", e)))?;

        let mut vulnerabilities = Vec::new();

        // Run all security checks
        Self::check_binary_size(&mut vulnerabilities, data.len());
        Self::check_elf_structure(&mut vulnerabilities, &elf);
        Self::check_sections(&mut vulnerabilities, &elf);
        Self::check_symbols(&mut vulnerabilities, &elf);
        Self::check_relocations(&mut vulnerabilities, &elf);
        Self::check_code_patterns(&mut vulnerabilities, data);
        Self::check_solana_security(&mut vulnerabilities, data);
        Self::check_arithmetic_safety(&mut vulnerabilities, data);
        Self::check_account_validation(&mut vulnerabilities, data);

        // Count by severity
        let critical_count = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .count();
        let high_count = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::High)
            .count();
        let medium_count = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::Medium)
            .count();
        let low_count = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::Low)
            .count();
        let info_count = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::Info)
            .count();

        // Calculate risk score (0-100) with exponential weighting
        // Multiple critical/high issues compound risk
        let base_score = critical_count * 30 + high_count * 18 + medium_count * 10 + low_count * 4;
        let multiplier = 1.0 + (critical_count as f32 * 0.2) + (high_count as f32 * 0.1);
        let risk_score = ((base_score as f32 * multiplier).min(100.0)) as u32;

        // Pass if no critical or high severity issues
        let passed = critical_count == 0 && high_count == 0;

        Ok(SecurityScanResult {
            binary_size: data.len(),
            vulnerabilities,
            critical_count,
            high_count,
            medium_count,
            low_count,
            info_count,
            risk_score,
            passed,
        })
    }

    fn check_binary_size(vulnerabilities: &mut Vec<Vulnerability>, size: usize) {
        // Warn about very large binaries
        if size > 256 * 1024 {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Medium,
                category: "Binary Size".to_string(),
                title: "Large binary size detected".to_string(),
                description: format!(
                    "Binary is {} bytes ({}KB). Solana has a 10MB limit, but smaller is better for faster loading and lower deployment costs.",
                    size,
                    size / 1024
                ),
                recommendation: "Consider optimizing binary size by removing debug symbols, unused code, and using release builds with size optimization.".to_string(),
                location: None,
            });
        }

        // Info for very small binaries (might be incomplete)
        if size < 2048 {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Info,
                category: "Binary Size".to_string(),
                title: "Very small binary detected".to_string(),
                description: format!(
                    "Binary is only {} bytes. This might indicate a minimal program or incomplete build.",
                    size
                ),
                recommendation: "Verify this is a complete, production-ready build.".to_string(),
                location: None,
            });
        }
    }

    fn check_elf_structure(vulnerabilities: &mut Vec<Vulnerability>, elf: &Elf) {
        // Check architecture
        if elf.header.e_machine != 0x107 && elf.header.e_machine != 0xF7 {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Critical,
                category: "ELF Structure".to_string(),
                title: "Invalid BPF architecture".to_string(),
                description: format!(
                    "ELF machine type is 0x{:x}, but Solana requires BPF (0x107 or 0xF7)",
                    elf.header.e_machine
                ),
                recommendation: "Rebuild using cargo build-sbf or anchor build.".to_string(),
                location: Some("ELF Header".to_string()),
            });
        }

        // Check entry point
        if elf.header.e_entry == 0 {
            vulnerabilities.push(Vulnerability {
                severity: Severity::High,
                category: "ELF Structure".to_string(),
                title: "Missing entry point".to_string(),
                description: "ELF entry point is 0, which means the program has no defined entry point.".to_string(),
                recommendation: "Ensure your program exports the entrypoint function correctly.".to_string(),
                location: Some("ELF Header".to_string()),
            });
        }
    }

    fn check_sections(vulnerabilities: &mut Vec<Vulnerability>, elf: &Elf) {
        let section_names: Vec<String> = elf
            .section_headers
            .iter()
            .filter_map(|sh| elf.shdr_strtab.get_at(sh.sh_name).map(|s| s.to_string()))
            .collect();

        // Check for .text section
        if !section_names.iter().any(|s| s == ".text") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Critical,
                category: "Sections".to_string(),
                title: "Missing .text section".to_string(),
                description: "The binary has no .text section containing executable code."
                    .to_string(),
                recommendation: "This is likely a malformed binary. Rebuild your program."
                    .to_string(),
                location: None,
            });
        }

        // Check for suspicious sections that shouldn't be in production
        let debug_sections = [".debug_info", ".debug_line", ".debug_str", ".debug_abbrev"];
        for debug_sec in &debug_sections {
            if section_names.iter().any(|s| s == debug_sec) {
                vulnerabilities.push(Vulnerability {
                    severity: Severity::Low,
                    category: "Sections".to_string(),
                    title: format!("Debug section {} present", debug_sec),
                    description: "Debug sections increase binary size and deployment cost without providing runtime benefit.".to_string(),
                    recommendation: "Build with --release and strip debug symbols: cargo build-sbf --release".to_string(),
                    location: Some(debug_sec.to_string()),
                });
                break; // Only report once
            }
        }

        // Look for .rodata section (good - indicates string/constant usage)
        if section_names.iter().any(|s| s == ".rodata") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Info,
                category: "Sections".to_string(),
                title: "Read-only data section present".to_string(),
                description: "Program contains read-only data (.rodata), which is normal for programs with string literals or constants.".to_string(),
                recommendation: "No action needed - this is expected.".to_string(),
                location: Some(".rodata".to_string()),
            });
        }
    }

    fn check_symbols(vulnerabilities: &mut Vec<Vulnerability>, elf: &Elf) {
        // Check if we have symbols
        if elf.syms.is_empty() {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Low,
                category: "Symbols".to_string(),
                title: "No symbols found".to_string(),
                description: "Binary has been stripped of all symbols. This is normal for release builds but makes debugging harder.".to_string(),
                recommendation: "Keep debug builds with symbols for development, use stripped builds for production.".to_string(),
                location: None,
            });
        } else {
            // Look for entrypoint symbol
            let has_entrypoint = elf.syms.iter().any(|sym| {
                if let Some(name) = elf.strtab.get_at(sym.st_name) {
                    name == "entrypoint" || name.contains("entrypoint")
                } else {
                    false
                }
            });

            if !has_entrypoint {
                vulnerabilities.push(Vulnerability {
                    severity: Severity::Medium,
                    category: "Symbols".to_string(),
                    title: "No entrypoint symbol found".to_string(),
                    description: "Could not locate an 'entrypoint' symbol in the binary."
                        .to_string(),
                    recommendation: "Verify your program exports the entrypoint correctly."
                        .to_string(),
                    location: None,
                });
            }
        }
    }

    fn check_relocations(vulnerabilities: &mut Vec<Vulnerability>, elf: &Elf) {
        // Check for dynamic relocations
        if !elf.dynrelas.is_empty() || !elf.dynrels.is_empty() {
            vulnerabilities.push(Vulnerability {
                severity: Severity::High,
                category: "Relocations".to_string(),
                title: "Dynamic relocations detected".to_string(),
                description: format!(
                    "Binary contains {} dynamic relocations. Solana BPF programs should be statically linked.",
                    elf.dynrelas.len() + elf.dynrels.len()
                ),
                recommendation: "Ensure all dependencies are statically linked. Check your build configuration.".to_string(),
                location: None,
            });
        }

        // PLT relocations are suspicious in BPF
        if !elf.pltrelocs.is_empty() {
            vulnerabilities.push(Vulnerability {
                severity: Severity::High,
                category: "Relocations".to_string(),
                title: "PLT relocations detected".to_string(),
                description: "Binary contains PLT (Procedure Linkage Table) relocations, which are not supported in Solana BPF.".to_string(),
                recommendation: "Rebuild with static linking and no PLT.".to_string(),
                location: None,
            });
        }
    }

    fn check_code_patterns(vulnerabilities: &mut Vec<Vulnerability>, data: &[u8]) {
        // Scan for common vulnerability patterns in the binary

        // 1. Look for panic/abort patterns (common in Rust debug builds)
        if Self::contains_pattern(data, b"panicked at") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Medium,
                category: "Code Patterns".to_string(),
                title: "Panic strings detected".to_string(),
                description: "Binary contains panic error messages, indicating possible panic! calls that waste compute units.".to_string(),
                recommendation: "Use Result types and proper error handling instead of panic! in production code.".to_string(),
                location: None,
            });
        }

        // 2. Look for unsafe patterns
        if Self::contains_pattern(data, b"unsafe") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Info,
                category: "Code Patterns".to_string(),
                title: "Unsafe code detected".to_string(),
                description: "Binary may contain unsafe Rust code. This is not necessarily bad but requires careful auditing.".to_string(),
                recommendation: "Carefully audit all unsafe blocks for memory safety issues.".to_string(),
                location: None,
            });
        }

        // 3. Check for unoptimized division/modulo (expensive in BPF)
        // BPF div/mod instructions are opcode 0x3* and 0x9*
        let mut div_count = 0;
        for window in data.windows(8) {
            // Look for BPF division instructions (simplified heuristic)
            if window[0] == 0x37 || window[0] == 0x97 {
                div_count += 1;
            }
        }

        if div_count > 50 {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Medium,
                category: "Performance".to_string(),
                title: "Excessive division operations".to_string(),
                description: format!(
                    "Detected approximately {} division/modulo operations. These are expensive in BPF (~20 compute units each).",
                    div_count
                ),
                recommendation: "Consider using bit shifts for power-of-2 divisions or precomputing values.".to_string(),
                location: None,
            });
        }

        // 4. Check binary size vs expected instruction count
        let estimated_instructions = data.len() / 8; // Rough estimate
        if estimated_instructions > 10000 {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Low,
                category: "Performance".to_string(),
                title: "High instruction count".to_string(),
                description: format!(
                    "Estimated ~{} BPF instructions. Large programs consume more compute units.",
                    estimated_instructions
                ),
                recommendation: "Profile your program and optimize hot paths. Consider breaking into multiple programs if needed.".to_string(),
                location: None,
            });
        }

        // 5. Look for common Solana security patterns (positive checks)
        if Self::contains_pattern(data, b"owner") && Self::contains_pattern(data, b"signer") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Info,
                category: "Security Patterns".to_string(),
                title: "Owner and signer checks detected".to_string(),
                description: "Binary appears to contain owner and signer validation logic (good practice).".to_string(),
                recommendation: "Verify these checks are performed on all privileged operations.".to_string(),
                location: None,
            });
        }
    }

    fn check_solana_security(vulnerabilities: &mut Vec<Vulnerability>, data: &[u8]) {
        // Check for Solana-specific security patterns and vulnerabilities

        // 1. CPI (Cross-Program Invocation) security
        if Self::contains_pattern(data, b"invoke") || Self::contains_pattern(data, b"invoke_signed") {
            // Check if there are proper signer checks around CPI calls
            let has_signer_check = Self::contains_pattern(data, b"is_signer");
            if !has_signer_check {
                vulnerabilities.push(Vulnerability {
                    severity: Severity::High,
                    category: "CPI Security".to_string(),
                    title: "CPI calls without signer validation".to_string(),
                    description: "Program contains CPI (invoke/invoke_signed) but no signer validation detected. This could allow unauthorized cross-program calls.".to_string(),
                    recommendation: "Always verify account.is_signer before making privileged CPI calls. Example: if !account.is_signer { return Err(...); }".to_string(),
                    location: None,
                });
            }
        }

        // 2. PDA (Program Derived Address) validation
        if Self::contains_pattern(data, b"create_program_address")
            || Self::contains_pattern(data, b"find_program_address") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Info,
                category: "PDA Usage".to_string(),
                title: "PDA derivation detected".to_string(),
                description: "Program uses Program Derived Addresses (PDAs). Ensure bump seeds are validated and PDAs are derived correctly.".to_string(),
                recommendation: "Always validate PDA bump seeds and verify derived addresses match expected values. Store bump seeds in account data.".to_string(),
                location: None,
            });
        }

        // 3. Rent exemption checks
        if !Self::contains_pattern(data, b"rent") && !Self::contains_pattern(data, b"minimum_balance") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Medium,
                category: "Rent Exemption".to_string(),
                title: "No rent checks detected".to_string(),
                description: "Program does not appear to check rent exemption. Accounts may be closed if not rent-exempt.".to_string(),
                recommendation: "Ensure all persistent accounts are rent-exempt: let rent = Rent::get()?; if !rent.is_exempt(account.lamports(), account.data_len()) { return Err(...); }".to_string(),
                location: None,
            });
        }

        // 4. Account reinitialization check
        if !Self::contains_pattern(data, b"is_initialized") && !Self::contains_pattern(data, b"discriminator") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::High,
                category: "Account Security".to_string(),
                title: "Missing initialization checks".to_string(),
                description: "No initialization state checks detected. Program may be vulnerable to reinitialization attacks.".to_string(),
                recommendation: "Add initialization flags to account data and check them: if account.is_initialized { return Err(AlreadyInitialized); }".to_string(),
                location: None,
            });
        }

        // 5. Token program security (if using SPL Token)
        if Self::contains_pattern(data, b"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
            // SPL Token program ID detected
            let has_amount_check = Self::contains_pattern(data, b"amount");
            if !has_amount_check {
                vulnerabilities.push(Vulnerability {
                    severity: Severity::High,
                    category: "Token Security".to_string(),
                    title: "SPL Token operations without amount validation".to_string(),
                    description: "Program interacts with SPL Token but no amount validation detected. May be vulnerable to zero-amount or overflow attacks.".to_string(),
                    recommendation: "Always validate token amounts: if amount == 0 || amount > MAX_AMOUNT { return Err(...); }".to_string(),
                    location: None,
                });
            }
        }
    }

    fn check_arithmetic_safety(vulnerabilities: &mut Vec<Vulnerability>, data: &[u8]) {
        // Check for arithmetic operation safety

        // 1. Look for checked math usage
        let has_checked_math = Self::contains_pattern(data, b"checked_add")
            || Self::contains_pattern(data, b"checked_sub")
            || Self::contains_pattern(data, b"checked_mul")
            || Self::contains_pattern(data, b"checked_div");

        // 2. Count potential arithmetic operations (BPF opcodes)
        let mut add_count = 0;
        let mut mul_count = 0;

        for window in data.windows(8) {
            match window[0] {
                0x07 | 0x0f => add_count += 1,      // BPF_ADD
                0x27 | 0x2f => mul_count += 1,      // BPF_MUL
                _ => {}
            }
        }

        let total_arithmetic = add_count + mul_count;

        // If there are many arithmetic operations but no checked math
        if total_arithmetic > 20 && !has_checked_math {
            vulnerabilities.push(Vulnerability {
                severity: Severity::High,
                category: "Arithmetic Safety".to_string(),
                title: "Unchecked arithmetic operations".to_string(),
                description: format!(
                    "Detected ~{} arithmetic operations (add/mul) with no checked_* function usage. Vulnerable to integer overflow/underflow.",
                    total_arithmetic
                ),
                recommendation: "Use checked arithmetic: amount.checked_add(value).ok_or(ErrorCode::Overflow)? instead of amount + value".to_string(),
                location: None,
            });
        }

        // 3. Look for potential price calculation vulnerabilities
        if Self::contains_pattern(data, b"price") || Self::contains_pattern(data, b"rate") {
            if !has_checked_math {
                vulnerabilities.push(Vulnerability {
                    severity: Severity::Critical,
                    category: "Arithmetic Safety".to_string(),
                    title: "Price calculations without overflow protection".to_string(),
                    description: "Program performs price/rate calculations without checked arithmetic. Critical vulnerability - attackers can manipulate prices via integer overflow.".to_string(),
                    recommendation: "CRITICAL: Use checked_mul() and checked_div() for all price calculations. Example: let value = price.checked_mul(amount)?.checked_div(decimals)?;".to_string(),
                    location: None,
                });
            }
        }
    }

    fn check_account_validation(vulnerabilities: &mut Vec<Vulnerability>, data: &[u8]) {
        // Check for proper account validation patterns

        // 1. Owner checks
        let has_owner_check = Self::contains_pattern(data, b"owner") && Self::contains_pattern(data, b"key");
        if !has_owner_check {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Critical,
                category: "Account Validation".to_string(),
                title: "Missing account owner checks".to_string(),
                description: "No account owner validation detected. This is the #1 cause of Solana exploits. Attackers can pass malicious accounts owned by their programs.".to_string(),
                recommendation: "CRITICAL: Verify account ownership: if account.owner != program_id { return Err(InvalidOwner); } or if account.owner != expected_program { return Err(InvalidOwner); }".to_string(),
                location: None,
            });
        }

        // 2. Signer checks
        let has_signer_check = Self::contains_pattern(data, b"is_signer");
        if !has_signer_check {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Critical,
                category: "Account Validation".to_string(),
                title: "Missing signer verification".to_string(),
                description: "No signer checks detected. Attackers can pass accounts they don't own and execute privileged operations.".to_string(),
                recommendation: "CRITICAL: Verify signers for privileged operations: if !authority.is_signer { return Err(MissingSignature); }".to_string(),
                location: None,
            });
        }

        // 3. Account data length validation
        if !Self::contains_pattern(data, b"data_len") && !Self::contains_pattern(data, b"len()") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::High,
                category: "Account Validation".to_string(),
                title: "No account data length validation".to_string(),
                description: "Program doesn't validate account data length before access. May cause out-of-bounds reads or panics.".to_string(),
                recommendation: "Validate data length: if account.data.len() < EXPECTED_SIZE { return Err(InvalidDataLength); }".to_string(),
                location: None,
            });
        }

        // 4. Writable account checks
        if Self::contains_pattern(data, b"is_writable") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Info,
                category: "Account Validation".to_string(),
                title: "Writable account checks present".to_string(),
                description: "Program validates account mutability (good practice). Ensure all accounts that will be modified are checked.".to_string(),
                recommendation: "Continue validating: if !account.is_writable { return Err(AccountNotWritable); }".to_string(),
                location: None,
            });
        }
    }

    fn contains_pattern(data: &[u8], pattern: &[u8]) -> bool {
        data.windows(pattern.len()).any(|window| window == pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_invalid_elf() {
        let data = vec![0u8; 1024];
        let result = SecurityScanner::scan(&data);
        assert!(matches!(result, Err(SbpfError::NotElfFile)));
    }

    #[test]
    fn test_contains_pattern() {
        let data = b"Hello world panicked at something";
        assert!(SecurityScanner::contains_pattern(data, b"panicked at"));
        assert!(!SecurityScanner::contains_pattern(data, b"not present"));
    }
}
