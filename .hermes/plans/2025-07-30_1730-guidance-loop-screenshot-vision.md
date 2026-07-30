# Guidance Loop: Direct Screenshot to LLM Implementation Plan

> **For Hermes:** This plan captures the work-in-progress from a compressed session. The implementation is partially done and needs verification/build.

**Goal:** Make the AI guidance loop send a live screenshot of the user's desktop directly to the guidance LLM, so it can give context-aware visual suggestions instead of relying only on text metadata.

**Architecture:** Add a `get_last_screenshot()` accessor to the screen capture pipeline, extend `AiMessage.content` to support multi-modal (text + image) JSON arrays, and modify the guidance loop in `api_server.rs` to fetch a screenshot and include it in the LLM request alongside the existing text context.

**Tech Stack:** Rust (Rust 2024 edition), Tauri v2, `wikilabs_observation` crate, `wikilabs_ai` provider, OpenAI-compatible API.

---

## Background: What We're Solving

The guidance loop sends text-only context (focused window, terminal commands, browser URL) to the LLM. It can't "see" what's actually on screen. The Vision Analyzer already captures screenshots and sends them to Claude, but the guidance loop doesn't use them. Fix: get a fresh screenshot and send it as a base64 image in the multi-modal request.

## Background: What Was Already Done (in previous session)

The following changes were written but NOT yet verified with `cargo check`:

### 1. `src/observation/src/screen_capture.rs`
Added `get_last_screenshot()` method to `ScreenCaptureProvider`:
```rust
pub fn get_last_screenshot(&self) -> Option<CapturedScreenshot> {
    self.state.lock().unwrap().last_screenshot.clone()
}
```
This returns the most recent `CapturedScreenshot` which contains `data_base64`, `width`, `height`, `focused_window`.

### 2. `src/observation/src/engine.rs`
Added `get_last_screenshot()` to `ObservationEngine` — iterates registry, downcasts to `ScreenCaptureProvider`, calls its getter.

### 3. `src/observation/src/lib.rs`
Added global `get_last_screenshot()` function delegating to the engine.

### 4. `src/ai/src/provider.rs`
Changed `AiMessage.content` from `String` to `serde_json::Value` to support multi-modal content. Updated response parsing to convert `String` to `Value` via `.into()`.

### 5. `src-tauri/src/api_server.rs`
Rewrote the guidance loop AI call section:
- Gets a screenshot via `observation::get_last_screenshot()`
- If screenshot exists: builds multi-modal user message with text + base64 image
- If no screenshot: falls back to text-only
- System prompt and response extraction also updated to handle `serde_json::Value`

## What Still Needs To Be Done

### Task 1: Verify Compilation
**Objective:** Run `cargo check` to confirm all changes compile cleanly.

Run:
```bash
cd ~/wikilabs-ai-copilot
cargo check 2>&1
```

Expected: May show false lint errors about "async fn in Rust 2015" — these are known false positives from the patch tool's pre-save lint check. **IGNORE them.** The actual Cargo.toml sets `edition = "2024"`. Focus on REAL errors only (undefined types, missing methods, type mismatches).

### Task 2: Fix Any Compilation Errors
If `cargo check` reveals real errors:
- `CapturedScreenshot` struct might have different field names than assumed — verify with `grep` on `screen_capture.rs`
- The `observation::get_last_screenshot()` function might not be in scope — verify the `observation` module exports it
- Any type mismatches in the response extraction code

### Task 3: Verify Vision Analyzer Still Works
**Objective:** Ensure the Vision Analyzer (which also sends screenshots) wasn't broken by the `AiMessage.content` change from `String` to `serde_json::Value`.

The Vision Analyzer builds its own raw HTTP request using `reqwest::Client` directly — it does NOT use `wikilabs_ai::provider`. So it should be unaffected. But verify:
```bash
cd ~/wikilabs-ai-copilot
cargo check -p wikilabs_observation 2>&1 | grep -i "error\[" | head -10
```

### Task 4: Run Tests
**Objective:** Run the full test suite.

Run:
```bash
cd ~/wikilabs-ai-copilot
cargo test 2>&1
```

### Task 5: Update the System Prompt (Optional Enhancement)
The system prompt could be updated to tell the LLM to look at the screenshot first before analyzing text context. This is optional — the current user message text already says "Here is a screenshot of what the user is looking at right now — use it to give precise, contextual guidance."

## Files Changed (Summary)

| File | Change |
|------|--------|
| `src/observation/src/screen_capture.rs` | Added `get_last_screenshot()` method |
| `src/observation/src/engine.rs` | Added `get_last_screenshot()` method |
| `src/observation/src/lib.rs` | Added `get_last_screenshot()` global |
| `src/ai/src/provider.rs` | `AiMessage.content: String` → `serde_json::Value`; response parsing updated |
| `src-tauri/src/api_server.rs` | Guidance loop: fetch screenshot, build multi-modal message, extract text from response |

## Key Data Structures

`CapturedScreenshot` (from `screen_capture.rs`):
- `data_base64: String` — full PNG base64
- `width: u32`
- `height: u32`
- `focused_window: String`

Multi-modal request format (OpenAI-compatible):
```json
{
  "model": "...",
  "messages": [{
    "role": "user",
    "content": [
      { "type": "text", "text": "Analyze this..." },
      { "type": "image_url", "image_url": { "url": "data:image/png;base64,..." } }
    ]
  }]
}
```

## Risks & Caveats

1. **Screenshot size:** Base64 PNG can be 100-500KB. For very high-res screenshots this could consume significant context window. Consider throttling screenshot frequency or downsizing in future.
2. **Performance:** Each guidance tick now captures a screenshot. The `get_last_screenshot()` returns the most recently captured one (not a live capture), so it's essentially free. No new screen capture is triggered.
3. **Vision Analyzer conflict:** Both the vision analyzer and guidance loop now request screenshots. The `get_last_screenshot()` returns the last captured one, so there's no conflict — it just reads the shared state buffer.