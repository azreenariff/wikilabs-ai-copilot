# SQL Server Engineering Guidance and Safety Rules

## Overview

This document defines the engineering guidance and safety rules for working with Microsoft SQL Server in enterprise environments. These rules ensure consistent, safe, and effective assistance.

## Safety Rules

### 1. Never Execute Commands Without Authorization

**Rule:** All SQL Server command execution requires explicit user authorization before proceeding.

- Provide advisory guidance with T-SQL examples only
- Never run commands against production systems without confirmation
- Always explain the impact of a command before providing it
- Include a risk assessment for each recommended action

### 2. Risk Assessment Required

**Rule:** Every recommendation must include a risk assessment.

**Risk levels:**
- **LOW** — Read-only operations, informational queries
- **MEDIUM** — Configuration changes, non-destructive modifications
- **HIGH** — Schema changes, data modifications, system-level changes
- **CRITICAL** — Production impact, data loss risk, service disruption

### 3. Verify Before Recommending Changes

**Rule:** Always verify current state before recommending changes.

- Check current configuration values
- Review current workload patterns
- Assess current health status
- Consider recent changes (patches, deployments)

### 4. Always Have Rollback Plan

**Rule:** Every recommendation must include a rollback strategy.

- Document the current state before changes
- Provide rollback commands where applicable
- Identify what to restore if changes cause problems
- Consider impact on backup and recovery

### 5. Production vs Development

**Rule:** Different rules apply for production vs development environments.

**Production rules:**
- Always schedule changes during maintenance windows
- Always have a rollback plan
- Always verify with testing before production deployment
- Always notify stakeholders of changes

**Development rules:**
- More flexible testing approaches
- Can use development databases for experimentation
- Still requires awareness of impact

## Configuration Safety

### Max Server Memory

**Guidance:**
- Always set `max server memory` on dedicated SQL Server instances
- Leave 4 GB minimum for OS
- On shared servers, leave more memory for other applications
- Monitor actual usage before finalizing settings

### TempDB Configuration

**Guidance:**
- Ensure at least one data file per logical CPU core
- All files should be equal size
- Use MB-based AUTO-GROWTH (not percentage)
- Place on high-performance storage

### Backup Strategy

**Guidance:**
- Never disable backups
- Always verify backups with restore tests
- Maintain backup retention policies
- Monitor backup job success/failure

## Security Safety

### Authentication

**Guidance:**
- Prefer Windows Authentication over SQL Server Authentication
- If using SQL Authentication, enforce strong passwords
- Enable password policies
- Disable unused logins

### Permissions

**Guidance:**
- Follow principle of least privilege
- Use database roles for permission management
- Regular permission reviews
- Audit elevated permissions

### Encryption

**Guidance:**
- Encrypt sensitive data at rest (TDE)
- Encrypt sensitive data in transit (TLS/SSL)
- Protect encryption keys and certificates
- Test decryption procedures before incident

## Change Management Safety

### Schema Changes

**Guidance:**
- Test schema changes in development first
- Consider online operations for production
- Monitor for plan cache invalidation
- Check for blocking during schema changes

### Index Changes

**Guidance:**
- Review index usage before adding indexes
- Consider impact on write operations
- Use Ola Hallengren maintenance solution
- Monitor after changes

### Statistics Changes

**Guidance:**
- Update statistics during maintenance windows
- Consider FULLSCAN for critical tables
- Monitor plan cache impact
- Verify improvement before committing

## Monitoring Safety

### DMV Queries

**Guidance:**
- DMV queries are read-only and safe
- Do not run DMV queries on production during peak hours
- Consider the overhead of frequent monitoring
- Use sampling for long-running monitoring

### Extended Events

**Guidance:**
- Extended Events have minimal overhead
- Use for targeted, short-duration troubleshooting
- Configure event filters to reduce overhead
- Store to file or ring buffer

## Version-Specific Safety

### SQL Server 2017

- Be aware of cross-platform limitations
- Python/R integration requires proper security configuration
- Graph database features have specific security considerations

### SQL Server 2019

- Smart memory grant improvements reduce plan regressions
- Accelerated Database Recovery affects recovery model
- Batch mode on rowstore changes query optimizer behavior

### SQL Server 2022

- Intelligent Query Processing changes plan selection
- Vectorized batch mode may affect query plans
- Enhanced security features may require configuration changes

## Escalation Rules

### When to Escalate

- **Data loss risk** — Immediate escalation to DBA
- **Service disruption** — Immediate escalation to operations team
- **Security breach** — Immediate escalation to security team
- **Unresolved performance issue** — Escalate after 30 minutes of troubleshooting
- **Hardware issues** — Escalate to infrastructure team

### Escalation Information

When escalating, provide:
- Problem description
- Impact assessment
- Troubleshooting steps taken
- Current state of affected systems
- Recommended next steps

## Compliance Safety

### Data Protection

**Guidance:**
- Identify PII and sensitive data
- Apply appropriate encryption and masking
- Follow GDPR/HIPAA requirements where applicable
- Audit access to sensitive data

### Backup Compliance

**Guidance:**
- Maintain required backup retention periods
- Test restore procedures regularly
- Document backup and recovery procedures
- Monitor backup job success rates

## Conclusion

These safety rules ensure that SQL Server operations are conducted responsibly and safely. Always follow these guidelines, adapt them to your organization's specific policies, and escalate when appropriate.