//! [`GameData`]: one loaded bundle with its indexes (pipeline.md §8.3) and the read side every other crate uses:
//! lookups by StableKey and uniqueName, search, the catalogue, detail and schema assembly of server-api.md §5, the
//! candidates of `/api/candidates` (§5.7), and document resolution in both directions (domain-model.md §9.2, AD-18).

use crate::error::DataError;
use std::fmt;
use std::path::Path;
use wfd_model::api::{
    Candidate, Catalog, EnemyDetail, ItemDetail, ItemKindName, ScenarioSchema, WeaponDetail,
};
use wfd_model::message::Message;
use wfd_model::{
    Build, Bundle, CalcInput, Enemy, Frame, Item, ModularPart, NameKey, Resolved, Scenario, SlotId,
    StableKey, UniqueName, Weapon,
};

/// One loaded bundle and its indexes (pipeline.md §8.3).
///
/// Created once per bundle: at startup from the embedded bundle ([`GameData::embedded`]) or the newest user bundle
/// ([`GameData::from_bytes`]); in tests from a JSON directory ([`GameData::from_json_dir`]). Immutable afterwards; the
/// server swaps whole snapshots (`ArcSwap<GameData>`, pipeline.md §9.2 "Activate"). T04 adds the indexes: by
/// StableKey (the session-local keys ≥ `LOCAL_KEY_BASE` of a user bundle included, AD-18) and by uniqueName.
pub struct GameData {
    bundle: Bundle,
}

impl GameData {
    /// The bundle embedded at compile time: `build.rs` reads `WFD_DATA_DIR`, else `data/generated/` when it exists,
    /// else `data/fixtures/mini/`, validates it and embeds `OUT_DIR/bundle.bin` (pipeline.md §8.2; work-breakdown.md
    /// T04). The fixture fallback emits a `cargo:warning` naming the missing `data/generated/`, so a binary built
    /// without the real data says so at build time; `xtask dist` (T39) refuses to package a fixture bundle (overview.md
    /// AD-11, ruling T02c-R5 of the contract review). Implemented by T04.
    pub fn embedded() -> Result<GameData, DataError> {
        todo!("T04: decode the bundle build.rs embeds (pipeline.md §8.2)")
    }

    /// A bundle file of the pipeline.md §8.2 format ([`crate::decode_bundle`]), with its indexes. Used for user bundles
    /// (`<data_dir>/bundles/*.bin`, pipeline.md §9.2). Implemented by T04.
    pub fn from_bytes(bytes: &[u8]) -> Result<GameData, DataError> {
        let _ = bytes;
        todo!("T04: decode_bundle + indexes")
    }

    /// A directory in the canonical JSON layout of pipeline.md §8.1 (`data/generated/`, `data/fixtures/mini/`): the
    /// per-kind files in the order that defines the dense ids, then `extra/*.json` merged in file-name order
    /// (work-breakdown.md §2.2; `data/fixtures/README.md`), checked by the reference validator. Tests and fixtures.
    /// Implemented by T04.
    pub fn from_json_dir(dir: &Path) -> Result<GameData, DataError> {
        let _ = dir;
        todo!("T04: read the pipeline.md §8.1 layout, merge extra/*.json, validate, index")
    }

    /// The loaded bundle.
    pub fn bundle(&self) -> &Bundle {
        &self.bundle
    }

