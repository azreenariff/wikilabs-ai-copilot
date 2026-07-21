# Operations Engineering Foundation

## Purpose

This document provides the shared engineering foundation for monitoring, automation, and database operations skill packs. Nagios XI, Nagios Log Server, Checkmk, Ansible, MySQL, EDB PostgreSQL, and Microsoft SQL Server skill packs reference this foundation rather than duplicating foundational concepts.

## Architecture Decision Record

**ADR-001:** Operations concepts are shared across technologies. Foundation knowledge lives in a single location to avoid duplication and ensure consistency across monitoring, automation, and database skill packs.

---

## Observability & Monitoring Concepts

### What is Observability?

Observability is the ability to understand the internal state of a system by examining its external outputs — metrics, logs, traces, and events. In enterprise infrastructure, it means answering: Is the system working? Why is it slow? What broke?

### Golden Signals

Four key signals for any operational system:

| Signal | Definition | Formula/Measurement |
|--------|------------|---------------------|
| **Latency** | Time to process a request | Request processing time, excluding latency for dead requests |
| **Traffic** | Demand placed on the system | Requests per second, concurrent connections, throughput |
| **Errors** | Rate of failed requests | Percentage of requests that return non-200 status |
| **Saturation** | System resource pressure | CPU, memory, disk I/O, network utilization |

### Metrics

Quantitative measurements of system behavior over time.

| Attribute | Description |
|-----------|-------------|
| **Counter** | Monotonically increasing value (e.g., requests served, errors) |
| **Gauge** | Value that can increase or decrease (e.g., CPU usage, memory) |
| **Histogram** | Distribution of values over time (e.g., request latency percentiles) |
| **Summary** | Pre-computed quantiles (e.g., p95, p99 latency) |

#### Metric Collection

| Method | Description | Use Case |
|--------|-------------|----------|
| **Polling** | Active collection at intervals | Nagios service checks, SNMP polling |
| **Push** | Agent pushes metrics to collector | Checkmk agent, Prometheus pushgateway |
| **Streaming** | Real-time data flow | Kafka, log streaming |
| **Sampling** | Periodic snapshots | Profiling, trace sampling |

### Logs

Immutable records of events that occurred in a system.

| Type | Description |
|------|-------------|
| **Application Logs** | Messages from application code (INFO, WARN, ERROR, DEBUG) |
| **System Logs** | OS-level events (syslog, Windows Event Log) |
| **Access Logs** | HTTP/API request/response records |
| **Audit Logs** | Security-relevant actions (authentication, authorization) |
| **Error Logs** | Failure-specific records with stack traces |

#### Log Lifecycle

```
Generate → Collect → Parse → Index → Store → Search → Alert → Archive
   ↑                                                        ↓
   └───────────── Retention & Cleanup ───────────────────────┘
```

### Traces

End-to-end request journey through a distributed system.

- **Span:** Single operation within a trace
- **Trace:** Complete request journey across services
- **Context Propagation:** Trace ID passed between services
- **Sampling Rate:** Fraction of traces recorded (e.g., 10%)

### Events

Discrete occurrences that represent a state change.

| Event Type | Description |
|-----------|-------------|
| **State Change** | Service stopped, host rebooted, config changed |
| **Threshold Breach** | Metric exceeded warning or critical threshold |
| **Deployment** | Application version updated, infrastructure modified |
| **Security** | Login success/failure, permission change, firewall rule change |

### Alerts and Incidents

| Concept | Definition |
|---------|------------|
| **Threshold** | Numerical boundary that triggers an alert (e.g., CPU > 90%) |
| **Warning** | Early indicator of potential issue |
| **Critical** | Active issue requiring intervention |
| **Flapping** | Rapid state changes between OK/WARNING/CRITICAL |
| **Alert Storm** | Multiple alerts triggered simultaneously from a single root cause |
| **Incident** | Service-impacting event requiring investigation and resolution |
| **On-Call** | Engineer responsible for responding to active incidents |

#### Alert Best Practices

1. **Actionable thresholds:** Alert only when human intervention is needed
2. **Alert deduplication:** Suppress duplicate alerts from correlated sources
3. **Alert routing:** Route alerts to the right team/person based on severity
4. **Silence windows:** Suppress alerts during planned maintenance
5. **Escalation paths:** Escalate if alert is not acknowledged within SLA

---

## Automation Concepts

### What is Infrastructure Automation?

Automation uses code and scripts to manage infrastructure consistently, reproducibly, and at scale. It eliminates manual configuration, reduces errors, and enables rapid recovery.

### Core Principles

