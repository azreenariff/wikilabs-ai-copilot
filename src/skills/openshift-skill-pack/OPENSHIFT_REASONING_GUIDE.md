# OpenShift Engineering Reasoning Guide

## Purpose

This guide documents the structured engineering reasoning framework for OpenShift issue diagnosis. The AI never exposes chain-of-thought — it stores and uses only structured engineering reasoning.

## Reasoning Pattern

For every observed symptom, follow this pattern:

1. **Observed**: What is the current state?
2. **Possible Causes**: What could explain this state?
3. **Evidence Needed**: What information confirms or rules out each cause?
4. **Evidence Collection**: Commands to gather evidence
5. **Recommendation**: What to do based on evidence

## Pattern 1: Pod CrashLoopBackOff

**Observed**: Pod status shows CrashLoopBackOff

**Possible Causes**:
- Application error (code crash, missing dependency)
- Configuration error (missing ConfigMap, wrong env vars)
- Resource issue (OOMKilled, insufficient resources)
- Health probe failure (liveness probe killing container)
- Image issue (wrong image, missing entrypoint)
- Init container failure

**Evidence Needed**:
1. Pod events — OOMKill, probe failures, image pull errors
2. Pod logs (current) — application error messages
3. Pod logs (previous) — last termination reason
4. Pod describe — resource limits, probe config, environment
5. Deployment config — probes, resources, image, init containers

**Evidence Collection**:
```bash
oc get events -n <namespace> --sort-by='.lastTimestamp'
oc describe pod <pod-name> -n <namespace>
oc logs <pod-name> -n <namespace> --previous
oc logs <pod-name> -n <namespace>
oc get pods -n <namespace> -o jsonpath='{range .items[?(@.metadata.name=="<pod>")].status.containerStatuses[*]}{.name}: {.lastState.terminated.reason}{"\n"}{end}'
```

**Decision Logic**:
- If lastState.reason == OOMKilled → Increase memory limits
- If events show LivenessProbe failed → Adjust probe timing
- If events show ErrImagePull → Fix image reference
- If events show FailedMount → Fix volume/ConfigMap/Secret
- If logs show application error → Debug application configuration
- If init container failed → Fix init container configuration

**Recommendation**: Review Events first, then previous logs. Events reveal scheduling, image, or resource issues before logs.

## Pattern 2: Pod Pending

**Observed**: Pod status shows Pending

**Possible Causes**:
- Insufficient cluster resources (CPU, memory, ephemeral storage)
- Node taints without matching tolerations
- PVC Pending (storage not available)
- Affinity rules cannot be satisfied
- Cluster capacity exceeded
- Invalid pod spec

**Evidence Needed**:
1. Pod events — FailedScheduling reason and message
2. Node capacity — available resources
3. PVC status — if storage-dependent
4. Taints/tolerations — pod vs node mismatch
5. Node labels — affinity/selector mismatch

**Evidence Collection**:
```bash
oc get events -n <namespace> --field-selector reason=FailedScheduling
oc describe pod <pod-name> -n <namespace>
oc get nodes -o wide
oc get nodes -o jsonpath='{range .items[*].spec.taints[*]}{.key}={.value}:{.effect}{"\n"}{end}'
oc get pvc -n <namespace>
```

**Decision Logic**:
- If FailedScheduling "Insufficient cpu" → Reduce CPU requests or add nodes
- If FailedScheduling "Insufficient memory" → Reduce memory requests or add nodes
- If FailedScheduling "node(s) didn't match Pod's node affinity" → Check nodeSelector
- If FailedScheduling "node(s) didn't match Pod's node selector" → Check nodeSelector
- If FailedScheduling "pvc not found" → Fix PVC reference
- If FailedScheduling "node(s) had taint" → Add toleration or remove taint

**Recommendation**: Check FailedScheduling events first, then node capacity.

## Pattern 3: Container OOMKilled

**Observed**: Container terminated with OOMKilled

**Possible Causes**:
- Memory limit set too low for application
- Memory leak in application code
- Multiple containers sharing memory limits
- Burstable QoS with memory spike
- Host-level memory pressure
- Java application without proper heap settings

**Evidence Needed**:
1. Container status — lastState.terminated.reason == OOMKilled
2. Pod resource configuration — limits vs requests
3. Memory metrics — actual usage vs limit
4. Node memory pressure — available memory

