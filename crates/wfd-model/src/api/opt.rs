//! Optimizer routes and events (server-api.md §8; optimizer.md §1, §9): `POST /api/optimize`, the SSE stream of
//! [`OptEvent`]s, `GET /api/optimize/{job}` ([`JobStatus`]).
//!
//! Spec 08 §10 adds the metric `ttk_noarmor` to the metrics of server-api.md §8.1 ([`Metric::TtkNoarmor`];
//! CONTRACTS.md, T01b).
//!
//! Scores are in the metric's unit, and the TTK metrics' unit is T_EV seconds, which is +∞ for a build that never
//! kills (08 E-8.21, E-8.26 band 2; engine.md §7.7). Every score-valued field therefore uses
//! `serde_util::ext_f64` and is written `"Infinity"` in that case (serde_json would write `null` and fail to read it
//! back).

use super::calc::{Summary, TtkEv};
use crate::build::{SlotContent, SlotId};
use crate::enums::{DamageType, View};
use crate::ids::{AttackKey, StableKey};
use crate::message::Message;
use crate::scenario::{BuffPick, CalcInput, CapacityMode, FormaTypes, Heavy, Toggles};
use crate::serde_util::flags_serde;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------------------------
// POST /api/optimize
// ---------------------------------------------------------------------------------------------------------------

/// `POST /api/optimize` body (server-api.md §8.1; defaults optimizer.md §11).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OptRequest {
    /// Current build (start point, locks, riven, variant) and scenario.
    pub input: CalcInput,
    /// What to maximize.
    pub objective: Objective,
    /// Constraints on the builds.
    pub constraints: Constraints,
    /// Which slot groups the search may change (the others stay as in the input).
    pub groups: GroupSwitches,
    /// External buffs.
    pub buffs: BuffPolicy,
    /// Search parameters.
    pub search: SearchParams,
}

/// The objective (08 §10).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Objective {
    /// Metric (ordering by the engine's `Score` keys, E-8.26).
    pub metric: Metric,
    /// View; `auto` = armor when the enemy has armor, else health.
    pub view: ViewSel,
    /// Weighted attacks (weights ≥ 0); empty = the first attack, weight 1.
    pub attacks: Vec<WeightedAttack>,
    /// T-09 for the search (start combo); ignored for explicit heavy attacks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub heavy: Option<Heavy>,
    /// Search and ranking window (s); default T-01 of the scenario.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub window_s: Option<f64>,
}

/// Optimizer metrics (08 §10).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Metric {
    /// E-8.12 (default).
    #[default]
    DpsWindow,
    /// E-8.14.
    DpsBurst,
    /// E-8.14.
    DpsSustained,
    /// E-8.21 on the layered view, ordering E-8.26.
    Ttk,
    /// E-8.21 with armor ignored (the EV counterpart of pass L0, D-802); spec 08 §10.
    TtkNoarmor,
    /// Expected damage of one parent hit at the steady state.
    HitInfight,
    /// Expected damage of the first shot on the fresh target.
    HitFresh,
}

/// A view or `auto`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ViewSel {
    /// Armor when the enemy has armor, else health (08 §10).
    #[default]
    Auto,
    /// [`View::Health`].
    Health,
    /// [`View::Armor`].
    Armor,
    /// [`View::Shield`].
    Shield,
    /// [`View::Overguard`].
    Overguard,
    /// [`View::Layered`] (the TTK metrics' view, 08 §10).
    Layered,
}

impl ViewSel {
    /// The view, or `None` for `auto`.
    pub fn view(self) -> Option<View> {
        match self {
            ViewSel::Auto => None,
            ViewSel::Health => Some(View::Health),
            ViewSel::Armor => Some(View::Armor),
            ViewSel::Shield => Some(View::Shield),
            ViewSel::Overguard => Some(View::Overguard),
            ViewSel::Layered => Some(View::Layered),
        }
    }
}

/// An attack of a weighted objective.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeightedAttack {
    /// The attack.
    pub attack: AttackKey,
    /// Weight w_u ≥ 0.
    pub weight: f64,
}

