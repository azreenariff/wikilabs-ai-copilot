//! HTTP server that bridges frontend REST API calls to local state.
//!
//! The frontend SPA hardcodes calls to `http://localhost:1420/api/commands/*`.
//! This server intercepts those calls and serves local state + chat history.

use axum::{
    extract::{Path, State},
    extract::Json,
    http::StatusCode,
    response::{Html, Response},
    routing::{get, post},
    Router,
};
use tauri::Manager;
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::sync::{Arc, Mutex};
use wikilabs_observation;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{debug, error, info, warn};
use wikilabs_ai::AiProvider;
use tauri::AppHandle;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Read screenshot files from the screenshots directory and encode them as
/// base64 data URLs suitable for multi-modal AI providers (OpenAI-compatible).
/// Returns a Vec of data URL strings.
fn load_screenshot_images(screenshot_dir: &std::path::Path, filenames: &[String]) -> Vec<String> {
    let mut images = Vec::new();
    for filename in filenames {
        let filepath = screenshot_dir.join(filename);
        if let Ok(bytes) = fs::read(&filepath) {
            let data = BASE64.encode(&bytes);
            let ext = if filename.ends_with(".png") { "png" } else { "jpeg" };
            let data_url = format!("data:image/{};base64,{}", ext, data);
            images.push(data_url);
        }
    }
    images
}

/// Check if the user's message indicates they want to analyze screenshots.
/// This triggers multi-modal image loading to avoid unnecessary overhead
/// on normal chat messages.
fn user_requests_screenshot_analysis(message: &str) -> bool {
    let lower = message.to_lowercase();
    let keywords = [
        "screenshot", "screen shot", "screen capture", "analyze the screen",
        "advise me on", "look at the screenshot", "look at the screen",
        "check the screenshot", "analyze screenshot", "review the screenshot",
        "what do you see", "what's on the screen", "help me with",
        "advise me", "advise on", "look at this",
    ];
    keywords.iter().any(|&kw| lower.contains(kw))
}

