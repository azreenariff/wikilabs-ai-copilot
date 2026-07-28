# CHANGELOG

## v1.1.101 - FIX: build.rs generates clean advice-chat.html with correct JS/CSS hashes at compile time

- **Critical Fix:** The `build.rs` script now dynamically copies JS/CSS files from `src/frontend/dist/assets/` into `src-tauri/assets/` at compile time, eliminating the hash mismatch between advice-chat.html and the actual bundled assets. On CI builds, Vite generates new content hashes — previously the advice-chat.html hardcoded old hashes from the local development build, causing React to fail to mount.
- **Fix:** Also generates advice-chat.html from the frontend's index.html, removing the debug "HTML LOADED OK" red overlay that was previously hardcoded in the HTML file.
- **Result:** The floating advice chat window now always loads the correct React bundle regardless of the build environment.

## v1.1.101 - FIX: Floating advice chat window renders correctly

- **Critical Fix:** The floating "AI Copilot — Live Advice" window showed a red screen with "HTML LOADED OK" instead of the Guidance UI. The API server was serving advice-chat.html but not the referenced static assets (/assets/*.js, /assets/*.css), so React never mounted. Added `tower-http` `fs` feature and `ServeDir::new("../assets")` to serve the static files at `/assets/` path.

## v1.1.99 - FIX: API server bind address — changed from 0.0.0.0 to 127.0.0.1

- **Critical Fix:** Setup wizard "Test Connection" freezes on "Testing..." — the API server was binding to `0.0.0.0:1420` which on Windows can cause TCP connections to succeed but HTTP responses to never be delivered (routing ambiguity with VPN clients/multi-NIC setups). Changed to bind to `127.0.0.1:1420` (loopback only) which eliminates the issue and is also more secure since the API only needs local access.

## v1.1.98 - FIX: Make build_context_system_prompt properly async

- **Fix:** Converted `build_context_system_prompt` from sync to async — replaced `Handle::current().block_on()` with direct `.await` calls
- **Fix:** Called async `build_context_system_prompt` via local `tokio::runtime::Runtime` from sync `handle_send_message` — avoids `Handler` trait issues
- **Fix:** No longer depends on blocking the current tokio runtime's worker thread for panel access

## v1.1.97 - FIX: Setup wizard "Cannot reach backend" — retry logic for early startup

- **Critical Fix:** Setup wizard and Settings "Test Connection" button now use exponential backoff retry when the API server hasn't fully initialized yet (~31s startup time) — eliminates the confusing "Cannot reach backend" error on fresh installs
- **Fix:** Setup wizard `/ready` health pre-check now uses `retryFetch` (5 retries) instead of a single 5s timeout fetch — the server takes ~31s to load knowledge packs, so the old check failed 99% of the time
- **Fix:** Settings "Test Connection" button shows "Testing..." state during retry attempts
- **Fix:** More descriptive error message: "API server may still be starting up, please retry" instead of generic "Cannot reach backend"

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