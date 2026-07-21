# Nagios XI Architecture Reference

## Overview

This document provides architectural details for Nagios XI server deployments.

## Core Architecture

### Components

1. **Nagios Core**: Monitoring engine
2. **NDOUtil**: Database synchronizer
3. **MySQL/MariaDB**: Database backend
4. **Apache/PHP**: Web interface
5. **NRPE/NSClient**: Remote check agents
6. **SNMP**: Network protocol support

### Data Flow

```
Nagios Core
       │
       ├─── Check Execution (Plugins)
       │
       ├─── NDOUtil → MySQL
       │
       └─── Web Interface (Apache/PHP) → Status Display
```

### Database Schema

Key tables: nagios_hosts, nagios_services, nagios_log, nagios_notifications, nagios_statehistory

## Nagios XI Specific Architecture

### WATO Configuration System

- Web-based configuration management
- Database-stored configuration
- Configuration history and rollback
- Validation and error checking

### Event Broker Architecture

- Plugin architecture for event processing
- NDOUtil as event broker module
- Custom event handlers
- External command API

### Monitoring Methods

- NRPE (Linux/Unix)
- NSClient++ (Windows)
- SNMP (Network devices)
- HTTP/HTTPS (Web services)
- ICMP (Host availability)
- Custom plugins (Various)

## References

- Nagios XI Architecture: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios Core Architecture: https://docs.nagios.com/nagioscore/
- Nagios XI Administration: https://assets.nagios.com/downloads/nagiosxi/docs/