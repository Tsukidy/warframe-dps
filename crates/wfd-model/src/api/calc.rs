//! Calculation routes (server-api.md §6): `POST /api/calc`, `/api/timeline`, `/api/explain`, and the report types the
//! engine produces (engine.md §1, §7.7, §8.4, §9: `CalcReport`, `EvOut`, `McOut`, `Timeline`, `Explanation` are API
//! DTOs, so the frontend's types come from one crate).
//!
//! Where spec 08 and server-api.md differ, the spec wins (work-breakdown.md §2.4; CONTRACTS.md, T01b):
//! `ViewEv.per_shot` and `.per_magazine` are optional (08 §8.5: none for N_mag = ∞ and for gauge-gated attacks), and
//! so are `ViewEv.dps_burst`, `.dps_sustained` and `.hit_infight` (08 E-8.14: "not available" for gauge-gated
//! attacks); `TtkEv.kappa` exists only in band 0 (08 E-8.21, E-8.26); `Summary.sd` and `.ci95` are `null` for one run
//! (08 E-8.19); MC crit histograms are normalised fractions (08 §8.4); `CapacityReport.forma` is `null` when no
//! polarity layout fits (09 E-9.19).

use super::common::{BaseModded, FalloffView, SourceDamage, SourceDps, double_option};
use crate::build::Violation;
use crate::enums::{DamageType, N_TIER_BINS, StatusType, View};
use crate::ids::{AttackKey, EffectKey, StableKey};
use crate::message::Message;
use crate::scenario::{CalcInput, CapacityMode, CondOverride};
use crate::serde_util::is_false;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------------------------------------------
// POST /api/calc
// ---------------------------------------------------------------------------------------------------------------

/// `POST /api/calc` body (server-api.md §6.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CalcRequest {
    /// Build and scenario.
    pub input: CalcInput,
    /// What to compute besides EV (always computed for every attack, group and present view).
    #[serde(default)]
    pub want: CalcWant,
}

/// The optional parts of a calculation (the server maps it onto the engine's `Want`, engine.md §1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CalcWant {
    /// Monte Carlo (passes V and L; L0 too with `parity` when the enemy has armor, 08 §6.4); absent = EV only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub mc: Option<McWant>,
    /// Parity outputs of 08 §8.8.
    #[serde(default, skip_serializing_if = "is_false")]
    pub parity: bool,
    /// Per-source breakdowns.
    #[serde(default, skip_serializing_if = "is_false")]
    pub breakdown: bool,
    /// Stage traces (engine.md §11.1).
    #[serde(default, skip_serializing_if = "is_false")]
    pub trace: bool,
    /// Views to evaluate; absent = every view the enemy has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub views: Option<Vec<View>>,
}

/// MC request parameters; absent fields take the scenario's `eval` values (T-29).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct McWant {
    /// Number of runs (1-100,000).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub runs: Option<u32>,
    /// Seed 0 ..= 2^53 − 1; required when `eval.seed` is `"auto"` (the frontend draws it).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::int53::option"
    )]
    #[cfg_attr(feature = "ts", ts(optional, type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "Option<u64>"))]
    pub seed: Option<u64>,
    /// Width of the series bins (s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub series_bin_s: Option<f64>,
}

/// `POST /api/calc` response: everything the calculator page shows (engine.md §9, spec 00 §4.5).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CalcReport {
    /// Data version the report was computed with.
    pub data_version: Box<str>,
    /// Share code of the inputs (filled by the server).
    pub code: Box<str>,
    /// Weapon panel (08 §8.10).
    pub weapon: WeaponPanel,
    /// Enemy panel (08 §8.10).
    pub enemy: EnemyPanel,
    /// Per attack: per view `{ ev, mc? }`.
    pub attacks: Vec<AttackReport>,
    /// Parent + child sums and T-22 combined units (08 §9).
    pub groups: Vec<GroupReport>,
    /// Every conditional effect of the build with its class and treatment (domain-model.md §9.4).
    pub conditions: Vec<ConditionView>,
    /// Register rows (`"D-023"`) the scenario exercised, shown as assumptions (engine.md §10).
    pub touched_rows: Vec<Box<str>>,
    /// Spec 09 rule violations; never block evaluation (09 §1.5).
    pub violations: Vec<Violation>,
    /// Capacity and Forma report (09 E-9.14-E-9.19). E-9.19 is a report in every capacity mode (a constraint only in
    /// `any_forma` and `budget`), so it is present under `ignore` too; absent only when the input has no weapon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub capacity: Option<CapacityReport>,
    /// Stage traces when `want.trace` was set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub trace: Option<Vec<TraceEntry>>,
    /// Unmodelled items, UNVERIFIED rules, EV-MC deviation … (engine.md §15).
    pub warnings: Vec<Message>,
    /// Server time spent (µs).
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub timing_us: u64,
}

