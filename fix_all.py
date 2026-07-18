#!/usr/bin/env python3
"""Fix all compilation errors in wikilabs-knowledge systematically."""
import subprocess, re, os, sys

def run_cmd(cmd):
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.returncode, result.stdout, result.stderr

def read_file(path):
    with open(path, 'r') as f:
        return f.read()

def write_file(path, content):
    with open(path, 'w') as f:
        f.write(content)
    print(f"  Wrote {path}")

def fix_file(path, old, new):
    content = read_file(path)
    if old in content:
        content = content.replace(old, new)
        write_file(path, content)
        return True
    return False

os.chdir("/home/khopu/wikilabs-ai-copilot")

# 1. Fix E0407: Add missing methods to ParserProvider trait
print("1. Fixing E0407...")
parser_mod = read_file("src/knowledge/src/processing/mod.rs")
trait_addition = """
    fn parse_table(&self, _html: &str) -> Option<Vec<Vec<String>>> { None }
    fn parse_list_items(&self, _html: &str) -> Vec<String> { vec![] }
    fn extract_structure(&self, _text: &str) -> Vec<DocumentElement> { vec![] }
    fn is_heading(&self, _line: &str, _lines: &[&str], _idx: usize) -> bool { false }
    fn estimate_heading_level(&self, _line: &str) -> u32 { 2 }
    fn is_code_block_line(&self, _line: &str) -> bool { false }
    fn parse_pipe_table(&self, _lines: &[&str], _i: &mut usize) -> Option<Vec<Vec<String>>> { None }
    fn parse_docx_bytes(&self, _bytes: &[u8], _author: &str, _source: &str) -> Document {
        Document::new(String::new(), "", "")
    }
    fn extract_text_from_docx(&self, _bytes: &[u8]) -> Option<String> { None }
    fn extract_structure_from_text(&self, _text: &str) -> Vec<DocumentElement> { vec![] }
"""
trait_start = parser_mod.find("pub trait ParserProvider")
if trait_start >= 0:
    remaining = parser_mod[trait_start:]
    parse_method = remaining.find("fn parse(&self, content: &str, author: &str, source: &str) -> Document;")
    if parse_method >= 0:
        insert_pos = trait_start + parse_method + len("fn parse(&self, content: &str, author: &str, source: &str) -> Document;")
        brace_count = 1
        trait_end = insert_pos
        for i, ch in enumerate(remaining[insert_pos:], 1):
            if ch == '{':
                brace_count += 1
            elif ch == '}':
                brace_count -= 1
                if brace_count == 0:
                    trait_end = insert_pos + i
                    break
        if trait_end > 0:
            parser_mod = parser_mod[:trait_end] + trait_addition + parser_mod[trait_end:]
            write_file("src/knowledge/src/processing/mod.rs", parser_mod)
            print("  Added methods to ParserProvider")

# 2. Fix E0004: Missing match arms for DocumentElement
print("2. Fixing E0004...")
# clean.rs - need to add match arms after Text arm
clean_rs = read_file("src/knowledge/src/pipeline/steps/clean.rs")
# Find the last match arm before the closing brace
old_clean = '''DocumentElement::Text(t) => text.push_str(&format!("{}\n", t.trim())),
        }'''
new_clean = '''DocumentElement::Text(t) => text.push_str(&format!("{}\n", t.trim())),
            DocumentElement::InlineCode(_) | DocumentElement::Bold(_) => {},
        }'''
fix_file("src/knowledge/src/pipeline/steps/clean.rs", old_clean, new_clean)

# normalize.rs
old_norm = '''DocumentElement::List(_) => {
                for item in item_list {
                    text.push_str(&format!("- {}\n", item));
                }
            }
        }'''
new_norm = '''DocumentElement::List(_) => {
                for item in item_list {
                    text.push_str(&format!("- {}\n", item));
                }
            }
            DocumentElement::InlineCode(_) | DocumentElement::Bold(_) => {},
        }'''
fix_file("src/knowledge/src/pipeline/steps/normalize.rs", old_norm, new_norm)

