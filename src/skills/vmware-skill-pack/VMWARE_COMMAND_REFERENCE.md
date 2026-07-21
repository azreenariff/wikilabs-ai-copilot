# VMware vSphere — Command Reference

## Purpose

This document consolidates all VMware vSphere command guidance from the YAML data source (`commands.yaml`) into a structured command reference.

## Foundation Reference

Commands in this document (esxcli, PowerCLI, service-control, certificate-manager) operate on the concepts defined in the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md) (datastores, clusters, HA, vMotion, resource pools). No foundational virtualization theory is repeated.

---

## Architecture Decision Record

**ADR-003:** VMware command guidance follows the same format as RHV — purpose, syntax, expected result, risk, and when to use. This ensures consistency across virtualization skill packs.

---

## ESXi CLI Commands

### esxcli

The ESXi command-line interface for host management.

| Attribute | Value |
|-----------|-------|
| **Purpose** | ESXi CLI for host management, storage, networking, and VM operations |
| **Syntax** | `esxcli [namespace] [command] [options]` |
| **Expected Result** | Operates on specified ESXi entity (host, storage, network, VM) |
| **Risk** | Low to Medium — read-heavy, some write operations carry risk |
| **When to Use** | Host diagnostics, storage investigation, network troubleshooting, VM management |
| **Examples** | `esxcli vm process list`, `esxcli storage filesystem list`, `esxcli network ip interface list` |

### esxcli vm process

Virtual machine process management.

| Command | Purpose | Risk |
|---------|---------|------|
| `esxcli vm process list` | List running VMs with world IDs | Low |
| `esxcli vm process kill --type=[soft|force|hard] --world-id=<id>` | Kill VM process | High |
| **When to use** | Stuck VM investigation, process listing | Varies |

### esxcli storage

Storage management commands.

| Command | Purpose | Risk |
|---------|---------|------|
| `esxcli storage filesystem list` | List VMFS filesystems | Low |
| `esxcli storage filesystem size` | Show filesystem capacity | Low |
| `esxcli storage vmfs snapshot list` | List VMFS snapshots | Low |
| `esxcli storage core device list` | List storage devices | Low |
| `esxcli storage core adapter rescan` | Rescan storage adapters | Medium |
| `esxcli storage nmp device list` | List storage devices with NMP policy | Low |
| **When to use** | Storage troubleshooting, capacity check, path troubleshooting | Varies |

### esxcli network

Network management commands.

| Command | Purpose | Risk |
|---------|---------|------|
| `esxcli network ip interface list` | List VMkernel adapters | Low |
| `esxcli network vswitch standard list` | List standard virtual switches | Low |
| `esxcli network vswitch standard portgroup list` | List port groups | Low |
| `esxcli network ip route list` | List routing table | Low |
| `esxcli network ping -d <destination>` | Ping from specific vmkernel adapter | Low |
| **When to use** | Network troubleshooting, connectivity testing | Low |

### esxcli system

System management commands.

| Command | Purpose | Risk |
|---------|---------|------|
| `esxcli system module get --module-name=vmw_ahci` | Get module status | Low |
| `esxcli system maintenanceMode set --enable=true` | Enter maintenance mode | Medium |
| `esxcli system shutdown reboot` | Reboot host | Critical |
| **When to use** | System diagnostics, maintenance mode, reboot | Varies |

## PowerCLI Commands

PowerCLI is the PowerShell module for VMware management.

### VM Management

| Command | Purpose | Risk |
|---------|---------|------|
| `Get-VM` | List all VMs | Low |
| `Get-VM -Name <name>` | Get specific VM | Low |
| `Get-VM | Select-Object Name,NumCpu,MemoryGB` | Get VM resource summary | Low |
| `Start-VM -VM <name>` | Power on VM | Low |
| `Stop-VM -VM <name> -Guest` | Graceful shutdown | Low |
| `Restart-VMGuest -VM <name>` | Restart guest OS | Medium |
| **When to use** | VM management, inventory, reporting | Varies |

### Host Management

| Command | Purpose | Risk |
|---------|---------|------|
| `Get-VMHost` | List all hosts | Low |
| `Get-VMHost -Name <name>` | Get specific host | Low |
| `Get-VMHost | Select-Object Name,ConnectionState,Version` | Get host status summary | Low |
| `Set-VMHost -State Maintenance` | Put host in maintenance mode | Medium |
| `Set-VMHostPatch -PatchUri <uri>` | Apply patch to host | High |
| **When to use** | Host inventory, maintenance, patching | Varies |

### Storage Management

| Command | Purpose | Risk |
|---------|---------|------|
| `Get-Datastore` | List all datastores | Low |
| `Get-Datastore -Name <name>` | Get specific datastore | Low |
| `Get-Datastore | Select-Object Name,FreespaceGB,CapacityGB` | Get datastore capacity | Low |
| `Get-VMHostStorage` | Get host storage configuration | Low |
| **When to use** | Storage inventory, capacity monitoring | Low |

