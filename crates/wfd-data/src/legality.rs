//! Build legality: spec 09, implemented once (AD-24, pipeline.md §8.3). The UI (`/api/candidates`,
//! `CalcReport.violations`), the build decoder and the optimizer's pools (optimizer.md §2 step 2) all call this module.
//!
//! Every rule of 09 is an equation `E-9.k` with a rule key (the message key, `wfd_model::message::LegalityKey`) and a
//! severity (`illegal`, `illegal_unverified`, `warning`; 09 §1.5); a broken rule is one [`Violation`] carrying both
//! and the slots involved. **Validation never blocks evaluation** (09 §1.5 rule 1): these functions only report; the
//! engine evaluates any build, and the optimizer treats the `illegal` and `illegal_unverified` rules of the enabled
//! groups as constraints (09 §1.5 rule 2).
//!
//! Rules (work-breakdown.md T04): slot counts and kinds (E-9.1-E-9.3a), compatibility (§3), exclusion and duplicates
//! (§4), ranks and drain (§5), capacity, polarity and minimum Forma under the capacity modes (§6, E-9.14-E-9.20),
//! exilus (§7), arcane families (§8), shards (§9), Helminth with the frame's host rules (§10, E-9.32), set counting
//! (§11, E-9.40-E-9.41), rivens (§12), Incarnon (§13), valence (§14, E-9.70).
//!
//! Skeleton of T02b; every body is implemented by T04.

use crate::GameData;
use smallvec::SmallVec;
use wfd_model::api::{CapacityReport, ValueRange};
use wfd_model::{
    Build, CapacityMode, FormaTypes, Pick, Riven, Scenario, SetId, SlotId, Violation, WeaponId,
};

/// The capacity and Forma report of 09 E-9.14-E-9.19: U, C_cap, the minimum Forma N_forma* per Forma type (`null`
/// when no layout fits), the capacity mode, the overlevel bound ceil((rank − 30)/2) and the warframe's capacity when
/// `frame_complete` (CONTRACTS.md T01b review fixes, request C-25: `min_forma` produces this report). It is the
/// `CalcReport.capacity` / `OptResult.capacity` DTO itself, so the server copies it unchanged.
pub type FormaReport = CapacityReport;

/// Every spec 09 rule over a resolved build and scenario: slots and kinds, compatibility, exclusion and duplicates,
/// ranks, capacity under the scenario's T-35 mode and Forma types (`toggles.capacity`, `toggles.forma_types`; the
/// warframe only when `frame_complete`, E-9.18), exilus, arcanes, shards, Helminth (the caster of each buff, E-9.32),
/// set counting with the companion kind (E-9.41), rivens, Incarnon and valence. Violations in canonical order (slot
/// order, then rule key) so results are deterministic (AD-19). Implemented by T04.
pub fn check(gd: &GameData, b: &Build, s: &Scenario) -> Vec<Violation> {
    let _ = (gd, b, s);
    todo!("T04: spec 09 rules (§2-§14)")
}

/// The rules that putting `pick` into `slot` of `b` would break, the rest of the build as it is (the content `b`
/// currently holds in `slot` is replaced): slot existence and kind (E-9.2, E-9.3), compatibility (§3), exclusion and
/// duplicates against the other slots (§4), rank range (§5.1), arcane family (§8), shard and Helminth rules (§9,
/// §10), riven binding and traits (§12), Incarnon tier (§13), valence (§14). Capacity is not checked here (it depends
/// on the whole layout: [`min_forma`]). Used by `/api/candidates` (blocked reason) and by the optimizer's pools.
/// Implemented by T04.
pub fn check_slot(gd: &GameData, b: &Build, slot: SlotId, pick: &Pick) -> Vec<Violation> {
    let _ = (gd, b, slot, pick);
    todo!("T04: per-slot legality (spec 09)")
}

/// Capacity and minimum Forma of the weapon (and of a complete warframe) under capacity mode `mode` with every Forma
/// type available (09 E-9.14-E-9.19; exact DP of E-9.19 over the innate polarity multiset, E-9.20). Equals
/// [`min_forma_with`] with `FormaTypes::all()`, the T-35 default.
pub fn min_forma(gd: &GameData, b: &Build, mode: CapacityMode) -> FormaReport {
    min_forma_with(gd, b, mode, FormaTypes::all())
}

/// [`min_forma`] restricted to the Forma types `forma` (T-35 allowed Forma types, 00 §6.1; 09 §6.1 table): a slot
/// may be re-polarized only with an allowed type, and Umbra Forma also only on the items 09 §6.1 allows (warframes,
/// melee weapons). Callers take `forma` from `Toggles.forma_types`, or from `CapacityRule::effective` in the
/// optimizer (server-api.md §8.1). Addition of T02b (CONTRACTS.md): the T-35 Forma types change N_forma*, and the
/// signature of work-breakdown.md T02b has no place for them. Implemented by T04.
pub fn min_forma_with(
    gd: &GameData,
    b: &Build,
    mode: CapacityMode,
    forma: FormaTypes,
) -> FormaReport {
    let _ = (gd, b, mode, forma);
    todo!("T04: capacity and minimum Forma (09 E-9.14-E-9.20)")
}

/// Pieces of each set that count (09 E-9.40: the active weapon's set mods, the warframe's, and the declared companion
/// pieces of `Build.companion_set_mods` when `Build.companion_kind` allows them, E-9.41), sorted by set id; sets with
/// no counting piece are absent. Read by the set-bonus effects of 04 §9. Implemented by T04.
pub fn set_counts(gd: &GameData, b: &Build) -> SmallVec<[(SetId, u8); 4]> {
    let _ = (gd, b);
    todo!("T04: set counting (09 E-9.40, E-9.41)")
}

/// The legal value range of each trait of `riven` on `weapon` (09 §12: E-9.53a order of checks, roles and layout
/// E-9.54, value range E-9.56 from 04 E-4.18 in binary32), one entry per trait in the riven's order. Used by the riven
/// editor (`RivenTraitSchema.range`) and the `riven.range` rule. Implemented by T04.
pub fn riven_range(gd: &GameData, weapon: WeaponId, riven: &Riven) -> Vec<TraitRange> {
    let _ = (gd, weapon, riven);
    todo!("T04: riven value ranges (09 E-9.54-E-9.56)")
}

/// The legal range of one riven trait (09 E-9.56) and the role it plays in the layout (E-9.54).
#[derive(Debug, Clone, PartialEq)]
pub struct TraitRange {
    /// Trait id (`Rules.riven_traits`), as in the riven.
    pub trait_id: Box<str>,
    /// sgn_i = sign(v_i / b_i) (E-9.54); `None` when the trait failed T1 or T2 of E-9.55 (a repeated trait, or no
    /// coefficient for the riven's column), which takes no further part (E-9.53a step 1).
    pub role: Option<TraitRole>,
    /// [v_lo, v_hi] in model units at the riven's rank for its layout (E-9.56, without the display tolerance);
    /// `None` exactly when `role` is.
    pub range: Option<ValueRange>,
}

/// Role of a riven trait in its layout (09 E-9.54).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraitRole {
    /// sgn = +1: a bonus (a beneficial value of a trait with b < 0 included).
    Positive,
    /// sgn = −1: a curse.
    Negative,
}