# chunk.rs - match on &DocumentElement
old_chunk = '''DocumentElement::Paragraph(p) => text.push_str(&p),
            }'''
new_chunk = '''DocumentElement::Paragraph(p) => text.push_str(&p),
            &DocumentElement::InlineCode(_) | &DocumentElement::Bold(_) => {},
            }'''
fix_file("src/knowledge/src/pipeline/steps/chunk.rs", old_chunk, new_chunk)

# 3. Fix E0432: namespace self-import
print("3. Fixing E0432...")
fix_file("src/knowledge/src/storage/namespace.rs", "use super::namespace::Namespace;\n", "")
fix_file("src/knowledge/src/storage/namespace.rs", "use super::namespace::Namespace;", "")

# 4. Fix E0433: double crate path
print("4. Fixing E0433...")
fix_file("src/knowledge/src/embedding_pipeline/provider.rs",
    "crate::embedding::crate::embedding::local::LocalEmbeddingProvider::new()",
    "crate::embedding::EmbeddingPipeline::new(crate::embedding::EmbeddingPipelineConfig::default())")

# 5. Fix E0425: path_str in discover.rs
print("5. Fixing E0425 in discover.rs...")
fix_file("src/knowledge/src/pipeline/steps/discover.rs",
    "let path = Path::new(entry_path);",
    "let path = Path::new(path_str);")
fix_file("src/knowledge/src/pipeline/steps/discover.rs",
    'format!("Failed to read directory: {}", path_str)',
    'format!("Failed to read directory: {}", path.to_str().unwrap_or("?"))')
fix_file("src/knowledge/src/pipeline/steps/discover.rs",
    'entry_path.metadata().with_context(|| format!("Failed to read metadata for {}", path_str))',
    'entry_path.metadata().with_context(|| format!("Failed to read metadata for {}", entry_path.display()))')

# 6. Fix E0422: DocumentManifest import in templates.rs
print("6. Fixing E0422...")
tmpl = read_file("src/knowledge/src/sdk/templates.rs")
if "use crate::sdk::DocumentManifest;" not in tmpl:
    tmpl = tmpl.replace("use super::schema::Manifest;", 
        "use super::schema::Manifest;\n    use crate::sdk::DocumentManifest;\n    use crate::sdk::PackMetadata;\n    use crate::sdk::DocumentMetadata;")
    write_file("src/knowledge/src/sdk/templates.rs", tmpl)

# 7. Fix E0599: TestResult missing Clone
print("7. Fixing E0599...")
fix_file("src/knowledge/src/sdk/testing.rs",
    "pub struct TestResult {",
    "#[derive(Clone)]\npub struct TestResult {")

# 8. Fix E0596: mutable borrow on &self in embedding/local.rs
print("8. Fixing E0596...")
local = read_file("src/knowledge/src/embedding/local.rs")
# Remove cache-based approach since &self can't mutate
old_embed = """async fn embed(&self, text: &str) -> anyhow::Result<EmbeddingResult> {
        let seed = self.hash_seed(text);
        if let Some(vector) = self.seed_cache.get(&text.to_string()) {
            return Ok(EmbeddingResult {
                embedding: vector.clone(),
                model: self.model_name().to_string(),
                dimensions: self.dimensions(),
                text: text.to_string(),
            });
        }
        let mut rng = StdRng::seed_from_u64(seed);
        let vector: Vec<f32> = (0..self.dimensions()).map(|_| rng.gen()).collect();
        self.seed_cache.insert(text.to_string(), vector.clone());
        Ok(EmbeddingResult {
            embedding: vector,
            model: self.model_name().to_string(),
            dimensions: self.dimensions(),
            text: text.to_string(),
        })
    }"""
new_embed = """async fn embed(&self, _text: &str) -> anyhow::Result<EmbeddingResult> {
        let mut rng = StdRng::seed_from_u64(self.hash_seed(_text));
        let vector: Vec<f32> = (0..self.dimensions()).map(|_| rng.gen()).collect();
        Ok(EmbeddingResult {
            embedding: vector,
            model: self.model_name().to_string(),
            dimensions: self.dimensions(),
            text: _text.to_string(),
        })
    }"""
