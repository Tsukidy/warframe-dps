//! Scenario documents, toggles and conditions (domain-model.md §9.1, §9.3, §9.4), the calculation input, and the
//! resolved scenario (§9.2). Every toggle and eval field has the default of the §9.3 table and is filled when missing
//! (server-api.md §4), so old share codes keep working.

use crate::build::{Build, BuildDoc, Violation};
use crate::compat::CompatFlags;
use crate::enums::{DamageType, StatusType};
use crate::ids::{AttackKey, EffectKey, EnemyId, ItemId, NameKey, ParamKey, StableKey, UniqueName};
use crate::item::ParamValue;
use crate::serde_util::{default_true, flags_serde, is_false, is_true};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// Current document version (`CalcInput.v`, server-api.md §2).
pub const DOC_VERSION: u16 = 1;

/// Build + scenario: the body of `/api/calc` and the content of share codes. `x` carries the uniqueName of every
/// session-local key (≥ 2^31) the document uses (AD-18); usually empty and then omitted.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CalcInput {
    /// Document version ([`DOC_VERSION`]).
    pub v: u16,
    /// The build.
    pub build: BuildDoc,
    /// The scenario.
    pub scenario: ScenarioDoc,
    /// uniqueName of every session-local key (≥ 2^31) used by the document (AD-18).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub x: BTreeMap<StableKey, UniqueName>,
}

/// The scenario part of a calculation input.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ScenarioDoc {
    /// T-23.
    pub enemy: EnemySetup,
    /// Toggles without another place (domain-model.md §9.3).
    #[serde(default)]
    pub toggles: Toggles,
    /// T-13..T-19 and every other buff.
    #[serde(default)]
    pub buffs: Vec<BuffPick>,
    /// T-26: custom stat id (`"base"`, `"crit_chance"` …) → model-unit value. One custom stat is an alias of a
    /// toggle, not an effect: `abilityCombo` (`p.u`, 00 §6.3 "T-11 value"; `CustomStatSchema.toggle` = `"T-11"`).
    /// `GameData::resolve` (T04) moves its value into `toggles.ability_combo` when T-11 is off (`None`) and drops it
    /// when T-11 is set (the toggle wins); the inspiration import (T10) writes T-11 directly. Domain-model.md §9.3;
    /// CONTRACTS.md "Contract review fixes (model-records)", M-18.
    #[serde(default)]
    pub custom: BTreeMap<Box<str>, f64>,
    /// T-02 + per-effect overrides.
    #[serde(default)]
    pub conditions: ConditionPolicy,
    /// T-01, T-29, horizons.
    #[serde(default)]
    pub eval: EvalSettings,
    /// `Game` (default) | `Compat` (oracle harness only; hidden in the UI).
    #[serde(default, skip_serializing_if = "Profile::is_game")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub profile: Profile,
}

impl ScenarioDoc {
    /// A scenario against an enemy with every default.
    pub fn against(enemy: EnemySetup) -> Self {
        ScenarioDoc {
            enemy,
            toggles: Toggles::default(),
            buffs: Vec::new(),
            custom: BTreeMap::new(),
            conditions: ConditionPolicy::default(),
            eval: EvalSettings::default(),
            profile: Profile::Game,
        }
    }
}

/// Enemy options (T-23, spec 00 §6.1). `mission` and `context_overguard` are the optional scenario fields of 05 §7.5
/// and §4.1 (additions, CONTRACTS.md T01a); the fields from `eximus_kind` on are the T-23 options the spec rounds
/// adopted from 05 OQ-523 (CONTRACTS.md T01d). Every optional field is omitted when it has its default.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemySetup {
    /// The enemy.
    pub enemy: StableKey,
    /// Enemy level.
    pub level: u16,
    /// Steel Path scaling.
    #[serde(default)]
    pub steel_path: bool,
    /// Eximus variant.
    #[serde(default)]
    pub eximus: bool,
    /// 1..=4 (1..=6 with `icebind`).
    #[serde(default = "default_party")]
    pub party: u8,
    /// Mission modifier (05 §7.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mission: Option<MissionModifier>,
    /// Use a context-only overguard (Dax Malleus, 05 §4.1).
    #[serde(default, skip_serializing_if = "is_false")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub context_overguard: bool,
    /// Eximus kind when `eximus` is on (05 §4.1 "Eximus kind"; the UI offers the kinds of `EximusData.kinds`).
    #[serde(default, skip_serializing_if = "EximusKind::is_generic")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub eximus_kind: EximusKind,
    /// The target was damaged by Hydroid: the first Corrosive stack strips 50 % (05 E-5.12, `armor_hydroid`).
    #[serde(default, skip_serializing_if = "is_false")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub armor_hydroid: bool,
    /// Corrosive Projection auras worn by squadmates, Σ_CP 0.18·aura_str of 05 E-5.10 (the build's own aura slots
    /// count through the build).
    #[serde(default, skip_serializing_if = "SquadAuras::is_none")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub squad_cp: SquadAuras,
    /// Shield Disruption auras worn by squadmates, the aura term of 05 E-5.16 (capped at 0.80).
    #[serde(default, skip_serializing_if = "SquadAuras::is_none")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub squad_shield_disruption: SquadAuras,
    /// External ability shield strips, each the final fraction s_j·Str_j of 05 E-5.16 (applied at fight start).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub shield_strips: Vec<f64>,
    /// Flat base-armor strip rate r_flat (base-armor points per second from sources outside the evaluated attacks,
    /// 05 E-5.11); default 0.
    #[serde(default, skip_serializing_if = "is_zero")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub flat_strip_rate: f64,
    /// The target stands inside a Guardian Eximus aura (05 E-5.27, `aura_guardian`).
    #[serde(default, skip_serializing_if = "is_false")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub aura_guardian: bool,
    /// Prosecutor auras around the target, by element (`HEA`, `COL`, `ELE`, `TOX`; repeats allowed; 05 E-5.27).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub aura_prosecutor: Vec<DamageType>,
    /// For `shield_weapon_immune` enemies: the layered pass starts with SH = 0 (05 E-5.26); default true.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub shield_removed_externally: bool,
    /// The Profit-Taker shield's displayed type `pt_shield_type` (05 E-5.26); `None` = no type damages the shield.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pt_shield_type: Option<DamageType>,
}

