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
//! Besides the checks, the parts of 09 that other code computes with are exposed here, so neither the optimizer nor
//! a test reimplements them (AD-24; contract review API-12, API-13, API-16): the capacity inputs of one pick
//! ([`slot_cost`], E-9.12, E-9.59), the capacity a slot charges ([`slot_charge`], E-9.14, E-9.15), the minimum-Forma
//! DP over a fixed context ([`CapacityModel`], E-9.16-E-9.19), the riven class and the traits it can roll
//! ([`riven_class`], [`riven_traits_for`], E-9.50, E-9.55, §12.3), valence fusion ([`fuse_valence`], E-9.71), the
//! shard count N_cav ([`shard_cav`], E-9.31) and set counting ([`set_counts`], E-9.40, through
//! `wfd_model::sets`). Per-rank values (E-9.13) are `wfd_model::RankCurve::value`.
//!
//! Skeleton of T02b, extended by the contract review; every body is implemented by T04, except the delegations
//! ([`min_forma`], [`set_counts`]).

use crate::GameData;
use smallvec::SmallVec;
use wfd_model::api::{CapacityReport, ValueRange};
use wfd_model::{
    Build, CapacityMode, FormaTypes, Pick, Polarity, Riven, RivenClass, RivenTraitDef, Scenario,
    SetId, SlotId, Violation, WeaponId,
};

/// Role of a riven trait in its layout (09 E-9.54): the API type itself, so `CalcReport.riven` (server-api.md §6.1)
/// carries [`TraitRange`]s unchanged (contract review API-14).
pub use wfd_model::api::TraitRole;

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
/// duplicates against the other slots (§4), rank range (§5.1), arcane family and gating (§8, E-9.30, E-9.30a), shard
/// rules (§9, E-9.31), riven binding and traits (§12), Incarnon tier (§13). Capacity is not checked here (it depends
/// on the whole layout: [`min_forma`]). Used by `/api/candidates` (blocked reason) and by the optimizer's pools.
///
/// The slots each [`Pick`] goes into (09 §2.1):
///
/// | `Pick` | `slot` |
/// |---|---|
/// | `Mod` | `Gen(0..=7)`, `Exilus` |
/// | `Riven` | `Gen(0..=7)` (a riven is not a slot, 09 §2.1) |
/// | `Arcane` | `Arcane(0..=1)` (weapon), `FArcane(0..=1)` (warframe) |
/// | `Perk` | `Inc(t)` with `t` = the perk's `tier` |
/// | `Stance` | `Stance` |
/// | `FrameMod` | `FGen(0..=7)`, `FExilus` |
/// | `Aura` | `Aura(0..=1)` |
/// | `Shard` | `Shard(0..=4)` |
///
/// Not covered (CONTRACTS.md T02b): `SlotId::Subsumed` has no `Pick` (engine.md §1 defines none), so the Helminth
/// rules (§10, E-9.32) run in [`check`] only; `KitgunParts`, `ZawParts` and `Progenitor` fill `Build.variant`, which no
/// `SlotId` names, so their rules (T-25 parts, valence §14, E-9.70) also run in [`check`] on a build that holds them.
/// A pairing outside the table, these included, is reported as `slot.kind` (E-9.3), so a caller error never reads
/// as legal. Implemented by T04.
pub fn check_slot(gd: &GameData, b: &Build, slot: SlotId, pick: &Pick) -> Vec<Violation> {
    let _ = (gd, b, slot, pick);
    todo!("T04: per-slot legality (spec 09)")
}

/// Capacity and minimum Forma of the weapon (and of a complete warframe) under capacity mode `mode` with every Forma
/// type available (09 E-9.14-E-9.19; exact DP of E-9.19 over the innate polarity multiset, E-9.20). Equals
/// [`min_forma_with`] with `FormaTypes::all()`, the T-35 default.
///
/// Mode `current` reads the build's polarity layout (`WeaponItem.polarities`, `FrameLoadout.polarities`): an absent
/// slot keeps its innate polarity, a `null` entry was Forma'd to no polarity (09 §6.1; contract review M-8). The
/// report names every riven whose drain or polarity is an E-9.59 default (`CapacityReport.riven_defaults`, TV-09-18;
/// contract review API-15).
pub fn min_forma(gd: &GameData, b: &Build, mode: CapacityMode) -> FormaReport {
    min_forma_with(gd, b, mode, FormaTypes::all())
}

