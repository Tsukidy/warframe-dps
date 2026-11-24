//! Scenario documents, toggles and conditions (domain-model.md §9.1, §9.3, §9.4), the calculation input, and the
//! resolved scenario (§9.2). Every toggle and eval field has the default of the §9.3 table and is filled when missing
//! (server-api.md §4), so old share codes keep working.

use crate::build::{Build, BuildDoc, Violation};
use crate::compat::CompatFlags;
use crate::ids::{EffectKey, EnemyId, ItemId, ParamKey, StableKey, UniqueName};
use crate::item::ParamValue;
use crate::serde_util::is_false;
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
    /// T-26: custom stat id (`"base"`, `"crit_chance"` …) → model-unit value.
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

/// Enemy options (T-23). `mission` and `context_overguard` are the optional scenario fields of 05 §7.5 and §4.1
/// (additions, CONTRACTS.md T01a).
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
}

fn default_party() -> u8 {
    1
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

/// A buff in the scenario. T-13..T-17 values are final values: caster Strength is not applied.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuffPick {
    /// The buff item.
    pub buff: StableKey,
    /// Parameter values (model units); missing keys take the item's defaults.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub params: BTreeMap<ParamKey, ParamValue>,
    /// Whose Strength scales the buff (04 E-4.10).
    #[serde(default, skip_serializing_if = "Caster::is_self")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub caster: Caster,
}

/// Whose Strength scales a buff (04 E-4.10). JSON `"self"` or `{ "ally": { "strength": 1.0 } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Caster {
    /// Str = T-20.
    #[default]
    #[serde(rename = "self")]
    SelfCast,
    Ally {
        /// The ally's Ability Strength (factor, default 1).
        #[serde(default = "crate::serde_util::default_one_f64")]
        strength: f64,
    },
}

impl Caster {
    /// `true` for a self-cast buff (the document then omits `caster`).
    pub fn is_self(&self) -> bool {
        *self == Caster::SelfCast
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

/// Evaluation profile. JSON `"game"` or `{ "compat": ["D001_QUANT_1_16", …] }`: flag names, never the u128
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
/// fields take the defaults (`#[serde(default)]`); the proposed toggles are optional in TypeScript as in
/// server-api.md §4.
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
    /// T-06: 1..=10.
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
    /// T-20: 0-10.
    pub strength: f64,
    /// T-21 (hidden).
    pub wp_cc_multiplicative: bool,
    /// T-22: group indices.
    #[cfg_attr(feature = "ts", ts(as = "Vec<u8>"))]
    pub combine_groups: SmallVec<[u8; 2]>,
    /// T-27: 0-1000 m.
    pub distance_m: f64,
    /// T-28.
    pub unique_dr: bool,
    /// Proposed T-30 (09 OQ-915; numbering request R-1).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub capacity: CapacityMode,
    /// Proposed T-32 (07 OQ-722 as "T-30"; R-1).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub melee_plan: MeleePlanSel,
    /// Index into `Weapon.zoom`; `None` = highest level under `all_max`, none under `none` (04 D-406).
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub zoom_level: Option<u8>,
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
            combine_groups: SmallVec::new(),
            distance_m: 0.0,
            unique_dr: true,
            capacity: CapacityMode::AnyForma,
            melee_plan: MeleePlanSel::Auto,
            zoom_level: None,
        }
    }
}

/// T-03 hit location.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HitLocation {
    #[default]
    Body,
    Head,
    WeakPoint,
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

/// Proposed T-30 capacity mode (09 E-9.18). JSON `"ignore"`, `"current"`, `"any_forma"` or `{ "budget": N }`.
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

/// 07 §8.1 plans with their parameters (proposed T-32). `Auto`: Normal Attack → light_loop(selected combo), heavy
/// (T-09 or selected) → heavy_loop, slide → slide_loop, slam → slam_loop, heavy slam → heavy_slam_loop, throws →
/// glaive_cycle, weapons without light combos → cycle_loop. JSON: `"auto"`, `"light_loop"`,
/// `{ "heavy_loop": { "policy": "full_regen" } }`, `{ "build_spend": { "k": 12 } }`, `"tennokai"`,
/// `{ "slam_loop": { "t_slam": 1.0 } }`, `{ "glaive_cycle": { "throw": "charged" } }` …
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
    /// Light loop until M ≥ k, then one heavy (default k = M_cap).
    BuildSpend {
        /// Combo multiplier M at which the heavy is spent.
        k: f64,
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

/// Where a toggle lives (domain-model.md §9.3), as the field path in the document.
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

/// Every toggle row of domain-model.md §9.3 that has a T-id, with its place and status; `xtask spec-sync` checks it
/// against spec 00 §6.1. The zoom level (04 D-406) has no T-id and is not listed (field `toggles.zoom_level`).
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
        TogglePlace::Scenario("toggles.capacity"),
        ToggleStatus::Proposed,
    ),
    (
        "T-31",
        TogglePlace::OptRequest("constraints.capacity.ranks"),
        ToggleStatus::Proposed,
    ),
    (
        "T-32",
        TogglePlace::Scenario("toggles.melee_plan"),
        ToggleStatus::Proposed,
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
    /// Whose Strength scales the buff.
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
