# Virtualization Engineering — Test Reference

## Purpose

This document defines the validation test specifications for the virtualization engineering skill packs.

## Test Suite Overview

| Test ID | Name | Target | Type | Status |
|---------|------|--------|------|--------|
| `T001` | Virtualization Foundation Completeness | Foundation doc | Structural | Pass |
| `T002` | VMware Skill Pack Completeness | VMware docs | Structural | Pass |
| `T003` | RHV Skill Pack Completeness | RHV docs | Structural | Pass |
| `T004` | VMware YAML Validation | manifest.yaml, technology.yaml, workflows.yaml, detection_rules.yaml | Syntax | Pass |
| `T005` | RHV YAML Validation | manifest.yaml, technology.yaml, workflows.yaml, detection_rules.yaml | Syntax | Pass |
| `T006` | No Concept Duplication | Foundation vs VMware vs RHV | Semantic | Pass |
| `T007` | Cross-Reference Integrity | All docs reference each other | Structural | Pass |
| `T008` | Version Consistency | All docs version tags | Semantic | Pass |

---

## T001: Virtualization Foundation Completeness

### Objective

Verify that `docs/virtualization/VIRTUALIZATION_FOUNDATION.md` covers all required foundation topics.

### Test Cases

| Case | Topic | Expected Content |
|------|-------|-----------------|
| T001.1 | Virtualization concepts | Definition, Type 1 vs Type 2, key terminology |
| T001.2 | Compute architecture | Host configuration, cluster fundamentals, design principles |
| T001.3 | Virtual machines | Lifecycle, sizing, templates, snapshot best practices |
| T001.4 | Storage architecture | Types, VMFS/vSAN, performance metrics, design |
| T001.5 | Networking | Virtual switch types, segregation, VMkernel adapters |
| T001.6 | Resource management | Resource pools, CPU/memory concepts, performance thresholds |
| T001.7 | High availability | HA architecture, components, design principles, failure scenarios |
| T001.8 | Distributed Resource Scheduler | DRS function, automation levels, best practices |
| T001.9 | Live migration | Types, requirements, best practices |
| T001.10 | CPU architecture | NUMA, CPU scheduling |
| T001.11 | Memory management | Techniques, best practices |
| T001.12 | Performance fundamentals | Hierarchy, investigation order, monitoring |
| T001.13 | Capacity planning | Process, formulas, checklist |
| T001.14 | Operational best practices | Change management, maintenance, incident response |
| T001.15 | Risk awareness | Critical risks, security considerations |
| T001.16 | Enterprise deployments | Small, medium, large, HA deployment profiles |

### Verification

- All 16 topics present in document
- No technology-specific details (no esxcli, no rhev-*, no VMFS specifics)
- References to technology-specific skill packs present
- Version table present

---

## T002: VMware Skill Pack Completeness

### Objective

Verify that VMware skill pack has all required documentation components.

### Required Files

| File | Path | Status |
|------|------|--------|
| `VMWARE_SKILL_PACK.md` | `src/skills/vmware-skill-pack/VMWARE_SKILL_PACK.md` | Present |
| `VMWARE_WORKFLOWS.md` | `src/skills/vmware-skill-pack/VMWARE_WORKFLOWS.md` | Present |
| `VMWARE_DETECTION.md` | `src/skills/vmware-skill-pack/VMWARE_DETECTION.md` | Present |
| `VMWARE_GUIDANCE.md` | `src/skills/vmware-skill-pack/VMWARE_GUIDANCE.md` | Present |
| `VMWARE_COMMAND_REFERENCE.md` | `src/skills/vmware-skill-pack/VMWARE_COMMAND_REFERENCE.md` | Present |
| `manifest.yaml` | `src/skills/vmware-skill-pack/manifest.yaml` | Present (existing) |
| `technology.yaml` | `src/skills/vmware-skill-pack/technology.yaml` | Present (existing) |
| `commands.yaml` | `src/skills/vmware-skill-pack/commands.yaml` | Present (existing) |
| `workflows.yaml` | `src/skills/vmware-skill-pack/workflows.yaml` | Present (existing) |
| `detection_rules.yaml` | `src/skills/vmware-skill-pack/detection_rules.yaml` | Present (existing) |
| `concepts/overview.md` | `src/skills/vmware-skill-pack/concepts/overview.md` | Present (existing) |
| `concepts/terminology.md` | `src/skills/vmware-skill-pack/concepts/terminology.md` | Present (existing) |
| `tests/reference.md` | `src/skills/vmware-skill-pack/tests/reference.md` | Present (existing) |
| `references/reference.md` | `src/skills/vmware-skill-pack/references/reference.md` | Present (existing) |

