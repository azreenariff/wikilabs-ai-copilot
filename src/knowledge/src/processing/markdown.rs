//! Markdown parser preserving structure.
//!
//! Parses headings, paragraphs, lists, code blocks, tables,
//! commands, examples, warnings, and references.

use super::Document;
use super::{DocumentElement, ParserProvider};
use once_cell::sync::Lazy;
use regex::Regex;

/// Markdown parser that preserves document structure.
pub struct MarkdownParser;

static HEADING_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(#{1,6})\s+(.*)$").unwrap());
static CODE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^```(\w*)\s*\n(.*?)```$").unwrap());
static LIST_ITEM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*[-*+]\s+(.*)$").unwrap());
static ORDERED_LIST_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\d+\.\s+(.*)$").unwrap());
#[allow(dead_code)]
static BOLD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\*\*(.*?)\*\*").unwrap());
#[allow(dead_code)]
static INLINE_CODE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`([^`]+)`").unwrap());
#[allow(dead_code)]
static REFERENCE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
static WARNING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(>|\*)\s*(?:⚠|WARNING|WARN|⛔)\s*(.*)$").unwrap());
#[allow(dead_code)]
static COMMAND_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^```\s*\n([^\n].*)$").unwrap());
static EXAMPLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^>\s+(.*)$").unwrap());
static HORIZONTAL_RULE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(-{3,}|_{3,}|\*{3,})$").unwrap());
static TABLE_HEADER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\|(.+)\|").unwrap());

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
}

impl ParserProvider for MarkdownParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let mut elements = Vec::new();
        let mut i = 0;
        let lines: Vec<&str> = content.lines().collect();

        while i < lines.len() {
            let line = lines[i];

            // Code block
            if line.starts_with("```") {
                if let Some(code_block) = self.parse_code_block(&lines, &mut i) {
                    elements.push(code_block);
                    continue;
                }
            }

            // Heading
            if let Some(cap) = HEADING_RE.captures(line) {
                let level = cap[1].len() as u32;
                let text = cap[2].trim().to_string();
                elements.push(DocumentElement::Heading(level, text));
                i += 1;
                continue;
            }

            // Horizontal rule (skip)
            if HORIZONTAL_RULE_RE.is_match(line) {
                i += 1;
                continue;
            }

            // Warning
            if let Some(cap) = WARNING_RE.captures(line) {
                elements.push(DocumentElement::Warning(cap[1].trim().to_string()));
                i += 1;
                continue;
            }

            // Example
            if let Some(cap) = EXAMPLE_RE.captures(line) {
                elements.push(DocumentElement::Example(cap[1].trim().to_string()));
                i += 1;
                continue;
            }

            // Table
            if line.starts_with('|') && line.contains('|') {
                if let Some(table) = self.parse_table(&lines, &mut i) {
                    elements.push(DocumentElement::Table(table));
                    continue;
                }
            }

            // List item
            if LIST_ITEM_RE.is_match(line) || ORDERED_LIST_RE.is_match(line) {
                let mut list_items = Vec::new();
                while i < lines.len()
                    && (LIST_ITEM_RE.is_match(lines[i]) || ORDERED_LIST_RE.is_match(lines[i]))
                {
                    if let Some(cap) = LIST_ITEM_RE.captures(lines[i]) {
                        list_items.push(cap[1].trim().to_string());
                    } else if let Some(cap) = ORDERED_LIST_RE.captures(lines[i]) {
                        list_items.push(cap[1].trim().to_string());
                    }
                    i += 1;
                }
                if !list_items.is_empty() {
                    elements.push(DocumentElement::List(list_items));
                }
                continue;
            }

            // Command (bash-like)
            if line.starts_with("$ ") || line.starts_with("# ") || line.starts_with("% ") {
                let cmd_line = if line.starts_with("$ ") || line.starts_with("# ") {
                    &line[2..]
                } else {
                    &line[1..]
                };
                // Collect command until blank line or another command
                let mut cmd_lines = vec![cmd_line.trim()];
                i += 1;
                while i < lines.len()
                    && !lines[i].is_empty()
                    && !HEADING_RE.is_match(lines[i])
                    && !lines[i].starts_with("$ ")
                    && !lines[i].starts_with("# ")
                {
                    if lines[i].starts_with('$') || lines[i].starts_with('#') {
                        cmd_lines.push(lines[i].trim());
                        i += 1;
                    } else {
                        break;
                    }
                }
                let full_cmd = cmd_lines.join(" ");
                elements.push(DocumentElement::Command(full_cmd));
                continue;
            }

            // Empty line
            if line.trim().is_empty() {
                i += 1;
                continue;
            }

            // Paragraph — collect consecutive non-empty, non-special lines
            let mut para_lines = Vec::new();
            while i < lines.len() {
                let l = lines[i];
                if l.trim().is_empty()
                    || HEADING_RE.is_match(l)
                    || l.starts_with("```")
                    || l.starts_with('|')
                    || LIST_ITEM_RE.is_match(l)
                    || WARNING_RE.is_match(l)
                    || EXAMPLE_RE.is_match(l)
                {
                    break;
                }
                para_lines.push(l);
                i += 1;
            }
            if !para_lines.is_empty() {
                let para = para_lines.join(" ");
                elements.push(DocumentElement::Paragraph(para));
            }
        }

        let full_text = content.to_string();
        let mut doc = Document::new(full_text, author, source);

        // Set derived fields
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("md");
        doc.set_derived(filename, extension);

        doc.elements = elements;
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["md".to_string(), "markdown".to_string()]
    }
}

impl MarkdownParser {
    fn parse_code_block(&self, lines: &[&str], i: &mut usize) -> Option<DocumentElement> {
        let start = *i;
        *i += 1; // skip opening ```

        let lang = if let Some(cap) = CODE_BLOCK_RE.captures(lines[start]) {
            cap[1].to_string()
        } else {
            String::new()
        };

        let mut code_lines = Vec::new();
        let mut end_found = false;

        while *i < lines.len() {
            if lines[*i].trim_start() == "```" {
                *i += 1;
                end_found = true;
                break;
            }
            code_lines.push(lines[*i]);
            *i += 1;
        }

        if end_found {
            let code = code_lines.join("\n");
            Some(DocumentElement::CodeBlock(lang, code))
        } else {
            // No closing ``` found, return as code block anyway
            let code = code_lines.join("\n");
            Some(DocumentElement::CodeBlock(lang, code))
        }
    }

    fn parse_table(&self, lines: &[&str], i: &mut usize) -> Option<Vec<Vec<String>>> {
        let mut table = Vec::new();

        // Collect rows while we see table lines
        while *i < lines.len()
            && (lines[*i].starts_with('|') || TABLE_HEADER_RE.is_match(lines[*i]))
        {
            let line = lines[*i];

            // Skip separator lines (---|---|---)
            if line.contains("---") && line.contains('|') {
                *i += 1;
                continue;
            }

            // Skip lines that are only a separator pattern
            if line
                .chars()
                .filter(|c| *c == '|' || *c == '-' || *c == ':')
                .count()
                > line.len() / 2
            {
                *i += 1;
                continue;
            }

            let cells: Vec<String> = line
                .split('|')
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.trim().to_string())
                .collect();

            if !cells.is_empty() {
                table.push(cells);
            }
            *i += 1;
        }

        if !table.is_empty() {
            Some(table)
        } else {
            None
        }
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}