/// Build a system prompt that includes the AI's own observation context,
/// recent recommendations with reasoning, and session state.
/// This makes the AI truly aware of what it has been observing and recommending.
async fn build_context_system_prompt(
    chat_history: &[ChatMessage],
) -> Option<String> {
    let panel = guidance_panel::GuidancePanel::instance();
    let mut parts: Vec<String> = Vec::new();

    // ── Recent observation events ──
    let now = chrono::Utc::now();
    let recent_events = panel.get_recent_events(60).await;
    if !recent_events.is_empty() {
        let mut lines = Vec::new();
        for e in recent_events.iter().take(30) {
            let desc = e.description.as_deref().unwrap_or("");
            let tech = e.technology.as_deref().unwrap_or(e.event_type.as_str());
            // Calculate how old this event is
            let age_label = chrono::DateTime::parse_from_rfc3339(&e.timestamp)
                .ok()
                .and_then(|dt| now.signed_duration_since(dt).to_std().ok())
                .map(|d| {
                    if d.as_secs() < 60 {
                        format!("({}s ago)", d.as_secs())
                    } else {
                        format!("({:.0}m ago)", d.as_secs_f64() / 60.0)
                    }
                })
                .unwrap_or_else(|| "(unknown age)".to_string());
            lines.push(format!("- {} on {}: {} {}", e.event_type, tech, desc, age_label));
        }
        if !lines.is_empty() {
            parts.push(format!(
                "## Recent Observations (last 60 minutes) — Data current as of {}\n{}\n",
                now.format("%H:%M:%S UTC"),
                lines.join("\n")
            ));
        }
    }

    // ── Recent recommendations with reasoning ──
    let all_recs = panel.all_recommendations().await;
    if !all_recs.is_empty() {
        let mut lines = Vec::new();
        for r in all_recs.iter().take(5) {
            let status_str = match r.status {
                guidance_panel::RecommendationStatus::Active => "🟢 active",
                guidance_panel::RecommendationStatus::Accepted => "✅ accepted",
                guidance_panel::RecommendationStatus::Rejected => "❌ rejected",
                guidance_panel::RecommendationStatus::Skipped => "⏭️ skipped",
                guidance_panel::RecommendationStatus::Dismissed => "❌ dismissed",
            };
            let next_step = r.recommended_next_step.as_deref().unwrap_or("");
            let conf = format!("{:.0}%", r.confidence * 100.0);
            lines.push(format!(
                "- **{}** ({}) — Reason: {} | Confidence: {} | Next: {}",
                r.title, status_str, r.reason, conf, next_step
            ));
        }
        if !lines.is_empty() {
            parts.push(format!(
                "## My Recent Recommendations\n{}\n",
                lines.join("\n")
            ));
        }
    }

    // ── Intent Summary (structured analysis from Observation Engine) ──
    // Phase 4: Include structured intent data from the Observation Engine
    // This provides a high-level synthesis of what the user is doing,
    // their inferred intent, detected issues, and suggested actions
    if let Some(intent) = crate::observation::get_last_intent() {
        let mut intent_lines = Vec::new();
        intent_lines.push(format!(
            "🎯 **Detected intent**: {} (confidence: {:.0}%)",
            intent.intent,
            intent.confidence * 100.0
        ));

        // Infrastructure targets
        if !intent.infrastructure_targets.is_empty() {
            intent_lines.push(format!(
                "🔗 **Systems involved**: {}",
                intent.infrastructure_targets.iter().take(5).map(|s| s.chars().take(60).collect::<String>()).collect::<Vec<_>>().join(", ")
            ));
        }

        // Suggested next steps from intent analysis
        if !intent.suggested_next_steps.is_empty() {
            for step in intent.suggested_next_steps.iter().take(3) {
                intent_lines.push(format!("📋 **Next step**: {}", step.chars().take(150).collect::<String>()));
            }
        }

        // Goal
        if let Some(ref goal) = intent.goal {
            intent_lines.push(format!("🎯 **Goal**: {}", goal.chars().take(200).collect::<String>()));
        }

        parts.push(format!(
            "## Intent Summary (AI-driven analysis)\n{}\n",
            intent_lines.join("\n")
        ));
    }

    // ── Screen Vision Analysis ──
    // Phase 4: Include the latest Vision analysis result in the AI prompt
    if let Some(vision_result) = crate::observation::get_vision_result() {
        let mut vision_lines = Vec::new();
        let conf = format!("{:.0}%", vision_result.confidence * 100.0);
        if let Some(ref intent) = vision_result.inferred_intent {
            vision_lines.push(format!(
                "🔍 **AI Vision detected**: You appear to be {} (confidence: {})",
                intent, conf
            ));
        }
        if !vision_result.errors_detected.is_empty() {
            for err in &vision_result.errors_detected {
                vision_lines.push(format!("⚠️ **Screen error detected**: {} ({})", err.description, err.severity));
            }
        }
        if !vision_result.suggestions.is_empty() {
            for sug in &vision_result.suggestions {
                vision_lines.push(format!("💡 **AI Vision suggests**: {}", sug));
            }
        }
        // Include focused app context
        if let Some(ref app) = vision_result.focused_app {
            vision_lines.push(format!("📺 **Active app**: {}", app));
        }
        if !vision_lines.is_empty() {
            parts.push(format!(
                "## Screen Vision Analysis (AI-powered)\n{}\n",
                vision_lines.join("\n")
            ));
        }
    }

    // If we have no context, return None (no system prompt needed)
    if parts.is_empty() {
        return None;
    }

    let context_block = parts.join("\n");

    // Build a system prompt that frames the AI's identity as an observer
    let _history_preview = if !chat_history.is_empty() {
        format!(
            "\n## Conversation History (recent)\n{}",
            chat_history
                .iter()
                .rev()
                .take(10)
                .map(|m| {
                    format!(
                        "- {}: {}",
                        if m.role == "user" { "User" } else { "Assistant" },
                        m.content.chars().take(200).collect::<String>()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        String::new()
    };

    let session_context = if !chat_history.is_empty() {
            // If there's conversation history, the AI should continue the conversation
            // as the same agent that did the observations
            format!(
                "{}\n\n## CORE RULES — DO NOT IGNORE THESE\n\n1. **Use only the data above.** The \"Recent Observations\", \"Intent Summary\", and \"Screen Vision Analysis\" sections contain REAL data you have collected. Base ALL your responses on that data.\n\n2. **Do NOT invent or hallucinate.** If the data above does not contain information about a specific file, command output, error message, or system state, do NOT make it up. Say \"I don't have that information\" instead of guessing.\n\n3. **If the observations section is empty, say so explicitly.** Do not pretend you are watching the user's screen if no observations have been recorded yet.\n\n4. **Connect the dots between what you actually see.** If you see a terminal command AND a browser error AND an application change, weave them together into a coherent analysis.\n\n5. **Be conversational and specific.** Talk like a teammate who is watching work happen. Say \"I see you're running...\" not \"The user may be...\"\n\nYou are the Wiki Labs AI Copilot. You've been actively observing the user's activity in real-time through terminal sessions, browser activity, window changes, and more. The observations and recommendations above are YOUR own — they're what you've noticed and suggested based on what you've seen. Continue your analysis and conversation with full awareness of your prior observations.",
                context_block
            )
        } else {
            // No conversation history yet — just set the context
            format!(
                "{}\n\n## CORE RULES — DO NOT IGNORE THESE\n\n1. **Use only the data above.** The \"Recent Observations\", \"Intent Summary\", and \"Screen Vision Analysis\" sections contain REAL data you have collected. Base ALL your responses on that data.\n\n2. **Do NOT invent or hallucinate.** If the data above does not contain information about a specific file, command output, error message, or system state, do NOT make it up. Say \"I don't have that information\" instead of guessing.\n\n3. **If the observations section is empty, say so explicitly.** Do not pretend you are watching the user's screen if no observations have been recorded yet.\n\n4. **Be conversational and specific.** Talk like a teammate who is watching work happen.\n\nYou are the Wiki Labs AI Copilot assistant. You've been actively observing the user's activity in real-time. The observations and recommendations above are YOUR own — what you've noticed and suggested based on what you've seen.",
                context_block
            )
        };

    Some(session_context)
}

use crate::guidance_loop;
use crate::guidance_panel;
use crate::knowledge_panel::KnowledgePanel;
use crate::skill_knowledge::create_skill_knowledge_base;
use crate::skill_management::SkillManagementPanel;

/// Request wrapper sent from the frontend.
#[derive(Debug, Deserialize, Clone)]
pub struct ApiRequest {
    #[serde(default)]
    pub params: Value,
}

/// Shared state for the HTTP server.
#[derive(Clone)]
pub struct ApiServerState {
    pub settings: Arc<Mutex<ApiServerSettings>>,
    pub config_path: Arc<Mutex<Option<PathBuf>>>,
    /// The knowledge packs directory used at startup (for preflight checks).
    pub knowledge_dir: Arc<Mutex<Option<PathBuf>>>,
    /// Optional AppHandle for sending native notifications.
    /// Wrapped in Arc<Mutex<...>> to ensure Send + Sync for axum Handler bounds.
    pub app_handle: Option<Arc<Mutex<tauri::AppHandle>>>,
}

#[derive(Debug, Clone)]
pub struct ApiServerSettings {
    pub settings: Value,
    pub providers: Vec<Value>,
    pub messages: Arc<Mutex<Vec<ChatMessage>>>,
    pub workspaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub workspace_id: Option<String>,
}

/// Response wrapper sent to the frontend.
pub fn api_response(success: bool, value: Option<Value>, error: Option<String>) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("success".to_string(), Value::from(success));
    obj.insert("value".to_string(), value.unwrap_or(Value::Null));
    if let Some(e) = error {
        obj.insert("error".to_string(), Value::from(e));
    }
    serde_json::to_string(&Value::Object(obj)).unwrap()
}

impl ApiServerSettings {
    pub fn new() -> Self {
        Self {
            settings: serde_json::json!({
                "ai_provider": {
                    "name": "openai",
                    "endpoint": "https://api.openai.com/v1",
                    "api_key": "",
                    "model": "gpt-4o",
                    "max_tokens": 4096,
                    "context_window": 128000
                },
                "theme": "dark",
                "log_level": "info",
                "privacy_mode": false
            }),
            providers: vec![
                serde_json::json!({
                    "name": "OpenAI",
                    "url": "https://api.openai.com/v1",
                    "api_version": "v1"
                }),
                serde_json::json!({
                    "name": "OpenRouter",
                    "url": "https://openrouter.ai/api/v1",
                    "api_version": "v1"
                }),
                serde_json::json!({
                    "name": "Custom Endpoint",
                    "url": "http://localhost:8000/v1",
                    "api_version": "v1"
                }),
                serde_json::json!({
                    "name": "Ollama",
                    "url": "http://localhost:11434/v1",
                    "api_version": "v1"
                }),
            ],
            messages: Arc::new(Mutex::new(Vec::new())),
            workspaces: vec!["default".to_string()],
        }
    }
}

/// Main request handler.
#[axum::debug_handler]
pub async fn api_handler(
    State(state): State<ApiServerState>,
    Path(method): Path<String>,
    Json(req): Json<ApiRequest>,
) -> (StatusCode, String) {
    info!(method, "API request received");

    let (status, body) = match method.as_str() {
        "get_settings" => handle_get_settings(&state).await,
        "update_settings" => handle_update_settings(&state, req.params).await,
        "set_first_run_complete" => handle_set_first_run_complete(&state).await,
        "test_connection" => handle_test_connection(&state, req.params).await,
        "send_message" => handle_send_message(&state, req.params),
        "get_history" => handle_get_history(&state),
        "list_providers" => handle_list_providers(&state),
        "list_models" => handle_list_models(&state, req.params).await,
        // Workspace commands
        "get_workspace_list" => handle_get_workspace_list(&state),
        "create_workspace" => handle_create_workspace(&state, req.params),
        // Skill commands
        "skill_list" => handle_skill_list(),
        "skill_get" => handle_skill_get(req.params),
        "skill_enable" => handle_skill_enable(req.params),
        "skill_disable" => handle_skill_disable(req.params),
        "skill_toggle" => handle_skill_toggle(req.params),
        "skill_validate" => handle_skill_validate(req.params),
        "skill_mark_validated" => handle_skill_mark_validated(req.params),
        "skill_set_active" => handle_skill_set_active(req.params),
        // Knowledge pack commands
        "knowledge_list_packs" => handle_knowledge_list_packs().await,
        "knowledge_enable_pack" => handle_knowledge_enable_pack(req.params).await,
        "knowledge_disable_pack" => handle_knowledge_disable_pack(req.params).await,
        "knowledge_reindex_pack" => handle_knowledge_reindex_pack(req.params).await,
        "knowledge_get_validation_report" => handle_knowledge_get_validation_report(req.params).await,
        "knowledge_get_pack_metadata" => handle_knowledge_get_metadata(req.params).await,
        "knowledge_export_pack" => handle_knowledge_export_pack(req.params).await,
        "knowledge_import_pack" => handle_knowledge_import_pack(req.params).await,
        // Guidance commands
        "guidance_get_active_recommendations" => handle_guidance_get_active_recommendations().await,
        "guidance_get_all_recommendations" => handle_guidance_get_all_recommendations().await,
        "guidance_dismiss_recommendation" => handle_guidance_dismiss_recommendation(req.params).await,
        "guidance_update_recommendation_status" => handle_guidance_update_recommendation_status(req.params).await,
        "guidance_get_evidence_status" => handle_guidance_get_evidence_status().await,
        "guidance_add_evidence" => handle_guidance_add_evidence(req.params).await,
        "guidance_mark_missing" => handle_guidance_mark_missing(req.params).await,
        "guidance_get_workflow_progress" => handle_guidance_get_workflow_progress().await,
        "guidance_start_workflow" => handle_guidance_start_workflow(req.params).await,
        "guidance_complete_step" => handle_guidance_complete_step(req.params).await,
        "guidance_get_timeline" => handle_guidance_get_timeline().await,
        "guidance_add_timeline_event" => handle_guidance_add_timeline_event(req.params).await,
        "guidance_get_recent_events" => handle_guidance_get_recent_events(req.params).await,
        "guidance_record_feedback" => handle_guidance_record_feedback(req.params).await,
        "guidance_get_feedback_stats" => handle_guidance_get_feedback_stats().await,
        "guidance_clear_all" => handle_guidance_clear_all().await,
        "guidance_show_toast" => {
            let title = req.params.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let body = req.params.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string();
            handle_guidance_show_toast(&title, &body).await
        },
        // Pre-flight checks
        "preflight_check" => handle_preflight_check(&state, req.params),
        // System commands
        "get_status" => handle_get_status().await,
        "observation_get_status" => handle_observation_get_status(&state).await,
        "observation_get_context" => handle_observation_get_context().await,
        "observation_start" => handle_observation_start(&state).await,
        "observation_stop" => handle_observation_stop(&state).await,
        "capture_screenshot" => handle_capture_screenshot(&state).await,
        "clear_screenshots" => handle_clear_screenshots().await,
        "hide_main_window" => handle_hide_main_window(&state).await,
        "advice_chat_open" => handle_advice_chat_open(&state).await,
        "hide_advice_window" => handle_hide_advice_window(&state).await,
        "drag_advice_window" => handle_drag_advice_window(&state, req.params).await,
        "restart" => handle_restart_api_server(&state).await,
        other => {
            warn!(other, "Unknown API method");
            (StatusCode::BAD_REQUEST, api_response(false, None, Some(format!("Unknown method: {}", other))))
        }
    };

    info!(method, "API request completed");
    (status, body)
}

async fn handle_test_connection(_state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let start_time = std::time::Instant::now();
    let api_key = params.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let endpoint = params.get("endpoint").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let model = params.get("model").and_then(|v| v.as_str()).unwrap_or("gpt-4o");
    let max_tokens = params.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(4096);
    let context_window = params.get("context_window").and_then(|v| v.as_u64()).unwrap_or(128000);

    debug!("[TEST] >>> test_connection called — endpoint: '{}', api_key_set: {}, model: '{}', max_tokens: {}, context_window: {}",
        endpoint, !api_key.is_empty(), model, max_tokens, context_window);
    info!("[TEST] test_connection request received");

    if api_key.is_empty() {
        warn!("[TEST] API key is empty — rejecting");
        return (StatusCode::OK, api_response(false, None, Some("API key is required".to_string())));
    }
    if endpoint.is_empty() {
        warn!("[TEST] Endpoint is empty — rejecting");
        return (StatusCode::OK, api_response(false, None, Some("Endpoint is required".to_string())));
    }

    // Parse and validate the endpoint URL
    let url_for_tcp = endpoint.trim_end_matches('/');
    let host_port = if let Some(rest) = url_for_tcp.strip_prefix("https://") {
        debug!("[TEST] Detected HTTPS scheme");
        rest.split('/').next().unwrap_or(rest)
    } else if let Some(rest) = url_for_tcp.strip_prefix("http://") {
        debug!("[TEST] Detected HTTP scheme");
        rest.split('/').next().unwrap_or(rest)
    } else {
        warn!("[TEST] No scheme detected in endpoint — defaulting to HTTP");
        url_for_tcp.split('/').next().unwrap_or(url_for_tcp)
    };

    let (host, port) = if let Some(idx) = host_port.rfind(':') {
        let h = &host_port[..idx];
        let p: u16 = host_port[idx+1..].parse().unwrap_or(80);
        (h.to_string(), p)
    } else {
        // Default port based on scheme
        let default_port = if endpoint.starts_with("https://") { 443 } else { 80 };
        (host_port.to_string(), default_port)
    };

    debug!("[TEST] Parsed host='{}' port={} from endpoint '{}'", host, port, endpoint);
    info!("[TEST] Step 1/2: TCP check — connecting to {}:{} (timeout: 10s)", host, port);

    // Step 1: Try a simple TCP connection to see if the host is reachable at all.
    // This bypasses HTTP entirely and tells us if the server is up.
    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        tokio::net::TcpStream::connect((host.as_str(), port))
    ).await {
        Ok(Ok(_stream)) => {
            info!("[TEST] TCP connection succeeded — {}:{} is reachable", host, port);
            _stream
        }
        Ok(Err(e)) => {
            error!("[TEST] TCP connection FAILED to {}:{} — error: {} (type: {})", host, port, e, e.kind());
            let msg = if e.to_string().contains("timed out") || e.to_string().contains("deadline") {
                format!("Cannot reach {}:{} — connection timed out after 10s. Check the IP address and ensure the LLM server is running and accessible from this machine.", host, port)
            } else if e.to_string().contains("refused") {
                format!("Connection refused at {}:{}. The LLM server may not be running or the port is wrong.", host, port)
            } else if e.to_string().contains("Temporary failure in name resolution") || e.to_string().contains("not found") {
                format!("Cannot resolve hostname '{}'. Check that the IP address or hostname is correct.", host)
            } else {
                format!("Cannot connect to {}:{} — {}: {}", host, port, e, e.kind())
            };
            warn!("[TEST] Returning early (TCP failed): {}", msg);
            return (StatusCode::OK, api_response(false, None, Some(msg)));
        }
        Err(_) => {
            error!("[TEST] TCP connection TOOK >10s to {}:{} — aborting", host, port);
            let msg = format!("Cannot reach {}:{} — connection timed out after 10s. The host may be unreachable from this machine (check firewall, routing, or VPN settings).", host, port);
            warn!("[TEST] Returning early (TCP timeout): {}", msg);
            return (StatusCode::OK, api_response(false, None, Some(msg)));
        }
    };

    // Step 2: TCP worked — now try the HTTP health check
    info!("[TEST] Step 2/2: HTTP health check — GET {}/models (timeout: 15s)", endpoint);

    let provider = wikilabs_ai::provider::OpenAICompatibleProvider::new(
        "custom", &endpoint, &api_key, "gpt-4o", 4096, 128000,
    );

    match provider.health().await {
        Ok(_) => {
            let elapsed = start_time.elapsed();
            info!("[TEST] test_connection SUCCESS — total elapsed: {:.2}s", elapsed.as_secs_f64());
            (StatusCode::OK, api_response(true, Some(serde_json::json!(true)), None))
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            error!("[TEST] test_connection FAILED — HTTP health check error after {:.2}s: {}", elapsed.as_secs_f64(), e);
            (StatusCode::OK, api_response(false, None, Some(format!(
                "TCP connection succeeded but HTTP health check failed after {:.2}s: {}",
                elapsed.as_secs_f64(), e
            ))))
        }
    }
}

/// Restart handler: signals the frontend that the API server needs a restart.
/// The frontend will call this and then re-check readiness.
async fn handle_restart_api_server(state: &ApiServerState) -> (StatusCode, String) {
    info!("[API] Restart requested — signaling frontend");
    // The server can't restart itself (it's already running).
    // Instead, signal the frontend to show the loading screen again and re-poll.
    // In a full restart scenario, the Tauri app would need to restart its process.
    (StatusCode::OK, api_response(true, None, Some("Restart initiated. Please refresh the window.".to_string())))
}

/// Pre-flight check handler: verifies API server health and optionally tests AI provider.
/// Returns structured results for frontend checklist UI.
fn handle_preflight_check(state: &ApiServerState, req_params: Value) -> (StatusCode, String) {
    let test_provider = req_params.get("test_provider")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut checks = serde_json::Map::new();
    let mut all_ok = true;

    // Check 1: API server process is running (we are it, so always pass)
    // Note: The frontend performs an actual HTTP /health check and shows the
    // result as "API Server" with the response body. This backend check is
    // kept as a fallback for non-HTTP invocations (e.g. embedded mode).
    checks.insert("api_server_running".to_string(), serde_json::json!({
        "status": "pass",
        "label": "API Server",
        "detail": "Running"
    }));

    // Check 2: /ready endpoint is responding
    let ready = crate::api_ready::is_server_ready();
    if ready {
        checks.insert("ready_endpoint".to_string(), serde_json::json!({
            "status": "pass",
            "label": "Server Ready",
            "detail": "Fully initialized"
        }));
    } else {
        all_ok = false;
        checks.insert("ready_endpoint".to_string(), serde_json::json!({
            "status": "fail",
            "label": "Server Ready",
            "detail": "Still initializing..."
        }));
    }

    // Check 3: Settings loaded from disk
    let settings_val = {
        let settings_guard = state.settings.lock().unwrap();
        settings_guard.settings.clone()
    };
    if let Ok(config_path) = state.config_path.lock() {
        if let Some(ref path) = *config_path {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(parsed) = serde_json::from_str::<Value>(&content) {
                    {
                        let mut settings_guard = state.settings.lock().unwrap();
                        settings_guard.settings = parsed.clone();
                    }
                    checks.insert("settings_loaded".to_string(), serde_json::json!({
                        "status": "pass",
                        "label": "Settings",
                        "detail": format!("Loaded from {}", path.display())
                    }));
                } else {
                    all_ok = false;
                    checks.insert("settings_loaded".to_string(), serde_json::json!({
                        "status": "fail",
                        "label": "Settings",
                        "detail": "Invalid JSON"
                    }));
                }
            } else {
                // No settings file yet — OK for first run
                checks.insert("settings_loaded".to_string(), serde_json::json!({
                    "status": "skip",
                    "label": "Settings",
                    "detail": "No config yet (first run)"
                }));
            }
        } else {
            checks.insert("settings_loaded".to_string(), serde_json::json!({
                "status": "skip",
                "label": "Settings",
                "detail": "No config path set"
            }));
        }
    }

    // Check 4: AI provider connection (optional) — reuse existing handle_test_connection logic
    if test_provider {
        if let Some(ai_provider) = settings_val.get("ai_provider") {
            if let Some(ai_obj) = ai_provider.as_object() {
                let api_key = ai_obj.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let endpoint = ai_obj.get("endpoint").and_then(|v| v.as_str()).unwrap_or("").to_string();

                if api_key.is_empty() || endpoint.is_empty() {
                    checks.insert("ai_provider".to_string(), serde_json::json!({
                        "status": "skip",
                        "label": "AI Provider",
                        "detail": "Not configured"
                    }));
                } else {
                                    // Block on the async test connection synchronously — must use
                                    // std::thread::spawn with a new tokio runtime because
                                    // Handle::current().block_on() panics inside axum's tokio runtime.
                                    // Clone the Arc handles (cheap, 'static) to move into the thread.
                                    let state_clone = ApiServerState {
                                        settings: state.settings.clone(),
                                        config_path: state.config_path.clone(),
                                        knowledge_dir: state.knowledge_dir.clone(),
                                        app_handle: None,
                                    };
                                    let spawn_handle = std::thread::spawn(move || {
                                        let rt = tokio::runtime::Runtime::new()
                                            .expect("Failed to create test runtime");
                                        rt.block_on(handle_test_connection(&state_clone, serde_json::json!({
                                            "api_key": api_key,
                                            "endpoint": endpoint
                                        })))
                                    });

                                    match spawn_handle.join() {
                                        Ok(test_result) => {
                                            if test_result.0 == StatusCode::OK {
                                                if let Ok(parsed_body) = serde_json::from_str::<Value>(&test_result.1) {
                                                    if let Some(val) = parsed_body.get("value") {
                                                        if val.as_bool() == Some(true) {
                                                            checks.insert("ai_provider".to_string(), serde_json::json!({
                                                                "status": "pass",
                                                                "label": "AI Provider",
                                                                "detail": "Connection successful"
                                                            }));
                                                        } else {
                                                            all_ok = false;
                                                            let detail = parsed_body.get("error")
                                                                .and_then(|e| e.as_str())
                                                                .unwrap_or("Connection test failed")
                                                                .to_string();
                                                            checks.insert("ai_provider".to_string(), serde_json::json!({
                                                                "status": "fail",
                                                                "label": "AI Provider",
                                                                "detail": detail
                                                            }));
                                                        }
                                                    }
                                                }
                                            } else {
                                                all_ok = false;
                                                checks.insert("ai_provider".to_string(), serde_json::json!({
                                                    "status": "fail",
                                                    "label": "AI Provider",
                                                    "detail": test_result.1
                                                }));
                                            }
                                        }
                                        Err(e) => {
                                            all_ok = false;
                                            checks.insert("ai_provider".to_string(), serde_json::json!({
                                                "status": "fail",
                                                "label": "AI Provider",
                                                "detail": format!("Thread panicked: {:?}", e)
                                            }));
                                        }
                                    }
                }
            }
        }
    }

    // Check 5: Knowledge packs loaded
    let kp_status = match state.knowledge_dir.lock().unwrap().as_ref() {
        Some(kdir) => {
            match std::fs::read_dir(kdir) {
                Ok(entries) => {
                    let mut pack_count = 0;
                    for entry in entries.flatten() {
                        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                            if ext == "yaml" || ext == "yml" || ext == "json" {
                                // Check if it's a manifest (not any yaml/json file)
                                let file_name = entry.file_name();
                                if file_name == "manifest.yaml" || file_name == "manifest.yml" || file_name == "pack.json" {
                                    pack_count += 1;
                                }
                            }
                        }
                    }
                    if pack_count > 0 {
                        (true, format!("{} pack(s) loaded from {}", pack_count, kdir.display()))
                    } else {
                        (false, "No packs found".to_string())
                    }
                }
                Err(_) => (false, format!("Cannot read knowledge directory: {}", kdir.display())),
            }
        }
        None => (false, "Not configured".to_string()),
    };
    checks.insert("knowledge_packs_loaded".to_string(), serde_json::json!({
        "status": if kp_status.0 { "pass" } else { "skip" },
        "label": "Knowledge Packs",
        "detail": kp_status.1
    }));

    (StatusCode::OK, api_response(all_ok, Some(Value::Object(checks)), None))
}

