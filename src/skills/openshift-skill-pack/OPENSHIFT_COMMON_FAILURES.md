# OpenShift Common Failure Patterns

## Purpose

This document documents common failure patterns, their causes, detection methods, and recommended actions.

## 1. CrashLoopBackOff

**Severity**: High
**Frequency**: Very common

### Symptoms
- Pod status: CrashLoopBackOff
- Restart count increasing
- Pod cycling between Running and Waiting

### Root Causes
- Application missing required configuration
- Application dependency failure (database, cache)
- Memory limit exceeded (OOMKilled)
- Liveness probe killing container
- Wrong image or entrypoint
- Init container failure

### Detection
```bash
oc get pods -n <namespace> | grep CrashLoopBackOff
oc describe pod <pod> -n <namespace>
oc logs <pod> -n <namespace> --previous
```

### Recommended Actions
1. Check previous logs for crash reason
2. Review events for OOMKill or probe failure
3. Verify image configuration and entrypoint
4. Check resource limits and application memory usage
5. Verify dependent services are available

## 2. OOMKilled

**Severity**: High
**Frequency**: Very common

### Symptoms
- Container terminated with reason: OOMKilled
- Pod restart count increasing
- Memory usage near limit

### Root Causes
- Memory limit too low for application
- Memory leak in application code
- Java application without proper heap settings
- Multiple containers competing for memory

### Detection
```bash
oc describe pod <pod> -n <namespace> | grep OOMKilled
oc get events -n <namespace> --field-selector reason=OOMKilled
oc top pod <pod> -n <namespace> --containers
```

### Recommended Actions
1. Increase memory limits
2. For Java apps, set JAVA_OPTS with -Xmx
3. Monitor memory usage after limit increase
4. If OOMKilled persists, investigate memory leaks

## 3. ImagePullBackOff

**Severity**: High
**Frequency**: Common

### Symptoms
- Pod status: ImagePullBackOff or ErrImagePull
- Pod stuck in ContainerCreating
- Pull error in events

### Root Causes
- Wrong image name or tag
- Image does not exist in registry
- Missing or expired image pull secrets
- Registry network connectivity issues

### Detection
```bash
oc describe pod <pod> -n <namespace> | grep "ImagePullBackOff\|ErrImagePull"
oc get events -n <namespace> --sort-by='.lastTimestamp'
oc get secrets -n <namespace> | grep "image-pull"
```

### Recommended Actions
1. Verify image reference and tag
2. Check pull secrets for namespace
3. Test registry accessibility
4. Update image in deployment if needed

## 4. Node NotReady

**Severity**: Critical
**Frequency**: Common

### Symptoms
- Node status: NotReady
- Pods on node may be evicted or stuck
- Node conditions show Pressure or other issues

### Root Causes
- Kubelet process stopped
- Disk pressure
- Memory pressure
- PID pressure
- Network partition
- Hardware failure

### Detection
```bash
oc get nodes
oc describe node <node>
oc debug node <node> -- chroot /host journalctl -u kubelet
```

### Recommended Actions
1. Check node conditions for specific pressure type
2. Access node for deeper investigation
3. Restart kubelet if needed
4. Clean disk or memory if pressure detected
5. Drain node and replace if hardware failure

## 5. PVC Pending

**Severity**: Medium
**Frequency**: Common

### Symptoms
- PVC status: Pending
- Pods using PVC stuck in Pending
- No associated PV

### Root Causes
- No matching storage class
- Storage class provisioner not configured
- Insufficient storage capacity
- Access mode mismatch

### Detection
```bash
oc get pvc -n <namespace>
oc describe pvc <pvc> -n <namespace>
oc get storageclass
```

### Recommended Actions
1. Verify storage class availability
2. Check provisioner configuration
3. Adjust access mode if mismatch
4. Set default storage class if needed

## 6. FailedScheduling

**Severity**: Medium
**Frequency**: Common

### Symptoms
- Pod status: Pending
- FailedScheduling in events
- No node can satisfy scheduling requirements

