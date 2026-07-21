# RHV — Concepts Overview

## Purpose

This document provides the shared concepts for the Red Hat Virtualization engineering skill pack, referencing the Virtualization Engineering Foundation for shared knowledge.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     RHV Manager (Engine)                        │
│                                                                 │
│  ┌───────────────────┐  ┌───────────────────┐                  │
│  │  ovirt-engine      │  │  PostgreSQL       │                  │
│  │  (Management Core) │  │  (Configuration)  │                  │
│  └───────────────────┘  └───────────────────┘                  │
│                                                                 │
│  ┌───────────────────┐  ┌───────────────────┐                  │
│  │  Cockpit Web UI   │  │  REST API         │                  │
│  │  (Administration) │  │  (Programmatic)   │                  │
│  └───────────────────┘  └───────────────────┘                  │
│                                                                 │
│  ┌──────────────────────────────────────────────────┐          │
│  │  Hosted Engine VM (runs on one of the hosts)     │          │
│  │  ┌────────────────────────────────────────────┐  │          │
│  │  │  ovirt-hosted-engine-ha (HA Monitor)       │  │          │
│  │  └────────────────────────────────────────────┘  │          │
│  └──────────────────────────────────────────────────┘          │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  vNetwork Management (via VDSM on each host)                    │
├──────┬──────┬──────┬──────┬──────┬──────┬──────┬───────────────┤
│      │      │      │      │      │      │      │               │
│ Host │ Host │ Host │ Host │ Host │ Host │ Host │ ...           │
│  1   │  2   │  3   │  4   │  N   │      │      │               │
│ KVM  │ KVM  │ KVM  │ KVM  │ KVM  │      │      │               │
│ VDSD │ VDSD │ VDSD │ VDSD │ VDSD │      │ VDSD │               │
└──────┴──────┴──────┴──────┴──────┴──────┴──────┴───────────────┘
        │         │         │
   Storage   Storage   Storage
  Domains   Domains   Domains
```

## Key Components

### Engine (ovirt-engine)

The core management server. Manages hosts, clusters, VMs, storage, and networking.

- **Services:** ovirt-engine, ovirt-engine-websocket-proxy, PostgreSQL
- **Configuration:** `/etc/ovirt-engine/`
- **Logs:** `/var/log/ovirt-engine/`
- **Management:** REST API, Cockpit Web UI, CLI tools

### Host (VDSM + KVM)

Physical servers running KVM hypervisor and VDSM management daemon.

- **Hypervisor:** KVM/QEMU via libvirt
- **Management:** VDSM (Virtual Machine Manager Daemon)
- **Services:** vdsm, libvirtd, virtlogd
- **Configuration:** `/etc/vdsm/`
- **Logs:** `/var/log/vdsm/`

### Hosted Engine

RHV Manager runs as a VM (Hosted Engine) on one of the cluster hosts.

- **VM Type:** Special VM reserved for Engine
- **HA:** ovirt-hosted-engine-ha agent monitors and restarts if needed
- **Failover:** Automatically migrates to another host on failure

### Clusters

Logical groupings of hosts with shared policies and settings.

- **Compatibility Version:** Determines supported features
- **HA Policies:** Automatic VM restart rules
- **Network/Storage Policies:** Shared across hosts

### Storage Domains

| Type | Purpose | Backends |
|------|---------|----------|
| ISO | ISO images for OS installation | NFS, FC, iSCSI |
| Data | VM disk images | Gluster, NFS, FC, iSCSI, POSIX |
| Export | Templates and exports | NFS, FC, iSCSI |
| Backup | Backup target | NFS |

### Virtual Networks

Managed by VDSM via Open vSwitch (OVS):

- **Virtual Networks:** Isolated to VMs only
- **Trusted Networks:** Connected to physical network
- **Bonded Interfaces:** NIC teaming for redundancy
- **VLAN Tagging:** Network segmentation

## Platform Differences from VMware

| Aspect | VMware vSphere | RHV |
|--------|---------------|-----|
| Hypervisor | ESXi | KVM/QEMU on RHEL |
| Management | vCenter Server | RHV Manager (Engine) |
| Cluster Config | vSphere Cluster | VDSM-configured cluster |
| Storage | VMFS/vSAN | Gluster/NFS/iSCSI |
| Live Migration | vMotion | Live Migration (via VDSM) |
| HA | vSphere HA | VDSM HA + Hosted Engine HA |
| Networking | vSS/vDS | Open vSwitch (VDSM) |
| CLI | esxcli, PowerCLI | rhev-*, virsh, virt-* |
| Console | HTML5 Console | SPICE (via Cockpit) |

## Reference Mapping

This document references the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md) for shared concepts (virtualization theory, compute architecture, storage, networking, HA, migration, performance, capacity planning).

Technology-specific details (RHV Manager, VDSM, Gluster, SPICE) are covered in this document.

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV concepts overview |