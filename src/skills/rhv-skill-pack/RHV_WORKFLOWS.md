# Red Hat Virtualization (RHV) — Troubleshooting Workflows

## Purpose

This guide documents the standard troubleshooting workflows for Red Hat Virtualization infrastructure issues.

## Foundation Reference

Workflows in this pack build on the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md) for shared concepts (HA, live migration, performance fundamentals, capacity planning). Technology-specific workflows (Hosted Engine failover, Gluster volume issues, OVS networking) are included here without restating foundational theory.

## How to Use Workflows

1. Identify the symptom from observation
2. Select the appropriate workflow
3. Follow the investigation steps in order
4. Collect evidence at each step
5. Make a diagnosis based on evidence
6. Apply the recommended fix
7. Verify resolution
8. Document findings

## Workflow 1: RHV Manager Not Starting

### Symptom

RHV Manager (ovirt-engine) service fails to start or restarts continuously.

### Evidence Collection

1. `systemctl status ovirt-engine` — Check engine service status
2. `journalctl -u ovirt-engine -n 100` — View recent engine logs
3. `df -h` — Check disk space on Manager host
4. `free -m` — Check memory on Manager host
5. `psql -U engine -d engine -c "SELECT version();" ` — Verify database accessibility
6. `tail -100 /var/log/ovirt-engine/server.log` — Check engine server log

### Decision Tree

- Service won't start → Check disk space and database connectivity
- Service starts but fails quickly → Check certificate validity
- Service starts but web UI unavailable → Check ports and firewall
- Database connection refused → Check PostgreSQL service
- Certificate expired → Renew engine certificate

### Commands

```bash
# Check engine status
systemctl status ovirt-engine

# View engine logs
tail -100 /var/log/ovirt-engine/server.log

# Check database
systemctl status postgresql
psql -U engine -d engine -c "SELECT 1;"

# Restart engine
systemctl restart ovirt-engine

# Check disk space
df -h

# Check memory
free -m
```

### Risk Assessment

- Restarting engine: Medium risk — brief management outage
- Database repair: High risk — data loss possible
- Certificate renewal: Low risk — planned procedure

## Workflow 2: Host Unavailable

### Symptom

An RHV host becomes unreachable from the RHV Manager. The host shows as "Down" or "Non-Responsive" in the web UI.

### Evidence Collection

1. `engine-admin status` (from Manager) — Check host status from engine
2. SSH to host and run: `systemctl status vdsm` — Check VDSM on host
3. SSH to host and run: `systemctl status libvirtd` — Check libvirt on host
4. SSH to host and run: `ping <engine-ip>` — Check network connectivity to Engine
5. SSH to host and run: `vdsm-tool collect-info` — Collect host diagnostics

### Decision Tree

- VDSM not running → Restart VDSM, check for core dumps
- Network unreachable → Check network config, switches, firewall
- Host hung/frozen → Console access, reboot if necessary
- Certificate mismatch → Re-install host certificate
- Storage inaccessible → Check storage network and storage domains

### Commands

```bash
# On the host:
systemctl status vdsm
systemctl status libvirtd
systemctl restart vdsm
vdsm-tool collect-info

# Check connectivity
ping <engine-ip>
traceroute <engine-ip>

# Check host status from engine
engine-admin status
```

### Risk Assessment

- Restarting VDSM: Low risk — VMs continue running
- Rebooting host: High risk — all VMs on host stop
- Reinstalling host: Medium risk — temporary management disruption

## Workflow 3: Storage Domain Problems

### Symptom

A storage domain becomes unavailable, read-only, or enters maintenance mode.

### Evidence Collection

1. Check storage domain status from Engine: `engine-admin storage-domain --list`
2. Check domain type and connection: `engine-admin storage-domain --info <domain>`
3. On host: `lsblk` — Check block devices
4. On host: `gluster volume status` (if Gluster) — Check Gluster volumes
5. On host: `cat /var/log/vdsm/storage.log` — Check VDSM storage logs
6. On host: `df -h` — Check local mount points

### Decision Tree

- Domain in maintenance → Check for ongoing maintenance operations
- Domain down → Check storage network, storage target, multipathing
- Domain read-only → Check quorum (Gluster), storage health
- Gluster volume down → Check Gluster peers, brick status, network
- NFS mount failed → Check NFS server, mount options, network

### Commands

```bash
# From Engine:
engine-admin storage-domain --list
engine-admin storage-domain --info <domain-name>

# On host - check storage:
lsblk
df -h
cat /var/log/vdsm/storage.log

# Gluster-specific:
gluster peer status
gluster volume status
gluster volume heal <volume-name> info
```

### Risk Assessment

- Changing storage domain: High risk — VM I/O affected
- Rebuilding Gluster volume: High risk — data at risk
- NFS re-mount: Low risk — brief I/O interruption

## Workflow 4: VM Performance Degradation

