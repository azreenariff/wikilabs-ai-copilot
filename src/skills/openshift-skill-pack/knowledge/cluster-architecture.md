# OpenShift Cluster Architecture

## Overview

Red Hat OpenShift Container Platform is an enterprise-grade Kubernetes distribution that adds operational automation, security, and developer tooling on top of upstream Kubernetes.

## Architecture Components

### Control Plane (Master Nodes)

The control plane manages the cluster state and coordinates all cluster operations.

#### API Server (kube-apiserver)
- **Purpose**: Central management component and entry point for all cluster operations
- **Role**: Authenticates, validates, and routes API requests
- **Port**: 6443 (HTTPS)
- **Observables**:
  - Pods: `openshift-kube-apiserver`
  - Process: `kube-apiserver`
  - Configuration: `/etc/kubernetes/master/config.yaml`
  - Logs: `journalctl -u kube-apiserver`

#### etcd
- **Purpose**: Distributed key-value store for all cluster data
- **Role**: Persistent storage for cluster state (pods, deployments, configs, secrets)
- **Data**: All objects stored as JSON in etcd
- **Observables**:
  - Pods: `etcd` in `openshift-etcd` namespace
  - Port: 2379-2380
  - Health: `etcdctl endpoint health`
  - Logs: `journalctl -u etcd`

#### Controller Manager
- **Purpose**: Runs cluster-level controllers that regulate cluster state
- **Role**: Node controller, replication controller, endpoint controller, service account controller
- **Observables**:
  - Pod: `openshift-kube-controller-manager`
  - Process: `kube-controller-manager`

#### Scheduler
- **Purpose**: Assigns pods to nodes based on resource availability and constraints
- **Role**: Evaluates available nodes and places pods according to resource requirements, affinity rules, taints, tolerations
- **Observables**:
  - Pod: `openshift-kube-scheduler`
  - Process: `kube-scheduler`
  - Logs: Events in `oc get events` when scheduling fails

### Worker Nodes

Worker nodes run the actual workloads (application pods).

#### kubelet
- **Purpose**: Primary node agent running on each worker node
- **Role**: Ensures containers are running in pods as defined in pod specifications
- **Observables**:
  - Service: `kubelet.service`
  - Port: 10250 (HTTPS)
  - Logs: `journalctl -u kubelet`
  - Health: `oc debug node/<node> -- chroot /host journalctl -u kubelet`

#### kube-proxy
- **Purpose**: Maintains network rules on nodes for pod communication
- **Role**: Implements services (ClusterIP, NodePort, LoadBalancer) using iptables/IPVS
- **Observables**:
  - DaemonSet: `openshift-node` namespace
  - Mode: iptables or IPVS (configurable)

#### CNI Plugin (typically OVN-Kubernetes or SDN)
- **Purpose**: Implements pod networking
- **Role**: Assigns IPs to pods, routes traffic between pods and external networks
- **Types**:
  - OVN-Kubernetes (default in OpenShift 4.x)
  - OpenShift SDN (legacy, deprecated in 4.x)
- **Configuration**: ClusterNetwork CR

### OpenShift-Specific Components

#### Cluster Operators
- **Purpose**: Managed lifecycle of OpenShift platform components
- **Components monitored**: authentication, console, image-registry, ingress, networking, storage, etc.
- **Status**: `oc get co` shows Available/Progressing/Degraded status
- **Management**: Operators reconcile cluster state automatically

#### Image Registry
- **Purpose**: Internal container image storage and management
- **Route**: `image-registry.openshift-image-registry.svc:5000`
- **Observables**: `oc get routes -n openshift-image-registry`
- **Configuration**: `oc edit imageregistry.config.openshift.io/cluster`

#### Ingress Controller
- **Purpose**: Manages external access to services via routes
- **Role**: HAProxy-based reverse proxy for OpenShift routes
- **Observables**: `oc get ingresscontrollers -n openshift-ingress-operator`
- **Route**: Wildcard route for external traffic

#### Operators (OLM)
- **Purpose**: Operator Lifecycle Manager for installing and managing application operators
- **Role**: Install, upgrade, and manage operators across the cluster
- **Components**:
  - CatalogSource: Package repositories
  - Subscription: Desired operator version
  - InstallPlan: Approval workflow for updates
  - ClusterServiceVersion (CSV): Operator version and metadata

#### Machine Config Operator
- **Purpose**: Manages node configuration and ensures consistency
- **Role**: Deploys systemd units, kernel parameters, and system configuration to nodes
- **Components**:
  - MachineConfigPool: Groups of nodes with same configuration
  - MachineConfig: Individual configuration objects
  - Nodes transition: Updating → Rebooting → Ready

### Networking Architecture

#### Pod Networking
- Each pod gets a unique IP address across the cluster
- Pods can communicate directly without NAT
- CNI plugin handles IP assignment and routing
- Pod-to-pod communication uses overlay network (OVN-Kubernetes)

