# Windows Engineering — Concepts Overview

## Core Concepts

This skill pack covers the fundamental concepts of Windows Server engineering.

## Domains

### 1. Windows Architecture
Windows Server uses a hybrid kernel architecture with multiple subsystems:
- **Win32 API**: Primary application programming interface
- **Kernel Mode**: Core OS components (process management, memory management, I/O)
- **User Mode**: Applications and services
- **HAL (Hardware Abstraction Layer)**: Hardware abstraction for portability
- **LSASS**: Local Security Authority Subsystem Service

### 2. Windows Services
Services are background processes that run independently:
- **Automatic**: Start at boot
- **Manual**: Start on demand
- **Disabled**: Cannot be started
- **Dependent services**: Services that rely on others
- **Recovery actions**: Actions on service failure (restart, run program, reboot)

### 3. Active Directory
Active Directory is the identity and access management system:
- **Domain**: Logical grouping of objects
- **Forest**: Collection of trusted domains
- **OU (Organizational Unit)**: Container for organizing objects
- **GPO (Group Policy Object)**: Centralized configuration management
- **FSMO Roles**: Five flexible single-master operation roles
- **Trust relationships**: Authentication between domains

### 4. PowerShell
PowerShell is the primary automation and administration tool:
- **Cmdlets**: Single-function commands (Get-Service, Set-Service)
- **Providers**: Data source abstraction (Registry, File System, Active Directory)
- **Modules**: Reusable command packages
- **Pipelines**: Pass objects between commands
- **Scripting**: PowerShell (.ps1) files for automation
- **Execution Policy**: Security setting for script execution

### 5. Event Logging
Windows uses the Event Log system for diagnostics:
- **System Log**: OS-level events
- **Application Log**: Application events
- **Security Log**: Security events (requires auditing enabled)
- **Setup Log**: Windows installation events
- **Forwarded Events**: Centralized event collection

### 6. Windows Networking
Windows networking stack includes:
- **TCP/IP stack**: Primary network protocol
- **DNS Client**: Name resolution
- **Windows Firewall**: Host-based firewall
- **Winsock**: Windows socket API
- **NetBT**: NetBIOS over TCP/IP
- **DHCP Client**: Dynamic address assignment

### 7. Windows Security
Windows implements layered security:
- **NTLM**: Legacy authentication protocol
- **Kerberos**: Modern authentication protocol
- **LSA**: Local Security Authority
- **Access Control Lists (ACLs)**: File and object permissions
- **Windows Defender**: Antivirus and endpoint protection
- **BitLocker**: Full disk encryption

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial concepts overview |