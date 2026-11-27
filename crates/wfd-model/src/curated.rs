//! Authoring form of `data/curated/` (feature `curated`; domain-model.md §15, pipeline.md §6.1).
//!
//! One file per item: `data/curated/<kind>/<family>/<slug>.toml` with the header of domain-model.md §15.1, the kind
//! fields of non-catalogue items, `[[param]]` and `[[effect]]`; one file per Incarnon family
//! (`incarnon/<slot>/<family>.toml`, §16.10); and the rule tables of `rules/` and `rules/melee/`. Every type rejects
//! unknown fields. The effect types are those of the bundle (`EffectDef`): binding fields (`bind`, `bind_max`,
//! `bind_duration`, `bind_chance`) exist only in authoring files and are removed by lowering (T19).
//!
//! [`load_curated_files`] loads a tree given as (path, content) pairs (a directory on disk or the tree embedded for
//! the in-app refresh, pipeline.md §2); [`load_curated_dir`] reads a directory and calls it. `overrides/`, `aliases/`
//! and the top-level `priority.toml` and `schema.json` are read by other tasks (T18, T10, T19, T01b) and are not loaded
//! here; any other unexpected file is an error, so a misplaced or misnamed file is noticed. The authoring types of the
//! override and alias files are defined here all the same ([`WeaponOverrideFile`], [`AttackOverrideFile`],
//! [`EnemyOverrideFile`], [`ProgenitorFile`], [`EnemyAliasFile`]; contract review M-6, DC-18), so their writers (T20-T22,
//! T29) and their reader (T18) share one schema.

use crate::effect::{EffectDef, Scope};
use crate::enemy::{DrClass, EnemyFlags, PartColumn};
use crate::enums::{AttackRoles, DamageType, ShardColor, ShotType, StatusSet, Trigger};
use crate::ids::UniqueName;
use crate::item::{BuffGroup, EffectStatus, ParamSpec, RivenTraitDef};
use crate::rules::{
    BodyPartRules, CategoryDef, HelminthRules, MultGroupDef, RivenFormula, SlotLayout,
    SpecialDrDef, ToggleDescriptor, ToggleId,
};
use crate::scenario::CasterKind;
use crate::weapon::{
    Battery, FinisherRow, ForcedProcRule, GaugeSpec, HeavyClassRow, KohmSpec, MeleeTriggerRow,
    ProgenitorSpec, ReloadSpec, SniperComboSpec, Spool, StanceCoRow,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Version of the curated authoring schema (`schema = 1` in every file header).
pub const CURATED_SCHEMA: u32 = 1;

fn default_schema() -> u32 {
    CURATED_SCHEMA
}

// ---------------------------------------------------------------------------------------------------------------
// Item files
// ---------------------------------------------------------------------------------------------------------------

/// `upstream = { field = "wiki.Description", text = "+165% Damage", sha256 = "9b1c…" }` (written by `curate rebind`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Upstream {
    pub field: Box<str>,
    pub text: Box<str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<Box<str>>,
}

/// `ignore_numbers = [{ token = "n5%", reason = "recoil" }]` (domain-model.md §15.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IgnoreNumber {
    pub token: Box<str>,
    pub reason: Box<str>,
}

/// A conditional upstream stat line that the file deliberately does not implement (04 §5.3 exclusion rule):
/// `{ line = "<the line as the upstream text writes it>", reason = "…" }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExcludedLine {
    /// The line, as the upstream text writes it.
    pub line: Box<str>,
    /// Why it has no effect (out of scope, no damage effect, no source for the condition …).
    pub reason: Box<str>,
}

/// Header `kind` of a curated item file (checked against the catalogue kind).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CuratedKind {
    WeaponMod,
    WeaponArcane,
    FrameMod,
    Aura,
    FrameArcane,
    Shard,
    Stance,
    /// Set bonus effects (`set/<slug>.toml`; `id` is the PE mod-set path).
    Set,
    Buff,
    CustomStat,
}

impl CuratedKind {
    /// The top-level directory of the kind in `data/curated/`.
    pub fn dir(self) -> &'static str {
        match self {
            CuratedKind::WeaponMod => "weapon_mod",
            CuratedKind::WeaponArcane => "weapon_arcane",
            CuratedKind::FrameMod => "frame_mod",
            CuratedKind::Aura => "aura",
            CuratedKind::FrameArcane => "frame_arcane",
            CuratedKind::Shard => "shard",
            CuratedKind::Stance => "stance",
            CuratedKind::Set => "set",
            CuratedKind::Buff => "buff",
            CuratedKind::CustomStat => "custom",
        }
    }

    /// Every kind.
    pub const ALL: [CuratedKind; 10] = [
        CuratedKind::WeaponMod,
        CuratedKind::WeaponArcane,
        CuratedKind::FrameMod,
        CuratedKind::Aura,
        CuratedKind::FrameArcane,
        CuratedKind::Shard,
        CuratedKind::Stance,
        CuratedKind::Set,
        CuratedKind::Buff,
        CuratedKind::CustomStat,
    ];
}

/// One curated item file (domain-model.md §15.1 header, kind fields, `[[param]]`, `[[effect]]`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CuratedItemFile {
    #[serde(default = "default_schema")]
    pub schema: u32,
    /// UniqueName: the join key (`"wfd:…"` for our items).
    pub id: UniqueName,
    /// Must equal the catalogue English name (checked).
    pub name: Box<str>,
    pub kind: CuratedKind,
    /// verified | draft | unmodelled | no_damage_effect (`auto` and `stale` are pipeline states, rejected by the
    /// validator in authoring files).
    pub status: EffectStatus,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
    #[serde(default)]
    pub upstream: Option<Upstream>,
    #[serde(default)]
    pub ignore_numbers: Vec<IgnoreNumber>,
    /// Conditional stat lines of the upstream text that no effect of the file implements: excluded from every
    /// bucket, never applied as unconditional, and listed in the data-build report (04 §5.3 "Uncatalogued
    /// conditional lines"; `DataKey::CheckConditionalLineExcluded`). Addition, CONTRACTS.md T01d.
    #[serde(default)]
    pub excluded_lines: Vec<ExcludedLine>,
    #[serde(default)]
    pub notes: Box<str>,
    /// Default scope of every effect whose own scope is the default.
    #[serde(default)]
    pub scope: Option<Scope>,
    // ---- kind fields of non-catalogue items ----
    /// Buffs.
    #[serde(default)]
    pub group: Option<BuffGroup>,
    /// Buffs: the toggle the buff implements (`"T-13"`).
    #[serde(default)]
    pub toggle: Option<ToggleId>,
    /// Buffs: value = listed(rank) · Str of the caster (04 E-4.10).
    #[serde(default)]
    pub strength_scaled: Option<bool>,
    /// Buffs: durations × Dur_e of the caster (04 §4.2 `dur_scaled`, E-4.10). Addition, CONTRACTS.md T01d.
    #[serde(default)]
    pub duration_scaled: Option<bool>,
    /// Buffs: caster of a pick without one (04 §4.2 `caster`; `ally` for T-13 Roar, spec 00 §6.1 T-37). Addition,
    /// CONTRACTS.md T01d.
    #[serde(default)]
    pub default_caster: Option<CasterKind>,
    /// Buffs: offered in the Helminth slot (09 §10).
    #[serde(default)]
    pub subsumable: Option<bool>,
    /// Shards.
    #[serde(default)]
    pub color: Option<ShardColor>,
    /// Shards (1.5).
    #[serde(default)]
    pub tauforged_factor: Option<f64>,
    /// Buffs and custom stats: the inspiration's link key (`"o.r"`, `"p.a"`).
    #[serde(default)]
    pub inspiration_key: Option<Box<str>>,
    /// Only for link import; never a data source.
    #[serde(default)]
    pub inspiration_name: Option<Box<str>>,
    /// Non-catalogue items with ranks.
    #[serde(default)]
    pub max_rank: Option<u8>,
    /// Sets: Sacrificial-style member scale (index = pieces − 1).
    #[serde(default)]
    pub member_scale: Option<Vec<f64>>,
    /// Sets: members that never count (`set_noncounting` of 09 E-9.40: Mecha Empowered), by uniqueName; T18 fills
    /// `SetDef.noncounting`. Addition, contract review DC-26.
    #[serde(default)]
    pub noncounting: Vec<UniqueName>,
    /// Buffs: the source is a Warframe ability, an augment or a Helminth-subsumed ability
    /// (`ItemKind::Buff.ability_source`; 04 §4.2 `ability_source` and §5.7 "Source kind"; 05 §13.1). Addition,
    /// contract review DC-16.
    #[serde(default)]
    pub ability_source: Option<bool>,
    #[serde(default, rename = "param")]
    pub params: Vec<ParamSpec>,
    #[serde(default, rename = "effect")]
    pub effects: Vec<EffectDef>,
}

