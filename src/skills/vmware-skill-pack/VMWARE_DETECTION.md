# VMware vSphere — Detection Reference

## Purpose

This document consolidates all VMware vSphere detection rules from the YAML source into a structured detection guide.

## Foundation Reference

Detection rules in this document apply VMware-specific patterns (vpxd, esxcli, vSphere Client URLs) to the shared detection framework defined by the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md).

## Detection Rule Catalog

### vCenter Server Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `vmware-vcenter-not-starting` | vCenter service failure | vpxd, vpxd-svcs, vsphere-client, SSO down | Critical |
| `vmware-vcenter-db-error` | vCenter database error | PostgreSQL or external DB connectivity failure | Critical |
| `vmware-vcenter-cert-error` | vCenter certificate expired | TLS certificate expiry or mismatch | Critical |
| `vmware-vcenter-storage-full` | vCenter storage full | VCSA datastore at capacity | Critical |

### ESXi Host Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `vmware-host-disconnected` | Host disconnected from vCenter | vpxa failure or management network loss | Critical |
| `vmware-host-unreachable` | Host not responding | Host management unreachable | Critical |
| `vmware-host-degraded` | Host in degraded state | High CPU, memory pressure, or disk errors | High |
| `vmware-host-kernel-panic` | ESXi kernel panic | vmkernel crash (Purple Screen) | Critical |
| `vmware-host-storage-error` | Storage path loss | Multipath failure or LUN loss | High |
| `vmware-host-memory-pressure` | Host memory exhaustion | Ballooning, swapping, or OOM | Critical |

### VM Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `vmware-vm-slow` | VM performance degraded | CPU ready, ballooning, or latency issues | Medium |
| `vmware-vm-not-running` | VM unexpectedly stopped | Guest OS crash or unexpected power off | High |
| `vmware-vm-disk-full` | VM guest disk full | Guest OS running out of space | High |
| `vmware-vm-tools-outdated` | VMware Tools outdated | Missing or outdated VMware Tools | Medium |
| `vmware-vm-migration-failed` | vMotion failed | Migration from source to destination | High |

### Storage Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `vmware-datastore-full` | Datastore nearly full | VMFS/NFS approaching capacity | Critical |
| `vmware-datastore-maintenance` | Datastore in maintenance | Being modified or repair | Medium |
| `vmware-path-loss` | Storage path lost | HBA, cable, or switch issue | High |
| `vmware-vsan-failure` | vSAN failure | Disk group, network, or host failure | Critical |

### Network Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `vmware-network-isolation` | vSwitch misconfigured | Port group or vSwitch issue | High |
| `vmware-network-vnic-error` | vNIC errors | VM network adapter configuration error | High |
| `vmware-network-bond-failure` | NIC team failover | vSwitch uplink failover occurred | Medium |
| `vmware-network-dns-error` | DNS resolution failure | DNS not resolving for hosts/VMs | High |

### Cluster Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `vmware-cluster-ha-failed` | HA failover occurred | VM restarted on another host | High |
| `vmware-cluster-compat-issue` | Compatibility mismatch | EVC or host version not matching | High |
| `vmware-cluster-drs-imbalance` | DRS imbalance | Cluster resource distribution poor | Medium |
| `vmware-cluster-admission-control` | Admission control blocking | Insufficient cluster capacity | Critical |

## Detection Patterns

### Terminal Command Patterns

```yaml
patterns:
  # VMware CLI tools
  - pattern: '^esxcli.*'
    confidence: 0.95
    technology_domain: VMware vSphere

  # PowerCLI
  - pattern: '^Get-VM|^Get-VMHost|^Move-VM|^Set-VMHostPatch'
    confidence: 0.92
    technology_domain: VMware vSphere

  # VCSA service control
  - pattern: 'systemctl.*(vpxd|vpxd-svcs|vsphere-client|sshd|sso|vsphere-ui)'
    confidence: 0.93
    technology_domain: VMware vSphere

  # service-control
  - pattern: 'service-control.*(status|start|stop)'
    confidence: 0.95
    technology_domain: VMware vSphere

  # Certificate management
  - pattern: 'certificate.*manager|certificate.*renew|ssl.*certificate'
    confidence: 0.88
    technology_domain: VMware vSphere

  # vCenter/VCSA
  - pattern: 'vpxd.*error|vpxd.*fail|vcsa.*error'
    confidence: 0.92
    technology_domain: VMware vSphere

  # ESXi host management
  - pattern: 'hostd.*error|vmkernel.*error|vpxa.*error'
    confidence: 0.90
    technology_domain: VMware vSphere

  # Storage management
  - pattern: 'VMFS|datastore.*full|storage.*path.*lost|multipath'
    confidence: 0.90
    technology_domain: VMware vSphere

  # Network management
  - pattern: 'vSwitch|portgroup|vmk|VMkernel|vDS'
    confidence: 0.87
    technology_domain: VMware vSphere

  # VM management
  - pattern: 'vMotion|VM.*migrate|VM.*snapshot|VM.*performance'
    confidence: 0.85
    technology_domain: VMware vSphere
```

### Browser URL Patterns

```yaml
browser_patterns:
  # vSphere Client
  - pattern: '.*vsphere-client.*|.*vsphere.local.*|.*vcenter.*ui.*'
    confidence: 0.95

  # vCenter Server
  - pattern: '.*vcenter.*:443.*|.*vcsa.*:443.*'
    confidence: 0.93

  # ESXi Host Client
  - pattern: '.*esxi.*:443.*|.*host.*client.*'
    confidence: 0.90
```

### Window Title Patterns

```yaml
window_patterns:
  - pattern: '.*VMware vSphere.*|.*vSphere Client.*'
    confidence: 0.95

  - pattern: '.*ESXi Host Client.*|.*ESXi.*Management.*'
    confidence: 0.92

  - pattern: '.*vCenter Server.*|.*VCSA.*'
    confidence: 0.93
```

### Text Pattern Matches

```yaml
text_patterns:
  # vCenter-related
  - pattern: 'vpxd.*error|vpxd.*fail|vcsa.*down'
    confidence: 0.90

  # Host-related
  - pattern: 'host.*disconnected|host.*unreachable|host.*not.*responding'
    confidence: 0.85

  # Storage-related
  - pattern: 'datastore.*full|datastore.*error|storage.*path.*lost'
    confidence: 0.90

  # VM-related
  - pattern: 'VM.*slow|VM.*not.*running|vMotion.*failed|VM.*stopped'
    confidence: 0.85

  # HA-related
  - pattern: 'HA.*failover|HA.*restart|non-responsive.*host'
    confidence: 0.90

  # Cluster-related
  - pattern: 'DRS.*rebalanc|cluster.*compatibility|admission.*control'
    confidence: 0.85
```

## Confidence Scoring

| Confidence | Description | Example |
|------------|-------------|---------|
| High (0.9+) | Clear error, specific cause | "vpxd service failed: disk full" |
| Medium (0.7-0.9) | Multiple possible causes | "Host unreachable from vCenter" |
| Low (0.5-0.7) | Vague symptoms, needs more info | "VM acting slow" |
| Very Low (<0.5) | Insufficient data | "Something wrong with vSphere" |

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Consolidated VMware vSphere detection rules |