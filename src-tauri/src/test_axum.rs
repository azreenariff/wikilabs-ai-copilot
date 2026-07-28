use axum::{routing::{get, post}, Router, http::StatusCode, Extension};
use serde::Deserialize;
use serde_json::Value;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct State {
    settings: std::sync::Arc<std::sync::Mutex<()>>,
}

#[derive(Deserialize)]
struct ApiRequest { params: Value }

async fn api_handler(
    Extension(_state): Extension<State>,
    _path: axum::extract::Path<String>,
    _req: axum::extract::Json<ApiRequest>,
) -> (StatusCode, String) {
    (StatusCode::OK, "ok".to_string())
}

#[tokio::main]
async fn main() {
    eprintln!("[TEST] Starting axum server test...");
    
    let state = State {
        settings: std::sync::Arc::new(std::sync::Mutex::new(())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/api/commands/:method", post(api_handler))
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(|| async { r#"{"ready":true}"#.to_string() }))
        .nest_service("/assets", ServeDir::new("/home/khopu/wikilabs-ai-copilot/src-tauri/assets"))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:17899").await.unwrap();
    eprintln!("[TEST] Listening on 127.0.0.1:17899");

    let server = tokio::spawn(async {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("[TEST] Server error: {}", e);
        }
    });
    
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let client = reqwest::Client::new();
    
    // Test 1: /health
    eprintln!("[TEST] Testing GET /health...");
    let rt = tokio::time::Instant::now();
    match client
        .get("http://127.0.0.1:17899/health")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(r) => eprintln!("[TEST] GET /health -> {} (body: {}, took {:?})", r.status(), r.text().await.unwrap_or_default(), rt.elapsed()),
        Err(e) => eprintln!("[TEST] GET /health -> ERROR: {}", e),
    }

    // Test 2: /api/commands/get_status
    eprintln!("[TEST] Testing POST /api/commands/get_status...");
    let rt = tokio::time::Instant::now();
    match client
        .post("http://127.0.0.1:17899/api/commands/get_status")
        .header("Content-Type", "application/json")
        .body(r#"{"params":{}}"#)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(r) => eprintln!("[TEST] POST /api/commands/get_status -> {} (body: {}, took {:?})", r.status(), r.text().await.unwrap_or_default(), rt.elapsed()),
        Err(e) => eprintln!("[TEST] POST /api/commands/get_status -> ERROR: {}", e),
    }

    server.abort();
    server.await.ok();
    eprintln!("[TEST] Done!");
}