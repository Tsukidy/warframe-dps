//! Traces (engine.md §11.1): with `Want.trace` every stage writes a [`TraceEntry`] into a [`TraceSink`]; the default
//! sink [`NoTrace`] is zero-sized and does nothing, so tracing costs nothing when off. Register rows a scenario
//! exercises are recorded whether or not their compat flag is set (`touched_rows`, engine.md §10). Owner: T17 (the
//! recording sink and the conversion to `wfd_model::api::TraceEntry`).

use smallvec::SmallVec;

use crate::probe::ProbePath;
use crate::stats::EffectRef;

/// One stage trace (engine.md §11.1): the probe it computes, the equation, its inputs and the effect contributions.
/// The `value` field is an addition to engine.md §11.1 (the DTO `wfd_model::api::TraceEntry` carries it).
#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntry {
    /// The probed symbol.
    pub probe: ProbePath,
    /// Equation id (`"E-5"`).
    pub equation: &'static str,
    /// The computed value.
    pub value: f64,
    /// Input probes and their values.
    pub inputs: SmallVec<[(ProbePath, f64); 4]>,
    /// Contributing effects and their contributions (model units).
    pub contributions: SmallVec<[(EffectRef, f64); 8]>,
    /// Register rows (`"D-023"`) the stage exercised.
    pub rows: SmallVec<[&'static str; 2]>,
}

/// Where stages write traces. `Sync`, because one [`crate::StageCtx`] is shared by the parallel MC runs; a recording
/// sink uses interior mutability.
pub trait TraceSink: Sync {
    /// Whether entries are wanted; stages skip building them otherwise.
    fn enabled(&self) -> bool;
    /// Records one stage trace.
    fn record(&self, entry: TraceEntry);
    /// Records that a register row was exercised (engine.md §10 `touched_rows`, 08 §11.4).
    fn touch_row(&self, row: &'static str);
}

/// The no-op sink (the default).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoTrace;

/// The shared no-op sink of [`crate::StageCtx::new`].
pub static NO_TRACE: NoTrace = NoTrace;

impl TraceSink for NoTrace {
    fn enabled(&self) -> bool {
        false
    }
    fn record(&self, _entry: TraceEntry) {}
    fn touch_row(&self, _row: &'static str) {}
}
