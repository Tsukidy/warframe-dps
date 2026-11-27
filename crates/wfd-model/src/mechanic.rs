//! Mechanic kinds (domain-model.md §11.2): a closed set, each implemented once in `wfd-engine::mechanics` for both
//! evaluators. A mechanic that fits no kind needs a new kind (code change reviewed against the spec plus a spec
//! vector); there are no per-item hooks (I-2).
//!
//! The magnitude of a mechanic is the effect's `value` (domain-model.md §11.2 "Magnitude slot": p_xh, the proc
//! chance, the repeat chance, the flat amount, the counter step, the random-multiplier chance, the spawn chance), so the
//! parameters below never repeat it. The readable form is `mechanic = "<kind>"` plus `[effect.params]`; the params
//! table is parsed into the union [`MechanicParamsRepr`] and checked against the kind.

use crate::effect::{ElemSel, HitUnit, RankCurve, Scope};
use crate::enums::{DamageType, DamageTypeSet, InputKind, StatusSet, StatusType};
use crate::stats::StatKey;
use crate::weapon::Lingering;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Name of a mechanic kind (`mechanic = "extra_hit"`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MechanicKind {
    ExtraHit,
    ProcOnEvent,
    ConvertDamage,
    RepeatHit,
    FlatDamageOnProc,
    ProcAfterTicks,
    CounterBonus,
    RandomMultiplier,
    SpawnAttack,
}

/// A mechanic with its typed parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Mechanic {
    /// E-14, 04 §6: Xata's Whisper, Toxic Lash, Resupply, Silken Stride, Demonium, Sangodae, Gastroparesis.
    /// Magnitude: p_xh (slot `xh.<kind index>`).
    ExtraHit(ExtraHitParams),
    /// 02 §4.7.3-4.7.5: Hunter Munitions, Internal Bleeding, Hemorrhage, Magnetic Welt, Secondary Encumber, Secondary
    /// Cryogenic, Archon Continuity. Magnitude: the chance (`mech.<n>`).
    ProcOnEvent(ProcOnEventParams),
    /// 01 §4.4, D-009: Impact-to-Puncture, Chromatic Blade, Reactive Storm (composition; no magnitude slot).
    ConvertDamage(ConvertDamageParams),
    /// 04 E-4.15, 08 E-8.12: Melee Duplicate (`on = { crit_tier_eq = 1 }`). Magnitude: the repeat chance.
    RepeatHit(RepeatHitParams),
    /// 04 §5.4: Cascadia Empowered. Magnitude: the flat amount per status.
    FlatDamageOnProc(FlatDamageOnProcParams),
    /// 04 §6.5: Primary Debilitate (no magnitude slot).
    ProcAfterTicks(ProcAfterTicksParams),
    /// 04 §5.4, §5.12, 07 §7.5: Secondary Enervate, Melee Animosity, Power Throw. Magnitude: the bonus per count.
    CounterBonus(CounterBonusParams),
    /// 04 §3.2: Devouring Attrition. Magnitude: the chance.
    RandomMultiplier(RandomMultiplierParams),
    /// 06 §9, D-074: Nightwatch Napalm, Volatile Quick Return. Magnitude: the spawn chance (1 = always).
    SpawnAttack(SpawnAttackParams),
}

impl Mechanic {
    /// The kind of the mechanic (the readable `mechanic = "…"` name).
    pub fn kind(&self) -> MechanicKind {
        match self {
            Mechanic::ExtraHit(_) => MechanicKind::ExtraHit,
            Mechanic::ProcOnEvent(_) => MechanicKind::ProcOnEvent,
            Mechanic::ConvertDamage(_) => MechanicKind::ConvertDamage,
            Mechanic::RepeatHit(_) => MechanicKind::RepeatHit,
            Mechanic::FlatDamageOnProc(_) => MechanicKind::FlatDamageOnProc,
            Mechanic::ProcAfterTicks(_) => MechanicKind::ProcAfterTicks,
            Mechanic::CounterBonus(_) => MechanicKind::CounterBonus,
            Mechanic::RandomMultiplier(_) => MechanicKind::RandomMultiplier,
            Mechanic::SpawnAttack(_) => MechanicKind::SpawnAttack,
        }
    }
}

