# Support Guide — Wiki Labs AI Copilot v1.0.0

> How to get help, support channels, and escalation procedures.

## Table of Contents

1. [Getting Help](#getting-help)
2. [Support Channels](#support-channels)
3. [Self-Help Resources](#self-help-resources)
4. [When to Contact Support](#when-to-contact-support)
5. [Submitting a Support Request](#submitting-a-support-request)
6. [What to Include in Your Request](#what-to-include-in-your-request)
7. [Support Response Times](#support-response-times)
8. [Escalation Procedures](#escalation-procedures)
9. [Enterprise Support](#enterprise-support)
10. [Community Resources](#community-resources)

## Getting Help

Wiki Labs AI Copilot provides multiple support channels to help you get the most out of the application.

## Support Channels

### Documentation (First Resource)

Before contacting support, review the comprehensive documentation:

| Document | Description |
|----------|-------------|
| [Quick Start Guide](QUICK_START.md) | 5-minute getting started guide |
| [User Guide](user-guide/USER_GUIDE.md) | Complete user manual with feature documentation |
| [Installation Guide](INSTALLATION_GUIDE.md) | Installation, upgrade, repair, uninstall |
| [Troubleshooting Guide](TROUBLESHOOTING.md) | Common issues and recovery procedures |
| [FAQ](FAQ.md) | Frequently asked questions |
| [Architecture Guide](ARCHITECTURE_GUIDE.md) | System architecture and design |
| [Security Guide](SECURITY_GUIDE.md) | Security model and configuration |
| [Operations Guide](OPERATIONS_GUIDE.md) | Monitoring, logging, maintenance |
| [Developer Guide](DEVELOPER_GUIDE.md) | Development workflow and contributing |
| [Skill Pack Development Guide](SKILL_PACK_DEVELOPMENT_GUIDE.md) | Creating and distributing skill packs |

### GitHub Issues

| Channel | URL | Best For |
|---------|-----|----------|
| Bug Reports | https://github.com/wikilabs/wikilabs-ai-copilot/issues | Bugs, crashes, unexpected behavior |
| Feature Requests | https://github.com/wikilabs/wikilabs-ai-copilot/issues | New feature ideas |
| Security Issues | https://github.com/wikilabs/wikilabs-ai-copilot/security/advisories | Security vulnerabilities (private) |
| Discussions | https://github.com/wikilabs/wikilabs-ai-copilot/discussions | General questions, tips, community |

**GitHub Response Time:** 48 hours (standard), 24 hours (security)

### Email Support

| Email | Use Case | Response Time |
|-------|----------|---------------|
| support@wikilabs.com | General support questions | 24 hours |
| sales@wikilabs.com | Licensing and purchasing | 48 hours |
| security@wikilabs.com | Security concerns | 48 hours |
| feedback@wikilabs.com | General feedback | 72 hours |

### In-Application Help

| Feature | Access | Description |
|---------|--------|-------------|
| Keyboard Shortcuts | `F1` or `Ctrl + ?` | Shows all keyboard shortcuts |
| About Dialog | Settings → About | Application version, platform info |
| Diagnostic Package | Settings → Diagnostics | Auto-generated diagnostic bundle |
| Crash Reports | `crash/` directory | Previous crash data |

## Self-Help Resources

### Built-In Help

1. **Keyboard Shortcuts Reference**
   - Press `F1` or `Ctrl + ?` to view all shortcuts
   - Available in Settings → UI Settings → Shortcuts Help

2. **Diagnostic Package Generator**
   - Access via Settings → Diagnostics
   - Generates a bundle with redacted settings and log metadata
   - Share with support for faster troubleshooting

3. **Error Handling UI**
   - The application displays error messages when issues occur
   - Error details are saved to `crash/last_crash.json`
   - Check the Error panel in Settings for recent errors

4. **Log Viewer**
   - Log files located at `%APPDATA%\com.wikilabs.copilot\logs\`
   - Main log: `wikilabs-copilot.log`
   - Use the Troubleshooting Guide to analyze log patterns

### Knowledge Base Search

1. Open the Knowledge panel
2. Type your question in the search bar
3. Results from imported documentation appear
4. Click a result to view the full document

### Troubleshooting Flow

```
Issue Identified
       │
       ├── Is it in the FAQ? → Read FAQ
       │       │
       │       └── Yes → Solution found ✓
       │       └── No → Continue ↓
       │
       ├── Is it in the Troubleshooting Guide? → Read Guide
       │       │
       │       └── Yes → Try recovery procedures
       │       └── No → Continue ↓
       │
       ├── Can I fix it with logs? → Read logs
       │       │
       │       └── Yes → Apply fix
       │       └── No → Continue ↓
       │
       ├── Generate Diagnostic Package → Settings → Diagnostics
       │       │
       │       └── Send to support ✓
       │
       └── Contact Support → See below
```

## When to Contact Support

### Contact Support When:

| Situation | Recommended Channel |
|-----------|-------------------|
| Application crashes repeatedly | GitHub Issues + diagnostic package |
| Data corruption (unresolvable) | Email support |
| AI provider connectivity issues | GitHub Issues + logs |
| Performance problems | GitHub Issues + diagnostic package |
| Security concerns | security@wikilabs.com |
| Licensing or purchasing | sales@wikilabs.com |
| Feature requests | GitHub Discussions |
| General questions | GitHub Discussions |

### Before Contacting Support

Please complete these steps:

1. **Check documentation** — Review the [Troubleshooting Guide](TROUBLESHOOTING.md) and [FAQ](FAQ.md)
2. **Check logs** — Review `%APPDATA%\com.wikilabs.copilot\logs\wikilabs-copilot.log`
3. **Try recovery** — Follow the recovery procedures in the Troubleshooting Guide
4. **Generate diagnostics** — Create a diagnostic package (Settings → Diagnostics)
5. **Note the version** — Check Settings → About for the current version

## Submitting a Support Request

### GitHub Issues

1. Go to https://github.com/wikilabs/wikilabs-ai-copilot/issues
2. Click **New Issue**
3. Select the appropriate issue type:
   - **Bug Report** — For crashes, unexpected behavior
   - **Feature Request** — For new feature ideas
   - **Question** — For general questions
4. Fill in the issue template:
   - Application version
   - Operating system
   - Steps to reproduce (for bugs)
   - Expected vs. actual behavior
   - Log excerpts (if applicable)
5. Attach diagnostic package or log files

### Email Support

1. Send an email to support@wikilabs.com
2. Include:
   - Subject line with brief description
   - Application version
   - Operating system
   - Detailed description of the issue
   - Steps to reproduce
   - Attach diagnostic package and logs

### Email Template

```
Subject: [Wiki Labs AI Copilot] Issue Description

Application Version: 1.0.0
Operating System: Windows 11 64-bit
Issue Type: [Bug / Question / Feature Request]

Description:
[Describe the issue in detail]

Steps to Reproduce:
1. [Step 1]
2. [Step 2]
3. [Step 3]

Expected Behavior:
[What you expected to happen]

Actual Behavior:
[What actually happened]

Logs/Attachments:
- Diagnostic package: [attached]
- Log excerpts: [attached or pasted]
```

## What to Include in Your Request

To help support resolve your issue quickly, include:

### Required Information

| Item | How to Find |
|------|-------------|
| Application version | Settings → About |
| Operating system | Windows Settings → System → About |
| Issue description | Your own description |
| Steps to reproduce | Reproduce the issue and note each step |

### Helpful Information

| Item | How to Find |
|------|-------------|
| Log file | `%APPDATA%\com.wikilabs.copilot\logs\wikilabs-copilot.log` |
| Diagnostic package | Settings → Diagnostics → Generate |
| Crash report | `%APPDATA%\com.wikilabs.copilot\crash\last_crash.json` |
| Settings file | `%APPDATA%\com.wikilabs.copilot\settings.json` (redact API keys first) |
| Screenshots | Describe visual issues or error dialogs |

### Information NOT Needed

| Item | Reason |
|------|--------|
| Full database file | Too large, contains sensitive data |
| Full API key | Already stored securely, not the issue |
| System hardware specs | Not typically relevant for application issues |

## Support Response Times

| Priority | Description | Response Time | Resolution Target |
|----------|-------------|---------------|------------------|
| **Critical** | Application unusable, data loss risk | 4 hours | 24 hours |
| **High** | Major feature broken, no workaround | 8 hours | 3 business days |
| **Medium** | Minor feature broken, workaround available | 24 hours | 1 week |
| **Low** | Cosmetic issue, enhancement request | 48 hours | Next release or backlog |

### Priority Definitions

| Priority | Examples |
|----------|----------|
| **Critical** | Crash on startup, data corruption, security vulnerability |
| **High** | AI provider connection fails, chat not sending, knowledge import broken |
| **Medium** | UI glitch, guidance panel not showing, export not working |
| **Low** | Typo in documentation, minor UI alignment issue, color suggestion |

## Escalation Procedures

### Standard Escalation Path

```
User
  │
  ├── 1. Self-help (docs, FAQ, troubleshooting)
  │       └── If unresolved →
  │
  ├── 2. Support Team (GitHub / Email)
  │       └── If unresolved →
  │
  ├── 3. Technical Lead Review
  │       └── If unresolved →
  │
  └── 4. Engineering Team Investigation
          └── If unresolved →
                  └── 5. Root cause analysis and fix
```

### Escalation Criteria

Escalate to the next level when:

| Level | Escalation Trigger |
|-------|-------------------|
| → Support Team | Issue not resolved in self-help |
| → Technical Lead | Issue is critical or has no workaround |
| → Engineering Team | Issue requires code change or deep investigation |
| → Product Team | Issue involves design or feature decisions |

### Enterprise Escalation

Enterprise customers have a dedicated support channel:

| Contact | Details |
|---------|---------|
| Dedicated Support Email | enterprise-support@wikilabs.com |
| Dedicated Support Portal | portal.wikilabs.com/support |
| Account Manager | Contact your Wiki Labs account manager |
| SLA | Contract-defined response times |
| Escalation | Direct access to engineering team |

## Enterprise Support

### Enterprise Customer Benefits

| Benefit | Description |
|---------|-------------|
| Dedicated support team | Assigned support engineer |
| Priority response | Contract-defined SLAs |
| Custom skill packs | Development of technology-specific packs |
| Onboarding assistance | Help with initial deployment |
| Training | Application training sessions |
| Regular updates | Early access to new releases |
| Custom builds | Tailored builds for specific environments |

### Enterprise Support Contact

| Channel | Contact |
|---------|---------|
| Email | enterprise-support@wikilabs.com |
| Portal | portal.wikilabs.com/support |
| Phone | Contact your account manager |
| Account Manager | Direct contact from sales team |

## Community Resources

### GitHub Community

| Resource | URL |
|----------|-----|
| Issues | https://github.com/wikilabs/wikilabs-ai-copilot/issues |
| Discussions | https://github.com/wikilabs/wikilabs-ai-copilot/discussions |
| Wiki | https://github.com/wikilabs/wikilabs-ai-copilot/wiki |

### Contributing

Want to help improve Wiki Labs AI Copilot? See the [Developer Guide](DEVELOPER_GUIDE.md#contributing) for contribution guidelines.

### Community Best Practices

- **Search before posting** — Check existing issues and discussions
- **Provide context** — Include version, OS, steps to reproduce
- **Be constructive** — Suggest solutions when possible
- **Stay respectful** — Follow the code of conduct
- **Share knowledge** — Help other users in discussions

---

*For self-help, see [Troubleshooting Guide](TROUBLESHOOTING.md) and [FAQ](FAQ.md).*
*For feature documentation, see [User Guide](user-guide/USER_GUIDE.md).*
*For administration, see [Administrator Guide](admin-guide/ADMINISTRATOR_GUIDE.md).*