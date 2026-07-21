# Red Hat Virtualization (RHV) — Command Reference

## Purpose

This document provides explanations for RHV engineering commands and tools.

## Foundation Reference

Commands in this document (rhev-*, vdsm-tool, virsh, hosted-engine, gluster, multipath, nmcli) operate on the concepts defined in the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md) (clusters, storage domains, HA, live migration, resource pools). No foundational virtualization theory is repeated.

## Architecture Decision Record

**ADR-002:** RHV command guidance follows the same format as VMware — purpose, syntax, expected result, risk, and when to use. This ensures consistency across skill packs.

---

## Engine Management Commands

### engine-setup

Initialize or reconfigure the RHV Manager.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Install or reconfigure RHV Engine (ovirt-engine) |
| **Syntax** | `engine-setup` |
| **Expected Result** | Interactive setup wizard configures database, SSL certificates, and services |
| **Risk** | Critical — reconfigures engine, may affect running Manager if run on active host |
| **When to Use** | Fresh installation, certificate renewal, database reconfiguration |
| **Example** | `engine-setup --accept-defaults` |

### engine-cleanup

Remove RHV Engine configuration.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Uninstall and clean up RHV Engine configuration |
| **Syntax** | `engine-cleanup` |
| **Expected Result** | Removes engine services, configuration, and database |
| **Risk** | Critical — permanently removes all engine configuration and data |
| **When to Use** | Reinstallation, troubleshooting engine configuration, decommissioning |
| **Example** | `engine-cleanup --quiet` |

### engine-upgrade

Upgrade RHV Engine to a newer version.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Upgrade ovirt-engine to a newer version |
| **Syntax** | `engine-upgrade` |
| **Expected Result** | Upgrades engine packages and database schema |
| **Risk** | High — database schema migration, requires backup before upgrade |
| **When to Use** | RHV platform upgrade, following upgrade path |
| **Example** | `engine-upgrade --accept-defaults` |

### engine-backup

Create a backup of the RHV Manager.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Backup engine configuration and database |
| **Syntax** | `engine-backup --mode=backup --file=<filename>` |
| **Expected Result** | Creates backup archive of engine state |
| **Risk** | Low — read-only operation |
| **When to Use** | Before upgrade, before changes, scheduled backup |
| **Example** | `engine-backup --mode=backup --file=/tmp/engine-backup.tar.gz` |

### engine-restore

Restore from an engine backup.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Restore engine from backup archive |
| **Syntax** | `engine-backup --mode=restore --file=<filename>` |
| **Expected Result** | Restores engine configuration and database from backup |
| **Risk** | High — replaces current engine state with backup |
| **When to Use** | After failed upgrade, configuration recovery, disaster recovery |
| **Example** | `engine-backup --mode=restore --file=/tmp/engine-backup.tar.gz` |

## Engine Administration CLI

### engine-admin

General purpose engine administration CLI.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Query and manage engine entities (hosts, clusters, VMs, domains) |
| **Syntax** | `engine-admin <command> [options]` |
| **Expected Result** | Lists, shows, or modifies engine entity configuration |
| **Risk** | Varies by subcommand |
| **When to Use** | Listing entities, checking status, querying configuration |
| **Examples** | `engine-admin host --list`, `engine-admin cluster --list` |

#### Common Subcommands

- `engine-admin host --list` — List all hosts
- `engine-admin host --info <host-name>` — Show host details
- `engine-admin cluster --list` — List all clusters
- `engine-admin cluster --info <cluster-name>` — Show cluster details
- `engine-admin storage-domain --list` — List storage domains
- `engine-admin storage-domain --info <domain>` — Show domain details
- `engine-admin vm --list` — List all VMs
- `engine-admin vm --info <vm-name>` — Show VM details
- `engine-admin user --list` — List users
- `engine-admin group --list` — List groups
- `engine-admin role --list` — List roles
- `engine-admin api-token --list` — List API tokens
- `engine-admin ha-policy --list` — List HA policies

