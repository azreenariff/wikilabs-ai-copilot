# CHANGELOG

## v1.1.96 - FIX: Blank window on launch — removed invalid `url` field from Tauri config

- **Critical Fix:** WebView2 blank screen on launch — the `"url": "local:index.html"` field in `tauri.conf.json` was incorrect for release builds (the `local:` URL scheme only works in dev mode). Removed the `url` field entirely, letting Tauri default to `index.html` from `frontendDist` — the same configuration that worked in v1.1.83-89.
- **Fix:** CI workflow now explicitly installs Node.js and runs `npm ci` + `npm run build` before `cargo tauri build`, ensuring the frontend is freshly built on Windows CI instead of using stale committed dist files.

## v1.1.95 - FIX: API server deadlock — switch to multi-threaded Tokio runtime

- **Critical Fix:** WebView2 stuck on `about:blank` due to API server deadlock — the single-threaded Tokio runtime caused `Handle::current().block_on()` to deadlock when sync IPC handlers tried to access async `GuidancePanel` methods
- **Fix:** Switched from `Runtime::new()` (single-threaded) to `Builder::new_multi_thread()` with 4 worker threads in `start_api_server()`
- **Fix:** Replaced `futures::executor::block_on()` with proper `tokio::runtime::Handle::current().block_on()` in `build_context_system_prompt()`
- **Cleanup:** Removed broken `get_recent_events_sync()` method and unused `futures` dependency

## v1.1.94 - DEBUG: Enable WebView2 devtools for blank window debugging

- **Critical Fix:** Blank/dark window on first launch after fresh install — the Tauri window URL was `"local:../src/frontend/dist/index.html"` which resolves OUTSIDE the `frontendDist` directory (Tauri prepends `frontendDist` to `local:` URLs, so this resolves to `../../src/frontend/dist/index.html`). Changed to `"local:index.html"` which correctly loads `frontendDist/index.html`.
- **Fix:** Removed CSP entirely (CSP was also causing issues in WebView2 on Windows).
- **Fix:** Added visible "Page Loaded — Checking..." debug indicator in HTML to confirm page renders even before React mounts.
- **Fix:** Added inline JS error handler to display errors on the page instead of blank screen.
- **Fix:** Frontend `/ready` polling increased to 20s, wrapped in try/catch/finally.
- **Fix:** Replaced `AbortSignal.timeout()` with manual `AbortController`.
- **Fix:** Added React ErrorBoundary wrapper.

## v1.1.92 - Blank Window Fix — Removed CSP, Added Debug Logging

- **Fix:** Blank/dark window on first launch after fresh install — removed CSP entirely (CSP was likely causing silent blocking of the React bundle in certain WebView2 contexts on Windows)
- **Fix:** Added inline JS error handler to `index.html` — if a JS error occurs, a visible error message is displayed instead of a blank screen
- **Fix:** Added visible "Initializing..." fallback in HTML — confirms whether the page is rendering at all, even before React mounts
- **Fix:** Added `Window.__react_ready__` signal so React mount is visible on the page
- **Fix:** Added `__react_ready__` type declaration in `vite-env.d.ts`

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