//! Authoring form of `data/curated/` (feature `curated`; domain-model.md §15, pipeline.md §6.1).
//!
//! One file per item: `data/curated/<kind>/<family>/<slug>.toml` with the header of domain-model.md §15.1, the kind
//! fields of non-catalogue items, `[[param]]` and `[[effect]]`; one file per Incarnon family
//! (`incarnon/<slot>/<family>.toml`, §16.10); and the rule tables of `rules/` and `rules/melee/`. Every type rejects
//! unknown fields. The effect types are those of the bundle (`EffectDef`): binding fields (`bind`, `bind_max`,
//! `bind_duration`, `bind_chance`) exist only in authoring files and are removed by lowering (T19).
//!
//! [`load_curated_dir`] reads a whole tree. `overrides/` and `aliases/` belong to later tasks (T18, T10) and are not
//! loaded here; any other unexpected file is an error, so a misplaced or misnamed file is noticed.

use crate::effect::{EffectDef, Scope};
use crate::enemy::{DrClass, EnemyFlags};
use crate::enums::{AttackRoles, DamageType, ShardColor, ShotType, StatusSet};
use crate::ids::UniqueName;
use crate::item::{BuffGroup, EffectStatus, ParamSpec, RivenTraitDef};
use crate::rules::{
    BodyPartRules, CategoryDef, HelminthRules, MultGroupDef, RivenFormula, SlotLayout,
    SpecialDrDef, ToggleDescriptor, ToggleId,
};
use crate::weapon::{FinisherRow, HeavyClassRow, MeleeTriggerRow, StanceCoRow};
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
    #[serde(default)]
    pub party_scaling: Option<[f64; 4]>,
    #[serde(default)]
    pub overguard_context: Option<Box<str>>,
    #[serde(default)]
    pub eximus_capable: Option<bool>,
    #[serde(default)]
    pub eximus_og: Option<bool>,
    #[serde(default)]
    pub sources: Vec<Box<str>>,
}