| Principle | Description |
|-----------|-------------|
| **Idempotency** | Running the same operation multiple times produces the same result |
| **Declarative** | Define desired state, not steps to get there |
| **Imperative** | Define specific steps to achieve a state |
| **Drift Detection** | Compare actual state vs desired state |
| **Rollback** | Revert to previous known-good state |

### Configuration Drift

When a system diverges from its intended configuration over time.

| Cause | Detection | Remediation |
|-------|-----------|-------------|
| Manual changes | Configuration audits, diff tools | Re-apply automation, enforce policies |
| Patch management | Version tracking, patch level checks | Run update playbooks |
| Manual overrides | Change tracking, audit logs | Document exceptions, automate exceptions |
| Application changes | Binary/hash comparison | Update automation to match new version |

### Playbooks and Roles

| Concept | Description |
|---------|-------------|
| **Playbook** | Ordered list of tasks to execute against hosts |
| **Task** | Single operation (install package, start service) |
| **Role** | Reusable collection of tasks, variables, handlers |
| **Module** | Reusable unit of work (package, service, file) |
| **Handler** | Task triggered by other tasks (e.g., restart service) |
| **Variable** | Data injected into playbooks (host-specific, group-specific) |
| **Template** | File with placeholders substituted at runtime |

### Dry Run / Check Mode

Execute operations without making changes to verify idempotency and expected behavior.

- **Check mode:** Parse resources, evaluate conditions, report what would change
- **Diff mode:** Show exact file/system differences
- **Limit scope:** Target specific hosts, tags, or roles
- **Review output:** Verify changes match expected state before applying

### Inventory Management

| Type | Description |
|------|-------------|
| **Static Inventory** | Fixed host list (INI, YAML, JSON files) |
| **Dynamic Inventory** | Host list generated from cloud/API sources |
| **Groups** | Logical grouping of hosts for targeted operations |
| **Variables** | Per-host or per-group configuration overrides |

---

## Database Operations Concepts

### Database Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                   Application Layer                      │
├──────────────────────────────────────────────────────────┤
│                Connection Pool (e.g., PgBouncer)         │
├──────────────────────────────────────────────────────────┤
│                  Database Engine                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │  Buffer │  │  WAL/   │  │  Query  │  │ Lock    │  │
│  │  Pool   │  │  Tx Log │  │  Planner│  │ Manager │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │
├──────────────────────────────────────────────────────────┤
│                    Storage Layer                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │  Index  │  │   Data  │  │   Temp  │  │   Audit │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │
└──────────────────────────────────────────────────────────┘
```

### Connection Pooling

| Aspect | Description |
|--------|-------------|
| **Purpose** | Reuse database connections to reduce overhead |
| **Problem** | Creating a new connection per request is expensive |
| **Pool Size** | Maximum concurrent connections to database |
| **Connection Leak** | Connections not returned to pool after use |
| **Exhaustion** | All pool connections in use, new requests blocked |
| **Best Practice** | Size pool = (CPU cores × 2) + disk spindles |

### Query Performance

| Metric | Description | Impact |
|--------|-------------|--------|
| **Query Latency** | Time to execute a query | Direct user-perceived performance |
| **Slow Queries** | Queries exceeding threshold duration | Indicates missing index, bad plan, or lock contention |
| **Execution Plan** | Steps database takes to execute query | Reveals full table scans, missing indexes |
| **Index Usage** | Whether indexes are being used | Missing indexes cause full table scans |
| **Lock Contention** | Queries waiting for locks | Blocking chains, deadlocks |
| **Temp Tables** | On-disk temporary table usage | Excessive I/O, memory pressure |

### Replication

| Concept | Description |
|---------|-------------|
| **Master-Slave** | Write on master, read on slaves |
| **Master-Master** | Write on both nodes (risk of conflicts) |
| **Group Replication** | Multi-master with consensus (MySQL GTID-based) |
| **Streaming Replication** | WAL/transaction log streaming to replicas |
| **Logical Replication** | Row-level changes (more flexible, lower throughput) |
| **Replication Lag** | Time between write on master and apply on replica |
| **Failover** | Promote replica to master when master fails |

### WAL / Transaction Logs

| Concept | Description |
|---------|-------------|
| **WAL (Write-Ahead Log)** | Records all changes before writing to data files |
| **Purpose** | Crash recovery, point-in-time recovery, replication |
| **WAL Size** | Can grow large with high write activity |
| **Full WAL** | Disk full when WAL is not archived/cleaned |
| **Cleanup** | Archive, backup, and remove completed WAL segments |
| **Auto-Vacuum** | Background process to reclaim dead tuple space |

### Backup and Restore

| Strategy | Description |
|----------|-------------|
| **Full Backup** | Complete database copy |
| **Incremental** | Changes since last backup |
| **Differential** | Changes since last full backup |
| **Point-in-Time** | Restore to specific moment using WAL replay |
| **Hot Backup** | Backup while database is running |
| **Cold Backup** | Backup while database is stopped |
| **Validation** | Test restore regularly to confirm backup integrity |

### Storage and Memory Management

| Resource | Monitoring | Best Practice |
|----------|------------|---------------|
| **Buffer Pool** | Hit ratio, dirty pages, cache usage | Size to fit working set, monitor eviction rate |
| **Disk I/O** | Latency, IOPS, throughput | Separate data, WAL, temp on different disks |
| **Memory** | Swap usage, OOM events | Reserve OS memory, leave buffer pool + overhead |
| **Disk Space** | Usage percentage, growth rate | Alert at 70%, critical at 80%, monitor WAL/archive |
| **Inode Usage** | File count per filesystem | Alert on inode exhaustion even if space available |

---

## Monitoring Skill Pack Architecture

### Common Detection Patterns

| Pattern | Technique | Example |
|---------|-----------|---------|
| **Browser URL** | Detect via URL patterns | `nagios`, `checkmk`, `grafana` |
| **Window Title** | Detect via window captions | "Nagios XI", "Checkmk Setup" |
| **Terminal CLI** | Detect via command names | `check_nrpe`, `cmk`, `ansible`, `mysql`, `psql` |
| **Process Names** | Detect via running processes | `nagios`, `checkmk`, `httpd`, `mysqld` |
| **Log Patterns** | Detect via log content | `ERROR`, `WARNING`, `CRITICAL` patterns |
| **Configuration Files** | Detect via file patterns | `nagios.conf`, `check_mk`, `ansible.cfg` |

### Common Skill Pack Structure

```
skill-pack/
├── manifest.yaml        # Skill pack metadata
├── concepts/
│   ├── overview.md      # Architecture and key components
│   └── terminology.md   # Glossary
├── detection/
│   └── reference.md     # Detection rules and patterns
├── diagnostics/
│   └── guide.md         # Diagnostic procedures
├── common-failures/
│   └── reference.md     # Known failure modes
├── examples/
│   └── worked-examples.md # Real scenarios
├── knowledge/
│   └── key-topics.md    # Detailed domain knowledge
├── references/
│   └── reference.md     # External references
├── reasoning/
│   └── reference.md     # Diagnostic reasoning
├── tests/
│   └── reference.md     # Validation tests
└── context/
    └── interpretation.md # Context interpretation
