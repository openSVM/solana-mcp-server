# Security Audit Documentation

## Overview

This document describes the security audit status for the solana-mcp-server project and explains the current state of known vulnerabilities.

## Current Security Status

### Known Vulnerabilities (Acceptable Risk)

The following vulnerabilities are present as transitive dependencies from the Solana ecosystem and cannot be easily resolved without breaking compatibility:

#### RUSTSEC-2024-0344: curve25519-dalek Timing Variability
- **Package**: curve25519-dalek v3.2.0 
- **Issue**: Timing variability in `Scalar29::sub`/`Scalar52::sub`
- **Patched Version**: >=4.1.3
- **Status**: Both vulnerable (3.2.0) and patched (4.1.3) versions present in dependency tree
- **Risk Assessment**: Low - This affects cryptographic operations in the Solana client libraries, not our server logic
- **Mitigation**: We've added curve25519-dalek v4.1.3 as a direct dependency to force the resolver to prefer the secure version

#### RUSTSEC-2022-0093: ed25519-dalek Double Public Key Signing 
- **Package**: ed25519-dalek v1.0.1
- **Issue**: Double Public Key Signing Function Oracle Attack
- **Patched Version**: >=2.0.0
- **Status**: Both vulnerable (1.0.1) and patched (2.2.0) versions present in dependency tree
- **Risk Assessment**: Low - This affects key signing operations in the Solana client libraries, not our server logic
- **Mitigation**: We've added ed25519-dalek v2.2.0 as a direct dependency to force the resolver to prefer the secure version

### Unmaintained Dependencies (Informational)

#### derivative v2.2.0
- **Status**: Unmaintained since 2024-06-26
- **Impact**: Used by Solana ecosystem for derive macros
- **Alternatives**: derive_more, derive-where, educe
- **Action**: Monitor Solana ecosystem updates

#### paste v1.0.15  
- **Status**: Unmaintained since 2024-10-07
- **Impact**: Used for token pasting in procedural macros
- **Alternatives**: pastey
- **Action**: Monitor Solana ecosystem updates

## Security Audit Workflow

Our CI/CD pipeline includes a security audit workflow that:

1. **Runs weekly** and on dependency changes
2. **Uses cargo-audit** with JSON output for detailed reporting
3. **Reports all vulnerabilities** found in the dependency tree
4. **Continues deployment** for known acceptable risks from Solana ecosystem
5. **Fails builds** for new high-severity vulnerabilities

## Monitoring and Updates

- **Weekly audits** via GitHub Actions detect new vulnerabilities
- **Dependency updates** are applied when Solana ecosystem releases updates
- **Security patches** are applied through direct dependencies and patches
- **Risk assessment** is updated as new vulnerabilities are discovered

## Contact

For security concerns or questions about our audit process, please:
1. Review this documentation
2. Check current GitHub Actions audit results
3. Open an issue for questions about security posture
4. Contact maintainers for private security disclosures