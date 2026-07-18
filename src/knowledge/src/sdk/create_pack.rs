//! Creates new knowledge pack scaffolding using templates.

use crate::sdk::templates::Template;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Creates a new knowledge pack at the specified output directory.
///
/// # Arguments
/// * `template_name` — Name of the template to use (e.g. "openshift", "engineering", "documentation")
/// * `output_dir` — Directory where the pack scaffolding will be created
/// * `pack_name` — Optional override for the pack name (defaults to template name)
pub fn create_openshift(output_dir: &str) -> Result<String> {
    create_pack("openshift", output_dir, None)
}

/// Creates a new knowledge pack with a custom name.
///
/// # Arguments
/// * `template_name` — Name of the template
/// * `output_dir` — Directory where the pack scaffolding will be created
/// * `custom_name` — Optional custom pack name
pub fn create_pack(
    template_name: &str,
    output_dir: &str,
    custom_name: Option<String>,
) -> Result<String> {
    let template =
        Template::from_name(template_name).context(format!("Unknown template: {}", template_name))?;

    let pack_name = custom_name.unwrap_or_else(|| {
        let base = template_name.to_string();
        format!("{}-pack", base)
    });

    let base_path = Path::new(output_dir).join(&pack_name);
    fs::create_dir_all(&base_path)
        .with_context(|| format!("Failed to create directory: {}", base_path.display()))?;

    info!(
        pack_name = %pack_name,
        output_dir = %output_dir,
        template = %template_name,
        "Creating knowledge pack scaffolding"
    );

    // Write manifest.yaml
    write_file(
        &base_path,
        "manifest.yaml",
        &serde_yaml::to_string(&template.manifest)
            .context("Failed to serialize manifest")?,
    )?;

    // Write metadata.yaml
    write_file(
        &base_path,
        "metadata.yaml",
        &serde_yaml::to_string(&template.metadata)
            .context("Failed to serialize metadata")?,
    )?;

    // Write directory structure
    for (path, is_file) in &template.directory_structure {
        if is_file {
            let file_path = base_path.join(path);
            let dir = file_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("No parent directory for {}", path))?;
            fs::create_dir_all(dir)?;

            let content = generate_file_content(template_name, path);
            write_file(&base_path, path, &content)?;
        } else {
            // Directory marker — create if not exists
            let dir_path = base_path.join(path);
            fs::create_dir_all(&dir_path)?;
            debug!(path = %dir_path.display(), "Created directory for knowledge pack");
        }
    }

    info!(pack_name = %pack_name, "Knowledge pack scaffolding created successfully");
    Ok(base_path.to_string_lossy().to_string())
}

/// Writes content to a file, creating parent directories as needed.
fn write_file(base_path: &Path, relative_path: &str, content: &str) -> Result<()> {
    let file_path = base_path.join(relative_path);
    fs::write(&file_path, content).with_context(|| {
        format!(
            "Failed to write file: {}",
            file_path.to_string_lossy()
        )
    })
}

