#![windows_subsystem = "windows"]
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info};
use uuid::Uuid;
use wikilabs_ai::provider::{AiProvider, AiRequest, OpenAICompatibleProvider, ProviderInfo};
use wikilabs_benchmark::{BenchmarkRegistry, categories};
use wikilabs_data_types::chat::ChatMessage;
use wikilabs_persistence::{schema::INIT_SQL, Database, RepositoryFactory};


mod api_server;
mod config;
mod error_handling;
mod guidance_panel;
mod knowledge_panel;
mod logging;
mod security;
mod skill_management;
mod windows_cleanup;
use config::{AiProviderConfig, AppSettings, AppSettingsStore};
use guidance_panel::{
    guidance_add_evidence, guidance_add_timeline_event, guidance_clear_all, guidance_complete_step,
    guidance_dismiss_recommendation, guidance_get_active_recommendations, guidance_get_all_recommendations,
    guidance_get_available_modes, guidance_get_evidence_status, guidance_get_feedback_stats,
    guidance_get_mode, guidance_get_recent_events, guidance_get_timeline, guidance_get_workflow_progress,
    guidance_mark_missing, guidance_record_feedback, guidance_set_mode, guidance_start_workflow,
    guidance_update_recommendation_status,
};
use knowledge_panel::{
    knowledge_disable_pack, knowledge_enable_pack, knowledge_export_pack, knowledge_get_pack_metadata,
    knowledge_get_validation_report, knowledge_import_pack, knowledge_list_packs, knowledge_reindex_pack,
};
use skill_management::{
    skill_disable, skill_enable, skill_get, skill_list, skill_mark_validated, skill_set_active,
    skill_toggle, skill_validate,
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
            settings: AppSettingsStore::with_path(data_dir.join("settings.json"))?,
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
    if let Err(e) = app_state.settings.persist() {
        error!(error = %e, "Failed to persist settings to disk");
        return Err(e.to_string());
    }
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
        "version": env!("CARGO_PKG_VERSION"),
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

/// Performance metrics from the benchmark registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    #[serde(default)]
    pub startup: serde_json::Value,
    #[serde(default)]
    pub ai_response: serde_json::Value,
    #[serde(default)]
    pub knowledge_indexing: serde_json::Value,
    #[serde(default)]
    pub skill_loading: serde_json::Value,
    #[serde(default)]
    pub screen_capture: serde_json::Value,
    #[serde(default)]
    pub ocr_processing: serde_json::Value,
    #[serde(default)]
    pub large_conversation: serde_json::Value,
}

#[tauri::command]
fn get_performance_metrics(
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let registry = app.state::<std::sync::Arc<std::sync::Mutex<wikilabs_benchmark::BenchmarkRegistry>>>();
    let reg = registry.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
    Ok(reg.to_diagnostics())
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
    let ai_start = std::time::Instant::now();
    let response = rt.block_on(provider.chat(ai_request)).map_err(|e| {
        error!(error = %e, "AI request failed");
        e.to_string()
    })?;
    let ai_time = ai_start.elapsed();
    tracing::info!(
        "AI response received in {} µs (tokens: prompt={}, completion={}, total={})",
        ai_time.as_micros(),
        response.usage.prompt_tokens,
        response.usage.completion_tokens,
        response.usage.total_tokens
    );

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
    let startup_start = Instant::now();

    // ── Register panic hook (FIX #2: ensure cleanup on crash) ──
    windows_cleanup::register_panic_hook();

    // Do NOT initialize a global logger here — tauri-plugin-log handles console
    // output and init_logging() (called later during setup) handles file output.
    // Calling both would cause "attempted to set a logger after the logging
    // system was already initialized" panics.
    println!("Wiki Labs AI Copilot starting");

    let settings_load_start = Instant::now();
    let settings = AppSettingsStore::new();
    let config_load_time = settings_load_start.elapsed();
    tracing::info!(
        "Config loaded in {} µs",
        config_load_time.as_micros()
    );

    // ── API server state (FIX #3: start API server BEFORE setup hook) ──
    // Starting the API server in the setup hook conflicts with WebView2 initialization.
    // We start it early (before tauri::Builder) to avoid the "Access is denied" panic.
    // Build the API server state (to be populated during setup)
    let api_server_state = std::sync::Arc::new(std::sync::Mutex::new(None));
    let api_server_state_clone = api_server_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Minimize to tray instead of closing
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .manage(settings)
        .setup(move |app| {
            let state_setup_start = Instant::now();
            let state = AppState::new(app.handle().clone())?;
            let state_time = state_setup_start.elapsed();
            info!(
                "Application state initialized in {} µs",
                state_time.as_micros()
            );

            // Construct config path for API server persistence
            let data_dir = app.handle().path().app_data_dir()?;
            let config_path = data_dir.join("settings.json");
            tracing::info!(path = %config_path.display(), "Wiring config path to API server");

            // Start the HTTP API server — NOW OUTSIDE the setup hook to avoid WebView2 conflicts
            // FIX #3: Previously this was in the setup hook, which caused "Access is denied"
            // because Tauri was still initializing WebView2 window classes.
            let config_path_clone = config_path.clone();
            let api_state_clone = api_server_state_clone.clone();

            // Resolve bundled skills resource path
            let skills_path = app.handle().path().resource_dir()
                .map(|rd| rd.join("skills"))
                .ok();
            // Resolve bundled knowledge resource path
            let knowledge_path = app.handle().path().resource_dir()
                .map(|rd| rd.join("knowledge"))
                .ok();

            std::thread::spawn(move || {
                match api_server::start_api_server(1420, Some(config_path_clone), skills_path, knowledge_path, Some(app.handle().clone())) {
                    Ok(_) => {
                        info!("API server started successfully in background thread");
                        *api_state_clone.lock().unwrap() = Some(true);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to start API server");
                        *api_state_clone.lock().unwrap() = None;
                    }
                }
            });

            // Record startup benchmark (startup = total time from process launch to ready)
            let _total_startup = startup_start.elapsed();
            let mut registry = BenchmarkRegistry::new();
            registry.record(
                wikilabs_benchmark::BenchmarkTimer::new(categories::STARTUP)
                    .with_metadata("state_init_us", &state_time.as_micros().to_string())
                    .with_metadata("config_load_us", &config_load_time.as_micros().to_string())
                    .finish(),
            );

            // Expose registry via Arc for use in commands
            app.manage(std::sync::Arc::new(std::sync::Mutex::new(registry)));
            app.manage(state);

            // ── System Tray Setup ──
            let handle = app.handle().clone();
            // Build tray context menu
            let show_item = tauri::menu::MenuItemBuilder::with_id("show", "Show Wiki Labs AI Copilot")
                .build(app)?;
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "Quit")
                .build(app)?;
            let tray_menu = tauri::menu::MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Wiki Labs AI Copilot")
                .menu(&tray_menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

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
            // Performance commands
            get_performance_metrics,
            // Knowledge panel commands
            knowledge_list_packs,
            knowledge_enable_pack,
            knowledge_disable_pack,
            knowledge_get_pack_metadata,
            knowledge_get_validation_report,
            knowledge_export_pack,
            knowledge_import_pack,
            knowledge_reindex_pack,
            // Skill management commands
            skill_list,
            skill_get,
            skill_enable,
            skill_disable,
            skill_toggle,
            skill_set_active,
            skill_validate,
            skill_mark_validated,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
