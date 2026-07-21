# Red Hat Virtualization (RHV) Skill Pack

## Skill Pack Summary

| Attribute | Value |
|-----------|-------|
| ID | rhv-engineering-skill-pack |
| Name | Red Hat Virtualization Engineering |
| Version | 1.0.0 |
| Platform | Red Hat Virtualization 4.4+ |
| Core Components | RHV Manager, Hosted Engine, Storage Domains, Hosts, VMs |
| Workflows | 10+ troubleshooting workflows |
| Commands | 50+ CLI and API references |

## Foundation Reference

This skill pack builds on the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md) for shared virtualization concepts (hypervisors, clusters, HA, live migration, resource management, performance fundamentals, capacity planning). Technology-specific details (RHV Manager, VDSM, Gluster, SPICE, VirtIO) are documented here without restating foundational theory.

## Overview

This skill pack provides engineering knowledge for Red Hat Virtualization (RHV), the enterprise virtualization platform based on Red Hat Enterprise Linux Virtualization (RHEL-V), itself built on KVM/QEMU.

RHV provides:

- Centralized management through RHV Manager
- Host management via libvirt/KVM
- Shared storage through Gluster, NFS, or iSCSI
- Automated VM lifecycle management
- High availability and live migration
- Network isolation and security

## Platform Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    RHV Manager                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │   Engine │ │  Database│ │  Web UI  │ │  API     │  │
│  │ (ovirt-  │ │ (Postgres│ │ (Cockpit │ │ (REST)   │  │
│  │  engine) │ │  / PGDG) │ │  / ovirt)│ │          │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
│                       │                                 │
├───────────────────────┼─────────────────────────────────┤
│                     Libvirt                            │
├───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┤
│   │   │   │   │   │   │   │   │   │   │   │   │   │   │
│  Host 1  Host 2  Host 3  Host N  ...                    │
│  KVM   KVM   KVM   KVM                               │
└─────────────────────────────────────────────────────────┘
         │           │
    Storage    Storage
  Domains    Domains
```

## Key Components

### RHV Manager

The central management node running:

- **ovirt-engine:** Core management engine
- **PostgreSQL:** Database for configuration and state
- **ovirt-engine-websocket-proxy:** WebSocket proxy for console access
- **Cockpit/ovirt-web-ui:** Web-based management interface
- **REST API:** Programmatic management interface

### Hosts

KVM-based hosts running:

- **vdsm:** Virtual Machine Manager daemon (orchestrates VMs)
- **libvirt:** Virtualization API for KVM management
- **qemu-kvm:** KVM hypervisor
- **virtlogd:** Serial console logging daemon
- **hosted-engine-ha:** High availability for the Manager itself

### Clusters

Logical groupings of hosts:

- **Compatibility version:** Determines supported features
- **Virtualization cluster:** For general-purpose VMs
- **Compute optimized:** For high-performance workloads
- **High availability:** With HA policies configured

### Storage Domains

| Domain Type | Purpose | Technology |
|-------------|---------|------------|
| **ISO Domain** | ISO images for OS installation | NFS, FC, iSCSI |
| **Data Domain** | VM disk images | Gluster, NFS, FC, iSCSI, POSIX |
| **Export Domain** | VM template/export storage | NFS, FC, iSCSI |
| **Backup Domain** | Backup target storage | NFS |

### Data Centers

Container for storage domains and clusters:

- **Default:** Initial data center created during installation
- **Custom:** Created for different environments (prod, dev, DR)
- **Compatibility version:** Must match or be lower than host compatibility

## VM Management

### VM Types

| Type | Description |
|------|-------------|
| **Desktop** | General-purpose VMs |
| **Server** | Server workloads |
| **High Performance** | CPU-intensive workloads |
| **Dedicated** | Host-pinned VMs |

### VM Lifecycle

```
Created → Running → (Suspended / Migrated / Snapshot)
    ↑                              ↓
    └───── Shutdown / Delete ──────┘
