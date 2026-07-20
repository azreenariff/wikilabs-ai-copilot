# OpenShift Engineering — Known Issues

## CrashLoopBackOff

### Issue
Pods repeatedly crash and restart, entering CrashLoopBackOff state.

### Common Causes
- Application missing required configuration files or environment variables
- Database connectivity failures at startup
- Image misconfiguration (wrong command or entrypoint)
- Resource limits too low (OOMKilled)
- Health probe misconfiguration (liveness probe too aggressive)

### Detection
- `oc get pods | grep CrashLoopBackOff`
- `oc describe pod <pod>` shows "CrashLoopBackOff" in state
- `oc logs <pod> --previous` shows termination reason

### Workarounds
- Review previous logs: `oc logs <pod> --previous`
- Increase probe delay: `oc set env` or update deployment probes
- Check resource limits: `oc top pod <pod> --containers`
- Verify image configuration: `oc debug pod/<pod>`
- For OOMKilled: increase memory limits

### References
- Documentation: Pod Troubleshooting Guide
- Knowledge: pod-crash-loopbackoff workflow

---

## OOMKilled

### Issue
Containers terminated due to exceeding memory limits.

### Common Causes
- Memory limit set too low for application requirements
- Memory leak in application code
- Multiple containers in same pod competing for shared memory limit
- Burstable QoS with unpredictable memory spikes
- Java applications without proper heap settings (-Xmx)

### Detection
- `oc get pods -o wide` shows OOMKilled in RESTARTS column
- `oc describe pod <pod>` shows "reason: OOMKilled" in lastState
- `oc get events --field-selector reason=OOMKilled`

### Workarounds
- Increase memory limits: `oc set resources deployment/<deployment> --limits=memory=<value>`
- For Java apps, set JAVA_OPTS or JVM_ARGS with -Xmx
- Ensure requests ≤ limits for Guaranteed QoS class
- Monitor with `oc top pod <pod> --containers`

### References
- Documentation: Resource Management
- Knowledge: pod-oomkilled workflow

---

## ImagePullBackOff

### Issue
Pods cannot pull container images from registry.

### Common Causes
- Wrong image name or tag
- Image does not exist at specified reference
- Missing or expired image pull secrets
- Registry network connectivity issues
- Image digest mismatch

### Detection
- `oc describe pod <pod>` shows "ImagePullBackOff" or "ErrImagePull"
- `oc get events` shows pull error details
- `oc get imagestreams` shows available images

### Workarounds
- Verify image reference: `oc get imagestreams -n <namespace>`
- Check pull secrets: `oc get secrets | grep -i image`
- Test registry access: `oc registry info`
- Update image: `oc set image deployment/<deployment> <container>=<image-ref>`
- For private registries: create image pull secret

### References
- Documentation: Image Management
- Knowledge: pod-imagepullbackoff workflow

---

## Node NotReady

### Issue
Nodes reporting NotReady status.

### Common Causes
- Kubelet process stopped or crashed
- Disk pressure (disk usage exceeds threshold)
- Memory pressure (available memory below threshold)
- PID pressure (too many processes on node)
- Network partition or DNS failure
- Machine failure (hardware, cloud provider issue)

### Detection
- `oc get nodes` shows "NotReady" status
- `oc describe node <node>` shows condition details
- `oc get events` shows NodeNotReady events

### Workarounds
- Check kubelet: `oc debug node/<node>` → `sudo journalctl -u kubelet`
- Clean disk: `oc debug node/<node>` → cleanup large files
- Restart kubelet: `oc debug node/<node>` → `sudo systemctl restart kubelet`
- Drain node: `oc adm drain <node> --delete-emptydir-data`
- Replace machine if hardware failure detected

### References
- Documentation: Node Management
- Knowledge: node-notready workflow

---

## PVC Pending

### Issue
Persistent Volume Claims stuck in Pending state.

### Common Causes
- No matching storage class available
- Storage class provisioner not configured
- Insufficient storage capacity
- Access mode mismatch (e.g., requesting ReadWriteMany when only ReadWriteOnce available)
- Storage provisioner failure