fn default_party() -> u8 {
    1
}

fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

impl EnemySetup {
    /// A level-`level` enemy with every other option at its default.
    pub fn new(enemy: StableKey, level: u16) -> Self {
        EnemySetup {
            enemy,
            level,
            steel_path: false,
            eximus: false,
            party: 1,
            mission: None,
            context_overguard: false,
            eximus_kind: EximusKind::Generic,
            armor_hydroid: false,
            squad_cp: SquadAuras::default(),
            squad_shield_disruption: SquadAuras::default(),
            shield_strips: Vec::new(),
            flat_strip_rate: 0.0,
            aura_guardian: false,
            aura_prosecutor: Vec::new(),
            shield_removed_externally: true,
            pt_shield_type: None,
        }
    }
}

/// Eximus kind (05 §4.1 "Eximus kind", §7.6; T-23). The Entropic and Guardian rows of 05 §4.5, §7 and §13.1 apply to
/// an Eximus of that kind; `generic` is every other Eximus.
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum EximusKind {
    /// Every Eximus that is neither Entropic nor Guardian (the default, 05 §4.1).
    #[default]
    Generic,
    /// Entropic Eximus: 0.90 DR against non-melee damage, Eximus base armor 50, ability debuffs blocked while its orb
    /// is active (05 §4.5 row, EC-05-20).
    Entropic,
    /// Guardian Eximus: 0.90 DR against radial damage on itself (05 §4.5 row); its aura on allies is the T-23 option
    /// `aura_guardian` (05 E-5.27).
    Guardian,
}

impl EximusKind {
    /// `true` for the default kind (the document then omits `eximus_kind`).
    pub fn is_generic(&self) -> bool {
        *self == EximusKind::Generic
    }
}

/// A squad aura count (05 E-5.10, E-5.16): `count` auras worn by squadmates, `coaction_drift` of them by wearers with
/// Coaction Drift (aura strength 1.15 instead of 1, 04 E-4.11). JSON `{ "count": 2, "coaction_drift": 1 }`.
///
/// Squadmates' Corrosive Projection and Shield Disruption enter only through `EnemySetup.squad_cp` and
/// `.squad_shield_disruption`, never as buff items: a class (u) squad-aura buff (04 §4.1, §5.8) of either aura would
/// count it twice, and the validator (T19) rejects one. Domain-model.md §9.1; CONTRACTS.md "Contract review fixes
/// (model-records)", M-19.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SquadAuras {
    /// Auras worn by squadmates (0 ..= party − 1; 5 with The Icebind's six players).
    pub count: u8,
    /// How many of them are worn with Coaction Drift (≤ `count`).
    #[serde(default)]
    pub coaction_drift: u8,
}

impl SquadAuras {
    /// `true` when no aura is worn (the document then omits the field).
    pub fn is_none(&self) -> bool {
        self.count == 0
    }
}

/// Mission modifiers (05 §7.5; proposed in OQ-523).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MissionModifier {
    /// Deep/Temporal Archimedea.
    Empowered,
    ArchonHunt,
    Icebind,
    BolsteredBelligerents,
}

/// A buff in the scenario. T-13..T-17 values are final values: caster Strength is not applied; the caster of T-13
/// still decides the Helminth rule 09 E-9.32 (T-37), and T-14 … T-16 are always self-cast (spec 00 §6.1 T-37).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuffPick {
    /// The buff item.
    pub buff: StableKey,
    /// Parameter values (model units); missing keys take the item's defaults. The weak spots of T-32 are the params
    /// that the buff's `Spot`-stage effect names in its `spot` field (`EffectDef.spot`, 02 E-2.10): by convention
    /// `spot_location` (`"hit_part"` default, `"torso"`) and `spot_nodes` (ν_σ, Sonar default 1; Detect
    /// Vulnerability has none).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub params: BTreeMap<ParamKey, ParamValue>,
    /// Whose Strength and Duration scale the buff (04 E-4.10; T-33, and T-37 for the T-13 buff); `None` = the
    /// buff's `ItemKind::Buff.default_caster` (`ally` for T-13 Roar, spec 00 §6.1 T-37; `self` otherwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub caster: Option<Caster>,
}

/// Whose Strength and Duration scale a buff (04 E-4.10). JSON `"self"` or
/// `{ "ally": { "strength": 1.0, "duration": 1.0 } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Caster {
    /// Str = the build's Strength (T-20 or derived, T-33), Dur_e = T-33's self-cast Ability Duration.
    #[default]
    #[serde(rename = "self")]
    SelfCast,
    Ally {
        /// The ally's Ability Strength (factor, default 1; T-33).
        #[serde(default = "crate::serde_util::default_one_f64")]
        strength: f64,
        /// The ally's Ability Duration Dur_e (factor, default 1; 04 E-4.10; addition, CONTRACTS.md T01d). Always
        /// written (a skipped field would break the compact form of `Caster` in `BuffInstance`); optional on input.
        #[serde(default = "crate::serde_util::default_one_f64")]
        #[cfg_attr(feature = "ts", ts(optional = nullable))]
        duration: f64,
    },
}

