# OpenShift Engineering — Best Practices

## General Best Practices

1. **Always check events first** — Events often reveal the root cause before logs
2. **Use descriptive names** — Pods, deployments, services should have clear, consistent naming
3. **Set resource limits and requests** — Prevent OOMKilled and ensure fair scheduling
4. **Configure health probes** — Liveness and readiness probes for automated recovery
5. **Use image tags, not 'latest'** — Deterministic deployments with specific image tags
6. **Limit per-replica resources** — Keep pod sizes small for better scheduling and recovery
7. **Use namespaces/projects effectively** — Isolate workloads by team, environment, or function
8. **Enable resource quotas** — Prevent resource exhaustion in multi-tenant projects
9. **Version control configurations** — Store deployment manifests, secrets, and configurations in git
10. **Automate with operators** — Use OpenShift operators for complex workloads where available

## Deployment Best Practices

1. **Use RollingUpdate strategy** — Default strategy ensures zero-downtime deployments
2. **Set maxUnavailable and maxSurge** — Control rollout speed (e.g., maxUnavailable=1, maxSurge=1)
3. **Configure probe thresholds** — Set appropriate initialDelaySeconds and periodSeconds
4. **Use startup probes** — For slow-starting applications, prevent premature liveness failures
5. **Set replicas ≥ 2 for production** — Ensure high availability for stateless services
6. **Test rollouts in lower environments** — Verify deployments in dev/staging before production
7. **Keep deployment revisions** — Review rollout history before making changes
8. **Document image references** — Use full registry path with digest when possible

## Resource Management Best Practices

1. **Set both requests and limits** — Requests for scheduling, limits for protection
2. **Request ≤ limit** — Ensure requests are less than or equal to limits
3. **Use QoS classes appropriately** — Guaranteed (requests==limits) for critical workloads
4. **Monitor with oc top** — Regularly check actual vs configured resource usage
5. **Right-size based on metrics** — Use historical data to adjust requests and limits
6. **Avoid over-provisioning** — Wasted resources increase costs and reduce cluster capacity

## Networking Best Practices

1. **Use routes for external access** — OpenShift routes provide TLS termination and host-based routing
2. **Configure TLS correctly** — Use edge termination for external traffic, reencrypt for downstream encryption
3. **Set up network policies** — Control ingress and egress traffic at the namespace level
4. **Use headless services for stateful workloads** — Direct pod DNS resolution for StatefulSets
5. **Monitor route health** — Verify routes point to healthy service endpoints
6. **Avoid overlapping routes** — Each route hostname should be unique within the cluster

## Storage Best Practices

1. **Use persistent volumes for stateful data** — Ensure data survives pod restarts
2. **Select appropriate storage class** — Match storage needs (performance, backup, replication)
3. **Set appropriate access modes** — ReadWriteOnce for single-pod access, ReadWriteMany for shared access
4. **Monitor PVC capacity** — Set up alerts for disk usage approaching limits
5. **Use volume snapshots** — Backup data with OpenShift volume snapshot functionality
6. **Clean up unused PVCs** — Remove abandoned claims to free storage capacity

## Security Best Practices

1. **Use service accounts** — Each workload should use its own service account
2. **Follow least privilege** — Grant minimum required RBAC permissions
3. **Configure security context** — Set runAsNonRoot, readOnlyRootFilesystem, drop capabilities
4. **Use SCCs appropriately** — Assign restricted SCC for most workloads, privileged only when necessary
5. **Rotate credentials** — Regularly update image pull secrets and API tokens
6. **Enable audit logging** — Track API server access and changes
7. **Scan images** — Use OpenShift image scanning or external tools for vulnerability detection

## Monitoring and Logging Best Practices

1. **Enable application logging** — Use structured logging for easier parsing and analysis
2. **Set up alerts** — Configure alerts for pod failures, node issues, and resource thresholds
3. **Use Prometheus metrics** — Expose application metrics for monitoring
4. **Implement log rotation** — Prevent disk exhaustion from log accumulation
5. **Centralize logging** — Use OpenShift cluster logging for consolidated log management

## Operator Best Practices

1. **Monitor operator status** — Regularly check `oc get co` for degraded operators
2. **Keep operators updated** — Apply operator updates during maintenance windows
3. **Understand operator dependencies** — Some operators depend on others; upgrade in order
4. **Back up operator configuration** — Store Custom Resource definitions for recovery
5. **Limit operator scope** — Use namespace-scoped operators where possible

## Upgrade Best Practices

1. **Read release notes** — Understand changes, deprecations, and breaking changes before upgrading
2. **Test upgrades in staging** — Validate cluster upgrade process in non-production environment
3. **Backup etcd** — Ensure etcd backup is available before initiating upgrade
4. **Upgrade in phases** — Control plane first, then workers, then operators
5. **Plan maintenance window** — Schedule upgrades during low-traffic periods
6. **Have rollback plan** — Know how to downgrade if upgrade fails

## Documentation References

- [Red Hat OpenShift Documentation](https://docs.openshift.com/container-platform/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [OpenShift Best Practices](https://docs.openshift.com/container-platform/latest/installing/index.html)