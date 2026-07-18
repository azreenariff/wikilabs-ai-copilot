# Knowledge Validation Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Knowledge Validation Framework validates knowledge packs at multiple levels — from manifest schema to document integrity, embedding compatibility, and cross-reference resolution. Validation produces detailed reports with pass/fail status, errors, and warnings.

## Validation Architecture

```
Knowledge Pack
    ↓
[Manifest Validator] ──→ manifest.yaml schema compliance
    ↓
[Metadata Validator] ──→ metadata.yaml schema compliance
    ↓
[Document Validator] ──→ Documents exist, readable, valid format
    ↓
[Embedding Validator] ──→ Embedding compatibility
    ↓
[Schema Version Validator] ──→ Schema version support
    ↓
[Duplicate ID Validator] ──→ No duplicate identifiers
    ↓
[Dependency Validator] ──→ Dependencies resolved
    ↓
[Broken Ref Validator] ──→ All references resolve
    ↓
[Version Compat Validator] ──→ Semantic version valid
    ↓
Validation Report
```

## Validation Orchestration

```rust
pub struct PackValidator {
    pub manifest_validator: ManifestValidator,
    pub metadata_validator: MetadataValidator,
    pub document_validator: DocumentValidator,
    pub embedding_validator: EmbeddingValidator,
    pub schema_validator: SchemaVersionValidator,
    pub duplicate_validator: DuplicateIdValidator,
    pub dependency_validator: DependencyValidator,
    pub broken_ref_validator: BrokenRefValidator,
    pub version_validator: VersionCompatValidator,
    pub config: ValidationConfig,
}

pub struct ValidationConfig {
    pub strict_mode: bool,              // Treat warnings as errors
    pub check_dependencies: bool,
    pub check_broken_refs: bool,
    pub check_embedding_compat: bool,
    pub check_versions: bool,
    pub parallel: bool,
    pub timeout: Duration,
}
```

## Manifest Validation

Validates manifest.yaml against the Knowledge Pack schema.

```rust
pub struct ManifestValidator;

impl ManifestValidator {
    pub fn validate(&self, manifest: &KnowledgePackManifest) -> ValidationReport {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Schema version must be present and valid
        if manifest.schema_version.is_empty() {
            errors.push(ValidationError {
                field: "schema_version",
                message: "schema_version is required".to_string(),
                severity: ErrorSeverity::Error,
            });
        } else if !self.supported_versions.contains(&manifest.schema_version) {
            errors.push(ValidationError {
                field: "schema_version",
                message: format!("Unsupported schema version: {}", manifest.schema_version),
                severity: ErrorSeverity::Error,
            });
        }

        // Name must be present and valid
        if manifest.name.is_empty() {
            errors.push(ValidationError {
                field: "name",
                message: "name is required".to_string(),
                severity: ErrorSeverity::Error,
            });
        } else if !self.validate_name(&manifest.name) {
            errors.push(ValidationError {
                field: "name",
                message: "name must be 3-64 chars, lowercase, hyphen-separated, alphanumeric only".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        // Version must be valid semver
        if let Err(e) = manifest.version.parse::<semver::Version>() {
            errors.push(ValidationError {
                field: "version",
                message: format!("Invalid semantic version: {}", e),
                severity: ErrorSeverity::Error,
            });
        }

        // Description is required
        if manifest.description.is_empty() {
            warnings.push(ValidationError {
                field: "description",
                message: "description is empty (recommended)".to_string(),
                severity: ErrorSeverity::Warning,
            });
        }

        // Vendor name is required
        if manifest.vendor.name.is_empty() {
            errors.push(ValidationError {
                field: "vendor.name",
                message: "vendor.name is required".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        // Technology must not be empty
        if manifest.technologies.is_empty() {
            warnings.push(ValidationError {
                field: "technology",
                message: "technology array is empty (recommended)".to_string(),
                severity: ErrorSeverity::Warning,
            });
        }

        // Priority must be in range
        if manifest.priority < 0 || manifest.priority > 1000 {
            errors.push(ValidationError {
                field: "priority",
                message: "priority must be between 0 and 1000".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        // Dependencies must reference valid pack names
        for dep in &manifest.dependencies {
            if dep.name.is_empty() {
                errors.push(ValidationError {
                    field: "dependencies.name",
                    message: "dependency name is required".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }
            if dep.version.is_empty() {
                errors.push(ValidationError {
                    field: "dependencies.version",
                    message: "dependency version constraint is required".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            checks_performed: 10,
            checks_passed: if errors.is_empty() { 10 } else { 10 - errors.len() },
            check_duration: Duration::from_millis(5),
        }
    }

    fn validate_name(&self, name: &str) -> bool {
        if name.len() < 3 || name.len() > 64 { return false; }
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    }
}

pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}
```

