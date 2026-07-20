# OpenShift Troubleshooting Workflows

## Purpose

This guide documents the standard troubleshooting workflows for OpenShift cluster issues.

## How to Use Workflows

1. Identify the symptom from observation
2. Select the appropriate workflow
3. Follow the investigation steps in order
4. Collect evidence at each step
5. Make a diagnosis based on evidence
6. Apply the recommended fix
7. Verify resolution
8. Document findings

## Workflow 1: Pod CrashLoopBackOff

### Symptom
Pod repeatedly crashes and enters CrashLoopBackOff state.

### Evidence Collection
1. `oc get pods -n <namespace>` — Confirm CrashLoopBackOff status
2. `oc describe pod <pod-name> -n <namespace>` — Check events and container state
3. `oc logs <pod-name> -n <namespace>` — View current logs
4. `oc logs <pod-name> -n <namespace> --previous` — Check last container logs
5. `oc get events -n <namespace> --sort-by='.lastTimestamp'` — Review events

### Decision Tree
- Crash reason is OOMKilled → Increase memory limits
- Crash reason is liveness probe failure → Adjust probe timing or fix application health endpoint
- Crash reason is image pull error → Fix image reference or registry authentication
- Crash reason is application error → Debug application, check configuration and environment
- No clear crash reason → Exec into container for investigation

### Commands
```bash
oc get pods -n <namespace> | grep CrashLoopBackOff
oc describe pod <pod-name> -n <namespace>
oc logs <pod-name> -n <namespace> --previous -c <container>
oc exec -it <pod-name> -n <namespace> -- /bin/sh
oc debug pod/<pod-name> -n <namespace>
```

### Risk Assessment
- Modifying probe settings: Low risk — may delay detection of real issues
- Increasing resource limits: Low risk — more resource usage
- Restarting pods: Medium risk — brief service interruption
- Exec into production container: Medium risk — may affect running application

## Workflow 2: Pending Pod

### Symptom
Pod stuck in Pending state.

### Evidence Collection
1. `oc get pods -n <namespace>` — Confirm Pending status
2. `oc describe pod <pod-name> -n <namespace>` — Check events for scheduling failures
3. `oc get nodes` — Check available nodes and resources
4. `oc describe node <node-name>` — Check node capacity and conditions
5. `oc get pvc -n <namespace>` — Check if PVC is pending

### Decision Tree
- FailedScheduling event → Check resource requests vs node capacity
- Insufficient cpu/memory → Reduce resource requests or scale cluster
- No matching node labels → Check node selectors and affinity rules
- PVC Pending → Fix storage class or create PV
- Taint/toleration mismatch → Add toleration or remove taint

### Commands
```bash
oc get pods -n <namespace> -o wide
oc describe pod <pod-name> -n <namespace>
oc get nodes -o wide
oc get pvc -n <namespace>
oc get events -n <namespace> --field-selector reason=FailedScheduling
```

### Risk Assessment
- Adding tolerations: Low risk — may affect security boundaries
- Scaling cluster: Low risk — adds capacity
- Changing resource requests: Medium risk — affects scheduling
- Removing taints: Medium risk — may affect node isolation

## Workflow 3: OOMKilled Container

### Symptom
Container terminated due to exceeding memory limit.

### Evidence Collection
1. `oc describe pod <pod-name> -n <namespace>` — Check lastState.terminated.reason
2. `oc get events -n <namespace>` — Check OOMKilled events
3. `oc top pod <pod-name> -n <namespace>` — Check current memory usage
4. `oc get pods -n <namespace> -o wide` — Check restart counts

### Decision Tree
- Memory limit too low → Increase limits
- Memory leak suspected → Investigate application code
- Multiple containers competing → Adjust per-container limits
- Java app without heap settings → Set -Xmx or JAVA_OPTS

### Commands
```bash
oc describe pod <pod-name> -n <namespace> | grep -A 5 "OOMKilled"
oc top pod <pod-name> -n <namespace> --containers
oc set resources deployment/<name> -n <namespace> --limits=memory=<value>
```

### Risk Assessment
- Increasing memory limits: Low risk — more resource usage
- Fixing memory leaks: Low risk — fixes root cause
- Adjusting Java heap: Medium risk — affects application performance

## Workflow 4: ImagePullBackOff

### Symptom
Pod cannot pull container image.

### Evidence Collection
1. `oc describe pod <pod-name> -n <namespace>` — Check pull errors
2. `oc get events -n <namespace>` — Check pull-related events
3. `oc get imagestreams -n <namespace>` — Verify available images
4. `oc get secrets -n <namespace>` — Check pull secrets

### Decision Tree
- Image not found → Verify image tag and registry
- Authentication failure → Create or update pull secret
- Registry unreachable → Check network connectivity to registry
- Wrong image reference → Correct image reference in deployment

### Commands
```bash
oc describe pod <pod-name> -n <namespace> | grep "ImagePullBackOff\|ErrImagePull"
oc get secrets -n <namespace> | grep "image-pull\|docker"
oc create secret docker-registry <name> -n <namespace> --docker-server=<server> --docker-username=<user> --docker-password=<pass> --docker-email=<email>
oc set image deployment/<name> <container>=<image> -n <namespace>
```

### Risk Assessment
- Changing image reference: Medium risk — new image may have issues
- Updating pull secrets: Low risk
- Adding new pull secret: Low risk

## Workflow 5: Node NotReady

### Symptom
Node reporting NotReady status.

