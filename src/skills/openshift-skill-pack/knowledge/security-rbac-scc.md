# OpenShift Security — RBAC and SCC

## Overview

OpenShift extends Kubernetes RBAC with Security Context Constraints (SCC) for pod-level security controls.

## RBAC (Role-Based Access Control)

### Core Concepts

#### Roles and ClusterRoles
- **Roles**: Namespace-scoped permissions
- **ClusterRoles**: Cluster-wide permissions
- Can bind to: Users, Groups, ServiceAccounts

#### RoleBindings and ClusterRoleBindings
- **RoleBindings**: Grant roles within a namespace
- **ClusterRoleBindings**: Grant cluster roles across all namespaces
- Subjects: Users, Groups, ServiceAccounts

### Built-in Roles

#### admin
- Full access to most resources in a namespace
- Cannot manage roles, role bindings, or namespaces
- Use for: Team leads, senior engineers

#### edit
- Can create, modify, and delete most objects
- Cannot view or modify roles/rolebindings
- Cannot manage PVCs or storage
- Use for: Developers

#### view
- Read-only access to all objects
- Cannot create, modify, or delete anything
- Use for: Auditors, monitoring

#### self-accessor
- Can view own permissions
- Use for: Diagnostic purposes

### Managing Roles

```bash
# Create a role
oc create role <name> --verb=get,list,watch --resource=pods,deployments -n <namespace>

# Create a cluster role
oc create clusterrole <name> --verb=get,list --resource=nodes,pods

# Bind a role to a user
oc adm policy add-role-to-user <role> <user> -n <namespace>

# Bind a role to a service account
oc adm policy add-role-to-user <role> system:serviceaccount:<namespace>:<sa> -n <namespace>

# Check permissions
oc auth can-i create pods -n <namespace> --as=<user>
oc auth can-i --list --as=<user>
```

### RBAC Best Practices

1. **Follow least privilege** — Grant minimum required permissions
2. **Use service accounts** — Assign roles to service accounts, not individual users
3. **Audit regularly** — Review role bindings and permissions
4. **Use custom roles** — Create role sets for specific workflows
5. **Separate admin and edit** — Don't give admin to developers

## Security Context Constraints (SCC)

### Overview

SCCs control what pods can do at the kernel level. They enforce security boundaries beyond Kubernetes pod security standards.

### Built-in SCCs

#### restricted
- **Default for most users**
- Must run as non-root
- Cannot use privileged containers
- Cannot use host network, host PID, host ports
- Cannot use hostPath volumes
- Capabilities dropped: ALL
- Use for: All standard workloads

#### nonroot
- Can run as any non-root UID
- More flexible than restricted
- Still prohibits privileged containers
- Use for: Applications that need more flexibility but not root

#### privileged
- No restrictions
- Can run as root, use host namespaces, privileged containers
- Security risk — use only when absolutely necessary
- Use for: Monitoring agents, debugging, system components

#### hostaccess
- Can use hostPath volumes
- Cannot use host network or host PID
- Use for: Log collectors, monitoring agents

#### hostnetwork
- Can use host network namespace
- Can bind to any port
- Use for: Network troubleshooting, debugging

#### hostpaths
- Can mount host paths
- Limited to specific configured paths
- Use for: Log collection, monitoring

### Granting SCC Access

```bash
# Grant restricted SCC to a service account (default)
oc adm policy add-scc-to-user restricted -z default -n <namespace>

# Grant privileged SCC to a service account
oc adm policy add-scc-to-user privileged -z default -n <namespace>

# List SCCs assigned to a user/service account
oc adm policy can-i --list --as=system:serviceaccount:<namespace>:<sa>

# Check which SCC a pod would use
oc policy can-i create pods --as=system:serviceaccount:<namespace>:<sa>
```

### Security Context

#### Pod-Level Security Context
```yaml
securityContext:
  runAsUser: 1000
  runAsGroup: 3000
  fsGroup: 2000
  runAsNonRoot: true
  seLinuxOptions:
    level: s0:c123,c456
```

#### Container-Level Security Context
```yaml
securityContext:
  runAsNonRoot: true
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
    add:
      - NET_BIND_SERVICE  # Only if needed for port < 1024
```

### Security Context Best Practices

1. **Always set runAsNonRoot: true**
2. **Set readOnlyRootFilesystem: true** where possible
3. **Drop ALL capabilities** — add only what's needed
4. **Set allowPrivilegeEscalation: false**
5. **Use non-root UIDs** (1000+)
6. **Avoid privileged SCC** — use restricted or nonroot

## Network Policies

### Overview

Network policies control pod-to-pod traffic within and between namespaces. Default: all pods can communicate.

### Creating Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all
  namespace: myproject
spec:
  podSelector: {}
  policyTypes:
    - Ingress
    - Egress
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-frontend-to-backend
  namespace: myproject
spec:
  podSelector:
    matchLabels:
      app: backend
  policyTypes:
    - Ingress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: frontend
      ports:
        - protocol: TCP
          port: 8080
```

### Network Policy Best Practices

1. **Start with deny-all** — Explicitly allow required traffic
2. **Scope by namespace** — Policies only apply within their namespace
3. **Allow monitoring** — Don't block Prometheus metrics endpoints
4. **Allow DNS** — Pods need DNS access (port 53 UDP/TCP)
5. **Review regularly** — Policies can block legitimate traffic

## OAuth and Authentication

### OAuth Clients

- **openshift-challenging**: Default OAuth for CLI login
- **openshift-browser-client**: Web console OAuth
- **cli**: OpenShift CLI OAuth client

### Authentication Methods

- **htpasswd**: Local user/password
- **LDAP/AD**: Directory integration
- **GitHub/GitLab**: OAuth integration
- **OIDC**: OpenID Connect (recommended for enterprise)
- **Kerberos**: GSS-API authentication
- **Token**: Service account tokens

### Token Management

```bash
# Get current token
oc whoami --show-token

# Login with token
oc login --server=https://api.cluster.example.com:6443 --token=sha256~xxxxx

# Login with username/password (interactive)
oc login --server=https://api.cluster.example.com:6443 -u username -p password

# Refresh expired token
oc login --refresh
```

## Audit Logging

### Audit Policy

```yaml
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
  - level: RequestResponse
    resources:
      - group: ""
        resources: ["secrets"]
  - level: Metadata
    verbs: ["create", "update", "patch", "delete"]
  - level: None
    users: ["system:serviceaccount:kube-system:default"]
```

### Audit Best Practices

1. **Log all secret access** — RequestResponse level
2. **Log all mutations** — Metadata level for create/update/delete
3. **Exclude system service accounts** — Reduce noise
4. **Store audit logs** — Send to external SIEM
5. **Review regularly** — Monitor for unauthorized access

## References

- [OpenShift RBAC](https://docs.openshift.com/container-platform/latest/security/rbac/rbac.html)
- [Security Context Constraints](https://docs.openshift.com/container-platform/latest/security/scc.html)
- [Network Policies](https://docs.openshift.com/container-platform/latest/networking/network_policy/net-policy-authoring.html)
- [Audit Logging](https://docs.openshift.com/container-platform/latest/audit/audit-engine.html)