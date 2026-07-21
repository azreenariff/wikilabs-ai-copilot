# VMware vSphere Engineering — Detection Reference

## Purpose

This document defines the detection rules for the VMware vSphere engineering skill pack.

## Detection Rule Catalog

### vCenter Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| vcenter-service-failure | vpxd status != running | vCenter core service not running | Critical |
| vcenter-disk-full | VCSA disk > 90% | VCSA approaching disk capacity | Critical |
| vcenter-cert-expired | Certificate expired | SSO or vCenter certificate expired | Critical |
| vcenter-db-error | Database connection failed | VCSA database connectivity issue | High |
| vcenter-sso-failed | SSO service down | Single Sign-On service failure | High |

### Host Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| host-disconnected | Host connection == disconnected | ESXi host lost vCenter connection | High |
| host-not-responding | Host ping failed + management unreachable | ESXi host unresponsive | Critical |
| host-psod | Purple screen detected | ESXi crash (Purple Screen of Death) | Critical |
| host-maintenance-mode | Host in maintenance mode | Host intentionally removed from cluster | Info |
| host-ssl-certificate-expired | Host SSL cert expired | ESXi host certificate expired | High |

### VM Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| vm-slow-performance | CPU ready > 5% or disk latency > 20ms | VM performance degradation | Medium |
| vm-disk-full | VM disk > 90% | Guest OS disk space critical | High |
| vm-snapshot-chain-long | Snapshot chain > 3 | Excessive snapshot accumulation | Medium |
| vm-not-responding | VM not responding to ping | VM hung or crashed | High |
| vm-mem-pressure | Memory ballooning or swapping | Host memory pressure | Medium |

### Storage Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| datastore-almost-full | Datastore > 90% capacity | Datastore space critically low | Critical |
| datastore-unavailable | Datastore disconnected | Datastore not accessible | Critical |
| storage-path-lost | Multipathing path failure | Storage path unavailable | High |
| vmfs-corruption | VMFS health check failed | VMFS filesystem error | Critical |
| vsan-health-failure | vSAN health check failed | vSAN cluster unhealthy | High |

### Network Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| network-isolation | Management network unreachable | Host lost management connectivity | High |
| vmotion-network-failed | vMotion network down | vMotion VMkernel adapter down | Medium |
| portgroup-misconfigured | Port group missing or error | vSwitch port group misconfigured | Medium |
| vlan-mismatch | VLAN ID mismatch | Network segmentation issue | Medium |

## Detection Priority

Failures are prioritized by impact:

1. **Critical** — Immediate action required, potential data loss
2. **High** — Urgent attention needed, service degraded
3. **Medium** — Investigation required, performance impact
4. **Low** — Monitoring recommended, no immediate action
5. **Info** — Observational, no action needed

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial detection rule catalog |