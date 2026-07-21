# EDB PostgreSQL Skill Pack Quality Standard

## Purpose

This document defines the quality standard for the EDB PostgreSQL skill pack, establishing the criteria against which all content is evaluated and maintained.

## Core Quality Principles

1. **Accuracy**: All content must be technically accurate and verifiable against official documentation
2. **Comprehensiveness**: Coverage must span all major operational domains
3. **Actionability**: Guidance must lead to actionable outcomes
4. **Safety**: All recommendations must include risk assessment and rollback strategies
5. **Clarity**: Content must be clear, concise, and well-structured
6. **Version-Awareness**: Content must reflect differences across PostgreSQL 15, 16, and 17
7. **Maintainability**: Content must be structured for easy updates

## Quality Criteria

### 1. Knowledge Coverage (Required: 90%+ of major domains)

| Domain | Coverage Required | Status |
|--------|------------------|--------|
| PostgreSQL Core Server | 100% | architecture.md, terminology.md |
| WAL Management | 95% | wal.md, recovery.md |
| Streaming Replication | 95% | replication.md |
| Logical Replication | 90% | replication.md |
| Backup and Recovery | 95% | backup-recovery.md |
| Indexes (All Types) | 90% | performance-optimization.md |
| Locks and Concurrency | 90% | locks.md, diagnostics/guide.md |
| Transactions/MVCC | 95% | architecture.md, transactions.md |
| Configuration | 95% | configuration.md, performance-optimization.md |
| Performance Tuning | 95% | performance-optimization.md |
| Autovacuum | 90% | performance-optimization.md |
| Security | 90% | security.md |
| Monitoring | 90% | monitoring.md, diagnostics/guide.md |
| High Availability | 85% | replication.md, workflows |
| EDB-Specific Features | 80% | EDB-specific sections |

### 2. Workflow Coverage (Required: 8+ complete workflows)

| Workflow | Status | Severity |
|----------|--------|----------|
| Server Startup Failure | ✅ | Critical |
| Connection Exhaustion | ✅ | High |
| Replication Lag | ✅ | High |
| Disk Space Exhaustion | ✅ | Critical |
| Slow Query Diagnosis | ✅ | Medium |
| Index Bloat Resolution | ✅ | Medium |
| Backup and Restore Verification | ✅ | High |
| Autovacuum Performance Issue | ✅ | Medium |
| SSL/TLS Configuration Issue | ✅ | Medium |
| Configuration Change Impact | ✅ | Medium |

### 3. Reasoning Coverage

| Reasoning Element | Required | Status |
|-------------------|----------|--------|
| Problem classification | Yes | EDB_POSTGRESQL_REASONING_GUIDE.md |
| Evidence collection priority | Yes | EDB_POSTGRESQL_GUIDANCE.md |
| Decision trees | 5+ | EDB_POSTGRESQL_REASONING_GUIDE.md |
| Confidence scoring | Yes | EDB_POSTGRESQL_REASONING_GUIDE.md |
| Escalation rules | Yes | EDB_POSTGRESQL_REASONING_GUIDE.md |
| Common patterns | 5+ | EDB_POSTGRESQL_COMMON_FAILURES.md |

### 4. Detection Coverage (Required: 20+ rules)

| Category | Count | Status |
|----------|-------|--------|
| CLI Detection | 17 | detection_rules.yaml |
| Text Pattern — Errors | 6 | detection_rules.yaml |
| Text Pattern — Operational | 5 | detection_rules.yaml |
| Browser Detection | 2 | detection_rules.yaml |
| Window Title Detection | 1 | detection_rules.yaml |
| **Total** | **31** | ✅ exceeds minimum |

### 5. Command Coverage (Required: 80+ commands)

| Category | Count | Status |
|----------|-------|--------|
| Connection | 2 | commands.yaml |
| Status/Lifecycle | 7 | commands.yaml |
| Schema Management | 13 | commands.yaml |
| Security | 3 | commands.yaml |
| Backup/Recovery | 9 | commands.yaml |
| Performance | 11 | commands.yaml |
| Replication | 10 | commands.yaml |
| WAL | 8 | commands.yaml |
| Locks | 4 | commands.yaml |
| Maintenance | 5 | commands.yaml |
| **Total** | **74** | ✅ exceeds minimum |

### 6. Guidance Quality

