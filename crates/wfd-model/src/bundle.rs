//! The bundle (domain-model.md §8): everything the engine and the server read, as one value. `data/generated/*.json`
//! is its readable form (pipeline.md §8.1); `bundle.bin` is postcard + zstd (pipeline.md §8.2).

use crate::enemy::Enemy;
use crate::frame::Frame;
use crate::ids::{NameKey, StableKey, UniqueName};
use crate::item::{EffectStatus, Item, SetDef};
use crate::rules::Rules;
use crate::weapon::{ComboTable, ModularTables, Stance, Weapon};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Layout version of [`Bundle`]: bumped on any change of the serialized layout of the bundle types (postcard is not
/// self-describing). The server loads only user bundles with this version (server-api.md §1.2).
pub const BUNDLE_SCHEMA_VERSION: u32 = 1;

/// The game data of one data version.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Bundle {
    pub manifest: Manifest,
    /// Index = `WeaponId`.
    pub weapons: Vec<Weapon>,
    /// Index = `ItemId`.
    pub items: Vec<Item>,
    /// Index = `SetId`.
    pub sets: Vec<SetDef>,
    /// Index = `StanceId`.
    pub stances: Vec<Stance>,
    pub modular: ModularTables,
    /// Index = `ComboTableId`: stance, stanceless class and unique-to-weapon tables (07 §4).
    pub combo_tables: Vec<ComboTable>,
    /// Index = `FrameId` (AD-26).
    pub frames: Vec<Frame>,
    /// Index = `EnemyId`.
    pub enemies: Vec<Enemy>,
    pub rules: Rules,
    /// Locale → (NameKey → text); `"en"` only for now.
    pub names: NameTable,
    /// The key of every record of the bundle (weapons, items, sets, enemies, frames …): global keys, and in a
    /// refreshed user bundle session-local keys too (domain-model.md §2). Every entry names a record; retired keys
    /// are in `retired_keys`.
    pub stable_keys: Vec<(StableKey, UniqueName)>,
    /// Retired keys: global keys whose uniqueName left the data (pipeline.md §5.3 "removed ones keep their key and are
    /// listed as `retired`"), sorted by key; disjoint from `stable_keys`. A key here is `doc.item_retired` (the entry
    /// is dropped with a warning), a key in neither list `doc.key_unknown` (422; server-api.md §6.1). Key assignment
    /// (the developer build and the in-app refresh) reads both lists, so a key is never reused and a re-added
    /// uniqueName gets its old key back. Addition, CONTRACTS.md "Contract review fixes (model-records)", API-5 (no
    /// bundle had been built, so `BUNDLE_SCHEMA_VERSION` stays 1 under the T01d ruling).
    #[serde(default)]
    pub retired_keys: Vec<(StableKey, UniqueName)>,
}

/// Provenance and versions of a bundle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Manifest {
    /// [`BUNDLE_SCHEMA_VERSION`] of the writer.
    pub schema_version: u32,
    /// `"2026.10.28.09.54+<blake3 of inputs, 8 hex>"`.
    pub data_version: Box<str>,
    /// `"Hotfix 44.1.4"` (spec 00 §1.1).
    pub game_baseline: Box<str>,
    pub created_at: Box<str>,
    /// Wiki module title + revid + timestamp; WFCD tag + sha; PE buildLabel + hashes.
    pub sources: Vec<SourceStamp>,
    /// `git describe` of `data/curated` at build time.
    pub curated_rev: Box<str>,
    /// Hash of the ingested spec tables.
    pub spec_rev: Box<str>,
    /// pipeline.md §10.
    pub licences: Vec<LicenceNotice>,
    /// Counts per EffectStatus per item kind.
    pub coverage: Coverage,
}

/// One input of a bundle build.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceStamp {
    /// `"wiki"`, `"pe"`, `"wfcd"`, `"spec"`, `"curated"`.
    pub source: Box<str>,
    /// Module title, manifest name or file name.
    pub id: Box<str>,
    /// Revid, content hash, tag or sha.
    pub revision: Box<str>,
    pub timestamp: Option<Box<str>>,
}

/// Licence and attribution of one source (pipeline.md §10).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LicenceNotice {
    pub source: Box<str>,
    pub licence: Box<str>,
    pub url: Box<str>,
    pub attribution: Box<str>,
    /// Wiki module titles and revision ids.
    pub modules: Vec<(Box<str>, u32)>,
}

/// Counts per item kind slug and effect status.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Coverage(pub BTreeMap<Box<str>, BTreeMap<EffectStatus, u32>>);

/// Display names per locale.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NameTable(pub BTreeMap<Box<str>, BTreeMap<NameKey, Box<str>>>);

impl NameTable {
    /// The name of a key in a locale, falling back to English.
    pub fn get(&self, locale: &str, key: &NameKey) -> Option<&str> {
        self.0
            .get(locale)
            .and_then(|m| m.get(key))
            .or_else(|| self.0.get("en").and_then(|m| m.get(key)))
            .map(|s| &**s)
    }
}
