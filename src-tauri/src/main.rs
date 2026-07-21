use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info};
use uuid::Uuid;
use wikilabs_ai::provider::{AiProvider, AiRequest, OpenAICompatibleProvider, ProviderInfo};
use wikilabs_data_types::chat::ChatMessage;
use wikilabs_persistence::{schema::INIT_SQL, Database, RepositoryFactory};

use wikilabs_knowledge::validate::{ValidationReport, ValidationResult, ValidationStatus, validate_pack_comprehensive};

mod config;
mod error_handling;
mod guidance_panel;
mod knowledge_panel;
mod logging;
mod security;
mod skill_management;
use config::{AiProviderConfig, AppSettings, AppSettingsStore};
use error_handling::{ErrorEvent, ErrorSeverity, ErrorHandler, GracefulShutdown};
use logging::redact_sensitive_data;
use security::EncryptionService;
use guidance_panel::{
    guidance_add_evidence, guidance_add_timeline_event, guidance_clear_all, guidance_complete_step,
    guidance_dismiss_recommendation, guidance_get_active_recommendations, guidance_get_all_recommendations,
    guidance_get_available_modes, guidance_get_evidence_status, guidance_get_feedback_stats,
    guidance_get_mode, guidance_get_recent_events, guidance_get_timeline, guidance_get_workflow_progress,
    guidance_mark_missing, guidance_record_feedback, guidance_set_mode, guidance_start_workflow,
    guidance_update_recommendation_status,
};

/// Shared application state — uses Arc for Clone safety.
#[derive(Clone)]
pub struct AppState {
    pub app_handle: Arc<std::sync::RwLock<Option<AppHandle>>>,
    pub db: Arc<Database>,
    pub repos: Arc<RepositoryFactory>,
    pub settings: AppSettingsStore,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Result<Self, anyhow::Error> {
        info!("Creating application state");

        let data_dir = app_handle.path().app_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("wikilabs.db");

        info!(path = %db_path.display(), "Opening database");
        let db = Database::new(&db_path.to_string_lossy())?;
        db.execute_batch(INIT_SQL)?;

        let repos = RepositoryFactory::new(db.clone());

        Ok(Self {
            app_handle: Arc::new(std::sync::RwLock::new(Some(app_handle))),
            db: Arc::new(db),
            repos: Arc::new(repos),
            settings: AppSettingsStore::new(),
        })
    }
}

// ── Tauri Commands ──────────────────────────────────────────────

#[tauri::command]
fn get_settings(app_state: tauri::State<AppState>) -> Result<AppSettings, String> {
    info!("get_settings called");
    Ok(app_state.settings.get())
}

#[tauri::command]
fn update_settings(app_state: tauri::State<AppState>, settings: AppSettings) -> Result<(), String> {
    info!("update_settings called");
    app_state.settings.save(settings);
    Ok(())
}

#[tauri::command]
fn list_providers() -> Result<Vec<ProviderInfo>, String> {
    let providers = vec![
        ProviderInfo {
            name: "OpenAI".to_string(),
            url: "https://api.openai.com/v1".to_string(),
            api_version: "v1".to_string(),
        },
        ProviderInfo {
            name: "vLLM".to_string(),
            url: "http://localhost:8000/v1".to_string(),
            api_version: "v1".to_string(),
        },
        ProviderInfo {
            name: "Ollama".to_string(),
            url: "http://localhost:11434/v1".to_string(),
            api_version: "v1".to_string(),
        },
    ];
    Ok(providers)
}

#[tauri::command]
fn test_connection(config: AiProviderConfig) -> Result<bool, String> {
    info!(provider = %config.name, endpoint = %config.endpoint, "Testing AI provider connection");

    if config.api_key.is_empty() {
        return Err("API key is required".to_string());
    }

    let provider = OpenAICompatibleProvider::new(
        &config.name,
        &config.endpoint,
        &config.api_key,
        &config.model,
        config.max_tokens,
        config.context_window,
    );

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(provider.health())
        .map(|_| {
            info!("Provider connection verified");
            true
        })
        .map_err(|e| {
            error!(error = %e, "Provider connection failed");
            e.to_string()
        })
}

#[tauri::command]
fn get_workspace_list(app_state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let workspaces = app_state
        .repos
        .workspace
        .list_all()
        .map_err(|e| e.to_string())?;
    Ok(workspaces.iter().map(|w| w.name.clone()).collect())
}

#[tauri::command]
fn create_workspace(
    app_state: tauri::State<AppState>,
    name: String,
    customer_name: String,
) -> Result<String, String> {
    let ws_id = Uuid::new_v4().to_string();
    app_state
        .repos
        .workspace
        .insert(&ws_id, &name, &customer_name, "[]")
        .map_err(|e| e.to_string())?;
    info!(id = %ws_id, name = %name, "Workspace created");
    Ok(ws_id)
}

#[tauri::command]
fn get_status() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "running",
        "version": "0.1.0",
        "features": {
            "chat": true,
            "workspace": true,
            "knowledge": true,
            "skills": false,
            "mcp": false,
            "automation": false
        }
    }))
}

