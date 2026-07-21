# Foundation Relationships

## Purpose

This document defines the dependency relationships between Engineering Foundations. Skill Packs use these relationships to understand which foundational knowledge to load for each technology.

---

## Relationship Model

```
Foundation A ──depends_on──> Foundation B
Foundation A ──builds_on──> Foundation B
Foundation A ──extends──> Foundation B
Foundation A ──shares_concepts──> Foundation B
```

---

## Foundation Dependency Graph

```
                    ┌─────────────────┐
                    │   SECURITY      │
                    │   FOUNDATION    │
                    └────┬─────┬──────┘
                         │     │
         ┌───────────────┘     └───────────────┐
         ▼                                     ▼
┌─────────────────┐               ┌─────────────────────────┐
│   NETWORKING    │               │      STORAGE            │
│   FOUNDATION    │               │      FOUNDATION         │
└────┬─────┬──────┘               └────┬─────┬─────────────┘
     │     │                           │     │
     │     └───────────┬───────────────┘     │
     ▼               ▼                       ▼
┌─────────────┐ ┌─────────────┐    ┌──────────────────┐
│    LINUX    │ │   WINDOWS   │    │    CLOUD         │
│   FOUNDATION│ │   FOUNDATION│    │    FOUNDATIONS   │
└──────┬──────┘ └──────┬──────┘    └──────────────────┘
       │                │
       │                │
┌──────▼────────────────▼──────┐
│  TECHNOLOGY-SPECIFIC SKILLS  │
│  (OpenShift, VMware, etc.)   │
└──────────────────────────────┘
```

---

## Detailed Relationships

### Linux Foundation
**Used by:** All Skill Packs

| Relationship | Depends On | Reason |
|-------------|-----------|--------|
| Linux Foundation | None | Self-contained, foundational |

### Windows Foundation
**Used by:** Windows-specific Skill Packs, Database Skill Packs

| Relationship | Depends On | Reason |
|-------------|-----------|--------|
| Windows Foundation | Linux Foundation | Some overlapping concepts (processes, permissions) |

### Networking Foundation
**Used by:** All Skill Packs

| Relationship | Depends On | Reason |
|-------------|-----------|--------|
| Networking Foundation | Linux Foundation | IP addressing, routing concepts apply to Linux |
| Networking Foundation | Windows Foundation | DNS, firewall concepts apply to Windows |

### Storage Foundation
**Used by:** Linux Foundation, Database Skill Packs, VMware Skill Pack

| Relationship | Depends On | Reason |
|-------------|-----------|--------|
| Storage Foundation | Linux Foundation | LVM, filesystem concepts |
| Storage Foundation | Windows Foundation | NTFS, disk management concepts |

### Security Foundation
**Used by:** All Skill Packs

| Relationship | Depends On | Reason |
|-------------|-----------|--------|
| Security Foundation | Linux Foundation | Local authentication, SELinux |
| Security Foundation | Windows Foundation | AD, Kerberos, Group Policy |

---

## Technology-to-Foundation Mapping

| Technology | Foundations Used | Priority |
|-----------|-----------------|----------|
| **OpenShift** | Linux, Networking, Storage, Security | High |
| **Linux** | None (it IS a foundation) | N/A |
| **Windows** | Linux | Medium |
| **VMware** | Linux, Networking, Storage | High |
| **MySQL** | Linux, Networking, Storage, Security | High |
| **PostgreSQL** | Linux, Networking, Storage, Security | High |
| **MSSQL** | Windows, Networking, Storage, Security | High |
| **Ansible** | Linux, Windows, Networking | Medium |
| **Nagios XI** | Linux, Networking, Security | Medium |
| **Checkmk** | Linux, Windows, Networking | Medium |

---

## Loading Order

When loading a Skill Pack, foundations must be loaded in this order:

1. **Security Foundation** — No dependencies, always load first
2. **Networking Foundation** — Depends on Linux/Windows concepts
3. **Storage Foundation** — Depends on Linux/Windows concepts
4. **Linux Foundation** — No dependencies, load early
5. **Windows Foundation** — Depends on Linux concepts
6. **Technology Skill Pack** — Depends on all relevant foundations

### Example: OpenShift Skill Pack Loading
1. Security Foundation (no deps)
2. Linux Foundation (no deps)
3. Networking Foundation (depends on Linux)
4. Storage Foundation (depends on Linux)
5. OpenShift Skill Pack (depends on Linux, Networking, Storage, Security)

---

## Cross-Foundation Examples

### OpenShift Troubleshooting
```
OpenShift Pod not scheduling
  │
  ├─→ Linux Foundation: Node resources, cgroups, scheduling
  ├─→ Storage Foundation: PV/PVC binding, storage class
  ├─→ Networking Foundation: Service networking, CNI plugins
  └─→ Security Foundation: RBAC, SCC, service accounts
```

### VMware Performance Issue
```
VM slow on VMware host
  │
  ├─→ Linux Foundation: Guest OS resource monitoring
  ├─→ Storage Foundation: Disk I/O, datastore latency
  ├─→ Networking Foundation: VM network adapter, vSwitch
  └─→ Security Foundation: VMware host hardening, patching
```

### Windows Service Failure
```
Windows service not starting
  │
  ├─→ Windows Foundation: Service model, Event Viewer
  ├─→ Storage Foundation: Disk space, permissions
  ├─→ Networking Foundation: Service port binding, firewall
  └─→ Security Foundation: Service account permissions, UAC
```

---

## Quality Requirements

Each relationship must be:
- **Documented:** Explicitly stated in this file
- **Validated:** Tested that loading order works correctly
- **Maintained:** Updated when foundations or skill packs change

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial foundation relationships |