/// Constraints (optimizer.md §1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Constraints {
    /// Locked slots (the inspiration's "keep installed" = every occupied slot).
    pub locked: Vec<SlotId>,
    /// Items every result must contain.
    pub must_include: Vec<StableKey>,
    /// The "prohibited mods" list.
    pub prohibited: Vec<StableKey>,
    /// Final damage types required or forbidden.
    pub elements: ElementRule,
    /// Rule switches.
    pub rules: RuleFlags,
    /// Capacity rule (09 E-9.18).
    pub capacity: CapacityRule,
    /// Riven policy.
    pub riven: RivenPolicy,
}

/// Final damage types a build must or must not have.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ElementRule {
    /// Required types.
    pub require: Vec<DamageType>,
    /// Forbidden types.
    pub forbid: Vec<DamageType>,
}

bitflags::bitflags! {
    /// Rule switches of the optimizer (optimizer.md §2 step 4). JSON: a list of names (`["no_faction"]`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct RuleFlags: u16 {
        /// Keep T-03; off sets T-03 to `body`.
        const USE_WEAK_POINT = 1 << 0;
        /// Zero faction targets.
        const NO_FACTION = 1 << 1;
        /// Zero effects triggered by kills.
        const NO_ON_KILL = 1 << 2;
        /// Keep the progenitor element of the input.
        const KEEP_PROGENITOR = 1 << 3;
        /// Search Incarnon perks.
        const USE_INCARNON = 1 << 4;
        /// Allow `next_shot_after` effects for DPS and TTK metrics.
        const ALLOW_NEXT_SHOT = 1 << 5;
        /// Exclude items with draft effects.
        const EXCLUDE_DRAFT = 1 << 6;
        /// Exclude items with stale effects.
        const EXCLUDE_STALE = 1 << 7;
    }
}
flags_serde!(RuleFlags, u16, Lower);

/// Capacity rule (09 E-9.15-E-9.18). Precedence over the scenario's T-35 toggles (server-api.md §8.1, CONTRACTS.md
/// T01d): the optimizer's search and the capacity report of every ranked build use `mode`, not
/// `input.scenario.toggles.capacity` (which the calculator uses and the share code keeps); the allowed Forma types
/// are `input.scenario.toggles.forma_types` with Umbra Forma allowed exactly when `umbra_forma` is set
/// ([`CapacityRule::effective`]); the warframe part is `input.build.warframe.frame_complete`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CapacityRule {
    /// Capacity mode of the search (spec 00 §6.1 T-35); wins over `input.scenario.toggles.capacity`.
    pub mode: CapacityMode,
    /// Umbra Forma allowed (09 §6.1, E-9.19); wins over the `umbra` flag of `input.scenario.toggles.forma_types`.
    pub umbra_forma: bool,
    /// Mod ranks (spec 00 §6.1 T-36).
    pub ranks: RankPolicy,
}

impl CapacityRule {
    /// The T-35 inputs the optimizer applies: this rule's capacity mode, and the scenario's allowed Forma types with
    /// the Umbra flag replaced by `umbra_forma` (server-api.md §8.1).
    pub fn effective(&self, scenario: &Toggles) -> (CapacityMode, FormaTypes) {
        let mut types = scenario.forma_types;
        types.set(FormaTypes::UMBRA, self.umbra_forma);
        (self.mode, types)
    }
}

/// Mod ranks the optimizer may choose.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RankPolicy {
    /// Maximum ranks only (default).
    #[default]
    Max,
    /// Lower ranks are candidates too.
    AllowLower,
}

/// Riven policy. JSON `"use_given"`, `"none"` or `{ "search": { "traits": 3, … } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RivenPolicy {
    /// Keep the input's riven (default).
    #[default]
    UseGiven,
    /// No riven.
    None,
    /// Search riven trait sets (optimizer.md §6.2 step 6).
    Search {
        /// Number of traits, 2 ..= 4.
        traits: u8,
        /// Allow a negative trait.
        allow_negative: bool,
        /// Position in the value range (0 = minimum, 1 = maximum).
        quality: f64,
    },
}