/// Capacity and Forma report (09 E-9.14-E-9.19). The top-level fields describe the weapon (server-api.md §6.1);
/// `warframe` the frame of a complete frame build.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CapacityReport {
    /// Capacity used U = Σ c_ω − β (E-9.17) of the layout `forma` describes (the minimum-Forma layout of E-9.19,
    /// ties broken by the smaller U); when no layout fits (`forma` is `null`), the smallest U any layout reaches, so
    /// `used − capacity` is the excess. Negative when the stance bonus exceeds the drain.
    pub used: i32,
    /// Item capacity C_cap (E-9.16).
    pub capacity: i32,
    /// Regular Forma of the minimum-Forma layout N_forma* (E-9.19, reported per Forma type); `null` when no layout
    /// reaches U ≤ C_cap (E-9.19: "No layout ⇒ cap.exceeded in every mode but ignore").
    pub forma: Option<u8>,
    /// Umbra Forma of that layout; `null` exactly when `forma` is.
    pub umbra_forma: Option<u8>,
    /// Capacity mode of the scenario (E-9.18): whether the report was also a constraint.
    pub mode: CapacityMode,
    /// Overlevelled weapon at rank > 30: the lower bound ceil((rank − 30)/2) on the Forma applied to the item
    /// (E-9.16, E-9.19 "Report"); absent at rank ≤ 30.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub forma_applied_min: Option<u8>,
    /// The warframe's capacity (E-9.17 warframe form: general and exilus slots minus the aura bonus); present when the
    /// build has a warframe with `frame_complete` = true, the only case E-9.18 checks it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub warframe: Option<ItemCapacity>,
}

/// Capacity and Forma of one more item of the build (09 E-9.17, E-9.19); fields as in [`CapacityReport`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ItemCapacity {
    /// Capacity used U of the layout `forma` describes (or the smallest U when no layout fits); negative when the
    /// aura bonus exceeds the drain.
    pub used: i32,
    /// Item capacity C_cap (E-9.16).
    pub capacity: i32,
    /// Regular Forma of the minimum-Forma layout; `null` when no layout reaches U ≤ C_cap.
    pub forma: Option<u8>,
    /// Umbra Forma of that layout; `null` exactly when `forma` is.
    pub umbra_forma: Option<u8>,
}

/// Weapon panel: base versus modded stats per attack (08 §8.10).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponPanel {
    /// One row per attack.
    pub attacks: Vec<WeaponPanelAttack>,
    /// Magazine size (rounds); absent for weapons without a magazine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub magazine: Option<BaseModded>,
    /// Incarnon-form magazine (rounds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub incarnon_magazine: Option<f64>,
    /// Reload time (s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub reload: Option<BaseModded>,
    /// Further rows by stat key (`"combo_dur"`, `"attack_speed"`, `"ammo_max"` …); `ammo_max` is `"Infinity"` on
    /// robotic weapons (06 §11.4).
    pub extra: BTreeMap<Box<str>, BaseModded>,
}

/// One attack of the weapon panel.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponPanelAttack {
    /// The attack.
    pub key: AttackKey,
    /// Display name (English from the data; the UI may localize through the name table).
    pub name: Box<str>,
    /// Unmodded stats.
    pub base: StatBlock,
    /// Resolved stats of ST-1 to ST-7 on the fresh target (08 §8.10).
    pub modded: StatBlock,
    /// Per-type damage `q_t·Y·(1 + F)` (nonzero types only).
    pub per_type: BTreeMap<DamageType, f64>,
    /// Sum of `per_type`.
    pub total: f64,
}