    /// The weapon, item, enemy, frame or modular part with this key (global or session-local, domain-model.md §2).
    /// A kitgun chamber and a zaw strike share the key of their weapon record (`data/fixtures/README.md`) and are
    /// found as that [`Entity::Weapon`]. Implemented by T04.
    pub fn by_key(&self, k: StableKey) -> Option<Entity<'_>> {
        let _ = k;
        todo!("T04: StableKey index")
    }

    /// The entity with this uniqueName (DE path or `wfd:` name, domain-model.md §2); how the `x` map of a document
    /// resolves session-local keys (AD-18). Implemented by T04.
    pub fn by_unique_name(&self, n: &str) -> Option<Entity<'_>> {
        let _ = n;
        todo!("T04: uniqueName index")
    }

    /// Localized substring and fuzzy search over names, filtered by entity kind, item kind, faction and stat
    /// (pipeline.md §8.3; frontend.md parity rows 1, 14, 17). Hits ordered by relevance, ties by key (AD-19).
    /// Implemented by T04.
    pub fn search(&self, q: &SearchQuery) -> Vec<SearchHit> {
        let _ = q;
        todo!("T04: search (pipeline.md §8.3)")
    }

    /// `GET /api/catalog` (server-api.md §5.2): every weapon, item, enemy and frame summary (AD-26), factions, sets,
    /// names. Implemented by T04.
    pub fn catalog(&self) -> Catalog {
        todo!("T04: catalogue assembly (server-api.md §5.2)")
    }

    /// `GET /api/weapons/{key}` (server-api.md §5.3); `None` when the key names no weapon. Implemented by T04.
    pub fn weapon_detail(&self, k: StableKey) -> Option<WeaponDetail> {
        let _ = k;
        todo!("T04: weapon detail (server-api.md §5.3)")
    }

    /// `GET /api/items/{key}` (server-api.md §5.4); `None` when the key names no item. Implemented by T04.
    pub fn item_detail(&self, k: StableKey) -> Option<ItemDetail> {
        let _ = k;
        todo!("T04: item detail (server-api.md §5.4)")
    }

    /// `GET /api/enemies/{key}` (server-api.md §5.5); `None` when the key names no enemy. Carries the enemy's 05 §4.1
    /// scenario defaults (`spawn_level`, `defaults`, `overguard_context`; `None` when the record has none), which the
    /// target pane applies when the user picks the enemy (contract review M-7). Implemented by T04.
    pub fn enemy_detail(&self, k: StableKey) -> Option<EnemyDetail> {
        let _ = k;
        todo!("T04: enemy detail (server-api.md §5.5)")
    }

    /// `GET /api/schema` (server-api.md §5.6, AD-17): toggles from `Rules.toggles`, buffs, melee plans, custom stats,
    /// riven traits, slot layouts, views, condition classes (`a`, `b`, `c` and `u`, 00 §6.2). The custom stat
    /// `abilityCombo` (`p.u`) carries `CustomStatSchema.toggle` = T-11: it is an alias of `toggles.ability_combo`
    /// (contract review M-18). `RivenTraitSchema.range` stays absent: a legal range depends on the weapon, rank and
    /// layout, and travels per riven in `CalcReport.riven` ([`crate::legality::riven_range`]; contract review
    /// API-14). Implemented by T04.
    pub fn schema(&self) -> ScenarioSchema {
        todo!("T04: scenario schema (server-api.md §5.6)")
    }

    /// `POST /api/candidates` (server-api.md §5.7): every item of the slot's kind for the document's weapon or frame,
    /// legal or blocked with the reason of [`crate::legality`] (the same rules the decoder and the optimizer use),
    /// filtered by `query` (localized substring and fuzzy) or `stat` (a stat key), ordered by relevance.
    /// Implemented by T04.
    pub fn candidates(
        &self,
        input: &CalcInput,
        slot: SlotId,
        query: Option<&str>,
        stat: Option<&str>,
    ) -> Vec<Candidate> {
        let _ = (input, slot, query, stat);
        todo!("T04: candidates (server-api.md §5.7, spec 09)")
    }

    /// Resolves a document into dense ids (domain-model.md §9.2). The `x` map is applied first: a session-local key
    /// is looked up by its uniqueName (AD-18; an unknown uniqueName is `doc.item_unknown`, a local key without an
    /// `x` entry `doc.local_key_unmapped`). A global key in `Bundle.retired_keys` ([`GameData::is_retired`]) is
    /// `doc.item_retired`, dropped with a warning; a key in neither key list is `doc.key_unknown` (contract review
    /// API-5). The custom stat `abilityCombo` (`p.u`, a T-11 alias) moves into `toggles.ability_combo` when T-11 is
    /// off and is dropped when T-11 is set (`ScenarioDoc.custom` doc; contract review M-18). Every spec 09 rule runs
    /// ([`crate::legality::check`]) and its violations are returned inside [`Resolved`]: validation never blocks
    /// evaluation (09 §1.5). `Err` only for malformed documents (unknown key, content kind impossible in its slot,
    /// duplicate slot or attack entries), as `doc.*` messages (`wfd_model::message::DocIssue`, ruling C-18).
    ///
    /// `Ok` also carries the document issues resolution dropped with a warning instead of rejecting the document
    /// (`doc.*` messages: a retired item that can be dropped, `doc.effect_unknown`, `doc.attack_unknown`;
    /// domain-model.md §9.2 "travel as warnings", `message/doc.rs`), in document order, for `CalcReport.warnings`
    /// (server-api.md §6.1). The plan's `Result<Resolved, Vec<Message>>` has no channel for them; they travel beside
    /// [`Resolved`], so the model type stays as domain-model.md §9.2 fixes it (CONTRACTS.md T02b, request T02b-3).
    /// The server appends them to `CalcReport.warnings` in document order, after the engine's (T11, T34).
    /// Implemented by T04.
    pub fn resolve(&self, input: &CalcInput) -> Result<(Resolved, Vec<Message>), Vec<Message>> {
        let _ = input;
        todo!("T04: document resolution (domain-model.md §9.2, AD-18)")
    }

    /// The document form of a resolved build and scenario (domain-model.md §9.1): StableKeys, defaults omitted where
    /// the document omits them, and an `x` entry with the uniqueName of every session-local key used (AD-18). The
    /// inverse of [`GameData::resolve`] up to canonical form. Every encoder input goes through it:
    /// `wfd_buildcode::encode` needs the `x` entries and cannot look them up (server-api.md §7; contract review
    /// API-21). Implemented by T04.
    pub fn to_doc(&self, b: &Build, s: &Scenario) -> CalcInput {
        let _ = (b, s);
        todo!("T04: resolved form to document form (AD-18)")
    }

    /// Whether `k` is a retired global key: its uniqueName left the data (`Bundle.retired_keys`, pipeline.md §5.3). A
    /// document entry with such a key is dropped with `doc.item_retired` ([`GameData::resolve`]); a key that is neither
    /// live ([`GameData::by_key`]) nor retired is `doc.key_unknown` (server-api.md §6.1). Addition of the contract
    /// review (API-5).
    pub fn is_retired(&self, k: StableKey) -> bool {
        self.bundle.retired_keys.iter().any(|(key, _)| *key == k)
    }

    /// The Public Export `textureLocation` of the weapon, item, frame or modular part with this key, which
    /// `GET /api/img/{key}` serves (server-api.md §10); `None` for enemies, unknown keys and records without one.
    pub fn image_path(&self, k: StableKey) -> Option<&str> {
        self.by_key(k)?.image()
    }
}

