# Checkmk Diagnostic Reasoning Reference

## Reasoning Patterns

### Pattern 1: Agent Connectivity Failure

```
Observation: Host not reachable
  ├── Agent running?
  │   ├── No: Start agent
  │   └── Yes: Check network
  ├── Firewall allowing port?
  │   ├── No: Open port
  │   └── Yes: Check configuration
  └── Agent configuration correct?
      ├── No: Fix configuration
      └── Yes: Check logs
```

### Pattern 2: Service Discovery Issues

```
Observation: Services not discovered
  ├── Agent output present?
  │   ├── No: Fix agent
  │   └── Yes: Check plugins
  ├── Rules matching?
  │   ├── No: Fix rules
  │   └── Yes: Check cache
  └── Cache stale?
      ├── Yes: Clear cache
      └── No: Re-run discovery
```

### Pattern 3: Performance Degradation

```
Observation: Slow system
  ├── Check interval high?
  │   ├── Yes: Lower interval
  │   └── No: Check other factors
  ├── CPU/memory high?
  │   ├── Yes: Optimize
  │   └── No: Check network
  └── Database slow?
      ├── Yes: Optimize DB
      └── No: Check other factors
```

## Confidence Assessment

### Evidence Quality

| Evidence Type | Quality | Example |
|--------------|---------|---------|
| Direct error | High | Checkmk error message |
| Agent output | High | Agent data output |
| Livestatus data | Medium | Status query result |
| User reports | Low | "System is slow" |

### Confidence Scoring

1. **High (85-100%)**: Multiple high-quality evidence points
2. **Medium (60-84%)**: Some evidence with supporting data
3. **Low (30-59%)**: Limited evidence, needs more investigation
4. **Very Low (0-29%)**: Insufficient evidence

## References

- Checkmk Diagnostics: https://docs.checkmk.com/master/en/
- Checkmk Troubleshooting: https://docs.checkmk.com/master/en/
- Checkmk Debugging: https://docs.checkmk.com/master/en/