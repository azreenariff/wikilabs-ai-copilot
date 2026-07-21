# EDB PostgreSQL Detection Rules Reference

## Purpose

This document describes the detection rules used by the EDB PostgreSQL skill pack to identify context, symptoms, and issues from terminal activity, browser usage, window titles, and text patterns in logs and output.

## Detection Rule Inventory

### CLI Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| edb-detect-psql | psql CLI usage | `^psql(\s+|$)` | 0.95 | 10 |
| edb-detect-pg-dump | pg_dump backup usage | `^pg_dump(\s+|$)` | 0.95 | 9 |
| edb-detect-pg-restore | pg_restore restore usage | `^pg_restore(\s+|$)` | 0.95 | 9 |
| edb-detect-pg-basebackup | pg_basebackup physical backup | `^pg_basebackup(\s+|$)` | 0.95 | 9 |
| edb-detect-pg-ctl | pg_ctl server control | `^pg_ctl\s+(start|stop|restart|reload|status|promote)` | 0.95 | 10 |
| edb-detect-pgbouncer | PgBouncer connection pooler | `^pgbouncer(\s+|$)|pgbouncer\s+.*\.ini` | 0.90 | 8 |
| edb-detect-pg-is-in-recovery | Replication/recovery queries | `pg_is_in_recovery\(\)` | 0.90 | 8 |
| edb-detect-pg-replication-slots | Replication slot operations | `pg_replication_slots|pg_create_physical_replication_slot` | 0.90 | 8 |
| edb-detect-pg-stat-statements | pg_stat_statements queries | `pg_stat_statements|pg_stat_statements_reset` | 0.90 | 8 |
| edb-detect-vacuum | VACUUM and autovacuum ops | `^\s*VACUUM\s|^\s*autovacuum|vacuum_full|vacuum_freeze` | 0.90 | 8 |
| edb-detect-explain-analyze | EXPLAIN ANALYZE queries | `^\s*EXPLAIN\s+(ANALYZE|BUFFERS|VERBOSE)?\s+(ANALYZE|BUFFERS|VERBOSE)?\s+SELECT` | 0.90 | 8 |
| edb-detect-create-index-concurrently | CREATE INDEX CONCURRENTLY | `CREATE\s+INDEX\s+CONCURRENTLY` | 0.90 | 8 |
| edb-detect-alter-system | ALTER SYSTEM SET | `^\s*ALTER\s+SYSTEM\s+SET` | 0.85 | 7 |
| edb-detect-pg-wal | WAL operations | `pg_switch_wal|pg_current_wal_insert_lsn|pg_walfile_name` | 0.90 | 8 |
| edb-detect-pg-rewind | pg_rewind operations | `^pg_rewind(\s+|$)` | 0.90 | 9 |
| edb-detect-pub-sub | Logical replication pub/sub | `CREATE\s+PUBLICATION|CREATE\s+SUBSCRIPTION|ALTER\s+PUBLICATION` | 0.90 | 8 |
| edb-detect-initdb | initdb cluster initialization | `^initdb(\s+|$)` | 0.95 | 10 |
| edb-detect-pg-resetwal | pg_resetwal emergency repair | `^pg_resetwal(\s+|$)` | 0.90 | 10 |

### Web Interface Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| edb-detect-pgadmin | pgAdmin web interface | `pgadmin|edb-admin|pg-admin|edb-postgres|cloudmanager` | 0.90 | 8 |
| edb-detect-pgadmin-window | pgAdmin window title | `(?i)pgAdmin|EDB\s+PostgreSQL|PostgreSQL.*Admin` | 0.85 | 7 |
| edb-detect-edb-cloudmanager | EDB Cloud Manager | `edb-cloud-manager|enterprise\s+db\s+cloud\s+manager|cloud\.enterprisedb\.com` | 0.90 | 8 |

