# MSSQL Diagnostic Reasoning Reference

## Reasoning Patterns

### Pattern 1: Slow Query

```
Observation: Query running slowly
  ├── Execution plan efficient?
  │   ├── No: Optimize query/index
  │   └── Yes: Check statistics
  ├── Statistics current?
  │   ├── No: Update statistics
  │   └── Yes: Check hardware
  └── Blocking present?
      ├── No: Check other factors
      └── Yes: Resolve blocking
```

### Pattern 2: Connection Issues

```
Observation: Cannot connect
  ├── Server running?
  │   ├── No: Start server
  │   └── Yes: Check configuration
  ├── Authentication successful?
  │   ├── No: Fix credentials
  │   └── Yes: Check network
  └── Max connections reached?
      ├── Yes: Kill idle sessions
      └── No: Check other issues
```

### Pattern 3: Replication Lag

```
Observation: Replication lagging
  ├── Network OK?
  │   ├── No: Fix network
  │   └── Yes: Check distribution DB
  ├── Distribution DB healthy?
  │   ├── No: Fix distribution DB
  │   └── Yes: Check agents
  └── Agents running?
      ├── No: Restart agents
      └── Yes: Monitor for recovery
```

## Confidence Assessment

### Evidence Quality

| Evidence Type | Quality | Example |
|--------------|---------|---------|
| Direct error | High | SQL Server error message |
| DMV output | High | sys.dm_* query results |
| Wait stats | Medium | Wait type analysis |
| User reports | Low | "Queries are slow" |

### Confidence Scoring

1. **High (85-100%)**: Multiple high-quality evidence points
2. **Medium (60-84%)**: Some evidence with supporting data
3. **Low (30-59%)**: Limited evidence, needs more investigation
4. **Very Low (0-29%)**: Insufficient evidence

## References

- SQL Server Diagnostics: https://learn.microsoft.com/en-us/sql/
- SQL Server Troubleshooting: https://learn.microsoft.com/en-us/sql/
- SQL Server DMVs: https://learn.microsoft.com/en-us/sql/