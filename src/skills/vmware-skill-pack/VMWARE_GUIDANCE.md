# VMware vSphere — Guidance Reference

## Purpose

This document provides engineering guidance for VMware vSphere operations, best practices, and decision frameworks.

## Foundation Reference

Operational best practices (capacity planning, performance tuning, maintenance, incident response) follow the shared [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md). VMware-specific guidance (VCSA certificate management, VMware Tools, vmxnet3, VMFS tuning) is included here without restating foundational theory.

---

## Operational Guidance

### Installation Guidance

1. **Pre-installation checklist:**
   - RHEL/CentOS/Windows server for VCSA with latest patches
   - DNS resolution for all hosts
   - NTP synchronized
   - Storage available (datastore for VCSA)
   - Network segregation (management, vMotion, storage, VM traffic)
   - Minimum 3 ESXi hosts for production (HA requires 3+)

2. **Installation order:**
   - Install VCSA via OVA/OVF
   - Add ESXi hosts to vCenter
   - Create clusters
   - Configure storage (VMFS/vSAN)
   - Configure networking (vSS/vDS)
   - Enable HA/DRS
   - Deploy VMs

3. **Post-installation validation:**
   - Verify all services running (vpxd, vpxd-svcs, vsphere-client)
   - Test VM deployment
   - Test vMotion
   - Test HA failover
   - Verify monitoring and logging

### Patching Guidance

1. **Before patching:**
   - Snapshot all critical VMs
   - Verify backups are current
   - Test patches in non-production environment
   - Schedule maintenance window

2. **Patching order:**
   - Enable DRS migration for maintenance mode
   - Put host in maintenance mode
   - Apply host patches via vSphere Client or `esxcli`
   - Reboot host if required
   - Verify host joins cluster
   - Repeat for each host
   - Apply VCSA patches (may require Manager downtime)

3. **After patching:**
   - Verify all services running
   - Test VM operations
   - Check for errors in logs
   - Validate HA functionality

### Upgrade Guidance

1. **Upgrade path:** Follow documented upgrade path strictly (e.g., 7.0 → 8.0)
2. **Backup first:** Complete backup of VCSA and all critical VMs
3. **Test upgrade:** Upgrade non-production environment first
4. **Upgrade order:** Hosts first, then VCSA
5. **Verify after:** Full regression test of all operations

---

## Best Practices

### Infrastructure Design

1. **Minimum 3 hosts** for HA environments
2. **EVC mode** — enable for cross-generation CPU compatibility
3. **Dual NICs per host** for redundancy
4. **Separate vMotion network** — dedicated vmkernel adapter
5. **Jumbo frames (MTU 9000)** for storage and vMotion networks
6. **vDS over vSS** — distributed switches for centralized management

### Storage Design

1. **VMFS-6** for production workloads
2. **Multipathing** — configure ALUA for SAN storage
3. **Separate datastores** for VMs and templates
4. **Monitor capacity** — alert at 70%, critical at 80%
5. **vSAN** — for hyperconverged deployments (all-flash recommended)
6. **Thin provisioning** — with monitoring (space can fill unexpectedly)

### VM Design

1. **Standardize templates** — consistent base images
2. **Install VMware Tools** — best performance for both disk and network
3. **Right-size VMs** — monitor actual usage before scaling
4. **Implement snapshot policy** — auto-delete after 7 days max
5. **Enable hardware version** latest for new VMs

### Security

1. **TLS everywhere** — all vCenter and host communication encrypted
2. **RBAC implementation** — least privilege for all users
3. **Network isolation** — separate management, storage, and VM traffic
4. **Certificate management** — monitor and renew before expiry
5. **Audit logging** — enabled and reviewed regularly
6. **Password policies** — enforced via vCenter/LDAP

---

## Performance Tuning

### Host Tuning

