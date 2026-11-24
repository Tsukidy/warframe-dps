//! Catalogue routes (server-api.md §5.1-§5.5, §5.7): `GET /api/meta`, `/api/catalog`, `/api/weapons/{key}`,
//! `/api/items/{key}`, `/api/enemies/{key}`, `POST /api/candidates`.
//!
//! Entities are referenced by `StableKey`; dense ids never leave the server (server-api.md §2), so the catalogue's
//! item kind is the kind slug ([`ItemKindName`]), not the bundle's `ItemKind` (which carries dense ids).

use super::common::{FalloffView, LayerValues, PerDamageType};
use crate::build::SlotId;
use crate::bundle::Coverage;
use crate::enemy::EnemyFlags;
use crate::enums::{
    AttackRoles, ComboType, DamageType, MeleeClass, Polarity, Rarity, ShardColor, ShotType,
    StatusType, Trigger, WeaponClass, WeaponSlot, WeaponTraits,
};
use crate::ids::{AttackKey, ComboKey, NameKey, StableKey};
use crate::item::{EffectStatus, ParamKindName, ParamValue};
use crate::message::Message;
use crate::scenario::CalcInput;
use crate::serde_util::is_false;
use crate::stats::StatKey;
use crate::weapon::{CoBehaviour, DisplayValue, ModularKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------------------------------------------
// GET /api/meta
// ---------------------------------------------------------------------------------------------------------------

/// `GET /api/meta` (server-api.md §5.1). A client that finds `app_id = "warframe-dps"` on a port knows another
/// instance runs there (§1.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Meta {
    /// Always `"warframe-dps"`.
    pub app_id: AppId,
    /// Version of the binary.
    pub app_version: Box<str>,
    /// Version of this API ([`super::API_VERSION`]).
    pub api_version: u16,
    /// `BUNDLE_SCHEMA_VERSION` of the active bundle.
    pub schema_version: u32,
    /// Data version of the active bundle.
    pub data_version: Box<str>,
    /// Game version the data describes (spec 00 §1.1).
    pub game_baseline: Box<str>,
    /// The active bundle.
    pub bundle: BundleInfo,
    /// Inputs of the bundle.
    pub sources: Vec<SourceInfo>,
    /// Licences and attribution (pipeline.md §10).
    pub licences: Vec<LicenceInfo>,
    /// Counts per item kind and effect status.
    pub coverage: Coverage,
    /// Effective configuration (server-api.md §12.1).
    pub config: ConfigInfo,
    /// The active bundle has session-local keys (AD-18).
    pub local_keys: bool,
    /// Optional features compiled in.
    pub features: Features,
}

/// The application id of [`Meta::app_id`]: JSON `"warframe-dps"`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum AppId {
    #[default]
    #[serde(rename = "warframe-dps")]
    WarframeDps,
}

/// Identity of a bundle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BundleInfo {
    /// `"embedded"` or the user bundle's id.
    pub id: Box<str>,
    /// Where the bundle came from.
    pub source: BundleSource,
    /// Creation time (RFC 3339).
    pub created_at: Box<str>,
}

/// Where a bundle came from.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum BundleSource {
    /// Built into the binary.
    Embedded,
    /// Written by an in-app refresh (`<data_dir>/bundles/`).
    User,
}

/// One input of the active bundle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceInfo {
    /// `"wiki"`, `"pe"`, `"wfcd"`, `"spec"`, `"curated"`.
    pub source: Box<str>,
    /// Revid, content hash, tag or sha.
    pub revision: Box<str>,
    /// Fetch time (RFC 3339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub fetched_at: Option<Box<str>>,
}

/// Licence and attribution of one source.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LicenceInfo {
    /// Source name.
    pub source: Box<str>,
    /// Licence (`"CC BY-NC-SA 3.0"`, `"MIT"`).
    pub licence: Box<str>,
    /// Licence or source URL.
    pub url: Box<str>,
    /// Attribution text.
    pub attribution: Box<str>,
}

/// Effective server configuration (server-api.md §12.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ConfigInfo {
    /// Optimizer pool size.
    pub optimizer_threads: u16,
    /// Interactive (calc MC) pool size.
    pub interactive_threads: u16,
    /// Default `eval.mc_runs` of new scenarios.
    pub mc_runs_default: u32,
    /// Images are fetched on first use.
    pub images_fetch: bool,
}

/// Optional features compiled into the binary.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Features {
    /// In-app data refresh (feature `refresh`).
    pub refresh: bool,
}

