# Skill Platform — Quick Start

## Prerequisites

- Rust 2021 edition
- Workspace with `src/skill_runtime/`, `src/skill_discovery/`, `src/skill_activation/`, `src/skill_sdk/`

## 1. Create a Skill

```bash
# Use the SDK to generate a skill template
# In code:
let sdk = SkillSDK::new("/path/to/sdk/dir")?;
let template = sdk.create_skill_template("my-skill")?;

// Write generated files to disk
for file in &template.generated_files {
    let path = PathBuf::from("/path/to/skills/my-skill").join(&file.path);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, &file.content)?;
}
```

## 2. Register with Discovery Engine

```rust
use wikilabs_skill_runtime::SkillDiscoveryEngine;
use wikilabs_skill_runtime::TechSignature;

let mut engine = SkillDiscoveryEngine::new();
engine.register_signature(TechSignature {
    domain: "MyTech".to_string(),
    file_patterns: vec!["**/myconfig.yaml".to_string()],
    command_patterns: vec!["mycommand".to_string()],
    base_confidence: 0.8,
    priority: 8,
});

// Register known skills
engine.register_known_skill(KnownSkill {
    id: "my-skill".to_string(),
    name: "My Skill".to_string(),
    technology: "MyTech".to_string(),
    category: "Engineering".to_string(),
    required_signals: vec!["MyTech".to_string()],
    optional_signals: vec![],
});
```

## 3. Set Up Skill Runtime

```rust
use wikilabs_skill_runtime::SkillRuntime;
use wikilabs_skill_runtime::DiscoveryConfig;
use wikilabs_skill_runtime::ActivationConfig;

let mut runtime = SkillRuntime::new("/path/to/skills");

// Load existing skills
runtime.discover_skills()?;
for skill_id in runtime.discover_skills()? {
    runtime.load_skill(&skill_id)?;
    runtime.enable_skill(&skill_id)?;
}

// Configure discovery and activation engines
let discovery_config = DiscoveryConfig::default();
let activation_config = ActivationConfig::default();
runtime.setup_engines(discovery_config, activation_config);
```

## 4. Scan and Activate

```rust
// Scan workspace
if let Some(report) = runtime.scan_workspace() {
    info!("Found {} signals, {} skills", report.technologies.len(), report.skills.len());
}

// Activate from signals
let signals = vec![
    TechSignal {
        id: "MyTech-file-1".to_string(),
        technology: "MyTech".to_string(),
        source: "/path/to/myconfig.yaml".to_string(),
        confidence: 0.85,
        priority: 8,
        pattern: "**/myconfig.yaml".to_string(),
        metadata: HashMap::new(),
    }
];

let activated = runtime.activate_from_signals(&signals)?;
for skill in &activated {
    info!("Activated: {} (confidence: {:.2})", skill.skill_name, skill.confidence);
}
```

## 5. Health Checks

```rust
// Periodic health checks
runtime.health_check_all()?;

// Check specific skill
if let Some(active_skills) = runtime.activation_engine() {
    for skill_id in active_skills.skill_ids() {
        // Health check is done internally during health_check_all()
    }
}
```

## 6. Validate a Skill

```rust
let sdk = SkillSDK::new("/path/to/sdk")?;
let report = sdk.validate_skill("/path/to/my-skill")?;

if report.is_valid {
    info!("Skill is valid");
} else {
    for error in &report.errors {
        warn!("Validation error: {}", error);
    }
}
```

## Full Example

```rust
use wikilabs_skill_runtime::*;
use std::collections::HashMap;

fn main() -> Result<()> {
    // 1. Create runtime
    let mut runtime = SkillRuntime::new("/path/to/skills");

    // 2. Load skills
    runtime.discover_skills()?;
    for id in runtime.discover_skills()? {
        runtime.load_skill(&id)?;
        runtime.enable_skill(&id)?;
    }

    // 3. Setup engines
    let discovery_config = DiscoveryConfig {
        scan_dirs: vec![PathBuf::from(".")],
        file_patterns: vec!["**/*.yaml".to_string()],
        confidence_threshold: 0.3,
        check_commands: true,
    };
    let activation_config = ActivationConfig {
        auto_activate: true,
        auto_activate_threshold: 0.5,
        health_checks: true,
        health_check_interval: 300,
        max_failure_count: 3,
    };
    runtime.setup_engines(discovery_config, activation_config);

    // 4. Scan workspace
    if let Some(report) = runtime.scan_workspace() {
        println!("Discovered {} technologies", report.technologies.len());
        println!("Discovered {} skills", report.skills.len());
    }

    Ok(())
}
```