/// Firing stats of an attack (fractions, factors, per-second rates).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatBlock {
    /// Critical chance.
    pub cc: f64,
    /// Critical damage multiplier.
    pub cd: f64,
    /// Status chance.
    pub sc: f64,
    /// Multishot.
    pub multishot: f64,
    /// Fire rate (attacks/s; attack speed for melee); `"Infinity"` for `math.huge` (Lanka, 06 §4.1).
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub fire_rate: f64,
    /// Effective fire rate EFR (06 E-6.25; 08 §8.10 lists it in the weapon panel).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub efr: Option<f64>,
    /// Charge time (s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub charge_time: Option<f64>,
    /// Delay between burst shots (s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub burst_delay: Option<f64>,
    /// Blast radius (m).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub radius: Option<f64>,
    /// Punch through (m).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub punch_through: Option<f64>,
    /// Ammo cost per shot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ammo_cost: Option<f64>,
    /// Damage falloff.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub falloff: Option<FalloffView>,
    /// Forced procs.
    pub forced: Vec<StatusType>,
}

/// Enemy panel (08 §8.10, 05).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyPanel {
    /// Enemy level.
    pub level: u16,
    /// Scaled pools (HP).
    pub pools: super::common::LayerValues,
    /// Scaled armor A0.
    pub armor0: f64,
    /// Effective armor after strips (E-15).
    pub armor_eff: f64,
    /// Armor damage reduction.
    pub dr: f64,
    /// Innate damage reduction per layer.
    pub innate_dr: super::common::LayerValues,
    /// Status immunities.
    pub immune: Vec<StatusType>,
    /// Unique damage reduction note (attenuation, flat DR, adaptation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub special_dr: Option<Message>,
    /// Faction damage modifiers that differ from 1 (05 §5.1).
    pub faction_mods: BTreeMap<DamageType, f64>,
    /// Squad size used for scaling.
    pub party: u8,
    /// Status stack caps that differ from the default (08 §8.10 "caps"; engine.md §9).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub caps: BTreeMap<StatusType, u16>,
}

/// Results of one attack.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackReport {
    /// The attack.
    pub key: AttackKey,
    /// Display name.
    pub name: Box<str>,
    /// One entry per evaluated view.
    pub views: Vec<ViewReport>,
}

/// Results of one view.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ViewReport {
    /// The layer view.
    pub view: View,
    /// Expected values.
    pub ev: ViewEv,
    /// Monte Carlo results, when requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub mc: Option<ViewMc>,
}

/// EV outputs of one view (engine.md §7.7; 08 §8, §10).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ViewEv {
    /// 08 E-8.14; absent for gauge-gated attacks (E-8.14: no steady metrics, "not available").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub dps_burst: Option<f64>,
    /// 08 E-8.14; absent for gauge-gated attacks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub dps_sustained: Option<f64>,
    /// 08 E-8.12 (headline).
    pub dps_window: f64,
    /// Expected damage of one shot at the steady state (08 §8.5); absent for gauge-gated attacks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub per_shot: Option<f64>,
    /// `per_shot · N_mag` (06 E-6.27); absent for N_mag = ∞ and gauge-gated attacks (08 §8.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub per_magazine: Option<f64>,
    /// Landed direct damage per second by type (nonzero types only, 08 §8.2).
    pub per_type: BTreeMap<DamageType, f64>,
    /// Landed damage per second by source (08 §8.2).
    pub per_source: Vec<SourceDps>,
    /// Steady stacks per status (08 §8.3).
    pub stacks: Vec<StackStat>,
    /// `E[n]`, the expected distinct-status count (08 §8.3).
    pub distinct_mean: f64,
    /// EV window crit-tier distribution `crit_tier_dist` (08 §8.4): the mean of P(k = x) over the direct instances of
    /// the window, weighted by their existence probabilities; the counterpart of `ViewMc.crit_hist`. Tiers 0 … 6 and
    /// 7 = "7 or more".
    pub crit_tiers: [f64; N_TIER_BINS],
    /// EV steady crit-tier distribution (08 §8.4: P(k = x) at the steady state, averaged over n_PUN); bins as
    /// `crit_tiers`. Absent for gauge-gated attacks (no steady state, E-8.14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub crit_tiers_steady: Option<[f64; N_TIER_BINS]>,
    /// Expected DPS per bin of `series_bin_s` (08 §8.1).
    pub series: Vec<f64>,
    /// EV window curves `uptime[s][k]` and `stacks_mean[s][k]` per status, bins as `series` (08 §8.3: the time
    /// average over the bin of the E-8.3 quantities, comparable with `ViewMc.uptime`); omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uptime: Vec<StatusSeries>,
    /// Damage landing after the window from procs applied in it (08 §8.7).
    pub pending_dot: f64,
    /// EV TTK, layered view only (08 E-8.21).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ttk: Option<TtkEv>,
    /// Expected damage of the first shot on the fresh target (08 §10).
    pub hit_fresh: f64,
    /// Expected damage of one parent hit at the steady state (08 §10); absent for gauge-gated attacks (E-8.14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub hit_infight: Option<f64>,
    /// Ammo spent in the window (08 §10).
    pub ammo_window: f64,
}

