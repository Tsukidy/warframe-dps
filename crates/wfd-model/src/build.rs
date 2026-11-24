//! Build documents (domain-model.md §9.1; slot ids, item fields and legality: spec 09 §2) and the resolved build
//! (§9.2). Documents reference everything by `StableKey`; the resolved form uses dense ids. Documents are JSON only
//! (API, share codes, local storage), so their optional fields are omitted when empty.
//!
//! TypeScript (server-api.md §2: the shapes are exactly the generated form): a field that serde omits is optional.
//! `Option` fields get `?: T` from `ts(optional_fields)`; a non-`Option` field with `skip_serializing_if` carries
//! `ts(optional = nullable)`, which ts-rs 12 renders as `?: T` for a non-`Option` type. `tests/ts_schema.rs` compares
//! the generated document types with server-api.md §4 and with the JSON that serde writes.

use crate::enums::{ComboType, DamageType, Polarity};
use crate::i18n::{MessageKey, MsgArg};
use crate::ids::{ComboKey, FrameId, ItemId, ParamKey, StableKey, StanceId, WeaponId};
use crate::item::{ParamValue, Riven};
use crate::serde_util::{default_true, is_false, is_true};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

/// The build part of a calculation input (09 §2.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BuildDoc {
    /// The weapon.
    pub weapon: StableKey,
    /// Item-level fields of the weapon (omitted when all default).
    #[serde(default, skip_serializing_if = "WeaponItemDoc::is_default")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub weapon_item: WeaponItemDoc,
    /// Modular parts and progenitor pick.
    #[serde(default)]
    pub variant: VariantDoc,
    /// Only non-empty slots, any order; canonical order on encode.
    #[serde(default)]
    pub slots: Vec<SlotEntry>,
    /// Selected combo type for light loops (07 §8.1); default `"neutral"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stance_combo: Option<ComboKey>,
    /// The warframe item (omitted when all default).
    #[serde(default, skip_serializing_if = "WarframeDoc::is_default")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub warframe: WarframeDoc,
    /// Declared companion / robotic-weapon set pieces (09 E-9.40, E-9.41).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub companion_set_mods: Vec<StableKey>,
    /// Kind of the companion that carries the declared pieces (09 E-9.41 `companion_kind`; addition, CONTRACTS.md).
    #[serde(default, skip_serializing_if = "CompanionKind::is_none")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub companion_kind: CompanionKind,
}

impl BuildDoc {
    /// An empty build of a weapon.
    pub fn of_weapon(weapon: StableKey) -> Self {
        BuildDoc {
            weapon,
            weapon_item: WeaponItemDoc::default(),
            variant: VariantDoc::default(),
            slots: Vec::new(),
            stance_combo: None,
            warframe: WarframeDoc::default(),
            companion_set_mods: Vec::new(),
            companion_kind: CompanionKind::None,
        }
    }
}

/// Item-level fields of the weapon (09 §2.1; all optional, defaults in brackets).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponItemDoc {
    /// Default: `MaxRank`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,
    /// [true].
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub catalyst: bool,
    /// Mod capacity instead of the rank's capacity (09 E-9.15).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_override: Option<u16>,
    /// `pol_gen0` … `pol_gen7`, `pol_exilus`, `pol_stance`; default: the innate polarity.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub polarities: BTreeMap<SlotId, Polarity>,
    /// Incarnon Genesis installed (09 E-9.60). `None` = the default of 09 §2.1: true when the weapon has an Incarnon
    /// line (spec 09 differs from domain-model.md's `bool` default false; CONTRACTS.md, T01a).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genesis_installed: Option<bool>,
}

impl Default for WeaponItemDoc {
    fn default() -> Self {
        WeaponItemDoc {
            rank: None,
            catalyst: true,
            capacity_override: None,
            polarities: BTreeMap::new(),
            genesis_installed: None,
        }
    }
}

impl WeaponItemDoc {
    /// `true` when every field has its default (the document then omits `weapon_item`).
    pub fn is_default(&self) -> bool {
        *self == WeaponItemDoc::default()
    }
}

/// Warframe item of the build (09 §2.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WarframeDoc {
    /// Optional identity → `Bundle.frames` (Helminth host rules 09 §10, aura count, polarities).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<StableKey>,
    /// [true].
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub reactor: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub polarities: BTreeMap<SlotId, Polarity>,
    /// [false]: warframe capacity is checked only when true (09 E-9.18).
    #[serde(default, skip_serializing_if = "is_false")]
    #[cfg_attr(feature = "ts", ts(optional = nullable))]
    pub frame_complete: bool,
}

