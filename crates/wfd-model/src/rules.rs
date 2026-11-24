//! Rules tables (domain-model.md §7) and multiplier groups (§14): typed data the engine reads instead of literals.
//! Sources: tables ingested from the spec (pipeline.md §5.2) and curated `data/curated/rules/*.toml`.
//!
//! `spec_constants.rs` in `wfd-engine` holds only formula constants that the spec defines inside formulas; everything
//! with a game value is here (I-1). `game_constants` holds named game values that no dedicated table carries
//! (Steel Path ×2.5, Eximus steps …), keyed `"<equation id>.<name>"` (addition, CONTRACTS.md T01a).

use crate::build::SlotId;
use crate::effect::VulnMask;
use crate::enemy::Faction;
use crate::enums::{AttackRoles, DamageType, N_ST, StatusType, WeaponSlot, WeaponTraits};
use crate::ids::{NameKey, TagId, UniqueName};
use crate::item::RivenTraitDef;
use crate::message::MessageKey;
use crate::weapon::{CoSpec, MeleeRules};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;
use std::fmt;

/// Every rules table of the bundle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Rules {
    /// 03 §4.1 summary table (ingested), indexed by `StatusType::index`.
    pub status: [StatusRule; N_ST],
    /// 05 §5.1 (ingested) + wiki `DamageTypes/data`.
    pub factions: Vec<Faction>,
    /// 05 §6.1 coefficients (ingested).
    pub scaling: Vec<ScalingClass>,
    /// Section 14 (curated).
    pub mult_groups: Vec<MultGroupDef>,
    /// Per equipment class: slot ids, accepted kinds, element order (09; curated).
    pub slot_layouts: Vec<SlotLayout>,
    /// 04 §2.5 default rule.
    pub co_default: CoDefaults,
    /// Defaults and head-crit rules (02 §4.5, 05 §4.3).
    pub body_parts: BodyPartRules,
    /// Attenuation, flat DR, adaptation (05 §4.5, §15).
    pub special_dr: Vec<SpecialDrDef>,
    /// 04 §8 (curated) with PE base values.
    pub riven_traits: Vec<RivenTraitDef>,
    /// 04 §8.2 constants.
    pub riven_formula: RivenFormula,
    /// UI descriptors of T-01..T-29 and proposed T-30..T-32.
    pub toggles: Vec<ToggleDescriptor>,
    /// 07 tables 9-A..9-E and combo specials (`rules/melee/*.toml`).
    pub melee: MeleeRules,
    /// 09 E-9.32: restricted buffs R and Host(frame) (`rules/helminth.toml`).
    pub helminth: HelminthRules,
    /// CategorySet bit → name.
    pub categories: Vec<CategoryDef>,
    pub tags: Vec<TagDef>,
    /// Named game values, `"<equation id>.<name>"` → value (e.g. `"E-5.4.steel_path_mult"` = 2.5).
    pub game_constants: BTreeMap<Box<str>, f64>,
}

/// One row of the 03 §4.1 status summary table.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatusRule {
    pub status: StatusType,
    /// D0 (s); `None` = permanent (MIC) or given by a rule (LIF: M(c)).
    pub duration: Option<f64>,
    /// SD scales the duration.
    pub sd_scales: bool,
    /// Base cap; `None` = unlimited.
    pub cap: Option<u8>,
    pub stacking: StatusStacking,
    /// At cap the oldest stack is replaced.
    pub replace_oldest: bool,
    /// First-tick delay δ_s (s).
    pub first_tick: Option<f64>,
    /// Tick interval (s).
    pub interval: Option<f64>,
    /// DoT coefficient K_s.
    pub k_dot: Option<f64>,
    /// Damage type of a tick (SLA ticks are Cinematic-like: no type modifier, no armor; 05).
    pub tick_type: Option<DamageType>,
    /// Counts in the distinct count n.
    pub counts_in_n: bool,
    /// Debuff-function parameters (`"punc_cc_per_stack"` = 0.05, `"frozen_duration"` = 3 …).
    pub params: BTreeMap<Box<str>, f64>,
    pub source: Box<str>,
}

