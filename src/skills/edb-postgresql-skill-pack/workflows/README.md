# PostgreSQL Workflows Documentation

## Overview

This directory contains detailed state-machine troubleshooting workflows for PostgreSQL operational issues. Each workflow provides a structured approach to diagnosing and resolving problems.

## Available Workflows

See the parent directory (`EDB_POSTGRESQL_WORKFLOWS.md`) for complete workflow documentation including:

1. **Server Startup Failure** — Diagnose and resolve PostgreSQL startup issues
2. **Connection Exhaustion** — Resolve connection pool and max_connections issues
3. **Replication Lag** — Measure and reduce replication lag
4. **Disk Space Exhaustion** — Identify and free disk space
5. **Slow Query Diagnosis** — Analyze and optimize slow queries
6. **Index Bloat Resolution** — Measure and repair bloated indexes
7. **Backup and Restore Verification** — Verify backup completeness and restore capability
8. **Autovacuum Performance Issue** — Tune autovacuum for optimal performance
9. **SSL/TLS Configuration Issue** — Fix SSL/TLS certificate and configuration problems
10. **Configuration Change Impact** — Safely apply and verify configuration changes

## Workflow Structure

Each workflow follows the STANDARD WORKFLOW MODEL:

1. **State**: Current operational state
2. **Observation**: Evidence collection and measurement
3. **Decision**: Analysis of evidence and diagnosis
4. **Transition**: Next state or completion

## Confidence Scoring

Each workflow uses confidence scoring:
- ≥ 0.95: Near-certain — proceed with action
- ≥ 0.90: High confidence — proceed with action
- ≥ 0.85: Medium-high — proceed with caution
- ≥ 0.80: Medium — consider verification
- ≥ 0.70: Low — need more evidence
- < 0.70: Very low — systematic investigation needed

## Risk Classification

Workflows are classified by risk level:
- **Critical**: Potential data loss or system downtime
- **High**: Service degradation or data inconsistency
- **Medium**: Performance impact
- **Low**: Configuration or maintenance task

## Usage

Refer to `EDB_POSTGRESQL_WORKFLOWS.md` in the root directory for the complete workflow documentation with detailed step-by-step procedures.

## References

- [EDB_POSTGRESQL_WORKFLOWS.md](../../EDB_POSTGRESQL_WORKFLOWS.md)
- [PostgreSQL Troubleshooting](https://www.postgresql.org/docs/current/monitoring-stats.html)