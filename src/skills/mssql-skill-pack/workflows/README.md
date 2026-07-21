# SQL Server Workflows Reference

## Overview

This directory contains workflow documentation for SQL Server troubleshooting and operational procedures. Each workflow follows the STANDARD WORKFLOW MODEL with defined states, transitions, decision points, and exit conditions.

## Workflow Categories

### Diagnostic Workflows

Diagnostic workflows guide the analyst through systematic problem-solving:

| Workflow | ID | Use Case |
|----------|----|----------|
| Performance Degradation | WP-001 | Overall query slowdown |
| Blocking Investigation | WP-002 | Application latency due to blocking |
| TempDB Contention | WP-003 | TempDB space or latch issues |
| Memory Pressure | WP-004 | Buffer pool and memory grant issues |
| Plan Regression | WP-005 | Query plan changes causing slowdown |
| Always On Failover | WP-006 | Availability group replica failure |
| Replication Lag | WP-007 | Replication synchronization issues |
| Backup Failure | WP-008 | Backup jobs failing or not completing |

### Operational Workflows

Operational workflows guide routine database administration:

| Workflow | ID | Use Case |
|----------|----|----------|
| Database Creation | WO-001 | New database provisioning |
| Index Maintenance | WO-002 | Regular index optimization |
| Statistics Update | WO-003 | Statistics refresh |
| Log Backup | WO-004 | Transaction log backup operation |
| Patching | WO-005 | SQL Server version update |
| Security Review | WO-006 | Permission audit |

## Workflow Structure

Each workflow follows this structure:

1. **Preconditions** — Required state before starting
2. **Entry Criteria** — When to initiate this workflow
3. **State Machine** — States and transitions
4. **Decision Points** — Branching logic
5. **Exit Criteria** — When the workflow completes
6. **Escalation Conditions** — When to escalate
7. **Verification** — Post-completion validation

## Usage Guidelines

- Each workflow is advisory — do not execute commands without explicit authorization
- Always assess risk before recommending any action
- Include confidence scoring in all recommendations
- Document all findings in the session history
- Verify results after each workflow completion

## Cross-Workflow References

- **Performance Degradation (WP-001)** → may branch to WP-003, WP-004, WP-005
- **Blocking (WP-002)** → may branch to WP-003 (TempDB contention)
- **Always On Failover (WP-006)** → may branch to WP-008 (backup after failover)
- **Plan Regression (WP-005)** → may branch to WO-003 (statistics update)