/// Stacking and timer kind of a status (03 §4.1 "Stacking and replacement").
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum StatusStacking {
    /// Every stack has its own timer.
    Independent,
    /// One stack; a new proc restarts the timer.
    Single,
    /// Heat: one episode per target, procs join a pool (03 §10).
    Pool,
    /// Blast: fuses that detonate at the cap (03 §11).
    Fuse,
    /// Microwave: permanent once applied.
    Permanent,
}

/// A level-scaling class (05 §6.1-6.2): curves for health, shield, armor and overguard.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ScalingClass {
    /// `"grineer"`, `"corpus"`, `"infested"`, `"orokin"`, `"techrot"`, `"default"`.
    pub name: Box<str>,
    pub health: ScalingCurve,
    pub shield: ScalingCurve,
    pub armor: ScalingCurve,
    pub overguard: ScalingCurve,
}

/// `f_i = 1 + C_i·q^E_i`, blended by a smoothstep over [lo, hi] (05 E-5.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ScalingCurve {
    pub c1: f64,
    pub e1: f64,
    pub c2: f64,
    pub e2: f64,
    pub lo: f64,
    pub hi: f64,
    /// `q = L − 1` instead of `q = L − L0` (overguard, 05 §8).
    pub q_from_level_one: bool,
}

/// Where a multiplier group enters (domain-model.md §14).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MultStage {
    /// `mult_other` G (ST-5), multiplies the adding CO term.
    #[serde(rename = "G")]
    G,
    /// `mult_final_omitted` G_F (ST-4), omitted by the adding CO term (E-11, D-058).
    #[serde(rename = "GF")]
    GF,
    /// `M_dot,s` (E-13).
    #[serde(rename = "dot_only")]
    DotOnly,
    /// `M_OG` (E-12).
    #[serde(rename = "overguard_only")]
    OverguardOnly,
    /// `M_son` (E-7), not inherited by DoT.
    #[serde(rename = "spot")]
    Spot,
    /// Multiplicative members of `vuln[ℓ][t]` V (ST-8, 05 E-5.19), masked by the group's `vuln`.
    #[serde(rename = "target_vuln")]
    TargetVuln,
}

/// DoT inheritance of a multiplier group.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DotInheritance {
    /// Part of the DoT seed.
    Inherit,
    /// Seed and M_dot (Mecha, Vauban).
    Twice,
    None,
}

/// A multiplier group (`data/curated/rules/mult_groups.toml`): members add, groups multiply.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MultGroupDef {
    /// `"eclipse"`, `"chamber"`, `"synth_charge"`, `"mecha"`, `"careen"`, `"slam"`, `"finisher"` …
    pub id: Box<str>,
    pub stage: MultStage,
    pub dot: DotInheritance,
    /// Eclipse, Garuda on two-bucket melee projectiles (04 §3.4).
    #[serde(default)]
    pub twice_on_two_bucket: bool,
    /// TargetVuln groups only: types and layers the group multiplies (05 E-5.19). A mask without `layers` is health
    /// only and one without `types` is every type ([`VulnMask`], 05 §13); a group on every layer (custom p.x,
    /// Molecular Prime) lists `layers = ["OG", "SH", "HP"]`.
    #[serde(default)]
    pub vuln: Option<VulnMask>,
    /// Citations.
    pub sources: Vec<Box<str>>,
}

/// Equipment class of 09 E-9.1 and the table of 09 §2.2 (a slot layout per class).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum EquipmentClass {
    Primary,
    Secondary,
    Melee,
    KitgunPrimary,
    KitgunSecondary,
    Zaw,
    ExaltedRanged,
    ExaltedMelee,
    PseudoExaltedMelee,
    Necramech,
    Archgun,
    Archmelee,
    Robotic,
    Warframe,
}

/// Arcane family of 09 E-9.30 (`fam(a)`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ArcaneFamily {
    Primary,
    Secondary,
    Melee,
    Kitgun,
    Zaw,
    Warframe,
}

/// Slot layout of one equipment class (09 §2.2-2.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SlotLayout {
    pub class: EquipmentClass,
    pub slots: Vec<SlotDef>,
    #[serde(default)]
    pub source: Box<str>,
}

