# EDB PostgreSQL Diagnostic Reasoning Reference

## Reasoning Patterns

### Pattern 1: Slow Query

```
Observation: Query running slowly
  ├── Index exists?
  │   ├── No: Add appropriate index
  │   └── Yes: Check index usage
  ├── Statistics current?
  │   ├── No: Run ANALYZE
  │   └── Yes: Check query plan
  └── Lock contention?
      ├── No: Check other factors
      └── Yes: Identify blocking queries
```

### Pattern 2: Connection Issues

```
Observation: Cannot connect
  ├── Server running?
  │   ├── No: Start server
  │   └── Yes: Check configuration
  ├── pg_hba.conf allows connection?
  │   ├── No: Update pg_hba.conf
  │   └── Yes: Check network
  └── Max connections reached?
      ├── No: Check other issues
      └── Yes: Kill idle or pool connections
```

### Pattern 3: Replication Lag

```
Observation: Replication lagging
  ├── Network OK?
  │   ├── No: Fix network
  │   └── Yes: Check standby
  ├── Standby disk full?
  │   ├── Yes: Free space
  │   └── No: Check configuration
  └── WAL settings correct?
      ├── No: Tune WAL settings
      └── Yes: Monitor for recovery
```

## Confidence Assessment

### Evidence Quality

| Evidence Type | Quality | Example |
|--------------|---------|---------|
| Direct error | High | PostgreSQL error message |
| Query plan | High | EXPLAIN ANALYZE output |
| System metrics | Medium | CPU, memory, I/O data |
| User reports | Low | "Queries are slow" |

### Confidence Scoring

1. **High (85-100%)**: Multiple high-quality evidence points
2. **Medium (60-84%)**: Some evidence with supporting data
3. **Low (30-59%)**: Limited evidence, needs more investigation
4. **Very Low (0-29%)**: Insufficient evidence

## References

- EDB PostgreSQL Diagnostics: https://www.enterprisedb.com/docs/
- PostgreSQL Diagnostics: https://www.postgresql.org/docs/current/
- Database Diagnostics: https://www.postgresql.org/docs/current/diag.html