| Area | Recommendation | Impact |
|------|----------------|--------|
| **CPU** | Enable hardware virtualization extensions | Performance |
| **Memory** | Configure memory reservation for critical VMs | Memory stability |
| **Storage** | Enable Jumbo frames for storage network | I/O latency |
| **I/O Scheduler** | Use `vmw_oss` or `none` for VMFS | Disk I/O |
| **NUMA** | Align VM NUMA topology with host | Memory latency |

### VM Tuning

| Area | Recommendation | Impact |
|------|----------------|--------|
| **CPU** | Match vCPU to actual workload needs | CPU scheduling |
| **Memory** | Set reservation for database VMs | Memory stability |
| **Disk** | Use thin provisioning with monitoring | Storage efficiency |
| **Network** | Use vmxnet3 adapter type | Network performance |
| **Storage IO** | Set I/O limits for non-critical VMs | I/O fairness |

### Storage Tuning

| Area | Recommendation | Impact |
|------|----------------|--------|
| **VMFS** | Use VMFS-6 for modern features | Performance |
| **NFS** | Use NFSv4.1 with multipathing | I/O redundancy |
| **iSCSI** | Use software iSCSI with multiple paths | Path redundancy |
| **vSAN** | Enable SSD cache tier | Cache performance |

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
- Monitor swapping — eliminate completely

### Storage Capacity

- Target: < 75% utilization per datastore
- Alert at: 70% utilization
- Critical at: 80% utilization
- Plan expansion at: 70% (quarterly review)
- Account for snapshot growth

### VM Count

- Target: < 80% VM capacity per host
- Peak: < 90% (accounting for HA failover)
- HA headroom: Enough capacity for 1 host failure

---

## Decision Frameworks

### Storage Selection

| Requirement | Recommended Storage | Rationale |
|-------------|-------------------|-----------|
| Small deployment (< 10 VMs) | Direct-attached or simple SAN | Simple, low cost |
| Medium deployment | FC/iSCSI SAN | High performance, redundancy |
| Large deployment | vSAN or FC SAN | Scalability, automation |
| High performance | vSAN all-flash or NVMe | Lowest latency |

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
| Business critical | Host failure + Host isolation | Automatic restart on any failure |
| Development | Host failure only | Restart only on failed host |
| Testing | Manual | Manual restart control |
| DR | Host failure + isolation | Automatic restart with priority |

### DRS Settings

| Environment | Automation | Sensitivity |
|-------------|-----------|-------------|
| Production | Fully Automated | Medium-High |
| Development | Fully Automated | Low-Medium |
| Testing | Manual | N/A |
| DR | Fully Automated | Medium |

---

## Incident Response

### Severity Classification

| Severity | Description | Response Time |
|----------|-------------|---------------|
| **P1 - Critical** | vCenter down, all hosts down, data loss risk | Immediate |
| **P2 - High** | Host down, datastore down, HA failing | 30 minutes |
| **P3 - Medium** | VM performance degradation, migration failure | 2 hours |
| **P4 - Low** | Single VM issue, cosmetic problem | 24 hours |

### Escalation Matrix

| Severity | Level 1 | Level 2 | Level 3 |
|----------|---------|---------|---------|
| P1 | vSphere Admin | vSphere Architect | VMware Support |
| P2 | vSphere Admin | vSphere Architect | VMware Support |
| P3 | vSphere Admin | — | — |
| P4 | vSphere Admin | — | — |

---

## Documentation References

### Official Documentation

- [VMware vSphere Documentation](https://docs.vmware.com/en/VMware-vSphere/)
- [VMware KB](https://kb.vmware.com/)
- [esxcli Documentation](https://docs.vmware.com/en/VMware-vSphere/8.0/com.vmware.vsphere.esx.cli.doc/)
- [PowerCLI Documentation](https://developer.vmware.com/apis/powercli)
- [VCSA Administration Guide](https://docs.vmware.com/en/VMware-vSphere/8.0/com.vmware.vsphere.install.doc/)

### Support Resources

- [VMware Support Portal](https://www.vmware.com/support/)
- [VMware Customer Connect](https://customerconnect.vmware.com/)
- [VMware Communities](https://communities.vmware.com/)

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Consolidated VMware vSphere guidance |