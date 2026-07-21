# FAQ — Wiki Labs AI Copilot v1.0.0

> Frequently asked questions.

## General

### What is Wiki Labs AI Copilot?

Wiki Labs AI Copilot is an enterprise engineering copilot — a desktop application that assists infrastructure engineers with real-time, context-aware guidance. It combines AI-powered chat, knowledge management, skill packs, and desktop observation into a unified tool.

### What platforms are supported?

**v1.0.0** supports Windows 10 and Windows 11 (64-bit). macOS and Linux support is planned for future releases.

### Is there a free/trial version?

The application is distributed under a proprietary license. Contact sales@wikilabs.com for licensing options.

### What AI models does it support?

The application supports any OpenAI-compatible API provider, including:
- **OpenAI** (GPT-4o, GPT-4, GPT-3.5-Turbo)
- **vLLM** (self-hosted OpenAI-compatible API)
- **Ollama** (local model serving)
- Any other OpenAI-compatible endpoint

### Can I use it offline?

The chat feature requires an AI provider connection. However, knowledge management, skill packs, and settings are fully functional offline. If you host a local provider (vLLM/Ollama), you can use the copilot with local models entirely offline.

## Installation

### What are the system requirements?

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| OS | Windows 10 64-bit | Windows 11 64-bit |
| RAM | 4 GB | 8 GB |
| Disk | 2 GB free | 5 GB free |
| .NET | .NET Desktop Runtime 8.0 | .NET Desktop Runtime 8.0 (latest) |

### Can I deploy via Group Policy or SCCM?

Yes. The MSI installer supports enterprise deployment via:
- Group Policy Software Installation
- Microsoft SCCM/MECM
- Microsoft Intune
- Manual silent install

See the [Installation Guide](INSTALLATION_GUIDE.md) for deployment details.

### Does the installer include WebView2?

Yes. The installer includes a WebView2 Runtime installer that runs automatically if WebView2 is not already installed.

## Usage

### How do I start chatting with the AI?

1. Install the application
2. Configure an AI provider in Settings → AI Provider
3. Create a workspace (or use the default)
4. Type a question in the chat and press Enter

### How do I create a workspace?

Click the Workspace panel → New Workspace → enter a name → click Create.

### Can I have multiple workspaces?

Yes. Each workspace has its own:
- Chat history
- Knowledge base
- Active skill packs
- Technology stack

### How does the AI get context about my environment?

The AI receives context from multiple sources:
- **Workspace settings** — Customer name, technology stack
- **Knowledge documents** — Imported documentation and SOPs
- **Skill packs** — Active technology-specific expertise
- **Observation data** — Screen, terminal, and app context (if enabled)
- **Current conversation** — Previous messages in the chat

### What is a skill pack?

A skill pack is a collection of technology-specific expertise. It includes detection rules, best practices, command references, troubleshooting guides, and knowledge documents. Skill packs enable the AI to provide guidance specific to technologies like Linux, MySQL, OpenShift, etc.

### How do skill packs get activated?

The Skill Discovery Engine automatically detects technologies in your environment:
- Browser URL patterns
- Terminal commands
- Active application context
- Configuration files

When a technology is detected, the corresponding skill pack activates automatically. You can also manually enable/disable skills in the Skills panel.

### What is the Guidance Panel?

The Guidance Panel provides proactive, context-aware engineering guidance. It shows recommendations, warnings, suggestions, and tips based on:
- Your current activity
- Detected technology
- Active skill packs
- Conversation context

You can control the guidance level using the operating mode (Minimal, Balanced, Teaching, Expert, Silent).

### Can I export my conversations?

Yes. Right-click any conversation and select "Export as JSON." This exports the full conversation history for archival or documentation purposes.

### Can I import knowledge from existing documentation?

Yes. Import `.wkl` knowledge archives containing `.txt`, `.md`, or `.json` files. The application automatically chunks, embeds, and indexes the content for search.

### How does search work in the Knowledge panel?

Knowledge search uses a hybrid approach:
- **Vector search** — Semantic search using 384-dim embeddings (SQLite VSS extension)
- **Keyword search** — Full-text search using SQLite FTS5

Both are combined for best results.

## AI & Intelligence

### How does the AI generate recommendations?

The copilot engine follows a structured workflow:
1. **Observe** — Collect evidence from screen, terminal, clipboard, and conversation
2. **Recognize** — Detect technology and intent
3. **Recommend** — Generate advice using skill pack knowledge and AI
4. **Evaluate** — Apply decision rules to determine visibility
5. **Present** — Show guidance in the Guidance Panel
6. **Wait for approval** — User approves or denies recommendations

### What is confidence scoring?

