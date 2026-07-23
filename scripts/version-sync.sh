#!/usr/bin/env bash
#
# version-sync.sh
# Syncs version across all project configuration files.
# Called during the GitHub Actions release workflow with the tag version.
#
# Usage: ./version-sync.sh <version>
#   <version> — the version string (e.g., "1.2.3" or "v1.2.3" — the 'v' prefix is stripped)
#
# Files updated:
#   - Cargo.toml                (workspace root — [workspace.package].version)
#   - src-tauri/Cargo.toml      (Tauri app package version)
#   - src-tauri/tauri.conf.json (Tauri configuration version)
#   - src/frontend/package.json (frontend Node.js version)
#   - src-tauri/update/latest.json  (Tauri updater manifest — URL + version)

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <version>"
  echo "  e.g., $0 v1.2.3  or  $0 1.2.3"
  exit 1
fi

VERSION="${1#v}"  # strip leading 'v' if present
echo "=== Syncing version to: ${VERSION} ==="

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# ── 1. Workspace root Cargo.toml ──────────────────────────────────
# [workspace.package]
# version = "X.Y.Z"
sed -i "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" "${REPO_ROOT}/Cargo.toml"
echo "  ✓ Cargo.toml (workspace root)"

# ── 2. src-tauri/Cargo.toml ───────────────────────────────────────
sed -i "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" "${REPO_ROOT}/src-tauri/Cargo.toml"
echo "  ✓ src-tauri/Cargo.toml"

# ── 3. src-tauri/tauri.conf.json ───────────────────────────────────
python3 -c "
import json, sys
version = sys.argv[1]
path = '${REPO_ROOT}/src-tauri/tauri.conf.json'
with open(path, 'r') as f:
    config = json.load(f)
config['version'] = version
with open(path, 'w') as f:
    json.dump(config, f, indent=2)
" "${VERSION}"
echo "  ✓ src-tauri/tauri.conf.json"

# ── 4. src/frontend/package.json ──────────────────────────────────
python3 -c "
import json, sys
version = sys.argv[1]
path = '${REPO_ROOT}/src/frontend/package.json'
with open(path, 'r') as f:
    pkg = json.load(f)
pkg['version'] = version
with open(path, 'w') as f:
    json.dump(pkg, f, indent=2)
" "${VERSION}"
echo "  ✓ src/frontend/package.json"

# ── 5. src-tauri/update/latest.json (updater manifest) ────────────
python3 -c "
import json, re, sys
from datetime import datetime, timezone
version = sys.argv[1]
path = '${REPO_ROOT}/src-tauri/update/latest.json'
with open(path, 'r') as f:
    manifest = json.load(f)

manifest['version'] = version
manifest['notes'] = 'Release Notes for Wiki Labs AI Copilot v' + version
manifest['pubdate'] = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')

# Update all platform download URLs to match the new version
for platform_key, platform in manifest.get('platforms', {}).items():
    url = platform.get('url', '')
    # Replace version in URL: /download/<old-ver>/ -> /download/<new-ver>/
    new_url = re.sub(r'/download/([^/]+)/', '/download/' + version + '/', url)
    # Also replace version in filename: AppName-<old-ver>- -> AppName-<new-ver>-
    new_url = re.sub(
        r'(Wiki_Labs_AI_Copilot-)\d+\.\d+\.\d+(-)',
        r'\g<1>' + version + r'\g<2>',
        new_url
    )
    platform['url'] = new_url

with open(path, 'w') as f:
    json.dump(manifest, f, indent=2)
" "${VERSION}"
echo "  ✓ src-tauri/update/latest.json"

echo ""
echo "=== Version sync complete: ${VERSION} ==="
echo ""
echo "Verification:"
grep '^version' "${REPO_ROOT}/Cargo.toml" | head -1
grep '^version' "${REPO_ROOT}/src-tauri/Cargo.toml" | head -1
grep '"version"' "${REPO_ROOT}/src-tauri/tauri.conf.json" | head -1
grep '"version"' "${REPO_ROOT}/src/frontend/package.json" | head -1
grep '"version"' "${REPO_ROOT}/src-tauri/update/latest.json" | head -1