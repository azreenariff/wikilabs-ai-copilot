---
description: "Security architecture for Wiki Labs AI Copilot — threat model (STRIDE), authentication, data protection, credential management, privacy controls, encryption, audit, compliance, and incident response."
icon: shield
---

# Wiki Labs AI Copilot — Security Architecture

## Security Overview

Wiki Labs AI Copilot is designed as a **local-first desktop application** that operates on engineer laptops. The security model assumes:

1. The laptop is the engineer's managed device (corporate policies apply)
2. The app does NOT install in customer environments
3. No customer production data leaves the laptop by default
4. The engineer is the sole operator and decision-maker
5. The AI is an advisory tool with zero autonomous execution capability

**Security Goal**: Protect the engineer's data, credentials, and privacy while providing useful AI assistance. The app operates in a trusted user environment with explicit consent controls.

---

## Threat Model (STRIDE)

### S — Spoofing

| Threat | Risk | Mitigation |
|--------|------|-----------|
| AI provider impersonation | Medium | TLS verification for all API calls; certificate pinning optional |
| MCP server impersonation | Low | MCP servers are local processes; verified by path and checksum |
| Update server impersonation | Medium | Signed update packages; HTTPS only; certificate verification |

### T — Tampering

| Threat | Risk | Mitigation |
|--------|------|-----------|
| SQLite database tampering | Medium | Database integrity checks on read; transaction-based writes |
| Knowledge document tampering | Low | File integrity verification on import; hash checksums stored |
| Configuration tampering | Low | Settings validated on load; invalid settings rejected |
| Local file tampering | Medium | Critical files verified on startup; checksums stored |

### E — Elevation of Privilege

| Threat | Risk | Mitigation |
|--------|------|-----------|
| MCP server privilege escalation | Medium | MCP servers run as the same user; no setuid/setgid; sandbox where available |
| Observation engine escalation | Low | Screen capture requires user permission; no access to protected windows |
| Credential theft via malware | Medium | Credentials stored in OS keychain; app memory cleared on focus loss |

### R — Repudiation

| Threat | Risk | Mitigation |
|--------|------|-----------|
| User denies sending a message | Low | Audit logs are immutable; signed if needed for compliance |
| User denies accepting a recommendation | Low | Recommendation acceptance logged with timestamp and workspace |
| System denies a security event | Medium | Audit logs written immediately; local backup of recent logs |

### I — Information Disclosure

| Threat | Risk | Mitigation |
|--------|------|-----------|
| Screenshot captures sensitive data | Medium | Opt-in only; resolution capped; screen-locked auto-pause |
| Clipboard captures credentials | High | Opt-in only; credential pattern detection and filtering |
| Terminal captures sensitive commands | Medium | Credential pattern detection; redaction of known secret patterns |
| Chat messages leak to disk | Medium | Local-only storage; no network transmission; encrypted columns for sensitive data |
| API keys stored in plaintext | Critical | Encrypted in SQLite; keychain for server credentials; never logged |
| Knowledge documents contain secrets | Medium | Credential pattern detection on import; alert on detection |

### D — Denial of Service

| Threat | Risk | Mitigation |
|--------|------|-----------|
| AI provider unavailable | Low | Graceful degradation; offline mode with cached knowledge |
| MCP server crash | Low | Auto-restart with backoff; individual skill isolation |
| Disk full | Medium | Storage monitoring; warnings at 80% and 90% usage |
| Memory exhaustion | Low | Observation capture bounded; streaming AI responses bounded |

---

## Authentication and Authorization Model

### Authentication

**No user authentication required.** The application runs on the engineer's laptop and trusts the OS user. However:

- **AI provider credentials**: API keys are required by the AI provider and stored securely
- **Skill credentials**: MCP servers may require credentials (OpenShift API tokens, vCenter passwords) — stored in OS credential manager
- **Session tokens**: If AI provider uses tokens (e.g., OpenAI access tokens), handled by the provider SDK

### Authorization

**Role-based access is not applicable** — the user is the sole operator. Instead, the authorization model is based on **permission controls**:

