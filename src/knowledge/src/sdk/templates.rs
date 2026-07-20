//! Templates for knowledge pack generation — predefined starter configurations.

use crate::sdk::schema::{DocumentManifest, Manifest, Metadata};

/// Supported template categories.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateCategory {
    /// OpenShift / Kubernetes knowledge packs.
    Openshift,
    /// General engineering knowledge.
    Engineering,
    /// Documentation / reference knowledge.
    Documentation,
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateCategory::Openshift => write!(f, "openshift"),
            TemplateCategory::Engineering => write!(f, "engineering"),
            TemplateCategory::Documentation => write!(f, "documentation"),
        }
    }
}

/// Predefined template for OpenShift knowledge packs.
pub fn openshift_template() -> Template {
    Template {
        category: TemplateCategory::Openshift,
        manifest: Manifest {
            schema_version: "1.0".to_string(),
            name: "openshift-knowledge".to_string(),
            version: "1.0.0".to_string(),
            description:
                "Knowledge pack for OpenShift container platform documentation and best practices"
                    .to_string(),
            author: "Wiki Labs Team".to_string(),
            license: "MIT".to_string(),
            format_version: "1.0".to_string(),
            documents: vec![
                DocumentManifest {
                    id: "getting-started".to_string(),
                    path: "documents/getting-started.md".to_string(),
                    format: "markdown".to_string(),
                    embed: true,
                },
                DocumentManifest {
                    id: "operator-basics".to_string(),
                    path: "documents/operator-basics.md".to_string(),
                    format: "markdown".to_string(),
                    embed: true,
                },
            ],
            dependencies: vec![],
        },
        metadata: Metadata::new(
            "openshift-knowledge",
            "1.0.0",
            "Comprehensive OpenShift knowledge base",
            "all-MiniLM-L6-v2",
        ),
        directory_structure: vec![
            ("documents/".to_string(), false),
            ("documents/getting-started.md".to_string(), false),
            ("documents/operator-basics.md".to_string(), false),
            ("tests/".to_string(), false),
            ("tests/validation.test".to_string(), false),
            ("documentation/".to_string(), false),
            ("documentation/README.md".to_string(), false),
        ],
    }
}

/// Predefined template for general engineering knowledge.
pub fn engineering_template() -> Template {
    Template {
        category: TemplateCategory::Engineering,
        manifest: Manifest {
            schema_version: "1.0".to_string(),
            name: "engineering-knowledge".to_string(),
            version: "1.0.0".to_string(),
            description:
                "Knowledge pack for engineering best practices, design patterns, and methodologies"
                    .to_string(),
            author: "Wiki Labs Team".to_string(),
            license: "MIT".to_string(),
            format_version: "1.0".to_string(),
            documents: vec![DocumentManifest {
                id: "architecture-patterns".to_string(),
                path: "documents/architecture-patterns.md".to_string(),
                format: "markdown".to_string(),
                embed: true,
            }],
            dependencies: vec![],
        },
        metadata: Metadata::new(
            "engineering-knowledge",
            "1.0.0",
            "Engineering best practices and design patterns",
            "all-MiniLM-L6-v2",
        ),
        directory_structure: vec![
            ("manifest.yaml".to_string(), true),
            ("metadata.yaml".to_string(), true),
            ("documents/".to_string(), false),
            ("documents/architecture-patterns.md".to_string(), false),
            ("tests/".to_string(), false),
            ("tests/validation.test".to_string(), false),
            ("documentation/".to_string(), false),
            ("documentation/README.md".to_string(), false),
        ],
    }
}

/// Predefined template for documentation knowledge.
pub fn documentation_template() -> Template {
    Template {
        category: TemplateCategory::Documentation,
        manifest: Manifest {
            schema_version: "1.0".to_string(),
            name: "documentation-knowledge".to_string(),
            version: "1.0.0".to_string(),
            description:
                "Knowledge pack for project documentation, API references, and user guides"
                    .to_string(),
            author: "Wiki Labs Team".to_string(),
            license: "MIT".to_string(),
            format_version: "1.0".to_string(),
            documents: vec![DocumentManifest {
                id: "api-reference".to_string(),
                path: "documents/api-reference.md".to_string(),
                format: "markdown".to_string(),
                embed: true,
            }],
            dependencies: vec![],
        },
        metadata: Metadata::new(
            "documentation-knowledge",
            "1.0.0",
            "Project documentation and API references",
            "all-MiniLM-L6-v2",
        ),
        directory_structure: vec![
            ("manifest.yaml".to_string(), true),
            ("metadata.yaml".to_string(), true),
            ("documents/".to_string(), false),
            ("documents/api-reference.md".to_string(), false),
            ("tests/".to_string(), false),
            ("tests/validation.test".to_string(), false),
            ("documentation/".to_string(), false),
            ("documentation/README.md".to_string(), false),
        ],
    }
}

/// A complete template including manifest, metadata, and directory structure.
#[derive(Debug, Clone)]
pub struct Template {
    pub category: TemplateCategory,
    pub manifest: Manifest,
    pub metadata: Metadata,
    pub directory_structure: Vec<(String, bool)>,
}

impl Template {
    /// Returns the template by name.
    pub fn from_name(name: &str) -> Option<Template> {
        match name.to_lowercase().as_str() {
            "openshift" | "ocp" | "kubernetes" => Some(openshift_template()),
            "engineering" | "eng" => Some(engineering_template()),
            "documentation" | "docs" => Some(documentation_template()),
            _ => None,
        }
    }

    /// Returns all available template names.
    pub fn list_names() -> Vec<String> {
        vec![
            "openshift".to_string(),
            "engineering".to_string(),
            "documentation".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openshift_template() {
        let t = openshift_template();
        assert_eq!(t.category, TemplateCategory::Openshift);
        assert_eq!(t.manifest.name, "openshift-knowledge");
        assert_eq!(t.manifest.documents.len(), 2);
        assert_eq!(t.metadata.embedding_model, "all-MiniLM-L6-v2");
    }

    #[test]
    fn test_template_from_name_openshift() {
        let t = Template::from_name("openshift").unwrap();
        assert_eq!(t.manifest.name, "openshift-knowledge");
    }

    #[test]
    fn test_template_from_name_engineering() {
        let t = Template::from_name("engineering").unwrap();
        assert_eq!(t.manifest.name, "engineering-knowledge");
    }

    #[test]
    fn test_template_from_name_docs() {
        let t = Template::from_name("docs").unwrap();
        assert_eq!(t.manifest.name, "documentation-knowledge");
    }

    #[test]
    fn test_template_from_name_unknown() {
        assert!(Template::from_name("nonexistent").is_none());
    }

    #[test]
    fn test_list_template_names() {
        let names = Template::list_names();
        assert!(names.contains(&"openshift".to_string()));
        assert!(names.contains(&"engineering".to_string()));
        assert!(names.contains(&"documentation".to_string()));
    }

    #[test]
    fn test_template_manifest_validates() {
        let t = openshift_template();
        assert!(t.manifest.validate().is_ok());
        assert!(t.metadata.validate().is_ok());
    }

    #[test]
    fn test_display_category() {
        assert_eq!(format!("{}", TemplateCategory::Openshift), "openshift");
        assert_eq!(format!("{}", TemplateCategory::Engineering), "engineering");
        assert_eq!(
            format!("{}", TemplateCategory::Documentation),
            "documentation"
        );
    }
}
