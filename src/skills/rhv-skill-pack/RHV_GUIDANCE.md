# Red Hat Virtualization (RHV) — Guidance Reference

## Purpose

This document provides engineering guidance for Red Hat Virtualization operations, best practices, and decision frameworks.

## Foundation Reference

Operational best practices (capacity planning, performance tuning, maintenance, incident response) follow the shared [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md). RHV-specific guidance (Gluster tuning, OVS networking, VirtIO optimization, Hosted Engine HA) is included here without restating foundational theory.

---

## Operational Guidance

### Installation Guidance

1. **Pre-installation checklist:**
   - RHEL 8.x hosts with latest patches
   - DNS resolution for all hosts
   - NTP synchronized
   - Storage domains configured (Gluster/NFS/iSCSI)
   - Network segregation (management, storage, VM traffic)
   - Minimum 3 hosts for production (HA requires 3+)

2. **Installation order:**
   - Install RHEL on Manager host
   - Install RHV Manager: `engine-setup`
   - Add hosts to environment
   - Configure storage domains
   - Create clusters
   - Deploy VMs

3. **Post-installation validation:**
   - Verify all services running
   - Test VM deployment
   - Test live migration
   - Test HA failover
   - Verify monitoring and logging

### Patching Guidance

1. **Before patching:**
   - Create snapshots of critical VMs
   - Verify backups are current
   - Test patches in non-production environment
   - Schedule maintenance window

2. **Patching order:**
   - Migrate VMs off host
   - Put host in maintenance mode
   - Apply host packages
   - Reboot host if required
   - Verify host joins cluster
   - Repeat for each host
   - Apply Engine patches (may require Manager downtime)

3. **After patching:**
   - Verify all services running
   - Test VM operations
   - Check for errors in logs
   - Validate HA functionality

### Upgrade Guidance

1. **Upgrade path:** Follow documented upgrade path strictly (e.g., 4.4 → 4.5 → next)
2. **Backup first:** Complete backup of Engine and all critical VMs
3. **Test upgrade:** Upgrade non-production environment first
4. **Upgrade order:** Hosts first, then Engine
5. **Verify after:** Full regression test of all operations

---

## Best Practices

### Infrastructure Design

1. **Minimum 3 hosts** for HA environments
2. **EVC-like compatibility:** Keep CPU generations similar within clusters
3. **Dual NICs per host** for redundancy
4. **Separate storage network** from VM and management traffic
5. **Jumbo frames (MTU 9000)** for storage and vMotion networks

### Storage Design

1. **Gluster recommended** for RHV deployments (integrated, high availability)
2. **Minimum 3 bricks** for Gluster volume redundancy
3. **Monitor Gluster healing** — proactive healing prevents data loss
4. **Separate ISO domain** from data domain
5. **Configure export domain** for template management

### VM Design

1. **Standardize templates** — consistent base images
2. **Use VirtIO drivers** — best performance for both disk and network
3. **Enable SPICE console** — rich remote console access
4. **Right-size VMs** — monitor actual usage before scaling
5. **Implement snapshot policy** — auto-delete after 7 days max

### Security

1. **TLS everywhere** — all management and VM traffic encrypted
2. **RBAC implementation** — least privilege for all users
3. **Network isolation** — management, storage, and VM traffic separated
4. **Certificate management** — monitor and renew before expiry
5. **Audit logging** — enabled and reviewed regularly

---

## Performance Tuning

### Host Tuning

| Area | Recommendation | Impact |
|------|----------------|--------|
| **CPU** | Enable hardware virtualization extensions | Performance |
| **Memory** | Configure huge pages for memory-intensive VMs | Memory performance |
| **Storage** | Enable Jumbo frames for storage network | I/O latency |
| **I/O Scheduler** | Use `mq-deadline` or `bfq` for storage | Disk I/O |
| **NUMA** | Align VM NUMA topology with host | Memory latency |

### VM Tuning

