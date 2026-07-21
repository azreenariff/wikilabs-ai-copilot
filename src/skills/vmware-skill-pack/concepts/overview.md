# VMware vSphere Engineering — Concepts Overview

## Core Concepts

This skill pack covers the fundamental concepts of VMware vSphere infrastructure engineering.

## Domains

### 1. Virtualization Architecture
vSphere is VMware's enterprise virtualization platform built on a layered architecture:
- **ESXi Hypervisor**: Bare-metal virtualization layer that runs directly on hardware
- **VMkernel**: Hypervisor kernel managing CPU scheduling, memory, networking, and storage
- **vCenter Server**: Centralized management platform coordinating hosts, VMs, and resources
- **vSphere Client**: HTML5-based management interface for administrators

### 2. Virtual Infrastructure Objects
vSphere organizes infrastructure into a hierarchical object model:
- **Datacenter**: Top-level logical container
- **Clusters**: Groups of ESXi hosts with shared management (HA/DRS)
- **Hosts**: Physical ESXi servers
- **VMs**: Guest operating systems running on ESXi
- **Resource Pools**: Hierarchical resource allocation
- **Datastores**: Storage containers for VM files
- **Networks**: Standard and distributed virtual switches

### 3. High Availability (HA)
vSphere HA provides automatic VM restart on host failure:
- **Heartbeat Monitoring**: Host-to-host communication through management network
- **Datastore Heartbeats**: Fallback monitoring through shared datastores
- **Admission Control**: Reserves resources for HA failover
- **VM Restart Priority**: Prioritizes critical VMs during failover
- **Host Isolation Response**: Action when host loses management connectivity

### 4. Distributed Resource Scheduler (DRS)
DRS automatically balances VM workloads across hosts:
- **Fully Automated**: vMotions VMs without human approval
- **Partially Automated**: Suggests vMotions for review
- **Manual**: Provides recommendations only
- **Anti-Affinity Rules**: VMs must run on different hosts
- **EVC Mode**: Ensures CPU feature set compatibility across hosts

### 5. vMotion
Live migration technology for zero-downtime maintenance:
- **vMotion**: Live migration of running VM between hosts
- **Storage vMotion**: Live migration of VM storage
- **Cross-vMotion**: Migration between different host types
- **Enhanced vMotion Compatibility (EVC)**: CPU feature masking for migration across generations

### 6. Storage Management
vSphere supports multiple storage technologies:
- **VMFS-6**: VMware's clustered file system for VM disks
- **NFS 3/4.1**: Network filesystem protocols
- **vSAN**: Software-defined storage across host disks
- **Thin Provisioning**: Allocates storage on demand
- **Storage I/O Control**: Prioritizes I/O for critical VMs

### 7. Virtual Networking
vSphere provides software-defined networking:
- **vSS (Standard Switch)**: Per-host virtual switch
- **vDS (Distributed Switch)**: Multi-host virtual switch with centralized management
- **VMkernel Adapters**: Specialized interfaces for management, vMotion, storage, fault tolerance
- **Network Teaming**: NIC redundancy and load balancing
- **VLAN Tagging**: 802.1Q segmentation

### 8. Security Model
vSphere implements layered security:
- **SSO**: Centralized authentication
- **Roles and Permissions**: RBAC for access control
- **Encryption**: VM encryption at rest
- **Certificate Management**: VMCA or custom CA
- **Audit Logging**: Security event logging
- **Lockdown Mode**: Restricts direct host access

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial concepts overview |