/// One Incarnon family file (`incarnon/<slot>/<family>.toml`, domain-model.md §16.10).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IncarnonFamilyFile {
    #[serde(default = "default_schema")]
    pub schema: u32,
    /// `"furis"`.
    pub family: Box<str>,
    /// Variant column → weapon uniqueName (`furis = "/Lotus/Weapons/Tenno/Pistol/AutoPistol"`).
    pub variants: BTreeMap<Box<str>, UniqueName>,
    /// Native Incarnon weapon (5 evolutions) instead of a Genesis (4), 09 E-9.60.
    #[serde(default)]
    pub native: bool,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
    #[serde(default)]
    pub notes: Box<str>,
    #[serde(default, rename = "perk")]
    pub perks: Vec<CuratedPerk>,
}

/// One perk of an Incarnon family file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CuratedPerk {
    /// `"wfd:perk/furis/haven_foray"`.
    pub id: UniqueName,
    pub name: Box<str>,
    /// Evolution 1..=5 (1 = the form).
    pub tier: u8,
    /// Position in the evolution's perk list.
    pub index: u8,
    pub status: EffectStatus,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
    #[serde(default)]
    pub upstream: Option<Upstream>,
    #[serde(default)]
    pub ignore_numbers: Vec<IgnoreNumber>,
    /// Conditional stat lines of the upstream text that no effect of the file implements: excluded from every
    /// bucket, never applied as unconditional, and listed in the data-build report (04 §5.3 "Uncatalogued
    /// conditional lines"; `DataKey::CheckConditionalLineExcluded`). Addition, CONTRACTS.md T01d.
    #[serde(default)]
    pub excluded_lines: Vec<ExcludedLine>,
    #[serde(default)]
    pub notes: Box<str>,
    #[serde(default)]
    pub scope: Option<Scope>,
    #[serde(default, rename = "effect")]
    pub effects: Vec<EffectDef>,
}

// ---------------------------------------------------------------------------------------------------------------
// Rule-table files (pipeline.md §6.1)
// ---------------------------------------------------------------------------------------------------------------