/// Parameters of `extra_hit` (E-14).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExtraHitParams {
    /// Damage type of the extra hit (e_xh).
    pub element: DamageType,
    /// E-14 `(Xata ? Part : 1)`.
    pub part: XhPart,
    /// How the extra hit procs.
    pub status: XhStatus,
    /// Factor on melee weapons (Toxic Lash 2.0).
    pub melee_factor: f64,
    /// Status duration added to the procs of the extra hit (Saryn passive, D-037).
    pub status_duration_add: f64,
    /// Events that cause an extra hit.
    #[cfg_attr(feature = "ts", ts(as = "Vec<XhTrigger>"))]
    pub triggers: SmallVec<[XhTrigger; 2]>,
}

/// Body-part factor of an extra hit.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum XhPart {
    #[default]
    Once,
    Twice,
}

/// Status behaviour of an extra hit.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum XhStatus {
    #[default]
    RollWeaponSc,
    Forced,
    None,
}

/// What creates an extra hit.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum XhTrigger {
    PelletHit,
    BlastDetonation,
}

/// Parameters of `proc_on_event` (02 §4.7.3-4.7.5).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProcOnEventParams {
    /// The event.
    pub on: ProcEvent,
    /// With `on = proc`: the status whose proc triggers (Internal Bleeding: IMP).
    pub on_status: Option<StatusType>,
    /// The proc applied.
    pub proc: ProcSel,
    /// Procs per event (default 1).
    pub count: u8,
    /// Chance factors (Internal Bleeding ×2 below 2.5 shots/s).
    #[cfg_attr(feature = "ts", ts(as = "Vec<ChanceIf>"))]
    pub chance_if: SmallVec<[ChanceIf; 1]>,
    /// At most one proc per instance among the members of a group, `P = 1 − Π(1 − p_i)` (`F:addmul` §8).
    pub exclusive_group: Option<Box<str>>,
}

/// Event of `proc_on_event`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ProcEvent {
    CritInstance,
    Tier1Crit,
    Proc,
    Hit,
}

/// The proc a mechanic applies: a status, a random status of a set, or a component of the combined status that
/// triggered (TOML `proc = "SLA"`, `proc = { random = ["HEA", "COL"] }`, `proc = "component_of_combined"`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ProcSel {
    Status(StatusType),
    Random(StatusSet),
    ComponentOfCombined,
}

/// Readable form of [`ProcSel`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ProcSelRepr {
    Name(String),
    /// One status drawn uniformly from the set.
    Random {
        random: StatusSet,
    },
}

impl From<&ProcSel> for ProcSelRepr {
    fn from(p: &ProcSel) -> Self {
        match p {
            ProcSel::Status(s) => ProcSelRepr::Name(s.name().to_owned()),
            ProcSel::Random(set) => ProcSelRepr::Random { random: *set },
            ProcSel::ComponentOfCombined => ProcSelRepr::Name("component_of_combined".to_owned()),
        }
    }
}

impl TryFrom<ProcSelRepr> for ProcSel {
    type Error = String;
    fn try_from(r: ProcSelRepr) -> Result<Self, String> {
        match r {
            ProcSelRepr::Name(n) if n == "component_of_combined" => {
                Ok(ProcSel::ComponentOfCombined)
            }
            ProcSelRepr::Name(n) => n.parse().map(ProcSel::Status).map_err(|e| e.to_string()),
            ProcSelRepr::Random { random } => Ok(ProcSel::Random(random)),
        }
    }
}

/// `{ fire_rate_below = 2.5, factor = 2.0 }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChanceIf {
    /// Applies when the attack's fire rate is below this (1/s).
    pub fire_rate_below: f64,
    /// Factor on the chance.
    pub factor: f64,
}

/// Parameters of `convert_damage` (01 §4.4, D-009).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConvertDamageParams {
    /// Types converted.
    pub from: ConvFrom,
    /// Target type.
    pub to: ElemSel,
    /// Fraction converted (default 1).
    pub fraction: f64,
}

/// Source types of a conversion (TOML `from = "all"` or a list of type IDs).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ConvFrom {
    All,
    Types(DamageTypeSet),
}

/// Readable form of [`ConvFrom`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ConvFromRepr {
    Keyword(String),
    Types(DamageTypeSet),
}

impl From<&ConvFrom> for ConvFromRepr {
    fn from(c: &ConvFrom) -> Self {
        match c {
            ConvFrom::All => ConvFromRepr::Keyword("all".to_owned()),
            ConvFrom::Types(t) => ConvFromRepr::Types(*t),
        }
    }
}

