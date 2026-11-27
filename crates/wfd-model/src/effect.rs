//! The effect DSL (domain-model.md §11.1, §12, §13).
//!
//! The bundle form is the authoring form after lowering of sugar; both use these types. Binding fields (`bind`,
//! `bind_max`, `bind_duration`, `bind_chance`, domain-model.md §15.2) exist in the types as `Option`s and are **kept**
//! in the bundle form (`data/generated/*.json` and `bundle.bin`) as provenance of the checked numbers: the data build
//! checks them (T19) and no consumer outside the data build reads them (CONTRACTS.md, contract review DC-27).
//!
//! Readable (TOML, JSON) forms follow the encoded examples of domain-model.md §16: the effect target is flattened
//! into the effect table (`stat = "cc_rel"`, `element = "HEA"`, `faction = "any"`, `mult = "eclipse"`,
//! `mechanic = "extra_hit"` with `[effect.params]`), values are untagged (`value = 1.65`,
//! `value = { max = 0.80, bind = "n1%" }`, `value = { param = "value" }`), conditions are tables with one kind key
//! (`when = { trigger = "on_damaged", chance = 0.21 }`).

use crate::enums::{
    AttackRoles, DamageType, DamageTypeSet, InputKind, LayerSet, MeleeClass, ShardColor,
    StatusType, Trigger, UnknownName, WeaponClass, WeaponSlot, WeaponTraits,
};
use crate::ids::{ParamKey, UniqueName};
use crate::mechanic::{Mechanic, MechanicKind, MechanicParamsRepr};
use crate::serde_util::{dual_serde, flags_serde, string_serde};
use crate::stats::StatKey;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------------------------------------------
// EffectDef
// ---------------------------------------------------------------------------------------------------------------

/// One effect of an item, a weapon or attack passive, a set bonus, a zoom level or a riven trait template.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "EffectDefRepr"))]
pub struct EffectDef {
    /// Stable within the item (`"e0"`, or curator-given `"co"`); `EffectKey = item#id`. Empty in an authoring file
    /// means "assigned at lowering" (`e<index of the effect>`).
    pub id: Box<str>,
    /// What the effect changes.
    pub target: Target,
    /// Absent in the readable form means `{ param = "value" }` (riven trait templates, buffs, custom stats).
    pub value: ValueExpr,
    /// The value is multiplied by the product of these scalars.
    pub per: SmallVec<[Per; 2]>,
    /// Dynamic condition (default `Always`); its T-02 class is derived (§12).
    pub when: Cond,
    /// Own stack counter (04 §4.2); `per = stack` reads it.
    pub stacks: Option<StackModel>,
    /// Static applicability, never assumed (04 §4.3 gates). An effect whose scope is the default inherits the item's
    /// scope at lowering.
    pub scope: Scope,
    /// Value × k on a sub-scope ("x2 for Heavy Attacks", "x2 for Bows").
    pub scope_factor: SmallVec<[ScopeFactor; 1]>,
    /// Only for element targets (§13).
    pub order: Option<ElemOrder>,
    /// Types and layers of a vulnerability (05 E-5.19): required with [`Target::VulnAdd`], optional with a TargetVuln
    /// `mult` group, rejected elsewhere (also with a single `vuln_add.<ℓ>.<t>` slot).
    pub vuln: Option<VulnMask>,
    /// Optional declared class; the validator checks that it equals the derived one.
    pub class: Option<ClassSet>,
    /// UNVERIFIED notes, carried to the UI and traces.
    pub note: Option<Box<str>>,
    /// Values used when the shard is Tauforged, replacing `value × tauforged_factor` (integer stats such as the
    /// Emerald Corrosion cap, domain-model.md §16.13); empty = the factor rule.
    pub tauforged_values: SmallVec<[f64; 1]>,
    /// Weak-spot source σ of 02 E-2.10 (T-32): required on, and only allowed on, effects whose target is a `mult`
    /// group of stage `Spot` (domain-model.md §14). Addition, CONTRACTS.md T01d.
    pub spot: Option<SpotSource>,
    /// 04 §4.2 `dot_scaling`: whether an elemental bonus also raises its element's DoT (`elem_dot_sum`, 03 §9.3,
    /// 01 E-1.15). Only on element targets (`element = …`, `stat = "elem_nc.<t>"`). `None` = the 03 §9.3 default:
    /// true, except `element = "target_weakness"` (Sentient Incision), false; the data sets `false` on buffing
    /// fields (Electric Shield, Energy Shell, weapon-made fields) and Nourish. Addition, contract review DC-11.
    pub dot_scaling: Option<bool>,
    /// Default uptime of the effect's class (a) and (c) conditions when no `CondOverride` gives one (04 §4.3, §9:
    /// the Mecha mark is on `on` s every `period` s, so u = on/period: 0.05 / 0.133 / 0.30 / 0.80 by set pieces).
    /// EV uses u = on/period; MC draws the alternating on/off process of 04 §4.3 with that on-time and period.
    /// `None` = u = 1. Only on conditional effects (a `when` or a `stacks.gain_on`). Addition, contract review DC-17.
    pub uptime: Option<UptimeSpec>,
}

impl EffectDef {
    /// An unconditional, unscoped effect with a constant value.
    pub fn new(target: Target, value: ValueExpr) -> Self {
        EffectDef {
            id: "".into(),
            target,
            value,
            per: SmallVec::new(),
            when: Cond::Always,
            stacks: None,
            scope: Scope::default(),
            scope_factor: SmallVec::new(),
            order: None,
            vuln: None,
            class: None,
            note: None,
            tauforged_values: SmallVec::new(),
            spot: None,
            dot_scaling: None,
            uptime: None,
        }
    }
}

impl EffectDef {
    /// The condition classes of the effect, derived by the table of domain-model.md §12.1 (never trusted from the
    /// author; the validator compares a declared `class` with this). Class (u) depends on the item kind (a buff) and
    /// is added by the caller. An empty set means unconditional.
    ///
    /// A target whose bucket the kernel multiplies by a target-state count ([`StatKey::is_target_state_count`]:
    /// `co_per_status` × n, `co_per_cold_stack` × n_COL) is class (a): Condition Overload and Secondary Shiver are
    /// (a) (spec 00 §6.2 "CO-type per status, Secondary Shiver"; domain-model.md §16.2-16.3), and Galvanized
    /// Aptitude's CO is (a) and (c) (§12.1, §16.2).
    pub fn derived_class(&self) -> ClassSet {
        let mut c = self.when.derived_class();
        // A separate gain event (`StackModel::gain_on`) classifies the stacks; `when` is then the activity condition
        // (domain-model.md §12.1).
        if let Some(g) = self.stacks.as_ref().and_then(|s| s.gain_on.as_ref()) {
            c |= g.derived_class();
        }
        for p in &self.per {
            c |= p.derived_class();
        }
        match &self.target {
            Target::Stat(k) if k.is_target_state_count() => c |= ClassSet::A,
            Target::Mechanic(crate::mechanic::Mechanic::CounterBonus(p)) => {
                // Own hits, inputs and throws.
                c |= ClassSet::B;
                if p.stat.is_target_state_count() {
                    c |= ClassSet::A;
                }
            }
            _ => {}
        }
        let hit = &self.scope.hit;
        if hit.outcome.is_some()
            || !hit.body_part.is_empty()
            || !hit.flags.is_empty()
            || hit.min_distance.is_some()
        {
            c |= ClassSet::GATE;
        }
        c
    }
}

impl Cond {
    /// Classes of a condition (domain-model.md §12.1).
    pub fn derived_class(&self) -> ClassSet {
        match self {
            Cond::Always => ClassSet::empty(),
            // `combo(n)` is a combo-state predicate M(c) ≥ n read from the combo model like `Per::ComboTier`
            // (driven by 07 or fixed by T-10, not switched off by T-02; contract review DC-24).
            Cond::Trigger {
                trigger: TriggerId::Combo(_),
                ..
            } => ClassSet::COMBO,
            Cond::Trigger { trigger, .. } => match trigger.class() {
                TriggerClass::OwnEvent => ClassSet::B,
                TriggerClass::External => ClassSet::C,
            },
            // The scenario's Eximus switch is static: a gate, never assumed (04 §5.3 Eximus Advantage, 05 §7).
            Cond::Target(TargetPred::Eximus) => ClassSet::GATE,
            Cond::Target(_) => ClassSet::A,
            Cond::Shot(_) => ClassSet::SCHEDULE,
            Cond::Attack(_) => ClassSet::FORMULA,
            Cond::All(v) | Cond::Any(v) => v
                .iter()
                .fold(ClassSet::empty(), |acc, c| acc | c.derived_class()),
        }
    }
}

impl Per {
    /// Class of a scalar (domain-model.md §12.1). `Stack` takes the class of its trigger, which is the effect's
    /// `when`, so it adds nothing here.
    pub fn derived_class(&self) -> ClassSet {
        match self {
            Per::Stack => ClassSet::empty(),
            Per::DistinctStatus | Per::StatusStacks(_) | Per::StatusPresent(_) => ClassSet::A,
            Per::ComboTier | Per::ComboTierAboveOne | Per::SniperCombo => ClassSet::COMBO,
            Per::SetPiece
            | Per::ShardCount(_)
            | Per::Strength
            | Per::Resolved { .. }
            | Per::AbilityDuration
            | Per::CompressionEff(_) => ClassSet::STATIC,
            Per::ReloadedFraction => ClassSet::SCHEDULE,
        }
    }
}

/// Readable form of [`EffectDef`]: the target is one of the keys `stat`, `element`, `faction`, `mult`, `mechanic`
/// (with `params`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EffectDefRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Box<str>>,
    /// A slot name, or `"vuln_add"` with `vuln` ([`StatName`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stat: Option<StatName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<ElemSel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction: Option<FactionSel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mult: Option<Box<str>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mechanic: Option<MechanicKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<MechanicParamsRepr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<ValueExpr>,
    #[serde(default, skip_serializing_if = "SmallVec::is_empty")]
    #[cfg_attr(feature = "ts", ts(as = "Option<Vec<Per>>", optional))]
    pub per: SmallVec<[Per; 2]>,
    #[serde(default, skip_serializing_if = "Cond::is_always")]
    #[cfg_attr(feature = "ts", ts(as = "Option<Cond>", optional))]
    pub when: Cond,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stacks: Option<StackModel>,
    #[serde(default, skip_serializing_if = "Scope::is_any")]
    #[cfg_attr(feature = "ts", ts(as = "Option<Scope>", optional))]
    pub scope: Scope,
    #[serde(default, skip_serializing_if = "SmallVec::is_empty")]
    #[cfg_attr(feature = "ts", ts(as = "Option<Vec<ScopeFactor>>", optional))]
    pub scope_factor: SmallVec<[ScopeFactor; 1]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<ElemOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vuln: Option<VulnMask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<ClassSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<Box<str>>,
    #[serde(default, skip_serializing_if = "SmallVec::is_empty")]
    #[cfg_attr(feature = "ts", ts(as = "Option<Vec<f64>>", optional))]
    pub tauforged_values: SmallVec<[f64; 1]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot: Option<SpotSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dot_scaling: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<UptimeSpec>,
}

