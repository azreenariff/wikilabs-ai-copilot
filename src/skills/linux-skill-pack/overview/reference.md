# Linux Engineering — Overview Reference

## Purpose

This directory provides an overview of the Linux engineering skill pack and its components.

## Skill Pack Summary

| Attribute | Value |
|-----------|-------|
| ID | linux-engineering-skill-pack |
| Version | 1.0.0 |
| Domain | Linux System Engineering |
| Dependencies | Linux, Networking, Storage, Security Foundations |
| Workflows | 9 troubleshooting workflows |
| Commands | 25+ reference commands |
| Knowledge Areas | 5 knowledge documents |
| Detection Rules | 10 rules with confidence scoring |

## Directory Structure

```
linux-skill-pack/
├── manifest.yaml              # Skill pack metadata
├── technology.yaml            # Technology definitions
├── workflows.yaml             # State machine workflows
├── detection_rules.yaml       # Context detection rules
├── commands.yaml              # Command reference
├── concepts/
│   ├── overview.md            # Core concepts overview
│   └── terminology.md         # Glossary of terms
├── context/
│   └── interpretation.md      # How to interpret context
├── diagnostics/
│   └── reference.md           # Diagnostics methodology
├── documentation/
│   └── reference.md           # Documentation standards
├── examples/
│   ├── reference.md           # Example categories
│   └── worked-examples.md     # Detailed scenarios
├── knowledge/
│   ├── system-architecture.md # Kernel, init, processes
│   ├── service-management.md  # systemd, units, logging
│   ├── security-hardening.md  # Auth, ACLs, firewalls
│   ├── network-configuration.md # Interfaces, routing, DNS
│   └── storage-management.md  # Disks, filesystems, LVM
├── overview/
│   └── reference.md           # This file
├── reasoning/
│   └── reference.md           # Diagnostic reasoning framework
├── references/
│   └── reference.md           # External documentation links
├── tests/
│   └── reference.md           # Test specifications
└── workflows/
    └── README.md              # Workflow catalog
```

## Quick Start

1. **Identify the issue** — user describes a problem
2. **Match detection rules** — find relevant rule(s)
3. **Select workflow** — choose appropriate workflow
4. **Collect evidence** — run diagnostic commands
5. **Diagnose** — analyze evidence
6. **Remediate** — apply fix
7. **Verify** — confirm resolution

## Supported Distributions

- **RHEL-family**: RHEL 8/9, Rocky 8/9, Alma 8/9, CentOS Stream 8/9
- **Debian-family**: Ubuntu 20.04/22.04/24.04, Debian 11/12
- **Other**: Alpine 3.18/3.19, Amazon Linux 2/2023, SLES 15, Oracle Linux 8/9

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial skill pack overview |