## Metadata Validation

Validates metadata.yaml against the metadata schema.

```rust
pub struct MetadataValidator;

impl MetadataValidator {
    pub fn validate(&self, metadata: &KnowledgePackMetadata) -> ValidationReport {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Validate last_indexed timestamp
        if let Some(ref ts) = metadata.last_indexed {
            if ts > Utc::now() {
                errors.push(ValidationError {
                    field: "last_indexed",
                    message: "last_indexed cannot be in the future".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        // Validate encoding
        if metadata.encoding != "UTF-8" {
            warnings.push(ValidationError {
                field: "encoding",
                message: format!("Non-UTF-8 encoding: {} (UTF-8 recommended)", metadata.encoding),
                severity: ErrorSeverity::Warning,
            });
        }

        // Validate documents list
        for doc in &metadata.documents {
            if doc.id.is_empty() {
                errors.push(ValidationError {
                    field: "documents.id",
                    message: "document id is required".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }
            if doc.title.is_empty() {
                errors.push(ValidationError {
                    field: "documents.title",
                    message: "document title is required".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }
            if doc.source.is_empty() {
                errors.push(ValidationError {
                    field: "documents.source",
                    message: "document source is required".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }
            if doc.format.is_empty() {
                errors.push(ValidationError {
                    field: "documents.format",
                    message: "document format is required".to_string(),
                    severity: ErrorSeverity::Error,
                });
            }
            if !self.supported_formats.contains(&doc.format) {
                warnings.push(ValidationError {
                    field: "documents.format",
                    message: format!("Unsupported format: {} (supported: {:?})", doc.format, self.supported_formats),
                    severity: ErrorSeverity::Warning,
                });
            }
        }

        // Validate embedding config
        if metadata.embedding.provider.is_empty() {
            errors.push(ValidationError {
                field: "embedding.provider",
                message: "embedding provider is required".to_string(),
                severity: ErrorSeverity::Error,
            });
        }
        if metadata.embedding.dimension == 0 {
            errors.push(ValidationError {
                field: "embedding.dimension",
                message: "embedding dimension must be greater than 0".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        // Validate index config
        if metadata.index.namespace.is_empty() {
            errors.push(ValidationError {
                field: "index.namespace",
                message: "index namespace is required".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            checks_performed: 12,
            checks_passed: 12 - errors.len(),
            check_duration: Duration::from_millis(3),
        }
    }
}
```

## Document Validation

Validates all documents listed in metadata.yaml exist and are readable.

```rust
pub struct DocumentValidator;

impl DocumentValidator {
    pub async fn validate(&self, metadata: &KnowledgePackMetadata, base_path: &Path) -> ValidationReport {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut results = vec![];

        for doc in &metadata.documents {
            let doc_path = base_path.join(&doc.source);
            
            // Check file exists
            if !doc_path.exists() {
                errors.push(ValidationError {
                    field: "documents",
                    message: format!("Document not found: {}", doc.source),
                    severity: ErrorSeverity::Error,
                });
                continue;
            }

            // Check file is readable
            match fs::metadata(&doc_path) {
                Ok(meta) => {
                    results.push(DocumentValidationResult {
                        source: doc.source.clone(),
                        exists: true,
                        readable: true,
                        size: meta.len(),
                        format: doc.format.clone(),
                        valid_format: self.validate_format(&doc.format, &doc_path).await,
                    });

                    // Check file size
                    if meta.len() > self.max_file_size {
                        warnings.push(ValidationError {
                            field: "documents",
                            message: format!("Document too large: {} bytes (max: {})", meta.len(), self.max_file_size),
                            severity: ErrorSeverity::Warning,
                        });
                    }
                }
                Err(_) => {
                    errors.push(ValidationError {
                        field: "documents",
                        message: format!("Document not readable: {}", doc.source),
                        severity: ErrorSeverity::Error,
                    });
                }
            }
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            checks_performed: metadata.documents.len(),
            checks_passed: metadata.documents.len() - errors.len(),
            check_duration: Duration::from_millis(10),
            details: Some(DocumentValidationDetails { results, total_found: metadata.documents.len(), total_missing: errors.len() }),
        }
    }

    async fn validate_format(&self, format: &str, path: &Path) -> bool {
        // Validate that file content matches declared format
        match format {
            "markdown" => self.is_markdown(path).await,
            "html" => self.is_html(path).await,
            "txt" => self.is_text(path).await,
            "yaml" => self.is_yaml(path).await,
            "json" => self.is_json(path).await,
            "xml" => self.is_xml(path).await,
            "pdf" => self.is_pdf(path).await,
            "docx" => self.is_docx(path).await,
            _ => true,
        }
    }
}
```

