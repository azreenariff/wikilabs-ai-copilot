# Windows Engineering — Examples Reference

## Purpose

This directory contains examples and worked troubleshooting scenarios for Windows Server.

## Directory Structure

| Directory | Content |
|-----------|---------|
| `./` | General examples and reference |
| `./work-examples.md` | Detailed worked troubleshooting scenarios |

## Example Categories

1. **Service Troubleshooting** — Service failures, dependency issues, permission problems
2. **AD Troubleshooting** — Domain join failure, replication issues, login problems
3. **DNS Troubleshooting** — Resolution failures, DNS service down, zone issues
4. **Network Troubleshooting** — Connectivity issues, firewall blocks, IP conflicts
5. **Storage Troubleshooting** — Disk full, NTFS issues, volume management
6. **IIS Troubleshooting** — Site down, app pool crash, configuration errors
7. **PowerShell Troubleshooting** — Execution policy, script errors, module issues
8. **Performance Troubleshooting** — CPU, memory, disk bottlenecks
9. **Update Troubleshooting** — Windows Update failures, patch issues

## Writing Examples

Each worked example should include:
- **Scenario** — realistic problem statement
- **Symptoms** — what users/operators observed
- **Evidence** — key diagnostic commands and output
- **Analysis** — step-by-step root cause identification
- **Resolution** — the fix applied
- **Verification** — confirmation the fix worked
- **Lessons** — key takeaways