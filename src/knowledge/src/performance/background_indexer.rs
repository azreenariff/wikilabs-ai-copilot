//! Background indexing using tokio tasks.

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, info};

use super::cancellation::CancellationToken;
use super::progress::{ProgressEvent, ProgressReporterHandle};

/// Configuration for background indexing.
pub struct BackgroundIndexerConfig {
    /// Interval between polling for new documents to index.
    pub poll_interval: Duration,
    /// Maximum number of concurrent indexing tasks.
    pub max_concurrent_tasks: usize,
}

impl Default for BackgroundIndexerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(60),
            max_concurrent_tasks: 4,
        }
    }
}

/// Background indexing manager that watches for document changes and queues indexing tasks.
pub struct BackgroundIndexer {
    config: BackgroundIndexerConfig,
    cancellation: CancellationToken,
    task_handle: Option<JoinHandle<()>>,
    progress_reporter: ProgressReporterHandle,
    doc_poll_sender: mpsc::Sender<DocChange>,
}

/// A document change event detected by the background indexer.
#[derive(Debug, Clone)]
pub struct DocChange {
    pub document_id: String,
    pub change_type: DocChangeType,
    pub modified_at: Option<String>,
}

/// Type of document change.
#[derive(Debug, Clone, PartialEq)]
pub enum DocChangeType {
    Created,
    Modified,
    Deleted,
}

impl BackgroundIndexer {
    /// Creates a new background indexer with the given config and progress reporter.
    pub fn new(config: BackgroundIndexerConfig, progress_reporter: ProgressReporterHandle) -> Self {
        let (tx, _rx) = mpsc::channel(100);
        Self {
            config,
            cancellation: CancellationToken::new(),
            task_handle: None,
            progress_reporter,
            doc_poll_sender: tx,
        }
    }

    /// Starts the background indexing loop.
    pub fn start<F>(&mut self, index_fn: F)
    where
        F: Fn(DocChange) -> tokio::task::JoinHandle<()> + Send + 'static,
    {
        if self.task_handle.is_some() {
            debug!("Background indexer already running");
            return;
        }

        let cancellation = self.cancellation.clone();
        let progress = self.progress_reporter.clone();
        let poll_interval = self.config.poll_interval;

        let handle = tokio::spawn(async move {
            let mut interval = interval(poll_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => {
                        info!("Background indexer stopped by cancellation");
                        break;
                    }
                    _ = interval.tick() => {
                        progress.report(ProgressEvent::Status {
                            message: "Checking for document changes...".to_string(),
                            progress: 0,
                            total: None,
                        });

                        // In a real implementation, this would check file
                        // system changes, database timestamps, etc.
                        // For now, we just report the check.
                        debug!("Poll interval elapsed, checking for changes");

                        // Placeholder: in real implementation, scan for changes
                        // and emit DocChange events to an internal channel.
                    }
                }
            }

            progress.report(ProgressEvent::Complete {
                message: "Background indexing stopped".to_string(),
            });
        });

        self.task_handle = Some(handle);
        info!(
            poll_interval = ?poll_interval,
            "Background indexer started"
        );
    }

    /// Stops the background indexer.
    pub fn stop(&mut self) {
        if self.task_handle.is_some() {
            self.cancellation.cancel();
            info!("Background indexer cancellation requested");
        }
    }

    /// Returns true if the background indexer is running.
    pub fn is_running(&self) -> bool {
        self.task_handle.is_some()
    }

    /// Queues a document change for indexing.
    ///
    /// In a real implementation, this would enqueue the change to be
    /// processed by the background task. Currently it reports the change
    /// via the progress reporter.
    pub fn queue_change(&self, change: DocChange) {
        let progress = self.progress_reporter.clone();

        let event = match change.change_type {
            DocChangeType::Created => ProgressEvent::Status {
                message: format!("New document detected: {}", change.document_id),
                progress: 0,
                total: None,
            },
            DocChangeType::Modified => ProgressEvent::Status {
                message: format!("Document modified: {}", change.document_id),
                progress: 50,
                total: Some(100),
            },
            DocChangeType::Deleted => ProgressEvent::Status {
                message: format!("Document deleted: {}", change.document_id),
                progress: 100,
                total: Some(100),
            },
        };

        progress.report(event);
    }

    /// Registers a document as changed for the next poll cycle.
    pub fn mark_changed(&self, document_id: &str, change_type: DocChangeType) {
        self.queue_change(DocChange {
            document_id: document_id.to_string(),
            change_type,
            modified_at: None,
        });
    }

    /// Returns a handle to the current cancellation token.
    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl Drop for BackgroundIndexer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_indexer() {
        let progress = ProgressReporterHandle::new();
        let config = BackgroundIndexerConfig::default();
        let indexer = BackgroundIndexer::new(config, progress);

        assert!(!indexer.is_running());
    }

    #[test]
    fn test_mark_changed() {
        let progress = ProgressReporterHandle::new();
        let config = BackgroundIndexerConfig::default();
        let indexer = BackgroundIndexer::new(config, progress);

        indexer.mark_changed("doc-123", DocChangeType::Modified);
        // No panics — just verifies queue_change works
    }

    #[test]
    fn test_cancellation_token() {
        let progress = ProgressReporterHandle::new();
        let config = BackgroundIndexerConfig::default();
        let indexer = BackgroundIndexer::new(config, progress);

        let token = indexer.cancellation();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }
}
