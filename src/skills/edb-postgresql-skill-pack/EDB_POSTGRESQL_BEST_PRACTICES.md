# EDB PostgreSQL Best Practices

## Purpose

This document captures enterprise best practices for PostgreSQL operational management, covering performance, reliability, security, and maintenance.

## Core Principles

1. **Evidence First**: Always collect evidence before making changes
2. **Minimal Impact**: Choose operations with the smallest operational footprint
3. **Verify After**: Always verify changes work and have no unintended effects
4. **Document Everything**: Changes, incidents, and decisions must be documented
5. **Test First**: Changes should be tested in non-production environments when possible
6. **Rollback Ready**: Every change must have an associated rollback plan
7. **Monitor After**: Changes require monitoring to verify no adverse effects

## Performance Best Practices

### Memory Configuration

- Set `shared_buffers` to 25% of available RAM (up to 8GB for small instances, scale for larger)
- Set `effective_cache_size` to 75% of available RAM
- Set `work_mem` conservatively — multiply by max_connections to estimate total memory requirement
- Set `maintenance_work_mem` to 1GB for fast VACUUM and index builds
- Set `wal_buffers` to 64MB (PostgreSQL 13+ can auto-tune)

### Connection Management

- Use connection pooling (PgBouncer or PgBouncer) for applications
- Set `max_connections` to the actual needed value, not the default 100
- Configure PgBouncer to manage connection multiplexing (pool_mode = transaction)
- Set `idle_in_transaction_session_timeout` to prevent idle transaction accumulation

### Autovacuum Tuning

- Enable autovacuum (default, always leave enabled)
- Set `autovacuum_max_workers` based on workload (2-4 for most systems)
- Configure `autovacuum_vacuum_cost_delay` to prevent I/O contention (25ms is a reasonable default)
- Set `autovacuum_vacuum_scale_factor` lower for write-heavy tables (0.05-0.1 instead of default 0.2)
- Consider table-level autovacuum overrides for critical tables
- Monitor `pg_stat_user_tables.n_dead_tup` to detect autovacuum issues

### Index Strategy

- Create indexes on frequently filtered columns (WHERE clause columns)
- Create indexes on frequently joined columns (JOIN columns)
- Use partial indexes when queries filter on a specific subset
- Use `CREATE INDEX CONCURRENTLY` on production tables
- Remove unused indexes (those with low `idx_scan` relative to writes)
- Consider BRIN indexes for time-series data with natural ordering
- Use GIN indexes for JSONB and full-text search columns
- Use GiST indexes for range queries and spatial data
- Monitor index usage with `pg_stat_user_indexes`
- Plan index creation during maintenance windows or low-traffic periods

### Query Optimization

- Always use `EXPLAIN (ANALYZE, BUFFERS)` before and after changes
- Use `pg_stat_statements` to identify top resource consumers
- Avoid SELECT * in production queries
- Use appropriate JOIN types (INNER vs LEFT vs CROSS)
- Partition large tables (partitioning is built into PostgreSQL)
- Use materialized views for expensive-to-compute aggregations
- Regularly run ANALYZE on tables with significant data changes

## Replication Best Practices

### Streaming Replication

- Always create replication slots to prevent WAL removal
- Configure synchronous replication for critical databases (synchronous_commit = on)
- Monitor replication lag continuously
- Set `wal_keep_size` appropriately to prevent replica lag issues
- Test failover procedures regularly
- Document current LSN positions before any maintenance

### Logical Replication

- Test logical replication thoroughly before production deployment
- Monitor `pg_stat_replication` for lag on each subscription
- Use conflict resolution strategies (ignore, overwrite) appropriate for each table
- Consider row-level filtering to reduce replication overhead
- Use `pg_output` plugin for logical decoding

### Monitoring

- Check `pg_stat_replication` for all replica states
- Monitor replication lag using `pg_wal_lsn_diff(pg_current_wal_lsn(), pg_last_wal_replay_lsn())`
- Alert when replication lag exceeds SLA thresholds
- Monitor replication slot health with `pg_replication_slots`

## Backup and Recovery Best Practices

### Backup Strategy

- Maintain daily logical backups (pg_dump) of all databases
- Maintain weekly physical backups (pg_basebackup) for disaster recovery
- Keep WAL archive for point-in-time recovery capability
- Test restore procedures regularly
- Verify backup integrity after each backup

### pg_dump Best Practices

- Use `-Fc` (custom format) for flexibility and parallel restore capability
- Use `-j` (parallel jobs) for large databases to reduce backup time
- Test restore with `pg_restore` on a non-production system
- Include `--create` and `--clean` flags in restore scripts for idempotent recovery

### pg_basebackup Best Practices

- Use `-X fetch` or `-X stream` to include WAL during backup
- Use `-P` for progress reporting on large backups
- Verify backup can be restored to a standby before relying on it
- Schedule during low-traffic periods to minimize I/O impact

### PITR Planning

- Configure `archive_mode = on` and set `archive_command`
- Test full PITR scenarios quarterly
- Document current archive locations and retention policies
- Maintain multiple WAL archives for extended recovery windows

