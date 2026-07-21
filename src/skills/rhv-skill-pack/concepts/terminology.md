# RHV — Terminology Reference

## Purpose

Glossary of Red Hat Virtualization specific terminology.

## Terminology

| Term | Definition | Notes |
|------|------------|-------|
| **Engine** | The RHV Manager (ovirt-engine), central management server | Sometimes called "Manager" |
| **vdsmd** | Virtual Machine Manager Daemon (VDSM), host management service | Runs on each host |
| **Hosted Engine** | RHV Manager VM that runs on a cluster host | Provides HA for Manager |
| **Gluster** | Distributed file system used as storage backend | Recommended for RHV |
| **VirtIO** | Paravirtualized drivers for VM devices | Network and disk adapters |
| **SPICE** | Simple Protocol for Independent Computing Environment | Remote console protocol |
| **Cluster** | Logical grouping of hosts with shared policies | Like vSphere Cluster |
| **Data Center** | Container for storage domains and clusters | Like vSphere Datacenter |
| **Storage Domain** | Storage repository for VM disks | ISO, Data, Export, Backup |
| **Non-Responsive** | HA mode where VM restarts on unresponsive host | One of three HA modes |
| **Compatibility Version** | Feature level for cluster/hosts | Must be same or lower |
| **vNetwork** | Virtual network managed by VDSM/OVS | Like vSphere Port Group |
| **Bonded Interface** | NIC teaming configuration | Like vSphere NIC Team |
| **SR-IOV** | Single Root I/O Virtualization | Direct PCI device passthrough |
| **OVS** | Open vSwitch | Software switch for virtual networks |

## RHV-Specific Terms

| Term | Definition | VMware Equivalent |
|------|------------|-------------------|
| **ovirt-engine** | RHV management server | vCenter Server |
| **vdsm** | VDSM daemon on hosts | hostd |
| **hosted-engine** | Engine running as VM | vCenter Appliance/Windows |
| **storage domain** | Storage repository | Datastore |
| **VirtIO** | Paravirtualized drivers | VMware Tools/VirtIO |
| **SPICE** | Console protocol | HTML5 Console |
| **compatibility version** | Feature level | Cluster Compatibility Level |
| **data center** | Container for domains/clusters | Datacenter |
| **cluster** | Group of hosts | Cluster |

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV terminology |