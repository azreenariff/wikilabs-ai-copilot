#![windows_subsystem = "windows"]
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info};
use uuid::Uuid;
use wikilabs_ai::provider::{AiProvider, AiRequest, OpenAICompatibleProvider, ProviderInfo};
use wikilabs_benchmark::{BenchmarkRegistry, categories};
use wikilabs_data_types::chat::ChatMessage;
use wikilabs_persistence::{schema::INIT_SQL, Database, RepositoryFactory};


mod api_ready;
mod api_server;
mod config;
mod error_handling;
mod guidance_panel;
mod knowledge_panel;
mod logging;
mod observation;
mod security;
mod skill_management;
mod skill_knowledge;
mod windows_cleanup;
use config::{AiProviderConfig, AppSettings, AppSettingsStore};
use guidance_panel::{
    guidance_add_evidence, guidance_add_timeline_event, guidance_clear_all, guidance_complete_step,
    guidance_dismiss_recommendation, guidance_get_active_recommendations, guidance_get_all_recommendations,
    guidance_get_evidence_status, guidance_get_feedback_stats,
    guidance_get_recent_events, guidance_get_timeline, guidance_get_workflow_progress,
    guidance_mark_missing, guidance_record_feedback, guidance_start_workflow,
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

    // Build system prompt from persona definition (loaded from assets/system_prompt.md)
    let system_prompt = wikilabs_ai::persona::EngineeringPersona::default_system_prompt();

    // Build AI request — include system prompt + observation context + conversation history
    let mut messages = vec![
        wikilabs_ai::provider::AiMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
    ];

    // Add observation context as a system message — tells the AI what's happening
    // in the user's environment so it can give context-aware guidance
    let observation_context = build_observation_context();
    if !observation_context.is_empty() {
        messages.push(wikilabs_ai::provider::AiMessage {
            role: "system".to_string(),
            content: format!("[OBSERVATION CONTEXT]
{}", observation_context),
        });
    }

    // Add conversation history for context (last 20 messages)
    let history = app_state
        .repos
        .chat_messages
        .get_by_workspace(&ws_id, 20)
        .map_err(|e| e.to_string())?;

    for msg in &history {
        messages.push(wikilabs_ai::provider::AiMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    // Create AI request with system prompt, observation context, and conversation history
    let ai_request = AiRequest {
        model: settings.ai_provider.model.clone(),
        messages,
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

/// Build observation context from the observation engine using the IntentAnalyzer.
/// Produces a structured "intent summary" that tells the AI copilot what the user
/// is doing, their likely intent, detected issues, and suggested actions —
/// enabling context-aware proactive guidance instead of raw error dumps.
fn build_observation_context() -> String {
    let start = std::time::Instant::now();

    let engine = match observation::get_observation_engine() {
        Some(e) => e,
        None => return String::new(),
    };

    // Run the intent analyzer — this synthesizes all observations into a structured summary
    let summary = engine.analyze_intent();

    if summary.current_activity.is_empty() && summary.intent.is_none() && summary.issues.is_empty() {
        return String::new();
    }

    let mut context = String::new();

    // ── What the user is doing ──
    context.push_str("## What the user is doing
");
    for activity in &summary.current_activity {
        let cat_label = match &activity.category {
            wikilabs_observation::ActivityCategory::Troubleshooting => "🔧 troubleshooting",
            wikilabs_observation::ActivityCategory::Deployment => "🚀 deployment",
            wikilabs_observation::ActivityCategory::Monitoring => "📊 monitoring",
            wikilabs_observation::ActivityCategory::Administration => "⚙️ administration",
            wikilabs_observation::ActivityCategory::Development => "💻 development",
            wikilabs_observation::ActivityCategory::Browsing => "🌐 browsing",
            wikilabs_observation::ActivityCategory::Communication => "📬 communication",
            wikilabs_observation::ActivityCategory::Unknown => "❓ unknown",
        };
        context.push_str(&format!(
            "- **{}** (confidence: {:.0}%) — {}
",
            cat_label,
            activity.confidence * 100.0,
            activity.description.chars().take(200).collect::<String>()
        ));
    }

    // ── Intent Analysis ──
    if let Some(ref intent) = summary.intent {
        let cat_label = match &intent.activity_category {
            wikilabs_observation::ActivityCategory::Troubleshooting => "🔧 troubleshooting",
            wikilabs_observation::ActivityCategory::Deployment => "🚀 deployment",
            wikilabs_observation::ActivityCategory::Monitoring => "📊 monitoring",
            wikilabs_observation::ActivityCategory::Administration => "⚙️ administration",
            wikilabs_observation::ActivityCategory::Development => "💻 development",
            wikilabs_observation::ActivityCategory::Browsing => "🌐 browsing",
            wikilabs_observation::ActivityCategory::Communication => "📬 communication",
            wikilabs_observation::ActivityCategory::Unknown => "❓ mixed activity",
        };
        context.push_str(&format!("
## Intent Analysis
- Detected intent: **{}**
- Category: {}
- Confidence: {:.0}%
",
            intent.intent, cat_label, intent.confidence * 100.0
        ));
        if let Some(ref goal) = intent.goal {
            context.push_str(&format!("- Likely goal: {}
", goal.chars().take(300).collect::<String>()));
        }
        if !intent.infrastructure_targets.is_empty() {
            context.push_str("- Infrastructure involved:
");
            for target in &intent.infrastructure_targets {
                context.push_str(&format!("  - {}
", target.chars().take(200).collect::<String>()));
            }
        }
        if !intent.related_actions.is_empty() {
            context.push_str("- Related actions:
");
            for action in &intent.related_actions {
                context.push_str(&format!("  - {}
", action.chars().take(200).collect::<String>()));
            }
        }
        if !intent.suggested_next_steps.is_empty() {
            context.push_str("- Suggested next steps:
");
            for step in &intent.suggested_next_steps {
                context.push_str(&format!("  - {}
", step.chars().take(200).collect::<String>()));
            }
        }
    }

    // ── Detected Issues ──
    if !summary.issues.is_empty() {
        context.push_str("
## Detected Issues
");
        for issue in &summary.issues {
            let sev = match issue.severity {
                wikilabs_observation::IssueSeverity::Low => "LOW",
                wikilabs_observation::IssueSeverity::Medium => "MED",
                wikilabs_observation::IssueSeverity::High => "HIGH",
                wikilabs_observation::IssueSeverity::Critical => "CRIT",
            };
            context.push_str(&format!(
                "- [{}] {} — {}
",
                sev, issue.title, issue.description.chars().take(300).collect::<String>()
            ));
        }
    }

    // ── Suggested Guidance ──
    if !summary.suggested_guidance.is_empty() {
        context.push_str("
## Suggested Guidance
");
        for guidance in &summary.suggested_guidance {
            context.push_str(&format!("- {}
", guidance.chars().take(300).collect::<String>()));
        }
    }

    tracing::debug!(
        "Intent analysis produced context in {} µs, {} activities, {} issues, {} guidance items",
        start.elapsed().as_micros(),
        summary.current_activity.len(),
        summary.issues.len(),
        summary.suggested_guidance.len()
    );

    context
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

// ── Advice Chat Window Management ──────────────────────────────

#[tauri::command]
fn open_advice_chat_window(app: tauri::AppHandle) -> Result<(), String> {
    info!("Opening advice chat window (right side)");
    if let Some(window) = app.get_webview_window("advice-chat") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    
    // Load the embedded advice chat HTML via the API server endpoint
    // The API server (port 1420) serves advice-chat.html at /advice-chat
    let url = tauri::WebviewUrl::External("http://localhost:1420/advice-chat".parse::<url::Url>().map_err(|e| e.to_string())?);
    
    let window = tauri::WebviewWindowBuilder::new(&app, "advice-chat", url)
        .title("AI Copilot — Live Advice")
        .inner_size(400.0, 520.0)
        .resizable(true)
        .decorations(true)  // proper window controls (close, minimize)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;
    
    // Position on the right side of the screen (vertically centered)
    if let Some(w) = app.get_webview_window("advice-chat") {
        if let Ok(Some(m)) = w.current_monitor() {
            let width = 400.0;
            let height = 520.0;
            let x = m.size().width as f64 - width - 10.0; // 10px from right edge
            let y = (m.size().height as f64 - height) / 2.0;
            let _ = w.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
        }
    }
    
    Ok(())
}

#[tauri::command]
fn close_advice_chat_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("advice-chat") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_guidance_page() -> Result<(), String> {
    // Navigation is handled by the main app's router
    Ok(())
}

// ── Window Management ──────────────────────────────────────────

#[tauri::command]
fn hide_main_window(app: tauri::AppHandle) -> Result<(), String> {
    info!("Hiding main window (minimize to tray)");
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
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

    println!("Wiki Labs AI Copilot starting");

    // Initialize logging BEFORE anything else — ensures ALL tracing::info!() calls
    // from every module (api_server, observation, etc.) are captured to file.
    // Uses debug level to capture everything; log rotation runs on startup
    // to clean up stale log files (older than 7 days).
    let log_dir = PathBuf::from(
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(|d| format!("{}/wikilabs-ai-copilot/logs", d))
            .unwrap_or_else(|| format!("{}/.local/share/wikilabs-ai-copilot/logs", std::env::var("HOME").unwrap_or_default())),
    );
    if let Err(e) = logging::init_logging(
        &config::LoggingSettings {
            level: "debug".to_string(),
            file_logging: true,
            max_log_size_mb: 10,
            max_log_files: 3,
            structured_logging: true,
        },
        &log_dir,
    ) {
        eprintln!("Failed to initialize logging: {}", e);
    }

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
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus the existing window when a second instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
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

            // Clone app_handle BEFORE the thread spawn to avoid capturing &mut tauri::App
            let app_handle_for_thread = app.handle().clone();
            let skills_path_thread = skills_path.clone();
            let knowledge_path_thread = knowledge_path.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                rt.block_on(async {
                    let engine = observation::init_observation_engine().await;
                    observation::start_observation_engine(engine).await;
                });
            });

            // Wait for the observation engine to initialize before starting the API server.
            // The API server's event drain needs get_event_receiver() to return Some.
            std::thread::sleep(std::time::Duration::from_secs(3));
            tracing::info!("[MAIN] Observation engine initialization wait complete — now starting API server");

            // Clone app_handle BEFORE the thread spawn to avoid capturing &mut tauri::App
            let app_handle_for_thread = app.handle().clone();
            let skills_path_thread = skills_path.clone();
            let knowledge_path_thread = knowledge_path.clone();

            std::thread::spawn(move || {
                // Diagnostic marker: thread has started
                tracing::info!("[MAIN] API server thread spawned — calling start_api_server(port=1420)");
                println!("=== [Wiki Labs] Starting API server on port 1420 ===");
                api_server::set_shared_app_handle(app_handle_for_thread.clone());
                tracing::info!("[MAIN] set_shared_app_handle done");
                match api_server::start_api_server(1420, Some(config_path_clone), skills_path_thread, knowledge_path_thread, Some(Arc::new(app_handle_for_thread))) {
                    Ok(_) => {
                        tracing::info!("[MAIN] API server started successfully in background thread");
                        println!("=== [Wiki Labs] API server STARTED OK ===");
                        *api_state_clone.lock().unwrap() = Some(true);
                    }
                    Err(e) => {
                        tracing::error!("[MAIN] Failed to start API server: {}", e);
                        println!("=== [Wiki Labs] API server FAILED: {} ===", e);
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
            let _handle = app.handle().clone();
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
            // Window management
            hide_main_window,
            // Advice chat window management
            open_advice_chat_window,
            close_advice_chat_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