impl Caster {
    /// `true` for a self-cast buff.
    pub fn is_self(&self) -> bool {
        *self == Caster::SelfCast
    }
    /// An ally caster with Strength and Duration 1.0 (the T-33 defaults).
    pub fn ally() -> Self {
        Caster::Ally {
            strength: 1.0,
            duration: 1.0,
        }
    }
}

/// Who casts a buff by default (`ItemKind::Buff.default_caster`; 04 §4.2 field `caster`, §4.4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CasterKind {
    /// The build's own frame.
    #[default]
    #[serde(rename = "self")]
    SelfCast,
    /// A squadmate (Strength and Duration 1.0 unless the document says otherwise).
    Ally,
}

impl CasterKind {
    /// The caster a buff pick without `caster` resolves to.
    pub fn caster(self) -> Caster {
        match self {
            CasterKind::SelfCast => Caster::SelfCast,
            CasterKind::Ally => Caster::ally(),
        }
    }
}

/// Evaluation settings: T-01, T-29, horizons.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EvalSettings {
    /// T-01, 1-60 (advanced 600), default 20.
    pub window_s: f64,
    /// t_ttk_max, default 60.
    pub ttk_horizon_s: f64,
    /// T-29, 1-100,000, default 1000.
    pub mc_runs: u32,
    /// T-29, default `Fixed(0)`.
    pub seed: SeedSel,
    /// Default 1.
    pub series_bin_s: f64,
}

impl Default for EvalSettings {
    fn default() -> Self {
        EvalSettings {
            window_s: 20.0,
            ttk_horizon_s: 60.0,
            mc_runs: 1000,
            seed: SeedSel::Fixed(0),
            series_bin_s: 1.0,
        }
    }
}

/// Largest seed that a browser can hold without losing precision: 2^53 − 1.
pub const MAX_SEED: u64 = (1 << 53) - 1;

/// MC seed selection. JSON: `"auto"` or an integer 0 ..= 2^53 − 1 (larger seeds are rejected, so the browser never
/// loses precision). Auto: the frontend draws a fresh seed for every MC request (parity row 25) and sends it in
/// `want.mc.seed`; the document and the share code store only `"auto"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "\"auto\" | number"))]
pub enum SeedSel {
    Auto,
    Fixed(u64),
}

impl Serialize for SeedSel {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            match self {
                SeedSel::Auto => s.serialize_str("auto"),
                SeedSel::Fixed(n) => s.serialize_u64(*n),
            }
        } else {
            match self {
                SeedSel::Auto => None::<u64>.serialize(s),
                SeedSel::Fixed(n) => Some(*n).serialize(s),
            }
        }
    }
}

impl<'de> Deserialize<'de> for SeedSel {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{Error, Visitor};
        struct V;
        impl Visitor<'_> for V {
            type Value = SeedSel;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("\"auto\" or an integer seed 0 ..= 2^53 − 1")
            }
            fn visit_str<E: Error>(self, v: &str) -> Result<SeedSel, E> {
                if v == "auto" {
                    Ok(SeedSel::Auto)
                } else {
                    Err(E::custom(format!("invalid seed \"{v}\"")))
                }
            }
            fn visit_u64<E: Error>(self, v: u64) -> Result<SeedSel, E> {
                if v <= MAX_SEED {
                    Ok(SeedSel::Fixed(v))
                } else {
                    Err(E::custom(format!("seed {v} exceeds 2^53 − 1")))
                }
            }
            fn visit_i64<E: Error>(self, v: i64) -> Result<SeedSel, E> {
                u64::try_from(v)
                    .map_err(|_| E::custom(format!("negative seed {v}")))
                    .and_then(|v| self.visit_u64(v))
            }
            fn visit_u128<E: Error>(self, v: u128) -> Result<SeedSel, E> {
                u64::try_from(v)
                    .map_err(|_| E::custom("seed exceeds 2^53 − 1"))
                    .and_then(|v| self.visit_u64(v))
            }
            fn visit_i128<E: Error>(self, v: i128) -> Result<SeedSel, E> {
                u64::try_from(v)
                    .map_err(|_| E::custom("seed out of range"))
                    .and_then(|v| self.visit_u64(v))
            }
            fn visit_f64<E: Error>(self, v: f64) -> Result<SeedSel, E> {
                Err(E::custom(format!("seed must be an integer, found {v}")))
            }
        }
        if d.is_human_readable() {
            d.deserialize_any(V)
        } else {
            Ok(match Option::<u64>::deserialize(d)? {
                None => SeedSel::Auto,
                Some(n) if n <= MAX_SEED => SeedSel::Fixed(n),
                Some(n) => return Err(D::Error::custom(format!("seed {n} exceeds 2^53 − 1"))),
            })
        }
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for SeedSel {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SeedSel".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "oneOf": [
                { "const": "auto" },
                { "type": "integer", "minimum": 0, "maximum": MAX_SEED }
            ]
        })
    }
}

/// Evaluation profile. JSON `"game"` or `{ "compat": ["D001_QUANTIZATION_GRID", …] }`: flag names, never the u128
/// (`CompatFlags` is not exported to TypeScript).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Profile {
    #[default]
    Game,
    Compat(CompatFlags),
}

