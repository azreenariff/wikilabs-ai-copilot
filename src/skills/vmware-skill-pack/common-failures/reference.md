# VMware vSphere Engineering — Common Failures

## Overview

This document catalogs common VMware vSphere failure modes with symptoms, root causes, and resolutions.

## Failure Catalog

### 1. vCenter Service Failure

**Severity:** Critical
**Impact:** vSphere Client unavailable, no management
**Frequency:** Medium

**Symptoms:**
- vSphere Client shows "Cannot connect to the server"
- Services: vpxd, vpxd-svcs, vsphere-client in FAILED state
- Cannot create, modify, or monitor VMs

**Root Causes:**
- Disk space exhaustion on VCSA
- Certificate expiry
- Database corruption
- Memory exhaustion
- Service dependency failure

**Resolution:**
```bash
# Check service status
service-control --status

# Check disk space
df -h

# Check logs
tail -100 /var/log/vmware/vpxd/vpxd.log

# Restart services
service-control --stop --all
service-control --start --all
```

### 2. ESXi Host Disconnection

**Severity:** High
**Impact:** Host unmanaged, VMs unmonitored
**Frequency:** Medium

**Symptoms:**
- Host shows red/disconnected in vSphere Client
- Alarms: "Host has been disconnected"
- Cannot perform operations on host or VMs

**Root Causes:**
- Management network failure
- vpxa service failure
- Certificate mismatch
- Network configuration error

**Resolution:**
```bash
# Check connectivity
esxcli network ping -d <vcenter-ip>

# Check host services
cat /var/log/vmware/hostd.log | tail -50

# Restart management services
services.sh restart
```

### 3. VM Performance Degradation

**Severity:** Medium
**Impact:** Poor application performance
**Frequency:** High

**Symptoms:**
- Slow application response times
- High CPU ready time (>5%)
- Memory ballooning or swapping
- High disk latency

**Root Causes:**
- vCPU overcommitment
- Memory pressure on host
- Storage I/O bottleneck
- Excessive snapshots
- Application-level issues

**Resolution:**
```bash
# Check resource usage
esxtop -b -n 1

# Check CPU ready
esxtop -b -n 1 | grep "CPU READY"

# Check memory
esxtop -b -n 1 | grep "MEM"

# Check disk latency
esxtop -b -n 1 | grep "CONC"
```

### 4. Datastore Space Exhaustion

**Severity:** Critical
**Impact:** VMs cannot write, potential data loss
**Frequency:** Medium

**Symptoms:**
- "No space left on device" errors
- VMs fail to power on
- Backups fail
- Alarms: "Datastore nearly full"

**Root Causes:**
- Unmonitored growth
- Excessive snapshots
- Log file accumulation
- Orphaned VM files
- Lack of capacity planning

**Resolution:**
```bash
# Check datastore usage
esxcli storage filesystem list

# Find large files
find /vmfs/volumes -type f -size +1G -exec ls -lh {} \;

# Delete old snapshots
esxcli storage vmfs snapshot list

# Clean VM logs
rm /vmfs/volumes/<ds>/<vm>/vmware.log.*
```

### 5. HA Failover Failure

**Severity:** Critical
**Impact:** VMs not restarting on host failure
**Frequency:** Low

**Symptoms:**
- HA shows red status
- VMs not restarting after host failure
- Alarms: "HA agent failure"
- Host isolation detected

**Root Causes:**
- Management network failure
- Incorrect admission control settings
- HA agent corruption
- Datastore heartbeat failure
- Configuration error

**Resolution:**
```bash
# Check HA status
esxcli system module get --module-name=ha-agent

# Check HA logs
tail -100 /var/log/vmware/fdm/vmware-fdm.log

# Restart HA agent
services.sh restart

# Toggle HA on cluster
vSphere Client → Cluster → Configure → HA
```

### 6. vMotion Failure

**Severity:** Medium
**Impact:** Cannot migrate VMs for maintenance
**Frequency:** Medium

**Symptoms:**
- vMotion errors in vSphere Client
- Migration tasks fail
- Alarms: "vMotion cannot complete"

**Root Causes:**
- Network compatibility issue
- EVC mode mismatch
- Resource constraints
- Configuration difference between hosts
- Storage access issue

**Resolution:**
```bash
# Check vMotion network configuration
esxcli network ip interface list

# Check EVC compatibility
vSphere Client → Hosts → EVC Mode

# Check resource availability
esxcli system resources memory get

# Verify storage connectivity
esxcli storage core device list
```

### 7. Storage Path Failure

**Severity:** High
**Impact:** Datastore inaccessible, VM I/O error
**Frequency:** Medium

**Symptoms:**
- Datastore shows red
- VMs report disk errors
- Alarms: "Storage path unavailable"
- Multipathing failure

**Root Causes:**
- SAN switch failure
- HBA failure
- Cable/fibre issue
- LUN masking problem
- Incorrect multipathing policy

**Resolution:**
```bash
# Check path status
esxcli storage nmp device list

# Rescan storage
esxcli storage core adapter rescan --all

# Check HBAs
esxcli storage san fc adapter list

# Check storage device status
esxcli storage core device list
```

### 8. vSAN Failure

**Severity:** High
**Impact:** Storage capacity reduced, potential data loss
**Frequency:** Low

**Symptoms:**
- vSAN capacity reduced
- Alarms: "vSAN health check failed"
- VMs cannot access storage
- Capacity alerts

**Root Causes:**
- Disk group failure
- Network issue between hosts
- Host failure
- Configuration error
- Capacity threshold exceeded

**Resolution:**
```bash
# Check vSAN status
esxcli system module get --module-name=vsan

# Review vSAN health
vSphere Client → vSAN → Health

# Check disk groups
esxcli storage vsan disk list

# Check vSAN network connectivity
esxcli network ping -d <vsan-host-ip>
```

## Failure Matrix

| Failure | Severity | Frequency | Auto-Recovery | Manual Intervention |
|---------|----------|-----------|---------------|-------------------|
| vCenter Service | Critical | Medium | No | Service restart |
| Host Disconnected | High | Medium | Yes | Network fix |
| VM Performance | Medium | High | No | Resource adjustment |
| Datastore Full | Critical | Medium | No | Space cleanup |
| HA Failure | Critical | Low | No | HA restart |
| vMotion Failure | Medium | Medium | No | Config fix |
| Storage Path | High | Medium | Yes | Path recovery |
| vSAN Failure | High | Low | Partial | Disk/host fix |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial common failures catalog |