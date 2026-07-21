# Administrator Guide — Wiki Labs AI Copilot v1.0.0

> Enterprise deployment, configuration, and system administration.

## Table of Contents

1. [Deployment Overview](#deployment-overview)
2. [System Requirements](#system-requirements)
3. [Installation Methods](#installation-methods)
4. [Configuration Management](#configuration-management)
5. [Settings Profile Management](#settings-profile-management)
6. [Security Configuration](#security-configuration)
7. [Update Management](#update-management)
8. [Monitoring & Logging](#monitoring--logging)
9. [Backup & Recovery](#backup--recovery)
10. [Troubleshooting Deployment](#troubleshooting-deployment)
11. [Performance Tuning](#performance-tuning)

## Deployment Overview

Wiki Labs AI Copilot is a desktop application distributed as MSI and NSIS installers for Windows. Each user runs their own instance with local data storage.

### Deployment Architecture

```
┌─────────────────────────────────────┐
│           User Desktop              │
│  ┌───────────────────────────────┐  │
│  │ Wiki Labs AI Copilot          │  │
│  │  - Tauri v2 + React           │  │
│  │  - SQLite Database            │  │
│  │  - Local Credential Store     │  │
│  │  - AI Provider (remote)       │  │
│  └───────────────────────────────┘  │
│  Data: %APPDATA%\com.wikilabs.copilot│
└─────────────────────────────────────┘
```

### Data Flow

1. User interacts with the desktop application (local)
2. AI requests sent to configured provider over HTTPS
3. All data stored locally in SQLite
4. Credentials stored in OS keychain (Credential Manager on Windows)
5. No cloud sync by default (opt-in future feature)

## System Requirements

### End-User Requirements

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| OS | Windows 10 64-bit | Windows 11 64-bit |
| CPU | Dual-core 2.0 GHz | Quad-core 2.5 GHz |
| RAM | 4 GB | 8 GB |
| Disk | 2 GB free | 5 GB free |
| .NET | .NET Desktop Runtime 8.0 | .NET Desktop Runtime 8.0 (latest) |
| WebView2 | WebView2 Runtime | WebView2 Runtime (pre-installed) |

### AI Provider Requirements

| Provider | Min Requirements | Notes |
|----------|-----------------|-------|
| OpenAI | Internet connection, valid API key | Per-token billing |
| vLLM | GPU (recommended), 16 GB RAM | Self-hosted |
| Ollama | GPU (recommended), 8 GB RAM | Local inference |

## Installation Methods

### MSI Installation

```powershell
# Silent install (per-user)
msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" INSTALLMODE="currentUser" /quiet

# Silent install (per-machine, requires admin)
msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" INSTALLMODE="perMachine" /quiet

# Verify installation
Get-Item "HKCU:\Software\com.wikilabs.copilot" 2>$null
```

### NSIS Installation

```powershell
# Silent install
Start-Process ".\wikilabs-ai-copilot-1.0.0-setup.exe" -ArgumentList "/S" -Wait

# Verify installation
Test-Path "$env:APPDATA\com.wikilabs.copilot\wikilabs.db"
```

### Group Policy Deployment

1. Copy MSI to network share: `\\fileserver\software\wikilabs-ai-copilot\`
2. Create Group Policy Object (GPO) for Software Installation
3. Assign package to target Organizational Unit (OU)
4. Users receive the application at next logon or `gpupdate /force`

### SCCM Deployment

1. Create application in SCCM Console
2. Set install command: `msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" /quiet`
3. Set detection method: file exists at `%APPDATA%\com.wikilabs.copilot`
4. Deploy to target collection

## Configuration Management

### Settings File

Settings are stored at:
```
%APPDATA%\com.wikilabs.copilot\settings.json
```

The settings file is a JSON document managed by the application. Do not edit it manually while the application is running.

### Schema Version

The settings file uses schema version `1.0.0`. The application validates settings on load and migrates older schemas automatically.

### Settings Sections

The settings file contains 8 configuration sections (see [Release Notes](RELEASE_NOTES.md#configuration-settings) for full details).

### Profile Management

The application supports named profiles for managing different configuration sets:

```json
{
  "current_profile": "default",
  "profiles": [
    {
      "name": "default",
      "display_name": "Default",
      "settings": { ... },
      "created_at": "2026-07-21T10:00:00Z",
      "updated_at": "2026-07-21T10:00:00Z"
    }
  ]
}
```

Profiles can be managed through the Settings UI:
- Create new profiles
- Switch between profiles
- Import/Export profile settings as JSON
- Backup and restore configurations

## Security Configuration

### Credential Storage

On Windows, credentials (API keys) are stored using Windows Credential Manager (DPAPI). The application falls back to encrypted local storage if Credential Manager is unavailable.

| Setting | Default | Description |
|---------|---------|-------------|
| `use_credential_manager` | `true` | Use Windows Credential Manager |
| `local_encryption_enabled` | `true` | Fallback to local AES-256-GCM encryption |
| `encryption_algorithm` | `aes-256-gcm` | Encryption algorithm |
| `auto_lock_minutes` | `30` | Auto-lock after inactivity |
| `pin_protection_enabled` | `false` | Require PIN for credential access |

### Privacy Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `screen_observation_enabled` | `false` | Allow screen content capture |
| `ocr_enabled` | `true` | Allow OCR on captured content |
| `clipboard_observation_enabled` | `false` | Allow clipboard observation |
| `diagnostics_enabled` | `true` | Allow crash reports |
| `telemetry_enabled` | `false` | Allow analytics |
| `privacy_mode` | `false` | One-click disable all observation |

### TLS Requirements

- All external API communication uses HTTPS/TLS 1.2+
- TLS certificates validated by system trust store
- No plaintext credential transmission

## Update Management

### Auto-Update Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `auto_check_enabled` | `true` | Automatically check for updates |
| `channel` | `stable` | Update channel |
| `show_dialog` | `true` | Show update notification dialog |
| `allow_deferral` | `true` | Allow user to defer updates |

### Update Channels

| Channel | Description |
|---------|-------------|
| `stable` | Production releases only |
| `preview` | Release candidates and beta builds |
| `internal` | Internal testing builds |

### Manual Update Process

1. Download new installer from release page
2. Run installer — detects existing installation
3. Click **Upgrade** to upgrade in place
4. Application restarts with new version

## Monitoring & Logging

### Log Locations

```
%APPDATA%\com.wikilabs.copilot\logs\
  wikilabs-copilot.log          # Main log file (daily rotation)
  wikilabs-copilot.log.1        # Previous day
  wikilabs-copilot.log.2        # Two days ago
```

### Log Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `level` | `info` | Log level (trace/debug/info/warn/error) |
| `file_logging` | `true` | Write logs to file |
| `max_log_size_mb` | `10` | Max log file size before rotation |
| `max_log_files` | `3` | Number of rotated files to keep |
| `structured_logging` | `true` | Use JSON structured logging |

### Log Format

Logs use structured JSON format with fields:
- `timestamp` — ISO 8601 timestamp
- `level` — Log level
- `target` — Module name
- `message` — Log message
- `event` — Optional structured data
- `stack_trace` — Optional stack trace

### Diagnostic Package

The application can generate a diagnostic package for support:
- Version and platform information
- Redacted settings report
- Log file metadata (not contents)
- Validation errors and warnings
- System information

## Backup & Recovery

### Data Backup

All user data is stored in a single directory:
```
%APPDATA%\com.wikilabs.copilot\
  wikilabs.db          # SQLite database (workspaces, chat, knowledge)
  settings.json        # Settings and profiles
  credentials.enc      # Encrypted credential store
  logs\                # Log files
  backups\             # Settings backups
  crash\               # Crash reports
```

**Backup recommended:**
1. Close the application
2. Copy `%APPDATA%\com.wikilabs.copilot\` to a backup location
3. Restore by copying back after reinstall

### Database Backup

The SQLite database can be backed up while the application is running using SQLite's online backup API, or by stopping the app first for a file copy.

### Settings Backup

Settings are automatically backed up to `%APPDATA%\com.wikilabs.copilot\backups\` on each save. Previous versions are retained for rollback.

### Crash Recovery

When the application crashes:
1. A crash report is saved to `%APPDATA%\com.wikilabs.copilot\crash\last_crash.json`
2. On next launch, the application checks for previous crashes
3. The user is notified of previous crash data
4. Crash reports can be cleared from the error management UI

## Troubleshooting Deployment

### Application Won't Start

1. Check Windows Event Viewer for application errors
2. Verify WebView2 is installed: `Get-AppxPackage *WebView2*`
3. Verify .NET Runtime: `Get-ItemProperty "HKLM:\SOFTWARE\dotnet\Setup\InstalledVersions\x64\desktop"`
4. Check log files at `%APPDATA%\com.wikilabs.copilot\logs\`

### Database Corruption

1. Close the application
2. Backup the database file
3. Delete the database file
4. Restart the application — it creates a fresh database
5. Re-configure settings from backup

### Credential Issues

1. Verify Windows Credential Manager is accessible
2. Clear stored credentials: delete `%APPDATA%\com.wikilabs.copilot\credentials.enc`
3. Re-enter API key in Settings
4. Alternatively, disable Credential Manager and use local encryption

### AI Provider Connection Failed

1. Verify network connectivity
2. Check API key is valid and not expired
3. Test connection from Settings UI
4. Check provider status page (e.g., OpenAI status)
5. Verify no firewall/proxy is blocking HTTPS traffic

## Performance Tuning

### Database Performance

- SQLite performance is excellent for the expected data volume
- For large knowledge bases, consider splitting workspaces
- Run `VACUUM` periodically to optimize the database:
  ```sql
  VACUUM;
  ```

### Memory Usage

- Baseline memory: ~50 MB
- With observation enabled: +20-50 MB
- With large knowledge base loaded: +100-200 MB
- Token budget settings can reduce memory for conversation context

### Log Rotation

For high-traffic environments with many log entries:
- Increase `max_log_size_mb` (default 10 MB)
- Reduce `max_log_files` (default 3)
- Set `level` to `warn` or `error` to reduce log volume

---

*For technical architecture details, see [Architecture Guide](ARCHITECTURE_GUIDE.md).*
*For security configuration, see [Security Guide](SECURITY_GUIDE.md).*
*For troubleshooting, see [Troubleshooting Guide](TROUBLESHOOTING.md).*