/// Common header fields of a rule-table file.
macro_rules! rule_file {
    ($(#[$m:meta])* $name:ident { $($(#[$fm:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        $(#[$m])*
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[serde(deny_unknown_fields)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name {
            #[serde(default = "default_schema")]
            pub schema: u32,
            #[serde(default)]
            pub sources: Vec<Box<str>>,
            #[serde(default)]
            pub notes: Box<str>,
            $($(#[$fm])* pub $field: $ty),*
        }

        impl CuratedFile for $name {
            fn schema(&self) -> u32 {
                self.schema
            }
        }
    };
}

rule_file!(
    /// `rules/mult_groups.toml`: `[[group]]` (domain-model.md §14; T03b).
    MultGroupsFile {
        #[serde(default)]
        group: Vec<MultGroupDef>,
    }
);

rule_file!(
    /// `rules/slot_layout.toml`: `[[layout]]` per equipment class (09 §2; T03b).
    SlotLayoutFile {
        #[serde(default)]
        layout: Vec<SlotLayout>,
    }
);

rule_file!(
    /// `rules/riven_traits.toml`: `[[trait]]` (04 §8.1, domain-model.md §16.6; T03b).
    RivenTraitsFile {
        #[serde(default, rename = "trait")]
        traits: Vec<RivenTraitDef>,
    }
);

rule_file!(
    /// `rules/riven_formula.toml` (04 §8.2; T03b).
    RivenFormulaFile {
        formula: RivenFormula,
    }
);

rule_file!(
    /// `rules/body_parts.toml` (02 §4.5, 05 §4.3; T03b).
    BodyPartsFile {
        rules: BodyPartRules,
    }
);

rule_file!(
    /// `rules/special_dr.toml` (05 §4.4-4.5, §15; T03b): `[[dr]]` definitions and `[[enemy]]` hand rows keyed by
    /// 05 §4.1 ids.
    SpecialDrFile {
        #[serde(default)]
        dr: Vec<SpecialDrDef>,
        #[serde(default)]
        enemy: Vec<EnemyRuleRow>,
    }
);

/// A hand-authored enemy row of 05 §4.4-4.5: the listed enemies (05 §4.1 ids, `"infestation:Juggernaut"`) get these
/// fields. Absent fields leave the data value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyRuleRow {
    pub enemies: Vec<Box<str>>,
    #[serde(default)]
    pub immune: Option<StatusSet>,
    /// Per-status caps (`{ COL = 4, BLA = 4 }`).
    #[serde(default)]
    pub caps: BTreeMap<crate::enums::StatusType, u8>,
    /// Cap of every status (`all` of 05 §4.4).
    #[serde(default)]
    pub cap_all: Option<u8>,
    /// Innate DR per layer.
    #[serde(default)]
    pub innate_dr: BTreeMap<crate::enums::Layer, f64>,
    #[serde(default)]
    pub status_dr: Option<f64>,
    #[serde(default)]
    pub type_res: BTreeMap<DamageType, f64>,
    #[serde(default)]
    pub dr_class: Option<DrClass>,
    #[serde(default)]
    pub dr_class_og: Option<DrClass>,
    #[serde(default)]
    pub flags: EnemyFlags,
    /// Id of a `[[dr]]` definition.
    #[serde(default)]
    pub special_dr: Option<Box<str>>,
    /// `Enemy.party_scaling`: P_1 … P_4, or more entries (P_5, P_6); the last entry clamps (contract review M-17).
    #[serde(default)]
    pub party_scaling: Option<Vec<f64>>,
    #[serde(default)]
    pub overguard_context: Option<Box<str>>,
    /// `Enemy.bolstered_og_factor` (05 §7.5; Techrot Babau 0.25). Addition, contract review M-17.
    #[serde(default)]
    pub bolstered_og_factor: Option<f64>,
    #[serde(default)]
    pub eximus_capable: Option<bool>,
    #[serde(default)]
    pub eximus_og: Option<bool>,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

rule_file!(
    /// `rules/toggles.toml`: `[[toggle]]` descriptors of T-01..T-43 (00 §6.1; T03b).
    TogglesFile {
        #[serde(default)]
        toggle: Vec<ToggleDescriptor>,
    }
);

rule_file!(
    /// `rules/factions.toml` (05 §4.2, §5.1, §6.2, 01 §4.10.4, D-008; T03b): faction identities, data-string
    /// normalisation, bane relations, weakness elements, scaling class and documented exceptions to the ingested
    /// modifier table.
    FactionsFile {
        #[serde(default)]
        faction: Vec<FactionRow>,
    }
);

/// One faction of `rules/factions.toml`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FactionRow {
    /// `"kuva_grineer"`.
    pub slug: Box<str>,
    /// English name.
    pub name: Box<str>,
    /// Data strings that map to the faction (05 §4.2: `"The Murmur"` → murmur).
    #[serde(default)]
    pub wiki_names: Vec<Box<str>>,
    /// Factions whose bane bonuses apply (D-008).
    #[serde(default)]
    pub bane_of: Vec<Box<str>>,
    /// Weakness element (01 §4.10.4).
    #[serde(default)]
    pub weakness: Option<DamageType>,
    /// Scaling class name (05 §6.2).
    pub scaling: Box<str>,
    /// Modifier differences from the wiki table that are intended, each with a citation.
    #[serde(default)]
    pub exceptions: BTreeMap<DamageType, f64>,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

rule_file!(
    /// `rules/categories.toml` (09 §3.1-3.2, wiki `Mods/itemCompatibility/data`; T03b): category bits, the
    /// compatibility chains per weapon slot and class, type-name normalisation and stance tags.
    CategoriesFile {
        #[serde(default)]
        category: Vec<CategoryDef>,
        #[serde(default)]
        chain: Vec<ClassChainRow>,
        /// `norm_type` and `melee_type` renames (`"Rifle (No AoE)" = "Rifle"`, `"Dagger" = "Daggers"`).
        #[serde(default)]
        type_alias: BTreeMap<Box<str>, Box<str>>,
        /// stance_tag(c) per melee class (09 §3.2: `"Two-Handed Nikana" = "LONG_KATANA_STANCE"`).
        #[serde(default)]
        stance_tag: BTreeMap<Box<str>, Box<str>>,
        /// Stance-tag normalisation applied to mod and weapon tags before comparison (09 §3.2, EC-09-21:
        /// `"BAYONET" = "BAYONET_STANCE"`). Addition, CONTRACTS.md T03b review fixes.
        #[serde(default)]
        stance_tag_rename: BTreeMap<Box<str>, Box<str>>,
    }
);

/// One row of the compatibility-chain table of 09 §3.1.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ClassChainRow {
    /// `M:Weapons.Slot`.
    pub slot: Box<str>,
    /// `M:Weapons.Class`; empty = any class of the slot.
    #[serde(default)]
    pub class: Option<Box<str>>,
    /// Weapons with their own chain (Exalted rows: Artemis Bow, Neutralizer …).
    #[serde(default)]
    pub weapons: Vec<UniqueName>,
    /// Category names of the chain.
    pub chain: Vec<Box<str>>,
    #[serde(default)]
    pub source: Box<str>,
}

rule_file!(
    /// `rules/helminth.toml` (09 E-9.32; T03b).
    HelminthFile {
        rules: HelminthRules,
    }
);

rule_file!(
    /// `rules/frames.toml` (09 §2.2, AD-26; T03b): aura slot overrides and variant links.
    FramesFile {
        #[serde(default)]
        frame: Vec<FrameRuleRow>,
    }
);

/// One frame override of `rules/frames.toml`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FrameRuleRow {
    pub id: UniqueName,
    /// 2 for Jade.
    #[serde(default)]
    pub aura_slots: Option<u8>,
    /// Prime and Umbra variants (09 E-9.32 H6).
    #[serde(default)]
    pub variant_of: Option<UniqueName>,
    /// The frame passive's effects (`Frame::passives`; 04 §5.11, §5.6): `[[frame.passive]]`. Addition, contract
    /// review DC-14.
    #[serde(default, rename = "passive")]
    pub passives: Vec<EffectDef>,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

rule_file!(
    /// `rules/game_constants.toml` (`Rules.game_constants`): named game values that no dedicated table carries (Steel
    /// Path ×2.5, Eximus steps …), one `[[constant]]` row each with its citation, so that T18 never hard-codes a game
    /// value (I-1). Owner: T03b (CONTRACTS.md). Addition, contract review DC-19.
    GameConstantsFile {
        #[serde(default)]
        constant: Vec<GameConstantRow>,
    }
);

/// One named game value of `rules/game_constants.toml`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GameConstantRow {
    /// `"<equation id>.<name>"`, the key of `Rules.game_constants` (`"E-5.4.steel_path_mult"`).
    pub key: Box<str>,
    pub value: f64,
    /// Citation (spec equation or wiki page).
    #[serde(default)]
    pub source: Box<str>,
}

rule_file!(
    /// `rules/attack_roles.toml` (pipeline.md §5.3; T18): regex rules on attack names → roles.
    AttackRolesFile {
        #[serde(default)]
        rule: Vec<AttackRoleRule>,
    }
);

/// One attack-role rule.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackRoleRule {
    /// Regex on the wiki `AttackName`.
    pub pattern: Box<str>,
    /// Only attacks of this shot type.
    #[serde(default)]
    pub shot: Option<ShotType>,
    /// Only child (`true`) or parent (`false`) attacks.
    #[serde(default)]
    pub child: Option<bool>,
    pub roles: AttackRoles,
    #[serde(default)]
    pub source: Box<str>,
}

rule_file!(
    /// `rules/stat_lines.toml` (pipeline.md §6.2 tier 1, 04 §5.1; T19): anchored stat-line patterns → effect
    /// templates, scope suffixes and colour tokens.
    StatLinesFile {
        #[serde(default)]
        line: Vec<StatLineRule>,
        #[serde(default)]
        suffix: Vec<StatLineSuffix>,
        /// DE colour tokens (`"<DT_FIRE_COLOR>"`) → damage type.
        #[serde(default)]
        color_token: BTreeMap<Box<str>, DamageType>,
    }
);

/// One stat-line pattern (`^\+(?P<v>[\d.]+)% Damage$` → `base_dmg`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatLineRule {
    pub id: Box<str>,
    /// Anchored regex with a named group `v` for the value.
    pub pattern: Box<str>,
    pub unit: StatLineUnit,
    /// The effect without value (the line value enters as `{ param = "value" }`).
    pub template: EffectDef,
    #[serde(default)]
    pub source: Box<str>,
}