/// Slot groups the search may change (optimizer.md §1, §6.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GroupSwitches {
    /// Exilus slot.
    pub exilus: bool,
    /// Weapon arcane slots.
    pub weapon_arcanes: bool,
    /// Stance and combo.
    pub stance: bool,
    /// Melee plan search (spec 00 §6.1 T-30).
    pub melee_plan: bool,
    /// Incarnon perks.
    pub incarnon: bool,
    /// General frame slots, 0 ..= 8 (09 E-9.2).
    pub frame_mods: u8,
    /// Frame exilus slot.
    pub frame_exilus: bool,
    /// Aura slot.
    pub aura: bool,
    /// Frame arcanes.
    pub frame_arcanes: bool,
    /// Archon shards.
    pub shards: ShardPolicy,
    /// Kitgun and zaw parts.
    pub modular_parts: bool,
    /// Progenitor element.
    pub progenitor: bool,
}

/// Archon shards the search may use. JSON `"off"`, `"any"` or `{ "owned": [ … ] }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ShardPolicy {
    /// Shards are not searched.
    #[default]
    Off,
    /// Any shard.
    Any,
    /// The user's shards.
    Owned(Vec<OwnedShard>),
}

/// Shards the user owns.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OwnedShard {
    /// Shard item (colour + bonus).
    pub item: StableKey,
    /// Tauforged.
    pub tauforged: bool,
    /// How many.
    pub count: u8,
}

/// External buffs of the search (optimizer.md §6.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuffPolicy {
    /// Allowed buffs with their values.
    pub allowed: Vec<BuffPick>,
    /// Solve "none", each buff alone and all buffs as separate scenarios (`OptResult.buff_table`).
    pub portfolio: bool,
    /// Ability Strength; default the scenario's T-20.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub strength: Option<f64>,
}

/// Search parameters (optimizer.md §1, §11).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SearchParams {
    /// Search mode.
    pub mode: SearchMode,
    /// Score leaves with `WindowExact`; the proof then carries no ε assumption (optimizer.md §5.2).
    pub certify: bool,
    /// Wall-clock limit (s). Absent in the JSON = the optimizer.md §11 default, 120 s
    /// ([`SearchParams::DEFAULT_TIME_LIMIT_S`]); `null` = no wall-clock limit (`None`). Always written.
    #[serde(default = "SearchParams::default_time_limit_s")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub time_limit_s: Option<f64>,
    /// Deterministic node budget (optimizer.md §8); absent = none.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::int53::option"
    )]
    #[cfg_attr(feature = "ts", ts(optional, type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "Option<u64>"))]
    pub node_budget: Option<u64>,
    /// Finalists kept (default 32).
    pub keep: u16,
    /// MC confirmation of the finalists. Absent in the JSON = the optimizer.md §11 default
    /// `{ finalists: 12, runs: 1000, seed: 0 }` ([`McConfirm::DEFAULT`]); `null` = no MC confirmation (`None`).
    /// Always written.
    #[serde(default = "SearchParams::default_mc_confirm")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub mc_confirm: Option<McConfirm>,
    /// Worker threads; absent = the optimizer pool size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub threads: Option<u16>,
    /// Context beam width B (default 4, 1-8).
    pub context_beam: u8,
    /// Preset (optimizer.md §10.2).
    pub preset: Preset,
}

impl SearchParams {
    /// Default wall-clock limit (s), optimizer.md §11.
    pub const DEFAULT_TIME_LIMIT_S: f64 = 120.0;

    /// Serde default of `time_limit_s` (the field is absent).
    fn default_time_limit_s() -> Option<f64> {
        Some(Self::DEFAULT_TIME_LIMIT_S)
    }

    /// Serde default of `mc_confirm` (the field is absent).
    fn default_mc_confirm() -> Option<McConfirm> {
        Some(McConfirm::DEFAULT)
    }
}

impl Default for SearchParams {
    /// The defaults of optimizer.md §11 (the same values an absent `time_limit_s` and `mc_confirm` take).
    fn default() -> Self {
        SearchParams {
            mode: SearchMode::Exact,
            certify: false,
            time_limit_s: Self::default_time_limit_s(),
            node_budget: None,
            keep: 32,
            mc_confirm: Self::default_mc_confirm(),
            threads: None,
            context_beam: 4,
            preset: Preset::Default,
        }
    }
}

/// Search modes (optimizer.md §1 and server-api.md §8.1 name them; their behaviour is the optimizer tasks', T32-T34).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SearchMode {
    /// `"exact"` (default).
    #[default]
    Exact,
    /// `"balanced"`.
    Balanced,
    /// `"quick"`.
    Quick,
}