impl Default for WarframeDoc {
    fn default() -> Self {
        WarframeDoc {
            frame: None,
            reactor: true,
            polarities: BTreeMap::new(),
            frame_complete: false,
        }
    }
}

impl WarframeDoc {
    /// `true` when every field has its default (the document then omits `warframe`).
    pub fn is_default(&self) -> bool {
        *self == WarframeDoc::default()
    }
}

/// Weapon variant inputs: modular parts (T-25) and the progenitor bonus (T-24).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(optional_fields))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kitgun: Option<KitgunPicks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zaw: Option<ZawPicks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progenitor: Option<ProgenitorPick>,
}

/// Kitgun grip and loader (T-25).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct KitgunPicks {
    pub grip: StableKey,
    pub loader: StableKey,
}

/// Zaw grip and link (T-25).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ZawPicks {
    pub grip: StableKey,
    pub link: StableKey,
}

/// Progenitor element and value in tenths of a percent: `progenitor_frac = v10/1000` (T-24, 09 E-9.70; legal
/// 250-579 or 600).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProgenitorPick {
    /// Progenitor element.
    pub element: DamageType,
    /// Value in tenths of a percent (250-579 or 600).
    pub v10: u16,
}

/// Companion kind of 09 E-9.41.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CompanionKind {
    #[default]
    None,
    Sentinel,
    Kubrow,
    Kavat,
    Predasite,
    Vulpaphyla,
    Moa,
    Hound,
}

impl CompanionKind {
    /// `true` for [`CompanionKind::None`] (the document then omits `companion_kind`).
    pub fn is_none(&self) -> bool {
        *self == CompanionKind::None
    }
}

/// A filled slot of a build document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SlotEntry {
    pub slot: SlotId,
    pub content: SlotContent,
}

/// Slot ids of spec 09 §2.1. JSON: `"gen0"` … `"gen7"`, `"exilus"`, `"stance"`, `"arcane0"`, `"arcane1"`,
/// `"inc2"` … `"inc5"`, `"fgen0"` … `"fgen7"`, `"fexilus"`, `"aura0"`, `"aura1"`, `"farcane0"`, `"farcane1"`,
/// `"shard0"` … `"shard4"`, `"subsumed"`. A riven is not a slot: it occupies a `gen` slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SlotId {
    /// 0..=7, reading order (the element order of 01 §4.10.1).
    Gen(u8),
    Exilus,
    Stance,
    /// 0..=1.
    Arcane(u8),
    /// Incarnon evolution 2..=5.
    Inc(u8),
    /// 0..=7.
    FGen(u8),
    FExilus,
    /// 0..=1 (`aura1` only on Jade).
    Aura(u8),
    /// 0..=1.
    FArcane(u8),
    /// 0..=4.
    Shard(u8),
    Subsumed,
}

impl SlotId {
    /// Every slot id in serialization order (09 §2.1).
    pub fn all() -> Vec<SlotId> {
        let mut v = Vec::new();
        v.extend((0..8).map(SlotId::Gen));
        v.push(SlotId::Exilus);
        v.push(SlotId::Stance);
        v.extend((0..2).map(SlotId::Arcane));
        v.extend((2..6).map(SlotId::Inc));
        v.extend((0..8).map(SlotId::FGen));
        v.push(SlotId::FExilus);
        v.extend((0..2).map(SlotId::Aura));
        v.extend((0..2).map(SlotId::FArcane));
        v.extend((0..5).map(SlotId::Shard));
        v.push(SlotId::Subsumed);
        v
    }

    /// `true` when the index is in its legal range.
    pub fn is_valid(self) -> bool {
        match self {
            SlotId::Gen(i) | SlotId::FGen(i) => i < 8,
            SlotId::Arcane(i) | SlotId::Aura(i) | SlotId::FArcane(i) => i < 2,
            SlotId::Inc(t) => (2..=5).contains(&t),
            SlotId::Shard(i) => i < 5,
            SlotId::Exilus | SlotId::Stance | SlotId::FExilus | SlotId::Subsumed => true,
        }
    }

