# Networking Engineering Foundation

## Architecture

### OSI Model (7 Layers)

| Layer | Name | Function | Examples |
|-------|------|----------|----------|
| 7 | Application | User-facing protocols | HTTP, HTTPS, FTP, SSH, DNS, SMTP |
| 6 | Presentation | Data formatting/encryption | TLS, SSL, SSL, SSL, SSL, SSL, SSL, SSL, SSL, SSL, SSL, SSL, SSL, SSL |
| 5 | Session | Connection management | NetBIOS, RPC, SIP |
| 4 | Transport | End-to-end delivery | TCP, UDP, SCTP |
| 3 | Network | Routing and addressing | IP, IPv4, IPv6, ICMP, IPSec |
| 2 | Data Link | Node-to-node delivery | Ethernet, VLAN, ARP, PPP |
| 1 | Physical | Physical transmission | Cables, connectors, radio waves |

### TCP/IP Model (4 Layers)

| Layer | OSI Equivalent | Protocol Examples |
|-------|---------------|-------------------|
| Application | 5-7 | HTTP, DNS, SSH, FTP, SMTP |
| Transport | 4 | TCP, UDP |
| Internet | 3 | IP, ICMP, ARP |
| Network Access | 1-2 | Ethernet, WiFi, PPP |

---

## Core Concepts

### IP Addressing

**IPv4**
- 32-bit addresses (e.g., 192.168.1.100)
- Subnet masks (e.g., /24, 255.255.255.0)
- Private ranges:
  - 10.0.0.0/8
  - 172.16.0.0/12
  - 192.168.0.0/16
- Public IPs assigned by ISPs

**IPv6**
- 128-bit addresses (e.g., 2001:0db8::1)
- No NAT required
- Auto-configuration (SLAAC)
- Link-local addresses (fe80::/10)

**Subnetting**
| Prefix | Mask | Hosts | Use Case |
|--------|------|-------|----------|
| /24 | 255.255.255.0 | 254 | Standard LAN |
| /23 | 255.255.254.0 | 510 | Large LAN |
| /25 | 255.255.255.128 | 126 | Segment |
| /26 | 255.255.255.192 | 62 | Small segment |
| /30 | 255.255.255.252 | 2 | Point-to-point |

### Routing

**Static Routing**
- Manually configured routes
- Predictable, no overhead
- No failover unless configured

**Dynamic Routing**
- OSPF (Open Shortest Path First) — link-state, IGP
- BGP (Border Gateway Protocol) — path-vector, EGP
- EIGRP (Enhanced IGP) — Cisco proprietary
- RIP (Routing Information Protocol) — distance vector, legacy

**Routing Table**
- Shows destination networks, gateways, interfaces, metrics
- Longest prefix match determines route selection

### Switching

**Layer 2 Switching**
- MAC address tables for frame forwarding
- VLANs for network segmentation
- Spanning Tree Protocol (STP) for loop prevention

**Layer 3 Switching (Multilayer)**
- IP routing at wire speed
- ACLs for traffic filtering
- QoS for traffic prioritization

### DNS (Domain Name System)

**Record Types**
| Record | Purpose |
|--------|---------|
| A | IPv4 address |
| AAAA | IPv6 address |
| CNAME | Alias |
| MX | Mail server |
| NS | Name server |
| SOA | Zone authority |
| TXT | Arbitrary text (SPF, DKIM, verification) |
| SRV | Service location |

**DNS Flow**
1. Client queries local DNS resolver
2. Resolver checks cache
3. If not cached: root → TLD → authoritative name server
4. Result cached for TTL duration

### DHCP (Dynamic Host Configuration Protocol)

**DHCP Process (DORA)**
1. **Discover** — Client broadcasts DHCPDISCOVER
2. **Offer** — Server responds with DHCPOFFER
3. **Request** — Client sends DHCPREQUEST
4. **Acknowledge** — Server sends DHCPACK

**DHCP Options**
- Option 3: Router (default gateway)
- Option 6: DNS servers
- Option 15: Domain name
- Option 53: Message type
- Option 51: Lease time

