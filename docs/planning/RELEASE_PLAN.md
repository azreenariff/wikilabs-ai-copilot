# Wiki Labs AI Copilot — Release Plan

> **Last Updated:** 2025-07-16  
> **Version:** 1.0  
> **Status:** Planning  
> **Owner:** Technical Lead

---

## Table of Contents

1. [Release Strategy](#release-strategy)
2. [Versioning Scheme](#versioning-scheme)
3. [Release Cadence](#release-cadence)
4. [Release Types](#release-types)
5. [Release Process](#release-process)
6. [Rollout Strategy](#rollout-strategy)
7. [Update Delivery Mechanism](#update-delivery-mechanism)
8. [Rollback Strategy](#rollback-strategy)
9. [Release Notes Template](#release-notes-template)
10. [Deployment Checklist](#deployment-checklist)
11. [Post-Release Monitoring](#post-release-monitoring)
12. [Customer Communication Plan](#customer-communication-plan)

---

## Release Strategy

Wiki Labs AI Copilot follows a **staged, safety-first release strategy** designed for an enterprise engineering tool where correctness, security, and data integrity are paramount. The copilot runs on engineer laptops and observes their work — any release defect could compromise privacy, expose credentials, or disrupt critical infrastructure work.

**Guiding principles:**

1. **No broken updates:** If an update has any chance of breaking, it doesn't ship to production.
2. **Engineer trust is the product:** Release quality is the most important feature.
3. **Transparent changelogs:** Every update includes a detailed changelog.
4. **Zero data loss:** Updates never corrupt, lose, or misplace user data.
5. **Rapid rollback:** If something goes wrong after deployment, rollback takes minutes, not hours.

---

## Versioning Scheme

### Semantic Versioning (semver)

Wiki Labs AI Copilot uses [Semantic Versioning 2.0.0](https://semver.org/) with the format `MAJOR.MINOR.PATCH`:

```
MAJOR.MINOR.PATCH
  ^     ^     ^
  │     │     └─ Patch: Bug fixes, security patches, data migrations
  │     └─────── Minor: New features, improvements, new skills
  └───────────── Major: Breaking changes, architectural shifts, new platforms
```

### Pre-release Identifiers

Pre-release versions use hyphen identifiers for clarity:

| Identifier | Meaning | Use Case |
|-----------|---------|----------|
| `-alpha.N` | Alpha release | Internal development testing (N = iteration number) |
| `-beta.N` | Beta release | Beta testing with external users |
| `-rc.N` | Release candidate | Pre-production verification (final candidate before GA) |
| `-dev` | Development build | Developer builds from main branch |

**Examples:**
- `1.0.0-alpha.1` — First alpha of version 1.0
- `1.0.0-beta.3` — Third beta of version 1.0
- `1.0.0-rc.1` — First release candidate of version 1.0
- `1.0.0` — General availability release

### Build Metadata

Build metadata is appended with a `+` for internal tracking:

```
1.0.0+build.20260315.sha.abc1234
                     └───────────┬──────────┘
                                 └── Build date and git SHA
```

### Version Numbering Rules

| Change Type | Bump | Example |
|------------|------|---------|
| New feature (non-breaking) | `MINOR` bump | `1.0.0` → `1.1.0` |
| Bug fix | `PATCH` bump | `1.1.0` → `1.1.1` |
| Breaking change | `MAJOR` bump | `1.1.0` → `2.0.0` |
| Security patch | `PATCH` bump (or `MAJOR` if severe) | `1.1.1` → `1.1.2` or `2.0.0` |
| New platform support | `MINOR` bump (e.g., Linux in addition to Win/Mac) | `1.2.0` → `1.3.0` |

### Version History

The `VERSION` file in the repository root is the single source of truth for the current version:

```
1.0.0-rc.1
```

---

## Release Cadence

### Development Releases (Pre-MVP)

| Phase | Cadence | Purpose |
|-------|---------|---------|
| Alpha | Weekly | Internal developer testing |
| Beta | Bi-weekly | External beta tester distribution |
| Release Candidate | As needed | Pre-GA verification before each release |

### Post-MVP Releases

| Release Type | Cadence | Purpose |
|-------------|---------|---------|
| **Patch** | As needed (typically monthly) | Bug fixes, security patches |
| **Minor** | Monthly | New features, improvements, new skills |
| **Major** | Quarterly (every 3 months) | Architectural changes, breaking changes, major platform additions |

### Release Freeze Windows

Release activity is frozen during:
- **Critical support periods** — when the team is actively supporting a major customer deployment
- **Holiday periods** — no releases on or within 3 days of major holidays (to avoid burnout)
- **Security incident response** — if a security incident is being investigated, release activity stops until resolved

---

## Release Types

### Major Release (`MAJOR` bump)

Characterized by any of the following:
- Breaking API or protocol changes
- New operating system support (e.g., Linux)
- Major architectural refactoring
- New distribution channel (e.g., Microsoft Store)
- Changes to data storage format requiring migration

**Requirements:**
- Full release process (see [Release Process](#release-process))
- Public release notes published on website and changelog
- Migration guide provided if data format changes
- 2-week regression testing period before GA
- Customer communication sent at least 2 weeks before release
- Rollback plan tested and documented

### Minor Release (`MINOR` bump)

Characterized by any of the following:
- New features or skills
- Significant improvements to existing features
- New AI provider support
- UI enhancements
- Performance improvements

**Requirements:**
- Standard release process (see [Release Process])
- Release notes published on website and changelog
- 1-week regression testing period before GA

### Patch Release (`PATCH` bump)

Characterized by any of the following:
- Bug fixes
- Security patches
- Minor performance improvements
- Documentation updates

**Requirements:**
- Streamlined release process (see [Release Process])
- Patch notes published on website and changelog
- 3-day regression testing period for non-trivial fixes

### Hotfix Release

An emergency patch release triggered by:
- Critical security vulnerability
- Data corruption or loss risk
- App crash affecting a significant user base
- Credential exposure risk

**Requirements:**
- Immediate release (same day as detection)
- Post-incident review within 48 hours
- Emergency communication to all customers
- Hotfixes are applied to the most recent minor release branch, then merged back to `main`

---

## Release Process

The release process flows through five gates:

```
Develop → Code Complete → Staging → QA → Production
   │         │            │         │        │
   └─────────┴────────────┴─────────┴────────┘
         Gate 1       Gate 2       Gate 3     Gate 4
```

### Gate 1: Code Complete

**When:** All features for the release are merged to the release branch.

**Actions:**
1. Tag the commit with the version number (e.g., `v1.0.0-rc.1`)
2. Run full CI pipeline: lint, test, build, E2E
3. Generate MSI (Windows) and DMG (macOS) installers
4. Code sign both installers
4. Run smoke tests on both platforms in isolated VMs
5. Create release branch from `main` (if not already using one)

**Gate Criteria:**
- [ ] All CI checks pass
- [ ] No critical or blocker bugs open
- [ ] Release notes draft completed
- [ ] Installation verified on both Windows 11 and macOS Sonoma
- [ ] Auto-update mechanism tested (update from previous version)
- [ ] Code signed and notarized (macOS) or signed (Windows)

---

### Gate 2: Staging

**When:** Release candidate built and deployed to staging environment.

**Actions:**
1. Deploy staging builds to a staging update server
2. Beta testers download and install the release candidate via auto-update
3. Collect telemetry, error reports, and user feedback
4. Run automated regression test suite on staging builds
5. Verify data migration (if data format changed)

**Gate Criteria:**
- [ ] At least 5 beta testers have installed and used the release
- [ ] No crash reports from staging installations
- [ ] No security findings from staging testing
- [ ] User feedback collected and reviewed
- [ ] Regression test suite passes on staging builds

---

### Gate 3: QA

**When:** Release candidate passes staging and is ready for formal QA.

**Actions:**
1. QA team runs the full regression test suite
2. QA team verifies all release notes items
3. QA team performs security review (credential handling, data encryption, privacy controls)
4. QA team performs accessibility review (basic checks)
5. QA team performs performance benchmarks (startup time, response latency, memory usage)
6. QA signs off on the release

**Gate Criteria:**
- [ ] All regression tests pass
- [ ] Release notes items verified
- [ ] Security review passed (no vulnerabilities found)
- [ ] Performance within acceptable thresholds (see MVP_SCOPE.md quality criteria)
- [ ] QA sign-off obtained
- [ ] No new critical or blocker bugs introduced

---

### Gate 4: Production

**When:** Release is approved for production deployment.

**Actions:**
1. Update release branch with final version tag
2. Build final production installers (MSI and DMG)
3. Code sign and notarize installers
4. Upload installers to distribution servers
5. Update staging update server with the new version
6. Deploy to auto-update (phased rollout — see [Rollout Strategy](#rollout-strategy))
7. Publish release notes to website and changelog
8. Notify customers (see [Customer Communication Plan](#customer-communication-plan))
9. Monitor post-release metrics (see [Post-Release Monitoring](#post-release-monitoring))

**Gate Criteria:**
- [ ] QA sign-off obtained
- [ ] All installers built, signed, and uploaded
- [ ] Release notes published
- [ ] Rollout started (phased — not all-at-once)
- [ ] Monitoring dashboards active
- [ ] Rollback plan documented and ready
- [ ] Release team on standby for the first 48 hours

---

## Rollout Strategy

### Phased Rollout (Recommended for All Releases)

| Phase | Audience | Duration | Action |
|-------|----------|----------|--------|
| **Phase 0** | Internal team | 24 hours | Release to internal team only. Monitor for crashes and issues. |
| **Phase 1** | Beta testers | 48 hours | Release to beta testers. Collect feedback. Fix any remaining issues. |
| **Phase 2** | 10% of users | 1 week | Release to 10% of users. Monitor error rates, crash reports, and user feedback. |
| **Phase 3** | 50% of users | 1 week | Release to 50% of users if Phase 2 is clean. |
| **Phase 4** | 100% of users | Ongoing | Full release. |

### Canary Release (For Major Releases)

For major releases, add a canary phase:
- **Canary:** Release to 1-2 internal customer environments for 2 weeks
- **Alpha:** Internal team only (Phase 0 above)
- **Beta:** Beta testers (Phase 1 above)
- Then proceed through Phases 2-4

### Rollout Criteria

| Metric | Accept Threshold |
|--------|-----------------|
| Crash rate | < 0.5% of installations |
| Error report rate | < 2% of active users |
| Customer support tickets | < 5 new tickets per 100 installations |
| Auto-update install success rate | > 98% |
| User satisfaction (NPS) | No decline from previous version |

### Rollout Pausing

If any metric exceeds its threshold:
1. **Pause rollout** — stop deploying to the next phase
2. **Investigate** — determine root cause within 4 hours
3. **Fix or rollback** — deploy a fix or roll back to previous version
4. **Resume** — only after the issue is resolved and verified

---

## Update Delivery Mechanism

### Auto-Update (Primary)

The Tauri update plugin (`tauri-plugin-updater`) handles all auto-update delivery:

**Update Check Flow:**
1. App checks for updates on startup and manually (Settings → "Check for Updates")
2. Update server returns the latest version, download URL, and release notes
3. If a newer version is available:
   - User sees a notification: "Version X.Y.Z is available. Review changelog →"
   - Clicking "Review" opens the release notes
   - Clicking "Update Now" downloads and installs
4. Download happens in background
5. After installation, app restarts automatically
6. If installation fails, the previous version is restored

**Update Server:**
- Hosted on a CDN (e.g., Cloudflare R2, AWS S3 + CloudFront)
- Returns JSON metadata: `{ version, notes, date, url_windows, url_macos }`
- Signed release artifacts (SHA-256 checksum verification)
- Version comparison done client-side (client only installs if new version > installed version)

### Manual Installer (Fallback)

If auto-update fails or is disabled:
1. User downloads the latest installer from wiki-labs.ai/download
2. Runs the installer (MSI on Windows, DMG on macOS)
3. Installer detects existing installation and performs an in-place upgrade
4. If upgrade fails, installer offers to rollback to previous version

### Update Configuration

Update settings are controlled by the `tauri.conf.json` configuration:

```json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://updates.wiki-labs.ai/{{target}}/{{arch}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "dWtpZSBwdWIgL3Jlcy9lZDI1NTE5...",
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

**Key settings:**
- `active`: Enable/disable auto-update (configurable per customer in enterprise deployments)
- `dialog`: Show update confirmation dialog (`true` in MVP — user must confirm)
- `pubkey`: Public key for verifying signed updates
- `windows.installMode`: `"passive"` — shows progress dialog; `"quiet"` — silent install (enterprise only)

### Enterprise Configuration Override

Enterprise deployments can override update settings via configuration:

```yaml
# wiki-labs-copilot-config.yaml
update:
  enabled: true              # or false for air-gapped deployments
  auto_install: false         # default: user must confirm
  channel: stable            # stable, beta, or alpha
  custom_update_server:      # optional: enterprise-managed update server
    url: https://updates.internal.corp/wiki-labs/
    pubkey: "dWtpZSBwdWIgL3Jlcy9lZDI1NTE5..."
```

### Air-Gapped / Offline Install

For air-gapped enterprise environments:
1. IT admin downloads all installers from wiki-labs.ai/download
2. Installers are distributed internally (via SCCM, Jamf, Intune, etc.)
3. Update server is disabled (`update.enabled: false`)
4. Admin can configure a custom update server within the enterprise network

---

## Rollback Strategy

### Auto-Update Rollback

When an auto-update installation fails:
1. The Tauri updater detects the failure
2. The previous version is restored from the backup (created before the update)
3. User is notified: "Update failed. Reverted to version X.Y.Z. Please try again later or download manually."
4. The failed update is logged in the audit trail

**Rollback flow:**
```
1. Backup current installation (copy to .wiki-labs-copilot/backup/)
2. Download new version to temp directory
3. Install new version
4. Verify installation (run smoke test)
5. If verification fails → Restore backup, delete temp
6. If verification passes → Remove backup, show success
```

### Manual Rollback

If a user wants to roll back manually:
1. User downloads the previous version installer from wiki-labs.ai/versions
2. Runs the installer (uninstalls the current version, installs the previous one)
3. Data and configuration are preserved (same storage paths)

### Emergency Rollback

If a major issue is discovered after production deployment:
1. **Stop rollout** — pause at the current phase immediately
2. **Hotfix or rollback** — decide within 4 hours:
   - If fix is simple: develop and deploy hotfix
   - If fix is complex: rollback to previous version
3. **Communicate** — notify all customers of the issue and rollback
4. **Investigate** — post-incident review within 48 hours
5. **Fix permanently** — develop, test, and release the permanent fix

### Rollback Testing

Rollback capability is tested as part of the QA gate for every release:
- Test rolling back from version X.Y.Z to X.Y.(Z-1)
- Verify data integrity after rollback (knowledge, sessions, settings, credentials)
- Verify the rollback installer works on both Windows and macOS

---

## Release Notes Template

Release notes are published on the website, in the app (via update dialog), and in the repository's `CHANGELOG.md`.

```markdown
# Release Notes — v{version}

**Release Date:** {YYYY-MM-DD}  
**Release Type:** {Major | Minor | Patch | Hotfix}

---

## What's New

### Features
- {Feature description with link to documentation if applicable}
- {Feature description}

### Skills
- Added: {New skill name and brief description}
- Improved: {Existing skill with specific improvements}

### Improvements
- {Specific improvement}
- {Specific improvement}

---

## What's Fixed

- {Bug description, including what was broken and what now works}
- {Bug description}
- {Security fix description}

---

## What's Changed

- {Breaking change or behavior change, if any}
- {Configuration change, if any}
- {Data format change, if any — with migration instructions}

---

## Upgrade Notes

### From v{previous_version} to v{current_version}
- {Migration instructions, if needed}
- {Configuration changes, if any}
- {Known issues, if any}

### System Requirements
- Windows 11 (22H2 or later)
- macOS Sonoma (14.0 or later)
- Minimum 8 GB RAM (16 GB recommended)

---

## Known Issues

- {Known issue 1, with workarounds if available}
- {Known issue 2}

---

## Download

- [Windows MSI Installer]({url})
- [macOS DMG Installer]({url})
- [Full Changelog]({changelog_url})
```

---

## Deployment Checklist

Use this checklist for every release (copy and fill in for each release).

### Pre-Release

- [ ] All features for this release are merged to the release branch
- [ ] Release branch created from `main` (if this is a minor/major release)
- [ ] Version bump committed and pushed
- [ ] Release notes draft completed
- [ ] All CI checks pass (lint, test, build, E2E)
- [ ] Code signing certificate valid (not expired)
- [ ] macOS developer certificate valid (not expired)
- [ ] Windows EV certificate valid (not expired)

### Build

- [ ] MSI built for Windows 11 (x64)
- [ ] DMG built for macOS Sonoma (x64 + ARM64 universal binary)
- [ ] Installers code signed
- [ ] macOS notarized with Apple Notarization service
- [ ] SHA-256 checksums generated for all installers
- [ ] Installers uploaded to update server and CDN
- [ ] Version metadata published to update server

### Staging

- [ ] Staging builds deployed to staging update server
- [ ] At least 5 beta testers installed and used the release
- [ ] No crash reports from staging
- [ ] User feedback collected and reviewed
- [ ] Regression test suite passes on staging builds
- [ ] Staging gate approved

### QA

- [ ] Full regression test suite passes
- [ ] Release notes items verified
- [ ] Security review passed (no vulnerabilities)
- [ ] Performance benchmarks within thresholds
- [ ] Accessibility checks passed (basic)
- [ ] Rollback tested (backup → install → verify → rollback → verify data)
- [ ] Auto-update tested (download → install → restart → verify)
- [ ] Data migration tested (if data format changed)
- [ ] QA sign-off obtained

### Production

- [ ] Final installers built, signed, notarized, and uploaded
- [ ] Release branch tagged with version
- [ ] Release notes published to website and changelog
- [ ] Auto-update enabled (phased rollout started)
- [ ] Monitoring dashboards active (crash rate, error rate, update success rate)
- [ ] Rollback plan documented and team notified
- [ ] Customer communication sent (email + in-app notification)
- [ ] Release team on standby for 48 hours

### Post-Release

- [ ] 24 hours post-release review conducted
- [ ] 48 hours post-release review conducted
- [ ] Release branch merged back to `main` (for minor/major releases)
- [ ] Release branch tagged with final version
- [ ] Lessons learned documented
- [ ] Customer feedback analyzed
- [ ] Release retrospective scheduled (within 1 week of release)

---

## Post-Release Monitoring

### Telemetry (Opt-In)

Telemetry is opt-in and clearly disclosed in the privacy policy. It helps us understand how the copilot is performing in the wild:

| Metric | What It Tracks | Why It Matters |
|--------|---------------|----------------|
| **Crash Reports** | App crashes, panics, unhandled exceptions | Detect critical bugs quickly |
| **Error Reports** | AI provider errors, update failures, skill errors | Track reliability of core features |
| **Performance Metrics** | Startup time, response latency, suggestion latency | Ensure quality targets are met |
| **Feature Usage** | Which features are used, how often | Guide prioritization of future work |
| **Helpfulness Ratings** | 👍/👎 on responses | Measure AI quality |
| **Update Metrics** | Update download/install success rate | Verify update mechanism reliability |
| **Session Duration** | Average session length | Understand engagement patterns |

### Dashboards

Post-release, the following dashboards are monitored:

1. **Crash Dashboard** — Real-time crash count and crash-free percentage by version
2. **Error Dashboard** — Error count by type (AI provider, skill, update, general)
3. **Update Dashboard** — Update install success/failure rates by version and platform
4. **Performance Dashboard** — Response latency percentiles (p50, p90, p95, p99)
5. **Release Health Dashboard** — Combined view of all the above, with alerts

### Alert Thresholds

| Metric | Warning Threshold | Critical Threshold |
|--------|------------------|-------------------|
| Crash-free rate | < 99.5% | < 99.0% |
| Error report rate | > 1% | > 3% |
| Update install success | < 97% | < 95% |
| New support tickets per 100 users | > 10 | > 20 |

### Alert Response

| Severity | Response Time | Who Responds |
|----------|--------------|-------------|
| **Critical** | Within 1 hour | On-call engineer + release lead |
| **Warning** | Within 4 hours | On-call engineer |
| **Info** | Next business day | Engineering team |

---

## Customer Communication Plan

### Communication Channels

| Channel | Use Case | Frequency |
|---------|----------|-----------|
| **In-App Notification** | Update available, important security notices | As needed |
| **Email** | Major releases, security advisories, maintenance windows | Per release/advisory |
| **Website Changelog** | Detailed release notes | Per release |
| **Status Page** | System status, incident communication | As needed |
| **Customer Success Manager** | Enterprise customer coordination, training | Per release (enterprise only) |
| **Support Tickets** | Reactive customer inquiries | Ongoing |

### Communication Timeline

| Time | Action | Audience |
|------|--------|----------|
| **2 weeks before release** | Notify enterprise customers of upcoming major release with migration guide | Enterprise customers |
| **1 week before release** | Publish pre-release notes (beta/RC) | Beta testers, changelog |
| **Day of release** | Publish release notes to website and changelog | All customers |
| **Day of release** | Send email notification with release highlights | All customers (email subscribers) |
| **Day of release** | Display release notes in-app (via update dialog) | All customers |
| **24 hours after release** | Follow-up email with known issues (if any) and early adopter feedback | All customers |
| **1 week after release** | Release summary with adoption metrics and customer feedback | All customers |
| **Ongoing** | Respond to support tickets and customer inquiries | Individual customers |

### Enterprise Customer Communication

For enterprise customers, add these steps:
1. **Pre-release coordination (2 weeks before):** Enterprise CSM contacts each customer's IT contact
2. **Migration guide (2 weeks before):** If data format changes, provide detailed migration guide
3. **Testing window (1 week before):** Enterprise CSM coordinates with customer's QA team
4. **Deployment coordination (day of release):** Suggest deployment window with customer's IT team
5. **Post-release check-in (1 week after):** Enterprise CSM follows up with customer to ensure smooth adoption

### Security Advisory Communication

If a security issue is discovered that requires a patch:

1. **Internal notification (immediate):** Security team notifies engineering and customer success
2. **Hotfix development (same day):** Hotfix is developed and tested
3. **Customer notification (same day):** Email sent to all customers describing the issue, impact, and fix
4. **Hotfix deployment (same day):** Hotfix released via auto-update
5. **Post-incident review (48 hours):** Security team conducts post-incident review and publishes internal findings
6. **Public advisory (if applicable):** If the vulnerability is publicly disclosed, publish a public security advisory

### Crisis Communication

If a release causes a widespread issue:

1. **Immediate response (within 1 hour):** On-call engineer investigates and assesses scope
2. **Status page update (within 1 hour):** Post incident on status page
3. **Customer notification (within 2 hours):** Email to all customers with known issues and status
4. **Rollback (within 4 hours):** If a fix is not immediately available, rollback to previous version
5. **Follow-up (within 24 hours):** Update with investigation progress and ETA for fix
6. **Resolution (when fixed):** Update with resolution details
7. **Post-incident report (within 1 week):** Publish detailed post-incident report

---

*See also: [ROADMAP.md](./ROADMAP.md) · [MVP_SCOPE.md](./MVP_SCOPE.md) · [BACKLOG.md](./BACKLOG.md)*