// ---------------------------------------------------------------------------------------------------------------
// GET /api/catalog
// ---------------------------------------------------------------------------------------------------------------

/// `GET /api/catalog` (server-api.md §5.2): every summary the pickers need, fetched once per `data_version`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Catalog {
    /// Data version (also the ETag).
    pub data_version: Box<str>,
    /// Weapons.
    pub weapons: Vec<WeaponSummary>,
    /// Mods, arcanes, shards, perks, buffs, custom stats.
    pub items: Vec<ItemSummary>,
    /// Enemies.
    pub enemies: Vec<EnemySummary>,
    /// Warframes (AD-26, 09 §2.2, §10).
    pub frames: Vec<FrameSummary>,
    /// Factions.
    pub factions: Vec<FactionSummary>,
    /// Mod sets.
    pub sets: Vec<SetSummary>,
    /// Display names by NameKey (English now; per locale later).
    pub names: BTreeMap<NameKey, Box<str>>,
}

/// A weapon in the catalogue.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponSummary {
    /// StableKey.
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// Equipment slot.
    pub slot: WeaponSlot,
    /// Weapon class.
    pub class: WeaponClass,
    /// Traits (`prime`, `kuva_lich` …).
    pub traits: WeaponTraits,
    /// `GET /api/img/{key}` has an image.
    pub image: bool,
    /// Has an Incarnon line.
    pub incarnon: bool,
    /// Modular weapon kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub modular: Option<ModularKind>,
    /// Adversary weapon with a progenitor bonus (T-24).
    pub progenitor: bool,
    /// Mastery rank requirement.
    pub mastery: u8,
}

/// The kind slug of an item (`ItemKind::slug`): JSON `"weapon_mod"`, `"stance"`, … The bundle's `ItemKind` carries
/// dense ids, which never leave the server; the kind's details are separate catalogue fields (`exilus`, `color`,
/// `subsumable`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ItemKindName {
    WeaponMod,
    Stance,
    WeaponArcane,
    FrameMod,
    Aura,
    FrameArcane,
    Shard,
    IncarnonPerk,
    Buff,
    CustomStat,
}

impl ItemKindName {
    /// The kind name of a bundle item kind.
    pub fn of(kind: &crate::item::ItemKind) -> Self {
        use crate::item::ItemKind as K;
        match kind {
            K::WeaponMod { .. } => ItemKindName::WeaponMod,
            K::Stance { .. } => ItemKindName::Stance,
            K::WeaponArcane { .. } => ItemKindName::WeaponArcane,
            K::FrameMod { .. } => ItemKindName::FrameMod,
            K::Aura => ItemKindName::Aura,
            K::FrameArcane => ItemKindName::FrameArcane,
            K::Shard { .. } => ItemKindName::Shard,
            K::IncarnonPerk { .. } => ItemKindName::IncarnonPerk,
            K::Buff { .. } => ItemKindName::Buff,
            K::CustomStat { .. } => ItemKindName::CustomStat,
        }
    }
}

/// An item in the catalogue.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ItemSummary {
    /// StableKey.
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// Kind slug.
    pub kind: ItemKindName,
    /// Mod polarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub polarity: Option<Polarity>,
    /// Drain at rank 0 (negative for aura and stance mods, 09 E-9.12).
    pub drain: i16,
    /// Maximum rank.
    pub max_rank: u8,
    /// Rarity.
    pub rarity: Rarity,
    /// Set id (index of `Catalog.sets`, stable within one `data_version`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub set: Option<u16>,
    /// `GET /api/img/{key}` has an image.
    pub image: bool,
    /// Effect status (domain-model.md §15.3).
    pub status: EffectStatus,
    /// Stat keys the item touches (search by stat).
    pub stats: Vec<StatKey>,
    /// Exilus mod.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub exilus: Option<bool>,
    /// Shard colour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub color: Option<ShardColor>,
    /// Buff offered in the Helminth slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub subsumable: Option<bool>,
}

/// An enemy in the catalogue.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemySummary {
    /// StableKey.
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// Faction slug.
    pub faction: Box<str>,
    /// Base level.
    pub base_level: u16,
    /// Layers the enemy has (armor counts as a layer here).
    pub layers: Vec<EnemyLayer>,
}

/// A defence layer of the catalogue (`"OG"`, `"SH"`, `"HP"`, `"AR"`).
#[allow(clippy::upper_case_acronyms)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum EnemyLayer {
    /// Overguard.
    OG,
    /// Shield.
    SH,
    /// Health.
    HP,
    /// Armor.
    AR,
}