## Embedding Validation

Validates embedding compatibility.

```rust
pub struct EmbeddingValidator;

impl EmbeddingValidator {
    pub fn validate(&self, metadata: &KnowledgePackMetadata) -> ValidationReport {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Check embedding provider is available
        if !self.available_providers.contains(&metadata.embedding.provider) {
            errors.push(ValidationError {
                field: "embedding.provider",
                message: format!("Unavailable embedding provider: {}", metadata.embedding.provider),
                severity: ErrorSeverity::Error,
            });
        }

        // Check dimension matches provider capability
        let provider_dim = self.provider_dimensions.get(&metadata.embedding.provider).unwrap_or(&384);
        if metadata.embedding.dimension != *provider_dim {
            warnings.push(ValidationError {
                field: "embedding.dimension",
                message: format!("Dimension {} may not match provider {} expected dimension {}", metadata.embedding.dimension, metadata.embedding.provider, provider_dim),
                severity: ErrorSeverity::Warning,
            });
        }

        // Check embedding model is valid
        if metadata.embedding.model.is_empty() {
            errors.push(ValidationError {
                field: "embedding.model",
                message: "embedding model is required".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        // Check embeddable documents
        let non_embeddable = metadata.documents.iter().filter(|d| !d.embeddable).count();
        if metadata.embedding.enabled && non_embeddable > 0 {
            warnings.push(ValidationError {
                field: "documents",
                message: format!("{} documents marked as non-embeddable but embedding is enabled", non_embeddable),
                severity: ErrorSeverity::Warning,
            });
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            checks_performed: 5,
            checks_passed: 5 - errors.len(),
            check_duration: Duration::from_millis(2),
        }
    }
}
```

## Schema Version Validation

Validates manifest schema version is supported.

```rust
pub struct SchemaVersionValidator;

impl SchemaVersionValidator {
    pub fn validate(&self, manifest: &KnowledgePackManifest) -> ValidationReport {
        let mut errors = vec![];

        if !self.supported_versions.contains(&manifest.schema_version) {
            errors.push(ValidationError {
                field: "schema_version",
                message: format!("Unsupported schema version: {} (supported: {:?})", manifest.schema_version, self.supported_versions),
                severity: ErrorSeverity::Error,
            });
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings: vec![],
            checks_performed: 1,
            checks_passed: if errors.is_empty() { 1 } else { 0 },
            check_duration: Duration::from_millis(1),
        }
    }
}
```

## Duplicate ID Validation

Checks for duplicate document identifiers within a pack.

```rust
pub struct DuplicateIdValidator;

impl DuplicateIdValidator {
    pub fn validate(&self, metadata: &KnowledgePackMetadata) -> ValidationReport {
        let mut errors = vec![];
        let mut seen_ids = HashSet::new();
        let mut duplicates = vec![];

        for doc in &metadata.documents {
            if !seen_ids.insert(&doc.id) {
                duplicates.push(doc.id.clone());
            }
        }

        if !duplicates.is_empty() {
            errors.push(ValidationError {
                field: "documents.id",
                message: format!("Duplicate document IDs: {}", duplicates.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                severity: ErrorSeverity::Error,
            });
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings: vec![],
            checks_performed: 1,
            checks_passed: if errors.is_empty() { 1 } else { 0 },
            check_duration: Duration::from_millis(1),
        }
    }
}
```

