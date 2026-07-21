# Virtualization Engineering Foundation

## Purpose

This document provides the shared engineering foundation for all virtualization skill packs. VMware vSphere and Red Hat Virtualization (RHV) skill packs reference this foundation rather than duplicating foundational concepts.

## Architecture Decision Record

**ADR-001:** Virtualization concepts are shared across technologies. Foundation knowledge lives in a single location to avoid duplication and ensure consistency across VMware and RHV skill packs.

---

## Virtualization Concepts

### What is Virtualization?

Virtualization is the abstraction of physical computing resources to create virtual (logical) representations. It enables:

- **Resource consolidation:** Multiple workloads on fewer physical hosts
- **Isolation:** Workloads separated from each other
- **Abstraction:** Software decoupled from specific hardware
- **Flexibility:** Resources scaled, moved, or replicated dynamically

### Hypervisor Types

| Type | Description | Examples |
|------|-------------|----------|
| **Type 1 (Bare Metal)** | Runs directly on hardware, no host OS | VMware ESXi, Red Hat KVM/RHV, Microsoft Hyper-V, Xen |
| **Type 2 (Hosted)** | Runs on top of a host operating system | VMware Workstation, Oracle VirtualBox, Parallels |

Enterprise environments exclusively use Type 1 hypervisors for performance, security, and management reasons.

### Key Virtualization Terminology

| Term | Definition |
|------|------------|
| **Hypervisor** | Software that creates and runs virtual machines |
| **Virtual Machine (VM)** | A software-defined computer with virtualized hardware |
| **Host** | The physical machine running the hypervisor |
| **Guest** | The operating system running inside a VM |
| **Cluster** | A group of hosts managed as a single unit |
| **Datastore** | Storage repository accessible by hosts for VM files |
| **Template** | A readable copy of a VM used to deploy new VMs |
| **Snapshot** | A point-in-time copy of a VM's state |
| **vMotion / Live Migration** | Moving a running VM between hosts with zero downtime |
| **Storage vMotion / Live Storage Migration** | Moving a VM's storage without downtime |
| **HA (High Availability)** | Automatic VM restart on another host after host failure |
| **DRS (Distributed Resource Scheduler)** | Automated load balancing across cluster hosts |
| **Resource Pool** | Logical grouping of CPU and memory resources |
| **NUMA** | Non-Uniform Memory Access architecture — CPU locality of memory |
| **Ballooning** | Memory reclamation technique when host is under pressure |
| **CPU Ready** | Time a vCPU waits for physical CPU time (contention metric) |

---

## Compute Architecture

### Host Configuration

Physical hosts provide:

- **CPU:** Multi-core processors, typically x86_64
- **RAM:** ECC memory for reliability
- **Storage:** Local disks (HDD/SSD/NVMe) and/or external SAN/NAS
- **Network:** Multiple NICs for redundancy and traffic separation

### Host Design Best Practices

1. **Dual power supplies** for redundancy
2. **Dual power feeds** from separate PDUs
3. **Multiple NICs** with teaming/multipathing
4. **RAID-configured local storage** for hypervisor OS
5. **Redundant switch connections** (LACP or active-active)
6. **Out-of-band management** (iDRAC, iLO, XCC)

### Cluster Fundamentals

A cluster is a group of hosts managed together:

- **Shared nothing architecture:** Each host manages its own storage
- **Centralized management:** Single point for monitoring and control
- **Resource pooling:** Aggregate CPU, memory, and storage capacity
- **Policy enforcement:** HA, DRS, and other cluster-wide policies

### Cluster Design Principles

1. **Similar hardware:** Same CPU generation within a cluster for compatibility
2. **Capacity headroom:** 20-30% free capacity for HA and maintenance
3. **EVC mode:** Enable Enhanced vMotion Compatibility for cross-generation migration
4. **Admission control:** Reserve resources for failover scenarios
5. **Avoid single points of failure:** Spread components across racks and switches

---

## Virtual Machines

### VM Lifecycle

```
Powered Off → Power On → Running → (Suspended / Migrated / Snapshot)
     ↑                                                      ↓
     └───────────── Reset / Revert / Shutdown ──────────────┘
```

### VM Sizing Principles

1. **Start small:** Right-size based on actual workload requirements
2. **Monitor before scaling:** Measure actual CPU, memory, disk I/O before resizing
3. **Avoid over-provisioning:** Wastes resources and masks application issues
4. **Plan for growth:** Leave headroom for growth and peak loads
5. **Match to SLA:** Higher-availability workloads need more resources

