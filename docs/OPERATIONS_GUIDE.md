# Operations Guide — Wiki Labs AI Copilot v1.0.0

> Monitoring, logging, maintenance, and backup procedures.

## Table of Contents

1. [Operations Overview](#operations-overview)
2. [Monitoring](#monitoring)
3. [Logging](#logging)
4. [Maintenance](#maintenance)
5. [Backup & Recovery](#backup--recovery)
6. [Performance Monitoring](#performance-monitoring)
7. [Error Management](#error-management)
8. [Health Checks](#health-checks)
9. [Capacity Planning](#capacity-planning)
10. [Operations Checklist](#operations-checklist)

## Operations Overview

Wiki Labs AI Copilot operates as a standalone desktop application with local data storage and remote AI provider communication. Operations focus on local system health, application performance, data integrity, and log management.

### Operations Scope

| Domain | Scope | Responsibility |
|--------|-------|----------------|
| Application Health | Startup, stability, performance | System admin / End user |
| Data Integrity | Database, knowledge base | System admin / End user |
| Log Management | Log rotation, analysis | System admin |
| Backup | Settings, database, credentials | System admin / End user |
| Updates | Version upgrades, patches | System admin / End user |
| Security | Encryption, credentials, privacy | System admin |

## Monitoring

### Application Health Monitoring

#### Key Health Indicators

| Indicator | Source | Check Method |
|-----------|--------|-------------|
| Application running | Process list | Task Manager / `Get-Process` |
| Database accessible | SQLite | Query `workspaces` table |
| Settings loadable | File system | Parse `settings.json` |
| Log file written | File system | Check recent log entries |
| Disk space available | OS | `Get-PSDrive` |

#### PowerShell Health Check

```powershell
# Check application health
$appData = "$env:APPDATA\com.wikilabs.copilot"

# Process check
$process = Get-Process -Name "wikilabs*" -ErrorAction SilentlyContinue
$running = $null -ne $process

# Database check
$dbExists = Test-Path "$appData\wikilabs.db"
$dbValid = if ($dbExists) {
    try {
        # Use sqlite3 CLI if available
        sqlite3 "$appData\wikilabs.db" "SELECT 1 FROM workspaces LIMIT 1;" 2>$null
        $true
    } catch { $false }
} else { $false }

# Settings check
$settingsExists = Test-Path "$appData\settings.json"

# Disk space check
$disk = Get-PSDrive $env:SystemDrive
$freeGB = [math]::Round($disk.Free / 1GB, 2)

# Output health report
Write-Host "=== Wiki Labs AI Copilot Health Report ===" -ForegroundColor Cyan
Write-Host "Application running: $running"
Write-Host "Database valid: $dbValid"
Write-Host "Settings exist: $settingsExists"
Write-Host "Disk space free: ${freeGB} GB"

if ($freeGB -lt 1) {
    Write-Host "WARNING: Low disk space!" -ForegroundColor Red
}
```

### AI Provider Monitoring

| Check | Method | Expected |
|-------|--------|----------|
| Provider reachable | `Test-NetConnection <endpoint>` | TCP connection succeeds |
| API key valid | Settings → Test Connection | 200 response |
| Model available | Settings → Test Connection | Model responds |
| Response time | Settings → Test Connection | < 2 seconds |

### Network Monitoring

| Check | Method | Tool |
|-------|--------|------|
| HTTPS connectivity | `Test-NetConnection` | PowerShell |
| DNS resolution | `Resolve-DnsName` | PowerShell |
| Proxy configuration | `netsh winhttp show proxy` | Command line |
| Certificate validity | Settings → Test Connection | App UI |

## Logging

### Log File Structure

```
%APPDATA%\com.wikilabs.copilot\logs\
├── wikilabs-copilot.log       # Current day's log
├── wikilabs-copilot.log.1     # Previous day
├── wikilabs-copilot.log.2     # Two days ago
└── wikilabs-copilot.log.3     # Three days ago (max retention)
```

### Log Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `level` | `info` | Minimum log level |
| `file_logging` | `true` | Write logs to file |
| `max_log_size_mb` | `10` | Max log file size (MB) |
| `max_log_files` | `3` | Number of rotated files |
| `structured_logging` | `true` | JSON structured format |

### Log Format

Logs use structured JSON format:

```json
{
  "timestamp": "2026-07-21T10:30:00Z",
  "level": "INFO",
  "target": "main",
  "message": "Application started",
  "event": {
    "version": "1.0.0",
    "platform": "windows-x64"
  }
}
```

### Log Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| `trace` | Most detailed | Deep debugging |
| `debug` | Detailed debugging | Development troubleshooting |
| `info` | Normal operations | Production logging |
| `warn` | Warning conditions | Investigating issues |
| `error` | Error conditions | Failure investigation |

### Viewing Logs

```powershell
# View last 50 lines of current log
Get-Content "$env:APPDATA\com.wikilabs.copilot\logs\wikilabs-copilot.log" -Tail 50

# View all error entries
Select-String -Path "$env:APPDATA\com.wikilabs.copilot\logs\*.log" -Pattern '"level":"ERROR"'

# View all warning entries from today
Get-Content "$env:APPDATA\com.wikilabs.copilot\logs\*.log" |
  Where-Object { $_ -match '"level":"WARN"' }

# Monitor log file in real-time
Get-Content "$env:APPDATA\com.wikilabs.copilot\logs\wikilabs-copilot.log" -Wait
```

### Log Analysis

#### Common Error Patterns

| Pattern | Meaning | Action |
|---------|---------|--------|
| `provider connection failed` | AI provider unreachable | Check network and API key |
| `database` + `corrupt` | Database integrity issue | [Run recovery](#database-corruption) |
| `encryption` + `error` | Credential store issue | Reset credentials |
| `panic` + `unreachable` | Unhandled panic | Collect crash report |
| `out of memory` | System memory pressure | Reduce context size |

#### Log Rotation Management

```powershell
# Check current log sizes
$logDir = "$env:APPDATA\com.wikilabs.copilot\logs"
Get-ChildItem $logDir |
  Format-Table Name, @{L='Size(MB)';E={[math]::Round($_.Length/1MB,2)}}

# Clean logs older than 7 days
Get-ChildItem $logDir -Filter "*.log*" |
  Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } |
  Remove-Item

# Count total log entries
$logFile = "$env:APPDATA\com.wikilabs.copilot\logs\wikilabs-copilot.log"
$lineCount = (Get-Content $logFile).Count
Write-Host "Total log entries: $lineCount"
```

## Maintenance

### Daily Operations

| Check | Frequency | Method |
|-------|-----------|--------|
| Application running | Continuous | Auto |
| Error rate | Hourly | Log analysis |
| Disk space | Daily | System monitor |
| Log rotation | Daily (auto) | Built-in rotation |

### Weekly Operations

| Task | Frequency | Method |
|------|-----------|--------|
| Review error logs | Weekly | `Select-String` for errors |
| Check update availability | Weekly | Settings → Update |
| Verify backup integrity | Weekly | Test restore |
| Review knowledge base | Weekly | Check import status |

### Monthly Operations

| Task | Frequency | Method |
|------|-----------|--------|
| Database VACUUM | Monthly | SQLite command |
| Full data backup | Monthly | Manual or scheduled |
| Review privacy settings | Monthly | Settings audit |
| Log archive | Monthly | Compress and store |

### Database Maintenance

#### VACUUM (Monthly Recommended)

```cmd
# Optimize the SQLite database
sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "VACUUM;"

# Check database integrity
sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "PRAGMA integrity_check;"
```

#### Database Statistics

```cmd
# Get table sizes
sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db" ".tables"
sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db" ".schema"

# Get row counts
sqlite3 "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "
  SELECT 'workspaces' AS table_name, COUNT(*) AS count FROM workspaces
  UNION ALL SELECT 'chat_messages', COUNT(*) FROM chat_messages
  UNION ALL SELECT 'knowledge_documents', COUNT(*) FROM knowledge_documents
  UNION ALL SELECT 'knowledge_chunks', COUNT(*) FROM knowledge_chunks
  UNION ALL SELECT 'audit_log', COUNT(*) FROM audit_log;
"
```

### Log File Management

#### Log Retention Policy

| Setting | Default | Recommended |
|---------|---------|-------------|
| Max log file size | 10 MB | 10-50 MB (high-traffic) |
| Max log files | 3 | 3-7 |
| Log level | `info` | `warn` (production), `debug` (development) |

#### Log Archive

```powershell
# Archive old logs
$logDir = "$env:APPDATA\com.wikilabs.copilot\logs"
$archiveDir = "$env:TEMP\wikilabs-logs-archive"
New-Item -ItemType Directory -Force -Path $archiveDir

# Compress log files
Get-ChildItem $logDir -Filter "*.log*" |
  Compress-Archive -DestinationPath "$archiveDir\wikilabs-logs-$(Get-Date -Format 'yyyyMMdd').zip"
```

## Backup & Recovery

### Backup Strategy

| Data Type | Frequency | Method | Retention |
|-----------|-----------|--------|-----------|
| Settings | Every save | Auto backup | Last 5 versions |
| Database | Daily | Manual/scheduled | 30 days |
| Knowledge base | After import | Manual | Until re-import |
| Logs | On rotation | Auto | 3 files |
| Crash reports | On crash | Auto | Until cleared |

### Backup Procedures

#### Full Backup

```powershell
# Close the application
Stop-Process -Name "wikilabs*" -ErrorAction SilentlyContinue

# Create backup
$source = "$env:APPDATA\com.wikilabs.copilot"
$backupPath = "$env:APPDATA\com.wikilabs.copilot.backup.$(Get-Date -Format 'yyyyMMdd')"
Copy-Item $source $backupPath -Recurse -Force

Write-Host "Backup created at: $backupPath"
```

#### Incremental Backup (Settings Only)

Settings are automatically backed up on each save. Previous versions are kept in `%APPDATA%\com.wikilabs.copilot\backups\`.

### Recovery Procedures

#### Database Recovery

1. Close the application
2. Backup current database
3. Run `VACUUM` to optimize
4. Run `PRAGMA integrity_check;` to verify
5. Restart the application

#### Settings Recovery

1. Close the application
2. Navigate to `%APPDATA%\com.wikilabs.copilot\backups\`
3. Copy the desired backup to `settings.json`
4. Restart the application

#### Full Data Recovery

1. Close the application
2. Navigate to backup directory
3. Copy backup contents to `%APPDATA%\com.wikilabs.copilot\`
4. Restart the application

## Performance Monitoring

### Key Performance Metrics

| Metric | Threshold | Action |
|--------|-----------|--------|
| Application startup | < 5 seconds | Acceptable |
| AI response time | < 10 seconds | Check provider |
| Database query time | < 100 ms | Acceptable |
| Memory usage | < 200 MB | Investigate if > 200 MB |
| Disk usage | < 500 MB | Clean logs if > 500 MB |

### Performance Tuning

| Setting | Current | Recommended |
|---------|---------|-------------|
| Log level | `info` | `warn` for production |
| Max tokens | 4096 | Adjust based on model |
| Context window | 128000 | Reduce for memory-constrained systems |
| Observation features | User-controlled | Disable unused features |
| Log file size | 10 MB | Increase for high-traffic |

### Performance Diagnostics

```powershell
# Check application resource usage
$process = Get-Process -Name "wikilabs*" -ErrorAction SilentlyContinue
if ($process) {
    Write-Host "Memory: $([math]::Round($process.WorkingSet64/1MB, 2)) MB"
    Write-Host "CPU: $($process.CPU) seconds"
}

# Check disk usage of application data
$appData = "$env:APPDATA\com.wikilabs.copilot"
$totalSize = (Get-ChildItem $appData -Recurse -ErrorAction SilentlyContinue |
  Measure-Object -Property Length -Sum).Sum
Write-Host "Total data size: $([math]::Round($totalSize/1MB, 2)) MB"
```

## Error Management

### Error Severity Classification

| Severity | Action | Examples |
|----------|--------|----------|
| **Warning** | Log and continue | Non-critical config issue |
| **Degraded** | Log + alert | Reduced functionality |
| **Error** | Log + alert + retry | Temporary failure |
| **Fatal** | Crash report + shutdown | Unrecoverable state |

### Error Recovery Strategies

| Strategy | When to Use | Example |
|----------|------------|---------|
| **Retry** | Transient failures | Network timeout, provider busy |
| **Fallback** | Primary unavailable | Provider down → use alternative |
| **UserPrompt** | Needs user input | Invalid API key |
| **Shutdown** | Unrecoverable | Database corruption |

### Crash Report Management

Crash reports are stored at `%APPDATA%\com.wikilabs.copilot\crash\last_crash.json`.

**Management:**
- Reports are overwritten on each crash
- Reports include diagnostic context (redacted)
- Reports can be shared with support
- Manual cleanup: delete the file

## Health Checks

### Startup Health Check

The application performs the following checks on startup:

1. **Data directory** — Existence and permissions
2. **Database** — Integrity and accessibility
3. **Settings** — Valid JSON, schema version
4. **Logging** — Log directory exists, file writeable
5. **Encryption** — Crypto provider initialized
6. **Update check** — Availability check (if enabled)

### External Health Check

```powershell
# Complete health check script
function Test-WikilabsHealth {
    $appData = "$env:APPDATA\com.wikilabs.copilot"
    $issues = @()

    # Check data directory
    if (-not (Test-Path $appData)) {
        $issues += "Data directory not found"
        Write-Host "FAIL: Data directory not found" -ForegroundColor Red
        return
    }
    Write-Host "PASS: Data directory exists" -ForegroundColor Green

    # Check database
    if (Test-Path "$appData\wikilabs.db") {
        Write-Host "PASS: Database exists" -ForegroundColor Green
    } else {
        $issues += "Database not found"
        Write-Host "WARN: Database not found (fresh install)" -ForegroundColor Yellow
    }

    # Check settings
    if (Test-Path "$appData\settings.json") {
        Write-Host "PASS: Settings exist" -ForegroundColor Green
    } else {
        $issues += "Settings not found (fresh install)"
        Write-Host "WARN: Settings not found (fresh install)" -ForegroundColor Yellow
    }

    # Check logs
    $logDir = "$appData\logs"
    if (Test-Path $logDir) {
        $logFiles = Get-ChildItem $logDir -ErrorAction SilentlyContinue
        Write-Host "PASS: Logs directory exists ($($logFiles.Count) files)" -ForegroundColor Green
    }

    # Check disk space
    $disk = Get-PSDrive $env:SystemDrive
    $freeGB = [math]::Round($disk.Free / 1GB, 2)
    if ($freeGB -lt 1) {
        $issues += "Low disk space: ${freeGB} GB free"
        Write-Host "FAIL: Low disk space (${freeGB} GB)" -ForegroundColor Red
    } else {
        Write-Host "PASS: Disk space: ${freeGB} GB free" -ForegroundColor Green
    }

    # Summary
    Write-Host "`n=== Health Summary ===" -ForegroundColor Cyan
    if ($issues.Count -eq 0) {
        Write-Host "All checks passed" -ForegroundColor Green
    } else {
        Write-Host "Issues found: $($issues.Count)" -ForegroundColor Yellow
        foreach ($issue in $issues) {
            Write-Host "  • $issue" -ForegroundColor Yellow
        }
    }
}
```

## Capacity Planning

### Storage Estimates

| Component | Average Size | Notes |
|-----------|-------------|-------|
| Application | ~50 MB | Tauri + binaries |
| Database (empty) | ~1 MB | Schema only |
| Database (per workspace) | ~5-10 MB | Chat history |
| Database (per 1000 chunks) | ~50 MB | Knowledge base |
| Settings | < 1 MB | JSON configuration |
| Logs (daily) | ~1-5 MB | Depends on log level |
| Crash reports | < 1 MB | Per crash |

### Growth Projections

| Metric | Low | Medium | High |
|--------|-----|--------|------|
| Workspaces | 5 | 20 | 50 |
| Chats per WS | 50 | 200 | 500 |
| Knowledge docs | 100 | 500 | 2000 |
| Storage (1 year) | 200 MB | 1 GB | 5 GB |

---

*For monitoring details, see [Operations Foundation](operations/OPERATIONS_FOUNDATION.md).*
*For troubleshooting, see [Troubleshooting Guide](TROUBLESHOOTING.md).*
*For administration, see [Administrator Guide](admin-guide/ADMINISTRATOR_GUIDE.md).*