impl Profile {
    /// `true` for the default `Game` profile (the document then omits `profile`).
    pub fn is_game(&self) -> bool {
        *self == Profile::Game
    }
    /// The compat flags in effect (empty for `Game`).
    pub fn flags(&self) -> CompatFlags {
        match self {
            Profile::Game => CompatFlags::empty(),
            Profile::Compat(f) => *f,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Toggles (§9.3)
// ---------------------------------------------------------------------------------------------------------------

/// Every toggle that is not already a field elsewhere (placement table of domain-model.md §9.3, [`TOGGLE_MAP`]).
/// Every field is written; an off toggle is `null` (server-api.md §4: `external_viral: number | null`). Missing
/// fields take the defaults (`#[serde(default)]`), so documents written before a field existed keep their meaning;
/// the one exception is `combine_groups`, whose default changed from `[]` to `null` with T01d (a document that omits
/// it now gets the profile default; one that writes `[]`, as every written document did, keeps its meaning). The
/// fields added after T01a (from `capacity` on) are optional in TypeScript, as in server-api.md §4.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Toggles {
    /// T-03.
    pub hit_location: HitLocation,
    /// T-04.
    pub co_multiplicative: bool,
    /// T-05.
    pub co_status_count: CoStatusCount,
    /// T-06: 1..=10. T-43 restricted to Viral, kept for link import (spec 00 §6.1); an `external_statuses` entry
    /// for `VIR` wins over it.
    pub external_viral: Option<u8>,
    /// T-07: 0-0.99.
    pub armor_strip: f64,
    /// T-08.
    pub stealth: Option<Stealth>,
    /// T-09.
    pub heavy: Option<Heavy>,
    /// T-10.
    pub light_combo: Option<u8>,
    /// T-11: 1..=12.
    pub ability_combo: Option<u8>,
    /// T-12: 1.5..=100.
    pub sniper_combo: Option<f64>,
    /// T-20: 0-10 (the total under T-33 `manual`).
    pub strength: f64,
    /// T-21 (hidden).
    pub wp_cc_multiplicative: bool,
    /// T-22: indices into `Weapon.groups` of the families evaluated as one unit; `None` = the profile default
    /// (every family in the game profile, none in the compat profile; spec 00 §6.1, CD-12, 08 §2). `Some([])` =
    /// every attack is its own unit.
    #[cfg_attr(feature = "ts", ts(as = "Option<Vec<u8>>"))]
    pub combine_groups: Option<SmallVec<[u8; 2]>>,
    /// T-27: 0-1000 m.
    pub distance_m: f64,
    /// T-28.
    pub unique_dr: bool,
    /// T-35 capacity mode (09 E-9.18; `ignore` reproduces the inspiration, D-092).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub capacity: CapacityMode,
    /// T-35 allowed Forma types (09 §6.1, E-9.19); default every type. The warframe part of T-35 is
    /// `BuildDoc.warframe.frame_complete`.
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub forma_types: FormaTypes,
    /// T-30 melee plan (07 §8.1): the plan of every melee attack without an entry in `melee_plans`; `Auto` = the
    /// 07 §8.1 default per attack kind, any other plan applies to the attacks its kind names (07 §8.1) and `Auto`
    /// to the rest.
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub melee_plan: MeleePlanSel,
    /// T-30 per attack ("plan per selected melee attack", spec 00 §6.1): wins over `melee_plan` for its attack. At
    /// most one entry per attack: a second entry for the same attack is the document issue `doc.attack_duplicate`
    /// (the document is rejected, like `doc.slot_duplicate`); an entry whose attack the weapon does not have is
    /// dropped with `doc.attack_unknown` (domain-model.md §9.3).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub melee_plans: Vec<AttackPlan>,
    /// T-38 zoom level: index into `Weapon.zoom`; `None` = highest level under `all_max`, none under `none`
    /// (04 D-406).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub zoom_level: Option<u8>,
    /// T-38 aiming (`while_aiming` buffs, sniper scope rules, zoom-only children; 06 §5.9, §7.2, §9.1); default on.
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub aiming: bool,
    /// T-31 exalted ability rank r, 0..=3, default 3 (01 E-1.1).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub exalted_rank: u8,
    /// T-33 Strength sources (04 §4.4): `manual` (T-20 is the total, default) or `derive`.
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub strength_mode: StrengthMode,
    /// T-33 self-cast Ability Duration Dur_e (factor, default 1.0; 04 E-4.10). Ally casters carry their own
    /// (`Caster::Ally`).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub ability_duration: f64,
    /// T-34 finisher mode (04 §7.2, E-4.17); `None` = off (default).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub finisher: Option<FinisherMode>,
    /// T-39 Incarnon switch time counted at form start and end (06 §6.6); default off.
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub incarnon_switch: bool,
    /// T-40 timed reload: non-battery weapons skip the 1/FR reload lead (06 §5.8); default off.
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub timed_reload: bool,
    /// T-41 pellet hit fraction per attack, 0..=1; an attack without an entry uses 1 (06 OQ-619 (b), 08 OQ-808).
    /// Entries follow the rules of `melee_plans` (one per attack, `doc.attack_duplicate`, `doc.attack_unknown`).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub pellet_hit: Vec<AttackFraction>,
    /// T-42 gauge uses per minute of gauge-gated attacks, ≥ 0, default 0 (06 §6.7).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub gauge_uses_per_min: f64,
    /// T-43 external statuses: N_ext,s stacks of s held for the whole window, 1 ..= cap_s (03 §6.3, `ext_stacks`);
    /// a type without an entry is off.
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub external_statuses: BTreeMap<StatusType, u8>,
}

impl Default for Toggles {
    fn default() -> Self {
        Toggles {
            hit_location: HitLocation::Body,
            co_multiplicative: false,
            co_status_count: CoStatusCount::default(),
            external_viral: None,
            armor_strip: 0.0,
            stealth: None,
            heavy: None,
            light_combo: None,
            ability_combo: None,
            sniper_combo: None,
            strength: 1.0,
            wp_cc_multiplicative: false,
            combine_groups: None,
            distance_m: 0.0,
            unique_dr: true,
            capacity: CapacityMode::AnyForma,
            forma_types: FormaTypes::all(),
            melee_plan: MeleePlanSel::Auto,
            melee_plans: Vec::new(),
            zoom_level: None,
            aiming: true,
            exalted_rank: Toggles::DEFAULT_EXALTED_RANK,
            strength_mode: StrengthMode::Manual,
            ability_duration: 1.0,
            finisher: None,
            incarnon_switch: false,
            timed_reload: false,
            pellet_hit: Vec::new(),
            gauge_uses_per_min: 0.0,
            external_statuses: BTreeMap::new(),
        }
    }
}

impl Toggles {
    /// The T-31 default exalted ability rank, 3 (spec 00 §6.1).
    pub const DEFAULT_EXALTED_RANK: u8 = 3;

    /// External stacks of `s` (T-43, with T-06 as T-43 restricted to Viral): the `external_statuses` entry, else
    /// `external_viral` for `VIR`.
    pub fn external_stacks(&self, s: StatusType) -> Option<u8> {
        match self.external_statuses.get(&s) {
            Some(n) => Some(*n),
            None if s == StatusType::VIR => self.external_viral,
            None => None,
        }
    }
}

/// T-03 hit location. JSON `"body"`, `"head"`, `"weak_point"` or `{ "part": "<part name key>" }` (any part of the
/// enemy's 05 §4.3 part list; spec 00 §6.1 T-03 `part(<name>)`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HitLocation {
    #[default]
    Body,
    Head,
    WeakPoint,
    /// A named part (`BodyPart.name`); addition, CONTRACTS.md T01d.
    Part(NameKey),
}

/// A per-attack melee plan (T-30).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackPlan {
    /// The melee attack.
    pub attack: AttackKey,
    /// Its plan.
    pub plan: MeleePlanSel,
}

