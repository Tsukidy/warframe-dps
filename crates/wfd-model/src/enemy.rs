//! Enemies and factions (domain-model.md §5; record of spec 05 §4.1).
//!
//! Fields added to the domain-model struct because spec 05 §4.1 and §7 define them (CONTRACTS.md, T01a):
//! `damage_faction`, `spawn_level`, `sp_shield_bonus`, `archimedea_health_bonus`, `dr_class`, `dr_class_og`,
//! `party_scaling`, `defaults`, `overguard_context`, the Eximus armor/overguard overrides and `eximus.og`, and the
//! body-part fields `column`, `crit_disabled`, `wp_bonus_ok`, `part_dr`, `is_default`.

use crate::enums::{DamageType, DamageVec, N_ST, StatusSet};
use crate::ids::{EnemyId, FactionId, NameKey, StableKey, UniqueName};
use crate::scenario::MissionModifier;
use crate::serde_util::flags_serde;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// An enemy record (05 §4.1). Enemies have no DE export; `unique_name` is `wfd:enemy/<wiki sub-module>/<slug>`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Enemy {
    pub id: EnemyId,
    pub key: StableKey,
    pub unique_name: UniqueName,
    pub name: NameKey,
    /// Drives the bane bucket (01) and the default scaling class.
    pub faction: FactionId,
    /// Drives φ_t (05 §5) when it differs from `faction` (`FactionDamageOverride`); `None` = `faction`.
    pub damage_faction: Option<FactionId>,
    pub base_level: u16,
    /// UI default level (`SpawnLevel`); `None` = `base_level`.
    pub spawn_level: Option<u16>,
    pub health: f64,
    pub shield: f64,
    pub armor: f64,
    pub overguard: f64,
    pub eximus: EximusData,
    /// D-055.
    pub sp_health_bonus: f64,
    pub sp_shield_bonus: f64,
    pub archimedea_health_bonus: f64,
    /// 05 §4.3 part records.
    #[cfg_attr(feature = "ts", ts(as = "Vec<BodyPart>"))]
    pub parts: SmallVec<[BodyPart; 4]>,
    pub immune: StatusSet,
    /// Per-status cap overrides (05 §4.4), indexed by `StatusType::index`.
    pub caps: [Option<u8>; N_ST],
    /// 05 §5.2-5.3.
    pub type_mod_override: Option<DamageVec>,
    /// Type resistances `type_res[t]` (05 §4.5, E-5.1).
    pub resist: Option<DamageVec>,
    /// Innate DR per layer, indexed by `Layer::index` (OG, SH, HP).
    pub innate_dr: [f64; 3],
    pub status_dr: f64,
    /// Class-conditional DR (05 §14).
    pub dr_class: DrClass,
    /// Class-conditional DR on the overguard layer only.
    pub dr_class_og: DrClass,
    /// Index into `Rules.special_dr` (attenuation, DPS-scaled DR, adaptation; 05 §4.5, §15).
    pub special_dr: Option<u16>,
    /// Health factor P_party for squad sizes 1-4 (05 E-5.6; Demolisher family).
    pub party_scaling: Option<[f64; 4]>,
    /// Scenario defaults when the user picks the enemy (never forced).
    pub defaults: EnemyDefaults,
    /// The record's overguard exists only in this context (Dax Malleus); used only with `context_overguard`.
    pub overguard_context: Option<Box<str>>,
    pub flags: EnemyFlags,
    /// Scaling class, index into `Rules.scaling` (05 §6.2).
    pub scaling: u16,
    /// `None`: enemies have no DE export; the UI shows a faction glyph.
    pub image: Option<Box<str>>,
}

/// Eximus data of an enemy (05 §4.1, §7.3, §8).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EximusData {
    pub health: Option<f64>,
    pub shield: Option<f64>,
    pub armor: Option<f64>,
    pub overguard: Option<f64>,
    /// The enemy cannot be Eximus (`eximus_capable = false`).
    pub off: bool,
    /// An Eximus of this enemy has overguard (`eximus_og`; false for Archwing, Warden, Prosecutor).
    pub og: bool,
}

/// Class-conditional damage reduction (05 §14): per class of instance.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrClass {
    pub radial_or_melee: f64,
    pub non_melee: f64,
    pub radial: f64,
}

/// Scenario defaults of an enemy (05 §4.1 `default_steel_path`, `default_eximus`, `default_mission`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyDefaults {
    pub steel_path: bool,
    pub eximus: bool,
    pub mission: Option<MissionModifier>,
}

