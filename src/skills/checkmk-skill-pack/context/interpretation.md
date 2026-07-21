# Checkmk Context Interpretation

## Overview

This document explains how to interpret Checkmk outputs, logs, and configuration.

## Interpreting Check Results

### Check States

| State | Value | Description |
|-------|-------|-------------|
| **OK** | 0 | Service is healthy |
| **WARNING** | 1 | Service degraded |
| **CRITICAL** | 2 | Service down |
| **UNKNOWN** | 3 | Cannot determine state |
| **PENDING** | 4 | Not yet checked |
| **SKIPPED** | 5 | Check skipped |

### Check Result Format

```
State|Message|Performance Data
1|Disk usage is 85% (warn at 80%, crit at 90%)|usage=85%;80;90 total=100
```

### Performance Data Format

```
label=value[;min][;max][;min_warn][;max_warn][;min_crit][;max_crit]
```

## Interpreting Agent Output

### Agent Output Format

```
<<<local>>>
Checkmk agent local output

<<<check_mk>>>
version: 1.6.0p11
agent_version: 1.6.0p11
OS: linux
Hostname: webserver01

<<<df>>>
Filesystem  Type  Size  Used  Avail  Use%  Mounted on
/dev/sda1   ext4  100G  85G   15G   85%  /
```

### Agent Output Sections

| Section | Purpose |
|---------|---------|
| **local** | Local agent data |
| **check_mk** | Agent metadata |
| **df** | Disk space |
| **cpu.loads** | CPU load averages |
| **mem.linux** | Memory usage |
| **host.cpu** | CPU metrics |
| **net.ifaces** | Network interfaces |
| **agent_version** | Agent version info |

## Interpreting Livestatus Output

### Livestatus Queries

```bash
# Get all services
echo "GET services" | unixcat /opt/omd/sites/sitename/var/run/live

# Get services with columns
echo -e "GET services\nColumns: host_name service_name state ack\nFilter: state = 2" | unixcat /opt/omd/sites/sitename/var/run/live

# Get host information
echo -e "GET hosts\nColumns: host_name address state plugins" | unixcat /opt/omd/sites/sitename/var/run/live
```

### Livestatus Filters

| Filter | Description |
|--------|-------------|
| **state = 0** | Only OK services |
| **state = 2** | Only CRITICAL services |
| **host_name = web01** | Specific host |
| **service_name ~ mysql** | Service name pattern |
| **plugins ~ check_mk** | Agent-based checks |

## Interpreting WATO Configuration

### Configuration Layers

1. **Host/Service Settings**: Per-object configuration
2. **Rulesets**: Rule-based configuration
3. **Global Settings**: Site-wide defaults
4. **Agent Configuration**: Agent-specific settings

### Ruleset Hierarchy

```
Global Rules → Host Group Rules → Host Rules → Service Rules
```

## References

- Checkmk Interpreting Output: https://docs.checkmk.com/master/en/
- Checkmk Check Results: https://docs.checkmk.com/master/en/
- Checkmk Livestatus: https://docs.checkmk.com/master/en/livestatus.html