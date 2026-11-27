//! Stance normalisation (07 E-7.0; pipeline.md §5.3). Owner: T18.

use crate::Report;
use std::collections::BTreeMap;
use wfd_model::{ComboTable, StanceId};

/// 07 E-7.0 on every input of every combo of the evaluated `Module:Stances/data` (`addSharedCombos` already run in
/// Lua): defaults, padding, proc holes, extra procs, strike expansion by `Hits[e]`, `Types[e]` → `StrikeType`; one
/// report line per correction; Conclave-only tables dropped. Returns `Bundle.combo_tables` (stance, stanceless class
/// and UniqueToWeapon tables, `from_weapon` kept), indexed by `ComboTableId`. TV-07-12 (Blind Justice) is its spec
/// vector (`tests/spec_vectors_07.rs`).
///
/// `stance_ids` maps the module's stance names (its table keys) to the dense `StanceId`s of `Bundle.stances`, which
/// the join assigns before this step, so a stance table's `ComboSource::Stance` names its stance; a stance the map
/// lacks is reported and its tables dropped. Parameter added by the contract review (API-22; T18 is the only caller).
pub fn normalise_stances(
    raw: &serde_json::Value,
    stance_ids: &BTreeMap<Box<str>, StanceId>,
    report: &mut Report,
) -> Vec<ComboTable> {
    let _ = (raw, stance_ids, report);
    todo!("T18: 07 E-7.0 stance normalisation")
}
