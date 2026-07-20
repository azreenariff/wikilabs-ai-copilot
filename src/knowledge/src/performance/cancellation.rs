//! Cancellation support — cooperative cancellation for long-running operations.

use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use tracing::debug;

/// A token that can be shared across tasks to request cancellation.
///
/// Uses tokio's watch channel for efficient broadcast. When `cancel()` is called,
/// all holders of the token are notified that cancellation has been requested.
/// This is cooperative — tasks must check `is_cancelled()` or use `cancelled()`
/// to respond to the cancellation signal.
#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<CancelInner>,
}

struct CancelInner {
    sender: watch::Sender<bool>,
    /// Keep a receiver alive so that sender.send() doesn't fail.
    _receiver: watch::Receiver<bool>,
    /// Parent sender for child tokens (optional).
    /// Wrapped in Mutex for mutation in child_token().
    parent: Mutex<Option<watch::Sender<bool>>>,
}

impl CancellationToken {
    /// Creates a new cancellation token (initially not cancelled).
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(false);
        Self {
            inner: Arc::new(CancelInner {
                sender,
                _receiver: receiver,
                parent: Mutex::new(None),
            }),
        }
    }

    /// Requests cancellation. All holders will see `is_cancelled()` return true.
    pub fn cancel(&self) {
        let _ = self.inner.sender.send(true);
        debug!("Cancellation requested");
    }

    /// Returns true if cancellation has been requested.
    /// For child tokens, also checks if the parent is cancelled.
    pub fn is_cancelled(&self) -> bool {
        // Check parent first (guard must be dropped before we access sender)
        {
            let parent_guard = self.inner.parent.lock().unwrap();
            if let Some(ref parent) = *parent_guard {
                if *parent.borrow() {
                    return true;
                }
            }
        }
        // Then check own state
        *self.inner.sender.borrow()
    }

    /// Returns a oneshot-based future that resolves when cancellation is requested.
    ///
    /// This can be used in `tokio::select!` to await cancellation.
    pub async fn cancelled(&self) {
        let mut rx = self.inner.sender.subscribe();
        if *rx.borrow() {
            return;
        }
        // Use a flag to avoid spurious wakeups
        let mut flag = false;
        loop {
            if *rx.borrow() {
                break;
            }
            if flag {
                break;
            }
            match rx.changed().await {
                Ok(_) => {
                    if *rx.borrow() {
                        break;
                    }
                    flag = true;
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    /// Creates a child cancellation token linked to this one.
    ///
    /// If this token is cancelled, the child will also be cancelled.
    /// The child can be cancelled independently without affecting the parent.
    pub fn child_token(&self) -> CancellationToken {
        let parent_cancelled = self.is_cancelled();

        // Create child with parent reference
        let child = CancellationToken::new();
        *child.inner.parent.lock().unwrap() = Some(self.inner.sender.clone());

        // If parent was already cancelled, mark child as cancelled too
        if parent_cancelled {
            child.cancel();
        }

        child
    }

    /// Returns true if cancellation was requested before this token was created.
    ///
    /// Useful for checking if an ancestor already cancelled.
    pub fn is_already_cancelled(&self) -> bool {
        self.is_cancelled()
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// An optional cancellation token that can be None.
///
/// Useful when cancellation is optional for an operation.
#[derive(Clone)]
pub struct OptionalCancelToken {
    token: Option<CancellationToken>,
}

impl OptionalCancelToken {
    /// Creates a new optional cancel token with no cancellation.
    pub fn none() -> Self {
        Self { token: None }
    }

    /// Creates a new optional cancel token from a regular token.
    pub fn some(token: CancellationToken) -> Self {
        Self { token: Some(token) }
    }

    /// Returns true if a token is present and cancelled.
    pub fn is_cancelled(&self) -> bool {
        match &self.token {
            Some(token) => token.is_cancelled(),
            None => false,
        }
    }

    /// Cancels if a token is present.
    pub fn cancel(&self) {
        if let Some(token) = &self.token {
            token.cancel();
        }
    }

    /// Returns a oneshot-based future for cancellation.
    pub async fn cancelled(&self) {
        if let Some(token) = &self.token {
            token.cancelled().await;
        }
    }

    /// Returns the inner token if present.
    pub fn inner(&self) -> Option<&CancellationToken> {
        self.token.as_ref()
    }
}

impl Default for OptionalCancelToken {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_not_cancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancel() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_multiple_holders() {
        let token = CancellationToken::new();

        let token2 = token.clone();
        assert!(!token.is_cancelled());
        assert!(!token2.is_cancelled());

        token.cancel();
        assert!(token2.is_cancelled());
    }

    #[tokio::test]
    async fn test_child_token_inherits() {
        let token = CancellationToken::new();
        let child = token.child_token();

        assert!(!child.is_cancelled());
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(child.is_cancelled());
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_child_token_independent_cancel() {
        let token = CancellationToken::new();
        let child = token.child_token();

        child.cancel();
        assert!(child.is_cancelled());
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_optional_cancel_none() {
        let opt = OptionalCancelToken::none();
        assert!(!opt.is_cancelled());
        opt.cancel(); // should not panic
        assert!(!opt.is_cancelled());
    }

    #[tokio::test]
    async fn test_optional_cancel_some() {
        let token = CancellationToken::new();
        let opt = OptionalCancelToken::some(token.clone());

        assert!(!opt.is_cancelled());
        opt.cancel();
        assert!(token.is_cancelled());
    }
}