### HTTP/HTTPS

**HTTP Methods**
| Method | Purpose |
|--------|---------|
| GET | Retrieve resource |
| POST | Submit data |
| PUT | Update resource |
| DELETE | Remove resource |
| PATCH | Partial update |
| HEAD | Get headers only |
| OPTIONS | Get allowed methods |

**HTTP Status Codes**
| Code | Meaning |
|------|---------|
| 200 | OK |
| 201 | Created |
| 204 | No Content |
| 301 | Moved Permanently |
| 302 | Found (redirect) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 429 | Too Many Requests |
| 500 | Internal Server Error |
| 502 | Bad Gateway |
| 503 | Service Unavailable |

**HTTPS/TLS**
- TLS 1.2 and 1.3 recommended
- TLS 1.0/1.1 deprecated
- Certificate validation chain: Server → Intermediate CA → Root CA
- SNI (Server Name Indication) for multiple certs on one IP

### SSH (Secure Shell)

**SSH Components**
- SSH-2 protocol (SSH-1 is insecure)
- Public key authentication (recommended)
- Password authentication (fallback)
- Port forwarding (local, remote, dynamic)
- X11 forwarding (GUI over SSH)

**SSH Config**
- `/etc/ssh/sshd_config` — Server configuration
- `~/.ssh/config` — Client configuration
- `~/.ssh/known_hosts` — Known server keys
- `~/.ssh/authorized_keys` — Authorized public keys

### Load Balancing

**Types**
| Type | Description | Examples |
|------|-------------|----------|
| L4 (Transport) | Routes based on IP/port | HAProxy L4, F5 |
| L7 (Application) | Routes based on HTTP headers/URL | NGINX, HAProxy L7, Apache |
| DNS Round Robin | Distributes via DNS responses | Cloudflare, AWS Route53 |
| Hardware | Dedicated appliances | F5 BIG-IP, Citrix ADC |
| Software | Application-level | NGINX, HAProxy, Envoy |

**Algorithms**
- Round Robin
- Least Connections
- IP Hash
- Weighted distribution

### Firewalls

**Types**
| Type | Function |
|------|----------|
| Packet Filter | Inspects headers (L3/L4) |
| Stateful Firewall | Tracks connection state |
| Application Firewall (WAF) | Inspects HTTP content |
| Next-Gen Firewall | Deep packet inspection, IDS/IPS |
| Host Firewall | Per-machine filtering (iptables, Windows FW) |

**Rules**
- Evaluated top to bottom
- First match wins
- Default deny (or default allow) policy
- Stateful rules track established connections

### NAT (Network Address Translation)

**Types**
| Type | Description |
|------|-------------|
| PAT (Port NAT) | Many-to-one, uses port numbers |
| Static NAT | One-to-one mapping |
| Dynamic NAT | Many-to-many pool |
| NAT64 | IPv6 to IPv4 translation |

### VPN (Virtual Private Network)

**Types**
| Type | Protocol | Use Case |
|------|----------|----------|
| Site-to-Site | IPsec, WireGuard | Connect offices/datacenters |
| Remote Access | OpenVPN, WireGuard, IPsec | Remote workers |
| SSL/TLS VPN | Browser-based | Zero-trust access |
| MPLS | Layer 2 | Carrier networks |

---

## Common Failures

### Connectivity Failures
| Symptom | Possible Cause |
|---------|----------------|
| No connectivity at all | Cable unplugged, interface down, DHCP failure |
| Can ping but can't browse | DNS failure, proxy misconfigured |
| Can browse but can't SSH | Firewall blocking port 22, SSH service down |
| Intermittent connectivity | Loose cable, wireless interference, routing flapping |

### DNS Failures
| Symptom | Possible Cause |
|---------|----------------|
| nslookup fails | DNS server unreachable, firewall blocking port 53 |
| Slow DNS resolution | DNS server overloaded, wrong DNS server configured |
| Wrong IP returned | Stale cache, wrong DNS record, DNS hijacking |
| CNAME loop | Misconfigured alias chain |

