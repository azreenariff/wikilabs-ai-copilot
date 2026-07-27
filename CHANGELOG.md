# CHANGELOG

## v1.1.89 - Knowledge Pack Schema Compatibility Fixes
- **Fix:** Knowledge packs with documents at root level (not under `documents/` subdirectory) now validate correctly
- **Fix:** Metadata.yaml files with `name`/`version` fields (instead of `pack_name`/`pack_version`) now parse correctly via serde aliases
- **Fix:** Missing `created_at`/`updated_at` timestamps in metadata.yaml no longer cause validation failures — defaults to current time
- **Fix:** Engineering Foundations knowledge packs (linux, networking, security, storage, windows) now pass all 5 validation checks (previously all 5 were INVALID with "Document not found" errors)
- Changed knowledge validation to check pack root as fallback when `documents/` directory is absent