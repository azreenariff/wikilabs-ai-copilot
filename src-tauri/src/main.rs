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
mod guidance_loop;
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
fn list_models(endpoint: String, api_key: String) -> Result<Vec<String>, String> {
    if endpoint.is_empty() {
        return Err("Endpoint is required".to_string());
    }

    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    info!(url = %url, "Listing AI models");

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let models = rt.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let mut request = client.get(&url);
        if !api_key.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await.map_err(|e| e.to_string())?;
        let status = response.status();

        if !status.is_success() {
            return Err(format!("HTTP {}: model listing failed", status));
        }

        let body = response.text().await.map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse models response: {}", e))?;

        // Handle OpenAI format ("data": [...]) — also supports Ollama bare array
        let models: Vec<String> = if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
            data.iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str().map(String::from)))
                .collect()
        } else if let Some(arr) = json.as_array() {
            arr.iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str().map(String::from)))
                .collect()
        } else {
            return Err("Unexpected response format: expected 'data' array or bare array".to_string());
        };

        info!(count = models.len(), "Model list retrieved");
        Ok(models)
    });

    models
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
            content: serde_json::Value::String(system_prompt),
        },
    ];

    // Add observation context as a system message — tells the AI what's happening
    // in the user's environment so it can give context-aware guidance
    let observation_context = build_observation_context();
    if !observation_context.is_empty() {
        messages.push(wikilabs_ai::provider::AiMessage {
            role: "system".to_string(),
            content: serde_json::Value::String(format!(
                "[OBSERVATION CONTEXT]\n{}", observation_context
            )),
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
            content: serde_json::Value::String(msg.content.clone()),
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
    // Extract text from response content (Value → String)
    let resp_content = match &response.message.content {
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

    let assistant_msg = ChatMessage::assistant(&resp_content);
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
            &resp_content,
            "[]",
        )
        .map_err(|e| e.to_string())?;

    Ok(ChatResponse {
        id: assistant_id,
        role: "assistant".to_string(),
        content: resp_content,
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

    if summary.current_activity.is_empty() && summary.intent.is_none() && summary.issues.is_empty() && summary.infrastructure_context.is_empty() && summary.suggested_guidance.is_empty() {
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
            wikilabs_observation::ActivityCategory::VisualInsight => "👁️ visual insight",
            wikilabs_observation::ActivityCategory::VisualError => "👁️ visual error",
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
            wikilabs_observation::ActivityCategory::VisualInsight => "👁️ visual insight",
            wikilabs_observation::ActivityCategory::VisualError => "👁️ visual error",
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
        context.push_str("\n## Suggested Guidance\n");
        for guidance in &summary.suggested_guidance {
            context.push_str(&format!("- {}\n", guidance.chars().take(300).collect::<String>()));
        }
    }

    // ── Infrastructure Context ──
    if !summary.infrastructure_context.is_empty() {
        context.push_str("\n## Infrastructure Context\n");
        for ctx in &summary.infrastructure_context {
            context.push_str(&format!("- {}\n", ctx.chars().take(300).collect::<String>()));
        }
    }

    // ── Command Correctness ──
    // Phase 4: Include terminal command correctness checks
    if !summary.command_correctness.is_empty() {
        let correct_count = summary.command_correctness.iter().filter(|c| c.is_correct).count();
        let incorrect_count = summary.command_correctness.len() - correct_count;
        context.push_str(&format!(
            "\n## Terminal Command Checks\n- {} correct, {} flagged for review\n",
            correct_count, incorrect_count
        ));
        for cmd in &summary.command_correctness {
            if !cmd.is_correct {
                let icon = match cmd.confidence {
                    c if c >= 0.9 => "🚨",
                    c if c >= 0.7 => "⚠️",
                    _ => "💡",
                };
                context.push_str(&format!(
                    "- {} **{}**: {}\n",
                    icon,
                    cmd.explanation,
                    cmd.command.chars().take(150).collect::<String>()
                ));
                if let Some(ref alt) = cmd.suggested_alternative {
                    context.push_str(&format!("  → {}\n", alt.chars().take(400).collect::<String>()));
                }
            } else {
                context.push_str(&format!(
                    "- ✅ **OK**: {}\n",
                    cmd.command.chars().take(120).collect::<String>()
                ));
            }
        }
    }

    // ── Correlated Insight ──
    // Help the AI connect the dots across browser + terminal + vision
    if !summary.issues.is_empty() && (!summary.current_activity.is_empty() || summary.intent.is_some()) {
        context.push_str("\n## What to Focus On\n");
        // Priority: errors first, then intent, then related context
        context.push_str("- **Fix errors first** — resolve detected issues before anything else\n");
        if let Some(ref intent) = summary.intent {
            context.push_str(&format!("- **User is trying to**: {} (confidence: {:.0}%)\n", intent.intent, intent.confidence * 100.0));
            if !intent.infrastructure_targets.is_empty() {
                context.push_str(&format!("- **Key systems**: {}\n", intent.infrastructure_targets.join(", ")));
            }
        }
        if !summary.current_activity.iter().any(|a| matches!(a.category, wikilabs_observation::ActivityCategory::VisualError)) {
            for activity in &summary.current_activity {
                if matches!(activity.category, wikilabs_observation::ActivityCategory::VisualInsight | wikilabs_observation::ActivityCategory::Troubleshooting) {
                    context.push_str(&format!("- **Watch for**: {}\n", activity.description.chars().take(150).collect::<String>()));
                    break;
                }
            }
        }
    }

    tracing::debug!(
        "Intent analysis produced context in {} µs, {} activities, {} issues, {} guidance items, {} infrastructure context",
        start.elapsed().as_micros(),
        summary.current_activity.len(),
        summary.issues.len(),
        summary.suggested_guidance.len(),
        summary.infrastructure_context.len()
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
    info!("Opening advice chat in main window");
    if let Some(main_window) = app.get_webview_window("main") {
        // Show and focus the main window (it was hidden after setup)
        let _ = main_window.show();
        let _ = main_window.set_focus();
        // Reset to original centered size (1400x900 from tauri.conf.json)
        let _ = main_window.set_position(tauri::PhysicalPosition::new(0, 0));
        let _ = main_window.set_size(tauri::PhysicalSize::new(1400, 900));
        // Navigate to the advice-chat route using the main app's React Router
        // This keeps the same SPA loaded — no full page reload needed
        let _ = main_window.eval("window.history.pushState({}, '', '/advice-chat'); window.dispatchEvent(new PopStateEvent('popstate'));");
        info!("Main window navigated to advice-chat route via React Router");
        Ok(())
    } else {
        info!("Main window not found, cannot open advice chat");
        Ok(())
    }
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
    // ── Log path: platform-aware ──
    // On Windows: %APPDATA%/wikilabs-ai-copilot/logs
    // On Linux/macOS: $XDG_DATA_HOME/wikilabs-ai-copilot/logs or ~/.local/share/wikilabs-ai-copilot/logs
    #[cfg(target_os = "windows")]
    let log_dir = PathBuf::from(
        std::env::var("APPDATA")
            .ok()
            .map(|d| format!("{}/wikilabs-ai-copilot/logs", d))
            .unwrap_or_else(|| {
                let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
                format!("{}/wikilabs-ai-copilot/logs", home)
            }),
    );
    #[cfg(not(target_os = "windows"))]
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

    // ── Startup diagnostic ──
    // Log WebView2/runtime info so blank-screen issues are diagnosable
    tracing::info!(
        "[STARTUP] Platform: {}, Rust edition: 2021, Tauri v2, PID: {}",
        std::env::consts::OS,
        std::process::id(),
    );
    // Log the expected data directory so we can verify it exists
    let _startup_log_dir = log_dir.clone();
    tracing::info!("[STARTUP] Log directory: {}", _startup_log_dir.display());

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
        .setup(move |app| {
            let state_setup_start = Instant::now();
            info!("[LIFECYCLE] setup() hook entered — creating AppState");

            // ── Create the main webview window with additional browser args ──
            // On Windows, some systems fail to render the WebView2 due to GPU driver issues.
            // We pass --disable-gpu and --no-sandbox to ensure the renderer starts.
            // This is especially important for users with older GPUs or virtual machines.
            #[cfg(target_os = "windows")]
            {
                let window_config = &app.config().app.windows[0];
                let builder = tauri::webview::WebviewWindowBuilder::from_config(app.handle(), window_config)
                    .map_err(|e| {
                        error!("[LIFECYCLE] Failed to create WebviewWindowBuilder from config: {}", e);
                        e
                    })?;
                // Apply GPU-disabling browser args to fix blank screen on some Windows systems
                let builder = builder.additional_browser_args("--disable-gpu --no-sandbox --disable-software-rasterizer");
                let window = builder.build().map_err(|e| {
                    error!("[LIFECYCLE] Failed to build main webview window: {}", e);
                    e
                })?;
                info!("[LIFECYCLE] Main webview window created with GPU-disabling args");
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Non-Windows: also create the window manually since config has create=false
                let window_config = &app.config().app.windows[0];
                let builder = tauri::webview::WebviewWindowBuilder::from_config(app.handle(), window_config)
                    .map_err(|e| {
                        error!("[LIFECYCLE] Failed to create WebviewWindowBuilder from config: {}", e);
                        e
                    })?;
                let _window = builder.build().map_err(|e| {
                    error!("[LIFECYCLE] Failed to build main webview window: {}", e);
                    e
                })?;
                info!("[LIFECYCLE] Main webview window created (non-Windows)");
            }

            let state = AppState::new(app.handle().clone())?;
            let state_time = state_setup_start.elapsed();
            info!(
                "[LIFECYCLE] AppState initialized in {} µs",
                state_time.as_micros()
            );

            // Construct config path for API server persistence
            let data_dir = app.handle().path().app_data_dir()?;
            let config_path = data_dir.join("settings.json");
            tracing::info!("[LIFECYCLE] Config path: {}", config_path.display());

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

            // Lazy-start observation engine: only begin observation + AI reasoning
            // when the user has already configured an API key (i.e., not the first-run
            // setup wizard). This prevents the observation/AI loop from starving the
            // API server's Tokio runtime before the user finishes configuring their AI provider.
            let config_path_for_check = config_path.clone();
            let should_start_observation = std::fs::read_to_string(&config_path_for_check)
                .map(|content| {
                    let parsed = serde_json::from_str::<serde_json::Value>(&content);
                    if let Ok(parsed_val) = parsed {
                        parsed_val.get("ai_provider")
                            .and_then(|p| p.get("api_key"))
                            .and_then(|k| k.as_str())
                            .map(|key| !key.is_empty())
                            .unwrap_or(false)
                    } else {
                        false
                    }
                })
                .unwrap_or(false);

            let obs_should_run = should_start_observation;
            info!(
                obs_should_run,
                "Observation engine launch condition checked (api_key configured: {})",
                should_start_observation
            );

            if obs_should_run {
                // Spawn observation engine on its own thread (non-blocking)
                let obs_app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    info!("[LIFECYCLE] Observation engine thread started (lazy start — api_key configured)");
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(async {
                        let engine = observation::init_observation_engine().await;
                        info!("[LIFECYCLE] Observation engine initialized");
                        observation::start_observation_engine(engine).await;
                    });
                });
            } else {
                info!("[LIFECYCLE] Observation engine LAUNCHED LAZILY — waiting until api_key is configured");
            }

            // No longer wait 3s for observation engine — it runs independently.
            // The API server handles missing events gracefully (returns empty).
            info!("[LIFECYCLE] Spawning API server thread (no delay)");

            // Clone app_handle BEFORE the thread spawn to avoid capturing &mut tauri::App
            let app_handle_for_thread = app.handle().clone();
            let skills_path_thread = skills_path.clone();
            let knowledge_path_thread = knowledge_path.clone();

            std::thread::spawn(move || {
                info!("[LIFECYCLE] API server thread spawned — calling start_api_server(port=1420)");
                println!("=== [Wiki Labs] Starting API server on port 1420 ===");
                api_server::set_shared_app_handle(app_handle_for_thread.clone());
                info!("[LIFECYCLE] set_shared_app_handle done");
                match api_server::start_api_server(1420, Some(config_path_clone), skills_path_thread, knowledge_path_thread, Some(Arc::new(app_handle_for_thread))) {
                    Ok(_) => {
                        info!("[LIFECYCLE] API server started successfully in background thread");
                        println!("=== [Wiki Labs] API server STARTED OK ===");
                        *api_state_clone.lock().unwrap() = Some(true);
                    }
                    Err(e) => {
                        error!("[LIFECYCLE] Failed to start API server: {}", e);
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
                    .with_metadata("config_load_us", "0")
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
            list_models,
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
    info!("[LIFECYCLE] Tauri run() exited — application shutting down");
}
