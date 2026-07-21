# Checkmk Engineering Skill Pack

## Overview

This skill pack provides comprehensive engineering knowledge, reasoning, and guidance for Checkmk monitoring platform versions 2.2 and above.

## Purpose

Wiki Labs AI Copilot is an engineering advisor. It never executes work. It helps engineers think by recommending, explaining, and guiding — while the engineer performs every action.

## Coverage

- **Agent Management**: check_mk_agent, plugin development, remote and local checks, agent health
- **Micro Core (CMC)**: High-performance check engine, parallelization, status management
- **Nagios Core**: Legacy check engine, contact management, time periods, host/service definitions
- **Piggyback Data**: Cluster monitoring, external data processing, K8s/VMware/SAP integration
- **Service Discovery**: Automatic service enumeration, fix-all, remove, scan, pattern matching
- **Rulesets**: Check parameters, service rules, notification rules, hierarchical organization
- **WATO/Setup**: Web-based host management, rulesets, notifications, site configuration
- **Notifications**: Email, SMS, webhooks, Slack, Teams, escalation, dampening, contact routing
- **Performance Views**: Metric visualization, trending, custom metrics, CSV/PNG export
- **SNMP Monitoring**: SNMP v1/v2c/v3, MIB browser, trap reception, walk/get/getnext
- **Check Plugin Development**: Python check API v2, Shell plugins, check_result format
- **Cluster Monitoring**: Kubernetes, VMWare, SAP HANA, active-active database clusters
- **Troubleshooting**: Detection rules, workflows, reasoning guides for 8+ failure patterns

## Quality Standards

This skill pack implements the reference standard for all future technology skill packs. It is evaluated against:

- Knowledge coverage
- Workflow coverage
- Reasoning coverage
- Detection coverage
- Guidance quality
- Safety
- Documentation
- Examples
- Testing
- Maintainability

## Structure

```
checkmk-skill-pack/
    manifest.yaml           # Skill pack metadata and configuration
    technology.yaml         # Technology coverage and features
    workflows.yaml          # Troubleshooting workflows
    detection_rules.yaml    # Context detection patterns
    commands.yaml           # Command knowledge base
    CHECKMK_SKILL_PACK.md   # Skill pack overview (this file)
    CHECKMK_DETECTION.md    # Detection rules reference
    CHECKMK_COMMAND_REFERENCE.md  # Command reference
    CHECKMK_GUIDANCE.md     # Engineering guidance
    CHECKMK_BEST_PRACTICES.md   # Best practices and standards
    CHECKMK_COMMON_FAILURES.md  # Known failure patterns
    CHECKMK_REASONING_GUIDE.md  # Diagnostic reasoning
    CHECKMK_WORKFLOWS.md    # State-machine workflows
    guidance/rules.md       # Engineering reasoning and guidance
    best-practices/reference.md  # Best practices reference
    documentation/reference.md # Reference documentation
    examples/worked-examples.md  # Worked examples
    examples/reference.md           # Examples reference
    tests/reference.md              # Validation tests
    references/reference.md # External references
    architecture/                 # Architecture details (planned)
    concepts/                     # Concept explanations
        overview.md               # Architecture and key components
        terminology.md            # Glossary
    workflows/                  # Workflow documentation (planned)
    reasoning/                  # Reasoning guides
        reference.md            # Reasoning reference
    detection/                  # Detection documentation
        reference.md            # Detection documentation
    diagnostics/                # Diagnostic procedures
        guide.md                # Diagnostic guide
    context/                    # Context interpretation
        interpretation.md       # Context interpretation
    commands/                   # Command reference (planned)
    common-failures/            # Failure pattern database
        reference.md            # Failure patterns reference
```

## Version

Current version: 1.0.0

## Supported Environments

- Checkmk Raw Edition (CRE) 2.2+
- Checkmk Enterprise Edition (CEE) 2.2+
- Checkmk Managed Edition (CME) 2.2+

## Safety

This skill pack follows strict safety rules:
- Never execute commands — only recommend and explain
- Always warn about risks before recommending actions
- Always provide rollback strategies
- Always recommend evidence collection before diagnosis
- Always consider cascade effects of recommended actions
- Never modify thresholds or configuration without explicit engineer approval
- Always recommend config validation before reload

## References

- [Checkmk Documentation](https://docs.checkmk.com/)
- [Checkmk Live Community](https://checkmk.com/cms_live)
- [Checkmk Agent Plugin Repository](https://exchange.checkmk.com/)
- [Checkmk Enterprise Documentation](https://docs.checkmk.com/master/en/enterprise.html)
- [Checkmk Raw Edition Documentation](https://docs.checkmk.com/master/en/raw.html)