// ── Chat Commands ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[tauri::command]
fn send_message(
    app_state: tauri::State<AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let settings = app_state.settings.get();
    let ws_id = request
        .workspace_id
        .clone()
        .unwrap_or_else(|| "default".to_string());

    // Create user message
    let user_msg = ChatMessage::user(&request.message);
    let user_id = user_msg.id.to_string();

    // Save user message to database
    app_state
        .repos
        .chat_messages
        .insert(&user_id, &ws_id, "user", &request.message, "[]")
        .map_err(|e| e.to_string())?;

    // Build AI request
    let ai_request = AiRequest {
        model: settings.ai_provider.model.clone(),
        messages: vec![wikilabs_ai::provider::AiMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        }],
        tools: vec![],
        temperature: None,
        max_tokens: Some(settings.ai_provider.max_tokens),
        stream: None,
    };

    // Create AI provider
    let provider = OpenAICompatibleProvider::new(
        &settings.ai_provider.name,
        &settings.ai_provider.endpoint,
        &settings.ai_provider.api_key,
        &settings.ai_provider.model,
        settings.ai_provider.max_tokens,
        settings.ai_provider.context_window,
    );

    // Call AI
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let response = rt.block_on(provider.chat(ai_request)).map_err(|e| {
        error!(error = %e, "AI request failed");
        e.to_string()
    })?;

    // Format assistant response
    let assistant_msg = ChatMessage::assistant(&response.message.content);
    let assistant_id = assistant_msg.id.to_string();
    let assistant_created = assistant_msg.created_at.to_rfc3339();

    // Save assistant message
    app_state
        .repos
        .chat_messages
        .insert(
            &assistant_id,
            &ws_id,
            "assistant",
            &response.message.content,
            "[]",
        )
        .map_err(|e| e.to_string())?;

    Ok(ChatResponse {
        id: assistant_id,
        role: "assistant".to_string(),
        content: response.message.content,
        created_at: assistant_created,
    })
}

#[tauri::command]
fn get_history(
    app_state: tauri::State<AppState>,
    workspace_id: String,
    limit: Option<usize>,
) -> Result<Vec<ChatResponse>, String> {
    let messages = app_state
        .repos
        .chat_messages
        .get_by_workspace(&workspace_id, limit.unwrap_or(50))
        .map_err(|e| e.to_string())?;

    let responses: Vec<ChatResponse> = messages
        .iter()
        .map(|m| ChatResponse {
            id: m.id.clone(),
            role: m.role.clone(),
            content: m.content.clone(),
            created_at: m.created_at.clone(),
        })
        .collect();

    Ok(responses)
}

#[tauri::command]
fn clear_history(app_state: tauri::State<AppState>, workspace_id: String) -> Result<(), String> {
    app_state
        .repos
        .chat_messages
        .delete_by_workspace(&workspace_id)
        .map_err(|e| e.to_string())?;
    info!(workspace_id, "Chat history cleared");
    Ok(())
}

// ── Database Commands ──────────────────────────────────────────

#[tauri::command]
fn get_conversations(app_state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let workspaces = app_state
        .repos
        .workspace
        .list_all()
        .map_err(|e| e.to_string())?;
    Ok(workspaces.iter().map(|w| w.name.clone()).collect())
}

#[tauri::command]
fn get_messages(
    app_state: tauri::State<AppState>,
    conversation_id: String,
    limit: Option<usize>,
) -> Result<Vec<ChatResponse>, String> {
    get_history(app_state, conversation_id, limit)
}

#[tauri::command]
fn save_message(
    app_state: tauri::State<AppState>,
    id: String,
    workspace_id: String,
    role: String,
    content: String,
) -> Result<(), String> {
    app_state
        .repos
        .chat_messages
        .insert(&id, &workspace_id, &role, &content, "[]")
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Logging Commands ───────────────────────────────────────────

#[tauri::command]
fn get_logs(_limit: Option<usize>) -> Result<Vec<String>, String> {
    Ok(vec![
        "Application started".to_string(),
        "Database initialized".to_string(),
        "AI provider configured".to_string(),
    ])
}

// ── Streaming (placeholder) ────────────────────────────────────

#[tauri::command]
async fn stream_message(
    message: String,
    _workspace_id: String,
    app: tauri::AppHandle,
    _settings: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!(message_len = message.len(), "Streaming message started");

    let placeholder = format!(
        "Streaming mode: You asked \"{}\" — full streaming support will be added in the next milestone.\n\nCurrent capabilities:\n- Non-streaming chat ✓\n- Workspace management ✓\n- Knowledge search ✓\n- Streaming responses (in progress)",
        message
    );

    app.emit(
        "assistant_message",
        &ChatResponse {
            id: Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: placeholder,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ── App Entry Point ────────────────────────────────────────────

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Wiki Labs AI Copilot starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(AppSettingsStore::new())
        .setup(|app| {
            let state = AppState::new(app.handle().clone())?;
            info!("Application state initialized");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Chat commands
            send_message,
            get_history,
            clear_history,
            stream_message,
            // Settings commands
            get_settings,
            update_settings,
            // Provider commands
            list_providers,
            test_connection,
            // Workspace commands
            get_workspace_list,
            create_workspace,
            // Database commands
            get_conversations,
            get_messages,
            save_message,
            // Knowledge panel commands
            guidance_get_active_recommendations,
            guidance_get_all_recommendations,
            guidance_dismiss_recommendation,
            guidance_update_recommendation_status,
            guidance_get_evidence_status,
            guidance_add_evidence,
            guidance_mark_missing,
            guidance_get_workflow_progress,
            guidance_start_workflow,
            guidance_complete_step,
            guidance_get_timeline,
            guidance_add_timeline_event,
            guidance_get_recent_events,
            guidance_record_feedback,
            guidance_get_feedback_stats,
            guidance_set_mode,
            guidance_get_mode,
            guidance_get_available_modes,
            guidance_clear_all,
            // System commands
            get_status,
            get_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
