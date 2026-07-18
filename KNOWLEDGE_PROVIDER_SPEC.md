# Knowledge Provider Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Knowledge Provider Framework provides an abstraction layer for sourcing documents into the Enterprise Knowledge Platform. Providers discover, parse, and normalize documents from various sources into a uniform `ProviderDocument` format.

## Design Principles

1. **Provider Independence** — The Knowledge Platform never depends on a single provider
2. **Extensibility** — New providers can be added without modifying existing code
3. **Uniform Interface** — All providers expose the same API contract
4. **Format Awareness** — Each provider knows its supported file formats
5. **Structured Output** — All providers output `DocumentElement` tree structure

## Provider Trait

All providers implement the `KnowledgeProvider` trait:

```rust
#[async_trait]
pub trait KnowledgeProvider: Send + Sync {
    /// Unique identifier for this provider.
    fn name(&self) -> &str;

    /// File formats this provider can parse.
    fn supported_formats(&self) -> &[&str];

    /// Discover documents in the given path.
    async fn discover(&self, path: &str) -> anyhow::Result<Vec<ProviderDocument>>;

    /// Parse a single document.
    async fn parse(&self, path: &str) -> anyhow::Result<ProviderDocument>;

    /// Get enabled status.
    fn get_enabled(&self, enabled: bool) -> bool;
}
```

## Provider Registry

Providers are registered in a `ProviderRegistry` that:

- Maintains a list of available providers
- Maps file extensions to provider
- Selects the best provider for a given file
- Supports dynamic provider registration/unregistration

```rust
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn KnowledgeProvider>>,
}
```

## Built-in Providers

### Filesystem Provider

- Scans directories for files
- Recursively discovers documents
- Filters by file extension
- Returns file metadata (size, modified time, permissions)

Supported formats: all formats

### Markdown Provider

- Parses Markdown files
- Preserves heading hierarchy
- Extracts tables, lists, code blocks
- Tracks YAML frontmatter as metadata

Supported formats: `.md`, `.markdown`

### HTML Provider

- Parses HTML files
- Extracts text content
- Preserves heading structure (h1-h6)
- Extracts tables
- Extracts code blocks
- Preserves links and references

Supported formats: `.html`, `.htm`

### PDF Provider

- Extracts text from PDF files
- Preserves heading structure when available
- Extracts tables when detectable
- Preserves metadata (title, author, creation date)

Supported formats: `.pdf`

### DOCX Provider

- Parses DOCX files
- Extracts heading hierarchy
- Extracts tables
- Extracts lists
- Preserves code blocks
- Preserves comments

Supported formats: `.docx`

### TXT Provider

- Plain text parsing
- Paragraph detection via blank lines
- Indentation detection for lists/code

Supported formats: `.txt`

### YAML Provider

- Parses YAML files
- Extracts frontmatter metadata
- Preserves key-value structure
- Detects engineering commands

Supported formats: `.yaml`, `.yml`

### JSON Provider

- Parses JSON files
- Extracts nested structure
- Detects engineering data structures
- Preserves formatting information

Supported formats: `.json`

### XML Provider

- Parses XML files
- Preserves element hierarchy
- Extracts attributes as metadata
- Preserves text content

Supported formats: `.xml`, `.xsd`

### Git Repository Provider

- Scans git repositories
- Tracks file history
- Uses git blame for authorship
- Supports branch/tag selection
- Incremental sync via commit hash

Supported formats: all formats (via sub-providers)

## Stub Providers (Future)

### Confluence Provider

- Fetches pages from Confluence API
- Supports space filtering
- Preserves page hierarchy
- Extracts attachments

### SharePoint Provider

- Fetches documents from SharePoint
- Supports folder navigation
- Preserves metadata
- Handles authentication

### REST API Provider

- Fetches documents from REST endpoints
- Configurable endpoints
- Supports pagination
- Extracts metadata from headers

### Cloud Storage Provider

- Supports S3, Azure Blob, GCS
- Bucket/container browsing
- Preserves metadata
- Supports pre-signed URLs

## Document Element Structure

All providers output a `DocumentElement` tree:

```rust
pub enum DocumentElement {
    Heading { level: u8, text: String },
    Paragraph(String),
    Table(Vec<Vec<String>>),
    List(Vec<String>),
    CodeBlock { language: Option<String>, code: String },
    Command(String),
    Example(String),
    Warning(String),
    Reference { text: String, url: String },
    Image { alt: String, url: String },
    Section { title: String, children: Vec<DocumentElement> },
    Raw(String),
}
```

## Provider Metadata

Each provider outputs a `ProviderDocument`:

```rust
pub struct ProviderDocument {
    pub id: Uuid,
    pub title: String,
    pub source: String,
    pub provider_name: String,
    pub format: String,
    pub elements: Vec<DocumentElement>,
    pub metadata: HashMap<String, String>,
    pub word_count: usize,
    pub char_count: usize,
    pub language: String,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}
```

## Provider Selection

Provider selection is based on:

1. File extension lookup in provider registry
2. Content sniffing (heuristic format detection)
3. MIME type detection
4. Priority ranking for ambiguous cases

## Error Handling

Providers return typed errors:

```rust
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Parse error at line {line}: {reason}")]
    ParseError { line: usize, reason: String },

    #[error("Encoding error: {reason}")]
    EncodingError { reason: String },

    #[error("Provider error: {reason}")]
    ProviderError { provider: String, reason: String },

    #[error("Authentication failed for {provider}")]
    AuthenticationFailed { provider: String },

    #[error("Rate limit exceeded for {provider}")]
    RateLimitExceeded { provider: String },
}
```

## Provider Configuration

Providers are configured via `ProviderConfig`:

```rust
pub struct ProviderConfig {
    pub max_file_size: usize,           // Maximum file size in bytes
    pub timeout: Duration,               // Network timeout
    pub retry_count: usize,              // Retry count for failed requests
    pub enable_incremental: bool,        // Enable incremental fetching
    pub cache_enabled: bool,             // Enable response caching
    pub custom_headers: HashMap<String, String>, // HTTP headers
}
```

## Testing

Each provider must implement:

- Unit tests for parsing various formats
- Integration tests with real documents
- Error handling tests
- Edge case tests (empty files, malformed files, large files)
- Performance benchmarks

## Provider Factory

A factory pattern creates providers:

```rust
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(provider_type: ProviderType, config: ProviderConfig) -> anyhow::Result<Box<dyn KnowledgeProvider>>;
    pub fn all_providers() -> Vec<Box<dyn KnowledgeProvider>>;
}
```

## Future Extensions

- Streaming parsing for large files
- Parallel parsing for directories
- Provider-specific chunking strategies
- Provider-level deduplication
- Provider authentication management
- Provider plugin system