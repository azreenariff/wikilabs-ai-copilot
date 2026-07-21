# VMware vSphere — Troubleshooting Workflows

## Purpose

This document consolidates all VMware vSphere troubleshooting workflows from the YAML data source (`workflows.yaml`) into a structured troubleshooting guide.

## Foundation Reference

This workflow pack builds on the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md) for shared concepts (HA, live migration, resource management, performance fundamentals, capacity planning). Technology-specific workflows (vCenter, vSphere HA, DRS, vMotion) are included here without restating foundational theory.

## How to Use Workflows

1. Identify the symptom from observation
2. Select the appropriate workflow
3. Follow the investigation steps in order
4. Collect evidence at each step
5. Make a diagnosis based on evidence
6. Apply the recommended fix
7. Verify resolution
8. Document findings

## Workflow 1: vCenter Server Not Starting

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather initial evidence to understand the failure | `service-control --status`, `df -h`, `free -m`, `tail -100 /var/log/vmware/vpxd/vpxd.log` |
| **Diagnosis** | Determine root cause: disk full, cert expired, database error, service dependency | Check disk, check certs, check DB, check services |
| **Remediation** | Apply fix: free space, renew cert, repair DB, restart service | `service-control --stop --all`, `service-control --start --all` |
| **Verification** | Confirm services are running and vSphere Client accessible | `service-control --status`, browser check |

### Decision Tree

- Disk full → Free space or expand VCSA disk
- Certificate expired → Renew certificate using certificate manager
- Database error → Repair or restore database
- Service dependency → Fix dependency, restart chain

### Risk Assessment

- Restarting all services: Medium risk — vSphere Client unavailable during restart
- Database repair: High risk — data loss possible
- Certificate renewal: Low risk — planned procedure

## Workflow 2: ESXi Host Disconnected

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather evidence about host disconnection | `esxcli network ping -d <vcenter>`, `cat /var/log/vmware/hostd.log | tail -50`, `esxcli network ip interface list` |
| **Diagnosis** | Determine cause: network failure, vpxa down, certificate mismatch | Check network, check services, check certificates |
| **Remediation** | Fix: reconfigure network, restart vpxa, reinstall certificate | `esxcli network vswitch standard portgroup set`, `services.sh restart` |
| **Verification** | Confirm host reconnected and management restored | `esxcli network ping -d <vcenter>`, vSphere Client check |

### Decision Tree

- Network unreachable → Check physical connectivity, reconfigure management network
- vpxa not running → Restart vpxa/service.sh
- Certificate mismatch → Reinstall host certificate from vCenter
- Host hung → Console access, reboot if necessary

### Risk Assessment

- Restarting management services: Low risk — VMs continue running
- Rebooting host: High risk — all VMs on host stop
- Reconfiguring network: Medium risk — brief management interruption

## Workflow 3: VM Performance Degradation

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather VM performance metrics | `esxcli vm process list`, `esxtop -b -n 1 | grep CPU READY`, `esxtop -b -n 1 | grep MEM`, `esxtop -b -n 1 | grep CONC` |
| **Diagnosis** | Determine if CPU, memory, disk, or snapshots causing slowdown | Check CPU ready, check ballooning, check latency, check snapshots |
| **Remediation** | Apply fix: add resources, migrate, consolidate snapshots | `Move-VM`, `esxcli storage vmfs snapshot list` |
| **Verification** | Confirm VM performance restored | `esxtop -b -n 1 | grep <vm-name>`, user verification |

### Decision Tree

- CPU Ready > 5% → Reduce vCPU count or migrate to less loaded host
- Memory ballooning → Add RAM to VM or migrate to host with more free memory
- Disk latency > 20ms → Move to faster storage, consolidate snapshots
- Many snapshots → Delete or consolidate old snapshots

### Risk Assessment

- Live migration (vMotion): Low risk — zero downtime
- Adding resources: Low risk — may increase resource usage
- Snapshot consolidation: Medium risk — brief VM pause

## Workflow 4: Datastore Nearly Full

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather storage capacity information | `esxcli storage filesystem list`, `find /vmfs/volumes -type f -size +1G`, `esxcli storage vmfs snapshot list` |
| **Diagnosis** | Determine cause of space consumption: snapshots, orphaned files, logs | Check snapshot sizes, check VM file sizes, check log sizes |
| **Remediation** | Free space: delete old snapshots, clean logs, remove orphaned files | `esxcli storage vmfs snapshot delete`, manual cleanup |
| **Verification** | Confirm free space adequate | `esxcli storage filesystem list` |

### Decision Tree

