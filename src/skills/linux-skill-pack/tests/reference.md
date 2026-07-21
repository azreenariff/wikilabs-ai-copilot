# Linux Engineering — Test Reference

## Purpose

This directory contains test specifications and validation criteria for the Linux engineering skill pack.

## Test Categories

### 1. Detection Rule Tests
Verify that detection rules correctly identify Linux-related issues:
- Service failure detection
- Performance issue detection
- Network issue detection
- Storage issue detection
- Security issue detection

### 2. Workflow Tests
Verify that workflows complete successfully:
- Evidence collection produces valid output
- Diagnosis identifies correct root cause
- Remediation applies appropriate fix
- Verification confirms resolution

### 3. Command Tests
Verify that commands execute correctly:
- Read-only commands return expected output
- Write commands require confirmation
- Privileged commands require root
- Commands handle error cases gracefully

### 4. Knowledge Tests
Verify that knowledge references are accurate:
- Architecture descriptions are correct
- Security procedures are safe
- Network configuration is standard
- Storage operations are documented

## Test Examples

### Detection Rule Test
```yaml
test: linux-service-failure detection
input: "nginx service failed to start after update"
expected:
  rule_matched: true
  rule_id: linux-service-failure
  confidence: 0.90
```

### Workflow Test
```yaml
test: service-not-starting workflow
scenario:
  service: nginx
  symptom: "failed to start"
  evidence:
    - systemctl status nginx
    - journalctl -u nginx --since "1 hour ago"
steps:
  - Collect evidence
  - Diagnose root cause
  - Apply remediation
  - Verify resolution
expected:
  workflow_complete: true
  service_running: true
```

### Command Test
```yaml
test: systemctl status returns valid output
command: systemctl status nginx
expected:
  exit_code: 0
  output_contains:
    - "Active:"
    - "Loaded:"
```

## Test Coverage Requirements

| Component | Minimum Coverage |
|-----------|-----------------|
| Detection Rules | 100% of rules tested |
| Workflows | 100% of workflows tested |
| Commands | 50% of commands tested |
| Knowledge | 100% of references validated |

## Running Tests

Tests are validated through:
1. Manual execution of commands in test environment
2. Review of workflow documentation
3. Validation of detection rules against known issues
4. Cross-reference with Engineering Foundations

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial test reference |