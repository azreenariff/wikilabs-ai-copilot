# OpenShift Engineering Skill Pack

## Overview

This skill pack provides comprehensive engineering knowledge, reasoning, and guidance for Red Hat OpenShift Container Platform 4.x and upstream Kubernetes.

## Purpose

Wiki Labs AI Copilot is an engineering advisor. It never executes work. It helps engineers think by recommending, explaining, and guiding — while the engineer performs every action.

## Coverage

- **Cluster Architecture**: Control plane, workers, operators, machine config
- **Workload Management**: Pods, deployments, statefulsets, daemonsets, jobs
- **Networking**: Services, routes, ingress, network policies, DNS, CNI
- **Storage**: PV, PVC, storage classes, CSI drivers
- **Security**: RBAC, SCCs, network policies, secrets, authentication
- **Operators**: Cluster operators, OLM, subscriptions, CSVs
- **Monitoring**: Prometheus, Grafana, logging, alerts
- **Troubleshooting**: Detection rules, workflows, reasoning guides
- **Commands**: 40+ oc commands with explanations, risks, and examples

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
openshift-skill-pack/
    manifest.yaml           # Skill pack metadata and configuration
    technology.yaml         # Technology coverage and features
    workflows.yaml          # Troubleshooting workflows
    detection_rules.yaml    # Context detection patterns
    commands.yaml           # Command knowledge base
    guidance/rules.md       # Engineering reasoning and guidance
    best-practices.md       # Best practices and standards
    known_issues.md         # Known failure patterns and workarounds
    knowledge/              # Deep knowledge base
        cluster-architecture.md
        security-rbac-scc.md
        networking-services-routes.md
        container-runtime.md
    documentation/          # Reference documentation
    examples/               # Worked examples
    tests/                  # Validation tests
    references/             # Documentation references
    architecture/           # Architecture details
    concepts/               # Concept explanations
    workflows/              # Workflow documentation
    reasoning/              # Reasoning guides
    detection/              # Detection documentation
    diagnostics/            # Diagnostic procedures
    context/                # Context interpretation
    commands/               # Command reference
    guidance/               # Guidance reference
    common-failures/        # Failure pattern database
    examples/               # Worked examples
    tests/                  # Test cases
```

## Version

Current version: 0.9.0-alpha

## Safety

This skill pack follows strict safety rules:
- Never execute commands — only recommend and explain
- Always warn about risks before recommending actions
- Always provide rollback strategies
- Always recommend evidence collection before diagnosis
- Always consider cascade effects of recommended actions

## References

- [Red Hat OpenShift Documentation](https://docs.openshift.com/container-platform/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [OpenShift Best Practices](https://docs.openshift.com/container-platform/latest/installing/index.html)
- [Red Hat OpenShift 4.x Release Notes](https://access.redhat.com/documentation/en-us/red_hat_openshift_container_platform/)