/// Steady stack statistics of one status (08 §8.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StackStat {
    /// The status.
    pub status: StatusType,
    /// `E[n_s]`.
    pub mean: f64,
    /// P(n_s ≥ 1).
    pub p_any: f64,
    /// P(n_s = cap_s).
    pub p_cap: f64,
}

/// EV TTK (08 E-8.21, §8.6).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TtkEv {
    /// Killed within the horizon, extrapolated beyond it, or never.
    pub band: TtkBand,
    /// T_EV (s); `null` = never.
    pub seconds: Option<f64>,
    /// Overkill margin κ at T_EV (E-8.21), defined for a kill on the expected path within the horizon: present
    /// exactly in band `killed` (E-8.26 uses it only there).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub kappa: Option<f64>,
    /// Shots to kill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub shots_to_kill: Option<u32>,
    /// Ammo to kill (rounds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ammo_to_kill: Option<f64>,
    /// Shields may regenerate during the fight (05 §10.5).
    pub shield_regen_possible: bool,
    /// Damage by source along the expected path up to T_EV (08 §8.2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breakdown: Vec<SourceDamage>,
}

/// TTK band of 08 E-8.26.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TtkBand {
    /// Killed by `t_ttk_max` (band 0).
    Killed,
    /// Extrapolated beyond `t_ttk_max`.
    Extrapolated,
    /// Never killed (a layer gets no damage).
    Never,
}

/// Statistics over MC runs (08 E-8.19).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Summary {
    /// Mean.
    pub mean: f64,
    /// Sample standard deviation (R − 1); `null` for one run (E-8.19).
    pub sd: Option<f64>,
    /// 95 % normal-approximation interval of the mean; `null` for one run.
    pub ci95: Option<[f64; 2]>,
    /// 99 % interval of the mean (the E-8.24 deviation test uses it); absent for one run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ci99: Option<[f64; 2]>,
    /// qnt(0.5).
    pub median: f64,
    /// qnt(0.05).
    pub p5: f64,
    /// qnt(0.25).
    pub p25: f64,
    /// qnt(0.75).
    pub p75: f64,
    /// qnt(0.95).
    pub p95: f64,
    /// Smallest value.
    pub min: f64,
    /// Largest value.
    pub max: f64,
}

/// MC outputs of one view (engine.md §8.4; 08 §6.5, §8).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ViewMc {
    /// Number of runs.
    pub runs: u32,
    /// Seed of the runs.
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub seed: u64,
    /// `dps_window_mc` (pass V).
    pub dps_window: Summary,
    /// Per bin: mean, qnt(0.10), qnt(0.90) (08 §8.1).
    pub series: Vec<BinSummary>,
    /// Uptime and mean stacks per status and bin (08 §8.3).
    pub uptime: Vec<StatusSeries>,
    /// Crit-tier histogram of direct instances, normalised, bins as `ViewEv.crit_tiers` (08 §8.4).
    pub crit_hist: [f64; N_TIER_BINS],
    /// Mean landed damage per second by source (08 §8.2).
    pub breakdown: Vec<SourceDps>,
    /// Mean damage landing after the window (08 §8.7).
    pub pending_dot: f64,
    /// TTK distribution, pass L, layered view only (08 E-8.20).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ttk: Option<TtkDist>,
    /// Parity rows (08 E-8.22), for the non-layered views when requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub parity: Option<ParityOut>,
}