/// A per-attack fraction (T-41 pellet hit fraction).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackFraction {
    /// The attack.
    pub attack: AttackKey,
    /// Fraction 0..=1.
    pub fraction: f64,
}

/// T-33 Strength mode (04 §4.4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum StrengthMode {
    /// T-20 is the user's total Strength (default).
    #[default]
    Manual,
    /// 1 + the build's Strength sources of 04 §5 (additive), Parasitic Link multiplicative.
    Derive,
}

/// T-34 finisher mode (04 §7.2, E-4.17; 07 §9.11 schedules one finisher of the chosen type per user interval).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FinisherMode {
    /// Stealth finishers: the "stealth" multipliers of the class (07 §9.11 Table 9-E) in 04 E-4.17.
    Stealth,
    /// Back finishers: the "back" multipliers of Table 9-E.
    Back,
    /// Front finishers: the "front" multipliers of Table 9-E.
    Front,
    /// Universal finishers: the "universal" multipliers of Table 9-E.
    Universal,
}

bitflags::bitflags! {
    /// T-35 allowed Forma types (09 §6.1). Readable form: lowercase names (`["forma", "umbra", …]`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FormaTypes: u8 {
        /// Forma: a standard polarity or none on any slot.
        const FORMA = 1 << 0;
        /// Umbra Forma (warframes and melee weapons only).
        const UMBRA = 1 << 1;
        /// Omni Forma: Any on every slot but the stance slot.
        const OMNI = 1 << 2;
        /// Stance Forma: Any on the stance slot.
        const STANCE = 1 << 3;
        /// Aura Forma: Any on the aura slot.
        const AURA = 1 << 4;
    }
}
flags_serde!(FormaTypes, u8, Lower);

impl Default for FormaTypes {
    fn default() -> Self {
        FormaTypes::all()
    }
}

/// T-05 CO status count. JSON `{ "mode": "expected" | "weapon_types" | "manual", "n"?: 1..50 }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CoStatusCount {
    /// How n is counted.
    pub mode: CoCountMode,
    /// N for `manual` (1..50).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
}

/// T-05 mode.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CoCountMode {
    #[default]
    Expected,
    WeaponTypes,
    Manual,
}

/// T-08 stealth (spec 00 §6.1): weapon rank 0 ..= the weapon's maximum rank (30 or 40), default the maximum.
/// JSON `{ "rank": 30 }` or `{}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Stealth {
    /// Weapon rank for the stealth damage bonus; `None` = the weapon's maximum rank (spec 00 §6.1 T-08 "default the
    /// maximum"; domain-model.md §9.3 has a required `0..30`, CONTRACTS.md T01a fixes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,
}

/// T-09 heavy attack and start combo multiplier (spec 00 §6.1: 1 ..= weapon cap, default 12; clamped to M_cap,
/// D-063). JSON `{ "start_combo": 12 }` or `{}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Heavy {
    /// Start combo multiplier of the first heavy attack (default 12).
    #[serde(default = "Heavy::default_start_combo")]
    pub start_combo: u8,
}

impl Heavy {
    /// The T-09 default start multiplier, 12 (spec 00 §6.1).
    pub const DEFAULT_START_COMBO: u8 = 12;

    fn default_start_combo() -> u8 {
        Self::DEFAULT_START_COMBO
    }
}

impl Default for Heavy {
    fn default() -> Self {
        Heavy {
            start_combo: Self::DEFAULT_START_COMBO,
        }
    }
}

/// T-35 capacity mode (09 E-9.18). JSON `"ignore"`, `"current"`, `"any_forma"` or `{ "budget": N }`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CapacityMode {
    Ignore,
    Current,
    #[default]
    AnyForma,
    Budget(u8),
}