### VM Template Strategy

1. **Standardize base images:** Consistent OS, patches, and baseline software
2. **Golden templates:** Maintain a current template with latest security patches
3. **Customization specs:** Automate hostname, IP, and domain join on deploy
4. **Regular updates:** Update templates monthly or after major patches

### Snapshot Best Practices

1. **Not a backup:** Snapshots do not replace backup solutions
2. **Retention policy:** Maximum 7 days, auto-delete after testing or changes
3. **Chain length:** Limit to 3 snapshots in chain — longer chains degrade performance
4. **Consolidation:** Consolidate regularly to merge delta files
5. **Monitor growth:** Alert when snapshot files grow beyond threshold

---

## Storage Architecture

### Storage Types

| Type | Description | Performance | Use Case |
|------|-------------|-------------|----------|
| **Direct-Attached** | Local disks on host | High | Hypervisor OS, local VMs |
| **SAN (FC/iSCSI)** | Shared storage via fiber or IP | High | Production VMs, shared storage |
| **NFS** | Network File System | Moderate | Backup targets, secondary storage |
| **vSAN** | Distributed software-defined storage | High (with SSD) | All-flash or hybrid hyperconverged |
| **Network-Attached** | Dedicated file storage appliances | Moderate | Templates, ISO libraries |

### VMFS and Storage Formats

- **VMFS-5 / VMFS-6:** VMware's clustered file system for VM storage
- **Raw Device Mapping (RDM):** Direct LUN access for specific use cases
- **Thin Provisioning:** Allocates space on demand (saves space, requires monitoring)
- **Thick Provisioning:** Pre-allocates all space (better performance, wastes space)

### Storage Performance Metrics

| Metric | Threshold | Impact |
|--------|-----------|--------|
| **Latency** | < 20ms average | Higher latency slows all I/O |
| **IOPS** | Sufficient for workload | Low IOPS limits throughput |
| **Throughput** | Sufficient for workload | Low throughput limits bandwidth |
| **Capacity** | < 80% used | Above 80% risks performance degradation |

### Storage Design Best Practices

1. **Separate storage networks:** Dedicated VLANs for storage traffic
2. **Multipathing:** Configure multiple paths for redundancy (ALUA)
3. **Load balancing:** Distribute I/O across multiple paths
4. **Storage policies:** Match storage tier to workload requirements
5. **Capacity planning:** Monitor growth trends, plan expansions quarterly

---

## Networking

### Virtual Switch Types

| Type | Description | Features |
|------|-------------|----------|
| **vSS (Standard Switch)** | Per-host switch, managed individually | Basic port groups, teaming |
| **vDS (Distributed Switch)** | Centralized management across cluster | Unified config, netflow, port mirroring |
| **VDS (RHV Virtual Switch)** | RHV's equivalent distributed switch | Network isolation, VLANs |

### Network Segregation

Enterprise virtualization requires separated network traffic:

1. **Management Network:** Host management and vCenter/RHV Manager communication
2. **vMotion / Migration Network:** Live migration traffic (low latency required)
3. **Storage Network:** SAN/NFS/iSCSI traffic (isolated, high bandwidth)
4. **VM Network:** Guest VM traffic (production workloads)
5. **Replication Network:** Replication traffic (separate from production)
6. **Fault Domain:** Each network on separate physical NICs and switches

### VMkernel Adapters

| Adapter | Purpose | Requirements |
|---------|---------|--------------|
| **Management (vmk0)** | Host management | IP, gateway, DNS |
| **vMotion (vmk1)** | Live migration | High bandwidth, low latency |
| **Storage (vmk2+)** | iSCSI/NFS/NVMe-oF | Dedicated VLAN, Jumbo frames |
| **Replication (vmk3+)** | vSphere Replication | Bandwidth and isolation |

### Virtual Networking Best Practices

1. **Jumbo frames (MTU 9000):** Enable for storage and vMotion networks
2. **NIC teaming:** Configure active/standby or load balancing modes
3. **vSphere Traffic Shaping:** Limit bandwidth for non-critical traffic
4. **Port group isolation:** Separate port groups for security domains
5. **Network monitoring:** Track utilization, errors, and dropped packets

---

## Resource Management

