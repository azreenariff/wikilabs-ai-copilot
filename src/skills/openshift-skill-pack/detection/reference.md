# OpenShift Detection Reference

## Detection Types

| Type | Description | Examples |
|------|-------------|----------|
| Command | CLI command patterns | oc get pods, oc describe pod |
| Browser | Web console patterns | console.openshift, openshift-web-console |
| TextPattern | Text pattern matching | CrashLoopBackOff, OOMKilled, NotReady |

## Detection Rules

See detection_rules.yaml for the complete detection rule configuration.

## Confidence Levels

- 0.95+: Near-certain match
- 0.90-0.94: High confidence
- 0.85-0.89: Medium-high confidence
- 0.80-0.84: Medium confidence
- 0.70-0.79: Low-medium confidence
- <0.70: Low confidence

## References

- [OpenShift Documentation](https://docs.openshift.com/container-platform/)
- [Kubernetes Reference](https://kubernetes.io/docs/reference/)