| Permission | Control | Default |
|------------|---------|---------|
| Screen observation | Settings toggle + OS permission | Disabled |
| Terminal observation | Settings toggle | Disabled |
| Clipboard observation | Settings toggle | Disabled |
| Screenshot saving | Settings toggle | Disabled |
| Skill tool execution | Skill enable/disable + tool confirmation | All skills disabled |
| Knowledge import | Settings toggle | Enabled |
| Audit log access | Read-only for user | Enabled |

**OS-Level Permissions**:
- **macOS**: Screen Recording, Full Disk Access, Accessibility (for terminal monitoring)
- **Windows**: Screen capture permissions (Windows 11+), UAC elevation not required

**Permission prompts** appear only when a permission is enabled for the first time. Once granted, the permission persists until revoked by the user.

---

## Data Protection Strategy

### Data Classification

| Level | Data Types | Storage | Encryption |
|-------|-----------|---------|-----------|
| Public | Application settings, theme, language | SQLite | None |
| Internal | Chat messages, workspace metadata, knowledge docs | SQLite | None (local-only) |
| Confidential | API keys, server credentials, clipboard content | SQLite + keychain | AES-256-GCM |
| Restricted | Sensitive chat content (user-flagged) | SQLite + encrypted columns | AES-256-GCM |

### Data Localization

**All data stays on the engineer's laptop.** No data is:
- Transmitted to external servers (except AI provider API calls and update checks)
- Stored in cloud databases
- Shared with other users or the application vendor

**Network connections (optional)**:
1. **AI Provider**: HTTPS to configured API endpoint (OpenAI, vLLM, enterprise)
2. **Update Server**: HTTPS to check for application and skill updates
3. **Knowledge Update**: HTTPS to sync new knowledge sources (if configured)

### Data Minimization

The app collects only the data necessary for its function:
- **Screen observation**: Only the active window is captured, not full screen dumps
- **Terminal observation**: Only commands and output are captured, not process memory
- **Clipboard observation**: Only text content is captured, not binary data
- **Memory**: Short-term memory automatically expires after 24 hours

---

## Credential Management

### Architecture

```
┌──────────────────────────────────────────────────┐
│              Credential Manager                   │
│                                                   │
│  ┌─────────────────┐  ┌───────────────────────┐  │
│  │  AI API Key     │  │  Server Credentials   │  │
│  │  (per provider) │  │  (per workspace/skill)│  │
│  └────────┬────────┘  └──────────┬────────────┘  │
│           │                      │                │
│           ▼                      ▼                │
│  ┌─────────────────┐  ┌───────────────────────┐  │
│  │  SQLite         │  │  OS Keychain          │  │
│  │  (encrypted)    │  │  (Credential Mgr /    │  │
│  │                 │  │   Keychain)           │  │
│  └────────┬────────┘  └──────────┬────────────┘  │
│           │                      │                │
│           ▼                      ▼                │
│  ┌─────────────────┐  ┌───────────────────────┐  │
│  │  In-Memory      │  │  In-Memory            │  │
│  │  (cleared on    │  │  (cleared on          │  │
│  │   idle timeout) │  │   use)                │  │
│  └─────────────────┘  └───────────────────────┘  │
└──────────────────────────────────────────────────┘
```

### AI API Keys

- **Storage**: Encrypted in `user_settings.ai_api_key` column (AES-256-GCM)
- **Key derivation**: User's login session key (Argon2id from OS user credentials)
- **Access**: Decrypted in-memory when making API calls
- **Never logged**: API keys are redacted from all logs
- **Validation**: Health-checked on settings save; invalid keys prompt user to re-enter

### Server Credentials

- **Storage**: OS credential manager via `keyring` crate (same approach as OpenHuman)
- **Service naming convention**: `wikilabs-copilot:<workspace_id>:<credential_name>`
- **Examples**:
  - `wikilabs-copilot:ws-acme-001:openshift_api`
  - `wikilabs-copilot:ws-acme-001:vmware_vcenter`
  - `wikilabs-copilot:ws-acme-001:mysql_db`
- **Access**: Decrypted in-memory only when an MCP server tool requires the credential
- **Never cached**: Credentials are read from keychain on-demand, not stored in app memory

### Credential Flow Example

