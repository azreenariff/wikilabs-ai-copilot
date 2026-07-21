# Diagnostic Reasoning Reference

## Overview

This document provides structured reasoning frameworks for MySQL troubleshooting. Each reasoning path follows evidence-based analysis with confidence scoring.

## Reasoning Patterns

### Pattern 1: Performance Degradation

```
Observation: Query performance degrading
  ├── New queries?
  │   ├── Yes: Analyze execution plans
  │   └── No: Check infrastructure changes
  ├── Schema changes?
  │   ├── Yes: Review DDL changes
  │   └── No: Check data volume changes
  └── Resource changes?
      ├── Yes: Compare current vs baseline
      └── No: Look for application changes
```

### Pattern 2: Connection Issues

```
Observation: Connection failures
  ├── Network problems?
  │   ├── Yes: Check network connectivity
  │   └── No: Check MySQL configuration
  ├── Max connections reached?
  │   ├── Yes: Increase max_connections
  │   └── No: Check authentication issues
  └── Authentication failures?
      ├── Yes: Verify credentials
      └── No: Check security plugins
```

### Pattern 3: Replication Issues

```
Observation: Replication lag
  ├── IO thread running?
  │   ├── No: Check network/SSL
  │   └── Yes: Check SQL thread
  ├── SQL thread running?
  │   ├── No: Check for errors
  │   └── Yes: Check query complexity
  └── Slave workload?
      ├── High: Optimize read queries
      └── Low: Check master load
```

## Decision Trees

### High CPU Decision Tree

```
High CPU Detected
  │
  ├── Is it consistent or intermittent?
  │   ├── Consistent: Check for long-running queries
  │   └── Intermittent: Look for burst patterns
  │
  ├── What percentage of time is user mode vs system mode?
  │   ├── High user mode: Application query processing
  │   └── High system mode: OS-level IO or context switching
  │
  └── What's the top consumer?
      ├── mysqld process: Check query patterns
      ├── System process: Check for IO pressure
      └── Background tasks: Check maintenance operations
```

### Memory Pressure Decision Tree

```
High Memory Usage
  │
  ├── Is it growing steadily or fluctuating?
  │   ├── Steadily growing: Check for memory leaks
  │   └── Fluctuating: Normal usage patterns
  │
  ├── What's consuming most memory?
  │   ├── InnoDB buffer pool: Check sizing
  │   ├── Thread buffers: Check connection count
  │   ├── OS caches: Check overall system usage
  │   └── Application memory: Check application behavior
  │
  └── Are there swap operations?
      ├── Yes: Critical - immediate action needed
      └── No: Monitor for trends
```

### Lock Contention Decision Tree

```
Lock Contention Detected
  │
  ├── What type of lock?
  │   ├── Table lock: Check DDL operations
  │   ├── Row lock: Check transaction patterns
  │   └── Metadata lock: Check schema changes
  │
  ├── Which sessions are waiting?
  │   ├── Application sessions: Check application behavior
  │   ├── Background tasks: Check maintenance operations
  │   └── Replication threads: Check replication performance
  │
  └── How long are they waiting?
      ├── Short (< 1s): Normal for high concurrency
      ├── Medium (1-10s): Monitor for patterns
      └── Long (> 10s): Investigate root cause
```

## Confidence Assessment

### Evidence Quality

| Evidence Type | Quality | Example |
|--------------|---------|---------|
| Direct error | High | "ERROR 1045: Access denied" |
| Performance metrics | High | Slow query log with duration |
| Configuration values | Medium | Buffer pool size comparison |
| Historical data | Medium | CPU usage trends |
| User reports | Low | "System feels slow" |

### Confidence Scoring Guidelines

1. **High (85-100%)**: Multiple high-quality evidence points
2. **Medium (60-84%)**: Some evidence with supporting data
3. **Low (30-59%)**: Limited evidence, needs more investigation
4. **Very Low (0-29%)**: Insufficient evidence for confident recommendation

### Confidence Documentation

Always document:
- What evidence was collected
- Evidence quality assessment
- Confidence level rationale
- Uncertainty areas
- Recommended next evidence collection steps

## References

- MySQL 8.0 Diagnostics: https://dev.mysql.com/doc/refman/8.0/en/diagnostics.html
- MySQL 8.0 Performance Schema: https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html
- MySQL 8.0 Troubleshooting: https://dev.mysql.com/doc/refman/8.0/en/troubleshooting.html