impl From<&EffectDef> for EffectDefRepr {
    fn from(e: &EffectDef) -> Self {
        let mut r = EffectDefRepr {
            id: (!e.id.is_empty()).then(|| e.id.clone()),
            value: if e.value == ValueExpr::default_param() {
                None
            } else {
                Some(e.value.clone())
            },
            per: e.per.clone(),
            when: e.when.clone(),
            stacks: e.stacks.clone(),
            scope: e.scope.clone(),
            scope_factor: e.scope_factor.clone(),
            order: e.order,
            vuln: e.vuln.clone(),
            class: e.class,
            note: e.note.clone(),
            tauforged_values: e.tauforged_values.clone(),
            spot: e.spot.clone(),
            dot_scaling: e.dot_scaling,
            uptime: e.uptime.clone(),
            ..Default::default()
        };
        match &e.target {
            Target::Stat(k) => r.stat = Some(StatName::Slot(*k)),
            Target::VulnAdd => r.stat = Some(StatName::VulnAddRow),
            Target::Element(s) => r.element = Some(s.clone()),
            Target::Faction(f) => r.faction = Some(f.clone()),
            Target::Mult(g) => r.mult = Some(g.clone()),
            Target::Mechanic(m) => {
                r.mechanic = Some(m.kind());
                r.params = Some(MechanicParamsRepr::from(m));
            }
        }
        r
    }
}

impl TryFrom<EffectDefRepr> for EffectDef {
    type Error = String;
    fn try_from(r: EffectDefRepr) -> Result<Self, String> {
        let mut targets = Vec::new();
        match r.stat {
            Some(StatName::Slot(k)) => targets.push(Target::Stat(k)),
            Some(StatName::VulnAddRow) => targets.push(Target::VulnAdd),
            None => {}
        }
        if let Some(s) = r.element {
            targets.push(Target::Element(s));
        }
        if let Some(f) = r.faction {
            targets.push(Target::Faction(f));
        }
        if let Some(g) = r.mult {
            targets.push(Target::Mult(g));
        }
        if let Some(kind) = r.mechanic {
            targets.push(Target::Mechanic(Mechanic::from_params(
                kind,
                r.params.unwrap_or_default(),
            )?));
        } else if r.params.is_some() {
            return Err("`params` is only allowed with `mechanic`".into());
        }
        if targets.len() != 1 {
            return Err(format!(
                "an effect needs exactly one target key of `stat`, `element`, `faction`, `mult`, `mechanic` (found {})",
                targets.len()
            ));
        }
        let target = targets.pop().unwrap_or(Target::Stat(StatKey::BaseDmg));
        // `vuln` masks the additive group `vuln_add` and TargetVuln multiplier groups only (domain-model.md §11.1;
        // whether a `mult` group is TargetVuln needs `Rules.mult_groups` and is checked by the validator, T19).
        match (&target, &r.vuln) {
            (Target::VulnAdd, None) => {
                return Err(
                    "`stat = \"vuln_add\"` needs a `vuln` mask (types and layers it adds to)"
                        .into(),
                );
            }
            (Target::Stat(k @ StatKey::VulnAdd(_)), Some(_)) => {
                return Err(format!(
                    "`stat = \"{k}\"` is one slot and takes no `vuln` mask; write `stat = \"vuln_add\"` with the mask"
                ));
            }
            (Target::VulnAdd | Target::Mult(_), _) | (_, None) => {}
            (_, Some(_)) => {
                return Err(
                    "`vuln` is only allowed with `stat = \"vuln_add\"` and with `mult` (TargetVuln groups)".into(),
                );
            }
        }
        // `spot` belongs to `mult` groups of stage `Spot` (02 E-2.10); the stage needs `Rules.mult_groups` and is
        // checked by the validator (T19).
        if r.spot.is_some() && !matches!(target, Target::Mult(_)) {
            return Err(
                "`spot` is only allowed with `mult` (a group of stage `spot`, 02 E-2.10)".into(),
            );
        }
        // `dot_scaling` is the 04 §4.2 flag of elemental bonuses (03 §9.3).
        if r.dot_scaling.is_some()
            && !matches!(
                target,
                Target::Element(_) | Target::Stat(StatKey::ElemNc(_))
            )
        {
            return Err(
                "`dot_scaling` is only allowed on element targets (`element`, `stat = \"elem_nc.<t>\"`; 03 §9.3)"
                    .into(),
            );
        }
        // `uptime` is the default u of a condition (04 §4.3): the effect's `when` or its stacks' gain event.
        let gain_on = r.stacks.as_ref().is_some_and(|st| st.gain_on.is_some());
        if r.uptime.is_some() && r.when.is_always() && !gain_on {
            return Err("`uptime` needs a condition (`when` or `stacks.gain_on`; 04 §4.3)".into());
        }
        Ok(EffectDef {
            id: r.id.unwrap_or_else(|| "".into()),
            target,
            value: r.value.unwrap_or_else(ValueExpr::default_param),
            per: r.per,
            when: r.when,
            stacks: r.stacks,
            scope: r.scope,
            scope_factor: r.scope_factor,
            order: r.order,
            vuln: r.vuln,
            class: r.class,
            note: r.note,
            tauforged_values: r.tauforged_values,
            spot: r.spot,
            dot_scaling: r.dot_scaling,
            uptime: r.uptime,
        })
    }
}

dual_serde!(EffectDef, EffectDefRepr);

/// What an effect changes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Target {
    /// One registry slot. TOML `stat = "cc_rel"`, `stat = "forced.SLA"`, `stat = "stack_cap.COR"`,
    /// `stat = "vuln_add.HP.HEA"` (one slot, no `vuln` mask).
    Stat(StatKey),
    /// TOML `element = "HEA"` | `"target_weakness"` | `{ param = "element" }`.
    Element(ElemSel),
    /// TOML `faction = "any"` | `["grineer", "kuva_grineer"]`.
    Faction(FactionSel),
    /// TOML `mult = "eclipse"` (group id from `Rules.mult_groups`).
    Mult(Box<str>),
    /// TOML `mechanic = "extra_hit"` + `[effect.params]`.
    Mechanic(Mechanic),
    /// The additive vulnerability group ΣV_add (05 E-5.19, 04 §5.7) on every slot `vuln_add.<ℓ>.<t>` with ℓ in the
    /// effect's `vuln.layers` and t in `vuln.types` (the 54-slot row, domain-model.md §16.13, §17: Magus
    /// Accelerant, Theorem Contagion with a parameter type). TOML `stat = "vuln_add"` with a required `vuln` mask.
    /// Addition (CONTRACTS.md, T01a fixes).
    VulnAdd,
}

/// The `stat` key of an effect's readable form: one registry slot (`"cc_rel"`, `"forced.SLA"`,
/// `"vuln_add.HP.HEA"`), or the row name `"vuln_add"`, which the effect's `vuln` mask spreads over the row's slots
/// ([`Target::VulnAdd`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatName {
    /// A full slot name ([`StatKey::parse`]).
    Slot(StatKey),
    /// `"vuln_add"`: the whole additive-vulnerability row, masked by `vuln`.
    VulnAddRow,
}

impl StatName {
    /// The row name that the `vuln` mask spreads (`"vuln_add"`).
    pub const VULN_ADD_ROW: &'static str = "vuln_add";

    /// Every legal name: the slot names, then `"vuln_add"` (for the JSON schema).
    pub fn all_names() -> Vec<String> {
        let mut v = StatKey::all_names();
        v.push(Self::VULN_ADD_ROW.to_owned());
        v
    }
}

impl fmt::Display for StatName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatName::Slot(k) => fmt::Display::fmt(k, f),
            StatName::VulnAddRow => f.write_str(Self::VULN_ADD_ROW),
        }
    }
}

impl FromStr for StatName {
    type Err = UnknownName;
    fn from_str(s: &str) -> Result<Self, UnknownName> {
        if s == Self::VULN_ADD_ROW {
            return Ok(StatName::VulnAddRow);
        }
        StatKey::from_str(s).map(StatName::Slot)
    }
}

string_serde!(StatName, StatName::all_names());

/// Element selector.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "ElemSelRepr"))]
pub enum ElemSel {
    /// A fixed element.
    Type(DamageType),
    /// The target faction's weakness element (01 §4.10.4).
    TargetWeakness,
    /// An item parameter (Chromatic Blade).
    Param(ParamKey),
}

/// Readable form of [`ElemSel`]: `"HEA"`, `"target_weakness"` or `{ param = "element" }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ElemSelRepr {
    Name(String),
    Param { param: ParamKey },
}

const TARGET_WEAKNESS: &str = "target_weakness";

impl From<&ElemSel> for ElemSelRepr {
    fn from(s: &ElemSel) -> Self {
        match s {
            ElemSel::Type(t) => ElemSelRepr::Name(t.name().to_owned()),
            ElemSel::TargetWeakness => ElemSelRepr::Name(TARGET_WEAKNESS.to_owned()),
            ElemSel::Param(p) => ElemSelRepr::Param { param: p.clone() },
        }
    }
}

impl TryFrom<ElemSelRepr> for ElemSel {
    type Error = UnknownName;
    fn try_from(r: ElemSelRepr) -> Result<Self, UnknownName> {
        match r {
            ElemSelRepr::Name(n) if n == TARGET_WEAKNESS => Ok(ElemSel::TargetWeakness),
            ElemSelRepr::Name(n) => DamageType::from_str(&n).map(ElemSel::Type),
            ElemSelRepr::Param { param } => Ok(ElemSel::Param(param)),
        }
    }
}

dual_serde!(ElemSel, ElemSelRepr);

/// Faction selector of a faction-bucket effect.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "FactionSelRepr"))]
pub enum FactionSel {
    /// Every faction (Roar).
    Any,
    /// Faction slugs of `Rules.factions` (`"grineer"`, `"kuva_grineer"`).
    Of(SmallVec<[Box<str>; 2]>),
}

/// Readable form of [`FactionSel`]: `"any"` or a list of faction slugs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FactionSelRepr {
    Keyword(String),
    List(Vec<Box<str>>),
}

impl From<&FactionSel> for FactionSelRepr {
    fn from(s: &FactionSel) -> Self {
        match s {
            FactionSel::Any => FactionSelRepr::Keyword("any".to_owned()),
            FactionSel::Of(v) => FactionSelRepr::List(v.to_vec()),
        }
    }
}

impl TryFrom<FactionSelRepr> for FactionSel {
    type Error = String;
    fn try_from(r: FactionSelRepr) -> Result<Self, String> {
        match r {
            FactionSelRepr::Keyword(k) if k == "any" => Ok(FactionSel::Any),
            FactionSelRepr::Keyword(k) => Err(format!(
                "faction selector must be \"any\" or a list, found \"{k}\""
            )),
            FactionSelRepr::List(v) if v.is_empty() => Err("empty faction list".into()),
            FactionSelRepr::List(v) => Ok(FactionSel::Of(v.into_iter().collect())),
        }
    }
}