/// [`min_forma`] restricted to the Forma types `forma` (T-35 allowed Forma types, 00 §6.1; 09 §6.1 table): a slot
/// may be re-polarized only with an allowed type, and Umbra Forma also only on the items 09 §6.1 allows (warframes,
/// melee weapons). Callers take `forma` from `Toggles.forma_types`, or from `CapacityRule::effective` in the
/// optimizer (server-api.md §8.1). Addition of T02b (CONTRACTS.md): the T-35 Forma types change N_forma*, and the
/// signature of work-breakdown.md T02b has no place for them. It is [`CapacityModel`] over the build's own general
/// slots: `CapacityModel::new(gd, b, mode, forma).report(<the slot_cost of each filled gen slot>)`, so the optimizer's
/// leaf check and the calculator's report are one computation (AD-24). Implemented by T04.
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
/// pieces of `Build.companion_set_mods`), sorted by set id; sets with no counting piece are absent. The rule is
/// implemented once in `wfd_model::sets::set_counts`, which the engine's set-bonus compilation (04 §9) also calls, so
/// TV-09-09 checks the count that drives damage (contract review API-6). Declared companion pieces count whatever
/// `Build.companion_kind` says; their legality (E-9.41, `set.companion`) is a violation of [`check`].
pub fn set_counts(gd: &GameData, b: &Build) -> SmallVec<[(SetId, u8); 4]> {
    wfd_model::sets::set_counts(gd.bundle(), b)
}

/// The legal value range of each trait of `riven` on `weapon` (09 §12: E-9.53a order of checks, roles and layout
/// E-9.54, value range E-9.56 from 04 E-4.18 in binary32), one entry per trait in the riven's order. Used by the
/// `riven.range` rule and by the riven editor's range hint, which the server fills into `CalcReport.riven` for every
/// riven of the build (`RivenTraitRange` = a [`TraitRange`] plus the riven's slot; contract review API-14).
/// Implemented by T04.
pub fn riven_range(gd: &GameData, weapon: WeaponId, riven: &Riven) -> Vec<TraitRange> {
    let _ = (gd, weapon, riven);
    todo!("T04: riven value ranges (09 E-9.54-E-9.56)")
}

/// The legal range of one riven trait (09 E-9.56) and the role it plays in the layout (E-9.54).
///
/// The fields follow the steps of 09 E-9.53a, each of which uses only the traits that passed the earlier ones:
/// `range` present implies `role` present, not the reverse.
///
/// | Case | `role` | `range` |
/// |---|---|---|
/// | the trait failed T1 or T2 of E-9.55 (step 1: a repeated trait, or no coefficient b for the riven's column) | `None` | `None` |
/// | v_i = 0: sgn_i = sign(0) = 0, so the trait counts in neither n_pos nor n_neg (E-9.54) | `None` | `None` |
/// | the layout is illegal (E-9.54: n_pos ∉ {2, 3} or n_neg ∉ {0, 1}): 04 E-4.18 has no layout factor L for it | `Some` | `None` |
/// | the trait failed T3 to T6 of E-9.55 (step 3, e.g. a negative Heat) | `Some` | `None` |
/// | otherwise (step 4) | `Some` | `Some` |
///
/// Spec 09 does not say which rule a zero-valued trait breaks (it has no L in E-4.18, so E-9.56 cannot place it);
/// T04 reports it as the manager rules (CONTRACTS.md T02b).
#[derive(Debug, Clone, PartialEq)]
pub struct TraitRange {
    /// Trait id (`Rules.riven_traits`), as in the riven.
    pub trait_id: Box<str>,
    /// sgn_i = sign(v_i / b_i) (E-9.54) when it is ±1; `None` when the trait failed T1 or T2 or has v_i = 0 (table
    /// above).
    pub role: Option<TraitRole>,
    /// [v_lo, v_hi] in model units at the riven's rank for its layout (E-9.56, without the display tolerance); only
    /// for a trait with a role, in a legal layout, that passed T3 to T6 (table above).
    pub range: Option<ValueRange>,
}