/// MC confirmation of finalists (optimizer.md §7 step 3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct McConfirm {
    /// Finalists confirmed.
    pub finalists: u16,
    /// Runs per finalist.
    pub runs: u32,
    /// Seed shared by every finalist (common random numbers, 08 E-8.16).
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub seed: u64,
}

impl McConfirm {
    /// The default of optimizer.md §11: 12 finalists, 1,000 runs, seed 0.
    pub const DEFAULT: McConfirm = McConfirm {
        finalists: 12,
        runs: 1000,
        seed: 0,
    };
}

/// Presets (optimizer.md §10.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Preset {
    /// The defaults of optimizer.md §11.
    #[default]
    Default,
    /// The inspiration's settings (optimizer.md §10.2).
    InspirationCompatible,
}

/// Response 202 of `POST /api/optimize` and `POST /api/data/refresh`: `{ "job": "opt-7f3a9c" }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct JobAccepted {
    /// Job id.
    pub job: Box<str>,
}

// ---------------------------------------------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------------------------------------------

/// Result of an optimizer job (optimizer.md §1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OptResult {
    /// The best build.
    pub best: RankedBuild,
    /// The other finalists, best first.
    pub alternatives: Vec<RankedBuild>,
    /// Best build per final damage-type mix.
    pub per_mix: Vec<RankedBuild>,
    /// How good the search was.
    pub proof: Proof,
    /// Score without each slot's content.
    pub leave_one_out: Vec<SlotValue>,
    /// Best replacement per slot.
    pub next_best: Vec<SlotValue>,
    /// Best build re-scored with T-02 = `none` (a score: `"Infinity"` for a TTK metric when it no longer kills).
    #[serde(with = "crate::serde_util::ext_f64::option")]
    #[cfg_attr(
        feature = "ts",
        ts(type = "number | \"Infinity\" | \"-Infinity\" | null")
    )]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::serde_util::ExtF64>")
    )]
    pub condition_sensitivity: Option<f64>,
    /// Buff portfolio table.
    pub buff_table: Option<BuffTable>,
    /// Items left out and why.
    pub excluded: Vec<ExcludedItem>,
    /// Search statistics.
    pub stats: SearchStats,
    /// Capacity and Forma report of the best build; absent under capacity mode `ignore` (optimizer.md §7 step 4:
    /// "capacity and Forma report (09 E-9.16) unless the mode is `ignore`").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub capacity: Option<super::calc::CapacityReport>,
}

/// A ranked build (optimizer.md §1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RankedBuild {
    /// The build with the job's scenario.
    pub input: CalcInput,
    /// Its share code.
    pub code: Box<str>,
    /// Metric value in its unit (DPS, damage, or T_EV seconds for TTK); `"Infinity"` for a TTK build that never kills
    /// (E-8.26 band 2).
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub score: f64,
    /// Ordered key used for ranking.
    pub key: Score,
    /// TTK metrics: band, T_EV, κ, shots and ammo to kill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ttk: Option<TtkEv>,
    /// MC confirmation of `dps_window`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub mc: Option<Summary>,
    /// EV-versus-MC deviation flag (08 §8.11).
    pub ev_mc_deviation: bool,
    /// Gap classes active when the flag fires (08 §10, §12).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gap_classes: Vec<Box<str>>,
    /// Rank by EV when the finalists were re-ranked by the MC mean (08 §10: "EV ranks stay visible").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ev_rank: Option<u16>,
    /// Final damage-type mix.
    pub mix: Vec<DamageType>,
    /// Items with draft or stale effects in this build.
    pub draft_items: Vec<StableKey>,
}

/// The engine's ordered score key (engine.md §7.9): lower `band` is better, then higher `primary`, `tie`, `tie2`.
/// Bounds and thresholds use ±∞ in the tie fields, written `"Infinity"` / `"-Infinity"`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Score {
    /// 0 killed or DPS/hit metric, 1 extrapolated, 2 never (E-8.26).
    pub band: u8,
    /// Metric value, or −T_EV.
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub primary: f64,
    /// First tie-break (−ammo_window, κ).
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub tie: f64,
    /// Second tie-break (−ammo_to_kill).
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub tie2: f64,
}