## Host Management Commands

### rhev-host-install

Install or update RHV host packages.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Install or update RHEV-H or RHEL virtualization packages |
| **Syntax** | `rhev-host-install` |
| **Expected Result** | Updates host packages to specified version |
| **Risk** | High — may require host reboot |
| **When to Use** | Host patching, version upgrade |
| **Example** | `rhev-host-install --version=<version>` |

### rhev-mkiso

Create ISO image for RHV host installation.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Create bootable ISO for RHV host installation |
| **Syntax** | `rhev-mkiso --output=<file.iso>` |
| **Expected Result** | Creates bootable ISO image |
| **Risk** | Low |
| **When to Use** | Creating boot media for new host installation |
| **Example** | `rhev-mkiso --output=/tmp/rhev-host.iso` |

## VDSM Commands

### vdsm-tool

Diagnostic and troubleshooting tool for VDSM.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Collect diagnostic information from VDSM |
| **Syntax** | `vdsm-tool [collect-info|configure|check] [options]` |
| **Expected Result** | Generates diagnostics report or configures VDSM |
| **Risk** | Low — collect-info is read-only |
| **When to Use** | Troubleshooting host issues, collecting evidence for support |
| **Examples** | `vdsm-tool collect-info`, `vdsm-tool check` |

#### collect-info

Generates comprehensive diagnostic information.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Collect all host diagnostics for troubleshooting |
| **Syntax** | `vdsm-tool collect-info` |
| **Expected Result** | Creates archive with logs, configs, and system information |
| **Risk** | Low |
| **When to Use** | Before opening support case, investigating host issues |
| **Example** | `vdsm-tool collect-info` |

### systemctl vdsm

Manage VDSM daemon.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Start, stop, restart, or check VDSM service |
| **Syntax** | `systemctl [status|start|stop|restart] vdsm` |
| **Expected Result** | VDSM service changes state |
| **Risk** | Medium — restarting VDSM affects VM management |
| **When to Use** | After host config changes, troubleshooting VDSM issues |
| **Examples** | `systemctl status vdsm`, `systemctl restart vdsm` |

### systemctl libvirtd

Manage libvirt daemon.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Start, stop, restart, or check libvirt service |
| **Syntax** | `systemctl [status|start|stop|restart] libvirtd` |
| **Expected Result** | libvirt service changes state |
| **Risk** | Medium — restarting libvirt may affect VM operations |
| **When to Use** | Troubleshooting VM creation, device passthrough issues |
| **Examples** | `systemctl status libvirtd`, `systemctl restart libvirtd` |

## Libvirt/KVM Commands

### virsh

Virtual machine shell for managing KVM guests.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Create, modify, and manage KVM virtual machines |
| **Syntax** | `virsh <command> [options]` |
| **Expected Result** | Operates on specified VM |
| **Risk** | Varies by subcommand |
| **When to Use** | VM power management, device attachment, performance metrics |
| **Examples** | `virsh list`, `virsh dominfo <vm-name>`, `virsh domstats <vm-name>` |

#### Common Subcommands

- `virsh list` — List running VMs
- `virsh list --all` — List all VMs (including shut off)
- `virsh dominfo <vm>` — Show VM information
- `virsh domstats <vm> --all` — Get VM statistics
- `virsh domblkstat <vm> <disk>` — Get disk I/O stats
- `virsh domifstat <vm> <interface>` — Get network I/O stats
- `virsh domblkinfo <vm> <disk>` — Get disk size info
- `virsh start <vm>` — Start VM
- `virsh shutdown <vm>` — Graceful shutdown
- `virsh reboot <vm>` — Reboot VM
- `virsh destroy <vm>` — Force stop VM
- `virsh suspend <vm>` — Pause VM
- `virsh resume <vm>` — Resume paused VM

### virt-install