/// The riven class of a weapon, `riv_class(W)` of 09 E-9.50: `None` for exalted weapons, archmelee, beast claws and
/// Garuda Talons (no rivens); `Kitgun` / `Zaw` for modular weapons, `Archgun`, `Companion` (robotic weapons), else
/// by slot and class (`Shotgun`, `Rifle`, `Pistol`, `Melee`). A generated riven carries it as `Riven.class`
/// (optimizer.md §6.2 step 6). Addition of the contract review (API-12). Implemented by T04.
pub fn riven_class(gd: &GameData, weapon: WeaponId) -> Option<RivenClass> {
    let _ = (gd, weapon);
    todo!("T04: riven class (09 E-9.50)")
}

/// The traits a riven on `weapon` can roll, in `Rules.riven_traits` order: those with a coefficient for the class's
/// column (T2 of 09 E-9.55; Kitgun → Pistol, Zaw → Melee, Companion → the weapon class's column), spliced traits only
/// where §12.3 makes them available for the class (T5), and on companion rivens without the vintage traits
/// (`RivenTraitDef.vintage_companion`: legal when present, but generation MUST NOT produce them, §12.3). Empty when
/// [`riven_class`] is `None`. The optimizer's riven mode (optimizer.md §6.2 step 6) enumerates these; signs (T3, T4)
/// and the layout (E-9.54) are the generator's choices, checked by [`riven_range`]. Addition of the contract review
/// (API-12). Implemented by T04.
pub fn riven_traits_for(gd: &GameData, weapon: WeaponId) -> Vec<&RivenTraitDef> {
    let _ = (gd, weapon);
    todo!("T04: riven traits per class (09 E-9.55 T2, T5; §12.3)")
}

/// Polarity of a pick for the capacity rules (09 E-9.14, E-9.19): a polarity, no polarity, or the `unknown` polarity
/// of a riven whose polarity is not given (E-9.59), which matches no slot (not even ANY) and cannot be matched by a
/// Forma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickPolarity {
    /// A polarity (09 §6.1: M, N, V, Z, U, P, UMB, ANY).
    Polarized(Polarity),
    /// No polarity (wiki `Universal`, `None`; PE `AP_UNIVERSAL`).
    Unpolarized,
    /// A riven without a given polarity (09 E-9.59).
    Unknown,
}

/// The capacity inputs of one pick (09 E-9.12, E-9.59, E-9.3a): what E-9.14, E-9.15 and E-9.19 read. Addition of the
/// contract review (API-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotCost {
    /// ξ(r): the drain at the pick's rank before polarity (E-9.12; a riven's drain by E-9.59; the X-STANCE rank of an
    /// exalted stance). ≥ 0 for general and exilus mods; < 0 for stance and aura mods, whose b = −ξ(r) adds capacity.
    pub drain: i16,
    /// The pick's polarity.
    pub polarity: PickPolarity,
    /// ξ is the E-9.59 default ξ(8) = 18 of a riven whose drain is not given (`CapacityReport.riven_defaults`).
    pub default_drain: bool,
}

impl SlotCost {
    /// A lower bound of what this pick charges in any general or exilus slot, under any slot polarity and any Forma
    /// option of E-9.19 (E-9.14: a matching slot halves the drain, rounded up; an `unknown` polarity matches
    /// nothing). The optimizer's per-candidate drain bound (`MainCandidate.drain` and the suffix minimum-drain table,
    /// optimizer.md §5.2). Implemented by T04.
    pub fn min_charge(&self) -> i16 {
        todo!("T04: lower bound of 09 E-9.14 over slot polarities and Forma")
    }
}

/// The capacity inputs of putting `pick` into `slot` of `b` (09 E-9.12 at the pick's rank, E-9.59 for a riven, the
/// X-STANCE fixed rank for an exalted stance): `None` when the slot charges no capacity (arcane, Incarnon, shard and
/// subsumed slots) or the pick does not go into the slot (the table of [`check_slot`]). Nothing else is checked
/// ([`check_slot`] does that). Used by the optimizer's capacity pruning and by [`CapacityModel`]; TV-09-01 and
/// TV-09-06 test ξ through it. Addition of the contract review (API-13, API-16). Implemented by T04.
pub fn slot_cost(gd: &GameData, b: &Build, slot: SlotId, pick: &Pick) -> Option<SlotCost> {
    let _ = (gd, b, slot, pick);
    todo!("T04: drain at rank (09 E-9.12, E-9.59)")
}