/// How good the search was. JSON `{ "proven_optimal": { "scope": …, "assumes_fast_error": 0.005 } }`,
/// `{ "gap": { "upper_bound": …, "relative": 0.01 } }` or `"heuristic"`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Proof {
    /// Branch-and-bound finished.
    ProvenOptimal {
        /// What is proven.
        scope: ProofScope,
        /// ε of the WindowFast error assumption; `null` when certified (optimizer.md §5.2).
        assumes_fast_error: Option<f64>,
    },
    /// The search stopped with a bound gap.
    Gap {
        /// Upper bound of the unexplored part.
        upper_bound: Score,
        /// Relative gap between the bound and the incumbent; may be `"Infinity"` (a TTK bound in a better band than an
        /// incumbent that never kills).
        #[serde(with = "crate::serde_util::ext_f64")]
        #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
        #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
        relative: f64,
    },
    /// No bound statement.
    Heuristic,
}

/// Scope of a proof (optimizer.md §6.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ProofScope {
    /// The main slots are optimal for the final context.
    MainGivenContext,
}

/// A slot's content and the score of a variation of the best build: leave-one-out (the score without the content)
/// or next-best (the content that best replaces the slot's and the resulting score).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SlotValue {
    /// The slot.
    pub slot: SlotId,
    /// The content removed (leave-one-out) or put in (next-best).
    pub content: SlotContent,
    /// Score of the varied build (`"Infinity"` for a TTK build that never kills).
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub score: f64,
}

/// Buff portfolio (optimizer.md §6.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuffTable {
    /// One row per buff scenario: none, each buff alone, all buffs.
    pub rows: Vec<BuffScenarioRow>,
}

/// One buff scenario of the portfolio.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuffScenarioRow {
    /// Buffs on in this scenario.
    pub buffs: Vec<StableKey>,
    /// Best build of the scenario.
    pub best: RankedBuild,
    /// Score of the no-buff best build with these buffs on, minus its score without them. ±∞ under a TTK metric when
    /// one of the two never kills; `null` when both never kill (∞ − ∞).
    #[serde(with = "crate::serde_util::ext_f64::option")]
    #[cfg_attr(
        feature = "ts",
        ts(type = "number | \"Infinity\" | \"-Infinity\" | null")
    )]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::serde_util::ExtF64>")
    )]
    pub buff_value: Option<f64>,
    /// Score of the scenario's best build minus the score of the no-buff best build with these buffs on (the gain
    /// of rebuilding for the buffs); ±∞ and `null` as `buff_value`.
    #[serde(with = "crate::serde_util::ext_f64::option")]
    #[cfg_attr(
        feature = "ts",
        ts(type = "number | \"Infinity\" | \"-Infinity\" | null")
    )]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::serde_util::ExtF64>")
    )]
    pub rebuild_gain: Option<f64>,
}

/// An item left out of the search.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExcludedItem {
    /// The item.
    pub key: StableKey,
    /// Why (`opt.excluded.*`).
    pub reason: Message,
}

/// Search statistics (optimizer.md §1, §8).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SearchStats {
    /// Builds scored.
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub evaluated: u64,
    /// Bounds computed.
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub bounded: u64,
    /// Nodes pruned.
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub pruned: u64,
    /// Leaves reached.
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub leaves: u64,
    /// Time per phase.
    pub phases: Vec<PhaseTime>,
    /// Worker threads used.
    pub threads: u16,
    /// The result is bit-identical for any thread count (false after a wall-clock stop with threads > 1).
    pub reproducible: bool,
    /// Total time (ms).
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub elapsed_ms: u64,
}

/// Time spent in one phase.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PhaseTime {
    /// The phase.
    pub phase: OptPhase,
    /// Time (ms).
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub ms: u64,
}

// ---------------------------------------------------------------------------------------------------------------
// Events (SSE)
// ---------------------------------------------------------------------------------------------------------------

