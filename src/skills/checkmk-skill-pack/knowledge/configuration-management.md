# Checkmk Configuration Management

## Overview

Checkmk configuration management covers WATO, rulesets, agent configuration, and site settings.

## WATO Configuration

### WATO Components

1. **Host Configuration**: Per-host settings
2. **Service Configuration**: Per-service settings
3. **Rulesets**: Rule-based configuration
4. **Notifications**: Alert configuration
5. **Time Periods**: Time-based rules
6. **Contact Management**: Contact definitions

### Host Configuration

| Setting | Description |
|---------|-------------|
| **IP Address** | Host IP address |
| **Tags** | Host categorization |
| **Checkmk Agent** | Agent connection settings |
| **SNMP Community** | SNMP community string |
| **Contact Groups** | Notification contacts |
| **Monitoring Period** | Active monitoring hours |

### Ruleset Management

Rulesets provide centralized configuration:

1. **Check Parameters**: Check-specific parameters
2. **Notification Rules**: Alert notification rules
3. **Contact Rules**: Contact assignment rules
4. **Service Rules**: Service discovery rules
5. **Custom Rules**: User-defined rules

## Agent Configuration

### Agent Settings

```bash
# Agent configuration file
/etc/check_mk/conf.d/check_mk_agent.cfg

# Agent configuration options
agent_max_retries = 3
agent_cache = 300
agent_ipv6 = yes
agent_tls = yes
```

### Agent Deployment

```bash
# Install agent on Linux
apt-get install check-mk-agent

# Configure agent
systemctl enable check-mk-agent
systemctl start check-mk-agent

# Test agent
echo '' | nc localhost 6556
```

## Site Configuration

### Site Settings

```bash
# Site configuration
omd config list

# Check site status
omd status

# Apply configuration
omd reload
```

### OMD Site Management

| Command | Description |
|---------|-------------|
| `omd create` | Create new site |
| `omd start` | Start site |
| `omd stop` | Stop site |
| `omd reload` | Reload configuration |
| `omd restart` | Restart site |
| `omd rename` | Rename site |
| `omd remove` | Remove site |

## References

- Checkmk Configuration: https://docs.checkmk.com/master/en/
- Checkmk WATO: https://docs.checkmk.com/master/en/wato.html
- Checkmk Site Management: https://docs.checkmk.com/master/en/site_management.html