# OpenShift Best Practices Reference

## Cluster Best Practices

### Resource Management
- Set both requests and limits for all containers
- Use Guaranteed QoS class for critical workloads (requests == limits)
- Monitor resource usage with `oc top`
- Right-size based on actual metrics

### Deployment Strategies
- Use RollingUpdate with maxUnavailable=1, maxSurge=1
- Configure liveness and readiness probes
- Use startup probes for slow-starting applications
- Keep deployment revisions for rollback capability

### Security
- Use restricted SCC for all workloads
- Set runAsNonRoot, readOnlyRootFilesystem, drop ALL capabilities
- Use service accounts with least privilege
- Rotate credentials regularly

## References

- [Red Hat OpenShift Best Practices](https://docs.openshift.com/container-platform/latest/installing/index.html)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/overview/working-with-objects/best-practices/)