dual_serde!(FactionSel, FactionSelRepr);

// ---------------------------------------------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------------------------------------------

/// The value of an effect (max rank unless a curve is given).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "ValueRepr"))]
pub enum ValueExpr {
    /// TOML `value = 1.65`.
    Const(f64),
    /// TOML `value = { max = 1.65, rank = "linear", bind = "n1%" }`; `rank` defaults to `linear`
    /// (`max·(r + 1)/(R + 1)`, 09 E-9.13).
    Ranked {
        /// Value at maximum rank.
        max: f64,
        /// Per-rank rule (09 E-9.13).
        rank: RankCurve,
        /// Binding of `max` to the upstream text (authoring only).
        bind: Option<Binding>,
    },
    /// TOML `value = { variants = { furis = 28.0, mk1_furis = 34.0 } }`: Incarnon per-variant values.
    Variants { by: BTreeMap<Box<str>, f64> },
    /// TOML `value = { param = "value" }`: a user parameter (buff value, custom stat value).
    Param(ParamKey),
    /// TOML `value = { base = 0.30, add = [{ coeff = 0.10, per = [...] }] }`: `base + Σ coeff·Π per`.
    Linear {
        /// Constant part.
        base: Box<ValueExpr>,
        /// Terms `coeff · Π per`.
        add: SmallVec<[Term; 1]>,
    },
    /// TOML `value = { scalar = "set_piece", values = [...] }`: value indexed by a static scalar (set pieces).
    TableBy { scalar: Per, values: Vec<f64> },
}

impl ValueExpr {
    /// `{ param = "value" }`, the default of an effect without `value`.
    pub fn default_param() -> ValueExpr {
        ValueExpr::Param(ParamKey::new("value"))
    }
}

/// One term of [`ValueExpr::Linear`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Term {
    /// Coefficient of the term.
    pub coeff: f64,
    /// Scalars multiplied into the term.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<Per>"))]
    pub per: SmallVec<[Per; 1]>,
}

/// Per-rank curve of a ranked value (09 E-9.13; X-RANK exceptions as explicit tables).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RankCurve {
    /// `max·(r + 1)/(R + 1)` (default).
    #[default]
    Linear,
    /// Explicit value per rank 0 ..= R (affine X-RANK rows and rank-gated lines are written as tables).
    Table(Vec<f64>),
    /// The same value at every rank.
    Fixed,
}

impl RankCurve {
    /// The value at rank `r` of an item with maximum rank `max_rank` (R) whose maximum-rank value is `max` (09 E-9.13):
    /// `Linear` = `max·(r + 1)/(R + 1)` in f64; `Table` = the entry of rank r (ranks 0 ..= R; a rank beyond the table
    /// takes its last entry, an empty table gives `max`); `Fixed` = `max`. A rank above R is read as R. The single
    /// evaluator of the per-rank rule: the engine (L1) and 09's TV-09-06 test call it (contract review API-16).
    pub fn value(&self, max: f64, r: u8, max_rank: u8) -> f64 {
        let r = r.min(max_rank);
        match self {
            RankCurve::Linear => max * (f64::from(r) + 1.0) / (f64::from(max_rank) + 1.0),
            RankCurve::Table(v) => v
                .get(usize::from(r))
                .or_else(|| v.last())
                .copied()
                .unwrap_or(max),
            RankCurve::Fixed => max,
        }
    }
}

/// Number binding to the upstream text (`"n1%"`, `"n2"`, `"n3x"`, domain-model.md §15.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "string"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Binding(pub Box<str>);

/// Readable form of [`ValueExpr`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ValueRepr {
    Num(f64),
    Table(Box<ValueTable>),
}

/// Table form of [`ValueRepr`]: exactly one of the key groups `max` (+ `rank`, `bind`), `variants`, `param`,
/// `base` (+ `add`), `scalar` + `values`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ValueTable {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<RankCurve>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<Binding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variants: Option<BTreeMap<Box<str>, f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<ParamKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<Box<ValueExpr>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(as = "Option<Vec<Term>>"))]
    pub add: Option<SmallVec<[Term; 1]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scalar: Option<Per>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<f64>>,
}

impl From<&ValueExpr> for ValueRepr {
    fn from(v: &ValueExpr) -> Self {
        let t = match v {
            ValueExpr::Const(x) => return ValueRepr::Num(*x),
            ValueExpr::Ranked { max, rank, bind } => ValueTable {
                max: Some(*max),
                rank: (*rank != RankCurve::Linear).then(|| rank.clone()),
                bind: bind.clone(),
                ..Default::default()
            },
            ValueExpr::Variants { by } => ValueTable {
                variants: Some(by.clone()),
                ..Default::default()
            },
            ValueExpr::Param(p) => ValueTable {
                param: Some(p.clone()),
                ..Default::default()
            },
            ValueExpr::Linear { base, add } => ValueTable {
                base: Some(base.clone()),
                add: Some(add.clone()),
                ..Default::default()
            },
            ValueExpr::TableBy { scalar, values } => ValueTable {
                scalar: Some(scalar.clone()),
                values: Some(values.clone()),
                ..Default::default()
            },
        };
        ValueRepr::Table(Box::new(t))
    }
}

impl TryFrom<ValueRepr> for ValueExpr {
    type Error = String;
    fn try_from(r: ValueRepr) -> Result<Self, String> {
        let t = match r {
            ValueRepr::Num(x) => return Ok(ValueExpr::Const(x)),
            ValueRepr::Table(t) => *t,
        };
        let groups = [
            t.max.is_some() || t.rank.is_some() || t.bind.is_some(),
            t.variants.is_some(),
            t.param.is_some(),
            t.base.is_some() || t.add.is_some(),
            t.scalar.is_some() || t.values.is_some(),
        ];
        if groups.iter().filter(|g| **g).count() != 1 {
            return Err("a value table needs exactly one of `max` (with `rank`, `bind`), `variants`, `param`, \
                        `base` (with `add`), `scalar` (with `values`)"
                .into());
        }
        if let Some(by) = t.variants {
            return Ok(ValueExpr::Variants { by });
        }
        if let Some(p) = t.param {
            return Ok(ValueExpr::Param(p));
        }
        if t.base.is_some() || t.add.is_some() {
            let base = t.base.ok_or("`add` needs `base`")?;
            return Ok(ValueExpr::Linear {
                base,
                add: t.add.unwrap_or_default(),
            });
        }
        if t.scalar.is_some() || t.values.is_some() {
            return Ok(ValueExpr::TableBy {
                scalar: t.scalar.ok_or("`values` needs `scalar`")?,
                values: t.values.ok_or("`scalar` needs `values`")?,
            });
        }
        Ok(ValueExpr::Ranked {
            max: t.max.ok_or("a ranked value needs `max`")?,
            rank: t.rank.unwrap_or_default(),
            bind: t.bind,
        })
    }
}

dual_serde!(ValueExpr, ValueRepr);

// ---------------------------------------------------------------------------------------------------------------
// Scalars
// ---------------------------------------------------------------------------------------------------------------

/// Scalars an effect value can be multiplied by. Static scalars are resolved at L2 (build); dynamic ones become
/// state variables (§12). TOML: `"stack"`, `{ status_stacks = "COR" }`, `{ shard_count = ["crimson"] }`,
/// `{ resolved = { stat = "sc_of.PUN", curve = { step = 0.10 } } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Per {
    /// Own stack counter (dynamic; class of its trigger).
    Stack,
    /// n, the CO count (dynamic, class a; T-05).
    DistinctStatus,
    /// Stacks of one status on the target (class a).
    StatusStacks(StatusType),
    /// Presence of one status on the target (class a).
    StatusPresent(StatusType),
    /// M(c) (combo model; T-10 may fix it).
    ComboTier,
    /// M(c) − 1.
    ComboTierAboveOne,
    /// Sniper combo multiplier (06 §7.2; T-12 may fix it).
    SniperCombo,
    /// Static: equipped pieces of the item's set (09 E-9.40).
    SetPiece,
    /// Static: equipped shards of these colours.
    ShardCount(#[cfg_attr(feature = "ts", ts(as = "Vec<ShardColor>"))] SmallVec<[ShardColor; 3]>),
    /// Static: T-20 or the buff's caster Strength (04 E-4.10).
    Strength,
    /// Static after stage ordering (Melee Doughty, Plated Round).
    Resolved {
        /// The resolved stat read.
        stat: ResolvedStat,
        /// The curve applied to it.
        curve: ResolvedCurve,
    },
    /// Dynamic, schedule-driven: (Mag − left)/Mag of the latest reload (06 ReloadRec).
    ReloadedFraction,
    /// Static: the caster's Ability Duration Dur_e (04 E-4.10, T-33; Prismatic Gem "SD +1.00 × Duration", Lavos
    /// passive). Addition, contract review DC-15.
    AbilityDuration,
    /// Static: the attack's Primary Compression effectiveness eff_a (04 §3.1 "Compression Effectiveness") when the
    /// attack's compression mode is this one, else 0 (no compression data: "Doesn't Work"). The `Multiplies` term
    /// goes to the G_F group, the `Adds` term to A (TOML `{ compression_eff = "multiplies" }`). Addition, contract
    /// review DC-8.
    CompressionEff(CompressionMode),
}

/// Bucket of Primary Compression on an attack (04 §3.1, the "Compression Effectiveness" column of
/// `W:Primary_Compression`): "Multiplies" attacks get the G_F factor, "Adds" attacks the same value in A. The attack
/// data carries the mode next to eff_a (`Attack.compression_mode` beside `Attack.compression_eff`); the default is
/// `multiplies` ("1.00 for most"). Addition, contract review DC-8 (CONTRACTS.md "Contract review fixes (model-core)"
/// and "(model-records)").
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CompressionMode {
    /// The page's "Adds": the value enters A (Ambassador and Battacor alt-fire, Opticor, Trumna …).
    Adds,
    /// The page's "Multiplies": one factor `1 + value` of the G_F group.
    #[default]
    Multiplies,
}

/// A curve applied to a resolved stat.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "ResolvedCurveRepr"))]
pub enum ResolvedCurve {
    /// `floor(x / step)` (Melee Doughty: +CD per 10 % Puncture status chance). TOML `{ step = 0.10 }`.
    Step { step: f64 },
    /// `(scale · x)^exp` (Plated Round: `0.15·sqrt(5·Mag)` = value 0.15, `Pow { scale: 5, exp: 0.5 }`).
    /// TOML `{ pow = { scale = 5.0, exp = 0.5 } }`.
    Pow { scale: f64, exp: f64 },
}

/// Readable form of [`ResolvedCurve`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ResolvedCurveRepr {
    Step(f64),
    Pow { scale: f64, exp: f64 },
}