/// Mean and band of one series bin.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BinSummary {
    /// Mean over runs.
    pub mean: f64,
    /// qnt(0.10).
    pub p10: f64,
    /// qnt(0.90).
    pub p90: f64,
}

/// Uptime and mean stacks of one status per bin (08 §8.3 window curves). MC: the time average over the bin,
/// averaged over runs; EV: the time average over the bin of the E-8.3 quantities.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatusSeries {
    /// The status.
    pub status: StatusType,
    /// `uptime[s][k]`: mean of 1{n_s ≥ 1} per bin.
    pub uptime: Vec<f64>,
    /// `stacks_mean[s][k]`: mean of n_s per bin.
    pub mean_stacks: Vec<f64>,
}

/// MC TTK distribution (08 E-8.20).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TtkDist {
    /// P_kill.
    pub kill_probability: f64,
    /// Median TTK (s); `null` = not killed.
    pub median: Option<f64>,
    /// Mean TTK of the runs that killed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub mean_killed: Option<f64>,
    /// qnt(0.05); `null` = not killed.
    pub p5: Option<f64>,
    /// qnt(0.95); `null` = not killed.
    pub p95: Option<f64>,
    /// Median shots to kill over killed runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub shots_to_kill: Option<f64>,
    /// Median ammo to kill over killed runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ammo_to_kill: Option<f64>,
    /// Shields may regenerate during the fight (05 §10.5).
    pub shield_regen_possible: bool,
    /// Mean damage by source up to the kill, "Magnetic break" included (08 §8.2, §8.6).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breakdown: Vec<SourceDamage>,
}

/// The inspiration's result rows (08 E-8.22): medians over runs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ParityOut {
    /// Median of `queue_dps_parity_r`.
    pub queue_dps: f64,
    /// Smallest `queue_dps_parity_r`.
    pub queue_dps_min: f64,
    /// Largest `queue_dps_parity_r`.
    pub queue_dps_max: f64,
    /// Median of `avg_direct_r`.
    pub avg_direct: f64,
    /// Median of `avg_status_r`.
    pub avg_status: f64,
    /// Median of `avg_total_r`.
    pub avg_total: f64,
    /// Median of `median_total_r`.
    pub median_total: f64,
    /// Parity TTK (Health, Shield, Overguard rows: pass L0; Armor row: pass L, D-802): absent when not computed,
    /// `null` = not killed.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub ttk: Option<Option<f64>>,
}

/// An attack group: parent + children summed (T-22 off) or one combined unit (T-22 on), 08 §9.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GroupReport {
    /// Attack indices of the members.
    pub members: Vec<u8>,
    /// T-22 on: one evaluation unit.
    pub combined: bool,
    /// One entry per view.
    pub views: Vec<ViewReport>,
}

/// A conditional effect of the build and how it is treated (domain-model.md §9.4, §12).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConditionView {
    /// The effect.
    pub effect: EffectKey,
    /// The item that carries it.
    pub item: StableKey,
    /// Derived condition classes (domain-model.md §12.1).
    pub class: Vec<CondClass>,
    /// Current treatment under T-02 and the overrides.
    pub treatment: Treatment,
    /// Value in effect (model units; stacks applied).
    pub value: f64,
    /// The user's override, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub r#override: Option<CondOverride>,
}

/// Condition classes (a), (b), (c) of 04 §4.1 / 00 §6.2.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CondClass {
    /// (a) target state: CO-type per-status bonuses, health thresholds (00 §6.2).
    A,
    /// (b) own events: on hit, on crit, on reload, combo … (00 §6.2).
    B,
    /// (c) external: on kill, on ability cast, while invisible … (00 §6.2).
    C,
}