/// How a stat-line value converts to model units.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum StatLineUnit {
    /// `x %` → x/100.
    Percent,
    /// `×m` → m − 1.
    Multiplier,
    /// Raw number.
    Absolute,
}

/// A suffix that adds a scope factor (`(x2 for Bows)`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatLineSuffix {
    pub pattern: Box<str>,
    pub scope_factor: crate::effect::ScopeFactor,
    #[serde(default)]
    pub source: Box<str>,
}

// ---- rules/melee/*.toml (07 §4.4; T03c) ----

rule_file!(
    /// `rules/melee/heavy_class.toml`: table 9-A (07 §9.1).
    HeavyClassFile {
        #[serde(default)]
        row: Vec<HeavyClassRow>,
    }
);

rule_file!(
    /// `rules/melee/heavy_kappa.toml`: table 9-B members (07 E-7.41).
    HeavyKappaFile {
        #[serde(default)]
        row: Vec<HeavyKappaRow>,
    }
);

/// One 9-B row.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HeavyKappaRow {
    pub weapon: UniqueName,
    pub kappa: f64,
    #[serde(default)]
    pub source: Box<str>,
}

rule_file!(
    /// `rules/melee/attack_triggers.toml`: table 9-C (07 §9.7).
    AttackTriggersFile {
        #[serde(default)]
        row: Vec<MeleeTriggerRow>,
    }
);

rule_file!(
    /// `rules/melee/stance_co.toml`: table 9-D (07 §7.6; 63 rows).
    StanceCoFile {
        #[serde(default)]
        row: Vec<StanceCoRow>,
    }
);

rule_file!(
    /// `rules/melee/finishers.toml`: table 9-E (07 §9.11).
    FinishersFile {
        #[serde(default)]
        row: Vec<FinisherRow>,
    }
);

rule_file!(
    /// `rules/melee/combo_specials.toml`: k_tier, M_cap, M_bcap, D0_cb overrides, innate Initial Combo (07 §6).
    ComboSpecialsFile {
        #[serde(default)]
        row: Vec<ComboSpecialsRow>,
    }
);

/// One combo-specials row.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComboSpecialsRow {
    pub weapon: UniqueName,
    #[serde(default)]
    pub k_tier: Option<u16>,
    #[serde(default)]
    pub m_cap: Option<u8>,
    #[serde(default)]
    pub m_bcap: Option<u8>,
    #[serde(default)]
    pub combo_dur: Option<f64>,
    #[serde(default)]
    pub initial_combo: Option<f64>,
    #[serde(default)]
    pub source: Box<str>,
}

impl ComboSpecialsRow {
    /// The specials of the row.
    pub fn specials(&self) -> crate::weapon::ComboSpecials {
        crate::weapon::ComboSpecials {
            k_tier: self.k_tier,
            m_cap: self.m_cap,
            m_bcap: self.m_bcap,
            combo_dur: self.combo_dur,
            initial_combo: self.initial_combo,
        }
    }
}

// ---- overrides/ and aliases/ (pipeline.md §5.3, §6.1; read by T18) ----

rule_file!(
    /// `overrides/weapons/<slot>/<slug>.toml`: the hand tables of 06 §4.3 for one weapon and its innate passives
    /// (owners T20, T21, T22, T29 by slot; read by T18). A hand entry overrides every generic data rule of 06 (06 §4.3
    /// "Precedence"). Absent fields keep the data value. Parent links, roles, comb groups and the 02 §4.10 attack flags
    /// are in [`AttackOverrideFile`]. Addition, contract review M-6, DC-18.
    WeaponOverrideFile {
        /// The weapon (PE uniqueName).
        id: UniqueName,
        /// Weapon-level trigger (06 §4.2 trigger overrides).
        #[serde(default)]
        trigger: Option<Trigger>,
        /// Magazine and ammo mode (06 §5.6-5.7, 11.3 exalted ammo mode: infinite, magazine or battery).
        #[serde(default)]
        magazine: Option<MagazineOverride>,
        /// Reload model: per-shell reload `{ per_shell = { start_end, per_shell } }` (06 §5.7), no-reload weapons
        /// `"none"` (Epitaph, 06 §6.4).
        #[serde(default)]
        reload: Option<ReloadSpec>,
        /// The weapon is a bow for 06's bow rules beyond `Class = Bow` (Artemis Bow, Artemis Bow Prime; 06 §5.1,
        /// §11.3); maps onto `WeaponTraits::BOW` (contract review M-6).
        #[serde(default)]
        bow: Option<bool>,
        /// Volley groups by data `AttackName` (Arbucep "1st Attack" … "6th Attack"; 06 §6.8, E-6.14b).
        #[serde(default)]
        volley: Vec<Vec<Box<str>>>,
        /// Weapon-level innate effects appended to `Weapon.passives` (Gotva Prime, Sagek Prime, Arca Titron, Torid;
        /// 04 §5.12): `[[passive]]`.
        #[serde(default, rename = "passive")]
        passives: Vec<EffectDef>,
        /// Per-attack firing overrides: `[[attack]]`.
        #[serde(default, rename = "attack")]
        attacks: Vec<AttackFireOverride>,
    }
);

/// Magazine override of a [`WeaponOverrideFile`]; absent fields keep the data value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MagazineOverride {
    /// Magazine size Mag0 (rounds).
    #[serde(default)]
    pub size: Option<f64>,
    /// No ammo consumption (06 §5.6; exalted `infinite`).
    #[serde(default)]
    pub infinite: Option<bool>,
    /// Battery recharge instead of a reload (06 §5.7; exalted `battery`).
    #[serde(default)]
    pub battery: Option<Battery>,
}

