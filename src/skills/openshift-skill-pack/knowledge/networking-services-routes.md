# OpenShift Networking — Services, Routes, and Network Policies

## Overview

OpenShift provides three layers of networking: pod networking, service networking, and route networking.

## Pod Networking

### OVN-Kubernetes (Default CNI)

OVN-Kubernetes is the default Container Network Interface (CNI) plugin for OpenShift 4.x.

- **Overlay network**: Uses VXLAN encapsulation for pod-to-pod communication
- **IP allocation**: Each pod gets a unique IP in the cluster IP address range
- **Routing**: OVN gateway mode provides access to external networks
- **Load balancing**: OVN implements service load balancing in the data plane

### Configuration

```bash
# View cluster network configuration
oc get clusternetwork -o yaml

# View OVN configuration
oc get networkoperator -n openshift-network-operator

# Check CNI plugin
oc get configmap cni-plugins -n openshift-network-operator -o yaml
```

### Troubleshooting Pod Networking

```bash
# Check pod IP assignment
oc get pods -o wide

# Test pod-to-pod connectivity
oc debug pod/<pod> -- ip route

# Check OVN logical topology
oc debug node/<node> -- ovn-nbctl show --bare

# Check network namespace
oc debug pod/<pod> -- ip netns exec
```

## Service Networking

### Service Types

#### ClusterIP (default)
- Internal virtual IP accessible only within the cluster
- Used for internal service-to-service communication
- No external access

#### NodePort
- Exposes service on each node's IP at a static port
- Accessible from outside the cluster via `<node-ip>:<node-port>`
- Useful for testing or simple external access

#### LoadBalancer
- Provisions an external load balancer (cloud provider)
- Assigns an external IP
- Used for production external services

#### ExternalName
- Maps a service to a DNS name (CNAME)
- Does not create a ClusterIP
- Useful for proxying to external services

### Service Discovery

#### DNS
- Services get DNS names in the format: `<service>.<namespace>.svc.cluster.local`
- Short name `<service>.<namespace>` also works within the same namespace
- CoreDNS runs as a pod in `openshift-dns` namespace

```bash
# Check DNS pods
oc get pods -n openshift-dns

# Test DNS resolution
oc debug pod/<pod> -- nslookup <service-name>

# Check DNS configuration
oc get endpoints dns-default -n openshift-dns
```

### Endpoints

Endpoints track which pods back a service:

```bash
# Check endpoints
oc get endpoints <service-name> -n <namespace>

# Detailed endpoint info
oc describe endpoints <service-name> -n <namespace>

# Watch endpoints in real-time
oc get endpoints <service-name> -n <namespace> -w
```

### Service Port Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
  namespace: myproject
spec:
  selector:
    app: myapp
  ports:
    - name: http
      protocol: TCP
      port: 80
      targetPort: 8080
      nodePort: 30080  # Only for NodePort type
  type: ClusterIP
```

### Service Troubleshooting

```bash
# Verify service exists
oc get svc <service-name> -n <namespace>

# Check service endpoints
oc get endpoints <service-name> -n <namespace>

# Check service pod selector
oc get svc <service-name> -n <namespace> -o yaml | grep selector

# Verify pods match selector
oc get pods -l <selector-labels> -n <namespace>

# Test service from within a pod
oc debug pod/<pod> -n <namespace> -- curl http://<service>:<port>

# Check kube-proxy status
oc get pods -n openshift-network-operator | grep kube-proxy
```

## Route Networking

### Routes

OpenShift routes provide external access to services via hostname-based routing.

#### Route Components

- **Host**: The hostname for the route (e.g., `app.example.com`)
- **Path**: URL path prefix (optional)
- **Service**: The backend service and port
- **TLS**: Transport Layer Security configuration
- **Wildcards**: Cluster can have a wildcard route for all subdomains

#### Route TLS Termination Types

| Type | Behavior | Certificate Management |
|------|----------|------------------------|
| Edge | TLS terminates at router; traffic to service is HTTP | Certificate managed by user |
| Passthrough | TLS encrypted end-to-end; router cannot inspect | Certificate on backend |
| Reencrypt | TLS terminates at router; re-encrypts to backend | Both user and backend certificates |
| Allow | Traffic reaches service in any encryption state | No enforcement |

#### Creating Routes

```yaml
apiVersion: route.openshift.io/v1
kind: Route
metadata:
  name: myapp-route
  namespace: myproject