| Area | Recommendation | Impact |
|------|----------------|--------|
| **CPU** | Match vCPU topology to workload | CPU scheduling |
| **Memory** | Disable ballooning for performance-critical VMs | Memory stability |
| **Disk** | Use VirtIO with writeback cache mode | Disk I/O |
| **Network** | Use VirtIO adapter type | Network performance |
| **GPU** | Use PCI passthrough for GPU workloads | GPU access |

### Storage Tuning

| Area | Recommendation | Impact |
|------|----------------|--------|
| **Gluster** | Enable `stripe` and `replica` volumes | Performance + redundancy |
| **NFS** | Use NFSv4 with directio for storage domain | I/O performance |
| **iSCSI** | Use multipath with ALUA | Path redundancy |
| **Cache** | Adjust block caching per workload | I/O latency |

---

## Capacity Planning

### CPU Capacity

- Target: < 70% average CPU utilization across cluster
- Peak: < 85% for at least 1 hour
- HA headroom: Reserve 1 host worth of capacity

### Memory Capacity

- Target: < 80% average memory utilization
- Peak: < 90% for sustained periods
- HA headroom: Reserve 1 host worth of capacity
- Monitor ballooning — keep under 1%

### Storage Capacity

- Target: < 75% utilization per storage domain
- Alert at: 70% utilization
- Critical at: 80% utilization
- Plan expansion at: 70% (quarterly review)

### VM Count

- Target: < 80% VM capacity per host
- Peak: < 90% (accounting for HA failover)
- HA headroom: Enough capacity for 1 host failure

---

## Decision Frameworks

### Platform Selection

| Requirement | Recommended Storage | Rationale |
|-------------|-------------------|-----------|
| Small deployment (< 10 VMs) | NFS | Simple, low cost |
| Medium deployment | Gluster | High availability, scalable |
| Large deployment | Gluster or iSCSI SAN | Performance, redundancy |
| High performance | NVMe-backed Gluster bricks | Low latency |

### Cluster Sizing

| VM Count | Host Count | Notes |
|----------|-----------|-------|
| 10-30 | 3 | Small production cluster |
| 30-100 | 5-7 | Medium production cluster |
| 100-500 | 10-20 | Large production cluster |
| 500+ | 20+ | Multiple clusters needed |

### HA Policy Selection

| Scenario | HA Policy | Notes |
|----------|-----------|-------|
| Business critical | Non-responsive + Down | Automatic restart on any failure |
| Development | Non-responsive only | Restart only on unresponsive host |
| Testing | Manual | Manual restart control |
| DR | Non-responsive + Down | Automatic restart with priority |

---

## Incident Response

### Severity Classification

| Severity | Description | Response Time |
|----------|-------------|---------------|
| **P1 - Critical** | Engine down, all hosts down, data loss risk | Immediate |
| **P2 - High** | Host down, storage domain down, HA failing | 30 minutes |
| **P3 - Medium** | VM performance degradation, migration failure | 2 hours |
| **P4 - Low** | Single VM issue, cosmetic problem | 24 hours |

### Escalation Matrix

| Severity | Level 1 | Level 2 | Level 3 |
|----------|---------|---------|---------|
| P1 | RHV Admin | RHV Architect | Red Hat Support |
| P2 | RHV Admin | RHV Architect | Red Hat Support |
| P3 | RHV Admin | — | — |
| P4 | RHV Admin | — | — |

---

## Documentation References

### Official Documentation

- [Red Hat Virtualization Administration Guide](https://access.redhat.com/documentation/en-us/red_hat_virtualization/)
- [Red Hat Virtualization Deployment Guide](https://access.redhat.com/documentation/en-us/red_hat_virtualization/)
- [Red Hat Virtualization Release Notes](https://access.redhat.com/documentation/en-us/red_hat_virtualization/)
- [Red Hat Virtualization Best Practices](https://access.redhat.com/documentation/en-us/red_hat_virtualization/)
- [Red Hat Virtualization Operations Guide](https://access.redhat.com/documentation/en-us/red_hat_virtualization/)

### Support Resources

- [Red Hat Customer Portal](https://access.redhat.com/)
- [Red Hat Knowledgebase](https://access.redhat.com/solutions)
- [Red Hat Virtualization Forum](https://access.redhat.com/forums/365)

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV guidance |