impl fmt::Debug for GameData {
    /// The data version and the record counts; the bundle itself is too large to print.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = &self.bundle;
        f.debug_struct("GameData")
            .field("data_version", &b.manifest.data_version)
            .field("weapons", &b.weapons.len())
            .field("items", &b.items.len())
            .field("enemies", &b.enemies.len())
            .field("frames", &b.frames.len())
            .field("stable_keys", &b.stable_keys.len())
            .finish_non_exhaustive()
    }
}

/// A record found by [`GameData::by_key`] or [`GameData::by_unique_name`]: anything that has a StableKey.
/// Weapons, items, enemies and frames carry their dense id; a modular part carries its index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entity<'a> {
    /// A weapon record (`Bundle.weapons`; kitgun chambers and zaw strikes included, see [`GameData::by_key`]).
    Weapon(&'a Weapon),
    /// A mod, arcane, shard, stance, Incarnon perk, buff or custom stat (`Bundle.items`).
    Item(&'a Item),
    /// An enemy (`Bundle.enemies`).
    Enemy(&'a Enemy),
    /// A warframe (`Bundle.frames`, AD-26).
    Frame(&'a Frame),
    /// A kitgun grip or loader, or a zaw grip or link (`VariantDoc.kitgun`, `.zaw`; T-25). The number is the index
    /// into the `ModularTables` list of the part's kind (`grips` for both grip kinds, `loaders`, `links`), as
    /// `Variant.kitgun` and `Variant.zaw` hold it.
    Part(u16, &'a ModularPart),
}

impl<'a> Entity<'a> {
    /// The kind of the record.
    pub fn kind(&self) -> EntityKind {
        match self {
            Entity::Weapon(_) => EntityKind::Weapon,
            Entity::Item(_) => EntityKind::Item,
            Entity::Enemy(_) => EntityKind::Enemy,
            Entity::Frame(_) => EntityKind::Frame,
            Entity::Part(..) => EntityKind::Part,
        }
    }

    /// The record's StableKey.
    pub fn key(&self) -> StableKey {
        match self {
            Entity::Weapon(w) => w.key,
            Entity::Item(i) => i.key,
            Entity::Enemy(e) => e.key,
            Entity::Frame(f) => f.key,
            Entity::Part(_, p) => p.key,
        }
    }

    /// The record's uniqueName (domain-model.md §2).
    pub fn unique_name(&self) -> &'a UniqueName {
        match self {
            Entity::Weapon(w) => &w.unique_name,
            Entity::Item(i) => &i.unique_name,
            Entity::Enemy(e) => &e.unique_name,
            Entity::Frame(f) => &f.unique_name,
            Entity::Part(_, p) => &p.unique_name,
        }
    }

    /// The record's name key into `Bundle.names`.
    pub fn name(&self) -> &'a NameKey {
        match self {
            Entity::Weapon(w) => &w.name,
            Entity::Item(i) => &i.name,
            Entity::Enemy(e) => &e.name,
            Entity::Frame(f) => &f.name,
            Entity::Part(_, p) => &p.name,
        }
    }

    /// The Public Export `textureLocation` (server-api.md §10); enemies have none.
    pub fn image(&self) -> Option<&'a str> {
        match self {
            Entity::Weapon(w) => w.image.as_deref(),
            Entity::Item(i) => i.image.as_deref(),
            Entity::Enemy(_) => None,
            Entity::Frame(f) => f.image.as_deref(),
            Entity::Part(_, p) => p.image.as_deref(),
        }
    }
}

/// Kind of an [`Entity`]; a [`SearchQuery`] filter and a [`SearchHit`] field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityKind {
    /// [`Entity::Weapon`].
    Weapon,
    /// [`Entity::Item`].
    Item,
    /// [`Entity::Enemy`].
    Enemy,
    /// [`Entity::Frame`].
    Frame,
    /// [`Entity::Part`].
    Part,
}