// ── Screenshot Utilities ─────────────────────────────────────

/// Get the screenshots directory path (cross-platform).
/// Saves to: ~/Documents/AI Copilot Screenshots/
pub fn get_screenshots_dir() -> PathBuf {
    // Windows: use USERPROFILE, Unix: use HOME. Avoid "~" because
    // it is NOT expanded automatically by PathBuf on Windows.
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| {
            std::env::var("HOME").unwrap_or_else(|_| "C:\\Users\\User".to_string())
        })
    } else {
        std::env::var("HOME").unwrap_or_else(|_| "~".to_string())
    };
    PathBuf::from(home).join("Documents").join("AI Copilot Screenshots")
}

/// Show a toast notification for screenshot capture.
fn show_screenshot_toast(title: &str, body: &str) {
    if let Some(ref app_handle) = get_shared_app_handle() {
        let _ = app_handle.emit("screenshot-captured", serde_json::json!({
            "title": title,
            "body": body,
        }));
    }
}

async fn handle_get_settings(state: &ApiServerState) -> (StatusCode, String) {
    let mut settings = state.settings.lock().unwrap();
    
    // Load from disk on each get to stay in sync
    if let Ok(config_path) = state.config_path.lock() {
        if let Some(ref path) = *config_path {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(parsed) = serde_json::from_str::<Value>(&content) {
                    settings.settings = parsed;
                }
            }
        }
    }
    
    (StatusCode::OK, api_response(true, Some(settings.settings.clone()), None))
}

