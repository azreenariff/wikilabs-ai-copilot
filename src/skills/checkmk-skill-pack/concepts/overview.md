# Checkmk Architecture Overview

## Overview

Checkmk is a monitoring platform that provides infrastructure and application monitoring with a focus on ease of use, scalability, and automation. It consists of the OMD-based site architecture, the Micro Core (CMC) check engine, and the WATO web-based configuration interface.

## Core Architecture

### Components

1. **Checkmk Site (OMD)**: Each Checkmk instance runs as an isolated site under /opt/omd/sites/
2. **Micro Core (CMC)**: High-performance check engine with parallelization
3. **Nagios Core**: Legacy check engine (optional fallback)
4. **Livestatus**: Unix socket API for status data retrieval
5. **Apache/PHP**: Web interface and API
6. **PostgreSQL/MySQL**: Database backend for long-term data storage
7. **RRD Tool**: Round-robin database for metric storage
8. **Nagios Plugins**: Standard check plugins for host/service monitoring

### Check Execution Architecture

```
WATO/Setup → Rules → CMC Check Engine → Agent → Plugin → Result → Livestatus → Web/API
```

### Data Flow

```
Agent (check_mk_agent) → Agent Plugins → CMC → Livestatus → Web Interface
                              ↓
                    Performance Data → RRD → PostgreSQL
                              ↓
                    Notifications → Contacts
```

### Site Architecture

Each Checkmk site is isolated:
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

### Check Engine Options

| Engine | Description | Use Case |
|--------|-------------|----------|
| **Micro Core (CMC)** | High-performance, parallel | Default for all deployments |
| **Nagios Core** | Legacy engine | Migration, specific plugin support |

### Livestatus API

Livestatus provides programmatic access to monitoring data:
```bash
# Query Livestatus via CLI
echo "GET services" | unixcat /opt/omd/sites/sitename/var/run/live

# SQL-like queries
echo "GET services\nColumns: host_name service_name state" | unixcat /opt/omd/sites/sitename/var/run/live
```

## Scale Architecture

| Scale | Check Interval | Max Hosts | Architecture |
|-------|---------------|-----------|-------------|
| Small (<500) | 30s-60s | 2000 | Single site, single node |
| Medium (500-5000) | 60s | 10000 | Single site, optimized |
| Large (5000+) | 60s+ | 20000+ | Distributed, multiple sites |
| Enterprise | 60s+ | Unlimited | Master/Slave, distributed |

## References

- Checkmk Architecture: https://docs.checkmk.com/master/en/internals.html
- Checkmk Micro Core: https://docs.checkmk.com/master/en/micro_core.html
- Checkmk Site Management: https://docs.checkmk.com/master/en/site_management.html