/// Generates default content for template files.
fn generate_file_content(template_name: &str, path: &str) -> String {
    match path {
        "tests/validation.test" => format!(
            "# Validation test for {} knowledge pack\n# Run with: cargo test\n# This file serves as a placeholder for your knowledge pack tests.\n\nfn test_pack_structure() {{\n    // Verify manifest and metadata files exist and are valid\n}}\n\nfn test_documents_accessible() {{\n    // Verify all documents referenced in manifest.yaml are accessible\n}}\n\nfn test_embedding_compatibility() {{\n    // Verify embedding model specified in metadata.yaml is compatible\n}}\n",
            template_name
        ),
        "documentation/README.md" => format!(
            "# {} Knowledge Pack\n\n## Overview\n\nThis knowledge pack contains structured documentation for {}.\n\n## Structure\n\n- `manifest.yaml` — Pack manifest with document list and dependencies\n- `metadata.yaml` — Pack metadata including embedding configuration\n- `documents/` — Knowledge documents (markdown, PDF, text)\n- `tests/` — Validation and integration tests\n- `documentation/` — Additional pack documentation\n\n## Embedding\n\n- Model: all-MiniLM-L6-v2\n- Dimensions: 384\n\n## Usage\n\nImport this pack into a workspace to enable knowledge retrieval.\n\n## License\n\nMIT\n",
            template_name.chars().next().unwrap_or(' ').to_uppercase().to_string()
                + &template_name.chars().skip(1).collect::<String>(),
            template_name
        ),
        path if path.starts_with("documents/") => {
            let doc_name = path
                .trim_start_matches("documents/")
                .trim_end_matches(".md");
            format!(
                "# {}\n\n## Introduction\n\nThis document is part of the {} knowledge pack.\n\n## Content\n\nReplace this placeholder with your actual knowledge content.\n\n## References\n\nAdd references and links to external resources here.\n",
                doc_name.replace('-', " ").replace('_', " "),
                template_name
            )
        }
        _ => "# Knowledge Pack\n\nThis file will be populated with pack-specific content.\n".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_openshift_pack() {
        let tmp = TempDir::new().unwrap();
        let output = create_openshift(tmp.path().to_str().unwrap()).unwrap();

        let pack_path = Path::new(&output);
        assert!(pack_path.join("manifest.yaml").exists());
        assert!(pack_path.join("metadata.yaml").exists());
        assert!(pack_path.join("documents/").is_dir());
        assert!(pack_path.join("tests/").is_dir());
        assert!(pack_path.join("documentation/").is_dir());
        assert!(pack_path.join("documents/getting-started.md").exists());
        assert!(pack_path.join("documents/operator-basics.md").exists());
    }

    #[test]
    fn test_create_pack_custom_name() {
        let tmp = TempDir::new().unwrap();
        let output = create_pack("openshift", tmp.path().to_str().unwrap(), Some("my-ocp-pack"))
            .unwrap();

        let pack_path = Path::new(&output);
        assert!(pack_path.exists());
        assert!(pack_path.join("manifest.yaml").exists());
    }

    #[test]
    fn test_create_engineering_pack() {
        let tmp = TempDir::new().unwrap();
        let output = create_pack("engineering", tmp.path().to_str().unwrap(), None).unwrap();

        let pack_path = Path::new(&output);
        assert!(pack_path.join("manifest.yaml").exists());
        assert!(pack_path.join("documents/architecture-patterns.md").exists());
    }

    #[test]
    fn test_create_documentation_pack() {
        let tmp = TempDir::new().unwrap();
        let output = create_pack("documentation", tmp.path().to_str().unwrap(), None).unwrap();

        let pack_path = Path::new(&output);
        assert!(pack_path.join("manifest.yaml").exists());
        assert!(pack_path.join("documents/api-reference.md").exists());
    }

    #[test]
    fn test_create_pack_unknown_template() {
        let tmp = TempDir::new().unwrap();
        let result = create_pack("nonexistent", tmp.path().to_str().unwrap(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_yaml_deserializes() {
        let tmp = TempDir::new().unwrap();
        let output = create_openshift(tmp.path().to_str().unwrap()).unwrap();

        let manifest_content =
            fs::read_to_string(format!("{}/manifest.yaml", output)).unwrap();
        let manifest: crate::sdk::schema::Manifest =
            serde_yaml::from_str(&manifest_content).unwrap();
        assert_eq!(manifest.schema_version, "1.0");
        assert_eq!(manifest.documents.len(), 2);
    }

    #[test]
    fn test_metadata_yaml_deserializes() {
        let tmp = TempDir::new().unwrap();
        let output = create_openshift(tmp.path().to_str().unwrap()).unwrap();

        let metadata_content =
            fs::read_to_string(format!("{}/metadata.yaml", output)).unwrap();
        let metadata: crate::sdk::schema::Metadata =
            serde_yaml::from_str(&metadata_content).unwrap();
        assert_eq!(metadata.embedding_model, "all-MiniLM-L6-v2");
        assert_eq!(metadata.embedding_dimensions, 384);
    }
}