**Evidence Collection**:
```bash
oc describe pod <pod-name> -n <namespace> | grep -A 10 "Last State"
oc top pod <pod-name> -n <namespace> --containers
oc get events -n <namespace> --field-selector reason=OOMKilled
oc top nodes
```

**Decision Logic**:
- If limit is low relative to usage → Increase limit
- If memory spike correlates with load → Increase limit or optimize code
- If Java app without -Xmx → Set JAVA_OPTS with -Xmx
- If multiple containers in pod → Adjust per-container limits
- If node MemoryPressure → Add node or reduce overall usage

**Recommendation**: Increase memory limits, verify with `oc top pod --containers`, then investigate if OOMKilled persists.

## Pattern 4: ImagePullBackOff

**Observed**: Pod cannot pull container image

**Possible Causes**:
- Wrong image name or tag
- Image does not exist in registry
- Registry authentication required but missing
- Network connectivity to registry
- Image digest mismatch
- Registry is down

**Evidence Needed**:
1. Pod events — pull error details
2. Image stream configuration — referenced image
3. Pull secrets — exists and valid for namespace
4. Registry accessibility — can reach registry URL

**Evidence Collection**:
```bash
oc describe pod <pod-name> -n <namespace> | grep "ImagePullBackOff\|ErrImagePull"
oc get events -n <namespace> --sort-by='.lastTimestamp'
oc get secrets -n <namespace> | grep "image-pull"
oc get imagestreams -n <namespace>
```

**Decision Logic**:
- If error "manifest unknown" → Verify image name and tag
- If error "unauthorized" → Create or update pull secret
- If error "net/http: TLS handshake timeout" → Check network connectivity
- If error "context deadline exceeded" → Registry may be slow or down
- If image tag is "latest" → Pin to specific version

**Recommendation**: Verify image reference, check pull secrets, test registry access with `oc debug` pod.

## Pattern 5: Node NotReady

**Observed**: Node reporting NotReady status

**Possible Causes**:
- Kubelet process stopped or crashed
- Disk pressure (disk usage exceeds threshold)
- Memory pressure (available memory below threshold)
- PID pressure (too many processes on node)
- Network partition or DNS failure
- Hardware failure

**Evidence Needed**:
1. Node conditions — Ready status and reason
2. Kubelet logs — process errors and crashes
3. Resource usage — disk, memory, PID count
4. Node network connectivity

**Evidence Collection**:
```bash
oc get nodes
oc describe node <node-name>
oc get events --field-selector involvedObject.name=<node-name>
oc debug node/<node-name> -- chroot /host journalctl -u kubelet --no-pager
```

**Decision Logic**:
- If DiskPressure → Clean disk space or expand volume
- If MemoryPressure → Reduce memory usage or add memory
- If PIDPressure → Reduce processes or increase PID limit
- If kubelet not running → Restart kubelet
- If network-related → Check node network configuration
- If multiple nodes affected → Check shared infrastructure

**Recommendation**: Check node conditions first, then access node for deeper investigation.

## Pattern 6: Operator Degraded

**Observed**: Cluster operator showing Degraded=True

**Possible Causes**:
- Operator pods failing to start
- Missing RBAC permissions
- Dependent operator also degraded
- Configuration errors in operator Custom Resource
- Resource constraints on operator pods

**Evidence Needed**:
1. Operator conditions — degradation reason
2. Operator pod status — Running, CrashLoopBackOff, Pending
3. Operator logs — error messages
4. Dependent operator status

**Evidence Collection**:
```bash
oc get co
oc describe clusteropeartor <operator>
oc get pods -n openshift-<component>-operator
oc logs -l app=<operator> -n openshift-<component>-operator
```

**Decision Logic**:
- If operator pods Pending → Check resource requests
- If operator pods CrashLoopBackOff → Check logs for errors
- If RBAC error → Fix role bindings
- If configuration error → Fix operator Custom Resource
- If dependent operator degraded → Fix dependency first

**Recommendation**: Check operator pod status and logs before making configuration changes.

## Pattern 7: PVC Pending

**Observed**: PVC stuck in Pending state

**Possible Causes**:
- No matching storage class available
- Storage class provisioner not configured
- Insufficient storage capacity
- Access mode mismatch
- Storage provisioner failure