```
Engineer types OpenShift API token in Settings
    │
    ▼
Credential Manager validates format
    │
    ▼
Stored in OS Credential Manager:
  service = "wikilabs-copilot:ws-acme-001:openshift_api"
  username = "api-token"
  password = <encrypted token>
    │
    ▼
MCP server needs to authenticate to OpenShift
    │
    ▼
Credential Manager reads from OS Keychain
    │
    ▼
Token injected into MCP server environment (not logged)
    │
    ▼
MCP server uses token for API calls
    │
    ▼
Token cleared from app memory after use
```

---

## Privacy Controls

### Observation Privacy

| Control | Description |
|---------|-------------|
| **Opt-in by default** | All observation sources are disabled by default |
| **Per-source toggle** | Screen, terminal, and clipboard have independent on/off controls |
| **Screen-locked auto-pause** | If the OS screen locks, all observation stops immediately |
| **Screenshot resolution cap** | Maximum 1920x1080; never captures at native resolution |
| **No persistent screenshots** | Screenshots processed in-memory only; never saved unless user explicitly saves a conversation |
| **Privacy indicator** | A small icon in the system tray shows when observation is active |
| **Audit logging** | All observation events (enable/disable, captures) are logged |

### Credential Detection

When observation is enabled, the app actively detects and filters potential credentials:

| Detection | Action |
|-----------|--------|
| Clipboard contains API key pattern | Alert user, optionally skip capture |
| Clipboard contains password pattern | Alert user, optionally skip capture |
| Terminal command contains `--password=` or `-p <secret>` | Redact from stored observation data |
| Knowledge import contains known secret patterns | Alert user, mark document for review |

### Privacy Settings

```
Settings → Privacy
├── Screen Observation
│   ├── Status: [●] Enabled  [○] Disabled
│   ├── Capture Interval: [2] seconds
│   ├── Resolution: [1920x1080] (max)
│   └── Auto-pause when screen locked: [●] Enabled
├── Terminal Observation
│   ├── Status: [●] Enabled  [○] Disabled
│   └── Credential filtering: [●] Enabled
├── Clipboard Observation
│   ├── Status: [●] Enabled  [○] Disabled
│   └── Credential filtering: [●] Enabled
└── Data Retention
    ├── Short-term memory: [24] hours
    ├── Audit log retention: [90] days
    └── Knowledge docs: [Until deleted]
```

---

## Secure Communication

### TLS Enforcement

All network communication uses TLS 1.2+ with certificate validation:

| Connection | Protocol | TLS Version | Certificate Verification |
|-----------|----------|-------------|---------------------------|
| AI Provider | HTTPS | 1.2+ | Required |
| Update Server | HTTPS | 1.2+ | Required |
| MCP Server (HTTP transport) | HTTPS | 1.2+ | Optional (local) |
| Local RPC | HTTP (loopback) | N/A | N/A (localhost only) |

### Implementation
- **rustls** for TLS (same as OpenHuman, no OpenSSL dependency)
- **reqwest** for HTTP client with rustls backend
- **tokio-tungstenite** for WebSocket connections with TLS
- Certificate pinning available for enterprise deployments

### Loopback Communication
Core ↔ Frontend communication occurs over `http://127.0.0.1:<port>/rpc` — this is:
- Not accessible from outside the machine (loopback only)
- Authenticated via per-launch hex bearer token (like OpenHuman)
- CORS-restricted by Tauri's WebView restrictions

---

## Encryption Standards and Algorithms

### Symmetric Encryption

| Algorithm | Use Case | Parameters |
|-----------|---------|-----------|
| AES-256-GCM | Column-level encryption (API keys, credentials) | 256-bit key, random IV, 128-bit auth tag |
| AES-256-GCM | File-level encryption (saved screenshots) | 256-bit key, random IV, 128-bit auth tag |

### Key Derivation

| Algorithm | Use Case | Parameters |
|-----------|---------|-----------|
| Argon2id | Master key from user login | Memory: 64 MB, Iterations: 3, Parallelism: 4 |
| HKDF | Derive sub-keys from master key | SHA-256, info field identifies purpose |

### Hash Functions

| Algorithm | Use Case |
|-----------|---------|
| SHA-256 | File integrity verification, UUID generation fallback |
| Argon2id | Key derivation, credential verification |

### Random Number Generation

| Source | Use Case |
|--------|---------|
| OS CSPRNG (`getrandom` crate) | IV generation, bearer tokens, skill IDs |

### In-Memory Security

