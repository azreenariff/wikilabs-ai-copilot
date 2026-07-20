# OpenShift Engineering Examples

## Purpose

This directory provides worked examples for common OpenShift scenarios.

## Example 1: Pod CrashLoopBackOff

### Scenario
A web application pod is stuck in CrashLoopBackOff state.

### Evidence Collection

```bash
# Check pod status
oc get pods -n myproject | grep CrashLoopBackOff
# Output: myapp-5d4b7c8f9-x2k4p 0/1 CrashLoopBackOff 5 10m

# Describe pod
oc describe pod myapp-5d4b7c8f9-x2k4p -n myproject
# Shows events with OOMKilled reason

# Check previous logs
oc logs myapp-5d4b7c8f9-x2k4p -n myproject --previous
# Shows application crash with memory error
```

### Diagnosis
Pod is being OOMKilled due to insufficient memory limits.

### Resolution

```bash
# Check current resource configuration
oc describe pod myapp-5d4b7c8f9-x2k4p -n myproject | grep -A 5 "Limits"
# Shows memory limit: 256Mi

# Increase memory limits
oc set resources deployment/myapp -n myproject --limits=memory=512Mi --requests=memory=256Mi

# Verify deployment rollout
oc rollout status deployment/myapp -n myproject
```

### Verification

```bash
# Check pods are running
oc get pods -n myproject
# Output: myapp-5d4b7c8f9-x2k4p 1/1 Running 0 5m

# Monitor memory usage
oc top pods -n myproject
# Shows memory usage within new limits
```

## Example 2: Node NotReady

### Scenario
A worker node reports NotReady status.

### Evidence Collection

```bash
# Check node status
oc get nodes
# Output: node01.example.com NotReady

# Describe node
oc describe node node01.example.com
# Shows conditions with DiskPressure=True

# Check events
oc get events --field-selector involvedObject.name=node01.example.com
# Shows disk pressure events
```

### Diagnosis
Node has disk pressure due to high disk usage.

### Resolution

```bash
# Access node for investigation
oc debug node/node01.example.com

# Inside debug session, check disk usage
chroot /host df -h
# Shows /var/log partition at 95%

# Clean disk space
chroot /host journalctl --vacuum-size=1G
chroot /host rm -f /var/log/*.gz

# Restart kubelet
chroot /host systemctl restart kubelet

# Exit debug session
exit
```

### Verification

```bash
# Check node is Ready
oc get nodes
# Output: node01.example.com Ready

# Verify pods are running on node
oc get pods -n myproject -o wide | grep node01
```

## Example 3: ImagePullBackOff

### Scenario
Pod cannot pull container image from private registry.

### Evidence Collection

```bash
# Check pod status
oc get pods -n myproject | grep ImagePullBackOff
# Output: myapp-5d4b7c8f9-x2k4p 0/1 ImagePullBackOff 0 5m

# Describe pod
oc describe pod myapp-5d4b7c8f9-x2k4p -n myproject
# Shows ErrImagePull with unauthorized error
```

### Diagnosis
Missing or expired image pull secret for private registry.

### Resolution

```bash
# Create image pull secret
oc create secret docker-registry myapp-registry-secret \
  --docker-server=registry.example.com \
  --docker-username=myuser \
  --docker-password=mypass \
  --docker-email=me@example.com \
  -n myproject

# Update deployment to use secret
oc patch deployment myapp -n myproject \
  -p '{"spec":{"template":{"spec":{"imagePullSecrets":[{"name":"myapp-registry-secret"}]}}}}'

# Verify rollout
oc rollout status deployment/myapp -n myproject
```

### Verification

```bash
# Check pods are running
oc get pods -n myproject
# Output: myapp-5d4b7c8f9-x2k4p 1/1 Running 0 2m

# Verify image pull
oc logs myapp-5d4b7c8f9-x2k4p -n myproject
# Shows successful startup logs
```

## Example 4: Operator Degraded

### Scenario
Authentication operator shows Degraded status.

### Evidence Collection

```bash
# Check operator status
oc get co
# Shows authentication operator with Degraded=True

# Describe operator
oc describe clusteroperator authentication
# Shows conditions with message about failed configuration

# Check operator pods
oc get pods -n openshift-authentication
# Shows authentication pods in CrashLoopBackOff
```

### Diagnosis
Operator pods are failing due to RBAC permission issues.

### Resolution

```bash
# Check operator logs
oc logs -l app=openshift-authentication -n openshift-authentication

# Fix RBAC permissions
oc adm policy add-cluster-role-to-user cluster-admin system:serviceaccount:openshift-authentication:default

# Restart operator pods
oc delete pods -l app=openshift-authentication -n openshift-authentication

# Monitor operator status
oc get co -w
```

### Verification

```bash
# Check operator is Available
oc get co authentication
# Shows Available=True, Progressing=False, Degraded=False
```

## References

- [Red Hat OpenShift Troubleshooting](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)
- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)