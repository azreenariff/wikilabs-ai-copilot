# OpenShift Architecture Reference

## Control Plane Architecture

### Master Components

| Component | Pod | Port | Description |
|-----------|-----|------|-------------|
| API Server | kube-apiserver | 6443 | Central management |
| etcd | etcd | 2379-2380 | Key-value store |
| Controller Manager | kube-controller-manager | 10257 | Cluster state controller |
| Scheduler | kube-scheduler | 10259 | Pod scheduling |

### Worker Components

| Component | Service | Port | Description |
|-----------|---------|------|-------------|
| kubelet | kubelet.service | 10250 | Node agent |
| kube-proxy | kube-proxy | 10256 | Network proxy |
| CNI | OVN-Kubernetes | Various | Container networking |

## OpenShift Components

### Cluster Operators
- authentication
- console
- image-registry
- ingress
- networking
- storage
- monitoring
- logging

### Operators Lifecycle

```
CatalogSource → Subscription → InstallPlan → CSV → Running
```

## Networking Architecture

```
External → Route → HAProxy → Service → Endpoints → Pods
```

### Pod Networking
- OVN-Kubernetes overlay network
- VXLAN encapsulation
- Pod IP range: 10.128.0.0/14 (default)

### Service Networking
- ClusterIP: Internal virtual IP
- NodePort: Node-level access
- LoadBalancer: External load balancer

## Storage Architecture

```
PVC → PV → Storage Class → Provisioner → Storage Backend
```

### Storage Classes
- Default: Provisioned dynamically
- Custom: Specialized storage tiers

## Security Architecture

```
User → OAuth → API Server → RBAC → SCC → Pod
```

## Monitoring Architecture

```
Pod → node-exporter → Prometheus → Alertmanager → Grafana
```

## References

- [OpenShift Architecture](https://docs.openshift.com/container-platform/latest/architecture/architecture.html)
- [Kubernetes Architecture](https://kubernetes.io/docs/concepts/architecture/)