/// Firing overrides of one attack (06 §4.3 per-attack rows), keyed by the data `AttackName`; absent fields keep the
/// data value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackFireOverride {
    /// The data `AttackName`.
    pub name: Box<str>,
    /// Trigger of the attack (06 §4.2).
    #[serde(default)]
    pub trigger: Option<Trigger>,
    /// Spool cadence s_sp, Δ_sp (06 §7.3).
    #[serde(default)]
    pub spool: Option<Spool>,
    /// Kohm family (06 §7.4).
    #[serde(default)]
    pub kohm: Option<KohmSpec>,
    /// Beam ramp start s_rmp (06 §7.1; default 0.2).
    #[serde(default)]
    pub ramp_start: Option<f64>,
    /// Semi-auto cap (Komorex ≈ 8; 06 §5.2).
    #[serde(default)]
    pub semi_cap: Option<f64>,
    /// Charge time CT0 when the data lacks it (Vulcax 1.0; 06 §5.3, §4.2).
    #[serde(default)]
    pub charge_time: Option<f64>,
    /// Charge time not affected by mods (Phantasma; 06 E-6.4).
    #[serde(default)]
    pub charge_fixed: Option<bool>,
    /// Incarnon form magazine without data (Miter 20, Stug 120; 06 §6.6).
    #[serde(default)]
    pub magazine_override: Option<f64>,
    /// Gauge-gated attack (06 §6.7, E-6.14a; also adds the role `gauge_gated`).
    #[serde(default)]
    pub gauge: Option<GaugeSpec>,
    /// Sniper combo (06 §7.2; exclusions: Arca Scisco, Knell with `min` absent).
    #[serde(default)]
    pub sniper_combo: Option<SniperComboSpec>,
    /// Firing roles added to the attack (`reload_triggered`, `whole_magazine`, `returning`; 06 §9.5, §6.4).
    #[serde(default)]
    pub roles: AttackRoles,
    /// Attack-level innate effects appended to `Attack.passives`: `[[attack.passive]]`.
    #[serde(default, rename = "passive")]
    pub passives: Vec<EffectDef>,
}

rule_file!(
    /// `overrides/attacks/<weapon-slug>.toml` (owner and reader T18): the hand-authored attack data of 02 §4.10
    /// (`[[attack]]` records with `[[attack.forced]]`), parent links for the inconsistent `AttackParentIndex` cases
    /// (06 E-6.23, §9.1), roles of gated children (06 §9.1-9.2) and T-22 comb groups (pipeline.md §5.3). Firing
    /// overrides are in [`WeaponOverrideFile`]. Addition, contract review M-6, DC-18.
    AttackOverrideFile {
        /// The weapon (PE uniqueName).
        weapon: UniqueName,
        #[serde(default, rename = "attack")]
        attacks: Vec<AttackRecord>,
        /// T-22 combinable groups and parent + child sums by data `AttackName` (CD-12).
        #[serde(default, rename = "group")]
        groups: Vec<AttackGroupOverride>,
    }
);

/// One 02 §4.10 attack record, keyed by the data `AttackName`; absent fields keep the data value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackRecord {
    /// The data `AttackName` (02 §4.10 `key` = weapon slug + AttackName).
    pub name: Box<str>,
    /// E-2.10 1×-headshot attack (`HeadshotRule::OneX`; OR-joined with data `ExtraHeadshotDmg = −2`).
    #[serde(default)]
    pub headshot_1x: Option<bool>,
    /// M_hsu (`Attack.hs_unique`; Cernos Prime 1.5).
    #[serde(default)]
    pub hs_unique: Option<f64>,
    /// E-2.1 MS = 1 (role `speargun_throw`).
    #[serde(default)]
    pub speargun_throw: Option<bool>,
    /// E-2.8, E-2.10 (role `radial_parent_part`; only on a child of a direct attack, data-build check).
    #[serde(default)]
    pub radial_parent_part: Option<bool>,
    /// E-2.10 (role `torso_only`).
    #[serde(default)]
    pub torso_only: Option<bool>,
    /// Conditional or corrected forced procs (E-2.14): `[[attack.forced]]`.
    #[serde(default, rename = "forced")]
    pub forced: Vec<ForcedProcRule>,
    /// Parent by data `AttackName`, replacing `AttackParentIndex` (06 E-6.23, §9.1).
    #[serde(default)]
    pub parent: Option<Box<str>>,
    /// The attack has no parent although the data links one (06 §9.1).
    #[serde(default)]
    pub no_parent: bool,
    /// Roles added (`zoom_only`, `headshot_only` of 06 §9.1-9.2, and `stun_gated`, the stun gate of 06 E-6.24).
    #[serde(default)]
    pub roles: AttackRoles,
}

/// An attack group of an [`AttackOverrideFile`] (`AttackGroup` by data `AttackName`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackGroupOverride {
    pub members: Vec<Box<str>>,
    /// T-22.
    #[serde(default)]
    pub combinable: bool,
    /// Parent + child sum.
    #[serde(default)]
    pub summed: bool,
}

rule_file!(
    /// `overrides/enemies/<slug>.toml` (owner T29, read by T18): the hand part records of 05 §4.3 and §4.5 for the
    /// listed enemies (Juggernaut maw h = 4, Hemocyte's default part, Isolator Bursa front, Scaldra `part_dr`).
    /// Record-level hand fields (immunities, caps, DR, flags) are `rules/special_dr.toml` rows ([`EnemyRuleRow`]).
    /// Addition, contract review M-6, DC-18.
    EnemyOverrideFile {
        /// 05 §4.1 ids (`"infestation:Juggernaut"`).
        enemies: Vec<Box<str>>,
        /// `true`: the parts below replace the parsed parts; `false` (default): each patches the parsed part of the
        /// same name, or is added when none has it.
        #[serde(default)]
        replace_parts: bool,
        /// Name of the default part (`BodyPart.is_default`; used for `body` hits and radial instances).
        #[serde(default)]
        default_part: Option<Box<str>>,
        #[serde(default, rename = "part")]
        parts: Vec<PartOverride>,
    }
);

/// One hand part record (fields of `BodyPart`, 05 §4.3); absent fields keep the parsed value (or the 05 §4.3 default
/// for a new part).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartOverride {
    /// Part name, matched to the parsed part of the same name.
    pub name: Box<str>,
    #[serde(default)]
    pub column: Option<PartColumn>,
    /// P_part.
    #[serde(default)]
    pub part_mult: Option<f64>,
    #[serde(default)]
    pub is_weak_point: Option<bool>,
    #[serde(default)]
    pub is_head: Option<bool>,
    /// h.
    #[serde(default)]
    pub headcrit: Option<f64>,
    /// π_p (02 E-2.6).
    #[serde(default)]
    pub tier_add: Option<u8>,
    #[serde(default)]
    pub crit_disabled: Option<bool>,
    #[serde(default)]
    pub wp_bonus_ok: Option<bool>,
    #[serde(default)]
    pub part_dr: Option<f64>,
    #[serde(default)]
    pub status_ok: Option<bool>,
    #[serde(default)]
    pub non_body: Option<bool>,
    #[serde(default)]
    pub no_armor: Option<bool>,
    #[serde(default)]
    pub no_faction_mod: Option<bool>,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

rule_file!(
    /// `overrides/progenitor.toml` (owner T29, read by T18; T-24, 09 §14): the progenitor spec every adversary weapon
    /// gets (`Weapon.progenitor`), and per-weapon rows for adversary weapons the data's traits miss (Kuva Ghoulsaw,
    /// EC-09-51) or that take another spec. Addition, contract review M-6, DC-18.
    ProgenitorFile {
        /// Elements (E-9.70 V2) and value range and default (fractions; T-24 default 0.60).
        default: ProgenitorSpec,
        #[serde(default, rename = "weapon")]
        weapons: Vec<ProgenitorRow>,
    }
);

