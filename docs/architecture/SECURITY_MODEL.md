# Security Model — Wiki Labs AI Copilot

## Key Derivation

```
First Launch:
  Random 256-bit master key (OS CSPRNG)
         │
         ▼
  Store in OS Keychain:
    - Windows: Credential Manager
    - macOS: Keychain
    - Linux: Secret Service (libsecret)
         │
         ▼
  Derive sub-keys via HKDF-SHA256:
    - data-enc: AES-256-GCM encryption key
    - memory-auth: HMAC-SHA256 integrity verification
    - session-N: Per-session ephemeral key
```

## Data Classification

| Classification | Encryption | Audit | Examples |
|---------------|-----------|-------|----------|
| Public | No | No | Settings, configuration |
| Internal | No | No | Chat messages, workspace config |
| Confidential | AES-256-GCM | No | API keys, SSH keys |
| Restricted | AES-256-GCM | Yes (signed) | Production credentials |

## Prompt Injection Defense

```
Input Sources → Layer 1: Normalize → Layer 2: Separate → Layer 3: Validate → AI Provider
```

- **Layer 1**: Strip control characters, normalize Unicode, remove injection patterns
- **Layer 2**: Tag observation data with `[OBSERVATION]` delimiters, separate from user chat
- **Layer 3**: Scan AI output for malicious commands, validate URLs, rate-limit outputs

## Audit Log Integrity

- Hash chain: each entry includes SHA-256 hash of previous entry
- Optional Ed25519 signature for cryptographic verification
- Stored in SQLite `audit_log` table