# OpenShift Cluster Architecture Concepts

## OpenShift vs Kubernetes

OpenShift is a Kubernetes distribution with additional enterprise features:

| Feature | Kubernetes | OpenShift |
|---------|-----------|-----------|
| Package management | Manual/CI-CD | Operators (OLM) |
| Networking | CNI plugins | OVN-Kubernetes (default) |
| Image registry | External | Built-in |
| Monitoring | External | Built-in (Prometheus) |
| Logging | External | Built-in (Elasticsearch) |
| Security | Pod Security Policies | SCC + RBAC |
| CLI | kubectl | oc (with extended commands) |
| Console | Limited | Full web console |
| Multi-tenancy | Namespaces | Projects + Quotas |
| Build automation | External | BuildConfig/BC |
| Application routing | Ingress | Routes |

## Core Concepts

### Project vs Namespace
- **Namespace**: Kubernetes-native resource isolation
- **Project**: OpenShift-enhanced namespace with additional security and quota management
- All projects are namespaces, but not all namespaces are projects

### Resource Hierarchy
```
Cluster
├── Node
│   └── Pod
│       ├── Container
│       ├── Volume
│       └── Service Account
├── Namespace/Project
│   ├── Deployment
│   ├── Service
│   ├── Route
│   ├── PVC
│   ├── ConfigMap
│   └── Secret
└── Cluster Operators
```

### Labels and Selectors
- **Labels**: Key-value pairs attached to resources
- **Selectors**: Filter resources by labels
- **Used by**: Deployments, Services, NetworkPolicies

### Annotations
- Key-value metadata not used for identification
- Used for documentation, tooling, and configuration
- Example: `kubectl.kubernetes.io/last-applied-configuration`

## OpenShift-Specific Concepts

### Operators
- Self-managing applications that operate other applications
- Use Custom Resources (CR) for configuration
- Example: PostgreSQL operator manages database instances

### BuildConfig
- OpenShift build automation system
- Defines build strategies (Source, Docker, Custom)
- Supports source-to-image (S2I) builds

### ImageStreams
- Abstraction for container images
- Tracks image tags and updates
- Triggers builds on image changes

### Routes
- OpenShift-specific external access mechanism
- Provides hostname-based routing
- TLS termination at the router

### Security Context Constraints (SCC)
- Pod security policies specific to OpenShift
- Controls what pods can do at kernel level
- More granular than Kubernetes PodSecurityPolicies

## References

- [OpenShift Concepts](https://docs.openshift.com/container-platform/latest/architecture/index.html)
- [Kubernetes Concepts](https://kubernetes.io/docs/concepts/)