impl TryFrom<ConvFromRepr> for ConvFrom {
    type Error = String;
    fn try_from(r: ConvFromRepr) -> Result<Self, String> {
        match r {
            ConvFromRepr::Keyword(k) if k == "all" => Ok(ConvFrom::All),
            ConvFromRepr::Keyword(k) => Err(format!(
                "`from` must be \"all\" or a list of types, found \"{k}\""
            )),
            ConvFromRepr::Types(t) => Ok(ConvFrom::Types(t)),
        }
    }
}

/// Crit-tier condition (`{ crit_tier_eq = 1 }`, `{ crit_tier_at_least = 2 }`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TierCond {
    CritTierEq(u8),
    CritTierAtLeast(u8),
}

/// Parameters of `repeat_hit` (04 E-4.15).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RepeatHitParams {
    /// Crit tiers that repeat the hit.
    pub on: TierCond,
}

/// Parameters of `flat_damage_on_proc` (04 §5.4).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FlatDamageOnProcParams {
    /// Procs that add the damage.
    pub statuses: StatusSet,
    /// Scaling of the amount.
    pub scaling: FlatScaling,
}

/// Scaling of a flat amount.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FlatScaling {
    #[default]
    None,
    Strength,
}

/// Parameters of `proc_after_ticks` (04 §6.5).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProcAfterTicksParams {
    /// The combined status whose ticks are counted.
    pub combined: StatusType,
    /// Ticks of the combined status before the proc.
    pub ticks: u8,
    /// What is added.
    pub adds: TickAdds,
}

/// What `proc_after_ticks` adds.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TickAdds {
    /// A proc of one component of the combined status.
    #[default]
    Component,
}

/// Parameters of `counter_bonus` (04 §5.4, §5.12, 07 §7.5).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CounterBonusParams {
    /// The stat the counter raises by `value` per count.
    pub stat: StatKey,
    /// The event that raises the counter by one.
    pub per_event: CounterEvent,
    /// Counter cap at maximum rank; 0 with `unbounded`.
    pub max: u16,
    /// No cap until the reset (04 §4.2 `max_stacks = none`: Secondary Enervate, Hata-Satya); readable form
    /// `unbounded = true` instead of `max` (ruling C-4, CONTRACTS.md T01d).
    #[serde(default)]
    pub unbounded: bool,
    /// The event that resets the counter; `None` = never reset.
    pub reset_on: Option<CounterReset>,
    /// Reset after this many reset events at maximum rank (Secondary Enervate: `r + 1` big crits, 6 at rank 5).
    pub reset_after: Option<u16>,
    /// The instances that read the counter (Melee Animosity: heavies only).
    pub applies_to: Option<Box<Scope>>,
    /// What one hit is for `per_event = "hit"`.
    pub hit_unit: HitUnit,
    /// Per-rank rule of `max` (09 E-9.13); `None` = the same at every rank. Addition (CONTRACTS.md, T01a fixes).
    pub max_rank: Option<RankCurve>,
    /// Per-rank rule of `reset_after` (09 X-RANK: Secondary Enervate `r + 1` = `"linear"` from 6 at R = 5);
    /// `None` = the same at every rank. Addition (CONTRACTS.md, T01a fixes).
    pub reset_after_rank: Option<RankCurve>,
}

/// Event that raises a counter (TOML `"hit"`, `{ input = "light" }`, `"throw"`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CounterEvent {
    Hit,
    Input(InputKind),
    Throw,
}

/// Event that resets a counter (TOML `{ crit_tier_at_least = 2 }`, `"heavy"`, `"other_input"`, `"reload"`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CounterReset {
    CritTierAtLeast(u8),
    Heavy,
    OtherInput,
    /// The weapon reloads (04 §5.3 Hata-Satya `reset_on = (on_reload, 1)`; holstering is not modelled). Addition,
    /// CONTRACTS.md T01d.
    Reload,
}

/// Parameters of `random_multiplier` (04 §3.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RandomMultiplierParams {
    /// Damage factor when the chance hits.
    pub factor: f64,
    /// Crit tiers eligible.
    pub on: TierCond,
}

