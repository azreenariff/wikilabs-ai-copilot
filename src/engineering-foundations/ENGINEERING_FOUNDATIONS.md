# Engineering Foundations Framework

## Overview

Engineering Foundations are reusable knowledge modules that capture the fundamental principles, architecture, and troubleshooting philosophy of core infrastructure technologies. They serve as the common knowledge layer shared across all technology-specific Skill Packs.

## Purpose

- **Avoid duplication:** Foundation knowledge is written once and referenced by all Skill Packs
- **Ensure consistency:** Common troubleshooting patterns, safety rules, and best practices are standardized
- **Enable cross-foundation reasoning:** The copilot understands how technologies interconnect
- **Simplify Skill Pack development:** New Skill Packs inherit foundation knowledge instead of reinventing it

## Structure

Each Engineering Foundation follows this structure:

| Section | Content |
|---------|---------|
| **Architecture** | How the technology works, core components, design principles |
| **Core Concepts** | Fundamental terminology and models |
| **Common Components** | Standard tools, utilities, and services |
| **Common Failures** | Typical failure modes and symptoms |
| **Troubleshooting Philosophy** | Step-by-step diagnostic approach |
| **Best Practices** | Operational recommendations |
| **Risk Awareness** | Safety considerations and warning patterns |
| **Decision Trees** | Conditional diagnostic logic |
| **References** | External documentation links |
| **Examples** | Worked scenarios |

## Integration Model

Skill Packs reference foundations instead of duplicating content:

```
Linux Skill Pack ──references──> Linux Foundation
Windows Skill Pack ──references──> Windows Foundation
OpenShift Skill Pack ──references──> Linux + Networking + Storage + Security Foundations
VMware Skill Pack ──references──> Linux + Networking + Storage Foundations
```

## Quality Requirements

Every Foundation must include:

- [ ] Architecture description with key components
- [ ] Core concepts and terminology
- [ ] Common components list
- [ ] Common failure modes
- [ ] Troubleshooting philosophy (methodology)
- [ ] Best practices (operational recommendations)
- [ ] Risk awareness (safety warnings)
- [ ] Decision trees (conditional logic)
- [ ] References (external documentation)
- [ ] Examples (worked scenarios)

## Foundation Index

| Foundation | Domain | Dependencies |
|------------|--------|--------------|
| Linux Engineering Foundation | Operating Systems | None |
| Windows Engineering Foundation | Operating Systems | Linux Foundation (concepts) |
| Networking Foundation | Infrastructure | None |
| Storage Foundation | Infrastructure | None |
| Security Foundation | Infrastructure | None |

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial framework, 5 foundations, 2 skill packs |