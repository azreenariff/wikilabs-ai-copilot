# Checkmk Architecture Reference

## Overview

This document provides architectural details for Checkmk server deployments.

## Core Architecture

### Components

1. **Checkmk Site**: Isolated OMD-based instance
2. **Micro Core (CMC)**: High-performance check engine
3. **Nagios Core**: Legacy check engine (optional)
4. **Livestatus**: Unix socket API
5. **Apache/PHP**: Web interface
6. **PostgreSQL/MySQL**: Database backend
7. **RRD Tool**: Metric storage

### Data Flow

```
Agent → CMC Check Engine → Livestatus → Web/API → Notifications
                  ↓
            Performance Data → RRD → PostgreSQL
```

### Site Architecture

```
/opt/omd/sites/SITENAME/
├── bin/           # Site-specific binaries
├── etc/           # Configuration files
├── var/           # Runtime data (logs, sockets)
├── lib/           # Python libraries
├── share/         # Shared resources
├── local/         # Local customizations
└── tmp/           # Temporary files
```

### Plugin Architecture

| Plugin Type | Purpose | Examples |
|-------------|---------|----------|
| **Agent Plugin** | Data collection | check_mk_agent |
| **Check Plugin** | Service checking | check_mk, snmp |
| **Discovery Plugin** | Service enumeration | inventory |
| **Custom Plugin** | User-defined checks | custom |

## References

- Checkmk Architecture: https://docs.checkmk.com/master/en/internals.html
- Checkmk Micro Core: https://docs.checkmk.com/master/en/micro_core.html
- Checkmk Site Management: https://docs.checkmk.com/master/en/site_management.html