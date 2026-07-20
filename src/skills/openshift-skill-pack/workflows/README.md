# OpenShift Troubleshooting Workflows Reference

## Purpose

This directory contains detailed troubleshooting workflow documentation for OpenShift issues.

## Available Workflows

See the main documentation files for complete workflow documentation:

- **OPENSHIFT_WORKFLOWS.md** — Complete troubleshooting workflows with evidence collection, decision trees, commands, and risk assessment

### Workflow Index

| Workflow | Document | Priority |
|----------|----------|----------|
| Pod CrashLoopBackOff | OPENSHIFT_WORKFLOWS.md | High |
| Pod Pending / FailedScheduling | OPENSHIFT_WORKFLOWS.md | Medium |
| Container OOMKilled | OPENSHIFT_WORKFLOWS.md | High |
| ImagePullBackOff | OPENSHIFT_WORKFLOWS.md | High |
| Node NotReady | OPENSHIFT_WORKFLOWS.md | Critical |
| Deployment Failure | OPENSHIFT_WORKFLOWS.md | Medium |
| Operator Degraded | OPENSHIFT_WORKFLOWS.md | High |
| Route Unavailable | OPENSHIFT_WORKFLOWS.md | Medium |
| PVC Pending | OPENSHIFT_WORKFLOWS.md | Medium |
| Authentication Failure | OPENSHIFT_WORKFLOWS.md | High |

## Workflow Structure

Each workflow follows this structure:
1. Symptom description
2. Evidence collection commands
3. Decision tree for diagnosis
4. Recommended commands with explanations
5. Risk assessment
6. Expected outcomes
7. Documentation references

## References

- [OpenShift Troubleshooting](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)
- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)