/// The capacity one slot of polarity `slot_polarity` (`None` = unpolarized) charges for a pick with `cost`: c_ω of 09
/// E-9.14 (≥ 0) for a general or exilus mod (`cost.drain` ≥ 0), or −β_ω of E-9.15 (≤ 0) for a stance or aura mod
/// (`cost.drain` < 0), each slot rounded on its own, so U of E-9.17 is the sum over the slots. The Decision rules of
/// 09 §6.2 and §6.3 apply (mismatch `floor(1.25·d + 0.5)` and `floor(0.8·b)`; UMB and ANY per OQ-903). TV-09-01 is
/// its vector. Addition of the contract review (API-13, API-16). Implemented by T04.
pub fn slot_charge(cost: SlotCost, slot_polarity: Option<Polarity>) -> i16 {
    let _ = (cost, slot_polarity);
    todo!("T04: capacity per slot (09 E-9.14, E-9.15)")
}

/// The minimum-Forma computation of 09 E-9.16-E-9.19 over a fixed context, for many fillings of the 8 general slots:
/// the optimizer's leaf check (optimizer.md §5.2; contract review API-13). [`CapacityModel::new`] takes everything
/// except the general slots from `context` (weapon and rank, Catalyst, capacity override, innate polarities and the
/// `WeaponItem.polarities` layout of mode `current`, the exilus and stance slots, the warframe part when
/// `frame_complete`); [`CapacityModel::report`] adds the general-slot picks. [`min_forma_with`] is this model over the
/// build's own general slots, so both give the same report for the same build.
#[derive(Debug, Clone)]
pub struct CapacityModel<'a> {
    // Private: T04 replaces them with the precomputed context (innate multiset, exilus and stance options, C_cap, the
    // warframe's report).
    gd: &'a GameData,
    context: &'a Build,
    mode: CapacityMode,
    forma: FormaTypes,
}

impl<'a> CapacityModel<'a> {
    /// The model of `context` under capacity mode `mode` and the allowed Forma types `forma` (T-35; the optimizer
    /// passes `constraints.capacity.mode` and `CapacityRule::effective`, server-api.md §8.1). The general slots of
    /// `context` are ignored. Implemented by T04.
    pub fn new(
        gd: &'a GameData,
        context: &'a Build,
        mode: CapacityMode,
        forma: FormaTypes,
    ) -> CapacityModel<'a> {
        let _ = (gd, context, mode, forma);
        todo!("T04: capacity context (09 E-9.15-E-9.19)")
    }

    /// The report of the context with `mains` in the general slots: each entry is a general slot and the
    /// [`slot_cost`] of its pick, at most 8 entries. Positions are free (E-9.20), so the slot ids matter only for
    /// `riven_defaults`. Equal to [`min_forma_with`] of the completed build. Implemented by T04.
    pub fn report(&self, mains: &[(SlotId, SlotCost)]) -> FormaReport {
        let _ = (self.gd, self.context, self.mode, self.forma, mains);
        todo!("T04: minimum Forma (09 E-9.19)")
    }
}

/// Valence fusion of 09 E-9.71 (the UI's helper, integer tenths): `fuse(a10, b10) = 600 if f ≥ 580 else f` with
/// `f = min(600, floor(11·max(a10, b10)/10))`. TV-09-12's fusion rows are its vector. Addition of the contract review
/// (API-16). Implemented by T04.
pub fn fuse_valence(a10: u16, b10: u16) -> u16 {
    let _ = (a10, b10);
    todo!("T04: valence fusion (09 E-9.71)")
}

/// N_cav of 09 E-9.31: the warframe's Crimson, Azure and Violet shards, whatever bonus each carries (the input of 04
/// E-4.12; TV-09-13). Addition of the contract review (API-16). Implemented by T04.
pub fn shard_cav(gd: &GameData, b: &Build) -> u8 {
    let _ = (gd, b);
    todo!("T04: shard count N_cav (09 E-9.31)")
}
