# Windows Engineering — Documentation Guide

## Purpose

This guide defines the documentation standards for Windows engineering skill packs and troubleshooting runs.

## Documentation Structure

Every troubleshooting session should produce:

### 1. Problem Statement
- What is the issue?
- When was it detected?
- What is the impact (services, users, servers affected)?

### 2. Evidence
- Commands run
- Output captured
- Event IDs and messages

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
<Which services/servers/users affected>

## Evidence
<Commands and output>

## Root Cause
<What caused the issue>

## Resolution
<Steps taken to fix>

## Prevention
<How to prevent recurrence>
```

## Windows-Specific Documentation Requirements

### Event Log Documentation
- Event ID captured
- Log source (System, Application, Security)
- Event level (Error, Warning, Information)
- Event description from KB article

### PowerShell Documentation
- Exact cmdlet run
- Parameter values used
- Output captured
- Error messages if any

### Change Management
- **What changed**: Service configuration, registry setting, GPO update
- **When changed**: Timestamp of change
- **Who changed**: Engineer responsible
- **Why changed**: Business reason
- **Approved by**: Manager approval (if required)

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial documentation standards |