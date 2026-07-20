# OpenShift Concepts Reference

## Key Terminology

### Pod
- Smallest deployable unit
- One or more containers sharing network/storage
- Ephemeral — recreated on failure

### Deployment
- Manages stateless applications
- Rolling updates and rollbacks
- ReplicaSet management

### StatefulSet
- Manages stateful applications
- Stable network identity
- Ordered deployment/deletion

### DaemonSet
- One pod per node
- System daemons and agents
- Automatic node addition

### Job
- Runs to completion
- One-time or parallel execution
- Failed job can be retried

### CronJob
- Scheduled Jobs
- Unix cron syntax
- Multiple concurrent jobs possible

### Service
- Stable network endpoint
- Load balances across pods
- DNS-based discovery

### Route
- OpenShift external access
- TLS termination
- Host-based routing

### PersistentVolumeClaim
- Storage request
- Bound to PersistentVolume
- Access modes and capacity

### ConfigMap
- Configuration data
- Mounted as files or env vars
- Non-secret configuration

### Secret
- Sensitive data
- Base64 encoded
- Mounted as files or env vars

### Project/namespace
- Resource isolation
- Quotas and limits
- Network policies

### Operator
- Self-managing application
- Custom Resources
- Automated lifecycle

## Resource Types

### Core Resources
- Pod, Deployment, StatefulSet, DaemonSet, Job, CronJob
- Service, Ingress, Route
- ConfigMap, Secret
- PersistentVolume, PersistentVolumeClaim
- ServiceAccount, Role, ClusterRole, RoleBinding, ClusterRoleBinding

### OpenShift Resources
- BuildConfig, ImageStream, Route
- ClusterOperator, ClusterVersion
- NetworkPolicy, SecurityContextConstraints
- Template, TemplateInstance

## References

- [Kubernetes Resources](https://kubernetes.io/docs/concepts/overview/working-with-objects/kubernetes-objects/)
- [OpenShift Resources](https://docs.openshift.com/container-platform/latest/architecture/resource-model.html)