local = local.replace(old_embed, new_embed)

old_batch = """async fn embed_batch(&self, texts: Vec<&str>) -> anyhow::Result<Vec<EmbeddingResult>> {
        let mut results = Vec::new();
        for text in texts {
            let mut rng = StdRng::seed_from_u64(self.hash_seed(text));
            if let Some(vector) = self.seed_cache.get(&text.to_string()) {
                results.push(EmbeddingResult {
                    embedding: vector.clone(),
                    model: self.model_name().to_string(),
                    dimensions: self.dimensions(),
                    text: text.to_string(),
                });
            } else {
                let vector: Vec<f32> = (0..self.dimensions()).map(|_| rng.gen()).collect();
                self.seed_cache.insert((*text).to_string(), vector.clone());
                results.push(EmbeddingResult {
                    embedding: vector,
                    model: self.model_name().to_string(),
                    dimensions: self.dimensions(),
                    text: text.to_string(),
                });
            }
        }
        Ok(results)
    }"""
new_batch = """async fn embed_batch(&self, texts: Vec<&str>) -> anyhow::Result<Vec<EmbeddingResult>> {
        let mut results = Vec::new();
        for text in texts {
            let mut rng = StdRng::seed_from_u64(self.hash_seed(text));
            let vector: Vec<f32> = (0..self.dimensions()).map(|_| rng.gen()).collect();
            results.push(EmbeddingResult {
                embedding: vector,
                model: self.model_name().to_string(),
                dimensions: self.dimensions(),
                text: text.to_string(),
            });
        }
        Ok(results)
    }"""
local = local.replace(old_batch, new_batch)
write_file("src/knowledge/src/embedding/local.rs", local)

# 9. Fix E0507: move out of shared reference in proximity.rs
print("9. Fixing E0507...")
fix_file("src/knowledge/src/association/proximity.rs",
    "s.relationship.unwrap_or(EdgeType::Related)",
    "s.relationship.clone().unwrap_or(EdgeType::Related)")

# 10. Fix E0716: temporary dropped while borrowed
print("10. Fixing E0716...")
prox = read_file("src/knowledge/src/association/proximity.rs")
prox = re.sub(
    r'let words_a: std::collections::HashSet<&str> = text_a\s+\.to_lowercase\(\)\s+\.split_whitespace\(\)\s+\.collect\(\);',
    'let lower_a = text_a.to_lowercase();\n        let words_a: std::collections::HashSet<&str> = lower_a\n            .split_whitespace()\n            .collect();',
    prox
)
prox = re.sub(
    r'let words_b: std::collections::HashSet<&str> = text_b\s+\.to_lowercase\(\)\s+\.split_whitespace\(\)\s+\.collect\(\);',
    'let lower_b = text_b.to_lowercase();\n        let words_b: std::collections::HashSet<&str> = lower_b\n            .split_whitespace()\n            .collect();',
    prox
)
write_file("src/knowledge/src/association/proximity.rs", prox)

# 11. Fix E0277: f32 not Eq
print("11. Fixing E0277...")
fix_file("src/knowledge/src/association/mod.rs",
    "#[derive(Debug, Clone, PartialEq, Eq)]",
    "#[derive(Debug, Clone, PartialEq)]")

# 12. Fix E0382: clone moved values
print("12. Fixing E0382...")
fix_file("src/knowledge/src/pack/mod.rs", "manifest: manifest,", "manifest: manifest.clone(),")
fix_file("src/knowledge/src/storage/migration.rs", "results.push(result);", "results.push(result.clone());")
fix_file("src/knowledge/src/retrieval/chunker.rs", "chunks.push(current_chunk);", "chunks.push(current_chunk.clone());")
fix_file("src/knowledge/src/association/graph.rs",
    "self.adjacency.entry(id).or_insert_with(Vec::new);",
    "self.adjacency.entry(id.clone()).or_insert_with(Vec::new);")
