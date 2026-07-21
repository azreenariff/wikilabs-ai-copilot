# Checkmk Common Failure Patterns

## Overview

This reference documents common Checkmk failure patterns, their symptoms, diagnosis methods, and recommended actions.

## Common Failures

### Agent Connection Failure

**Symptoms**:
- Host shows CRITICAL or UNKNOWN
- Agent not responding
- Connection refused errors

**Diagnosis**:
```bash
# Test agent connectivity
nc -zv <host> 6556

# Test agent data
echo '' | nc localhost 6556 | head -20

# Check agent service
systemctl status check-mk-agent
```

**Remediation**:
1. Verify agent service is running
2. Check firewall rules allow port 6556
3. Verify agent configuration on host
4. Restart agent service
5. Check agent logs

### Service Discovery Issues

**Symptoms**:
- Services not discovered
- New services not showing
- Duplicate services found

**Diagnosis**:
```bash
# Run service discovery
check_mk -I <host>

# Check inventory results
check_mk -I --discover <host>

# Remove discovered services
check_mk -I --remove <host>
```

**Remediation**:
1. Verify agent output for services
2. Check service rules configuration
3. Clear cached agent data
4. Re-run discovery
5. Apply discovered services

### Notification Issues

**Symptoms**:
- Alerts not reaching contacts
- Notification failures
- Contact routing issues

**Diagnosis**:
```bash
# Check notification queue
omd sitename run check_mk -O

# Check notification configuration
omd sitename config notification.cfg

# Check mail delivery
mailq
```

**Remediation**:
1. Verify SMTP configuration
2. Check contact email addresses
3. Verify notification rules
4. Test notification command
5. Check mail server logs

### Performance Degradation

**Symptoms**:
- Slow web interface
- Check execution delays
- High CPU usage

**Diagnosis**:
```bash
# Check system resources
top -bn1 | head -20

# Check Checkmk status
omd sitename status

# Check agent cache
cat /var/tmp/check_mk/cache/<host>
```

**Remediation**:
1. Review check intervals
2. Optimize plugin execution
3. Clear agent cache
4. Check for stuck checks
5. Consider distributed monitoring

### Piggyback Data Issues

**Symptoms**:
- Cluster services missing
- Child host data not showing
- Piggyback data errors

**Diagnosis**:
```bash
# Check piggyback configuration
check_mk -p <parent_host>

# Verify piggyback data
check_mk -D <parent_host>
```

**Remediation**:
1. Verify parent host configuration
2. Check piggyback data delivery
3. Restart parent host checks
4. Verify child host configuration
5. Check agent piggyback output

## References

- Checkmk Troubleshooting: https://docs.checkmk.com/master/en/
- Checkmk Common Issues: https://docs.checkmk.com/master/en/
- Checkmk Support: https://checkmk.com/support