# WF-007: Disk Space Exhaustion Troubleshooting

## Scenario

MySQL server disk is full or near full, causing write failures, potential data loss, and service disruption.

## Observation

| Metric | Check With | Concerning Value |
|--------|-----------|-----------------|
| Disk usage | `df -h /path/to/mysql/data` | > 85% |
| Data directory size | `du -sh /path/to/mysql/data/*` | Rapidly growing |
| Binary log size | `SHOW BINARY LOGS` | Large/uncontrolled growth |
| Error log | Tail of error log | "No space left on device" |

## Interpretation

Disk space exhaustion on MySQL servers is caused by:
- Uncontrolled binary log growth (most common)
- Large `.ibd` files from data growth
- General log or slow query log enabled and growing
- Temporary files from large operations
- Backup files left on the server
- InnoDB redo/undo logs

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | Binary logs not rotated or purged | High |
| 2 | Data growth without archiving | High |
| 3 | General log or slow query log enabled | Medium |
| 4 | Temporary files from large operations | Medium |
| 5 | Backup files left on server | Low |

## Evidence Required

1. **Disk usage by directory**:
   ```bash
   df -h /path/to/mysql/data
   du -sh /path/to/mysql/data/* | sort -h | tail -10
   ```

2. **Binary log inventory**:
   ```sql
   -- ADVISORY: Run on source
   SHOW BINARY LOGS;
   SHOW VARIABLES LIKE 'log_bin';
   SHOW VARIABLES LIKE 'binlog_expire_logs_seconds';
   ```

3. **InnoDB file sizes**:
   ```bash
   # ADVISORY: Run on server
   ls -lh /var/lib/mysql/ibdata1
   ls -lh /var/lib/mysql/ib_logfile*
   ```

4. **Log file sizes**:
   ```bash
   # ADVISORY: Run on server
   ls -lh /var/log/mysql/
   ls -lh /var/lib/mysql/general_log*
   ls -lh /var/lib/mysql/slow_query_log*
   ```

## Investigation Order

1. Check overall disk usage
2. Identify largest consumers in data directory
3. Check binary log count and total size
4. Check for enabled logs (general, slow query)
5. Check for temporary files
6. Check for backup files on server
7. Review `binlog_expire_logs_seconds` setting

## Recommended Actions

1. **Immediate (free space)**:
   ```sql
   -- ADVISORY: Only purge logs you have backups for
   -- First check which logs can be safely purged
   SHOW BINARY LOGS;
   
   -- Then purge old logs (only if backed up)
   PURGE BINARY LOGS BEFORE DATE_SUB(NOW(), INTERVAL 7 DAY);
   ```

2. **Clean up temporary files**:
   ```bash
   # ADVISORY: Check what's in tmpdir before removing
   du -sh /tmp/mysql-*
   rm -f /tmp/mysql-*
   ```

3. **Configure binlog expiration**:
   ```sql
   -- ADVISORY: Set expiration to prevent future growth
   SET GLOBAL binlog_expire_logs_seconds = 604800;  -- 7 days
   SET PERSIST binlog_expire_logs_seconds = 604800;
   ```

4. **Disable unnecessary logs**:
   ```sql
   -- ADVISORY: Disable general log in production
   SET GLOBAL general_log = 'OFF';
   ```

5. **Preventive measures**:
   - Set up disk space monitoring with alerts at 70% and 85%
   - Implement automated binary log rotation
   - Move backups off the data disk
   - Consider partitioning for large tables

## Expected Findings

- Binary logs are the largest consumers (most common)
- `binlog_expire_logs_seconds` not configured or set too high
- General log enabled and growing uncontrolled
- Temporary files from incomplete large operations
- Backup files left on data disk after transfer

## Possible Conclusions

- If binary logs dominate: Configure `binlog_expire_logs_seconds` and purge
- If data files dominate: Implement data archiving strategy
- If logs dominate: Disable unnecessary logging
- If temp files dominate: Investigate failed operations

## Recommended Next Step

After clearing space, configure monitoring and automated retention.

## Expected Outcome

- Disk usage drops below 70%
- Binary logs automatically expire after configured period
- Monitoring alerts trigger before disk fills again

## Risk Warnings

- **CRITICAL**: Purging binary logs removes data needed for PITR to points within the purged period
- Always verify backups exist before purging logs
- Never purge logs that haven't been replicated to all replicas
- Always ensure free disk space before attempting to start MySQL
- Monitor disk usage after fix to confirm proper management

## Documentation References

- [MySQL Binary Log](https://dev.mysql.com/doc/refman/8.0/en/binary-log.html)
- [MySQL Binary Log Maintenance](https://dev.mysql.com/doc/refman/8.0/en/binary-log-maintenance.html)
- [MySQL binlog_expire_logs_seconds](https://dev.mysql.com/doc/refman/8.0/en/replication-options-replica.html#sysvar_binlog_expire_logs_seconds)
- [MySQL Disk Space Issues](https://dev.mysql.com/doc/refman/8.0/en/disk-space-problems.html)