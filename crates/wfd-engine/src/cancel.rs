//! Cooperative cancellation (engine.md §1, §8.3). Complete in T02a. Owner: T17 (manager ruling C-31).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::Cancelled;

/// A shared cancellation flag: the server cancels a superseded calculation (server-api.md §11), MC checks it between
/// runs (engine.md §8.3). Clones share the flag.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// A token that is not cancelled.
    pub fn new() -> Self {
        CancelToken::default()
    }

    /// Requests cancellation; every clone sees it.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// `Err(Cancelled)` once cancellation was requested.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_the_flag() {
        let t = CancelToken::new();
        let c = t.clone();
        assert!(t.check().is_ok());
        c.cancel();
        assert!(t.is_cancelled());
        assert_eq!(t.check(), Err(Cancelled));
    }
}