## Security Best Practices

### Authentication

- Use `scram-sha-256` for password authentication (more secure than md5)
- Configure `pg_hba.conf` with least-privilege access
- Use SSL/TLS for all remote connections
- Rotate credentials regularly
- Use roles instead of individual users for privilege management

### Access Control

- Follow least privilege principle — grant only what is needed
- Use roles to manage groups of privileges
- Separate read-only and read-write roles
- Review `pg_stat_user_tables` for unused schemas or tables
- Use `GRANT` and `REVOKE` explicitly rather than relying on defaults

### Data Protection

- Enable data checksums (`data_checksums = on` during initdb)
- Encrypt data at rest using filesystem or storage encryption
- Encrypt data in transit using SSL/TLS
- Implement proper backup encryption

### Configuration Hardening

- Set `ssl = on` in production
- Set `ssl_cert_file` and `ssl_key_file` properly
- Configure `ssl_ciphers` to exclude weak ciphers
- Set `password_encryption = scram-sha-256`

## Monitoring and Observability

### Required Views

- `pg_stat_activity` — Active sessions and queries
- `pg_stat_database` — Database-level statistics
- `pg_stat_user_tables` — Table-level statistics
- `pg_stat_replication` — Replication status
- `pg_stat_bgwriter` — Background writer stats
- `pg_stat_wal` — WAL stats (PG 14+)
- `pg_stat_archiver` — WAL archiving stats
- `pg_stat_user_indexes` — Index usage statistics

### Key Metrics to Monitor

1. **Connection count**: Compare `pg_stat_activity` count to `max_connections`
2. **Dead tuples**: Monitor `n_dead_tup` in `pg_stat_user_tables`
3. **Replication lag**: Calculate LSN differences
4. **WAL generation**: Use `pg_stat_wal`
5. **Cache hit ratio**: `blks_hit / (blks_hit + blks_read)` from `pg_stat_database`
6. **Checkpoint frequency**: `pg_stat_bgwriter`
7. **Transaction count**: `xact_commit` and `xact_rollback` from `pg_stat_database`

### Alert Thresholds

- Connections > 80% of max_connections
- Replication lag > 30 seconds (adjust based on SLA)
- Disk usage > 85% (adjust based on capacity)
- Dead tuples > 100,000 on any single table
- Cache hit ratio < 95% on OLTP workloads
- WAL archive failures: any failure count increase

## Maintenance Best Practices

### Regular Maintenance

- Monitor autovacuum activity (should handle most maintenance automatically)
- Review `pg_stat_user_tables.n_dead_tup` weekly
- Check index usage quarterly
- Review and tune autovacuum settings annually
- Test restore procedures quarterly
- Review and rotate SSL certificates
- Review and update security configurations
- Run `pg_stat_statements_reset()` after investigating performance issues
- Review and tune query performance periodically

### Capacity Planning

- Monitor database growth trends
- Monitor index size growth
- Monitor WAL archive disk usage
- Monitor pg_stat_user_tables.bloat for index bloat
- Plan capacity increases before reaching limits
- Consider tablespace placement for capacity management

## High Availability Best Practices

### Architecture

- Use at least one standby for production
- Consider synchronous replication for zero-data-loss requirements
- Implement monitoring for all nodes in the cluster
- Plan failover procedures and test them regularly
- Maintain documentation of cluster topology

### Failover Planning

- Document step-by-step failover procedures
- Test failover quarterly
- Maintain a runbook for common failure scenarios
- Ensure applications can handle failover gracefully
- Consider automated failover tools (Patroni, EDB Postgres Operator)

### Disaster Recovery

- Maintain off-site backups (cloud storage for WAL archives)
- Document recovery procedures with RTO/RPO targets
- Test full disaster recovery scenarios annually
- Maintain multiple recovery points
- Consider cross-region replication for critical systems

## Configuration Management Best Practices

### postgresql.conf

- Use `ALTER SYSTEM SET` for configuration changes (writes to postgresql.auto.conf)
- Document all configuration changes
- Use parameter files for complex configurations
- Never edit `postgresql.conf` directly in production — use `ALTER SYSTEM` or parameter files
- Test configuration changes in staging before production

### Schema Management

- Use version control for DDL scripts
- Use migration tools for production schema changes
- Test schema changes in staging first
- Use `CREATE INDEX CONCURRENTLY` for production index changes
- Plan schema changes during low-traffic periods when possible

## Incident Response Best Practices

### During an Incident

1. **Assess severity** — Use severity classification to determine priority
2. **Collect evidence** — Logs, pg_stat_activity, replication status, disk space
3. **Apply stabilization** — Fix immediate issues to prevent further damage
4. **Communicate** — Share status with stakeholders
5. **Root cause** — Determine root cause after stabilization
6. **Document** — Document the incident, root cause, and resolution

### Post-Incident

1. **Review** — What went well, what could be improved
2. **Prevent** — Implement measures to prevent recurrence
3. **Document** — Update runbooks and procedures
4. **Test** — Verify fixes work under load
5. **Share** — Share lessons learned with the team