impl From<&ResolvedCurve> for ResolvedCurveRepr {
    fn from(c: &ResolvedCurve) -> Self {
        match *c {
            ResolvedCurve::Step { step } => ResolvedCurveRepr::Step(step),
            ResolvedCurve::Pow { scale, exp } => ResolvedCurveRepr::Pow { scale, exp },
        }
    }
}

impl TryFrom<ResolvedCurveRepr> for ResolvedCurve {
    type Error = String;
    fn try_from(r: ResolvedCurveRepr) -> Result<Self, String> {
        Ok(match r {
            ResolvedCurveRepr::Step(step) if step > 0.0 => ResolvedCurve::Step { step },
            ResolvedCurveRepr::Step(step) => {
                return Err(format!("step must be positive, found {step}"));
            }
            ResolvedCurveRepr::Pow { scale, exp } => ResolvedCurve::Pow { scale, exp },
        })
    }
}

dual_serde!(ResolvedCurve, ResolvedCurveRepr);

/// A resolved stat of the attack that a scalar (`Per::Resolved`) or condition (`Cond::Attack`) reads. Name form:
/// `"cc"`, `"sc_of.PUN"` (the per-pellet chance of a proc of that type, SC·w_s), `"magazine_normal_form"` …
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolvedStat {
    /// Final crit chance incl. Puncture (E-5).
    Cc,
    /// Final crit damage multiplier (E-4).
    Cd,
    /// Final status chance (E-8).
    Sc,
    /// Per-pellet proc chance of one status type, SC·w_s (E-9).
    ScOf(StatusType),
    /// Final multishot (E-10).
    Multishot,
    /// Modded fire rate (06 §5.1).
    FireRate,
    /// Modded magazine of the attack (06 §5.5).
    Magazine,
    /// Modded magazine of the weapon's normal (non-Incarnon) form (Plated Round, TV-04-16).
    MagazineNormalForm,
    /// Modded reload time (06 §5.7).
    ReloadTime,
    /// Modded attack speed (07 §5.2).
    AttackSpeed,
    /// Modded combo duration (07 E-7.12).
    ComboDuration,
    /// Initial Combo (07 E-7.15).
    InitialCombo,
    /// B', the base total of the attack (E-1).
    BaseTotal,
    /// Modded AoE radius R_mod of the attack (m; 06 E-6.20: radius × (1 + ΣRadius); Primary Compression, 04 §3.1).
    /// Addition, contract review DC-8.
    Radius,
}

impl ResolvedStat {
    const SIMPLE: [(ResolvedStat, &'static str); 13] = [
        (ResolvedStat::Cc, "cc"),
        (ResolvedStat::Cd, "cd"),
        (ResolvedStat::Sc, "sc"),
        (ResolvedStat::Multishot, "multishot"),
        (ResolvedStat::FireRate, "fire_rate"),
        (ResolvedStat::Magazine, "magazine"),
        (ResolvedStat::MagazineNormalForm, "magazine_normal_form"),
        (ResolvedStat::ReloadTime, "reload_time"),
        (ResolvedStat::AttackSpeed, "attack_speed"),
        (ResolvedStat::ComboDuration, "combo_dur"),
        (ResolvedStat::InitialCombo, "initial_combo"),
        (ResolvedStat::BaseTotal, "base_total"),
        (ResolvedStat::Radius, "radius"),
    ];

    /// Every name (for the JSON schema).
    pub fn all_names() -> Vec<String> {
        let mut v: Vec<String> = Self::SIMPLE.iter().map(|(_, n)| (*n).to_owned()).collect();
        v.extend(
            StatusType::ALL
                .iter()
                .map(|s| format!("sc_of.{}", s.name())),
        );
        v
    }
}

impl fmt::Display for ResolvedStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let ResolvedStat::ScOf(s) = self {
            return write!(f, "sc_of.{}", s.name());
        }
        let name = Self::SIMPLE
            .iter()
            .find(|(k, _)| k == self)
            .map(|(_, n)| *n)
            .unwrap_or("?");
        f.write_str(name)
    }
}

impl FromStr for ResolvedStat {
    type Err = UnknownName;
    fn from_str(s: &str) -> Result<Self, UnknownName> {
        if let Some(t) = s.strip_prefix("sc_of.") {
            return StatusType::from_str(t).map(ResolvedStat::ScOf);
        }
        Self::SIMPLE
            .iter()
            .find(|(_, n)| *n == s)
            .map(|(k, _)| *k)
            .ok_or_else(|| UnknownName {
                what: "resolved stat",
                name: s.to_owned(),
            })
    }
}

string_serde!(ResolvedStat, ResolvedStat::all_names());

// ---------------------------------------------------------------------------------------------------------------
// Conditions
// ---------------------------------------------------------------------------------------------------------------

/// Dynamic condition of an effect.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "CondRepr"))]
pub enum Cond {
    #[default]
    Always,
    /// A 04 §4.1 trigger with an independent chance per event; class (b) or (c) by id. `hit_unit` says what one hit
    /// is for on-hit triggers (04 §4.2; addition, CONTRACTS.md T01a). TOML
    /// `when = { trigger = "on_damaged", chance = 0.21, bind_chance = "n1%" }`.
    Trigger {
        /// The event.
        trigger: TriggerId,
        /// Chance per event at maximum rank (fraction, default 1).
        chance: f64,
        /// What one hit is for on-hit triggers.
        hit_unit: HitUnit,
        /// Binding of `chance` to the upstream text (authoring only).
        bind_chance: Option<Binding>,
        /// Per-rank rule of `chance` (09 E-9.13); `None` = the same chance at every rank. TOML
        /// `chance_rank = "linear"`. Addition (CONTRACTS.md, T01a fixes).
        chance_rank: Option<RankCurve>,
    },
    /// Class (a). TOML `when = { target = "frozen" }`, `when = { target = { health_below = 0.5 } }`.
    Target(TargetPred),
    /// Schedule-driven in every mode. TOML `when = { shot = "first_of_magazine" }`.
    Shot(ShotPos),
    /// Formula on the resolved attack, not T-02. TOML `when = { attack = { stat_below = { stat = "cc", x = 0.4 } } }`.
    Attack(AttackPred),
    /// Every condition holds. As the gain event of stacks (the effect's `when` without `StackModel::gain_on`, or
    /// `gain_on` itself) it holds at most one `Trigger`: the event; the other conjuncts are filters evaluated at the
    /// event (target state before the instance, attack predicates), so `all = [on_hit, { target = { has_status =
    /// "ELE" } }]` gains on hits on an Electricity-affected target. As an activity condition (`when` with `gain_on`)
    /// every conjunct is evaluated at use (domain-model.md §12.1; contract review DC-10).
    All(Vec<Cond>),
    /// At least one condition holds (same reading of triggers and filters as `All`).
    Any(Vec<Cond>),
}

impl Cond {
    /// `true` for the unconditional [`Cond::Always`] (the readable form then omits `when`).
    pub fn is_always(&self) -> bool {
        *self == Cond::Always
    }
    /// A class (b)/(c) trigger with chance 1, per pellet.
    pub fn trigger(trigger: TriggerId) -> Cond {
        Cond::Trigger {
            trigger,
            chance: 1.0,
            hit_unit: HitUnit::Pellet,
            bind_chance: None,
            chance_rank: None,
        }
    }
}

/// Readable form of [`Cond`]: a table with exactly one of `trigger`, `target`, `shot`, `attack`, `all`, `any`
/// (`chance`, `hit_unit`, `bind_chance`, `chance_rank` only with `trigger`); the empty table is `Always`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CondRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<TriggerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_unit: Option<HitUnit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind_chance: Option<Binding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance_rank: Option<RankCurve>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<TargetPred>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot: Option<ShotPos>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack: Option<AttackPred>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all: Option<Vec<Cond>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub any: Option<Vec<Cond>>,
}

impl From<&Cond> for CondRepr {
    fn from(c: &Cond) -> Self {
        let d = CondRepr::default();
        match c {
            Cond::Always => d,
            Cond::Trigger {
                trigger,
                chance,
                hit_unit,
                bind_chance,
                chance_rank,
            } => CondRepr {
                trigger: Some(trigger.clone()),
                chance: (*chance != 1.0).then_some(*chance),
                hit_unit: (*hit_unit != HitUnit::Pellet).then_some(*hit_unit),
                bind_chance: bind_chance.clone(),
                chance_rank: chance_rank.clone(),
                ..d
            },
            Cond::Target(p) => CondRepr {
                target: Some(p.clone()),
                ..d
            },
            Cond::Shot(p) => CondRepr {
                shot: Some(p.clone()),
                ..d
            },
            Cond::Attack(p) => CondRepr {
                attack: Some(p.clone()),
                ..d
            },
            Cond::All(v) => CondRepr {
                all: Some(v.clone()),
                ..d
            },
            Cond::Any(v) => CondRepr {
                any: Some(v.clone()),
                ..d
            },
        }
    }
}

impl TryFrom<CondRepr> for Cond {
    type Error = String;
    fn try_from(r: CondRepr) -> Result<Self, String> {
        let kinds = [
            r.trigger.is_some(),
            r.target.is_some(),
            r.shot.is_some(),
            r.attack.is_some(),
            r.all.is_some(),
            r.any.is_some(),
        ];
        let n = kinds.iter().filter(|k| **k).count();
        if n > 1 {
            return Err("a condition has exactly one of `trigger`, `target`, `shot`, `attack`, `all`, `any`".into());
        }
        if r.trigger.is_none()
            && (r.chance.is_some()
                || r.hit_unit.is_some()
                || r.bind_chance.is_some()
                || r.chance_rank.is_some())
        {
            return Err(
                "`chance`, `hit_unit`, `bind_chance` and `chance_rank` need `trigger`".into(),
            );
        }
        if let Some(trigger) = r.trigger {
            let chance = r.chance.unwrap_or(1.0);
            if !(0.0..=1.0).contains(&chance) {
                return Err(format!("trigger chance must be in [0, 1], found {chance}"));
            }
            return Ok(Cond::Trigger {
                trigger,
                chance,
                hit_unit: r.hit_unit.unwrap_or_default(),
                bind_chance: r.bind_chance,
                chance_rank: r.chance_rank,
            });
        }
        Ok(if let Some(p) = r.target {
            Cond::Target(p)
        } else if let Some(p) = r.shot {
            Cond::Shot(p)
        } else if let Some(p) = r.attack {
            Cond::Attack(p)
        } else if let Some(v) = r.all {
            Cond::All(v)
        } else if let Some(v) = r.any {
            Cond::Any(v)
        } else {
            Cond::Always
        })
    }
}

dual_serde!(Cond, CondRepr);

/// What one "hit" is for on-hit triggers and counters (04 §4.1-4.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HitUnit {
    /// Each pellet instance (default).
    #[default]
    Pellet,
    /// All pellets of one shot count once.
    Shot,
    /// Every damage instance, extra hits included.
    Instance,
    /// One per melee input (light, slide, aerial or wall input), whatever its number of strikes or enemies (04 §4.1
    /// `hit_unit = input`, Melee Animosity; 07 `input_start`). Addition, CONTRACTS.md T01d.
    Input,
}