```

### VM Configuration

- **CPU:** Pinning, topology, features, NUMA awareness
- **Memory:** Ballooning, hot-add, huge pages
- **Disk:** Thin/thick provisioning, cache modes, I/O limits
- **Network:** VirtIO, paravirtualized drivers
- **GPU:** PCI passthrough, vGPU
- **Serial Console:** Console access via libvirt

## Networking

### Network Configuration

- **Virtual networks:** Isolated networks (VM-only)
- **Trusted networks:** Connected to physical network
- **Bonded interfaces:** NIC teaming for redundancy
- **VLAN tagging:** Network segmentation
- **SR-IOV:** Direct PCI device assignment to VMs

### vNetworks

```
vNetwork: Production
  ├── VLAN: 100
  ├── Port Groups: Web, App
  └── Isolation: Yes
  
vNetwork: Management
  ├── VLAN: 200
  ├── Port Groups: Mgmt, vMotion
  └── Isolation: Yes
```

## High Availability

### Host HA

- **Non-responsive:** VM restarts when host stops responding
- **Down:** VM restarts when host is powered off
- **Manual:** VM does not restart automatically

### Engine HA (Hosted Engine)

- **Hosted Engine:** RHV Manager runs as a VM on one of the hosts
- **HA Monitor:** Watches the Engine VM
- **Failover:** Restarts Engine on another host if it fails

## Live Migration

### Migration Types

| Type | Description |
|------|-------------|
| **Live Migration** | Migrate running VM between hosts |
| **Storage Migration** | Move VM disk to different storage domain |
| **Cutover** | Migrate VM from source to destination |

### Migration Requirements

1. Compatible host versions (same or higher compatibility)
2. Shared storage accessible from both hosts
3. Compatible network configuration
4. Sufficient resources on destination host
5. Network connectivity between hosts

## Performance Monitoring

### Key Metrics

| Metric | Tool | Threshold |
|--------|------|-----------|
| **CPU Usage** | ovirt-engine, cockpit | > 80% sustained |
| **Memory Usage** | ovirt-engine, cockpit | > 85% sustained |
| **Disk I/O** | vdsm, cockpit | High latency |
| **Network I/O** | vdsm, cockpit | High utilization |
| **Host Health** | ovirt-engine | Any red status |

### Monitoring Tools

- **ovirt-engine:** Primary monitoring interface
- **cockpit:** Host-level monitoring
- **vdsm-log:** VDSM daemon logs
- **libvirt logs:** Hypervisor-level logging
- **Gluster volumes:** Storage monitoring

## Logging

### Log Locations

| Component | Log Location |
|-----------|--------------|
| **ovirt-engine** | `/var/log/ovirt-engine/` |
| **vdsm** | `/var/log/vdsm/` |
| **libvirt** | `/var/log/libvirt/` |
| **hosted-engine** | `/var/log/hosted-engine/` |
| **PostgreSQL** | `/var/lib/pgsql/data/log/` |
| **Gluster** | `/var/log/glusterfs/` |

### Log Analysis

```bash
# Engine logs
tail -100 /var/log/ovirt-engine/server.log

# VDSM logs
tail -100 /var/log/vdsm/vdsm.log

# libvirt logs
tail -100 /var/log/libvirt/qemu/<vm-name>.log

# All recent errors
grep -r "ERROR\|FAIL" /var/log/ovirt-engine/ | tail -50
```

## Permissions and Security

### RBAC Model

| Role | Permissions |
|------|-------------|
| **Administrator** | Full access to all resources |
| **Operator** | Can start/stop VMs, manage hosts |
| **User** | Can access assigned VMs |
| **Custom** | Configurable role definitions |

### Security Best Practices

1. Use RBAC with least privilege
2. Enable TLS for all communication
3. Regularly rotate API tokens
4. Monitor audit logs
5. Network segmentation for management traffic

## Operational Guidance

### Installation

1. Install RHEL on Manager host
2. Install RHV Manager packages
3. Initialize the database
4. Configure networking
5. Add hosts to environment
6. Configure storage domains
7. Create clusters and data centers
8. Deploy VMs

### Patching

1. **Pre-patch:** Verify backups, test in non-prod
2. **Host patching:** Migrate VMs, apply patches, reboot
3. **Engine patching:** Stop engine, apply packages, restart
4. **Post-patch:** Verify all services, test VMs

### Upgrade Path

```
RHV 4.x → RHV 4.y → RHV 5.x (future)
  ↓
1. Update hosts first
2. Update Engine second
3. Verify compatibility
4. Test all workloads
```

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV skill pack |