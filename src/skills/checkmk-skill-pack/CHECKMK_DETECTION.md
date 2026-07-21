# Checkmk Detection Rules Reference

## Purpose

This document describes the detection rules used by the Checkmk skill pack to identify context, symptoms, and issues.

## Context Detection Rules

### Browser URL Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-browser-url | Checkmk web interface | `checkmk\|check_mk\|checkmk/setup\|checkmk/monitoring` | 0.95 | 10 |
| checkmk-detect-browser-setup | Checkmk WATO setup interface | `checkmk/setup\|checkmk/wato\|checkmk/monitoring` | 0.92 | 9 |
| checkmk-detect-browser-monitoring | Checkmk monitoring view | `checkmk/monitoring\|checkmk/view` | 0.90 | 9 |
| checkmk-detect-browser-service-discovery | Service discovery operations | `service.discovery\|service-discovery\|discovered` | 0.85 | 8 |

### Window Title Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-window-title | Checkmk window title | `Checkmk\|Checkmk Setup\|check_mk` | 0.95 | 10 |

### CLI Command Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-cli-cmk | Checkmk CLI usage | `^cmk \w+` | 0.95 | 10 |
| checkmk-detect-cli-agent | Agent command | `check_mk_agent` | 0.93 | 9 |
| checkmk-detect-cli-snmp | SNMP commands | `cmk\s+(snmpwalk\|snmpget\|snmpcheck)` | 0.90 | 8 |
| checkmk-detect-cli-service-discovery | Service discovery CLI | `cmk\s+--service-discovery` | 0.95 | 10 |
| checkmk-detect-cli-debug | Debug mode | `cmk\s+--debug\|--debug` | 0.90 | 9 |
| checkmk-detect-cli-reload | Config reload | `cmk\s+-R` | 0.95 | 10 |
| checkmk-detect-cli-validate | Config validation | `cmk\s+--validate-config` | 0.92 | 9 |
| checkmk-detect-cli-generate-config | Config generation | `cmk\s+(-U\|--generate-config)` | 0.93 | 9 |
| checkmk-detect-cli-perfview | Performance view | `cmk\s+--perfview\|wato\s+perfview` | 0.85 | 7 |

### Process Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-process-cmc | CMC check engine | `check_mk-cmc\|check_mk\.cmc` | 0.95 | 10 |
| checkmk-detect-process-nagios | Nagios Core process | `^nagios\b` | 0.90 | 9 |
| checkmk-detect-process-apache | Apache web server | `apache.*checkmk\|httpd.*checkmk` | 0.85 | 8 |
| checkmk-detect-process-agent | Agent processes | `check_mk\|checkmk.*agent` | 0.90 | 9 |

### Log Pattern Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-log-agent-output | Agent output in logs | `check_mk_agent\|Checkmk agent\|agent check` | 0.92 | 9 |
| checkmk-detect-log-cmc | Micro Core log entries | `CMC\|check_mk-cmc\|micro.core` | 0.95 | 10 |
| checkmk-detect-log-service-discovery | Service discovery logs | `service.discovery\|Service.discovery\|discovered.services\|discovery.log` | 0.88 | 8 |
| checkmk-detect-log-notification | Notification logs | `notification.*sent\|notification.*failed\|notify.*contact\|NOTIFY` | 0.85 | 8 |
| checkmk-detect-log-piggyback | Piggyback data logs | `piggyback\|Piggyback\|piggyback.data\|piggyback.processing` | 0.90 | 9 |
| checkmk-detect-log-stale-check | Stale check warnings | `stale.check\|stale.checks\|check.timed.out\|stale` | 0.90 | 10 |
| checkmk-detect-log-agent-unreachable | Agent failures | `agent.*unreachable\|agent.*timeout\|agent.*error\|Agent.error` | 0.92 | 10 |
| checkmk-detect-log-high-latency | Check latency warnings | `check.latency\|high.latency\|long.check\|check.time` | 0.85 | 9 |

### Configuration File Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-config-main | Main config file | `check_mk_main\.mk\|agents\.mk\|checkmk_main\.mk` | 0.95 | 10 |
| checkmk-detect-config-params | Check parameters | `check_params\|check\.mk\.conf\|local.mk` | 0.90 | 9 |
| checkmk-detect-config-ruleset | Ruleset configuration | `rulesets\|hosts\|mkeventd\.conf\|wato` | 0.90 | 9 |
| checkmk-detect-config-notification | Notification config | `notification\.(mk\|conf)\|contact\.(mk\|conf)\|contacts` | 0.88 | 8 |
| checkmk-detect-config-perfview | Performance view config | `perf.view\|performance.view\|perfview` | 0.80 | 7 |

### State/Status Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-state-critical | Critical service | `CRITICAL\|critical\|state.*critical\|State:.*CRITICAL` | 0.95 | 10 |
| checkmk-detect-state-warning | Warning service | `WARNING\|warning\|state.*warning\|State:.*WARNING` | 0.90 | 9 |
| checkmk-detect-state-unreachable | Host unreachable | `UNREACHABLE\|unreachable\|host.*unreachable\|HOST_UNREACHABLE` | 0.95 | 10 |
| checkmk-detect-state-pending | Pending check | `PENDING\|pending.check\|check.*pending\|State:.*PENDING` | 0.90 | 9 |
| checkmk-detect-state-flapping | Flapping services | `flapping\|Flapping\|state.flapping` | 0.92 | 9 |

### SNMP Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| checkmk-detect-snmp-community | SNMP community strings | `community.*public\|community.*private\|snmp.*community\|SNMP.community` | 0.85 | 7 |
| checkmk-detect-snmp-trap | SNMP trap processing | `snmp.trap\|SNMP.trap\|trap.received\|trap.processing` | 0.88 | 8 |

## Confidence Scoring

### Confidence Levels

- **0.95+**: Near-certain match — strong pattern indicators
- **0.90-0.94**: High confidence — clear indicators
- **0.85-0.89**: Medium-high confidence — strong indicators
- **0.80-0.84**: Low-medium confidence — suggestive indicators
- **< 0.80**: Low confidence — weak indicators

### Confidence Adjustments

Context can increase confidence:
- CLI commands + text patterns about services = higher service confidence
- Multiple matching rules = higher overall context confidence
- Consistent host references = higher context accuracy
- SNMP output + snmpcheck command = higher SNMP confidence

## Priority Levels

Priority determines which detection rules take precedence when multiple rules match:

- **10**: Critical — immediate attention required
- **9**: High — important context
- **8**: Medium — relevant context
- **7**: Low — supporting context
- **< 7**: Informational — supplementary context

## Pattern Extraction

Some rules extract specific values for use in diagnostics:

| Rule ID | Extract Pattern | Example |
|---------|----------------|---------|
| checkmk-detect-cli-cmk | `cmk (get|set|list) (.+)` | `cmk get hosts` |
| checkmk-detect-state-critical | `host/(.+): service/(.+)` | `host/web01: service/http` |
| checkmk-detect-log-cmc | `CMC: (.+)` | `CMC: check timed out` |

## Rule Management

### Adding New Rules

When adding detection rules:
1. Follow the existing YAML format
2. Set appropriate confidence based on pattern specificity
3. Set priority based on operational importance
4. Include clear name and description
5. Test pattern against real Checkmk output

### Rule Ordering

Rules are processed in order of appearance. Higher priority rules should be placed first for faster matching.

## References

- [Checkmk Documentation](https://docs.checkmk.com/)
- [Checkmk Agent Documentation](https://docs.checkmk.com/master/en/agent.html)