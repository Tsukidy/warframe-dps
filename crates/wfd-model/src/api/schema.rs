//! `GET /api/schema` (server-api.md §5.6): everything the schema-driven panels render (AD-17): toggles, buffs, melee
//! plans, custom stats, riven traits, slot layouts, views and condition classes. Labels and help texts are i18n keys.

use super::calc::CondClass;
use super::catalog::ParamView;
use crate::build::SlotId;
use crate::enums::{AttackRoles, DamageType, RivenClass, View, WeaponSlot, WeaponTraits};
use crate::ids::StableKey;
use crate::item::{BuffGroup, RivenUnit};
use crate::rules::{ControlType, ToggleGroup, ToggleId};
use crate::scenario::ConditionMode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// `GET /api/schema` response.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ScenarioSchema {
    /// Toggle descriptors (`rules/toggles.toml`).
    pub toggles: Vec<ToggleSchema>,
    /// Buff items.
    pub buffs: Vec<BuffSchema>,
    /// Melee plans of 07 §8.1 (spec 00 §6.1 T-30).
    pub melee_plans: Vec<MeleePlanSchema>,
    /// Custom stats of spec 00 §6.3 (T-26).
    pub custom_stats: Vec<CustomStatSchema>,
    /// Riven traits (04 §8).
    pub riven_traits: Vec<RivenTraitSchema>,
    /// Slots per weapon slot (09 §2.2); every weapon slot is present.
    pub slot_layouts: BTreeMap<WeaponSlot, SlotLayoutSchema>,
    /// Layer views.
    pub views: Vec<ViewSchema>,
    /// T-02 modes and condition classes.
    pub conditions: ConditionsSchema,
}

/// A toggle descriptor (the API form of `rules::ToggleDescriptor`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ToggleSchema {
    /// `"T-03"`.
    pub id: ToggleId,
    /// Where the value lives (`"toggles.hit_location"`): the conventions of `rules::ToggleDescriptor::field`
    /// (several paths joined by `|`, `"buffs"`, `build.` and `opt.` prefixes; CONTRACTS.md T03b).
    pub field: Box<str>,
    /// Panel group.
    pub group: ToggleGroup,
    /// Label i18n key.
    pub label: Box<str>,
    /// Help i18n key.
    pub help: Box<str>,
    /// Control.
    pub control: ControlSchema,
    /// Default value in its JSON form.
    #[cfg_attr(feature = "ts", ts(type = "unknown"))]
    pub default: serde_json::Value,
    /// When the toggle is shown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub visible_if: Option<VisibilitySchema>,
    /// Shown only in the advanced view.
    pub advanced: bool,
}

/// Control of a toggle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ControlSchema {
    /// Control type.
    pub kind: ControlType,
    /// Lowest value (model units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub min: Option<f64>,
    /// Highest value (model units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub max: Option<f64>,
    /// Step (model units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub step: Option<f64>,
    /// Options of a select control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub options: Option<Vec<super::catalog::ChoiceView>>,
}

/// When a toggle is shown; an absent list does not restrict.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VisibilitySchema {
    /// Weapon slots.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weapon_slot: Vec<WeaponSlot>,
    /// Attack roles.
    #[serde(default, skip_serializing_if = "AttackRoles::is_empty")]
    pub attack_role: AttackRoles,
    /// Weapon traits.
    #[serde(
        default,
        rename = "trait",
        skip_serializing_if = "WeaponTraits::is_empty"
    )]
    pub traits: WeaponTraits,
}

/// A buff item of the schema.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuffSchema {
    /// StableKey of the buff item.
    pub key: StableKey,
    /// Slug (`"roar"`).
    pub id: Box<str>,
    /// English name.
    pub name: Box<str>,
    /// Buff group.
    pub group: BuffGroup,
    /// The toggle the buff implements (`"T-13"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub toggle: Option<ToggleId>,
    /// The inspiration's link key (`"o.r"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub inspiration_key: Option<Box<str>>,
    /// Caster Strength applies (04 E-4.10).
    pub strength_scaled: bool,
    /// Caster Duration applies (04 §4.2 `dur_scaled`, T-33; addition, CONTRACTS.md T01d).
    pub duration_scaled: bool,
    /// The caster of a pick without one (`ally` for T-13 Roar, spec 00 §6.1 T-37; addition, CONTRACTS.md T01d).
    pub default_caster: crate::scenario::CasterKind,
    /// Offered in the Helminth slot (09 §10).
    pub subsumable: bool,
    /// Parameters.
    pub params: Vec<ParamView>,
}

