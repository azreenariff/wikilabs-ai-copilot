# MySQL Output and Log Interpretation

## Purpose

This document describes how to interpret common MySQL output, log entries, and diagnostic information. Accurate interpretation is essential for effective troubleshooting and performance analysis.

---

## InnoDB STATUS Output Interpretation

The `SHOW ENGINE INNODB STATUS\G` output is the most comprehensive single diagnostic for InnoDB issues. Key sections:

### TRANSACTIONS Section

```
---TRANSACTIONS---
Trx id counter 12345678
Purge done for trx's n:o < 12345600 undo n:o < 0 state: running
History list length 123
LIST OF TRANSACTIONS FOR EACH SESSION:
---TRANSACTION 42101234, not started, process no 1234,
OS thread id 1234567890 running
MySQL thread id 56, process id 1234, thread id 1234567890,
query id 1234567 host 192.168.1.100 user app_user
SELECT * FROM orders WHERE status = 'pending'
------- TRX HAS BEEN WAITING 3 SEC FOR THIS LOCK TO BE GRANTED:
RECORD LOCKS space id 56 page no 123 n bits 72 index PRIMARY
of table db.orders trx id 12345678 lock_mode X waiting
```

**Interpretation**:
- `Trx id counter`: Current transaction ID — compare with old values to detect stalled transactions
- `History list length`: Undo log history entries — growing value may indicate long-running transactions
- `TRX HAS BEEN WAITING 3 SEC`: Lock wait information — identifies blocking relationship
- `lock_mode X waiting`: Exclusive lock wait — transaction trying to write but blocked
- `lock_mode X`: Transaction that holds the lock (the blocker)

**Action**: Identify the blocking transaction, check if it can be completed or killed.

### LATEST DETECTED DEADLOCK Section

```
*** (1) TRANSACTION:
TRANSACTION 12345678, ACTIVE 5 sec starting index read
MySQL thread id 56, process id 1234
*** (1) HOLDS THE LOCK(S):
RECORD LOCKS space id 56 page no 123 n bits 72 index PRIMARY
*** (1) WAITING FOR THIS LOCK TO BE GRANTED:
RECORD LOCKS space id 57 page no 456 n bits 72 index idx_status
*** (2) TRANSACTION:
TRANSACTION 12345679, ACTIVE 4 sec starting index read
MySQL thread id 57, process id 1234
*** (2) HOLDS THE LOCK(S):
RECORD LOCKS space id 57 page no 456 n bits 72 index idx_status
*** (2) WAITING FOR THIS LOCK TO BE GRANTED:
RECORD LOCKS space id 56 page no 123 n bits 72 index PRIMARY
*** WE ROLL BACK TRANSACTION (2)
```

**Interpretation**:
- Transaction 1 holds lock on PRIMARY index, wants idx_status
- Transaction 2 holds lock on idx_status, wants PRIMARY
- Circular dependency = deadlock
- InnoDB rolled back transaction 2 (the victim)

**Action**: Fix application lock ordering, add missing indexes, reduce transaction scope.

### BUFFER POOL AND MEMORY Section

```
---------------------
BUFFER POOL AND MEMORY
---------------------
Total memory allocated 10737418240; in database allocated 8589934592
Free memory 536870912; allocated free 268435456
Database pages 524288; modified database pages 1024
Pages read ahead 0.00/s, evicted without access 0.00/s
```

**Interpretation**:
- `Total memory allocated`: Current buffer pool size
- `in database allocated`: Memory actually containing data pages
- `Free memory`: Unused pages available — low value may indicate need to increase size
- `Database pages`: Pages containing data in buffer pool
- `modified database pages`: Dirty pages needing flush — high value may indicate write pressure
- `Pages read ahead`: Pre-read pages — low value with sequential access may indicate good configuration

**Action**: Monitor `Free memory` and `modified database pages` over time.

### ROW OPERATIONS Section

```
----------------------
ROW OPERATIONS
----------------------
0 queries inside InnoDB, 0 queries in queue
1 read views open inside InnoDB
Process ID 1234, Main thread ID 1234567890, Thread id 1234567890
Spin rounds per ms: 0
```

