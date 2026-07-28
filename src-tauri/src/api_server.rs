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
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{error, info, warn};
use wikilabs_ai::AiProvider;
use tauri::AppHandle;

/// Build a system prompt that includes the AI's own observation context,
/// recent recommendations with reasoning, and session state.
/// This makes the AI truly aware of what it has been observing and recommending.
async fn build_context_system_prompt(
    chat_history: &[ChatMessage],
) -> Option<String> {
    let panel = guidance_panel::GuidancePanel::instance();
    let mut parts: Vec<String> = Vec::new();

    // ── Recent observation events ──
    let recent_events = panel.get_recent_events(60).await;
    if !recent_events.is_empty() {
        let mut lines = Vec::new();
        for e in recent_events.iter().take(30) {
            let desc = e.description.as_deref().unwrap_or("");
            let tech = e.technology.as_deref().unwrap_or(e.event_type.as_str());
            lines.push(format!("- {} on {}: {}", e.event_type, tech, desc));
        }
        if !lines.is_empty() {
            parts.push(format!(
                "## Recent Observations (last 60 minutes)\n{}\n",
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
            lines.push(format!(
                "- **{}** ({}) — Reason: {} | Next: {}",
                r.title, status_str, r.reason, next_step
            ));
        }
        if !lines.is_empty() {
            parts.push(format!(
                "## My Recent Recommendations\n{}\n",
                lines.join("\n")
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
            "{}\n\nYou are the Wiki Labs AI Copilot. You've been actively observing the user's activity in real-time through terminal sessions, browser activity, window changes, and more. The observations and recommendations above are YOUR own — they're what you've noticed and suggested based on what you've seen. Continue your analysis and conversation with full awareness of your prior observations.",
            context_block
        )
    } else {
        // No conversation history yet — just set the context
        format!(
            "{}\n\nYou are the Wiki Labs AI Copilot assistant. You've been actively observing the user's activity in real-time. The observations and recommendations above are YOUR own — what you've noticed and suggested based on what you've seen.",
            context_block
        )
    };

    Some(session_context)
}

use crate::guidance_panel;
use crate::knowledge_panel::KnowledgePanel;
use crate::skill_knowledge::create_skill_knowledge_base;
use crate::skill_management::SkillManagementPanel;

/// Request wrapper sent from the frontend.
#[derive(Debug, Deserialize)]
pub struct ApiRequest {
    #[serde(default)]
    pub params: Value,
}

/// Shared state for the HTTP server.
#[derive(Clone)]
pub struct ApiServerState {
    pub settings: Arc<Mutex<ApiServerSettings>>,
    pub config_path: Arc<Mutex<Option<PathBuf>>>,
    /// Optional AppHandle for sending native notifications.
    pub app_handle: Option<Arc<tauri::AppHandle>>,
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
        // System commands
        "get_status" => handle_get_status().await,
        "observation_get_status" => handle_observation_get_status(&state).await,
        "observation_get_context" => handle_observation_get_context().await,
        "observation_start" => handle_observation_start(&state).await,
        "observation_stop" => handle_observation_stop(&state).await,
        "hide_main_window" => handle_hide_main_window(&state).await,
        "advice_chat_open" => handle_advice_chat_open(&state).await,
        other => {
            warn!(other, "Unknown API method");
            (StatusCode::BAD_REQUEST, api_response(false, None, Some(format!("Unknown method: {}", other))))
        }
    };

    info!(method, "API request completed");
    (status, body)
}

async fn handle_test_connection(_state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let api_key = params.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if api_key.is_empty() {
        return (StatusCode::OK, api_response(false, None, Some("API key is required".to_string())));
    }
    let endpoint = params.get("endpoint").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if endpoint.is_empty() {
        return (StatusCode::OK, api_response(false, None, Some("Endpoint is required".to_string())));
    }
    // Actually test the connection by hitting the /models endpoint
    // Normalize: if endpoint ends with /v1, just append /models; if just a base URL, append /v1/models
    let url = if endpoint.ends_with("/v1") {
        format!("{}{}/models", endpoint.trim_end_matches('/'), "")
    } else if endpoint.contains("/v1/") {
        format!("{}{}/models", endpoint.trim_end_matches('/'), "")
    } else {
        format!("{}/v1/models", endpoint.trim_end_matches('/'))
    };
    info!(endpoint, url, "Testing AI provider connection");
    match reqwest::Client::new()
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Provider connection verified");
            (StatusCode::OK, api_response(true, Some(serde_json::json!(true)), None))
        }
        Ok(response) => {
            let status = response.status();
            error!("Provider health check failed: {}", status);
            (StatusCode::OK, api_response(false, None, Some(format!("Connection refused or bad response: {}", status))))
        }
        Err(e) => {
            error!("Provider connection failed: {}", e);
            (StatusCode::OK, api_response(false, None, Some(format!("Cannot reach endpoint: {}", e))))
        }
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
    settings.settings = params.clone();

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

        // ── Build context-aware message array ──
        // 1. Get chat history (already saved user message above)
        let history = {
            let settings_ref = state.settings.lock().unwrap();
            let msgs = settings_ref.messages.lock().unwrap();
            msgs.clone()
        };

        // 2. Build system prompt with observation context + recommendations
        let system_prompt = {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime for system prompt");
            rt.block_on(build_context_system_prompt(&history))
        };

        // 3. Build the messages array for the AI
        let mut messages: Vec<wikilabs_ai::provider::AiMessage> = Vec::new();

        // Add system prompt if we have observation context
        if let Some(sys) = system_prompt {
            messages.push(wikilabs_ai::provider::AiMessage {
                role: "system".to_string(),
                content: sys,
            });
        }

        // Add all prior chat messages as history
        for msg in history.iter() {
            messages.push(wikilabs_ai::provider::AiMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        // Add the current user message
        messages.push(wikilabs_ai::provider::AiMessage {
            role: "user".to_string(),
            content: message.clone(),
        });

        let ai_request = wikilabs_ai::provider::AiRequest {
            model: model.clone(),
            messages,
            tools: vec![],
            temperature: None,
            max_tokens: Some(max_tokens as usize),
            stream: None,
        };

        // Run the AI call on a separate thread with its own tokio runtime
        // to avoid blocking the axum server's runtime
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
                (aid, acreated, response.message.content)
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
    if endpoint.is_empty() {
        return (StatusCode::OK, api_response(false, None, Some("Endpoint is required".to_string())));
    }

    // Normalize URL: ensure it ends with /v1/models
    let url = if endpoint.ends_with("/v1") || endpoint.contains("/v1/") {
        format!("{}/models", endpoint.trim_end_matches('/'))
    } else {
        format!("{}/v1/models", endpoint.trim_end_matches('/'))
    };

    info!(endpoint, url, "Fetching models from provider");

    let mut builder = reqwest::Client::new().get(&url).header("Content-Type", "application/json");
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {}", api_key));
    }

    match builder.timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(response) if response.status().is_success() => {
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
                    info!(count = models.len(), "Models fetched successfully");
                    (StatusCode::OK, api_response(true, Some(serde_json::json!(models)), None))
                }
                Err(e) => {
                    error!("Failed to parse models response: {}", e);
                    (StatusCode::OK, api_response(false, None, Some(format!("Failed to parse response: {}", e))))
                }
            }
        }
        Ok(response) => {
            let status = response.status();
            error!("Failed to fetch models: HTTP {}", status);
            // Try fallback: /models (without /v1)
            let base_url = endpoint.trim_end_matches('/').trim_end_matches("/v1");
            let fallback_url = format!("{}/models", base_url);
            info!("Trying fallback URL: {}", fallback_url);
            let mut fb_builder = reqwest::Client::new().get(&fallback_url).header("Content-Type", "application/json");
            if !api_key.is_empty() {
                fb_builder = fb_builder.header("Authorization", format!("Bearer {}", api_key));
            }
            match fb_builder.timeout(std::time::Duration::from_secs(10)).send().await {
                Ok(fb_resp) if fb_resp.status().is_success() => {
                    match fb_resp.json::<Value>().await {
                        Ok(data) => {
                            let models = data.get("data")
                                .and_then(|d| d.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|m| m.get("id").and_then(|id| id.as_str().map(String::from)))
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();
                            (StatusCode::OK, api_response(true, Some(serde_json::json!(models)), None))
                        }
                        Err(e) => {
                            (StatusCode::OK, api_response(false, None, Some(format!("Fallback parse failed: {}", e))))
                        }
                    }
                }
                _ => {
                    (StatusCode::OK, api_response(false, None, Some(format!("HTTP error: {}", status))))
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to provider: {}", e);
            (StatusCode::OK, api_response(false, None, Some(format!("Cannot reach endpoint: {}", e))))
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

async fn handle_observation_start(_state: &ApiServerState) -> (StatusCode, String) {
    info!("Observation start requested");
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"status": "started"})), None))
}