    /// `true` for slots of the warframe item.
    pub fn is_warframe(self) -> bool {
        matches!(
            self,
            SlotId::FGen(_)
                | SlotId::FExilus
                | SlotId::Aura(_)
                | SlotId::FArcane(_)
                | SlotId::Shard(_)
                | SlotId::Subsumed
        )
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlotId::Gen(i) => write!(f, "gen{i}"),
            SlotId::Exilus => f.write_str("exilus"),
            SlotId::Stance => f.write_str("stance"),
            SlotId::Arcane(i) => write!(f, "arcane{i}"),
            SlotId::Inc(t) => write!(f, "inc{t}"),
            SlotId::FGen(i) => write!(f, "fgen{i}"),
            SlotId::FExilus => f.write_str("fexilus"),
            SlotId::Aura(i) => write!(f, "aura{i}"),
            SlotId::FArcane(i) => write!(f, "farcane{i}"),
            SlotId::Shard(i) => write!(f, "shard{i}"),
            SlotId::Subsumed => f.write_str("subsumed"),
        }
    }
}

impl FromStr for SlotId {
    type Err = crate::enums::UnknownName;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || crate::enums::UnknownName {
            what: "slot id",
            name: s.to_owned(),
        };
        let fixed = match s {
            "exilus" => Some(SlotId::Exilus),
            "stance" => Some(SlotId::Stance),
            "fexilus" => Some(SlotId::FExilus),
            "subsumed" => Some(SlotId::Subsumed),
            _ => None,
        };
        if let Some(id) = fixed {
            return Ok(id);
        }
        // Longest prefixes first: "fgen" before "gen", "farcane" before "arcane".
        type Ctor = fn(u8) -> SlotId;
        let prefixes: [(&str, Ctor); 7] = [
            ("fgen", SlotId::FGen),
            ("farcane", SlotId::FArcane),
            ("gen", SlotId::Gen),
            ("arcane", SlotId::Arcane),
            ("inc", SlotId::Inc),
            ("aura", SlotId::Aura),
            ("shard", SlotId::Shard),
        ];
        for (p, make) in prefixes {
            if let Some(rest) = s.strip_prefix(p) {
                if rest.len() != 1 {
                    return Err(err());
                }
                let i = rest.parse::<u8>().map_err(|_| err())?;
                let id = make(i);
                return if id.is_valid() { Ok(id) } else { Err(err()) };
            }
        }
        Err(err())
    }
}

impl Serialize for SlotId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for SlotId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: std::borrow::Cow<'de, str> = Deserialize::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "ts")]
impl ts_rs::TS for SlotId {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;
    /// A union of string literals: maps keyed by `SlotId` become `{ [key in SlotId]?: V }`
    /// (`Partial<Record<SlotId, V>>`, server-api.md §4 `polarities`), not a mapped type with every key required.
    const IS_ENUM: bool = true;
    fn name(_: &ts_rs::Config) -> String {
        let alts: Vec<String> = SlotId::all().iter().map(|s| format!("\"{s}\"")).collect();
        alts.join(" | ")
    }
    fn inline(cfg: &ts_rs::Config) -> String {
        <Self as ts_rs::TS>::name(cfg)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for SlotId {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SlotId".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let ids: Vec<String> = SlotId::all().iter().map(|s| s.to_string()).collect();
        schemars::json_schema!({ "type": "string", "enum": ids })
    }
}

/// Content of a slot. JSON: `{ "item": { "item": 3301, "rank"?: 3, "params"?: {} } }`, `{ "riven": { … } }`,
/// `{ "perk": { "index": 0 } }`, `{ "shard": { "item": 7, "tauforged": false } }`,
/// `{ "subsumed": { "ability": 12, "replaced"?: "…" } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SlotContent {
    Item {
        item: StableKey,
        /// `None` = max rank.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "ts", ts(optional))]
        rank: Option<u8>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        #[cfg_attr(
            feature = "ts",
            ts(as = "Option<BTreeMap<ParamKey, ParamValue>>", optional)
        )]
        params: BTreeMap<ParamKey, ParamValue>,
    },
    /// Only in `gen` slots (a riven is not a slot, 09 §2.1).
    Riven(Riven),
    /// `inc<t>`: index into `IncarnonDef.tiers[t-1]`.
    Perk { index: u8 },
    /// Colour and bonus are the shard item.
    Shard { item: StableKey, tauforged: bool },
    /// 09 E-9.32: the buff item of the subsumed ability and the name of the replaced native ability.
    Subsumed {
        ability: StableKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "ts", ts(optional))]
        replaced: Option<Box<str>>,
    },
}

/// A rule violation of spec 09 (never blocks evaluation, 09 §1.5). JSON
/// `{ "key": "cap.exceeded", "severity": "illegal", "slots": ["gen3"], "args": {} }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Violation {
    /// The rule key of 09, e.g. `"cap.exceeded"`.
    pub key: MessageKey,
    pub severity: RuleSeverity,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<SlotId>"))]
    pub slots: SmallVec<[SlotId; 2]>,
    #[serde(default)]
    pub args: BTreeMap<Box<str>, MsgArg>,
}