### Text Pattern Detection — Errors

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| edb-detect-fatal-error | FATAL/ERROR messages | `(?i)\bFATAL\b|\bERROR\b` | 0.95 | 10 |
| edb-detect-deadlock | Deadlock errors | `(?i)deadlock\s+detected|deadlock\s+timeout` | 0.95 | 10 |
| edb-detect-connection-limit | Connection limit reached | `(?i)too\s+many\s+connections|sorry\s+too\s+many\s+connections` | 0.90 | 10 |
| edb-disk-space | Disk space warnings | `(?i)disk\s+full|no\s+space\s+left|failing\s+to\s+allocate` | 0.95 | 10 |
| edb-detect-data-corruption | Data corruption indicators | `(?i)corrupted\s+block|invalid\s+page|checksum\s+failed` | 0.95 | 10 |
| edb-detect-xid-wraparound | Transaction ID wraparound | `(?i)xid\s+wraparound|transaction\s+id\s+wraparound|freeze.*age` | 0.95 | 10 |
| edb-detect-lock-timeout | Lock timeout/blocking | `(?i)lock\s+timeout|statement\s+timeout|canceling\s+statement` | 0.90 | 9 |
| edb-detect-postmaster | Postmaster process issues | `(?i)postmaster|could\s+not\s+create\s+postmaster` | 0.85 | 8 |

### Text Pattern Detection — Operational

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| edb-detect-replication-lag | Replication lag issues | `(?i)replication\s+lag|replication\s+delay|streaming\s+replication.*lag` | 0.90 | 9 |
| edb-detect-wal-archiving | WAL archiving status | `(?i)wal.*archive|archive_command|wal_level|archiving.*failed` | 0.85 | 8 |
| edb-detect-checkpoint | Checkpoint messages | `(?i)checkpoint\s+(beginning|complete|starting)|log_checkpoints` | 0.85 | 7 |
| edb-detect-vacuum-progress | Autovacuum activity | `(?i)automatic\s+vacuum.*table|autovacuum.*running` | 0.85 | 7 |
| edb-detect-table-bloat | Table bloat indicators | `(?i)table\s+bloat|dead\s+tuples|n_dead_tup` | 0.80 | 7 |
| edb-detect-ssl-tls | SSL/TLS configuration | `(?i)ssl\s+is\s+not\s+enabled|ssl\s+error|ssl_certificate` | 0.80 | 7 |
| edb-detect-max-connections | Max connections rejected | `(?i)sorry\s+too\s+many\s+connections.*already|connection\s+request\s+rejected` | 0.90 | 10 |

## Confidence Scoring

### Confidence Levels

- **0.95+**: Near-certain match — strong pattern indicators
- **0.90-0.94**: High confidence — clear indicators
- **0.85-0.89**: Medium-high confidence — strong indicators
- **0.80-0.84**: Medium confidence — reasonable indicators
- **0.70-0.79**: Low-medium confidence — suggestive indicators
- **< 0.70**: Low confidence — weak indicators

### Confidence Adjustments

Context can increase confidence:
- CLI commands + text patterns about replication = higher replication confidence
- Multiple matching rules = higher overall context confidence
- Consistent database/role references = higher context accuracy

## Priority Levels

Priority determines which detection rules take precedence when multiple rules match:

- **10**: Critical — immediate attention required
- **9**: High — important context
- **8**: Medium — relevant context
- **7**: Low — supporting context
- **< 7**: Informational — supplementary context

## Pattern Extraction

Some rules extract specific values for use in diagnostics:

| Rule ID | Extract Pattern | Example |
|---------|----------------|---------|
| edb-detect-psql | `psql -d (.+)` | `mydb` |
| edb-detect-pg-ctl | `pg_ctl -D (.+)` | `/var/lib/pgsql/data` |
| edb-detect-pg-dump | `pg_dump (.+)` | `mydb` |
| edb-detect-replication-lag | `sent_lsn=([^,]+)` | `0/A1234567` |
| edb-detect-pg-basebackup | `pg_basebackup -D (.+)` | `/backup/pg_base` |

## Rule Management

### Adding New Rules

When adding new detection rules, follow these guidelines:

1. **Generate unique rule ID**: Use `edb-detect-<category>-<specific>` format
2. **Choose detection type**: Command, Browser, WindowTitle, or TextPattern
3. **Set appropriate confidence**: Base on pattern specificity and reliability
4. **Assign priority**: Match the operational significance of the pattern
5. **Include extract patterns**: If the pattern captures actionable identifiers
6. **Test pattern against real outputs**: Verify with actual PostgreSQL logs, pg_dump output, and psql sessions
7. **Document in this reference**: Add to the appropriate table above

### Removing Stale Rules

When removing detection rules:

1. **Deprecate first**: Mark confidence to 0.0 temporarily
2. **Monitor**: Ensure no new symptoms are missed
3. **Remove from this reference**: Update tables
4. **Document deprecation reason**: Note what replaced it