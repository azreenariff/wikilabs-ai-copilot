# User Guide — Wiki Labs AI Copilot v1.0.0

> Complete user manual for end users.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Workspace Management](#workspace-management)
3. [AI Chat](#ai-chat)
4. [Knowledge Management](#knowledge-management)
5. [Skill Packs](#skill-packs)
6. [Guidance Panel](#guidance-panel)
7. [Settings](#settings)
8. [Privacy Controls](#privacy-controls)
9. [Keyboard Shortcuts](#keyboard-shortcuts)
10. [Common Tasks](#common-tasks)
11. [Tips & Best Practices](#tips--best-practices)

## Getting Started

### First Launch

When you first launch Wiki Labs AI Copilot, you will see:

1. **Sidebar** — Left side with navigation panels
2. **Chat area** — Central area for conversations
3. **Knowledge panel** — Right side for knowledge documents
4. **Skills panel** — Right side for active skill packs

### Configuring Your AI Provider

Before you can chat with the AI, you need to configure an AI provider:

1. Click the **Settings** icon (gear ⚙) in the sidebar
2. Navigate to the **AI Provider** section
3. Fill in the required fields:

   | Field | Description | Example |
   |-------|-------------|---------|
   | Name | Provider name | `OpenAI` |
   | Endpoint | API URL | `https://api.openai.com/v1` |
   | API Key | Your API key | `sk-...` |
   | Model | Model to use | `gpt-4o` |
   | Max Tokens | Response length limit | `4096` |
   | Context Window | Available context tokens | `128000` |

4. Click **Test Connection** to verify your configuration
5. Click **Save** to store your settings

The API key is stored securely using Windows Credential Manager (or local encryption as a fallback).

### Available AI Providers

| Provider | Endpoint | API Key | Notes |
|----------|----------|---------|-------|
| **OpenAI** | `https://api.openai.com/v1` | Yes | Standard GPT models |
| **vLLM** | `http://localhost:8000/v1` | No | Self-hosted, local |
| **Ollama** | `http://localhost:11434/v1` | No | Local inference |

## Workspace Management

### Creating a Workspace

1. Click the **Workspace** panel in the sidebar
2. Click **New Workspace**
3. Enter a **Workspace Name** (required)
4. Enter a **Customer Name** (optional)
5. Add **Technology Stack** items (optional):
   - Click the technology dropdown
   - Select one or more technologies
   - Technologies provide context for AI responses

### Switching Workspaces

1. Click the **Workspace** panel
2. Click on any workspace in the list
3. The sidebar and chat area update to show that workspace's data

### Deleting a Workspace

1. Click the **Workspace** panel
2. Right-click (or click the menu) on the workspace you want to delete
3. Click **Delete Workspace**
4. Confirm the deletion in the dialog

> **Warning:** Deleting a workspace removes all its chat history and knowledge. This cannot be undone.

### Workspace Structure

Each workspace maintains its own:
- Chat conversations and history
- Knowledge documents
- Active skill packs
- Technology stack
- Settings overrides

## AI Chat

### Starting a Conversation

1. Select your workspace from the sidebar
2. Type your question or message in the chat input
3. Press **Enter** to send, or click the **Send** button

### Sending Messages

- Press **Enter** to send a message
- Press **Shift + Enter** to insert a new line
- Messages are sent to the AI provider and streamed back in real time

### Managing Conversations

| Action | How |
|--------|-----|
| New conversation | Click **New Conversation** in the sidebar |
| Switch conversation | Click on a conversation in the list |
| Rename conversation | Right-click conversation → **Rename** |
| Delete conversation | Right-click conversation → **Delete** |
| Export conversation | Right-click conversation → **Export as JSON** |
| Clear conversation | Click **Clear** button (removes all messages) |

### Chat Features

- **Streaming responses:** Messages appear character by character as the AI generates them
- **Tool calls:** Some models support tool calls (function calling) — these appear inline
- **Conversation history:** Previous messages are preserved and sent with each new message
- **Tag-based categorization:** Tag conversations for easier organization

### AI Best Practices

1. **Be specific:** Provide detailed context in your questions
2. **Use technology context:** Set your workspace technology stack for better responses
3. **Reference knowledge:** Import relevant documentation into your knowledge base
4. **Iterate:** Ask follow-up questions to refine answers
5. **Check confidence:** The guidance panel shows confidence levels for recommendations

## Knowledge Management

### Importing Knowledge

1. Click the **Knowledge** panel in the sidebar
2. Click **Import Knowledge**
3. Select a `.wkl` knowledge archive (or individual `.txt`, `.md`, `.json` files)
4. The application parses and indexes the documents

### Knowledge Features

- **Vector search:** Semantic search over your knowledge documents
- **Keyword search:** Full-text search using SQLite FTS5
- **Automatic chunking:** Documents are automatically split into searchable chunks
- **384-dim embeddings:** Generated locally using ONNX Runtime

### Managing Knowledge Documents

| Action | How |
|--------|-----|
| Import documents | Click **Import Knowledge** |
| View documents | Click on a document in the list |
| Search knowledge | Use the search bar in the Knowledge panel |
| Delete documents | Right-click document → **Delete** |

### Knowledge Best Practices

1. **Import relevant SOPs:** Import standard operating procedures and manuals
2. **Organize by workspace:** Create separate knowledge bases per workspace
3. **Keep documents current:** Re-import when procedures change
4. **Use descriptive titles:** Makes searching and browsing easier

## Skill Packs

### What Are Skill Packs?

Skill packs provide expert knowledge for specific technologies. They include:
- Detection rules to identify relevant technology
- Best practices and troubleshooting guides
- Command references and workflows
- Engineering context and reasoning

### Browsing Skills

1. Click the **Skills** panel in the sidebar
2. Browse the list of available skills
3. Click on a skill to see details and configuration

### Enabling and Disabling Skills

| Action | How |
|--------|-----|
| Enable skill | Click the skill in the list → Click **Enable** |
| Disable skill | Click the skill → Click **Disable** |
| Update skill | Click the skill → Click **Update** (if available) |

### Available Skill Packs

| Skill Pack | Technology | Description |
|-----------|------------|-------------|
| OpenShift | Red Hat OpenShift 4.x | Container platform administration |
| Linux Engineering | Linux Administration | General Linux system management |
| VMware vSphere | VMware vSphere | Virtualization management |
| Nagios XI | Nagios XI | Monitoring system administration |
| Nagios Log Server | Nagios Log Server | Log management |
| Checkmk | Checkmk | Monitoring and management |
| Ansible | Ansible | Configuration management |
| MySQL | MySQL 8.0 | Database administration |
| EDB PostgreSQL | EDB PostgreSQL 15/16 | Database administration |
| Microsoft SQL Server | SQL Server 2022 | Database administration |

### Skill Detection

The Skill Discovery Engine automatically detects technology in your environment:
- Browser URL patterns
- Terminal commands
- Active application context
- Configuration files

When a technology is detected, the corresponding skill pack activates automatically.

## Guidance Panel

### What Is the Guidance Panel?

The Guidance Panel provides context-aware engineering guidance based on:
- Your current activity
- Detected technology
- Active skill packs
- Conversation context

### Guidance Types

| Type | Description | Example |
|------|-------------|---------|
| **Recommendation** | Actionable advice | "Consider increasing buffer pool size" |
| **Warning** | Potential issue alert | "Replication lag exceeds 30 seconds" |
| **Suggestion** | Helpful tip | "Check the slow query log" |
| **Tip** | Best practice reminder | "Use EXPLAIN ANALYZE" |
| **Explanation** | Context explanation | "This error indicates a lock timeout" |

### Controlling Guidance

1. Click the **Guidance** panel in the sidebar
2. Adjust the operating mode:
   - **Minimal** — Only critical recommendations
   - **Balanced** — Critical + important (default)
   - **Teaching** — All recommendations with explanations
   - **Expert** — All recommendations, technical detail
   - **Silent** — No recommendations

### Human Approval

For actionable recommendations, the guidance panel supports approval workflows:
- **Pending** — Recommendation awaiting your action
- **Approved** — You accepted the recommendation
- **Denied** — You rejected the recommendation
- **AutoApproved** — Auto-approved after timeout

## Settings

### Overview

Settings are organized into 8 sections in the Settings panel:

### 1. AI Provider

Configure your AI provider connection.

| Setting | Description |
|---------|-------------|
| Provider Name | Display name for this provider |
| Endpoint | API endpoint URL |
| API Key | Securely stored API key |
| Model | AI model to use |
| Max Tokens | Maximum response length |
| Context Window | Available context size |

### 2. UI Settings

Customize the application appearance.

| Setting | Description |
|---------|-------------|
| Theme | Dark, Light, or System default |
| Font Size | Text size in the interface |
| Zoom Level | Overall UI scaling |
| Language | Interface language |
| Minimize to Tray | Minimize to system tray instead of closing |
| Shortcuts Help | Show keyboard shortcuts reference |

### 3. Privacy Settings

Control what data the application can observe.

| Setting | Description | Default |
|---------|-------------|---------|
| Screen Observation | Allow screen content capture | **Off** |
| OCR | Optical character recognition on screens | **On** |
| Clipboard Observation | Allow clipboard content capture | **Off** |
| Diagnostics | Allow crash reports | **On** |
| Telemetry | Allow analytics data | **Off** |
| Privacy Mode | One-click disable all observation | **Off** |

> **Privacy mode** disables all observation and data collection when enabled. This is recommended when sharing your screen or using the application in public environments.

### 4. Security Settings

Configure credential storage and encryption.

| Setting | Description | Default |
|---------|-------------|---------|
| Use Credential Manager | Use Windows Credential Manager | **On** |
| Local Encryption | Fallback to local AES-256-GCM | **On** |
| Encryption Algorithm | AES-256-GCM or ChaCha20-Poly1305 | AES-256-GCM |
| Auto-Lock | Lock after N minutes of inactivity | 30 minutes |
| PIN Protection | Require PIN for credential access | **Off** |

### 5. Update Settings

Configure automatic update behavior.

| Setting | Description | Default |
|---------|-------------|---------|
| Auto-Check | Automatically check for updates | **On** |
| Channel | Update channel (stable/preview/internal) | Stable |
| Show Dialog | Show notification when update available | **On** |
| Allow Deferral | Allow user to postpone update | **On** |

### 6. Logging Settings

Configure application logging.

| Setting | Description | Default |
|---------|-------------|---------|
| Log Level | Minimum log level (trace/debug/info/warn/error) | Info |
| File Logging | Write logs to file | **On** |
| Max Log Size | Maximum log file size (MB) | 10 MB |
| Max Log Files | Number of log files to retain | 3 |
| Structured Logging | Use JSON format | **On** |

### 7. Window Settings

Control window behavior and appearance.

| Setting | Description |
|---------|-------------|
| Window Width | Width in pixels |
| Window Height | Height in pixels |
| Window Position | X, Y coordinates |
| Maximized | Window starts maximized |
| Last Workspace | Resume to this workspace on launch |
| Active Panel | Default panel to show |

### 8. Profile Management

Manage named profiles with independent settings.

| Action | Description |
|--------|-------------|
| Create Profile | New named profile |
| Switch Profile | Change active profile |
| Export Profile | Export settings as JSON |
| Import Profile | Import settings from JSON |
| Delete Profile | Remove a profile |

## Privacy Controls

### Understanding Privacy Controls

All observation features are **disabled by default**. The application respects your privacy choices:

| Feature | What It Does | Default |
|---------|-------------|---------|
| Screen Observation | Captures screen content for context | **Off** |
| OCR | Extracts text from screen images | **On** (but only when screen observation is on) |
| Clipboard Observation | Reads clipboard contents | **Off** |
| Diagnostics | Sends crash reports | **On** |
| Telemetry | Sends usage analytics | **Off** |

### One-Click Privacy Mode

Privacy mode is a quick way to disable all observation features:

1. Open Settings → Privacy
2. Toggle **Privacy Mode** to On
3. All observation features are immediately disabled
4. Toggle off to re-enable individual features at their saved settings

### What Data Is Collected

- **Local only:** All data (chats, knowledge, settings) is stored locally
- **AI requests:** Only the current conversation context is sent to the AI provider
- **No telemetry by default:** Analytics are opt-in only
- **No screen storage:** Screen observations are not stored, only processed in memory

## Keyboard Shortcuts

### General Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift + Enter` | New line in message |
| `Ctrl + N` | New conversation |
| `Ctrl + W` | New workspace |
| `Ctrl + E` | Export conversation |
| `Ctrl + K` | Clear conversation |
| `Ctrl + ,` | Open Settings |
| `Ctrl + L` | Focus search |
| `Escape` | Close panel/modal |
| `F1` | Help / Keyboard shortcuts |

### Navigation Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + Tab` | Cycle through panels |
| `Ctrl + 1` | Switch to Workspace panel |
| `Ctrl + 2` | Switch to Knowledge panel |
| `Ctrl + 3` | Switch to Skills panel |

### Guidance Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + G` | Focus Guidance panel |
| `Ctrl + A` | Approve recommendation |
| `Ctrl + D` | Deny recommendation |

## Common Tasks

### Task: Ask About a Linux Server Issue

1. Create a workspace named "Production Server"
2. Set the technology stack to include "Linux"
3. Import your server documentation into the Knowledge panel
4. Type in chat: "How do I diagnose high CPU on this Linux server?"
5. The AI responds with evidence-based guidance

### Task: Search Knowledge for a Configuration

1. Open the Knowledge panel
2. Type your search term in the search bar
3. Results appear from both vector and keyword search
4. Click a result to view the full document

### Task: Get Guidance on a Specific Technology

1. Browse the Skills panel
2. Find your technology (e.g., "MySQL")
3. Enable the skill pack
4. The Guidance panel will now show MySQL-specific recommendations

### Task: Export a Conversation for Documentation

1. In the conversation list, right-click the conversation
2. Select **Export as JSON**
3. Save the file to share or archive

### Task: Switch Between Work Environments

1. Create profiles for different environments (e.g., "Work", "Home")
2. Each profile has its own provider, settings, and observation preferences
3. Switch profiles from Settings → Profiles

## Tips & Best Practices

### Productivity Tips

1. **Use workspaces per project:** Create separate workspaces for different projects or environments
2. **Import documentation:** Build a knowledge base of your SOPs and manuals
3. **Enable relevant skills:** Activate skill packs for technologies you use daily
4. **Use conversation tags:** Tag conversations for easy retrieval later
5. **Leverage guidance:** Pay attention to the guidance panel for proactive suggestions

### Privacy Tips

1. **Start with privacy mode on:** Enable it during sensitive work
2. **Disable screen observation:** Unless you specifically need it for context
3. **Review privacy settings:** Periodically check your privacy configuration
4. **Use profiles:** Separate work and personal environments

### AI Interaction Tips

1. **Provide context:** Tell the AI what technology you're working with
2. **Be specific:** Detailed questions get better answers
3. **Iterate:** Follow up with clarifying questions
4. **Check confidence:** The guidance panel shows confidence levels
5. **Verify critical advice:** Always verify important recommendations before acting

---

*For system administration, see [Administrator Guide](admin-guide/ADMINISTRATOR_GUIDE.md).*
*For installation guidance, see [Installation Guide](INSTALLATION_GUIDE.md).*
*For technical details, see [Architecture Guide](ARCHITECTURE_GUIDE.md).*
*For support, see [Support Guide](SUPPORT_GUIDE.md).*