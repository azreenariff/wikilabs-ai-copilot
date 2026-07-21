# VMware vSphere Engineering — Documentation Guide

## Purpose

This guide defines the documentation standards for VMware vSphere engineering skill packs and troubleshooting runs.

## Documentation Structure

Every troubleshooting session should produce:

### 1. Problem Statement
- What is the issue?
- When was it detected?
- What is the impact (number of VMs, services affected)?

### 2. Evidence
- Commands run
- Output captured
- Screenshots from vSphere Client (if applicable)

### 3. Analysis
- What the evidence means
- Root cause identified
- Alternative causes ruled out

### 4. Resolution
- Steps taken to fix
- Verification results
- Rollback plan (if applicable)

### 5. Lessons Learned
- What went well
- What could be improved
- Prevention measures

## Template

```markdown
# Troubleshooting: <Issue Title>

## Summary
<One-line description of the issue>

## Impact
<Which VMs/hosts/clusters affected>

## Evidence
<Commands and output>

## Root Cause
<What caused the issue>

## Resolution
<Steps taken to fix>

## Prevention
<How to prevent recurrence>
```

## VMware-Specific Documentation Requirements

### Change Management Documentation
- **What changed**: VM configuration, cluster settings, storage layout
- **When changed**: Timestamp of change
- **Who changed**: Engineer responsible
- **Why changed**: Business reason
- **Approved by**: Manager approval (if required)

### Incident Documentation
- **Alert source**: vSphere alarms, monitoring tool, user report
- **Initial response**: First actions taken
- **Escalation**: When and to whom escalation occurred
- **Resolution time**: Duration from detection to resolution
- **Post-mortem**: Lessons learned document

## Storage

All troubleshooting documentation should be stored in:
- `/documentation/` — Standard operating procedures
- `/examples/worked-examples.md` — Worked troubleshooting scenarios

## Version Control

All documentation is version controlled. When updating:
1. Update the version table at the bottom
2. Note what changed and why
3. Link to related troubleshooting sessions

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial documentation standards |