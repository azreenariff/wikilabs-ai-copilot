|# Observation Engine — Screen Capture + Vision AI Integration
|
||## Project Status: Phase 1 ✓ Complete, Phase 2 ✓ Complete, Phase 3 ✓ Complete, Phase 4 ✓ Complete
||
|### Background
The Observation Framework has structural support for screen capture (`screen_capture.rs`, `capture.rs`, `ocr.rs`) but these are all **stubs**. The real gap is that structured data (Win32 API window text, clipboard, process names) can't reliably capture what's actually on screen — leading to the copilot missing critical context like browser error pages and terminal commands.

### Goal
Build a true "copilot advisor" that can:
1. See the full screen (via screen capture)
2. Understand what's on screen (via Vision AI — GPT-4o / Claude / Gemini)
3. Understand what the user is doing and their intent (terminal commands, browser pages, clipboard, errors)
4. Check if the user's commands are correct
5. Correlate all signals and provide proactive guidance/advice

### Architecture Design

#### Current Components (existing)
- `src/observation/src/engine.rs` — ObservationEngine orchestrates providers, runs polling loop
- `src/observation/src/provider.rs` — Trait-based provider interface (start/stop/pause/observe)
- `src/observation/src/event.rs` — EventType enum, ObservationEvent, ObservationPayload
- `src/observation/src/event_bus.rs` — Pub/sub event bus (subscribe_all, subscribe_type)
- `src/observation/src/browser.rs` — Windows-only, EnumChildWindows + GetWindowTextW
- `src/observation/src/terminal.rs` — Windows/Mac/Linux terminal detection
- `src/observation/src/correlation.rs` — Links browser URL + terminal command
- `src/observation/src/intent_analyzer.rs` — Synthesizes UserIntent from structured data
- `src/observation/src/screen_capture.rs` — **STUB** — creates HDC but never saves pixels
- `src/observation/src/capture.rs` — **STUB** — `unimplemented!()`
- `src/observation/src/ocr.rs` — **STUB** — Tesseract placeholder
- `src/observation/src/ai_guidance.rs` — AI guidance module
- `src/observation/src/guidance.rs` — Guidance suggestion system

#### What needs to be built

**Phase 1: Real Screen Capture Provider**
- Implement `screen_capture.rs` to actually capture pixels and save as PNG/base64
- Store screenshots in a rotating buffer (last N captures, ~30 sec intervals)
- New `ObservationEvent` type: `ScreenshotCaptured` with base64 image data
- Provider registered with ObservationEngine
- Platform: Windows first (BitBlt → GDI Bitmap → PNG via libwebp or encode to PNG in Rust), then Linux (xdg-desktop-portal or X11)

**Phase 2: Vision Analysis Provider** ✅ COMPLETE (v1.1.77)
- Created `src/observation/src/vision_analyzer.rs` with full Vision AI integration
- Takes latest screenshot from engine buffer (via `feed_event()` → `feed_screenshot_to_vision_analyzer()`)
- Sends to Vision model (OpenRouter API, Claude Sonnet 4, GPT-4o, etc.)
- Returns structured analysis: "what's on screen", "what app", "what's wrong", "user intent"
- Emits `VisionAnalysisResult` event with structured JSON
- Configurable via settings (model, API key, endpoint, poll interval — default 30s)
- Rate limited: max 1 call per configured interval (default 30s)
- 6 unit tests passing
- Wired into engine via `feed_event()` → `ProviderType::ScreenCapture` handler
- Added `as_any()` downcasting to `ObservationProvider` trait
- Added `queue_screenshot()` external API for engine → provider communication

**Phase 3: Enhanced Intent Synthesis**
- Feed Vision analysis results into `IntentAnalyzer` alongside structured data
- Enhanced correlation: Vision says "Nagios with Database Error" + terminal says "systemctl status" = troubleshooting Nagios DB
- New activity categories from Vision: `VisualInsight`, `ErrorDetectedOnScreen`, `UserTypingInTerminal`

**Phase 4: Copilot Advisor Integration**
- Enhance `api_server.rs` AI loop to include Vision analysis in the system prompt
- Better system prompt that incorporates: structured data + visual understanding + correlation
- Proactive advice generation with confidence scores
- "Did the user run the right command?" checking against known best practices

### Technical Decisions

**Screen capture approach:**
- Windows: BitBlt → GDI Bitmap → encode PNG with `image` crate (simpler than libwebp)
- Store as base64 in JSON events (avoids filesystem I/O in polling loop)
- Max resolution: 1920x1080 (scale down from full screen for cost/latency)
- Color format: RGB24 (simpler processing)
- Screenshot buffer: last 5 captures in memory, LRU eviction

**Vision AI approach:**
- Use existing `OpenRouterCompatibleProvider` or new `VisionProvider` 
- Send screenshot as base64 data URL: `data:image/png;base64,...`
- Prompt template: "Analyze this screenshot. What app is in focus? What's the user doing? Are there any errors? What's the user's likely intent?"
- Rate limit: 1 vision call per 30 seconds max (not every poll)
- Only send Vision when significant state change detected

**Integration points:**
1. `screen_capture.rs` provider → emits `ScreenshotCaptured` events
2. `vision_analyzer.rs` provider → subscribes to screenshots, calls Vision API, emits `VisionAnalysis` events
3. `api_server.rs` loop → drains events from bus, includes Vision analysis in AI prompt
4. `intent_analyzer.rs` → consumes Vision analysis events to enhance understanding

### Files to Create/Modify

**Create:**
- `src/observation/src/screen_capture_real.rs` — actual screen capture implementation
- `src/observation/src/vision_analyzer.rs` — Vision AI analysis provider
- Update `src/observation/src/lib.rs` — register new modules

**Modify:**
- `src/observation/src/screen_capture.rs` — implement real capture logic
- `src/observation/src/capture.rs` — remove stub, implement
- `src/observation/src/ocr.rs` — can be a thin wrapper around Vision output (Vision already does OCR)
- `src/observation/src/engine.rs` — register new providers
- `src/observation/src/event.rs` — add new event types if needed
- `src-tauri/src/observation.rs` — register new providers
- `src-tauri/src/main.rs` — start new providers
- `src-tauri/src/api_server.rs` — include Vision analysis in AI prompt
- `src/observation/src/intent_analyzer.rs` — consume Vision events
- `src/observation/src/provider.rs` — add config fields for Vision provider

### Session Continuity
When context is lost, new sessions should:
1. Read this PLAN.md to understand project state
2. Check `~/.hermes/TODO.md` for current work state
3. Review `src/observation/IMPLEMENTATION_NOTES.md` for technical decisions made during implementation
4. Read `src/observation/src/engine.rs` to understand the provider registration flow
5. Read `src-tauri/src/observation.rs` to understand how providers are started

### Known Issues to Address
1. Terminal provider captures only last line of MobaXterm — needs enhancement
2. Browser provider's visible text collection may miss deep DOM elements
3. Vision API costs money — need rate limiting and smart triggering
4. Privacy concerns — screenshots may capture sensitive data
5. Performance — screen capture + Vision on every poll will slow down the app

### Next Steps (for new sessions)
1. Phase 3: Feed Vision analysis into IntentAnalyzer for enhanced intent synthesis
2. Wire Vision analysis results into api_server.rs AI prompt
3. Implement real screen capture (Phase 1) — BitBlt on Windows, xdg-desktop-portal on Linux
4. Add Vision to the main app config/settings UI
5. Consider privacy: blur/redact sensitive content before sending to Vision API