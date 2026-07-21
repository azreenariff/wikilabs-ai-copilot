# Engineering Guidance and Safety Rules

## Purpose

This document defines the engineering guidance and safety rules for PostgreSQL operations. It establishes the principles and constraints that guide all diagnostic and operational activities.

## Core Principles

### 1. Evidence-First Approach

Always collect evidence before making changes. The evidence collection checklist:

- [ ] PostgreSQL log file reviewed for errors and warnings
- [ ] `pg_isready` status checked
- [ ] `pg_stat_activity` examined for active sessions
- [ ] Disk space verified (system + data directory)
- [ ] Replication status checked (if applicable)
- [ ] Lock status examined (if performance issue)
- [ ] Configuration values checked (`SHOW ALL`)
- [ ] Backup status verified (if backup-related issue)
- [ ] Recent changes reviewed (config, DDL, deployment)
- [ ] External factors assessed (network, storage, OS)

### 2. Minimal Impact Operations

Choose operations with the smallest operational footprint:

- Prefer `VACUUM` over `VACUUM FULL` (VACUUM FULL requires exclusive lock)
- Prefer `CREATE INDEX CONCURRENTLY` over `CREATE INDEX` (no blocking writes)
- Prefer `REINDEX CONCURRENTLY` (PostgreSQL 15+) over `REINDEX`
- Prefer `ALTER SYSTEM SET` over editing `postgresql.conf` directly
- Prefer `pg_reload_conf()` over server restart
- Use connection pooling to avoid connection storms
- Schedule maintenance during low-traffic periods

### 3. Verify After Changes

Always verify that changes work and have no unintended effects:

1. Verify parameter change: `SHOW parameter_name`
2. Check PostgreSQL log for errors after change
3. Monitor `pg_stat_activity` for query performance
4. Check replication status
5. Monitor for 15 minutes after change
6. Document the change and its effects

### 4. Document Everything

All changes, incidents, and decisions must be documented:

- What was changed
- Why it was changed
- When it was changed
- Who approved the change
- What the impact was
- What the rollback plan is

### 5. Test in Staging

Changes should be tested in non-production environments:

- Configuration changes → Test in staging
- Schema changes → Test in staging
- Backup/restore procedures → Test in staging
- Replication changes → Test in staging
- Performance tuning → Test in staging

### 6. Rollback Ready

Every change must have an associated rollback plan:

- Document the original configuration
- Take a backup before significant changes
- Have the rollback commands ready
- Test the rollback procedure

## Safety Rules

### Critical Operations

The following operations are classified as CRITICAL and require special care:

1. **VACUUM FULL** — Requires exclusive lock on table
   - Risk: Blocks all access to the table
   - Mitigation: Schedule during maintenance window, use pg_repack for online option

2. **DROP TABLE/DATABASE** — Irreversible data loss
   - Risk: Permanent data loss
   - Mitigation: Verify table/database name, confirm with backup

3. **pg_resetwal** — Emergency recovery tool
   - Risk: Can lose uncommitted transactions
   - Mitigation: Only use when no other option, after expert consultation

4. **ALTER SYSTEM RESET** — Removes configuration override
   - Risk: May revert important configuration
   - Mitigation: Document current value before reset

5. **DROP REPLICATION SLOT** — May cause WAL accumulation
   - Risk: WAL accumulation if slot was protecting WAL for a standby
   - Mitigation: Verify slot is not needed before dropping

### High-Risk Operations

The following operations are classified as HIGH RISK:

1. **REINDEX** — Can lock the table (not CONCURRENTLY)
   - Risk: Temporary blocking of writes
   - Mitigation: Use REINDEX CONCURRENTLY (PG 15+) or schedule during maintenance

2. **ALTER TABLE** — Can require AccessExclusiveLock
   - Risk: Blocks all access during the operation
   - Mitigation: Test in staging, schedule during low-traffic

3. **Changing max_connections** — Affects all connections
   - Risk: May require PostgreSQL restart
   - Mitigation: Test impact on connection pool

4. **WAL archiving configuration changes** — Affects PITR capability
   - Risk: May break replication or recovery
   - Mitigation: Test archive_command after change

### Medium-Risk Operations

1. **Autovacuum tuning** — May affect maintenance performance
2. **Work_mem changes** — May affect memory usage
3. **Index creation** — May consume disk space and I/O
4. **Configuration changes** — May affect performance

### Low-Risk Operations

1. **CREATE INDEX CONCURRENTLY** — No blocking
2. **ANALYZE** — Read-only, no locks
3. **VIEW queries** — Read-only operations
4. **SHOW commands** — Read-only configuration inspection

## Response Format

When responding to issues, use the following format:

```
## Assessment
- **Severity**: [Critical | High | Medium | Low]
- **Confidence**: [0.00 - 1.00]
- **Root Cause**: [Description]

## Evidence
1. [Evidence point 1]
2. [Evidence point 2]

## Recommendations
1. [Action 1] — Confidence: 0.XX, Risk: [Low | Medium | High | Critical]
2. [Action 2] — Confidence: 0.XX, Risk: [Low | Medium | High | Critical]

## Rollback Plan
[Describe how to reverse the recommended actions]

## References
- [Documentation links]
```

## Severity Classification

| Severity | Criteria | Response Time |
|----------|----------|---------------|
| Critical | Data loss, system down, security breach | Immediate |
| High | Service degradation, replication failure | Within 1 hour |
| Medium | Performance impact, configuration issue | Within 4 hours |
| Low | Advisory, documentation, monitoring | Within 24 hours |

## Version-Aware Guidance

### PostgreSQL 15
- `REINDEX CONCURRENTLY` is available (no exclusive table lock)
- `pg_basebackup -Fd -j N` for parallel physical backup
- `pg_restore --jobs=N` for parallel logical restore
- Improved parallel query and partition pruning

### PostgreSQL 16
- Resource group management
- Logical replication improvements
- Better query plan caching
- Enhanced maintenance operations

### PostgreSQL 17
- Further query optimization improvements
- Enhanced logical replication performance
- Improved parallel planning
- Better cost estimation

## EDB-Specific Guidance

EDB Postgres Advanced Server adds:
- Oracle compatibility features
- EDB Replicator for enterprise replication
- Resource Manager for query control
- EDB Postgres Operator for Kubernetes deployment

When working with EDB features:
- Document EDB-specific configurations
- Reference EDB documentation alongside PostgreSQL documentation
- Test EDB features in staging before production
- Understand the difference between PostgreSQL native and EDB-extended features

## References

- [PostgreSQL Administration Guide](https://www.postgresql.org/docs/current/admin-guide.html)
- [PostgreSQL Security Guide](https://www.postgresql.org/docs/current/security.html)
- [PostgreSQL Performance Guide](https://www.postgresql.org/docs/current/performance-tips.html)
- [EDB Documentation](https://www.enterprisedb.com/docs/)