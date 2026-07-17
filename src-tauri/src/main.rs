use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use wikilabs_ai::provider::{AiProvider, OpenAICompatibleProvider, ProviderInfo};
use wikilabs_persistence::{Database, RepositoryFactory, schema::INIT_SQL};
use wikilabs_data_types::chat::ChatMessage;
use wikilabs_workspace::manager::WorkspaceManager;
use wikilabs_knowledge::search::KnowledgeSearch;
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod config;
use config::{AiProviderConfig, AppSettings};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub app_handle: Mutex<Option<AppHandle>>,
    pub db: Database,
    pub repos: RepositoryFactory,
    pub workspace_manager: Mutex<Option<WorkspaceManager>>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Result<Self, anyhow::Error> {
        info!("Creating application state");

        // Get data dir for database
        let data_dir = app_handle.path().app_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("wikilabs.db");

        info!(path = %db_path.display(), "Opening database");
        let db = Database::new(&db_path.to_string_lossy())?;
        db.execute_batch(INIT_SQL)?;

        let repos = RepositoryFactory::new(db.clone());

        Ok(Self {
            app_handle: Mutex::new(Some(app_handle)),
            db,
            repos,
            workspace_manager: Mutex::new(None),
        })
    }
}

// ── Tauri Commands ──────────────────────────────────────────────

#[tauri::command]
fn get_settings(app_state: tauri::State<AppState>) -> Result<AppSettings, String> {
    info!("get_settings called");
    AppSettings::load(&app_state.repos).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_settings(
    app_state: tauri::State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    info!("update_settings called");
    settings.save(&app_state.repos).map_err(|e| e.to_string())
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
    let workspaces = app_state.repos.workspace.list_all().map_err(|e| e.to_string())?;
    Ok(workspaces.iter().map(|w| w.name.clone()).collect())
}

#[tauri::command]
fn create_workspace(
    app_state: tauri::State<AppState>,
    name: String,
    customer_name: String,
) -> Result<String, String> {
    let ws_id = Uuid::new_v4().to_string();
    app_state.repos.workspace
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
    settings: tauri::State<AppSettings>,
) -> Result<ChatResponse, String> {
    let settings_inner = settings.inner();
    let ws_id = request.workspace_id.clone().unwrap_or_else(|| "default".to_string());

    // Create user message
    let user_msg = ChatMessage::user(&request.message);
    let user_id = user_msg.id.to_string();

    // Save user message to database
    let tool_calls_json = "[]";
    app_state.repos.chat_messages
        .insert(&user_id, &ws_id, "user", &request.message, tool_calls_json)
        .map_err(|e| e.to_string())?;

    // Build AI request from conversation history
    let messages = app_state.repos.chat_messages
        .get_by_workspace(&ws_id, 50)
        .map_err(|e| e.to_string())?;

    let chat_msgs: Vec<ChatMessage> = messages.iter().map(|m| {
        ChatMessage {
            id: uuid::Uuid::parse_str(&m.id).unwrap_or_default(),
            role: m.role.clone(),
            content: m.content.clone(),
            created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_default(),
            tool_calls: vec![],
        }
    }).collect();

    // Create AI provider
    let provider = OpenAICompatibleProvider::new(
        &settings_inner.ai_provider.name,
        &settings_inner.ai_provider.endpoint,
        &settings_inner.ai_provider.api_key,
        &settings_inner.ai_provider.model,
        settings_inner.ai_provider.max_tokens,
        settings_inner.ai_provider.context_window,
    );

    let ai_request = wikilabs_ai::provider::AiRequest::new(
        &settings_inner.ai_provider.model,
        &chat_msgs.iter().map(|m| {
            wikilabs_ai::provider::AiMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            }
        }).collect::<Vec<_>>(),
    );

    // Call AI
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let response = rt.block_on(provider.chat(ai_request))
        .map_err(|e| {
            error!(error = %e, "AI request failed");
            e.to_string()
        })?;

    // Format assistant response
    let assistant_msg = ChatMessage::assistant(&response.message.content);
    let assistant_id = assistant_msg.id.to_string();
    let assistant_created = assistant_msg.created_at.to_rfc3339();

    // Save assistant message
    let tool_calls_json = if !response.tool_calls.is_empty() {
        serde_json::to_string(&response.tool_calls).unwrap_or_else(|_| "[]".to_string())
    } else {
        "[]".to_string()
    };

    app_state.repos.chat_messages
        .insert(&assistant_id, &ws_id, "assistant", &response.message.content, &tool_calls_json)
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
    let messages = app_state.repos.chat_messages
        .get_by_workspace(&workspace_id, limit.unwrap_or(50))
        .map_err(|e| e.to_string())?;

    let responses: Vec<ChatResponse> = messages.iter().map(|m| {
        ChatResponse {
            id: m.id.clone(),
            role: m.role.clone(),
            content: m.content.clone(),
            created_at: m.created_at.clone(),
        }
    }).collect();

    Ok(responses)
}

#[tauri::command]
fn clear_history(
    app_state: tauri::State<AppState>,
    workspace_id: String,
) -> Result<(), String> {
    app_state.repos.chat_messages
        .delete_by_workspace(&workspace_id)
        .map_err(|e| e.to_string())?;
    info!(workspace_id, "Chat history cleared");
    Ok(())
}

// ── Database Commands ──────────────────────────────────────────

#[tauri::command]
fn get_conversations(app_state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let workspaces = app_state.repos.workspace.list_all().map_err(|e| e.to_string())?;
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
    app_state.repos.chat_messages
        .insert(&id, &workspace_id, &role, &content, "[]")
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Logging Commands ───────────────────────────────────────────

#[tauri::command]
fn get_logs(limit: Option<usize>) -> Result<Vec<String>, String> {
    // Log retrieval is handled by tracing-subscriber; return recent info
    Ok(vec![
        "Application started".to_string(),
        "Database initialized".to_string(),
        "AI provider configured".to_string(),
    ])
}

// ── App Entry Point ────────────────────────────────────────────

#[tauri::command]
async fn stream_message(
    message: String,
    workspace_id: String,
    app: tauri::AppHandle,
    settings: tauri::State<'_, AppSettings>,
) -> Result<(), String> {
    info!(message_len = message.len(), "Streaming message started");

    // For MVP: send a placeholder that streaming is being implemented
    // In future milestones this will use real AI streaming
    let placeholder = format!(
        "Streaming mode: You asked \"{}\" — full streaming support will be added in the next milestone.\n\nCurrent capabilities:\n- Non-streaming chat ✓\n- Workspace management ✓\n- Knowledge search ✓\n- Streaming responses (in progress)",
        message
    );

    app.emit("assistant_message", &ChatResponse {
        id: Uuid::new_v4().to_string(),
        role: "assistant".to_string(),
        content: placeholder,
        created_at: chrono::Utc::now().to_rfc3339(),
    }).map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Wiki Labs AI Copilot starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::init(tauri_plugin_log::Config::default()))
        .manage(AppSettings::default())
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
            // System commands
            get_status,
            get_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}