### Resource Pools

Resource pools organize and allocate CPU and memory:

- **Hierarchical structure:** Root pool can have child pools
- **Shares:** Priority allocation during resource contention
- **Reservations:** Guaranteed minimum resources
- **Limits:** Maximum resources a pool can consume

### Resource Pool Design

```
Resource Pool: Production
  ├── Resource Pool: Web Tier (shares: high)
  │   ├── VM: Web01
  │   └── VM: Web02
  ├── Resource Pool: App Tier (shares: high)
  │   └── VM: App01
  └── Resource Pool: DB Tier (shares: high)
      └── VM: DB01
```

### CPU and Memory Concepts

| Concept | Description |
|---------|-------------|
| **Overcommit** | Allocating more vCPUs/RAM than physical |
| **CPU Ready** | Percentage of time vCPU waits for physical CPU |
| **Memory Ballooning** | Guest OS driver reclaims memory under pressure |
| **Memory Swapping** | Host moves memory to disk under pressure |
| **Memory Compression** | Alternative to swapping, compresses memory |

### Performance Thresholds

| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| **CPU Ready** | < 5% | 5-10% | > 10% |
| **Memory Ballooning** | < 1% | 1-5% | > 5% |
| **Memory Swapping** | 0% | > 0% | > 10 MB/s |
| **Disk Latency** | < 20ms | 20-50ms | > 50ms |
| **Network Dropped** | 0 | < 0.1% | > 0.1% |

---

## High Availability (HA)

### HA Architecture

```
vCenter/RHV Manager ──→ Cluster ──→ Host 1 (Master)
                                    ──→ Host 2 (Slave)
                                    ──→ Host 3 (Slave)
```

### HA Components

1. **Heartbeating:** Hosts monitor each other via management network and datastores
2. **Admission Control:** Reserves resources to guarantee failover capacity
3. **Restart Priority:** Defines VM restart order during failover
4. **Isolation Response:** Action when host loses network connectivity
5. **Datastore Heartbeat:** Fallback heartbeat mechanism over storage

### HA Design Principles

1. **Dual management networks:** Prevent single network failure from causing isolation
2. **Multiple datastore heartbeats:** Use 3+ datastores for heartbeat
3. **Admission control:** Configure to reserve 1 host for failover (N+1)
4. **Consistent host config:** Similar hardware reduces restart failures
5. **Test regularly:** Validate HA failover during maintenance windows

### HA Failure Scenarios

| Scenario | Cause | Resolution |
|----------|-------|------------|
| **Host isolation** | Management network failure | Dual NICs, switch redundancy |
| **Split brain** | All heartbeats lost simultaneously | Additional heartbeat paths |
| **Admission control blocks** | Insufficient reserved resources | Adjust admission control policy |
| **Restart priority too low** | Critical VMs start last | Adjust restart priority ordering |

---

## Distributed Resource Scheduler (DRS)

### DRS Function

DRS automatically balances VM placement and resource utilization:

- **Initial placement:** Places new VMs on the least loaded host
- **Rebalancing:** Moves VMs periodically to improve balance
- **Load-based placement:** Uses actual resource utilization metrics

### DRS Automation Levels

| Level | Description |
|-------|-------------|
| **Fully Automated** | DRS decides and executes migrations |
| **Partially Automated** | DRS recommends migrations, human approves |
| **Manual** | DRS provides recommendations only |

### DRS Best Practices

1. **Fully automated for production:** Let DRS handle balancing automatically
2. **Sensitivity tuning:** Adjust based on migration frequency needs
3. **Affinity rules:** Define positive/negative VM-VM rules
4. **Host affinity:** Define VM-Host rules for specialized workloads
5. **Avoid over-migration:** High sensitivity causes excessive migrations

---

## Live Migration

### Migration Types

| Type | Technology | What Moves | Downtime |
|------|------------|------------|----------|
| **Compute Migration** | vSphere vMotion / RHV Live Migration | VM compute (CPU, RAM) | Zero |
| **Storage Migration** | Storage vMotion / RHV Storage Migration | VM disk files | Zero |
| **Both** | Live Migration (vSphere) / Live Migration (RHV) | Compute + Storage | Zero |

### Migration Requirements

1. **Shared storage:** VM storage accessible to both source and destination
2. **Compatible CPUs:** Same CPU family or EVC mode enabled
3. **Network connectivity:** Migration network between hosts
4. **Resource availability:** Destination host has capacity
5. **VM state:** VM must be powered on (for live migration)