### Symptom

A VM is running slowly, experiencing high latency, or not meeting performance expectations.

### Evidence Collection

1. From Engine web UI: Check VM performance tab (CPU, memory, I/O)
2. From Engine API: `curl -k -u user:pass https://engine/api/vms/<vm-id>/performance`
3. On host: `virsh domstats <vm-name> --all` — Check guest metrics
4. On host: `cat /proc/driver/virtio/virtio*/stats` — VirtIO stats
5. On host: `iostat -x 1 5` — Check host disk I/O
6. On host: `top` — Check host resource usage

### Decision Tree

- High CPU usage → Check guest OS for CPU-intensive processes, adjust vCPU allocation
- High memory usage → Check for memory leaks, adjust RAM allocation
- High disk I/O latency → Check storage performance, IOPS limits, storage tier
- High network latency → Check network bandwidth, virtio driver, network isolation
- Guest OS issue → Investigate within guest (df, top, iostat, dmesg)

### Commands

```bash
# Host-level diagnostics:
virsh domstats <vm-name> --all
virsh domblkstat <vm-name> vda
virsh domifstat <vm-name> vnet0

# I/O monitoring:
iostat -x 1 5
iotop

# Host resource usage:
top
htop
free -m
```

### Risk Assessment

- Adjusting VM resources: Low risk — may require VM reboot
- Live migration: Low risk — zero downtime
- Host resource adjustment: Medium risk — affects other VMs

## Workflow 5: Network Isolation

### Symptom

VMs lose network connectivity, or specific virtual networks become isolated.

### Evidence Collection

1. From Engine: Check virtual network configuration and status
2. On host: `ovs-vsctl show` — Check Open vSwitch configuration
3. On host: `nmcli device status` — Check physical network devices
4. On host: `cat /etc/ovirt-engine-networking.conf` — Check network config
5. On host: `ping` from VM — Check connectivity within and outside VM

### Decision Tree

- All VMs affected → Check physical network, switches, bonds
- Specific network affected → Check virtual network config, VLAN settings
- Bond failover detected → Check NIC status, switch port, bond mode
- vNIC errors → Check VM network adapter configuration
- DNS issues → Check DNS settings in Engine and guest VMs

### Commands

```bash
# OVS configuration:
ovs-vsctl show
ovs-vsctl list interfaces

# Physical network:
nmcli device status
nmcli device show <device>
ip addr show

# Network configuration:
cat /etc/ovirt-engine-networking.conf
cat /etc/sysconfig/network-scripts/ifcfg-*

# Connectivity tests:
ping <gateway>
ping <external-dns>
traceroute <destination>
```

### Risk Assessment

- Changing network configuration: Medium risk — may disconnect VMs
- Bond reconfiguration: Medium risk — brief network interruption
- VLAN changes: Medium risk — may isolate affected traffic

## Workflow 6: Cluster Compatibility Issues

### Symptom

A host cannot join a cluster, or VMs cannot migrate between hosts in a cluster.

### Evidence Collection

1. From Engine: Check cluster compatibility version
2. From Engine: Check host compatibility version
3. Compare host RHEL/RHV version against cluster compatibility
4. Check for EVC-like compatibility issues between CPU generations

### Decision Tree

- Host version < cluster version → Upgrade host to match cluster
- Host version > cluster version → Downgrade cluster compatibility
- CPU generation mismatch → Upgrade cluster compatibility or replace host
- Mixed RHV versions → Standardize to single version before mixing

### Commands

```bash
# From Engine:
engine-admin cluster --list
engine-admin cluster --info <cluster-name>
engine-admin host --list
engine-admin host --info <host-name>

# On host:
rpm -q rhev-host
rpm -q vdsm
uname -r
```

### Risk Assessment

- Upgrading cluster compatibility: High risk — requires maintenance window
- Downgrading cluster compatibility: Medium risk — may limit features
- Adding new host to cluster: Low risk — if versions compatible

## Workflow 7: High Availability Failover

### Symptom

HA has triggered a failover, or HA is not functioning as expected.

### Evidence Collection

1. From Engine: Check HA events and log entries
2. From Engine: Check which hosts had VMs restarted
3. On failed host (if accessible): `journalctl -u vdsm` — Check VDSM logs
4. On other hosts: Check resource availability for future failovers
5. Check admission control configuration in Engine

### Decision Tree

- HA failed to restart VM → Check resource availability, host health
- HA restarted on wrong host → Check HA policy, host compatibility
- No HA events but host down → Check heartbeat mechanism, storage access
- HA repeatedly fails → Check root cause of original failure, adjust policy

### Commands

```bash
# HA events from Engine:
journalctl -u ovirt-engine --grep "HA\|Failover\|Non-Responsive" -n 100

# On the host:
journalctl -u vdsm --grep "HA\|Failover\|Non-Responsive" -n 100

# Engine HA configuration:
engine-admin ha-policy --list
engine-admin ha-policy --info <policy-name>
```

