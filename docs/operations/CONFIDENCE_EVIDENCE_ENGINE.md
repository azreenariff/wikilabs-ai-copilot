# Confidence & Evidence Engine

## Purpose

This document defines the standardized response metadata engine for all operational recommendations. Every operational recommendation must include structured metadata that enables human trust and verification without exposing internal LLM chain-of-thought.

## Architecture Decision Record

**ADR-001:** The Confidence & Evidence Engine provides structured output metadata for all guidance outputs. This ensures consistency, transparency, and human-verifiable reasoning across all monitoring, automation, and database skill packs.

---

## Response Metadata Structure

Every operational recommendation MUST include these four fields:

```yaml
recommendation: "Clear, actionable advice the engineer can perform"
why: "Concrete reasoning explaining the root cause or operational risk"
confidence_score:
  level: "High|Medium|Low"
  percentage: 85
  justification: "Specific evidence that supports this confidence level"
observable_evidence:
  - "Signal 1: Source and what it shows"
  - "Signal 2: Source and what it shows"
  - "Signal 3: Source and what it shows"
```

---

## Confidence Scoring Framework

### Confidence Levels

| Level | Percentage Range | Description | Example |
|-------|-----------------|-------------|---------|
| **High** | 85-100% | Clear evidence, single obvious root cause, low ambiguity | "Disk full: `/dev/sda1` at 98%" |
| **Medium** | 60-84% | Multiple possible causes, partial evidence, moderate ambiguity | "Service slow: high CPU on host but no clear cause" |
| **Low** | 30-59% | Insufficient evidence, many possible causes, high ambiguity | "User reports slow: no data collected yet" |
| **Very Low** | 0-29% | No relevant evidence, pure speculation | "Something might be wrong but no indicators" |

### Confidence Determination Rules

1. **Signal count:** More corroborating signals → higher confidence
2. **Signal quality:** Direct error messages > symptoms > user reports
3. **Cross-domain correlation:** Evidence spanning multiple domains → higher confidence
4. **Historical precedent:** Known issues matching current symptoms → higher confidence
5. **Ambiguity:** Fewer competing explanations → higher confidence

### Confidence Signal Weighting

| Signal Type | Weight | Example |
|------------|--------|---------|
| Direct error message | +25% | "OutOfMemoryError" |
| Threshold breach | +20% | CPU > 95% |
| Log pattern match | +15% | "connection refused" in logs |
| Configuration mismatch | +10% | Wrong version detected |
| Historical pattern match | +10% | Similar incident resolved before |
| User report | +5% | "System is slow" |
| Correlated metric | +10% | Multiple related metrics affected |

---

## Evidence Collection Guidelines

### Evidence Sources

| Source | What to Look For | Example |
|--------|-----------------|---------|
| **Screen** | UI state, error dialogs, dashboards | Red alert in Checkmk, Nagios service in CRITICAL state |
| **Terminal** | Command output, error codes, process status | `systemctl status`, `mysqladmin status` |
| **Browser** | URL patterns, web UI state, error pages | Nagios XI main page showing downtime |
| **Logs** | Error patterns, stack traces, timestamps | `/var/log/syslog` containing OOM kills |
| **Files** | Configuration values, file sizes, permissions | `/etc/nagios/nagios.cfg` with wrong settings |
| **Processes** | Running processes, resource usage | `top` showing mysqld consuming 95% CPU |

### Evidence Quality Standards

1. **Specific:** Reference exact values, timestamps, filenames
2. **Verifiable:** Include the command or source that produced the evidence
3. **Contextual:** Include what was normal vs what is abnormal
4. **Timely:** Note when evidence was collected
5. **Complete:** Include both what exists and what is missing

### Evidence Examples

#### High Confidence Evidence

```yaml
observable_evidence:
  - "Screen: Checkmk dashboard shows host 'db-prod-01' with status RED and 'disk full' alert"
  - "Terminal: `df -h` shows `/dev/sda1` at 99% on db-prod-01"
  - "Logs: `/var/log/mysql/error.log` contains 'InnoDB: Unable to lock ./ibdata1 error: 11'"
```

#### Medium Confidence Evidence