### Migration Best Practices

1. **Schedule maintenance:** Use DRS automation for proactive rebalancing
2. **Avoid peak hours:** Manual migrations during maintenance windows
3. **Monitor migration traffic:** Ensure migration network has capacity
4. **Test first:** Validate migration paths before production changes
5. **Document procedures:** Standard migration checklists for consistency

---

## CPU Architecture for Virtualization

### NUMA Concepts

NUMA (Non-Uniform Memory Access) architecture affects VM performance:

- **NUMA Node:** CPU with attached memory — local access is faster
- **Cross-NUMA Access:** Accessing remote memory is slower
- **NUMA Awareness:** Modern hypervisors align VMs to NUMA boundaries

### NUMA Best Practices

1. **vCPU-to-socket ratio:** Match VM vCPU topology to physical layout
2. **Memory affinity:** Ensure VM memory aligns with NUMA nodes
3. **Monitor cross-NUMA:** Alert when cross-NUMA access exceeds threshold
4. **Size hosts appropriately:** Large NUMA domains increase contention risk

### CPU Scheduling

| Parameter | Description |
|-----------|-------------|
| **vCPU Count** | Number of virtual CPUs assigned to VM |
| **CPU Affinity** | Pin vCPUs to specific physical CPUs |
| **CPU Limits** | Cap maximum CPU usage for VM |
| **CPU Reservations** | Guarantee minimum CPU for VM |
| **CPU Shares** | Relative priority during contention |

---

## Memory Management

### Memory Techniques

| Technique | Description | Risk |
|-----------|-------------|------|
| **Overcommit** | Allocate more RAM than physical | Risk of swapping |
| **Ballooning** | Guest driver reclaims unused pages | Low risk |
| **Swapping** | Host moves pages to disk | High risk — I/O impact |
| **Compression** | Compress unused memory pages | Low risk |
| **Transparent Page Sharing** | Deduplicate identical memory pages | Low risk |

### Memory Best Practices

1. **Minimize overcommit:** Keep vRAM/physical RAM ratio below 1.2:1
2. **Monitor ballooning:** Alert when ballooning exceeds 1%
3. **Eliminate swapping:** Swapping indicates serious resource deficit
4. **Plan for growth:** Reserve 20% memory for unexpected demand
5. **Match application needs:** Size memory based on actual application requirements

---

## Performance Fundamentals

### The Performance Hierarchy

```
Application → Guest OS → Hypervisor → Host Hardware → Storage
     ↑           ↑           ↑           ↑            ↑
  Bottleneck  Kernel      Scheduling    CPU/RAM      I/O Path
```

### Performance Investigation Order

1. **Application level:** Is the application itself the bottleneck?
2. **Guest OS level:** Check OS metrics (CPU, memory, disk, network)
3. **Hypervisor level:** Check VM metrics (CPU ready, ballooning, latency)
4. **Host level:** Check host resource utilization and contention
5. **Storage level:** Check IOPS, latency, throughput, path status
6. **Network level:** Check utilization, errors, latency, drops

### Monitoring Strategy

| Frequency | What to Monitor | Action |
|-----------|-----------------|--------|
| **Real-time** | CPU ready, memory ballooning, disk latency | Alert on thresholds |
| **Hourly** | VM performance summary, host utilization | Trend analysis |
| **Daily** | Capacity trends, growth patterns | Capacity planning |
| **Monthly** | Cluster balance, resource allocation | Optimization |

---

## Capacity Planning

### Capacity Planning Process

1. **Measure baseline:** Document current utilization across all metrics
2. **Identify growth trends:** Project utilization over 6-12 months
3. **Calculate headroom:** Reserve 20-30% for peaks and growth
4. **Plan expansion triggers:** Define when to add capacity
5. **Review quarterly:** Update projections based on actual usage

### Capacity Formulas

| Resource | Utilization | Formula |
|----------|-------------|---------|
| **CPU** | Peak CPU usage across all hosts | `Sum(Peak vCPU usage) / (Hosts × Cores × CPU per core)` |
| **Memory** | Peak memory usage | `Sum(Peak vRAM usage) / (Hosts × Physical RAM)` |
| **Storage** | Peak utilization | `Current Used / (Total × 0.8)` — expand when approaching 80% |
| **IOPS** | Peak I/O demand | `Sum(Peak IOPS) / (Storage array peak IOPS × 0.8)` |

