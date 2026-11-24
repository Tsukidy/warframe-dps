//! Items, sets, Incarnon lines, rivens (domain-model.md §6).
//!
//! Mods, arcanes, shards, Incarnon perks, buffs and custom stats share one shape. A buff or custom stat is an item that
//! lives in the scenario instead of a slot.

use crate::effect::EffectDef;
use crate::enums::{
    ArcaneClass, CategorySet, Polarity, Rarity, RivenClass, RivenColumn, ShardColor, Trigger,
};
use crate::ids::{
    FamilyId, ItemId, NameKey, ParamKey, SetId, StableKey, StanceId, TagId, UniqueName, WeaponId,
};
use crate::message::MessageKey;
use crate::rules::ToggleId;
use crate::serde_util::dual_serde;
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// A mod, arcane, shard, Incarnon perk, buff or custom stat.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Item {
    pub id: ItemId,
    pub key: StableKey,
    pub unique_name: UniqueName,
    pub name: NameKey,
    pub kind: ItemKind,
    /// Categories / tags accepted, forbidding tags, trigger requirement (Semi), magazine requirement (09 §3).
    pub compat: CompatRule,
    /// Union of both directions of the wiki lists (D-093, 09 E-9.9).
    #[cfg_attr(feature = "ts", ts(as = "Vec<ItemId>"))]
    pub incompat: SmallVec<[ItemId; 4]>,
    pub polarity: Option<Polarity>,
    pub base_drain: i8,
    pub max_rank: u8,
    pub rarity: Rarity,
    pub set: Option<SetId>,
    /// User parameters (Chromatic Blade element, buff value).
    pub params: Vec<ParamSpec>,
    /// Section 11 of domain-model.md.
    pub effects: Vec<EffectDef>,
    /// Section 15.3.
    pub status: EffectStatus,
    /// Which tier produced the effects (pipeline.md §6.2; addition, CONTRACTS.md T01a).
    pub effect_tier: Option<EffectTier>,
    /// Number-only drift was auto-patched in the generated data (pipeline.md §7; addition).
    pub auto_patched: bool,
    /// Citation keys (`"W:Serration"`, `"F:core §2"`).
    pub sources: Vec<Box<str>>,
    /// Max-rank description the effects were checked against.
    pub upstream_text: Option<Box<str>>,
    /// Only for link import; never a data source.
    pub inspiration_name: Option<Box<str>>,
    pub image: Option<Box<str>>,
}

/// Kind of an item. Curated headers write `kind = "weapon_mod"` with the kind fields at the top level
/// (domain-model.md §15.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ItemKind {
    WeaponMod {
        /// PE `isUtility`.
        exilus: bool,
        augment_of: Option<UniqueName>,
    },
    Stance {
        stance: StanceId,
    },
    WeaponArcane {
        class: ArcaneClass,
    },
    /// Warframe mod; `exilus` = PE `isUtility` (09 E-9.3; addition, CONTRACTS.md T01a).
    FrameMod {
        exilus: bool,
    },
    Aura,
    FrameArcane,
    Shard {
        color: ShardColor,
        /// 1.5.
        tauforged_factor: f64,
    },
    IncarnonPerk {
        family: FamilyId,
        /// 1..=5, tier 1 = form.
        tier: u8,
        index: u8,
    },
    Buff {
        /// Ability, focus, companion, squad, debuff, environment.
        group: BuffGroup,
        toggle: Option<ToggleId>,
        /// value = listed(rank) · Str of the caster (04 E-4.10).
        strength_scaled: bool,
        /// Offered in the Helminth slot (09 §10).
        subsumable: bool,
        /// The inspiration's link key (`"o.r"`), for link import and the schema (addition, CONTRACTS.md T01a).
        inspiration_key: Option<Box<str>>,
    },
    CustomStat {
        /// `"p.a"`.
        inspiration_key: Box<str>,
    },
}

impl ItemKind {
    /// The `kind` slug of curated headers and catalogue summaries.
    pub fn slug(&self) -> &'static str {
        match self {
            ItemKind::WeaponMod { .. } => "weapon_mod",
            ItemKind::Stance { .. } => "stance",
            ItemKind::WeaponArcane { .. } => "weapon_arcane",
            ItemKind::FrameMod { .. } => "frame_mod",
            ItemKind::Aura => "aura",
            ItemKind::FrameArcane => "frame_arcane",
            ItemKind::Shard { .. } => "shard",
            ItemKind::IncarnonPerk { .. } => "incarnon_perk",
            ItemKind::Buff { .. } => "buff",
            ItemKind::CustomStat { .. } => "custom_stat",
        }
    }
}

/// Buff groups (curated `buff/<group>/`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum BuffGroup {
    Ability,
    Focus,
    Companion,
    Squad,
    Debuff,
    Environment,
}