/// An optimizer event (optimizer.md §9). JSON (NDJSON of the CLI, `JobStatus.last_progress`): internally tagged,
/// `{ "event": "progress", "phase": …, … }`. On the SSE stream (server-api.md §8.2) the `event:` field carries the
/// tag and `data:` the remaining fields: [`OptEvent::to_sse`], [`OptEvent::from_sse`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum OptEvent {
    /// The job started: positions, pool sizes after pruning, composition sets, excluded items.
    Started {
        /// Job id.
        job: Box<str>,
        /// The search plan.
        plan: Plan,
    },
    /// A phase began.
    Phase {
        /// The phase.
        phase: OptPhase,
        /// 1-based index of the phase.
        index: u8,
        /// Number of phases.
        total: u8,
        /// Phase message (`opt.phase.*`).
        message: Message,
    },
    /// Progress (throttled to 4 per second).
    Progress {
        /// Current phase.
        phase: OptPhase,
        /// Done fraction (exact in the main search; `null` in local search).
        fraction: Option<f64>,
        /// Builds scored.
        #[serde(with = "super::int53")]
        #[cfg_attr(feature = "ts", ts(type = "number"))]
        #[cfg_attr(feature = "schema", schemars(with = "u64"))]
        evaluated: u64,
        /// Bounds computed.
        #[serde(with = "super::int53")]
        #[cfg_attr(feature = "ts", ts(type = "number"))]
        #[cfg_attr(feature = "schema", schemars(with = "u64"))]
        bounded: u64,
        /// Nodes pruned.
        #[serde(with = "super::int53")]
        #[cfg_attr(feature = "ts", ts(type = "number"))]
        #[cfg_attr(feature = "schema", schemars(with = "u64"))]
        pruned: u64,
        /// Evaluations per second.
        rate_per_s: f64,
        /// Elapsed time (ms).
        #[serde(with = "super::int53")]
        #[cfg_attr(feature = "ts", ts(type = "number"))]
        #[cfg_attr(feature = "schema", schemars(with = "u64"))]
        elapsed_ms: u64,
        /// Estimated remaining time (s).
        eta_s: Option<f64>,
        /// Current relative bound gap (as `Proof::Gap.relative`, may be `"Infinity"`); `null` before a bound exists.
        #[serde(with = "crate::serde_util::ext_f64::option")]
        #[cfg_attr(
            feature = "ts",
            ts(type = "number | \"Infinity\" | \"-Infinity\" | null")
        )]
        #[cfg_attr(
            feature = "schema",
            schemars(with = "Option<crate::serde_util::ExtF64>")
        )]
        gap: Option<f64>,
    },
    /// The top builds changed (every change of the top 1; top 5 changes throttled).
    Incumbent {
        /// 1-based rank.
        rank: u16,
        /// The build.
        build: Box<RankedBuild>,
    },
    /// A finalist after MC confirmation.
    Finalist {
        /// The build.
        build: Box<RankedBuild>,
    },
    /// A log line (i18n key and arguments, I-8).
    Log {
        /// The message.
        message: Message,
    },
    /// The job finished.
    Done {
        /// The result.
        result: Box<OptResult>,
    },
    /// The job was cancelled; the anytime result if any.
    Cancelled {
        /// The anytime result.
        result: Option<Box<OptResult>>,
    },
    /// The job failed.
    Failed {
        /// The reason.
        error: Message,
    },
}

impl OptEvent {
    /// The SSE `event:` name (the variant in `snake_case`).
    pub fn name(&self) -> &'static str {
        match self {
            OptEvent::Started { .. } => "started",
            OptEvent::Phase { .. } => "phase",
            OptEvent::Progress { .. } => "progress",
            OptEvent::Incumbent { .. } => "incumbent",
            OptEvent::Finalist { .. } => "finalist",
            OptEvent::Log { .. } => "log",
            OptEvent::Done { .. } => "done",
            OptEvent::Cancelled { .. } => "cancelled",
            OptEvent::Failed { .. } => "failed",
        }
    }

    /// `true` for the events after which the server closes the stream.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OptEvent::Done { .. } | OptEvent::Cancelled { .. } | OptEvent::Failed { .. }
        )
    }

    /// The SSE `event:` name and `data:` JSON (server-api.md §8.2).
    pub fn to_sse(&self) -> Result<(&'static str, String), serde_json::Error> {
        super::sse::to_sse(self, self.name())
    }

    /// Parses an SSE message (`event:` name and `data:` JSON).
    pub fn from_sse(event: &str, data: &str) -> Result<Self, serde_json::Error> {
        super::sse::from_sse(event, data)
    }
}

