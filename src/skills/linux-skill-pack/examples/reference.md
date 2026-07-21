# Linux Engineering — Examples Reference

## Purpose

This directory contains examples and worked troubleshooting scenarios for Linux systems.

## Directory Structure

| Directory | Content |
|-----------|---------|
| `./` | General examples and reference |
| `./work-examples.md` | Detailed worked troubleshooting scenarios |

## Example Categories

1. **Service Troubleshooting** — nginx, sshd, docker, postgres
2. **Performance** — CPU spike, memory leak, disk I/O bottleneck
3. **Network** — DNS failure, connectivity loss, firewall misconfiguration
4. **Storage** — disk full, LVM expansion, filesystem repair
5. **Security** — SSH key rotation, certificate renewal, firewall hardening

## Writing Examples

Each worked example should include:
- **Scenario** — realistic problem statement
- **Symptoms** — what users/operators observed
- **Evidence** — key diagnostic commands and output
- **Analysis** — step-by-step root cause identification
- **Resolution** — the fix applied
- **Verification** — confirmation the fix worked
- **Lessons** — key takeaways