async fn handle_update_settings(state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let mut settings = state.settings.lock().unwrap();

    // Merge incoming params into existing settings instead of replacing entirely.
    // This preserves theme, log_level, privacy_mode, first_run_complete, etc.
    let mut merged = if let Some(existing) = settings.settings.as_object() {
        existing.clone()
    } else {
        serde_json::Map::new()
    };
    if let Some(params_obj) = params.as_object() {
        for (key, value) in params_obj {
            merged.insert(key.clone(), value.clone());
        }
    }
    settings.settings = serde_json::Value::Object(merged);

    // Update AI connection status based on whether api_key is configured
    if let Some(ai_provider) = settings.settings.get("ai_provider") {
        if let Some(ai_obj) = ai_provider.as_object() {
            let api_key = ai_obj.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
            let status = if api_key.is_empty() { "not_configured" } else { "connected" };
            let mut new_obj = ai_obj.clone();
            new_obj.insert("ai_connection_status".to_string(), serde_json::json!(status));
            settings.settings["ai_provider"] = serde_json::json!(new_obj);
        }
    }

    // If observation engine toggle changed, start/stop the polling loop
    let new_obs_enabled = settings.settings.get("observation_engine_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let new_guidance_enabled = settings.settings.get("guidance_loop_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if new_obs_enabled {
        info!("Observation engine enabled via settings toggle — starting");
        // Spawn lazy init — the observation engine polls settings, so once
        // observation_engine_enabled is true in settings, the engine knows to run.
        let settings_clone = state.settings.clone();
        tokio::spawn(async move {
            let started = crate::observation::lazy_start_observation_engine().await;
            if started {
                tracing::info!("Observation engine started (settings toggle)");
            } else {
                tracing::error!("Failed to start observation engine (settings toggle)");
            }
            // Persist the enabled flag again in case the engine read it before we wrote
            {
                let mut s = settings_clone.lock().unwrap();
                s.settings["observation_engine_enabled"] = serde_json::json!(true);
            }
        });
    } else if !new_obs_enabled {
        // Observation engine disabled via settings toggle — stop it
        info!("Observation engine disabled via settings toggle — stopping");
        let settings_clone = state.settings.clone();
        tokio::spawn(async move {
            // Persist the disabled flag
            {
                let mut s = settings_clone.lock().unwrap();
                s.settings["observation_engine_enabled"] = serde_json::json!(false);
            }
            // Stop any existing observation engine
            crate::observation::stop_observation_engine().await;
            tracing::info!("Observation engine stopped (settings toggle)");
        });
    }
    if new_guidance_enabled {
        info!("AI Guidance enabled via settings toggle — spawning loop");
        let poll_settings = state.settings.clone();
        let knowledge_dir = {
            let guard = state.knowledge_dir.lock().unwrap();
            guard.clone()
        };
        let skill_kb: crate::skill_knowledge::SkillKnowledgeBaseArc = if let Some(ref dir) = knowledge_dir {
            crate::skill_knowledge::create_skill_knowledge_base(&dir.to_string_lossy())
        } else {
            crate::skill_knowledge::create_skill_knowledge_base("")
        };
        guidance_loop::spawn_ai_guidance_loop(
            poll_settings,
            skill_kb,
            None,
        );
        tracing::info!("AI Guidance loop spawned (settings toggle)");
    }

    // Persist to disk
    if let Ok(config_path) = state.config_path.lock() {
        if let Some(ref path) = *config_path {
            match fs::write(path, serde_json::to_string_pretty(&settings.settings).unwrap_or_default()) {
                Ok(_) => info!("Settings persisted to disk: {}", path.display()),
                Err(e) => error!(error = %e, "Failed to persist settings to disk"),
            }
        }
    }
    
    (StatusCode::OK, api_response(true, Some(serde_json::json!({ "status": "updated" })), None))
}

async fn handle_set_first_run_complete(state: &ApiServerState) -> (StatusCode, String) {
    // Drop the settings guard before the async block to ensure MutexGuard is not held across .await
    {
        let mut settings = state.settings.lock().unwrap();
        // Set first_run_complete in the settings object
        settings.settings["first_run_complete"] = serde_json::json!(true);
        
        // Persist to disk
        if let Ok(config_path) = state.config_path.lock() {
            if let Some(ref path) = *config_path {
                match fs::write(path, serde_json::to_string_pretty(&settings.settings).unwrap_or_default()) {
                    Ok(_) => info!("first_run_complete persisted to disk"),
                    Err(e) => error!(error = %e, "Failed to persist first_run_complete"),
                }
            }
        }
    }  // MutexGuard dropped here
    
    // Start observation engine and guidance loop NOW that the wizard is complete,
    // but ONLY if the user has them enabled in settings.
    let obs_enabled = {
        let settings = state.settings.lock().unwrap();
        settings.settings.get("observation_engine_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };
    let guidance_enabled = {
        let settings = state.settings.lock().unwrap();
        settings.settings.get("guidance_loop_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };

    if obs_enabled || guidance_enabled {
        let app_handle_clone = state.app_handle.clone();
        let poll_settings = state.settings.clone();
        let knowledge_dir = {
            let guard = state.knowledge_dir.lock().unwrap();
            guard.clone()
        };

        if obs_enabled {
            // Create skill knowledge base from the knowledge directory
            let skill_kb: crate::skill_knowledge::SkillKnowledgeBaseArc = if let Some(ref dir) = knowledge_dir {
                crate::skill_knowledge::create_skill_knowledge_base(&dir.to_string_lossy())
            } else {
                crate::skill_knowledge::create_skill_knowledge_base("")
            };

            // Initialize observation engine directly on the existing tokio runtime
            // (axum's async handler chain). Do NOT create a nested runtime — that
            // causes "Cannot start a runtime from within a runtime" panic.
            info!("[LAZY] Initializing observation engine (post-wizard, enabled by settings) — synchronously");
            let engine = crate::observation::init_observation_engine().await;
            // Start providers — spawns the polling loop task on the existing runtime.
            crate::observation::start_observation_engine(engine).await;
            info!("[LAZY] Observation engine initialized and started (post-wizard)");

            // Now spawn guidance loop — the globals are already set above
            if let Some(ref ah) = app_handle_clone {
                let app_h = ah.lock().unwrap();
                guidance_loop::spawn_ai_guidance_loop(
                    poll_settings,
                    skill_kb,
                    Some(Arc::new((*app_h).clone())),
                );
                info!("[LAZY] Guidance loop spawned (post-wizard)");
            }
        }
    }
    
    (StatusCode::OK, api_response(true, Some(serde_json::json!({ "first_run_complete": true })), None))
}

fn handle_send_message(state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let workspace_id = params.get("workspace_id").and_then(|v| v.as_str()).unwrap_or("default");
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    {
        let settings_ref = state.settings.lock().unwrap();
        let mut msgs = settings_ref.messages.lock().unwrap();
        msgs.push(ChatMessage { id: id.clone(), role: "user".to_string(), content: message.clone(), created_at: created_at.clone(), workspace_id: Some(workspace_id.to_string()) });
        // Cap chat history to prevent unbounded growth — keep last 50 messages
        if msgs.len() > 50 {
            let dropped = msgs.len() - 50;
            msgs.drain(..dropped);
        }
    }

    // Try to get AI response, fall back to echo if provider not configured
    let settings = state.settings.lock().unwrap();
    let config = settings.settings.clone();
    drop(settings);

    let api_key = config.get("ai_provider")
        .and_then(|p| p.get("api_key"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Scan for pending manual screenshots BEFORE the if-else scope
    let screenshot_dir = get_screenshots_dir();
    let screenshot_files: Vec<String> = {
        let mut files: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir(&screenshot_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Ok(filename) = entry.file_name().into_string() {
                            if filename.starts_with("screenshot_") && (filename.ends_with(".jpg") || filename.ends_with(".png")) {
                                files.push(filename);
                            }
                        }
                    }
                }
            }
        }
        files.sort();
        files
    };

    // Load screenshot images as base64 data URLs for multi-modal AI — only when
    // the user explicitly asks for screenshot analysis (to avoid unnecessary overhead
    // on normal chat messages).
    let load_screenshots = user_requests_screenshot_analysis(&message);
    let screenshot_images = if load_screenshots && !screenshot_files.is_empty() {
        let images = load_screenshot_images(&screenshot_dir, &screenshot_files);
        if !images.is_empty() {
            info!(count = images.len(), "Loaded screenshot images for multi-modal AI analysis");
        }
        images
    } else {
        Vec::new()
    };

    if !load_screenshots && !screenshot_files.is_empty() {
        debug!("Skipping screenshot loading — user did not request analysis");
    }

    let (assistant_id, assistant_created, response) = if !api_key.is_empty() {
        let model = config.get("ai_provider")
            .and_then(|p| p.get("model"))
            .and_then(|v| v.as_str())
            .unwrap_or("gpt-4o")
            .to_string();
        let endpoint = config.get("ai_provider")
            .and_then(|p| p.get("endpoint"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let provider_name = config.get("ai_provider")
            .and_then(|p| p.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("openai")
            .to_string();
        let max_tokens = config.get("ai_provider")
            .and_then(|p| p.get("max_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(4096) as u32;
        let context_window = config.get("ai_provider")
            .and_then(|p| p.get("context_window"))
            .and_then(|v| v.as_u64())
            .unwrap_or(128000) as u32;

        let provider = wikilabs_ai::provider::OpenAICompatibleProvider::new(
            &provider_name,
            &endpoint,
            &api_key,
            &model,
            max_tokens as usize,
            context_window as usize,
        );

        // Build context-aware message array
        // 1. Get chat history (already saved user message above)
        let history = {
            let settings_ref = state.settings.lock().unwrap();
            let msgs = settings_ref.messages.lock().unwrap();
            msgs.clone()
        };

        // 3. Build system prompt with observation context + recommendations + screenshots (screenshot_files already scanned above)
        // Spawn a dedicated thread with its own tokio runtime to avoid
        // "Cannot start a runtime from within a runtime" panic.
        let system_prompt = {
            let history2 = history.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create runtime for system prompt");
                let prompt = rt.block_on(build_context_system_prompt(&history2));
                let _ = tx.send(prompt);
            });
            // Wait for the spawned thread to complete
            let mut result: Option<String> = None;
            let start = std::time::Instant::now();
            while start.elapsed() < std::time::Duration::from_secs(5) {
                if let Ok(r) = rx.try_recv() {
                    result = r;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            result
        };

        // 3. Build the messages array for the AI
        let mut messages: Vec<wikilabs_ai::provider::AiMessage> = Vec::new();

        // Add system prompt if we have observation context, with screenshots appended
        let mut final_system_prompt = if let Some(sys) = system_prompt.clone() {
            if !screenshot_files.is_empty() {
                let screen_count = screenshot_files.len();
                let default_latest = String::new();
                let latest = screenshot_files.last().unwrap_or(&default_latest);
                format!(
                    "{}\n\n## Manual Screenshots\nYou have {} manual screenshot(s) available in ~/Documents/AI Copilot Screenshots/.\n\nScreenshots captured by the user (oldest → newest):\n{}\n\nAnalyze these screenshots to understand the user's context. After providing your advice, the screenshots will be automatically deleted.\nLatest: {}\n",
                    sys,
                    screen_count,
                    screenshot_files.iter().enumerate().map(|(i, f)| format!("{}. {}", i + 1, f)).collect::<Vec<_>>().join("\n"),
                    latest
                )
            } else {
                sys
            }
        } else {
            if !screenshot_files.is_empty() {
                let screen_count = screenshot_files.len();
                format!(
                    "You are a helpful technical assistant. The user has {} manual screenshot(s) available in ~/Documents/AI Copilot Screenshots/ for context.\n\nScreenshots captured by the user (oldest → newest):\n{}\n\nAfter providing your advice, the screenshots will be automatically deleted.\n",
                    screen_count,
                    screenshot_files.iter().enumerate().map(|(i, f)| format!("{}. {}", i + 1, f)).collect::<Vec<_>>().join("\n")
                )
            } else {
                "You are a helpful technical assistant.".to_string()
            }
        };

        if let Some(sys) = system_prompt {
            messages.push(wikilabs_ai::provider::AiMessage {
                role: "system".to_string(),
                content: serde_json::Value::String(final_system_prompt),
            });
        }

        // Add all prior chat messages as history
        for msg in history.iter() {
            messages.push(wikilabs_ai::provider::AiMessage {
                role: msg.role.clone(),
                content: serde_json::Value::String(msg.content.clone()),
            });
        }

        // 4. Add the current user message as a multi-modal message if screenshots exist
        //    This gives the AI actual image content to analyze, not just filenames.
        if !screenshot_images.is_empty() {
            let mut content_parts: Vec<serde_json::Value> = Vec::new();
            // Add the text prompt
            content_parts.push(serde_json::Value::String(message.clone()));
            // Add each screenshot as an image_url part
            for img_url in &screenshot_images {
                content_parts.push(serde_json::json!({
                    "type": "image_url",
                    "image_url": { "url": img_url }
                }));
            }
            messages.push(wikilabs_ai::provider::AiMessage {
                role: "user".to_string(),
                content: serde_json::Value::Array(content_parts),
            });
        } else {
            messages.push(wikilabs_ai::provider::AiMessage {
                role: "user".to_string(),
                content: serde_json::Value::String(message.clone()),
            });
        }

        let ai_request = wikilabs_ai::provider::AiRequest {
            model: model.clone(),
            messages,
            tools: vec![],
            temperature: None,
            max_tokens: Some(max_tokens as usize),
            stream: None,
        };

        // Run the AI call in a blocking thread to avoid holding up the server
        let response_result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new()
                .expect("Failed to create blocking runtime for AI call");
            rt.block_on(provider.chat(ai_request))
        })
        .join()
        .map_err(|e| format!("Thread panicked: {:?}", e))
        .and_then(|r| r.map_err(|e| e.to_string()));

        match response_result {
            Ok(response) => {
                let aid = uuid::Uuid::new_v4().to_string();
                let acreated = chrono::Utc::now().to_rfc3339();
                // Extract text from response content
                let resp_text = match &response.message.content {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(parts) => {
                        parts.iter().find_map(|p| {
                            if let Some(obj) = p.as_object() {
                                if obj.get("type").and_then(|t| t.as_str()) == Some("text") {
                                    obj.get("text").and_then(|t| t.as_str()).map(String::from)
                                } else { None }
                            } else { None }
                        }).unwrap_or_default()
                    }
                    _ => String::new(),
                };
                (aid, acreated, resp_text)
            }
            Err(e) => {
                error!(error = %e, "AI chat failed");
                let aid = uuid::Uuid::new_v4().to_string();
                let acreated = chrono::Utc::now().to_rfc3339();
                (aid, acreated, format!("AI response error: {}\n\nYour message was: \"{}\"", e, message))
            }
        }
    } else {
        let aid = uuid::Uuid::new_v4().to_string();
        let acreated = chrono::Utc::now().to_rfc3339();
        let fallback = format!(
            "Message received: \"{}\"\n\nNote: Configure an AI provider in Settings to get AI responses.",
            message
        );
        (aid, acreated, fallback)
    };

    {
        let settings_ref = state.settings.lock().unwrap();
        let mut msgs = settings_ref.messages.lock().unwrap();
        msgs.push(ChatMessage { id: assistant_id.clone(), role: "assistant".to_string(), content: response.clone(), created_at: assistant_created.clone(), workspace_id: Some(workspace_id.to_string()) });
    }

    // Auto-clear screenshots after AI responds (they've been consumed into the prompt)
    if !screenshot_files.is_empty() {
        if let Ok(entries) = fs::read_dir(&screenshot_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Ok(filename) = entry.file_name().into_string() {
                            if filename.starts_with("screenshot_") && (filename.ends_with(".jpg") || filename.ends_with(".png")) {
                                let _ = fs::remove_file(&entry.path());
                            }
                        }
                    }
                }
            }
        }
        info!(count = screenshot_files.len(), "Manual screenshots auto-deleted after AI response");
    }

    (StatusCode::OK, api_response(true, Some(serde_json::json!({
        "id": assistant_id,
        "role": "assistant",
        "content": response,
        "created_at": assistant_created,
    })), None))
}

fn handle_get_history(state: &ApiServerState) -> (StatusCode, String) {
    let settings_ref = state.settings.lock().unwrap();
    let msgs = settings_ref.messages.lock().unwrap();
    let result = msgs.clone();
    drop(msgs);
    (StatusCode::OK, api_response(true, Some(serde_json::json!(result)), None))
}

fn handle_list_providers(state: &ApiServerState) -> (StatusCode, String) {
    let settings = state.settings.lock().unwrap();
    let providers = settings.providers.clone();
    drop(settings);
    (StatusCode::OK, api_response(true, Some(serde_json::Value::Array(providers)), None))
}

fn handle_get_workspace_list(state: &ApiServerState) -> (StatusCode, String) {
    let settings = state.settings.lock().unwrap();
    let workspaces = settings.workspaces.clone();
    drop(settings);
    let value = serde_json::to_value(workspaces).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

fn handle_create_workspace(state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("New Workspace").to_string();
    let _customer = params.get("customer_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let ws_id = uuid::Uuid::new_v4().to_string();
    let mut settings = state.settings.lock().unwrap();
    settings.workspaces.push(name.clone());
    drop(settings);
    info!(id = %ws_id, name = %name, "Workspace created");
    (StatusCode::OK, api_response(true, Some(serde_json::json!(ws_id)), None))
}

async fn handle_list_models(_state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let api_key = params.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let endpoint = params.get("endpoint").and_then(|v| v.as_str()).unwrap_or("").to_string();
    info!("[TEST] list_models called — endpoint: '{}', api_key_set: {}", endpoint, !api_key.is_empty());
    if endpoint.is_empty() {
        return (StatusCode::OK, api_response(false, None, Some("Endpoint is required".to_string())));
    }

    // Use the existing OpenAICompatibleProvider to fetch models.
    // It constructs the URL as {base_url}/models which works for OpenAI-compatible endpoints.
    let provider = wikilabs_ai::provider::OpenAICompatibleProvider::new(
        "custom",
        &endpoint,
        &api_key,
        "gpt-4o",
        4096,
        128000,
    );

    // The provider's client is pre-configured with proper timeouts.
    // We make a GET request to {base_url}/models to list available models.
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    info!("[TEST] list_models: fetching from {}", url);

    let client = reqwest::Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(0))
        .pool_max_idle_per_host(0)
        .build();
    let client = match client {
        Ok(c) => c,
        Err(e) => {
            error!("[TEST] list_models: failed to build client: {}", e);
            return (StatusCode::OK, api_response(false, None, Some(format!("Client error: {}", e))));
        }
    };

    let mut request = client.get(&url)
        .header("Content-Type", "application/json");
    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    match request.timeout(std::time::Duration::from_secs(10)).send().await {
        Ok(response) => {
            let status = response.status();
            info!("[TEST] list_models: response status = {}", status);
            if status.is_success() {
                match response.json::<Value>().await {
                    Ok(data) => {
                        let models = data.get("data")
                            .and_then(|d| d.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|m| m.get("id").and_then(|id| id.as_str().map(String::from)))
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        info!("[TEST] list_models: found {} models", models.len());
                        (StatusCode::OK, api_response(true, Some(serde_json::json!(models)), None))
                    }
                    Err(e) => {
                        error!("[TEST] list_models: failed to parse JSON: {}", e);
                        // Non-fatal — return empty list so the wizard can use hardcoded fallback
                        (StatusCode::OK, api_response(true, Some(serde_json::json!(Vec::<String>::new())), None))
                    }
                }
            } else {
                let body = response.text().await.unwrap_or_default().chars().take(200).collect::<String>();
                error!("[TEST] list_models: HTTP {} — body: {}", status, body);
                // Return empty list — wizard falls back to hardcoded models
                (StatusCode::OK, api_response(true, Some(serde_json::json!(Vec::<String>::new())), None))
            }
        }
        Err(e) => {
            error!("[TEST] list_models: request failed: {}", e);
            // Return empty list — wizard falls back to hardcoded models
            (StatusCode::OK, api_response(true, Some(serde_json::json!(Vec::<String>::new())), None))
        }
    }
}

/// Skill management handlers
fn handle_skill_list() -> (StatusCode, String) {
    let skills = SkillManagementPanel::instance().list_skills();
    let value = serde_json::to_value(skills).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

fn handle_skill_get(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let skill = SkillManagementPanel::instance().get_skill(name);
    let value = skill.map(|s| serde_json::to_value(s).unwrap_or_default());
    (StatusCode::OK, api_response(true, value, None))
}

fn handle_skill_enable(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    match SkillManagementPanel::instance().enable_skill(name) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

fn handle_skill_disable(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    match SkillManagementPanel::instance().disable_skill(name) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

fn handle_skill_toggle(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    match SkillManagementPanel::instance().toggle_skill(name) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

fn handle_skill_validate(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    match SkillManagementPanel::instance().validate_skill(name) {
        Ok(issues) => {
            let value = serde_json::to_value(issues).unwrap_or_default();
            (StatusCode::OK, api_response(true, Some(value), None))
        }
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

fn handle_skill_mark_validated(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let validated = params.get("validated").and_then(|v| v.as_bool()).unwrap_or(true);
    match SkillManagementPanel::instance().mark_validated(name, validated) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

fn handle_skill_set_active(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let active = params.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
    let confidence = params.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
    match SkillManagementPanel::instance().set_active(name, active, confidence) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

/// Knowledge pack handlers
async fn handle_knowledge_list_packs() -> (StatusCode, String) {
    let panel = KnowledgePanel::instance();
    let packs = panel.list_packs().await;
    let value = serde_json::to_value(packs).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_knowledge_enable_pack(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let panel = KnowledgePanel::instance();
    match panel.enable_pack(name).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_knowledge_disable_pack(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let panel = KnowledgePanel::instance();
    match panel.disable_pack(name).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_knowledge_reindex_pack(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let panel = KnowledgePanel::instance();
    match panel.reindex_pack(name).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_knowledge_get_validation_report(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let panel = KnowledgePanel::instance();
    match panel.get_validation_report(name).await {
        Ok(report) => {
            let value = serde_json::to_value(report).unwrap_or_default();
            (StatusCode::OK, api_response(true, Some(value), None))
        }
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_knowledge_get_metadata(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let panel = KnowledgePanel::instance();
    let pack = panel.get_metadata(name).await;
    let value = pack.map(|p| serde_json::to_value(p).unwrap_or_default());
    (StatusCode::OK, api_response(true, value, None))
}

async fn handle_knowledge_export_pack(params: Value) -> (StatusCode, String) {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let output = params.get("output_path").and_then(|v| v.as_str()).unwrap_or("");
    let panel = KnowledgePanel::instance();
    match panel.export_pack(name, output).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_knowledge_import_pack(params: Value) -> (StatusCode, String) {
    let archive = params.get("archive_path").and_then(|v| v.as_str()).unwrap_or("");
    let dest = params.get("destination").and_then(|v| v.as_str()).unwrap_or("");
    let panel = KnowledgePanel::instance();
    match panel.import_pack(archive, dest).await {
        Ok(path) => (StatusCode::OK, api_response(true, Some(serde_json::json!(path)), None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

// ── Guidance Engine Handlers ────────────────────────────────────

async fn handle_guidance_get_active_recommendations() -> (StatusCode, String) {
    let recs = guidance_panel::GuidancePanel::instance().active_recommendations().await;
    let value = serde_json::to_value(recs).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_get_all_recommendations() -> (StatusCode, String) {
    let recs = guidance_panel::GuidancePanel::instance().all_recommendations().await;
    let value = serde_json::to_value(recs).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_dismiss_recommendation(params: Value) -> (StatusCode, String) {
    let rec_id = params.get("rec_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match guidance_panel::GuidancePanel::instance().dismiss_recommendation(&rec_id).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_guidance_update_recommendation_status(params: Value) -> (StatusCode, String) {
    let rec_id = params.get("rec_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let status = params.get("status").and_then(|v| serde_json::from_value::<guidance_panel::RecommendationStatus>(v.clone()).ok());
    match status {
        Some(s) => match guidance_panel::GuidancePanel::instance().update_recommendation_status(&rec_id, s).await {
            Ok(_) => (StatusCode::OK, api_response(true, None, None)),
            Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
        },
        None => (StatusCode::OK, api_response(false, None, Some("Invalid status".to_string()))),
    }
}

async fn handle_guidance_get_evidence_status() -> (StatusCode, String) {
    let status = guidance_panel::GuidancePanel::instance().get_evidence_status().await;
    let value = serde_json::to_value(status).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_add_evidence(params: Value) -> (StatusCode, String) {
    let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let finding = params.get("finding").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let importance = params.get("importance").and_then(|v| v.as_str()).unwrap_or("medium").to_string();
    let confidence = params.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
    match guidance_panel::GuidancePanel::instance().add_evidence(&source, &finding, &importance, confidence).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_guidance_mark_missing(params: Value) -> (StatusCode, String) {
    let needed = params.get("needed").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = params.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let importance = params.get("importance").and_then(|v| v.as_str()).unwrap_or("medium").to_string();
    match guidance_panel::GuidancePanel::instance().mark_missing(&needed, &description, &importance).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_guidance_get_workflow_progress() -> (StatusCode, String) {
    let progress = guidance_panel::GuidancePanel::instance().get_workflow_progress().await;
    let value = serde_json::to_value(progress).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_start_workflow(params: Value) -> (StatusCode, String) {
    let workflow_id = params.get("workflow_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let workflow_name = params.get("workflow_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let problem_category = params.get("problem_category").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let steps = params.get("steps").and_then(|v| serde_json::from_value::<Vec<guidance_panel::WorkflowStepCard>>(v.clone()).ok()).unwrap_or_default();
    match guidance_panel::GuidancePanel::instance().start_workflow(&workflow_id, &workflow_name, &problem_category, steps).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_guidance_complete_step(params: Value) -> (StatusCode, String) {
    let step_id = params.get("step_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let observation = params.get("observation").and_then(|v| v.as_str()).map(|s| s.to_string());
    match guidance_panel::GuidancePanel::instance().complete_step(&step_id, observation).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_guidance_get_timeline() -> (StatusCode, String) {
    let timeline = guidance_panel::GuidancePanel::instance().get_timeline().await;
    let value = serde_json::to_value(timeline).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_add_timeline_event(params: Value) -> (StatusCode, String) {
    let event_type = params.get("event_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let title = params.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let technology = params.get("technology").and_then(|v| v.as_str()).map(|s| s.to_string());
    let finding = params.get("finding").and_then(|v| v.as_str()).map(|s| s.to_string());
    let description = params.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let confidence = params.get("confidence").and_then(|v| v.as_f64());
    let recommendation_id = params.get("recommendation_id").and_then(|v| v.as_str()).map(|s| s.to_string());
    match guidance_panel::GuidancePanel::instance().add_timeline_event(&event_type, title.as_deref(), technology.as_deref(), finding.as_deref(), description.as_deref(), confidence, recommendation_id).await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_guidance_get_recent_events(params: Value) -> (StatusCode, String) {
    let minutes = params.get("minutes").and_then(|v| v.as_u64()).unwrap_or(60);
    let events = guidance_panel::GuidancePanel::instance().get_recent_events(minutes).await;
    let value = serde_json::to_value(events).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_record_feedback(params: Value) -> (StatusCode, String) {
    let recommendation_id = params.get("recommendation_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let feedback_type = params.get("feedback_type").and_then(|v| serde_json::from_value::<guidance_panel::FeedbackType>(v.clone()).ok());
    let notes = params.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
    match feedback_type {
        Some(ft) => match guidance_panel::GuidancePanel::instance().record_feedback(&recommendation_id, ft, notes.as_deref()).await {
            Ok(_) => (StatusCode::OK, api_response(true, None, None)),
            Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
        },
        None => (StatusCode::OK, api_response(false, None, Some("Invalid feedback_type".to_string()))),
    }
}

async fn handle_guidance_get_feedback_stats() -> (StatusCode, String) {
    let stats = guidance_panel::GuidancePanel::instance().get_feedback_stats().await;
    let value = serde_json::to_value(stats).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_clear_all() -> (StatusCode, String) {
    match guidance_panel::GuidancePanel::instance().clear_all().await {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e.to_string()))),
    }
}

async fn handle_guidance_show_toast(_title: &str, _body: &str) -> (StatusCode, String) {
    // API endpoint exists for frontend compatibility but notification is handled client-side.
    // The browser's Notification API is used in the GuidanceToast component.
    (StatusCode::OK, api_response(true, None, None))
}

/// Internal: shared AppHandle for notifications (set from main.rs after Tauri init).
static SHARED_APP_HANDLE: std::sync::Mutex<Option<AppHandle>> = std::sync::Mutex::new(None);

pub fn set_shared_app_handle(handle: AppHandle) {
    let mut guard = SHARED_APP_HANDLE.lock().unwrap();
    *guard = Some(handle);
}

pub fn get_shared_app_handle() -> Option<AppHandle> {
    let guard = SHARED_APP_HANDLE.lock().unwrap();
    guard.clone()
}

async fn handle_observation_get_status(_state: &ApiServerState) -> (StatusCode, String) {
    // Report real observation engine status
    let status_value = match crate::observation::get_observation_engine() {
        Some(engine) => {
            let status = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    engine.get_provider_status().await
                })
            });
            let running = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    engine.is_running().await
                })
            });
            let active_providers: Vec<String> = status.iter()
                .filter(|p| p.state == wikilabs_observation::provider::ProviderState::Active)
                .map(|p| p.name.clone())
                .collect();
            serde_json::json!({
                "observation_enabled": running,
                "status": if running { "active" } else { "stopped" },
                "providers": active_providers,
                "provider_count": status.len(),
                "active_provider_count": active_providers.len()
            })
        },
        None => serde_json::json!({
            "observation_enabled": false,
            "status": "stopped",
            "providers": [],
            "provider_count": 0,
            "active_provider_count": 0
        })
    };
    (StatusCode::OK, api_response(true, Some(status_value), None))
}

async fn handle_observation_start(state: &ApiServerState) -> (StatusCode, String) {
    info!("Observation start requested");

    // Check if API key is configured before starting
    let settings = state.settings.lock().unwrap();
    let has_api_key = settings.settings.get("ai_provider")
        .and_then(|p| p.get("api_key"))
        .and_then(|k| k.as_str())
        .map(|key| !key.is_empty())
        .unwrap_or(false);
    drop(settings);

    if !has_api_key {
        info!("Observation start skipped — no API key configured");
    }

    // Spawn lazy init — run on the API server's existing multi-threaded runtime.
    // This is critical: creating a new single-threaded runtime + block_on() in a
    // separate thread would drop the runtime immediately after the async block
    // completes, killing any tokio::spawn'd tasks (including the observation
    // polling loop). By running on the API server's runtime, the spawned tasks
    // keep the observation engine alive as long as the API server lives.
    tokio::spawn(async move {
        let started = crate::observation::lazy_start_observation_engine().await;
        if started {
            tracing::info!("Observation engine started successfully (post-setup)");
        } else {
            tracing::error!("Failed to start observation engine (post-setup)");
        }
    });

    (StatusCode::OK, api_response(true, Some(serde_json::json!({ "status": "starting" })), None))
}

async fn handle_observation_stop(state: &ApiServerState) -> (StatusCode, String) {
    info!("Observation stop requested");
    // Stop observation by signaling via settings — the observation engine
    // polls the settings to know whether to continue.
    {
        let mut settings = state.settings.lock().unwrap();
        settings.settings["observation_engine_enabled"] = serde_json::json!(false);
    }
    (StatusCode::OK, api_response(true, Some(serde_json::json!({ "status": "stopped" })), None))
}

// Save screenshot to user's Documents folder
/// Capture a screenshot and save it to the screenshots directory.
/// Minimizes the AI Copilot window before capturing so the window itself isn't in the screenshot.
async fn handle_capture_screenshot(state: &ApiServerState) -> (StatusCode, String) {
    let screenshot_dir = get_screenshots_dir();
    if let Err(e) = fs::create_dir_all(&screenshot_dir) {
        error!(error = %e, "Failed to create screenshots directory");
        return (StatusCode::INTERNAL_SERVER_ERROR, api_response(false, None, Some(format!("Failed to create directory: {}", e))));
    }

    // Hide the AI Copilot "main" window so it won't appear in the screenshot.
    // We do NOT restore the main window after capture — only the advice-chat
    // window should be shown. The user clicked the screenshot button from the
    // advice-chat window and expects it to remain visible.
    {
        if let Some(ref app_handle) = state.app_handle {
            let guard = app_handle.lock().unwrap();
            if let Some(window) = guard.get_webview_window("main") {
                let _ = window.hide();
                info!("Hidden main window before screenshot capture");
            }
        }
    }

    // Brief pause to let the window actually hide and the screen update
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Get the latest screenshot from the observation engine
    let screenshot = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            crate::observation::get_last_screenshot_async().await
        })
    });

    match screenshot {
        Some(ss) => {
            let timestamp = ss.timestamp.format("%Y%m%d_%H%M%S");
            let filename = format!("screenshot_{}.jpg", timestamp);
            let filepath = screenshot_dir.join(&filename);

            // Decode base64 and write to file
            let decoded = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &ss.data_base64) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(error = %e, "Failed to decode screenshot base64");
                    // Show advice-chat window on error
                    if let Some(ref app_handle) = state.app_handle {
                        let guard = app_handle.lock().unwrap();
                        if let Some(window) = guard.get_webview_window("advice-chat") {
                            let _ = window.show();
                        }
                    }
                    return (StatusCode::INTERNAL_SERVER_ERROR, api_response(false, None, Some(format!("Failed to decode screenshot: {}", e))));
                }
            };

            match fs::File::create(&filepath).and_then(|mut f| f.write_all(&decoded)) {
                Ok(_) => {
                                            info!(path = ?filepath, width = ss.width, height = ss.height, "Screenshot captured and saved");
                                            // Show advice-chat window (where the user clicked the screenshot button)
                                            if let Some(ref app_handle) = state.app_handle {
                                                let guard = app_handle.lock().unwrap();
                                                if let Some(window) = guard.get_webview_window("advice-chat") {
                                                    let _ = window.show();
                                                }
                                            }
                    // Notify the frontend to show a toast
                    let filepath_str = filepath.to_string_lossy().to_string();
                    let _ = show_screenshot_toast("Screenshot captured", &filepath_str);
                    (StatusCode::OK, api_response(true, Some(serde_json::json!({
                        "path": filepath_str,
                        "timestamp": timestamp.to_string(),
                        "width": ss.width,
                        "height": ss.height,
                    })), None))
                }
                Err(e) => {
                    error!(error = %e, "Failed to save screenshot");
                    // Restore window even on error
                    if let Some(ref app_handle) = state.app_handle {
                        let guard = app_handle.lock().unwrap();
                        if let Some(window) = guard.get_webview_window("main") {
                            let _ = window.show();
                        }
                    }
                    (StatusCode::INTERNAL_SERVER_ERROR, api_response(false, None, Some(format!("Failed to save screenshot: {}", e))))
                }
            }
        }
        None => {
            // No screenshot available — try to capture one fresh
            info!("No previous screenshot available — attempting to trigger one");
            // Give it a moment and retry — the screen capture provider may need to run its cycle
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let screenshot2 = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    crate::observation::get_last_screenshot_async().await
                })
            });

            // Restore window before waiting
            if let Some(ref app_handle) = state.app_handle {
                let guard = app_handle.lock().unwrap();
                if let Some(window) = guard.get_webview_window("main") {
                    let _ = window.show();
                }
            }

            match screenshot2 {
                Some(ss) => {
                    let timestamp = ss.timestamp.format("%Y%m%d_%H%M%S");
                    let filename = format!("screenshot_{}.jpg", timestamp);
                    let filepath = screenshot_dir.join(&filename);

                    let decoded = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &ss.data_base64) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            error!(error = %e, "Failed to decode screenshot base64");
                            return (StatusCode::INTERNAL_SERVER_ERROR, api_response(false, None, Some(format!("Failed to decode screenshot: {}", e))));
                        }
                    };

                    match fs::File::create(&filepath).and_then(|mut f| f.write_all(&decoded)) {
                        Ok(_) => {
                            info!(path = ?filepath, width = ss.width, height = ss.height, "Screenshot captured and saved");
                            let filepath_str = filepath.to_string_lossy().to_string();
                            let _ = show_screenshot_toast("Screenshot captured", &filepath_str);
                            (StatusCode::OK, api_response(true, Some(serde_json::json!({
                                "path": filepath_str,
                                "timestamp": timestamp.to_string(),
                                "width": ss.width,
                                "height": ss.height,
                            })), None))
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to save screenshot");
                            (StatusCode::INTERNAL_SERVER_ERROR, api_response(false, None, Some(format!("Failed to save screenshot: {}", e))))
                        }
                    }
                }
                None => {
                    warn!("No screenshot available from observation engine");
                    (StatusCode::SERVICE_UNAVAILABLE, api_response(false, None, Some("No screenshot available. Screen capture provider may not be initialized. Enable the Observation Engine in Settings.".to_string())))
                }
            }
        }
    }
}

/// Clear all screenshots from the screenshots directory.
async fn handle_clear_screenshots() -> (StatusCode, String) {
    let screenshot_dir = get_screenshots_dir();

    match fs::read_dir(&screenshot_dir) {
        Ok(entries) => {
            let mut deleted = 0u32;
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if fs::remove_file(&entry.path()).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
            info!(deleted, "Screenshots cleared");
            (StatusCode::OK, api_response(true, Some(serde_json::json!({ "deleted": deleted })), None))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("Screenshots directory does not exist, nothing to clear");
            (StatusCode::OK, api_response(true, Some(serde_json::json!({ "deleted": 0 })), None))
        }
        Err(e) => {
            error!(error = %e, "Failed to clear screenshots");
            (StatusCode::INTERNAL_SERVER_ERROR, api_response(false, None, Some(format!("Failed to clear screenshots: {}", e))))
        }
    }
}

/// Hide the main window (minimize to tray).
async fn handle_hide_main_window(state: &ApiServerState) -> (StatusCode, String) {
    info!("Hide main window (minimize to tray) requested");
    if let Some(ref app_handle) = state.app_handle {
        if let Some(window) = app_handle.lock().unwrap().get_webview_window("main") {
            let _ = window.hide();
        }
    }
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"hidden": true})), None))
}

/// Open the floating advice chat window on the right side.
/// Creates the window if it doesn't exist (lazy init), then shows and focuses it.
/// The window has no close (X) button — clicking X will minimize to a roll-up pill.
/// Minimize button rolls up the window into just the title bar at bottom-right.
async fn handle_advice_chat_open(state: &ApiServerState) -> (StatusCode, String) {
    info!("Opening advice chat floating window");
    if let Some(ref app_handle) = state.app_handle {
        let ah = app_handle.lock().unwrap().clone();
        // If the window already exists, just show and focus it
        if let Some(window) = ah.get_webview_window("advice-chat") {
            info!("advice-chat window exists — showing and focusing");
            if let Err(e) = window.show() {
                error!("Failed to show advice-chat window: {}", e);
            } else {
                let _ = window.set_focus();
            }
            return (StatusCode::OK, api_response(true, Some(serde_json::json!({"opened": true})), None));
        }
        // Window doesn't exist — create it (lazy init)
        info!("advice-chat window not found, creating it now");
        let url = tauri::WebviewUrl::External("http://localhost:1420/advice-chat".parse::<url::Url>().unwrap());

        // Build window builder with GPU-disabling browser args (same as main window).
        // On some Windows systems, WebView2 fails to render a second window without
        // these args, resulting in a blank or invisible window.
        #[cfg(target_os = "windows")]
        let builder = tauri::webview::WebviewWindowBuilder::new(&ah, "advice-chat", url)
            .title("AI Copilot — Live Advice")
            .inner_size(400.0, 520.0)
            .resizable(true)
            .decorations(false)
            .always_on_top(true)
            .additional_browser_args("--disable-gpu --no-sandbox --disable-software-rasterizer");

        #[cfg(not(target_os = "windows"))]
        let builder = tauri::webview::WebviewWindowBuilder::new(&ah, "advice-chat", url)
            .title("AI Copilot — Live Advice")
            .inner_size(400.0, 520.0)
            .resizable(true)
            .decorations(false)
            .always_on_top(true);

        let result = builder.build();
        if let Ok(window) = result {
            info!("advice-chat window built successfully");
            // Position on the right side of the screen (vertically centered)
            if let Ok(Some(monitor)) = window.current_monitor() {
                let width = 400.0;
                let height = 520.0;
                let x = monitor.size().width as f64 - width - 10.0;
                let y = (monitor.size().height as f64 - height) / 2.0;
                if let Err(e) = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32)) {
                    warn!("Failed to position advice-chat window: {}", e);
                }
                info!("advice-chat window positioned at ({}, {})", x as i32, y as i32);
            }
            // Prevent the window from being closed (X button is hidden via decorations=false)
            let w_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let _ = w_clone.hide();
                    api.prevent_close();
                }
            });
            // Show and focus the window — log any failures
            if let Err(e) = window.show() {
                error!("Failed to show advice-chat window after build: {}", e);
            } else {
                info!("advice-chat window.show() succeeded");
                if let Err(e) = window.set_focus() {
                    warn!("Failed to focus advice-chat window: {}", e);
                } else {
                    info!("advice-chat window.set_focus() succeeded");
                }
            }
        } else {
            error!("Failed to create advice-chat window: {:?}", result.err());
        }
    } else {
        warn!("advice_chat_open: app_handle is None — cannot create window");
    }
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"opened": true})), None))
}

/// Hide the advice chat window (minimize to desktop roll-up pill).
/// This actually hides the OS window, not just the internal UI.
async fn handle_hide_advice_window(state: &ApiServerState) -> (StatusCode, String) {
    info!("Hiding advice chat window");
    if let Some(ref app_handle) = state.app_handle {
        if let Some(window) = app_handle.lock().unwrap().get_webview_window("advice-chat") {
            let _ = window.hide();
        }
    }
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"hidden": true})), None))
}

