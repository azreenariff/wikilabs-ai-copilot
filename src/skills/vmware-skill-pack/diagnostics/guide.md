# VMware vSphere — Diagnostics Guide

## Purpose

This guide documents the standard diagnostics methodology for VMware vSphere infrastructure troubleshooting.

## Diagnostic Approach

### Phase 1: Symptom Identification

1. **Collect initial symptoms** — user report, monitoring alert, manual observation
2. **Classify the issue** — vCenter, ESXi host, VM, cluster, storage, network
3. **Determine scope** — single component, cluster-wide, datacenter-wide

### Phase 2: Evidence Collection

#### vCenter Issues
```bash
# Check vCenter services
service-control --status

# Check VCSA disk space
df -h

# Check VCSA memory
free -m

# Check vCenter logs
tail -100 /var/log/vmware/vpxd/vpxd.log

# Check SSO status
service-control --status ssoadm
```

#### ESXi Host Issues
```bash
# Check host connectivity
esxcli network ping -d <vcenter-ip>

# Check host services
cat /var/log/vmware/hostd.log | tail -50

# Check management network
esxcli network ip interface list

# Check storage connectivity
esxcli storage core device list

# Check system uptime
esxcli system uptime get
```

#### VM Performance Issues
```bash
# Check VM resource usage
esxcli vm process list

# Check CPU ready time
esxtop -b -n 1 | grep "CPU READY"

# Check memory usage
esxtop -b -n 1 | grep "MEM"

# Check disk latency
esxtop -b -n 1 | grep "CONC"

# Check snapshot count
# vSphere Client → VM → Snapshots
```

#### Storage Issues
```bash
# Check datastore capacity
esxcli storage filesystem list

# Check storage paths
esxcli storage nmp device list

# Check LUN status
esxcli storage core device list

# Check VMFS snapshots
esxcli storage vmfs snapshot list
```

#### Network Issues
```bash
# Check VMkernel interfaces
esxcli network ip interface list

# Check routing table
esxcli network ip route list

# Test connectivity
esxcli network ping -d <destination>

# Check port group configuration
# vSphere Client → Network → Port Groups
```

### Phase 3: Root Cause Analysis

Use the diagnostic flowchart:
1. Narrow down to the most likely cause
2. Verify with targeted commands
3. Document findings before remediation

### Phase 4: Remediation & Verification

1. Apply the fix
2. Verify with the same commands used in evidence collection
3. Monitor for regression

## Diagnostic Tools

| Tool | Purpose | Risk |
|------|---------|------|
| service-control --status | Check vCenter services | Low |
| esxcli system uptime get | Check host uptime | Low |
| esxtop | Real-time resource monitoring | Low |
| esxcli storage filesystem list | Check datastore usage | Low |
| esxcli network ip interface list | Check network config | Low |
| df -h (VCSA) | Check VCSA disk space | Low |
| free -m (VCSA) | Check VCSA memory | Low |
| tail /var/log/vmware/*/vpxd.log | Check vCenter logs | Low |
| tail /var/log/vmware/hostd.log | Check host logs | Low |

## Diagnostic Flowchart

```
vSphere Issue
    │
    ├─→ Is it a vCenter issue?
    │   ├─→ Yes → Check: service-control, df, free, logs
    │   │           ↓
    │   │       Service down? → Restart service
    │   │       Disk full? → Free space
    │   │       Cert expired? → Renew cert
    │   │
    │   └─→ No
    │
    ├─→ Is it a host issue?
    │   ├─→ Yes → Check: esxcli ping, hostd.log, management net
    │   │           ↓
    │   │       Network issue? → Fix network config
    │   │       Service issue? → Restart hostd
    │   │       Hardware issue? → Check hardware
    │   │
    │   └─→ No
    │
    ├─→ Is it a VM issue?
    │   ├─→ Yes → Check: esxcli vm process, esxtop, snapshots
    │   │           ↓
    │   │       CPU issue? → Add vCPU or migrate
    │   │       Memory issue? → Add RAM or migrate
    │   │       Disk issue? → Clean up or expand
    │   │
    │   └─→ No
    │
    ├─→ Is it a storage issue?
    │   ├─→ Yes → Check: filesystem list, nmp device, vmfs snapshot
    │   │           ↓
    │   │       Full? → Clean or expand
    │   │       Path lost? → Fix storage path
    │   │       Snapshot issue? → Consolidate
    │   │
    │   └─→ No
    │
    └─→ Is it a network issue?
        ├─→ Yes → Check: interface list, route list, ping
        │           ↓
        │       Config issue? → Fix config
        │       Connectivity? → Fix network path
        │       Switch issue? → Fix vSwitch/vDS
        │
        └─→ No → Unknown → Escalate
```

## Evidence Collection Best Practices

1. **Timestamp everything** — record when each diagnostic step runs
2. **Save outputs to files** — redirect output for later reference
3. **Document commands run** — create a troubleshooting checklist
4. **Capture before and after** — evidence before and after remediation
5. **Check multiple sources** — don't rely on a single data point

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial diagnostics methodology |