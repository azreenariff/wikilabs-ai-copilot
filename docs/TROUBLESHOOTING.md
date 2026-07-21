# Troubleshooting Guide — Wiki Labs AI Copilot v1.0.0

> Common issues, diagnostics, and recovery procedures.

## Table of Contents

1. [Quick Diagnostics](#quick-diagnostics)
2. [Installation Issues](#installation-issues)
3. [Application Startup Issues](#application-startup-issues)
4. [AI Provider Issues](#ai-provider-issues)
5. [Workspace Issues](#workspace-issues)
6. [Chat & Conversation Issues](#chat--conversation-issues)
7. [Knowledge Base Issues](#knowledge-base-issues)
8. [Skill Pack Issues](#skill-pack-issues)
9. [Performance Issues](#performance-issues)
10. [Security & Credential Issues](#security--credential-issues)
11. [Log Analysis](#log-analysis)
12. [Recovery Procedures](#recovery-procedures)
13. [Support Escalation](#support-escalation)

## Quick Diagnostics

### System Health Check

Run this PowerShell script to check overall system health:

```powershell
$issues = @()

# Check WebView2
$webView2 = Get-AppxPackage *WebView2* -ErrorAction SilentlyContinue
if (-not $webView2) { $issues += "WebView2 Runtime not installed" }

# Check .NET Runtime
$dotnet = Get-ItemProperty "HKLM:\SOFTWARE\dotnet\Setup\InstalledVersions\x64\desktop" -ErrorAction SilentlyContinue
if (-not $dotnet) { $issues += ".NET Desktop Runtime 8.0 not installed" }

# Check app data directory
$appData = "$env:APPDATA\com.wikilabs.copilot"
if (Test-Path $appData) {
    $issues += "Application data directory exists: $appData"
    # Check database
    if (Test-Path "$appData\wikilabs.db") {
        $dbSize = (Get-Item "$appData\wikilabs.db").Length
        $issues += "Database size: $([math]::Round($dbSize/1MB, 2)) MB"
    }
    # Check logs
    $logCount = (Get-ChildItem "$appData\logs" -ErrorAction SilentlyContinue).Count
    $issues += "Log files: $logCount"
} else {
    $issues += "Application data directory not found"
}

# Check disk space
$disk = Get-PSDrive $env:SystemDrive
$freePercent = [math]::Round($disk.Free / $disk.Used * 100, 1)
if ($freePercent -lt 10) { $issues += "Low disk space: ${freePercent}% free" }

# Display results
if ($issues.Count -gt 0) {
    Write-Host "Diagnostics Results ($($issues.Count) findings):" -ForegroundColor Cyan
    $issues | ForEach-Object { Write-Host "  • $_" }
} else {
    Write-Host "All checks passed." -ForegroundColor Green
}
```

### Log File Locations

| Component | Location |
|-----------|----------|
| Application logs | `%APPDATA%\com.wikilabs.copilot\logs\wikilabs-copilot.log` |
| Error log | `%APPDATA%\com.wikilabs.copilot\crash\error_log.jsonl` |
| Crash report | `%APPDATA%\com.wikilabs.copilot\crash\last_crash.json` |
| Settings | `%APPDATA%\com.wikilabs.copilot\settings.json` |
| Database | `%APPDATA%\com.wikilabs.copilot\wikilabs.db` |

### Viewing Recent Errors

```powershell
# View last 20 lines of log
Get-Content "$env:APPDATA\com.wikilabs.copilot\logs\wikilabs-copilot.log" -Tail 20

# Check for crash reports
if (Test-Path "$env:APPDATA\com.wikilabs.copilot\crash\last_crash.json") {
    Get-Content "$env:APPDATA\com.wikilabs.copilot\crash\last_crash.json"
}
```

## Installation Issues

### Problem: Installer Won't Start

| Cause | Solution |
|-------|----------|
| File blocked by Windows SmartScreen | Right-click → Properties → Check "Unblock" |
| Insufficient disk space | Free at least 2 GB of disk space |
| Corrupted installer | Re-download the installer from the release page |
| Antivirus blocking | Add exclusion for `%APPDATA%\com.wikilabs.copilot` |

### Problem: "Another Version Already Installed"

The installer detects an existing installation.

**Solution:**
1. Uninstall the existing version via Settings → Apps
2. Restart the computer
3. Run the new installer

### Problem: MSI Install Fails with Error Code

| Error Code | Meaning | Solution |
|------------|---------|----------|
| `1603` | Fatal error during install | Check Windows Event Viewer for details |
| `1618` | Another installation in progress | Wait for the other installation to complete |
| `1642` | Restart required | Restart the computer and retry |
| `3010` | Restart required but install succeeded | Restart the computer |

## Application Startup Issues

### Problem: Application Won't Launch

| Cause | Diagnostic | Solution |
|-------|-----------|----------|
| WebView2 missing | Run: `Get-AppxPackage *WebView2*` | Install WebView2 Runtime |
| .NET Runtime missing | Check: `HKLM:\SOFTWARE\dotnet\Setup\` | Install .NET Desktop Runtime 8.0 |
| Database locked | Check: another instance running | Close all instances via Task Manager |
| Corrupt settings | Check: `settings.json` syntax | Rename `settings.json` to `settings.json.bak`, restart |
| Permissions issue | Check: Event Viewer | Run as Administrator (per-machine install) |

### Problem: Application Crashes on Startup

1. **Check crash report:**
   ```powershell
   Get-Content "$env:APPDATA\com.wikilabs.copilot\crash\last_crash.json"
   ```

2. **Check log files:**
   ```powershell
   Get-Content "$env:APPDATA\com.wikilabs.copilot\logs\wikilabs-copilot.log" -Tail 50
   ```

3. **Common crash causes:**
   - Database corruption → [Database Recovery](#problem-database-corruption)
   - Invalid settings → [Reset Settings](#problem-corrupt-settings)
   - Missing WebView2 → Install WebView2 Runtime

4. **Reset to defaults:**
   ```cmd
   ren "%APPDATA%\com.wikilabs.copilot\settings.json" "settings.json.bak"
   ```

### Problem: Window Opens Blank/Black

| Cause | Solution |
|-------|----------|
| WebView2 not rendering | Restart WebView2: `Stop-Process -Name "MicrosoftEdgeWebView2" -Force` |
| GPU acceleration issue | Disable GPU in tauri config: set `transparent` to `false` |
| Insufficient display resolution | Resize window to ≥1024×768 |

## AI Provider Issues

### Problem: "Provider Connection Failed"

| Cause | Diagnostic | Solution |
|-------|-----------|----------|
| Invalid API key | Check key format and expiration | Regenerate API key from provider dashboard |
| Wrong endpoint URL | Verify URL format | Check provider documentation for correct URL |
| Network connectivity | `Test-NetConnection <endpoint>` | Check proxy/firewall settings |
| Provider outage | Check provider status page | Wait for provider to recover |

**Step-by-step fix:**
1. Open Settings → AI Provider
2. Click **Test Connection**
3. If it fails, verify:
   - Endpoint URL is correct (e.g., `https://api.openai.com/v1`)
   - API key is valid (no trailing spaces)
   - Model name exists for the provider
   - Network allows outbound HTTPS to the endpoint

### Problem: "API Key Required"

The provider requires an API key but none is configured.

**Solution:**
1. Open Settings → AI Provider
2. Enter a valid API key for your provider
3. Click **Test Connection**
4. Click **Save**

### Problem: Slow AI Responses

| Cause | Solution |
|-------|----------|
| Local provider (vLLM/Ollama) | Use a faster model or upgrade hardware |
| Large context window | Reduce context window size in settings |
| Network latency (cloud provider) | Use a closer endpoint or local provider |
| Low model capacity | Try a larger model |
| System resource constraints | Close other applications, check CPU/RAM |

### Problem: Streaming Not Working

| Cause | Solution |
|-------|----------|
| Provider doesn't support streaming | Check provider feature support |
| Network issue | Verify connectivity to provider |
| Application placeholder | Check release notes for streaming support status |

## Workspace Issues

### Problem: Cannot Create Workspace

| Cause | Solution |
|-------|----------|
| Database locked | Close all application instances |
| Invalid workspace name | Use alphanumeric characters and spaces only |
| Database full | Check disk space, [run VACUUM](#recovery-procedures) |

### Problem: Workspace Not Switching

| Cause | Solution |
|-------|----------|
| Database corruption | [Reset workspace configuration](#problem-corrupt-workspaces) |
| Settings issue | [Reset settings](#problem-corrupt-settings) |
| Application state | Close and reopen application |

### Problem: Corrupt Workspaces

1. Close the application
2. Backup the database:
   ```cmd
   copy "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "%APPDATA%\com.wikilabs.copilot\wikilabs.db.bak"
   ```
3. Open database with SQLite tool:
   ```cmd
   sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db"
   ```
4. Check workspace table:
   ```sql
   SELECT * FROM workspaces;
   ```
5. If workspaces are corrupt, delete and recreate:
   ```sql
   DELETE FROM workspaces;
   .quit
   ```
6. Restart the application

## Chat & Conversation Issues

### Problem: Messages Not Sending

| Cause | Solution |
|-------|----------|
| AI provider not configured | [Configure AI provider](#problem-provider-connection-failed) |
| Network disconnected | Check network connectivity |
| API key expired | Regenerate API key |
| Message too long | Reduce message length |

### Problem: AI Response Incorrect

| Cause | Solution |
|-------|----------|
| No workspace context | Add workspace and set technology stack |
| No knowledge loaded | Import relevant knowledge documents |
| Wrong model selected | Switch to a more capable model |
| Insufficient context | Add manual context via the conversation |

### Problem: Conversation History Missing

| Cause | Solution |
|-------|----------|
| Workspace deleted | Cannot recover deleted workspaces |
| Clear history action | Re-engage with AI for fresh conversation |
| Database corruption | [Recover database](#problem-database-corruption) |

## Knowledge Base Issues

### Problem: Knowledge Search Returns No Results

| Cause | Solution |
|-------|----------|
| No documents imported | Import knowledge documents |
| Wrong workspace | Switch to the workspace with knowledge |
| Documents not indexed | Re-import the knowledge documents |
| Embedding failure | Check logs for embedding errors |

### Problem: Knowledge Import Failed

| Cause | Solution |
|-------|----------|
| Invalid `.wkl` format | Verify the archive format |
| Document too large | Split into smaller documents |
| Disk full | Free disk space |
| Encoding issue | Ensure documents are UTF-8 encoded |

### Problem: Embedding Performance Slow

| Cause | Solution |
|-------|----------|
| Large documents | Split documents into smaller chunks |
| No GPU available | Use local embedding model (CPU-optimized) |
| Many documents indexed | Index documents in batches |

## Skill Pack Issues

### Problem: Skill Not Activating

| Cause | Solution |
|-------|----------|
| Skill not installed | Install the skill pack |
| Skill disabled | Enable skill in Skills panel |
| Missing dependencies | Check skill dependencies and install them |
| Version incompatibility | Update skill pack to compatible version |

### Problem: Skill Provides Irrelevant Guidance

| Cause | Solution |
|-------|----------|
| Wrong workspace context | Set correct technology stack in workspace |
| Skill not detecting context | Manually activate the skill |
| Outdated skill pack | Update to latest version |
| Multiple conflicting skills | Disable conflicting skills |

## Performance Issues

### Problem: High Memory Usage

| Cause | Solution |
|-------|----------|
| Large knowledge base | Reduce number of indexed documents |
| Long conversation history | Archive old conversations |
| Multiple workspaces | Close unused workspaces |
| Observation engine running | Disable unnecessary observation features |

### Problem: High CPU Usage

| Cause | Solution |
|-------|----------|
| AI provider processing | Check provider server load |
| Embedding generation | Embeddings run on import, not runtime |
| Observation engine active | Reduce observation scope |
| Background processes | Check for runaway processes in Task Manager |

### Problem: Slow Application Startup

| Cause | Solution |
|-------|----------|
| Large database | [Run VACUUM](#recovery-procedures) |
| Many workspaces | Reduce number of workspaces |
| Startup checks | Disable auto-update checks if needed |

## Security & Credential Issues

### Problem: API Key Not Saved Securely

| Cause | Solution |
|-------|----------|
| Credential Manager unavailable | Enable local encryption in Security settings |
| Encryption disabled | Enable `local_encryption_enabled` in settings |
| Corrupt credential store | Delete `credentials.enc` and re-enter API key |

### Problem: Privacy Mode Not Working

| Cause | Solution |
|-------|----------|
| Settings not saved | Click Save after enabling privacy mode |
| Application restarted with old settings | Re-enable privacy mode |
| Configuration override | Check for configuration file overrides |

### Problem: Cannot Access Saved Credentials

| Cause | Solution |
|-------|----------|
| PIN wrong | Reset PIN in Security settings |
| System fingerprint changed | Hardware change — delete `credentials.enc` and re-enter |
| Credential file corrupt | Delete `credentials.enc` and re-enter credentials |

## Log Analysis

### Log Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| `trace` | Most detailed | Debugging specific issues |
| `debug` | Detailed debugging | Development troubleshooting |
| `info` | General operations | Normal operation logging |
| `warn` | Warning conditions | Investigating degraded performance |
| `error` | Error conditions | Investigating failures |

### Common Log Patterns

```json
// AI Request
{
  "target": "main",
  "level": "INFO",
  "message": "Testing AI provider connection",
  "provider": "OpenAI",
  "endpoint": "https://api.openai.com/v1"
}

// AI Response
{
  "target": "ai.provider",
  "level": "INFO",
  "message": "Provider connection verified"
}

// Error
{
  "target": "main",
  "level": "ERROR",
  "message": "Provider connection failed",
  "error": "connection refused"
}

// Database
{
  "target": "main",
  "level": "INFO",
  "message": "Database initialized"
}
```

### Extracting Errors from Logs

```powershell
# Find all error entries
Select-String -Path "$env:APPDATA\com.wikilabs.copilot\logs\wikilabs-copilot.log" -Pattern '"level":"ERROR"'

# Find recent warnings
Get-Content "$env:APPDATA\com.wikilabs.copilot\logs\wikilabs-copilot.log" |
  Select-String -Pattern '"level":"WARN"'
```

### Redacted Fields

Sensitive fields are automatically redacted in logs:
- `password` → `PASSWORD_REDACTED`
- `secret` → `SECRET_REDACTED`
- `token` → `TOKEN_REDACTED`
- `api_key` → `API_KEY_REDACTED`
- `authorization` → `AUTH_REDACTED`

## Recovery Procedures

### Database Recovery

1. **Close the application completely**
2. **Backup the database:**
   ```cmd
   copy "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "%APPDATA%\com.wikilabs.copilot\wikilabs.db.bak"
   ```
3. **Optimize with VACUUM:**
   ```cmd
   sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "VACUUM;"
   ```
4. **Verify database integrity:**
   ```cmd
   sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "PRAGMA integrity_check;"
   ```
5. **Restart the application**

### Reset Settings to Defaults

1. Close the application
2. Rename settings file:
   ```cmd
   ren "%APPDATA%\com.wikilabs.copilot\settings.json" "settings.json.bak"
   ```
3. Restart the application — fresh defaults are created
4. Re-configure the AI provider and settings

### Reset to Fresh State

1. **Close the application**
2. **Backup all data:**
   ```cmd
   xcopy "%APPDATA%\com.wikilabs.copilot" "%APPDATA%\com.wikilabs.copilot.backup" /E /H /I
   ```
3. **Delete application data:**
   ```cmd
   rmdir /S /Q "%APPDATA%\com.wikilabs.copilot"
   ```
4. **Restart the application** — creates fresh state
5. **Restore from backup** if needed:
   ```cmd
   xcopy "%APPDATA%\com.wikilabs.copilot.backup\*" "%APPDATA%\com.wikilabs.copilot\" /E /H /I
   ```

### Log Cleanup

```powershell
# Rotate old log files (keep last 7 days)
$logDir = "$env:APPDATA\com.wikilabs.copilot\logs"
Get-ChildItem $logDir -Filter "*.log*" |
  Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } |
  Remove-Item

# Check current log sizes
Get-ChildItem $logDir |
  Format-Table Name, @{L='Size(MB)';E={[math]::Round($_.Length/1MB,2)}}, LastWriteTime
```

### Generating Diagnostic Package

The application includes a built-in diagnostic package generator. Access it through the application's error management UI. The diagnostic package includes:

- Application version and platform info
- Redacted settings report
- Log file metadata
- Validation errors and warnings
- System information

Alternatively, use the `generate_diagnostics` function available in the application for programmatic access.

## Support Escalation

### When to Escalate

Escalate to support when:
- The application crashes repeatedly
- Data corruption cannot be resolved with recovery procedures
- AI provider connectivity issues persist after troubleshooting
- Performance issues cannot be resolved with tuning
- Security concerns are identified

### Support Channels

| Channel | Contact | Response Time |
|---------|---------|--------------|
| GitHub Issues | https://github.com/wikilabs/wikilabs-ai-copilot/issues | 48 hours |
| Email Support | support@wikilabs.com | 24 hours |
| Enterprise Support | Dedicated support team | 4 hours |

### Escalation Information

When escalating, provide:
1. Application version (check Settings → About)
2. Operating system version
3. Log files from `%APPDATA%\com.wikilabs.copilot\logs\`
4. Diagnostic package (from Settings → Diagnostics)
5. Steps to reproduce the issue
6. Expected vs. actual behavior

---

*For installation guidance, see [Installation Guide](INSTALLATION_GUIDE.md).*
*For configuration details, see [Administrator Guide](admin-guide/ADMINISTRATOR_GUIDE.md).*
*For support channels, see [Support Guide](SUPPORT_GUIDE.md).*