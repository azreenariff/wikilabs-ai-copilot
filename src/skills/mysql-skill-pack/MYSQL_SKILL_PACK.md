# MySQL Engineering Skill Pack

## Overview

This skill pack provides comprehensive engineering knowledge, reasoning, and guidance for MySQL database server versions 8.0 and 8.4, covering storage engines, replication, performance optimization, security, backup and recovery, and enterprise high-availability patterns.

## Purpose

Wiki Labs AI Copilot is an engineering advisor. It never executes work. It helps engineers think by recommending, explaining, and guiding — while the engineer performs every action.

## Coverage

- **MySQL Core Server**: mysqld daemon architecture, query optimizer, thread management, data dictionary, server startup and configuration
- **Storage Engines**: InnoDB (default transactional engine), MyISAM, Memory, Archive, Blackhole, and Federated engines
- **Replication**: Master-Slave, Master-Master, Group Replication (MGR), MySQL Shell, replication filters, GTID-based replication
- **Performance Schema**: Performance Schema instrumentation, sys schema, statement analysis, index usage monitoring, lock waits
- **Configuration Management**: my.cnf, my.ini, server variables, persistent variables via SET PERSIST, MySQL Config Editor
- **Security**: Users, privileges, roles, SSL/TLS, authentication plugins, password policies, resource limits, RBAC
- **Backup and Recovery**: mysqldump, mysqlpump, binary logs, point-in-time recovery, Percona XtraBackup
- **Information Schema**: Schema metadata queries, table statistics, index usage, locked tables, session information
- **Query Optimization**: EXPLAIN, EXPLAIN ANALYZE, optimizer trace, index design, query plan analysis
- **High Availability**: MySQL Router, InnoDB Cluster, ProxySQL, Keepalived, read/write splitting
- **Monitoring**: Performance Schema, sys schema, slow query log, general log, audit log
- **Data Dictionary**: MySQL 8.0+ internal data dictionary, hidden columns, JSON schema
- **Partitioning**: RANGE, LIST, HASH, KEY partitioning; partition maintenance
- **Troubleshooting**: Connection issues, replication lag, deadlocks, lock waits, query optimization

## Quality Standards

This skill pack implements the reference standard for MySQL engineering knowledge. It is evaluated against:

- Knowledge coverage (storage engines, replication, performance, security, backup, HA)
- Workflow coverage (8+ state-machine troubleshooting workflows)
- Reasoning coverage (decision trees for diagnostic scenarios)
- Detection coverage (47+ detection rules across CLI, browser, window, and text patterns)
- Command coverage (147+ command entries across 10 categories)
- Guidance quality (engineering rules, safety protocols, confidence scoring)
- Safety (never recommend command execution — only advisory guidance)
- Documentation (external references to official MySQL documentation)
- Examples (3+ detailed worked examples)
- Testing (validation tests and quality assurance procedures)

## Structure

```
mysql-skill-pack/
    manifest.yaml                  # Skill pack metadata and configuration
    technology.yaml                # Technology coverage and features
    detection_rules.yaml           # Context detection patterns
    commands.yaml                  # Command knowledge base
    MYSQL_SKILL_PACK.md            # Skill pack overview (this file)
    MYSQL_COMMAND_REFERENCE.md     # Command reference
    MYSQL_DETECTION.md             # Detection rules reference
    MYSQL_GUIDANCE.md              # Engineering guidance
    MYSQL_BEST_PRACTICES.md        # Best practices and standards
    MYSQL_COMMON_FAILURES.md       # Known failure patterns
    MYSQL_REASONING_GUIDE.md       # Diagnostic reasoning
    MYSQL_WORKFLOWS.md             # State-machine workflows
    MYSQL_SKILL_PACK_QUALITY_STANDARD.md  # Quality standard
    architecture/reference.md      # Architecture details
    concepts/
        overview.md                # Architecture and key components
        terminology.md             # Glossary
    context/
        interpretation.md          # Context interpretation
    knowledge/
        storage-engines.md         # InnoDB, MyISAM, Archive, Memory engines
        replication.md             # Replication topologies, MGR, MySQL Shell
        performance-optimization.md  # Query optimization, indexes, Perf Schema
        backup-recovery.md         # mysqldump, mysqlpump, xtrabackup, binary logs
        security.md                # Users, privileges, SSL/TLS, authentication
    workflows/
        README.md                  # Workflow documentation index
    guidance/
        rules.md                   # Engineering guidance and safety rules
    best-practices/
        reference.md               # Best practices reference
    documentation/
        reference.md               # External documentation references
    examples/
        reference.md               # Examples index
        worked-examples.md         # Detailed worked examples
    tests/
        reference.md               # Validation tests
    references/
        reference.md               # External reference links
    diagnostics/
        guide.md                   # Diagnostic procedures and troubleshooting
    reasoning/
        reference.md               # Diagnostic reasoning reference
    common-failures/
        reference.md               # Common failure reference
    detection/
        reference.md               # Detection documentation
```

## Version

Current version: 1.0.0

## Supported Environments

- MySQL Server 8.0 (standard supported version)
- MySQL Server 8.4 (LTS version)
- Percona Server for MySQL 8.0 (compatible)
- MariaDB 10.6+ (compatible modes, differences noted)

## Safety

This skill pack follows strict safety rules:

- Never execute commands — only recommend and explain
- Always warn about risks before recommending actions
- Always provide rollback strategies for destructive operations
- Always recommend evidence collection before diagnosis
- Always consider cascade effects of recommended actions
- Never modify configuration without explicit engineer approval
- Always recommend configuration validation before reload
- Include confidence scoring in all recommendations
- Include risk warnings in all high-risk operations

## References

- [MySQL 8.0 Documentation](https://dev.mysql.com/doc/refman/8.0/en/)
- [MySQL 8.4 Documentation](https://dev.mysql.com/doc/refman/8.4/en/)
- [MySQL Internals Manual](https://dev.mysql.com/doc/internals/en/)
- [Percona XtraBackup Documentation](https://www.percona.com/doc/percona-xtrabackup/LATEST/index.html)
- [MySQL Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html)
- [MySQL Sys Schema Reference](https://dev.mysql.com/doc/refman/8.0/en/sys-schema.html)
- [MySQL Replication Documentation](https://dev.mysql.com/doc/refman/8.0/en/replication.html)