/// A warframe in the catalogue (AD-26).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FrameSummary {
    /// StableKey.
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// The base frame of a Prime or Umbra variant (09 E-9.32 H6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub variant_of: Option<StableKey>,
    /// Innate polarities of the general slots.
    pub polarities: Vec<Polarity>,
    /// Innate aura polarities.
    pub aura_polarities: Vec<Polarity>,
    /// Innate exilus polarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub exilus_polarity: Option<Polarity>,
    /// Number of aura slots (Jade: 2).
    pub aura_slots: u8,
    /// The four abilities.
    pub abilities: Vec<FrameAbilitySummary>,
    /// The native buff that a subsumed restricted buff replaces (Helminth Host, 09 E-9.32 H3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub helminth_host: Option<StableKey>,
    /// `GET /api/img/{key}` has an image.
    pub image: bool,
}

/// An ability of a warframe.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FrameAbilitySummary {
    /// English name.
    pub name: Box<str>,
    /// The buff item of a self-cast buff ability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub self_buff: Option<StableKey>,
}

/// A faction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FactionSummary {
    /// Slug (`"grineer"`).
    pub slug: Box<str>,
    /// English name.
    pub name: Box<str>,
}

/// A mod set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SetSummary {
    /// Set id (stable within one `data_version`).
    pub id: u16,
    /// English name.
    pub name: Box<str>,
    /// Member items.
    pub members: Vec<StableKey>,
}

// ---------------------------------------------------------------------------------------------------------------
// GET /api/weapons/{key}
// ---------------------------------------------------------------------------------------------------------------

/// `GET /api/weapons/{key}` (server-api.md §5.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponDetail {
    /// StableKey.
    pub key: StableKey,
    /// DE path or `wfd:` name.
    pub unique_name: Box<str>,
    /// English name.
    pub name: Box<str>,
    /// Equipment slot.
    pub slot: WeaponSlot,
    /// Weapon class.
    pub class: WeaponClass,
    /// Trigger of the main attack.
    pub trigger: Trigger,
    /// Magazine; absent for weapons without one (melee).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub magazine: Option<MagazineView>,
    /// Reload.
    pub reload: ReloadView,
    /// Attacks in data order (index = attack index).
    pub attacks: Vec<AttackDetail>,
    /// Attack groups (08 §9).
    pub groups: Vec<AttackGroupView>,
    /// Incarnon evolutions and perks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub incarnon: Option<IncarnonView>,
    /// Kitgun or zaw parts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub modular: Option<ModularView>,
    /// Progenitor bonus (T-24, 09 §14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub progenitor: Option<ProgenitorView>,
    /// Compatible stances (melee), combos resolved per 07 E-7.0b.
    pub stances: Vec<StanceView>,
    /// Melee data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub melee: Option<MeleeView>,
    /// Zoom levels (04 D-406).
    pub zoom: Vec<ZoomView>,
    /// Slots of the weapon (09 §2).
    pub slot_layout: SlotLayoutView,
    /// Innate polarities of the general slots.
    pub polarities: Vec<Polarity>,
    /// Innate exilus polarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub exilus_polarity: Option<Polarity>,
    /// Maximum rank (30 or 40).
    pub max_rank: u8,
    /// Mastery rank requirement.
    pub mastery: u8,
    /// Display-only values (accuracy, noise, flight speed, range …).
    pub display: BTreeMap<Box<str>, DisplayValue>,
    /// `GET /api/img/{key}` has an image.
    pub image: bool,
    /// Data provenance.
    pub provenance: Vec<ProvenanceView>,
}

/// Magazine of a weapon (06 §5.6-§5.7, §11.4).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MagazineView {
    /// Magazine size (rounds).
    pub size: f64,
    /// Reserve ammo; `"Infinity"` on robotic weapons (06 §11.4).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "crate::serde_util::ext_f64::option"
    )]
    #[cfg_attr(
        feature = "ts",
        ts(optional, type = "number | \"Infinity\" | \"-Infinity\"")
    )]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::serde_util::ExtF64>")
    )]
    pub ammo_max: Option<f64>,
    /// No ammo consumption.
    pub infinite: bool,
    /// Battery recharge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub battery: Option<BatteryView>,
}

/// Battery recharge (06 §5.7).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BatteryView {
    /// Delay after the last shot (s).
    pub delay: f64,
    /// Delay when empty (s).
    pub delay_empty: f64,
    /// Rounds per second.
    pub rate: f64,
}