### Cluster Management

| Command | Purpose | Risk |
|---------|---------|------|
| `Get-Cluster` | List all clusters | Low |
| `Get-Cluster -Name <name>` | Get specific cluster | Low |
| `Get-Cluster | Select-Object Name,DhaEnabled,DrsEnabled` | Get cluster status | Low |
| `Set-Cluster -HAEnabled:$true` | Enable HA on cluster | Medium |
| `Set-Cluster -DRSEnabled:$true` | Enable DRS on cluster | Medium |
| **When to use** | Cluster configuration, status monitoring | Varies |

### Migration

| Command | Purpose | Risk |
|---------|---------|------|
| `Move-VM -VM <name> -Destination <host>` | Live migrate VM | Low |
| `Move-VM -VM <name> -Datastore <datastore>` | Storage vMotion VM | Medium |
| `New-VM -Name <name> -Datastore <ds> -VMHost <host>` | Create new VM | Low |
| **When to use** | VM migration, deployment | Varies |

## VCSA Service Management

### service-control

Manage VCSA services.

| Command | Purpose | Risk |
|---------|---------|------|
| `service-control --status` | List all services and their states | Low |
| `service-control --stop --all` | Stop all services | Medium |
| `service-control --start --all` | Start all services | Medium |
| `service-control --start --services vpxd` | Start specific service | Medium |
| `service-control --stop --services vpxd` | Stop specific service | Medium |
| **When to use** | Service troubleshooting, maintenance, restart | Varies |

### systemctl

Systemd service management for VCSA.

| Command | Purpose | Risk |
|---------|---------|------|
| `systemctl status vpxd` | Check vpxd service status | Low |
| `systemctl status vpxd-svcs` | Check vpxd-svcs status | Low |
| `systemctl status vsphere-client` | Check vSphere Client status | Low |
| `systemctl restart vpxd` | Restart vpxd service | Medium |
| `systemctl restart vsphere-client` | Restart vSphere Client | Medium |
| **When to use** | VCSA service status checking and management | Varies |

### VM service commands

| Command | Purpose | Risk |
|---------|---------|------|
| `get-pod-info.sh` | Get containerized deployment info | Low |
| `vmware-vmafd-adm status` | Check vMAFD (directory service) status | Low |
| `vmware-ssoadm list-servers` | List SSO servers | Low |
| **When to use** | VCSA infrastructure diagnostics | Low |

## Certificate Management

### certificate-manager

Manage SSL certificates on VCSA.

| Command | Purpose | Risk |
|---------|---------|------|
| `certificate-manager` | Interactive certificate management wizard | Medium |
| `vmware-certificate-manager` | Alternative cert manager | Medium |
| **When to use** | Certificate renewal, replacement, verification | Medium |

### SSL Certificate Locations

| Component | Certificate Location |
|-----------|---------------------|
| vpxd | `/var/lib/vmware/vpxd/ssl/` |
| vsphere-client | `/var/lib/vmware/vsphere-client/ssl/` |
| sso | `/var/lib/vmware/vmware-sso/ssl/` |
| hostd | `/etc/vmware/ssl/` |
| vpxd-svcs | `/var/lib/vmware/vpxd-svcs/ssl/` |

## Log Locations

| Component | Log Location |
|-----------|--------------|
| vpxd | `/var/log/vmware/vpxd/vpxd.log` |
| vpxd-svcs | `/var/log/vmware/vpxd-svcs/` |
| hostd | `/var/log/vmware/hostd.log` |
| vmkernel | `/var/log/vmware/vmkernel.log` |
| vsphere-client | `/var/log/vmware/vsphere-client/` |
| SSO | `/var/log/vmware/sso/` |
| vmafdd | `/var/log/vmware/vmafdd/` |
| VCSA | `/var/log/vmware/vcsa/` |

## Common Diagnostic Commands

| Command | Purpose |
|---------|---------|
| `esxcli vm process list` | List all running VMs with resource usage |
| `esxcli storage filesystem list` | List all VMFS volumes and capacity |
| `esxcli network ip interface list` | List all VMkernel adapters |
| `esxcli storage core device list` | List all storage devices and paths |
| `esxtop -b -n 1` | Capture performance metrics (batch mode) |
| `df -h` | Check filesystem usage |
| `free -m` | Check memory usage |
| `service-control --status` | Check VCSA service status |
| `journalctl -u vpxd -n 100` | View recent vpxd logs |
| `tail -100 /var/log/vmware/vpxd/vpxd.log` | View recent vpxd logs |

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Consolidated VMware vSphere command reference |