/// Class of a trigger (04 §4.1).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TriggerClass {
    /// (b) own events.
    OwnEvent,
    /// (c) external.
    External,
}

/// Trigger ids of 04 §4.1: every (b) and (c) id of its table, plus `OnMeleeInput` and `OnThrow` (07 §7.5, §9.4).
/// `OnBigCrit`, `OnFire`, `OnMiss`, `OnDodge` and `HealthLevel` were added by T01a; `WhileFiring`, `OnPlainHit`,
/// `WhileAirborne`, `WhileCrouching`, `WhileMoving` and `OnPull` by T01d (CONTRACTS.md), appended after the T01a ids so
/// the compact form's variant indices stay unchanged. 04's `hits_within(k, Δt)` ("k hit units within Δt s": Final
/// Tap, Triple Tap, Sudden Justice) is `CountWithin { of: OnHit, n: k, window: Δt }`, the hits counted in the
/// `hit_unit` of the condition. TOML: `"on_kill"`, `"on_plain_hit"`, `{ on_status = "SLA" }`, `{ combo = 3 }`,
/// `{ count_within = { of = "on_headshot", n = 2, window = 2.0 } }`, `{ on_melee_input = "light" }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TriggerId {
    // (b) own events
    OnHit,
    OnCrit,
    OnTier1Crit,
    /// Crit tier k ≥ 2.
    OnBigCrit,
    OnWeakPointHit,
    OnHeadshot,
    OnStatus(StatusType),
    OnReload,
    OnReloadFromEmpty,
    OnFire,
    OnMiss,
    WhileAiming,
    FirstShot,
    LastShot,
    /// `combo(c)` of 04 §4.1: the combo multiplier M(c) is at least n. A combo-state predicate read from the combo
    /// model (07, or fixed by T-10) per instance, like `Per::ComboTier`: an effect conditioned on it derives the class
    /// COMBO, not (b), so T-02 does not switch it off (Amanata Pressure's M(c) ≥ 8; domain-model.md §12.1). Doc,
    /// contract review DC-24.
    Combo(u8),
    OnMeleeHitCount(u16),
    NextShotAfter(Box<TriggerId>),
    /// n events of `of` within `window` s (Phenmor Lingering Judgement: 2 headshots within 2 s); class (b).
    CountWithin {
        /// The counted event.
        of: Box<TriggerId>,
        /// Events needed.
        n: u8,
        /// Window length (s).
        window: f64,
    },
    /// Light / slide / aerial / wall input start (07 §7.5 `input_start`).
    OnMeleeInput(InputKind),
    /// Glaive or thrown-weapon throw (07 §9.4).
    OnThrow,
    // (c) external
    OnKill,
    OnWeakPointKill,
    OnMeleeKill,
    OnPrimaryKill,
    OnAbilityCast,
    WhileInvisible,
    OnWallLatch,
    OnHolster,
    OnDamaged,
    OnRoll,
    OnDodge,
    WhileOvershields,
    OnShieldBreak,
    InZone,
    AllyBuffs,
    EnergyLevel,
    ShieldLevel,
    ArmorLevel,
    HealthLevel,
    // Added by T01d (04 §4.1 as the spec rounds left it; CONTRACTS.md T01d).
    /// (b) While the weapon fires (Flux Overdrive, 04 §5.3).
    WhileFiring,
    /// (b) A hit unit (the condition's `hit_unit`) in which no instance crits (k = 0) and no instance delivers a
    /// status proc (04 §4.1; E-4.9 rate: hit units/s × Π over the unit's instances of P(k = 0)·P(no delivered proc |
    /// k = 0), 08 E-8.15). Laetum Overwhelming Attrition (04 §5.12, D-410, TV-04-47).
    OnPlainHit,
    /// (c) While airborne (Soaring Strike, 04 §5.3).
    WhileAirborne,
    /// (c) While crouching (Lie In Wait, 04 §5.3).
    WhileCrouching,
    /// (c) While moving (04 §4.1).
    WhileMoving,
    /// (c) Pulling an enemy with the alternate fire (Exposing Harpoon, 04 §5.3).
    OnPull,
    // Added by the contract review (CONTRACTS.md, "Contract review fixes (model-core)").
    /// (b) `on_status(any)` of 04 §5.12: the instance delivers at least one status proc (π_any of 02 E-2.17; several
    /// statuses in one hit count once). Gotva Prime passive. Addition, contract review DC-2.
    OnAnyStatus,
}

impl TriggerId {
    /// The T-02 class of the trigger (domain-model.md §12.1; 04 §4.1).
    pub fn class(&self) -> TriggerClass {
        use TriggerId::*;
        match self {
            OnHit
            | OnCrit
            | OnTier1Crit
            | OnBigCrit
            | OnWeakPointHit
            | OnHeadshot
            | OnStatus(_)
            | OnReload
            | OnReloadFromEmpty
            | OnFire
            | OnMiss
            | WhileAiming
            | FirstShot
            | LastShot
            | Combo(_)
            | OnMeleeHitCount(_)
            | NextShotAfter(_)
            | CountWithin { .. }
            | OnMeleeInput(_)
            | OnThrow
            | WhileFiring
            | OnPlainHit
            | OnAnyStatus => TriggerClass::OwnEvent,
            OnKill | OnWeakPointKill | OnMeleeKill | OnPrimaryKill | OnAbilityCast
            | WhileInvisible | OnWallLatch | OnHolster | OnDamaged | OnRoll | OnDodge
            | WhileOvershields | OnShieldBreak | InZone | AllyBuffs | EnergyLevel | ShieldLevel
            | ArmorLevel | HealthLevel | WhileAirborne | WhileCrouching | WhileMoving | OnPull => {
                TriggerClass::External
            }
        }
    }
}

/// Target-state predicates (class (a)).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TargetPred {
    HasStatus(StatusType),
    StatusCountAtLeast(u8),
    DistinctBelow(u8),
    Frozen,
    /// The target carries a mark (Mecha set, 04 §9). Not simulated: true under `all_max`, under `simulated` the
    /// effect's uptime (`EffectDef::uptime`, a `CondOverride`, default 1; domain-model.md §12.2).
    Marked,
    /// The target is incapacitated. Not simulated: as `Marked`.
    Incapacitated,
    /// Health fraction below x (live only in the layered pass).
    HealthBelow(f64),
    /// The target carries the named ability debuff (`"doomed"`: Grave Spirit; `"reap"`: Shadow Haze; 04 §5.6). A
    /// class (a) sub-condition that is never simulated: as `Marked`. The name is data (the label and override key),
    /// never matched by the engine. Addition, contract review DC-17.
    Debuffed(Box<str>),
    /// The scenario's target is an Eximus (05 §7, T-23; Eximus Advantage, 04 §5.3). Static per scenario: a gate,
    /// evaluated in every mode and never assumed (derived class GATE). Addition, contract review DC-17.
    Eximus,
}

/// Shot position conditions (schedule-driven in every mode, 00 §6.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ShotPos {
    /// The first shot of a magazine.
    FirstOfMagazine,
    /// The last shot of a magazine.
    LastOfMagazine {
        /// Only on weapons whose base magazine is at least this (rounds; Synth Charge 6).
        #[serde(default)]
        min_base_magazine: u16,
    },
}

/// Conditions on the resolved attack (formula, evaluated in every mode).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum AttackPred {
    /// The resolved stat is below x.
    StatBelow { stat: ResolvedStat, x: f64 },
    /// The resolved stat is at least x.
    StatAtLeast { stat: ResolvedStat, x: f64 },
}

// ---------------------------------------------------------------------------------------------------------------
// Stacks
// ---------------------------------------------------------------------------------------------------------------

/// Own stack counter of an effect (04 §4.2 field names and values). `consume_on`, `reset_on` and the `idle` decay
/// are 04 §4.2 fields added here (CONTRACTS.md, T01a). TOML
/// `stacks = { max = 2, bind_max = "n4", duration = 20.0, bind_duration = "n3", stack_timer = "shared_lose_one" }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StackModel {
    /// Stack cap m at maximum rank; with `unbounded`, the default stack count of the user input instead.
    pub max: u16,
    /// No cap (04 §4.2 `max_stacks = none`, 04 §5.5 Magus Accelerant "no limit"): the effect is a class (c) or (u)
    /// user count whose `CondOverride::Stacks` may exceed `max` (`max` is its default). Event-driven counters
    /// without a cap are `counter_bonus` with `unbounded` (ruling C-4, CONTRACTS.md T01d).
    #[serde(default)]
    pub unbounded: bool,
    /// Stacks gained per trigger event (g).
    #[serde(default = "crate::serde_util::default_one_u16")]
    pub gain: u16,
    /// Seconds at maximum rank; 0 = no expiry, in every class (E-4.9 with D = ∞): user-driven stacks of class (c)
    /// and (u), and event-driven stacks without a stated timer (Strun Blazing Barrel, reset on reload; Critical
    /// Precision; Arca Titron charges). Doc, contract review DC-21.
    #[serde(default)]
    pub duration: f64,
    /// Per-rank rule of `max` (09 E-9.13 covers stack caps; X-RANK: Double Tap's table 80, 40, 26, 20). `None` = the
    /// same cap at every rank; `linear` = `max·(r + 1)/(R + 1)`; a table lists ranks 0 ..= R and its last entry
    /// equals `max`. Addition (CONTRACTS.md, T01a fixes).
    #[serde(default)]
    pub max_rank: Option<RankCurve>,
    /// Per-rank rule of `duration` (09 E-9.13 covers durations; X-RANK: Primary Plated Round
    /// `{ table = [3.0, 4.0, 5.0, 6.0, 8.0, 10.0] }`, Double Tap `linear`). `None` = the same at every rank.
    #[serde(default)]
    pub duration_rank: Option<RankCurve>,
    /// Expiry rule.
    #[serde(default)]
    pub stack_timer: StackTimer,
    /// Timer refresh rule.
    #[serde(default)]
    pub refresh: Refresh,
    /// Value decay.
    #[serde(default)]
    pub decay: ValueDecay,
    /// Use that consumes the stacks.
    #[serde(default)]
    pub consume_on: ConsumeOn,
    /// Event that resets all stacks.
    #[serde(default)]
    pub reset_on: Option<ResetOn>,
    /// Binding of `max` (authoring only).
    #[serde(default)]
    pub bind_max: Option<Binding>,
    /// Binding of `duration` (authoring only).
    #[serde(default)]
    pub bind_duration: Option<Binding>,
    /// Default stack count of a user count (class (c) or (u) stacks whose count `m_e` is a user input, 04 §4.3);
    /// `None` = `max`. Piercing Navigator: `max = 5`, `default_count = 0` (04 §5.3 "user count, default 0").
    /// Addition, contract review DC-22.
    #[serde(default)]
    pub default_count: Option<u16>,
    /// When an instance reads the stacks (contract review DC-9): `before_instance` (default) is E-4.8's value(τ⁻)
    /// (CD-04); `after_shot` counts the gains of the instance's own shot first, so every pellet of a shot reads the
    /// count including that shot's hitting pellets (Double Tap, 04 §3.2: H includes this shot's hitting pellets;
    /// TV-04-25).
    #[serde(default)]
    pub read: ReadTiming,
    /// The gain event with its event filters, when it differs from the activity condition: then the effect's `when`
    /// is the condition under which the stacks apply (evaluated at use) and this is the event that adds stacks
    /// (Argon Scope: `when = { trigger = "while_aiming" }`, `gain_on = { trigger = "on_weak_point_hit" }`).
    /// `None` = the effect's `when` is the gain event (domain-model.md §12.1). Its class joins the effect's derived
    /// class. Addition, contract review DC-10.
    #[serde(default)]
    pub gain_on: Option<Cond>,
}