### Evidence Collection
1. `oc get nodes` — Check node status
2. `oc describe node <node-name>` — Check conditions
3. `oc get events --field-selector involvedObject.name=<node-name>` — Check events
4. `oc debug node/<node-name>` — Access node for deeper investigation

### Decision Tree
- DiskPressure → Clean disk space or expand volume
- MemoryPressure → Reduce memory usage or add memory
- Kubelet not running → Restart kubelet
- Network issue → Check node network configuration
- Hardware failure → Plan node replacement

### Commands
```bash
oc get nodes
oc describe node <node-name>
oc debug node/<node-name> -- chroot /host journalctl -u kubelet
oc adm drain <node-name> --delete-emptydir-data
```

### Risk Assessment
- Restarting kubelet: Medium risk — may affect pods on node
- Draining node: High risk — evicts all pods
- Cleaning disk: Medium risk — may affect other processes
- Replacing node: Medium risk — scheduled during maintenance

## Workflow 6: Deployment Failure

### Symptom
Deployment not rolling out successfully.

### Evidence Collection
1. `oc get deployments -n <namespace>` — Check deployment status
2. `oc rollout status deployment/<name> -n <namespace>` — Check rollout status
3. `oc describe deployment/<name> -n <namespace>` — Check deployment conditions
4. `oc get replicasets -n <namespace>` — Check associated ReplicaSets
5. `oc get pods -l app=<app> -n <namespace>` — Check pod status

### Decision Tree
- ReplicaCount mismatch → Check pod errors and restart
- ProgressDeadlineExceeded → Check pod scheduling and startup
- ImagePullBackOff → Fix image reference (see Workflow 4)
- Probe failures → Fix health endpoint or adjust probes
- Resource errors → Adjust resource requests/limits

### Commands
```bash
oc rollout status deployment/<name> -n <namespace>
oc rollout history deployment/<name> -n <namespace>
oc rollout undo deployment/<name> -n <namespace>
oc set image deployment/<name> <container>=<image> -n <namespace>
```

### Risk Assessment
- Rolling back deployment: Medium risk — reverts latest changes
- Setting new image: Medium risk — new image may have issues
- Adjusting probes: Low risk — affects monitoring only

## Workflow 7: Operator Degraded

### Symptom
Cluster operator showing Degraded status.

### Evidence Collection
1. `oc get co` — Check operator status
2. `oc describe clusteropeartor <operator>` — Check operator conditions
3. `oc get pods -n openshift-<component>-operator` — Check operator pods
4. `oc logs -l app=<operator> -n openshift-<component>-operator` — Check operator logs

### Decision Tree
- Operator pods not running → Check pod events and logs
- RBAC issues → Fix role bindings
- Configuration errors → Fix operator Custom Resource
- Dependent operator degraded → Fix dependent operator first
- Resource constraints → Adjust resource limits

### Commands
```bash
oc get co
oc describe clusteropeartor <operator>
oc get pods -n openshift-<component>-operator
oc logs -l app=<operator> -n openshift-<component>-operator
oc delete pods -l app=<operator> -n openshift-<component>-operator
```

### Risk Assessment
- Restarting operator pods: Low risk — operator restarts automatically
- Changing operator configuration: Medium risk — may affect component behavior
- Fixing RBAC: Low risk — restores proper access

## Workflow 8: Route Unavailable

### Symptom
Route not accessible or returning errors.

### Evidence Collection
1. `oc get routes -n <namespace>` — Check route status
2. `oc describe route <route-name> -n <namespace>` — Check route details
3. `oc get endpoints <service-name> -n <namespace>` — Check service endpoints
4. `oc get svc <service-name> -n <namespace>` — Check service configuration

### Decision Tree
- No endpoints → Fix pods backing the service
- Wrong service target → Correct route target
- TLS issues → Fix certificate or termination type
- Network policy blocking → Adjust network policy
- Wildcard DNS issue → Check DNS configuration

### Commands
```bash
oc get routes -n <namespace> -o wide
oc describe route <route-name> -n <namespace>
oc get endpoints <service-name> -n <namespace>
oc get svc <service-name> -n <namespace>
```

### Risk Assessment
- Changing route target: Low risk
- Fixing TLS: Low risk
- Adjusting network policy: Medium risk — may affect other traffic
- Fixing DNS: Low risk

## Workflow 9: PVC Pending

### Symptom
Persistent Volume Claim stuck in Pending state.

### Evidence Collection
1. `oc get pvc -n <namespace>` — Check PVC status
2. `oc describe pvc <pvc-name> -n <namespace>` — Check provisioning events
3. `oc get storageclass` — Check available storage classes
4. `oc get pv` — Check available persistent volumes

### Decision Tree
- No matching storage class → Fix storage class or adjust claim
- Provisioner not available → Check provisioner configuration
- Insufficient capacity → Expand storage or reduce request
- Access mode mismatch → Adjust claim access mode

### Commands
```bash
oc get pvc -n <namespace>
oc describe pvc <pvc-name> -n <namespace>
oc get storageclass
oc get pv
```

### Risk Assessment
- Changing storage class: Low risk
- Adjusting access mode: Low risk
- Creating PV manually: Medium risk — manual intervention

## General Evidence Collection Strategy

### Priority Order
1. **Pod/Node events** — Fastest way to identify issues
2. **Pod/Node describe** — Comprehensive resource state
3. **Logs** — Application-specific details
4. **Resource configuration** — Deployment specs, probes, limits
5. **Cluster state** — Operators, nodes, network

### Documentation References
- [Red Hat OpenShift Troubleshooting Guide](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)
- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)