#### Service Networking
- **ClusterIP**: Internal-only virtual IP
- **NodePort**: Exposes service on each node's IP
- **LoadBalancer**: External load balancer (cloud provider)
- **ExternalIP**: Static external IP (less common)

#### Route Networking
- OpenShift-specific external access mechanism
- Wildcard DNS route for hostname-based routing
- TLS termination at route level (Edge, Passthrough, Reencrypt)
- HAProxy-based implementation

### Storage Architecture

#### Persistent Volumes (PV)
- Cluster-wide storage resources
- Can be provisioned dynamically or created statically
- Access modes: ReadWriteOnce (RWO), ReadOnlyMany (ROX), ReadWriteMany (RWX)
- Reclaim policies: Retain, Recycle, Delete

#### Persistent Volume Claims (PVC)
- Request for storage by a pod
- Bound to a specific PV
- Storage class determines provisioning behavior
- Can request specific size, access mode, and storage class

#### Storage Classes
- Defines different storage tiers (performance, backup, replication)
- Dynamic provisioning via storage provisioners (Ceph, NFS, cloud providers)
- Default storage class auto-assigns PV to PVCs
- Configuration: `oc get storageclass`

### Security Architecture

#### Security Context Constraints (SCC)
- Pod security policies for OpenShift
- Controls what pods can do (privileged, root, capabilities)
- Built-in SCCs: privileged, hostaccess, hostnetwork, hostpaths, nonroot, restricted
- Service accounts can be granted SCCs
- Configuration: `oc get scc`

#### RBAC (Role-Based Access Control)
- **Roles**: Permissions within a namespace
- **ClusterRoles**: Permissions cluster-wide
- **RoleBindings**: Assign roles to users/groups/service accounts
- **ClusterRoleBindings**: Assign cluster roles cluster-wide
- Built-in roles: admin, edit, view
- Configuration: `oc describe role <name>`

#### Network Policies
- Control pod-to-pod traffic within namespaces
- Default: all pods can communicate
- Policies restrict ingress and egress traffic
- Implements network segmentation
- Configuration: `oc apply -f networkpolicy.yaml`

### Monitoring Architecture

#### Cluster Monitoring (Prometheus-based)
- **Prometheus**: Metrics collection and storage
- **Alertmanager**: Alert routing and notification
- **Grafana**: Metrics visualization
- **Node Exporter**: Node-level metrics (CPU, memory, disk, network)
- **Kube State Metrics**: Kubernetes resource metrics
- **Alerts**: Predefined alerts for cluster health

#### Cluster Logging (Elasticsearch-based)
- **Elasticsearch**: Log storage and indexing
- **Kibana**: Log visualization
- **Fluentd**: Log collection from pods and nodes
- **Log Router**: Routes logs from various sources
- **Elasticsearch Operator**: Manages Elasticsearch clusters

### Node Lifecycle

#### Machine Sets
- Defines desired node configuration (count, type, labels, taints)
- Auto-scaling: Scale up/down based on workload demand
- Machine types: Cloud provider instance types
- Configuration: `oc get machinedeployment`, `oc get machineset`

#### Machine Config Pools
- Groups nodes by configuration state
- Nodes transition: Updating → Rebooting → Updated
- Auto-updates when MachineConfig changes
- Configuration: `oc get machineconfigpool`

#### Node Scheduling
- **Taints**: Nodes can reject pods (NoSchedule, NoExecute, PreferNoSchedule)
- **Tolerations**: Pods can tolerate node taints
- **Affinity**: Pod affinity/anti-affinity rules
- **NodeSelector**: Simple label-based scheduling
- **PriorityClass**: Pod priority for resource contention

## Version Management

### Update Channels
- **stable-4.x**: Recommended production channel
- **fast-4.x**: Earlier updates for testing
- **candidate-4.x**: Candidates before release
- **e4-x.y-z**: Specific version targeting

### Viewing Updates
```bash
oc get clusterversion
oc get clusterversion version -o yaml
```

### Upgrade Process
1. Set target version on clusterversion
2. Control plane pods update first
3. Worker nodes update via Machine Config Operator
4. Operators update automatically
5. Verify all operators Available=True

## High Availability

### Control Plane
- Minimum 3 master nodes for HA
- etcd requires odd number of members (3, 5)
- API servers load-balanced externally

### Worker Nodes
- Minimum 2 worker nodes for HA
- Pods distributed across nodes (anti-affinity)
- Service endpoints ensure redundancy

### Storage
- Distributed storage (Ceph, cloud providers)
- Replication across failure domains
- Backup and snapshot capabilities

## References

- [Red Hat OpenShift Architecture](https://docs.openshift.com/container-platform/latest/architecture/architecture.html)
- [Kubernetes Architecture](https://kubernetes.io/docs/concepts/architecture/)
- [OpenShift Components](https://docs.openshift.com/container-platform/latest/architecture/infrastructure-architecture.html)