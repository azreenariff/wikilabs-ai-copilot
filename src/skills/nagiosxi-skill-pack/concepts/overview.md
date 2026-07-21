# Nagios XI Architecture Overview

## Overview

Nagios XI is a comprehensive network monitoring platform built on the Nagios Core engine with a web-based management interface. It provides real-time monitoring, alerting, and reporting for IT infrastructure.

## Core Architecture

### Components

1. **Nagios Core**: The monitoring engine that performs checks and generates alerts
2. **NDOUtil (Nagios Data Out)**: Translates Nagios events into database records
3. **MySQL/MariaDB Database**: Stores configuration, historical data, and status
4. **PHP Web Interface**: Manages configuration, displays status, generates reports
5. **NRPE (Nagios Remote Plugin Executor)**: Agent for remote Linux/Unix hosts
6. **NSClient++**: Agent for Windows hosts
7. **SNMP**: Protocol for monitoring network devices
8. **Check_MK Agent**: Alternative agent-based monitoring plugin

### Data Flow

```
Monitoring Targets
       │
       ├─── Nagios Core Engine
       │        │
       │        ├─── NRPE/NSClient++/SNMP Checks
       │        │
       │        └─── NDOUtil ──→ MySQL Database
       │
       └─── Web Interface ──→ Status Display/Reports
```

### Database Schema

Nagios XI uses several key tables:
- **nagios_hosts**: Host configuration and status
- **nagios_services**: Service configuration and status
- **nagios_comments**: Historical comments
- **nagios_notifications**: Notification history
- **nagios_log**: Event log
- **nagios_statehistory**: State change history

### Monitoring Methods

| Method | Protocol | Use Case |
|--------|----------|----------|
| NRPE | TCP 5666 | Linux/Unix hosts |
| NSClient++ | TCP 12489 | Windows hosts |
| SNMP | UDP 161/162 | Network devices |
| Check Command | Local | Server-side checks |
| HTTP/HTTPS | TCP 80/443 | Web services |
| ICMP | IP | Ping/host availability |
| SMTP | TCP 25 | Mail servers |
| POP3/IMAP | TCP 110/143 | Mail servers |
| SSH | TCP 22 | Secure shell |
| FTP | TCP 21 | File transfer |
| JDBC | TCP | Database monitoring |
| JMX | TCP | Java applications |

### High Availability

Nagios XI HA setup includes:
- Primary/Standby servers
- Shared storage or database replication
- Heartbeat/corosync for failover
- VIP (Virtual IP) for access

### Performance Considerations

1. **Database Size**: Large monitoring environments need optimized MySQL
2. **Check Intervals**: Balancing check frequency with system load
3. **Network Bandwidth**: SNMP polling and plugin output
4. **CPU/Memory**: Monitoring server resources for check execution
5. **Disk Space**: Log retention and historical data

## References

- Nagios XI Documentation: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios Core Documentation: https://docs.nagios.com/nagioscore/
- NRPE Documentation: https://github.com/NagiosEnterprises/nrpe