/// Reload of a weapon (06 §5.8).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReloadView {
    /// Reload kind.
    pub kind: ReloadKindName,
    /// Reload time (s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub time: Option<f64>,
}

/// Reload kinds.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ReloadKindName {
    Standard,
    PerShell,
    Regenerate,
    None,
}

/// One attack of a weapon (base data).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackDetail {
    /// The attack.
    pub key: AttackKey,
    /// English name.
    pub name: Box<str>,
    /// Attack roles.
    pub roles: AttackRoles,
    /// Shot type.
    pub shot: ShotType,
    /// Index of the parent attack (children).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub parent: Option<u8>,
    /// Base damage per type.
    pub base: PerDamageType,
    /// Critical chance.
    pub cc: f64,
    /// Critical damage multiplier.
    pub cd: f64,
    /// Status chance.
    pub sc: f64,
    /// Multishot.
    pub multishot: f64,
    /// Firing model.
    pub fire: FireView,
    /// Damage falloff.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub falloff: Option<FalloffView>,
    /// Blast radius (m).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub radius: Option<f64>,
    /// Forced procs.
    pub forced: Vec<StatusType>,
    /// Heavy attack (melee).
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_heavy: bool,
    /// Incarnon charges per shot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub incarnon_charges: Option<u16>,
    /// Condition Overload behaviour (04 §2).
    pub co: CoView,
}

/// Firing model of an attack.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FireView {
    /// Trigger.
    pub trigger: Trigger,
    /// Fire rate (attacks/s); `"Infinity"` for `math.huge` (Lanka, 06 §4.1).
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub rate: f64,
    /// Charge time (s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub charge_time: Option<f64>,
    /// Burst.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub burst: Option<BurstView>,
}

/// Burst parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BurstView {
    /// Shots per burst.
    pub count: u16,
    /// Delay between burst shots (s).
    pub delay: f64,
}

/// Condition Overload behaviour of an attack (04 §2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CoView {
    /// `adding`, `multiplying` or `none`.
    pub behaviour: CoBehaviour,
    /// ρ_co.
    pub ratio: f64,
    /// Row of the CO catalogue (04 §2.14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub row: Option<u16>,
}

/// An attack group (08 E-8.23).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackGroupView {
    /// Attack indices.
    pub members: Vec<u8>,
    /// T-22 can combine the group.
    pub combinable: bool,
    /// Outputs are summed (parent + children).
    pub summed: bool,
}

/// Incarnon evolutions (09 E-9.60).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IncarnonView {
    /// Evolutions 2 … T(W).
    pub tiers: Vec<IncarnonTierView>,
}

/// One Incarnon evolution.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IncarnonTierView {
    /// Evolution number (2..=5).
    pub tier: u8,
    /// Selectable perks.
    pub perks: Vec<PerkView>,
}

/// One Incarnon perk.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PerkView {
    /// Position in the evolution's list (`SlotContent::Perk.index`).
    pub index: u8,
    /// StableKey of the perk item.
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// Effects rendered as messages.
    pub effects_text: Vec<Message>,
}

/// Kitgun or zaw parts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ModularView {
    /// Kitgun or zaw.
    pub kind: ModularKind,
    /// Grips (kitgun) or grips (zaw).
    pub grips: Vec<PartSummary>,
    /// Kitgun loaders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub loaders: Option<Vec<PartSummary>>,
    /// Zaw links.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub links: Option<Vec<PartSummary>>,
}

/// A kitgun or zaw part (server-api.md §5.3 names the type without fields).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartSummary {
    /// StableKey (`VariantDoc.kitgun.grip` …).
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// `GET /api/img/{key}` has an image.
    pub image: bool,
    /// Stat changes of the part, for display (`"fire_rate"`, `"base_dmg"` …).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub display: BTreeMap<Box<str>, DisplayValue>,
}

/// Progenitor bonus range (09 E-9.70).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProgenitorView {
    /// Allowed elements.
    pub elements: Vec<DamageType>,
    /// Lowest value (fraction).
    pub min: f64,
    /// Highest value (fraction).
    pub max: f64,
    /// Default value (fraction, 0.60).
    pub default: f64,
}

/// A compatible stance.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StanceView {
    /// StableKey of the stance mod.
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// Combos resolved per 07 E-7.0b.
    pub combos: Vec<ComboView>,
}

