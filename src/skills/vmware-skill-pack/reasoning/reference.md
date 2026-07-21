# VMware vSphere Engineering — Reasoning Reference

## Purpose

This document defines the diagnostic reasoning framework for VMware vSphere engineering troubleshooting.

## Reasoning Model

The VMware engineering skill pack uses a **hierarchical diagnostic reasoning** model:

### Level 1: Symptom Classification

First, classify the symptom into one of these categories:

1. **vCenter Infrastructure** — vCenter Server, SSO, certificates, database
2. **ESXi Host** — Host connectivity, services, hardware, configuration
3. **Virtual Machine** — VM performance, state, resources, snapshots
4. **Storage** — Datastores, LUNs, VMFS, vSAN
5. **Networking** — vSwitch, vDS, VMkernel, VLANs
6. **Cluster** — HA, DRS, EVC, admission control

### Level 2: Root Cause Analysis

For each category, apply the **elimination tree**:

```
Symptom Detected
    │
    ├─→ Is it infrastructure (vCenter/Host)?
    │   ├─→ Check services → service-control --status
    │   ├─→ Check connectivity → ping/esxcli network ping
    │   ├─→ Check resources → df -h, free -m
    │   └─→ Check logs → tail /var/log/vmware/*/
    │
    ├─→ Is it a VM issue?
    │   ├─→ Check CPU → esxtop CPU Ready
    │   ├─→ Check memory → esxtop MEM ballooning/swapping
    │   ├─→ Check disk → esxtop CONC latency
    │   └─→ Check snapshots → esxcli storage vmfs snapshot list
    │
    ├─→ Is it a storage issue?
    │   ├─→ Check capacity → esxcli storage filesystem list
    │   ├─→ Check paths → esxcli storage nmp device list
    │   ├─→ Check LUN → esxcli storage core device list
    │   └─→ Check health → dmesg errors
    │
    ├─→ Is it a network issue?
    │   ├─→ Check interfaces → esxcli network ip interface list
    │   ├─→ Check routing → esxcli network ip route list
    │   ├─→ Check connectivity → ping
    │   └─→ Check vSwitch → esxcli network vswitch standard list
    │
    └─→ Is it a cluster issue?
        ├─→ Check HA → esxcli system module get --module-name=ha-agent
        ├─→ Check DRS → vSphere Client cluster settings
        ├─→ Check EVC → vSphere Client cluster EVC mode
        └─→ Check admission control → cluster settings
```

### Level 3: Remediation Strategy

Once root cause is identified:

1. **Immediate fix** — Restore service/functionality
2. **Verification** — Confirm fix worked
3. **Prevention** — Prevent recurrence

### Decision Trees

#### vCenter Decision Tree
```
vCenter not starting?
    │
    ├─→ Service stopped → Restart service
    │       ↓
    │       Still failed? → Check logs
    │           ↓
    │           Disk full? → Free space
    │           Cert expired? → Renew cert
    │           DB error? → Check database
    │
    ├─→ Certificate expired → Renew certificate
    │       ↓
    │       Service restart needed
    │
    └─→ Database error → Check PostgreSQL
            ↓
            Restore from backup if needed
```

#### VM Performance Decision Tree
```
VM slow?
    │
    ├─→ CPU Ready > 5%? → Reduce vCPU or migrate
    ├─→ Memory ballooning? → Add RAM or migrate
    ├─→ Disk latency > 20ms? → Move to faster storage
    ├─→ Many snapshots? → Consolidate snapshots
    └─→ All metrics normal? → Application-level issue
```

#### Storage Decision Tree
```
Storage issue?
    │
    ├─→ Datastore full? → Delete files or expand
    ├─→ LUN unavailable? → Check SAN connectivity
    ├─→ Path failure? → Fix multipathing
    ├─→ VMFS error? → Repair or recreate
    └─→ vSAN issue? → Check disk groups and network
```

## Confidence Scoring

| Confidence | Description | Example |
|------------|-------------|---------|
| High (0.9+) | Clear error, specific cause | "vpxd failed: disk full" |
| Medium (0.7-0.9) | Multiple possible causes | "Host disconnected" |
| Low (0.5-0.7) | Vague symptoms, needs more info | "Something wrong with cluster" |
| Very Low (<0.5) | Insufficient data | "?" |

## Escalation Criteria

Escalate to VMware Support when:

1. **Data loss risk** — Potential for data corruption
2. **Cannot diagnose** — Root cause unclear after evidence collection
3. **Bug suspected** — Issue matches known VMware bug patterns
4. **Hardware failure** — Physical hardware failure suspected
5. **Out of scope** — Issue beyond skill pack scope

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial reasoning framework |