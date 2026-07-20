# OpenShift Best Practices

## General Best Practices

### 1. Always Check Events First
Events often reveal the root cause before logs. Always run:
```bash
oc get events -n <namespace> --sort-by='.lastTimestamp'
```

### 2. Use Descriptive Names
Pods, deployments, and services should have clear, consistent naming:
- Use lowercase letters, numbers, and hyphens
- Include the application name: `web-app-frontend`
- Include the environment: `web-app-frontend-prod`

### 3. Set Resource Limits and Requests
```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "250m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

### 4. Configure Health Probes
Always set liveness and readiness probes:
```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

### 5. Use Specific Image Tags
Avoid `latest` tag in production:
```yaml
image: registry.example.com/myapp:1.2.3
# Or use digest for absolute certainty:
image: registry.example.com/myapp@sha256:abc123...
```

### 6. Limit Per-Replica Resources
Keep pod sizes small for better scheduling and recovery:
- Start with reasonable defaults
- Scale horizontally before scaling vertically
- Use pod disruption budgets for critical workloads

### 7. Use Namespaces Effectively
- Isolate workloads by team, environment, or function
- Set resource quotas per namespace
- Use network policies for security

### 8. Enable Resource Quotas
```bash
oc adm quota --add resource-quota \
  --hard=requests.cpu=4,requests.memory=8Gi \
  --hard=limits.cpu=8,limits.memory=16Gi \
  --hard=pods=100 \
  -n <namespace>
```

### 9. Version Control Configurations
Store deployment manifests, secrets, and configurations in git:
- Use Kustomize or Helm for templating
- Tag releases in git
- Track configuration changes

### 10. Automate with Operators
Use OpenShift operators for complex workloads where available:
- Database operators (PostgreSQL, MySQL)
- Monitoring operators (Prometheus)
- Custom operators for application-specific needs

## Deployment Best Practices

### 1. Use RollingUpdate Strategy
Default strategy ensures zero-downtime deployments:
```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxUnavailable: 1
    maxSurge: 1
```

### 2. Set Appropriate Probe Thresholds
- initialDelaySeconds: Allow application startup time
- periodSeconds: Check frequently enough to detect failures
- failureThreshold: Allow for transient failures

### 3. Use Startup Probes for Slow Applications
Prevent premature liveness probe failures:
```yaml
startupProbe:
  httpGet:
    path: /healthz
    port: 8080
  failureThreshold: 30
  periodSeconds: 10
```

### 4. Set Replicas >= 2 for Production
Ensure high availability for stateless services.

### 5. Test Rollouts in Lower Environments
Verify deployments in dev/staging before production.

### 6. Keep Deployment Revisions
Review rollout history before making changes:
```bash
oc rollout history deployment/<name> -n <namespace>
```

## Resource Management Best Practices

### 1. Set Both Requests and Limits
- Requests: Minimum guaranteed resources for scheduling
- Limits: Maximum resources the container can use

### 2. Use Appropriate QoS Classes
- Guaranteed (requests == limits): Critical workloads
- Burstable (requests < limits): Standard workloads
- BestEffort (no requests/limits): Development/testing

### 3. Monitor Resource Usage
```bash
oc top pods -n <namespace>
oc top nodes
```

### 4. Right-Size Based on Metrics
Use historical data to adjust requests and limits.

### 5. Avoid Over-Provisioning
Wasted resources increase costs and reduce cluster capacity.

## Security Best Practices

### 1. Use Service Accounts
Each workload should use its own service account:
```yaml
serviceAccountName: myapp-service-account
```

### 2. Follow Least Privilege
Grant minimum required RBAC permissions:
```bash
oc adm policy add-role-to-user edit developer-user -n <namespace>
```

### 3. Configure Security Context
```yaml
securityContext:
  runAsNonRoot: true
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
```

### 4. Use Restricted SCC
Assign restricted Security Context Constraints for most workloads.

### 5. Rotate Credentials
Regularly update image pull secrets and API tokens.

### 6. Enable Audit Logging
Track API server access and changes.

### 7. Scan Images
Use OpenShift image scanning or external tools for vulnerability detection.

## Networking Best Practices

### 1. Use Routes for External Access
OpenShift routes provide TLS termination and hostname-based routing.

### 2. Configure TLS Correctly
- Edge termination: TLS terminates at router
- Reencrypt: TLS re-encrypted to backend service
- Passthrough: End-to-end TLS encryption

### 3. Set Up Network Policies
Control ingress and egress traffic at the namespace level.

### 4. Use Headless Services for StatefulWorkloads
Direct pod DNS resolution for StatefulSets.

### 5. Monitor Route Health
Verify routes point to healthy service endpoints.

### 6. Avoid Overlapping Routes
Each route hostname should be unique within the cluster.

## Storage Best Practices

### 1. Use Persistent Volumes for Stateful Data
Ensure data survives pod restarts.

### 2. Select Appropriate Storage Class
Match storage needs (performance, backup, replication).

### 3. Set Appropriate Access Modes
- ReadWriteOnce: Single pod access
- ReadWriteMany: Shared access across pods

### 4. Monitor PVC Capacity
Set up alerts for disk usage approaching limits.

### 5. Use Volume Snapshots
Backup data with OpenShift volume snapshot functionality.

### 6. Clean Up Unused PVCs
Remove abandoned claims to free storage capacity.

## Monitoring Best Practices

### 1. Enable Application Logging
Use structured logging for easier parsing and analysis.

### 2. Set Up Alerts
Configure alerts for pod failures, node issues, and resource thresholds.

### 3. Use Prometheus Metrics
Expose application metrics for monitoring.

### 4. Implement Log Rotation
Prevent disk exhaustion from log accumulation.

### 5. Centralize Logging
Use OpenShift cluster logging for consolidated log management.

## Operator Best Practices

### 1. Monitor Operator Status
Regularly check `oc get co` for degraded operators.

### 2. Keep Operators Updated
Apply operator updates during maintenance windows.

### 3. Understand Operator Dependencies
Some operators depend on others; upgrade in order.

### 4. Back Up Operator Configuration
Store Custom Resource definitions for recovery.

### 5. Limit Operator Scope
Use namespace-scoped operators where possible.

## Upgrade Best Practices

### 1. Read Release Notes
Understand changes, deprecations, and breaking changes.

### 2. Test Upgrades in Staging
Validate cluster upgrade process in non-production environment.

### 3. Backup etcd
Ensure etcd backup is available before initiating upgrade.

### 4. Upgrade in Phases
Control plane first, then workers, then operators.

### 5. Plan Maintenance Window
Schedule upgrades during low-traffic periods.

### 6. Have Rollback Plan
Know how to downgrade if upgrade fails.

## References

- [Red Hat OpenShift Best Practices](https://docs.openshift.com/container-platform/latest/installing/index.html)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/overview/working-with-objects/best-practices/)
- [OpenShift Security Best Practices](https://docs.openshift.com/container-platform/latest/security/)
- [OpenShift Networking Best Practices](https://docs.openshift.com/container-platform/latest/networking/)