/// Optimizer phases (optimizer.md §9).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum OptPhase {
    /// Problem compilation: pools, pruning, dominance (optimizer.md §2).
    Prepare,
    /// Composition sets of the main slots (optimizer.md §5.1).
    Composition,
    /// Branch and bound over the main slots (optimizer.md §5.2).
    MainSearch,
    /// Context search over the other slot groups (optimizer.md §6.2).
    Context,
    /// Local search (optimizer.md §6.2).
    LocalSearch,
    /// Best build per final damage-type mix (optimizer.md §1 `per_mix`).
    PerMix,
    /// Riven trait search (optimizer.md §6.2 step 6).
    Riven,
    /// Buff portfolio scenarios (optimizer.md §6.3).
    Buffs,
    /// Re-scoring and ranking of the finalists (optimizer.md §7 step 2).
    Finalists,
    /// MC confirmation of the finalists (optimizer.md §7 step 3).
    McConfirm,
}

/// The search plan of a job.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Plan {
    /// Searched positions and their pool sizes after pruning.
    pub positions: Vec<PlanPosition>,
    /// Composition sets (optimizer.md §5.1).
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub composition_sets: u64,
    /// Items left out and why.
    pub excluded: Vec<ExcludedItem>,
}

/// A searched position.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PlanPosition {
    /// Slot group.
    pub group: OptGroup,
    /// Candidates after pruning.
    pub pool: u32,
}

/// Slot groups of the search (optimizer.md §2 step 2, §6.1).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum OptGroup {
    /// The 8 main slots (one shared pool).
    Main,
    /// Weapon exilus slot ([`GroupSwitches::exilus`]).
    Exilus,
    /// Weapon arcane slots.
    WeaponArcanes,
    /// Stance and combo.
    Stance,
    /// Melee plan (spec 00 §6.1 T-30).
    MeleePlan,
    /// Incarnon perks.
    Incarnon,
    /// General frame slots.
    FrameMods,
    /// Frame exilus slot.
    FrameExilus,
    /// Aura slots.
    Aura,
    /// Frame arcanes.
    FrameArcanes,
    /// Archon shards.
    Shards,
    /// Kitgun and zaw parts.
    ModularParts,
    /// Progenitor element.
    Progenitor,
    /// Riven traits (optimizer.md §6.2 step 6).
    Riven,
    /// External buffs (optimizer.md §6.3).
    Buffs,
}

/// `GET /api/optimize/{job}` (server-api.md §8.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, bound(deserialize = "R: Deserialize<'de>"))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct JobStatus<R> {
    /// Job id.
    pub job: Box<str>,
    /// State.
    pub state: JobState,
    /// Start time (RFC 3339).
    pub started_at: Box<str>,
    /// The last progress event (its JSON form).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional, type = "unknown"))]
    pub last_progress: Option<serde_json::Value>,
    /// The result of a finished (or cancelled) job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub result: Option<R>,
    /// The reason of a failed job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub error: Option<Message>,
}

/// State of a job.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum JobState {
    /// The job is running.
    Running,
    /// The job finished; `result` holds the result.
    Done,
    /// The job was cancelled; `result` holds the anytime result if any.
    Cancelled,
    /// The job failed; `error` holds the reason.
    Failed,
}

#[cfg(test)]
mod capacity_tests {
    use super::*;

    /// The precedence of server-api.md §8.1: the rule's mode and Umbra flag win, the other Forma types come from the
    /// scenario.
    #[test]
    fn capacity_rule_wins_over_the_scenario_toggles() {
        let mut toggles = Toggles {
            capacity: CapacityMode::Ignore,
            ..Toggles::default()
        };
        toggles.forma_types.remove(FormaTypes::OMNI);
        let rule = CapacityRule {
            mode: CapacityMode::Budget(2),
            umbra_forma: false,
            ranks: RankPolicy::Max,
        };
        let (mode, types) = rule.effective(&toggles);
        assert_eq!(mode, CapacityMode::Budget(2));
        assert_eq!(
            types,
            FormaTypes::FORMA | FormaTypes::STANCE | FormaTypes::AURA
        );
        toggles.forma_types.remove(FormaTypes::UMBRA);
        let allow = CapacityRule {
            umbra_forma: true,
            ..rule
        };
        assert!(allow.effective(&toggles).1.contains(FormaTypes::UMBRA));
    }
}
