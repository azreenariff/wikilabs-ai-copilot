# OpenShift Reasoning Reference

## Purpose

This document documents the structured reasoning patterns used for OpenShift issue diagnosis.

## Reasoning Framework

### Evidence-Based Reasoning
Every diagnosis must be based on evidence, not assumptions:

1. **Observe**: What is the current state?
2. **Collect**: Gather relevant evidence
3. **Analyze**: Match evidence to known patterns
4. **Diagnose**: Determine root cause
5. **Recommend**: Suggest resolution
6. **Verify**: Confirm fix works
7. **Document**: Record findings

### Decision Trees

Each failure pattern has an associated decision tree:

1. Start with symptom
2. Check events first
3. Check describe output
4. Check logs
5. Check resource configuration
6. Check related resources
7. Check cluster state
8. Make diagnosis

### Reasoning Safety

- Never assume — verify with evidence
- Consider all components in the system
- Document assumptions and limitations
- Provide rollback strategies
- Consider cascade effects

## Pattern Reference

See OPENSHIFT_REASONING_GUIDE.md for detailed reasoning patterns.

## References

- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)
- [OpenShift Troubleshooting](https://docs.openshift.com/container-platform/latest/support/troubleshooting/index.html)