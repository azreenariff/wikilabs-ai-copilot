# Checkmk Diagnostic Procedures

## Overview

Systematic diagnostic procedures for common Checkmk issues.

## Connectivity Issues

### Agent Connectivity

**Symptoms**:
- Host not reachable
- Agent not responding

**Diagnostic Steps**:
```bash
# Check agent service
systemctl status check-mk-agent

# Test agent port
nc -zv <host> 6556

# Get agent output
echo '' | nc localhost 6556
```

**Evidence Required**:
- Agent service status
- Port connectivity test
- Agent output sample

**Remediation**:
1. Start agent service if stopped
2. Fix firewall rules
3. Verify agent configuration
4. Restart agent
5. Test again

### Livestatus Issues

**Symptoms**:
- Web interface not loading
- API queries failing

**Diagnostic Steps**:
```bash
# Check Livestatus socket
ls -la /opt/omd/sites/sitename/var/run/live

# Test Livestatus
echo "GET hosts" | unixcat /opt/omd/sites/sitename/var/run/live

# Check web server
systemctl status apache2
```

**Evidence Required**:
- Socket existence and permissions
- Livestatus query result
- Web server status

**Remediation**:
1. Verify socket exists
2. Check permissions
3. Restart Apache
4. Reload site
5. Test again

## Configuration Issues

### Ruleset Issues

**Symptoms**:
- Rules not applying
- Check parameters wrong
- Inherited rules broken

**Diagnostic Steps**:
```bash
# Validate rulesets
check_mk --validate-rulesets

# List rules
check_mk -D --list-rulesets

# Check effective configuration
check_mk -d <host>
```

**Evidence Required**:
- Validation result
- Rules list output
- Effective config

**Remediation**:
1. Fix syntax errors
2. Remove conflicting rules
3. Verify inheritance
4. Reload configuration
5. Test again

### Host Configuration

**Symptoms**:
- Host not monitored
- Wrong IP address
- Agent connection failing

**Diagnostic Steps**:
```bash
# Check host configuration
check_mk -D <host>

# Test connectivity
ping <host>
nc -zv <host> 6556

# Check WATO configuration
# (via web interface)
```

**Evidence Required**:
- Host configuration
- Connectivity test
- Agent output

**Remediation**:
1. Fix IP address
2. Verify WATO settings
3. Update agent configuration
4. Reload site
5. Test again

## References

- Checkmk Diagnostics: https://docs.checkmk.com/master/en/
- Checkmk Troubleshooting: https://docs.checkmk.com/master/en/
- Checkmk Debugging: https://docs.checkmk.com/master/en/