**Interpretation**:
- `queries inside InnoDB`: Queries actively executing within InnoDB
- `queries in queue`: Queries waiting for access
- `read views open`: Active MVCC snapshots

---

## Error Log Interpretation

### Error Log Format

```
2024-01-15T10:30:45.123456Z 12 [ERROR] [MY-012345] [InnoDB] Some error description
2024-01-15T10:30:45.123456Z 12 [Warning] [MY-012346] [Server] Some warning
2024-01-15T10:30:45.123456Z 12 [Note] [MY-012347] [Server] Some informational message
```

**Levels**:
- **ERROR**: Problem that may affect service — requires attention
- **Warning**: Potentially problematic condition — monitor
- **Note**: Informational message — routine operations

### Common Error Messages

| Message | Meaning | Severity |
|---------|---------|----------|
| `InnoDB: Unable to lock ./ibdata1 error: 11` | Another process holding ibdata1 | High — possible crash or concurrent access |
| `InnoDB: Error: trying to access page` | Corrupt page | Critical — data corruption |
| `InnoDB: Crash recovery may be necessary` | Unexpected shutdown | High — check disk, OOM, power |
| `Aborted connection N to db: 'db' user: 'user'` | Connection drop | Low — normal for idle timeout |
| `Can't create a new thread (errno 11)` | OS thread creation failed | Critical — OOM or ulimit reached |
| `Access denied for user` | Authentication failure | Medium — wrong credentials or ACL |
| `Got an error reading communication packets` | Network interruption | Medium — check network stability |

### Error Log Analysis Strategy

1. **Sort by timestamp** — identify chronology of events
2. **Group by severity** — address ERROR and Warning first
3. **Look for patterns** — repeated errors indicate systemic issues
4. **Cross-reference** — correlate with performance issues, replication lag, etc.
5. **Check error log rotation** — ensure old errors aren't lost

---

## Binary Log Interpretation

### mysqlbinlog Output

```
/*!50530 SET @@SESSION.PSEUDO_SLAVE_MODE=1*/;
/*!50003 SET @OLD_COMPLETION_TYPE=@@COMPLETION_TYPE,COMPLETION_TYPE=0*/;
DELIMITER /*!*/;
# at 4
#240115 10:30:00 server id 1  end_log_pos 123 CRC32 0x12345678 	Start: binlog v 4, server v 8.0.35 created 240115 10:30:00 at startup
ROLLBACK/*!*/;
BINLOG '
base64-encoded-data
'/*!*/;
# at 123
#240115 10:30:01 server id 1  end_log_pos 200 CRC32 0xabcdef12 	Previous-GTIDs
# [gateway]
#240115 10:30:15 server id 1  end_log_pos 300 CRC32 0x98765432 	Anonymous_GTID	last_committed=1	sequence_number=2	rbr_only=no
SET @@SESSION.GTID_NEXT= 'ANONYMOUS'/*!*/;
# at 300
#240115 10:30:15 server id 1  end_log_pos 400 CRC32 0x11223344 	Query	thread_id=56	exec_time=0	error_code=0
SET TIMESTAMP=1705312215/*!*/;
SET @@session.pseudo_thread_id=56/*!*/;
INSERT INTO orders (id, status, total) VALUES (1001, 'pending', 99.99)
/*!*/;
```

**Interpretation**:
- `server id`: Source server ID in replication
- `end_log_pos`: Position of the next event
- `Previous-GTIDs`: GTID set up to this position
- `Anonymous_GTID` / `SET @@SESSION.GTID_NEXT`: GTID context
- `thread_id`: Connection/thread that executed the statement
- `exec_time`: Execution time in seconds
- `error_code`: 0 if successful, non-zero if error was recorded

### Binary Log Analysis Strategy

