# Known Limitations — Wiki Labs AI Copilot v1.0.0

> Current limitations and workarounds.

## Table of Contents

1. [Platform Limitations](#platform-limitations)
2. [AI & Intelligence Limitations](#ai--intelligence-limitations)
3. [Feature Limitations](#feature-limitations)
4. [Performance Limitations](#performance-limitations)
5. [Security Limitations](#security-limitations)
6. [Integration Limitations](#integration-limitations)
7. [Data Limitations](#data-limitations)
8. [Operational Limitations](#operational-limitations)

## Platform Limitations

### Windows Only

**Limitation:** Wiki Labs AI Copilot v1.0.0 supports only Windows 10 and Windows 11 (64-bit).

**Impact:** Cannot use on macOS or Linux with this version.

**Workaround:** None. Windows-only support is by design for this release. Cross-platform support (macOS, Linux) is planned for future releases.

**Planned for:** Future releases (cross-platform expansion)

### WebView2 Dependency

**Limitation:** The application requires Microsoft Edge WebView2 Runtime.

**Impact:** Cannot install on systems where WebView2 cannot be installed or is blocked by group policy.

**Workaround:** Install WebView2 Runtime separately from https://developer.microsoft.com/en-us/microsoft-edge/webview2/

**Planned for:** Not planned — WebView2 is the supported rendering engine for Tauri v2 on Windows.

### .NET Desktop Runtime

**Limitation:** Requires .NET Desktop Runtime 8.0.

**Impact:** Cannot install on systems without .NET Runtime 8.0 or with an incompatible version.

**Workaround:** Install .NET Desktop Runtime 8.0 from https://dotnet.microsoft.com/download/dotnet/8.0

**Planned for:** The installer bundles the .NET Runtime, so this is not typically an issue.

## AI & Intelligence Limitations

### OpenAI-Compatible Providers Only

**Limitation:** The AI provider abstraction supports only OpenAI-compatible API endpoints (OpenAI, vLLM, Ollama, etc.).

**Impact:** Cannot use providers that do not implement the OpenAI-compatible API format (e.g., Azure OpenAI with custom endpoints, Google Gemini, Anthropic).

**Workaround:** Use a proxy or adapter that converts the provider's API to OpenAI-compatible format.

**Planned for:** Future provider support may be added based on demand.

### No Native Azure OpenAI Support

**Limitation:** Azure OpenAI uses a slightly different API format than standard OpenAI.

**Impact:** Azure OpenAI requires manual API key configuration with custom header setup.

**Workaround:** Configure the Azure OpenAI endpoint URL and set the `api-key` header via the endpoint configuration if supported.

**Planned for:** Native Azure OpenAI support in a future release.

### No Image/Vision Models

**Limitation:** The chat interface sends text-only prompts to the AI provider.

**Impact:** Cannot use vision-capable models to analyze images or screenshots.

**Workaround:** Describe the image content in the chat message manually.

**Planned for:** Future release (vision model integration)

### No Multi-Model Chats

**Limitation:** Each workspace uses a single AI provider and model.

**Impact:** Cannot switch models mid-conversation or use different models for different tasks.

**Workaround:** Create separate workspaces with different model configurations.

**Planned for:** Not currently planned; single-model-per-workspace is the design.

### Streaming Response Limitations

**Limitation:** Streaming support depends on the AI provider.

**Impact:** Some providers may not support streaming, causing delays in response appearance.

**Workaround:** Use a provider that supports streaming (OpenAI, vLLM with streaming enabled).

**Planned for:** Streaming support is being improved in future releases.

### Context Window Limits

**Limitation:** The token budget manager enforces context window limits.

**Impact:** Very long conversations may result in truncation of earlier messages.

**Workaround:** Start new conversations periodically, or reduce context window size in settings.

**Planned for:** Improved context management in future releases.

## Feature Limitations

### No Real-Time Collaboration

**Limitation:** Each user runs their own instance with local data.

**Impact:** Cannot collaboratively edit workspaces, chat, or knowledge with other users.

**Workaround:** Share knowledge documents or conversation exports via file sharing.

**Planned for:** Multi-user collaboration is planned for a future release.

### No Cloud Sync

**Limitation:** Data is stored locally and does not sync across devices.

**Impact:** Cannot access the same workspace from multiple machines.

**Workaround:** Manually copy the application data directory between machines.

**Planned for:** Cloud sync is planned for a future release.

### No Email Integration

**Limitation:** Cannot send or receive emails from within the application.

**Impact:** Cannot use the copilot for email-based workflows.

**Workaround:** Copy/paste email content into the chat for analysis.

**Planned for:** Email integration is not planned for v1.x.

### No Calendar Integration

**Limitation:** Cannot access or modify calendar events.

**Impact:** Cannot use the copilot for scheduling or calendar management.

**Workaround:** None at this time.

**Planned for:** Not planned for v1.x.

### Limited Export Formats

**Limitation:** Conversations can only be exported as JSON files.

**Impact:** Cannot export to PDF, markdown, or other formats directly.

**Workaround:** Write a script to convert JSON to other formats.

**Planned for:** Additional export formats may be added in future releases.

### No Built-in PDF Reader

**Limitation:** Cannot view PDF files within the application.

**Impact:** Cannot import PDF documents directly.

**Workaround:** Extract text from PDFs using an external tool (e.g., `pdftotext`) and import the text as a `.wkl` archive.

**Planned for:** PDF import is planned for a future release.

## Performance Limitations

### Database Size Scaling

**Limitation:** SQLite performance is excellent for moderate data sizes but may degrade with very large knowledge bases.

**Impact:** Knowledge bases with more than 50,000 chunks may experience slower search times.

**Workaround:** Split large knowledge bases across multiple workspaces. Run `VACUUM` periodically.

**Planned for:** Database optimization and sharding for large-scale deployments.

### Memory Usage with Observation

**Limitation:** The observation engine consumes additional memory.

**Impact:** With all observation features enabled, memory usage may exceed 200 MB.

**Workaround:** Disable unused observation features in Settings → Privacy.

**Planned for:** Memory optimization in future releases.

### Embedding Generation Speed

**Limitation:** Embedding generation uses ONNX Runtime on CPU (no GPU acceleration).

**Impact:** Importing large knowledge bases may take several minutes.

**Workaround:** Import documents in smaller batches.

**Planned for:** GPU acceleration for embeddings is planned for future releases.

### Startup Time

**Limitation:** Application startup time includes database initialization and skill pack loading.

**Impact:** Startup may take 3-5 seconds with many skill packs installed.

**Workaround:** Reduce the number of installed skill packs.

**Planned for:** Startup optimization is an ongoing improvement area.

## Security Limitations

### Credential Manager Only on Windows

**Limitation:** Windows Credential Manager integration works only on Windows.

**Impact:** On non-Windows platforms, all credentials use the local encrypted file fallback.

**Workaround:** Ensure PIN protection is enabled when using local encryption.

**Planned for:** Platform-specific credential storage for each platform.

### No Multi-Factor Authentication

**Limitation:** Authentication is API key-based only (for the AI provider).

**Impact:** No built-in user authentication for the application itself.

**Workaround:** Use Windows-level security (password, PIN, biometric) to protect the desktop session.

**Planned for:** Built-in application authentication is planned for future releases.

### No Audit Log Export

**Limitation:** The audit log is stored in SQLite but has no export mechanism.

**Impact:** Cannot export audit logs for compliance review.

**Workaround:** Query the audit log table directly using SQLite tools.

**Planned for:** Audit log export and reporting is planned for future releases.

## Integration Limitations

### No API / REST API

**Limitation:** The application does not expose a REST API for external integration.

**Impact:** Cannot programmatically interact with the application from external tools.

**Workaround:** Use the application's command-line interface (if available) or direct database queries.

**Planned for:** REST API for external integration is planned for future releases.

### No Webhook Support

**Limitation:** The application does not send webhooks for events.

**Impact:** Cannot trigger external workflows based on application events.

**Workaround:** Monitor the application data directory for file changes.

**Planned for:** Webhook support is planned for future releases.

### No Slack/Teams Integration

**Limitation:** Cannot send notifications or receive messages from Slack or Microsoft Teams.

**Impact:** Cannot use the copilot for communication-based workflows.

**Workaround:** None at this time.

**Planned for:** Communication platform integration is planned for future releases.

### Limited Technology Coverage

**Limitation:** Only a subset of enterprise technologies has skill packs available.

**Impact:** Technologies without skill packs receive generic AI guidance rather than expert guidance.

**Workaround:** Create custom skill packs for technologies not covered by existing packs. See the [Skill Pack Development Guide](SKILL_PACK_DEVELOPMENT_GUIDE.md).

**Planned for:** Additional skill packs added in future releases.

## Data Limitations

### Single Database

**Limitation:** All data is stored in a single SQLite database.

**Impact:** If the database file becomes corrupt, all data is at risk.

**Workaround:** Regular backups of the application data directory.

**Planned for:** Not planned — SQLite is the supported storage engine.

### No Database Encryption

**Limitation:** The SQLite database itself is not encrypted on disk.

**Impact:** Database file contents are readable if the file is accessed outside the application.

**Workaround:** Use full-disk encryption on the Windows system.

**Planned for:** SQLite encryption may be added in future releases.

### No Conversation Deduplication

**Limitation:** Conversations are not deduplicated.

**Impact:** Duplicate conversations may accumulate over time.

**Workaround:** Manually delete duplicate conversations.

**Planned for:** Deduplication is planned for future releases.

## Operational Limitations

### Manual Updates

**Limitation:** Updates require manual action (download and install).

**Impact:** Users may fall behind on updates.

**Workaround:** Enable auto-update checks in Settings → Update.

**Planned for:** Automatic silent update installation is planned for future releases.

### No Central Management

**Limitation:** Each user manages their own settings and configuration.

**Impact:** No centralized deployment or configuration management.

**Workaround:** Use profile import/export for configuration consistency across users.

**Planned for:** Central management console is planned for future releases.

### Log Retention Limited to 3 Files

**Limitation:** Only the last 3 log files are retained (configurable, but defaults to 3).

**Impact:** Older log data is not available for analysis.

**Workaround:** Archive log files manually before rotation.

**Planned for:** No change planned — log rotation is a disk space management feature.

---

*For troubleshooting, see [Troubleshooting Guide](TROUBLESHOOTING.md).*
*For support, see [Support Guide](SUPPORT_GUIDE.md).*
*For release information, see [Release Notes](RELEASE_NOTES.md).*