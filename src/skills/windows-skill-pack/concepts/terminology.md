# Windows Engineering — Terminology

## Core Terms

| Term | Definition |
|------|------------|
| Windows Service | Background process managed by SCM |
| Service Control Manager (SCM) | Manages and tracks Windows services |
| Active Directory | Microsoft directory service for identity management |
| Domain Controller | Server hosting Active Directory |
| GPO | Group Policy Object — centralized configuration |
| FSMO | Flexible Single Master Operations — 5 AD roles |
| PowerShell | Windows scripting and automation engine |
| Cmdlet | Single-function PowerShell command |
| Event Log | Windows diagnostic logging system |
| Registry | Windows hierarchical configuration database |
| WMI | Windows Management Instrumentation — management framework |
| CIM | Common Information Model — standards-based management |
| DNS | Domain Name System — name resolution |
| DHCP | Dynamic Host Configuration Protocol — address assignment |
| IIS | Internet Information Services — web server |
| Netsh | Network Shell — network configuration CLI |
| sc.exe | Service Control utility |
| net.exe | Network management utility |
| DISM | Deployment Image Servicing and Management |
| Systeminfo | System information reporting tool |

## Service Terms

| Term | Definition |
|------|------------|
| Automatic | Service starts at boot |
| Manual | Service starts on demand |
| Disabled | Service cannot be started |
| Dependency | Service required by another service |
| Recovery Action | Action taken on service failure |
| Log On As | Account used to run the service |
| Start Type | When the service starts (auto/manual/disabled) |

## AD Terms

| Term | Definition |
|------|------------|
| Domain | Logical grouping of AD objects |
| Forest | Collection of trusted domains |
| Tree | Connected domains in AD |
| OU | Organizational Unit — object container |
| GPO | Group Policy Object — configuration |
| SYSVOL | Shared domain system volume |
| NTDS.dit | AD database file |
| FSMO | Flexible Single Master Operations |
| RID Master | Relative ID Master role |
| PDC Emulator | Primary Domain Controller Emulator |
| Schema Master | Schema update master |
| Domain Naming Master | Domain addition master |
| Infrastructure Master | Reference update master |
| KDC | Key Distribution Center — Kerberos |
| SAM | Security Account Manager |

## PowerShell Terms

| Term | Definition |
|------|------------|
| Cmdlet | Single-function command (verb-noun format) |
| Pipeline | Pass objects between cmdlets |
| Provider | Data source abstraction |
| Module | Reusable command package |
| Script | PowerShell (.ps1) file |
| Execution Policy | Security setting for script execution |
| Variable | PowerShell data storage ($var) |
| Function | Reusable code block |
| Filter | Process objects through pipeline |
| Alias | Alternate name for cmdlet |
| Profile | PowerShell startup configuration |

## Networking Terms

| Term | Definition |
|------|------------|
| TCP/IP | Transmission Control Protocol/Internet Protocol |
| IP Address | Network address for host |
| Subnet Mask | Network/portion definition |
| Default Gateway | Route for external traffic |
| DNS Server | Name resolution server |
| DHCP Server | Dynamic address assignment server |
| Winsock | Windows socket API |
| NIC | Network Interface Card |
| VLAN | Virtual LAN — logical segmentation |
| NAT | Network Address Translation |
| VPN | Virtual Private Network |
| Firewall | Network traffic filtering |
| Port | Network endpoint identifier |
| Socket | Network communication endpoint |

## Security Terms

| Term | Definition |
|------|------------|
| Kerberos | Modern authentication protocol |
| NTLM | Legacy authentication protocol |
| LSA | Local Security Authority |
| SAM | Security Account Manager |
| ACL | Access Control List |
| DACL | Discretionary ACL — object permissions |
| SACL | System ACL — audit permissions |
| SID | Security Identifier |
| Token | User security context |
| UAC | User Account Control |
| BitLocker | Full disk encryption |
| Windows Defender | Endpoint protection |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial terminology glossary |