### Content Verification

| Test | Check | Expected |
|------|-------|----------|
| T002.1 | VMWARE_SKILL_PACK.md | Contains overview, architecture, components, reference mapping |
| T002.2 | VMWARE_WORKFLOWS.md | Contains 10+ workflows with evidence → diagnosis → remediation → verification |
| T002.3 | VMWARE_DETECTION.md | Contains rule catalog, patterns, confidence scoring |
| T002.4 | VMWARE_GUIDANCE.md | Contains operations, best practices, tuning, planning, decision frameworks |
| T002.5 | VMWARE_COMMAND_REFERENCE.md | Contains esxcli, PowerCLI, service-control, certificate-manager entries |
| T002.6 | Cross-references to foundation | References `docs/virtualization/VIRTUALIZATION_FOUNDATION.md` |
| T002.7 | Version table | Present in all docs |
| T002.8 | No foundation duplication | No repeated virtualization theory (architecture, HA, migration, etc.) |

---

## T003: RHV Skill Pack Completeness

### Objective

Verify that RHV skill pack has all required documentation components.

### Required Files

| File | Path | Status |
|------|------|--------|
| `RHV_SKILL_PACK.md` | `src/skills/rhv-skill-pack/RHV_SKILL_PACK.md` | Present |
| `RHV_WORKFLOWS.md` | `src/skills/rhv-skill-pack/RHV_WORKFLOWS.md` | Present |
| `RHV_DETECTION.md` (in detection/) | `src/skills/rhv-skill-pack/detection/reference.md` | Present |
| `RHV_GUIDANCE.md` | `src/skills/rhv-skill-pack/RHV_GUIDANCE.md` | Present |
| `RHV_COMMAND_REFERENCE.md` | `src/skills/rhv-skill-pack/RHV_COMMAND_REFERENCE.md` | Present |
| `manifest.yaml` | `src/skills/rhv-skill-pack/manifest.yaml` | Present |
| `technology.yaml` | `src/skills/rhv-skill-pack/technology.yaml` | Present |
| `concepts/overview.md` | `src/skills/rhv-skill-pack/concepts/overview.md` | Present |
| `concepts/terminology.md` | `src/skills/rhv-skill-pack/concepts/terminology.md` | Present |
| `workflows.yaml` | `src/skills/rhv-skill-pack/workflows.yaml` | Present |
| `detection_rules.yaml` | `src/skills/rhv-skill-pack/detection_rules.yaml` | Present |

### Content Verification

| Test | Check | Expected |
|------|-------|----------|
| T003.1 | RHV_SKILL_PACK.md | Contains overview, architecture, components, reference mapping |
| T003.2 | RHV_WORKFLOWS.md | Contains 10 workflows with evidence → diagnosis → remediation → verification |
| T003.3 | detection/reference.md | Contains detection rule catalog, patterns, confidence scoring |
| T003.4 | RHV_GUIDANCE.md | Contains operations, best practices, tuning, planning, decision frameworks |
| T003.5 | RHV_COMMAND_REFERENCE.md | Contains rhev-*, vdsm-tool, virsh, hosted-engine, gluster entries |
| T003.6 | Cross-references to foundation | References `docs/virtualization/VIRTUALIZATION_FOUNDATION.md` |
| T003.7 | Version table | Present in all docs |
| T003.8 | No foundation duplication | No repeated virtualization theory |

---

## T004: VMware YAML Validation

### Objective

Verify all VMware YAML files parse correctly.

### Test Cases

| File | Check | Expected |
|------|-------|----------|
| `manifest.yaml` | Parse as YAML | Valid YAML, required fields present |
| `technology.yaml` | Parse as YAML | Valid YAML, technology definition complete |
| `commands.yaml` | Parse as YAML | Valid YAML, command definitions present |
| `workflows.yaml` | Parse as YAML | Valid YAML, workflow definitions present |
| `detection_rules.yaml` | Parse as YAML | Valid YAML, detection rules present |

### Required YAML Fields

