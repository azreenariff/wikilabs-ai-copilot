# FAQ — Wiki Labs AI Copilot v1.0.0

> Frequently asked questions for end users and administrators.

## Table of Contents

### General
1. [What is Wiki Labs AI Copilot?](#what-is-wiki-labs-ai-copilot)
2. [What platforms does it support?](#what-platforms-does-it-support)
3. [Is it free or commercial?](#is-it-free-or-commercial)
4. [How is it different from other AI coding assistants?](#how-is-it-different-from-other-ai-coding-assistants)
5. [Does it store my data in the cloud?](#does-it-store-my-data-in-the-cloud)

### Getting Started
6. [How do I install Wiki Labs AI Copilot?](#how-do-i-install-wiki-labs-ai-copilot)
7. [What do I need to configure before first use?](#what-do-i-need-to-configure-before-first-use)
8. [How do I add an AI provider?](#how-do-i-add-an-ai-provider)
9. [What AI providers are supported?](#what-ai-providers-are-supported)
10. [Can I use my own AI provider (self-hosted)?](#can-i-use-my-own-ai-provider-self-hosted)

### Skills and Technology
11. [What are skill packs?](#what-are-skill-packs)
12. [What skill packs come pre-installed?](#what-skill-packs-come-pre-installed)
13. [How do I install a new skill pack?](#how-do-i-install-a-new-skill-pack)
14. [How do I remove a skill pack?](#how-do-i-remove-a-skill-pack)
15. [How does the app know which skill pack to use?](#how-does-the-app-know-which-skill-pack-to-use)
16. [Can I create my own skill pack?](#can-i-create-my-own-skill-pack)
17. [How does technology auto-detection work?](#how-does-technology-auto-detection-work)

### Using the AI Chat
18. [How do I chat with the AI assistant?](#how-do-i-chat-with-the-ai-assistant)
19. [Does the AI know what I'm working on?](#does-the-ai-know-what-im-working-on)
20. [How can I correct the AI when it's wrong?](#how-can-i-correct-the-ai-when-its-wrong)
21. [Does the AI execute commands on my behalf?](#does-the-ai-execute-commands-on-my-behalf)
22. [How do I save or export a conversation?](#how-do-i-save-or-export-a-conversation)

### Proactive Guidance
23. [What is proactive guidance?](#what-is-proactive-guidance)
24. [How do I enable or disable proactive guidance?](#how-do-i-enable-or-disable-proactive-guidance)
25. [What are the recommendation modes?](#what-are-the-recommendation-modes)
26. [Can I dismiss a recommendation?](#can-i-dismiss-a-recommendation)

### Privacy and Security
27. [What data does the application collect?](#what-data-does-the-application-collect)
28. [Can I disable all observation features?](#can-i-disable-all-observation-features)
29. [What is Privacy Mode?](#what-is-privacy-mode)
30. [How are my API keys stored?](#how-are-my-api-keys-stored)
31. [Is the data on my machine encrypted?](#is-the-data-on-my-machine-encrypted)

### Administration
32. [How do I back up my data?](#how-do-i-back-up-my-data)
33. [How do I restore from a backup?](#how-do-i-restore-from-a-backup)
34. [How do I update the application?](#how-do-i-update-the-application)
35. [How do I generate a diagnostic package?](#how-do-i-generate-a-diagnostic-package)
36. [Where are the log files located?](#where-are-the-log-files-located)
37. [How do I change the log level?](#how-do-i-change-the-log-level)

### Troubleshooting
38. [The AI is not responding. What should I check?](#the-ai-is-not-responding-what-should-i-check)
39. [The application crashes on startup. What should I do?](#the-application-crashes-on-startup-what-should-i-do)
40. [A skill pack is not being loaded. What should I check?](#a-skill-pack-is-not-being-loaded-what-should-i-check)
41. [How do I reset the application to defaults?](#how-do-i-reset-the-application-to-defaults)

---

## General

### What is Wiki Labs AI Copilot?

Wiki Labs AI Copilot is an enterprise-grade desktop application that provides real-time, context-aware guidance for infrastructure engineers. It combines AI-powered chat, local knowledge management, and skill-based engineering knowledge to help engineers think, diagnose, and solve complex infrastructure problems.

The application is designed for infrastructure engineers who work with technologies like Kubernetes, Linux, databases, virtualization, monitoring systems, and automation tools.

### What platforms does it support?

Currently, Wiki Labs AI Copilot supports:
- **Windows** — Native desktop application via Tauri v2 with WebView2

Future platform support (planned for future releases) may include Linux and macOS.

### Is it free or commercial?

The application is provided by Wiki Labs. Licensing terms are defined in the LICENSE file included with the installation. Refer to the [Release Notes](RELEASE_NOTES.md) for the current licensing status of version 1.0.0.

### How is it different from other AI coding assistants?

| Feature | Other AI Assistants | Wiki Labs AI Copilot |
|---------|-------------------|---------------------|
| Focus | General coding | Infrastructure engineering |
| Context | Source code only | Observation, terminal, browser, clipboard |
| Knowledge | Pre-trained models | Custom skill packs per technology |
| Recommendations | Code suggestions | Evidence-based engineering guidance |
| Data location | Cloud-based | Local-first (data stays on your machine) |
| Execution | Some execute code | Advisory-only (never executes) |
| Reasoning | Black-box AI | Transparent reasoning with confidence scores |
| Human feedback | Limited | Always overrides AI inference |

### Does it store my data in the cloud?

No. All conversation data, knowledge base content, and settings are stored locally on your machine in SQLite (`wikilabs.db`) and JSON files under `%APPDATA%\com.wikilabs.copilot`. The application does not synchronize data to remote servers.

When you send a message to the AI assistant, only that message and the assembled context are sent to your configured AI provider. No other application data leaves your machine.

---

## Getting Started

### How do I install Wiki Labs AI Copilot?

1. Download the installer (MSI or NSIS) from the release page
2. Run the installer as a standard user (or administrator for per-machine install)
3. Follow the installation wizard
4. Launch the application from the Start menu or desktop shortcut

For detailed instructions, see the [Installation Guide](INSTALLATION_GUIDE.md).

### What do I need to configure before first use?

Before first use, you need to configure:

1. **AI Provider** — Configure the AI provider endpoint, API key, and model (Settings → AI Provider)
2. **Workspace** — Set up a workspace for organizing your work (optional but recommended)
3. **Privacy Settings** — Review observation feature toggles (Settings → Privacy)

All other features (skill packs, knowledge base) are pre-configured with the default installation.

### How do I add an AI provider?

1. Open the application
2. Go to **Settings** → **AI Provider**
3. Enter the following:
   - **Endpoint URL** — e.g., `https://api.openai.com/v1` or `http://localhost:8080` for local providers
   - **API Key** — Your API key (stored encrypted via Windows Credential Manager)
   - **Model** — The model to use (e.g., `gpt-4`, `my-local-model`)
   - **Max Tokens** — Maximum tokens per response (e.g., `4096`)
   - **Context Window** — Context window size (e.g., `32768`)
4. Click **Test Connection** to verify the configuration
5. Click **Save**

### What AI providers are supported?

The application supports any AI provider that implements the OpenAI-compatible API:

| Provider | Type | Example Endpoint |
|----------|------|-----------------|
| **OpenAI** | Cloud | `https://api.openai.com/v1` |
| **vLLM** | Self-hosted | `http://localhost:8080/v1` |
| **Ollama** | Self-hosted | `http://localhost:11434/v1` |
| **Other OpenAI-compatible** | Any | Your provider's endpoint |

The application uses a trait-based provider abstraction, so any provider with an OpenAI-compatible endpoint can be configured.

### Can I use my own AI provider (self-hosted)?

Yes. The application supports self-hosted providers through the OpenAI-compatible interface:

1. Set the **endpoint URL** to your provider's address (e.g., `http://localhost:8080/v1`)
2. If your provider doesn't require authentication, leave the **API key** blank
3. Enter the correct **model** name as defined by your provider
4. Click **Test Connection** to verify

Self-hosted providers communicate over HTTP (not HTTPS) since they run on localhost.

---

## Skills and Technology

### What are skill packs?

Skill packs are bundles of technology-specific knowledge that tell the AI Copilot about a specific technology domain. Each skill pack includes:

- **Detection rules** — Patterns to identify when you're working with that technology
- **Troubleshooting workflows** — Step-by-step diagnostic procedures
- **Command knowledge** — CLI commands with explanations and risks
- **Knowledge base** — Deep technical documentation
- **Reasoning guides** — How to analyze and diagnose problems
- **Best practices** — Engineering standards and conventions
- **Known issues** — Documented problems and workarounds

Skill packs are loaded at application startup and can be installed, removed, or updated by managing the files in `src/skills/`.

### What skill packs come pre-installed?

Version 1.0.0 ships with 10 pre-installed skill packs:

| Skill Pack | Technology | Files |
|------------|-----------|-------|
| openshift-skill-pack | Red Hat OpenShift 4.x | 40 |
| linux-engineering | Linux Administration | 40 |
| vmware-vsphere-skill-pack | VMware vSphere | 40 |
| mysql-skill-pack | MySQL DBA 8.0 | 41 |
| edb-postgresql-skill-pack | EDB PostgreSQL 15/16 | 34 |
| mssql-skill-pack | Microsoft SQL Server 2022 | 28 |
| checkmk-skill-pack | Checkmk 2.3/2.4 | 21 |
| ansible-skill-pack | Ansible | 20 |
| nagios-logserver-skill-pack | Nagios Log Server | 20 |
| nagios-xi-skill-pack | Nagios XI | 19 |

### How do I install a new skill pack?

1. Stop the application (close all running instances)
2. Copy the skill pack directory to `src/skills/`:
   ```powershell
   Copy-Item ".\my-new-skill-pack" ".\src\skills\" -Recurse
   ```
3. Start the application — skills are loaded on startup only

The new skill pack will appear in the Skills panel after restart.

### How do I remove a skill pack?

1. Stop the application
2. Delete the skill pack directory from `src/skills/`:
   ```powershell
   Remove-Item ".\src\skills\my-skill-pack" -Recurse
   ```
3. Start the application

### How does the app know which skill pack to use?

The application uses a multi-stage detection pipeline:

1. **Skill Discovery** — At startup, the application scans `src/skills/` for skill directories and loads all manifests
2. **Observation** — The observation engine monitors your screen, terminal, browser, and clipboard
3. **Technology Recognition** — The recognition engine matches observation events against detection rules in skill packs
4. **Context Fusion** — Detected technologies are included in the fused context sent to the AI
5. **Active Activation** — The activation engine enables skills based on workspace signals (current task, detected technologies)

When you type a command or open a relevant application, the detection rules in the matching skill pack's `detection_rules.yaml` identify the technology and the AI can provide context-aware guidance.

### Can I create my own skill pack?

Yes. The Skill SDK provides tools for creating, validating, and generating skill packs. See the [Skill Pack Development Guide](SKILL_PACK_DEVELOPMENT_GUIDE.md) for a complete walkthrough.

### How does technology auto-detection work?

Technology auto-detection uses the following pipeline:

1. The **observation engine** captures events from screen, terminal, browser, and clipboard
2. Each event is published to an **event bus**
3. The **technology recognition engine** matches events against detection rules from all loaded skill packs
4. Detection rules specify patterns (command regex, file path, browser URL) with confidence scores and priorities
5. Multiple detection rules are combined through a multi-pass pipeline
6. The result is a **technology inference** with composite confidence, included in the fused context

Detection rules are defined in each skill pack's `detection_rules.yaml` file. The engine does not have any hardcoded technology logic — it relies entirely on detection rules in skill packs.

---

## Using the AI Chat

### How do I chat with the AI assistant?

1. Open the application
2. Go to the **AI Chat** panel
3. Type your question or description of the problem
4. Press Enter or click Send
5. The AI responds with guidance, explanations, and recommendations
6. You can continue the conversation — the AI maintains context from the full conversation history

### Does the AI know what I'm working on?

Yes, when observation features are enabled. The AI receives:

- **Technology inferences** — What technologies you're working with (detected from terminal commands, file paths, browser URLs)
- **Intent inferences** — What you might be trying to accomplish (detected from patterns)
- **Workflow state** — Current troubleshooting workflow and progress
- **Timeline entries** — Recent engineering activities
- **Human corrections** — Any corrections you've made to previous AI inferences

This context is assembled by the Context Fusion Engine and included in every AI request to improve relevance.

### How can I correct the AI when it's wrong?

The application has multiple correction mechanisms:

1. **Human Feedback Panel** — When the AI makes an incorrect inference, you can mark it as incorrect in the guidance panel. The correction is stored and applied to future inferences.

2. **Chat correction** — You can explicitly tell the AI in the chat: "Actually, I'm using vSphere, not OpenShift." This correction is added to the context for future responses.

3. **Intent correction** — If the AI misclassifies your intent, you can correct it through the guidance panel. Future inferences about your intent will use your correction.

The principle of **human supremacy** ensures that all human corrections always override AI inferences.

### Does the AI execute commands on my behalf?

**No.** Wiki Labs AI Copilot is advisory-only. The AI assistant:
- Recommends commands and actions
- Explains what commands do and their risks
- Provides reasoning and evidence for recommendations
- Never executes commands on your behalf
- Never modifies files or configurations without your explicit action

Every recommendation includes:
- The command or action
- An explanation of what it does
- Risk assessment (low/medium/high)
- A rollback strategy
- A confidence score

### How do I save or export a conversation?

Conversations are automatically saved in the local SQLite database. To export:

1. Go to the conversation in the AI Chat panel
2. Use the export function (available in the conversation menu)
3. Choose the export format (text or JSON)
4. The conversation history is saved to a file on your local machine

Conversations are also viewable through the application's conversation history interface (Settings → Conversations).

---

## Proactive Guidance

### What is proactive guidance?

Proactive guidance is the application's ability to provide recommendations without being explicitly asked. When enabled, the application:

- Observes your work through the observation engine
- Detects technologies you're using
- Analyzes patterns and identifies potential issues
- Generates recommendations when appropriate
- Displays them in the guidance panel

Proactive guidance is **advisory only** — it never executes actions, only recommends them.

### How do I enable or disable proactive guidance?

1. Go to **Settings** → **Guidance**
2. Toggle **Proactive Guidance** on or off
3. When enabled, also select your preferred **Recommendation Mode**

You can also toggle observation features individually in **Settings** → **Privacy**.

### What are the recommendation modes?

| Mode | Behavior |
|------|----------|
| **Minimal** | Only critical warnings shown |
| **Balanced** | Standard recommendations with moderate detail |
| **Teaching** | Explanations with reasoning and evidence |
| **Expert** | Detailed technical guidance for experienced engineers |
| **Silent** | No proactive recommendations (on-demand only) |

### Can I dismiss a recommendation?

Yes. When a recommendation is displayed in the guidance panel:

1. Click **Dismiss** to hide the recommendation
2. Optionally select a dismissal reason (not relevant, already known, etc.)
3. The dismissal is tracked in session memory for personalization
4. Similar recommendations may be shown less frequently in the future

---

## Privacy and Security

### What data does the application collect?

The application only collects data that it needs to function:

| Data | Stored Locally | Sent to Provider | Purpose |
|------|-------------|-----------------|---------|
| Conversations | Yes | Only current message | Chat history |
| API keys | Yes (encrypted) | Yes (for auth) | AI provider auth |
| Settings | Yes (encrypted) | No | Application config |
| Knowledge base | Yes | Only relevant snippets | Context-aware responses |
| Observation events | Yes (filtered) | Only fused context | Technology detection |
| Skill packs | Yes (disk) | No | Engineering knowledge |

### Can I disable all observation features?

Yes. Go to **Settings** → **Privacy** and toggle off:
- Screen observation
- OCR processing
- Clipboard observation
- Diagnostics collection
- Telemetry

Or use **Privacy Mode** for a one-click disable of all observation and collection features.

### What is Privacy Mode?

Privacy Mode is a one-click toggle in the Settings → Privacy panel that:

- Stops all running observation providers
- Blocks new observation events from being recorded
- Prevents screenshots, clipboard data, and terminal commands from being captured
- Ensures the AI receives no observation context

Toggle Privacy Mode on when you need a break from observation, or in environments where observation is not permitted.

### How are my API keys stored?

API keys are stored using multiple layers of protection:

1. **Encryption** — Keys are encrypted using AES-256-GCM before storage
2. **OS Credential Manager** — Keys are also stored in Windows Credential Manager (DPAPI-protected)
3. **Redaction** — Keys are automatically redacted in all logs and diagnostic output
4. **Never in plain text** — Keys are never written to settings.json or any log file

### Is the data on my machine encrypted?

The application uses a multi-layered encryption model:

| Data Type | Encryption | Storage |
|-----------|-----------|---------|
| API keys | AES-256-GCM | SQLite, Windows Credential Manager |
| Settings (sensitive fields) | AES-256-GCM | settings.json |
| Conversations | Not encrypted by default | SQLite (confidential data classification supports encryption) |
| Knowledge base | Not encrypted by default | SQLite |
| Log files | Not encrypted (credentials redacted) | File system |

Confidential and Restricted data classifications require encryption per the security model.

---

## Administration

### How do I back up my data?

The most important data to back up is:
- **Database** — `%APPDATA%\com.wikilabs.copilot\wikilabs.db` (conversations, workspaces, skills)
- **Settings** — `%APPDATA%\com.wikilabs.copilot\settings.json` (configuration)

To back up:

```powershell
# Manual backup
Copy-Item "$env:APPDATA\com.wikilabs.copilot\wikilabs.db" "C:\Backup\"
Copy-Item "$env:APPDATA\com.wikilabs.copilot\settings.json" "C:\Backup\"
```

For automated backups, set up a scheduled task. See the [Operations Guide](OPERATIONS_GUIDE.md) for detailed backup procedures.

### How do I restore from a backup?

1. Stop the application
2. Copy the backup files over the originals:
   ```powershell
   Copy-Item "C:\Backup\wikilabs.db" "$env:APPDATA\com.wikilabs.copilot\" -Force
   Copy-Item "C:\Backup\settings.json" "$env:APPDATA\com.wikilabs.copilot\" -Force
   ```
3. Start the application

See the [Operations Guide](OPERATIONS_GUIDE.md) for full restore procedures.

### How do I update the application?

1. Open **Settings** → **Update**
2. Click **Check for Updates**
3. If a newer version is available, follow the update wizard
4. Or download the installer from the release page and run it

The installer preserves all user data (database, settings, logs) during upgrades.

### How do I generate a diagnostic package?

1. Open **Settings** → **Diagnostics**
2. Click **Generate Diagnostic Package**
3. Choose a save location
4. The package is saved as a `.zip` archive containing:
   - Version information
   - Redacted settings report
   - Log files (with sensitive data redacted)
   - System information
   - Database info
   - Validation errors
   - Crash info (if applicable)

This package is useful for troubleshooting and support requests.

### Where are the log files located?

Log files are stored at:
```
%APPDATA%\com.wikilabs.copilot\logs\
```

Files include:
- `wikilabs-copilot.log` — Current log file
- `wikilabs-copilot.log.1`, `.2`, etc. — Rotated log files

The logs use structured JSON format and automatic rotation (3 files, 10 MB each).

### How do I change the log level?

1. Open **Settings** → **Logging**
2. Select the desired log level:
   - **DEBUG** — Most verbose, for troubleshooting
   - **INFO** — Default, normal operational events
   - **WARN** — Warnings and recoverable failures
   - **ERROR** — Only errors and failures

Changing the log level takes effect immediately after saving settings.

---

## Troubleshooting

### The AI is not responding. What should I check?

1. **Verify AI Provider configuration**
   - Go to Settings → AI Provider
   - Check the endpoint URL is correct
   - Verify the API key is valid

2. **Test the connection**
   - Click **Test Connection** in Settings → AI Provider
   - Check the response time (should be < 5 seconds)

3. **Check the provider status**
   - For cloud providers (OpenAI), check the provider's status page
   - For local providers (vLLM, Ollama), verify the service is running

4. **Check logs**
   ```powershell
   Select-String -Path "$env:APPDATA\com.wikilabs.copilot\logs\*.log" -Pattern "provider|error"
   ```

5. **Check network connectivity**
   - Ensure your machine can reach the provider endpoint
   - For local providers, ensure no firewall is blocking the port

6. **Restart the application** — Sometimes a restart resolves temporary issues

### The application crashes on startup. What should I do?

1. **Check the crash report**
   ```powershell
   Get-Content "$env:APPDATA\com.wikilabs.copilot\crash\last_crash.json"
   ```

2. **Check the error log**
   ```powershell
   Get-Content "$env:APPDATA\com.wikilabs.copilot\crash\error_log.jsonl" -Tail 50
   ```

3. **Try a clean start**
   - Rename the settings file: `ren "%APPDATA%\com.wikilabs.copilot\settings.json" "settings.json.bak"`
   - Start the application (creates fresh default settings)
   - If this works, reconfigure your settings

4. **Check disk space**
   ```powershell
   Get-PSDrive $env:SystemDrive | Select-Object Used, Free
   ```
   Ensure at least 10% free space on the system drive.

5. **Generate a diagnostic package** and contact support.

### A skill pack is not being loaded. What should I check?

1. **Verify the directory exists**
   ```powershell
   Test-Path ".\src\skills\my-skill-pack"
   ```

2. **Verify manifest.yaml exists**
   ```powershell
   Test-Path ".\src\skills\my-skill-pack\manifest.yaml"
   ```

3. **Validate the manifest**
   - Check that `id`, `name`, and `version` fields are present and non-empty
   - Verify the YAML is valid

4. **Check for errors in logs**
   ```powershell
   Select-String -Path "$env:APPDATA\com.wikilabs.copilot\logs\*.log" -Pattern "skill"
   ```

5. **Restart the application** — Skills are loaded on startup only

6. **Use the Skills panel** in Settings to verify the skill appears in the loaded skills list

### How do I reset the application to defaults?

1. **Stop the application**
2. **Rename the settings file**
   ```powershell
   ren "%APPDATA%\com.wikilabs.copilot\settings.json" "settings.json.bak"
   ```
3. **Start the application** — Creates fresh default settings
4. **Reconfigure** — Re-enter your AI provider settings and preferences

This does not affect your conversations, knowledge base, or skill packs.

---

*For more detailed troubleshooting procedures, see the [Troubleshooting Guide](TROUBLESHOOTING.md). For support escalation procedures, see the [Support Guide](SUPPORT_GUIDE.md).*