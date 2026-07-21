# SQL Server Engineering Skill Pack

## Overview

This skill pack provides comprehensive engineering knowledge, reasoning, and guidance for Microsoft SQL Server 2017, 2019, and 2022.

## Purpose

Wiki Labs AI Copilot is an engineering advisor. It never executes work. It helps engineers think by recommending, explaining, and guiding — while the engineer performs every action.

## Coverage

- **Core Engine Architecture**: Buffer pool, query processor, storage engine, lock manager
- **T-SQL**: Procedural programming, error handling, window functions, JSON, CTEs, dynamic SQL
- **Always On Availability Groups**: Automatic failover, synchronous/asynchronous replicas, read-only routing
- **Failover Cluster Instances**: Windows Server failover clustering for SQL Server
- **Backup and Recovery**: Full, differential, transaction log backups, PITR, page restore
- **Indexes and Statistics**: Clustered/nonclustered, columnstore, filtered indexes, statistics management
- **Execution Plans**: Query plan analysis, missing index recommendations, cost analysis
- **Transaction Processing**: Isolation levels, deadlock detection, distributed transactions
- **Performance Monitoring**: DMVs, DMFs, wait statistics, query store, Extended Events
- **Memory Management**: Buffer pool, memory grants, memory clerks, max server memory
- **TempDB Management**: Configuration, contention, version store, sizing strategies
- **Maintenance Plans**: Index rebuild, reorganize, statistics update, DBCC CHECKDB
- **Security**: Logins, roles, permissions, TDE, ECP, BCE, row-level security, auditing

## Quality Standards

This skill pack implements the reference standard for all future technology skill packs. It is evaluated against:

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
mssql-skill-pack/
    manifest.yaml                 # Skill pack metadata and configuration
    technology.yaml               # Technology coverage and features
    detection_rules.yaml          # Context detection patterns
    commands.yaml                 # Command knowledge base
    MSSQL_SKILL_PACK.md           # Skill pack overview (this file)
    MSSQL_DETECTION.md            # Detection rules reference
    MSSQL_COMMAND_REFERENCE.md    # Command reference
    MSSQL_GUIDANCE.md             # Engineering guidance
    MSSQL_BEST_PRACTICES.md       # Best practices and standards
    MSSQL_COMMON_FAILURES.md      # Known failure patterns
    MSSQL_REASONING_GUIDE.md      # Diagnostic reasoning
    MSSQL_WORKFLOWS.md            # State-machine workflows
    MSSQL_SKILL_PACK_QUALITY_STANDARD.md  # Quality standard
    concepts/
        overview.md               # SQL Server architecture and key components
        terminology.md            # Glossary of SQL Server terms
    context/
        interpretation.md         # How to read SQL Server outputs, logs, metrics
    knowledge/
        architecture.md           # SQL Server core architecture
        availability-groups.md    # Always On Availability Groups
        backups-recovery.md       # Backup and recovery strategies
        indexes-statistics.md     # Index types, statistics, execution plans
        tempdb-memory.md          # TempDB and memory management
        security.md               # Security, permissions, encryption, auditing
    workflows/
        README.md                 # Workflow documentation index
    guidance/
        rules.md                  # Engineering guidance and safety rules
    best-practices/
        reference.md              # Best practices reference
    documentation/
        reference.md              # External documentation references
    examples/
        reference.md              # Examples index
        worked-examples.md        # Detailed worked examples
    tests/
        reference.md              # Validation tests
    references/
        reference.md              # External reference links
    architecture/
        reference.md              # SQL Server architecture details
    diagnostics/
        guide.md                  # Diagnostic procedures and troubleshooting
    reasoning/
        reference.md              # Diagnostic reasoning reference
    common-failures/
        reference.md              # Common failure reference
    detection/
        reference.md              # Detection documentation
```

## Version

Current version: 1.0.0

## Supported Environments

- SQL Server 2017 (14.x) — Standard, Enterprise, Web, Express
- SQL Server 2019 (15.x) — Standard, Enterprise, Web, Express
- SQL Server 2022 (16.x) — Standard, Enterprise, Web, Express

All guidance is version-aware where relevant differences exist between versions.

## Safety

This skill pack follows strict safety rules:
- Never execute commands — only recommend and explain
- Always warn about risks before recommending actions
- Always provide rollback strategies
- Always recommend evidence collection before diagnosis
- Always consider cascade effects of recommended actions
- Never modify memory, index, or configuration without explicit engineer approval
- Always recommend testing changes in non-production first

## References

- [Microsoft SQL Server Documentation](https://learn.microsoft.com/sql/)
- [SQL Server Always On Documentation](https://learn.microsoft.com/sql/database-engine/availability-groups/windows/always-on-availability-groups-sql-server)
- [SQL Server Backup and Restore](https://learn.microsoft.com/sql/relational-databases/backup-restore/backup-restore-of-sql-server)
- [SQL Server Performance Tuning](https://learn.microsoft.com/sql/relational-databases/performance/)
- [SQL Server Security Documentation](https://learn.microsoft.com/sql/relational-databases/security/)