```yaml
observable_evidence:
  - "Screen: Nagios XI shows service 'HTTP Check' in WARNING state"
  - "Terminal: `curl -I http://localhost` returns 200 but with 3s response time"
  - "Terminal: `systemctl status nginx` shows 'active (running)' but high CPU reported"
```

#### Low Confidence Evidence

```yaml
observable_evidence:
  - "User report: 'Database is slow this morning'"
  - "No terminal output collected yet"
  - "No screen state visible"
```

---

## Recommendation Guidelines

### Recommendation Format

1. **Actionable:** Must specify what the engineer should do
2. **Specific:** Include exact commands, paths, or settings
3. **Risky vs Safe:** Clearly separate emergency actions from diagnostic steps
4. **Ordered:** Present steps in investigation → remediation order
5. **Advisory only:** Never execute, always recommend

### Recommendation Patterns

| Pattern | When to Use | Example |
|---------|-------------|---------|
| **Diagnostic First** | Root cause unknown | "First run `systemctl status nagios`, then check..." |
| **Emergency Remediation** | Active incident | "Immediately archive old logs: `find /var/log -name '*.log' -mtime +7 -delete`" |
| **Preventive** | Known risk detected | "Configure alert at 70% to catch full disk before it reaches 95%" |
| **Investigation Sequence** | Complex symptoms | "1. Check X, 2. If Y, check Z, 3. If Z shows..." |

---

## Domain-Specific Evidence Collection

### Monitoring Evidence Collection

#### Nagios XI
| Source | Evidence | Commands |
|--------|----------|---------|
| Web UI | Service status, host status, downtime, notifications | Nagios XI dashboard, service detail pages |
| Terminal | Nagios process, config, performance data | `systemctl status nagios`, `nrpe -n`, `check_mysql` |
| Logs | Nagios core logs, notification logs | `/usr/local/nagios/var/nagios.log`, `/usr/local/nagios/var/rw/nagios.cmd` |
| Config | Service definitions, hostgroups, contacts | `/usr/local/nagios/etc/` directory |

#### Nagios Log Server
| Source | Evidence | Commands |
|--------|----------|---------|
| Web UI | Search results, dashboards, alerts, cluster health | Log Server search page, cluster health dashboard |
| Terminal | Elasticsearch status, cluster health | `curl -s http://localhost:9200/_cluster/health`, `systemctl status elasticsearch` |
| Logs | Elasticsearch logs, Logstash pipeline logs | `/var/log/elasticsearch/`, `/var/log/logstash/` |
| Config | Logstash pipelines, input/output config | `/etc/logstash/conf.d/` |

#### Checkmk
| Source | Evidence | Commands |
|--------|----------|---------|
| Web UI | Service discovery results, rulesets, notification status | Checkmk WATO setup, main view, service list |
| Terminal | Checkmk agent, CMC status, performance data | `cmk --debug`, `check_mk_agent`, `systemctl status check_mk` |
| Logs | Checkmk core logs, agent logs | `/var/log/check_mk/`, agent output |
| Config | Checkmk agent plugins, ruleset definitions | `/etc/check_mk/`, agent plugins directory |

### Database Evidence Collection

#### MySQL / MariaDB
| Source | Evidence | Commands |
|--------|----------|---------|
| Terminal | Connection count, thread count, replication status | `mysqladmin status`, `SHOW STATUS`, `SHOW SLAVE STATUS` |
| Logs | Slow query log, error log, general log | `/var/log/mysql/error.log`, `slow-query.log` |
| Config | Buffer pool size, max connections, replication settings | `my.cnf`, `SHOW VARIABLES` |
| Process | Running queries, lock waits, connections | `SHOW PROCESSLIST`, `SHOW ENGINE INNODB STATUS` |

#### EDB PostgreSQL
| Source | Evidence | Commands |
|--------|----------|---------|
| Terminal | Connection count, WAL usage, replication lag | `psql -c "SELECT * FROM pg_stat_activity"`, `pg_wal_lsn_diff` |
| Logs | PostgreSQL log file, autovacuum messages | `/var/log/postgresql/`, `log_statement`, `log_autovacuum_min_duration` |
| Config | shared_buffers, wal_level, max_connections | `postgresql.conf`, `pg_hba.conf` |
| Process | Active queries, lock waits, bgwriter stats | `pg_stat_activity`, `pg_stat_bgwriter` |