rule_file!(
    /// `rules/toggles.toml`: `[[toggle]]` descriptors of T-01..T-29 and the proposed T-30..T-32 (00 §6.1; T03b).
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
        /// Stance tag per melee class and tag normalisation (`"BAYONET" = "BAYONET_STANCE"`).
        #[serde(default)]
        stance_tag: BTreeMap<Box<str>, Box<str>>,
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
    #[serde(default)]
    pub sources: Vec<Box<str>>,
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

/// Directories of `data/curated/` that belong to later tasks and are not loaded here.
pub const UNLOADED_DIRS: [&str; 2] = ["overrides", "aliases"];

/// Loads a whole `data/curated/` tree: every item file of the kind directories, every Incarnon family file and every
/// known rule table. Unknown files are errors. All problems are collected and returned together.
pub fn load_curated_dir(root: &Path) -> Result<CuratedTree, CuratedErrors> {
    let mut tree = CuratedTree::default();
    let mut errors = Vec::new();
    let entries = match sorted_entries(root) {
        Ok(e) => e,
        Err(e) => return Err(CuratedErrors(vec![e])),
    };
    for entry in entries {
        let name = file_name(&entry);
        if !entry.is_dir() {
            if is_toml(&entry) {
                errors.push(err(
                    root,
                    &entry,
                    "unexpected file at the top level of data/curated",
                ));
            }
            continue;
        }
        if UNLOADED_DIRS.contains(&name.as_str()) {
            continue;
        }
        if name == "rules" {
            load_rules(root, &entry, &mut tree.rules, &mut errors);
        } else if name == "incarnon" {
            for path in toml_files(root, &entry, &mut errors) {
                match load_toml::<IncarnonFamilyFile>(&path) {
                    Ok(f) => tree.incarnon.push((rel(root, &path), f)),
                    Err(e) => errors.push(rel_err(root, e)),
                }
            }
        } else if let Some(kind) = CuratedKind::ALL.iter().find(|k| k.dir() == name) {
            for path in toml_files(root, &entry, &mut errors) {
                match load_toml::<CuratedItemFile>(&path) {
                    Ok(f) if f.kind != *kind => errors.push(err(
                        root,
                        &path,
                        &format!(
                            "kind `{:?}` does not match directory `{}`",
                            f.kind,
                            kind.dir()
                        ),
                    )),
                    Ok(f) => tree.items.push((rel(root, &path), f)),
                    Err(e) => errors.push(rel_err(root, e)),
                }
            }
        } else {
            errors.push(err(root, &entry, "unknown directory in data/curated"));
        }
    }
    if errors.is_empty() {
        Ok(tree)
    } else {
        Err(CuratedErrors(errors))
    }
}

fn load_rules(root: &Path, dir: &Path, rules: &mut CuratedRules, errors: &mut Vec<CuratedError>) {
    let entries = match sorted_entries(dir) {
        Ok(e) => e,
        Err(e) => {
            errors.push(e);
            return;
        }
    };
    for entry in entries {
        let name = file_name(&entry);
        if entry.is_dir() {
            if name == "melee" {
                load_melee_rules(root, &entry, &mut rules.melee, errors);
            } else {
                errors.push(err(root, &entry, "unknown directory in rules/"));
            }
            continue;
        }
        if !is_toml(&entry) {
            continue;
        }
        macro_rules! slot {
            ($field:ident) => {
                match load_toml(&entry) {
                    Ok(f) => rules.$field = Some(f),
                    Err(e) => errors.push(rel_err(root, e)),
                }
            };
        }
        match name.as_str() {
            "mult_groups.toml" => slot!(mult_groups),
            "slot_layout.toml" => slot!(slot_layout),
            "riven_traits.toml" => slot!(riven_traits),
            "riven_formula.toml" => slot!(riven_formula),
            "body_parts.toml" => slot!(body_parts),
            "special_dr.toml" => slot!(special_dr),
            "toggles.toml" => slot!(toggles),
            "factions.toml" => slot!(factions),
            "categories.toml" => slot!(categories),
            "helminth.toml" => slot!(helminth),
            "frames.toml" => slot!(frames),
            "attack_roles.toml" => slot!(attack_roles),
            "stat_lines.toml" => slot!(stat_lines),
            _ => errors.push(err(root, &entry, "unknown rule table (pipeline.md §6.1)")),
        }
    }
}

fn load_melee_rules(
    root: &Path,
    dir: &Path,
    melee: &mut CuratedMeleeRules,
    errors: &mut Vec<CuratedError>,
) {
    let entries = match sorted_entries(dir) {
        Ok(e) => e,
        Err(e) => {
            errors.push(e);
            return;
        }
    };
    for entry in entries {
        if entry.is_dir() {
            errors.push(err(root, &entry, "unknown directory in rules/melee/"));
            continue;
        }
        if !is_toml(&entry) {
            continue;
        }
        macro_rules! slot {
            ($field:ident) => {
                match load_toml(&entry) {
                    Ok(f) => melee.$field = Some(f),
                    Err(e) => errors.push(rel_err(root, e)),
                }
            };
        }
        match file_name(&entry).as_str() {
            "heavy_class.toml" => slot!(heavy_class),
            "heavy_kappa.toml" => slot!(heavy_kappa),
            "attack_triggers.toml" => slot!(attack_triggers),
            "stance_co.toml" => slot!(stance_co),
            "finishers.toml" => slot!(finishers),
            "combo_specials.toml" => slot!(combo_specials),
            _ => errors.push(err(root, &entry, "unknown melee rule table (07 §4.4)")),
        }
    }
}

fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>, CuratedError> {
    let rd = std::fs::read_dir(dir).map_err(|e| CuratedError {
        path: dir.to_owned(),
        message: format!("cannot read directory: {e}"),
    })?;
    let mut v = Vec::new();
    for entry in rd {
        let entry = entry.map_err(|e| CuratedError {
            path: dir.to_owned(),
            message: format!("cannot read directory: {e}"),
        })?;
        v.push(entry.path());
    }
    v.sort();
    Ok(v)
}

/// Every `.toml` file below `dir`, recursively, in path order.
fn toml_files(root: &Path, dir: &Path, errors: &mut Vec<CuratedError>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_owned()];
    while let Some(d) = stack.pop() {
        match sorted_entries(&d) {
            Ok(entries) => {
                for e in entries {
                    if e.is_dir() {
                        stack.push(e);
                    } else if is_toml(&e) {
                        out.push(e);
                    }
                }
            }
            Err(e) => errors.push(rel_err(root, e)),
        }
    }
    out.sort();
    out
}

fn is_toml(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "toml")
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn rel(root: &Path, p: &Path) -> PathBuf {
    p.strip_prefix(root)
        .map(Path::to_owned)
        .unwrap_or_else(|_| p.to_owned())
}

fn err(root: &Path, p: &Path, msg: &str) -> CuratedError {
    CuratedError {
        path: rel(root, p),
        message: msg.to_owned(),
    }
}

fn rel_err(root: &Path, e: CuratedError) -> CuratedError {
    CuratedError {
        path: rel(root, &e.path),
        message: e.message,
    }
}
