# VMware vSphere — Examples Reference

## Purpose

This directory contains examples and worked troubleshooting scenarios for VMware vSphere infrastructure.

## Directory Structure

| Directory | Content |
|-----------|---------|
| `./` | General examples and reference |
| `./work-examples.md` | Detailed worked troubleshooting scenarios |

## Example Categories

1. **vCenter Troubleshooting** — vpxd failure, SSO issues, certificate expiry, storage full
2. **Host Troubleshooting** — ESXi crash (PSOD), host disconnection, management network loss
3. **VM Troubleshooting** — VM slow, disk full, VMtools issues, snapshot problems
4. **Storage Troubleshooting** — Datastore full, LUN loss, VMFS corruption, vSAN issues
5. **Network Troubleshooting** — vSwitch misconfiguration, VLAN issues, vMotion network failure
6. **Cluster Troubleshooting** — HA failure, DRS imbalance, admission control issues
7. **Backup Troubleshooting** — Snapshot consolidation failure, backup job errors

## Writing Examples

Each worked example should include:
- **Scenario** — realistic problem statement
- **Symptoms** — what users/operators observed
- **Evidence** — key diagnostic commands and output
- **Analysis** — step-by-step root cause identification
- **Resolution** — the fix applied
- **Verification** — confirmation the fix worked
- **Lessons** — key takeaways