//! Progress reporting — trait-based progress events for indexing and retrieval.

use std::fmt;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::debug;

/// A progress event emitted during indexing or search operations.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// General status message.
    Status {
        message: String,
        /// Current progress value (0-100).
        progress: i32,
        /// Total items (if known, for percentage calculation).
        total: Option<u64>,
    },
    /// A specific document/item is being processed.
    Document {
        document_id: String,
        action: String,
    },
    /// The operation has completed successfully.
    Complete {
        message: String,
    },
    /// The operation encountered an error.
    Error {
        message: String,
    },
    /// A warning was issued during the operation.
    Warning {
        message: String,
    },
}

impl ProgressEvent {
    /// Returns a human-readable summary of the event.
    pub fn summary(&self) -> String {
        match self {
            ProgressEvent::Status { message, .. } => message.clone(),
            ProgressEvent::Document {
                document_id,
                action,
            } => format!("{}: {}", document_id, action),
            ProgressEvent::Complete { message } => message.clone(),
            ProgressEvent::Error { message } => format!("ERROR: {}", message),
            ProgressEvent::Warning { message } => format!("WARNING: {}", message),
        }
    }
}

/// Trait for objects that can report progress.
///
/// Implementations can forward progress events to callbacks,
/// channels, or other consumers.
pub trait ProgressReporter: Send + Sync + 'static {
    /// Reports a progress event.
    fn report(&self, event: ProgressEvent);
}

/// A concrete progress reporter backed by an async channel.
pub struct ChannelProgressReporter {
    sender: mpsc::Sender<ProgressEvent>,
}

impl ChannelProgressReporter {
    /// Creates a new channel-based progress reporter.
    pub fn new(sender: mpsc::Sender<ProgressEvent>) -> Self {
        Self { sender }
    }
}

impl ProgressReporter for ChannelProgressReporter {
    fn report(&self, event: ProgressEvent) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(event).await {
                debug!(error = %e, "Failed to send progress event");
            }
        });
    }
}

/// A handle for reporting progress from anywhere in the async runtime.
#[derive(Clone)]
pub struct ProgressReporterHandle {
    inner: Arc<Mutex<Option<mpsc::Sender<ProgressEvent>>>>,
}

impl ProgressReporterHandle {
    /// Creates a new handle (initially inactive).
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets the channel sender for this handle.
    pub async fn set_sender(&self, sender: mpsc::Sender<ProgressEvent>) {
        let mut inner = self.inner.lock().await;
        *inner = Some(sender);
    }

    /// Reports a progress event via the current sender.
    ///
    /// If no sender is set, the event is logged at debug level.
    pub fn report(&self, event: ProgressEvent) {
        let sender_clone = {
            let inner = self.inner.blocking_lock();
            inner.clone()
        };

        if let Some(sender) = sender_clone {
            let sender = sender.clone();
            tokio::spawn(async move {
                if let Err(e) = sender.send(event).await {
                    debug!(error = %e, "Failed to send progress event via handle");
                }
            });
        } else {
            debug!(message = %event.summary(), "Progress event (no sender set)");
        }
    }

    /// Checks if a sender has been set.
    pub fn has_sender(&self) -> bool {
        let inner = self.inner.blocking_lock();
        inner.is_some()
    }
}

impl Default for ProgressReporterHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// A callback-based progress reporter.
pub struct CallbackProgressReporter {
    callback: Box<dyn Fn(ProgressEvent) + Send + Sync>,
}

impl CallbackProgressReporter {
    /// Creates a new callback-based reporter.
    pub fn new(callback: impl Fn(ProgressEvent) + Send + Sync + 'static) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl ProgressReporter for CallbackProgressReporter {
    fn report(&self, event: ProgressEvent) {
        (self.callback)(event);
    }
}

/// Calculates a progress percentage from current and total counts.
///
/// Returns None if total is zero.
pub fn calculate_progress(current: u64, total: u64) -> Option<i32> {
    if total == 0 {
        return Some(100);
    }
    Some((current as f64 / total as f64 * 100.0) as i32)
}

impl fmt::Display for ProgressEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_progress() {
        assert_eq!(calculate_progress(0, 10), Some(0));
        assert_eq!(calculate_progress(5, 10), Some(50));
        assert_eq!(calculate_progress(10, 10), Some(100));
        assert_eq!(calculate_progress(3, 10), Some(30));
    }

    #[test]
    fn test_calculate_progress_zero_total() {
        // When total is zero, treat as complete
        assert_eq!(calculate_progress(0, 0), Some(100));
    }

    #[test]
    fn test_progress_event_display() {
        let event = ProgressEvent::Status {
            message: "Processing documents".to_string(),
            progress: 50,
            total: Some(100),
        };
        assert!(format!("{}", event).contains("Processing documents"));

        let event = ProgressEvent::Complete {
            message: "Done".to_string(),
        };
        assert!(format!("{}", event).contains("Done"));

        let event = ProgressEvent::Error {
            message: "Failed".to_string(),
        };
        assert!(format!("{}", event).contains("ERROR"));
    }

    #[test]
    fn test_progress_event_summary() {
        let event = ProgressEvent::Document {
            document_id: "doc-1".to_string(),
            action: "indexing".to_string(),
        };
        assert_eq!(event.summary(), "doc-1: indexing");
    }

    #[test]
    fn test_callback_reporter() {
        let reported = Arc::new(Mutex::new(Vec::new()));
        let reported_clone = reported.clone();

        let reporter = CallbackProgressReporter::new(move |event: ProgressEvent| {
            reported_clone.lock().unwrap().push(event.summary());
        });

        reporter.report(ProgressEvent::Status {
            message: "test".to_string(),
            progress: 10,
            total: None,
        });

        let reports = reported.lock().unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0], "test");
    }

    #[test]
    fn test_handle_with_no_sender() {
        let handle = ProgressReporterHandle::new();
        assert!(!handle.has_sender());

        // Should not panic even with no sender
        handle.report(ProgressEvent::Status {
            message: "test".to_string(),
            progress: 10,
            total: None,
        });
    }
}