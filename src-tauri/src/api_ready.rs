//! Readiness tracking for the API server startup sequence.
//!
//! The frontend must wait for the server to be fully initialized
//! (knowledge packs loaded, axum server listening) before making API calls.
//! This module provides a static AtomicBool that is set to true once
//! the server is ready to handle requests.

use std::sync::atomic::{AtomicBool, Ordering};

static API_SERVER_READY: AtomicBool = AtomicBool::new(false);

/// Returns true if the API server has finished initialization and is ready to handle requests.
pub fn is_server_ready() -> bool {
    API_SERVER_READY.load(Ordering::SeqCst)
}

/// Marks the API server as ready. Called when axum::serve starts listening.
pub fn mark_server_ready() {
    API_SERVER_READY.store(true, Ordering::SeqCst);
}