async fn handle_observation_stop(_state: &ApiServerState) -> (StatusCode, String) {
    info!("Observation stop requested");
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"status": "stopped"})), None))
}

/// Hide the main window (minimize to tray).
async fn handle_hide_main_window(state: &ApiServerState) -> (StatusCode, String) {
    info!("Hide main window (minimize to tray) requested");
    if let Some(ref app_handle) = state.app_handle {
        if let Some(window) = (**app_handle).get_webview_window("main") {
            let _ = window.hide();
        }
    }
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"hidden": true})), None))
}

/// Open the floating advice chat window on the right side.
/// Creates the window if it doesn't exist (lazy init), then shows and focuses it.
async fn handle_advice_chat_open(state: &ApiServerState) -> (StatusCode, String) {
    info!("Opening advice chat floating window");
    if let Some(ref app_handle) = state.app_handle {
        let ah = (**app_handle).clone();
        // If the window already exists, just show and focus it
        if let Some(window) = ah.get_webview_window("advice-chat") {
            let _ = window.show();
            let _ = window.set_focus();
            return (StatusCode::OK, api_response(true, Some(serde_json::json!({"opened": true})), None));
        }
        // Window doesn't exist — create it (lazy init)
        info!("advice-chat window not found, creating it now");
        let url = tauri::WebviewUrl::External("http://localhost:1420/advice-chat".parse::<url::Url>().unwrap());
        let result = tauri::WebviewWindowBuilder::new(&ah, "advice-chat", url)
            .title("AI Copilot — Live Advice")
            .inner_size(400.0, 520.0)
            .resizable(true)
            .decorations(true)
            .always_on_top(true)
            .build();
        if let Ok(window) = result {
            // Position on the right side of the screen (vertically centered)
            if let Ok(Some(monitor)) = window.current_monitor() {
                let width = 400.0;
                let height = 520.0;
                let x = monitor.size().width as f64 - width - 10.0;
                let y = (monitor.size().height as f64 - height) / 2.0;
                let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
            }
            let _ = window.show();
            let _ = window.set_focus();
        } else {
            error!("Failed to create advice-chat window");
        }
    }
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"opened": true})), None))
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
pub fn create_router(state: ApiServerState) -> Router {
    info!("[API] Creating router with state...");
    
    // Debug: list all registered routes
    info!("[API] Route creation complete — now registering fallback");
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let advice_html = include_str!("../assets/advice-chat.html");
    let router = Router::new()
        .route("/api/commands/:method", post(api_handler))
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(|| async { 
            let ready = crate::api_ready::is_server_ready();
            format!("{{\"ready\":{}}}", if ready { "true" } else { "false" })
        }))
        .route("/advice-chat", get(move || async move { Html(advice_html.to_string()) }))
        // Serve static assets (JS, CSS, images) for the advice-chat window
        .nest_service("/assets", ServeDir::new("../assets"))
        .layer(cors)
        .fallback(|method: axum::http::Method, uri: axum::http::Uri| async move {
            warn!("[API] FALLBACK HIT — method={} uri={}", method, uri);
            (StatusCode::NOT_FOUND, format!("No route for {} {}", method, uri))
        })
        .with_state(state);
    
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
    // Clone app_handle for use inside the AI loop closure (state will be consumed by the move)
    let app_handle_for_loop = app_handle.clone();

    // Track whether we have already notified the user about missing AI key
    let ai_has_notified = Arc::new(std::sync::Mutex::new(false));

    let state = ApiServerState {
        settings: Arc::new(Mutex::new(ApiServerSettings::new())),
        config_path: Arc::new(Mutex::new(config_path.clone())),
        app_handle,
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
    let router = create_router(state);

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
    let skill_kb_clone = skill_kb.clone();

    // Initialize knowledge packs from data directory
    let knowledge_dir_to_use = knowledge_path.clone().or_else(|| {
        config_path.as_ref().and_then(|cp| cp.parent().map(|p| p.join("knowledge")))
    });

    // Store knowledge path for later async initialization
    let kdir = knowledge_dir_to_use.clone();
    let kdir_str = kdir.as_ref().map(|d| d.to_string_lossy().to_string());

    let addr = format!("127.0.0.1:{port}");

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
            // Initialize knowledge packs inside the tokio runtime
            if let Some(ref kdir_path) = kdir_str {
                info!(dir = %kdir_path, "Loading knowledge packs");
                let panel = KnowledgePanel::instance();
                let kdir = kdir_path.clone();
                rt.block_on(async {
                    if let Err(e) = panel.initialize(&kdir).await {
                        error!(error = %e, "Failed to load knowledge packs");
                    }
                });
            }

            // Initialize observation engine — providers already registered in main.rs
            // via observation::init_observation_engine(). We reuse the shared engine here.
            info!("Observation engine initialized (shared from main.rs)");

            // Reset guidance state on startup — fresh session, zero evidence
            {
                let panel = guidance_panel::GuidancePanel::instance();
                let _ = rt.block_on(panel.clear_all());
                info!("Guidance panel state cleared on startup");
            }

            // ── Background observation polling + AI reasoning loop ────────────
            let poll_settings = obs_settings.clone();
            let skill_kb_for_loop = skill_kb_clone.clone();
            let ai_notify_flag = ai_has_notified.clone();
            rt.spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                let mut ai_interval = tokio::time::interval(std::time::Duration::from_secs(15));

                // Structured event summaries (rich JSON payloads, not flat strings)
                #[derive(Clone)]
                struct StructuredEvent {
                    provider: String,
                    event_type: String,
                    source: String,
                    summary: String,
                    payload_json: serde_json::Value,
                }
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
                    if event.provider == "ActiveWindow" && (
                        event.source == "inactive"
                        || summary_lower.contains("no_window_info_available")
                        || summary_lower.contains("platform:")
                    ) {
                        return true;
                    }
                    false
                };

                // Subscribe to the shared observation engine's event bus
                let event_rx = crate::observation::get_event_receiver();
                let engine = crate::observation::get_observation_engine();

                loop {
                    tokio::select! {
                        // Phase 1: Collect observation events every 5 seconds
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
                                        wikilabs_observation::event::EventType::ApplicationChanged => "high",
                                        wikilabs_observation::event::EventType::ConfigurationFileOpened => "medium",
                                        wikilabs_observation::event::EventType::ClipboardChanged => "low",
                                        _ => "low",
                                    };
                                    let panel = guidance_panel::GuidancePanel::instance();
                                    let _ = panel.add_evidence(
                                        &event.provider.to_string(),
                                        &finding,
                                        &importance.to_string(),
                                        event.confidence as f64,
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
                                        if last_events.len() > 30 { last_events.remove(0); }
                                    }
                                }
                            }

                            // Also poll engine errors for immediate error detection
                            if let Some(ref eng) = engine {
                                let errors = eng.get_errors();
                                let panel = guidance_panel::GuidancePanel::instance();
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

                        // Phase 2: AI reasoning every 30 seconds
                        _ = ai_interval.tick() => {
                            if last_events.is_empty() { continue; }

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

                            let panel = guidance_panel::GuidancePanel::instance();

                            if api_key.is_empty() {
                                // No AI key configured — notify user once, then skip
                                let mut notified = ai_notify_flag.lock().unwrap();
                                if !*notified {
                                    // Set status so frontend can display it
                                    {
                                        let mut settings = poll_settings.lock().unwrap();
                                        if let Some(ai) = settings.settings.get_mut("ai_provider") {
                                            if let Some(obj) = ai.as_object_mut() {
                                                obj.insert("ai_connection_status".to_string(), serde_json::json!("not_configured"));
                                            }
                                        }
                                    }
                                    // Show toast notification
                                    {
                                        let flag_clone = ai_notify_flag.clone();
                                        let mut flag_guard = flag_clone.lock().unwrap();
                                        *flag_guard = true;
                                    }
                                    if let Some(ref ah) = app_handle_for_loop {
                                        let handle_clone = (**ah).clone();
                                        tokio::task::spawn_blocking(move || {
                                            if let Some(w) = handle_clone.get_webview_window("main") {
                                                let _ = w.show();
                                            }
                                        });
                                        // Send notification event to frontend
                                        let _ = ah.emit("ai-connection-status", serde_json::json!({
                                            "status": "not_configured",
                                            "message": "AI Copilot is not connected — no API key configured. Go to Settings to connect your AI provider for intelligent guidance."
                                        }));
                                    }
                                }
                                continue;
                            }

                            // ── AI-powered cross-context reasoning ──

                            // Build per-event summaries for AI prompt
                            let events_for_ai: Vec<String> = last_events.iter().map(|e| {
                                format!("[{}] {} on {}: {}",
                                    e.event_type, e.provider, e.source,
                                    e.summary.chars().take(300).collect::<String>())
                            }).collect();

                            // Extract cross-context data (browser URLs, terminal commands, errors)
                            let browser_urls: Vec<&str> = last_events.iter()
                                .filter(|e| e.provider == "Browser")
                                .filter_map(|e| e.payload_json.get("url").and_then(|u| u.as_str()))
                                .filter(|&u| !u.contains("about:blank") && !u.contains("devtools"))
                                .collect();

                            let terminal_cmds: Vec<&str> = last_events.iter()
                                .filter(|e| e.provider == "Terminal")
                                .filter_map(|e| e.payload_json.get("command_text").and_then(|c| c.as_str()))
                                .collect();

                            let browser_errors: Vec<&str> = last_events.iter()
                                .filter(|e| e.provider == "Browser")
                                .filter_map(|e| e.payload_json.get("detected_errors"))
                                .filter_map(|e| e.as_array())
                                .flat_map(|arr| arr.iter().filter_map(|er| er.get("description").and_then(|d| d.as_str())))
                                .collect();

                            // Build correlated session narrative — lead with browser errors (highest priority for actionable guidance)
                            let mut session_narrative = String::new();
                            let has_any_context = !browser_urls.is_empty() || !terminal_cmds.is_empty() || !browser_errors.is_empty();
                            if has_any_context {
                                // Always lead with browser context — this is what the engineer is looking at
                                if !browser_urls.is_empty() {
                                    session_narrative.push_str("🔴 BROWSER CONTEXT (HIGH PRIORITY — engineer is looking at this):\n");
                                    for &url in &browser_urls {
                                        session_narrative.push_str(&format!("  🌐 URL: {}\n", url));
                                    }
                                }
                                // CRITICAL: Browser errors are the most actionable signal — the engineer is actively seeing these
                                if !browser_errors.is_empty() {
                                    session_narrative.push_str("  ⚠️ ACTIVE ERRORS on the engineer's screen:\n");
                                    for &err in &browser_errors {
                                        session_narrative.push_str(&format!("    🔴 {}\n", err));
                                    }
                                }
                                if !terminal_cmds.is_empty() {
                                    session_narrative.push_str("  💻 TERMINAL ACTIVITY (engineer is doing this):\n");
                                    for &cmd in &terminal_cmds {
                                        session_narrative.push_str(&format!("    > {}\n", cmd));
                                    }
                                }
                            }

                            // Keywords for knowledge/skill matching
                            let keywords_str = events_for_ai.join(" ").to_lowercase();

                            // Match against skills and knowledge packs
                            let skill_kb_guard = skill_kb_for_loop.lock().await;
                            let matched_skills = skill_kb_guard.match_observations(&keywords_str);
                            let skill_context = skill_kb_guard.format_for_prompt(&matched_skills);
                            drop(skill_kb_guard);

                            let _knowledge_context = {
                                let kp = KnowledgePanel::instance();
                                let matched_packs = kp.match_observations(&keywords_str).await;
                                kp.format_for_prompt(&matched_packs).await
                                                            };

                                                            // ── Build AI system prompt ──
                                                            let system_prompt = format!(
                                                                "You are Wiki Labs AI Copilot — a helpful teammate who watches what someone is doing and gives proactive, relevant guidance.\n\
                                                                You can see: applications they switch to, commands they type, browser tabs/URLs/errors, files they open, terminal activity.\n\
                                                                ## Your job\n\
                                                                Analyze the correlated session context and give ONE specific, actionable suggestion.\n\
                                                                ## How to think\n\
                                                                Connect dots across data sources. If they see an error on their screen, that's what they need help with most.\n\

                                                                ## Priority Order:\n\
                                                                1. 🔴 Active errors or problems on their screen — fix that first\n\
                                                                2. 💻 Commands they're running — what are they trying to do?\n\
                                                                3. 🖥️ What app has focus — what context are they in?\n\

                                                                ## Connect the dots:\n\
                                                                - Error on screen + terminal command → suggest next diagnostic step\n\
                                                                - Dashboard showing issue + system commands → suggest related checks they haven't done\n\
                                                                - Config file edit + validation command → suggest what to verify next\n\
                                                                - Search for something + commands about it → connect the search to the action\n\
                                                                - Multiple things happening at once → figure out the underlying goal and help with that\n\

                                                                ## Speak like a real person — GOOD examples:\n\
                                                                - \"I see the Nagios page returned an error, and you're checking `systemctl status nagios`. Check the database first — `systemctl status mysqld` — Nagios stores data in MySQL.\"\n\
                                                                - \"You're editing Nginx config and ran `nginx -t`. Now reload with `systemctl reload nginx` and check. If it still breaks, `tail -f /var/log/nginx/error.log`.\"\n\
                                                                - \"Docker container in CrashLoopBackOff + you ran `docker logs`. Also check exit code: `docker inspect <container> | grep ExitCode` to see WHY it crashed.\"\n\
                                                                - \"Looking at K8s dashboard with pods failing + `kubectl get pods`. Run `kubectl describe pod <pod-name>` to see the actual error.\"\n\
                                                                - \"You're checking MySQL status. If it's down, check the error log — `journalctl -u mysql --no-pager -n 50` — that usually tells you why it crashed.\"\n\
                                                                - \"I see a database error on the Grafana page. Grafana uses SQLite/PostgreSQL — check the DB first: `systemctl status <db-service>` and `journalctl -u <db-service> -n 30`.\"\n\
                                                                - \"You're looking at a 404 page — check if the URL is correct or if the server needs to be started.\"\n\
                                                                - \"I see you're setting up a new project — have you considered using a package manager to handle dependencies instead of manual downloads?\"\

                                                                ## Don't sound robotic — BAD examples:\n\
                                                                - \"You appear to be working on something.\"\n\
                                                                - \"I observed activity in your browser.\"\n\
                                                                - \"You're running bash commands repeatedly.\"\n\
                                                                - \"I see you're busy with the terminal.\"\n\
                                                                - Generic advice that doesn't reference what they're actually seeing\n\
                                                                - Suggestions that ignore visible errors and focus on something else\n\

                                                                ## Relevant knowledge (from loaded skill/knowledge packs):\n\
                                                                {}\n\

                                                                ## Correlated session context:\n\
                                                                {}\n\

                                                                ## Recent observations:\n\
                                                                {}\n\

                                                                Give ONE short piece of guidance (1-3 sentences). Be specific, actionable, and conversational — like a knowledgeable teammate sitting next to them.\n\
                                                                If you can connect the dots across what they're doing, do it. Never repeat the same type of suggestion.\"\n\

                                                                If you truly can't tell what they're doing, stay quiet or ask a brief question.",
                                                                skill_context,
                                                                session_narrative,
                                                                events_for_ai.join("\n"),
                                                            );

                            let provider = wikilabs_ai::provider::OpenAICompatibleProvider::new(
                                &provider_name, &endpoint, &api_key, &model, max_tokens, 128000
                            );
                            let ai_request = wikilabs_ai::provider::AiRequest {
                                model: model.clone(),
                                messages: vec![
                                    wikilabs_ai::provider::AiMessage { role: "system".to_string(), content: system_prompt },
                                    wikilabs_ai::provider::AiMessage { role: "user".to_string(), content: "Analyze the user's correlated session and give ONE specific recommendation.".to_string() },
                                ],
                                tools: vec![],
                                temperature: None,
                                max_tokens: Some(max_tokens),
                                stream: None,
                            };

                            match provider.chat(ai_request).await {
                                Ok(response) => {
                                    let suggestion_content = &response.message.content;

                                    if panel.should_skip_suggestion(suggestion_content).await {
                                        tracing::debug!("Skipping repetitive AI suggestion");
                                        continue;
                                    }

                                    // Remove previous AI recommendation so each tick replaces it
                                    let all = panel.all_recommendations().await;
                                    for prev in &all {
                                        if prev.title.starts_with("🧭") {
                                            let _ = panel.dismiss_recommendation(&prev.id).await;
                                        }
                                    }

                                    let title = if suggestion_content.len() > 60 {
                                        format!("🧭 {}", &suggestion_content[..60].replace('\n', " ").trim())
                                    } else {
                                        format!("🧭 {}", suggestion_content.replace('\n', " ").trim())
                                    };

                                    let _ = panel.add_recommendation(
                                        &title,
                                        suggestion_content,
                                        "AI analyzed correlated session context",
                                        "AI Copilot",
                                        "General",
                                        0.8,
                                        guidance_panel::CardRiskLevel::Low,
                                        vec![],
                                        None,
                                    ).await;
                                    panel.record_suggestion(suggestion_content).await;
                                    
                                    // Open advice chat window if not already open
                                    if let Some(ref ah) = app_handle_for_loop {
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
            });

            let result = rt.block_on(async {
                let listener = tokio::net::TcpListener::bind(&addr)
                    .await
                    .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

                info!(addr, "API server listening");

                // Signal to frontend that the server is ready to handle requests
                crate::api_ready::mark_server_ready();

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