### Detection
- `oc get pvc` shows "Pending" status
- `oc describe pvc <pvc>` shows provisioning events and errors
- `oc get events` shows provisioning failures

### Workarounds
- Check storage classes: `oc get storageclass`
- Verify provisioner: `oc describe storageclass <class>`
- Adjust access mode: `oc patch pvc <pvc> -p '{"spec":{"accessModes":["ReadWriteOnce"]}}'`
- Set default storage class if needed
- Create PV manually if using static provisioning

### References
- Documentation: Storage Management
- Knowledge: pvc-pending workflow

---

## FailedScheduling

### Issue
Pods stuck in Pending due to scheduling failures.

### Common Causes
- Insufficient cluster resources (CPU, memory)
- Node taints without matching tolerations
- Affinity rules cannot be satisfied
- PVC cannot be bound (storage unavailable)
- Cluster at capacity

### Detection
- `oc describe pod <pod>` shows "FailedScheduling" event
- `oc get events --field-selector reason=FailedScheduling`
- `oc get nodes` shows resource capacity

### Workarounds
- Scale cluster: Add nodes or machines
- Check resource requests: `oc describe pod <pod>`
- Review taints: `oc get nodes -o jsonpath='{.items[*].spec.taints}'`
- Review affinity rules in deployment spec
- Reduce resource requests if over-provisioned

### References
- Documentation: Scheduling
- Knowledge: pod-pending workflow

---

## Degraded Operator

### Issue
Cluster operator in Degraded state.

### Common Causes
- Operator pods failing to start
- Missing RBAC permissions
- Dependent operator also degraded
- Configuration errors in operator Custom Resource
- Resource constraints on operator pods

### Detection
- `oc get co` shows operator with "Degraded=True"
- `oc describe clusteropeartor <operator>` shows error conditions
- `oc get pods -n openshift-<component>-operator` shows failures

### Workarounds
- Check operator pods: `oc get pods -n openshift-<component>-operator`
- Review operator logs: `oc logs -l app=<operator> -n openshift-<component>-operator`
- Check dependent operators status
- Verify operator RBAC permissions
- Restart operator pods if stuck: `oc delete pods -l app=<operator> -n openshift-<component>-operator`

### References
- Documentation: Operator Lifecycle Management
- Knowledge: operator-degraded workflow

---

## Route Unavailable

### Issue
OpenShift routes not accessible.

### Common Causes
- Service has no endpoint pods
- Route misconfiguration (wrong target service or port)
- TLS certificate issues
- Network policies blocking traffic
- Route admission controller issues

### Detection
- `oc get routes` shows route exists
- `oc describe route <route>` shows destination details
- `oc get endpoints <service>` shows no ready addresses
- `curl https://<route-hostname>` fails

### Workarounds
- Verify service endpoints: `oc get endpoints <service>`
- Check route target: `oc describe route <route>`
- Review network policies: `oc get networkpolicies`
- Test route DNS resolution
- Check route admission controller logs

### References
- Documentation: Networking
- Knowledge: route-unavailable workflow

---

## Authentication Issues

### Issue
Users or service accounts unable to authenticate or authorize.

### Common Causes
- Expired or invalid authentication token
- Missing RBAC role or rolebinding
- Service account misconfiguration
- OAuth provider configuration issues
- User permissions revoked

### Detection
- `oc auth can-i <verb> <resource>` returns "no"
- `oc whoami` shows unexpected user
- API server logs show authorization errors
- `oc describe rolebinding <binding>` shows empty subjects

### Workarounds
- Refresh token: `oc login --server=<server> --token=<token>`
- Add rolebinding: `oc adm policy add-role-to-user <role> <user> -n <namespace>`
- Check service account: `oc describe sa <sa-name>`
- Verify OAuth provider: `oc describe oauthclient <client>`
- Review cluster roles and bindings

### References
- Documentation: Security and RBAC
- Knowledge: authentication-issue workflow