/// When an instance reads its effect's stacks ([`StackModel::read`]).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ReadTiming {
    /// The state before the instance (04 E-4.8 value(τ⁻), CD-04).
    #[default]
    BeforeInstance,
    /// After the gains of the instance's own shot (04 §3.2 Double Tap).
    AfterShot,
}

/// Default uptime of an effect's conditions ([`EffectDef::uptime`], 04 §4.3): on for `on` s in every `period` s.
/// TOML `uptime = { on = { scalar = "set_piece", values = [3.0, 6.0, 9.0, 12.0] }, period = { scalar = "set_piece",
/// values = [60.0, 45.0, 30.0, 15.0] } }` (the Mecha mark, 04 §9).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UptimeSpec {
    /// On-time of one cycle (s).
    pub on: ValueExpr,
    /// Period of one cycle (s); u = on/period.
    pub period: ValueExpr,
}

/// Expiry rule of a stack counter (04 §4.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum StackTimer {
    #[default]
    SharedLoseOne,
    SharedLoseAll,
    Independent,
}

/// Timer refresh rule (04 §4.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Refresh {
    #[default]
    Refresh,
    NoRefreshWhileActive,
}

/// Value decay (04 §4.2): `linear` over the duration, `garuda` (04 §3.1), `idle` after `t_idle` s without a trigger.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ValueDecay {
    #[default]
    None,
    Linear,
    Garuda,
    Idle {
        /// Seconds without a trigger before the decay starts.
        t_idle: f64,
        /// Value lost per second while decaying (fraction).
        rate: f64,
    },
}

/// The buff is used up by the next qualifying use (04 §4.2 `consume_on`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ConsumeOn {
    #[default]
    None,
    NextShot,
    NextHeavy,
    NextShots(u8),
}

/// All stacks return to 0 after `count` events of `on` (04 §4.2 `reset_on`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ResetOn {
    /// The reset event.
    pub on: TriggerId,
    /// Events needed (default 1).
    #[serde(default = "crate::serde_util::default_one_u16")]
    pub count: u16,
}

// ---------------------------------------------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------------------------------------------

/// Static applicability of an effect, resolved per attack and hit kind at L1. TOML
/// `scope = { weapon = { slots = ["melee"] } }`, `scope = { attacks = { any_of = ["heavy"] } }`,
/// `scope = { hit = { body_part = ["weak_point"] } }`, `scope = { form = "normal" }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Scope {
    /// Weapons the effect applies to.
    #[serde(default)]
    pub weapon: WeaponFilter,
    /// Attack roles the effect applies to.
    #[serde(default)]
    pub attacks: RoleFilter,
    /// Weapon form (normal or Incarnon).
    #[serde(default)]
    pub form: FormSel,
    /// Damage instances the effect applies to.
    #[serde(default)]
    pub hit: HitFilter,
    /// Default all three.
    #[serde(default)]
    pub layers: LayerSet,
}

impl Scope {
    /// `true` for the default scope (no restriction).
    pub fn is_any(&self) -> bool {
        *self == Scope::default()
    }
}

/// Weapon part of a scope. Empty lists accept every weapon. `melee_classes` and `traits` are additions for the
/// weapon-class scopes of 04 §4.2 (`weapon_class_scope`: glaive, battery, beam …; CONTRACTS.md, T01a).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponFilter {
    /// Equipment slots; empty = any.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<WeaponSlot>"))]
    pub slots: SmallVec<[WeaponSlot; 2]>,
    /// Weapon classes; empty = any.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<WeaponClass>"))]
    pub classes: SmallVec<[WeaponClass; 2]>,
    /// Melee classes; empty = any.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<MeleeClass>"))]
    pub melee_classes: SmallVec<[MeleeClass; 2]>,
    /// Category names of `Rules.categories`.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<String>"))]
    pub categories: SmallVec<[Box<str>; 2]>,
    /// Compatibility tag names of `Rules.tags`.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<String>"))]
    pub tags: SmallVec<[Box<str>; 2]>,
    /// The weapon must have every listed trait.
    #[serde(default)]
    pub traits: WeaponTraits,
    /// Exalted-weapon filter.
    #[serde(default)]
    pub exalted: ExaltedSel,
    /// The weapon must have none of these traits (Resupply: "not battery weapons", 04 §6.1). Addition, contract
    /// review DC-24.
    #[serde(default)]
    pub none_traits: WeaponTraits,
    /// Weapons named by uniqueName; empty = any. Like every list of the filter it is combined with the others by
    /// "and" (Enraged: Hysteria only; Primal Rage: Iron Staff only; Excalibur's passive reaches Exalted Blade through
    /// a second effect naming it; 04 §5.6). The names are data; the engine compares them with the weapon's uniqueName
    /// (I-2). Addition, contract review DC-24.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<UniqueName>"))]
    pub weapons: SmallVec<[UniqueName; 1]>,
}

/// Exalted-weapon filter.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ExaltedSel {
    #[default]
    Any,
    Never,
    Only,
}

/// Attack-role part of a scope: the attack has any role of `any_of` (empty = any), none of `none_of`, and one of the
/// fire-mode `triggers` (empty = any).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RoleFilter {
    /// The attack has one of these roles (empty = any).
    #[serde(default)]
    pub any_of: AttackRoles,
    /// The attack has none of these roles.
    #[serde(default)]
    pub none_of: AttackRoles,
    /// The attack's trigger class (`FireSpec.trigger`, 06 §4.2) is one of these; empty = any (Critical Precision:
    /// "Burst Fire Only", 04 §5.3). Addition, contract review DC-24.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<Trigger>"))]
    pub triggers: SmallVec<[Trigger; 2]>,
}

/// Weapon form filter.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FormSel {
    #[default]
    Any,
    Normal,
    Incarnon,
}

/// Hit part of a scope. Empty lists accept every kind and body part.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HitFilter {
    /// Instance kinds; empty = every kind.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<HitKind>"))]
    pub kinds: SmallVec<[HitKind; 4]>,
    /// Gate bound to T-03, never assumed.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<BodyPartSel>"))]
    pub body_part: SmallVec<[BodyPartSel; 3]>,
    /// Pellet filter.
    #[serde(default)]
    pub pellet: PelletSel,
    /// Hit-outcome gate on the instance's own rolls (domain-model.md §11.1; no 04 catalogue row uses it since Laetum
    /// Overwhelming Attrition became the `on_plain_hit` trigger, D-410).
    #[serde(default)]
    pub outcome: Option<OutcomeFilter>,
    /// The instance carries every listed flag (07 §7.5 instance flags: `tennokai` for the Tennokai mods, 04 §5.3,
    /// "Tennokai attacks only"). A gate, never assumed. Addition, contract review DC-6.
    #[serde(default)]
    pub flags: InstanceFlags,
    /// The scenario's target distance d (T-27) is strictly greater than this many metres (Spring-Loaded Broadhead 15,
    /// Range Advantage 10; 04 §3.2, §5.3: "a target distance (T-27) … applies only to instances that have that
    /// property, in every mode"). A gate, never assumed. Addition, contract review DC-7.
    #[serde(default)]
    pub min_distance: Option<f64>,
}

bitflags::bitflags! {
    /// Instance flags a scope can require (07 §7.5). Readable form: lowercase names. Addition, contract review DC-6.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct InstanceFlags: u16 {
        /// A heavy or heavy slam executed inside a Tennokai window (07 E-7.17, §7.5 `tennokai`).
        const TENNOKAI = 1 << 0;
    }
}
flags_serde!(InstanceFlags, u16, Lower);

/// Kind of damage instance.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HitKind {
    Direct,
    Radial,
    DotTick,
    ExtraHit,
}

/// Body part of a hit (T-03).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum BodyPartSel {
    Body,
    Head,
    WeakPoint,
}

/// Pellet filter.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PelletSel {
    #[default]
    Any,
    /// Only pellets j ≥ 1 of a shot (multishot-created pellets).
    ExtraOnly,
}

/// Hit-outcome gate: the effect applies only to instances whose own rolls satisfy it (`{ crit = "none", status =
/// "none" }`). Procs of extra hits do not count. EV: the kernel adds the term only on the branch k = 0 with weight
/// P(k = 0)·P(no proc of the instance); MC: evaluated live after the rolls. A DSL construct of the architecture review
/// (work-breakdown.md §2.4, R-2): no 04 catalogue row uses it since 04's second tail round made Laetum Overwhelming
/// Attrition an on-hit buff whose trigger is [`TriggerId::OnPlainHit`] and which later instances use whatever their
/// own rolls (04 §5.12, D-410).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OutcomeFilter {
    /// Crit tier of the instance.
    #[serde(default)]
    pub crit: CritOutcome,
    /// Procs of the instance.
    #[serde(default)]
    pub status: ProcOutcome,
}

/// Crit-tier outcome of an instance.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CritOutcome {
    #[default]
    Any,
    None,
    Exactly(u8),
    AtLeast(u8),
}

/// Proc outcome of an instance.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ProcOutcome {
    #[default]
    Any,
    None,
    Some,
}

/// Value × factor on a sub-scope.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ScopeFactor {
    /// The sub-scope.
    pub when: Scope,
    /// Factor on the value inside the sub-scope.
    pub factor: f64,
}

/// Element ordering class (domain-model.md §13; 01 §4.10.1 positions, E-1.8 sub-indices). 04 §4.2 `position`
/// values: `mod_slot` = `Slot` (and `Riven` for riven rows), `warframe_arcane` = `WarframeArcane`, `innate_group` =
/// `Innate`, `non_combining` = `NonCombining`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ElemOrder {
    /// The slot the item sits in, at the slot's `SlotDef.element_order`: main slots 0-7, exilus 8, the weapon arcane
    /// slots and the Incarnon perk slots 9 (sub-index n for `arcane<n>`, the evolution t (2-5) for `inc<t>`, so no
    /// two slots tie; 01 §4.10.1, E-1.8; contract review T03b-R2, UNVERIFIED until the 01 spec round confirms).
    Slot,
    /// The riven's slot, lines bottom-up.
    Riven,
    /// After modded elements, with innate and progenitor elements, in HCET order.
    Innate,
    /// Added to its type after combination (01 §4.10.3).
    NonCombining,
    /// Warframe-arcane elements (Theorem Demulcent): position 10, after every weapon-side position and before the
    /// innate group, sub-index = the HCET index of the element (HEA 0, COL 1, ELE 2, TOX 3); 01 §4.10.1, E-1.8,
    /// TV-01-38; 04 §4.2 `position = warframe_arcane`. Addition, CONTRACTS.md T01d.
    WarframeArcane,
}