/// Parameters of `spawn_attack` (06 §9, D-074).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SpawnAttackParams {
    /// Data of the spawned attack.
    pub template: SpawnTemplate,
    /// Share of the parent's damage the spawned instance deals when `template.base` is empty (Volatile Quick Return
    /// 0.5 of the heavy explosion); default 1.
    pub fraction: f64,
    /// Event that spawns it.
    pub on: SpawnEvent,
}

/// Event that spawns the attack.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SpawnEvent {
    /// Every hit of the attack in scope.
    Hit,
    /// The detonation of an explosive projectile (Nightwatch Napalm on the explosion attack).
    Detonation,
    /// A glaive bounce (Volatile Quick Return).
    Bounce,
}

/// Attack data of a spawned attack. An empty `base` means "the parent's per-type damage × `fraction`".
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SpawnTemplate {
    /// Base damage per type (Napalm: 150 Heat per tick).
    #[serde(default)]
    pub base: std::collections::BTreeMap<DamageType, f64>,
    /// Critical chance (fraction).
    #[serde(default)]
    pub cc: f64,
    /// Critical multiplier (factor).
    #[serde(default)]
    pub cd: f64,
    /// Status chance (fraction).
    #[serde(default)]
    pub sc: f64,
    /// Whether the spawned attack can crit.
    #[serde(default)]
    pub can_crit: bool,
    /// Whether it is a radial attack.
    #[serde(default)]
    pub radial: bool,
    /// Forced procs.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<StatusType>"))]
    pub forced: SmallVec<[StatusType; 2]>,
    /// Status duration of the forced procs (Napalm Heat procs 3 s), when fixed.
    #[serde(default)]
    pub proc_duration: Option<f64>,
    /// Lingering cloud (06 §9.4).
    #[serde(default)]
    pub lingering: Option<Lingering>,
}

// ---------------------------------------------------------------------------------------------------------------
// Readable params
// ---------------------------------------------------------------------------------------------------------------

/// Union of every kind's parameters: the readable `[effect.params]` table. [`Mechanic::from_params`] checks that
/// only the fields of the kind are present and fills defaults.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MechanicParamsRepr {
    // extra_hit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<DamageType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<XhPart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<XhStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub melee_factor: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_duration_add: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggers: Option<Vec<XhTrigger>>,
    // proc_on_event, repeat_hit, random_multiplier, spawn_attack
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on: Option<MechOn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_status: Option<StatusType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proc: Option<ProcSelRepr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance_if: Option<Vec<ChanceIf>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive_group: Option<Box<str>>,
    // convert_damage
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<ConvFromRepr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<ElemSel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fraction: Option<f64>,
    // flat_damage_on_proc
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statuses: Option<StatusSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scaling: Option<FlatScaling>,
    // proc_after_ticks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combined: Option<StatusType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticks: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adds: Option<TickAdds>,
    // counter_bonus
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stat: Option<StatKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_event: Option<CounterEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unbounded: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_on: Option<CounterReset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_after: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applies_to: Option<Scope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_unit: Option<HitUnit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_rank: Option<RankCurve>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_after_rank: Option<RankCurve>,
    // random_multiplier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factor: Option<f64>,
    // spawn_attack
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<SpawnTemplate>,
}

/// The `on` parameter of the kinds that have one (readable form; checked per kind).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MechOn {
    CritInstance,
    Tier1Crit,
    Proc,
    Hit,
    Detonation,
    Bounce,
    CritTierEq(u8),
    CritTierAtLeast(u8),
}