/// One weapon row of `overrides/progenitor.toml`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProgenitorRow {
    pub id: UniqueName,
    /// The weapon's spec; `None` = the file's `default`.
    #[serde(default)]
    pub spec: Option<ProgenitorSpec>,
    #[serde(default)]
    pub source: Box<str>,
}

rule_file!(
    /// `aliases/enemies.toml` (owner T29, read by T18): wiki enemy renames. An enemy's uniqueName is synthetic
    /// (`wfd:enemy/<wiki sub-module>/<slug of the wiki Name>`, domain-model.md §2), so a renamed wiki entry keeps its
    /// stable key only through a row here. Addition, contract review DC-18.
    EnemyAliasFile {
        #[serde(default, rename = "alias")]
        aliases: Vec<EnemyAlias>,
    }
);

/// One enemy rename.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyAlias {
    /// The uniqueName the enemy keeps (its old synthetic name).
    pub unique_name: UniqueName,
    /// The current wiki entry, as a 05 §4.1 id (`"<sub-module>:<Name>"`).
    pub wiki: Box<str>,
    #[serde(default)]
    pub source: Box<str>,
}

/// Every rule table of `rules/` (absent files are `None`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CuratedRules {
    pub mult_groups: Option<MultGroupsFile>,
    pub slot_layout: Option<SlotLayoutFile>,
    pub riven_traits: Option<RivenTraitsFile>,
    pub riven_formula: Option<RivenFormulaFile>,
    pub body_parts: Option<BodyPartsFile>,
    pub special_dr: Option<SpecialDrFile>,
    pub toggles: Option<TogglesFile>,
    pub factions: Option<FactionsFile>,
    pub categories: Option<CategoriesFile>,
    pub helminth: Option<HelminthFile>,
    pub frames: Option<FramesFile>,
    pub attack_roles: Option<AttackRolesFile>,
    pub stat_lines: Option<StatLinesFile>,
    pub game_constants: Option<GameConstantsFile>,
    pub melee: CuratedMeleeRules,
}

/// Every table of `rules/melee/`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CuratedMeleeRules {
    pub heavy_class: Option<HeavyClassFile>,
    pub heavy_kappa: Option<HeavyKappaFile>,
    pub attack_triggers: Option<AttackTriggersFile>,
    pub stance_co: Option<StanceCoFile>,
    pub finishers: Option<FinishersFile>,
    pub combo_specials: Option<ComboSpecialsFile>,
}

/// A loaded `data/curated/` tree: files in path order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CuratedTree {
    /// Item files with their paths relative to the tree root.
    pub items: Vec<(PathBuf, CuratedItemFile)>,
    pub incarnon: Vec<(PathBuf, IncarnonFamilyFile)>,
    pub rules: CuratedRules,
}

/// One problem of a curated file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuratedError {
    pub path: PathBuf,
    pub message: String,
}

impl fmt::Display for CuratedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

/// Every problem found while loading a tree.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CuratedErrors(pub Vec<CuratedError>);