/// 07 §8.1 plans with their parameters (T-30). `Auto`: Normal Attack → light_loop(selected combo), heavy
/// (T-09 or selected) → heavy_loop, slide → slide_loop, slam → slam_loop, heavy slam → heavy_slam_loop, throws →
/// glaive_cycle, weapons without light combos → cycle_loop, a directly selected data attack that no trigger, stance
/// entry or other plan produces → data_attack_loop. JSON: `"auto"`, `"light_loop"`,
/// `{ "heavy_loop": { "policy": "full_regen" } }`, `{ "build_spend": { "k": 12 } }` or `{ "build_spend": {} }`,
/// `"tennokai"`, `{ "slam_loop": { "t_slam": 1.0 } }`, `{ "glaive_cycle": { "throw": "charged" } }` …
/// `heavy_cancel`, `CycleLoop` and `DataAttackLoop` are spec 07 §8.1 additions (CONTRACTS.md, T01a).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MeleePlanSel {
    #[default]
    Auto,
    /// Combo = `BuildDoc.stance_combo` (default neutral).
    LightLoop,
    SlideLoop,
    HeavyLoop {
        /// Heavy loop policy.
        #[serde(default)]
        policy: HeavyPolicy,
        /// `heavy_cancel` with the user-set t_dmg (s); `None` = off (07 E-7.30, OQ-701).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "ts", ts(optional))]
        heavy_cancel: Option<f64>,
    },
    /// Light loop until M ≥ k, then one heavy (07 §8.1 `build_spend(M_trig)`).
    BuildSpend {
        /// Combo multiplier M_trig at which the heavy is spent; `None` = the weapon's M_cap (07 §8.1 default "M_trig
        /// = M_cap (12)", E-7.10: 13 for Venka Prime, 11 for Dex Nikana). An `Option` since CONTRACTS.md "Contract
        /// review fixes (model-records)", M-14 (approved breaking change).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "ts", ts(optional))]
        k: Option<f64>,
    },
    /// n_tk from E-7.17.
    Tennokai,
    SlamLoop {
        /// Slam cycle time (s, default 1).
        #[serde(default = "default_t_slam")]
        t_slam: f64,
    },
    HeavySlamLoop {
        /// Heavy slam cycle time (s, default 1).
        #[serde(default = "default_t_slam")]
        t_slam: f64,
    },
    GlaiveCycle {
        /// Throw kind of the cycle.
        #[serde(default)]
        throw: ThrowKind,
    },
    /// One 100 % strike every T_cyc (weapons without light combos, 07 §8.1).
    CycleLoop,
    /// One instance of a data attack every T_cyc; `None` = 1 / FireRate of the attack (07 §8.1, OQ-727).
    DataAttackLoop {
        /// Cycle time T_cyc (s).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "ts", ts(optional))]
        t_cyc: Option<f64>,
    },
}

fn default_t_slam() -> f64 {
    1.0
}

/// Heavy loop policy (07 §8.1).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HeavyPolicy {
    #[default]
    FullRegen,
    BackToBack,
}

/// Glaive throw kind (07 E-7.36).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ThrowKind {
    #[default]
    Charged,
    Quick,
}