Every AI inference includes a confidence score:
- **High** (85-100%) — Auto-confirm, no user interaction needed
- **Medium** (60-84%) — Auto-confirm with confirmation prompt
- **Low** (30-59%) — Ask user to confirm before proceeding

Confidence is calculated based on signal count, signal quality, cross-domain correlation, and historical precedent.

### Can the AI execute commands?

No. The copilot provides **advisory recommendations only**. It never executes commands, changes configurations, or performs automation directly. All actions are performed by the human engineer.

### How does the AI handle prompt injection?

The application includes prompt injection defense mechanisms:
- Input validation on all user messages
- Output validation on AI responses
- Detection of suspicious patterns in messages
- Context isolation between user and system prompts

## Privacy & Security

### Is my data sent to the cloud?

No. All data (chats, knowledge, settings) is stored locally on your machine. Only AI chat requests are sent to your configured AI provider.

### Where is my API key stored?

API keys are stored securely using one of two methods:
1. **Windows Credential Manager** (preferred) — Uses Windows DPAPI
2. **Local encrypted file** (fallback) — Uses AES-256-GCM encryption

Your API key is never stored in plain text.

### What data does the application collect?

| Data | Stored Locally | Sent Remotely |
|------|---------------|---------------|
| Chat messages | Yes | To AI provider (current conversation only) |
| Knowledge documents | Yes | No |
| Settings | Yes | No |
| Screen observation | No (in-memory only) | No |
| Clipboard data | No (in-memory only) | No |
| Crash reports | Yes | Optional (if diagnostics enabled) |
| Telemetry | No | Optional (if enabled) |

### Can I disable all observation features?

Yes. Toggle **Privacy Mode** in Settings → Privacy to disable all observation features with one click. This disables screen observation, OCR, clipboard observation, diagnostics, and telemetry.

### Does the application record my screen?

No. Screen observation is disabled by default and only captures the screen when:
1. The user enables screen observation
2. The feature is actively triggered by the observation engine
3. The captured data is processed in memory and not stored

### Is my data encrypted?

- **Credentials:** Yes, AES-256-GCM or ChaCha20-Poly1305 encryption
- **Network communication:** Yes, TLS 1.2+
- **Database:** Stored on disk; encrypted credential store within
- **Logs:** Sensitive fields are redacted

## Configuration

### Can I use multiple AI providers?

Yes. The application supports multiple AI providers through the profile system. Each profile can have its own provider configuration.

### Can I use a local AI model?

Yes. Configure vLLM or Ollama as your AI provider:
- **vLLM:** Set endpoint to `http://localhost:8000/v1`
- **Ollama:** Set endpoint to `http://localhost:11434/v1`

No API key is required for local providers.

### How do I update the application?

Updates are handled automatically through the tauri-plugin-updater:
1. On startup, the application checks for updates
2. If an update is available, a notification appears
3. Click **Download and Install** to proceed

You can also manually install by downloading the latest installer and running it (it detects and upgrades existing installations).

### Where are settings stored?

Settings are stored at `%APPDATA%\com.wikilabs.copilot\settings.json`. The application manages this file — do not edit it manually while the application is running.

### Where is the database stored?

The SQLite database is stored at `%APPDATA%\com.wikilabs.copilot\wikilabs.db`. It contains all workspaces, chat history, and knowledge data.

### How do I back up my data?

Close the application and copy the `%APPDATA%\com.wikilabs.copilot\` directory to a backup location. To restore, copy it back after reinstalling.

## Troubleshooting

### The application won't start

1. Check that WebView2 is installed
2. Check that .NET Desktop Runtime 8.0 is installed
3. Check log files at `%APPDATA%\com.wikilabs.copilot\logs\`
4. Try resetting settings (rename `settings.json` to `settings.json.bak`)

### AI responses are slow

1. Check your network connection
2. Try a faster model in settings
3. Reduce the context window size
4. If using a local provider, check GPU/CPU resources

### Search returns no results

1. Verify documents are imported into the correct workspace
2. Check that embeddings were generated successfully
3. Try a different search term
4. Check logs for embedding errors

### Skill pack not activating

1. Verify the skill pack is installed in `src/skills/`
2. Check that the detection rules match your environment
3. Try enabling the skill manually in the Skills panel

### Where can I get help?

See the [Support Guide](SUPPORT_GUIDE.md) for support channels and escalation procedures.

---

*For detailed feature documentation, see [User Guide](user-guide/USER_GUIDE.md).*
*For installation, see [Installation Guide](INSTALLATION_GUIDE.md).*
*For administration, see [Administrator Guide](admin-guide/ADMINISTRATOR_GUIDE.md).*