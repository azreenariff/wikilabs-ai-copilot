//! HTML parser preserving structure.

use super::DocumentElement;
use super::Document;
use super::ParserProvider;
use regex::Regex;
use once_cell::sync::Lazy;

pub struct HtmlParser;

static HEADING_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<h([1-6])[^>]*>(.*?)</h\1>").unwrap());
static P_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<p[^>]*>(.*?)</p>").unwrap());
static TABLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<table[^>]*>(.*?)</table>").unwrap());
static TR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<tr[^>]*>(.*?)</tr>").unwrap());
static TD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(?:td|th)[^>]*>(.*?)</(?:td|th)>").unwrap());
static UL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<ul[^>]*>(.*?)</ul>").unwrap());
static OL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<ol[^>]*>(.*?)</ol>").unwrap());
static LI_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<li[^>]*>(.*?)</li>").unwrap());
static CODE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<code[^>]*>(.*?)</code>").unwrap());
static PRE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<pre[^>]*>(.*?)</pre>").unwrap());
static BLOCKQUOTE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<blockquote[^>]*>(.*?)</blockquote>").unwrap());
static A_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<a[^>]*href=([^>\s]+)[^>]*>(.*?)</a>").unwrap());
static WARN_TAGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(warning|alert|danger|note)").unwrap());

fn strip_html_tags(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(html, "").to_string()
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
        .replace("&ndash;", "-")
        .replace("&hellip;", "...")
}

impl HtmlParser {
    pub fn new() -> Self {
        Self
    }

    fn clean_html_text(&self, text: &str) -> String {
        let decoded = decode_html_entities(text);
        let stripped = strip_html_tags(&decoded);
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(&stripped, " ").trim().to_string()
    }

    fn parse_table(&self, html: &str) -> Option<Vec<Vec<String>>> {
        let mut table = Vec::new();
        for tr_cap in TR_RE.captures_iter(html) {
            let row_html = &tr_cap[1];
            let mut row = Vec::new();
            for td_cap in TD_RE.captures_iter(row_html) {
                let cell_text = self.clean_html_text(&td_cap[1]);
                row.push(cell_text);
            }
            if !row.is_empty() {
                table.push(row);
            }
        }
        if !table.is_empty() {
            Some(table)
        } else {
            None
        }
    }

    fn parse_list_items(&self, html: &str) -> Vec<String> {
        let mut items = Vec::new();
        for li_cap in LI_RE.captures_iter(html) {
            let item_text = self.clean_html_text(&li_cap[1]);
            if !item_text.is_empty() {
                items.push(item_text);
            }
        }
        items
    }
}

impl ParserProvider for HtmlParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let mut elements = Vec::new();

        for cap in HEADING_RE.captures_iter(content) {
            let level: u32 = cap[1].parse().unwrap_or(1);
            let text = self.clean_html_text(&cap[2]);
            elements.push(DocumentElement::Heading(level, text));
        }

        for table_cap in TABLE_RE.captures_iter(content) {
            if let Some(table) = self.parse_table(&table_cap[1]) {
                elements.push(DocumentElement::Table(table));
            }
        }

        for p_cap in P_RE.captures_iter(content) {
            let text = self.clean_html_text(&p_cap[1]);
            if !text.is_empty() {
                elements.push(DocumentElement::Paragraph(text));
            }
        }

        if let Some(ul_cap) = UL_RE.captures(content) {
            let items = self.parse_list_items(&ul_cap[1]);
            if !items.is_empty() {
                elements.push(DocumentElement::List(items));
            }
        }
        if let Some(ol_cap) = OL_RE.captures(content) {
            let items = self.parse_list_items(&ol_cap[1]);
            if !items.is_empty() {
                elements.push(DocumentElement::List(items));
            }
        }

        for code_cap in CODE_RE.captures_iter(content) {
            let text = self.clean_html_text(&code_cap[1]);
            if !text.is_empty() {
                elements.push(DocumentElement::CodeBlock(String::new(), text));
            }
        }
        for pre_cap in PRE_RE.captures_iter(content) {
            let text = self.clean_html_text(&pre_cap[1]);
            if !text.is_empty() {
                elements.push(DocumentElement::CodeBlock(String::new(), text));
            }
        }

        for bq_cap in BLOCKQUOTE_RE.captures_iter(content) {
            let text = self.clean_html_text(&bq_cap[1]);
            if !text.is_empty() {
                elements.push(DocumentElement::Example(text));
            }
        }

        for a_cap in A_RE.captures_iter(content) {
            let url = a_cap[1].replace(['"', '\''], "");
            let text = self.clean_html_text(&a_cap[2]);
            if !url.is_empty() && !text.is_empty() {
                elements.push(DocumentElement::Reference(text, url));
            }
        }

        if WARN_TAGS.is_match(content) {
            let warning_texts: Vec<String> = content.lines()
                .filter(|l| l.to_lowercase().contains("warning") || l.to_lowercase().contains("alert") || l.to_lowercase().contains("danger"))
                .map(|l| self.clean_html_text(l))
                .filter(|t| !t.is_empty())
                .collect();
            for wt in warning_texts {
                elements.push(DocumentElement::Warning(wt));
            }
        }

        let full_text = self.clean_html_text(content);
        let mut doc = Document::new(full_text, author, source);
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("html");
        doc.set_derived(filename, extension);
        doc.elements = elements;
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["html".to_string(), "htm".to_string()]
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}