/// Where a toggle lives (domain-model.md §9.3), as the field path in the document. Several paths are separated by
/// `|`; a path through a list names the list (`"buffs.caster"` = the `caster` of every `buffs[]` entry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TogglePlace {
    /// A field of `ScenarioDoc` (`"toggles.hit_location"`, `"eval.window_s"`, `"conditions.mode"`, `"enemy"`).
    Scenario(&'static str),
    /// A field of `BuildDoc` (`"variant.progenitor"`).
    Build(&'static str),
    /// The buff items whose `ItemKind::Buff.toggle` is the toggle id, param `value`.
    Buffs,
    /// A field of the optimizer request (`"constraints.capacity.ranks"`).
    OptRequest(&'static str),
}

/// Whether spec 00 §6.1 lists the toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToggleStatus {
    /// In spec 00 §6.1.
    Spec,
    /// Proposed by a topic document, numbered provisionally (overview.md §6.2 R-1).
    Proposed,
}

/// Every toggle row of domain-model.md §9.3, with its place and status; `xtask spec-sync` checks it against spec 00
/// §6.1 (T-01 … T-43, every one `Spec`; renumbered to the final spec by T01d, CONTRACTS.md).
pub static TOGGLE_MAP: &[(&str, TogglePlace, ToggleStatus)] = &[
    (
        "T-01",
        TogglePlace::Scenario("eval.window_s"),
        ToggleStatus::Spec,
    ),
    (
        "T-02",
        TogglePlace::Scenario("conditions.mode"),
        ToggleStatus::Spec,
    ),
    (
        "T-03",
        TogglePlace::Scenario("toggles.hit_location"),
        ToggleStatus::Spec,
    ),
    (
        "T-04",
        TogglePlace::Scenario("toggles.co_multiplicative"),
        ToggleStatus::Spec,
    ),
    (
        "T-05",
        TogglePlace::Scenario("toggles.co_status_count"),
        ToggleStatus::Spec,
    ),
    (
        "T-06",
        TogglePlace::Scenario("toggles.external_viral"),
        ToggleStatus::Spec,
    ),
    (
        "T-07",
        TogglePlace::Scenario("toggles.armor_strip"),
        ToggleStatus::Spec,
    ),
    (
        "T-08",
        TogglePlace::Scenario("toggles.stealth"),
        ToggleStatus::Spec,
    ),
    (
        "T-09",
        TogglePlace::Scenario("toggles.heavy"),
        ToggleStatus::Spec,
    ),
    (
        "T-10",
        TogglePlace::Scenario("toggles.light_combo"),
        ToggleStatus::Spec,
    ),
    (
        "T-11",
        TogglePlace::Scenario("toggles.ability_combo"),
        ToggleStatus::Spec,
    ),
    (
        "T-12",
        TogglePlace::Scenario("toggles.sniper_combo"),
        ToggleStatus::Spec,
    ),
    ("T-13", TogglePlace::Buffs, ToggleStatus::Spec),
    ("T-14", TogglePlace::Buffs, ToggleStatus::Spec),
    ("T-15", TogglePlace::Buffs, ToggleStatus::Spec),
    ("T-16", TogglePlace::Buffs, ToggleStatus::Spec),
    ("T-17", TogglePlace::Buffs, ToggleStatus::Spec),
    ("T-18", TogglePlace::Buffs, ToggleStatus::Spec),
    ("T-19", TogglePlace::Buffs, ToggleStatus::Spec),
    (
        "T-20",
        TogglePlace::Scenario("toggles.strength"),
        ToggleStatus::Spec,
    ),
    (
        "T-21",
        TogglePlace::Scenario("toggles.wp_cc_multiplicative"),
        ToggleStatus::Spec,
    ),
    (
        "T-22",
        TogglePlace::Scenario("toggles.combine_groups"),
        ToggleStatus::Spec,
    ),
    ("T-23", TogglePlace::Scenario("enemy"), ToggleStatus::Spec),
    (
        "T-24",
        TogglePlace::Build("variant.progenitor"),
        ToggleStatus::Spec,
    ),
    (
        "T-25",
        TogglePlace::Build("variant.kitgun|variant.zaw|slots.content.item.params"),
        ToggleStatus::Spec,
    ),
    ("T-26", TogglePlace::Scenario("custom"), ToggleStatus::Spec),
    (
        "T-27",
        TogglePlace::Scenario("toggles.distance_m"),
        ToggleStatus::Spec,
    ),
    (
        "T-28",
        TogglePlace::Scenario("toggles.unique_dr"),
        ToggleStatus::Spec,
    ),
    (
        "T-29",
        TogglePlace::Scenario("eval.mc_runs|eval.seed"),
        ToggleStatus::Spec,
    ),
    (
        "T-30",
        TogglePlace::Scenario("toggles.melee_plan|toggles.melee_plans"),
        ToggleStatus::Spec,
    ),
    (
        "T-31",
        TogglePlace::Scenario("toggles.exalted_rank"),
        ToggleStatus::Spec,
    ),
    (
        "T-32",
        TogglePlace::Scenario("buffs.params.spot_location|buffs.params.spot_nodes"),
        ToggleStatus::Spec,
    ),
    (
        "T-33",
        TogglePlace::Scenario("toggles.strength_mode|toggles.ability_duration|buffs.caster"),
        ToggleStatus::Spec,
    ),
    (
        "T-34",
        TogglePlace::Scenario("toggles.finisher"),
        ToggleStatus::Spec,
    ),
    (
        "T-35",
        TogglePlace::Scenario("toggles.capacity|toggles.forma_types"),
        ToggleStatus::Spec,
    ),
    (
        "T-36",
        TogglePlace::OptRequest("constraints.capacity.ranks"),
        ToggleStatus::Spec,
    ),
    (
        "T-37",
        TogglePlace::Scenario("buffs.caster"),
        ToggleStatus::Spec,
    ),
    (
        "T-38",
        TogglePlace::Scenario("toggles.aiming|toggles.zoom_level"),
        ToggleStatus::Spec,
    ),
    (
        "T-39",
        TogglePlace::Scenario("toggles.incarnon_switch"),
        ToggleStatus::Spec,
    ),
    (
        "T-40",
        TogglePlace::Scenario("toggles.timed_reload"),
        ToggleStatus::Spec,
    ),
    (
        "T-41",
        TogglePlace::Scenario("toggles.pellet_hit"),
        ToggleStatus::Spec,
    ),
    (
        "T-42",
        TogglePlace::Scenario("toggles.gauge_uses_per_min"),
        ToggleStatus::Spec,
    ),
    (
        "T-43",
        TogglePlace::Scenario("toggles.external_statuses"),
        ToggleStatus::Spec,
    ),
];

// ---------------------------------------------------------------------------------------------------------------
// Conditions policy (§9.4)
// ---------------------------------------------------------------------------------------------------------------

/// T-02 mode plus per-effect overrides.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields, default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConditionPolicy {
    pub mode: ConditionMode,
    pub overrides: BTreeMap<EffectKey, CondOverride>,
}

/// T-02 (00 §6.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ConditionMode {
    #[default]
    AllMax,
    None,
    Simulated,
}

/// A per-effect condition override. JSON `{ "force": true }`, `{ "stacks": 2 }`,
/// `{ "uptime": { "u": 0.5, "stacks"?: 1 } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CondOverride {
    /// Always on / always off, any mode.
    Force(bool),
    /// Fixed stack count.
    Stacks(u16),
    /// Class (c) under `simulated`: u_e and m_e of 04 §4.3.
    Uptime {
        /// Uptime u_e (fraction).
        u: f64,
        /// Stacks m_e; `None` = the maximum.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "ts", ts(optional))]
        stacks: Option<u16>,
    },
}

// ---------------------------------------------------------------------------------------------------------------
// Resolved form (§9.2)
// ---------------------------------------------------------------------------------------------------------------

/// A scenario with dense ids, produced by `wfd-data::GameData::resolve`.
///
/// Readable form only: the compact (postcard) form fails, because six fields of the `Toggles` and
/// `ConditionPolicy` tree skip `None` when serialized ([`crate::serde_util`]: the compact form must not skip
/// fields): `CoStatusCount.n`, `Stealth.rank`, `MeleePlanSel::HeavyLoop.heavy_cancel`, `MeleePlanSel::BuildSpend.k`,
/// `MeleePlanSel::DataAttackLoop.t_cyc` and `CondOverride::Uptime.stacks`. `Toggles::default()` already fails
/// (`co_status_count.n` is `None`). Whoever first postcard-encodes a resolved scenario (T32/T33 cache keys) gives
/// these types a `dual_serde!` form or hashes another encoding, and records it in CONTRACTS.md (ruling on T01d,
/// amended by the contract review, M-11).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Scenario {
    /// The resolved enemy setup.
    pub enemy: EnemyInstance,
    /// The toggles.
    pub toggles: Toggles,
    /// The resolved buffs.
    #[cfg_attr(feature = "ts", ts(as = "Vec<BuffInstance>"))]
    pub buffs: SmallVec<[BuffInstance; 4]>,
    /// Custom-stat items and their values.
    #[cfg_attr(feature = "ts", ts(as = "Vec<(ItemId, f64)>"))]
    pub custom: SmallVec<[(ItemId, f64); 4]>,
    /// T-02 and per-effect overrides.
    pub conditions: ConditionPolicy,
    /// T-01, T-29, horizons.
    pub eval: EvalSettings,
    /// Evaluation profile.
    pub profile: Profile,
}

