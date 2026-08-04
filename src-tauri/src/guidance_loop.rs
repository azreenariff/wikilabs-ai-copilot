//! Background AI reasoning loop — runs on its own Tokio runtime, isolated from the API server.
//!
//! This module is responsible for:
//! - Polling the shared observation engine's event bus for new events
//! - Building structured event summaries and filtering noise
//! - Periodically reasoning about the user's activity using the configured AI provider
//! - Writing suggestions to the GuidancePanel and chat history
//!
//! It runs in a separate std::thread with its own tokio runtime to prevent
//! the AI reasoning loop from starving the API server's HTTP handlers.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;
use tauri::AppHandle;
use tauri::Manager;
use tauri::Emitter;
use tracing::{debug, error, info, warn};

use crate::guidance_panel::{GuidancePanel, CardRiskLevel};
use crate::knowledge_panel::KnowledgePanel;
use crate::skill_knowledge::SkillKnowledgeBaseArc;
use crate::api_server::ChatMessage;
use crate::observation;
use wikilabs_observation::event::EventType;
use wikilabs_ai::AiProvider;
use wikilabs_ai::provider::OpenAICompatibleProvider;

/// Structured event summaries passed through the AI reasoning loop.
#[derive(Clone)]
struct StructuredEvent {
    provider: String,
    event_type: String,
    source: String,
    summary: String,
    payload_json: serde_json::Value,
}

/// Background AI reasoning loop — runs on its own Tokio runtime, isolated from the API server.
///
/// This function polls observation events every 3 seconds and runs AI reasoning
/// every 10 seconds. It is deliberately kept in a separate std::thread with its own
/// tokio runtime so that long AI API calls never starve the HTTP server's event loop.
pub fn spawn_ai_guidance_loop(
    poll_settings: Arc<Mutex<crate::api_server::ApiServerSettings>>,
    skill_kb: SkillKnowledgeBaseArc,
    app_handle: Option<Arc<AppHandle>>,
) {
    info!("[GUIDE] Spawning AI guidance loop on isolated Tokio runtime");

    // Create the runtime INSIDE the spawned thread to avoid "Cannot start a runtime
    // from within a runtime" panic. When called from an axum handler, the current
    // thread is already inside the API server's multi-threaded runtime; creating a
    // new runtime on that thread would cause a deterministic crash.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for AI guidance loop");
        rt.block_on(async {
            run_guidance_loop_inner(
                poll_settings,
                skill_kb,
                app_handle,
            ).await;
        });
        info!("[GUIDE] AI guidance loop exited");
    });
}