/// Drag the advice chat window (move it by dx, dy from HTTP API).
/// Called from frontend title bar drag-to-move via periodic flush.
async fn handle_drag_advice_window(state: &ApiServerState, params: Value) -> (StatusCode, String) {
    if let Some(ref app_handle) = state.app_handle {
        let dx = params.get("dx").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        let dy = params.get("dy").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        if let Some(window) = app_handle.lock().unwrap().get_webview_window("advice-chat") {
            if let Ok(current_pos) = window.outer_position() {
                let new_x = current_pos.x as f64 + dx;
                let new_y = current_pos.y as f64 + dy;
                let _ = window.set_position(tauri::PhysicalPosition::new(new_x as i32, new_y as i32));
            }
        }
    }
    (StatusCode::OK, api_response(true, None, None))
}

async fn handle_observation_get_context() -> (StatusCode, String) {
    let panel = guidance_panel::GuidancePanel::instance();
    let active = panel.active_recommendations().await;
    let all = panel.all_recommendations().await;
    let value = serde_json::json!({
        "active_recommendations": active,
        "all_recommendations": all,
        "status": "active"
    });
    (StatusCode::OK, api_response(true, Some(value), None))
}

/// GET /api/commands/get_status — returns app version and backend status.
async fn handle_get_status() -> (StatusCode, String) {
    let version = env!("CARGO_PKG_VERSION");
    let value = serde_json::json!({
        "version": version,
        "status": "running",
        "features": {
            "chat": true,
            "knowledge": true,
            "workspace": true,
            "skills": true,
            "mcp": false,
            "automation": false
        }
    });
    (StatusCode::OK, api_response(true, Some(value), None))
}
/// Create the router for the API server.
pub fn create_router(state: ApiServerState, assets_dir: Option<String>) -> Router {
    info!("[API] Creating router with state...");
    
    // Debug: list all registered routes
    info!("[API] Route creation complete — now registering fallback");
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let advice_html = include_str!("../assets/advice-chat.html");
    
    // Clone state for the closure that captures it
    let state_for_closure = state.clone();
    
    // Build router — ServeDir assets path uses absolute path to avoid relative-path issues on Windows
    let mut router = Router::new()
        .route("/api/commands/:method", post(move |method: Path<String>, req: Json<ApiRequest>| {
            let state = state_for_closure.clone();
            async move { api_handler(State(state), method, req).await }
        }))
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(|| async { 
            let ready = crate::api_ready::is_server_ready();
            format!("{{\"ready\":{}}}", if ready { "true" } else { "false" })
        }))
        .route("/advice-chat", get(move || async move { Html(advice_html.to_string()) }))
        .layer(cors)
        .fallback(|method: axum::http::Method, uri: axum::http::Uri| async move {
            warn!("[API] FALLBACK HIT — method={} uri={}", method, uri);
            (StatusCode::NOT_FOUND, format!("No route for {} {}", method, uri))
        })
        .with_state(state);
    
    // Serve static assets (JS, CSS, images) for the advice-chat window
    // Use absolute path resolved from the assets_dir parameter — avoids relative-path CWD issues on Windows
    if let Some(ref assets_dir) = assets_dir {
        router = router.nest_service("/assets", ServeDir::new(assets_dir));
        info!(assets_dir = %assets_dir, "Serving static assets");
    } else {
        warn!("[API] No assets directory configured — static assets (JS/CSS) will not be served");
    }
    
    info!("[API] Router fully configured with fallback");
    router
}

