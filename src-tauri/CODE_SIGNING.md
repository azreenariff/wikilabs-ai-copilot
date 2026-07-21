# Code Signing Guide — Wiki Labs AI Copilot

## Overview

This document provides step-by-step instructions for code-signing the Wiki Labs AI Copilot
Windows installer and executables. Code signing establishes publisher identity, prevents
Windows SmartScreen warnings, and assures users the software has not been tampered with.

## Certificate Requirements

### Recommended Certificate Authority

| CA | Notes |
|-----|-------|
| **Sectigo** (formerly Comodo) | Cost-effective, widely accepted |
| **DigiCert** | Premium, best SmartScreen reputation |
| **GlobalSign** | Good balance of price and reputation |
| **Let's Encrypt EV** | Free EV certificates (limited availability) |

### What You Need

1. **Code Signing Certificate (EV recommended)**
   - Extended Validation (EV) certificate provides the best SmartScreen reputation
   - Standard code signing certificate works but may take longer to build reputation
   - Must be in `.pfx` or `.p12` format for signtool

2. **Timestamp Server** (included with most CAs)
   - RFC 3161 compliant — ensures signatures remain valid after certificate expires
   - Standard: `http://timestamp.digicert.com` or `http://sha256timestamp.ws.symantec.com/sha256/timestamp`

3. **sign.exe / signtool.exe**
   - Ships with Windows SDK or Visual Studio
   - Standalone: [Windows SDK Downloads](https://developer.microsoft.com/en-us/windows/downloads/sdk-archive/)

## Step-by-Step Signing

### 1. Export the Certificate

From Windows Certificate Manager:
```
certmgr.msc
→ Personal → Certificates
→ Right-click your code signing cert → All Tasks → Export
→ Select "Yes, export the private key"
→ Save as .pfx file
→ Remember the export password
```

### 2. Sign the Executable

```powershell
# Sign the main application executable
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ^
  /f "path\to\certificate.pfx" /p "certificate-password" ^
  "C:\path\to\Wiki Labs AI Copilot\wikilabs-copilot.exe"
```

### 3. Sign the Installer (NSIS)

```powershell
# Sign the generated NSIS installer
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ^
  /f "path\to\certificate.pfx" /p "certificate-password" ^
  "C:\path\to\output\Wiki Labs AI Copilot_1.0.0_x64-setup.exe"
```

### 4. Verify the Signature

```powershell
signtool verify /pa /v "C:\path\to\Wiki Labs AI Copilot_1.0.0_x64-setup.exe"
```

## Automating Signing in the Build Pipeline

### Option A: Environment Variables

```powershell
# In your CI/CD pipeline (GitHub Actions, etc.)
$env:SIGN_CERT_PATH = "C:\certs\wikilabs-code-signing.pfx"
$env:SIGN_CERT_PASSWORD = $env:SIGN_CERT_PASSWORD  # From secrets
$env:SIGN_TIMESTAMP_URL = "http://timestamp.digicert.com"

signtool sign /fd SHA256 /tr $env:SIGN_TIMESTAMP_URL /td SHA256 ^
  /f $env:SIGN_CERT_PATH /p $env:SIGN_CERT_PASSWORD ^
  "$env:OUTPUT_PATH\Wiki Labs AI Copilot_*-setup.exe"
```

### Option B: Post-Build Script

Create `src-tauri/scripts/sign.ps1`:

```powershell
param(
    [string]$CertPath,
    [string]$CertPassword,
    [string]$TimestampUrl = "http://timestamp.digicert.com",
    [string]$InputPath
)

signtool sign /fd SHA256 /tr $TimestampUrl /td SHA256 `
  /f $CertPath /p $CertPassword `
  $InputPath

if ($LASTEXITCODE -ne 0) {
    Write-Error "Code signing failed"
    exit 1
}
```

## SmartScreen Reputation

To build Windows SmartScreen reputation:

1. **Use an EV certificate** — instant reputation
2. **Publish via Microsoft Partner Center** — submit installer for analysis
3. **Wait for reputation to build** — standard certificates need ~1000 downloads
4. **Use consistent signing** — always sign with the same certificate

## Tauri-Specific Notes

### Configuring tauri.conf.json for Signing

In `tauri.conf.json` under `bundle.windows`:

```json
{
  "windows": {
    "nsis": {
      "certificateFile": "path/to/cert.pfx",
      "certificatePassword": "${SIGN_CERT_PASSWORD}",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

### Using Environment Variables for Secrets

Never commit certificate passwords to version control. Use:
- GitHub Actions secrets: `${{ secrets.SIGN_CERT_PASSWORD }}`
- Tauri's environment variable expansion
- CI/CD pipeline secret management

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "The trust relationship with the primary domain failed" | Run on the domain-joined machine or use .pfx directly |
| "The specified module could not be found" | Ensure Windows SDK is installed and signtool is in PATH |
| SmartScreen blocks installer | Wait for reputation to build or use EV cert |
| Signature verification fails | Re-sign with `/fd SHA256 /tr http://timestamp.digicert.com` |
| "Invalid signature" after update | Re-sign the new build before distribution |

## Security Considerations

- **Store certificates securely** — in a hardware token (YubiKey) or encrypted vault
- **Never commit .pfx files** to version control
- **Rotate certificates** before expiration (renewal invalidates old signatures' trust)
- **Use SHA-256** fingerprint algorithm — SHA-1 is deprecated
- **Always timestamp** — signatures remain valid even after cert expires

## Quick Reference

```bash
# Sign with SHA-256 and timestamp (quick command)
signtool sign /fd SHA256 /tr http://timestamp.digicert.com ^
  /f "cert.pfx" /p "password" "file.exe"

# Verify a signed file
signtool verify /pa /v "file.exe"

# Sign multiple files in a directory
Get-ChildItem *.exe | ForEach-Object {
    signtool sign /fd SHA256 /tr http://timestamp.digicert.com ^
      /f "cert.pfx" /p "password" $_.FullName
}
```