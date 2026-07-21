# VMware vSphere Engineering — Skill Pack

## Purpose

This document consolidates the VMware vSphere engineering skill pack into a single authoritative reference. The underlying data is sourced from:

- `src/skills/vmware-skill-pack/manifest.yaml`
- `src/skills/vmware-skill-pack/technology.yaml`
- `src/skills/vmware-skill-pack/commands.yaml`
- `src/skills/vmware-skill-pack/workflows.yaml`
- `src/skills/vmware-skill-pack/detection_rules.yaml`
- `src/skills/vmware-skill-pack/concepts/overview.md`
- `src/skills/vmware-skill-pack/concepts/terminology.md`
- `src/skills/vmware-skill-pack/examples/worked-examples.md`
- `src/skills/vmware-skill-pack/diagnostics/guide.md`
- `src/skills/vmware-skill-pack/common-failures/reference.md`
- `src/skills/vmware-skill-pack/reasoning/reference.md`
- `src/skills/vmware-skill-pack/context/interpretation.md`
- `src/skills/vmware-skill-pack/tests/reference.md`
- `src/skills/vmware-skill-pack/references/reference.md`
- `src/skills/vmware-skill-pack/knowledge/vm-management.md`

## Skill Pack Summary

| Attribute | Value |
|-----------|-------|
| ID | vmware-vsphere-engineering-skill-pack |
| Name | VMware vSphere Engineering |
| Version | 1.0.0 |
| Platform | VMware vSphere 7.0, 8.0 |
| Core Components | vCenter Server, ESXi, VMs, vSAN |
| Workflows | 12+ troubleshooting workflows |
| Commands | 60+ esxcli and PowerCLI references |

## Architecture

VMware vSphere consists of:

- **vCenter Server:** Centralized management (VCSA or Windows)
- **ESXi:** Bare-metal hypervisor
- **Clusters:** Groups of hosts with HA, DRS, EVC
- **Datastores:** Shared storage (VMFS, NFS, vSAN)
- **VMs:** Virtual machines running workloads
- **Distributed Switches:** Centralized network management

## Platform Components

### vCenter Server

- **VCSA:** VMware vCenter Server Appliance (Linux-based)
- **Windows vCenter:** Legacy Windows-based installation (deprecated)
- **Services:** vpxd, vpxd-svcs, vsphere-client, sso, PSC
- **Database:** Embedded PostgreSQL or external database

### ESXi

- **Hypervisor:** Bare-metal type 1 hypervisor
- **Management:** Host Client (web) or vCenter
- **Services:** hostd, vpxa, mgmt-vmware, sfcbd
- **Storage:** VMFS, NFS, iSCSI, FC, vSAN

### Clusters

- **HA:** High Availability — automatic VM restart
- **DRS:** Distributed Resource Scheduler — load balancing
- **EVC:** Enhanced vMotion Compatibility — CPU generation alignment
- **Admission Control:** Reserves capacity for failover

## Detection Rules

Detection rules identify VMware environments through:

- Browser URLs (vSphere Client, vCenter)
- Terminal commands (esxcli, PowerCLI, service-control)
- Window titles (vSphere Client, ESXi Host Client)
- Log patterns (vpxd, hostd, vmkernel)
- System commands (service-control, df, free)
- UI elements (red host icon, yellow VM icon)

See `src/skills/vmware-skill-pack/detection_rules.yaml` for the complete rule catalog.

## Common Failure Modes

1. **vCenter Service Failure** — vpxd, vpxd-svcs, vsphere-client down
2. **ESXi Host Disconnection** — Management network or vpxa failure
3. **VM Performance Degradation** — CPU ready, memory ballooning, disk latency
4. **Datastore Space Exhaustion** — Nearly full, snapshots, orphaned files
5. **HA Failover Failure** — Heartbeat loss, admission control
6. **vMotion Failure** — Compatibility, network, resource issues
7. **Storage Path Failure** — Multipath, HBA, cable, LUN issues
8. **vSAN Failure** — Disk group, network, host failure

## Command Guidance

### esxcli

- **Purpose:** ESXi command-line interface for host management
- **Key subcommands:** `esxcli vm process list`, `esxcli storage filesystem list`, `esxcli network ip interface list`, `esxcli storage core device list`, `esxcli system module get`
- **Risk:** Low to Medium — read-heavy, some write operations
- **When to use:** Host diagnostics, storage investigation, network troubleshooting

### PowerCLI

- **Purpose:** PowerShell module for vCenter/ESXi management
- **Key commands:** `Get-VM`, `Get-VMHost`, `Get-Datastore`, `Get-Cluster`, `Move-VM`, `Set-VMHostPatch`
- **Risk:** Varies by command — read commands low risk, write commands medium to high
- **When to use:** Bulk operations, automation scripts, reporting

### service-control

- **Purpose:** VCSA service management
- **Key commands:** `service-control --status`, `service-control --stop --all`, `service-control --start --all`
- **Risk:** Medium — affects vCenter availability
- **When to use:** Service troubleshooting, maintenance

## Workflows

Workflows follow the evidence → diagnosis → remediation → verification pattern:

1. **vCenter Not Starting** — service control, disk space, certificates
2. **Host Disconnected** — network, vpxa, certificate
3. **VM Performance** — CPU ready, memory, disk, snapshots
4. **Datastore Full** — cleanup, expansion, snapshot consolidation
5. **HA Failure** — heartbeat, admission control, configuration
6. **vMotion Failed** — compatibility, network, resources
7. **Storage Path Lost** — multipath, HBA, cable, LUN

## Reasoning Model

Hierarchical diagnostic reasoning:

- **Level 1:** Symptom classification (vCenter, Host, VM, Storage, Network, Cluster)
- **Level 2:** Root cause elimination tree
- **Level 3:** Remediation strategy (immediate, verification, prevention)

## Knowledge Reuse

This skill pack reuses shared concepts from:

- `docs/virtualization/VIRTUALIZATION_FOUNDATION.md` — Core virtualization concepts

Technology-specific content (vCenter details, esxcli commands, VMware-specific workflows) is not duplicated in the foundation.

## References

- [VMware vSphere Documentation](https://docs.vmware.com/en/VMware-vSphere/)
- [VMware KB](https://kb.vmware.com/)
- [esxcli Documentation](https://docs.vmware.com/en/VMware-vSphere/8.0/com.vmware.vsphere.esx.cli.doc/)
- [PowerCLI Documentation](https://developer.vmware.com/apis/powercli)

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Consolidated VMware vSphere skill pack reference |