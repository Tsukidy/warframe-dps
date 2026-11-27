//! Progress reporting and cancellation (pipeline.md §2; frozen API of T02c). The server turns [`Progress`] calls into
//! `DataEvent::Step` and `DataEvent::Log` (server-api.md §9); the CLI prints them.

use crate::DatagenError;
use std::sync::atomic::{AtomicBool, Ordering};
use wfd_model::message::Message;

/// The pipeline steps of pipeline.md §2: the `DataStep` of the API (`wfd_model::api::DataStep`), so progress maps to
/// `DataEvent::Step` without conversion.
pub type PipelineStep = wfd_model::api::DataStep;

/// Receives progress of a pipeline run. Called from the thread that runs the pipeline; implementations forward to a
/// channel or print.
pub trait Progress: Sync {
    /// `done` of `total` units of `step` are finished (`total` = 0 when unknown).
    fn step(&self, step: PipelineStep, done: u32, total: u32);
    /// A log line (`data.log.*`) or a warning worth showing while the run goes on.
    fn message(&self, m: Message);
}

/// Discards all progress.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoProgress;

impl Progress for NoProgress {
    fn step(&self, _step: PipelineStep, _done: u32, _total: u32) {}
    fn message(&self, _m: Message) {}
}

/// `Err(DatagenError::Cancelled)` once `cancel` is set: call it between requests and between stages (pipeline.md §3.1).
pub fn check_cancel(cancel: &AtomicBool) -> Result<(), DatagenError> {
    if cancel.load(Ordering::Relaxed) {
        Err(DatagenError::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use wfd_model::message::DataKey;

    #[test]
    fn cancel_flag() {
        let cancel = AtomicBool::new(false);
        assert!(check_cancel(&cancel).is_ok());
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            check_cancel(&cancel),
            Err(DatagenError::Cancelled)
        ));
    }

    /// A progress sink that records calls is a valid `&dyn Progress` shared across threads (`Sync`).
    #[test]
    fn progress_is_object_safe_and_sync() {
        #[derive(Default)]
        struct Rec(Mutex<Vec<String>>);
        impl Progress for Rec {
            fn step(&self, step: PipelineStep, done: u32, total: u32) {
                self.0
                    .lock()
                    .unwrap()
                    .push(format!("{step:?} {done}/{total}"));
            }
            fn message(&self, m: Message) {
                self.0.lock().unwrap().push(m.key.as_str().to_owned());
            }
        }
        let rec = Rec::default();
        let p: &dyn Progress = &rec;
        std::thread::scope(|s| {
            s.spawn(|| p.step(PipelineStep::Fetch, 1, 60));
        });
        p.message(DataKey::LogFetched.message());
        NoProgress.step(PipelineStep::Emit, 0, 0);
        assert_eq!(*rec.0.lock().unwrap(), ["Fetch 1/60", "data.log.fetched"]);
    }
}
