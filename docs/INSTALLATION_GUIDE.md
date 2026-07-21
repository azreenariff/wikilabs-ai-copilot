# Installation Guide — Wiki Labs AI Copilot v1.0.0

> Windows installation, upgrade, repair, and uninstall procedures.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation Options](#installation-options)
3. [MSI Installation](#msi-installation)
4. [NSIS Installation](#nsis-installation)
5. [Silent/Unattended Installation](#silentunattended-installation)
6. [Verification](#verification)
7. [First Launch](#first-launch)
8. [Configuration After Installation](#configuration-after-installation)
9. [Upgrading](#upgrading)
10. [Repair](#repair)
11. [Uninstall](#uninstall)
12. [Troubleshooting Installation](#troubleshooting-installation)

## Prerequisites

### System Requirements

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| OS | Windows 10 64-bit | Windows 11 64-bit |
| CPU | Dual-core 2.0 GHz | Quad-core 2.5 GHz |
| RAM | 4 GB | 8 GB |
| Disk Space | 2 GB free | 5 GB free |
| Architecture | x64 | x64 |

### Required Components

- **.NET Desktop Runtime 8.0** — Included with both installers. If already installed, it is reused.
- **Microsoft Edge WebView2 Runtime** — Included with the installer. Pre-installed on Windows 11 and most Windows 10 devices.

### Network Requirements

| Endpoint | Port | Purpose |
|----------|------|---------|
| `github.com` (releases) | 443 | Download updates |
| AI Provider endpoint | 443 | AI API requests (user-configured) |

No inbound ports required. The application only initiates outbound connections.

### Permissions

| Installation Mode | Permissions Required |
|-------------------|---------------------|
| Per-user (default) | Standard user account |
| Per-machine | Administrator rights |

## Installation Options

| Option | Format | Use Case | Admin Required |
|--------|--------|----------|---------------|
| MSI | `.msi` | Enterprise deployment (SCCM, GPO, Intune) | Per-machine: Yes |
| NSIS | `.exe` | Individual user installation | Per-user: No |

## MSI Installation

### Interactive Installation

1. Download the MSI installer from the release page
2. Double-click the `.msi` file
3. Click **Install** in the User Account Control (UAC) prompt
4. The installation wizard opens
5. Review and accept the license agreement
6. Choose installation mode:
   - **Per User** — Installs for current user only (recommended)
   - **Per Machine** — Installs for all users (requires admin)
7. Choose installation directory (default is appropriate)
8. Click **Install**
9. Click **Finish** when installation completes

### Verifying MSI Installation

```powershell
# Check installed products
Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" |
  Where-Object { $_.DisplayName -like "*Wiki Labs*" }

# Check per-user installation
Get-ItemProperty "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" |
  Where-Object { $_.DisplayName -like "*Wiki Labs*" }

# Verify application files exist
Test-Path "$env:APPDATA\com.wikilabs.copilot\wikilabs.db"
Test-Path "$env:APPDATA\com.wikilabs.copilot\settings.json"
```

## NSIS Installation

### Interactive Installation

1. Download the NSIS installer (`.exe`) from the release page
2. Double-click the executable
3. Click **Install** in the UAC prompt (if running per-machine)
4. The installation wizard opens
5. Review and accept the license agreement
6. Choose installation directory (default: `%APPDATA%\com.wikilabs.copilot`)
7. Choose additional tasks:
   - [x] Create desktop shortcut
   - [x] Add to Start Menu
   - [x] Associate `.wkl` files
8. Click **Install**
9. Click **Finish** when installation completes

### NSIS Installer Parameters

| Parameter | Description |
|-----------|-------------|
| `/S` | Silent installation |
| `/D=C:\path\to\dir` | Install directory |
| `/LANG=English` | Interface language |

## Silent/Unattended Installation

### MSI Silent Install (Per-User)

```cmd
msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" /quiet INSTALLMODE="currentUser"
```

### MSI Silent Install (Per-Machine)

```cmd
msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" /quiet INSTALLMODE="perMachine"
```

### MSI Silent Install with Logging

```cmd
msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" /quiet INSTALLMODE="currentUser" /L*v "C:\logs\wikilabs-install.log"
```

### NSIS Silent Install

```powershell
Start-Process ".\wikilabs-ai-copilot-1.0.0-setup.exe" -ArgumentList "/S", "/D=%APPDATA%\com.wikilabs.copilot" -Wait -NoNewWindow
```

### PowerShell Automated Deployment

```powershell
$installer = "wikilabs-ai-copilot-1.0.0-x64.msi"
$installArgs = "/i `"$installer`" /quiet INSTALLMODE=`"currentUser`""

$process = Start-Process msiexec.exe -ArgumentList $installArgs -Wait -PassThru

if ($process.ExitCode -eq 0) {
    Write-Host "Installation successful"
} else {
    Write-Error "Installation failed with exit code: $($process.ExitCode)"
}
```

### SCCM Application Deployment

**Install Command:**
```
msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" /quiet INSTALLMODE="perMachine"
```

**Uninstall Command:**
```
msiexec /x "{PRODUCT-GUID}" /quiet
```

**Detection Rule:**
```
File exists: %APPDATA%\com.wikilabs.copilot\wikilabs.db
```

## Verification

### Post-Installation Checks

1. **Application launches:**
   - Open Start Menu and search "Wiki Labs AI Copilot"
   - Click to launch the application
   - The main window should open showing the chat interface

2. **Files created:**
   ```
   %APPDATA%\com.wikilabs.copilot\
     wikilabs.db          ← SQLite database (created on first launch)
     settings.json        ← Settings file (created on first launch)
     logs\                ← Log directory (created on first launch)
   ```

3. **Registry entries (per-machine):**
   ```
   HKLM:\SOFTWARE\com.wikilabs.copilot    ← App settings
   HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{GUID}  ← Uninstall info
   ```

4. **Start Menu entry:**
   - Verify "Wiki Labs AI Copilot" appears in Start Menu
   - Verify shortcut exists at `C:\Users\%USERNAME%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Wiki Labs\`

5. **Desktop shortcut (if selected):**
   - Verify shortcut exists on desktop
   - Double-click to verify application launches

## First Launch

On first launch, the application:

1. Creates the data directory at `%APPDATA%\com.wikilabs.copilot\`
2. Creates the SQLite database with the initial schema
3. Initializes settings with defaults
4. Creates the log directory
5. Opens the main window with the default workspace

### Default Settings

| Setting | Value |
|---------|-------|
| AI Provider Name | `openai` |
| AI Endpoint | `https://api.openai.com/v1` |
| AI Model | `gpt-4o` |
| Max Tokens | 4096 |
| Theme | Dark |
| Screen Observation | Disabled |
| Privacy Mode | Disabled |

## Configuration After Installation

After installation, users need to configure the AI provider:

1. Launch Wiki Labs AI Copilot
2. Open Settings (gear icon in sidebar)
3. Navigate to AI Provider section
4. Enter provider details (name, endpoint, API key, model)
5. Click **Test Connection** to verify
6. Click **Save**

See [Administrator Guide](../admin-guide/ADMINISTRATOR_GUIDE.md) for enterprise deployment scenarios.

## Upgrading

### Automatic Upgrade

The application includes an auto-update mechanism via tauri-plugin-updater:

1. On startup, the application checks for updates
2. If an update is available, a notification appears
3. Click **Download and Install** to proceed
4. The application downloads and applies the update
5. The application restarts with the new version

### Manual Upgrade

#### From MSI

1. Download the new MSI installer
2. Run the new MSI — it detects the existing installation
3. Click **Upgrade** in the prompt
4. The installer upgrades in place preserving all data

#### From NSIS

1. Download the new NSIS installer
2. Run the new installer
3. The installer detects the existing installation
4. Click **Upgrade** to upgrade in place
5. All data, settings, and workspaces are preserved

### Upgrade Preserved Data

The following data is preserved during upgrades:
- All workspaces and their configurations
- All chat history
- All knowledge base documents
- All skill pack configurations
- All settings and profiles
- All credential stores

### Upgrade Verified Data

The following data is NOT preserved (by design):
- Log files (new log files created on upgrade)
- Crash reports (preserved during upgrade)
- Temporary files (cleaned during upgrade)

## Repair

### Repairing Installation

If the application is malfunctioning but data is intact:

#### From Windows Settings

1. Open **Settings → Apps → Installed Apps**
2. Find "Wiki Labs AI Copilot"
3. Click the menu (⋯) and select **Modify**
4. Select **Repair** when prompted
5. Click **Repair**
6. Click **Finish** when complete

#### From Command Line (MSI)

```cmd
msiexec /fecom "wikilabs-ai-copilot-1.0.0-x64.msi"
```

#### From Command Line (NSIS)

1. Run the NSIS installer
2. The installer detects the existing installation
3. Select **Repair** from the options
4. Follow the repair wizard

### Repairing Database

If the SQLite database is corrupted:

1. **Close the application completely**
2. **Backup the database:**
   ```cmd
   copy "%APPDATA%\com.wikilabs.copilot\wikilabs.db" "%APPDATA%\com.wikilabs.copilot\wikilabs.db.bak"
   ```
3. **Delete the corrupted database:**
   ```cmd
   del "%APPDATA%\com.wikilabs.copilot\wikilabs.db"
   ```
4. **Restart the application** — it creates a fresh database
5. **Reconfigure the AI provider** and re-create workspaces

> **Warning:** Step 3 deletes all workspaces and data. The backup in step 2 allows restoration.

## Uninstall

### Standard Uninstall (via Windows Settings)

1. Open **Settings → Apps → Installed Apps**
2. Find "Wiki Labs AI Copilot"
3. Click the menu (⋯) and select **Uninstall**
4. Click **Uninstall** in the confirmation dialog
5. Click **Finish** when complete

### Standard Uninstall (via Control Panel)

1. Open **Control Panel → Programs and Features**
2. Find "Wiki Labs AI Copilot"
3. Click **Uninstall**
4. Follow the uninstall wizard

### Command Line Uninstall (MSI)

```cmd
# Find the product code
wmic product where "name like '%Wiki Labs%'" get name, identificationsnumber

# Uninstall
msiexec /x "{PRODUCT-GUID}" /quiet
```

### Command Line Uninstall (NSIS)

```cmd
"%APPDATA%\com.wikilabs.copilot\Uninstall.exe" /S
```

### Complete Data Removal

To fully remove all application data (caution: irreversible):

```powershell
# Close the application first
Stop-Process -Name "wikilabs*" -ErrorAction SilentlyContinue

# Remove application data
Remove-Item "$env:APPDATA\com.wikilabs.copilot" -Recurse -Force

# Remove Start Menu shortcuts
Remove-Item "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Wiki Labs" -Recurse -Force 2>$null

# Remove desktop shortcut if it exists
Remove-Item "$env:USERPROFILE\Desktop\Wiki Labs AI Copilot.lnk" -ErrorAction SilentlyContinue 2>$null
```

> **Warning:** This permanently deletes all workspaces, chat history, knowledge bases, and settings.

## Troubleshooting Installation

### Installation Fails — "Another version is installed"

The installer detects an existing installation and may require a repair or upgrade instead:

1. Uninstall the existing version first
2. Then install the new version

### Installation Fails — "Insufficient disk space"

Free at least 2 GB of disk space in the installation directory (`%APPDATA%\com.wikilabs.copilot`).

### Application Won't Launch — "WebView2 not found"

Install WebView2 Runtime:
```powershell
# Check if WebView2 is installed
Get-AppxPackage *WebView2*

# Download and install WebView2
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

### Application Won't Launch — ".NET Runtime missing"

Install .NET Desktop Runtime 8.0:
```powershell
# Check if .NET Runtime is installed
Get-ItemProperty "HKLM:\SOFTWARE\dotnet\Setup\InstalledVersions\x64\desktop"

# Download and install .NET Desktop Runtime 8.0
# https://dotnet.microsoft.com/download/dotnet/8.0
```

### Installation Hangs

1. Check Windows Event Viewer for application errors
2. Check Windows Defender for false positives
3. Run the installer as Administrator
4. Temporarily disable antivirus during installation

### "Access Denied" During Installation

Run the installer as Administrator:
```powershell
Start-Process ".\wikilabs-ai-copilot-1.0.0-setup.exe" -Verb RunAs
```

Or use per-user installation mode (does not require admin):
```cmd
msiexec /i "wikilabs-ai-copilot-1.0.0-x64.msi" INSTALLMODE="currentUser" /quiet
```

### Firewall Blocking Updates

If the auto-update fails, verify outbound HTTPS (port 443) to `github.com` is allowed:
```powershell
Test-NetConnection github.com -Port 443
```

---

*For configuration details, see [Administrator Guide](../admin-guide/ADMINISTRATOR_GUIDE.md).*
*For troubleshooting, see [Troubleshooting Guide](TROUBLESHOOTING.md).*
*For getting help, see [Support Guide](SUPPORT_GUIDE.md).*