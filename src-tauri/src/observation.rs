//! Observation Engine integration — starts providers, polls them,
//! and feeds detected errors/suggestions into the guidance panel.

use std::sync::Arc;
use crossbeam_channel::Receiver;
use wikilabs_observation::engine::{ObservationEngine, ObservationEngineConfig};
use wikilabs_observation::event::ObservationEvent;

/// Shared observation engine instance (global singleton).
static mut OBSERVATION_ENGINE: Option<Arc<ObservationEngine>> = None;

/// Shared event receiver for downstream consumers (api_server, guidance panel).
static mut EVENT_RECEIVER: Option<Receiver<ObservationEvent>> = None;

/// Initialize the observation engine, register providers, and create
/// a shared event subscription for downstream consumers.
pub async fn init_observation_engine() -> Arc<ObservationEngine> {
    // Create the engine
    let config = ObservationEngineConfig::default();
    let engine = Arc::new(ObservationEngine::new(config));
    println!("[OBS] >>> ObservationEngine created");

    // Register available providers
    register_providers(&engine).await;
    println!("[OBS] >>> Providers registered");

    // Subscribe to the event bus and store the receiver globally
    let bus = engine.event_bus().clone();
    let (sub, rx) = bus.subscribe_all();
    unsafe {
        EVENT_RECEIVER = Some(rx);
    }
    tracing::info!(
        "[Observation] Event subscription created: {}",
        sub.id
    );

    // Store the engine as a global for other modules to access
    unsafe {
        OBSERVATION_ENGINE = Some(engine.clone());
    }

    tracing::info!(
        "[Observation] Observation engine initialized with providers"
    );
    println!("[OBS] >>> Observation engine initialized OK — global store set");
    engine
}

/// Get the shared observation engine.
pub fn get_observation_engine() -> Option<Arc<ObservationEngine>> {
    unsafe { OBSERVATION_ENGINE.clone() }
}

/// Get the shared event receiver for observation events.
/// Returns None if the engine hasn't been initialized yet.
pub fn get_event_receiver() -> Option<Receiver<ObservationEvent>> {
    unsafe { EVENT_RECEIVER.clone() }
}

/// Get the latest Vision analysis result.
/// Delegates to the global observation engine.
pub fn get_vision_result() -> Option<wikilabs_observation::vision_analyzer::VisionAnalysisResult> {
    get_observation_engine().and_then(|engine| engine.get_vision_result())
}

/// Start the observation engine providers.
pub async fn start_observation_engine(engine: Arc<ObservationEngine>) {
    tracing::info!("[Observation] Starting observation providers");
    println!("[OBS] >>> Starting observation providers");

    let results = engine.start().await;
    println!("[OBS] >>> engine.start() returned — {} results", results.len());
    for (name, result) in &results {
        match result {
            Ok(_) => {
                tracing::info!("[Observation] Provider {} started", name);
                println!("[OBS] >>> Provider {} started OK", name);
            }
            Err(e) => {
                tracing::error!(
                    "[Observation] Provider {} failed: {}",
                    name,
                    e
                );
                println!("[OBS] >>> Provider {} FAILED: {}", name, e);
            }
        }
    }
    println!("[OBS] >>> All providers started — spawning polling loop");

    // Start the polling loop in a background task and block the thread until engine stops
    // This ensures the tokio runtime stays alive while the polling loop runs
    // Clone engine Arc for the blocking loop (spawn moves one Arc)
    let engine_clone = engine.clone();
    tokio::spawn(async move {
        engine.run_loop().await;
        println!("[OBS] >>> Polling loop exited");
    });

    // Block the observation thread so the tokio runtime stays alive
    // and the polling loop continues running. The runtime will only
    // be dropped (and the loop cancelled) when the app exits.
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        let running = engine_clone.is_running().await;
        if !running {
            break;
        }
    }
    println!("[OBS] >>> Observation thread exiting");
}

/// Register all available providers with the engine.
async fn register_providers(engine: &ObservationEngine) {
    println!("[OBS] >>> Starting provider registration");
    // Active window provider — works on all platforms
    {
        let provider =
            wikilabs_observation::app_monitor::ActiveWindowProvider::new();
        tracing::info!("[Observation] Registering app_monitor provider");
        println!("[OBS] >>> Registered app_monitor provider");
        engine.register_provider(Box::new(provider)).await;
    }

    // Clipboard provider — works on all platforms
    {
        let provider =
            wikilabs_observation::clipboard::ClipboardProvider::new();
        tracing::info!("[Observation] Registering clipboard provider");
        println!("[OBS] >>> Registered clipboard provider");
        engine.register_provider(Box::new(provider)).await;
    }

    // Browser provider — Windows only (Win32 API for URL extraction)
    #[cfg(target_os = "windows")]
    {
        let provider =
            wikilabs_observation::browser::BrowserProvider::new();
        tracing::info!(
            "[Observation] Registering browser provider"
        );
        println!("[OBS] >>> Registered browser provider (Windows)");
        engine.register_provider(Box::new(provider)).await;
    }

    // Screen capture provider — captures screenshots, feeds to vision analyzer
    #[cfg(target_os = "windows")]
    {
        let provider = wikilabs_observation::screen_capture::ScreenCaptureProvider::new();
        tracing::info!("[Observation] Registering screen_capture provider");
        println!("[OBS] >>> Registered screen_capture provider (Windows)");
        engine.register_provider(Box::new(provider)).await;
    }

    // Vision analysis provider — analyzes screenshots via Vision AI
    {
        let provider = wikilabs_observation::vision_analyzer::VisionAnalyzerProvider::new();
        tracing::info!("[Observation] Registering vision_analyzer provider");
        println!("[OBS] >>> Registered vision_analyzer provider");
        engine.register_provider(Box::new(provider)).await;
    }
    println!("[OBS] >>> Provider registration complete");

    let count = {
        let mut c = 2; // active_window + clipboard (always)
        c += 1; // vision_analyzer (always)
        #[cfg(target_os = "windows")]
        { c += 2; } // browser + screen_capture (Windows)
        c
    };
    tracing::info!("[Observation] {} providers registered", count);
}