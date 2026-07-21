# Release Process — Wiki Labs AI Copilot

## Overview

This document documents the complete release process for Wiki Labs AI Copilot,
including building, code signing, publishing, and deploying updates via the
Tauri updater plugin.

## Prerequisites

- [ ] Code signing certificate (`.pfx` file) from a trusted CA
- [ ] Access to GitHub repository (wikilabs/wikilabs-ai-copilot)
- [ ] GitHub Pages configured for the `docs` or `gh-pages` branch
- [ ] Tauri v2 CLI installed: `npm install -g @tauri-apps/cli`
- [ ] Rust toolchain (1.77+) installed
- [ ] Windows build environment for NSIS/MSI bundling

## Versioning

This project follows [Semantic Versioning](https://semver.org/):
- `MAJOR` — breaking changes
- `MINOR` — new features (backward compatible)
- `PATCH` — bug fixes (backward compatible)

Channels:
- **stable** — latest stable release (default updater channel)
- **preview** — beta/release candidate builds
- **internal** — internal builds for QA/testing

## Step-by-Step Release

### Step 1: Update Version Numbers

Update the version in both locations:

```bash
# src-tauri/tauri.conf.json
"version": "1.1.0"

# src-tauri/Cargo.toml
version = "0.1.0"
```

### Step 2: Update Changelog

Update the project CHANGELOG.md (or equivalent) with all changes for this release.

### Step 3: Build the Installer

```bash
cd ~/wikilabs-ai-copilot

# Build with Tauri CLI (Windows target)
cargo tauri build

# This produces:
#   src-tauri/target/release/bundle/nsis/Wiki Labs AI Copilot_1.0.0_x64-setup.exe
#   src-tauri/target/release/bundle/msi/Wiki Labs AI Copilot_1.0.0_x64.msi
```

### Step 4: Code Sign the Installer

```powershell
# Sign the NSIS installer
signtool sign /fd SHA256 ^
  /tr http://timestamp.digicert.com ^
  /td SHA256 ^
  /f "C:\certs\wikilabs-code-signing.pfx" ^
  /p "YOUR_CERT_PASSWORD" ^
  "src-tauri\target\release\bundle\nsis\Wiki Labs AI Copilot_1.0.0_x64-setup.exe"

# Sign the MSI installer (if applicable)
signtool sign /fd SHA256 ^
  /tr http://timestamp.digicert.com ^
  /td SHA256 ^
  /f "C:\certs\wikilabs-code-signing.pfx" ^
  /p "YOUR_CERT_PASSWORD" ^
  "src-tauri\target\release\bundle\msi\Wiki Labs AI Copilot_1.0.0_x64.msi"

# Verify
signtool verify /pa /v "src-tauri\target\release\bundle\nsis\Wiki Labs AI Copilot_1.0.0_x64-setup.exe"
```

> See `src-tauri/CODE_SIGNING.md` for detailed signing instructions.

### Step 5: Test the Installer

1. **Clean install** — install on a clean Windows machine
2. **Silent install** — test `installer.exe /S` for unattended deployment
3. **Silent uninstall** — test `uninstaller.exe /S`
4. **Upgrade flow** — install an older version, then upgrade to the new version
5. **Verify shortcuts** — desktop and Start Menu shortcuts work correctly
6. **Verify file associations** — `.wikilabs` files open in the app
7. **Verify updater** — install, then test auto-update to next version

### Step 6: Create and Push a GitHub Release

```bash
# Tag the release
git tag -a v1.0.0 -m "Release v1.0.0 — Initial stable release"

# Push the tag
git push origin v1.0.0

# Upload release assets via GitHub CLI
gh release create v1.0.0 \
  --title "v1.0.0 — Initial Release" \
  --generate-notes \
  "src-tauri/target/release/bundle/nsis/Wiki Labs AI Copilot_1.0.0_x64-setup.exe" \
  "src-tauri/target/release/bundle/msi/Wiki Labs AI Copilot_1.0.0_x64.msi"
```

The release page should have:
- `latest.json` — the updater manifest (see Step 7)
- `Wiki Labs AI Copilot_1.0.0_x64-setup.exe` — NSIS installer
- `Wiki Labs AI Copilot_1.0.0_x64.msi` — MSI installer (alternative)
- Release notes with changelog

### Step 7: Deploy the Update Manifest (`latest.json`)

The Tauri updater reads a `latest.json` manifest to determine available updates.
Deploy this to both configured endpoints:

#### Option A: GitHub Release (primary)

```bash
# Create latest.json from template
# See src-tauri/update/latest.json for the template structure

# Upload latest.json to the GitHub Release
gh release upload v1.0.0 \
  src-tauri/update/latest.json \
  --clobber

# The Tauri updater fetches it from:
# https://github.com/wikilabs/wikilabs-ai-copilot/releases/download/1.0.0/latest.json
```

#### Option B: GitHub Pages (secondary, for fast CDN delivery)

```bash
# Deploy to GitHub Pages at:
# https://wikilabs.github.io/wikilabs-ai-copilot/updates/1.0.0/latest.json

# Push the update directory
cd src-tauri/update
cp latest.json ../update/1.0.0/latest.json

# Deploy to gh-pages branch (example using git subtree push)
cd ~/wikilabs-ai-copilot
git subtree push --prefix src-tauri/update origin gh-pages
```

#### Latest.json Schema

```json
{
  "version": "1.0.0",
  "notes": "Release notes (supports Markdown)",
  "pubdate": "2026-07-21T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "",    # RSA signature of the installer (fill after signing)
      "url": "https://..."  # Direct download URL to the installer
    }
  }
}
```

Supported platform keys:
- `windows-x86_64` — 64-bit Windows (primary target)
- `darwin-aarch64` — macOS Apple Silicon (future)
- `darwin-x86_64` — macOS Intel (future)
- `linux-x86_64` — Linux x86_64 (future)

### Step 8: Handle Channel Releases

#### Stable Channel (default)
- Use standard version tags (e.g., `v1.0.0`)
- `latest.json` in GitHub release root
- Updater fetches from `releases/download/{version}/latest.json`

#### Preview Channel (beta)
```json
// preview/latest.json
{
  "version": "1.1.0-beta.1",
  "notes": "Preview build for testing",
  "pubdate": "2026-07-21T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "",
      "url": "https://github.com/wikilabs/wikilabs-ai-copilot/releases/download/1.1.0-beta.1/Wiki_Labs_AI_Copilot-1.1.0-beta.1-x86_64-setup.exe"
    }
  }
}
```

#### Internal Channel
```json
// internal/latest.json
{
  "version": "1.0.1-internal.42",
  "notes": "Internal QA build #42",
  "pubdate": "2026-07-21T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "",
      "url": "https://internal.wikilabs.com/updates/1.0.1-internal.42/Wiki_Labs_AI_Copilot-1.0.1-internal.42-x86_64-setup.exe"
    }
  }
}
```

To switch channels in the app, update the `endpoints` array in `tauri.conf.json`:

```json
"endpoints": [
  "https://github.com/wikilabs/wikilabs-ai-copilot/releases/download/{version}/latest.json",
  "https://wikilabs.github.io/wikilabs-ai-copilot/updates/{version}/latest.json"
]
```

### Step 9: Post-Release Verification

1. [ ] Download installer from GitHub release page — verify it runs
2. [ ] Install on a clean machine — verify shortcuts, file associations
3. [ ] Launch app — verify it starts correctly
4. [ ] Check for updates — verify updater finds the new version
5. [ ] Apply update — verify upgrade preserves user data and settings
6. [ ] Run the new version — verify no regressions
7. [ ] Check SmartScreen — verify no warnings (may take time to build reputation)

## Rollback Procedure

If a release has critical issues:

1. **Remove the problematic release** from GitHub
2. **Update the next version** in the repository to prevent automatic rollback
3. **Manually notify** affected users
4. **Publish a fix release** with the corrected version number

## Automated Release (Optional CI/CD)

### GitHub Actions Example

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
      
      - name: Install dependencies
        run: npm ci
      
      - name: Build Tauri app
        run: cargo tauri build
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Code sign
        run: |
          signtool sign /fd SHA256 /tr http://timestamp.digicert.com ^
            /td SHA256 /f ${{ secrets.SIGN_CERT_PATH }} ^
            /p ${{ secrets.SIGN_CERT_PASSWORD }} ^
            src-tauri\target\release\bundle\nsis\*.exe
      
      - name: Create GitHub Release
        run: |
          gh release create ${{ github.ref_name }} ^
            --title ${{ github.ref_name }} ^
            --generate-notes ^
            src-tauri/target/release/bundle/nsis/*.exe ^
            src-tauri/update/latest.json
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Updater doesn't find update | Verify `latest.json` is accessible at the configured endpoint URL |
| Signature verification fails | Ensure the signature in `latest.json` matches the installer's code signature |
| SmartScreen warning | Wait for reputation to build, or use an EV certificate |
| Silent install fails | Check that `custom.nsi` is properly configured for silent mode |
| User data lost during upgrade | Verify `custom.nsi` does NOT delete `%APPDATA%\Wiki Labs\AI Copilot` |
| Update endpoint returns 404 | Ensure `latest.json` is uploaded to the release assets or GitHub Pages |

## Checklist

Release checklist for each version:

- [ ] Version updated in `tauri.conf.json` and `Cargo.toml`
- [ ] Changelog updated
- [ ] Code signing certificate valid and not expired
- [ ] Installer builds successfully
- [ ] Installer is code-signed
- [ ] `latest.json` generated and deployed
- [ ] GitHub release created with assets
- [ ] Clean install tested
- [ ] Silent install tested
- [ ] Upgrade flow tested (old → new, data preserved)
- [ ] Update check works
- [ ] Post-release verification complete