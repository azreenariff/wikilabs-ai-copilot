# OpenShift Common Failures Reference

## Failure Patterns

| Pattern | Severity | Frequency | Documentation |
|---------|----------|-----------|---------------|
| CrashLoopBackOff | High | Very common | /knowledge/pod-crashloopbackoff.md |
| OOMKilled | High | Very common | /knowledge/pod-oomkilled.md |
| ImagePullBackOff | High | Common | /knowledge/pod-imagepullbackoff.md |
| Node NotReady | Critical | Common | /knowledge/node-notready.md |
| FailedScheduling | Medium | Common | /knowledge/pod-pending.md |
| PVC Pending | Medium | Common | /knowledge/pvc-pending.md |
| Operator Degraded | High | Common | /knowledge/operator-degraded.md |
| Route Unavailable | Medium | Common | /knowledge/route-unavailable.md |
| Auth Failure | High | Common | /knowledge/auth-failure.md |
| DNS Failure | Medium | Less common | /knowledge/dns-failure.md |

## References

- [Red Hat OpenShift Troubleshooting](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)
- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)