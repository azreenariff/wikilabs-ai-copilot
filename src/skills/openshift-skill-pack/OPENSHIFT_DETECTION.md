# OpenShift Detection Rules Reference

## Purpose

This document describes the detection rules used by the OpenShift skill pack to identify context, symptoms, and issues.

## Context Detection Rules

### CLI Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| openshift-detect-oc | OpenShift CLI usage | `^oc \w+` | 0.95 | 10 |
| openshift-detect-kubectl | Kubernetes CLI usage | `^kubectl \w+` | 0.85 | 9 |
| openshift-detect-adm | OpenShift admin commands | `oc adm (inspect\|troubleshoot\|diagnostics\|verify-image-registry\|upgrade\|quota)` | 0.90 | 9 |
| openshift-detect-project | Project/namespace context | `^oc project|oc project ` | 0.90 | 10 |

### Console Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| openshift-detect-console | OpenShift web console | `console\.openshift|openshift-web-console|openshift-master` | 0.90 | 9 |

### Resource Type Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| openshift-detect-pod | Pod operations | `oc\s+(get\|describe\|logs\|delete\|exec\|debug)\s+pod` | 0.85 | 8 |
| openshift-detect-deployment | Deployment operations | `oc\s+(get\|describe\|rollout)\s+deployment` | 0.85 | 8 |
| openshift-detect-route | Route operations | `oc\s+(get\|describe\|create\|delete)\s+route` | 0.90 | 8 |
| openshift-detect-operator | Operator operations | `oc\s+(get\|describe)\s+(cluster-)?operator|oc\s+(get\|describe)\s+(csv\|subscription)` | 0.90 | 9 |
| openshift-detect-node | Node operations | `oc\s+(get\|describe)\s+node|oc\s+adm\s+taint|oc\s+adm\s+drain` | 0.85 | 8 |
| openshift-detect-pvc | PVC operations | `oc\s+(get\|describe)\s+pvc|oc\s+(get\|describe)\s+pv|oc\s+(get\|describe)\s+storageclass` | 0.90 | 8 |
| openshift-detect-service | Service operations | `oc\s+(get\|describe)\s+svc|oc\s+(get\|describe)\s+service` | 0.85 | 8 |
| openshift-detect-rbac | RBAC operations | `oc\s+auth\s+can-i|oc\s+(create\|delete\|get)\s+(role\|rolebinding\|clusterrole\|clusterrolebinding)` | 0.85 | 8 |
| openshift-detect-configmap | ConfigMap operations | `oc\s+(get\|describe\|create\|delete)\s+configmap` | 0.85 | 7 |
| openshift-detect-secret | Secret operations | `oc\s+(get\|describe)\s+secret|oc\s+create\s+secret` | 0.85 | 7 |
| openshift-detect-imagestream | Image stream operations | `oc\s+(get\|describe\|tag)\s+imagestream` | 0.85 | 7 |

### Text Pattern Detection

| Rule ID | Description | Pattern | Confidence | Priority |
|---------|-------------|---------|------------|----------|
| openshift-detect-event-crash | CrashLoopBackOff | `CrashLoopBackOff` | 0.95 | 10 |
| openshift-detect-event-imagepullbackoff | ImagePullBackOff | `ImagePullBackOff|ErrImagePull` | 0.95 | 10 |
| openshift-detect-event-oomkilled | OOMKilled | `OOMKilled|out of memory` | 0.95 | 10 |
| openshift-detect-event-notready | Node NotReady | `NotReady` | 0.95 | 10 |
| openshift-detect-event-evicted | Evicted | `Evicted` | 0.90 | 9 |
| openshift-detect-event-pending | Pending | `(?i)pending|Pending` | 0.80 | 9 |
| openshift-detect-event-degraded | Operator Degraded | `(?i)degraded|Degraded` | 0.90 | 9 |
| openshift-detect-event-nodepressure | Node Pressure | `DiskPressure|MemoryPressure|PIDPressure|KernelDebt` | 0.90 | 9 |
| openshift-detect-event-failedscheduling | Failed Scheduling | `FailedScheduling` | 0.90 | 9 |
| openshift-detect-event-probefail | Probe Failure | `LivenessProbe|ReadinessProbe|StartupProbe` | 0.85 | 8 |
| openshift-detect-docker | Docker image reference | Image registry patterns | 0.70 | 6 |

## Confidence Scoring

### Confidence Levels

- **0.95+**: Near-certain match — strong pattern indicators
- **0.90-0.94**: High confidence — clear indicators
- **0.85-0.89**: Medium-high confidence — strong indicators
- **0.80-0.84**: Medium confidence — reasonable indicators
- **0.70-0.79**: Low-medium confidence — suggestive indicators
- **< 0.70**: Low confidence — weak indicators

### Confidence Adjustments

Context can increase confidence:
- CLI commands + text patterns about pods = higher pod confidence
- Multiple matching rules = higher overall context confidence
- Consistent namespace references = higher context accuracy

## Priority Levels

Priority determines which detection rules take precedence when multiple rules match:

- **10**: Critical — immediate attention required
- **9**: High — important context
- **8**: Medium — relevant context
- **7**: Low — supporting context
- **< 7**: Informational — supplementary context

## Pattern Extraction

Some rules extract specific values for use in diagnostics:

| Rule ID | Extract Pattern | Example |
|---------|----------------|---------|
| openshift-detect-project | `oc project (.+)` | `myproject` |
| openshift-detect-pod | `pod/(\S+)` | `myapp-5d4b7c8f9-x2k4p` |
| openshift-detect-deployment | `deployment/(\S+)` | `web-app` |
| openshift-detect-route | `route/(\S+)` | `web-app-route` |
| openshift-detect-node | `node/(\S+)` | `node01.example.com` |
| openshift-detect-pvc | `pvc/(\S+)` | `data-vol` |

## Rule Management

### Adding New Rules

When adding detection rules:
1. Follow the existing YAML format
2. Set appropriate confidence based on pattern specificity
3. Set priority based on operational importance
4. Include clear name and description
5. Test pattern against real OpenShift output

### Rule Ordering

Rules are processed in order of appearance. Higher priority rules should be placed first for faster matching.

## References

- [OpenShift Documentation](https://docs.openshift.com/container-platform/)
- [Kubernetes API Reference](https://kubernetes.io/docs/reference/)