1. **Check format** (`--base64-output=decode-rows -v`) — see actual row changes
2. **Filter by time** (`--start-datetime`, `--stop-datetime`) — narrow scope
3. **Filter by database** (`--database=db`) — focus on specific database
4. **Check GTID consistency** (`--include-gtids`, `--exclude-gtids`)
5. **Look for large transactions** — single DML affecting many rows

---

## Replication Status Interpretation

### SHOW REPLICA STATUS Output

```
               Slave_IO_State: Waiting for source communication
                  Source_Host: 192.168.1.100
                  Source_User: repl_user
                  Source_Port: 3306
                  Connect_Retry: 60
                Source_Log_File: mysql-bin.000015
            Read_Source_Log_Pos: 1234567
                 Relay_Log_File: relay.000020
                  Relay_Log_Pos: 1234500
          Relay_Source_Log_File: mysql-bin.000015
              Replicate_Do_DB:
          Replicate_Ignore_DB:
           Replicate_Do_Table:
       Replicate_Ignore_Table:
      Replicate_Wild_Do_Table:
  Replicate_Wild_Ignore_Table:
                  Last_Errno: 0
                  Last_Error:
                  Skip_Counter: 0
          Exec_Source_Log_Pos: 1234400
              Relay_Space: 2345678
      Until_Condition: None
       Until_Log_File:
        Until_Log_Pos: 0
           Source_SSL_Allowed: Yes
           Source_SSL_CA_File:
           Source_SSL_CA_Path:
           Source_SSL_Cert:
           Source_SSL_Cipher:
              Source_SSL_Key:
          Seconds_Behind_Source: 167
```

**Key Fields**:
| Field | Meaning | Normal | Problem |
|-------|---------|--------|---------|
| Slave_IO_State | IO thread state | "Waiting for source" | Should be active or waiting |
| Source_Log_File | Current binlog on source | Matches master | Mismatch if binlog expired |
| Read_Source_Log_Pos | Position read on source | Consistent | Gap indicates lag |
| Exec_Source_Log_Pos | Position executed on replica | Should catch up to Read | Large gap = lag |
| Last_Errno | Last error number | 0 | Non-zero = error |
| Last_Error | Last error message | Empty | Non-empty = needs investigation |
| Seconds_Behind_Source | Replication lag in seconds | Low (< 30) | Increasing = problem |

**Interpretation**:
- `Read_Source_Log_Pos - Exec_Source_Log_Pos = gap`: Larger gap = more lag
- `Last_Error` non-empty: Specific error to investigate
- `Source_SSL_Allowed: Yes`: Connection uses SSL — verify certificate not expired
- `Source_SSL_Allowed: No`: Connection not encrypted — security risk

### Common Replication Status Patterns

| Pattern | Interpretation | Action |
|---------|---------------|--------|
| `Slave_IO_Running: Yes`, `Slave_SQL_Running: Yes`, `Seconds_Behind_Source: 0` | Healthy replication | Normal operation |
| `Slave_IO_Running: No`, `Last_Error: Log file not found` | Missing binlog on source | Use CHANGE REPLICATION SOURCE TO |
| `Slave_SQL_Running: No`, `Last_Error: Duplicate entry` | Data inconsistency | Fix data or skip error |
| `Seconds_Behind_Source: NULL` | SQL thread stopped | Check Last_Error, restart SQL thread |
| `Seconds_Behind_Source: Increasing` | Replication falling behind | Scale replica, optimize master writes |

---

## Slow Query Log Interpretation

### Slow Query Log Format

```
# Time: 2024-01-15T10:30:15.123456Z
# User@Host: app_user[app_user] @  [192.168.1.105]  Id:    56
# Query_time: 12.345678  Lock_time: 0.000123  Rows_sent: 1500  Rows_examined: 5000000
SET timestamp=1705312215;
SELECT o.id, o.status, o.total, c.name
FROM orders o
JOIN customers c ON o.customer_id = c.id
WHERE o.created_at > '2024-01-01'
ORDER BY o.total DESC
LIMIT 100;
```