spec:
  host: app.example.com
  to:
    kind: Service
    name: myapp-service
    weight: 100
  port:
    targetPort: http
  tls:
    termination: edge
    insecureEdgeTerminationPolicy: Redirect
```

### Route Troubleshooting

```bash
# List routes
oc get routes -n <namespace> -o wide

# Describe a route
oc describe route <route-name> -n <namespace>

# Check route host resolution
nslookup <route-hostname>

# Check route admission controller
oc get pods -n openshift-ingress-operator

# Test route accessibility
curl -k https://<route-hostname>

# Check endpoint readiness
oc get endpoints <service-name> -n <namespace>
```

### Route Best Practices

1. **Use consistent hostnames** — Follow a pattern (e.g., `app.env.example.com`)
2. **Enable TLS redirect** — Use `insecureEdgeTerminationPolicy: Redirect`
3. **Use reencrypt for sensitive apps** — Maintain encryption to backend
4. **Monitor route health** — Set up alerts for route failures
5. **Avoid overlapping hosts** — One hostname per route per cluster
6. **Use path-based routing** — For multiple apps on same hostname

## Network Policies

### NetworkPolicy API

NetworkPolicy controls pod-to-pod traffic within a namespace.

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
  name: allow-frontend
  namespace: myproject
spec:
  podSelector:
    matchLabels:
      app: frontend
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: client
      ports:
        - protocol: TCP
          port: 8080
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: backend
      ports:
        - protocol: TCP
          port: 8080
```

### Default Behavior

- **No policies**: All pods can communicate freely
- **Deny ingress**: Pods cannot receive traffic (unless allowed)
- **Deny egress**: Pods cannot send traffic (unless allowed)
- **Deny both**: Pod is isolated (unless allowed)

### Network Policy Best Practices

1. **Start with deny-all** — Explicitly allow required traffic
2. **Always allow DNS** — Pods need DNS resolution (port 53)
3. **Allow monitoring** — Prometheus metrics endpoints
4. **Allow intra-service communication** — Service-to-service
5. **Review regularly** — Policies can cause unexpected blockages

### Network Policy Troubleshooting

```bash
# List network policies
oc get networkpolicies -n <namespace>

# Describe a network policy
oc describe networkpolicy <policy-name> -n <namespace>

# Test connectivity from debug pod
oc debug pod/<pod> -n <namespace> -- curl http://<target-pod>:<port>

# Check if policy matches pod
oc get pods <pod> -n <namespace> -o jsonpath='{.metadata.labels}'
```

## DNS

### CoreDNS Configuration

CoreDNS handles DNS resolution for services and external domains.

```bash
# Check DNS pods
oc get pods -n openshift-dns

# View CoreDNS config
oc get configmap dns-default -n openshift-dns -o yaml

# Test DNS resolution
oc debug pod/<pod> -- nslookup kubernetes.default.svc.cluster.local
oc debug pod/<pod> -- nslookup external-domain.com

# Check DNS forwarder
oc get configmap dns-default -n openshift-dns
```

### DNS Best Practices

1. **Use fully qualified names** — `service.namespace.svc.cluster.local`
2. **Monitor DNS latency** — Slow DNS affects application performance
3. **Configure forwarders** — Add external DNS resolvers as needed
4. **Cache DNS** — CoreDNS caches by default; tune cache settings

## References

- [OpenShift Networking](https://docs.openshift.com/container-platform/latest/networking/)
- [Kubernetes Services](https://kubernetes.io/docs/concepts/services-networking/service/)
- [Network Policies](https://kubernetes.io/docs/concepts/services-networking/network-policies/)
- [OVN-Kubernetes](https://docs.openshift.com/container-platform/latest/networking/ovn_kubernetes_network_provider/understanding-ovn-kubernetes.html)