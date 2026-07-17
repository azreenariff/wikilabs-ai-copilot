# Security — Wiki Labs AI Copilot

## Secure Development Practices

### General

- Apply the principle of least privilege to all permissions and access controls.
- Validate all external inputs before processing.
- Never trust data from external sources (user input, observation data, AI responses).
- Use secure defaults — deny by default, allow explicitly.

### Code Security

- **Dependency security**: Run `cargo audit` on every CI run.
- **Secret scanning**: Never commit credentials, keys, or tokens.
- **No `unwrap()` or `expect()`**: All error paths must be handled explicitly.
- **No raw SQL**: Use parameterized queries through rusqlite's prepared statements.

### Data Security

- **Encryption at rest**: AES-256-GCM for Confidential and Restricted data.
- **Encryption in transit**: TLS 1.3 for all network connections.
- **Credential storage**: OS keychain only (Windows Credential Manager, macOS Keychain, Linux Secret Service).
- **Data classification**: Public, Internal, Confidential, Restricted — enforced at compile time via types.

### Privacy Principles

- **Local-first**: All data stays on the engineer's laptop.
- **Opt-in observation**: All tiers are independently toggleable.
- **Credential filtering**: Passwords, API keys, and tokens are redacted from stored observation data.
- **No telemetry**: The application does not send usage data to external services.

## Secret Handling

### What to Never Commit

- API keys, tokens, passwords
- PEM files, certificates, private keys
- `.env` files with sensitive values
- SSH keys
- Database connection strings with credentials

### What to Use Instead

- Environment variables (for local development)
- OS keychain (for production)
- Configuration files with sensitive values redacted and documented

## Vulnerability Reporting

If you discover a security vulnerability, please report it using the [Security Issue template](../.github/ISSUE_TEMPLATE/security-issue.md).

All security reports are handled with urgency. You should receive an acknowledgment within 24 hours.

## Dependency Security

### Policy

- All dependencies must be from trusted sources (crates.io, GitHub).
- Dependencies must not have known critical vulnerabilities (CVE).
- Dependency updates must be evaluated for security impact before merging.

### Tooling

- `cargo audit` — Run on every CI run and periodically as a cron job.
- `cargo deny` — Configure to block known-vulnerable dependency versions.
- `cargo clippy` — Check for unsafe code and suspicious patterns.

## Compliance

- **SOC 2**: Data encryption, audit logging, access control
- **GDPR**: Data minimization, right to erasure (workspace deletion)
- **HIPAA**: (if applicable) Access control, audit logging, encryption

See [REVISED_ARCHITECTURE.md](../REVISED_ARCHITECTURE.md) for the full security architecture.