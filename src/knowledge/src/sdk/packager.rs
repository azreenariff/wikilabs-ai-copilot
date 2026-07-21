//! Packager — produces .wkl (Wiki Labs Knowledge) archive files.

use anyhow::{Context, Result};
use std::fs;

use std::path::Path;
use tracing::{debug, info};

/// The file extension for knowledge pack archives.
pub const WKL_EXTENSION: &str = "wkl";

/// Creates a .wkl archive from a knowledge pack directory.
///
/// The archive is a ZIP file with a specific internal structure:
/// ```text
/// pack.wkl
/// ├── manifest.yaml
/// ├── metadata.yaml
/// ├── documents/
/// │   ├── doc1.md
/// │   └── doc2.md
/// ├── tests/
/// └── documentation/
/// ```
///
/// # Arguments
/// * `pack_path` — Path to the knowledge pack directory
/// * `output_path` — Path where the .wkl archive will be written
pub fn package_pack(pack_path: &str, output_path: &str) -> Result<()> {
    let pack_dir = Path::new(pack_path);
    let output = Path::new(output_path);

    if !pack_dir.exists() {
        anyhow::bail!("Knowledge pack directory not found: {}", pack_path);
    }

    // Verify required files exist
    let manifest = pack_dir.join("manifest.yaml");
    let metadata = pack_dir.join("metadata.yaml");

    if !manifest.exists() {
        anyhow::bail!("manifest.yaml not found in pack directory");
    }
    if !metadata.exists() {
        anyhow::bail!("metadata.yaml not found in pack directory");
    }

    info!(pack_path = %pack_path, output_path = %output_path, "Packaging knowledge pack");

    // Read required files
    let manifest_bytes = fs::read(&manifest).with_context(|| "Failed to read manifest.yaml")?;
    let metadata_bytes = fs::read(&metadata).with_context(|| "Failed to read metadata.yaml")?;

    // Create the archive
    let mut archive = ArchiveBuilder::new();
    archive.add_file("manifest.yaml", &manifest_bytes)?;
    archive.add_file("metadata.yaml", &metadata_bytes)?;

    // Add documents/ if it exists
    let docs_dir = pack_dir.join("documents");
    if docs_dir.exists() {
        add_directory_recursive(&mut archive, &docs_dir, "documents/")?;
    }

    // Add tests/ if it exists
    let tests_dir = pack_dir.join("tests");
    if tests_dir.exists() {
        add_directory_recursive(&mut archive, &tests_dir, "tests/")?;
    }

    // Add documentation/ if it exists
    let doc_dir = pack_dir.join("documentation");
    if doc_dir.exists() {
        add_directory_recursive(&mut archive, &doc_dir, "documentation/")?;
    }

    // Write archive to output
    let archive_bytes = archive.build()?;
    fs::write(output, archive_bytes)
        .with_context(|| format!("Failed to write archive to {}", output.display()))?;

    // Verify the output has the right extension
    let ext = output.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != WKL_EXTENSION {
        debug!(
            "Output extension is '{}', expected '{}'",
            ext, WKL_EXTENSION
        );
    }

    let file_size = fs::metadata(output)?.len();
    info!(
        output_path = %output_path,
        file_size,
        "Knowledge pack packaged successfully"
    );

    Ok(())
}

/// Extracts a .wkl archive to the specified directory.
///
/// # Arguments
/// * `archive_path` — Path to the .wkl archive
/// * `output_dir` — Directory to extract to
pub fn extract_pack(archive_path: &str, output_dir: &str) -> Result<()> {
    let archive = Path::new(archive_path);
    let output = Path::new(output_dir);

    if !archive.exists() {
        anyhow::bail!("Archive not found: {}", archive_path);
    }

    info!(archive_path = %archive_path, output_dir = %output_dir, "Extracting knowledge pack");

    let archive_bytes = fs::read(archive).with_context(|| "Failed to read archive")?;
    let archive = Archive::parse(&archive_bytes)?;

    // Create output directory
    fs::create_dir_all(output)
        .with_context(|| format!("Failed to create output directory: {}", output.display()))?;

    for entry in archive.entries {
        let target_path = output.join(&entry.path);
        let dir = target_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("No parent directory for entry: {}", entry.path))?;
        fs::create_dir_all(dir)?;
        fs::write(&target_path, &entry.data)
            .with_context(|| format!("Failed to write extracted file: {}", entry.path))?;
    }

    info!("Knowledge pack extracted successfully to {}", output_dir);
    Ok(())
}

/// A simple archive format for knowledge packs.
/// Uses raw file entries with path + data pairs.
struct Archive {
    entries: Vec<ArchiveEntry>,
}

struct ArchiveEntry {
    path: String,
    data: Vec<u8>,
}

struct ArchiveBuilder {
    entries: Vec<ArchiveEntry>,
}

impl ArchiveBuilder {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add_file(&mut self, path: &str, data: &[u8]) -> Result<()> {
        self.entries.push(ArchiveEntry {
            path: path.to_string(),
            data: data.to_vec(),
        });
        debug!(entry_path = %path, "Added file to archive");
        Ok(())
    }

