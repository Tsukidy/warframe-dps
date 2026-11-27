//! Enemies and factions (domain-model.md §5; record of spec 05 §4.1).
//!
//! Fields added to the domain-model struct because spec 05 §4.1 and §7 define them (CONTRACTS.md, T01a):
//! `damage_faction`, `spawn_level`, `sp_shield_bonus`, `archimedea_health_bonus`, `dr_class`, `dr_class_og`,
//! `party_scaling`, `defaults`, `overguard_context`, the Eximus armor/overguard overrides and `eximus.og`, and the
//! body-part fields `column`, `crit_disabled`, `wp_bonus_ok`, `part_dr`, `is_default`. Added by T01d (spec rounds):
//! `BodyPart.non_body`, `EnemyFlags::CC_IMMUNE`, `EximusData.kinds`. Contract review (CONTRACTS.md "Contract
//! review fixes (model-records)"): `Enemy.bolstered_og_factor`, `EnemyFlags::GUARDIAN_AURA_IMMUNE`, and
//! `party_scaling` as a list.

use crate::enums::{DamageType, DamageVec, N_ST, StatusSet};
use crate::ids::{EnemyId, FactionId, NameKey, StableKey, UniqueName};
use crate::scenario::{EximusKind, MissionModifier};
use crate::serde_util::flags_serde;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// An enemy record (05 §4.1). Enemies have no DE export; `unique_name` is `wfd:enemy/<wiki sub-module>/<slug>`.
///
/// The record holds no mission-conditional field. The 05 §4.5 rows stated "in Archon Hunts" (innate and status DR,
/// `armor_strip_immune = all`, `ability_debuff_immune = all`, caps 4, damage faction Narmer) are unconditional data of
/// the Archon records, which describe the Archon Hunt units; the mission modifier `archon_hunt` (05 §7.5) adds only
/// the E-5.4 health and shield factor. Convention of the contract review (M-17), UNVERIFIED that no other mission
/// fields an Archon record; the 05 spec round may add a mission-gated form.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Enemy {
    /// Dense id (bundle order).
    pub id: EnemyId,
    /// Stable key used by documents.
    pub key: StableKey,
    /// Join key (wiki-derived for enemies).
    pub unique_name: UniqueName,
    /// Name lookup key.
    pub name: NameKey,
    /// Drives the bane bucket (01) and the default scaling class.
    pub faction: FactionId,
    /// Drives φ_t (05 §5) when it differs from `faction` (`FactionDamageOverride`); `None` = `faction`.
    pub damage_faction: Option<FactionId>,
    /// Base level L0 (`Stats.BaseLevel`, 1 if absent; 05 §4.1).
    pub base_level: u16,
    /// UI default level (`SpawnLevel`); `None` = `base_level`.
    pub spawn_level: Option<u16>,
    /// Base health at L0 (HP; 0 = layer absent).
    pub health: f64,
    /// Base shield at L0 (HP; 0 = layer absent).
    pub shield: f64,
    /// Base armor A_base at L0 (armor points).
    pub armor: f64,
    /// Base overguard (HP; 0 = none; semantics 05 §8).
    pub overguard: f64,
    /// Eximus overrides (05 §7.3, §8).
    pub eximus: EximusData,
    /// D-055.
    pub sp_health_bonus: f64,
    /// Steel Path shield bonus (`SteelPathShieldBonus`, 0 if absent).
    pub sp_shield_bonus: f64,
    /// Archimedea health bonus (`ArchimedeaHealthBonus`, 0 if absent).
    pub archimedea_health_bonus: f64,
    /// 05 §4.3 part records.
    #[cfg_attr(feature = "ts", ts(as = "Vec<BodyPart>"))]
    pub parts: SmallVec<[BodyPart; 4]>,
    /// Statuses the enemy is immune to.
    pub immune: StatusSet,
    /// Per-status cap overrides (05 §4.4), indexed by `StatusType::index`.
    pub caps: [Option<u8>; N_ST],
    /// 05 §5.2-5.3.
    pub type_mod_override: Option<DamageVec>,
    /// Type resistances `type_res[t]` (05 §4.5, E-5.1).
    pub resist: Option<DamageVec>,
    /// Innate DR per layer, indexed by `Layer::index` (OG, SH, HP).
    pub innate_dr: [f64; 3],
    /// Damage reduction of status damage (fraction, 05 §4.1; Archons 0.5).
    pub status_dr: f64,
    /// Class-conditional DR (05 §14).
    pub dr_class: DrClass,
    /// Class-conditional DR on the overguard layer only.
    pub dr_class_og: DrClass,
    /// Index into `Rules.special_dr` (attenuation, DPS-scaled DR, adaptation; 05 §4.5, §15).
    pub special_dr: Option<u16>,
    /// Health factors P_1, P_2, … of 05 E-5.6 by squad size (Demolisher family: 1, 1.5, 2, 3). A squad larger than
    /// the list (The Icebind's 5 and 6 players, 05 E-5.4) uses the last entry, P_min(n, len), until 05 gives P_5 and
    /// P_6; the list form carries either answer (spec question of the contract review, M-17). JSON and TOML
    /// `[1.0, 1.5, 2.0, 3.0]` as before; a list since CONTRACTS.md "Contract review fixes (model-records)".
    pub party_scaling: Option<Vec<f64>>,
    /// Scenario defaults when the user picks the enemy (never forced).
    pub defaults: EnemyDefaults,
    /// The record's overguard exists only in this context (Dax Malleus); used only with `context_overguard`.
    pub overguard_context: Option<Box<str>>,
    /// Factor on the overguard that the mission modifier `bolstered_belligerents` gives the enemy (05 §7.5:
    /// OG_max = 0.5·HP_max; Techrot Babau "75 % less", U38.5: 0.25); `None` = 1. Addition, CONTRACTS.md "Contract
    /// review fixes (model-records)", M-17.
    #[serde(default)]
    pub bolstered_og_factor: Option<f64>,
    /// Enumerated 05 §4.1 flags.
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
    /// Eximus health (`EximusHealth`); `None` = the normal value.
    pub health: Option<f64>,
    /// Eximus shield (`EximusShield`); `None` = the normal value.
    pub shield: Option<f64>,
    /// Eximus armor (`EximusArmor`); `None` = the normal value.
    pub armor: Option<f64>,
    /// Eximus overguard (`EximusOverguard`); `None` = the rule of 05 §8.
    pub overguard: Option<f64>,
    /// The enemy cannot be Eximus (`eximus_capable = false`).
    pub off: bool,
    /// An Eximus of this enemy has overguard (`eximus_og`; false for Archwing, Warden, Prosecutor).
    pub og: bool,
    /// Eximus kinds the record can take (`eximus_kinds`, 05 §4.1: the ticked columns of `W:Eximus/Compatibilities`,
    /// Entropic and Guardian by name, every other column as `generic`); the UI offers only these for T-23's Eximus
    /// kind. Addition, CONTRACTS.md T01d.
    #[serde(default)]
    pub kinds: Vec<EximusKind>,
}

