# VMware vSphere Engineering — Terminology

## Infrastructure Terms

| Term | Definition |
|------|------------|
| ESXi | VMware's bare-metal Type-1 hypervisor |
| vCenter Server | Centralized management platform for ESXi hosts and VMs |
| VCSA | vCenter Server Appliance (VMware-managed VM) |
| vSphere Client | HTML5-based web interface for managing vSphere |
| VMkernel | Hypervisor kernel managing hardware services |
| Cluster | Group of ESXi hosts managed together |
| Datacenter | Top-level logical container in vSphere |
| Host Cluster | Group of hosts with HA/DRS configured |
| Resource Pool | Hierarchical resource allocation within cluster |
| Datastore | Storage container for VM files |
| Standard Switch (vSS) | Per-host virtual switch |
| Distributed Switch (vDS) | Multi-host virtual switch with centralized management |

## Virtual Machine Terms

| Term | Definition |
|------|------------|
| VM | Virtual Machine — guest OS running on ESXi |
| VMX | VM configuration file |
| VMDK | Virtual Machine Disk — virtual disk file |
| Snapshot | Point-in-time state of a VM |
| Clone | Copy of a VM with unique identity |
| Template | Read-only VM used as vMotion deployment source |
| OVF/OVA | Open Virtualization Format (export/import standard) |
| Guest OS | Operating system installed inside a VM |
| VMtools | Agent software installed inside guest OS |
| Hardware Version | VM compatibility level with ESXi host |
| vCPU | Virtual CPU — logical processor assigned to VM |
| vRAM | Virtual RAM assigned to VM |
| Thin Provision | VM disk allocated on demand |
| Thick Provision | VM disk pre-allocated fully |

## HA/DRS Terms

| Term | Definition |
|------|------------|
| HA | High Availability — automatic VM restart on host failure |
| DRS | Distributed Resource Scheduler — automated VM workload balancing |
| EVC | Enhanced vMotion Compatibility — CPU feature set standardization |
| Admission Control | Resource reservation policy for HA failover |
| Heartbeat | Periodic HA agent health check |
| Datastore Heartbeat | Fallback monitoring through shared storage |
| Host Isolation | Host loses management network connectivity |
| Anti-Affinity | VM must NOT run on same host as another VM |
| Affinity | VM must run on same host as another VM |
| VM-VM Rule | Rule controlling VM/host placement |
| Host-Host Rule | Rule controlling VM placement for specific hosts |

## vMotion Terms

| Term | Definition |
|------|------------|
| vMotion | Live migration of running VM between hosts |
| Storage vMotion | Live migration of VM storage between datastores |
| Cross-vMotion | Migration between different host types |
| DRS vMotion | Automated migration by DRS |
| Manual vMotion | Administrator-initiated migration |
| Linked vMotion | vMotion + Storage vMotion together |
| Migration Priority | vMotion resource reservation (Low/Med/High/Max) |
| EVC Mode | CPU feature set compatibility for vMotion |

## Storage Terms

| Term | Definition |
|------|------------|
| VMFS | VMware File System — clustered file system for VM disks |
| NFS | Network File System — network-attached storage |
| vSAN | VMware vSphere Storage — software-defined storage |
| LUN | Logical Unit — storage array volume |
| RDM | Raw Device Mapping — direct LUN access from VM |
| Thin Provision | Storage allocated on demand |
| Thick Provision | Storage pre-allocated fully |
| Datastore Cluster | Group of datastores for Storage DRS |
| Storage I/O Control | I/O priority management for datastores |
| Multipathing | Multiple paths to same storage device |
| VMFS extent | Portion of LUN allocated to a VMFS datastore |
| VMFS extent | Portion of LUN allocated to a VMFS datastore |

## Networking Terms

| Term | Definition |
|------|------------|
| vSwitch | Virtual switch for VM networking |
| VMkernel NIC (vmk) | Specialized network interface for host services |
| VLAN | Virtual LAN — logical network segmentation |
| Port Group | Logical grouping of ports on vSwitch |
| Uplink | Physical NIC connected to vSwitch |
| Teaming | NIC teaming with load balancing policies |
| Failover Order | NIC priority for failover during failure |
| Promiscuous Mode | Network adapter security — all traffic allowed |
| MAC Address Changes | Network adapter security — MAC changes allowed |
| Forged Transmits | Network adapter security — MAC changes in outbound |
| Jumbo Frames | MTU > 1500 bytes for reduced CPU overhead |
| MTU | Maximum Transmission Unit size |

## Security Terms

| Term | Definition |
|------|------------|
| SSO | Single Sign-On — central authentication |
| vCenter SSO | vCenter's authentication service |
| AD Integration | Active Directory integration for SSO |
| Roles | Predefined or custom permission sets |
| Permissions | Role assignment to user/group on object |
| Lockdown Mode | Restricts direct host access (no console, SSH, DCUI) |
| Maintenance Mode | Host taken out of service for maintenance |
| Certificate | X.509 certificate for component authentication |
| VMCA | VMware Certificate Authority (default CA) |
| Custom Certificate | Organization's own PKI certificates |
| vCenter Certificate | vCenter Server TLS certificate |
| Host Certificate | ESXi host TLS certificate |
| Encryption | VM encryption at rest |

## Backup Terms

| Term | Definition |
|------|------------|
| Snapshot | Point-in-time state of a VM |
| Snapshot Chain | Sequence of snapshot deltas |
| Snapshot Consolidation | Merging delta files into base disk |
| Backup Window | Time period when backups run |
| Incremental | Backup of only changed blocks since last backup |
| Full Backup | Complete backup of all data |
| Changed Block Tracking (CBT) | Tracks changed disk blocks for incremental backup |
| VM-Level Backup | Backup of individual VM |
| Host-Based Backup | Backup from ESXi host perspective |

## Service Terms

| Term | Definition |
|------|------------|
| vpxd | vCenter Server core management service |
| vpxd-svcs | vCenter application services |
| vsphere-client | HTML5 web client service |
| sso | Single Sign-On service |
| vmware-fdm | Host HA agent service |
| vmware-vpxa | vCenter Agent on ESXi hosts |
| vmware-hostd | ESXi management daemon |
| vpxd-svcs | vCenter application services |
| vsphere-web | vSphere web services |
| vmware-cis | VMware Infrastructure Services |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial terminology glossary |