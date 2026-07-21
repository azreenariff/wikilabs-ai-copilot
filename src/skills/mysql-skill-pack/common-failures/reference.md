# Common Failure Reference

## Overview

This reference documents common MySQL failure patterns, their symptoms, diagnosis methods, and recommended actions.

## Database-Level Failures

### Table Corruption

**Symptoms**:
- "Table is marked as crashed" errors
- Inconsistent query results
- Failed table operations

**Diagnosis**:
```sql
-- Check table integrity
CHECK TABLE table_name;

-- For InnoDB
CHECK TABLE table_name EXTENDED;

-- For MyISAM
REPAIR TABLE table_name;
```

**Causes**:
- Power failure during write
- Hardware issues
- Software bugs
- Improper shutdown

**Prevention**:
- Use InnoDB (automatic crash recovery)
- Proper shutdown procedures
- Regular backups
- Hardware monitoring

### Full Disk

**Symptoms**:
- "No space left on device" errors
- Write failures
- Service crashes

**Diagnosis**:
```sql
-- Check disk usage
SHOW VARIABLES LIKE 'datadir';
-- Check from OS level
df -h
du -sh /var/lib/mysql/*
```

**Prevention**:
- Disk space monitoring
- Automated log rotation
- Data archiving policies
- Storage capacity planning

### Binary Log Issues

**Symptoms**:
- Replication failures
- "Could not read from BINLOG" errors
- Missing binary logs

**Diagnosis**:
```sql
-- Check binary log status
SHOW BINARY LOGS;
SHOW MASTER STATUS;

-- Check binary log configuration
SHOW VARIABLES LIKE 'log_bin%';
```

**Prevention**:
- Regular binary log rotation
- Binary log backup
- Consistent naming conventions
- Monitoring for missing logs

## Connection Failures

### Max Connections Reached

**Symptoms**:
- "Too many connections" error
- Application connection failures
- High `Threads_connected`

**Diagnosis**:
```sql
-- Check connection status
SHOW STATUS LIKE 'Threads_connected';
SHOW STATUS LIKE 'Max_used_connections';
SHOW VARIABLES LIKE 'max_connections';
```

**Prevention**:
- Connection pooling
- Appropriate `max_connections` sizing
- Thread cache optimization
- Application connection management

### Authentication Failures

**Symptoms**:
- "Access denied" errors
- Failed login attempts
- Authentication plugin mismatches

**Diagnosis**:
```sql
-- Check user authentication
SELECT user, host, plugin FROM mysql.user;

-- Check authentication errors
SHOW VARIABLES LIKE 'log_error';
```

**Prevention**:
- Proper user account setup
- Authentication plugin consistency
- Password policy enforcement
- Regular credential audits

## Performance Failures

### Buffer Pool Exhaustion

**Symptoms**:
- High disk IO
- Slow queries
- Low buffer pool hit ratio

**Diagnosis**:
```sql
-- Check buffer pool status
SHOW STATUS LIKE 'Innodb_buffer_pool%';

-- Calculate hit ratio
SELECT 
  (1 - (Innodb_buffer_pool_reads / Innodb_buffer_pool_read_requests)) * 100 AS hit_ratio
FROM information_schema.GLOBAL_STATUS;
```

**Prevention**:
- Proper buffer pool sizing
- Monitoring hit ratios
- Query optimization
- Appropriate workload distribution

### Lock Contention

**Symptoms**:
- Transaction timeouts
- Deadlock errors
- Slow transactions

**Diagnosis**:
```sql
-- Check current locks
SELECT * FROM performance_schema.data_locks;
SELECT * FROM performance_schema.data_lock_waits;

-- Check for deadlocks
SHOW ENGINE INNODB STATUS;
```

**Prevention**:
- Short transaction durations
- Consistent lock ordering
- Appropriate isolation levels
- Query optimization

## Infrastructure Failures

### Network Issues

**Symptoms**:
- Intermittent connections
- High latency
- Connection timeouts

**Diagnosis**:
```sql
-- Check network metrics
SHOW STATUS LIKE 'Network%';
SHOW STATUS LIKE 'Tcp%';
```

**Prevention**:
- Network monitoring
- Appropriate timeout settings
- Load balancing
- Redundant network paths

### Hardware Failures

**Symptoms**:
- Disk errors
- Memory errors
- CPU throttling

**Diagnosis**:
```bash
# Check system logs
dmesg
journalctl -xe

# Check hardware health
smartctl -a /dev/sda
memtest86+
```

**Prevention**:
- Hardware monitoring
- Regular maintenance
- Redundant hardware
- Predictive maintenance

## References

- MySQL 8.0 Troubleshooting: https://dev.mysql.com/doc/refman/8.0/en/troubleshooting.html
- MySQL 8.0 Error Handling: https://dev.mysql.com/doc/refman/8.0/en/error-handling.html
- MySQL 8.0 Recovery: https://dev.mysql.com/doc/refman/8.0/en/backup-restore.html