/// Treatment of a conditional effect (00 §6.2; domain-model.md §12.1-§12.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Treatment {
    /// Active at maximum stacks for the whole window (T-02 `all_max`, or the override `force: true`).
    On,
    /// Inactive (T-02 `none`, or the override `force: false`).
    Off,
    /// Class (a) under `simulated`: from the target state, its expectation in EV, live in MC.
    Expected,
    /// Class (b) under `simulated`: triggered by the attack's own events, with stacks, durations and decay (04).
    Simulated,
    /// Class (c) under `simulated`, and timing-stat effects with random triggers: the uptime u_e and stacks m_e of
    /// 04 §4.3 (user inputs or the override `uptime`).
    Uptime,
    /// Driven by the magazine schedule in every mode (first/last shot, reloaded fraction; 00 §6.2 last paragraphs).
    Schedule,
    /// Evaluated on the resolved attack in every mode (`Cond::Attack`, domain-model.md §12.1).
    Formula,
}

/// One stage trace (engine.md §11.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TraceEntry {
    /// Probe path (engine.md §11.2).
    pub probe: Box<str>,
    /// Equation id (`"E-5"`).
    pub equation: Box<str>,
    /// Value.
    pub value: f64,
    /// Input probes and their values.
    pub inputs: Vec<ProbeValue>,
    /// Effects that contributed and their contributions.
    pub contributions: Vec<Contribution>,
    /// Register rows the stage exercised.
    pub rows: Vec<Box<str>>,
}

/// A probe and its value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProbeValue {
    /// Probe path.
    pub probe: Box<str>,
    /// Value.
    pub value: f64,
}

/// An effect's contribution to a traced value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Contribution {
    /// The effect.
    pub effect: EffectKey,
    /// Its contribution (model units).
    pub value: f64,
}

/// EV outputs of one evaluation unit, per view (engine.md §7.7; re-exported by `wfd-engine`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EvOut {
    /// One entry per evaluated view.
    #[cfg_attr(feature = "ts", ts(as = "Vec<(View, ViewEv)>"))]
    pub views: SmallVec<[(View, ViewEv); 5]>,
}

/// MC outputs of one evaluation unit, per view (engine.md §8.4; re-exported by `wfd-engine`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct McOut {
    /// One entry per evaluated view.
    #[cfg_attr(feature = "ts", ts(as = "Vec<(View, ViewMc)>"))]
    pub views: SmallVec<[(View, ViewMc); 5]>,
    /// Number of runs.
    pub runs: u32,
    /// Seed of the runs.
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub seed: u64,
}

// ---------------------------------------------------------------------------------------------------------------
// POST /api/timeline
// ---------------------------------------------------------------------------------------------------------------

/// `POST /api/timeline` body (server-api.md §6.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TimelineRequest {
    /// Build and scenario.
    pub input: CalcInput,
    /// An attack or an attack group.
    pub attack: TimelineTarget,
    /// The view.
    pub view: View,
    /// MC seed; default the scenario's.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::int53::option"
    )]
    #[cfg_attr(feature = "ts", ts(optional, type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "Option<u64>"))]
    pub seed: Option<u64>,
    /// Run index (default 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub run: Option<u32>,
    /// Maximum number of hits (default 2000).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub limit: Option<u32>,
}

/// Which unit a timeline shows: `{ "slug": …, "idx": … }` or `{ "group": n }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TimelineTarget {
    /// One attack.
    Attack(AttackKey),
    /// An attack group (index into `CalcReport.groups`).
    Group(GroupRef),
}

/// `{ "group": n }`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GroupRef {
    /// Index into `CalcReport.groups`.
    pub group: u8,
}

/// Per-hit event log of one MC run (08 §8.9).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Timeline {
    /// Simulated time (s).
    pub duration: f64,
    /// Parent hits in time order.
    pub hits: Vec<TimelineHit>,
    /// Reloads, numbered "↻ #n".
    pub reloads: Vec<ReloadRecord>,
    /// Heat episodes (08 §8.9).
    pub heat_episodes: Vec<HeatEpisode>,
    /// `true` when `limit` cut the log.
    pub truncated: bool,
}

