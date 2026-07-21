# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0-alpha] — 2026-07-21

### Added

- **Virtualization Engineering Foundation** (`docs/virtualization/VIRTUALIZATION_FOUNDATION.md`)
  - Comprehensive shared engineering foundation for all virtualization skill packs
  - Covers: virtualization theory, compute architecture, VM lifecycle, storage architecture,
    networking, resource management, high availability, DRS, live migration, CPU/memory architecture,
    performance fundamentals, capacity planning, operational best practices, risk awareness,
    enterprise deployment profiles
  - Referenced by VMware and RHV skill packs to avoid concept duplication

- **Red Hat Virtualization (RHV) Skill Pack** (`src/skills/rhv-skill-pack/`)
  - Complete skill pack for RHV 4.4, 4.5, 4.6
  - `RHV_SKILL_PACK.md` — Skill pack overview, architecture, components, reference mapping
  - `RHV_WORKFLOWS.md` — 10 troubleshooting workflows (Engine, Host, Storage, VM perf, HA, etc.)
  - `RHV_DETECTION.md` — 12 detection rules with confidence scoring and pattern matching
  - `RHV_COMMAND_REFERENCE.md` — 50+ CLI commands (rhev-*, vdsm-tool, virsh, hosted-engine, gluster)
  - `RHV_GUIDANCE.md` — Operations, best practices, performance tuning, capacity planning, decision frameworks
  - `manifest.yaml` — Skill pack manifest with platform, components, storage types, networking, HA
  - `technology.yaml` — Technology profile (architecture, storage, virtualization, security, scaling)
  - `concepts/overview.md` — Architecture diagram, key components, VMware comparison
  - `concepts/terminology.md` — Glossary with VMware equivalents
  - `workflows.yaml` — State machine workflow definitions
  - `detection_rules.yaml` — Context detection rules with triggers and confidence
  - `tests/reference.md` — 5 validation tests for the RHV skill pack

- **VMware vSphere Consolidated Documentation** (`src/skills/vmware-skill-pack/`)
  - `VMWARE_SKILL_PACK.md` — Consolidated skill pack overview
  - `VMWARE_WORKFLOWS.md` — 12 consolidated troubleshooting workflows
  - `VMWARE_DETECTION.md` — Consolidated detection rules catalog
  - `VMWARE_GUIDANCE.md` — Consolidated operations guidance and best practices
  - `VMWARE_COMMAND_REFERENCE.md` — Consolidated command reference (esxcli, PowerCLI, service-control)

- **Shared Test Reference** (`src/skills/vmware-skill-pack/tests/reference.md`)
  - 8 validation tests covering foundation completeness, YAML validation,
    cross-reference integrity, version consistency, and no-duplication checks

### Changed

- Updated ROADMAP.md to reflect 1.2.0-alpha status with virtualization skill packs complete
- All documentation now references the shared Virtualization Engineering Foundation
- Consistent format across VMware and RHV: purpose → architecture → detection → workflows → commands → guidance

### Fixed

- Eliminated concept duplication between foundation and technology-specific docs
- Standardized cross-reference pattern across all skill packs

---

## [1.1.0-alpha] — 2026-07-15

### Added

- VMware vSphere skill pack (initial)
- OpenShift skill pack consolidation
- Linux engineering skill pack enhancements

---

## [1.0.0-alpha] — 2026-06-01

### Added

- Initial project structure and skill pack framework
- Linux engineering skill pack
- Basic observability and knowledge system

---

[Unreleased]

### Added

- (planned features)

[1.2.0-alpha]: https://github.com/wikilabs-ai/wikilabs-ai-copilot/compare/v1.1.0-alpha...v1.2.0-alpha
[1.1.0-alpha]: https://github.com/wikilabs-ai/wikilabs-ai-copilot/compare/v1.0.0-alpha...v1.1.0-alpha
[1.0.0-alpha]: https://github.com/wikilabs-ai/wikilabs-ai-copilot/releases/tag/v1.0.0-alpha