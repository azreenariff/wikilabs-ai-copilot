# OpenShift Troubleshooting Guide

## Overview

This guide provides structured troubleshooting procedures for OpenShift cluster issues.

## Troubleshooting Methodology

### Step 1: Identify the Symptom
- Check pod status: `oc get pods -n <namespace>`
- Check node status: `oc get nodes`
- Check operator status: `oc get co`
- Check recent events: `oc get events -n <namespace> --sort-by='.lastTimestamp'`

### Step 2: Gather Evidence
- Describe the affected resource
- Check logs (current and previous)
- Review events
- Check resource configuration

### Step 3: Determine Root Cause
- Match symptoms to known patterns
- Follow decision trees
- Consider cascade effects

### Step 4: Apply Fix
- Follow documented procedures
- Verify each step
- Monitor for resolution

### Step 5: Document Findings
- Record what was checked
- Document the root cause
- Record the fix applied
- Update this guide if needed

## Quick Diagnostic Commands

```bash
# Cluster health
oc get co
oc get nodes

# Pod health
oc get pods -n <namespace> -o wide
oc get events -n <namespace> --sort-by='.lastTimestamp'

# Deployment health
oc get deployments -n <namespace>
oc rollout status deployment/<name> -n <namespace>

# Node health
oc describe node <node-name>

# Resource usage
oc top pods -n <namespace>
oc top nodes

# Storage health
oc get pvc -n <namespace>
oc get storageclass
```

## Evidence Collection Template

For each issue, collect:

1. **Pod/Node events**: `oc get events -n <namespace> --sort-by='.lastTimestamp'`
2. **Resource description**: `oc describe <resource> <name> -n <namespace>`
3. **Logs**: `oc logs <pod> -n <namespace>` and `--previous`
4. **Resource configuration**: `oc get <resource> <name> -n <namespace> -o yaml`
5. **Resource usage**: `oc top <resource> -n <namespace>`

## Support Escalation

If the issue cannot be resolved with documented procedures:

1. Collect all evidence using the template above
2. Document steps already taken
3. Include cluster version: `oc version`
4. Include operator status: `oc get co`
5. Submit to Red Hat support with evidence

## References

- [Red Hat OpenShift Troubleshooting](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)
- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)