async fn run_guidance_loop_inner(
    poll_settings: Arc<Mutex<crate::api_server::ApiServerSettings>>,
    skill_kb: SkillKnowledgeBaseArc,
    app_handle: Option<Arc<AppHandle>>,
) {
    // Event polling: every 3 seconds — collect fresh observation events
    let mut interval = tokio::time::interval(Duration::from_secs(3));
    // AI reasoning: every 10 seconds — much more responsive to user activity
    let mut ai_interval = tokio::time::interval(Duration::from_secs(10));

    let mut last_events: Vec<StructuredEvent> = Vec::new();

    // Noise filter: check if an event is non-engineering UI wrapper, crashpad, or background process.
    // Returns true if the event should be SKIPPED from AI guidance.
    let noise_sources = [
        "crashpad_handler", "ui_wrapper", "ui-wrapper", "chrome_crashpad",
        "chrome_crashpad_handler", "ew_rust_backend_ew_rust", "ew_backend",
        "ew_rust", "webview", "WebView", "WebViews", "content_helper",
        "content-helpers", "gpu-process", "gpu_process", "service_worker",
        "network_service", "sandbox", "xdg-desktop-portal", "xdg-screensaver",
        "dconf-service", "gmain", "gdbus", "systemd", "dbus-daemon",
        "logind", "accounts-daemon", "thermald", "power-profiles-daemon",
        "wpa_supplicant", "NetworkManager", "bluetoothd", "avahi-daemon",
        "cupsd", "cron", "atd", "getty", "login", "sshd",
        "chrome.exe", "chromium.exe", "chrome_crashpad_handler.exe",
        "ui_wrapper.exe", "ew_rust_backend.exe", "ew_backend.exe",
        "ew_rust.exe", "content-helpers.exe", "gpu-process.exe",
        "service_worker.exe", "network_service.exe",
    ];
    let engineering_terminal_keywords = [
        "bash", "zsh", "powershell", "cmd", "ssh", "code",
        "firefox", "chrome", "terminal", "alacritty", "wezterm",
    ];
    let is_noise_event = |event: &StructuredEvent| -> bool {
        let source_lower = event.source.to_lowercase();
        let summary_lower = event.summary.to_lowercase();
        for noise in &noise_sources {
            if source_lower.contains(noise) || summary_lower.contains(noise) {
                return true;
            }
        }
        if event.source.trim().is_empty() {
            return true;
        }
        if event.source.starts_with("pid:")
            && !event.source.contains('/')
            && !event.source.contains('\\')
            && !engineering_terminal_keywords.iter().any(|kw| event.source.contains(kw))
        {
            return true;
        }
        if event.provider == "active_window" && (
            event.source == "inactive"
            || summary_lower.contains("no_window_info_available")
            || summary_lower.contains("platform:")
        ) {
            return true;
        }
        // Allow heartbeat events — they keep the AI context alive
        if event.source == "engine_heartbeat" {
            return false;
        }
        false
    };

    // Subscribe to the shared observation engine's event bus
    let event_rx = observation::get_event_receiver();
    let engine = observation::get_observation_engine();

    loop {
        tokio::select! {
            // Phase 1: Collect observation events every 3 seconds
            _ = interval.tick() => {
                // Drain any buffered events from the shared event bus
                if let Some(ref rx) = event_rx {
                    while let Ok(event) = rx.try_recv() {
                        // Build structured summary from the full event payload
                        let summary = if let Some(obj) = event.payload.data.as_object() {
                            let mut parts: Vec<String> = Vec::new();
                            for (key, value) in obj {
                                let display = match value {
                                    serde_json::Value::String(s) if s.len() > 300 => format!("{}...", &s[..300]),
                                    v => v.to_string(),
                                };
                                parts.push(format!("{}: {}", key, display));
                            }
                            parts.join(" | ")
                        } else {
                            event.payload.data.to_string()
                        };

                        let finding = format!("{:?}: {} — {}", event.event_type, event.source, summary.chars().take(200).collect::<String>());
                        let importance = match event.event_type {
                            EventType::ApplicationChanged => "high",
                            EventType::ConfigurationFileOpened => "medium",
                            _ => "low",
                        };
                        let panel = GuidancePanel::instance();
                        let _ = panel.add_evidence(
                            &event.provider.to_string(),
                            &finding,
                            &importance.to_string(),
                            event.confidence as f64,
                        ).await;

                        // Also store the event in the timeline so
                        // build_context_system_prompt can read real observation
                        // data instead of an empty context block.
                        let event_type_str = format!("{:?}", event.event_type);
                        let tech = event.provider.to_string();
                        let _ = panel.add_timeline_event(
                            &event_type_str,
                            Some(&event.source),
                            Some(&tech),
                            None,
                            Some(&summary.chars().take(500).collect::<String>()),
                            Some(event.confidence as f64),
                            None,
                        ).await;

                        // Filter out noise events
                        let structured = StructuredEvent {
                            provider: event.provider.to_string(),
                            event_type: format!("{:?}", event.event_type),
                            source: event.source.clone(),
                            summary,
                            payload_json: event.payload.data.clone(),
                        };
                        if !is_noise_event(&structured) {
                            last_events.push(structured);
                        }
                    }
                }

                // Also poll engine errors for immediate error detection
                if let Some(ref eng) = engine {
                    let errors = eng.get_errors();
                    let panel = GuidancePanel::instance();
                    for err in &errors {
                        let _ = panel.add_evidence(
                            &format!("{:?}", err.source),
                            &err.title,
                            &format!("{:?}", err.severity),
                            0.9,
                        ).await;
                    }
                }
            }

            // Phase 2: AI reasoning every 10 seconds
            _ = ai_interval.tick() => {
                // Check for NEW events from the last interval window
                let new_events_count = if let Some(ref rx) = event_rx {
                    let mut count = 0u32;
                    while let Ok(_event) = rx.try_recv() {
                        count += 1;
                    }
                    count
                } else {
                    debug!("No event receiver available — events will not be collected");
                    0
                };

                // DIAGNOSTIC: Log event counts to help debug event flow
                let event_providers: Vec<String> = last_events.iter()
                    .map(|e| e.provider.clone())
                    .collect();
                let provider_counts: std::collections::HashMap<&str, usize> = last_events.iter()
                    .fold(std::collections::HashMap::new(), |mut map, e| {
                        *map.entry(&e.provider).or_insert(0) += 1;
                        map
                    });
                let has_new_events = new_events_count > 0 || !last_events.is_empty();
                info!(
                    "Guidance loop AI tick — new_events_count={}, last_events.len()={}, has_new_events={}, providers={:?}",
                    new_events_count, last_events.len(), has_new_events, event_providers
                );
                for (provider, count) in &provider_counts {
                    info!(
                        "  Events from provider '{}': {}",
                        provider, count
                    );
                }

                // Also fire reasoning if there's been no activity for a while but terminal/screen events exist
                // This handles the case where user is working in a single window (e.g., MobaXterm)
                // and no ApplicationChanged event fires, but TerminalProvider or ScreenCapture
                // is still producing events.
                if !has_new_events && new_events_count == 0 {
                    // Check if we have any terminal or screen events at all in last_events
                    let has_terminal_or_screen = last_events.iter().any(|e| {
                        e.provider == "terminal" || e.provider == "screen_capture"
                    });
                    if !has_terminal_or_screen {
                        debug!("No new events and no terminal/screen activity, skipping AI reasoning for this tick");
                        continue;
                    }
                    // There ARE terminal/screen events but they were collected in a previous cycle
                    // This is still worth reasoning about — user is actively doing something
                }

                // Read AI config
                let (api_key, model, endpoint, provider_name, max_tokens) = {
                    let settings = poll_settings.lock().unwrap();
                    let config = settings.settings.clone();
                    (
                        config.get("ai_provider")
                            .and_then(|p| p.get("api_key")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        config.get("ai_provider")
                            .and_then(|p| p.get("model")).and_then(|v| v.as_str()).unwrap_or("gpt-4o").to_string(),
                        config.get("ai_provider")
                            .and_then(|p| p.get("endpoint")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        config.get("ai_provider")
                            .and_then(|p| p.get("name")).and_then(|v| v.as_str()).unwrap_or("openai").to_string(),
                        config.get("ai_provider")
                            .and_then(|p| p.get("max_tokens")).and_then(|v| v.as_u64()).unwrap_or(512) as usize,
                    )
                };

                let panel = GuidancePanel::instance();

                if api_key.is_empty() {
                    // No AI key configured — skip AI reasoning for this tick.
                    // Observation events are still collected by the engine and stored
                    // in the guidance panel evidence/timeline stores. Once the user
                    // configures an API key, the guidance loop picks it up on the
                    // next 10-second tick.
                    let event_count = last_events.len();
                    debug!("No API key configured, skipping AI reasoning for this tick (events_collected={}, total={})", new_events_count, event_count);
                    continue;
                }

                // API key is set — log the provider info (mask the key)
                let masked_key = if api_key.len() > 4 {
                    format!("{}****{}", &api_key[..3], &api_key[api_key.len()-4..])
                } else {
                    "****".to_string()
                };
                info!("Guidance loop tick: AI reasoning enabled — provider={}, model={}, endpoint={}, masked_key={}", 
                    provider_name, model, endpoint, masked_key);

                info!("Guidance loop tick started — events in queue: {}", last_events.len());

                // Clear the event queue BEFORE building AI context so the next
                // tick starts fresh. This prevents events from accumulating across
                // ticks, which grows the AI request body until nginx rejects it
                // with 413 / Payload Too Large.
                last_events.clear();

                // ── AI-powered cross-context reasoning ──

                // Filter out events that are too noisy for AI context:
                // - clipboard events (user just copied a URL/password, not meaningful context)
                // - rapid active_window events with no meaningful change
                // Also filter out events with empty or trivial summaries
                let filtered_events: Vec<&StructuredEvent> = last_events.iter()
                    .filter(|e| {
                        // Skip clipboard entirely — it's noise for AI reasoning
                        if e.provider == "clipboard" {
                            return false;
                        }
                        // Skip active_window events that are just the same window repeatedly
                        // or have trivial summaries (e.g., empty window title)
                        if e.provider == "active_window" {
                            // Keep if the summary has useful content (non-empty window title)
                            let has_useful_content = !e.summary.trim().is_empty()
                                && e.summary.trim().chars().count() > 2;
                            return has_useful_content;
                        }
                        true
                    })
                    .collect();

                let last_events = filtered_events;

                // Build per-event summaries for AI prompt
                let events_for_ai: Vec<String> = last_events.iter().map(|e| {
                    format!("[{}] {} on {}: {}",
                        e.event_type, e.provider, e.source,
                        e.summary.chars().take(300).collect::<String>())
                }).collect();

                // Extract cross-context data (browser URLs, terminal commands, errors)
                // Handle both old format (flat url/visible_text) and new format (all_browsers array)
                let browser_urls: Vec<&str> = last_events.iter()
                    .filter(|e| e.provider == "browser")
                    .filter_map(|e| {
                        // Try new all_browsers format first
                        if let Some(all_browsers) = e.payload_json.get("all_browsers").and_then(|a| a.as_array()) {
                            return Some(all_browsers.iter()
                                .filter_map(|b| b.get("url").and_then(|u| u.as_str()))
                                .collect::<Vec<_>>());
                        }
                        // Fall back to old flat format
                        e.payload_json.get("url").and_then(|u| u.as_str()).map(|u| vec![u])
                    })
                    .flatten()
                    .filter(|&u| !u.contains("about:blank") && !u.contains("devtools"))
                    .collect();

                let browser_visible_texts: Vec<&str> = last_events.iter()
                    .filter(|e| e.provider == "browser")
                    .filter_map(|e| {
                        if let Some(all_browsers) = e.payload_json.get("all_browsers").and_then(|a| a.as_array()) {
                            return Some(all_browsers.iter()
                                .filter_map(|b| b.get("visible_text").and_then(|v| v.as_str()))
                                .collect::<Vec<_>>());
                        }
                        e.payload_json.get("visible_text").and_then(|v| v.as_str()).map(|t| vec![t])
                    })
                    .flatten()
                    .filter(|&t| !t.is_empty())
                    .collect();

                // Terminal command_text — the last command line (from last_command field)
                // Falls back to command_text if last_command is empty.
                let terminal_cmds: Vec<&str> = last_events.iter()
                    .filter(|e| e.provider == "terminal")
                    .filter_map(|e| {
                        e.payload_json.get("last_command")
                            .or_else(|| e.payload_json.get("command_text"))
                            .and_then(|c| c.as_str())
                            .filter(|s| !s.trim().is_empty())
                    })
                    .collect();

                // Terminal FULL OUTPUT — the complete visible terminal buffer (all output, errors, status)
                // Now simplified: only gets top-level window text (not recursive child windows),
                // so it doesn't include MobaXterm menu bars, file managers, etc.
                let terminal_outputs: Vec<&str> = last_events.iter()
                    .filter(|e| e.provider == "terminal")
                    .filter_map(|e| e.payload_json.get("output").and_then(|o| o.as_str()))
                    .filter(|&o| !o.trim().is_empty())
                    .collect();

                let browser_errors: Vec<&str> = last_events.iter()
                    .filter(|e| e.provider == "browser")
                    .filter_map(|e| {
                        if let Some(all_browsers) = e.payload_json.get("all_browsers").and_then(|a| a.as_array()) {
                            // New all_browsers format: collect errors from each browser
                            let errs: Vec<&str> = all_browsers.iter().flat_map(|b| {
                                b.get("detected_errors").and_then(|d| d.as_array())
                            }).flatten().filter_map(|er| er.get("description").and_then(|d| d.as_str())).collect();
                            if errs.is_empty() { None } else { Some(errs) }
                        } else {
                            // Old flat format: detected_errors is at top level
                            let errs: Vec<&str> = e.payload_json.get("detected_errors")
                                .and_then(|d| d.as_array())
                                .into_iter()
                                .flat_map(|a| a.iter())
                                .filter_map(|er| er.get("description").and_then(|d| d.as_str()))
                                .collect();
                            if errs.is_empty() { None } else { Some(errs) }
                        }
                    })
                    .flatten()
                    .collect();

                // ── Text extraction is still computed for ErrorDetector/EvidencePanel ──
                // but NOT sent to AI reasoning loop (screenshot is the only data source).
                let _session_narrative = || -> String {
                    let mut narrative = String::new();
                    let has_any_context = !browser_urls.is_empty() || !terminal_cmds.is_empty() || !terminal_outputs.is_empty() || !browser_errors.is_empty() || !browser_visible_texts.is_empty();
                    if has_any_context {
                        if !browser_urls.is_empty() {
                            narrative.push_str("🔴 BROWSER CONTEXT (HIGH PRIORITY):\n");
                            for &url in &browser_urls {
                                narrative.push_str(&format!("  🌐 URL: {}\n", url));
                            }
                        }
                        if !browser_visible_texts.is_empty() {
                            narrative.push_str("  📄 RAW PAGE CONTENT:\n");
                            for &text in &browser_visible_texts {
                                let display = if text.len() > 3000 { &text[..3000] } else { text };
                                narrative.push_str(&format!("    {}\n\n", display.replace('\n', " ")));
                            }
                        }
                        if !browser_errors.is_empty() {
                            narrative.push_str("  ⚠️ Pattern-detected errors:\n");
                            for &err in &browser_errors {
                                narrative.push_str(&format!("    🔴 {}\n", err));
                            }
                        }
                        if !terminal_outputs.is_empty() {
                            narrative.push_str("  💻 RAW TERMINAL OUTPUT:\n");
                            for &output in &terminal_outputs {
                                let display = if output.len() > 5000 { &output[..5000] } else { output };
                                narrative.push_str(&format!("    {}\n\n", display));
                            }
                        }
                        if !terminal_cmds.is_empty() {
                            narrative.push_str("  💻 Last terminal command:\n");
                            for &cmd in &terminal_cmds {
                                narrative.push_str(&format!("    > {}\n", cmd));
                            }
                        }
                    }
                    narrative
                }();

                // Keywords for knowledge/skill matching (still needed)
                let keywords_str = events_for_ai.join(" ").to_lowercase();

                // Match against skills and knowledge packs
                let skill_kb_guard = skill_kb.lock().await;
                let matched_skills = skill_kb_guard.match_observations(&keywords_str);
                let skill_context = skill_kb_guard.format_for_prompt(&matched_skills);
                drop(skill_kb_guard);

                let _knowledge_context = {
                    let kp = KnowledgePanel::instance();
                    let matched_packs = kp.match_observations(&keywords_str).await;
                    kp.format_for_prompt(&matched_packs).await
                };

                // ── Build AI system prompt (concise version to reduce payload) ──
                                // The system prompt is intentionally kept compact. Detailed rules are
                                // embedded in matched skill/knowledge packs for the current context.
                                let system_prompt = format!(
                                    "You are Wiki Labs AI Copilot — a technical engineer's teammate who watches their screen and gives proactive guidance.\n\n\
                                    ## CRITICAL: The screenshot is your ONLY data source.\n\
                                    - READ text DIRECTLY from the screenshot — error messages, terminal commands, window titles, browser content.\n\
                                    - DO NOT rely on extracted text or correlated context — it's unreliable. The screenshot is the only real signal.\n\
                                    - If you see an error on screen, address it first.\n\
                                    - If you see multiple things (browser + terminal + IDE), connect the dots between them.\n\n\
                                    ## What to ignore:\n\
                                    - Taskbar, system tray, desktop icons, notification banners.\n\
                                    - Minimized or covered windows. Focus on the main active windows.\n\n\
                                    ## Confidence:\n\
                                    - Call out active errors (red alert, dialog, broken page) immediately.\n\
                                    - If uncertain, stay quiet rather than give generic advice.\n\n\
                                    ## Speak like a teammate:\n\
                                    - Reference actual visible content (error messages, file names, terminal output).\n\
                                    - Be specific. Avoid vague statements. Never suggest commands unless the domain is visible.\n\
                                    - CRITICAL: NEVER mention specific tools/commands unless visible on the screenshot.\n\n\
                                    ## Relevant knowledge (matched skill/knowledge pack):\n{}\n\n\
                                    If you can't determine what the user is doing from the screenshot, stay quiet. Otherwise give short, actionable guidance (1-3 sentences).",
                                    skill_context
                                );

                let provider = OpenAICompatibleProvider::new(
                    &provider_name, &endpoint, &api_key, &model, max_tokens, 128000
                );

                // Get a fresh screenshot for the AI reasoning loop
                // If no screenshot is available (provider hasn't captured yet), skip this tick
                // to avoid hallucinations from text-only context
                let user_message = if let Some(screenshot) = crate::observation::get_last_screenshot_async().await {
                    wikilabs_ai::provider::AiMessage {
                        role: "user".to_string(),
                        content: serde_json::json!([
                            {
                                "type": "text",
                                "text": format!("Look at the screenshot and give specific, actionable recommendations. The currently focused window is: {:?}. Here is a screenshot of what the user is looking at right now — use it to give precise, contextual guidance.", &screenshot.focused_window)
                            },
                            {
                                "type": "image_url",
                                "image_url": {
                                    "url": format!("data:image/jpeg;base64,{}", screenshot.data_base64)
                                }
                            }
                        ])
                    }
                } else {
                    // No screenshot available — skip AI reasoning entirely
                    // Sending a generic prompt without a screenshot causes hallucinations
                    debug!("No screenshot available for AI reasoning, skipping tick");
                    continue;
                };

                let ai_request = wikilabs_ai::provider::AiRequest {
                    model: model.clone(),
                    messages: vec![
                        wikilabs_ai::provider::AiMessage { role: "system".to_string(), content: serde_json::json!(system_prompt) },
                        user_message,
                    ],
                    tools: vec![],
                    temperature: None,
                    max_tokens: Some(max_tokens),
                    stream: None,
                };

                match provider.chat(ai_request).await {
                    Ok(response) => {
                        // Extract text from the response content (may be string or array — take first text part)
                        let suggestion_content = match &response.message.content {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Array(parts) => {
                                // Multi-modal response: take first text part
                                parts.iter().find_map(|p| {
                                    if let Some(obj) = p.as_object() {
                                        if obj.get("type").and_then(|t| t.as_str()) == Some("text") {
                                            obj.get("text").and_then(|t| t.as_str())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }).unwrap_or("")
                            }
                            _ => "",
                        };

                        if panel.should_skip_suggestion(suggestion_content).await {
                            tracing::debug!("Skipping repetitive AI suggestion");
                            continue;
                        }

                        panel.record_suggestion(suggestion_content).await;

                        // Also create a structured RecommendationCard so it appears
                        // in the active_recommendations() list (frontend card panel).
                        let card_title = suggestion_content.chars().take(80).collect::<String>();
                        let _ = panel.add_recommendation(
                            &card_title,
                            suggestion_content,
                            "AI Copilot generated this suggestion based on real-time observation.",
                            "general",
                            "guidance",
                            0.75,
                            CardRiskLevel::Low,
                            vec![],
                            None,
                        ).await;

                        // Also send the suggestion into the AI chat thread so the user sees it
                        let chat_msg = ChatMessage {
                            id: format!("suggestion-{}", chrono::Utc::now().timestamp_millis()),
                            role: "assistant".to_string(),
                            content: format!("[💡 AI Copilot Suggestion]\n\n{}", suggestion_content),
                            created_at: chrono::Utc::now().to_rfc3339(),
                            workspace_id: None,
                        };
                        let settings_clone = poll_settings.clone();
                        {
                            let mut settings = settings_clone.lock().unwrap();
                            settings.messages.lock().unwrap().push(chat_msg);
                        }

                        // Open advice chat window if not already open
                        if let Some(ref ah) = app_handle {
                            if ah.get_webview_window("advice-chat").is_none() {
                                let handle_clone = (**ah).clone();
                                // Spawn a task to open the window (non-blocking)
                                tokio::task::spawn_blocking(move || {
                                    if let Some(w) = handle_clone.get_webview_window("advice-chat") {
                                        let _ = w.show();
                                    }
                                });
                            }
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "AI recommendation failed");
                    }
                }
            }
        }
    }
}