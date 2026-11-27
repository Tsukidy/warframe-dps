//! Stage registry of the spec-vector harness (engine.md §14): stage vectors call a stage function by name with the
//! vector's JSON input and compare its JSON output (`tests/spec_vectors.rs`). Owner: T09 (it fills the table with
//! adapters around the frozen stage functions: `crit_multiplier`, `crit_tiers`, `quantize`, `philox`, `rng_mech`,
//! `capped_poisson`, `oracle_compare` …).
//!
//! Registration rule (contract review ENG-17, the C-21 rule): a stage that has no frozen function when T09 runs
//! (engine.md §14 names `faction`, `melee_timing`, `combo_gain`, `heavy_hits`, `zaw_assemble`, `heat_strip`,
//! `parity_buckets`) is registered by the task that writes its function (T08b, T13, T14, T15b, T16), which adds the
//! adapter and its entry here through a CONTRACTS.md entry; until then the vectors naming it stay
//! `pending = "Txx"`.

use std::fmt;

/// A stage adapter: JSON input of one vector case to its JSON output.
pub type StageFn = fn(&serde_json::Value) -> Result<serde_json::Value, StageError>;

/// Every stage the vectors may name: `&'static [(&'static str, fn(&serde_json::Value) -> Result<serde_json::Value,
/// StageError>)]` (work-breakdown.md T02a; [`StageFn`] is that function type). Empty until T09 fills it.
pub fn stage_registry() -> &'static [(&'static str, StageFn)] {
    &[]
}

/// The adapter registered under `name`.
pub fn lookup(name: &str) -> Option<StageFn> {
    stage_registry()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
}

/// Why a stage vector could not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageError {
    /// No stage of that name.
    UnknownStage(Box<str>),
    /// The input lacks a field or has the wrong type.
    BadInput(Box<str>),
    /// The stage rejected the input (out of its domain).
    Rejected(Box<str>),
}

impl fmt::Display for StageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StageError::UnknownStage(s) => write!(f, "unknown stage {s:?}"),
            StageError::BadInput(s) => write!(f, "bad stage input: {s}"),
            StageError::Rejected(s) => write!(f, "stage rejected the input: {s}"),
        }
    }
}

impl std::error::Error for StageError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_are_unique() {
        let names: std::collections::BTreeSet<_> =
            stage_registry().iter().map(|(n, _)| *n).collect();
        assert_eq!(names.len(), stage_registry().len());
        assert!(lookup("no_such_stage").is_none());
    }
}