/// A weak-spot source σ of 02 E-2.10 (toggle T-32, spec 00 §6.1): the effect's value is the factor s_σ of one
/// highlighted node (Sonar 2 / 3 / 4 / 5 × Str by ability rank, Detect Vulnerability up to 2.75; 02 §4.10 "Weak
/// spots", 04 §5.6), and the two named params of the item hold the user's location L_σ and node count ν_σ, so the
/// T-32 inputs are `buffs[].params` of the scenario. The engine multiplies the sources as
/// `M_son = Π_σ s_σ^(ν_σ·spot_σ)` (not the "members add" rule of other groups), and a source with spot_σ = 1 on a
/// direct hit makes the instance a weak-point hit (domain-model.md §14). TOML
/// `spot = { location = "spot_location", nodes = "spot_nodes" }`. Addition, CONTRACTS.md T01d.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SpotSource {
    /// Choice param of the item holding L_σ: `"hit_part"` (the T-03 part; the param's default) or `"torso"` (the
    /// body part; reaches radial instances), 02 E-2.10.
    pub location: ParamKey,
    /// Number param of the item holding ν_σ ≥ 1 (the exponent of s_σ; Sonar default 1); `None` = one node
    /// (Detect Vulnerability, "always 1", spec 00 §6.1 T-32).
    #[serde(default)]
    pub nodes: Option<ParamKey>,
}

/// Vulnerability mask: the damage types and layers a vulnerability multiplies (05 E-5.19). TOML
/// `vuln = { types = ["HEA"] }`, `vuln = { types = { param = "element" } }`,
/// `vuln = { types = ["PUN"], layers = ["OG", "SH", "HP"] }`.
///
/// Defaults follow the spec, not domain-model.md §11.1 ("layers default all"): a mask without `layers` applies to
/// health only (05 §13 "An entry without an explicit mask applies to health only (Decision)"; 04 §4.2 `layer_mask`
/// default `{HP}`, §5.7 "`{HP}` unless the U37 list above names it"), and a mask without `types` applies to every
/// type (04 §4.2 `type_mask` default `all`). Request C-14 of CONTRACTS.md.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VulnMask {
    /// Damage types the vulnerability applies to; default every type.
    #[serde(default = "VulnMask::default_types")]
    pub types: TypeSel,
    /// Layers the vulnerability applies to; default `{HP}` (not the all-layer default of [`LayerSet`]).
    #[serde(default = "VulnMask::default_layers")]
    pub layers: LayerSet,
}

impl VulnMask {
    /// Default of `types`: every damage type (04 §4.2 `type_mask` default `all`).
    pub fn default_types() -> TypeSel {
        TypeSel::Set(DamageTypeSet::all())
    }

    /// Default of `layers`: health only (05 §13, 04 §4.2 `layer_mask` default `{HP}`).
    pub fn default_layers() -> LayerSet {
        LayerSet::HP
    }
}

/// Damage-type selector of a vulnerability mask.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "TypeSelRepr"))]
pub enum TypeSel {
    Set(DamageTypeSet),
    /// An item parameter (Theorem Contagion: orb type).
    Param(ParamKey),
}

/// Readable form of [`TypeSel`]: a list of type IDs or `{ param = "element" }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TypeSelRepr {
    Set(DamageTypeSet),
    Param { param: ParamKey },
}

impl From<&TypeSel> for TypeSelRepr {
    fn from(s: &TypeSel) -> Self {
        match s {
            TypeSel::Set(t) => TypeSelRepr::Set(*t),
            TypeSel::Param(p) => TypeSelRepr::Param { param: p.clone() },
        }
    }
}

impl From<TypeSelRepr> for TypeSel {
    fn from(r: TypeSelRepr) -> Self {
        match r {
            TypeSelRepr::Set(t) => TypeSel::Set(t),
            TypeSelRepr::Param { param } => TypeSel::Param(param),
        }
    }
}

dual_serde!(TypeSel, TypeSelRepr);