/// One parent hit (08 §8.9).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TimelineHit {
    /// 1-based index of the parent hit.
    pub n: u32,
    /// Time (s).
    pub t: f64,
    /// Damage on the view.
    pub damage: f64,
    /// Crit tier k.
    pub crit_tier: u8,
    /// Crit factor c_k.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub crit_mult: Option<f64>,
    /// Effective armor at the hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub armor_eff: Option<f64>,
    /// Armor damage reduction at the hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub armor_reduction: Option<f64>,
    /// Stance multiplier (melee).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub stance_mult: Option<f64>,
    /// Heavy-attack combo multiplier (melee).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub heavy_combo_mult: Option<f64>,
    /// Extra hits of this hit: `kind` = `xata`, `toxic_lash`, `toxic_lash_tick`, `flat_per_status`, or an effect key.
    pub extra_hits: Vec<ExtraHitRecord>,
    /// Statuses applied by the hit.
    pub statuses_applied: Vec<StatusType>,
    /// Component procs added by Primary Debilitate (marked separately, 08 §8.9).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses_added: Vec<StatusType>,
    /// One-tick weapon-side value of each DoT proc applied.
    pub tick_values: BTreeMap<StatusType, f64>,
    /// Live stacks before the hit (game order, CD-04).
    pub stacks: BTreeMap<StatusType, u16>,
    /// Markers.
    pub flags: Vec<HitFlag>,
}

/// An extra hit in the timeline.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExtraHitRecord {
    /// Source kind (see `TimelineHit.extra_hits`).
    pub kind: Box<str>,
    /// Damage on the view.
    pub damage: f64,
}

/// Markers of a timeline hit: the schedule flags of its shot (06 §6.1, E-6.13).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HitFlag {
    /// The shot's `first` flag (06 E-6.13: first shot of a magazine by the HUD counter; first-shot effects).
    First,
    /// The shot's `last` flag (06 E-6.13: the shot ends its magazine; last-shot effects).
    Last,
    /// The shot's `after_reload` flag of the schedule record (06 §6.1 shot record).
    AfterReload,
}

/// A reload in the timeline.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReloadRecord {
    /// 1-based reload number.
    pub n: u32,
    /// Start (s).
    pub start: f64,
    /// End (s).
    pub end: f64,
}

/// A Heat episode (ignition to end, 08 §8.9).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HeatEpisode {
    /// Ignition time (s).
    pub ignition: f64,
    /// End time (s).
    pub end: f64,
}

// ---------------------------------------------------------------------------------------------------------------
// POST /api/explain
// ---------------------------------------------------------------------------------------------------------------

/// `POST /api/explain` body (server-api.md §6.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExplainRequest {
    /// Build and scenario.
    pub input: CalcInput,
    /// Probe path (engine.md §11.2).
    pub probe: Box<str>,
}

/// Breakdown of one probed value (engine.md §11.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Explanation {
    /// Probe path.
    pub probe: Box<str>,
    /// Value.
    pub value: f64,
    /// Equation id (`"E-5"`).
    pub equation: Box<str>,
    /// The formula as a message.
    pub formula: Message,
    /// Inputs of the equation.
    pub inputs: Vec<ExplainInput>,
    /// Effects that contributed.
    pub contributions: Vec<ExplainContribution>,
    /// Register rows the value depends on.
    pub rows: Vec<ExplainRow>,
}

/// An input of an explained value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExplainInput {
    /// Probe path.
    pub probe: Box<str>,
    /// Label.
    pub label: Message,
    /// Value.
    pub value: f64,
}

/// An effect's contribution to an explained value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExplainContribution {
    /// The effect.
    pub effect: EffectKey,
    /// The item that carries it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub item: Option<StableKey>,
    /// Label.
    pub label: Message,
    /// Contribution (model units).
    pub value: f64,
}

/// A register row behind an explained value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExplainRow {
    /// Row id (`"D-023"`).
    pub id: Box<str>,
    /// Its text.
    pub text: Message,
}