### Routing Failures
| Symptom | Possible Cause |
|---------|----------------|
| No route to host | Missing route, gateway down |
| Asymmetric routing | Return path via different route, firewall blocks |
| Routing loop | Misconfigured static routes, routing protocol issue |
| Suboptimal routing | Missing lower-metric routes, ECMP imbalance |

### TLS/SSL Failures
| Symptom | Possible Cause |
|---------|----------------|
| Certificate expired | Certificate renewal missed |
| Certificate mismatch | Wrong CN/SAN, self-signed in production |
| SSL handshake fails | Protocol mismatch, cipher incompatibility |
| OCSP/Stapling fails | OCSP responder unreachable |

### Performance Issues
| Symptom | Possible Cause |
|---------|----------------|
| High latency | Long distance, congestion, DNS resolution delay |
| Packet loss | Hardware failure, congestion, buffer overflow |
| Low throughput | Bandwidth limit, TCP window size, MTU mismatch |
| Jitter | Congestion, QoS misconfiguration |

---

## Troubleshooting Philosophy

### Network Diagnostic Flow

```
Network issue
  │
  ├─→ Can the host ping itself?
  │   └─→ No → Network stack broken, reinstall NIC driver
  │
  ├─→ Can the host ping gateway?
  │   └─→ No → Layer 2 issue (cable, VLAN, switch port)
  │
  ├─→ Can the host ping external IP (8.8.8.8)?
  │   └─→ No → Gateway issue, routing problem, ISP issue
  │
  ├─→ Can the host resolve DNS? (nslookup google.com)
  │   └─→ No → DNS server issue, /etc/hosts override
  │
  ├─→ Can the host reach service by IP? (curl http://<IP>:<port>)
  │   └─→ No → Service not running, firewall blocking, port wrong
  │
  └─→ Can the host reach service by name? (curl http://<domain>)
      └─→ No → DNS issue, certificate issue, proxy misconfigured
```

### Essential Commands
- **ip addr / ifconfig** — Interface configuration
- **ip route / route** — Routing table
- **ss -tlnp / netstat** — Listening ports and processes
- **ping** — Connectivity test
- **traceroute / tracepath** — Path to destination
- **nslookup / dig / host** — DNS resolution
- **curl / wget** — HTTP connectivity
- **mtr** — Combined ping + traceroute
- **tcpdump / Wireshark** — Packet capture

---

## Best Practices

### Network Design
- **Plan IP addressing** — Use RFC 1918 ranges, document subnet scheme
- **VLANs for segmentation** — Separate traffic by function (servers, workstations, IoT)
- **Redundant paths** — Multiple paths, multiple gateways, multiple ISPs
- **Document topology** — Keep network diagrams updated
- **Change management** — Test network changes before production

### Security
- **Default deny** — Deny all traffic, explicitly allow
- **Segment sensitive systems** — Isolate servers from general network
- **Monitor traffic** — IDS/IPS, netflow analysis
- **TLS everywhere** — Encrypt all transit traffic
- **DNS security** — Use DNSSEC, validate DNS responses
- **Patch networking gear** — Keep firmware and OS updated

### Monitoring
- **Monitor connectivity** — Ping critical hosts continuously
- **Monitor DNS** — Check resolution times and failures
- **Monitor bandwidth** — Track utilization trends
- **Monitor packet loss** — Alert on loss >1%
- **Monitor certificates** — Alert 30 days before expiration

---

## References

- [RFC 791 — IP](https://www.rfc-editor.org/rfc/rfc791)
- [RFC 793 — TCP](https://www.rfc-editor.org/rfc/rfc793)
- [RFC 1035 — DNS](https://www.rfc-editor.org/rfc/rfc1035)
- [RFC 2131 — DHCP](https://www.rfc-editor.org/rfc/rfc2131)
- [RFC 2460 — IPv6](https://www.rfc-editor.org/rfc/rfc2460)
- [Cloudflare DNS Guide](https://developers.cloudflare.com/dns/)
- [MDN HTTP Guide](https://developer.mozilla.org/en-US/docs/Web/HTTP)

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial Networking Engineering Foundation |