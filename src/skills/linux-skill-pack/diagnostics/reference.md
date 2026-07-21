# Linux Engineering — Diagnostics Guide

## Purpose

This guide documents the standard diagnostics methodology for Linux systems.

## Diagnostic Approach

### Phase 1: Symptom Identification

1. **Collect initial symptoms** — user report, monitoring alert, manual observation
2. **Classify the issue** — service, performance, security, network, storage
3. **Determine scope** — single host, cluster, application-specific

### Phase 2: Evidence Collection

For each category, collect relevant evidence:

#### Service Issues
```
systemctl status <service>
journalctl -u <service> --since "1 hour ago"
ss -tlnp | grep <port>
```

#### Performance Issues
```
uptime
top -bn1 | head -20
free -m
iostat -x 1 3
```

#### Storage Issues
```
df -h
df -i
du -sh /* 2>/dev/null | sort -rh | head -10
```

#### Network Issues
```
ip addr
ip route
ping -c 3 8.8.8.8
nslookup google.com
```

### Phase 3: Root Cause Analysis

Use the diagnostic flowchart:
1. Narrow down to the most likely cause
2. Verify with targeted commands
3. Document findings before remediation

### Phase 4: Remediation & Verification

1. Apply the fix
2. Verify with the same commands used in evidence collection
3. Monitor for regression

## Diagnostic Tools

| Tool | Purpose | Risk |
|------|---------|------|
| journalctl | System logs | Low |
| dmesg | Kernel messages | Low |
| strace | System call tracing | Low |
| lsof | Open file/ports | Low |
| iotop | Per-process I/O | Low |
| tcpdump | Packet capture | Low |

## Documentation

When documenting diagnostics:
1. **Timestamp** — when the issue was detected
2. **Symptoms** — what was observed
3. **Evidence** — commands run and output
4. **Root Cause** — identified cause
5. **Remediation** — action taken
6. **Verification** — confirmation of fix
7. **Prevention** — steps to prevent recurrence