/// A combo of a stance.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComboView {
    /// Combo key (`BuildDoc.stance_combo`).
    pub key: ComboKey,
    /// Combo type.
    pub kind: ComboType,
    /// English name.
    pub name: Box<str>,
    /// Which table the combo comes from (07 E-7.0b).
    pub source: ComboSourceKind,
}

/// Source table of a combo (07 E-7.0b).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ComboSourceKind {
    Unique,
    Stance,
    Stanceless,
}

/// Melee data of a weapon.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeleeView {
    /// Melee class (07 table 9-A).
    pub class: MeleeClass,
    /// Plans of 07 §8.1 valid for this weapon.
    pub plans: Vec<super::schema::MeleePlanKind>,
    /// Attack index of the heavy attack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub heavy: Option<u8>,
    /// Exalted weapon with an ability combo (T-11).
    pub ability_combo: bool,
}

/// A zoom level (04 D-406).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ZoomView {
    /// English name.
    pub name: Box<str>,
    /// Effects rendered as messages.
    pub effects_text: Vec<Message>,
}

/// Slots of an item (09 §2.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SlotLayoutView {
    /// Slot ids.
    pub slots: Vec<SlotId>,
    /// Number of arcane slots.
    pub arcane_slots: u8,
}

/// Provenance of weapon data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProvenanceView {
    /// `"wiki"`, `"pe"`, `"wfcd"`, `"curated"`.
    pub source: Box<str>,
    /// Revision.
    pub revision: Box<str>,
}

// ---------------------------------------------------------------------------------------------------------------
// GET /api/items/{key}
// ---------------------------------------------------------------------------------------------------------------

/// `GET /api/items/{key}` (server-api.md §5.4).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ItemDetail {
    /// StableKey.
    pub key: StableKey,
    /// DE path or `wfd:` name.
    pub unique_name: Box<str>,
    /// English name.
    pub name: Box<str>,
    /// Kind slug.
    pub kind: ItemKindName,
    /// Mod polarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub polarity: Option<Polarity>,
    /// Drain at rank 0.
    pub drain: i16,
    /// Maximum rank.
    pub max_rank: u8,
    /// Rarity.
    pub rarity: Rarity,
    /// The item's set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub set: Option<SetView>,
    /// User parameters.
    pub params: Vec<ParamView>,
    /// Structured effects, rendered by the UI through i18n.
    pub effects: Vec<EffectView>,
    /// Effect status.
    pub status: EffectStatus,
    /// Sources of the encoding.
    pub sources: Vec<Box<str>>,
    /// Upstream description text (English).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub upstream_text: Option<Box<str>>,
    /// Incompatible items (09 E-9.9).
    pub incompatible: Vec<StableKey>,
    /// Compatibility summary.
    pub compat: CompatView,
    /// `GET /api/img/{key}` has an image.
    pub image: bool,
}

/// A set with its bonus text.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SetView {
    /// Set id.
    pub id: u16,
    /// English name.
    pub name: Box<str>,
    /// Member items.
    pub members: Vec<StableKey>,
    /// Bonus per piece count, as messages.
    pub bonus_text: Vec<Message>,
}

/// A user parameter of an item (also the buff parameters of the schema).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ParamView {
    /// Parameter key.
    pub key: Box<str>,
    /// Kind.
    pub kind: ParamKindName,
    /// Lowest value (`percent`, `number`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub min: Option<f64>,
    /// Highest value (`percent`, `number`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub max: Option<f64>,
    /// Choices (`choice`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub choices: Option<Vec<ChoiceView>>,
    /// Default value (model units).
    pub default: ParamValue,
    /// Label i18n key (derived from the parameter key when the data has none).
    pub label: Box<str>,
}

/// A choice of a parameter or control.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChoiceView {
    /// Stored value.
    pub value: Box<str>,
    /// Label i18n key.
    pub label: Box<str>,
}

/// A structured effect (domain-model.md §11), for display.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EffectView {
    /// Effect id (the part after `#` of the EffectKey).
    pub id: Box<str>,
    /// Target.
    pub target: EffectTargetView,
    /// Value at maximum rank; `null` when it depends on a parameter.
    pub value_max: Option<f64>,
    /// Scalars the value is multiplied by (`"combo_mult"`, `"stacks"` …).
    pub per: Vec<Box<str>>,
    /// Condition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub condition: Option<EffectConditionView>,
    /// Scope restrictions as messages.
    pub scope_text: Vec<Message>,
    /// Curator note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub note: Option<Box<str>>,
}

