# Engineering Guidance and Safety Rules

## Purpose

This document defines the engineering guidance and safety rules for the MySQL Engineering Skill Pack. All guidance outputs must follow these principles to ensure safe, reliable, and actionable recommendations.

## Engineering Principles

### Evidence-Based Reasoning

All recommendations must be based on observable evidence. The following hierarchy applies:

1. **Direct Error Messages**: Highest confidence source
2. **Performance Metrics**: Second highest (monitoring data)
3. **Configuration Values**: Supporting evidence
4. **Historical Patterns**: Contextual information
5. **Documentation References**: General guidance

### Safety First

1. **Read-Only First**: Always start with diagnostic commands
2. **Low Risk**: Prefer non-disruptive actions
3. **Testing Required**: Changes should be tested in staging first
4. **Rollback Plan**: Always have a recovery path
5. **Change Window**: Make changes during maintenance windows when possible

### Command Risk Assessment

| Risk Level | Description | Examples |
|-----------|-------------|----------|
| **LOW** | No impact on data or performance | SELECT, SHOW, INFORMATION_SCHEMA queries |
| **MEDIUM** | Read-only but may impact performance | EXPLAIN, ANALYZE TABLE, large SELECT |
| **HIGH** | Data modification or performance impact | ALTER TABLE, DDL operations |
| **POTENTIALLY DISRUPTIVE** | Service impact or data loss risk | DROP, TRUNCATE, restart, major config change |

## Guidance Standards

### Response Format

Every recommendation must include:

1. **Observation**: What was detected
2. **Interpretation**: What it means
3. **Recommendation**: Actionable advice
4. **Why**: Reasoning behind the recommendation
5. **Evidence**: Supporting data
6. **Confidence**: Confidence level and percentage
7. **Next Step**: Suggested follow-up
8. **Risk Warning**: Any safety concerns

### Confidence Scoring

| Level | Percentage | Justification |
|-------|-----------|---------------|
| **High** | 85-100% | Direct error message with clear cause |
| **Medium** | 60-84% | Multiple symptoms pointing to same cause |
| **Low** | 30-59% | Single symptom, multiple possible causes |
| **Very Low** | 0-29% | Insufficient evidence, requires more investigation |

### Safety Warnings

Always include safety warnings for:

1. **Data Modification**: ALTER, DROP, DELETE, UPDATE
2. **Configuration Changes**: Parameter modifications
3. **Server Restarts**: Service disruption
4. **Index Operations**: May impact write performance temporarily
5. **Backup Operations**: Resource consumption

## Operational Guidelines

### Before Making Changes

1. **Document Current State**: Record current configuration
2. **Take Snapshot**: Create backup or checkpoint
3. **Test in Non-Production**: Validate in staging environment
4. **Monitor Before and After**: Baseline metrics for comparison
5. **Plan Rollback**: Have recovery procedure ready

### After Making Changes

1. **Verify Success**: Confirm change applied correctly
2. **Monitor Performance**: Watch for unexpected impacts
3. **Update Documentation**: Record changes made
4. **Review Logs**: Check for errors or warnings
5. **Confirm Recovery**: Ensure service is healthy

## MySQL-Specific Safety Rules

### Connection Management

- Never change `max_connections` without load testing
- Monitor `Threads_running` before adjusting connection limits
- Use connection pooling instead of high max_connections

### Schema Changes

- Always test `ALTER TABLE` in non-production first
- Use `pt-online-schema-change` for large tables
- Consider online DDL capabilities in MySQL 8.0

### Replication Changes

- Test failover procedures before implementing
- Monitor replication lag after any configuration change
- Use GTID-based replication for easier recovery

### Backup Operations

- Always test restore procedures
- Verify backup integrity before relying on them
- Maintain backup rotation policy

## Decision Tree Guidance

### High CPU

```
Observation: High CPU usage
  ├── Is it user queries?
  │   ├── Yes: Analyze slow query log
  │   └── No: Check system processes
  └── Is it background tasks?
      ├── Yes: Review maintenance operations
      └── No: Investigate system-level processes
```

### High Memory

```
Observation: High memory usage
  ├── Is it InnoDB buffer pool?
  │   ├── Yes: Verify buffer pool sizing
  │   └── No: Check other memory consumers
  └── Is it connection memory?
      ├── Yes: Review thread cache and buffers
      └── No: Monitor for memory leaks
```

## References

- MySQL 8.0 Configuration: https://dev.mysql.com/doc/refman/8.0/en/server-configuration.html
- MySQL 8.0 Performance: https://dev.mysql.com/doc/refman/8.0/en/optimization.html
- MySQL 8.0 Security: https://dev.mysql.com/doc/refman/8.0/en/security.html
- MySQL 8.0 Replication: https://dev.mysql.com/doc/refman/8.0/en/replication.html