| Practice | Detail |
|----------|--------|
| Secure memory clearing | Sensitive data zeroed from memory when no longer needed |
| No swap leakage | Critical buffers marked as non-pageable where OS supports it |
| IDE debugger sensitive data | Not available during release builds |

---

## Audit and Compliance

### Audit Capabilities

The application maintains comprehensive audit logs:

| Event | Logged | Immutable | Exportable |
|-------|--------|-----------|------------|
| User login to OS | No (OS responsibility) | — | — |
| Settings changes | Yes | Yes | Yes |
| AI conversations | Yes | Yes | Yes |
| AI responses | Yes | Yes | Yes |
| Observation events | Yes | Yes | Yes |
| Skill enable/disable | Yes | Yes | Yes |
| Credential access | Yes | Yes | Yes |
| Knowledge imports | Yes | Yes | Yes |
| Workspace operations | Yes | Yes | Yes |
| System errors | Yes | Yes | Yes |

### Compliance Considerations

| Framework | Applicability | Implementation |
|-----------|--------------|----------------|
| **SOC 2** | If deployed by enterprise | Audit logs, access controls, encryption at rest |
| **GDPR** | If handling EU data | Local-first data, user can delete all data, no data sharing |
| **HIPAA** | If used in healthcare | BAA required; local storage meets "encrypt at rest" |
| **FedRAMP** | If used by US government | Not certified; requires vendor assessment |
| **ISO 27001** | If deployed by enterprise | Security policies, audit logs, access controls |
| **NIST 800-53** | If used by US agencies | IA-2 (identification), SC-8 (transmission), SC-28 (protection) |

### Audit Log Export

Engineers and administrators can export audit logs:
- **Format**: JSON or CSV
- **Scope**: Full log or filtered by date range, component, event type
- **Purpose**: Compliance reporting, incident investigation, security review
- **Protection**: Exported logs are not re-encrypted (the original SQLite log is the source of truth)

---

## Security Testing Requirements

### Pre-Release Testing

| Test | Frequency | Tool |
|------|----------|------|
| Dependency vulnerability scan | Every PR | `cargo audit`, npm audit |
| Secret detection in code | Every PR | `gitleaks`, `trufflehog` |
| SAST (Static Analysis) | Every PR | `cargo clippy`, `rustc` warnings |
| DAST (Dynamic Analysis) | Every release | Manual testing, OWASP ZAP (local) |
| Fuzz testing | Every release | `cargo fuzz` for parsing code |
| Penetration testing | Annually or major releases | Third-party security firm |

### Ongoing Testing

| Test | Frequency | Scope |
|------|----------|-------|
| Automated security tests | Every CI run | Unit + integration tests for security components |
| Dependency updates review | Weekly | Cargo.toml and package.json dependency updates |
| Credential handling review | Quarterly | Code review of credential access patterns |
| Privacy impact assessment | Annually | Review observation and data collection practices |
| Security incident drill | Annually | Tabletop exercise for security breach scenario |

### Security Test Checklist

- [ ] API keys never appear in logs, error messages, or console output
- [ ] Credentials read from keychain are not stored in plain memory after use
- [ ] Screenshot resolution is capped at configured maximum
- [ ] Screen-locked state pauses all observation
- [ ] Database integrity is verified on startup
- [ ] TLS certificates are validated for all network connections
- [ ] MCP server checksums are verified on update
- [ ] Update packages are signed and verified
- [ ] No sensitive data written to crash dumps or core files
- [ ] Audit logs are written immediately and cannot be deleted (except by user)

---

## Incident Response Considerations

### Incident Classification

| Severity | Criteria | Response Time |
|----------|----------|---------------|
| **P1 - Critical** | API key leaked, credential exposure, observation capturing credentials unintentionally | Immediate |
| **P2 - High** | MCP server compromise, update tampering detected, database corruption | 1 hour |
| **P3 - Medium** | Privacy setting bypass, audit log gap, sensitive data in knowledge import | 1 day |
| **P4 - Low** | Minor information disclosure, cosmetic security issue | 1 week |

### Response Procedures

#### P1: Credential / Privacy Breach
1. **Contain**: User is prompted to revoke compromised credentials immediately
2. **Assess**: Review audit logs for scope of exposure
3. **Notify**: If third-party credentials are compromised, guide user to change them
4. **Remediate**: Update affected settings, revoke compromised API keys
5. **Document**: Record incident details in audit log export
6. **Prevent**: Patch the vulnerability; add detection for similar patterns