/// Start the HTTP server on the given port (default 1420).
/// Runs in a dedicated thread to keep the tokio runtime alive.
pub fn start_api_server(
    port: u16,
    config_path: Option<std::path::PathBuf>,
    skills_path: Option<std::path::PathBuf>,
    knowledge_path: Option<std::path::PathBuf>,
    app_handle: Option<Arc<tauri::AppHandle>>,
) -> Result<(), String> {
    println!("[API] >>> start_api_server(port={}) called", port);
    tracing::info!("[API] >>> start_api_server(port={}) called", port);
    // Clone app_handle for use inside the AI loop closure (state will be consumed by the move)
    let app_handle_for_loop = app_handle.clone();

    // Resolve absolute assets directory path for static file serving
    // The build.rs copies frontend dist assets into src-tauri/assets/
    // We need the absolute path to avoid relative-path CWD issues on Windows
    // MUST be done BEFORE app_handle is moved into state
    let assets_dir = {
        // Try 1: From current exe's parent — for release builds on Windows NSIS,
        // the binary is in the install dir and assets are installed alongside it
        std::env::current_exe().ok().and_then(|exe| {
            exe.parent()
                .map(|parent| parent.join("assets"))
                .and_then(|p| p.canonicalize().ok())
                .map(|p| p.to_string_lossy().to_string())
        })
        // Try 2: Use Tauri resource_dir if app_handle is available (for Tauri bundler)
        .or_else(|| {
            if let Some(ref ah) = app_handle {
                if let Ok(resource_dir) = ah.path().resource_dir() {
                    let candidate = resource_dir.join("assets");
                    if candidate.exists() {
                        return Some(candidate.to_string_lossy().to_string());
                    }
                }
            }
            None
        })
        // Try 3: From current working directory (debug/development)
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|cwd| cwd.join("assets").canonicalize().ok())
                .map(|p| p.to_string_lossy().to_string())
        })
        // Try 4: From crate root's assets directory (build-time path)
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|cwd| cwd.join("../assets").canonicalize().ok())
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| {
            warn!("[API] Could not resolve assets directory — static assets will not be served");
            String::new()
        })
    };
    // Strip Windows long-path prefix (\\?\) which can cause issues with ServeDir
    let assets_dir = assets_dir.trim_start_matches("\\\\?\\").to_string();
    info!(assets_dir = %assets_dir, "Resolved assets directory");

    // Initialize knowledge packs from multiple possible locations:
    // 1. Bundled resource directory (from tauri.conf.json bundle.resources)
    // 2. App data directory (for user-installed/updated packs)
    // 3. Config directory parent (legacy fallback)
    // On Windows NSIS installer, resource_dir() may not always resolve correctly,
    // so we try data_dir as a fallback where bundled resources are also copied.
    let data_dir = config_path.as_ref().and_then(|cp| cp.parent().map(|p| p.to_path_buf()));
    let knowledge_dir_to_use = knowledge_path.clone()
        .or_else(|| data_dir.clone().map(|dd| dd.join("knowledge")))
        .or_else(|| config_path.as_ref().and_then(|cp| cp.parent().map(|p| p.join("knowledge"))));

    // Clone app_handle for use inside the thread (auto-start guidance spawn)
    let app_handle_for_thread = app_handle.clone();

    let state = ApiServerState {
        settings: Arc::new(Mutex::new(ApiServerSettings::new())),
        config_path: Arc::new(Mutex::new(config_path.clone())),
        knowledge_dir: Arc::new(Mutex::new(knowledge_dir_to_use.clone())),
        app_handle: app_handle.map(|h| Arc::new(Mutex::new(AppHandle::clone(&*h)))),
    };

    // Load settings from disk at startup so the AI loop sees the configured API key
    if let Some(ref cp) = config_path {
        if let Ok(content) = fs::read_to_string(cp) {
            if let Ok(parsed) = serde_json::from_str::<Value>(&content) {
                let mut settings = state.settings.lock().unwrap();
                settings.settings = parsed;
                info!("Settings loaded from disk at startup");
            }
        }
    }

    let obs_settings = state.settings.clone();

    // ── Auto-start observation/guidance on subsequent launches ──────────
    // If the user already completed the wizard (first_run_complete=true),
    // start observation engine + guidance loop immediately. This covers
    // all launches after the first setup.
    //
    // NOTE: The observation engine init + guidance loop spawn are deferred
    // until INSIDE the multi-threaded runtime's block_on (see below). This
    // avoids using get_or_create_auto_runtime() which creates a single-threaded
    // runtime where tokio::spawn tasks can get stuck — the same runtime that
    // serves HTTP and where the guidance loop (on its own rt) can also send
    // requests. Sharing one multi-threaded runtime guarantees the spawned
    // polling loop task always gets scheduled and executed.
    let should_auto_start = {
        let settings = state.settings.lock().unwrap();
        let first_run = settings.settings.get("first_run_complete")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        // Log the auto-start decision with full settings context
        let ai_info = settings.settings.get("ai_provider")
            .and_then(|p| p.as_object())
            .and_then(|p| p.get("api_key"))
            .and_then(|v| v.as_str())
            .map(|k| if k.is_empty() { "EMPTY" } else { "SET" })
            .unwrap_or("NOT_FOUND");
        info!("Auto-start decision: first_run_complete={}, ai_provider.api_key_status={}", first_run, ai_info);
        first_run
    };

    let router = create_router(state, Some(assets_dir));

    // Initialize skill and knowledge panels
    if let Some(ref skills_dir) = skills_path {
        info!(dir = %skills_dir.display(), "Loading skills from resource path");
        if let Err(e) = SkillManagementPanel::instance().load_from_directory(&skills_dir.to_string_lossy()) {
            error!(error = %e, "Failed to load skills from resource path");
        }
    } else {
        // Fallback: try loading from data directory
        if let Some(ref cp) = config_path {
            if let Some(data_dir) = cp.parent() {
                let skills_dir = data_dir.join("skills");
                info!(dir = %skills_dir.display(), "Loading skills from data directory");
                if let Err(e) = SkillManagementPanel::instance().load_from_directory(&skills_dir.to_string_lossy()) {
                    error!(error = %e, "Failed to load skills");
                }
            }
        }
    }

    // Build skill knowledge base for AI prompt injection
    let skill_kb = if let Some(ref skills_dir) = skills_path {
        create_skill_knowledge_base(&skills_dir.to_string_lossy())
    } else if let Some(ref cp) = config_path {
        if let Some(data_dir) = cp.parent() {
            create_skill_knowledge_base(&data_dir.join("skills").to_string_lossy())
        } else {
            create_skill_knowledge_base("")
        }
    } else {
        create_skill_knowledge_base("")
    };
    let _skill_kb_clone = skill_kb.clone();

    // Store knowledge path for later async initialization
    let kdir = knowledge_dir_to_use.clone();
    let kdir_str = kdir.as_ref().map(|d| d.to_string_lossy().to_string());

    let addr = format!("0.0.0.0:{port}");

    info!(addr, "Starting API server in background thread");

    std::thread::spawn(move || {
        // Use multi-threaded runtime to prevent deadlock:
        // - Single-threaded runtime can deadlock when Handle::current().block_on()
        //   blocks the only worker thread while a spawned task holds a lock
        // - Multi-threaded runtime allows HTTP handlers to progress on other threads
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create tokio runtime: {}", e));
        
        if let Ok(rt) = rt {
            println!("[API] >>> Tokio multi-threaded runtime created OK");
            tracing::info!("[API] >>> Tokio multi-threaded runtime created OK");
            // Knowledge packs are loaded AFTER the server starts listening
            // (see the axum::serve block below). This avoids blocking startup.

            // Initialize observation engine — providers already registered in main.rs
            // via observation::init_observation_engine(). We reuse the shared engine here.
            info!("Observation engine initialized (shared from main.rs)");
            println!("[API] >>> Observation engine initialized");

            // Reset guidance state on startup — fresh session, zero evidence
            {
                let panel = guidance_panel::GuidancePanel::instance();
                let _ = rt.block_on(panel.clear_all());
                info!("Guidance panel state cleared on startup");
            }

            // Guidance loop and observation engine are now triggered lazily
            // via handle_set_first_run_complete (after the user completes the wizard).
            // This prevents background polling from running during the wizard.
            info!("Guidance loop will start after wizard completion (triggered by set_first_run_complete)");


            let result = rt.block_on(async {
                // TCP bind inside tokio runtime
                let listener = match tokio::net::TcpListener::bind(&addr).await {
                    Ok(l) => {
                        println!("[API] >>> Tokio TCP bound to {}", addr);
                        tracing::info!("[API] >>> Tokio TCP bound to {}", addr);
                        l
                    }
                    Err(e) => {
                        println!("[API] >>> Tokio TCP bind FAILED to {}: {}", addr, e);
                        tracing::error!("[API] >>> Tokio TCP bind FAILED to {}: {}", addr, e);
                        return Err(format!("Failed to bind to {}: {}", addr, e));
                    }
                };

                info!(addr, "API server listening");

                // Signal to frontend that the server is ready to handle requests
                // MUST happen BEFORE knowledge pack loading — frontend polls /ready
                // to determine if the API server is alive. Knowledge packs are an
                // optional enhancement; the server can serve API commands without them.
                println!("[API] >>> Ready marker set — server is live!");
                tracing::info!("[API] >>> Ready marker set — server is live!");
                crate::api_ready::mark_server_ready();

                // ── Auto-start observation/guidance on subsequent launches ──────
                // Run inside this multi-threaded runtime so the spawned polling
                // loop task is guaranteed to execute (avoids the single-threaded
                // runtime bug where tokio::spawn tasks get stuck after block_on).
                // ONLY auto-start if the user has explicitly enabled these features.
                if should_auto_start {
                    // Re-check settings inside the async block
                    let obs_enabled = {
                        let settings = obs_settings.lock().unwrap();
                        settings.settings.get("observation_engine_enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    };
                    let guidance_enabled = {
                        let settings = obs_settings.lock().unwrap();
                        settings.settings.get("guidance_loop_enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    };

                    if obs_enabled || guidance_enabled {
                        info!("AUTO-START: first_run_complete=true, observation_engine_enabled={}, guidance_loop_enabled=true — starting on multi-threaded runtime", obs_enabled);

                        let has_key = {
                            let settings = obs_settings.lock().unwrap();
                            settings.settings.get("ai_provider")
                                .and_then(|p| p.as_object())
                                .and_then(|p| p.get("api_key"))
                                .and_then(|v| v.as_str())
                                .map_or(false, |k| !k.is_empty())
                        };

                        info!("AUTO-START: AI API key is {}", if has_key { "configured" } else { "NOT configured — loop will skip AI reasoning" });

                        if obs_enabled {
                            // Initialize observation engine on THIS multi-threaded runtime
                            info!("[AUTO] Initializing observation engine (subsequent launch, enabled by settings)");
                            let engine = crate::observation::init_observation_engine().await;
                            crate::observation::start_observation_engine(engine).await;
                            info!("[AUTO] Observation engine initialized and started (subsequent launch)");

                            // Spawn guidance loop — globals are already set above
                            if let Some(ref ah) = app_handle_for_thread {
                                info!("[AUTO] Spawning guidance loop with obs_settings and skill_kb...");
                                guidance_loop::spawn_ai_guidance_loop(
                                    obs_settings.clone(),
                                    crate::skill_knowledge::create_skill_knowledge_base(""),
                                    Some(Arc::new(ah.as_ref().clone())),
                                );
                                info!("[AUTO] Guidance loop spawned successfully (subsequent launch)");
                            } else {
                                warn!("[AUTO] No app_handle available — skipping guidance loop spawn");
                            }
                        }
                    } else {
                        info!("AUTO-START skipped: observation_engine_enabled=false, guidance_loop_enabled=false");
                    }
                }

                // Load knowledge packs inside the runtime but AFTER marking ready
                // so the frontend can start communicating with the server immediately
                if let Some(ref kdir_path) = kdir_str {
                    info!(dir = %kdir_path, "Loading knowledge packs (background)");
                    let panel = KnowledgePanel::instance();
                    let kdir = kdir_path.clone();
                    rt.spawn(async move {
                        if let Err(e) = panel.initialize(&kdir).await {
                            error!(error = %e, "Failed to load knowledge packs");
                            println!("[API] >>> Knowledge packs load FAILED: {}", e);
                        } else {
                            println!("[API] >>> Knowledge packs loaded OK");
                        }
                    });
                }

                if let Err(e) = axum::serve(listener, router).await {
                    error!(error = %e, "API server error");
                }
                
                Ok::<(), String>(())
            });
            
            if let Err(e) = result {
                error!(error = %e, "API server failed");
            }
        } else {
            error!("API server runtime creation failed");
        }
    });

    Ok(())
}