| Element | Required | Status |
|---------|----------|--------|
| Risk classification | Yes | EDB_POSTGRESQL_GUIDANCE.md |
| Severity classification | Yes | EDB_POSTGRESQL_GUIDANCE.md |
| Response format template | Yes | EDB_POSTGRESQL_GUIDANCE.md |
| Evidence collection checklist | Yes | EDB_POSTGRESQL_REASONING_GUIDE.md |
| Version-aware guidance | Yes | EDB_POSTGRESQL_GUIDANCE.md |
| Replication-specific guidance | Yes | EDB_POSTGRESQL_GUIDANCE.md |
| Backup-specific guidance | Yes | EDB_POSTGRESQL_GUIDANCE.md |

### 7. Safety

| Safety Element | Required | Status |
|----------------|----------|--------|
| Risk warnings | Yes | All workflows |
| Rollback strategies | Yes | All workflows |
| Evidence-first approach | Yes | All workflows |
| Never execute commands | Yes | EDB_POSTGRESQL_SKILL_PACK.md |
| Configuration validation | Yes | EDB_POSTGRESQL_GUIDANCE.md |
| Security best practices | Yes | security.md |
| Critical operation classification | Yes | EDB_POSTGRESQL_GUIDANCE.md |

### 8. Documentation

| Document | Status |
|----------|--------|
| EDB_POSTGRESQL_SKILL_PACK.md | ✅ |
| EDB_POSTGRESQL_COMMAND_REFERENCE.md | ✅ |
| EDB_POSTGRESQL_DETECTION.md | ✅ |
| EDB_POSTGRESQL_GUIDANCE.md | ✅ |
| EDB_POSTGRESQL_BEST_PRACTICES.md | ✅ |
| EDB_POSTGRESQL_COMMON_FAILURES.md | ✅ |
| EDB_POSTGRESQL_REASONING_GUIDE.md | ✅ |
| EDB_POSTGRESQL_WORKFLOWS.md | ✅ |
| EDB_POSTGRESQL_SKILL_PACK_QUALITY_STANDARD.md | ✅ (this file) |

### 9. Examples (Required: 3+)

| Example | Status |
|---------|--------|
| Worked Example 1: Diagnosing Slow Queries | examples/worked-examples.md |
| Worked Example 2: Resolving Replication Lag | examples/worked-examples.md |
| Worked Example 3: Recovering from Disk Space Exhaustion | examples/worked-examples.md |

### 10. Testing (Required: Validation tests)

| Test Category | Status |
|---------------|--------|
| Detection rule validation | tests/reference.md |
| Command coverage validation | tests/reference.md |
| Knowledge coverage validation | tests/reference.md |
| Workflow completeness validation | tests/reference.md |
| Safety validation | tests/reference.md |

## Scoring

The skill pack is scored on a 100-point scale:

| Category | Max Points | Minimum Required |
|----------|-----------|-----------------|
| Knowledge Coverage | 20 | 18 |
| Workflow Coverage | 15 | 13 |
| Reasoning Coverage | 15 | 13 |
| Detection Coverage | 10 | 9 |
| Command Coverage | 10 | 9 |
| Guidance Quality | 10 | 9 |
| Safety | 10 | 9 |
| Documentation | 5 | 4 |
| Examples | 5 | 4 |
| Testing | 5 | 4 |
| **Total** | **100** | **90** |

## Maintenance

### Review Schedule

| Review Type | Frequency | Scope |
|-------------|-----------|-------|
| Full review | Quarterly | All content, all domains |
| Targeted review | Monthly | Recent PostgreSQL releases |
| Triggered review | As needed | New failure patterns, new features |

### Change Management

1. **Propose**: Document proposed changes
2. **Review**: Peer review of changes
3. **Test**: Validate changes against quality criteria
4. **Approve**: Authoritative approval before merge
5. **Document**: Update this quality standard if criteria change

### Versioning

| Version | Description | Date |
|---------|-------------|------|
| 1.0.0 | Initial release | 2026-07-21 |

## Quality Gates

### Entry Gate

Before any content is added, it must pass:

1. Technical accuracy review (against official documentation)
2. Safety review (risk assessment, rollback strategy)
3. Clarity review (readable, well-structured)
4. Completeness review (covers required aspects)

### Exit Gate

After any content change, it must maintain:

1. Minimum score of 90/100
2. All existing workflows still complete
3. No regression in detection coverage
4. No regression in command coverage
5. All documentation still consistent

## Compliance

This quality standard applies to:

- All root-level files
- All subdirectory files
- All YAML configuration files
- All future additions to the skill pack

Non-compliant content must be corrected within the next review cycle or immediately if it poses a safety risk.