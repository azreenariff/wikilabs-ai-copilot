# Nagios XI Diagnostic Reasoning Reference

## Reasoning Patterns

### Pattern 1: Monitoring Gap

```
Observation: Services not being monitored
  ├── Nagios Core running?
  │   ├── No: Start/fix Nagios
  │   └── Yes: Check configuration
  ├── Host defined?
  │   ├── No: Add host configuration
  │   └── Yes: Check service definition
  └── Check enabled?
      ├── No: Enable service checks
      └── Yes: Check connectivity
```

### Pattern 2: Alert Failure

```
Observation: Alert not received
  ├── Contact configured?
  │   ├── No: Add contact
  │   └── Yes: Check contact group
  ├── Notification enabled?
  │   ├── No: Enable notifications
  │   └── Yes: Check notification options
  └── Notification method working?
      ├── No: Fix notification method
      └── Yes: Check contact availability
```

### Pattern 3: Performance Degradation

```
Observation: Slow web interface
  ├── Database growing?
  │   ├── Yes: Clean old data
  │   └── No: Check database performance
  ├── CPU high?
  │   ├── Yes: Check Nagios processes
  │   └── No: Check web server
  └── Disk space low?
      ├── Yes: Free disk space
      └── No: Check network
```

## Confidence Assessment

### Evidence Quality

| Evidence Type | Quality | Example |
|--------------|---------|---------|
| Direct error | High | Log entry with error message |
| Service status | High | ps/kill/status output |
| Database metrics | Medium | Table sizes, query times |
| Web interface | Medium | Status display issues |
| User reports | Low | "System feels slow" |

### Confidence Scoring

1. **High (85-100%)**: Multiple high-quality evidence points
2. **Medium (60-84%)**: Some evidence with supporting data
3. **Low (30-59%)**: Limited evidence, needs more investigation
4. **Very Low (0-29%)**: Insufficient evidence

## References

- Nagios XI Diagnostics: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Troubleshooting: https://assets.nagios.com/downloads/nagiosxi/docs/
- Monitoring Diagnostics: https://www.monitoringportal.org/