/// Compatibility of a mod with a weapon (09 §3). The weapon side (categories of its tree, tags) is on `Weapon`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CompatRule {
    /// PE `type` slot (E-9.4 `pe_type_ok`); `None` for buffs and custom stats.
    pub slot_type: Option<ModSlotType>,
    /// Compatibility-tree nodes the mod attaches to (`norm_type(Type)`, E-9.4); any of.
    pub categories: CategorySet,
    /// Weapons named by the mod `Type` (a weapon or family name, `Unique` override list).
    #[cfg_attr(feature = "ts", ts(as = "Vec<WeaponId>"))]
    pub weapons: SmallVec<[WeaponId; 2]>,
    /// `CompatibilityTags`: any of (E-9.5); empty = no requirement.
    #[cfg_attr(feature = "ts", ts(as = "Vec<TagId>"))]
    pub tags: SmallVec<[TagId; 2]>,
    /// `IncompatibilityTags`.
    #[cfg_attr(feature = "ts", ts(as = "Vec<TagId>"))]
    pub forbid_tags: SmallVec<[TagId; 2]>,
    /// X-EXALT: the mod's `POWER_WEAPON` incompatibility is lifted on exalted weapons (`All`) or on listed weapons.
    pub exalted_lift: ExaltedLift,
    /// X-EXALT `exalt_req(m)`: weapon tags that keep the mod off exalted weapons (`NO_AIM`, `NO_SLIDE`).
    #[cfg_attr(feature = "ts", ts(as = "Vec<TagId>"))]
    pub exalt_req: SmallVec<[TagId; 1]>,
    /// Trigger requirement (Cannonade: Semi, through the derived `SEMI_AUTO` tag, E-9.6).
    pub trigger: Option<Trigger>,
    /// Base-magazine requirement (Synth Charge: 6, E-9.7).
    pub min_base_magazine: Option<f64>,
    /// Weapons exempt from the magazine requirement (Kulstar, E-9.7).
    #[cfg_attr(feature = "ts", ts(as = "Vec<WeaponId>"))]
    pub magazine_exempt: SmallVec<[WeaponId; 1]>,
}

/// PE `type` of a mod (09 E-9.3, E-9.4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ModSlotType {
    Primary,
    Secondary,
    Melee,
    ArchGun,
    ArchMelee,
    Warframe,
    Aura,
    Stance,
    Companion,
}

/// X-EXALT lift of the `POWER_WEAPON` incompatibility tag (09 §3.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ExaltedLift {
    #[default]
    None,
    AllExalted,
    Only(Vec<WeaponId>),
}

/// Effect-status lattice (domain-model.md §15.3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum EffectStatus {
    /// Reviewed curated entry with sources.
    Verified,
    /// Tier-1 stat-line import or tier-2 spec-table ingestion, all lines mapped, validators clean.
    Auto,
    /// Scaffolded or partially reviewed.
    Draft,
    /// Upstream text changed since review.
    Stale,
    /// Known to affect damage, not encoded yet (also every catalogue item with no effects).
    Unmodelled,
    /// Recoil, zoom without stat change, utility.
    NoDamageEffect,
}

impl EffectStatus {
    /// In the default optimizer pool (domain-model.md §15.3; AD-23).
    pub fn in_optimizer_pool(self) -> bool {
        matches!(
            self,
            EffectStatus::Verified | EffectStatus::Auto | EffectStatus::Draft | EffectStatus::Stale
        )
    }
}

/// Which pipeline tier produced an item's effects (pipeline.md §6.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum EffectTier {
    StatLine,
    SpecTable,
    Curated,
}

/// A user parameter of an item. Readable form (curated `[[param]]`): `key`, `kind` (`"percent"`, `"number"`,
/// `"choice"`, `"bool"`) with `min`, `max`, `step`, `choices` at the same level, `default`, optional `label`,
/// `helper`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "ParamSpecRepr"))]
pub struct ParamSpec {
    pub key: ParamKey,
    pub kind: ParamKind,
    pub default: ParamValue,
    /// `None`: the UI derives the label from the parameter key (CONTRACTS.md, T01a).
    pub label: Option<MessageKey>,
    /// E.g. Roar: value = rank value × Strength (UI calculator).
    pub helper: Option<ParamHelper>,
}

/// Kind and range of a parameter.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ParamKind {
    Percent { min: f64, max: f64 },
    Number { min: f64, max: f64, step: f64 },
    Choice(Vec<Choice>),
    Bool,
}

/// One choice of a choice parameter.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Choice {
    pub value: Box<str>,
    #[serde(default)]
    pub label: Option<MessageKey>,
}

/// UI helper of a parameter (`{ formula = "rank_value * strength", rank_values = [0.10, 0.15, 0.25, 0.50] }`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ParamHelper {
    pub formula: Box<str>,
    #[serde(default)]
    pub rank_values: Vec<f64>,
}

