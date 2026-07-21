# Linux Knowledge — Network Configuration

## Network Stack

### OSI Model (Applied to Linux)

| Layer | Linux Implementation |
|-------|---------------------|
| Physical | Network interface hardware (eth0, enp0s3) |
| Data Link | Ethernet frames, VLAN tagging, MAC addresses |
| Network | IP routing, IP packets, IPv4/IPv6 |
| Transport | TCP/UDP sockets, ports |
| Application | Network services (HTTP, DNS, SSH) |

### Network Interfaces

```bash
# List interfaces
ip addr
ip link show

# Interface naming
eth0      # Traditional naming
enp0s3    # Predictable (bus slot format)
ens192    # Predictable (hotplug slot format)
wlan0     # Wireless interface
docker0   # Docker bridge

# Configure interface
ip addr add 192.168.1.100/24 dev eth0
ip link set eth0 up
ip link set eth0 down

# Persistent configuration
# RHEL: /etc/sysconfig/network-scripts/ifcfg-eth0
# Ubuntu: /etc/netplan/00-installer-config.yaml
```

### Routing

```bash
# View routing table
ip route
ip route show

# Add route
ip route add 10.0.0.0/8 via 192.168.1.1

# Default route
ip route show default

# Static routes configuration
# RHEL: /etc/sysconfig/network-scripts/route-eth0
# Ubuntu: netplan yaml or /etc/network/interfaces
```

## Network Tools

### Connectivity Testing

```bash
# Ping
ping -c 4 8.8.8.8
ping -c 4 google.com

# Traceroute
traceroute 8.8.8.8
tracepath 8.8.8.8

# MTR (combined ping + traceroute)
mtr 8.8.8.8
```

### DNS

```bash
# DNS resolution
nslookup google.com
dig google.com
host google.com

# /etc/hosts
cat /etc/hosts

# DNS configuration
cat /etc/resolv.conf

# systemd-resolved
systemd-resolve --status
resolvectl status
```

### Socket Statistics

```bash
# Listening ports
ss -tlnp

# All connections
ss -anp

# Connection statistics
ss -s

# By process
ss -p | grep nginx

# Specific port
ss -tlnp | grep :80
```

### Network Performance

```bash
# Bandwidth
iperf3 -c server

# Packet capture
sudo tcpdump -i eth0 port 80
sudo tcpdump -i eth0 -w capture.pcap

# Network monitoring
nethogs
iftop
```

## Network Services

### DNS Server (BIND)

```bash
sudo systemctl status named
sudo systemctl enable named

# Configuration
/etc/named.conf
/etc/named.rfc1912.zones
/var/named/
```

### DHCP Server (isc-dhcp-server)

```bash
sudo systemctl status dhcpd
sudo systemctl enable dhcpd

# Configuration
/etc/dhcp/dhcpd.conf
```

### Web Servers

| Server | Package | Config | Service |
|--------|---------|--------|---------|
| nginx | nginx | /etc/nginx/nginx.conf | nginx.service |
| Apache | httpd | /etc/httpd/conf/httpd.conf | httpd.service |
| Caddy | caddy | /etc/caddy/Caddyfile | caddy.service |

### SSH Server

```bash
sudo systemctl status sshd
sudo systemctl enable sshd

# Configuration
/etc/ssh/sshd_config
```

### Proxy Servers

| Type | Software | Port | Config |
|------|----------|------|--------|
| HTTP | nginx, squid | 8080 | /etc/squid/squid.conf |
| HTTPS | nginx, HAProxy | 443 | /etc/haproxy/haproxy.cfg |
| SOCKS | shadowsocks | 1080 | /etc/shadowsocks/config.json |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial network configuration knowledge |