macro_rules! take {
    ($p:ident, $kind:literal, allowed: [$($f:ident),*]) => {{
        let mut present: Vec<&'static str> = Vec::new();
        if $p.element.is_some() { present.push("element"); }
        if $p.part.is_some() { present.push("part"); }
        if $p.status.is_some() { present.push("status"); }
        if $p.melee_factor.is_some() { present.push("melee_factor"); }
        if $p.status_duration_add.is_some() { present.push("status_duration_add"); }
        if $p.triggers.is_some() { present.push("triggers"); }
        if $p.on.is_some() { present.push("on"); }
        if $p.on_status.is_some() { present.push("on_status"); }
        if $p.proc.is_some() { present.push("proc"); }
        if $p.count.is_some() { present.push("count"); }
        if $p.chance_if.is_some() { present.push("chance_if"); }
        if $p.exclusive_group.is_some() { present.push("exclusive_group"); }
        if $p.from.is_some() { present.push("from"); }
        if $p.to.is_some() { present.push("to"); }
        if $p.fraction.is_some() { present.push("fraction"); }
        if $p.statuses.is_some() { present.push("statuses"); }
        if $p.scaling.is_some() { present.push("scaling"); }
        if $p.combined.is_some() { present.push("combined"); }
        if $p.ticks.is_some() { present.push("ticks"); }
        if $p.adds.is_some() { present.push("adds"); }
        if $p.stat.is_some() { present.push("stat"); }
        if $p.per_event.is_some() { present.push("per_event"); }
        if $p.max.is_some() { present.push("max"); }
        if $p.unbounded.is_some() { present.push("unbounded"); }
        if $p.reset_on.is_some() { present.push("reset_on"); }
        if $p.reset_after.is_some() { present.push("reset_after"); }
        if $p.applies_to.is_some() { present.push("applies_to"); }
        if $p.hit_unit.is_some() { present.push("hit_unit"); }
        if $p.max_rank.is_some() { present.push("max_rank"); }
        if $p.reset_after_rank.is_some() { present.push("reset_after_rank"); }
        if $p.factor.is_some() { present.push("factor"); }
        if $p.template.is_some() { present.push("template"); }
        let allowed: &[&str] = &[$(stringify!($f)),*];
        if let Some(bad) = present.iter().find(|f| !allowed.contains(f)) {
            return Err(format!("parameter `{}` is not a parameter of mechanic `{}` (allowed: {:?})", bad, $kind, allowed));
        }
    }};
}

fn need<T>(v: Option<T>, kind: &str, field: &str) -> Result<T, String> {
    v.ok_or_else(|| format!("mechanic `{kind}` needs parameter `{field}`"))
}

fn tier_cond(on: Option<MechOn>, kind: &str) -> Result<TierCond, String> {
    match need(on, kind, "on")? {
        MechOn::CritTierEq(k) => Ok(TierCond::CritTierEq(k)),
        MechOn::CritTierAtLeast(k) => Ok(TierCond::CritTierAtLeast(k)),
        other => Err(format!(
            "mechanic `{kind}`: `on` must be a crit-tier condition, found {other:?}"
        )),
    }
}

