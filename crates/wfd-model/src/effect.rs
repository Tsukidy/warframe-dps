//! The effect DSL (domain-model.md §11.1, §12, §13).
//!
//! The bundle form is the authoring form after lowering of sugar and binding; both use these types. Binding fields
//! (`bind`, `bind_max`, `bind_duration`, `bind_chance`, domain-model.md §15.2) exist in the types as `Option`s and are
//! `None` after lowering.
//!
//! Readable (TOML, JSON) forms follow the encoded examples of domain-model.md §16: the effect target is flattened
//! into the effect table (`stat = "cc_rel"`, `element = "HEA"`, `faction = "any"`, `mult = "eclipse"`,
//! `mechanic = "extra_hit"` with `[effect.params]`), values are untagged (`value = 1.65`,
//! `value = { max = 0.80, bind = "n1%" }`, `value = { param = "value" }`), conditions are tables with one kind key
//! (`when = { trigger = "on_damaged", chance = 0.21 }`).

use crate::enums::{
    AttackRoles, DamageType, DamageTypeSet, InputKind, LayerSet, MeleeClass, ShardColor,
    StatusType, UnknownName, WeaponClass, WeaponSlot, WeaponTraits,
};
use crate::ids::ParamKey;
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
    /// Only for `vuln_add` and TargetVuln groups (05 E-5.19).
    pub vuln: Option<VulnMask>,
    /// Optional declared class; the validator checks that it equals the derived one.
    pub class: Option<ClassSet>,
    /// UNVERIFIED notes, carried to the UI and traces.
    pub note: Option<Box<str>>,
    /// Values used when the shard is Tauforged, replacing `value × tauforged_factor` (integer stats such as the
    /// Emerald Corrosion cap, domain-model.md §16.13); empty = the factor rule.
    pub tauforged_values: SmallVec<[f64; 1]>,
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
        }
    }
}

