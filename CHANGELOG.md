# CHANGELOG

## v1.1.91 - Blank Window Fix — CSP, ErrorBoundary & Robust Startup

- **Fix:** Blank/dark window on first launch after fresh install — added explicit `script-src` CSP directive to allow frontend ES module scripts (the missing directive caused silent CSP violations in some WebView2 contexts)
- **Fix:** Added React `ErrorBoundary` wrapper to catch and display runtime render errors instead of showing a blank screen
- **Fix:** Frontend `/ready` polling timeout increased from 10s to 20s (40 × 500ms) to handle slower Windows initialization (tokio runtime + knowledge pack loading on first launch)
- **Fix:** Startup check wrapped in `try/catch/finally` to guarantee `setChecking(false)` is always called — prevents the app from being stuck in the loading state forever
- **Fix:** Replaced `AbortSignal.timeout()` with manual `AbortController` to avoid environment-specific failures in certain WebView2 contexts

## v1.1.90 - API Server Readiness Fix

- **Fix:** "Cannot reach backend" error during API provider wizard setup — frontend now waits for `/ready` endpoint before making any API calls
- **Fix:** Setup wizard "Test Connection" button now pre-checks server readiness to avoid confusing timeout errors when the backend is still initializing (knowledge packs loading, tokio runtime starting)

## v1.1.89 - Knowledge Pack Schema Compatibility Fixes
- **Fix:** Knowledge packs with documents at root level (not under `documents/` subdirectory) now validate correctly
- **Fix:** Metadata.yaml files with `name`/`version` fields (instead of `pack_name`/`pack_version`) now parse correctly via serde aliases
- **Fix:** Missing `created_at`/`updated_at` timestamps in metadata.yaml no longer cause validation failures — defaults to current time
- **Fix:** Engineering Foundations knowledge packs (linux, networking, security, storage, windows) now pass all 5 validation checks (previously all 5 were INVALID with "Document not found" errors)
- Changed knowledge validation to check pack root as fallback when `documents/` directory is absent