- `manifest.yaml`: id, name, version, technology_domain, enabled, schema_version
- `technology.yaml`: architecture, storage, virtualization, security sections
- `commands.yaml`: command definitions with purpose, syntax, risk, examples
- `workflows.yaml`: workflow definitions with phases, commands, risk
- `detection_rules.yaml`: rule definitions with triggers, context, confidence

---

## T005: RHV YAML Validation

### Objective

Verify all RHV YAML files parse correctly.

### Test Cases

| File | Check | Expected |
|------|-------|----------|
| `manifest.yaml` | Parse as YAML | Valid YAML, required fields present |
| `technology.yaml` | Parse as YAML | Valid YAML, technology definition complete |
| `workflows.yaml` | Parse as YAML | Valid YAML, workflow definitions present |
| `detection_rules.yaml` | Parse as YAML | Valid YAML, detection rules present |

### Required YAML Fields

- `manifest.yaml`: id, name, version, technology_domain, enabled, schema_version
- `technology.yaml`: architecture, storage, virtualization, security sections
- `workflows.yaml`: workflow definitions with phases, commands, risk
- `detection_rules.yaml`: rule definitions with triggers, context, confidence

---

## T006: No Concept Duplication

### Objective

Verify that foundation concepts are not duplicated across VMware and RHV docs.

### Test Cases

| Check | Expected Result |
|-------|-----------------|
| Foundation defines virtualization theory | Yes — in VIRTUALIZATION_FOUNDATION.md |
| VMware docs reference foundation | Yes — cross-links present |
| RHV docs reference foundation | Yes — cross-links present |
| VMware docs repeat HA theory | No — only VMware-specific HA (vSphere HA) |
| RHV docs repeat HA theory | No — only RHV-specific HA (Hosted Engine HA) |
| VMware docs repeat migration theory | No — only VMware-specific (vMotion) |
| RHV docs repeat migration theory | No — only RHV-specific (Live Migration) |
| VMware docs repeat storage theory | No — only VMware-specific (VMFS/vSAN) |
| RHV docs repeat storage theory | No — only RHV-specific (Gluster/NFS) |
| Foundation does not mention esxcli | No esxcli commands in foundation |
| Foundation does not mention rhev-* | No rhev-* commands in foundation |

---

## T007: Cross-Reference Integrity

### Objective

Verify that cross-references between documents resolve correctly.

### Test Cases

| From | To | Check |
|------|---|-------|
| VMWARE_SKILL_PACK.md | docs/virtualization/VIRTUALIZATION_FOUNDATION.md | Reference present and valid |
| RHV_SKILL_PACK.md | docs/virtualization/VIRTUALIZATION_FOUNDATION.md | Reference present and valid |
| VMWARE_SKILL_PACK.md | VMWARE_WORKFLOWS.md | Reference present |
| VMWARE_SKILL_PACK.md | VMWARE_DETECTION.md | Reference present |
| VMWARE_SKILL_PACK.md | VMWARE_COMMAND_REFERENCE.md | Reference present |
| VMWARE_SKILL_PACK.md | VMWARE_GUIDANCE.md | Reference present |
| RHV_SKILL_PACK.md | RHV_WORKFLOWS.md | Reference present |
| RHV_SKILL_PACK.md | RHV_DETECTION.md | Reference present |
| RHV_SKILL_PACK.md | RHV_COMMAND_REFERENCE.md | Reference present |
| RHV_SKILL_PACK.md | RHV_GUIDANCE.md | Reference present |
| workflow docs | detection rules | Related workflows referenced |

---

## T008: Version Consistency

### Objective

Verify version tags are consistent across all documents.

### Test Cases

| Check | Expected |
|-------|----------|
| VMware foundation version | 1.0.0 or consistent with release |
| RHV foundation version | 1.0.0 or consistent with release |
| VMware YAML manifest version | Matches skill pack version |
| RHV YAML manifest version | Matches skill pack version |
| All version tables present | Every .md file has version table |
| Version dates consistent | All dated 2026-07-21 for initial release |

---

## Test Results Summary

| Test | Status | Notes |
|------|--------|-------|
| T001 | PASS | Foundation covers all 16 required topics |
| T002 | PASS | VMware has all required docs with correct content |
| T003 | PASS | RHV has all required docs with correct content |
| T004 | PASS | All VMware YAML files parse correctly |
| T005 | PASS | All RHV YAML files parse correctly |
| T006 | PASS | No foundation duplication — clean separation |
| T007 | PASS | All cross-references valid |
| T008 | PASS | Version consistency maintained |

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial test specification |