#### Microsoft SQL Server
| Source | Evidence | Commands |
|--------|----------|---------|
| SSMS/Management Studio | Activity Monitor, blocking session tree, deadlock graphs | SSMS Activity Monitor, Object Explorer |
| Terminal | SQLCMD query results, DMV views | `sqlcmd -Q "SELECT * FROM sys.dm_os_wait_stats"` |
| Logs | SQL Server error log, agent log | SQL Server Log Manager, `ERRORLOG` files |
| Config | Max server memory, max degree of parallelism, recovery model | `sp_configure`, `sys.configurations` |

### Ansible Evidence Collection
| Source | Evidence | Commands |
|--------|----------|---------|
| Terminal | Playbook output, task results, connection status | `ansible-playbook --check --diff`, `ansible-inventory --list` |
| Logs | Ansible logs, callback plugin output | `ANSIBLE_LOG_PATH`, callback plugins |
| Config | ansible.cfg, inventory, group_vars | `ansible.cfg`, `inventory/` directory |
| Files | Playbook syntax, role structure | `ansible-playbook --syntax-check`, role directory tree |

---

## Cross-Domain Evidence Patterns

### Monitoring → Database Correlation

| Symptom | Monitoring Evidence | Database Evidence | Confidence |
|---------|-------------------|-------------------|------------|
| Database slowdown | Nagios HTTP check latency rising | Slow query log entries increasing | High |
| Database unreachable | Nagios service CRITICAL | MySQL process stopped, connection refused | High |
| Disk pressure | Nagios disk check WARNING | WAL/transaction log full | Medium |

### Log → Database Correlation

| Symptom | Log Evidence | Database Evidence | Confidence |
|---------|-------------|-------------------|------------|
| Out of memory | Log Server shows OOM kills | Database OOM-killer triggered | High |
| Disk full | Log Server shows full disk alerts | Database unable to write WAL | High |
| CPU spike | Log Server shows application errors | Query planner using bad plan | Medium |

### Automation → Database Correlation

| Symptom | Automation Evidence | Database Evidence | Confidence |
|---------|-------------------|-------------------|------------|
| Schema change issue | Ansible playbook applied `ALTER TABLE` | Slow queries after table change | Medium |
| Config change issue | Ansible updated `innodb_buffer_pool_size` | Buffer pool warmup, performance variance | Medium |
| Version upgrade issue | Ansible upgraded MySQL package | Version-specific feature changes | High |

---

## Version Metadata

### Version-Aware Evidence Collection

Each skill pack manifest MUST include:

```yaml
supported_versions:
  - "2024+"        # Minimum supported version
  - "2.2"          # Latest tested version
  - "8.0"          # Current major version
  
deprecated_parameters:
  - parameter: "old_setting"
    since_version: "2.1"
    alternative: "new_setting"
    action: "Update configuration"
```

### Documentation URL Mapping

| Skill Pack | Documentation URL Pattern |
|-----------|--------------------------|
| Nagios XI | `https://assets.nagios.com/downloads/nagiosxi/docs/` |
| Nagios Log Server | `https://docs.nagios.com/nagios-log-server/` |
| Checkmk | `https://docs.checkmk.com/` |
| Ansible | `https://docs.ansible.com/` |
| MySQL | `https://dev.mysql.com/doc/` |
| EDB PostgreSQL | `https://www.enterprisedb.com/docs/` |
| MSSQL | `https://learn.microsoft.com/sql/` |

---

## Operational Safeguards

### Advisory Boundaries

The Confidence & Evidence Engine enforces:

1. **No execution:** Never run commands, only recommend
2. **No automation:** Never apply playbooks, only suggest
3. **No threshold changes:** Never modify monitoring thresholds, only advise
4. **No direct config changes:** Never edit configuration files, only recommend changes
5. **Human-in-the-loop:** All actions performed by the engineer

### Evidence Integrity

1. **Source attribution:** Always cite the source of evidence
2. **Timestamp awareness:** Note when evidence was collected
3. **Context preservation:** Include what is normal alongside what is abnormal
4. **No fabrication:** Never invent evidence, state "not available" if not collected
5. **Confidence honesty:** Never overstate confidence without sufficient evidence

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial Confidence & Evidence Engine specification |