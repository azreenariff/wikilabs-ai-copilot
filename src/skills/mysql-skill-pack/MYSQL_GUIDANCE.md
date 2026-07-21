# MySQL Engineering Guidance

## Purpose

This document provides engineering guidance, safety protocols, and recommendations for MySQL engineering tasks. It is advisory only — it never recommends command execution.

## Safety Rules

### Golden Rules

1. **Never execute commands — only recommend and explain**
   - The AI Copilot is an advisory tool. Every action must be performed by the engineer.
   - Provide clear explanations of what a command does, why it's needed, and what alternatives exist.

2. **Always warn about risks before recommending actions**
   - Every recommendation must include a risk assessment (Low, Medium, High).
   - High-risk actions must include explicit warnings and rollback strategies.

3. **Always provide rollback strategies for destructive operations**
   - Before any DROP, DELETE, TRUNCATE, or ALTER TABLE: recommend backup first.
   - Provide the exact steps to reverse the action if needed.

4. **Always recommend evidence collection before diagnosis**
   - Before troubleshooting: collect SHOW ENGINE INNODB STATUS, SHOW PROCESSLIST, and relevant log excerpts.
   - Document the state before making changes for comparison.

5. **Always consider cascade effects of recommended actions**
   - A configuration change may affect replication, performance, or application behavior.
   - Evaluate the full impact chain before recommending changes.

6. **Never modify configuration without explicit engineer approval**
   - Present the recommended change, its justification, and its impact.
   - The engineer makes the final decision.

7. **Always recommend configuration validation before reload**
   - Use `mysqld --verbose --help` or `mysql --validate-config` to validate new settings.
   - Test in staging before production changes.

8. **Include confidence scoring in all recommendations**
   - Low confidence (< 0.6): The diagnosis is uncertain; more evidence is needed.
   - Medium confidence (0.6–0.8): Likely correct but should be verified.
   - High confidence (> 0.8): Strong evidence; proceed with standard precautions.

9. **Include risk warnings in all high-risk operations**
   - High-risk operations: DDL changes, data modifications, replication resets, shutdowns.
   - Always require engineer confirmation before proceeding.

## Reasoning Guidelines

### Evidence-Based Diagnosis

When diagnosing MySQL issues:

1. **Collect evidence first**: SHOW GLOBAL STATUS, SHOW GLOBAL VARIABLES, SHOW FULL PROCESSLIST, SHOW ENGINE INNODB STATUS, relevant log excerpts
2. **Identify symptoms**: What is the user experiencing? (slow queries, errors, connectivity issues, replication lag)
3. **Correlate evidence with symptoms**: Which metrics/logs align with the symptoms?
4. **Form a hypothesis**: What is the most likely root cause?
5. **Verify the hypothesis**: What additional evidence would confirm or refute it?
6. **Recommend actions**: What steps should the engineer take?
7. **Define success criteria**: How will we know the fix worked?

### Confidence Scoring

- **High confidence (> 0.8)**: Multiple independent evidence sources confirm the diagnosis
- **Medium confidence (0.6–0.8)**: Evidence is consistent but incomplete; further verification needed
- **Low confidence (< 0.6)**: Evidence is ambiguous; more diagnostic information is required

### Risk Assessment

| Risk Level | Description | Examples |
|-----------|-------------|----------|
| Low | Read-only operations, diagnostics, status checks | SHOW STATUS, SHOW PROCESSLIST, SELECT |
| Medium | Operations that may cause brief disruption | ALTER TABLE (INPLACE), START REPLICA, SET GLOBAL |
| High | Operations that may cause extended disruption or data loss | DROP TABLE, DROP DATABASE, mysqladmin shutdown, ALTER TABLE (online), RESET REPLICA |

## Recommendations

### Query Optimization Recommendations

1. **Use EXPLAIN ANALYZE** to identify full table scans (type=ALL), filesorts, and temporary tables
2. **Check index usage** via sys.schema_unused_indexes and sys.schema_unused_indexes
3. **Analyze slow queries** from the slow query log (long_query_time threshold should be tuned)
4. **Verify table statistics** are current (ANALYZE TABLE after bulk operations)
5. **Monitor optimizer decisions** with optimizer_trace for complex queries

### Configuration Recommendations

1. **innodb_buffer_pool_size**: Set to 50–70% of available RAM on dedicated MySQL servers
2. **innodb_log_file_size**: Larger values improve write performance; monitor via Performance Schema
3. **max_connections**: Tune based on application connection patterns; use connection pooling
4. **innodb_flush_log_at_trx_commit**: Set to 1 for full ACID compliance, 2 for better performance (accepting potential data loss)
5. **sync_binlog**: Set to 1 for full durability; 0 for better performance with replication safety
6. **binlog_expire_logs_seconds**: Retain for at least 7 days for replication and recovery

### Security Recommendations

1. **Use strong passwords** with sufficient length and complexity
2. **Restrict host access** — avoid wildcard '%' hosts in production
3. **Enable SSL/TLS** for all remote connections
4. **Follow principle of least privilege** — grant only what is needed
5. **Use roles** (MySQL 8.0+) for privilege management
6. **Enable password expiration** for service accounts
7. **Regular privilege audits** — review GRANTS periodically

### Backup Recommendations

1. **mysqldump** for small databases (< 50GB) or for logical backups with portability
2. **Percona XtraBackup** for large databases — physical backup without downtime
3. **Always use --single-transaction** for InnoDB consistency in mysqldump
4. **Test restores regularly** — a backup that cannot be restored is not a backup
5. **Follow the 3-2-1 rule**: 3 copies, 2 different media, 1 offsite

### Replication Recommendations

1. **Use GTID-based replication** (auto_position=1) for simpler management
2. **Monitor Seconds_Behind_Master** — alert when it exceeds your SLA
3. **Plan for replica failover** — test the process regularly
4. **Use binary log retention** long enough to rebuild any replica
5. **Consider MySQL Group Replication** for multi-master setups with conflict detection

### Monitoring Recommendations

1. **Enable slow query log** with an appropriate long_query_time threshold
2. **Use Performance Schema** for detailed instrumentation
3. **Monitor sys schema** for actionable insights
4. **Track key metrics**: Connections, QPS, InnoDB buffer pool hit ratio, replication lag, lock waits
5. **Set up alerting** for critical thresholds (too many connections, replication lag, disk space)

## Documentation References

- [MySQL 8.0 Reference Manual](https://dev.mysql.com/doc/refman/8.0/en/)
- [MySQL 8.4 Reference Manual](https://dev.mysql.com/doc/refman/8.4/en/)
- [MySQL Internals Manual](https://dev.mysql.com/doc/internals/en/)
- [Percona XtraBackup Documentation](https://www.percona.com/doc/percona-xtrabackup/LATEST/index.html)
- [MySQL Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html)
- [MySQL Sys Schema](https://dev.mysql.com/doc/refman/8.0/en/sys-schema.html)
- [MySQL Replication](https://dev.mysql.com/doc/refman/8.0/en/replication.html)
- [MySQL Security](https://dev.mysql.com/doc/refman/8.0/en/security.html)