/// Readable form of [`ParamSpec`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ParamSpecRepr {
    pub key: ParamKey,
    pub kind: ParamKindName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<Choice>>,
    pub default: ParamValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<MessageKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub helper: Option<ParamHelper>,
}

/// Name of a [`ParamKind`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ParamKindName {
    Percent,
    Number,
    Choice,
    Bool,
}

impl From<&ParamSpec> for ParamSpecRepr {
    fn from(p: &ParamSpec) -> Self {
        let mut r = ParamSpecRepr {
            key: p.key.clone(),
            kind: ParamKindName::Bool,
            min: None,
            max: None,
            step: None,
            choices: None,
            default: p.default.clone(),
            label: p.label.clone(),
            helper: p.helper.clone(),
        };
        match &p.kind {
            ParamKind::Percent { min, max } => {
                r.kind = ParamKindName::Percent;
                r.min = Some(*min);
                r.max = Some(*max);
            }
            ParamKind::Number { min, max, step } => {
                r.kind = ParamKindName::Number;
                r.min = Some(*min);
                r.max = Some(*max);
                r.step = Some(*step);
            }
            ParamKind::Choice(c) => {
                r.kind = ParamKindName::Choice;
                r.choices = Some(c.clone());
            }
            ParamKind::Bool => {}
        }
        r
    }
}

impl TryFrom<ParamSpecRepr> for ParamSpec {
    type Error = String;
    fn try_from(r: ParamSpecRepr) -> Result<Self, String> {
        let range_only = r.choices.is_none();
        let kind = match r.kind {
            ParamKindName::Percent if range_only && r.step.is_none() => ParamKind::Percent {
                min: r.min.unwrap_or(0.0),
                max: r.max.ok_or("percent parameter needs `max`")?,
            },
            ParamKindName::Number if range_only => ParamKind::Number {
                min: r.min.ok_or("number parameter needs `min`")?,
                max: r.max.ok_or("number parameter needs `max`")?,
                step: r.step.unwrap_or(0.0),
            },
            ParamKindName::Choice if r.min.is_none() && r.max.is_none() && r.step.is_none() => {
                ParamKind::Choice(r.choices.ok_or("choice parameter needs `choices`")?)
            }
            ParamKindName::Bool
                if range_only && r.min.is_none() && r.max.is_none() && r.step.is_none() =>
            {
                ParamKind::Bool
            }
            k => return Err(format!("fields do not match parameter kind {k:?}")),
        };
        Ok(ParamSpec {
            key: r.key,
            kind,
            default: r.default,
            label: r.label,
            helper: r.helper,
        })
    }
}

dual_serde!(ParamSpec, ParamSpecRepr);

/// Value of a parameter. JSON: `number | string | boolean`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "ParamValueRepr"))]
pub enum ParamValue {
    Num(f64),
    Choice(Box<str>),
    Bool(bool),
}

/// Readable form of [`ParamValue`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ParamValueRepr {
    Num(f64),
    Choice(Box<str>),
    Bool(bool),
}

impl From<&ParamValue> for ParamValueRepr {
    fn from(v: &ParamValue) -> Self {
        match v {
            ParamValue::Num(x) => ParamValueRepr::Num(*x),
            ParamValue::Choice(c) => ParamValueRepr::Choice(c.clone()),
            ParamValue::Bool(b) => ParamValueRepr::Bool(*b),
        }
    }
}

impl From<ParamValueRepr> for ParamValue {
    fn from(r: ParamValueRepr) -> Self {
        match r {
            ParamValueRepr::Num(x) => ParamValue::Num(x),
            ParamValueRepr::Choice(c) => ParamValue::Choice(c),
            ParamValueRepr::Bool(b) => ParamValue::Bool(b),
        }
    }
}

dual_serde!(ParamValue, ParamValueRepr);

/// A mod set (`ExportModSet`, `M:Mods.Sets`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SetDef {
    pub id: SetId,
    /// PE `modSet` path (join key of `data/curated/set/*.toml`; addition, CONTRACTS.md T01a).
    pub unique_name: UniqueName,
    pub name: NameKey,
    #[cfg_attr(feature = "ts", ts(as = "Vec<ItemId>"))]
    pub members: SmallVec<[ItemId; 6]>,
    /// Members that never count (`set_noncounting`, 09 E-9.40: Mecha Empowered; addition).
    #[cfg_attr(feature = "ts", ts(as = "Vec<ItemId>"))]
    pub noncounting: SmallVec<[ItemId; 1]>,
    /// Uses `per = set_piece`.
    pub bonus: Vec<EffectDef>,
    /// Sacrificial: index = pieces − 1.
    pub member_scale: Option<Vec<f64>>,
}