## Dependency Validation

Validates pack dependencies.

```rust
pub struct DependencyValidator;

impl DependencyValidator {
    pub async fn validate(&self, manifest: &KnowledgePackManifest) -> ValidationReport {
        let mut errors = vec![];

        for dep in &manifest.dependencies {
            // Check dependency is installed
            if !self.installed_packs.contains_key(&dep.name) {
                errors.push(ValidationError {
                    field: "dependencies",
                    message: format!("Dependency '{}' not installed", dep.name),
                    severity: ErrorSeverity::Error,
                });
                continue;
            }

            // Check version constraint
            let dep_version = self.installed_packs.get(&dep.name).unwrap();
            if !self.version_matches(dep_version, &dep.version) {
                errors.push(ValidationError {
                    field: "dependencies",
                    message: format!("Dependency '{}' version {} does not satisfy '{}'", dep.name, dep_version, dep.version),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        // Check for circular dependencies
        if self.has_circular_dependency(&manifest.name, &manifest.dependencies) {
            errors.push(ValidationError {
                field: "dependencies",
                message: "Circular dependency detected".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings: vec![],
            checks_performed: 2 * manifest.dependencies.len() + 1,
            checks_passed: 2 * manifest.dependencies.len() + 1 - errors.len(),
            check_duration: Duration::from_millis(5),
        }
    }
}
```

## Broken Reference Validation

Checks for broken internal references within the pack.

```rust
pub struct BrokenRefValidator;

impl BrokenRefValidator {
    pub async fn validate(&self, metadata: &KnowledgePackMetadata, base_path: &Path) -> ValidationReport {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Check all document sources resolve to files
        for doc in &metadata.documents {
            let doc_path = base_path.join(&doc.source);
            if !doc_path.exists() {
                errors.push(ValidationError {
                    field: "documents",
                    message: format!("Broken reference: {} not found", doc.source),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        // Check README.md exists
        let readme_path = base_path.join("documentation/README.md");
        if !readme_path.exists() {
            warnings.push(ValidationError {
                field: "documentation",
                message: "documentation/README.md not found".to_string(),
                severity: ErrorSeverity::Warning,
            });
        }

        // Check tests directory exists
        let tests_path = base_path.join("tests/");
        if !tests_path.exists() {
            warnings.push(ValidationError {
                field: "tests",
                message: "tests/ directory not found".to_string(),
                severity: ErrorSeverity::Warning,
            });
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            checks_performed: metadata.documents.len() + 2,
            checks_passed: metadata.documents.len() + 2 - errors.len(),
            check_duration: Duration::from_millis(3),
        }
    }
}
```

## Version Compatibility Validation

Validates semantic versions and version constraints.

```rust
pub struct VersionCompatValidator;

impl VersionCompatValidator {
    pub fn validate(&self, manifest: &KnowledgePackManifest) -> ValidationReport {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Validate pack version is valid semver
        if let Err(e) = manifest.version.parse::<semver::Version>() {
            errors.push(ValidationError {
                field: "version",
                message: format!("Invalid semantic version: {}", e),
                severity: ErrorSeverity::Error,
            });
        }

        // Validate min_sdk_version is valid semver
        if let Err(e) = manifest.min_sdk_version.parse::<semver::Version>() {
            errors.push(ValidationError {
                field: "min_sdk_version",
                message: format!("Invalid semantic version: {}", e),
                severity: ErrorSeverity::Error,
            });
        }

        // Validate dependency versions are valid semver constraints
        for dep in &manifest.dependencies {
            if let Err(e) = semver::VersionReq::parse(&dep.version) {
                errors.push(ValidationError {
                    field: "dependencies",
                    message: format!("Invalid version constraint for '{}': {}", dep.name, e),
                    severity: ErrorSeverity::Error,
                });
            }
        }

        // Check for prerelease version
        if manifest.version.pre.is_empty() && manifest.version.build.is_empty() {
            warnings.push(ValidationError {
                field: "version",
                message: "Consider using prerelease versions during pack development".to_string(),
                severity: ErrorSeverity::Warning,
            });
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            checks_performed: 3 + manifest.dependencies.len(),
            checks_passed: 3 + manifest.dependencies.len() - errors.len(),
            check_duration: Duration::from_millis(2),
        }
    }
}
```

