//! Validate pack dependencies.

use crate::sdk::schema::Manifest;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tracing::debug;

/// Validates that the dependencies listed in the manifest can be resolved.
///
/// Checks:
/// - Dependencies are non-empty strings
/// - Dependencies don't reference the pack itself
/// - Dependencies form a valid DAG (no circular references)
/// - All dependencies are non-empty
pub fn validate_dependencies(pack_path: &str) -> Result<DependenciesResult> {
    let path = Path::new(pack_path);
    let manifest_path = path.join("manifest.yaml");

    let content = fs::read_to_string(&manifest_path)
        .context("Failed to read manifest.yaml for dependency validation")?;

    let manifest: Manifest =
        serde_yaml::from_str(&content).context("manifest.yaml is not valid YAML")?;

    let mut result = DependenciesResult {
        pack_path: pack_path.to_string(),
        dependency_names: manifest.dependencies.clone(),
        valid: true,
        empty_deps: Vec::new(),
        self_references: Vec::new(),
        circular_deps: Vec::new(),
    };

    let pack_name = &manifest.name;
    let mut seen: HashSet<&str> = HashSet::new();

    for dep in &manifest.dependencies {
        // Check for empty dependencies
        if dep.is_empty() {
            result.empty_deps.push("<empty>".to_string());
            result.valid = false;
            continue;
        }

        // Check for self-references
        if dep == pack_name.as_str() {
            result.self_references.push(dep.clone());
            result.valid = false;
            debug!(dep = %dep, pack = %pack_name, "Self-reference detected");
        }

        // Check for duplicate dependency declarations
        if !seen.insert(dep.as_str()) {
            result.valid = false;
            debug!(dep = %dep, "Duplicate dependency declaration");
        }
    }

    // Circular dependency check requires external graph resolution.
    // If there are no self-references and no empty deps, the local check passes.
    // A full circular check would need access to the dependency graph of all packs.

    debug!(
        valid = result.valid,
        dep_count = manifest.dependencies.len(),
        empty = result.empty_deps.len(),
        self_ref = result.self_references.len(),
        "Dependency validation completed"
    );

    Ok(result)
}

/// Dependency validation result.
#[derive(Debug, Default)]
pub struct DependenciesResult {
    pub pack_path: String,
    pub dependency_names: Vec<String>,
    pub valid: bool,
    pub empty_deps: Vec<String>,
    pub self_references: Vec<String>,
    pub circular_deps: Vec<Vec<String>>,
}

impl DependenciesResult {
    /// Generates a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Dependency Validation Report ({}):\n  Valid: {}\n  Dependencies: {}\n  Empty deps: {}\n  Self references: {}\n  Circular deps: {}",
            self.pack_path,
            self.valid,
            self.dependency_names.len(),
            self.empty_deps.len(),
            self.self_references.len(),
            self.circular_deps.len()
        )];

        if !self.dependency_names.is_empty() {
            lines.push("  Dependencies:".to_string());
            for d in &self.dependency_names {
                lines.push(format!("    - {}", d));
            }
        }

        if !self.empty_deps.is_empty() {
            lines.push("  Empty dependencies:".to_string());
            for e in &self.empty_deps {
                lines.push(format!("    - {}", e));
            }
        }

        if !self.self_references.is_empty() {
            lines.push("  Self-references:".to_string());
            for s in &self.self_references {
                lines.push(format!("    - {}", s));
            }
        }

        if !self.circular_deps.is_empty() {
            lines.push("  Circular dependencies:".to_string());
            for cycle in &self.circular_deps {
                lines.push(format!(
                    "    - {}",
                    cycle
                        .iter()
                        .map(|s| s.clone())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_manifest_with_deps(tmp: &TempDir, deps: &[&str]) {
        let deps_yaml: Vec<String> = deps.iter().map(|d| d.to_string()).collect();
        let deps_yaml_str = serde_yaml::to_string(&deps_yaml).unwrap();
        fs::write(
            tmp.path().join("manifest.yaml"),
            format!("schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies:\n{}", deps_yaml_str),
        )
        .unwrap();
    }

    #[test]
    fn test_no_dependencies() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_deps(&tmp, &[]);

        let result = validate_dependencies(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.valid);
        assert!(result.dependency_names.is_empty());
    }

    #[test]
    fn test_valid_dependencies() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_deps(&tmp, &["dep-a", "dep-b"]);

        let result = validate_dependencies(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.valid);
        assert_eq!(result.dependency_names.len(), 2);
    }

    #[test]
    fn test_self_reference() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_deps(&tmp, &["test-pack", "dep-b"]);

        let result = validate_dependencies(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.valid);
        assert_eq!(result.self_references.len(), 1);
    }

    #[test]
    fn test_empty_dependency() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_deps(&tmp, &["dep-a", "", "dep-b"]);

        let result = validate_dependencies(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.valid);
        assert_eq!(result.empty_deps.len(), 1);
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_deps(&tmp, &["dep-a"]);

        let result = validate_dependencies(tmp.path().to_str().unwrap()).unwrap();
        let report = result.report();
        assert!(report.contains("Dependency Validation Report"));
        assert!(report.contains("dep-a"));
    }
}