    fn build(self) -> Result<Vec<u8>> {
        // Create a simple archive format:
        // [4-byte entry count] [each entry: 4-byte path len, path bytes, 4-byte data len, data bytes]
        let mut bytes = Vec::new();

        // Write entry count
        let count = self.entries.len() as u32;
        bytes.extend_from_slice(&count.to_le_bytes());

        for entry in self.entries {
            // Write path
            let path_bytes = entry.path.as_bytes();
            let path_len = path_bytes.len() as u32;
            bytes.extend_from_slice(&path_len.to_le_bytes());
            bytes.extend_from_slice(path_bytes);

            // Write data
            let data_len = entry.data.len() as u32;
            bytes.extend_from_slice(&data_len.to_le_bytes());
            bytes.extend_from_slice(&entry.data);
        }

        Ok(bytes)
    }
}

impl Archive {
    /// Parses an archive from bytes.
    fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            anyhow::bail!("Archive too short");
        }

        let entry_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let mut entries = Vec::with_capacity(entry_count);
        let mut offset = 4usize;

        for _ in 0..entry_count {
            // Read path length
            if offset + 4 > data.len() {
                anyhow::bail!("Unexpected end of archive reading path length");
            }
            let path_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            // Read path bytes
            if offset + path_len > data.len() {
                anyhow::bail!("Unexpected end of archive reading path");
            }
            let path = String::from_utf8(data[offset..offset + path_len].to_vec())
                .context("Invalid path in archive")?;
            offset += path_len;

            // Read data length
            if offset + 4 > data.len() {
                anyhow::bail!("Unexpected end of archive reading data length");
            }
            let data_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            // Read data bytes
            if offset + data_len > data.len() {
                anyhow::bail!("Unexpected end of archive reading data");
            }
            let entry_data = data[offset..offset + data_len].to_vec();
            offset += data_len;

            entries.push(ArchiveEntry {
                path,
                data: entry_data,
            });
        }

        Ok(Self { entries })
    }
}

/// Recursively adds files from a directory to the archive.
fn add_directory_recursive(builder: &mut ArchiveBuilder, dir: &Path, prefix: &str) -> Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let relative_path = format!("{}{}", prefix, file_name);

        if path.is_file() {
            let data = fs::read(&path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            builder.add_file(&relative_path, &data)?;
        } else if path.is_dir() {
            add_directory_recursive(builder, &path, &format!("{}{}/", prefix, file_name))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_pack(tmp: &TempDir) -> (String, String) {
        let pack_dir = tmp.path().join("test-pack");
        fs::create_dir_all(pack_dir.join("documents")).unwrap();
        fs::write(
            pack_dir.join("manifest.yaml"),
            "schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: Test pack\nauthor: Test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n",
        )
        .unwrap();
        fs::write(
            pack_dir.join("metadata.yaml"),
            "pack_name: test-pack\npack_version: '1.0.0'\ndescription: Test pack\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();
        fs::write(pack_dir.join("documents/test.md"), "# Test\n").unwrap();

        (
            pack_dir.to_string_lossy().to_string(),
            tmp.path().to_string_lossy().to_string(),
        )
    }

    #[test]
    fn test_package_and_extract() {
        let tmp = TempDir::new().unwrap();
        let (pack_path, tmp_path) = create_test_pack(&tmp);

        let output_path = tmp.path().join("test.wkl");
        package_pack(&pack_path, output_path.to_str().unwrap()).unwrap();

        assert!(output_path.exists());
        assert_eq!(output_path.extension().unwrap(), "wkl");

        // Verify archive can be parsed
        let archive_bytes = fs::read(&output_path).unwrap();
        let archive = Archive::parse(&archive_bytes).unwrap();
        assert!(archive.entries.len() >= 2); // manifest + metadata
        let paths: Vec<&str> = archive.entries.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"manifest.yaml"));
        assert!(paths.contains(&"metadata.yaml"));
        assert!(paths.contains(&"documents/test.md"));
    }

    #[test]
    fn test_extract_pack() {
        let tmp = TempDir::new().unwrap();
        let (pack_path, tmp_path) = create_test_pack(&tmp);

        let archive_path = tmp.path().join("test.wkl");
        package_pack(&pack_path, archive_path.to_str().unwrap()).unwrap();

        let extract_dir = tmp.path().join("extracted");
        extract_pack(
            archive_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
        )
        .unwrap();

        assert!(extract_dir.join("manifest.yaml").exists());
        assert!(extract_dir.join("metadata.yaml").exists());
        assert!(extract_dir.join("documents/test.md").exists());
    }

    #[test]
    fn test_package_missing_directory() {
        let tmp = TempDir::new().unwrap();
        let result = package_pack(
            "/nonexistent/path",
            tmp.path().join("out.wkl").to_str().unwrap(),
        );
        assert!(result.is_err());
    }
}