fix_file("src/knowledge/src/association/graph.rs",
    "self.adjacency.get(node_id).unwrap_or(&Vec::new())",
    "self.adjacency.get(node_id).cloned().unwrap_or_default()")
fix_file("src/knowledge/src/association/topic.rs",
    "self.topics.insert(topic.name.clone(), topic);",
    "self.topics.insert(topic.name.clone(), topic.clone());")
fix_file("src/knowledge/src/association/topic.rs",
    "self.associations.push(association);",
    "self.associations.push(association.clone());")
fix_file("src/knowledge/src/association/filter.rs",
    ".into_iter()",
    ".iter().cloned().into_iter()")
fix_file("src/knowledge/src/performance/cache.rs",
    "self.access_order.push(key);",
    "self.access_order.push(key.clone());")
fix_file("src/knowledge/src/performance/cache.rs",
    "self.access_order.push(access_key);",
    "self.access_order.push(access_key.clone());")
fix_file("src/knowledge/src/performance/cache.rs",
    "for key in expired_keys {",
    "for key in &expired_keys {")

# 13. Fix E0308: mismatched types
print("13. Fixing E0308...")
fix_file("src/knowledge/src/sdk/create_pack.rs", "if is_file {", "if *is_file {")

# metadata/mod.rs - Error type mismatch
mm = read_file("src/knowledge/src/metadata/mod.rs")
mm = mm.replace(
    '.query_map(params, |row| self.row_to_entry(row))?;',
    '.query_map(params, |row| self.row_to_entry(row).map_err(|e| rusqlite::Error::QueryFailed(e.to_string())))?;'
)
write_file("src/knowledge/src/metadata/mod.rs", mm)

# 14. Fix E0282: type inference
print("14. Fixing E0282...")
fix_file("src/knowledge/src/sdk/validate.rs",
    "serde_yaml::from_str(&manifest_content)",
    "serde_yaml::from_str::<crate::sdk::schema::Manifest>(&manifest_content)")

# 15. Fix E0425: EmbeddingResult not found in embedding/mod.rs
print("15. Fixing E0425...")
emb = read_file("src/knowledge/src/embedding/mod.rs")
if "use crate::embedding::provider::EmbeddingResult;" not in emb:
    emb = emb.replace("use anyhow::Result;", "use anyhow::Result;\nuse crate::embedding::provider::EmbeddingResult;")
    write_file("src/knowledge/src/embedding/mod.rs", emb)

# 16. Fix docx.rs - content undefined
print("16. Fixing docx.rs...")
fix_file("src/knowledge/src/processing/docx.rs",
    "let content = document_xml;\n        let elements = self.extract_structure_from_text(content);\n        let mut doc = Document::new(content.to_string(), author, source);",
    "let content = document_xml;\n        let elements = self.extract_structure_from_text(&content);\n        let mut doc = Document::new(content.clone(), author, source);")

# 17. Fix yaml.rs/json.rs - DocumentMetadata import
print("17. Fixing yaml.rs/json.rs DocumentMetadata...")
fix_file("src/knowledge/src/processing/yaml.rs",
    "super::DocumentMetadata::new()",
    "crate::pipeline::steps::metadata_extract::DocumentMetadata::new()")
fix_file("src/knowledge/src/processing/json.rs",
    "super::DocumentMetadata::new()",
    "crate::pipeline::steps::metadata_extract::DocumentMetadata::new()")

print("\n=== All fixes applied ===")
exit_code, stdout, stderr = run_cmd("source ~/.cargo/env && cargo build 2>&1 | grep -c '^error'")
print(f"Remaining errors: {stdout.strip()}")

# Print error breakdown
ec2, st2, _ = run_cmd("source ~/.cargo/env && cargo build 2>&1 | grep 'error\\[' | sed 's/.*error\\[E\\([0-9]*\\)\\].*/E\\1/' | sort | uniq -c | sort -rn")
if st2.strip():
    print("\nError breakdown:")
    print(st2.strip())