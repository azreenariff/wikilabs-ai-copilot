# VMware vSphere Engineering — Overview Reference

## Purpose

This directory provides an overview of the VMware vSphere engineering skill pack and its components.

## Skill Pack Summary

| Attribute | Value |
|-----------|-------|
| ID | vmware-vsphere-engineering-skill-pack |
| Name | VMware vSphere Engineering |
| Version | 1.0.0 |
| Platform | VMware vSphere 7.0, 8.0 |
| Components | vCenter Server, ESXi, VMs, vSAN |
| Workflows | 12+ troubleshooting workflows |
| Commands | 60+ esxcli and PowerCLI references |

## Directory Structure

| Directory | Content |
|-----------|---------|
| `./` | Core YAML configuration files |
| `concepts/` | Architecture and terminology |
| `context/` | Context interpretation and matching |
| `diagnostics/` | Diagnostic methodology |
| `documentation/` | Documentation standards |
| `examples/` | Worked troubleshooting scenarios |
| `knowledge/` | Detailed domain knowledge |
| `best-practices/` | Operational best practices |
| `common-failures/` | Common failure modes |
| `detection/` | Detection rules |
| `overview/` | This overview |
| `reasoning/` | Diagnostic reasoning framework |
| `references/` | External references and links |
| `tests/` | Test specifications |
| `workflows/` | Workflow definitions |

## Core Components

### 1. YAML Configuration
- **manifest.yaml** — Skill pack metadata
- **technology.yaml** — Supported technologies
- **workflows.yaml** — Troubleshooting workflows
- **detection_rules.yaml** — Context detection rules
- **commands.yaml** — CLI reference commands

### 2. Documentation
- **concepts/** — Architecture, terminology
- **context/** — Context interpretation
- **diagnostics/** — Diagnostic methodology
- **documentation/** — Standards and procedures
- **examples/** — Worked scenarios
- **knowledge/** — Domain expertise
- **best-practices/** — Operational standards
- **common-failures/** — Failure catalog
- **detection/** — Detection rules
- **reasoning/** — Diagnostic reasoning
- **references/** — External resources
- **tests/** — Validation criteria
- **workflows/** — Workflow state machines

## Supported Platforms

| Platform | Version | Type |
|----------|---------|------|
| vCenter Server | 7.0 U3+, 8.0 | Management |
| ESXi | 7.0, 8.0 | Hypervisor |
| vSAN | 7.0, 8.0 | Storage |
| vSphere Client | HTML5 | Interface |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial overview |