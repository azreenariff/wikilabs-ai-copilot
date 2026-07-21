# Checkmk Monitoring Architecture

## Overview

Checkmk monitoring architecture covers the check execution model, plugin ecosystem, data flow, and integration patterns.

## Core Monitoring Components

### Check Execution Flow

```
WATO → Rules → CMC → Agent → Plugin → Result → Livestatus → Web/API
```

### Agent Architecture

The Checkmk agent collects data from various sources:

1. **Agent Program**: Main agent script
2. **Agent Plugins**: Data collection scripts
3. **Agent Cache**: Cached data for deduplication
4. **Agent Relay**: Data relay for piggyback monitoring

### Check Plugin Types

| Type | Description | Use Case |
|------|-------------|----------|
| **Agent Plugin** | Collects data from agent | Standard monitoring |
| **Check Plugin** | Processes agent data | Service checking |
| **Inventory Plugin** | Discovers services | Auto-discovery |
| **Discovery Plugin** | Finds new services | Service enumeration |
| **Custom Plugin** | User-defined checks | Custom monitoring |
| **SNMP Plugin** | SNMP-based checks | Network devices |

### Cluster Monitoring Architecture

```
Parent Host → Piggyback Data → Child Hosts
     ↓
Agent Plugins → Check Plugins → Results
```

### Notification Architecture

```
Check Result → Notification Rules → Contact → Notification Method
```

### Performance Data Pipeline

```
Check Plugin → Perfdata → RRD → PostgreSQL → Visualization
```

## Integration Patterns

### External Systems

| System | Integration Method | Purpose |
|--------|-------------------|---------|
| **Nagios XI** | Livestatus API | Unified monitoring |
| **Zabbix** | SNMP traps | Alert forwarding |
| **Elasticsearch** | Log integration | Log monitoring |
| **Prometheus** | Exporter | Metrics export |
| **Slack/Teams** | Webhooks | Notifications |

### API Integration

```bash
# Livestatus query via socket
echo "GET services" | unixcat /opt/omd/sites/sitename/var/run/live

# REST API (Enterprise)
curl -u user:password https://checkmk.example.com/check_mk/webapi.py?action=hosts
```

## References

- Checkmk Monitoring: https://docs.checkmk.com/master/en/
- Checkmk Plugin Architecture: https://docs.checkmk.com/master/en/
- Checkmk Integration: https://docs.checkmk.com/master/en/