/// A search over the loaded data ([`GameData::search`]). The default query matches everything.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchQuery {
    /// Text matched against localized names: substring first, then fuzzy (pipeline.md §8.3; frontend.md parity
    /// rows 1, 14, 17). Empty: every record that passes the filters.
    pub text: Box<str>,
    /// Locale of the names searched (`Bundle.names`); empty or unknown = `"en"`.
    pub locale: Box<str>,
    /// Kinds to search; empty = every kind.
    pub kinds: Vec<EntityKind>,
    /// Item kinds to keep (catalogue names, server-api.md §5.2); empty = every kind. Applies to items only.
    pub item_kinds: Vec<ItemKindName>,
    /// Enemy faction slug (`Rules.factions`); `None` = any. Applies to enemies only (parity row 17).
    pub faction: Option<Box<str>>,
    /// A stat key (`StatKey::parse` form, e.g. `"crit_chance"`) that one of the item's effects targets; `None` =
    /// any. Applies to items only (parity row 14).
    pub stat: Option<Box<str>>,
    /// At most this many hits; `None` = all.
    pub limit: Option<usize>,
}

/// One result of [`GameData::search`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    /// The record found.
    pub key: StableKey,
    /// Its kind.
    pub kind: EntityKind,
    /// Relevance, higher first (exact name, prefix, substring, fuzzy; the scale is T04's). Equal scores are ordered
    /// by key, so results are deterministic (AD-19).
    pub score: u32,
}
