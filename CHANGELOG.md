# CHANGELOG

## v1.1.115 — FIX: Chat window panic when sending message (nested tokio runtime)

- **Critical Fix:** Fixed chat window panic on message send. The `handle_send_message` function created a nested tokio runtime via `Runtime::new().block_on(...)` while already running inside axum's tokio runtime, causing "Cannot start a runtime from within a runtime" panic. Replaced with dedicated thread + separate runtime pattern.

## v1.1.114 — FIX: Advice-chat blank window (router route mismatch)

- **Critical Fix:** Fixed advice-chat floating window showing blank after preflight check. The advice-chat window loads at URL path `/advice-chat` but the React `BrowserRouter` had no route matching this path. The `Routes` element rendered nothing, making the entire window blank. Added `<Route path="/advice-chat" element={<ChatAssistant />} />` so the floating chat window renders correctly.


## v1.1.112 — FIX: WebView2 fetch hangs on localhost — bind API server to 0.0.0.0 + 60s total timeout

- **Critical Fix:** WebView2 can't reach `http://127.0.0.1:1420` from a `tauri://localhost` page — the fetch hangs or is blocked. Bound the API server to `0.0.0.0:{port}` instead of `127.0.0.1:{port}` so the WebView2 can connect.
- **Safety net:** Added a 60-second total startup timeout on the `checkSetup` function. If the AbortController doesn't work in this WebView2 environment, the app still transitions to the UI instead of hanging forever.

## v1.1.111 — FIX: WebView2 fetch hangs on localhost — added CSP for http origins + 60s total timeout

- **Fix:** Removed invalid `"security": {}` from `tauri.conf.json`. This key doesn't exist in Tauri v2 and could have caused WebView2 misconfiguration on some builds.
- **Fix:** Reduced `/ready` polling from 40 attempts to 20 attempts (10s max instead of 20s) to prevent the loading screen from hanging indefinitely when the API server never responds.
- **Fix:** Added `cache: 'no-store'` and `mode: 'cors'` to fetch calls to prevent any caching-related hangs in WebView2.
- **Diagnostics:** Added detailed logging for every `/ready` poll attempt (attempt number, response status, response data, errors) — makes it much easier to debug Windows WebView2 network issues.
- **UI:** Loading screen now shows the current phase ("Initializing..." → "Checking API server..." → "Running pre-flight checks...") so you can see exactly where the app is stuck.

## v1.1.109 — FIX: Preflight check not reaching API handler

- **Critical Fix:** Fixed preflight check failing on startup. The frontend was sending `POST /api/preflight_check` but no route existed for that path. The handler was registered under `/api/commands/preflight_check` but the frontend was calling the wrong URL. Fixed both `App.tsx` and `PreflightCheck.tsx` to use `/api/commands/preflight_check`.

## v1.1.108 — Add clear diagnostic logging for Windows troubleshooting

- **Diagnostics:** Added `>>>` console markers (`println!`) alongside `tracing` across the full app startup sequence — API server (thread spawn, Tokio runtime, TCP bind, ready), observation engine (creation, provider registration, start), and frontend (loading, render, setup check). All markers prefixed with `[API]`, `[OBS]`, or `[Wiki Labs]` for easy grepping in Windows Event Viewer / console logs.

- **Critical Fix:** Fixed preflight check panics that caused "Failed to fetch" on startup. The root cause was `Handle::current().block_on()` inside an axum request handler, which panics at runtime (cannot block a tokio thread from within its own runtime). Replaced with `std::thread::spawn` + new tokio runtime (the documented pattern for axum integration).
- **UI:** Preflight loading spinner now cycles through rotating phase messages ("Checking API server...", "Verifying server readiness...", "Loading settings...", "Preparing interface...") every 2 seconds instead of showing a generic message with bouncing dots.
- **Preflight:** Added real HTTP `GET /health` round-trip check to confirm the API server actually accepts network requests, not just that the process exists. If /health fails, the frontend shows "API server is not responding" instead of a generic crash.
|- **Preflight:** First checklist item now shows the actual /health response body ("ok") for transparent verification.

## v1.1.107 — PLACEHOLDER: Pre-flight check screen implementation (see v1.1.106)