impl Mechanic {
    /// Builds a mechanic from its readable params, rejecting fields of other kinds and filling defaults.
    pub fn from_params(kind: MechanicKind, p: MechanicParamsRepr) -> Result<Mechanic, String> {
        Ok(match kind {
            MechanicKind::ExtraHit => {
                take!(p, "extra_hit", allowed: [element, part, status, melee_factor, status_duration_add, triggers]);
                Mechanic::ExtraHit(ExtraHitParams {
                    element: need(p.element, "extra_hit", "element")?,
                    part: p.part.unwrap_or_default(),
                    status: p.status.unwrap_or_default(),
                    melee_factor: p.melee_factor.unwrap_or(1.0),
                    status_duration_add: p.status_duration_add.unwrap_or(0.0),
                    triggers: match p.triggers {
                        Some(t) => t.into_iter().collect(),
                        None => SmallVec::from_slice(&[XhTrigger::PelletHit]),
                    },
                })
            }
            MechanicKind::ProcOnEvent => {
                take!(p, "proc_on_event", allowed: [on, on_status, proc, count, chance_if, exclusive_group]);
                let on = match need(p.on, "proc_on_event", "on")? {
                    MechOn::CritInstance => ProcEvent::CritInstance,
                    MechOn::Tier1Crit => ProcEvent::Tier1Crit,
                    MechOn::Proc => ProcEvent::Proc,
                    MechOn::Hit => ProcEvent::Hit,
                    other => {
                        return Err(format!(
                            "mechanic `proc_on_event`: unsupported `on` {other:?}"
                        ));
                    }
                };
                if (on == ProcEvent::Proc) != p.on_status.is_some() {
                    return Err("mechanic `proc_on_event`: `on_status` is required with `on = \"proc\"` only".into());
                }
                Mechanic::ProcOnEvent(ProcOnEventParams {
                    on,
                    on_status: p.on_status,
                    proc: ProcSel::try_from(need(p.proc, "proc_on_event", "proc")?)?,
                    count: p.count.unwrap_or(1),
                    chance_if: p.chance_if.unwrap_or_default().into_iter().collect(),
                    exclusive_group: p.exclusive_group,
                })
            }
            MechanicKind::ConvertDamage => {
                take!(p, "convert_damage", allowed: [from, to, fraction]);
                Mechanic::ConvertDamage(ConvertDamageParams {
                    from: ConvFrom::try_from(need(p.from, "convert_damage", "from")?)?,
                    to: need(p.to, "convert_damage", "to")?,
                    fraction: p.fraction.unwrap_or(1.0),
                })
            }
            MechanicKind::RepeatHit => {
                take!(p, "repeat_hit", allowed: [on]);
                Mechanic::RepeatHit(RepeatHitParams {
                    on: tier_cond(p.on, "repeat_hit")?,
                })
            }
            MechanicKind::FlatDamageOnProc => {
                take!(p, "flat_damage_on_proc", allowed: [statuses, scaling]);
                Mechanic::FlatDamageOnProc(FlatDamageOnProcParams {
                    statuses: need(p.statuses, "flat_damage_on_proc", "statuses")?,
                    scaling: p.scaling.unwrap_or_default(),
                })
            }
            MechanicKind::ProcAfterTicks => {
                take!(p, "proc_after_ticks", allowed: [combined, ticks, adds]);
                Mechanic::ProcAfterTicks(ProcAfterTicksParams {
                    combined: need(p.combined, "proc_after_ticks", "combined")?,
                    ticks: need(p.ticks, "proc_after_ticks", "ticks")?,
                    adds: p.adds.unwrap_or_default(),
                })
            }
            MechanicKind::CounterBonus => {
                take!(p, "counter_bonus", allowed: [stat, per_event, max, unbounded, reset_on, reset_after, applies_to, hit_unit, max_rank, reset_after_rank]);
                if p.reset_after_rank.is_some() && p.reset_after.is_none() {
                    return Err(
                        "mechanic `counter_bonus`: `reset_after_rank` needs `reset_after`".into(),
                    );
                }
                let unbounded = p.unbounded.unwrap_or(false);
                let max = match (unbounded, p.max) {
                    (true, None) if p.max_rank.is_none() && p.reset_on.is_some() => 0,
                    (true, _) => {
                        return Err("mechanic `counter_bonus`: `unbounded = true` takes no `max` or `max_rank` \
                                    and needs `reset_on` (04 §4.2: unbounded until reset)"
                            .into());
                    }
                    (false, max) => need(max, "counter_bonus", "max")?,
                };
                Mechanic::CounterBonus(CounterBonusParams {
                    stat: need(p.stat, "counter_bonus", "stat")?,
                    per_event: need(p.per_event, "counter_bonus", "per_event")?,
                    max,
                    unbounded,
                    reset_on: p.reset_on,
                    reset_after: p.reset_after,
                    applies_to: p.applies_to.map(Box::new),
                    hit_unit: p.hit_unit.unwrap_or_default(),
                    max_rank: p.max_rank,
                    reset_after_rank: p.reset_after_rank,
                })
            }
            MechanicKind::RandomMultiplier => {
                take!(p, "random_multiplier", allowed: [factor, on]);
                Mechanic::RandomMultiplier(RandomMultiplierParams {
                    factor: need(p.factor, "random_multiplier", "factor")?,
                    on: tier_cond(p.on, "random_multiplier")?,
                })
            }
            MechanicKind::SpawnAttack => {
                take!(p, "spawn_attack", allowed: [template, fraction, on]);
                let on = match p.on {
                    None | Some(MechOn::Hit) => SpawnEvent::Hit,
                    Some(MechOn::Detonation) => SpawnEvent::Detonation,
                    Some(MechOn::Bounce) => SpawnEvent::Bounce,
                    Some(other) => {
                        return Err(format!(
                            "mechanic `spawn_attack`: unsupported `on` {other:?}"
                        ));
                    }
                };
                Mechanic::SpawnAttack(SpawnAttackParams {
                    template: need(p.template, "spawn_attack", "template")?,
                    fraction: p.fraction.unwrap_or(1.0),
                    on,
                })
            }
        })
    }
}