### Risk Assessment

- Manually restarting VMs: Low risk — planned procedure
- Adjusting HA policy: Medium risk — changes future behavior
- Investigating root cause: Low risk — read-only analysis

## Workflow 8: Hosted Engine Failover

### Symptom

The RHV Manager (Hosted Engine VM) has failed over to another host, or the Engine VM is unhealthy.

### Evidence Collection

1. From any host: `hosted-engine --vm-status` — Check Engine VM status
2. From Engine host: `systemctl status ovirt-hosted-engine-ha` — Check HA agent
3. Check Engine VM resource usage (CPU, memory, disk)
4. Check Engine service health: `systemctl status ovirt-engine`
5. Check Engine database health: `psql -U engine -d engine -c "SELECT 1;"`

### Decision Tree

- Engine VM failed over → Normal HA behavior, investigate why original host failed
- Engine VM not running → Check Hosted Engine HA agent, try manual start
- Engine running but degraded → Check resources (disk, memory, CPU), restart engine
- Hosted Engine HA agent stuck → Restart HA agent service
- Quorum issue → Check storage domain availability, resolve storage issue

### Commands

```bash
# Check Engine VM status:
hosted-engine --vm-status

# HA agent status:
systemctl status ovirt-hosted-engine-ha
systemctl status ovirt-hosted-engine-setup

# Restart HA agent:
systemctl restart ovirt-hosted-engine-ha

# Engine health:
systemctl status ovirt-engine
journalctl -u ovirt-engine -n 50
```

### Risk Assessment

- Manually starting Engine VM: Medium risk — affects management
- Restarting HA agent: Low risk — HA agent restarts automatically
- Investigating Engine health: Low risk — read-only diagnostics

## Workflow 9: Storage Performance Issues

### Symptom

VMs experience slow disk I/O, storage domain shows high latency, or VMs hang due to storage issues.

### Evidence Collection

1. From Engine: Check storage domain performance metrics
2. On host: `iostat -x 1 10` — Check storage device latency
3. On host: `multipath -ll` — Check multipathing status
4. On host: `cat /sys/block/*/queue/health` — Check device health
5. On Gluster host: `gluster volume status` — Check brick health
6. On NFS host: `nfsstat -s` — Check NFS server stats

### Decision Tree

- Single path slow → Check cable, HBA, switch port
- All paths slow → Check storage array, storage network, storage server
- Gluster brick slow → Check brick disk, network to brick server
- NFS mount slow → Check NFS server, NFS options, network
- Multipath imbalanced → Check ALUA, path priorities, I/O scheduler

### Commands

```bash
# Storage device health:
iostat -x 1 10
smartctl -a /dev/<device>
multipath -ll

# Gluster diagnostics:
gluster volume status
gluster volume heal <volume> info
gluster peer status

# NFS diagnostics:
nfsstat -s
showmount -e <nfs-server>
```

### Risk Assessment

- Changing multipathing: Medium risk — may affect I/O paths
- Rebalancing Gluster: Medium risk — temporary performance impact
- Replacing storage: High risk — requires migration plan

## Workflow 10: Permission and Access Issues

### Symptom

Users cannot access VMs, hosts, or storage domains they should have access to.

### Evidence Collection

1. From Engine: Check user role and permissions
2. From Engine: Check user-group membership
3. From Engine: Check role definition (custom roles if used)
4. Check API token validity: `engine-admin api-token --list`
5. Check TLS certificate validity for API access

### Decision Tree

- User has no permissions → Assign appropriate role
- User in wrong group → Move to correct group
- API token expired → Regenerate API token
- Certificate mismatch → Re-issue or renew certificate
- Role restriction too tight → Adjust role permissions

### Commands

```bash
# From Engine:
engine-admin user --list
engine-admin user --info <user-name>
engine-admin group --list
engine-admin role --list
engine-admin role --info <role-name>
engine-admin api-token --list

# Certificate check:
engine-certificates --check
```

### Risk Assessment

- Changing permissions: Low risk — immediate effect
- Changing roles: Medium risk — affects all users in role
- API token rotation: Medium risk — disrupts existing API sessions

## General Evidence Collection Strategy

### Priority Order

1. **Engine events and logs** — Fastest way to identify issues
2. **Host status from Engine** — Comprehensive host state
3. **Host-level logs** — VDSM, libvirt, and system logs
4. **Storage diagnostics** — Storage health and performance
5. **Network diagnostics** — Connectivity and configuration

### Documentation References

- [Red Hat Virtualization Documentation](https://access.redhat.com/documentation/en-us/red_hat_virtualization/)
- [Red Hat Virtualization Administration Guide](https://access.redhat.com/documentation/en-us/red_hat_virtualization/)
- [KVM/QEMU Documentation](https://www.qemu.org/docs/)

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV workflows |