- Snapshots consuming space → Delete old or consolidate snapshots
- VM logs growing → Clean up log rotation files
- Orphaned VM files → Remove orphaned .vmdk/.vmx files
- Growth trend → Plan capacity expansion

### Risk Assessment

- Deleting snapshots: Medium risk — loses point-in-time state
- Cleaning log files: Low risk — logs can be regenerated
- Removing orphaned files: Low risk — verified as unused

## Workflow 5: HA Cluster Failure

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather HA status information | `esxcli system module get --module-name=ha-agent`, `tail -100 /var/log/vmware/fdm/vmware-fdm.log`, check vSphere Client HA status |
| **Diagnosis** | Determine HA failure cause: network, heartbeat, admission control | Check network paths, check heartbeat logs, check admission control settings |
| **Remediation** | Fix HA: restore network, restart HA agent, adjust admission control | `services.sh restart`, vSphere Client HA reconfiguration |
| **Verification** | Confirm HA is functioning and VMs can restart | HA agent status, failover test in maintenance window |

### Decision Tree

- Management network issue → Restore network connectivity, dual uplinks
- HA agent down → Restart FDM agent (`services.sh restart`)
- Admission control blocking → Adjust admission control policy
- Datastore heartbeat failing → Add additional heartbeat datastores

### Risk Assessment

- Restarting HA agent: Low risk — HA briefly unavailable during restart
- Adjusting admission control: Medium risk — affects failover capacity
- Retesting HA failover: Medium risk — VMs will restart during test

## Workflow 6: vMotion Failure

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather vMotion configuration | `esxcli network ip interface list | grep vmk1`, check vSphere Client migration policies |
| **Diagnosis** | Determine vMotion failure cause: compatibility, network, resources | Check EVC, check vMotion network, check host resources |
| **Remediation** | Fix: match EVC, fix vMotion network, free resources | `vSphere Client → Cluster → EVC`, network reconfiguration |
| **Verification** | Confirm vMotion works between hosts | Test vMotion in maintenance window |

### Decision Tree

- CPU generation mismatch → Enable EVC mode
- vMotion network down → Reconfigure vMotion network, check vmk1
- Resources insufficient → Free resources on destination host
- Configuration difference → Align host configurations

### Risk Assessment

- Enabling EVC: Low risk — may reduce some performance features
- Reconfiguring network: Medium risk — affects all VMs using network
- Changing host configuration: Medium risk — may affect VMs

## Workflow 7: Storage Path Failure

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather storage path information | `esxcli storage nmp device list`, `esxcli storage core device list`, `esxcli storage san fc adapter list` |
| **Diagnosis** | Determine storage path issue: cable, HBA, switch, LUN masking | Check paths, rescan adapters, check HBAs, check LUN mask |
| **Remediation** | Fix: reconnect cables, replace HBAs, rescan storage | `esxcli storage core adapter rescan --all`, physical cable repair |
| **Verification** | Confirm all paths restored and datastore accessible | `esxcli storage nmp device list` shows all paths |

### Decision Tree

- Single path down → Check cable, switch port, HBA for that path
- All paths down → Check storage array, SAN switch, LUN masking
- Path flapping → Check cable quality, switch firmware, HBA firmware
- Multipathing policy → Adjust ALUA policy if needed

### Risk Assessment

- Rescanning storage: Low risk — brief I/O interruption
- Replacing HBAs: Medium risk — requires maintenance window
- Reconnecting cables: Low risk — if done during maintenance

## Workflow 8: vSAN Failure

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather vSAN health information | `esxcli system module get --module-name=vsan`, check vSphere Client vSAN health, `esxcli storage vsan disk list` |
| **Diagnosis** | Determine vSAN issue: disk group, network, host failure | Check disk groups, check vSAN network, check host status |
| **Remediation** | Fix: replace failed disk, fix network, replace host | Replace physical disk, fix network, add new host |
| **Verification** | Confirm vSAN healthy and capacity restored | vSAN health dashboard, capacity check |

### Decision Tree

- Disk group failure → Replace failed disks in group
- Network issue → Fix vSAN network connectivity
- Host failure → Replace or repair host, vSAN rebuilds automatically
- Capacity exceeded → Add disks or hosts

### Risk Assessment

- Replacing disks: Medium risk — vSAN rebuilds during replacement
- Replacing hosts: High risk — requires full migration plan
- Network repair: Medium risk — vSAN traffic affected during repair