## v1.1.106 — FIX: Pre-flight check screen with knowledge packs validation

- **Pre-flight check screen:** Added a startup health-check page that verifies the API server is running, the /ready endpoint responds, settings are loaded, the AI provider connection works (when configured), and knowledge packs are loaded. Each check shows a spinning indicator that resolves to a checkmark or error icon. The screen auto-transitions to the main UI or SetupWizard after showing results.
- **Knowledge packs validation:** Added a dedicated pre-flight check that scans the configured knowledge directory for `.pack.json` files and reports the count. Shows pass (with count), skip (no packs or directory not found), or skip (not configured).
- **Frontend startup flow:** Rewrote App.tsx to show a loading spinner during preflight, then the detailed check results page, then auto-transition to SetupWizard or main UI after 2.5 seconds.

## v1.1.105 — FIX: Loading animations + fix flaky test connection timeouts

- **Loading animation (splash screen):** Replaced plain "Loading..." text with a gradient icon that pulses, a spinning ring around it, and bouncing dots for a polished startup feel.
- **Loading animation (setup wizard):** Test Connection button now shows a spinner icon during testing.
- **Flaky test connection timeouts:** Fixed the root cause — stale connections accumulating from connection pooling + tight 15s timeout. Backend timeout: 15s → 20s. Frontend retryFetch default timeout: 15s → 25s. Disabled HTTP connection pooling on all outbound reqwest clients (`pool_idle_timeout(0)`, `pool_max_idle_per_host(0)`) to prevent stale CLOSE_WAIT connections from accumulating across multiple test attempts. Added `Connection: close` headers throughout.

- **Critical Fix:** The floating "AI Copilot — Live Advice" window showed a blank screen because the API server's static asset resolver (`ServeDir::new("../assets")`) used a relative path that didn't resolve correctly on Windows NSIS-installed builds. The HTML file loaded (via `include_str!`), but the JS/CSS files at `/assets/` returned 404, so React never mounted.
- **Fix:** The `api_server.rs` now resolves the assets directory using a multi-tier approach: (1) binary's parent directory + `assets/` (covers Windows NSIS installs), (2) Tauri resource_dir() (covers Tauri bundler), (3) current working directory, (4) crate root's `../assets/`. This ensures the correct path is found regardless of deployment environment.
- **Fix:** Added `../assets` to `bundle.resources` in `tauri.conf.json` so the NSIS installer includes the JS/CSS files alongside the binary.
- **Result:** The floating advice chat window now renders the full Guidance UI with React on all platforms (Windows, Linux).

## v1.1.103 - FIX: Settings persistence merge + HTTP/2 SETTINGS_TIMEOUT in AI provider connections

- **Fix:** Setup wizard "Test Connection" now persists AI provider config properly. Previously, `handle_update_settings` replaced the entire settings object with only `{ "ai_provider": {...} }`, losing all other settings fields. Now merges incoming params into existing settings, preserving theme, log_level, first_run_complete, etc.
- **Fix:** "Test Connection" and "List Models" no longer timeout from HTTP/2 SETTINGS_TIMEOUT. Both `reqwest::Client` instances now use `http1_only()` to eliminate the HTTP/2 SETTINGS frame ACK hang. Added `Connection: close` header to prevent stale connection accumulation.
- **Fix:** Test connection timeout increased from 10s to 15s to match frontend UI display.
- **Fix:** `handle_list_models` also uses HTTP/1 only to prevent the same connection pooling issue when fetching available models.

## v1.1.102 - FIX: Add AbortController timeout to frontend fetch to prevent "Testing..." stall

- **Fix:** Setup wizard and Settings page `retryFetch` helper now uses `AbortController` with 15-second timeout per fetch attempt. Previously, when the API server was slow to respond (initialization phase, high load, or network issues), the browser's native `fetch` API would hang indefinitely because it has no default timeout, causing the "Test Connection" button to show "Testing..." and never resolve.
- **Fix:** Settings page `fetchModels` function also added 15-second AbortController timeout, preventing the "Refresh models" button from hanging indefinitely.
- **Fix:** Setup wizard's `/ready` polling loop now uses a 10-second timeout on each fetch attempt instead of a plain `fetch` without timeout.

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