**Interpretation**:
| Field | Meaning | Good | Problem |
|-------|---------|------|---------|
| Query_time | Total query execution time | < 1s | > 1s = slow |
| Lock_time | Time spent waiting for locks | < 0.1s | > 1s = lock contention |
| Rows_sent | Rows returned to client | Expected | N/A |
| Rows_examined | Rows checked before filtering | Close to Rows_sent | Much larger = inefficient scan |

**Diagnostic Indicators**:
- `Rows_examined >> Rows_sent`: Full table scan or poor index usage
- `Lock_time >> Query_time`: Severe lock contention
- High `Query_time` with low `Lock_time`: CPU or I/O bound query
- Query runs during peak hours: May need query optimization or caching

---

## Performance Schema Interpretation

### Key Metrics

| Metric Category | Key Tables | Purpose |
|----------------|-----------|---------|
| Wait events | `events_waits_*` | I/O, lock, and sync wait times |
| Statement events | `events_statements_*` | Query execution times and counts |
| Stage events | `events_stages_*` | Query phase timing |
| Transaction events | `events_transactions_*` | Transaction duration and counts |
| Table I/O | `table_io_waits_summary_*` | Per-table read/write frequency |

### Sys Schema Key Views

| View | Purpose | Key Insights |
|------|---------|-------------|
| `schema_table_statistics` | Table I/O and lock stats | Hot tables, lock contention |
| `host_by_current_memory_usage` | Memory by host | Connection memory profile |
| `user_by_current_memory_usage` | Memory by user | Per-user resource consumption |
| `statement_analysis` | SQL statement performance | Frequent slow queries |
| `memory_global_total` | Total memory usage | Memory pressure |
| `waits_lost_instrumented` | Dropped wait events | Instrumentation capacity |

---

## MySQL Command Output Interpretation

### SHOW GLOBAL STATUS Key Metrics

| Variable | Meaning | Normal Range | Problem Indicator |
|----------|---------|-------------|-------------------|
| Threads_connected | Active connections | < 80% max_connections | Near max_connections |
| Threads_running | Executing threads | < 10% max_connections | > 50% = CPU pressure |
| Threads_created | Total threads created | Low growth rate | High growth = connection churn |
| Connections | Total connection attempts | N/A | High vs Threads_connected = auth issues |
| Aborted_connects | Failed connections | Low | High = auth/network problems |
| Queries | Total queries | Increasing | Sudden drop = server issue |
| Uptime | Server uptime | Stable | Drops = restart |
| Handler_read_next | Full table scan rows | Low growth | High = missing indexes |
| Innodb_buffer_pool_read_requests | Buffer pool reads | N/A | N/A |
| Innodb_buffer_pool_reads | Physical reads from disk | < 1% of read_requests | > 1% = buffer pool too small |

### SHOW GLOBAL VARIABLES Key Checks

| Variable | Check | Recommended |
|----------|-------|-------------|
| innodb_buffer_pool_size | Size vs available RAM | 60-80% of RAM |
| max_connections | Value vs needs | 200-500+ (tuned) |
| wait_timeout | Idle timeout value | 300-600 for web apps |
| long_query_time | Slow query threshold | 0.5-1.0 seconds |
| log_bin | Binary logging enabled | ON for production |
| gtid_mode | GTID enabled | ON for replication |
| innodb_flush_log_at_trx_commit | Flush strategy | 1 (ACID) or 2 (balance) |
| innodb_flush_method | I/O method | O_DIRECT |

---

## References

- [MySQL 8.0 Reference Manual: Error Handling](https://dev.mysql.com/doc/refman/8.0/en/error-handling.html)
- [MySQL 8.0 Reference Manual: Error Log](https://dev.mysql.com/doc/refman/8.0/en/error-log.html)
- [MySQL 8.0 Reference Manual: Binary Log](https://dev.mysql.com/doc/refman/8.0/en/binary-log.html)
- [MySQL 8.0 Reference Manual: Performance Schema Tables](https://dev.mysql.com/doc/refman/8.0/en/performance-schema-tables.html)
- [MySQL 8.0 Reference Manual: Sys Schema](https://dev.mysql.com/doc/refman/8.0/en/sys-schema.html)