impl EffectDef {
    /// The condition classes of the effect, derived by the table of domain-model.md §12.1 (never trusted from the
    /// author; the validator compares a declared `class` with this). Class (u) depends on the item kind (a buff) and
    /// is added by the caller. An empty set means unconditional.
    pub fn derived_class(&self) -> ClassSet {
        let mut c = self.when.derived_class();
        for p in &self.per {
            c |= p.derived_class();
        }
        if let Target::Mechanic(crate::mechanic::Mechanic::CounterBonus(_)) = &self.target {
            // Own hits, inputs and throws.
            c |= ClassSet::B;
        }
        if self.scope.hit.outcome.is_some() || !self.scope.hit.body_part.is_empty() {
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
            Cond::Trigger { trigger, .. } => match trigger.class() {
                TriggerClass::OwnEvent => ClassSet::B,
                TriggerClass::External => ClassSet::C,
            },
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
            Per::SetPiece | Per::ShardCount(_) | Per::Strength | Per::Resolved { .. } => {
                ClassSet::STATIC
            }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stat: Option<StatKey>,
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
            ..Default::default()
        };
        match &e.target {
            Target::Stat(k) => r.stat = Some(*k),
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
        if let Some(k) = r.stat {
            targets.push(Target::Stat(k));
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
    /// TOML `stat = "cc_rel"`, `stat = "forced.SLA"`, `stat = "stack_cap.COR"`.
    Stat(StatKey),
    /// TOML `element = "HEA"` | `"target_weakness"` | `{ param = "element" }`.
    Element(ElemSel),
    /// TOML `faction = "any"` | `["grineer", "kuva_grineer"]`.
    Faction(FactionSel),
    /// TOML `mult = "eclipse"` (group id from `Rules.mult_groups`).
    Mult(Box<str>),
    /// TOML `mechanic = "extra_hit"` + `[effect.params]`.
    Mechanic(Mechanic),
}

/// Element selector.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "ElemSelRepr"))]
pub enum ElemSel {
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
        max: f64,
        rank: RankCurve,
        bind: Option<Binding>,
    },
    /// TOML `value = { variants = { furis = 28.0, mk1_furis = 34.0 } }`: Incarnon per-variant values.
    Variants { by: BTreeMap<Box<str>, f64> },
    /// TOML `value = { param = "value" }`: a user parameter (buff value, custom stat value).
    Param(ParamKey),
    /// TOML `value = { base = 0.30, add = [{ coeff = 0.10, per = [...] }] }`: `base + Σ coeff·Π per`.
    Linear {
        base: Box<ValueExpr>,
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
    pub coeff: f64,
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
        stat: ResolvedStat,
        curve: ResolvedCurve,
    },
    /// Dynamic, schedule-driven: (Mag − left)/Mag of the latest reload (06 ReloadRec).
    ReloadedFraction,
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
}

impl ResolvedStat {
    const SIMPLE: [(ResolvedStat, &'static str); 12] = [
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
        trigger: TriggerId,
        chance: f64,
        hit_unit: HitUnit,
        bind_chance: Option<Binding>,
    },
    /// Class (a). TOML `when = { target = "frozen" }`, `when = { target = { health_below = 0.5 } }`.
    Target(TargetPred),
    /// Schedule-driven in every mode. TOML `when = { shot = "first_of_magazine" }`.
    Shot(ShotPos),
    /// Formula on the resolved attack, not T-02. TOML `when = { attack = { stat_below = { stat = "cc", x = 0.4 } } }`.
    Attack(AttackPred),
    All(Vec<Cond>),
    Any(Vec<Cond>),
}

impl Cond {
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
        }
    }
}

/// Readable form of [`Cond`]: a table with exactly one of `trigger`, `target`, `shot`, `attack`, `all`, `any`
/// (`chance`, `hit_unit`, `bind_chance` only with `trigger`); the empty table is `Always`.
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
            } => CondRepr {
                trigger: Some(trigger.clone()),
                chance: (*chance != 1.0).then_some(*chance),
                hit_unit: (*hit_unit != HitUnit::Pellet).then_some(*hit_unit),
                bind_chance: bind_chance.clone(),
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
            && (r.chance.is_some() || r.hit_unit.is_some() || r.bind_chance.is_some())
        {
            return Err("`chance`, `hit_unit` and `bind_chance` need `trigger`".into());
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

/// Trigger ids of 04 §4.1. `OnBigCrit`, `OnFire`, `OnMiss`, `OnDodge` and `HealthLevel` are listed by 04 §4.1 and
/// added here (CONTRACTS.md, T01a). TOML: `"on_kill"`, `{ on_status = "SLA" }`, `{ combo = 3 }`,
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
    Combo(u8),
    OnMeleeHitCount(u16),
    NextShotAfter(Box<TriggerId>),
    /// n events of `of` within `window` s (Phenmor Lingering Judgement: 2 headshots within 2 s); class (b).
    CountWithin {
        of: Box<TriggerId>,
        n: u8,
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
            | OnThrow => TriggerClass::OwnEvent,
            OnKill | OnWeakPointKill | OnMeleeKill | OnPrimaryKill | OnAbilityCast
            | WhileInvisible | OnWallLatch | OnHolster | OnDamaged | OnRoll | OnDodge
            | WhileOvershields | OnShieldBreak | InZone | AllyBuffs | EnergyLevel | ShieldLevel
            | ArmorLevel | HealthLevel => TriggerClass::External,
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
    Marked,
    Incapacitated,
    /// Health fraction below x (live only in the layered pass).
    HealthBelow(f64),
}

/// Shot position conditions (schedule-driven in every mode, 00 §6.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ShotPos {
    FirstOfMagazine,
    LastOfMagazine {
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
    StatBelow { stat: ResolvedStat, x: f64 },
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
    pub max: u16,
    #[serde(default = "crate::serde_util::default_one_u16")]
    pub gain: u16,
    /// Seconds; 0 = no timer (user-driven stacks of class (c) and (u)).
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub stack_timer: StackTimer,
    #[serde(default)]
    pub refresh: Refresh,
    #[serde(default)]
    pub decay: ValueDecay,
    #[serde(default)]
    pub consume_on: ConsumeOn,
    #[serde(default)]
    pub reset_on: Option<ResetOn>,
    #[serde(default)]
    pub bind_max: Option<Binding>,
    #[serde(default)]
    pub bind_duration: Option<Binding>,
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
        t_idle: f64,
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
    pub on: TriggerId,
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
    #[serde(default)]
    pub weapon: WeaponFilter,
    #[serde(default)]
    pub attacks: RoleFilter,
    #[serde(default)]
    pub form: FormSel,
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
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<WeaponSlot>"))]
    pub slots: SmallVec<[WeaponSlot; 2]>,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<WeaponClass>"))]
    pub classes: SmallVec<[WeaponClass; 2]>,
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
    #[serde(default)]
    pub exalted: ExaltedSel,
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

/// Attack-role part of a scope: the attack has any role of `any_of` (empty = any) and none of `none_of`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RoleFilter {
    #[serde(default)]
    pub any_of: AttackRoles,
    #[serde(default)]
    pub none_of: AttackRoles,
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
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<HitKind>"))]
    pub kinds: SmallVec<[HitKind; 4]>,
    /// Gate bound to T-03, never assumed.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<BodyPartSel>"))]
    pub body_part: SmallVec<[BodyPartSel; 3]>,
    #[serde(default)]
    pub pellet: PelletSel,
    /// Hit-outcome gate (Laetum Overwhelming Attrition).
    #[serde(default)]
    pub outcome: Option<OutcomeFilter>,
}

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

/// Hit-outcome gate: the effect applies only to instances whose own rolls satisfy it (Laetum Overwhelming
/// Attrition: `{ crit = "none", status = "none" }`). Procs of extra hits do not count. EV: the kernel adds the term
/// only on the branch k = 0 with weight P(k = 0)·P(no proc of the instance); MC: evaluated live after the rolls.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OutcomeFilter {
    #[serde(default)]
    pub crit: CritOutcome,
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
    pub when: Scope,
    pub factor: f64,
}

/// Element ordering class (§13).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ElemOrder {
    /// The slot the item sits in (exilus after slot 7).
    Slot,
    /// The riven's slot, lines bottom-up.
    Riven,
    /// After modded elements, with innate and progenitor elements, in HCET order.
    Innate,
    /// Added to its type after combination (01 §4.10.3).
    NonCombining,
}

/// Vulnerability mask: the damage types and layers a vulnerability multiplies (05 E-5.19). TOML
/// `vuln = { types = ["HEA"] }`, `vuln = { types = { param = "element" } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VulnMask {
    pub types: TypeSel,
    #[serde(default)]
    pub layers: LayerSet,
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
        // Galvanized Aptitude's CO: (c) through the trigger; (a) enters through `co_per_status` in the kernel.
        assert_eq!(ga.derived_class(), ClassSet::C);
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
}
