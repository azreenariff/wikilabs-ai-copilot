# OpenShift Container Runtime — Pods, Deployments, and Workloads

## Overview

OpenShift manages workloads through Kubernetes primitives: Pods, Deployments, ReplicaSets, StatefulSets, DaemonSets, Jobs, and CronJobs.

## Pods

### Pod Basics

A Pod is the smallest deployable unit in Kubernetes/OpenShift. It contains one or more containers that share network and storage.

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: myapp
  namespace: myproject
  labels:
    app: myapp
spec:
  containers:
    - name: myapp
      image: registry.example.com/myapp:1.0.0
      ports:
        - containerPort: 8080
      resources:
        requests:
          memory: "256Mi"
          cpu: "250m"
        limits:
          memory: "512Mi"
          cpu: "500m"
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

### Pod Lifecycle

1. **Pending** — Pod accepted, containers not yet created
2. **ContainerCreating** — Images being pulled, containers starting
3. **Running** — Container running (or initiating restarts)
4. **Succeeded** — All containers terminated successfully (Jobs)
5. **Failed** — All containers terminated with failure
6. **Unknown** — Pod state cannot be determined

### Pod Status Fields

```bash
# Check pod status
oc get pods -n <namespace>
# Output: NAME READY STATUS RESTARTS AGE

# Detailed pod information
oc describe pod <pod-name> -n <namespace>
```

### Multi-Container Pods

Sidecar containers are common in OpenShift:

```yaml
spec:
  containers:
    - name: app
      image: myapp:1.0.0
    - name: log-agent
      image: fluentd:latest
    - name: config-init
      image: config-init:latest
      command: ["/bin/sh", "-c", "cp /config/*.conf /etc/app/ && exit 0"]
      volumeMounts:
        - name: config-volume
          mountPath: /config
  volumes:
    - name: config-volume
      configMap:
        name: app-config
```

### Troubleshooting Pods

```bash
# List pods with all statuses
oc get pods -n <namespace> -o wide

# Check pod events
oc describe pod <pod-name> -n <namespace>

# View logs
oc logs <pod-name> -n <namespace>
oc logs <pod-name> -n <namespace> -c <container-name>

# View previous logs (after crash)
oc logs <pod-name> -n <namespace> --previous

# Follow logs in real-time
oc logs <pod-name> -n <namespace> -f

# Execute command in pod
oc exec -it <pod-name> -n <namespace> -- /bin/bash

# Debug session
oc debug pod/<pod-name> -n <namespace>
```

## Deployments

### Deployment Overview

Deployments manage stateless applications with rolling updates and rollbacks.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  namespace: myproject
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
        - name: myapp
          image: registry.example.com/myapp:1.0.0
          ports:
            - containerPort: 8080
```

### Deployment Strategy

| Strategy | Description | Use Case |
|----------|-------------|----------|
| RollingUpdate | Gradually replaces old pods with new | Zero-downtime deployments |
| Recreate | Kills all old pods, then creates new | Stateful apps that cannot run multiple versions |

### Rolling Update Process

1. Create new ReplicaSet with updated configuration
2. Gradually scale up new ReplicaSet
3. Gradually scale down old ReplicaSet
4. Repeat until all pods use new configuration
5. Clean up old ReplicaSets

### Deployment Commands

```bash
# List deployments
oc get deployments -n <namespace>

# Check rollout status
oc rollout status deployment/<name> -n <namespace>

# View rollout history
oc rollout history deployment/<name> -n <namespace>

# Undo deployment
oc rollout undo deployment/<name> -n <namespace>
oc rollout undo deployment/<name> -n <namespace> --to-revision=2

# Restart deployment
oc rollout restart deployment/<name> -n <namespace>

# Update deployment image
oc set image deployment/<name> <container>=<image> -n <namespace>

# Scale deployment
oc scale deployment/<name> --replicas=5 -n <namespace>
```

### Deployment Troubleshooting

```bash
# Check deployment status
oc describe deployment/<name> -n <namespace>

# Check associated ReplicaSets
oc get replicasets -n <namespace> -l app=myapp

# Check pods from deployment
oc get pods -n <namespace> -l app=myapp

# Check rollout progress
oc rollout status deployment/<name> -n <namespace> --timeout=120s

# View deployment conditions
oc describe deployment/<name> -n <namespace> | grep "Conditions"
```

## ReplicaSets

### ReplicaSet Overview

ReplicaSets ensure the desired number of pod replicas are running.

```yaml
apiVersion: apps/v1
kind: ReplicaSet
metadata:
  name: myapp-rs
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
        - name: myapp
          image: registry.example.com/myapp:1.0.0
```

### ReplicaSet Commands

```bash
# List ReplicaSets
oc get replicasets -n <namespace>

# Describe ReplicaSet
oc describe replicaset <rs-name> -n <namespace>

# Scale ReplicaSet
oc scale replicaset <rs-name> --replicas=5 -n <namespace>
```

## StatefulSets

### StatefulSet Overview

StatefulSets manage stateful applications with stable identities and storage.

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: database
spec:
  serviceName: database
  replicas: 3
  selector:
    matchLabels:
      app: database
  template:
    metadata:
      labels:
        app: database
    spec:
      containers:
        - name: db
          image: postgres:15
          ports:
            - containerPort: 5432
          volumeMounts:
            - name: data
              mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 10Gi
```

