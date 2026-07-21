# PostgreSQL Best Practices Reference

## Purpose

This document provides best practices for PostgreSQL operational management, organized by topic area.

## Configuration Best Practices

1. **Always use ALTER SYSTEM SET** for configuration changes
2. **Set effective_cache_size to 75% of RAM**
3. **Set shared_buffers to 25% of RAM** (up to 8GB effective)
4. **Enable data checksums** at cluster initialization
5. **Use scram-sha-256** for password encryption
6. **Enable SSL/TLS** for all remote connections
7. **Set appropriate work_mem** — conservative default of 4MB
8. **Set maintenance_work_mem to 1GB** for efficient maintenance
9. **Enable logging_collector** for centralized logging
10. **Set idle_in_transaction_session_timeout** to prevent idle transaction accumulation

## Performance Best Practices

1. **Use EXPLAIN (ANALYZE, BUFFERS)** before and after changes
2. **Create indexes on frequently filtered columns**
3. **Use partial indexes** for queries with WHERE clauses
4. **Use BRIN indexes** for time-series data with natural ordering
5. **Use GIN indexes** for JSONB and full-text search
6. **Remove unused indexes** (those with low idx_scan)
7. **Create indexes CONCURRENTLY** on production tables
8. **Monitor pg_stat_statements** for top resource consumers
9. **Partition large tables** for better query and maintenance performance
10. **Use materialized views** for expensive-to-compute aggregations

## Autovacuum Best Practices

1. **Always leave autovacuum enabled** (default is on)
2. **Set autovacuum_max_workers based on workload** (2-4 for most)
3. **Increase autovacuum_vacuum_cost_limit** for write-heavy systems
4. **Lower autovacuum_vacuum_scale_factor** for write-heavy tables
5. **Use table-level overrides** for critical tables
6. **Monitor n_dead_tup** for autovacuum health
7. **Check for autovacuum failures** in logs regularly

## Replication Best Practices

1. **Always create replication slots** to prevent WAL removal
2. **Monitor replication lag** continuously
3. **Use synchronous replication** for critical data
4. **Test failover procedures** regularly
5. **Use pg_rewind** for post-failover recovery
6. **Configure archive_mode** for PITR capability
7. **Monitor pg_stat_replication** for replica health
8. **Document current LSN positions** before maintenance

## Backup Best Practices

1. **Maintain daily logical backups** (pg_dump)
2. **Maintain weekly physical backups** (pg_basebackup)
3. **Keep WAL archive** for PITR capability
4. **Test restore procedures** regularly
5. **Verify backup integrity** after each backup
6. **Use custom format** (-Fc) for flexibility
7. **Use parallel backup** (-j flag) for large databases
8. **Include --create and --clean** in restore scripts

## Security Best Practices

1. **Use least privilege** principle for roles
2. **Separate read-only and read-write roles**
3. **Use pg_hba.conf** with specific user/database restrictions
4. **Enable SSL/TLS** for remote connections
5. **Rotate credentials** regularly
6. **Review privileges** quarterly
7. **Use schemas** for access isolation
8. **Restrict listen_addresses** to known networks
9. **Monitor authentication failures**
10. **Enable audit logging** for compliance

## Monitoring Best Practices

1. **Monitor all key views** (pg_stat_activity, pg_stat_database, pg_replication)
2. **Set up alerting** for critical thresholds
3. **Track cache hit ratio** (> 95% target)
4. **Monitor disk usage** (alert at 80%)
5. **Monitor replication lag** (alert at 30 seconds)
6. **Monitor connection count** (alert at 80% of max_connections)
7. **Monitor dead tuples** (alert at 100,000)
8. **Monitor WAL archive failures** (alert on any failure)
9. **Monitor checkpoint frequency**
10. **Review statistics periodically**

## Maintenance Best Practices

1. **Monitor autovacuum activity** weekly
2. **Review index usage** quarterly
3. **Tune autovacuum settings** annually
4. **Test restore procedures** quarterly
5. **Review and rotate SSL certificates**
6. **Review and update security configurations**
7. **Plan capacity increases** based on growth trends
8. **Document all changes** and their impact
9. **Test changes in staging** before production
10. **Maintain documentation** of cluster topology and procedures

## Incident Response Best Practices

1. **Assess severity** before responding
2. **Collect evidence** before making changes
3. **Apply stabilization** measures first
4. **Communicate status** to stakeholders
5. **Determine root cause** after stabilization
6. **Document incident** thoroughly
7. **Review lessons learned** after resolution
8. **Update runbooks** based on incident
9. **Test fixes** under load after resolution
10. **Share knowledge** with the team

## Capacity Planning Best Practices

1. **Monitor database growth trends** monthly
2. **Monitor index size growth** quarterly
3. **Monitor WAL archive disk usage** weekly
4. **Plan capacity increases** before reaching limits
5. **Consider tablespace placement** for capacity management
6. **Track I/O performance** trends
7. **Monitor connection growth** patterns
8. **Review workload patterns** seasonally
9. **Budget for storage growth** annually
10. **Evaluate hardware upgrades** quarterly

## References

- [PostgreSQL Performance Tips](https://www.postgresql.org/docs/current/performance-tips.html)
- [PostgreSQL Security Guide](https://www.postgresql.org/docs/current/security.html)
- [PostgreSQL Backup and Recovery](https://www.postgresql.org/docs/current/backup.html)
- [EDB Documentation](https://www.enterprisedb.com/docs/)