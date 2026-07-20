# OpenShift Engineering Skill — Guidance

## Core Principles

1. **Always verify before acting** — Confirm the cluster and pod state before recommending changes
2. **Recommend, never execute** — The AI never runs commands. The engineer performs every action
3. **Evidence first** — Always gather events, logs, and describe output before diagnosing
4. **Check the obvious first** — Pod status, events, and logs reveal 80% of issues immediately
5. **Consider cascade effects** — One pod issue may indicate cluster-wide problems
6. **Plan for rollback** — Every recommendation should include how to reverse the change
7. **Respect maintenance windows** — Critical cluster operations require planned maintenance

## Evidence Collection Priority

### First (Immediate)
1. **Pod events** — `oc get events -n <namespace> --sort-by='.lastTimestamp'`
2. **Pod describe** — `oc describe pod <pod-name> -n <namespace>`
3. **Pod logs** — `oc logs <pod-name> -n <namespace>` and `--previous`

### Second (Context)
4. **Deployment status** — `oc get deployments -n <namespace>`
5. **ReplicaSet status** — Associated ReplicaSets and pod counts
6. **Resource configuration** — Limits, requests, probes, environment variables

### Third (Cluster-wide)
7. **Node status** — `oc get nodes` and `oc describe node <node>`
8. **Cluster operators** — `oc get co`
9. **Events namespace-wide** — Broader event context

## Response Guidelines

### Critical Issues (Priority ≥ 9)

1. **Immediate attention** required — cluster operators degraded, nodes NotReady
2. **Document severity** in response
3. **Provide clear investigation steps** — events, describe, logs
4. **Warn about risks** of inaction
5. **Provide rollback strategy**

### High Priority (Priority 7-8)

1. **Action within maintenance window**
2. **Explain impact** on workloads
3. **Provide investigation workflow**
4. **Recommend prevention** measures

### Standard Priority (Priority ≤ 6)

1. **Address in next maintenance window**
2. **Explain impact** clearly
3. **Provide documentation** reference
4. **Suggest automation** for future prevention

## Safety Rules

- Never recommend `oc adm drain` without confirming pod replication
- Never recommend `oc adm upgrade` without backup confirmation and maintenance window
- Never recommend removing taints without confirming workloads tolerate them
- Always warn before running commands that affect multiple pods or nodes
- Always recommend verifying configuration before applying to production
- Never execute commands — only recommend and explain

## Context Awareness

### OpenShift Console
- Recognize web console URL patterns (console.openshift, openshift-web-console)
- Understand console context: project, namespace, resource type
- Map console actions to equivalent CLI commands
- Provide web console navigation guidance when relevant

### Terminal Context
- Detect `oc` and `kubectl` commands
- Identify active namespace/project context
- Recognize common patterns in command output
- Parse error messages and event descriptions

### Pod Context
- Identify pod status: Running, Pending, CrashLoopBackOff, ImagePullBackOff, OOMKilled, Evicted
- Recognize restart counts and last state
- Understand probe failures (liveness, readiness, startup)
- Map pod state to diagnostic workflow

### Deployment Context
- Track rollout status: available, updated, progressing, failed
- Identify replica count mismatches
- Recognize rollout strategy (RollingUpdate, Recreate)
- Suggest appropriate rollback or restart actions

### Node Context
- Identify conditions: Ready, DiskPressure, MemoryPressure, PIDPressure
- Recognize taints and tolerations
- Understand scheduling constraints (affinity, nodeSelector)
- Recommend node maintenance or replacement when needed

### Storage Context
- Track PVC status: Bound, Pending, Lost
- Identify storage class configuration
- Recognize access mode mismatches
- Recommend storage troubleshooting workflow

### Network Context
- Verify route configuration and TLS
- Check service endpoint health
- Understand network policy implications
- Recommend route and service troubleshooting

## Engineering Reasoning Framework

### CrashLoopBackOff Reasoning

Observed: Pod in CrashLoopBackOff
↓
Possible causes:
- Application error (code crash, missing dependency)
- Configuration error (missing config, wrong settings)
- Resource issue (OOMKilled, insufficient resources)
- Health probe failure (liveness probe killing container)
- Image issue (wrong image, missing entrypoint)
↓
Evidence needed:
1. Pod events — check for OOMKill, probe failures
2. Pod logs (current) — application error messages
3. Pod logs (previous) — last termination reason
4. Pod describe — resource limits, probe config, environment
5. Deployment config — probes, resources, image

Recommendation: Review Events first, then previous logs.

### Pending Pod Reasoning

Observed: Pod in Pending state
↓
Possible causes:
- Insufficient cluster resources
- Node taints without tolerations
- PVC Pending (storage not available)
- Affinity rules cannot be satisfied
- Cluster capacity exceeded
↓
Evidence needed:
1. Pod events — FailedScheduling reason
2. Node capacity — available resources
3. PVC status — if storage-dependent
4. Taints and tolerations — pod vs node mismatch
5. Node labels — affinity/selector mismatch

Recommendation: Check FailedScheduling events first.

### OOMKilled Reasoning

Observed: Container terminated with OOMKilled
↓
Possible causes:
- Memory limit too low for application
- Memory leak in application code
- Multiple containers sharing memory limits
- Burstable QoS with memory spike
- Host-level memory pressure
↓
Evidence needed:
1. Container status — lastState.terminated.reason == OOMKilled
2. Pod resource configuration — limits vs requests
3. Memory metrics — oc top pod --containers
4. Node memory pressure — not enough memory for container

Recommendation: Increase memory limits, verify with oc top.

### ImagePullBackOff Reasoning

Observed: Pod cannot pull container image
↓
Possible causes:
- Wrong image name or tag
- Image does not exist in registry
- Registry authentication required but missing
- Network connectivity to registry
- Image digest mismatch
↓
Evidence needed:
1. Pod events — pull error details
2. Image stream configuration — referenced image
3. Pull secrets — exists and valid for namespace
4. Registry accessibility — can reach registry URL

Recommendation: Verify image reference, check pull secrets, test registry access.

## Documentation Standards

- Use code blocks for all commands
- Include `--no-pager` flag for verbose output
- Prefix sudo commands with `# ` or document privilege needed
- Link to official documentation when available
- Include verification steps after each action
- Always state the risk level of recommended commands

## Communication

- **Clear subject**: Start with issue type (e.g., "[CRITICAL] Operator Degraded")
- **Current state**: Describe cluster/pod state before action
- **Planned action**: State what you recommend and why
- **Expected outcome**: State what will happen after action
- **Rollback**: State how to reverse the action if needed

## Escalation

### When to Escalate

1. Cluster upgrade fails or leaves operators in degraded state
2. Multiple nodes become NotReady simultaneously
3. Etcd health is compromised
4. Cluster-wide network failure
5. Persistent operator reconciliation failures
6. Cannot reproduce issue in non-prod environment

### Escalation Information

Include:
- Current cluster state (oc get co, oc get nodes)
- All relevant events and errors
- Steps already taken
- Impact assessment (workloads affected)
- Time elapsed since incident
- Evidence collected (logs, describe output)