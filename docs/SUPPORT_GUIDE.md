# Support Guide — Wiki Labs AI Copilot v1.0.0

> How to get help, submit bug reports, collect diagnostic information, and navigate support escalation procedures.

## Table of Contents

1. [Support Overview](#support-overview)
2. [Support Channels](#support-channels)
3. [Getting Help](#getting-help)
4. [Self-Help Resources](#self-help-resources)
5. [Bug Reporting](#bug-reporting)
6. [Diagnostic Information Collection](#diagnostic-information-collection)
7. [Creating a Support Ticket](#creating-a-support-ticket)
8. [Escalation Procedures](#escalation-procedures)
9. [Incident Severity Levels](#incident-severity-levels)
10. [Response Time Expectations](#response-time-expectations)
11. [Follow-Up Process](#follow-up-process)
12. [Known Limitations](#known-limitations)
13. [Security Incident Reporting](#security-incident-reporting)
14. [Feature Requests](#feature-requests)
15. [What Information to Provide](#what-information-to-provide)

---

## Support Overview

Wiki Labs AI Copilot provides multiple channels for support. This guide walks you through:

- Self-help resources (documentation, FAQs, known issues)
- How to collect diagnostic information for bug reports
- The bug reporting process
- Escalation procedures for critical issues
- Security incident reporting

**Before reaching out to support**, check the self-help resources below — most common issues are documented.

---

## Support Channels

### Documentation

| Resource | Description |
|----------|-------------|
| [Quick Start](QUICK_START.md) | 5-minute getting started guide |
| [User Guide](user-guide/USER_GUIDE.md) | Complete end-user documentation |
| [Installation Guide](INSTALLATION_GUIDE.md) | Installation and upgrade instructions |
| [Architecture Guide](ARCHITECTURE_GUIDE.md) | Technical architecture overview |
| [Security Guide](SECURITY_GUIDE.md) | Security model, encryption, threat model |
| [Developer Guide](DEVELOPER_GUIDE.md) | Development and contribution guide |
| [Operations Guide](OPERATIONS_GUIDE.md) | Monitoring, logging, backup/restore |
| [Skill Pack Dev Guide](SKILL_PACK_DEVELOPMENT_GUIDE.md) | Creating custom skill packs |
| [Troubleshooting](TROUBLESHOOTING.md) | Common issues and fixes |
| [Known Limitations](KNOWN_LIMITATIONS.md) | Documented limitations |
| [Release Notes](RELEASE_NOTES.md) | Version changelog |
| [FAQ](FAQ.md) | 35+ common questions |

### Community

| Channel | Link |
|---------|------|
| GitHub Issues | [github.com/wikilabs/ai-copilot/issues](https://github.com/wikilabs/ai-copilot/issues) |
| GitHub Discussions | [github.com/wikilabs/ai-copilot/discussions](https://github.com/wikilabs/ai-copilot/discussions) |
| Discord | Join the Wiki Labs Discord for real-time support |
| Wiki Labs Blog | [blog.wikilabs.com](https://blog.wikilabs.com) for announcements |

### Enterprise Support

Enterprise customers have access to dedicated support through their contractual SLA. Contact your Wiki Labs account manager for the enterprise support portal URL.

---

## Getting Help

### Step 1: Check the Documentation

Before creating a support ticket, check these resources:

1. **Is this a known issue?** → Read [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md)
2. **Is this a setup/configuration question?** → Read [USER_GUIDE.md](user-guide/USER_GUIDE.md) and [INSTALLATION_GUIDE.md](INSTALLATION_GUIDE.md)
3. **Is this a common problem?** → Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md) and [FAQ.md](FAQ.md)
4. **Is this a security question?** → Read [SECURITY_GUIDE.md](SECURITY_GUIDE.md)
5. **Is this about skill packs?** → Read [SKILL_PACK_DEVELOPMENT_GUIDE.md](SKILL_PACK_DEVELOPMENT_GUIDE.md)

### Step 2: Search Existing Issues

Search [GitHub Issues](https://github.com/wikilabs/ai-copilot/issues) for your problem. Your issue may already be reported and possibly fixed in a newer version.

### Step 3: Try Self-Healing Steps

Many issues have documented self-fix procedures:

| Problem | Self-Fix | See Also |
|---------|----------|----------|
| Application won't start | Rename corrupt `settings.json` | [FAQ](FAQ.md) |
| AI provider not responding | Test connection in Settings | [FAQ](FAQ.md) |
| Skill not activating | Check detection rules and dependencies | [FAQ](FAQ.md) |
| Database corrupted | Attempt SQLite repair | [OPERATIONS_GUIDE.md](OPERATIONS_GUIDE.md) |
| Performance issues | Check disk space and logs | [OPERATIONS_GUIDE.md](OPERATIONS_GUIDE.md) |

### Step 4: Collect Diagnostics

If the issue persists, collect diagnostic information before contacting support. See [Diagnostic Information Collection](#diagnostic-information-collection) below.

---

## Self-Help Resources

### Quick Reference

#### Finding Your Log Files

```powershell
# Windows
Get-ChildItem "$env:APPDATA\com.wikilabs.copilot\logs\"

# Linux
ls -la ~/.config/com.wikilabs.copilot/logs/
```

#### Checking App Data Directory Size

```powershell
# Windows
$apps = Get-ChildItem "$env:APPDATA\com.wikilabs.copilot" -Recurse -File
$apps | Group-Object Directory | ForEach-Object {
    $sizeMB = ($_.Group | Measure-Object Length -Sum).Sum / 1MB
    Write-Host "$($_.Name): $([math]::Round($sizeMB, 2)) MB"
}
```

#### Testing AI Provider Connection

1. Open Settings → AI Provider
2. Click **Test Connection**
3. Check the response time (should be < 5 seconds)

#### Restarting the Application

```powershell
# Stop
Stop-Process -Name "wikilabs-ai-copilot" -Force -ErrorAction SilentlyContinue

# Start
Start-Process "wikilabs-ai-copilot"
```

### Common Error Messages

| Error Message | Likely Cause | Resolution |
|--------------|-------------|------------|
| "No encryption available" | Encryption initialization failed | Restart application; check system fingerprint |
| "Provider connection failed" | AI provider unreachable | Check endpoint URL and network connectivity |
| "Skill 'X' not found" | Skill pack not installed | Install the missing skill pack |
| "Invalid YAML" | Malformed skill manifest | Validate YAML syntax in manifest files |
| "Database lock" | Another instance running | Close other instances or wait |

---

## Bug Reporting

### Before You Report

1. **Verify the issue** — Can you reproduce it consistently?
2. **Check the latest version** — Update to the latest release; the issue may be fixed
3. **Search existing issues** — Look for duplicate reports on GitHub
4. **Check Known Limitations** — Some behaviors are documented limitations, not bugs

### Creating a Bug Report

Use the bug report template on [GitHub Issues](https://github.com/wikilabs/ai-copilot/issues). Include:

1. **Title** — Clear, concise description of the issue
2. **Version** — Your current version (Settings → About)
3. **Platform** — Windows/Linux, OS version, architecture
4. **Steps to Reproduce** — Numbered steps that reliably reproduce the issue
5. **Expected Behavior** — What you expected to happen
6. **Actual Behavior** — What actually happened
7. **Severity** — Your assessment (see Severity Levels below)
8. **Diagnostic Package** — Attach your diagnostic zip file (see below)
9. **Screenshots/Logs** — Any relevant screenshots or log excerpts
10. **Additional Context** — Any other relevant information

### Example Bug Report

```
## Summary
AI Copilot crashes when opening a conversation with more than 100 messages

## Version
1.0.0 (Build 20260721)

## Platform
Windows 11 23H2, x86_64

## Steps to Reproduce
1. Start AI Copilot
2. Open a conversation with 100+ messages
3. Scroll to the bottom of the conversation
4. Application crashes

## Expected Behavior
Conversation displays and scrolls normally

## Actual Behavior
Application crashes with error in crash report

## Severity
P2 — Major feature broken

## Diagnostic Package
[attach wikilabs-diag-20260721-100000.zip]
```

---

## Diagnostic Information Collection

### Method 1: Automated Diagnostic Package (Recommended)

The application provides a built-in diagnostic package generator:

1. Open Settings → Diagnostics
2. Click **Generate Diagnostic Package**
3. Choose a save location
4. Attach the `.zip` file to your support request

**What's included (all data redacted of credentials):**
- Application version and platform info
- Redacted settings summary
- Log files (with secrets redacted)
- System information (OS, disk, memory)
- Database info (schema version, table counts, sizes)
- Validation errors (configuration issues)
- Last crash details (if any)
- Benchmark metrics (performance data)

### Method 2: Manual Log Collection

If the automated generator is unavailable:

```powershell
# Create diagnostic directory
$diagDir = "$env:TEMP\wikilabs-diag-manual-$(Get-Date -Format 'yyyyMMdd')"
New-Item -ItemType Directory -Force -Path $diagDir

# Version info
Get-ItemProperty "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" |
  Where-Object { $_.DisplayName -like "*Wiki Labs*" } |
  Select-Object DisplayName, DisplayVersion |
  ConvertTo-Json |
  Out-File "$diagDir\version.json"

# Log files
Copy-Item "$env:APPDATA\com.wikilabs.copilot\logs\*" "$diagDir\logs\" -ErrorAction SilentlyContinue

# Settings summary (redacted — do NOT copy settings.json directly)
Get-Content "$env:APPDATA\com.wikilabs.copilot\settings.json" |
  ConvertFrom-Json |
  ForEach-Object {
    $_ | ConvertTo-Json -Depth 10 |
    ForEach-Object {
      $_ -replace '"api_key"\s*:\s*"[^"]*"', '"api_key": "***REDACTED***"' |
      ForEach-Object { $_ -replace '"token"\s*:\s*"[^"]*"', '"token": "***REDACTED***"' }
    }
  } |
  Out-File "$diagDir\settings_redacted.json"

# Database info
sqlite3 "$env:APPDATA\com.wikilabs.copilot\wikilabs.db" "PRAGMA integrity_check; SELECT count(*) FROM chat_messages;" > "$diagDir\db_info.txt"

# Crash info
Copy-Item "$env:APPDATA\com.wikilabs.copilot\crash\*" "$diagDir\crash\" -ErrorAction SilentlyContinue

# Compress
Compress-Archive -Path "$diagDir\*" -DestinationPath "$env:TEMP\wikilabs-diag-manual.zip"

Write-Host "Diagnostic package: $env:TEMP\wikilabs-diag-manual.zip"
```

### Method 3: Specific Error Logs

For specific error investigation, extract relevant log lines:

```powershell
# All errors from the last 24 hours
$yesterday = (Get-Date).AddDays(-1).ToString("yyyy-MM-dd")
Select-String -Path "$env:APPDATA\com.wikilabs.copilot\logs\*.log" -Pattern "ERROR\|FATAL" |
  Where-Object { $_.LineNumber -ge 1 } |  # Filter by date in JSON logs
  Select-Object -First 100

# Check last crash report
Get-Content "$env:APPDATA\com.wikilabs.copilot\crash\last_crash.json" | ConvertFrom-Json
```

---

## Creating a Support Ticket

### GitHub Issues (Open Source Users)

1. Go to [github.com/wikilabs/ai-copilot/issues](https://github.com/wikilabs/ai-copilot/issues)
2. Click **New Issue**
3. Select the appropriate template (Bug Report, Feature Request, Question)
4. Fill in all required fields
5. Attach your diagnostic package
6. Submit

### Discord (Community Support)

1. Join the Wiki Labs Discord server
2. Go to the `#support` channel
3. Describe your issue with context
4. Attach your diagnostic package if available
5. A Wiki Labs team member will respond

### Email (Enterprise Customers)

Email your enterprise support contact with:
- Subject: `[Support] [Severity] Brief description`
- Your organization name
- Attached diagnostic package
- Detailed issue description

### Website Contact Form

Use the contact form on [wikilabs.com/support](https://wikilabs.com/support) for general inquiries.

---

## Escalation Procedures

### When to Escalate

Escalate a support request when:

1. **No response within expected time** (see Response Time Expectations)
2. **Issue severity has increased** since initial report
3. **Workaround is unavailable** for a critical issue
4. **Multiple users are affected**
5. **Data loss or security concern** has been identified

### Escalation Path

```
Level 1: Community Support (GitHub Issues, Discord)
    │ (unresolved or urgent)
    ▼
Level 2: Wiki Labs Support Team (support@wikilabs.com)
    │ (unresolved or P1/P2)
    ▼
Level 3: Engineering Team (via support team)
    │ (unresolved or P1)
    ▼
Level 4: Leadership / Customer Success (enterprise only)
```

### Enterprise Escalation

Enterprise customers have a direct escalation path through their account manager. For critical issues, the account manager will engage the engineering team directly.

### Emergency Contact

For P1 (Critical) issues affecting production environments, use the emergency contact channel:

- **Enterprise customers:** Contact your account manager or use the emergency hotline provided in your contract
- **Open source users:** Mark your GitHub issue with the `P1-critical` label and mention `@wikilabs-engineering`

---

## Incident Severity Levels

### P1 — Critical

| Attribute | Detail |
|-----------|--------|
| **Definition** | Application unusable, data at risk, or security breach |
| **Examples** | Data corruption affecting multiple workspaces, credential exposure, application won't start for all users |
| **Impact** | All users affected or data loss risk |
| **Response** | Immediate acknowledgment, engineering engagement within 1 hour |

### P2 — High

| Attribute | Detail |
|-----------|--------|
| **Definition** | Major feature broken; workarounds exist or are limited |
| **Examples** | AI provider not responding, skill pack not loading, crash on specific operation |
| **Impact** | Significant functionality impaired |
| **Response** | Acknowledgment within 4 hours, workaround or fix timeline within 24 hours |

### P3 — Medium

| Attribute | Detail |
|-----------|--------|
| **Definition** | Minor feature broken; no data impact |
| **Examples** | UI glitch, incorrect recommendation, missing detection rule |
| **Impact** | Limited functionality impaired |
| **Response** | Acknowledgment within 24 hours, fix timeline within next release |

### P4 — Low

| Attribute | Detail |
|-----------|--------|
| **Definition** | Cosmetic or convenience issue |
| **Examples** | Typo in UI, inconsistent formatting, minor performance issue |
| **Impact** | No functionality impaired |
| **Response** | Added to backlog, fix in future release |

---

## Response Time Expectations

### GitHub Issues (Open Source)

| Severity | Acknowledgment | Initial Response |
|----------|---------------|-----------------|
| P1 — Critical | Same business day | Within 4 hours |
| P2 — High | Within 24 hours | Within 48 hours |
| P3 — Medium | Within 24 hours | Within 1 week |
| P4 — Low | As time permits | Next release cycle |

### Enterprise Support (Contractual SLA)

| Severity | Acknowledgment | Resolution Target |
|----------|---------------|------------------|
| P1 — Critical | 15 minutes | 4 hours |
| P2 — High | 1 hour | 8 hours |
| P3 — Medium | 4 hours | 24 hours |
| P4 — Low | 1 business day | Next release |

---

## Follow-Up Process

### After Submitting a Report

1. **You will receive** an acknowledgment within the expected response time
2. **Check for follow-up questions** — The support team may request additional information
3. **Test provided workarounds** — If a workaround is provided, test it and report results
4. **Update your report** — Add new information to the original issue rather than creating a new one
5. **Verify resolution** — Confirm the fix works in your environment before closing

### During Investigation

- **Be patient** — Debugging can take time, especially for race conditions
- **Provide requested info promptly** — Delays in providing diagnostics slow down resolution
- **Test on the latest version** — Always reproduce on the most recent release
- **One report per issue** — Duplicate reports fragment attention

### After Resolution

1. **Verify the fix** in your environment
2. **Update your report** with resolution confirmation
3. **Optional: Share your experience** — Help others by documenting your resolution in the issue comments
4. **Consider contributing** — If you found a solution, consider adding it to the documentation

---

## Known Limitations

Before reporting an issue, check [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) for documented behaviors that are not bugs but known constraints of the current release.

Common known limitations include:

- Single AI provider configuration (multiple providers supported but only one active at a time)
- No automatic data synchronization to remote servers (local-first design)
- Skill pack detection may not cover all technology versions
- No macOS native build (Windows-first)
- Local model (Ollama/vLLM) requires pre-deployment on the user's machine

---

## Security Incident Reporting

### If You Discover a Security Vulnerability

1. **Do NOT disclose publicly** — Use the private reporting channel
2. **Enable Privacy Mode** in Settings → Privacy as a precaution
3. **Collect evidence** — Generate a diagnostic package (Settings → Diagnostics)
4. **Do NOT modify logs** — Preserve original evidence
5. **Report through the private channel:**
   - GitHub: Mark the issue as `private-security`
   - Email: `security@wikilabs.com`
   - Discord: Message a Wiki Labs maintainer directly (private DM)

### What to Include in a Security Report

- **Description** — Clear description of the vulnerability
- **Impact** — What an attacker could achieve
- **Reproducibility** — Steps to reproduce (or indication if it's intermittent)
- **Affected versions** — Which versions are impacted
- **Evidence** — Screenshots, logs, or crash reports (without credentials)
- **Suggested fix** — If you have one (optional but appreciated)

### Security Response

- **Acknowledgment:** Within 24 hours
- **Assessment:** Within 3 business days
- **Fix timeline:** Depending on severity (critical fixes within days)
- **Disclosure:** Coordinated with the reporter

### Security Advisories

Security advisories are published on GitHub under [github.com/wikilabs/ai-copilot/security/advisories](https://github.com/wikilabs/ai-copilot/security/advisories). Subscribe to these advisories for critical security updates.

---

## Feature Requests

### Submitting a Feature Request

1. Go to [GitHub Issues](https://github.com/wikilabs/ai-copilot/issues)
2. Click **New Issue** → Select "Feature Request" template
3. Include:
   - **Summary** — What the feature does
   - **Motivation** — Why it's needed
   - **Proposed Solution** — How you think it should work
   - **Alternatives Considered** — Other approaches you've evaluated
   - **Use Cases** — Specific scenarios where this feature is needed

### Feature Request Lifecycle

```
New → Triage → Approved → Planned → In Development → Released
```

- **Triage** — A maintainer reviews and categorizes the request
- **Approved** — The feature is accepted for implementation
- **Planned** — The feature is added to the roadmap
- **In Development** — Engineering is working on the feature
- **Released** — The feature is shipped in a release

### Contributing a Feature

If you want to implement a feature yourself:

1. Discuss the approach in the feature request issue
2. Submit a design proposal (for complex features)
3. Open a PR with your implementation
4. The team will review and provide feedback

See the [Developer Guide](DEVELOPER_GUIDE.md) for contribution guidelines.

---

## What Information to Provide

### The Checklist

When submitting any support request, provide:

| Item | Required | Where to Find |
|------|----------|---------------|
| Application version | Yes | Settings → About |
| Platform/OS | Yes | System settings |
| Steps to reproduce | Yes | Your observation |
| Expected behavior | Yes | Your expectation |
| Actual behavior | Yes | Your observation |
| Diagnostic package | Recommended | Settings → Diagnostics |
| Screenshots | When helpful | Your observation |
| Log excerpts | When helpful | Logs directory |
| Severity assessment | Recommended | Based on impact |

### Diagnostic Package Contents

A diagnostic package (`wikilabs-diag-*.zip`) includes:

```
wikilabs-diag-20260721-100000.zip
├── version.json              # App version, platform info
├── settings_report.json      # Redacted settings summary
├── log_files/                # Log files with redaction
├── system_info.json          # OS, disk, memory info
├── database_info.json        # Schema, table counts, sizes
├── validation_errors.json    # Configuration issues
└── crash_info.json           # Last crash details (if any)
```

All sensitive data (API keys, tokens, credentials) is redacted. It is safe to attach the diagnostic package to any support request.

---

## Appendix: Quick Reference Cards

### Getting Help — Quick Start

```
1. Search FAQ: docs/FAQ.md
2. Search Troubleshooting: docs/TROUBLESHOOTING.md
3. Check Known Limitations: docs/KNOWN_LIMITATIONS.md
4. Generate diagnostics: Settings → Diagnostics → Generate
5. Open issue: github.com/wikilabs/ai-copilot/issues
6. Join Discord: Real-time community support
```

### Bug Report — Checklist

```
□ Version number
□ Platform and OS
□ Steps to reproduce
□ Expected behavior
□ Actual behavior
□ Diagnostic package attached
□ Screenshots (if applicable)
□ Checked for existing issues
□ Checked Known Limitations
```

### Emergency — P1 Critical

```
□ Enable Privacy Mode
□ Collect diagnostic package
□ Preserve log files
□ Contact via:
   - Enterprise: Account manager / emergency hotline
   - Open source: GitHub issue with P1-critical label
□ Describe impact clearly
□ Document affected systems
```

---

*For technical details, see the [Architecture Guide](ARCHITECTURE_GUIDE.md), [Security Guide](SECURITY_GUIDE.md), [Developer Guide](DEVELOPER_GUIDE.md), [Operations Guide](OPERATIONS_GUIDE.md), and [Troubleshooting Guide](TROUBLESHOOTING.md).*