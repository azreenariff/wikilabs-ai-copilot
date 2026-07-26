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
pub fn init_observation_engine() -> Arc<ObservationEngine> {
    // Create the engine
    let config = ObservationEngineConfig::default();
    let engine = Arc::new(ObservationEngine::new(config));

    // Register available providers
    register_providers(&engine);

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

/// Start the observation engine providers.
pub async fn start_observation_engine(engine: Arc<ObservationEngine>) {
    tracing::info!("[Observation] Starting observation providers");

    let results = engine.start().await;
    for (name, result) in &results {
        match result {
            Ok(_) => tracing::info!("[Observation] Provider {} started", name),
            Err(e) => {
                tracing::error!(
                    "[Observation] Provider {} failed: {}",
                    name,
                    e
                )
            }
        }
    }

    // Start the polling loop in a background task
    tokio::spawn(async move {
        engine.run_loop().await;
    });
}

/// Register all available providers with the engine.
fn register_providers(engine: &ObservationEngine) {
    // Active window provider
    #[cfg(target_os = "linux")]
    {
        let provider =
            wikilabs_observation::app_monitor::ActiveWindowProvider::new();
        tracing::info!("[Observation] Registering app_monitor provider");
        engine.register_provider(Box::new(provider));
    }

    // Clipboard provider
    #[cfg(target_os = "linux")]
    {
        let provider =
            wikilabs_observation::clipboard::ClipboardProvider::new();
        tracing::info!("[Observation] Registering clipboard provider");
        engine.register_provider(Box::new(provider));
    }

    // Screen capture provider
    #[cfg(target_os = "linux")]
    {
        let provider =
            wikilabs_observation::screen_capture::ScreenCaptureProvider::new();
        tracing::info!(
            "[Observation] Registering screen_capture provider"
        );
        engine.register_provider(Box::new(provider));
    }

    let count = {
        let mut c = 0;
        #[cfg(target_os = "linux")]
        {
            c += 1;
        }
        #[cfg(target_os = "linux")]
        {
            c += 1;
        }
        #[cfg(target_os = "linux")]
        {
            c += 1;
        }
        c
    };
    tracing::info!("[Observation] {} providers registered", count);
}