bitflags::bitflags! {
    /// Condition classes of an effect (domain-model.md §12.1; class (u) of 04 §4.1). Readable form: lowercase names.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct ClassSet: u16 {
        /// (a) target state.
        const A = 1 << 0;
        /// (b) own events.
        const B = 1 << 1;
        /// (c) external.
        const C = 1 << 2;
        /// (u) user-enabled buff (04 §4.1, OQ-431).
        const U = 1 << 3;
        /// Combo model (07/06 or fixed by T-10/T-12).
        const COMBO = 1 << 4;
        /// Schedule-driven in every mode.
        const SCHEDULE = 1 << 5;
        /// Formula on the resolved attack.
        const FORMULA = 1 << 6;
        /// Gate (body part, hit outcome).
        const GATE = 1 << 7;
        /// Static scalar.
        const STATIC = 1 << 8;
    }
}
flags_serde!(ClassSet, u16, Lower);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_forms() {
        let e: EffectDef = toml::from_str(
            r#"
stat = "cc_rel"
value = { max = 0.40, rank = "linear", bind = "n1%" }
per = ["combo_tier_above_one"]
scope = { weapon = { slots = ["melee"] } }
"#,
        )
        .unwrap();
        assert_eq!(e.target, Target::Stat(StatKey::CcRel));
        assert_eq!(
            e.value,
            ValueExpr::Ranked {
                max: 0.40,
                rank: RankCurve::Linear,
                bind: Some(Binding("n1%".into()))
            }
        );
        assert_eq!(e.scope.weapon.slots.as_slice(), &[WeaponSlot::Melee]);
        let back = toml::to_string(&e).unwrap();
        assert_eq!(toml::from_str::<EffectDef>(&back).unwrap(), e);
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(serde_json::from_str::<EffectDef>(&json).unwrap(), e);
        let bytes = postcard::to_stdvec(&e).unwrap();
        assert_eq!(postcard::from_bytes::<EffectDef>(&bytes).unwrap(), e);
    }

    #[test]
    fn rejects_bad_effects() {
        assert!(
            toml::from_str::<EffectDef>("stat = \"cc_rel\"\nelement = \"HEA\"\nvalue = 1.0")
                .is_err()
        );
        assert!(toml::from_str::<EffectDef>("value = 1.0").is_err());
        assert!(toml::from_str::<EffectDef>("stat = \"cc_rell\"\nvalue = 1.0").is_err());
        assert!(toml::from_str::<EffectDef>("stat = \"cc_rel\"\nvalue = 1.0\nbogus = 1").is_err());
        assert!(
            toml::from_str::<EffectDef>(
                "stat = \"cc_rel\"\nvalue = { max = 1.0, param = \"value\" }"
            )
            .is_err()
        );
        assert!(
            toml::from_str::<EffectDef>("stat = \"cc_rel\"\nvalue = 1.0\nwhen = { chance = 0.5 }")
                .is_err()
        );
    }

    #[test]
    fn derived_classes() {
        let ga: EffectDef = toml::from_str(
            "stat = \"co_per_status\"\nvalue = 0.4\nper = [\"stack\"]\nwhen = { trigger = \"on_kill\" }\nstacks = { max = 2, duration = 20.0 }",
        )
        .unwrap();
        // Galvanized Aptitude's CO: (c) through the trigger and (a) through `co_per_status` (domain-model.md §16.2).
        assert_eq!(ga.derived_class(), ClassSet::A | ClassSet::C);
        // Condition Overload (§16.3) and Secondary Shiver (§16.13) are class (a), like Per::DistinctStatus.
        let co: EffectDef = toml::from_str("stat = \"co_per_status\"\nvalue = 0.80").unwrap();
        assert_eq!(co.derived_class(), ClassSet::A);
        let shiver: EffectDef =
            toml::from_str("stat = \"co_per_cold_stack\"\nvalue = 0.45").unwrap();
        assert_eq!(shiver.derived_class(), ClassSet::A);
        // A counter that raises a CO bucket is (b) and (a).
        let ctr: EffectDef = toml::from_str(
            "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"co_per_status\"\nper_event = \"hit\"\nmax = 3",
        )
        .unwrap();
        assert_eq!(ctr.derived_class(), ClassSet::A | ClassSet::B);
        // The constant CO terms are not a count: only their condition classifies them.
        let flat: EffectDef = toml::from_str("stat = \"co_flat\"\nvalue = 1.0").unwrap();
        assert!(flat.derived_class().is_empty());
        let br: EffectDef =
            toml::from_str("stat = \"cc_rel\"\nvalue = 0.4\nper = [\"combo_tier_above_one\"]")
                .unwrap();
        assert_eq!(br.derived_class(), ClassSet::COMBO);
        let pr: EffectDef = toml::from_str(
            "stat = \"base_dmg\"\nvalue = 0.15\nper = [\"reloaded_fraction\"]\nwhen = { trigger = \"on_reload\" }",
        )
        .unwrap();
        assert_eq!(pr.derived_class(), ClassSet::B | ClassSet::SCHEDULE);
        let ac: EffectDef = toml::from_str(
            "stat = \"cc_rel\"\nvalue = 1.0\nscope = { hit = { body_part = [\"weak_point\"] } }\nwhen = { target = \"frozen\" }",
        )
        .unwrap();
        assert_eq!(ac.derived_class(), ClassSet::A | ClassSet::GATE);
        let plain: EffectDef = toml::from_str("stat = \"cc_rel\"\nvalue = 1.0").unwrap();
        assert!(plain.derived_class().is_empty());
    }

    #[test]
    fn conditions() {
        let c: Cond = serde_json::from_str(
            r#"{"trigger":{"count_within":{"of":"on_headshot","n":2,"window":2.0}}}"#,
        )
        .unwrap();
        match &c {
            Cond::Trigger {
                trigger: TriggerId::CountWithin { of, n: 2, .. },
                chance,
                ..
            } => {
                assert_eq!(**of, TriggerId::OnHeadshot);
                assert_eq!(*chance, 1.0);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(TriggerId::OnKill.class(), TriggerClass::External);
        assert_eq!(TriggerId::OnBigCrit.class(), TriggerClass::OwnEvent);
        let c2: Cond = serde_json::from_str(r#"{"target":{"health_below":0.5}}"#).unwrap();
        assert_eq!(c2, Cond::Target(TargetPred::HealthBelow(0.5)));
        assert_eq!(serde_json::from_str::<Cond>("{}").unwrap(), Cond::Always);
        let rs: ResolvedStat = "sc_of.PUN".parse().unwrap();
        assert_eq!(rs, ResolvedStat::ScOf(StatusType::PUN));
        assert_eq!(rs.to_string(), "sc_of.PUN");
    }

    /// The trigger ids T01d added from 04 §4.1 parse from their readable names and have the class of the 04 §4.1
    /// table: `while_firing` and `on_plain_hit` (b), the others (c).
    #[test]
    fn trigger_ids_added_by_t01d() {
        for (name, id, class) in [
            (
                "while_firing",
                TriggerId::WhileFiring,
                TriggerClass::OwnEvent,
            ),
            (
                "on_plain_hit",
                TriggerId::OnPlainHit,
                TriggerClass::OwnEvent,
            ),
            (
                "while_airborne",
                TriggerId::WhileAirborne,
                TriggerClass::External,
            ),
            (
                "while_crouching",
                TriggerId::WhileCrouching,
                TriggerClass::External,
            ),
            (
                "while_moving",
                TriggerId::WhileMoving,
                TriggerClass::External,
            ),
            ("on_pull", TriggerId::OnPull, TriggerClass::External),
        ] {
            let e: EffectDef = toml::from_str(&format!(
                "stat = \"base_dmg\"\nvalue = 1.0\nwhen = {{ trigger = \"{name}\" }}"
            ))
            .unwrap_or_else(|err| panic!("{name}: {err}"));
            match &e.when {
                Cond::Trigger { trigger, .. } => assert_eq!(trigger, &id, "{name}"),
                other => panic!("{name}: {other:?}"),
            }
            assert_eq!(id.class(), class, "{name}");
            let back = toml::to_string(&e).unwrap();
            assert!(back.contains(&format!("trigger = \"{name}\"")), "{back}");
            let bytes = postcard::to_stdvec(&e).unwrap();
            assert_eq!(
                postcard::from_bytes::<EffectDef>(&bytes).unwrap(),
                e,
                "{name}"
            );
        }
        // Laetum Overwhelming Attrition (04 §5.12, D-410): +4.00 per stack, 3 stacks, 10 s, `shared_lose_one`, gained
        // from `on_plain_hit` per pellet; class (b).
        let laetum: EffectDef = toml::from_str(
            r#"stat = "base_dmg"
value = 4.0
per = ["stack"]
when = { trigger = "on_plain_hit", hit_unit = "pellet" }
stacks = { max = 3, duration = 10.0, stack_timer = "shared_lose_one" }"#,
        )
        .unwrap();
        assert_eq!(laetum.derived_class(), ClassSet::B);
        // 04 `hits_within(4, 0.05)` (Final Tap) is `count_within` of `on_hit` in the condition's hit unit.
        let final_tap: EffectDef = toml::from_str(
            r#"stat = "base_dmg"
value = 1.0
when = { trigger = { count_within = { of = "on_hit", n = 4, window = 0.05 } }, hit_unit = "pellet" }
stacks = { max = 1, duration = 0.2 }"#,
        )
        .unwrap();
        match &final_tap.when {
            Cond::Trigger {
                trigger: TriggerId::CountWithin { of, n: 4, window },
                hit_unit: HitUnit::Pellet,
                ..
            } => {
                assert_eq!(**of, TriggerId::OnHit);
                assert_eq!(*window, 0.05);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(final_tap.derived_class(), ClassSet::B);
    }

    /// 04 §4.2 `position = warframe_arcane` (01 §4.10.1 position 10: Theorem Demulcent).
    #[test]
    fn warframe_arcane_element_order() {
        let e: EffectDef = toml::from_str(
            "element = { param = \"element\" }\nvalue = 1.80\norder = \"warframe_arcane\"",
        )
        .unwrap();
        assert_eq!(e.order, Some(ElemOrder::WarframeArcane));
        let back = toml::to_string(&e).unwrap();
        assert!(back.contains("order = \"warframe_arcane\""), "{back}");
        let bytes = postcard::to_stdvec(&e).unwrap();
        assert_eq!(postcard::from_bytes::<EffectDef>(&bytes).unwrap(), e);
    }

    /// 09 E-9.13 through [`RankCurve::value`] (contract review API-16): Serration 1.65 at R = 10, an X-RANK table,
    /// a fixed value, ranks above R.
    #[test]
    fn rank_curve_values() {
        assert_eq!(RankCurve::Linear.value(1.65, 10, 10), 1.65);
        assert!((RankCurve::Linear.value(1.65, 4, 10) - 1.65 * 5.0 / 11.0).abs() < 1e-15);
        assert!((RankCurve::Linear.value(0.6, 0, 5) - 0.1).abs() < 1e-15);
        let t = RankCurve::Table(vec![3.0, 4.0, 5.0, 6.0, 8.0, 10.0]);
        assert_eq!(t.value(10.0, 4, 5), 8.0);
        assert_eq!(t.value(10.0, 9, 5), 10.0);
        assert_eq!(RankCurve::Table(vec![]).value(2.0, 1, 5), 2.0);
        assert_eq!(RankCurve::Fixed.value(0.3, 0, 5), 0.3);
    }

    /// Classes and checks of the constructs added by the contract review (DC-7, DC-10, DC-11, DC-17, DC-24).
    #[test]
    fn contract_review_constructs() {
        // Argon Scope: active while aiming, stacks gained on weak-point hits: (b) through both.
        let argon: EffectDef = toml::from_str(
            "stat = \"cc_rel\"\nvalue = 1.35\nper = [\"stack\"]\nwhen = { trigger = \"while_aiming\" }\nstacks = { max = 1, duration = 9.0, gain_on = { trigger = \"on_weak_point_hit\" } }",
        )
        .unwrap();
        assert_eq!(argon.derived_class(), ClassSet::B);
        // A kill-gained stack used while aiming: (b) and (c).
        let bladed: EffectDef = toml::from_str(
            "stat = \"cd_rel\"\nvalue = 1.2\nper = [\"stack\"]\nwhen = { trigger = \"while_aiming\" }\nstacks = { max = 1, duration = 9.0, gain_on = { trigger = \"on_kill\" } }",
        )
        .unwrap();
        assert_eq!(bladed.derived_class(), ClassSet::B | ClassSet::C);
        // combo(n) is the combo model, not (b); the Eximus switch and the distance and Tennokai gates are gates.
        let amanata: EffectDef =
            toml::from_str("stat = \"cc_rel\"\nvalue = 1.0\nwhen = { trigger = { combo = 8 } }")
                .unwrap();
        assert_eq!(amanata.derived_class(), ClassSet::COMBO);
        let eximus: EffectDef = toml::from_str(
            "stat = \"base_dmg\"\nvalue = 6.0\nwhen = { all = [{ trigger = \"on_headshot\" }, { target = \"eximus\" }] }\nstacks = { max = 1, duration = 10.0 }",
        )
        .unwrap();
        assert_eq!(eximus.derived_class(), ClassSet::B | ClassSet::GATE);
        let broadhead: EffectDef =
            toml::from_str("mult = \"spring_loaded_broadhead\"\nvalue = 0.4\nscope = { hit = { min_distance = 15.0 } }")
                .unwrap();
        assert_eq!(broadhead.derived_class(), ClassSet::GATE);
        let tennokai: EffectDef = toml::from_str(
            "stat = \"base_dmg\"\nvalue = 0.6\nscope = { hit = { flags = [\"tennokai\"] } }",
        )
        .unwrap();
        assert_eq!(tennokai.scope.hit.flags, InstanceFlags::TENNOKAI);
        assert_eq!(tennokai.derived_class(), ClassSet::GATE);
        let grave: EffectDef = toml::from_str(
            "stat = \"cd_rel\"\nvalue = 1.0\nwhen = { target = { debuffed = \"doomed\" } }",
        )
        .unwrap();
        assert_eq!(grave.derived_class(), ClassSet::A);
        assert_eq!(TriggerId::OnAnyStatus.class(), TriggerClass::OwnEvent);
        // `dot_scaling` only on element targets; `uptime` only with a condition.
        assert!(
            toml::from_str::<EffectDef>("element = \"ELE\"\nvalue = 0.5\ndot_scaling = false")
                .is_ok()
        );
        assert!(
            toml::from_str::<EffectDef>("stat = \"elem_nc.HEA\"\nvalue = 1.0\ndot_scaling = true")
                .is_ok()
        );
        assert!(
            toml::from_str::<EffectDef>("stat = \"cc_rel\"\nvalue = 0.5\ndot_scaling = false")
                .is_err()
        );
        assert!(
            toml::from_str::<EffectDef>(
                "stat = \"cc_rel\"\nvalue = 0.5\nuptime = { on = 3.0, period = 60.0 }"
            )
            .is_err()
        );
        let mecha: EffectDef = toml::from_str(
            "mult = \"mecha\"\nvalue = 1.5\nwhen = { target = \"marked\" }\nuptime = { on = { scalar = \"set_piece\", values = [3.0, 6.0, 9.0, 12.0] }, period = { scalar = \"set_piece\", values = [60.0, 45.0, 30.0, 15.0] } }",
        )
        .unwrap();
        for e in [&argon, &eximus, &broadhead, &tennokai, &grave, &mecha] {
            let t = toml::to_string(e).unwrap();
            assert_eq!(&toml::from_str::<EffectDef>(&t).unwrap(), e, "{t}");
            let p = postcard::to_stdvec(e).unwrap();
            assert_eq!(&postcard::from_bytes::<EffectDef>(&p).unwrap(), e);
        }
    }

    /// The T-32 binding (02 E-2.10): a `Spot` group member names the params that hold L_σ and ν_σ.
    #[test]
    fn spot_sources() {
        let sonar: EffectDef = toml::from_str(
            r#"mult = "sonar"
value = { param = "value" }
per = ["strength"]
spot = { location = "spot_location", nodes = "spot_nodes" }"#,
        )
        .unwrap();
        assert_eq!(
            sonar.spot,
            Some(SpotSource {
                location: ParamKey::new("spot_location"),
                nodes: Some(ParamKey::new("spot_nodes")),
            })
        );
        let dv: EffectDef = toml::from_str(
            "mult = \"detect_vulnerability\"\nvalue = 2.75\nspot = { location = \"spot_location\" }",
        )
        .unwrap();
        assert_eq!(dv.spot.as_ref().unwrap().nodes, None);
        for e in [&sonar, &dv] {
            let t = toml::to_string(e).unwrap();
            assert_eq!(&toml::from_str::<EffectDef>(&t).unwrap(), e, "{t}");
            let j = serde_json::to_string(e).unwrap();
            assert_eq!(&serde_json::from_str::<EffectDef>(&j).unwrap(), e, "{j}");
            let p = postcard::to_stdvec(e).unwrap();
            assert_eq!(&postcard::from_bytes::<EffectDef>(&p).unwrap(), e);
        }
        // Only on `mult` targets; unknown keys rejected.
        assert!(
            toml::from_str::<EffectDef>(
                "stat = \"wp_dmg\"\nvalue = 1.0\nspot = { location = \"spot_location\" }"
            )
            .is_err()
        );
        assert!(
            toml::from_str::<EffectDef>(
                "mult = \"sonar\"\nvalue = 5.0\nspot = { location = \"spot_location\", count = 2 }"
            )
            .is_err()
        );
    }
}
