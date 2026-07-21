# Quick Start — Wiki Labs AI Copilot v1.0.0

> Get up and running in 5 minutes.

## Prerequisites

- **Windows 10 or later** (64-bit)
- **.NET Desktop Runtime 8.0** (included with the installer)
- **WebView2 Runtime** (included with the installer, or pre-installed on modern Windows)
- **Internet connection** (for AI provider API calls)

## Installation (2 minutes)

### Option A: MSI Installer (Recommended)

1. Download the latest MSI installer from the Wiki Labs download page or your organization's package server
2. Double-click the `.msi` file
3. Follow the installation wizard
4. Click **Finish** when done

### Option B: NSIS Installer

1. Download the latest `.exe` installer
2. Double-click the executable
3. Choose installation directory (default: `%APPDATA%\com.wikilabs.copilot`)
4. Click **Install**
5. Click **Finish** when done

## First Launch (1 minute)

1. Open **Wiki Labs AI Copilot** from Start Menu or Desktop
2. The application opens with the default **Getting Started** workspace
3. You'll see the sidebar with these panels:
   - **Workspace** — Current workspace list
   - **AI Chat** — Main chat interface
   - **Knowledge** — Knowledge management
   - **Skills** — Active skill packs
   - **Settings** — Application settings

## Configure AI Provider (1 minute)

1. Click the **Settings** icon (gear) in the sidebar
2. Navigate to **AI Provider** section
3. Enter your AI provider details:

   | Field | Example | Description |
   |-------|---------|-------------|
   | Name | `OpenAI` | Provider name |
   | Endpoint | `https://api.openai.com/v1` | API URL |
   | API Key | `sk-...` | Your API key (stored securely) |
   | Model | `gpt-4o` | Default model |
   | Max Tokens | `4096` | Response length limit |
   | Context Window | `128000` | Available context tokens |

4. Click **Test Connection** to verify
5. Click **Save**

### Available Providers

| Provider | Endpoint | API Key Required |
|----------|----------|-----------------|
| OpenAI | `https://api.openai.com/v1` | Yes |
| vLLM | `http://localhost:8000/v1` | No |
| Ollama | `http://localhost:11434/v1` | No |

## Create a Workspace (1 minute)

1. Click **New Workspace** in the sidebar
2. Enter a **Workspace Name** (e.g., "Production Server")
3. Enter a **Customer Name** (optional)
4. The workspace is created with default settings

Each workspace maintains its own:
- Chat history
- Knowledge base
- Active skill packs
- Settings

## Start Chatting (1 minute)

1. Select your workspace from the sidebar
2. Click **New Conversation** or start typing in the chat input
3. Type your question (e.g., "How do I check disk usage on Linux?")
4. Press **Enter** or click **Send**
5. The AI responds with evidence-based guidance

## Privacy Controls

By default, all observation features are disabled for privacy:

- **Screen observation**: Off (toggle in Settings → Privacy)
- **Clipboard observation**: Off
- **OCR**: On (can be toggled)
- **Diagnostics**: On (crash reports)
- **Telemetry**: Off

Enable observation features only if needed for your workflow.

## What's Next

- Read the [User Guide](user-guide/USER_GUIDE.md) for detailed feature documentation
- Read the [Architecture Guide](ARCHITECTURE_GUIDE.md) to understand how the system works
- Browse the [Skills](src/skills/) directory to see available skill packs
- Join the [Support channels](SUPPORT_GUIDE.md) if you need help

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift + Enter` | New line |
| `Ctrl + N` | New conversation |
| `Ctrl + W` | New workspace |
| `Ctrl + E` | Export conversation |
| `Ctrl + K` | Clear conversation |

---

*For a complete feature walkthrough, see the [User Guide](user-guide/USER_GUIDE.md).*