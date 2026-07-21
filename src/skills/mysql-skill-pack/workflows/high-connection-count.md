# WF-001: High Connection Count Troubleshooting

## Scenario

The MySQL server is experiencing connection failures, and the `Threads_connected` count is approaching `max_connections`.

## Observation

| Metric | Current Value | Threshold |
|--------|--------------|-----------|
| Threads_connected | Check with `SHOW STATUS LIKE 'Threads_connected'` | > 80% of max_connections |
| Max_used_connections | Check with `SHOW STATUS LIKE 'Max_used_connections'` | Approaching max_connections |
| Threads_running | Check with `SHOW STATUS LIKE 'Threads_running'` | High value indicates contention |
| Connections (total) | Check with `SHOW STATUS LIKE 'Connections'` | Rapidly increasing |

## Interpretation

A high connection count indicates:
- Too many concurrent clients connecting to the server
- Connection pooling not properly configured
- Long-running connections not being released
- Application connection leak
- `max_connections` set too low for the workload

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | Application connection pool misconfigured | High |
| 2 | Missing connection pooling middleware | Medium |
| 3 | Long-running queries holding connections | Medium |
| 4 | `max_connections` undersized | Medium |
| 5 | Connection leak in application code | Low-Medium |

## Evidence Required

1. **Connection distribution**:
   ```sql
   -- Check connections by user
   SELECT user, host, COUNT(*) as conn_count 
   FROM information_schema.processlist 
   GROUP BY user, host 
   ORDER BY conn_count DESC;
   ```

2. **Active vs idle connections**:
   ```sql
   SELECT command, COUNT(*) as count 
   FROM information_schema.processlist 
   GROUP BY command;
   ```

3. **Long-running queries**:
   ```sql
   SELECT id, user, host, db, command, time, state, info
   FROM information_schema.processlist
   WHERE time > 60
   ORDER BY time DESC;
   ```

4. **Connection history**:
   ```sql
   SELECT variable_name, variable_value
   FROM information_schema.session_status
   WHERE variable_name IN ('Connections', 'Max_used_connections', 'Threads_connected', 'Threads_running');
   ```

## Investigation Order

1. Check current connection count vs max
2. Identify which users have most connections
3. Identify long-running queries
4. Check application connection pool settings
5. Review `wait_timeout` and `interactive_timeout` settings
6. Check for connection leaks

## Recommended Actions

1. **Immediate**: Kill idle connections blocking the server
   ```sql
   -- ADVISORY: Review before killing
   -- SELECT CONCAT('KILL ', id, ';') FROM information_schema.processlist WHERE time > 300 AND command = 'Sleep';
   ```

2. **Short-term**: Increase `max_connections` if appropriate
   ```sql
   -- ADVISORY: Test on staging
   SET GLOBAL max_connections = 500;  -- Adjust based on capacity
   SET PERSIST max_connections = 500;
   ```

3. **Medium-term**: Implement connection pooling
   - Use ProxySQL, ProxySQL, or application-level pooling
   - Set `thread_cache_size` appropriately
   - Tune `wait_timeout` for idle connection cleanup

4. **Long-term**: Fix application connection management
   - Review connection pool configuration
   - Implement connection lifecycle management
   - Add monitoring for connection trends

## Expected Findings

- Most connections from application users during peak hours
- High number of `Sleep` state connections (idle)
- Connection count correlates with application deployment or traffic spike
- `Max_used_connections` approaches `max_connections` before failures

## Possible Conclusions

- If most connections are from one user: Application needs connection pool tuning
- If many long-running queries: Identify and optimize slow queries
- If `wait_timeout` too high: Idle connections persist too long
- If connection pool max too high: Pool settings exceed MySQL capacity

## Recommended Next Step

After fixing the immediate issue, implement connection pooling and monitoring.

## Expected Outcome

- Connection count stabilizes within healthy range
- No more connection refusal errors
- `Max_used_connections` stays below 75% of `max_connections`

## Risk Warnings

- Increasing `max_connections` increases memory usage (each connection uses ~thread buffers)
- Killing connections may disrupt active operations — review before executing
- Connection pool tuning requires application code changes
- Test all changes on staging before production

## Documentation References

- [MySQL max_connections](https://dev.mysql.com/doc/refman/8.0/en/too-many-connections.html)
- [MySQL Thread Cache](https://dev.mysql.com/doc/refman/8.0/en/server-status-variables.html#statvar_Thread_cache_size)
- [MySQL wait_timeout](https://dev.mysql.com/doc/refman/8.0/en/server-system-variables.html#sysvar_wait_timeout)
- [ProxySQL](https://proxysql.com/documentation/)