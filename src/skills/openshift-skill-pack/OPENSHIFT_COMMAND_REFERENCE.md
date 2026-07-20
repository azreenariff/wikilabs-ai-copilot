# OpenShift Command Reference

## Purpose

This document provides a structured reference for all OpenShift CLI (oc) and Kubernetes CLI (kubectl) commands covered in the OpenShift skill pack.

## Command Categories

### Pod Management

| Command | Category | Risk |
|---------|----------|------|
| `oc get pods` | Status | Low |
| `oc describe pod` | Diagnostic | Low |
| `oc logs` | Diagnostic | Low |
| `oc exec` | Interactive | Medium |
| `oc debug pod` | Diagnostic | Low |
| `oc delete pod` | Action | Medium |
| `oc port-forward` | Network | Low |

### Deployment Management

| Command | Category | Risk |
|---------|----------|------|
| `oc get deployments` | Status | Low |
| `oc describe deployment` | Diagnostic | Low |
| `oc rollout status` | Status | Low |
| `oc rollout undo` | Action | Medium |
| `oc rollout restart` | Action | Medium |
| `oc set image` | Action | Medium |
| `oc set env` | Action | Medium |
| `oc set resources` | Action | Medium |
| `oc scale` | Action | Medium |

### Node Management

| Command | Category | Risk |
|---------|----------|------|
| `oc get nodes` | Status | Low |
| `oc describe node` | Diagnostic | Low |
| `oc adm taint` | Action | Medium |
| `oc adm drain` | Action | High |
| `oc debug node` | Diagnostic | Low |

### Cluster Management

| Command | Category | Risk |
|---------|----------|------|
| `oc get co` | Status | Low |
| `oc describe clusteropeartor` | Diagnostic | Low |
| `oc get clusterversion` | Status | Low |
| `oc adm upgrade` | Action | Critical |
| `oc adm repair` | Action | Low |
| `oc adm verify-image-registry` | Diagnostic | Low |

### Networking

| Command | Category | Risk |
|---------|----------|------|
| `oc get routes` | Status | Low |
| `oc describe route` | Diagnostic | Low |
| `oc get services` | Status | Low |
| `oc describe service` | Diagnostic | Low |
| `oc get endpoints` | Status | Low |
| `oc get networkpolicies` | Status | Low |

### Storage

| Command | Category | Risk |
|---------|----------|------|
| `oc get pvc` | Status | Low |
| `oc describe pvc` | Diagnostic | Low |
| `oc get pv` | Status | Low |
| `oc get storageclass` | Status | Low |
| `oc describe storageclass` | Diagnostic | Low |

### Security

| Command | Category | Risk |
|---------|----------|------|
| `oc auth can-i` | Diagnostic | Low |
| `oc whoami` | Diagnostic | Low |
| `oc login` | Authentication | Medium |
| `oc adm policy add-role-to-user` | Action | Medium |
| `oc adm policy add-scc-to-user` | Action | Medium |
| `oc create serviceaccount` | Action | Low |

### Image Management

| Command | Category | Risk |
|---------|----------|------|
| `oc get imagestreams` | Status | Low |
| `oc describe is` | Diagnostic | Low |
| `oc tag` | Action | Medium |
| `oc registry info` | Diagnostic | Low |

### Diagnostics

| Command | Category | Risk |
|---------|----------|------|
| `oc adm inspect` | Diagnostic | Low |
| `oc adm troubleshoot` | Diagnostic | Low |
| `oc adm diagnostics` | Diagnostic | Low |
| `oc adm debug` | Diagnostic | Low |

## Command Syntax Reference

### Common Flags

| Flag | Description |
|------|-------------|
| `-n, --namespace` | Specify namespace |
| `-o, --output` | Output format (yaml, json, wide, name) |
| `-l, --selector` | Label selector |
| `-f, --filename` | Read from file |
| `-a, --all-namespaces` | Operate on all namespaces |
| `-w, --watch` | Watch for changes |
| `-v, --verbose` | Verbose output |
| `--sort-by` | Sort output by field |
| `--show-labels` | Show labels |
| `--show-managed-fields` | Show managed fields |
| `--output=jsonpath=...` | Use jsonpath output format |
| `--output=yaml` | Use yaml output format |

### Common Output Formats

| Format | Use Case |
|--------|----------|
| `wide` | Human-readable with extra columns |
| `json` | Machine-readable, full details |
| `yaml` | Configuration files, full details |
| `name` | Resource names only |
| `custom-columns` | Custom output fields |
| `jsonpath=...` | Extract specific fields |

## Frequently Used Commands

### Quick Diagnostics

```bash
# Check overall cluster health
oc get co

# Check node status
oc get nodes -o wide

# Check pod status in namespace
oc get pods -n <namespace> -o wide

# Check recent events
oc get events -n <namespace> --sort-by='.lastTimestamp'

# Check deployment status
oc rollout status deployment/<name> -n <namespace>
```

### Resource Inspection

```bash
# Detailed pod information
oc describe pod <name> -n <namespace>

# Pod logs
oc logs <pod> -n <namespace>
oc logs <pod> -n <namespace> --previous

# Resource usage
oc top pods -n <namespace>
oc top nodes

# Resource configuration
oc get pod <name> -n <namespace> -o yaml | grep -A 5 "resources:"
```

### Debugging

```bash
# Debug session for pod
oc debug pod/<name> -n <namespace>

# Exec into pod
oc exec -it <pod> -n <namespace> -- /bin/bash

# Inspect pod for diagnostics
oc adm inspect pod/<name> -n <namespace> --dest-dir=/tmp/inspect

# Network debugging
oc debug pod/<name> -n <namespace> -- curl http://<service>:<port>
```

## Risk Levels

### Low Risk
- Status checks (get, describe)
- Diagnostic collection (logs, inspect)
- Information gathering (top, whoami)

### Medium Risk
- Configuration changes (set image, set env, set resources)
- Pod management (delete, restart)
- Role and permission changes
- Image tag updates

### High Risk
- Node draining (evicts pods)
- Cluster operations (upgrade)
- SCC changes
- Network policy changes

### Critical Risk
- Cluster upgrades
- etcd operations
- Control plane modifications
- Machine configuration changes

## References

- [OpenShift CLI Documentation](https://docs.openshift.com/container-platform/latest/cli_reference/openshift_cli/index.html)
- [Kubernetes CLI Documentation](https://kubernetes.io/docs/reference/kubectl/)
- [oc CLI Reference](https://docs.openshift.com/container-platform/latest/cli_reference/openshift_cli/understanding-oc-cli.html)