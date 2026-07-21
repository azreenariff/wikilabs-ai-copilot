# Linux Engineering — Documentation Guide

## Purpose

This guide defines the documentation standards for Linux engineering skill packs and troubleshooting runs.

## Documentation Structure

Every troubleshooting session should produce:

### 1. Problem Statement
- What is the issue?
- When was it detected?
- What is the impact?

### 2. Evidence
- Commands run
- Output captured
- Screenshots (if applicable)

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
<Who/what was affected>

## Evidence
<Commands and output>

## Root Cause
<What caused the issue>

## Resolution
<Steps taken to fix>

## Prevention
<How to prevent recurrence>
```

## Storage

All troubleshooting documentation should be stored in:
- `/documentation/` — Standard operating procedures
- `/examples/worked-examples.md` — Worked troubleshooting scenarios

## Version Control

All documentation is version controlled. When updating:
1. Update the version table at the bottom
2. Note what changed and why
3. Link to related troubleshooting sessions