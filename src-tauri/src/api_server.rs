//! HTTP server that bridges frontend REST API calls to local state.
//!
//! The frontend SPA hardcodes calls to `http://localhost:1420/api/commands/*`.
//! This server intercepts those calls and serves local state + chat history.

use axum::{
    extract::{Path, State},
    extract::Json,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};
use wikilabs_ai::AiProvider;

use crate::guidance_panel;
use crate::knowledge_panel::{KnowledgePanel, PackInfo, ValidationReport};
use crate::skill_management::{SkillCard, SkillManagementPanel};
use crate::config::AiProviderConfig;

use observation::provider::ProviderRegistry;
use observation::app_monitor::ActiveWindowProvider;
use observation::browser::BrowserProvider;
use observation::clipboard::ClipboardProvider;
use observation::terminal::TerminalProvider;
use observation::screen_capture::ScreenCaptureProvider;
use observation::file_observer::FileObserverProvider;

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
        "guidance_set_mode" => handle_guidance_set_mode(req.params).await,
        "guidance_get_mode" => handle_guidance_get_mode().await,
        "guidance_get_available_modes" => handle_guidance_get_available_modes().await,
        "guidance_clear_all" => handle_guidance_clear_all().await,
        // Observation commands
        "observation_get_status" => handle_observation_get_status(&state).await,
        "observation_start" => handle_observation_start(&state).await,
        "observation_stop" => handle_observation_stop(&state).await,
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
    
    // Persist to disk
    if let Ok(config_path) = state.config_path.lock() {
        if let Some(ref path) = *config_path {
            match fs::write(path, serde_json::to_string_pretty(&params).unwrap_or_default()) {
                Ok(_) => info!("Settings persisted to disk: {}", path.display()),
                Err(e) => error!(error = %e, "Failed to persist settings to disk"),
            }
        }
    }
    
    (StatusCode::OK, api_response(true, Some(serde_json::json!({ "status": "updated" })), None))
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

        let ai_request = wikilabs_ai::provider::AiRequest {
            model: model.clone(),
            messages: vec![wikilabs_ai::provider::AiMessage {
                role: "user".to_string(),
                content: message.clone(),
            }],
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
    let url = if endpoint.ends_with("/v1") {
        format!("{}/models", endpoint.trim_end_matches('/'))
    } else if endpoint.contains("/v1/") {
        format!("{}/models", endpoint.trim_end_matches('/'))
    } else {
        format!("{}/v1/models", endpoint.trim_end_matches('/'))
    };

    info!(endpoint, url, "Fetching models from provider");

    let mut builder = reqwest::Client::new().get(&url).header("Content-Type", "application/json");
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {}", api_key));
    }

    match builder.timeout(std::time::Duration::from_secs(10)).send().await {
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
    let recs = guidance_panel::guidance_get_active_recommendations();
    let value = serde_json::to_value(recs).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_get_all_recommendations() -> (StatusCode, String) {
    let recs = guidance_panel::guidance_get_all_recommendations();
    let value = serde_json::to_value(recs).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_dismiss_recommendation(params: Value) -> (StatusCode, String) {
    let rec_id = params.get("rec_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match guidance_panel::guidance_dismiss_recommendation(rec_id) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

async fn handle_guidance_update_recommendation_status(params: Value) -> (StatusCode, String) {
    let rec_id = params.get("rec_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let status = params.get("status").and_then(|v| serde_json::from_value::<guidance_panel::RecommendationStatus>(v.clone()).ok());
    match status {
        Some(s) => match guidance_panel::guidance_update_recommendation_status(rec_id, s) {
            Ok(_) => (StatusCode::OK, api_response(true, None, None)),
            Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
        },
        None => (StatusCode::OK, api_response(false, None, Some("Invalid status".to_string()))),
    }
}

async fn handle_guidance_get_evidence_status() -> (StatusCode, String) {
    let status = guidance_panel::guidance_get_evidence_status();
    let value = serde_json::to_value(status).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_add_evidence(params: Value) -> (StatusCode, String) {
    let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let finding = params.get("finding").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let importance = params.get("importance").and_then(|v| v.as_str()).unwrap_or("medium").to_string();
    let confidence = params.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
    match guidance_panel::guidance_add_evidence(source, finding, importance, confidence) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

async fn handle_guidance_mark_missing(params: Value) -> (StatusCode, String) {
    let needed = params.get("needed").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = params.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let importance = params.get("importance").and_then(|v| v.as_str()).unwrap_or("medium").to_string();
    match guidance_panel::guidance_mark_missing(needed, description, importance) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

async fn handle_guidance_get_workflow_progress() -> (StatusCode, String) {
    let progress = guidance_panel::guidance_get_workflow_progress();
    let value = serde_json::to_value(progress).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_start_workflow(params: Value) -> (StatusCode, String) {
    let workflow_id = params.get("workflow_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let workflow_name = params.get("workflow_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let problem_category = params.get("problem_category").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let steps = params.get("steps").and_then(|v| serde_json::from_value::<Vec<guidance_panel::WorkflowStepCard>>(v.clone()).ok()).unwrap_or_default();
    match guidance_panel::guidance_start_workflow(workflow_id, workflow_name, problem_category, steps) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

async fn handle_guidance_complete_step(params: Value) -> (StatusCode, String) {
    let step_id = params.get("step_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let observation = params.get("observation").and_then(|v| v.as_str()).map(|s| s.to_string());
    match guidance_panel::guidance_complete_step(step_id, observation) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

async fn handle_guidance_get_timeline() -> (StatusCode, String) {
    let timeline = guidance_panel::guidance_get_timeline();
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
    match guidance_panel::guidance_add_timeline_event(event_type, title, technology, finding, description, confidence, recommendation_id) {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

async fn handle_guidance_get_recent_events(params: Value) -> (StatusCode, String) {
    let minutes = params.get("minutes").and_then(|v| v.as_u64()).unwrap_or(60);
    let events = guidance_panel::guidance_get_recent_events(minutes);
    let value = serde_json::to_value(events).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_record_feedback(params: Value) -> (StatusCode, String) {
    let recommendation_id = params.get("recommendation_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let feedback_type = params.get("feedback_type").and_then(|v| serde_json::from_value::<guidance_panel::FeedbackType>(v.clone()).ok());
    let notes = params.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
    match feedback_type {
        Some(ft) => match guidance_panel::guidance_record_feedback(recommendation_id, ft, notes) {
            Ok(_) => (StatusCode::OK, api_response(true, None, None)),
            Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
        },
        None => (StatusCode::OK, api_response(false, None, Some("Invalid feedback_type".to_string()))),
    }
}

async fn handle_guidance_get_feedback_stats() -> (StatusCode, String) {
    let stats = guidance_panel::guidance_get_feedback_stats();
    let value = serde_json::to_value(stats).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_set_mode(params: Value) -> (StatusCode, String) {
    let mode = params.get("mode").and_then(|v| serde_json::from_value::<guidance_panel::CopilotMode>(v.clone()).ok());
    match mode {
        Some(m) => match guidance_panel::guidance_set_mode(m) {
            Ok(_) => (StatusCode::OK, api_response(true, None, None)),
            Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
        },
        None => (StatusCode::OK, api_response(false, None, Some("Invalid mode".to_string()))),
    }
}

async fn handle_guidance_get_mode() -> (StatusCode, String) {
    let mode = guidance_panel::guidance_get_mode();
    let value = serde_json::to_value(mode).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_get_available_modes() -> (StatusCode, String) {
    let modes = guidance_panel::guidance_get_available_modes();
    let value = serde_json::to_value(modes).unwrap_or_default();
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_guidance_clear_all() -> (StatusCode, String) {
    match guidance_panel::guidance_clear_all() {
        Ok(_) => (StatusCode::OK, api_response(true, None, None)),
        Err(e) => (StatusCode::OK, api_response(false, None, Some(e))),
    }
}

async fn handle_observation_get_status(_state: &ApiServerState) -> (StatusCode, String) {
    // Return current observation status
    let value = serde_json::json!({
        "observation_enabled": true,
        "status": "active",
        "providers": ["app_monitor", "browser", "clipboard", "terminal"]
    });
    (StatusCode::OK, api_response(true, Some(value), None))
}

async fn handle_observation_start(_state: &ApiServerState) -> (StatusCode, String) {
    info!("Observation start requested");
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"status": "started"})), None))
}

async fn handle_observation_stop(_state: &ApiServerState) -> (StatusCode, String) {
    info!("Observation stop requested");
    (StatusCode::OK, api_response(true, Some(serde_json::json!({"status": "stopped"})), None))
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

    let router = Router::new()
        .route("/api/commands/:method", post(api_handler))
        .route("/health", get(|| async { "ok" }))
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
pub fn start_api_server(port: u16, config_path: Option<std::path::PathBuf>, skills_path: Option<std::path::PathBuf>, knowledge_path: Option<std::path::PathBuf>) -> Result<(), String> {
    let state = ApiServerState {
        settings: Arc::new(Mutex::new(ApiServerSettings::new())),
        config_path: Arc::new(Mutex::new(config_path.clone())),
    };
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

    // Initialize knowledge packs from data directory
    let knowledge_dir_to_use = knowledge_path.clone().or_else(|| {
        config_path.as_ref().and_then(|cp| cp.parent().map(|p| p.join("knowledge")))
    });

    // Store knowledge path for later async initialization
    let kdir = knowledge_dir_to_use.clone();
    let kdir_str = kdir.as_ref().map(|d| d.to_string_lossy().to_string());

    let addr = format!("0.0.0.0:{port}");

    info!(addr, "Starting API server in background thread");

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
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

            // Initialize and start observation engine
            let mut registry = ProviderRegistry::new();
            registry.register(Box::new(ActiveWindowProvider::new()));
            registry.register(Box::new(BrowserProvider::new()));
            registry.register(Box::new(TerminalProvider::new()));
            registry.register(Box::new(ClipboardProvider::new()));
            registry.register(Box::new(ScreenCaptureProvider::new()));
            registry.register(Box::new(FileObserverProvider::new()));
            info!(count = registry.provider_names().len(), "Observation providers registered");
            let results = registry.start_all().await;
            for (name, result) in &results {
                match result {
                    Ok(_) => info!(name, "Observation provider started"),
                    Err(e) => warn!(name, error = %e, "Observation provider failed to start"),
                }
            }
            info!("Observation engine initialized");

            let result = rt.block_on(async {
                let listener = tokio::net::TcpListener::bind(&addr)
                    .await
                    .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

                info!(addr, "API server listening");

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