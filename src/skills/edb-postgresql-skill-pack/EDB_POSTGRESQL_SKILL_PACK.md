# EDB PostgreSQL Engineering Skill Pack

## Overview

This skill pack provides comprehensive engineering knowledge, reasoning, and guidance for EnterpriseDB (EDB) PostgreSQL distributions — the enterprise-grade variant of the PostgreSQL database engine developed by EnterpriseDB. It covers PostgreSQL 15, 16, and 17 across all enterprise operational domains.

## Purpose

Wiki Labs AI Copilot is an engineering advisor. It never executes work. It helps engineers think by recommending, explaining, and guiding — while the engineer performs every action. This skill pack encodes deep PostgreSQL engineering knowledge so the AI can reason about database issues at an expert level.

## Coverage

- **PostgreSQL Core Server**: Architecture, background processes, shared memory, server lifecycle management (pg_ctl, pg_ctlcluster), initialization (initdb), health monitoring (pg_isready)
- **EDB-Specific Features**: EnterpriseDB extensions, Oracle compatibility, EDB configuration options, EDB-specific HA tools
- **WAL Management**: Write-Ahead Logging configuration, checkpointing, WAL retention, WAL archiving, WAL segment management, pg_resetwal, pg_test_fsync
- **Streaming Replication**: Physical streaming replication, replication slots, synchronous replication, lag monitoring, pg_is_in_recovery(), failover procedures, pg_rewind
- **Physical Replication**: pg_basebackup, standby configuration, recovery.conf (pre-12) vs standby.signal (12+), hot standby queries, pg_rewind
- **Logical Replication**: Publication/subscription model, logical decoding, table-level replication, conflict resolution
- **Backup and Recovery**: pg_dump, pg_dumpall, pg_restore, pg_basebackup, PITR, WAL archiving, backup validation, restore procedures
- **Indexes**: B-Tree, GiST, GIN, BRIN, CREATE INDEX CONCURRENTLY, REINDEX CONCURRENTLY (PG 15+), pg_stat_user_indexes
- **Locks and Concurrency**: Row-level locking, table-level locks (7 lock modes), lock timeouts, deadlock detection and resolution, pg_locks, pg_blocking_pids()
- **Transactions and MVCC**: Transaction isolation levels, snapshot isolation, prepared transactions, xid wraparound prevention, txid functions
- **Configuration**: postgresql.conf tuning, ALTER SYSTEM, shared_buffers, work_mem, maintenance_work_mem, effective_cache_size, max_connections, wal_buffers
- **Performance Tuning**: Query optimization, EXPLAIN ANALYZE, pg_stat_statements, autovacuum tuning, pg_stat_activity, pg_wait_events
- **Autovacuum**: autovacuum settings, table-level tuning, vacuum delays, freeze settings, bloat management
- **Monitoring and Statistics**: pg_stat_activity, pg_stat_database, pg_stat_user_tables, pg_stat_replication, pg_stat_bgwriter, pg_stat_wal, pg_stat_progress_*

## Quality Standards

This skill pack implements the reference standard for all future database technology skill packs. It is evaluated against:

- Knowledge coverage
- Workflow coverage
- Reasoning coverage
- Detection coverage
- Guidance quality
- Safety
- Documentation
- Examples
- Testing
- Maintainability

## Structure

```
edb-postgresql-skill-pack/
    manifest.yaml                # Skill pack metadata and configuration
    technology.yaml              # Technology coverage and features
    detection_rules.yaml         # Context detection patterns
    commands.yaml                # Command knowledge base
    EDB_POSTGRESQL_SKILL_PACK.md         # Skill pack overview (this file)
    EDB_POSTGRESQL_COMMAND_REFERENCE.md  # Command reference
    EDB_POSTGRESQL_DETECTION.md          # Detection rules reference
    EDB_POSTGRESQL_GUIDANCE.md           # Engineering guidance
    EDB_POSTGRESQL_BEST_PRACTICES.md     # Best practices
    EDB_POSTGRESQL_COMMON_FAILURES.md    # Known failure patterns
    EDB_POSTGRESQL_REASONING_GUIDE.md    # Diagnostic reasoning
    EDB_POSTGRESQL_WORKFLOWS.md          # State-machine workflows
    EDB_POSTGRESQL_SKILL_PACK_QUALITY_STANDARD.md  # Quality standard
    concepts/
        overview.md              # Architecture and key components
        terminology.md           # Glossary of terms
    context/
        interpretation.md        # How to read outputs, logs, metrics
    knowledge/
        architecture.md          # Core architecture
        replication.md           # Streaming and logical replication
        wal.md                   # WAL configuration and management
        performance-optimization.md  # Query optimization and tuning
        backup-recovery.md       # Backup and PITR
        security.md              # Users, roles, SSL, pg_hba.conf
    workflows/
        README.md                # Workflow documentation index
    guidance/
        rules.md                 # Engineering guidance and safety rules
    best-practices/
        reference.md             # Best practices reference
    documentation/
        reference.md             # External documentation references
    examples/
        reference.md             # Examples index
        worked-examples.md       # Detailed worked examples (3+)
    tests/
        reference.md             # Validation tests
    references/
        reference.md             # External reference links
    architecture/
        reference.md             # Architecture details
    diagnostics/
        guide.md                 # Diagnostic procedures
    reasoning/
        reference.md             # Diagnostic reasoning reference
    common-failures/
        reference.md             # Common failure patterns reference
```

## Version

Current version: 1.0.0

## Supported Environments

- PostgreSQL 15 (EDB Postgres Advanced Server)
- PostgreSQL 16 (EDB Postgres Advanced Server)
- PostgreSQL 17 (EDB Postgres Advanced Server)

## Safety

This skill pack follows strict safety rules:

- Never execute commands — only recommend and explain
- Always warn about risks before recommending actions
- Always provide rollback strategies
- Always recommend evidence collection before diagnosis
- Always consider cascade effects of recommended actions
- Never modify configuration without explicit engineer approval
- Always recommend configuration validation before reload
- Always consider the impact on replication and backup integrity

## References

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [EnterpriseDB Documentation](https://www.enterprisedb.com/docs/)
- [PostgreSQL 15 Release Notes](https://www.postgresql.org/about/featurelist/article/postgresql-15-feature-list/)
- [PostgreSQL 16 Release Notes](https://www.postgresql.org/about/featurelist/article/postgresql-16-feature-list/)
- [PostgreSQL 17 Release Notes](https://www.postgresql.org/about/featurelist/article/postgresql-17-feature-list/)
- [PgBouncer Documentation](https://www.pgbouncer.org/)