/// Target of an effect: exactly one field is set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EffectTargetView {
    /// Stat key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub stat: Option<Box<str>>,
    /// Element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub element: Option<Box<str>>,
    /// Faction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub faction: Option<Box<str>>,
    /// Multiplier group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub mult: Option<Box<str>>,
    /// Mechanic kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub mechanic: Option<Box<str>>,
}

/// Condition of an effect.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EffectConditionView {
    /// Derived class (domain-model.md §12.1).
    pub class: EffectClass,
    /// Trigger id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub trigger: Option<Box<str>>,
    /// Trigger chance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub chance: Option<f64>,
    /// Stacking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub stacks: Option<StacksView>,
}

/// Display class of an effect condition.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum EffectClass {
    A,
    B,
    C,
    Schedule,
    Formula,
}

/// Stacking of an effect.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StacksView {
    /// Maximum stacks.
    pub max: u16,
    /// Duration (s).
    pub duration: f64,
}

/// Compatibility summary of an item.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CompatView {
    /// Slot kinds the item fits.
    pub slots: Vec<Box<str>>,
    /// Required trigger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub trigger: Option<Trigger>,
    /// Compatibility categories (09 §3.1).
    pub categories: Vec<Box<str>>,
}

// ---------------------------------------------------------------------------------------------------------------
// GET /api/enemies/{key}
// ---------------------------------------------------------------------------------------------------------------

/// `GET /api/enemies/{key}` (server-api.md §5.5).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyDetail {
    /// StableKey.
    pub key: StableKey,
    /// English name.
    pub name: Box<str>,
    /// Faction slug.
    pub faction: Box<str>,
    /// Base level.
    pub base_level: u16,
    /// Base values at the base level.
    pub base: EnemyBase,
    /// Eximus variant.
    pub eximus: EximusView,
    /// Body parts (05 §4.3).
    pub parts: Vec<BodyPartView>,
    /// Status immunities.
    pub immune: Vec<StatusType>,
    /// Status stack caps that differ from the default.
    pub caps: BTreeMap<StatusType, u16>,
    /// Innate damage reduction per layer.
    pub innate_dr: LayerValues,
    /// Status damage reduction.
    pub status_dr: f64,
    /// Unique damage reduction (attenuation, flat DR, adaptation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub special_dr: Option<SpecialDrView>,
    /// Enemy flags.
    pub flags: EnemyFlags,
    /// Health scales with the squad size.
    pub party_scaled: bool,
    /// Faction damage modifiers per type (05 §5.1).
    pub faction_mods: PerDamageType,
}

/// Base values of an enemy.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EnemyBase {
    /// Health.
    pub health: f64,
    /// Shield.
    pub shield: f64,
    /// Armor.
    pub armor: f64,
    /// Overguard.
    pub overguard: f64,
}

/// Eximus variant of an enemy.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EximusView {
    /// Eximus health.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub health: Option<f64>,
    /// Eximus shield.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub shield: Option<f64>,
    /// The enemy has an Eximus variant.
    pub available: bool,
}

/// A body part (05 §4.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BodyPartView {
    /// Part name.
    pub name: Box<str>,
    /// Damage multiplier.
    pub mult: f64,
    /// Weak point.
    pub weak_point: bool,
    /// Head.
    pub head: bool,
    /// Headshot crit bonus applies.
    pub head_crit: bool,
}

/// Unique damage reduction of an enemy.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SpecialDrView {
    /// The special-DR rule id (`rules.special_dr`).
    pub kind: Box<str>,
    /// Explanation (unique-DR warning).
    pub note: Message,
}

// ---------------------------------------------------------------------------------------------------------------
// POST /api/candidates
// ---------------------------------------------------------------------------------------------------------------

/// `POST /api/candidates` body (server-api.md §5.7).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CandidatesRequest {
    /// Current build and scenario.
    pub input: CalcInput,
    /// The slot to fill.
    pub slot: SlotId,
    /// Localized substring or fuzzy query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub query: Option<Box<str>>,
    /// Stat key filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub stat: Option<Box<str>>,
}

/// A candidate for a slot: legal, or blocked with the reason. JSON `{ "key": 3301, "legal": true }` or
/// `{ "key": 3390, "legal": false, "reason": { … } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Candidate {
    /// StableKey of the item.
    pub key: StableKey,
    /// The item can go into the slot.
    pub legal: bool,
    /// Why not (present exactly when `legal` is false).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub reason: Option<Message>,
}
