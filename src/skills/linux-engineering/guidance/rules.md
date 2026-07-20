# Linux Engineering Skill — Guidance

## Core Principles

1. **Always verify before acting** — Confirm the system state before making changes
2. **Ask for confirmation** — Never execute destructive commands without explicit approval
3. **Document changes** — Record all modifications for audit trail
4. **Test in staging** — Validate changes on non-production systems first
5. **Have rollback plans** — Always know how to reverse changes

## Detection Guidelines

### File-Based Detection

- **Confidence scoring**: Base confidence 0.7-0.9 for config file matches
- **Multiple signals**: Require ≥2 signals for confident detection
- **Version context**: Always check version numbers (e.g., kernel version, package version)
- **State context**: Check if a service is running, stopped, or failed

### Command-Based Detection

- **Lower confidence**: Base confidence 0.5-0.7 for command-only signals
- **Output analysis**: Parse command output for meaningful patterns
- **Environment**: Note which shell, which distro, which init system
- **Permissions**: Note if command requires sudo/root

### Pattern Matching

- **Use regex for flexibility**: Match variations in log formats, config options
- **Prioritize specificity**: More specific patterns = higher confidence
- **Handle edge cases**: Account for different distro conventions (systemd vs init)
- **Time-sensitive**: Some patterns only apply to specific versions

## Response Guidelines

### Critical Issues (Priority ≥ 9)

1. **Immediate action** required
2. **Document severity** in response
3. **Provide clear recovery steps**
4. **Set up monitoring** after resolution

### High Priority (Priority 7-8)

1. **Action within 24 hours**
2. **Explain risk** of inaction
3. **Provide workaround** if available
4. **Recommend prevention** measures

### Standard Priority (Priority ≤ 6)

1. **Address in next maintenance window**
2. **Explain impact** clearly
3. **Provide documentation** link
4. **Suggest automation** for future prevention

## Safety Rules

- Never run `rm -rf` without explicit confirmation and path verification
- Never change firewall rules without confirming current connectivity
- Never restart critical services without confirming maintenance window
- Always backup config files before modification
- Use `systemctl is-active` to confirm service state before action
- Use `df -h` to confirm disk space before cleanup actions

## Environment Awareness

### Ubuntu/Debian

- Package manager: `apt`/`dpkg`
- Init: systemd (all versions)
- Config: `/etc/hostname`, `/etc/fstab`, `/etc/network/`
- Logs: `/var/log/syslog`, `/var/log/auth.log`, journalctl

### RHEL/CentOS/Fedora

- Package manager: `yum`/`dnf`/`rpm`
- Init: systemd (all versions)
- Config: `/etc/hostname`, `/etc/fstab`, `/etc/sysconfig/`
- Logs: `/var/log/messages`, `/var/log/secure`, journalctl

### SUSE

- Package manager: `zypper`/`rpm`
- Init: systemd
- Config: `/etc/hostname`, `/etc/fstab`, `/etc/sysconfig/`
- Logs: `/var/log/messages`, journalctl

### Alpine

- Package manager: `apk`
- Init: openrc (not systemd)
- Config: `/etc/hostname`, `/etc/fstab`, `/etc/network/`
- Logs: `/var/log/syslog`, dmesg

## Documentation Standards

- Use code blocks for all commands
- Include `--no-pager` flag for verbose output
- Prefix sudo commands with `# ` or document privilege needed
- Link to official documentation when available
- Include verification steps after each action

## Communication

- **Clear subject**: Start with issue type (e.g., "[CRITICAL] Disk Space")
- **Current state**: Describe system state before action
- **Planned action**: State what you will do and why
- **Expected outcome**: State what will happen after action
- **Rollback**: State how to reverse the action if needed

## Escalation

### When to Escalate

1. Database corruption detected
2. Production system outage
3. Security breach suspected
4. Data loss imminent
5. Cannot reproduce issue in non-prod

### Escalation Information

Include:
- Current system state
- All relevant logs
- Steps already taken
- Impact assessment
- Time elapsed since incident