/// A resolved enemy setup.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyInstance {
    /// The enemy.
    pub enemy: EnemyId,
    /// Enemy level.
    pub level: u16,
    /// Steel Path scaling.
    pub steel_path: bool,
    /// Eximus variant.
    pub eximus: bool,
    /// Squad size.
    pub party: u8,
    /// Mission modifier (05 §7.5).
    pub mission: Option<MissionModifier>,
    /// Use the context-only overguard (05 §4.1).
    pub context_overguard: bool,
    /// Eximus kind (05 §4.1).
    pub eximus_kind: EximusKind,
    /// Hydroid Corrosive variant (05 E-5.12).
    pub armor_hydroid: bool,
    /// Squadmates' Corrosive Projection auras (05 E-5.10).
    pub squad_cp: SquadAuras,
    /// Squadmates' Shield Disruption auras (05 E-5.16).
    pub squad_shield_disruption: SquadAuras,
    /// External ability shield strips s_j·Str_j (05 E-5.16).
    pub shield_strips: Vec<f64>,
    /// Flat base-armor strip rate r_flat, points per second (05 E-5.11).
    pub flat_strip_rate: f64,
    /// Inside a Guardian Eximus aura (05 E-5.27).
    pub aura_guardian: bool,
    /// Prosecutor auras by element (05 E-5.27).
    pub aura_prosecutor: Vec<DamageType>,
    /// Layered pass starts with SH = 0 for `shield_weapon_immune` enemies (05 E-5.26).
    pub shield_removed_externally: bool,
    /// Profit-Taker displayed shield type (05 E-5.26).
    pub pt_shield_type: Option<DamageType>,
}

/// A resolved buff.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuffInstance {
    /// The buff item.
    pub item: ItemId,
    /// Parameter values (model units).
    #[cfg_attr(feature = "ts", ts(as = "Vec<(ParamKey, ParamValue)>"))]
    pub params: SmallVec<[(ParamKey, ParamValue); 1]>,
    /// Whose Strength and Duration scale the buff (the pick's caster, else the buff's default caster).
    pub caster: Caster,
}

/// Result of `GameData::resolve`: validation never blocks evaluation (09 §1.5), so violations travel with the
/// resolved inputs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Resolved {
    pub build: Build,
    pub scenario: Scenario,
    pub violations: Vec<Violation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Caster` has no skipped field, so the compact (postcard) form of a resolved buff round-trips for every caster,
    /// the default ally included (serde_util.rs: the compact form must not use skipped fields).
    #[test]
    fn caster_compact_round_trip() {
        for caster in [
            Caster::SelfCast,
            Caster::ally(),
            Caster::Ally {
                strength: 2.0,
                duration: 1.5,
            },
        ] {
            let b = BuffInstance {
                item: crate::ids::ItemId(3),
                params: smallvec::smallvec![(ParamKey::new("value"), ParamValue::Num(0.5))],
                caster: caster.clone(),
            };
            let bytes = postcard::to_stdvec(&b).unwrap();
            assert_eq!(
                postcard::from_bytes::<BuffInstance>(&bytes).unwrap(),
                b,
                "{caster:?}"
            );
            let json = serde_json::to_string(&b).unwrap();
            assert_eq!(serde_json::from_str::<BuffInstance>(&json).unwrap(), b);
        }
        // The readable form always writes the ally Duration and still accepts it absent.
        assert_eq!(
            serde_json::to_string(&Caster::ally()).unwrap(),
            r#"{"ally":{"strength":1.0,"duration":1.0}}"#
        );
        assert_eq!(
            serde_json::from_str::<Caster>(r#"{"ally":{"strength":1.0}}"#).unwrap(),
            Caster::ally()
        );
    }

    /// T-22 (breaking change of T01d): `null` and an absent field are the profile default, an explicit `[]` keeps its
    /// old meaning (no family combined) and round-trips as `[]`.
    #[test]
    fn combine_groups_forms() {
        let absent: Toggles = serde_json::from_str("{}").unwrap();
        assert_eq!(absent.combine_groups, None);
        let null: Toggles = serde_json::from_str(r#"{"combine_groups":null}"#).unwrap();
        assert_eq!(null.combine_groups, None);
        let empty: Toggles = serde_json::from_str(r#"{"combine_groups":[]}"#).unwrap();
        assert_eq!(empty.combine_groups, Some(SmallVec::new()));
        assert!(
            serde_json::to_string(&empty)
                .unwrap()
                .contains(r#""combine_groups":[]"#)
        );
        let some: Toggles = serde_json::from_str(r#"{"combine_groups":[0,2]}"#).unwrap();
        assert_eq!(some.combine_groups.as_deref(), Some(&[0u8, 2][..]));
    }

    /// T-37 (breaking change of T01d): an absent `caster` is `None`, which resolves to the buff's default caster.
    #[test]
    fn buff_pick_caster_absent_is_default() {
        let p: BuffPick = serde_json::from_str(r#"{"buff":40}"#).unwrap();
        assert_eq!(p.caster, None);
        assert!(!serde_json::to_string(&p).unwrap().contains("caster"));
        let s: BuffPick = serde_json::from_str(r#"{"buff":40,"caster":"self"}"#).unwrap();
        assert_eq!(s.caster, Some(Caster::SelfCast));
    }
}
