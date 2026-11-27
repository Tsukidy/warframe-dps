//! Optimizer events (optimizer.md §9; `OptEvent` of `wfd_model::api`, server-api.md §8.2).
//!
//! Owner: T33. [`EventSink`] is where [`crate::optimize`] sends its events; the server's sink feeds the job's ring
//! buffer and the SSE subscribers (server-api.md §11), the CLI's prints NDJSON. `Progress` is throttled to 4 per second
//! ([`PROGRESS_MIN_INTERVAL`]); every change of the top 1 is an `Incumbent` event, top-5 changes are throttled.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use wfd_model::api::OptEvent;

/// Receives the events of one job, in order (optimizer.md §1). Called from worker threads; implementations must not
/// block for long.
pub trait EventSink: Send + Sync {
    /// Receives one event.
    fn emit(&self, e: OptEvent);
}

/// Any thread-safe closure is a sink.
impl<F> EventSink for F
where
    F: Fn(OptEvent) + Send + Sync,
{
    fn emit(&self, e: OptEvent) {
        self(e)
    }
}

/// Minimum time between two `Progress` events (optimizer.md §9: "throttled to 4 per second").
pub const PROGRESS_MIN_INTERVAL: Duration = Duration::from_millis(250);

/// A sink that drops every event.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullSink;

impl EventSink for NullSink {
    fn emit(&self, _e: OptEvent) {}
}

/// A sink that keeps every event in order (tests, the CLI's buffered output).
#[derive(Debug, Default)]
pub struct CollectSink {
    events: Mutex<Vec<OptEvent>>,
}

impl CollectSink {
    /// An empty sink.
    pub fn new() -> Self {
        CollectSink::default()
    }

    /// Takes the events received so far, leaving the sink empty.
    pub fn take(&self) -> Vec<OptEvent> {
        std::mem::take(&mut *self.events.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// Number of events held.
    pub fn len(&self) -> usize {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    /// Whether the sink holds no event.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl EventSink for CollectSink {
    fn emit(&self, e: OptEvent) {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(e);
    }
}