Install a new virtual machine.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Create and configure new virtual machines |
| **Syntax** | `virt-install --name=<name> --memory=<size> --vcpus=<count> ...` |
| **Expected Result** | New VM created and optionally started |
| **Risk** | Low — creates new VM |
| **When to Use** | VM deployment, testing, cloning |
| **Example** | `virt-install --name=test-vm --memory=2048 --vcpus=2 --disk=size=50` |

## Storage Management

### gluster

GlusterFS cluster management.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Manage Gluster volumes, peers, and bricks |
| **Syntax** | `gluster <command> [options]` |
| **Expected Result** | Modifies Gluster volume or peer configuration |
| **Risk** | Varies — some operations affect data availability |
| **When to Use** | Storage domain creation, volume management, troubleshooting |
| **Examples** | `gluster peer status`, `gluster volume status`, `gluster volume heal info` |

#### Common Subcommands

- `gluster peer status` — List storage peers
- `gluster volume status` — List volume status
- `gluster volume info` — List volume configuration
- `gluster volume heal <name> info` — Check healing status
- `gluster volume heal <name> full` — Trigger full heal
- `gluster peer probe <host>` — Add new peer

### multipath

Multipath device management.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Manage storage multipathing |
| **Syntax** | `multipath [list|ll|reconfig] [options]` |
| **Expected Result** | Shows or modifies multipath configuration |
| **Risk** | Medium — changing paths affects storage availability |
| **When to Use** | Troubleshooting storage paths, verifying redundancy |
| **Examples** | `multipath -ll`, `multipath -r` |

## Network Management

### ovirt-engine-networking.conf

VDSM network configuration file.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Configure virtual networks for VDSM |
| **Syntax** | Edit `/etc/ovirt-engine-networking.conf` |
| **Expected Result** | New networks applied after vdsm restart |
| **Risk** | Medium — may disconnect VMs |
| **When to Use** | Adding virtual networks, modifying network configuration |
| **Example** | Add network definition, then `systemctl restart vdsm` |

### nmcli

Network management CLI.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Manage network interfaces and connections |
| **Syntax** | `nmcli [device|connection|general] [command] [options]` |
| **Expected Result** | Network configuration changes |
| **Risk** | Medium — may interrupt network connectivity |
| **When to Use** | Troubleshooting host network issues, bond configuration |
| **Examples** | `nmcli device status`, `nmcli connection show` |

## Hosted Engine Commands

### hosted-engine

Hosted Engine management utility.

| Attribute | Value |
|-----------|-------|
| **Purpose** | Manage and monitor the Hosted Engine VM |
| **Syntax** | `hosted-engine [command] [options]` |
| **Expected Result** | Operates on Hosted Engine VM |
| **Risk** | High — affects RHV Manager availability |
| **When to Use** | HA monitoring, Engine VM troubleshooting, failover |
| **Examples** | `hosted-engine --vm-status`, `hosted-engine --deploy` |

#### Common Subcommands

- `hosted-engine --vm-status` — Check Engine VM status
- `hosted-engine --deploy` — Deploy Hosted Engine
- `hosted-engine --set-maintenance` — Enter maintenance mode
- `hosted-engine --clear-maintenance` — Exit maintenance mode

## Monitoring and Logging

### Journal Commands

View system and service logs.

| Attribute | Value |
|-----------|-------|
| **Purpose** | View logs from systemd services |
| **Syntax** | `journalctl -u <service> [-n <lines>]` |
| **Expected Result** | Shows recent log entries for the service |
| **Risk** | Low — read-only |
| **When to Use** | Troubleshooting any service, evidence collection |
| **Examples** | `journalctl -u ovirt-engine -n 100`, `journalctl -u vdsm --grep "ERROR"` |

### Log Locations

| Component | Log Location |
|-----------|--------------|
| ovirt-engine | `/var/log/ovirt-engine/` |
| vdsm | `/var/log/vdsm/` |
| libvirt | `/var/log/libvirt/` |
| hosted-engine | `/var/log/hosted-engine/` |
| PostgreSQL | `/var/lib/pgsql/data/log/` |
| Gluster | `/var/log/glusterfs/` |

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV command reference |