/// Severity of a 09 rule (09 §1.5).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RuleSeverity {
    Illegal,
    IllegalUnverified,
    Warning,
}

// ---------------------------------------------------------------------------------------------------------------
// Resolved form (§9.2)
// ---------------------------------------------------------------------------------------------------------------

/// A build with dense ids, produced by `wfd-data::GameData::resolve`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Build {
    pub weapon: WeaponId,
    pub item: WeaponItem,
    pub variant: Variant,
    pub r#gen: [Slot; 8],
    pub exilus: Slot,
    pub stance: Option<StanceId>,
    /// Rank of the stance mod (`None` = max; X-STANCE stances are fixed at 3). Needed by 09 E-9.15 (addition,
    /// CONTRACTS.md T01a).
    pub stance_rank: Option<u8>,
    /// Selected combo type for light loops.
    pub combo: ComboType,
    /// `arcane0`, `arcane1` (families per 09 E-9.30): item and rank.
    pub arcanes: [Option<(ItemId, u8)>; 2],
    pub genesis_installed: bool,
    /// Perk index for evolutions 2..=5.
    pub incarnon: [Option<u8>; 4],
    pub frame: FrameLoadout,
    #[cfg_attr(feature = "ts", ts(as = "Vec<ItemId>"))]
    pub companion_set_mods: SmallVec<[ItemId; 4]>,
    pub companion_kind: CompanionKind,
}

/// Resolved item-level fields of the weapon.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponItem {
    /// Weapon rank.
    pub rank: u8,
    /// Orokin Catalyst installed (doubles capacity).
    pub catalyst: bool,
    /// Mod capacity instead of the rank's capacity.
    pub capacity_override: Option<u16>,
    /// Slot polarities after forma.
    pub polarities: BTreeMap<SlotId, Polarity>,
}

/// Resolved variant: part indices into `Bundle.modular` and the progenitor pick.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Variant {
    /// (grip, loader) indices into `ModularTables.grips` / `.loaders`.
    pub kitgun: Option<(u16, u16)>,
    /// (grip, link) indices into `ModularTables.grips` / `.links`.
    pub zaw: Option<(u16, u16)>,
    /// Progenitor element and value (T-24).
    pub progenitor: Option<ProgenitorPick>,
}

/// Content of a resolved mod slot.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Slot {
    /// No item.
    #[default]
    Empty,
    /// A mod, arcane or other item at a rank.
    Mod {
        /// The item.
        item: ItemId,
        /// Item rank.
        rank: u8,
        /// Parameter values.
        #[cfg_attr(feature = "ts", ts(as = "Vec<(ParamKey, ParamValue)>"))]
        params: SmallVec<[(ParamKey, ParamValue); 1]>,
    },
    /// A riven mod.
    Riven(Riven),
}

/// Resolved warframe item.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FrameLoadout {
    pub frame: Option<FrameId>,
    pub r#gen: [Slot; 8],
    pub exilus: Slot,
    pub auras: [Slot; 2],
    pub arcanes: [Option<(ItemId, u8)>; 2],
    pub shards: [Option<ShardPick>; 5],
    pub subsumed: Option<Subsumed>,
    pub polarities: BTreeMap<SlotId, Polarity>,
    pub reactor: bool,
    pub frame_complete: bool,
}

impl Default for FrameLoadout {
    fn default() -> Self {
        FrameLoadout {
            frame: None,
            r#gen: Default::default(),
            exilus: Slot::Empty,
            auras: Default::default(),
            arcanes: [None, None],
            shards: [None, None, None, None, None],
            subsumed: None,
            polarities: BTreeMap::new(),
            reactor: true,
            frame_complete: false,
        }
    }
}

/// An Archon shard on the warframe.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ShardPick {
    pub item: ItemId,
    pub tauforged: bool,
}

/// The Helminth-infused ability.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Subsumed {
    pub ability: ItemId,
    pub replaced: Option<Box<str>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_ids() {
        for id in SlotId::all() {
            let s = id.to_string();
            assert_eq!(s.parse::<SlotId>().unwrap(), id);
            assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{s}\""));
        }
        assert_eq!(SlotId::all().len(), 35);
        for bad in [
            "gen8", "inc1", "inc6", "shard5", "aura2", "gen", "gen01", "riven", "fgen-1",
        ] {
            assert!(bad.parse::<SlotId>().is_err(), "{bad}");
        }
    }
}