**Evidence Needed**:
1. PVC events — provisioning failure reason
2. Storage class configuration — available classes
3. PV availability — existing persistent volumes
4. Access mode compatibility

**Evidence Collection**:
```bash
oc describe pvc <pvc-name> -n <namespace>
oc get storageclass
oc get pv
oc get events -n <namespace> --field-selector involvedObject.name=<pvc-name>
```

**Decision Logic**:
- If no default storage class → Set default class
- If storage class not found → Fix PVC storageClassName
- If provisioner not available → Check provisioner configuration
- If access mode mismatch → Adjust claim access mode

**Recommendation**: Verify storage class availability and PVC configuration.

## Pattern 8: FailedScheduling

**Observed**: Pod stuck in Pending with FailedScheduling reason

**Possible Causes**:
- Insufficient cluster resources (CPU, memory)
- Node taints without matching tolerations
- Affinity rules cannot be satisfied
- PVC cannot be bound
- Cluster at capacity

**Evidence Needed**:
1. Pod events — FailedScheduling reason and message
2. Node capacity — available resources
3. Taints and tolerations — pod vs node mismatch
4. Affinity rules — pod scheduling constraints

**Evidence Collection**:
```bash
oc get events -n <namespace> --field-selector reason=FailedScheduling
oc describe pod <pod-name> -n <namespace>
oc get nodes -o wide
```

**Decision Logic**:
- If "Insufficient cpu" → Reduce CPU requests or add nodes
- If "Insufficient memory" → Reduce memory requests or add nodes
- If "node(s) didn't match Pod's node affinity" → Check affinity rules
- If "node(s) didn't match Pod's node selector" → Check nodeSelector
- If "node(s) had taint" → Add toleration to pod spec

**Recommendation**: Check FailedScheduling events first, then analyze scheduling constraints.

## Pattern 9: Route Unavailable

**Observed**: OpenShift route not accessible

**Possible Causes**:
- Service has no endpoint pods
- Route misconfiguration (wrong target service or port)
- TLS certificate issues
- Network policies blocking traffic
- Route admission controller issues

**Evidence Needed**:
1. Route configuration — target service and port
2. Service endpoints — healthy pods backing service
3. TLS configuration — certificate and termination type
4. Network policies — blocking rules

**Evidence Collection**:
```bash
oc describe route <route-name> -n <namespace>
oc get endpoints <service-name> -n <namespace>
oc get networkpolicies -n <namespace>
```

**Decision Logic**:
- If endpoints empty → Fix pods backing the service
- If wrong service target → Correct route target
- If TLS issues → Fix certificate or termination type
- If network policy → Adjust ingress rules

**Recommendation**: Verify service endpoints first, then route configuration.

## Pattern 10: Authentication/Authorization Failure

**Observed**: User or service account cannot access resources

**Possible Causes**:
- Expired or invalid authentication token
- Missing RBAC role or rolebinding
- Service account misconfiguration
- SCC violation
- User permissions revoked

**Evidence Needed**:
1. Token validity — oc whoami
2. Role bindings — roles assigned to user/service account
3. SCC assignments — security context constraints
4. Permission checks — oc auth can-i

**Evidence Collection**:
```bash
oc whoami
oc auth can-i <verb> <resource> --as=<user>
oc adm policy can-i --list --as=system:serviceaccount:<ns>:<sa>
oc describe rolebinding <binding-name> -n <namespace>
```

**Decision Logic**:
- If token expired → Refresh token with oc login
- If role not bound → Add rolebinding
- If SCC denied → Grant SCC to service account
- If namespace-scoped issue → Check namespace role bindings

**Recommendation**: Verify identity first, then check permissions, then check SCC.

## Reasoning Safety Rules

1. **Never assume** — Always verify with evidence
2. **Consider cascade effects** — One issue may indicate broader problems
3. **Plan for rollback** — Every recommendation should include reversal
4. **Start simple** — Check events and describe before deep investigation
5. **Respect maintenance windows** — Critical operations require planning
6. **Document findings** — Record evidence and decisions

## References

- [Kubernetes Troubleshooting Guide](https://kubernetes.io/docs/tasks/debug/)
- [Red Hat OpenShift Troubleshooting](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)