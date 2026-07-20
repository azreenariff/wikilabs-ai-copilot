# Skill Discovery

> Wiki Labs AI Copilot v0.8.0-alpha  
> Phase 11 — Enterprise Skill Platform

## Purpose

The Skill Discovery Engine scans the engineer's workspace to detect which technologies are present, then maps those technologies to available skills with confidence scoring.

## Discovery Flow

```
Engineer Activity (screen, terminal, browser, apps)
        │
        ▼
┌─────────────────────┐
│  Observation         │  ← Current system (reads signals from
│  Framework           │   screen capture, window titles, etc.)
└─────────┬───────────┘
          │ raw signals
          ▼
┌─────────────────────┐
│  Skill Discovery     │  ← Scans for technology signatures
│  Engine              │    using glob patterns, command detection,
│                      │    config file matching
└─────────┬───────────┘
          │ DiscoveryReport
          ▼
┌─────────────────────┐
│  Skill Activation    │  ← Matches discovered technologies
│  Engine              │    to skill definitions, resolves
│                      │    dependencies, activates
└─────────────────────┘
```

## Inputs

The discovery engine receives signals from these sources:

| Source | What it provides | Example |
|--------|-----------------|---------|
| **File patterns** | File paths matching glob patterns | `**/manifest.yaml`, `**/Dockerfile` |
| **Command detection** | Shell commands present in terminal | `systemctl`, `oc`, `kubectl` |
| **Configuration files** | Presence of config files | `/etc/ssh/sshd_config`, `/etc/docker/daemon.json` |
| **Application detection** | Currently active applications | `VMware Client`, `Chrome (vCenter)` |
| **Browser URLs** | URLs loaded in browser | `https://ocp.example.com/console` |
| **Window titles** | Text from application windows | `vSphere Client - Inventory` |

## Core Types

### TechSignal

A single detected technology signature.

```rust
pub struct TechSignal {
    pub id: String,           // Unique signal ID
    pub technology: String,   // Technology domain (e.g., "Linux")
    pub source: String,       // Source of the signal (file path, command, etc.)
    pub confidence: f64,      // 0.0 – 1.0 confidence score
    pub priority: u32,        // Priority level (higher = more important)
    pub pattern: String,      // Pattern that matched
    pub metadata: HashMap<String, String>, // Extracted metadata
}
```

### TechSignature

A registered pattern the engine matches against. Defined in the manifest or runtime config.

```rust
pub struct TechSignature {
    pub domain: String,              // Technology domain
    pub file_patterns: Vec<String>,  // Glob patterns to match files
    pub command_patterns: Vec<String>, // Shell commands to look for
    pub base_confidence: f64,        // Default confidence when matched
    pub priority: u32,               // Priority level
}
```

### DiscoveredSkill

A skill candidate discovered during the scan.

```rust
pub struct DiscoveredSkill {
    pub id: String,
    pub name: String,
    pub technology: String,
    pub category: String,
    pub confidence: f64,
    pub signal_count: usize,
    pub signals: Vec<TechSignal>,
    pub has_definition: bool,
    pub recommendation: String,
}
```

### DiscoveryReport

Full report delivered to the Skill Activation Engine.

```rust
pub struct DiscoveryReport {
    pub technologies: Vec<TechSignal>,  // All detected technology signals
    pub skills: Vec<DiscoveredSkill>,   // Skills that should activate
    pub scanned_dirs: Vec<String>,      // Directories that were scanned
    pub files_scanned: usize,           // Total files examined
    pub scan_timestamp: String,         // ISO timestamp of scan
}
```

## Configuration

```rust
pub struct DiscoveryConfig {
    pub scan_paths: Vec<PathBuf>,       // Directories to scan
    pub signal_threshold: f64,          // Minimum confidence to report (0.0–1.0)
    pub ignored_directories: Vec<String>, // Skip these paths
    pub scan_file_patterns: Vec<String>, // File patterns to look for
    pub command_patterns: Vec<String>,   // Commands to detect
    pub timeout_ms: u64,               // Scan timeout
}
```

## Discovery Algorithm

1. **Register signatures** — Load `TechSignature` definitions from manifests and config
2. **Scan directories** — Walk `scan_paths` looking for `file_patterns`
3. **Detect commands** — Check terminal output for `command_patterns`
4. **Match signals** — Compare findings against registered signatures
5. **Score confidence** — Multiply base confidence by signal count and source reliability
6. **Rank skills** — Sort by confidence score, then priority
7. **Filter** — Remove signals below `signal_threshold`
8. **Report** — Return `DiscoveryReport` to activation engine

## Confidence Calculation

```
confidence = base_confidence × (1 + 0.1 × signal_count)
```

Where:
- `base_confidence` comes from the registered `TechSignature`
- `signal_count` is the number of distinct signals supporting the same technology
- Signal types (file, command, config) provide different confidence multipliers
- Multiple independent signal types increase confidence multiplicatively

## Example: Linux Detection

```
Scan finds:
  - /etc/systemd/system/        (file pattern match)     → confidence 0.9
  - systemctl                   (command detection)      → confidence 0.85
  - /etc/ssh/sshd_config        (file pattern match)     → confidence 0.9
  - /proc/1/comm == systemd     (process detection)      → confidence 0.95

Result:
  TechSignal: Linux, confidence = 0.9 × (1 + 0.1 × 4) = 1.26 → capped at 1.0
  DiscoveredSkill: linux-engineering, confidence = 1.0, signal_count = 4
```

## Example: OpenShift Detection

```
Scan finds:
  - https://ocp.example.com/console (browser URL)          → confidence 0.95
  - oc                                (command detection)    → confidence 0.9
  - kubectl                           (command detection)    → confidence 0.8
  - .kube/config                       (file pattern match)  → confidence 0.85

Result:
  TechSignal: Kubernetes/OpenShift, confidence = 0.95 × (1 + 0.1 × 4) = 1.325 → capped at 1.0
  DiscoveredSkill: openshift-engineering, confidence = 1.0, signal_count = 4
```

## Built-in Signatures

The Skill Runtime registers default signatures for common technologies:

| Domain | File Patterns | Command Patterns | Base Confidence |
|--------|--------------|-----------------|-----------------|
| Linux | `**/manifest.yaml`, `**/Dockerfile` | `uname`, `lsmod`, `iptables` | 0.7 |
| Docker | `**/Dockerfile`, `**/docker-compose.yml` | `docker`, `docker-compose` | 0.7 |
| Kubernetes | `**/.kube/config` | `kubectl`, `helm` | 0.8 |
| SSH | `**/sshd_config` | `ssh`, `scp` | 0.9 |
| Git | `**/.git/config` | `git` | 0.9 |

## Testing

The discovery engine includes unit tests for:

- `test_scan_empty_dir` — Empty directory produces empty report
- `test_register_known_skill` — Skills register with their signatures
- `test_scan_with_file_patterns` — File pattern matching works correctly
- `test_confidence_scoring` — Confidence calculation is correct
- `test_multiple_signal_confidence` — Multiple signals boost confidence
- `test_command_pattern_matching` — Command detection identifies patterns
- `test_signal_priority_ranking` — Signals are ranked by priority
- `test_filter_below_threshold` — Low-confidence signals are filtered out
- `test_discovery_report_structure` — Report contains all required fields

## Integration

The discovery engine does NOT decide when to activate skills. It only produces reports. The Skill Activation Engine consumes reports and decides activation based on confidence thresholds and dependency resolution.