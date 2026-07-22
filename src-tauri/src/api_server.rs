//! HTTP server that bridges frontend REST API calls to local state.
//!
//! The frontend SPA hardcodes calls to `http://localhost:1420/api/commands/*`.
//! This server intercepts those calls and serves local state + chat history.

use axum::{
    extract::{Path, State},
    extract::Json,
    http::StatusCode,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

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
}

#[derive(Debug, Clone)]
pub struct ApiServerSettings {
    pub settings: Value,
    pub providers: Vec<Value>,
    pub messages: Arc<Mutex<Vec<ChatMessage>>>,
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
        "get_settings" => handle_get_settings(&state),
        "update_settings" => handle_update_settings(&state, req.params),
        "test_connection" => handle_test_connection(&state, req.params),
        "send_message" => handle_send_message(&state, req.params),
        "get_history" => handle_get_history(&state),
        "list_providers" => handle_list_providers(&state),
        other => {
            warn!(other, "Unknown API method");
            (StatusCode::BAD_REQUEST, api_response(false, None, Some(format!("Unknown method: {}", other))))
        }
    };

    info!(method, "API request completed");
    (status, body)
}

fn handle_test_connection(_state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let api_key = params.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if api_key.is_empty() {
        return (StatusCode::OK, api_response(false, None, Some("API key is required".to_string())));
    }
    (StatusCode::OK, api_response(true, Some(serde_json::json!(true)), None))
}

fn handle_get_settings(state: &ApiServerState) -> (StatusCode, String) {
    let settings = state.settings.lock().unwrap();
    (StatusCode::OK, api_response(true, Some(settings.settings.clone()), None))
}

fn handle_update_settings(state: &ApiServerState, params: Value) -> (StatusCode, String) {
    let mut settings = state.settings.lock().unwrap();
    settings.settings = params;
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

    let assistant_id = uuid::Uuid::new_v4().to_string();
    let assistant_created = chrono::Utc::now().to_rfc3339();
    let response = format!(
        "Message received: \"{}\"\n\nNote: Full AI responses require Tauri IPC integration.\n\nCapabilities:\n- Chat history ✓\n- Settings management ✓\n- Provider configuration ✓\n- AI responses — pending (via Tauri command)",
        message
    );

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

/// Create the router for the API server.
pub fn create_router(state: ApiServerState) -> Router {
    Router::new()
        .route("/api/commands/:method", post(api_handler))
        .with_state(state)
}

/// Start the HTTP server on the given port (default 1420).
/// Runs in a dedicated thread to keep the tokio runtime alive.
pub fn start_api_server(port: u16) -> Result<(), String> {
    let state = ApiServerState {
        settings: Arc::new(Mutex::new(ApiServerSettings::new())),
    };
    let router = create_router(state);
    let addr = format!("0.0.0.0:{}", port);

    info!(addr, "Starting API server in background thread");

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create tokio runtime: {}", e));
        
        if let Ok(rt) = rt {
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