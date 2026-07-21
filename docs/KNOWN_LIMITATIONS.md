# Known Limitations — Wiki Labs AI Copilot v1.0.0

This document catalogs all known limitations in Wiki Labs AI Copilot v1.0.0. Each limitation links to its source file for reference.

## Security Module (`src/security/`)

### Key Derivation Not Implemented
**File:** `src/security/src/key_derivation.rs`
- `derive_data_enc_key()` — Placeholder returns `[0u8; 32]`. Should use HKDF-SHA256 with info="data-enc".
- `derive_memory_auth_key()` — Placeholder returns `[0u8; 32]`. Should use HKDF-SHA256 with info="memory-auth".
- `derive_session_key()` — Placeholder returns `[0u8; 32]`. Should use HKDF-SHA256 with info prefix "session:".
- **Impact:** Encryption keys are not cryptographically derived from master key. All derived keys are identical zero-filled arrays.

### Encryption Not Implemented
**File:** `src/security/src/encryption.rs`
- `encrypt()` — Returns "Not yet implemented". Should use AES-256-GCM.
- `decrypt()` — Returns "Not yet implemented". Should use AES-256-GCM.
- **Impact:** No data-at-rest encryption available.

### Keychain (OS Credential Store) Not Implemented
**File:** `src/security/src/keychain.rs`
- `store()` — Returns "Not yet implemented". Should use Windows Credential Manager, macOS Keychain, or Linux Secret Service.
- `retrieve()` — Returns "Not yet implemented". Same platform-specific store.
- **Impact:** Secrets cannot be persisted to OS-native secure storage.

### Credential Storage Not Implemented
**File:** `src/security/src/credentials.rs`
- `CredentialStore::store()` — Returns "Not yet implemented". Should persist encrypted credentials to SQLite backend.
- `CredentialStore::list()` — Returns "Not yet implemented". Should query SQLite and return credential list.
- **Impact:** No persistent credential management.

### Injection Defense Not Implemented
**File:** `src/security/src/injection_defense.rs`
- `normalize()` — Returns empty string. Should strip control characters and normalize Unicode (NFC).
- `separate_context()` — Returns empty string. Should wrap inputs in delimited section tags.
- `validate_output()` — Returns empty OK string. Should scan for malicious patterns.
- `detect_injection()` — Returns `false`. Should detect known injection patterns (prompt injection, code injection).
- **Impact:** No prompt injection defense, no input sanitization, no output validation.

### Audit Log Not Implemented
**File:** `src/security/src/audit.rs`
- `AuditLog::append()` — Returns "Not yet implemented". Should append with hash chain integrity (SHA-256 linked entries, optional Ed25519 signatures).
- **Impact:** No tamper-evident audit trail.

### Test Compatibility Note
**File:** `src/security/src/tests.rs`
- Tests in `injection_defense_tests` expect default implementations to return empty/fake values. These tests verify stub behavior, not production functionality.

## Observation Module (`src/observation/`)

### Tier 1 — Shell Integration Not Implemented
**File:** `src/observation/src/tier1.rs`
- `Tier1Engine::start()` — Returns "Not yet implemented". Should start shell integration hooks (bash, zsh, PowerShell).

**File:** `src/observation/src/shell.rs`
- `ShellObserver::register()` — Returns "Not yet implemented". Should register shell integration hooks per platform.

### Tier 2 — Window Polling Not Implemented
**File:** `src/observation/src/tier2.rs`
- `Tier2Engine::start()` — Returns "Not yet implemented". Should start window polling for active app detection.

### Tier 3 — Screen Capture Not Implemented
**File:** `src/observation/src/tier3.rs`
- `Tier3Engine::start()` — Returns "Not yet implemented". Should start screen capture loop.

**File:** `src/observation/src/capture.rs`
- `ScreenCapture::capture()` — Returns "Not yet implemented". Should capture screen via DXGI (Windows), CoreGraphics (macOS), X11, or Wayland.

### OCR Not Implemented
**File:** `src/observation/src/ocr.rs`
- `OCREngine::recognize()` — Returns "Not yet implemented". Should run Tesseract OCR on captured image.

### Credential Filter Not Implemented
**File:** `src/observation/src/credential_filter.rs`
- `CredentialFilter::filter()` — Returns empty string. Should detect and redact passwords, API keys, and tokens using pattern matching.

## MCP Module (`src/mcp/`)

### MCP Server Not Implemented
**File:** `src/mcp/src/server.rs`
- `McpServer::initialize()` — Returns "Not yet implemented". Should implement MCP protocol handshake.
- `McpServer::list_tools()` — Returns "Not yet implemented". Should aggregate tools from all skill modules.
- `McpServer::call_tool()` — Returns "Not yet implemented". Should route to appropriate skill module.

### MCP Transport Not Implemented
**File:** `src/mcp/src/transport.rs`
- `TransportLayer::start()` — Returns "Not yet implemented". Should start JSON-RPC server on localhost.

### Tool Catalog Not Implemented
**File:** `src/mcp/registry/src/catalog.rs`
- `ToolCatalog::register()` — Returns "Not yet implemented". Should store tool in global catalog.
- `ToolCatalog::resolve()` — Returns "Not yet implemented". Should resolve tool by namespace format "skill__tool".

### Namespace Resolver Not Implemented
**File:** `src/mcp/registry/src/resolver.rs`
- `NamespaceResolver::resolve()` — Returns "Not yet implemented". Should parse "skill__tool" qualified names.

### Skill Manager Not Implemented
**File:** `src/mcp/skill_manager/src/manager.rs`
- `SkillManager::load_module()` — Returns "Not yet implemented". Should load a skill module.
- `SkillManager::unload_module()` — Returns "Not yet implemented". Should unload a skill module.
- `SkillManager::call_tool()` — Returns "Not yet implemented". Should route tool call to appropriate module.
- `SkillManager::list_tools()` — Returns "Not yet implemented". Should aggregate all tool definitions.

### Context Bus Not Implemented
**File:** `src/mcp/skill_manager/src/context_bus.rs`
- `ContextBus::publish()` — Returns "Not yet implemented". Should publish context events.
- `ContextBus::subscribe()` — No-op. Should register handler for context events.

## Intent Engine (`src/intent/`)

### ML Model Not Implemented
**File:** `src/intent/src/model.rs`
- `IntentModel::predict()` — Returns "Not yet implemented". Should load ML model and classify intent from features.

## Testing Module (`src/testing/`)

### OpenAI Mock Not Implemented
**File:** `src/testing/src/mocks/openai_mock.rs`
- `OpenAIMock::chat()` — Returns "Not yet implemented". Should return a mock AI response.
- `OpenAIMock::embed()` — Returns "Not yet implemented". Should return a mock embedding response.
- **Note:** These are stubs for the mock provider. Should be implemented to return deterministic test data.

---

## Summary

| Module | Count | Status |
|--------|-------|--------|
| Security (key_derivation, encryption, keychain, credentials, injection_defense, audit) | 15 | Stub implementations |
| Observation (tier1-3, shell, capture, ocr, credential_filter) | 8 | Stub implementations |
| MCP (server, transport, catalog, resolver, skill_manager, context_bus) | 14 | Stub implementations |
| Intent (model) | 1 | Stub implementation |
| Testing (openai_mock) | 2 | Stub implementations |
| **Total** | **40** | **All stubs for v1.0.0** |

All 40 instances are intentional skeleton stubs for the v1.0.0 release. They return "Not yet implemented" or default/empty values. Production functionality will be added in future releases.