/// An Incarnon line of a weapon family (09 §13).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IncarnonDef {
    pub family: FamilyId,
    /// `"furis"`, `"mk1_furis"`: the variant column of per-variant values.
    pub variant: Box<str>,
    /// Native Incarnon weapon (5 evolutions) vs Genesis (4), 09 E-9.60 T(W) (addition, CONTRACTS.md T01a).
    pub native: bool,
    /// Perks per evolution 1..=5 (index 0 = evolution I, the form).
    #[cfg_attr(feature = "ts", ts(as = "[Vec<ItemId>; 5]"))]
    pub tiers: [SmallVec<[ItemId; 3]>; 5],
}

/// A riven trait definition (`data/curated/rules/riven_traits.toml`, 04 §8, 09 §12).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RivenTraitDef {
    /// `"critical_chance"`, `"damage_to_grineer"`, `"weak_point_critical_chance"` …
    pub id: Box<str>,
    /// Public Export `upgradeEntries` tag (value formula base b).
    #[serde(default)]
    pub pe_tag: Option<Box<str>>,
    pub input: RivenUnit,
    /// Classes that can roll the trait; empty = every class with a coefficient.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<RivenClass>"))]
    pub classes: SmallVec<[RivenClass; 4]>,
    /// `None` = no damage effect (zoom, recoil). The riven value enters as the param `value`.
    #[serde(default)]
    pub template: Option<EffectDef>,
    #[serde(default)]
    pub label: Option<MessageKey>,
    /// Coefficient b per column (04 §8.3; PE base values); missing column = cannot roll (addition, CONTRACTS.md).
    #[serde(default)]
    pub coeff: BTreeMap<RivenColumn, f64>,
    /// Sign rule (09 E-9.55 T3).
    #[serde(default)]
    pub sign: RivenSign,
    /// A spliced trait (09 §12.3).
    #[serde(default)]
    pub spliced: bool,
    /// Splice recipes: pairs of consumed trait ids.
    #[serde(default)]
    pub recipes: Vec<[Box<str>; 2]>,
    /// The inspiration's riven catalogue key (`"crit_chance"`, `"addBlast"`; 09 §2.4).
    #[serde(default)]
    pub inspiration_key: Option<Box<str>>,
}

/// How a riven trait value is typed (04 §8.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RivenUnit {
    /// `±x %` → x/100.
    Percent,
    /// `×y` → y − 1 (typed 1.45 → bonus 0.45).
    FactionMultiplier,
    /// Metres, seconds, counts: raw.
    Absolute,
}

/// Sign rule of a riven trait (09 E-9.55 T3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RivenSign {
    #[default]
    Any,
    PositiveOnly,
    /// Positive-only with severity `illegal_unverified` (Initial Combo, OQ-921).
    PositiveOnlyUnverified,
    NegativeOnly,
}

/// A riven instance (user input, 09 §12.2). JSON (server-api.md §4):
/// `{ "class": "pistol", "traits": [{ "trait": "critical_damage", "value": 1.114 }] }` plus the optional 09 §12.2
/// fields `rank` (0..=8, default 8), `polarity` (default unknown), `drain` (the drain the card shows at `rank`; default
/// per E-9.59) and `family` (default: bound to the weapon it sits on), omitted when absent. The optional fields are
/// additions (CONTRACTS.md, T01a).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "RivenRepr"))]
pub struct Riven {
    pub class: RivenClass,
    pub traits: ArrayVec<RivenTrait, 4>,
    pub rank: Option<u8>,
    pub polarity: Option<Polarity>,
    pub drain: Option<i8>,
    pub family: Option<Box<str>>,
}

/// Readable form of [`Riven`]: absent optional fields are omitted.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RivenRepr {
    pub class: RivenClass,
    #[cfg_attr(feature = "ts", ts(as = "Vec<RivenTrait>"))]
    pub traits: ArrayVec<RivenTrait, 4>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polarity: Option<Polarity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drain: Option<i8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<Box<str>>,
}

impl From<&Riven> for RivenRepr {
    fn from(r: &Riven) -> Self {
        RivenRepr {
            class: r.class,
            traits: r.traits.clone(),
            rank: r.rank,
            polarity: r.polarity,
            drain: r.drain,
            family: r.family.clone(),
        }
    }
}

impl From<RivenRepr> for Riven {
    fn from(r: RivenRepr) -> Self {
        Riven {
            class: r.class,
            traits: r.traits,
            rank: r.rank,
            polarity: r.polarity,
            drain: r.drain,
            family: r.family,
        }
    }
}

dual_serde!(Riven, RivenRepr);

/// One riven trait: id and value in model units (fraction or bonus; 04 §8.2 conversion of the displayed value).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RivenTrait {
    #[serde(rename = "trait")]
    pub trait_id: Box<str>,
    pub value: f64,
}