/// A body part (05 §4.3; field names of the spec).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BodyPart {
    pub name: NameKey,
    /// Column of the source table.
    pub column: PartColumn,
    /// P_part.
    pub part_mult: f64,
    pub is_weak_point: bool,
    pub is_head: bool,
    /// h: factor on the crit multiplier, `c_k = 1 + k·(h·CDM − 1)`.
    pub headcrit: f64,
    /// Critical promotion (02 E-2.6 π_p): Rogue Culverin canisters, Scaldra backpacks.
    pub tier_add: u8,
    /// Hits never crit (Scaldra TI-92 turret).
    pub crit_disabled: bool,
    /// ΣWP applies on the part.
    pub wp_bonus_ok: bool,
    /// Flat DR of instances that hit the part (E-5.20).
    pub part_dr: f64,
    pub status_ok: bool,
    pub no_armor: bool,
    pub no_faction_mod: bool,
    /// The part used for `body` hits and every radial instance.
    pub is_default: bool,
}

/// Column of a body part in `W:Enemy_Body_Parts` (05 §4.3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PartColumn {
    #[default]
    General,
    WeakPoint,
    Other,
}

bitflags::bitflags! {
    /// Enemy flags (05 §4.1, §4.4, §10, §13.1). Readable form: lowercase names.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct EnemyFlags: u32 {
        /// Shields absorb Toxin (`toxin_blocked = absorb`, Hounds).
        const TOXIN_BLOCKED = 1 << 0;
        /// Toxin is nullified (`toxin_blocked = nullify`, Treasurer).
        const TOXIN_NULLIFY = 1 << 1;
        /// Health scales with squad size (`party_scaling`).
        const PARTY_SCALED = 1 << 2;
        /// `boss` (05 §4.1 type set and override list).
        const BOSS = 1 << 3;
        /// The record carries per-status caps.
        const STATUS_CAPPED = 1 << 4;
        /// `armor_strip_immune = all`.
        const STRIP_IMMUNE = 1 << 5;
        /// `armor_strip_immune = flat_only`.
        const STRIP_IMMUNE_FLAT = 1 << 6;
        /// `shield_strip_immune`.
        const SHIELD_STRIP_IMMUNE = 1 << 7;
        /// `shield_weapon_immune = all`.
        const SHIELD_WEAPON_IMMUNE = 1 << 8;
        /// `shield_weapon_immune = displayed_type`.
        const SHIELD_DISPLAYED_TYPE = 1 << 9;
        /// `shield_regen = none`.
        const SHIELD_NO_REGEN = 1 << 10;
        /// `shield_regen = none_after_break`.
        const SHIELD_NO_REGEN_AFTER_BREAK = 1 << 11;
        /// `ability_debuff_immune = all`.
        const ABILITY_DEBUFF_IMMUNE = 1 << 12;
        /// `ability_debuff_immune = dispelled`.
        const ABILITY_DEBUFF_DISPELLED = 1 << 13;
        /// No part can headcrit.
        const NO_HEADCRIT = 1 << 14;
        /// `no_freeze`.
        const NO_FREEZE = 1 << 15;
        /// `vip`.
        const VIP = 1 << 16;
        /// `is_object`.
        const IS_OBJECT = 1 << 17;
        /// `adversary`.
        const ADVERSARY = 1 << 18;
        /// Squad scaling exists but is not modelled (Acolytes, OQ-518).
        const SQUAD_SCALING_UNMODELLED = 1 << 19;
        /// Test or hidden record (`[Wiki Dummy]`).
        const HIDDEN = 1 << 20;
    }
}
flags_serde!(EnemyFlags, u32, Lower);

/// A faction of `Rules.factions` (05 §5.1 ingested + wiki `DamageTypes/data`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Faction {
    pub id: FactionId,
    pub slug: Box<str>,
    pub name: NameKey,
    /// φ_t (1.5 / 1 / 0.5).
    pub type_mods: DamageVec,
    /// Which faction bonuses apply (Kuva Grineer takes Grineer banes, D-008).
    #[cfg_attr(feature = "ts", ts(as = "Vec<FactionId>"))]
    pub bane_of: SmallVec<[FactionId; 2]>,
    /// Weakness-element map (01 §4.10.4).
    pub weakness: Option<DamageType>,
    /// Default scaling class, index into `Rules.scaling` (05 §6.2).
    pub scaling_family: u16,
}
