//! Knowledge chunking — split documents into embedding-sized chunks.

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
/// Strategy for chunking documents.
#[derive(Debug, Clone)]
pub enum ChunkStrategy {
    /// Split by document structure (headings, sections).
    ByStructure {
        max_chunk_size: usize,
        min_chunk_size: usize,
        overlap: usize,
    },
    /// Split by fixed token size with overlap.
    BySize {
        max_chunk_size: usize,
        overlap: usize,
    },
    /// Split by paragraphs.
    ByParagraph {
        max_chunk_size: usize,
        overlap: usize,
    },
}

impl Default for ChunkStrategy {
    fn default() -> Self {
        Self::ByStructure {
            max_chunk_size: 512,
            min_chunk_size: 64,
            overlap: 32,
        }
    }
}

/// Chunker — splits documents into embedding-sized chunks.
pub struct Chunker {
    strategy: ChunkStrategy,
}

impl Chunker {
    pub fn new(strategy: ChunkStrategy) -> Self {
        Self { strategy }
    }

    pub fn by_structure(max_size: usize, min_size: usize, overlap: usize) -> Self {
        Self {
            strategy: ChunkStrategy::ByStructure {
                max_chunk_size: max_size,
                min_chunk_size: min_size,
                overlap,
            },
        }
    }

    pub fn by_size(max_size: usize, overlap: usize) -> Self {
        Self {
            strategy: ChunkStrategy::BySize {
                max_chunk_size: max_size,
                overlap,
            },
        }
    }

    pub fn by_paragraph(max_size: usize, overlap: usize) -> Self {
        Self {
            strategy: ChunkStrategy::ByParagraph {
                max_chunk_size: max_size,
                overlap,
            },
        }
    }

    /// Chunk a document's text content.
    pub fn chunk(&self, text: &str, metadata: serde_json::Value) -> Result<Vec<KnowledgeChunk>> {
        let chunks = match &self.strategy {
            ChunkStrategy::ByStructure {
                max_chunk_size,
                min_chunk_size,
                overlap,
            } => self.chunk_by_structure(text, *max_chunk_size, *min_chunk_size, *overlap),
            ChunkStrategy::BySize {
                max_chunk_size,
                overlap,
            } => self.chunk_by_size(text, *max_chunk_size, *overlap),
            ChunkStrategy::ByParagraph {
                max_chunk_size,
                overlap,
            } => self.chunk_by_paragraph(text, *max_chunk_size, *overlap),
        };

        // Assign IDs and timestamps
        chunks
            .into_iter()
            .map(|chunk_text| {
                let id = Uuid::new_v4().to_string();
                Ok(KnowledgeChunk::new(id, &chunk_text, metadata.clone()))
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Chunk by document structure (headings, sections).
    fn chunk_by_structure(
        &self,
        text: &str,
        max_size: usize,
        min_size: usize,
        overlap: usize,
    ) -> Vec<String> {
        let mut chunks = Vec::new();
        let current = text.lines().collect::<Vec<_>>();
        let mut i = 0;

        while i < current.len() {
            // Find heading
            let mut _heading_line = i;
            if let Some(heading_idx) = current.iter().position(|l| l.starts_with('#')) {
                _heading_line = heading_idx;
                i = heading_idx;
            }

            // Collect section content
            let mut section_lines = Vec::new();
            let mut heading_context = String::new();

            // Collect heading context
            for line in current[i..].iter().take_while(|l| l.starts_with('#')) {
                heading_context.push_str(line);
                heading_context.push(' ');
            }

            // Collect section content until next heading or end
            let mut j = i;
            while j < current.len() {
                if current[j].starts_with('#') && j > i {
                    break;
                }
                section_lines.push(current[j]);
                j += 1;
            }

            let section_text = section_lines.join("\n");
            let text_len = section_text.len();

            if text_len <= max_size {
                if text_len >= min_size {
                    chunks.push(section_text);
                }
                i = j;
            } else {
                // Split large section into smaller chunks
                let lines: Vec<&str> = section_text.lines().collect();
                let mut start = 0;
                while start < lines.len() {
                    let end = std::cmp::min(start + lines.len() / 2, lines.len());
                    let chunk_text = lines[start..end].join("\n");
                    if chunk_text.len() >= min_size {
                        chunks.push(chunk_text);
                    }
                    start = end.saturating_sub(overlap);
                    if start >= lines.len() {
                        break;
                    }
                }
                i = j;
            }
        }

        chunks
    }

    /// Chunk by fixed size with overlap.
    fn chunk_by_size(&self, text: &str, max_size: usize, overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;

        while start < chars.len() {
            let end = std::cmp::min(start + max_size, chars.len());
            // Try to break at word boundary
            let mut break_point = end;
            while break_point > start && chars[break_point] != ' ' && chars[break_point] != '\n' {
                if break_point == 0 {
                    break;
                }
                break_point -= 1;
            }
            if break_point == start {
                break_point = end;
            }
            chunks.push(chars[..break_point].iter().collect());
            start = break_point.saturating_sub(overlap);
            if start >= chars.len() {
                break;
            }
        }

        chunks
    }

    /// Chunk by paragraphs with max size.
    fn chunk_by_paragraph(&self, text: &str, max_size: usize, overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut current_chunk = String::new();

        for paragraph in paragraphs {
            if paragraph.is_empty() {
                continue;
            }

            if current_chunk.is_empty() {
                current_chunk = paragraph.to_string();
            } else if current_chunk.len() + paragraph.len() + 2 <= max_size {
                current_chunk.push_str("\n\n");
                current_chunk.push_str(paragraph);
            } else {
                chunks.push(current_chunk.clone());
                current_chunk = if current_chunk.len() > overlap {
                    let chars: Vec<char> = current_chunk.chars().collect();
                    chars[chars.len() - overlap..].iter().collect()
                } else {
                    String::new()
                };
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(paragraph);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

/// A single knowledge chunk for embedding.
#[derive(Debug, Clone)]
pub struct KnowledgeChunk {
    pub id: String,
    pub text: String,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub heading_context: Option<String>,
    pub section: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl KnowledgeChunk {
    pub fn new(id: String, text: &str, metadata: serde_json::Value) -> Self {
        Self {
            id,
            text: text.to_string(),
            metadata,
            embedding: None,
            heading_context: None,
            section: None,
            created_at: Utc::now(),
        }
    }
}
