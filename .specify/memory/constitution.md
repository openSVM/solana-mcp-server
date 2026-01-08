<!--
SYNC IMPACT REPORT
==================
Version: 0.0.0 → 1.0.0 (INITIAL CONSTITUTION)

Changes:
- Initial constitution creation from template
- All placeholders replaced with project-specific values
- Established 7 core principles for Solana MCP Server

Modified Principles:
- N/A (initial creation)

Added Sections:
- Core Principles: RPC Reliability, Type Safety, Error Handling, Testing First, Observability, Caching Strategy, MCP Protocol Compliance
- Security Requirements
- Development Workflow
- Governance

Removed Sections:
- N/A (initial creation)

Templates Requiring Updates:
- ✅ .specify/templates/plan-template.md - Constitution Check section aligned
- ✅ .specify/templates/spec-template.md - Requirements section aligned with principles
- ✅ .specify/templates/tasks-template.md - Task categorization reflects principles
- ⚠️ .specify/templates/commands/*.md - Review pending for consistency

Follow-up TODOs:
- None - all critical placeholders filled

Ratification Note:
This is the initial constitution establishing the governance framework for the Solana MCP Server project.
The version starts at 1.0.0 as this represents the first complete, ratified governance document.
-->

# Solana MCP Server Constitution

## Core Principles

### I. RPC Reliability First

Every RPC interaction MUST be designed for reliability and graceful degradation:

- All RPC calls MUST include comprehensive error handling with typed McpError responses
- Request logging MUST capture request_id, method, duration, network, and outcome (success/failure)
- Timeouts MUST be configurable and enforced at both client and server layers
- Network failures MUST NOT crash the server; errors return structured JSON-RPC responses
- Multi-network support MUST maintain isolation; one network failure MUST NOT affect others

**Rationale**: As a bridge between AI assistants and blockchain data, reliability is non-negotiable. Users depend on consistent, predictable responses even when upstream RPC nodes fail or exhibit latency.

### II. Type Safety & Validation (NON-NEGOTIABLE)

Input validation and type safety are mandatory at all boundaries:

- ALL public-facing parameters MUST be validated before use (pubkeys, signatures, slot numbers, etc.)
- Validation errors MUST return HTTP 400/JSON-RPC error codes with clear, safe error messages
- Use Rust's type system aggressively: newtype wrappers for domain concepts (Pubkey, Signature, Slot)
- No unsafe casting or .unwrap() in production code paths; use ? operator with proper error context
- Parameter schemas in MCP ToolDefinitions MUST match validation logic exactly

**Rationale**: Invalid input from AI assistants or malicious actors can cause panics, incorrect blockchain queries, or security vulnerabilities. Strong typing and validation prevent entire classes of bugs.

### III. Error Handling & User Experience

Errors MUST be clear, actionable, and never expose sensitive information:

- Use McpError builder pattern: `.with_method()`, `.with_request_id()`, `.with_parameter()`
- Error messages MUST be user-friendly via `.safe_message()` (no stack traces, internal paths, or RPC URLs in client responses)
- Structured logging (tracing) captures full error context for debugging without leaking to clients
- Every error type (Client, Server, Rpc, Validation, Network, Auth) MUST map to correct JSON-RPC error codes
- When RPC calls fail, MUST distinguish between client errors (invalid params) and server errors (RPC node issues)

**Rationale**: AI assistants relay errors to end users. Cryptic or verbose errors harm UX. Safe, structured errors enable debugging without security risks.

### IV. Testing First

Tests MUST be written before or alongside implementation for all non-trivial changes:

- Integration tests MUST cover end-to-end flows: stdio mode, web mode, websocket mode
- RPC method tests MUST verify both success cases and error handling (invalid params, network failures)
- Cache integration tests MUST validate TTL behavior, eviction, hit/miss metrics
- New RPC methods MUST include test coverage in tests/ directory before merging
- Use `cargo test -- --nocapture` to debug failing tests with full output

**Rationale**: The server mediates critical financial data queries. Regressions can cause incorrect balances, failed transactions, or broken integrations. Tests are insurance.

### V. Observability & Monitoring

Production deployments MUST be observable:

- Prometheus metrics MUST track: request counts (by method, network), latency histograms, error rates (by error type), cache hit/miss rates
- Structured logging (tracing with JSON output) MUST be used for all request lifecycle events
- Health endpoint (`/health`) MUST reflect actual server capability (RPC connectivity, cache status)
- Metrics endpoint (`/metrics`) MUST be accessible for Prometheus scraping
- Log levels MUST be configurable via RUST_LOG; default to INFO in production, DEBUG in development

**Rationale**: Without observability, debugging production issues is guesswork. Metrics and logs enable SRE teams to diagnose latency spikes, RPC failures, and cache performance.

### VI. Caching Strategy

RPC response caching MUST balance freshness and performance:

- TTL-based caching using DashMap for thread-safe concurrent access
- Method-specific TTL configuration in config.json (e.g., getBlock: 30s, getBalance: 5s)
- Cache keys MUST include method name AND full parameter set to prevent stale data
- Cache MUST track size and evict LRU entries when limit reached
- Prometheus metrics MUST expose cache_hits_total, cache_misses_total, cache_size
- Cached responses MUST include metadata (e.g., cached_at timestamp) where appropriate

**Rationale**: Solana RPC nodes can be rate-limited or slow. Caching reduces latency and costs while respecting data freshness requirements (recent blocks shouldn't be cached long).

### VII. MCP Protocol Compliance

MCP protocol implementation MUST be complete and correct:

- Protocol version MUST be declared and supported (currently `2024-11-05`)
- ToolDefinitions MUST include complete JSON Schema for input_schema
- JSON-RPC 2.0 MUST be strictly followed: id, method, params, result/error
- Stdio mode MUST use stderr for ALL logging (stdout reserved for JSON-RPC)
- Web mode MUST expose `/api/mcp` endpoint accepting MCP JSON-RPC requests
- Handle initialize, tools/list, and tools/call methods per MCP spec

**Rationale**: Claude Desktop and other MCP clients depend on strict protocol adherence. Deviation breaks integrations and user trust.

## Security Requirements

### Dependency Management

- Run `cargo audit` regularly (weekly minimum) to detect vulnerable dependencies
- Keep Solana SDK dependencies (~2.3) up to date with security patches
- Pin critical cryptographic dependencies (curve25519-dalek, ed25519-dalek) to secure versions
- Document acceptable security risks (e.g., deep transitive dependencies) in docs/security-audit.md

### Input Sanitization

- ALL user-provided pubkeys, signatures, and addresses MUST be validated via solana-sdk types
- Reject malformed base58 strings, invalid lengths, or out-of-range slot numbers
- Use URL parsing for RPC endpoint configuration; reject invalid URLs
- Never construct RPC calls via string concatenation; use typed client methods

### Deployment Security

- Environment variables (SOLANA_RPC_URL) MUST NOT be logged or exposed in error messages
- Health/metrics endpoints MUST NOT expose sensitive config (API keys, internal URLs)
- HTTPS MUST be enforced for web mode in production (TLS termination at load balancer acceptable)
- Rate limiting MUST be implemented at reverse proxy layer (not in-app) for DoS protection

## Development Workflow

### Code Review Requirements

- All PRs MUST pass: cargo test, cargo clippy, cargo fmt --check, cargo audit
- New RPC methods MUST include: implementation in src/rpc/, ToolDefinition, handler dispatch, tests
- Breaking changes to config.json schema require MAJOR version bump and migration guide
- Dependency updates MUST be tested in staging environment before production

### Quality Gates

- CI pipeline MUST enforce: build success, test pass rate 100%, clippy warnings=0, formatting check
- Integration tests MUST run against live RPC endpoint (testnet acceptable) in CI
- Benchmark regressions (>10% latency increase) MUST be investigated before merge
- Documentation updates MUST accompany feature changes (README, docs/, CLAUDE.md)

### Release Process

- Semantic versioning (MAJOR.MINOR.PATCH) strictly enforced
- MAJOR: Protocol version changes, config schema changes, breaking API changes
- MINOR: New RPC methods, new features (x402, websocket subscriptions)
- PATCH: Bug fixes, performance improvements, dependency updates
- GitHub releases MUST include: changelog, pre-built binaries (Linux, macOS, Windows), docker image tag

## Governance

**Constitution Authority**: This constitution supersedes all other development practices. In conflicts between this document and ad-hoc decisions, the constitution prevails.

**Amendment Process**:
1. Propose amendment via PR to `.specify/memory/constitution.md`
2. Document rationale in PR description (why needed, what problem it solves)
3. Update dependent templates (plan-template.md, spec-template.md, tasks-template.md) in same PR
4. Increment CONSTITUTION_VERSION per semantic versioning rules (MAJOR for breaking governance changes, MINOR for new principles, PATCH for clarifications)
5. Require approval from repository maintainers before merge

**Compliance Review**:
- All PRs MUST include a constitution check: which principles apply, are they followed?
- Violations MUST be justified in PR description (e.g., "Complexity added because [reason]; simpler alternative rejected because [reason]")
- Maintainers MAY reject PRs that violate principles without adequate justification
- Annual constitution review to ensure principles remain relevant to project evolution

**Runtime Guidance**:
- Use CLAUDE.md for agent-specific development guidance (build commands, architecture, patterns)
- Use constitution for non-negotiable governance and quality standards
- When in doubt, favor simplicity and maintainability over premature optimization

**Version**: 1.0.0 | **Ratified**: 2026-01-07 | **Last Amended**: 2026-01-07