## Validation Report

```rust
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub checks_performed: usize,
    pub checks_passed: usize,
    pub check_duration: Duration,
    pub details: Option<ValidationDetails>,
}

pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ErrorSeverity,
    pub suggestion: Option<String>,
}

pub enum ValidationDetails {
    Documents(DocumentValidationDetails),
    Embedding(EmbeddingValidationDetails),
    Generic(HashMap<String, String>),
}

pub struct DocumentValidationDetails {
    pub results: Vec<DocumentValidationResult>,
    pub total_found: usize,
    pub total_missing: usize,
}

pub struct DocumentValidationResult {
    pub source: String,
    pub exists: bool,
    pub readable: bool,
    pub size: u64,
    pub format: String,
    pub valid_format: bool,
}
```

## Validation Summary

The overall validation aggregates all individual reports:

```rust
pub struct OverallValidationReport {
    pub valid: bool,
    pub overall_status: ValidationStatus,
    pub manifests: ValidationReport,
    pub metadata: ValidationReport,
    pub documents: ValidationReport,
    pub embedding: ValidationReport,
    pub schema_version: ValidationReport,
    pub duplicate_id: ValidationReport,
    pub dependencies: ValidationReport,
    pub broken_refs: ValidationReport,
    pub version_compat: ValidationReport,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub total_checks: usize,
    pub total_passed: usize,
    pub total_duration: Duration,
    pub timestamp: DateTime<Utc>,
    pub pack_name: String,
    pub pack_version: String,
}

pub enum ValidationStatus {
    Valid,
    Invalid,
    Warning,
    NotValidated,
}
```

## Validation Output Formats

### Text Output

```
Knowledge Pack Validation Report
================================
Pack: openshift v1.2.0
Validated: 2025-01-15 10:30:00 UTC

✓ manifest.yaml — Valid (5 checks passed)
✓ metadata.yaml — Valid (12 checks passed)
✓ 245 documents — All exist and readable (245 checks passed)
✓ embedding compatibility — Compatible (5 checks passed)
✓ schema version — 1.0.0 (1 check passed)
✓ identifiers — No duplicates (1 check passed)
✓ dependencies — All resolved (7 checks passed)
✓ references — No broken references (247 checks passed)
✓ versions — All valid semver (5 checks passed)

Result: 531 checks passed, 0 failed
Warnings: 2
Duration: 120ms
```

### JSON Output

```json
{
  "valid": true,
  "pack_name": "openshift",
  "pack_version": "1.2.0",
  "validated_at": "2025-01-15T10:30:00Z",
  "checks_performed": 531,
  "checks_passed": 531,
  "errors": [],
  "warnings": [
    {
      "field": "description",
      "message": "description is empty (recommended)",
      "severity": "warning"
    }
  ],
  "duration_ms": 120
}
```

## Integration with Knowledge Platform

Validation results are stored in:

1. **metadata.yaml** — `validation` section updated after each validation
2. **Validation report** — saved to `validation-reports/` directory
3. **Pack repository** — overall validation status tracked per pack

```rust
impl PackValidator {
    pub async fn validate_and_update(&self, pack_path: &Path) -> anyhow::Result<OverallValidationReport> {
        let report = self.validate(pack_path).await?;
        
        // Update metadata.yaml with validation status
        let mut metadata = self.load_metadata(pack_path)?;
        metadata.validation = ValidationStatus {
            status: if report.valid { "pass" } else { "fail" }.to_string(),
            last_validated: Utc::now(),
            errors: report.errors.iter().map(|e| e.message.clone()).collect(),
            warnings: report.warnings.iter().map(|e| e.message.clone()).collect(),
        };
        self.save_metadata(pack_path, &metadata)?;

        // Save validation report
        self.save_report(pack_path, &report)?;

        Ok(report)
    }
}
```

## Best Practices

1. **Validate early and often** — validate after every change
2. **Include validation in CI/CD** — reject invalid packs
3. **Use strict mode in production** — treat warnings as errors
4. **Maintain validation reports** — track validation history
5. **Test edge cases** — large files, special characters, empty packs
6. **Parallel validation** — validate independent checks in parallel
7. **Timeout validation** — don't let validation hang