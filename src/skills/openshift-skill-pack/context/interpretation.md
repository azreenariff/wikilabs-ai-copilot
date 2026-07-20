# OpenShift Context Interpretation

## Purpose

This document defines how to interpret different observation contexts when providing guidance.

## Terminal Context

### CLI Command Detection
When a user runs or shows oc/kubectl commands:

1. Identify command type (get, describe, logs, delete, etc.)
2. Identify resource type (pod, deployment, node, etc.)
3. Identify flags and options used
4. Map to documentation reference
5. Suggest relevant diagnostics

### Command Output Analysis
When a user shows command output:

1. Identify resource status and state
2. Check for error messages or warnings
3. Compare expected vs actual values
4. Recommend next diagnostic step
5. Provide interpretation guidance

### Example Interpretations

**Input**: `oc get pods` output showing CrashLoopBackOff
**Interpretation**: Pod is repeatedly crashing. Check events and logs.

**Input**: `oc get co` showing Degraded=True
**Interpretation**: Cluster operator has issues. Check operator logs and dependent operators.

**Input**: `oc describe pod` showing OOMKilled
**Interpretation**: Container exceeded memory limit. Increase limits or fix memory leak.

## Console Context

### Web Console Navigation
When a user interacts with the OpenShift web console:

1. Identify current page and resource
2. Map console actions to CLI equivalents
3. Suggest relevant CLI diagnostics
4. Provide web console navigation tips

### Console Error Messages
When a user shows console error messages:

1. Identify the error type and context
2. Map to known failure patterns
3. Provide CLI diagnostic commands
4. Suggest resolution steps

## Text Pattern Context

### Error Message Detection
When error messages are detected:

1. Match to known patterns
2. Provide root cause analysis
3. Suggest evidence collection
4. Recommend resolution steps
5. Include documentation references

## Cross-Context Integration

### Terminal + Console
When both terminal and console contexts are present:
1. Correlate findings from both sources
2. Provide unified diagnosis
3. Suggest both CLI and console actions
4. Document findings in both contexts

### Pattern + Resource
When text patterns and resource context are present:
1. Use patterns to identify issues
2. Use resource context to scope diagnosis
3. Provide targeted recommendations
4. Include specific resource commands

## Confidence Scoring

### High Confidence
- Clear error messages
- Matching detection rules
- Consistent evidence

### Medium Confidence
- Partial error messages
- Inconsistent evidence
- Multiple possible causes

### Low Confidence
- Ambiguous output
- Insufficient context
- Need more evidence

## References

- [OpenShift CLI Documentation](https://docs.openshift.com/container-platform/latest/cli_reference/openshift_cli/index.html)
- [OpenShift Web Console](https://docs.openshift.com/container-platform/latest/web_console/index.html)