/// Class-conditional damage reduction (05 §14): per class of instance.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrClass {
    /// DR of radial or melee instances (fraction).
    pub radial_or_melee: f64,
    /// DR of non-melee instances (fraction).
    pub non_melee: f64,
    /// DR of radial instances (fraction).
    pub radial: f64,
}

/// Scenario defaults of an enemy (05 §4.1 `default_steel_path`, `default_eximus`, `default_mission`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyDefaults {
    /// Steel Path on by default (`default_steel_path`).
    pub steel_path: bool,
    /// Eximus on by default (`default_eximus`).
    pub eximus: bool,
    /// Default mission modifier (`default_mission`).
    pub mission: Option<MissionModifier>,
}

/// A body part (05 §4.3; field names of the spec).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BodyPart {
    /// Part name lookup key.
    pub name: NameKey,
    /// Column of the source table.
    pub column: PartColumn,
    /// P_part.
    pub part_mult: f64,
    /// The part is a weak point (ΣWP, weak-point triggers).
    pub is_weak_point: bool,
    /// The part is a head (headshot triggers, ΣHS).
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
    /// Procs apply on hits to the part.
    pub status_ok: bool,
    /// Non-body hitbox (05 §4.3 `non_body`, `W:Enemy_Body_Parts` L37): neither bodyshot nor headshot (02 E-2.10
    /// `Part_B = 1`); implies `is_head = false`, `status_ok = false`, `no_armor = no_faction_mod = true`. Addition,
    /// CONTRACTS.md T01d.
    #[serde(default)]
    pub non_body: bool,
    /// Armor does not reduce hits to the part.
    pub no_armor: bool,
    /// The faction type modifiers φ_t do not apply on the part.
    pub no_faction_mod: bool,
    /// The part used for `body` hits and every radial instance except those of `radial_parent_part` attacks.
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
        /// `cc_immune` (05 §4.1: bosses, Acolytes, the Demolisher family): immune to complete crowd control and
        /// ragdoll (03 E-3.13; 06 E-6.24 stun-gated children). Overguard adds the same immunity as a state
        /// (`og_present`), not as this flag. Addition, CONTRACTS.md T01d.
        const CC_IMMUNE = 1 << 21;
        /// Warframe-ability effects on the enemy last 1/4 as long (Sister Hounds, 05 §13.1, `W:Hound` L35). A flag
        /// only: no stage reads it. Addition, CONTRACTS.md T03b review fixes.
        const ABILITY_DURATION_QUARTER = 1 << 22;
        /// The Guardian Eximus aura of 05 E-5.27 (T-23 `aura_guardian`) has no effect on the enemy: set by curated
        /// data on the Demolisher family (05 §4.1 group table, `W:Demolisher`); `boss` records are excluded through
        /// `BOSS`, a target that is itself a Guardian Eximus through T-23's Eximus kind. Addition, CONTRACTS.md
        /// "Contract review fixes (model-records)", M-16.
        const GUARDIAN_AURA_IMMUNE = 1 << 23;
    }
}
flags_serde!(EnemyFlags, u32, Lower);

/// A faction of `Rules.factions` (05 §5.1 ingested + wiki `DamageTypes/data`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Faction {
    /// Dense id.
    pub id: FactionId,
    /// Slug used by curated data (`"grineer"`).
    pub slug: Box<str>,
    /// Name lookup key.
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