## Workflow 9: Snapshot Growth and Consolidation

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather snapshot information | `esxcli storage vmfs snapshot list`, check datastore free space |
| **Diagnosis** | Determine snapshot accumulation cause | Review snapshot chain, identify VMs with excessive snapshots |
| **Remediation** | Clean up: delete old snapshots, consolidate delta files | `esxcli storage vmfs snapshot delete`, vSphere Client snapshot manager |
| **Verification** | Confirm space freed and VM performance restored | `esxcli storage filesystem list`, VM performance check |

### Decision Tree

- Large individual snapshots → Delete oldest first
- Long snapshot chains → Consolidate entire chain
- Performance impact → Consolidate immediately, enforce retention policy
- Multiple VMs affected → Script cleanup, establish policy

### Risk Assessment

- Deleting snapshots: Medium risk — loses point-in-time state
- Consolidating chain: Medium risk — brief VM I/O pause
- Establishing policy: Low risk — long-term preventive measure

## Workflow 10: Cluster DRS Imbalance

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather cluster resource usage | `esxcli system resources memory get`, vSphere Client cluster view, `esxtop` on each host |
| **Diagnosis** | Determine imbalance cause: workload distribution, DRS settings, resource pools | Check VM placement, check DRS automation level, check resource pool allocation |
| **Remediation** | Balance: adjust DRS settings, manual migration, resource pool adjustment | `vSphere Client → Cluster → Configure → DRS`, manual vMotion |
| **Verification** | Confirm balanced resource usage across hosts | `esxtop` on each host, cluster view comparison |

### Decision Tree

- DRS disabled → Enable DRS, set to Fully Automated
- DRS sensitivity too low → Increase sensitivity for more proactive balancing
- Resource pool imbalance → Adjust pool shares and reservations
- Special workload VMs → Use affinity rules to pin to appropriate hosts

### Risk Assessment

- Enabling DRS: Low risk — automatic but controlled balancing
- Increasing DRS sensitivity: Low risk — more migrations but controlled
- Manual migration: Low risk — scheduled during maintenance

## Workflow 11: VM Tools Outdated

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather VM Tools version information | vSphere Client → VM → Summary → VMware Tools status |
| **Diagnosis** | Determine update necessity and compatibility | Check current version, check latest available, check compatibility |
| **Remediation** | Update: upgrade VM Tools, install drivers | vSphere Client → Guest OS → Install/Upgrade VMware Tools |
| **Verification** | Confirm tools running and latest version | vSphere Client → VM → Summary → VMware Tools status |

### Decision Tree

- Out of date → Schedule upgrade during maintenance window
- Not installed → Install VMware Tools
- Compatibility issue → Match tools version to VM hardware version
- Upgrade fails → Check guest OS compatibility, update guest OS first

### Risk Assessment

- Upgrading tools: Low risk — usually requires VM reboot
- Installing tools: Low risk — requires guest OS access
- Compatibility mismatch: Medium risk — may require hardware version upgrade

## Workflow 12: Networking Issues

### States

| State | Description | Commands |
|-------|-------------|----------|
| **Evidence Collection** | Gather network configuration | `esxcli network ip interface list`, `esxcli network vswitch standard list`, `esxcli network ip route list` |
| **Diagnosis** | Determine network issue type: vSwitch, port group, routing, DNS | Check vSwitch config, check port group, check routes, check DNS |
| **Remediation** | Fix: reconfigure vSwitch, add port group, fix routes | `esxcli network vswitch standard portgroup add`, route configuration |
| **Verification** | Confirm network connectivity restored | `esxcli network ping -d <destination>`, VM connectivity test |

### Decision Tree

- vSwitch misconfigured → Reconfigure vSwitch ports and uplinks
- Port group missing → Create port group on vSwitch or vDS
- Routing issue → Add or correct default gateway
- DNS issue → Configure DNS servers in vSwitch or VMkernel
- VLAN mismatch → Correct VLAN ID on port group

### Risk Assessment

- Reconfiguring vSwitch: Medium risk — may disconnect connected VMs
- Changing port groups: Medium risk — affects VM network connectivity
- Correcting routes: Low risk — minimal connectivity impact during fix

## General Evidence Collection Strategy

### Priority Order

1. **vSphere Client UI** — Fastest visual identification of issues
2. **Alarms and events** — Automated detection of problems
3. **Host-level commands** — Detailed diagnostic information
4. **Log files** — Root cause analysis
5. **Performance charts** — Trend analysis and capacity

## Documentation References

- [VMware vSphere Troubleshooting Guide](https://docs.vmware.com/en/VMware-vSphere/)
- [VMware KB Articles](https://kb.vmware.com/)
- [ESXi Command Line Reference](https://docs.vmware.com/en/VMware-vSphere/8.0/com.vmware.vsphere.esx.cli.doc/)

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Consolidated VMware vSphere workflows |