impl fmt::Display for CuratedErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in &self.0 {
            writeln!(f, "{e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for CuratedErrors {}

/// Parses one TOML document into an authoring type, with the path in the error.
pub fn parse_toml<T: DeserializeOwned>(path: &Path, text: &str) -> Result<T, CuratedError> {
    toml::from_str(text).map_err(|e| CuratedError {
        path: path.to_owned(),
        message: e.to_string(),
    })
}

/// Reads and parses one file.
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T, CuratedError> {
    let text = std::fs::read_to_string(path).map_err(|e| CuratedError {
        path: path.to_owned(),
        message: format!("cannot read: {e}"),
    })?;
    parse_toml(path, &text)
}

/// Directories of `data/curated/` that are not loaded here: their readers parse them with the types of this module
/// (pipeline.md §6.1, work-breakdown.md §8). `overrides/`: `attacks/` and `co_names.toml` by T18, `weapons/<slot>/` by
/// T20 (primary), T21 (secondary), T22 (melee) and T29 (archgun, companion, exalted), `enemies/` and
/// `progenitor.toml` by T29; all read by T18. `aliases/`: `inspiration.toml` by T10, `enemies.toml` by T29 (read by
/// T18). Their files are not read here.
pub const UNLOADED_DIRS: [&str; 2] = ["overrides", "aliases"];

/// Files at the top of `data/curated/` that belong to other tasks and are not loaded here: `priority.toml` (the
/// curation priority list, pipeline.md §6.3, T19) and `schema.json` (the generated editor schema, pipeline.md §6.1,
/// T01b). Their content is not read.
pub const UNLOADED_TOP_FILES: [&str; 2] = ["priority.toml", "schema.json"];

/// Rule tables of `rules/` (pipeline.md §6.1).
const RULE_TABLES: [&str; 14] = [
    "mult_groups.toml",
    "slot_layout.toml",
    "riven_traits.toml",
    "riven_formula.toml",
    "body_parts.toml",
    "special_dr.toml",
    "toggles.toml",
    "factions.toml",
    "categories.toml",
    "helminth.toml",
    "frames.toml",
    "attack_roles.toml",
    "stat_lines.toml",
    "game_constants.toml",
];

/// Rule tables of `rules/melee/` (07 §4.4 tables 9-A to 9-E and the specials).
const MELEE_TABLES: [&str; 6] = [
    "heavy_class.toml",
    "heavy_kappa.toml",
    "attack_triggers.toml",
    "stance_co.toml",
    "finishers.toml",
    "combo_specials.toml",
];

/// A curated file type with the `schema` header field (domain-model.md §15.1).
pub trait CuratedFile: DeserializeOwned {
    /// The file's `schema` value; the loader accepts [`CURATED_SCHEMA`] only.
    fn schema(&self) -> u32;
}

impl CuratedFile for CuratedItemFile {
    fn schema(&self) -> u32 {
        self.schema
    }
}

impl CuratedFile for IncarnonFamilyFile {
    fn schema(&self) -> u32 {
        self.schema
    }
}

/// Parses one curated file and checks its `schema` header.
pub fn parse_curated<T: CuratedFile>(path: &Path, text: &str) -> Result<T, CuratedError> {
    let f: T = parse_toml(path, text)?;
    if f.schema() != CURATED_SCHEMA {
        return Err(CuratedError {
            path: path.to_owned(),
            message: format!(
                "unsupported `schema = {}` (this build reads schema {CURATED_SCHEMA})",
                f.schema()
            ),
        });
    }
    Ok(f)
}

/// Where a file of the tree goes.
enum Dest {
    Item(CuratedKind),
    Incarnon,
    Rule(&'static str),
    Melee(&'static str),
}

/// What the loader does with the file at `rel` (relative to the tree root): `Ok(Some(dest))` parses it, `Ok(None)`
/// skips it, `Err((path, message))` reports it (the path may be a directory, so one error covers all its files).
fn classify(rel: &Path) -> Result<Option<Dest>, (PathBuf, String)> {
    let mut comps = Vec::new();
    for c in rel.components() {
        match c {
            std::path::Component::Normal(n) => comps.push(n.to_string_lossy().into_owned()),
            _ => {
                return Err((
                    rel.to_owned(),
                    "not a relative path inside data/curated".to_owned(),
                ));
            }
        }
    }
    // Hidden files and directories (`.gitkeep`, editor folders) are not data.
    if comps.is_empty() || comps.iter().any(|c| c.starts_with('.')) {
        return Ok(None);
    }
    let is_toml = rel.extension().is_some_and(|e| e == "toml");
    let not_toml = || {
        Err((
            rel.to_owned(),
            "not a .toml file (curated data files are `<name>.toml`)".to_owned(),
        ))
    };
    let name = comps[comps.len() - 1].as_str();
    let top = comps[0].as_str();
    if comps.len() == 1 {
        if UNLOADED_TOP_FILES.contains(&name) {
            return Ok(None);
        }
        return Err((
            rel.to_owned(),
            format!(
                "unexpected file at the top level of data/curated (expected a kind directory, `incarnon/`, `rules/`, \
                 {UNLOADED_DIRS:?} or one of {UNLOADED_TOP_FILES:?})"
            ),
        ));
    }
    if UNLOADED_DIRS.contains(&top) {
        return Ok(None);
    }
    if top == "rules" {
        if comps.len() == 2 {
            return match RULE_TABLES.iter().find(|t| **t == name) {
                Some(t) => Ok(Some(Dest::Rule(t))),
                None if is_toml => Err((
                    rel.to_owned(),
                    format!("unknown rule table (pipeline.md §6.1: one of {RULE_TABLES:?})"),
                )),
                None => not_toml(),
            };
        }
        if comps[1] != "melee" {
            return Err((
                Path::new("rules").join(&comps[1]),
                "unknown directory in rules/ (only `melee/`)".to_owned(),
            ));
        }
        if comps.len() > 3 {
            return Err((
                Path::new("rules/melee").join(&comps[2]),
                "unknown directory in rules/melee/".to_owned(),
            ));
        }
        return match MELEE_TABLES.iter().find(|t| **t == name) {
            Some(t) => Ok(Some(Dest::Melee(t))),
            None if is_toml => Err((
                rel.to_owned(),
                format!("unknown melee rule table (07 §4.4: one of {MELEE_TABLES:?})"),
            )),
            None => not_toml(),
        };
    }
    let dest = if top == "incarnon" {
        Dest::Incarnon
    } else if let Some(kind) = CuratedKind::ALL.iter().find(|k| k.dir() == top) {
        Dest::Item(*kind)
    } else {
        return Err((
            PathBuf::from(top),
            "unknown directory in data/curated (pipeline.md §6.1)".to_owned(),
        ));
    };
    if !is_toml {
        return not_toml();
    }
    Ok(Some(dest))
}

/// Loads a `data/curated/` tree given as `(path relative to the tree root, file content)` pairs, in any order: every
/// item file of the kind directories, every Incarnon family file and every known rule table. This is the entry point
/// for trees that are not a directory on disk (the in-app refresh reads `data/curated/` embedded with `include_dir!`,
/// pipeline.md §2 `InputTree::Embedded`, AD-13); [`load_curated_dir`] reads a directory and calls it.
///
/// Skipped (content not read): [`UNLOADED_DIRS`], [`UNLOADED_TOP_FILES`], hidden files and directories (a name
/// starting with `.`). Every other unexpected file is an error: a file that is not `.toml` (`x.toml.bak`), an unknown
/// directory or rule table, a file at the top level. Each file must have `schema = 1` ([`CURATED_SCHEMA`]), parse
/// into its type, and an item file's `kind` must match its directory. All problems are collected and returned
/// together; items and Incarnon files are returned in path order.
pub fn load_curated_files<I>(files: I) -> Result<CuratedTree, CuratedErrors>
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let mut files: Vec<(PathBuf, String)> = files.into_iter().collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut tree = CuratedTree::default();
    let mut errors: Vec<CuratedError> = Vec::new();
    let report = |errors: &mut Vec<CuratedError>, path: PathBuf, message: String| {
        if !errors
            .iter()
            .any(|e| e.path == path && e.message == message)
        {
            errors.push(CuratedError { path, message });
        }
    };
    for (path, text) in files {
        let dest = match classify(&path) {
            Ok(Some(d)) => d,
            Ok(None) => continue,
            Err((p, m)) => {
                report(&mut errors, p, m);
                continue;
            }
        };
        match dest {
            Dest::Item(kind) => match parse_curated::<CuratedItemFile>(&path, &text) {
                Ok(f) if f.kind != kind => report(
                    &mut errors,
                    path,
                    format!(
                        "kind `{:?}` does not match directory `{}`",
                        f.kind,
                        kind.dir()
                    ),
                ),
                Ok(f) => tree.items.push((path, f)),
                Err(e) => errors.push(e),
            },
            Dest::Incarnon => match parse_curated::<IncarnonFamilyFile>(&path, &text) {
                Ok(f) => tree.incarnon.push((path, f)),
                Err(e) => errors.push(e),
            },
            Dest::Rule(table) => {
                if let Err(e) = load_rule(&mut tree.rules, table, &path, &text) {
                    errors.push(e);
                }
            }
            Dest::Melee(table) => {
                if let Err(e) = load_melee_rule(&mut tree.rules.melee, table, &path, &text) {
                    errors.push(e);
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(tree)
    } else {
        Err(CuratedErrors(errors))
    }
}

/// Stores a parsed rule table in its slot; a second file for the same table is an error.
fn store<T: CuratedFile>(
    slot: &mut Option<T>,
    path: &Path,
    text: &str,
) -> Result<(), CuratedError> {
    let f = parse_curated::<T>(path, text)?;
    if slot.is_some() {
        return Err(CuratedError {
            path: path.to_owned(),
            message: "duplicate rule table".to_owned(),
        });
    }
    *slot = Some(f);
    Ok(())
}

fn load_rule(
    rules: &mut CuratedRules,
    table: &str,
    path: &Path,
    text: &str,
) -> Result<(), CuratedError> {
    match table {
        "mult_groups.toml" => store(&mut rules.mult_groups, path, text),
        "slot_layout.toml" => store(&mut rules.slot_layout, path, text),
        "riven_traits.toml" => store(&mut rules.riven_traits, path, text),
        "riven_formula.toml" => store(&mut rules.riven_formula, path, text),
        "body_parts.toml" => store(&mut rules.body_parts, path, text),
        "special_dr.toml" => store(&mut rules.special_dr, path, text),
        "toggles.toml" => store(&mut rules.toggles, path, text),
        "factions.toml" => store(&mut rules.factions, path, text),
        "categories.toml" => store(&mut rules.categories, path, text),
        "helminth.toml" => store(&mut rules.helminth, path, text),
        "frames.toml" => store(&mut rules.frames, path, text),
        "attack_roles.toml" => store(&mut rules.attack_roles, path, text),
        "stat_lines.toml" => store(&mut rules.stat_lines, path, text),
        "game_constants.toml" => store(&mut rules.game_constants, path, text),
        other => unreachable!("RULE_TABLES lists {other}"),
    }
}

fn load_melee_rule(
    melee: &mut CuratedMeleeRules,
    table: &str,
    path: &Path,
    text: &str,
) -> Result<(), CuratedError> {
    match table {
        "heavy_class.toml" => store(&mut melee.heavy_class, path, text),
        "heavy_kappa.toml" => store(&mut melee.heavy_kappa, path, text),
        "attack_triggers.toml" => store(&mut melee.attack_triggers, path, text),
        "stance_co.toml" => store(&mut melee.stance_co, path, text),
        "finishers.toml" => store(&mut melee.finishers, path, text),
        "combo_specials.toml" => store(&mut melee.combo_specials, path, text),
        other => unreachable!("MELEE_TABLES lists {other}"),
    }
}

/// Loads a whole `data/curated/` tree from a directory: walks it (not into [`UNLOADED_DIRS`] or hidden directories),
/// reads the files [`load_curated_files`] parses and passes every other file by name only, so its rules (skipped
/// files, errors for unexpected ones) apply unchanged. Unreadable files and directories are errors too.
pub fn load_curated_dir(root: &Path) -> Result<CuratedTree, CuratedErrors> {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    let mut stack = vec![root.to_owned()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) => {
                errors.push(CuratedError {
                    // The root itself is reported by its full path.
                    path: if dir == root {
                        root.to_owned()
                    } else {
                        rel(root, &dir)
                    },
                    message: format!("cannot read directory: {e}"),
                });
                continue;
            }
        };
        for entry in entries {
            let path = match entry {
                Ok(e) => e.path(),
                Err(e) => {
                    errors.push(CuratedError {
                        path: rel(root, &dir),
                        message: format!("cannot read directory: {e}"),
                    });
                    continue;
                }
            };
            let r = rel(root, &path);
            let hidden = path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'));
            if path.is_dir() {
                let unloaded =
                    r.components().count() == 1 && UNLOADED_DIRS.iter().any(|d| r == Path::new(d));
                if !hidden && !unloaded {
                    stack.push(path);
                }
                continue;
            }
            let text = if matches!(classify(&r), Ok(Some(_))) {
                match std::fs::read_to_string(&path) {
                    Ok(t) => t,
                    Err(e) => {
                        errors.push(CuratedError {
                            path: r,
                            message: format!("cannot read: {e}"),
                        });
                        continue;
                    }
                }
            } else {
                String::new()
            };
            files.push((r, text));
        }
    }
    match load_curated_files(files) {
        Ok(tree) if errors.is_empty() => Ok(tree),
        Ok(_) => Err(CuratedErrors(errors)),
        Err(CuratedErrors(more)) => {
            errors.extend(more);
            errors.sort_by(|a, b| a.path.cmp(&b.path));
            Err(CuratedErrors(errors))
        }
    }
}

fn rel(root: &Path, p: &Path) -> PathBuf {
    p.strip_prefix(root)
        .map(Path::to_owned)
        .unwrap_or_else(|_| p.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The override and alias files of pipeline.md §6.1 and `rules/game_constants.toml` parse into their authoring
    /// types and reject unknown fields (contract review M-6, DC-18, DC-19).
    #[test]
    fn override_and_alias_files_parse() {
        let p = Path::new("overrides/weapons/primary/example.toml");
        let w: WeaponOverrideFile = parse_curated(
            p,
            r#"
id = "/Lotus/Weapons/Example/Rifle"
sources = ["06 §4.3"]
reload = "none"
magazine = { infinite = true }
volley = [["1st Attack", "2nd Attack"]]

[[passive]]
stat = "cc_set"
value = 3.0
when = { trigger = "on_any_status", chance = 0.15 }
stacks = { max = 1, consume_on = "next_shot" }

[[attack]]
name = "Normal Attack"
trigger = "auto_spool"
spool = { shots = 9, start_frac = 0.2 }
roles = ["whole_magazine"]
"#,
        )
        .unwrap();
        assert_eq!(w.reload, Some(ReloadSpec::None));
        assert_eq!(w.attacks[0].trigger, Some(Trigger::AutoSpool));
        assert_eq!(w.passives.len(), 1);
        let a: AttackOverrideFile = parse_curated(
            Path::new("overrides/attacks/example.toml"),
            r#"
weapon = "/Lotus/Weapons/Example/Rifle"

[[attack]]
name = "Headshot Explosion"
radial_parent_part = true
parent = "Normal Attack"
roles = ["headshot_only"]

[[attack.forced]]
condition = { distance_max = 10.0 }
procs = [{ type = "SLA", count = 1 }]

[[group]]
members = ["Normal Attack", "Headshot Explosion"]
combinable = true
"#,
        )
        .unwrap();
        assert_eq!(a.attacks[0].forced.len(), 1);
        assert_eq!(a.groups[0].members.len(), 2);
        let e: EnemyOverrideFile = parse_curated(
            Path::new("overrides/enemies/example.toml"),
            r#"
enemies = ["infestation:Juggernaut"]
default_part = "Body"

[[part]]
name = "Maw"
is_weak_point = true
headcrit = 4.0
"#,
        )
        .unwrap();
        assert_eq!(e.parts[0].headcrit, Some(4.0));
        let g: ProgenitorFile = parse_curated(
            Path::new("overrides/progenitor.toml"),
            r#"
default = { elements = ["IMP", "HEA", "COL", "ELE", "TOX", "MAG", "RAD"], min = 0.25, max = 0.6, default = 0.6 }

[[weapon]]
id = "/Lotus/Weapons/Example/Ghoulsaw"
"#,
        )
        .unwrap();
        assert_eq!(g.default.elements.len(), 7);
        let al: EnemyAliasFile = parse_curated(
            Path::new("aliases/enemies.toml"),
            "[[alias]]\nunique_name = \"wfd:enemy/grineer/old_name\"\nwiki = \"grineer:New Name\"\n",
        )
        .unwrap();
        assert_eq!(al.aliases.len(), 1);
        let c: GameConstantsFile = parse_curated(
            Path::new("rules/game_constants.toml"),
            "[[constant]]\nkey = \"E-5.4.steel_path_mult\"\nvalue = 2.5\nsource = \"05 E-5.4\"\n",
        )
        .unwrap();
        assert_eq!(c.constant[0].value, 2.5);
        assert!(
            parse_curated::<EnemyOverrideFile>(
                Path::new("overrides/enemies/bad.toml"),
                "enemies = []\n[[part]]\nname = \"Head\"\nbogus = 1\n",
            )
            .is_err()
        );
        // The loader reads `rules/game_constants.toml`.
        let tree = load_curated_files(vec![(
            PathBuf::from("rules/game_constants.toml"),
            "[[constant]]\nkey = \"E-5.5.c_exi\"\nvalue = 0.25\n".to_owned(),
        )])
        .unwrap();
        assert_eq!(tree.rules.game_constants.unwrap().constant.len(), 1);
    }
}
