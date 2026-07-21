# Security Guide — Wiki Labs AI Copilot v1.0.0

> Security model, threat model, credentials, encryption, and compliance.

## Table of Contents

1. [Security Overview](#security-overview)
2. [Security Model](#security-model)
3. [Threat Model](#threat-model)
4. [Encryption & Key Management](#encryption--key-management)
5. [Credential Management](#credential-management)
6. [Secure Communication](#secure-communication)
7. [Log Security](#log-security)
8. [Privacy Controls](#privacy-controls)
9. [Audit & Compliance](#audit--compliance)
10. [Configuration Security](#configuration-security)
11. [Development Security](#development-security)
12. [Security Incidents](#security-incidents)
13. [Security Checklist](#security-checklist)

## Security Overview

Wiki Labs AI Copilot employs a defense-in-depth security approach. The application is designed as a local-first desktop application with minimal external exposure, encrypting all sensitive data at rest and in transit.

### Security Principles

1. **Privacy by default:** All observation features disabled by default
2. **Local-first data:** All user data stored locally; nothing uploaded without explicit user action
3. **Least privilege:** Components operate with minimal required permissions
4. **Defense in depth:** Multiple layers of security controls
5. **Transparency:** Clear user consent requirements for data collection
6. **Secure defaults:** Deny-by-default configuration

### Security Posture

| Aspect | Status | Details |
|--------|--------|---------|
| Data at rest | **Protected** | AES-256-GCM / ChaCha20-Poly1305 encryption |
| Data in transit | **Protected** | TLS 1.2+ to all external endpoints |
| Credentials | **Protected** | Windows Credential Manager / encrypted file |
| Logs | **Protected** | Automatic redaction of sensitive fields |
| Observation | **User-controlled** | Per-feature toggles with privacy mode |
| Authentication | **API key-based** | Provider API keys managed securely |
| Audit trail | **Supported** | SQLite audit log with hash chain |

## Security Model

### Architecture Layers

```
┌─────────────────────────────────────────────────┐
│  Layer 1: Application Boundary                   │
│  - Tauri window permissions                     │
│  - WebView2 security sandbox                    │
│  - Tauri capabilities (file system, etc.)       │
├─────────────────────────────────────────────────┤
│  Layer 2: Data Security                          │
│  - Encryption at rest (AES-256-GCM)             │
│  - Encrypted credential store                    │
│  - Database access controls                      │
├─────────────────────────────────────────────────┤
│  Layer 3: Network Security                       │
│  - TLS 1.2+ for all outbound connections         │
│  - Certificate validation                       │
│  - No inbound network exposure                  │
├─────────────────────────────────────────────────┤
│  Layer 4: Privacy Controls                       │
│  - Per-feature observation toggles              │
│  - One-click privacy mode                        │
│  - Explicit user consent                         │
├─────────────────────────────────────────────────┤
│  Layer 5: Application Security                   │
│  - Input validation                              │
│  - Secret redaction in logs                      │
│  - Error handling without information leakage   │
└─────────────────────────────────────────────────┘
```

### Trust Boundaries

| Boundary | Protection | Description |
|----------|-----------|-------------|
| User ↔ Application | Privacy toggles | User controls observation features |
| Application ↔ Provider | TLS 1.2+ | Encrypted network transport |
| Application ↔ Storage | AES-256-GCM | Encrypted credential file |
| Application ↔ SQLite | File permissions | Local filesystem access control |
| Application ↔ OS | Credential Manager | OS-level credential protection |

## Threat Model

### Assets

| Asset | Sensitivity | Protection |
|-------|------------|------------|
| API Keys | **High** | Encrypted storage (AES-256-GCM), Credential Manager |
| Chat History | **Medium** | Local storage, workspace-scoped |
| Knowledge Documents | **Medium** | Local storage, encrypted |
| User Settings | **Medium** | Local storage, profiled |
| Observation Data | **High** | Local only, user-controlled, not stored |
| AI Responses | **Medium** | Stored in chat history |

### Threats and Mitigations

#### T1: Credential Theft

| Threat | Risk | Mitigation |
|--------|------|------------|
| API key extraction from disk | High | AES-256-GCM encryption, Credential Manager |
| API key extraction from memory | Medium | Secure string handling where possible |
| Shared computer credential theft | Medium | Auto-lock after inactivity, PIN protection |
| Credential dump via process | Medium | No plaintext credentials in logs |

#### T2: Data Exfiltration

| Threat | Risk | Mitigation |
|--------|------|------------|
| Unauthorized network upload | Medium | No data leaves device without API request |
| Screen capture by malware | Low | Screen observation disabled by default |
| Clipboard data exposure | Low | Clipboard observation disabled by default |
| Log files contain secrets | Medium | Automatic redaction of sensitive fields |

#### T3: Application Compromise

| Threat | Risk | Mitigation |
|--------|------|------------|
| WebView2 XSS | Low | Tauri capabilities restrict WebView2 access |
| Supply chain attack | Medium | GitHub Actions CI verification |
| Dependency vulnerabilities | Medium | Regular `cargo audit` checks |
| Malicious skill pack | Medium | Skill validation before loading |

#### T4: Replay / Injection

| Threat | Risk | Mitigation |
|--------|------|------------|
| Prompt injection in chat | Medium | Input validation on user/AI messages |
| Replay attack on AI | Low | TLS + API key authentication |
| Skill injection | Low | Skill validation and version checking |

#### T5: Denial of Service

| Threat | Risk | Mitigation |
|--------|------|------------|
| Resource exhaustion | Low | Token budget management |
| Provider unavailability | Medium | Graceful degradation |
| Disk space exhaustion | Low | Log rotation |

### Threat Model Summary

```
Attack Surface: Minimal (desktop app, single outbound HTTPS)
Data Exposure: None by default (local-first architecture)
Credential Risk: Low (encryption + Credential Manager)
Network Risk: Low (outbound HTTPS only, no inbound)
Observation Risk: User-controlled (disabled by default)
```

## Encryption & Key Management

### Encryption Algorithms

| Algorithm | Use Case | Key Size | Mode |
|-----------|----------|----------|------|
| AES-256-GCM | Credential storage, data at rest | 256 bits | GCM (authenticated) |
| ChaCha20-Poly1305 | Alternative credential storage | 256 bits | Poly1305 (authenticated) |

### Key Derivation

```
Key Derivation Function:
┌────────────────────────────────────────────────┐
│                                                │
│  Inputs:                                       │
│  ├── System Fingerprint (SHA-256)               │
│  │   ├── CPU identifier                        │
│  │   ├── Disk identifier                       │
│  │   └── OS information                        │
│  └── Optional PIN (SHA-256)                    │
│                                                │
│  Process:                                      │
│  ├── SHA-256(SystemFingerprint) → key_1        │
│  ├── SHA-256(PIN) → key_2 (if PIN set)         │
│  └── SHA-256(key_1 + key_2) → master_key       │
│                                                │
│  Output: 256-bit encryption key                 │
│                                                │
└────────────────────────────────────────────────┘
```

### Encryption Operations

```
Encryption:
┌────────────────────────────────────────────────┐
│  Input: plaintext + encryption_key              │
│  ├── Generate random IV                          │
│  ├── AES-256-GCM(plaintext, key, IV)             │
│  └── Output: ciphertext + tag + IV               │
└────────────────────────────────────────────────┘

Decryption:
┌────────────────────────────────────────────────┐
│  Input: ciphertext + tag + IV + encryption_key  │
│  ├── AES-256-GCM-decrypt(ciphertext, key, IV)   │
│  ├── Verify tag (authentication)                 │
│  └── Output: plaintext or error                 │
└────────────────────────────────────────────────┘
```

### Storage Locations

| Item | Storage | Encrypted |
|------|---------|-----------|
| API Key | Credential Manager (preferred) | Yes (DPAPI) |
| API Key | credentials.enc file (fallback) | Yes (AES-256-GCM) |
| PIN | Memory only | N/A (not stored) |
| System Fingerprint | Memory only | N/A (computed at runtime) |

## Credential Management

### Windows Credential Manager Integration

The application uses Windows Credential Manager (Data Protection API) for credential storage on Windows:

```
┌─────────────────────────────────────────────────┐
│          Credential Management Flow               │
│                                                  │
│  1. User enters API key in Settings               │
│         │                                         │
│         ▼                                         │
│  2. System Fingerprint computed                   │
│         │                                         │
│         ▼                                         │
│  3. Credential Manager queried                    │
│         │                                         │
│         ├── Found → Decrypt & return key          │
│         └── Not found → Create new entry          │
│                                                    │
│  4. On next launch:                                │
│     ├── Credential Manager → Decrypt & return     │
│     └── If unavailable → Fallback to file         │
│                                                  │
└─────────────────────────────────────────────────┘
```

### Fallback to Local Encryption

If Windows Credential Manager is unavailable (e.g., headless environment, non-Windows platform):

1. Derive encryption key from system fingerprint + optional PIN
2. Encrypt API key using AES-256-GCM
3. Store encrypted blob at `credentials.enc` in application data directory
4. On access: decrypt using the same key derivation

### Credential Operations

| Operation | Description |
|-----------|-------------|
| `store_credentials()` | Encrypt and store API keys |
| `get_credentials()` | Decrypt and return API keys |
| `delete_credentials()` | Remove all stored credentials |
| `rotate_credentials()` | Delete and require re-entry |

## Secure Communication

### Network Security

All outbound network connections use TLS 1.2 or higher:

| Endpoint | Protocol | TLS Version |
|----------|----------|-------------|
| AI Provider (OpenAI) | HTTPS | 1.2+ |
| AI Provider (vLLM) | HTTPS / HTTP | 1.2+ / local |
| AI Provider (Ollama) | HTTPS / HTTP | 1.2+ / local |
| Update check (GitHub) | HTTPS | 1.2+ |

### Certificate Validation

- System trust store validation for HTTPS connections
- No certificate pinning (allows trust store updates)
- Failed certificate validation aborts the connection
- Certificate validation utility available in security module

### No Inbound Connections

The application does not:
- Listen on any network ports
- Accept inbound connections
- Expose any network services
- Require firewall exceptions

## Log Security

### Automatic Redaction

The logging system automatically redacts sensitive fields:

```rust
// Pattern matching for redaction
const REDACTED_FIELDS: &[&str] = &[
    "password",
    "secret",
    "token",
    "api_key",
    "authorization",
];
```

**Redaction examples:**

| Original | Redacted |
|----------|----------|
| `password: mySecret123` | `password: PASSWORD_REDACTED` |
| `api_key: sk-abc123...` | `api_key: API_KEY_REDACTED` |
| `token: eyJhbGci...` | `token: TOKEN_REDACTED` |
| `authorization: Bearer xyz` | `authorization: AUTH_REDACTED` |

### Log File Protection

- Logs stored in `%APPDATA%\com.wikilabs.copilot\logs\`
- Logs only written when `file_logging` is enabled
- Sensitive fields are redacted before writing to disk
- Log rotation prevents disk space exhaustion

### Diagnostic Package Safety

The diagnostic package generator:
- Redacts all API keys before bundling
- Redacts credential settings
- Includes only metadata about log files (not contents)
- Designed for safe sharing with support teams

## Privacy Controls

### Per-Feature Privacy Toggles

| Feature | Default | Setting Path |
|---------|---------|-------------|
| Screen observation | **Disabled** | Settings → Privacy → Screen observation |
| OCR | **Enabled** | Settings → Privacy → OCR |
| Clipboard observation | **Disabled** | Settings → Privacy → Clipboard |
| Diagnostics (crash reports) | **Enabled** | Settings → Privacy → Diagnostics |
| Telemetry (analytics) | **Disabled** | Settings → Privacy → Telemetry |
| Consent tracking | **Enabled** | Settings → Privacy → Consent |

### Privacy Mode

One-click privacy mode that disables all observation and data collection:

```
Privacy Mode Toggle:
┌─────────────────────────────────────────────────┐
│                                                  │
│  When ENABLED:                                  │
│  ├── Screen observation → Disabled               │
│  ├── OCR → Disabled                              │
│  ├── Clipboard observation → Disabled            │
│  ├── Diagnostics → Disabled                      │
│  └── Telemetry → Disabled                        │
│                                                  │
│  When DISABLED:                                 │
│  └── Features revert to individual toggle        │
│     values (not the default values)              │
│                                                  │
└─────────────────────────────────────────────────┘
```

### Data Collection Transparency

- All data collection is logged in the audit trail
- No data leaves the device without user action
- AI requests include only current conversation context
- No automatic telemetry collection (opt-in only)

## Audit & Compliance

### Audit Log

The application maintains an audit log in SQLite:

```sql
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,        -- UUID
    timestamp TEXT,             -- ISO 8601
    action TEXT,                -- Action description
    actor TEXT,                 -- User/system identifier
    hash TEXT,                  -- SHA-256 of previous entry
    signature TEXT              -- Ed25519 signature (optional)
);
```

**Hash chain integrity:**
- Each entry includes SHA-256 hash of the previous entry
- Tampering breaks the chain (detectable)
- Optional Ed25519 signatures for signed audits

### Audit Events

| Event | Description | Logged |
|-------|-------------|--------|
| Settings changed | Configuration modification | Yes |
| Credentials accessed | API key read/write | Yes |
| Workspace created | New workspace creation | Yes |
| Knowledge imported | Document import | Yes |
| Skill loaded | Skill pack activation | Yes |
| Privacy mode changed | Privacy toggle change | Yes |

### Compliance Considerations

| Standard | Status | Notes |
|----------|--------|-------|
| GDPR (data minimization) | **Met** | Local-first, no cloud storage |
| SOC 2 (data protection) | **Partial** | Encryption at rest, TLS in transit |
| NIST (access control) | **Partial** | API key authentication |
| HIPAA (PHI protection) | **Not designed** | Not a HIPAA-covered application |
| PCI DSS (card data) | **N/A** | No card data handling |

## Configuration Security

### Settings Security

| Setting | Sensitive | Encrypted |
|---------|-----------|-----------|
| Provider name | No | No |
| Provider endpoint | No | No |
| API key | **Yes** | **Yes** (AES-256-GCM) |
| Model name | No | No |
| Max tokens | No | No |
| Log level | No | No |
| Encryption algorithm | No | No |
| Auto-lock | No | No |

### Secure Settings Management

1. Settings loaded into memory on startup
2. Sensitive fields (API keys) encrypted in storage
3. API keys never written to logs
4. Settings file permissions controlled by OS
5. Multiple profiles allow per-context configuration

### Configuration Profiles

Profiles provide isolation between different usage contexts:

| Profile | Use Case | Separation |
|---------|----------|------------|
| `work` | Work environment | Separate provider, observation settings |
| `home` | Personal use | Separate provider, preferences |
| `default` | Fallback | Default settings |

## Development Security

### Secure Coding Practices

| Practice | Implementation |
|----------|----------------|
| Input validation | All external inputs validated before processing |
| No unwrap/expect | All error paths handled explicitly |
| Parameterized queries | SQLite uses parameterized queries |
| Secure defaults | Deny-by-default configuration |
| Secret management | Redaction patterns in logging |

### Dependency Security

- Regular `cargo audit` checks for known vulnerabilities
- Minimal dependency surface (focused workspace)
- No network-dependent crates in core engine
- All dependencies pinned to specific versions

### CI/CD Security

- GitHub Actions with environment secrets
- No secrets in code or configuration files
- Automated security checks in CI pipeline
- Release signing (future phase)

## Security Incidents

### Incident Response

If a security incident is suspected:

1. **Identify:** Determine the nature and scope of the incident
2. **Contain:** Disable affected features (privacy mode, credential rotation)
3. **Assess:** Check logs and audit trail for evidence
4. **Remediate:** Apply the appropriate fix
5. **Report:** Notify affected users and support team

### Reporting Security Issues

| Method | Details |
|--------|---------|
| GitHub Issues | Create a **private** security advisory |
| Email | security@wikilabs.com |
| Response Time | Acknowledged within 48 hours |

### Security Hardening Checklist

| Check | Action |
|-------|--------|
| Update API keys | Rotate keys if compromised |
| Verify encryption | Confirm AES-256-GCM is active |
| Check privacy settings | Review per-feature toggles |
| Review audit log | Check for unauthorized changes |
| Verify log redaction | Ensure sensitive fields are redacted |
| Update application | Install latest security patches |
| Check permissions | Verify file and directory permissions |

---

*For configuration details, see [Administrator Guide](admin-guide/ADMINISTRATOR_GUIDE.md).*
*For security architecture, see [Architecture Guide](ARCHITECTURE_GUIDE.md).*
*For troubleshooting security issues, see [Troubleshooting Guide](TROUBLESHOOTING.md).*