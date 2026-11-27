//! Stance normalisation (07 E-7.0; pipeline.md §5.3). Owner: T18.

use crate::Report;
use wfd_model::ComboTable;

/// 07 E-7.0 on every input of every combo of the evaluated `Module:Stances/data` (`addSharedCombos` already run in
/// Lua): defaults, padding, proc holes, extra procs, strike expansion by `Hits[e]`, `Types[e]` → `StrikeType`; one
/// report line per correction; Conclave-only tables dropped. Returns `Bundle.combo_tables` (stance, stanceless class
/// and UniqueToWeapon tables, `from_weapon` kept), indexed by `ComboTableId`. TV-07-12 (Blind Justice) is its spec
/// vector (`tests/spec_vectors_07.rs`).
pub fn normalise_stances(raw: &serde_json::Value, report: &mut Report) -> Vec<ComboTable> {
    let _ = (raw, report);
    todo!("T18: 07 E-7.0 stance normalisation")
}