#### P2: MCP Server Compromise
1. **Contain**: Disable the affected skill immediately
2. **Assess**: Check audit logs for tool calls made with compromised server
3. **Remediate**: Reinstall the MCP server from verified source
4. **Document**: Record incident details
5. **Prevent**: Improve server verification (checksums, code signing)

#### P3: Audit Log Gap
1. **Assess**: Determine the scope of the gap
2. **Remediate**: Fix the bug that caused the gap
3. **Document**: Note the gap period in incident records
4. **Prevent**: Add monitoring for audit log continuity

### Incident Communication

Incident details are:
- **Private**: Only visible to the engineer (local-only)
- **Audit-logged**: All incident response actions are logged
- **Exportable**: Engineers can export incidents for compliance reporting
- **NOT transmitted**: No incident data is sent to external parties unless the engineer chooses to share audit log exports

---

## Security Architecture Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SECURITY ARCHITECTURE                            │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    TRUST BOUNDARIES                          │   │
│  │                                                             │   │
│  │  ┌───────────────────────────────────────────────────────┐   │   │
│  │  │  Trusted: Engineer's Laptop (OS user session)         │   │   │
│  │  │                                                       │   │   │
│  │  │  ┌─────────────────────────────────────────────────┐   │   │   │
│  │  │  │  Trusted: Wiki Labs AI Copilot (Rust Core)      │   │   │   │
│  │  │  │                                                 │   │   │   │
│  │  │  │  ┌───────────┐ ┌───────────┐ ┌───────────────┐ │   │   │   │
│  │  │  │  │ Credential │ │  Privacy  │ │   Encryption  │ │   │   │   │
│  │  │  │  │  Manager  │ │   Controls│ │     Layer     │ │   │   │   │
│  │  │  │  └───────────┘ └───────────┘ └───────────────┘ │   │   │   │
│  │  │  │  ┌───────────┐ ┌───────────┐ ┌───────────────┐ │   │   │   │
│  │  │  │  │  Observation│ │  Audit   │ │  Secure Comms │ │   │   │   │
│  │  │  │  │   Engine   │ │  Logger  │ │    (rustls)   │ │   │   │   │
│  │  │  │  └───────────┘ └───────────┘ └───────────────┘ │   │   │   │
│  │  │  └─────────────────────────────────────────────────┘   │   │   │
│  │  └───────────────────────────────────────────────────────┘   │   │
│  │                                                             │   │
│  │  ┌───────────────────────────────────────────────────────┐   │   │
│  │  │  Untrusted: AI Provider, Update Server, MCP Servers   │   │   │
│  │  │                                                       │   │   │
│  │  │  ┌───────────┐ ┌───────────┐ ┌─────────────────────┐ │   │   │
│  │  │  │   TLS     │ │ Checksum  │ │  Process Isolation  │ │   │   │
│  │  │  │  Verify   │ │  Verify   │ │  (per-skill proc)  │ │   │   │
│  │  │  └───────────┘ └───────────┘ └─────────────────────┘ │   │   │
│  │  └───────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              THREAT MITIGATION COVERAGE                      │   │
│  │                                                             │   │
│  │  STRIDE    │  Covered By                                   │   │
│  │  ──────────┼───────────────────────────────────────────────│   │
│  │  Spoofing  │  TLS, signed updates, MCP verification        │   │
│  │  Tampering │  DB integrity, file checksums, validation     │   │
│  │  Elevation │  No setuid, sandbox, minimal permissions      │   │
│  │  Repudiation│ Immutable audit logs                          │   │
│  │  Info Disc │  Opt-in obs, credential filtering, encryption │   │
│  │  DoS       │  Auto-restart, storage monitoring, bounds     │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## References

- [ARCHITECTURE.md](ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](DATA_MODEL.md) — Data storage and encryption model
- [COMPONENT_DESIGN.md](COMPONENT_DESIGN.md) — Component error handling and security
- [MCP_ARCHITECTURE.md](MCP_ARCHITECTURE.md) — MCP server security
- [TECHNOLOGY_SELECTION.md](TECHNOLOGY_SELECTION.md) — Security-relevant technology choices (rustls, keyring)