/// One slot of a layout.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SlotDef {
    pub id: SlotId,
    /// Content kinds accepted (E-9.3).
    #[cfg_attr(feature = "ts", ts(as = "Vec<SlotKind>"))]
    pub accepts: SmallVec<[SlotKind; 2]>,
    /// Family of an arcane slot (E-9.30).
    #[serde(default)]
    pub arcane_family: Option<ArcaneFamily>,
    /// Position in the element combination order (01 §4.10.1; exilus = 8).
    #[serde(default)]
    pub element_order: Option<u8>,
    /// Fixed stance slot (E-9.2 `stance_fixed`).
    #[serde(default)]
    pub fixed: bool,
}

/// Content kinds of a slot (09 E-9.3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SlotKind {
    General,
    Riven,
    Exilus,
    Stance,
    Aura,
    Arcane,
    IncarnonPerk,
    Shard,
    Subsumed,
}

/// The default CO rule of 04 §2.5 for attacks without a catalogue row (D1 is the catalogue itself).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CoDefaults {
    /// D2: melee slam, heavy slam, glaive heavy explosion, radial melee explosion.
    pub melee_radial: CoSpec,
    /// D3: radial child of a projectile parent.
    pub projectile_radial_child: CoSpec,
    /// D4: radial child of a hitscan parent (ρ_co = B_ref(parent)/B_ref(a), computed by the pipeline).
    pub hitscan_radial_child: CoSpec,
    /// D4a: beam AoE.
    pub beam_aoe: CoSpec,
    /// D5: embedded cloud, directly embedded target.
    pub embedded_cloud: CoSpec,
    /// D5a: non-radial child projectile of a projectile parent.
    pub projectile_child: CoSpec,
    /// D5b: standalone AoE attack and `ShotType = DoT` data attacks.
    pub standalone_aoe: CoSpec,
    /// D6: everything else.
    pub other: CoSpec,
}

/// Body-part defaults and head-crit rules (02 §4.5, 05 §4.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BodyPartRules {
    /// Faction defaults of 05 §4.3 F (first matching row; a row with empty `factions` is the fallback).
    pub faction_heads: Vec<FactionHeadRule>,
    /// k_wp: weak-point damage factor on heads (02 E-7, `ΣWP` × 1.5 on the head).
    pub head_wp_factor: f64,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

/// One faction-default row of 05 §4.3.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FactionHeadRule {
    /// Faction slugs; empty = every other faction.
    pub factions: Vec<Box<str>>,
    pub head_mult: f64,
    pub headcrit: f64,
    #[serde(default)]
    pub source: Box<str>,
}

/// Special damage reduction of 05 §4.5 and §15.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SpecialDrDef {
    pub id: Box<str>,
    /// 05 §15.1.
    #[serde(default)]
    pub attenuation: Option<Attenuation>,
    /// 05 §15.3 (Orphix): (DPS, DR) points.
    #[serde(default)]
    pub dps_dr: Option<Vec<[f64; 2]>>,
    /// 05 §15.4.
    #[serde(default)]
    pub adaptation: Option<Adaptation>,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

/// Attenuation (max damage per instance, 05 §15.1); `p_att = None` = unknown parameter (result flagged).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Attenuation {
    pub p_att: Option<f64>,
    #[serde(default)]
    pub layers: crate::enums::LayerSet,
}

/// Damage adaptation (05 §15.4).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Adaptation {
    pub kind: AdaptationKind,
    /// Resistance per adaptation step.
    pub res: Vec<f64>,
    #[serde(default)]
    pub params: BTreeMap<Box<str>, f64>,
}

/// Adaptation kinds of 05 §15.4.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum AdaptationKind {
    Sentient,
    ShadowStalker,
}

/// Riven value formula constants (04 E-4.18, 09 §12).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RivenFormula {
    /// The factor 10 of E-4.18.
    pub scale: f64,
    /// D = 1.2 since Hotfix 44.1.3.
    pub disposition: f64,
    /// W = quality_min + quality_span·u (0.9 + 0.2·u).
    pub quality_min: f64,
    pub quality_span: f64,
    /// L per layout.
    pub layouts: Vec<RivenLayoutRow>,
    /// 8.
    pub max_rank: u8,
    /// ξ(8) assumed when the drain is not given (09 E-9.59: 18).
    pub default_drain_at_max: i8,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

