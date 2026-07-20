//! Chunk generation step — split into chunks with overlap.

use crate::doc::KnowledgeChunk;
use crate::processing::Document;
use crate::processing::DocumentElement;
use chrono::Utc;
use tracing::debug;

/// The chunk generation pipeline step.
pub struct ChunkStep {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl ChunkStep {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap: chunk_overlap.min(chunk_size / 4), // Overlap at most 25% of chunk size
        }
    }

    /// Run the chunk generation step on a normalized document.
    pub fn run(
        &self,
        doc: &Document,
        workspace_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<KnowledgeChunk>> {
        if doc.full_text.is_empty() {
            debug!("Empty document, no chunks generated");
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();

        // First try to chunk by document structure (headings)
        let struct_chunks = self.chunk_by_structure(doc);
        if struct_chunks.len() >= 2 {
            // Use structural chunks if meaningful
            for (idx, content) in struct_chunks {
                chunks.push(self.create_chunk(idx, &content, &doc.source, workspace_id, true));
            }
        } else {
            // Fall back to text-based chunking with overlap
            let text_chunks = self.chunk_by_text(&doc.full_text);
            for (idx, content) in text_chunks {
                chunks.push(self.create_chunk(idx, &content, &doc.source, workspace_id, false));
            }
        }

        debug!(
            path = ?doc.source,
            chunk_count = chunks.len(),
            chunk_size = self.chunk_size,
            chunk_overlap = self.chunk_overlap,
            "Chunk generation complete"
        );

        Ok(chunks)
    }

    /// Chunk by document structure (headings).
    fn chunk_by_structure(&self, doc: &Document) -> Vec<(usize, String)> {
        let mut chunks: Vec<(usize, String)> = Vec::new();
        let mut current_content = String::new();
        let mut _current_level: Option<u32> = None;
        let mut idx = 0;

        for element in &doc.elements {
            match element {
                DocumentElement::Heading(level, text) => {
                    // Save previous chunk
                    if !current_content.is_empty() {
                        idx += 1;
                        chunks.push((idx, current_content.trim().to_string()));
                    }
                    // Start new chunk with heading
                    current_content = format!("# {}\n", text);
                    _current_level = Some(*level);
                }
                _ => {
                    let text = self.element_to_text(element);
                    if !text.is_empty() {
                        if !current_content.is_empty() && !current_content.ends_with('\n') {
                            current_content.push('\n');
                        }
                        current_content.push_str(&text);
                        current_content.push('\n');
                    }
                }
            }
        }

        // Save last chunk
        if !current_content.is_empty() {
            idx += 1;
            chunks.push((idx, current_content.trim().to_string()));
        }

        chunks
    }

    /// Chunk by text with configurable size and overlap.
    fn chunk_by_text(&self, text: &str) -> Vec<(usize, String)> {
        let mut chunks: Vec<(usize, String)> = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total = chars.len();

        if total == 0 {
            return chunks;
        }

        let mut start = 0;
        let mut idx = 0;

        while start < total {
            let end = (start + self.chunk_size).min(total);
            let chunk_text: String = chars[start..end].iter().collect();

            // Try to break at sentence boundary
            let end_pos = self.find_break_point(&chunk_text, end, total);

            let final_end = if end_pos > start + self.chunk_size / 2 {
                end_pos
            } else {
                end
            };

            let chunk_content: String = chars[start..final_end].iter().collect();
            idx += 1;
            chunks.push((idx, chunk_content.trim().to_string()));

            // Calculate next start with overlap
            start = final_end.saturating_sub(self.chunk_overlap);
            if start >= final_end {
                start = final_end; // Avoid infinite loop
            }
        }

        chunks
    }

    fn find_break_point(&self, text: &str, current_end: usize, total: usize) -> usize {
        // Look for sentence/paragraph boundaries after current_size
        let after_text = &text[current_end.min(text.len())..];
        let after: Vec<char> = after_text.chars().collect();
        for (i, c) in after.iter().enumerate() {
            if *c == '.' || *c == '!' || *c == '?' || *c == '\n' {
                return current_end + i + 1;
            }
            if i > 200 {
                break; // Don't search too far
            }
        }
        current_end
    }

    fn element_to_text(&self, element: &DocumentElement) -> String {
        match element {
            DocumentElement::Heading(level, text) => {
                format!("{} {}", "#".repeat(*level as usize), text)
            }
            DocumentElement::Paragraph(text) => text.clone(),
            DocumentElement::Table(rows) => rows
                .iter()
                .map(|row| row.join(" | "))
                .collect::<Vec<_>>()
                .join("\n"),
            DocumentElement::List(items) => items
                .iter()
                .map(|item| format!("- {}", item))
                .collect::<Vec<_>>()
                .join("\n"),
            DocumentElement::CodeBlock(lang, code) => {
                if lang.is_empty() {
                    code.clone()
                } else {
                    format!("```{}\n{}\n```", lang, code)
                }
            }
            DocumentElement::Command(text) => {
                format!("```\n{}\n```", text)
            }
            DocumentElement::Example(text) => {
                format!("> {}\n", text.trim())
            }
            DocumentElement::Warning(text) => {
                format!("⚠ {}\n", text.trim())
            }
            DocumentElement::Reference(text, url) => {
                format!("[{}]({})", text, url)
            }
            DocumentElement::InlineCode(text) => {
                format!("`{}`", text)
            }
            DocumentElement::Bold(text) => {
                format!("**{}**", text)
            }
        }
    }

    fn create_chunk(
        &self,
        idx: usize,
        content: &str,
        source: &str,
        workspace_id: uuid::Uuid,
        structured: bool,
    ) -> KnowledgeChunk {
        let id = uuid::Uuid::new_v4();
        let vector_id = format!(
            "{}_{}_{}",
            source.replace(|c: char| !c.is_alphanumeric(), "_"),
            idx,
            if structured { "s" } else { "t" }
        );
        let now = Utc::now();

        KnowledgeChunk {
            id,
            document_id: uuid::Uuid::new_v4(), // Will be set during index preparation
            content: content.to_string(),
            vector_id,
        }
    }
}

impl Default for ChunkStep {
    fn default() -> Self {
        Self::new(1000, 100)
    }
}