### StatefulSet Features

- **Stable pod names**: database-0, database-1, database-2
- **Stable storage**: Each pod gets its own PersistentVolumeClaim
- **Ordered deployment**: Pods created in order (0, 1, 2)
- **Ordered deletion**: Pods deleted in reverse order (2, 1, 0)
- **Headless service**: Required for DNS-based pod discovery

### StatefulSet Commands

```bash
# List StatefulSets
oc get statefulsets -n <namespace>

# Check pod identity
oc get pods -n <namespace> -l app=database

# Scale StatefulSet
oc scale statefulset database --replicas=5 -n <namespace>

# Update StatefulSet
oc set image statefulset/database db=postgres:16 -n <namespace>
```

## DaemonSets

### DaemonSet Overview

DaemonSets ensure a pod runs on every (or selected) node.

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: log-collector
spec:
  selector:
    matchLabels:
      app: log-collector
  template:
    metadata:
      labels:
        app: log-collector
    spec:
      tolerations:
        - effect: NoSchedule
          key: node-role.kubernetes.io/master
      containers:
        - name: fluentd
          image: fluentd:latest
          volumeMounts:
            - name: varlog
              mountPath: /var/log
      volumes:
        - name: varlog
          hostPath:
            path: /var/log
```

### DaemonSet Use Cases

- Log collectors (Fluentd, Filebeat)
- Monitoring agents (node-exporter, Datadog)
- Network plugins (CNI plugins)
- Security agents (Falco, Wazuh)

### DaemonSet Commands

```bash
# List DaemonSets
oc get daemonsets -n <namespace>

# Check pods per node
oc get pods -n <namespace> -o wide -l app=log-collector

# Describe DaemonSet
oc describe daemonset <name> -n <namespace>
```

## Jobs and CronJobs

### Job Overview

Jobs run pods to completion.

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: migrate-database
spec:
  template:
    spec:
      containers:
        - name: migrate
          image: myapp:migration
          command: ["./migrate.sh"]
      restartPolicy: Never
  completions: 1
  parallelism: 1
```

### CronJob Overview

CronJobs run Jobs on a schedule.

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: daily-backup
spec:
  schedule: "0 2 * * *"  # Every day at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: backup
              image: myapp:backup
          restartPolicy: OnFailure
```

### Job Commands

```bash
# List jobs
oc get jobs -n <namespace>

# Check job status
oc describe job <name> -n <namespace>

# View job logs
oc logs job/<name> -n <namespace>

# List cronjobs
oc get cronjobs -n <namespace>

# Manually run a cronjob
oc create job --from=cronjob/daily-backup manual-backup -n <namespace>
```

## Health Probes

### Types of Probes

| Type | Purpose | Failure Action |
|------|---------|----------------|
| Liveness | Detect if container needs restart | Kill and restart container |
| Readiness | Detect if ready to receive traffic | Remove from service endpoints |
| Startup | Detect if slow-starting container is ready | Wait before other probes run |

### Probe Configuration

```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  failureThreshold: 3
  timeoutSeconds: 5

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  successThreshold: 1
  failureThreshold: 3

startupProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 0
  periodSeconds: 10
  failureThreshold: 30
```

### Probe Best Practices

1. **Always configure health probes** — Enable automatic recovery
2. **Set appropriate delays** — Allow application startup time
3. **Use different endpoints** — `/healthz` for liveness, `/ready` for readiness
4. **Start with startup probe** — Prevent premature liveness failures
5. **Configure failure thresholds** — Avoid restart loops

### Probe Troubleshooting

```bash
# Check probe status in pod describe
oc describe pod <pod> -n <namespace> | grep -A 10 "Probe"

# Test probe endpoint manually
oc debug pod/<pod> -n <namespace> -- curl http://localhost:8080/healthz

# Check probe events
oc get events -n <namespace> --field-selector reason=Unhealthy
```

## Resource Management

### Resource Classes

| QoS Class | Requirements | Guarantee | Use For |
|-----------|-------------|-----------|---------|
| Guaranteed | requests == limits | Highest | Critical workloads |
| Burstable | requests < limits | Best effort | Standard workloads |
| BestEffort | No requests/limits | None | Development/testing |

### Resource Commands

```bash
# Check resource usage
oc top pods -n <namespace>
oc top nodes

# Check resource configuration
oc describe pod <pod> -n <namespace> | grep -A 5 "Limits\|Requests"

# Set resources on deployment
oc set resources deployment/<name> --limits=memory=512Mi --requests=memory=256Mi -n <namespace>
```

## References

- [Kubernetes Pods](https://kubernetes.io/docs/concepts/workloads/pods/)
- [Kubernetes Deployments](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)
- [Kubernetes StatefulSets](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/)
- [OpenShift Container Runtime](https://docs.openshift.com/container-platform/latest/nodes/pod_nodes_nodes_multiple_pods.html)