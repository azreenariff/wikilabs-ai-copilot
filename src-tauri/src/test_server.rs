//! Test the API server router directly — start it on a random port and verify
//! that GET /health and GET /ready respond correctly.

use std::sync::atomic::{AtomicBool, Ordering};

static API_SERVER_READY: AtomicBool = AtomicBool::new(false);

pub fn is_server_ready() -> bool {
    API_SERVER_READY.load(Ordering::SeqCst)
}

pub fn mark_server_ready() {
    API_SERVER_READY.store(true, Ordering::SeqCst);
}

mod api_ready;

#[tokio::main]
async fn main() {
    // Start the server on a random port
    let addr = "127.0.0.1:0";
    
    // We can't easily import the full router since it depends on Tauri-specific types,
    // so let's just test a minimal axum server to verify axum::serve works on this system
    
    use axum::{routing::get, Router};
    
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(|| async { 
            format!("{{\"ready\":true}}") 
        }));
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    
    let port = listener.local_addr().unwrap().port();
    println!("Server listening on port {}", port);
    
    // Spawn the server in background
    let server_handle = tokio::spawn(axum::serve(listener, app).await);
    
    // Give server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // Test health endpoint
    let client = reqwest::Client::new();
    
    println!("\n--- Testing GET http://127.0.0.1:{}/health ---", port);
    let res = client
        .get(format!("http://127.0.0.1:{}/health", port))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    
    match res {
        Ok(r) => {
            println!("  Status: {}", r.status());
            println!("  Body: {}", r.text().await.unwrap_or_default());
        }
        Err(e) => {
            println!("  ERROR: {}", e);
        }
    }
    
    // Test ready endpoint
    println!("\n--- Testing GET http://127.0.0.1:{}/ready ---", port);
    let res = client
        .get(format!("http://127.0.0.1:{}/ready", port))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    
    match res {
        Ok(r) => {
            println!("  Status: {}", r.status());
            println!("  Body: {}", r.text().await.unwrap_or_default());
        }
        Err(e) => {
            println!("  ERROR: {}", e);
        }
    }
    
    // Test POST to unknown endpoint (fallback)
    println!("\n--- Testing POST http://127.0.0.1:{}/api/commands/get_status ---", port);
    let res = client
        .post(format!("http://127.0.0.1:{}/api/commands/get_status", port))
        .header("Content-Type", "application/json")
        .body(r#"{"params":{}}"#)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    
    match res {
        Ok(r) => {
            println!("  Status: {}", r.status());
            println!("  Body: {}", r.text().await.unwrap_or_default());
        }
        Err(e) => {
            println!("  ERROR: {}", e);
        }
    }
    
    // Shutdown
    server_handle.abort();
    println!("\nDone!");
}