### Capacity Planning Checklist

- [ ] CPU utilization < 70% at peak
- [ ] Memory utilization < 80% at peak
- [ ] Storage utilization < 80%
- [ ] I/O latency < 20ms at peak
- [ ] HA headroom for at least 1 host failure
- [ ] Maintenance window capacity (N+1)
- [ ] Project growth for next 12 months
- [ ] Documented capacity thresholds and alert triggers

---

## Operational Best Practices

### Change Management

1. **Document all changes:** Before, during, after
2. **Test in non-production:** Validate changes before production
3. **Schedule maintenance windows:** Minimize business impact
4. **Rollback plan:** Always have a tested rollback procedure
5. **Communicate:** Inform stakeholders of planned changes

### Maintenance Procedures

1. **Patch scheduling:** Regular patch cycles (monthly recommended)
2. **Firmware updates:** Quarterly BIOS/BMC updates
3. **Certificate renewal:** Monitor and renew certificates proactively
4. **Configuration audits:** Quarterly review of configurations against standards
5. **Disaster recovery tests:** Semi-annual failover and recovery tests

### Incident Response

1. **Severity classification:** Critical, High, Medium, Low
2. **Escalation procedures:** Define who to contact at each severity
3. **Evidence collection:** Document all observations before remediation
4. **Resolution tracking:** Track incidents to completion
5. **Post-incident review:** Analyze root cause and prevent recurrence

---

## Risk Awareness

### Critical Risks

| Risk | Impact | Prevention |
|------|--------|------------|
| **No HA configuration** | Single host failure kills all VMs | Enable HA with admission control |
| **No backups** | Data loss on storage failure | Implement backup solution |
| **Single point of failure** | Network/storage outage | Redundant paths and components |
| **Snapshot overload** | Performance degradation | Enforce snapshot retention policy |
| **Expired certificates** | Service outages | Certificate monitoring and alerting |
| **Outdated firmware** | Compatibility and security issues | Regular firmware update schedule |
| **Resource exhaustion** | VMs become unresponsive | Capacity monitoring and planning |
| **No maintenance procedures** | Extended downtime | Documented and tested procedures |

### Security Considerations

1. **Principle of least privilege:** Minimal permissions for each role
2. **Network segmentation:** Isolate management, storage, and VM networks
3. **Encryption:** Enable encryption for sensitive VMs and storage
4. **Audit logging:** Enable and review audit logs regularly
5. **Patch management:** Keep hypervisors and management platforms current
6. **Physical security:** Restrict physical access to host hardware

---

## Common Enterprise Deployments

### Small Deployment (1-5 hosts)

- Single cluster, basic HA
- Direct-attached or simple SAN storage
- Standard switches
- Minimal DRS automation

### Medium Deployment (5-20 hosts)

- Multiple clusters, DRS enabled
- SAN or vSAN storage
- Distributed switches
- Regular backup integration
- Dedicated migration network

### Large Deployment (20+ hosts)

- Multiple clusters per vCenter/RHV Manager
- Advanced DRS policies
- vSAN or enterprise SAN
- Multiple management domains
- Dedicated NOC/SOC integration
- Multi-site replication

### High Availability Deployment

- Minimum 3 hosts per cluster (N+2)
- Dual vCenter/RHV Manager (active/standby or active/active)
- Cross-site replication
- Geographic redundancy for critical workloads
- Automated failover testing

---

## Foundation Reference Mapping

This foundation is referenced by:

- **VMware vSphere Skill Pack:** Architecture, clusters, hosts, VMs, networking, storage
- **Red Hat Virtualization Skill Pack:** Architecture, clusters, hosts, VMs, networking, storage
- **Future virtualization skill packs:** Any hypervisor technology

### Shared Concepts (do not duplicate)

When creating technology-specific content:

1. Reference this foundation for shared concepts
2. Include technology-specific details only
3. Avoid restating foundational definitions
4. Use technology-specific terminology alongside shared terms

### Example Reference Pattern

```markdown
## Host Configuration

Hosts in this platform follow the shared [Virtualization Engineering Foundation](VIRTUALIZATION_FOUNDATION.md#compute-architecture).

### Technology-Specific Details

For [technology name], hosts are configured as follows...
```

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial virtualization engineering foundation |