impl From<&Mechanic> for MechanicParamsRepr {
    fn from(m: &Mechanic) -> Self {
        let d = MechanicParamsRepr::default();
        let tier = |t: TierCond| match t {
            TierCond::CritTierEq(k) => MechOn::CritTierEq(k),
            TierCond::CritTierAtLeast(k) => MechOn::CritTierAtLeast(k),
        };
        match m {
            Mechanic::ExtraHit(x) => MechanicParamsRepr {
                element: Some(x.element),
                part: Some(x.part),
                status: Some(x.status),
                melee_factor: Some(x.melee_factor),
                status_duration_add: Some(x.status_duration_add),
                triggers: Some(x.triggers.to_vec()),
                ..d
            },
            Mechanic::ProcOnEvent(x) => MechanicParamsRepr {
                on: Some(match x.on {
                    ProcEvent::CritInstance => MechOn::CritInstance,
                    ProcEvent::Tier1Crit => MechOn::Tier1Crit,
                    ProcEvent::Proc => MechOn::Proc,
                    ProcEvent::Hit => MechOn::Hit,
                }),
                on_status: x.on_status,
                proc: Some(ProcSelRepr::from(&x.proc)),
                count: Some(x.count),
                chance_if: (!x.chance_if.is_empty()).then(|| x.chance_if.to_vec()),
                exclusive_group: x.exclusive_group.clone(),
                ..d
            },
            Mechanic::ConvertDamage(x) => MechanicParamsRepr {
                from: Some(ConvFromRepr::from(&x.from)),
                to: Some(x.to.clone()),
                fraction: Some(x.fraction),
                ..d
            },
            Mechanic::RepeatHit(x) => MechanicParamsRepr {
                on: Some(tier(x.on)),
                ..d
            },
            Mechanic::FlatDamageOnProc(x) => MechanicParamsRepr {
                statuses: Some(x.statuses),
                scaling: Some(x.scaling),
                ..d
            },
            Mechanic::ProcAfterTicks(x) => MechanicParamsRepr {
                combined: Some(x.combined),
                ticks: Some(x.ticks),
                adds: Some(x.adds),
                ..d
            },
            Mechanic::CounterBonus(x) => MechanicParamsRepr {
                stat: Some(x.stat),
                per_event: Some(x.per_event),
                max: (!x.unbounded).then_some(x.max),
                unbounded: x.unbounded.then_some(true),
                reset_on: x.reset_on,
                reset_after: x.reset_after,
                applies_to: x.applies_to.as_deref().cloned(),
                hit_unit: Some(x.hit_unit),
                max_rank: x.max_rank.clone(),
                reset_after_rank: x.reset_after_rank.clone(),
                ..d
            },
            Mechanic::RandomMultiplier(x) => MechanicParamsRepr {
                factor: Some(x.factor),
                on: Some(tier(x.on)),
                ..d
            },
            Mechanic::SpawnAttack(x) => MechanicParamsRepr {
                template: Some(x.template.clone()),
                fraction: Some(x.fraction),
                on: Some(match x.on {
                    SpawnEvent::Hit => MechOn::Hit,
                    SpawnEvent::Detonation => MechOn::Detonation,
                    SpawnEvent::Bounce => MechOn::Bounce,
                }),
                ..d
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_checked_per_kind() {
        let p: MechanicParamsRepr = toml::from_str(
            "on = \"crit_instance\"\nproc = \"SLA\"\nexclusive_group = \"forced_slash\"",
        )
        .unwrap();
        let m = Mechanic::from_params(MechanicKind::ProcOnEvent, p.clone()).unwrap();
        assert!(
            matches!(&m, Mechanic::ProcOnEvent(x) if x.proc == ProcSel::Status(StatusType::SLA) && x.count == 1)
        );
        assert!(Mechanic::from_params(MechanicKind::RepeatHit, p).is_err());
        let back = MechanicParamsRepr::from(&m);
        assert_eq!(
            Mechanic::from_params(MechanicKind::ProcOnEvent, back).unwrap(),
            m
        );
        let ib: MechanicParamsRepr = toml::from_str(
            "on = \"proc\"\non_status = \"IMP\"\nproc = \"SLA\"\nchance_if = [{ fire_rate_below = 2.5, factor = 2.0 }]",
        )
        .unwrap();
        assert!(Mechanic::from_params(MechanicKind::ProcOnEvent, ib).is_ok());
        let missing: MechanicParamsRepr = toml::from_str("on = \"proc\"\nproc = \"SLA\"").unwrap();
        assert!(Mechanic::from_params(MechanicKind::ProcOnEvent, missing).is_err());
    }
}