### Root Causes
- Insufficient cluster resources
- Node taints without matching tolerations
- Affinity rules cannot be satisfied
- PVC not available

### Detection
```bash
oc get events -n <namespace> --field-selector reason=FailedScheduling
oc describe pod <pod> -n <namespace>
oc get nodes -o wide
```

### Recommended Actions
1. Check FailedScheduling event message
2. Verify node resource capacity
3. Check taints and tolerations
4. Adjust resource requests if over-provisioned
5. Scale cluster if capacity exceeded

## 7. Operator Degraded

**Severity**: High
**Frequency**: Common

### Symptoms
- Operator status in get co: Degraded=True
- Operator pods not running or crashing
- Component not functioning

### Root Causes
- Operator pod failures
- Missing RBAC permissions
- Configuration errors
- Dependent operator degraded
- Resource constraints

### Detection
```bash
oc get co
oc describe clusteropeartor <operator>
oc get pods -n openshift-<component>-operator
```

### Recommended Actions
1. Check operator pod status and logs
2. Fix RBAC issues if present
3. Correct operator configuration
4. Fix dependent operators first
5. Restart operator pods if stuck

## 8. Route Unavailable

**Severity**: Medium
**Frequency**: Common

### Symptoms
- Route not accessible
- 503 or timeout errors
- Empty service endpoints

### Root Causes
- Service has no endpoint pods
- Route misconfiguration
- TLS certificate issues
- Network policies blocking traffic

### Detection
```bash
oc get routes -n <namespace>
oc describe route <route> -n <namespace>
oc get endpoints <service> -n <namespace>
```

### Recommended Actions
1. Verify service endpoints are healthy
2. Check route target configuration
3. Fix TLS certificates if needed
4. Review network policies

## 9. Authentication Failure

**Severity**: High
**Frequency**: Common

### Symptoms
- oc login fails
- API calls return 403 Forbidden
- Service account cannot access resources

### Root Causes
- Expired authentication token
- Missing RBAC role bindings
- Service account misconfiguration
- OAuth provider issues

### Detection
```bash
oc whoami
oc auth can-i <verb> <resource> --as=<user>
oc adm policy can-i --list --as=<user>
```

### Recommended Actions
1. Refresh authentication token
2. Add missing role bindings
3. Verify service account configuration
4. Check OAuth provider status

## 10. DNS Resolution Failure

**Severity**: Medium
**Frequency**: Less common

### Symptoms
- Pod cannot resolve service DNS names
- Applications fail to connect to other services
- nslookup fails in debug pod

### Root Causes
- CoreDNS pods not running
- DNS configuration errors
- Network policies blocking DNS
- DNS forwarder issues

### Detection
```bash
oc get pods -n openshift-dns
oc debug pod/<pod> -- nslookup kubernetes.default
oc get configmap dns-default -n openshift-dns
```

### Recommended Actions
1. Check CoreDNS pod status
2. Review DNS configuration
3. Verify network policies allow DNS (port 53)
4. Check DNS forwarder configuration

## Failure Pattern Matrix

| Pattern | Likelihood | Impact | Detection Time | Fix Difficulty |
|---------|-----------|--------|----------------|----------------|
| CrashLoopBackOff | Very High | High | Fast | Easy |
| OOMKilled | Very High | High | Fast | Easy |
| ImagePullBackOff | High | High | Fast | Easy |
| Node NotReady | High | Critical | Fast | Medium |
| FailedScheduling | High | Medium | Fast | Easy |
| PVC Pending | Medium | Medium | Fast | Medium |
| Operator Degraded | Medium | High | Fast | Medium |
| Route Unavailable | Medium | Medium | Fast | Easy |
| Auth Failure | Medium | High | Fast | Easy |
| DNS Failure | Low | High | Medium | Medium |

## References

- [Red Hat OpenShift Troubleshooting](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)
- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)
- [OpenShift Known Issues](https://access.redhat.com/documentation/en-us/red_hat_openshift_container_platform/)