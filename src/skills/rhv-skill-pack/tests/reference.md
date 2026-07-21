# RHV Engineering — Test Reference

## Purpose

This document defines the validation test specifications for the Red Hat Virtualization engineering skill pack.

## Test Suite Overview

| Test ID | Name | Target | Type | Status |
|---------|------|--------|------|--------|
| `RT001` | RHV Foundation Completeness | RHV docs | Structural | Pass |
| `RT002` | RHV YAML Validation | manifest.yaml, technology.yaml, workflows.yaml, detection_rules.yaml | Syntax | Pass |
| `RT003` | No Foundation Duplication | RHV docs vs Virtualization Foundation | Semantic | Pass |
| `RT004` | Cross-Reference Integrity | All RHV docs reference each other | Structural | Pass |
| `RT005` | Version Consistency | All RHV docs version tags | Semantic | Pass |

---

## RT001: RHV Foundation Completeness

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
| RT001.1 | RHV_SKILL_PACK.md | Contains overview, architecture, components, reference mapping |
| RT001.2 | RHV_WORKFLOWS.md | Contains 10+ workflows with evidence → diagnosis → remediation → verification |
| RT001.3 | detection/reference.md | Contains detection rule catalog, patterns, confidence scoring |
| RT001.4 | RHV_GUIDANCE.md | Contains operations, best practices, tuning, planning, decision frameworks |
| RT001.5 | RHV_COMMAND_REFERENCE.md | Contains rhev-*, vdsm-tool, virsh, hosted-engine, gluster entries |
| RT001.6 | concepts/overview.md | Contains architecture diagram, key components, platform differences |
| RT001.7 | concepts/terminology.md | Contains glossary, RHV-specific terms, VMware equivalents |
| RT001.8 | Cross-references to foundation | References `docs/virtualization/VIRTUALIZATION_FOUNDATION.md` |
| RT001.9 | Version table | Present in all docs |
| RT001.10 | No foundation duplication | No repeated virtualization theory (architecture, HA, migration, etc.) |

---

## RT002: RHV YAML Validation

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

## RT003: No Foundation Duplication

### Objective

Verify that RHV docs don't duplicate foundation virtualization concepts.

### Test Cases

| Check | Expected Result |
|-------|-----------------|
| Foundation defines virtualization theory | Yes — in VIRTUALIZATION_FOUNDATION.md |
| RHV docs reference foundation | Yes — cross-links present |
| RHV docs repeat HA theory | No — only RHV-specific HA (Hosted Engine HA) |
| RHV docs repeat migration theory | No — only RHV-specific (Live Migration via VDSM) |
| RHV docs repeat storage theory | No — only RHV-specific (Gluster/NFS) |
| Foundation does not mention rhev-* | No rhev-* commands in foundation |
| RHV concepts reference foundation | Yes — architecture doc references foundation |

---

## RT004: Cross-Reference Integrity

### Objective

Verify that cross-references between RHV documents resolve correctly.

### Test Cases

| From | To | Check |
|------|---|-------|
| RHV_SKILL_PACK.md | docs/virtualization/VIRTUALIZATION_FOUNDATION.md | Reference present and valid |
| RHV_SKILL_PACK.md | RHV_WORKFLOWS.md | Reference present |
| RHV_SKILL_PACK.md | RHV_DETECTION.md | Reference present |
| RHV_SKILL_PACK.md | RHV_COMMAND_REFERENCE.md | Reference present |
| RHV_SKILL_PACK.md | RHV_GUIDANCE.md | Reference present |
| workflow docs | detection rules | Related workflows referenced |
| technology.yaml | workflow YAML files | Consistent workflow IDs |
| detection_rules.yaml | workflow YAML files | Related workflow IDs match |

---

## RT005: Version Consistency

### Objective

Verify version tags are consistent across all RHV documents.

### Test Cases

| Check | Expected |
|-------|----------|
| RHV foundation version | 1.0.0 or consistent with release |
| RHV YAML manifest version | Matches skill pack version |
| All version tables present | Every .md file has version table |
| Version dates consistent | All dated 2026-07-21 for initial release |

---

## Test Results Summary

| Test | Status | Notes |
|------|--------|-------|
| RT001 | PASS | RHV has all required docs with correct content |
| RT002 | PASS | All RHV YAML files parse correctly |
| RT003 | PASS | No foundation duplication — clean separation |
| RT004 | PASS | All cross-references valid |
| RT005 | PASS | Version consistency maintained |

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV test specification |