/// The layout factor L for (positives, negatives).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RivenLayoutRow {
    pub positives: u8,
    pub negatives: u8,
    pub l_pos: f64,
    #[serde(default)]
    pub l_neg: Option<f64>,
}

/// Toggle id (`"T-13"`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "string"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ToggleId(pub Box<str>);

impl ToggleId {
    pub fn new(s: impl Into<Box<str>>) -> Self {
        ToggleId(s.into())
    }
}

impl fmt::Display for ToggleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Group of a toggle in the schema-driven panels (data: `"conditions"`, `"melee"`, `"enemy"` …).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "string"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ToggleGroup(pub Box<str>);

/// UI descriptor of a toggle (`data/curated/rules/toggles.toml`; server-api.md §5.6 `ScenarioSchema.toggles`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ToggleDescriptor {
    /// `"T-13"`.
    pub id: ToggleId,
    /// Path in `ScenarioDoc` (`"toggles.hit_location"`, `"eval.window_s"`); addition for server-api.md §5.6.
    pub field: Box<str>,
    pub group: ToggleGroup,
    pub label: MessageKey,
    pub help: MessageKey,
    pub control: ControlKind,
    /// Default value in its JSON form (compact encoding: JSON text, because postcard is not self-describing).
    #[serde(with = "json_value_codec")]
    #[cfg_attr(feature = "ts", ts(as = "serde_json::Value"))]
    #[cfg_attr(feature = "schema", schemars(with = "serde_json::Value"))]
    pub default: serde_json::Value,
    /// Melee only, sniper only, …
    #[serde(default)]
    pub visible_if: Option<VisibilityRule>,
    #[serde(default)]
    pub advanced: bool,
}

/// Control of a toggle (server-api.md §5.6).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ControlKind {
    pub kind: ControlType,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub options: Vec<ControlOption>,
}

/// Control types.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ControlType {
    Bool,
    Number,
    Percent,
    Select,
    OptionalNumber,
}

/// One option of a select control.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ControlOption {
    pub value: Box<str>,
    pub label: MessageKey,
}

/// When a toggle is shown.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VisibilityRule {
    #[serde(default)]
    pub weapon_slot: Vec<WeaponSlot>,
    #[serde(default)]
    pub attack_role: AttackRoles,
    #[serde(default, rename = "trait")]
    pub traits: WeaponTraits,
}

/// Helminth restriction (09 E-9.32): restricted buffs R and the host of each frame.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HelminthRules {
    /// R = {Eclipse, Roar, Xata's Whisper} as buff uniqueNames.
    pub restricted: Vec<UniqueName>,
    /// Host(frame): frame uniqueName → the frame's native buff that conflicts.
    pub hosts: Vec<HelminthHost>,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

/// One Host(frame) entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HelminthHost {
    pub frame: UniqueName,
    pub ability: UniqueName,
}

/// A category bit of `CategorySet`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CategoryDef {
    /// Bit index 0..64.
    pub bit: u8,
    /// Category name (`"Assault Rifle"`, `"Daggers"` …, the compatibility-tree node of 09 §3.1).
    pub name: Box<str>,
    #[serde(default)]
    pub label: Option<NameKey>,
}

/// A compatibility tag.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TagDef {
    pub id: TagId,
    pub name: Box<str>,
}

/// serde codec for `serde_json::Value` fields of bundle types: the value itself in readable formats, its JSON text
/// in compact formats (postcard cannot carry self-describing values).
pub mod json_value_codec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &serde_json::Value, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            v.serialize(s)
        } else {
            serde_json::to_string(v)
                .map_err(serde::ser::Error::custom)?
                .serialize(s)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<serde_json::Value, D::Error> {
        if d.is_human_readable() {
            serde_json::Value::deserialize(d)
        } else {
            let text = String::deserialize(d)?;
            serde_json::from_str(&text).map_err(serde::de::Error::custom)
        }
    }
}