/// The plans of 07 §8.1 (`MeleePlanSel` variants without parameters). JSON `"light_loop"`, …
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MeleePlanKind {
    /// The default plan of the attack (07 §8.1).
    Auto,
    /// `light_loop(K)`: combo K repeated.
    LightLoop,
    /// `slide_loop`: the Slide combo repeated.
    SlideLoop,
    /// `heavy_loop`: heavies back to back.
    HeavyLoop,
    /// `build_spend(M_trig)`: light loop until M ≥ M_trig, then one heavy.
    BuildSpend,
    /// `tennokai`: n_tk inputs of a light loop, then one Tennokai heavy.
    Tennokai,
    /// `slam_loop`: one slam every T_slam.
    SlamLoop,
    /// `heavy_slam_loop`: one heavy slam every max(T_slam, regeneration wait).
    HeavySlamLoop,
    /// `glaive_cycle`: throw, hit, heavy detonation, catch.
    GlaiveCycle,
    /// `cycle_loop`: one 100 % strike every T_cyc.
    CycleLoop,
    /// `data_attack_loop`: one instance of a data attack every T_cyc.
    DataAttackLoop,
}

impl MeleePlanKind {
    /// The kind of a plan selection.
    pub fn of(sel: &crate::scenario::MeleePlanSel) -> Self {
        use crate::scenario::MeleePlanSel as S;
        match sel {
            S::Auto => MeleePlanKind::Auto,
            S::LightLoop => MeleePlanKind::LightLoop,
            S::SlideLoop => MeleePlanKind::SlideLoop,
            S::HeavyLoop { .. } => MeleePlanKind::HeavyLoop,
            S::BuildSpend { .. } => MeleePlanKind::BuildSpend,
            S::Tennokai => MeleePlanKind::Tennokai,
            S::SlamLoop { .. } => MeleePlanKind::SlamLoop,
            S::HeavySlamLoop { .. } => MeleePlanKind::HeavySlamLoop,
            S::GlaiveCycle { .. } => MeleePlanKind::GlaiveCycle,
            S::CycleLoop => MeleePlanKind::CycleLoop,
            S::DataAttackLoop { .. } => MeleePlanKind::DataAttackLoop,
        }
    }
}

/// A melee plan with its parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeleePlanSchema {
    /// Plan id.
    pub id: MeleePlanKind,
    /// Label i18n key.
    pub label: Box<str>,
    /// Help i18n key.
    pub help: Box<str>,
    /// Parameters (`policy`, `k`, `t_slam`, `throw`, `heavy_cancel`, `t_cyc`).
    pub params: Vec<MeleePlanParam>,
}

/// A parameter of a melee plan.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeleePlanParam {
    /// Parameter key in `MeleePlanSel`.
    pub key: Box<str>,
    /// `number` or `select`.
    pub kind: PlanParamKind,
    /// Default value in its JSON form.
    #[cfg_attr(feature = "ts", ts(type = "unknown"))]
    pub default: serde_json::Value,
    /// Values of a select parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub options: Option<Vec<Box<str>>>,
}

/// Kind of a melee-plan parameter.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PlanParamKind {
    /// A number input.
    Number,
    /// One of `options`.
    Select,
}

/// A custom stat (spec 00 §6.3, T-26).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CustomStatSchema {
    /// Id in `ScenarioDoc.custom` (`"base"`, `"crit_chance"`).
    pub id: Box<str>,
    /// StableKey of the custom-stat item.
    pub key: StableKey,
    /// Label i18n key.
    pub label: Box<str>,
    /// Unit of the input.
    pub unit: CustomUnit,
    /// Lowest value (model units).
    pub min: f64,
    /// Highest value (model units).
    pub max: f64,
    /// The inspiration's link key (`"p.a"`).
    pub inspiration_key: Box<str>,
}

/// Unit of a custom stat.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CustomUnit {
    /// Shown and entered as a percentage.
    Percent,
    /// A plain number.
    Number,
}

/// A riven trait (04 §8, 09 §12).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RivenTraitSchema {
    /// Trait id.
    pub id: Box<str>,
    /// Label i18n key.
    pub label: Box<str>,
    /// Unit of the displayed value (04 §8.2).
    pub unit: RivenUnit,
    /// Riven classes that can roll the trait.
    pub classes: Vec<RivenClass>,
    /// Element of an element trait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub element: Option<DamageType>,
    /// Legal value range for the current weapon (09 E-9.56), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub range: Option<ValueRange>,
}

/// A closed value range.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ValueRange {
    /// Lowest value.
    pub min: f64,
    /// Highest value.
    pub max: f64,
}

/// Slots of a weapon slot kind.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SlotLayoutSchema {
    /// Slot ids.
    pub slots: Vec<SlotId>,
}

/// A layer view with its texts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ViewSchema {
    /// The view.
    pub view: View,
    /// Label i18n key.
    pub label: Box<str>,
    /// Help i18n key.
    pub help: Box<str>,
}

/// T-02 modes and condition classes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConditionsSchema {
    /// T-02 modes.
    pub modes: Vec<ConditionMode>,
    /// Condition classes with their texts.
    pub classes: Vec<ConditionClassSchema>,
}

/// A condition class with its texts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConditionClassSchema {
    /// Class id.
    pub id: CondClass,
    /// Label i18n key.
    pub label: Box<str>,
    /// Help i18n key.
    pub help: Box<str>,
}
