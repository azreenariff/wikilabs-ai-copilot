# Nagios XI Monitoring Architecture

## Overview

Nagios XI monitoring architecture covers the check execution model, plugin ecosystem, data flow, and integration patterns. Understanding architecture is critical for effective monitoring design and troubleshooting.

## Check Execution Model

### Active vs Passive Checks

| Type | Initiated By | Use Case |
|------|-------------|----------|
| **Active** | Nagios Core engine | Standard monitoring checks |
| **Passive** | External source | Load balancers, CI/CD, external monitoring |

### Active Check Flow

```
1. Nagios Core evaluates check schedule
2. Check command is dispatched to worker
3. Plugin executes on monitoring server or remote host
4. Plugin returns exit code and output
5. Nagios Core processes results
6. NDOUtil updates database
7. Web interface displays status
8. Notifications sent if state changed
```

### Passive Check Flow

```
1. External source sends check result
2. Nagios Core receives via external command
3. Nagios Core processes result
4. NDOUtil updates database
5. Web interface displays status
6. Notifications sent if state changed
```

## Plugin Architecture

### Plugin Types

1. **Host Checks**: Determine host availability (ping, SSH, TCP)
2. **Service Checks**: Determine service health (HTTP, MySQL, CPU)
3. **Dependency Checks**: Handle cascading failures
4. **Notification Checks**: Trigger notification workflows

### Plugin Location

```bash
/usr/lib/nagios/plugins/     # Core plugins
/usr/local/nagios/libexec/  # Custom plugins
/var/lib/nagios/plugins/    # System plugins
```

### Plugin Exit Codes

| Code | Meaning |
|------|---------|
| 0 | OK |
| 1 | WARNING |
| 2 | CRITICAL |
| 3 | UNKNOWN |

### Plugin Output Format

```
Plugin output | perfdata=value;warn;crit;min;max
```

Example:
```
HTTP OK: 200 in 0.523 seconds | time=0.523s;5;10;0;
```

## Monitoring Protocols

### NRPE (Nagios Remote Plugin Executor)

NRPE allows Nagios to execute checks on remote Linux/Unix hosts.

**Configuration**:
- Server: `/etc/nagios/nrpe.cfg`
- Port: TCP 5666
- Security: Option 2 (TLS) recommended

**Usage**:
```bash
# Check remote host via NRPE
/usr/lib/nagios/plugins/check_nrpe -H 192.168.1.100 -c check_load

# Check with arguments
/usr/lib/nagios/plugins/check_nrpe -H 192.168.1.100 -c check_disk -a -w 20 -c 10
```

### NSClient++

NSClient++ is the Windows agent for remote monitoring.

**Configuration**:
- File: `nsclient.ini`
- Port: TCP 12489
- Security: TLS and password authentication

**Usage**:
```bash
# Check remote Windows host via NSClient++
/usr/lib/nagios/plugins/check_nt -H 192.168.1.100 -p 12489 -s password -v CPULOAD -l 80,60,50
```

### SNMP

Simple Network Management Protocol for network devices.

**Configuration**:
- Community string (v1/v2c) or credentials (v3)
- Port: UDP 161 (queries), UDP 162 (traps)
- MIBs: Management Information Base definitions

**Usage**:
```bash
# SNMP walk
snmpwalk -v2c -c public 192.168.1.1 .1.3.6.1.2.1.1

# SNMP get
snmpget -v2c -c public 192.168.1.1 .1.3.6.1.2.1.1.5.0
```

### HTTP/HTTPS

Web service monitoring via HTTP/HTTPS checks.

**Usage**:
```bash
# Check HTTP status
/usr/lib/nagios/plugins/check_http -H webserver -u /health -w 5 -c 10

# Check HTTPS with certificate expiry
/usr/lib/nagios/plugins/check_http -H webserver -S -C 30 -w 15 -c 7
```

### ICMP/Ping

Host availability checking via ICMP echo.

**Usage**:
```bash
# Ping check
/usr/lib/nagios/plugins/check_ping -H 192.168.1.100 -w 100,20% -c 500,50%
```

## Data Flow Architecture

### Event Processing Chain

```
Check Execution
       │
       ├── Plugin returns result
       │
       ├── Nagios Core processes result
       │
       ├── NDOUtil converts to DB format
       │
       ├── MySQL stores in NDODB
       │
       ├── Web Interface reads from NDODB
       │
       └── Notifications dispatched
```

### NDOUtil Database Tables

| Table | Purpose |
|-------|---------|
| nagios_hosts | Host configuration and status |
| nagios_services | Service configuration and status |
| nagios_hoststatus | Current host status information |
| nagios_servicestatus | Current service status information |
| nagios_statehistory | Historical state changes |
| nagios_comments | Comments and acknowledgements |
| nagios_notifications | Notification history |
| nagios_log | Event log entries |
| nagios_programstatus | Program status information |

## Integration Patterns

### External Systems Integration

1. **Active Directory/LDAP**: Authentication integration
2. **LDAP/AD Groups**: Contact group synchronization
3. **SNMP Traps**: Incoming trap processing
4. **Email Integration**: Notification via SMTP
5. **Slack/Teams**: Webhook notifications
6. **REST API**: Programmatic access
7. **Database Queries**: JDBC/ODBC monitoring

### High Availability Architecture

```
                  ┌─────────────────────┐
                  │   VIP (Floating IP) │
                  └──────────┬──────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
      ┌───────▼───────┐          ┌─────────▼─────────┐
      │   Primary      │          │   Standby          │
      │   Nagios XI   │          │   Nagios XI       │
      └───────┬───────┘          └─────────┬─────────┘
              │                             │
      ┌───────▼───────┐          ┌─────────▼─────────┐
      │  MySQL (Master)│          │  MySQL (Replica)  │
      └───────────────┘          └───────────────────┘
```

### Performance Scaling

| Scale | Nodes | Considerations |
|-------|-------|---------------|
| Small (<500 hosts) | Single | No scaling needed |
| Medium (500-2000) | Multiple cores | Optimize check intervals |
| Large (2000-10000) | Multiple servers | Distributed monitoring |
| Enterprise (>10000) | Multiple sites | Geographic distribution |

## References

- Nagios XI Architecture: https://assets.nagios.com/downloads/nagiosxi/docs/
- NRPE Documentation: https://github.com/NagiosEnterprises/nrpe
- NSClient++ Documentation: https://docs.nsclient.org/
- Nagios XI Plugin Development: https://assets.nagios.com/downloads/nagiosxi/docs/