//! Synthesized heavy attacks (07 §9.1, E-7.40, E-7.41; pipeline.md §5.3). Owner: T18.

use crate::JoinedWeapon;
use wfd_model::{ComboTable, HeavyDef, MeleeRules};

/// The `HeavyDef` of a melee weapon with a heavy attack: the hit lists H_j of E-7.40 (the weapon's unique override,
/// else the class row of table 9-A with σ, else the 9-A breakdown) and κ_w of E-7.41 (from Public Export data, checked
/// against table 9-B). The synthesized `Attack` has roles `HEAVY` and `synthesized = HeavyFrom { normal }`; the engine
/// resolves its damage per scenario. `None` for classes without heavies. TV-07-10 (Machete) is its spec vector
/// (`tests/spec_vectors_07.rs`).
pub fn heavy_def(w: &JoinedWeapon, tables: &[ComboTable], rules: &MeleeRules) -> Option<HeavyDef> {
    let _ = (w, tables, rules);
    todo!("T18: 07 E-7.40, E-7.41 heavy synthesis")
}