```

---

## Shared Knowledge Rules

### When to Reference Foundation

Use this foundation for:
- Monitoring concepts (metrics, logs, alerts)
- Automation concepts (idempotency, playbooks, roles)
- Database concepts (connection pooling, replication, WAL)
- Performance fundamentals (latency, throughput, I/O)
- Capacity planning and operational best practices

### When to Include Technology-Specific Content

Include technology-specific details in skill packs:
- Vendor-specific terminology and commands
- Configuration file locations and formats
- Vendor-specific features and options
- Vendor-specific failure modes and remediation
- Vendor-specific best practices

### Example Reference Pattern

```markdown
## Database Architecture

The database architecture follows the [Operations Engineering Foundation](OPERATIONS_FOUNDATION.md#database-operations-concepts).

### MySQL-Specific Details

In MySQL, the buffer pool is configured via `innodb_buffer_pool_size` in `my.cnf`...
```

---

## Operational Best Practices

### Change Management

1. **Document all changes:** Before, during, after
2. **Test in non-production:** Validate changes before production
3. **Schedule maintenance windows:** Minimize business impact
4. **Rollback plan:** Always have a tested rollback procedure
5. **Automate where possible:** Use playbooks for repeatable changes

### Monitoring Best Practices

1. **Actionable alerts:** Only alert when human intervention is needed
2. **Golden signals:** Monitor latency, traffic, errors, saturation
3. **Dashboards:** Create operational dashboards for key services
4. **Runbooks:** Document procedures for common alerts
5. **Regular reviews:** Quarterly review alert effectiveness

### Automation Best Practices

1. **Idempotency first:** Ensure all playbooks are idempotent
2. **Check mode:** Test playbooks with `--check --diff` before applying
3. **Version control:** Store all playbooks and roles in Git
4. **Testing:** Run unit and integration tests on playbooks
5. **Documentation:** Document inventory structure, variables, and roles

### Database Best Practices

1. **Monitor before scaling:** Measure actual usage before changing settings
2. **Backup validation:** Test restores regularly
3. **Query optimization:** Monitor slow queries and add indexes
4. **Connection management:** Monitor pool utilization
5. **Replication health:** Monitor lag and failover readiness

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial operations engineering foundation |