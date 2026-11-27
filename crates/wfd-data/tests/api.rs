//! The public API of `wfd-data` as work-breakdown.md T02b fixes it, and the parts the skeleton already implements.
//!
//! `signatures_of_work_breakdown_t02b` pins every signature of the T02b "Provides" block, and
//! `signatures_of_the_contract_review` the additions of the contract review (CONTRACTS.md "Contract review fixes
//! (crates)"): a change fails to compile here, and needs a CONTRACTS.md entry (work-breakdown.md §2.1). Behaviour
//! tests of the T04 bodies live in `tests/load.rs` (ignored until T04).

use smallvec::SmallVec;
use std::path::{Path, PathBuf};
use wfd_data::legality::{
    self, CapacityModel, FormaReport, PickPolarity, SlotCost, TraitRange, TraitRole,
};
use wfd_data::{
    BundleCounts, BundleHeader, DataError, Entity, EntityKind, GameData, SearchHit, SearchQuery,
    decode_bundle, encode_bundle, read_header,
};
use wfd_model::api::{Candidate, Catalog, EnemyDetail, ItemDetail, ScenarioSchema, WeaponDetail};
use wfd_model::message::Message;
use wfd_model::*;

#[test]
#[expect(
    clippy::type_complexity,
    reason = "the pinned signatures are spelled out as work-breakdown.md T02b writes them"
)]
fn signatures_of_work_breakdown_t02b() {
    let _: fn() -> Result<GameData, DataError> = GameData::embedded;
    let _: fn(&[u8]) -> Result<GameData, DataError> = GameData::from_bytes;
    let _: fn(&Path) -> Result<GameData, DataError> = GameData::from_json_dir;
    let _: fn(&GameData) -> &Bundle = GameData::bundle;
    let _: fn(&GameData, StableKey) -> Option<Entity<'_>> = GameData::by_key;
    let _: for<'a, 'n> fn(&'a GameData, &'n str) -> Option<Entity<'a>> = GameData::by_unique_name;
    let _: fn(&GameData, &SearchQuery) -> Vec<SearchHit> = GameData::search;
    let _: fn(&GameData) -> Catalog = GameData::catalog;
    let _: fn(&GameData, StableKey) -> Option<WeaponDetail> = GameData::weapon_detail;
    let _: fn(&GameData, StableKey) -> Option<ItemDetail> = GameData::item_detail;
    let _: fn(&GameData, StableKey) -> Option<EnemyDetail> = GameData::enemy_detail;
    let _: fn(&GameData) -> ScenarioSchema = GameData::schema;
    let _: fn(&GameData, &CalcInput, SlotId, Option<&str>, Option<&str>) -> Vec<Candidate> =
        GameData::candidates;
    // The plan's `Result<Resolved, Vec<Message>>` plus the resolve warnings (CONTRACTS.md T02b, request T02b-3).
    let _: fn(&GameData, &CalcInput) -> Result<(Resolved, Vec<Message>), Vec<Message>> =
        GameData::resolve;
    let _: fn(&GameData, &Build, &Scenario) -> CalcInput = GameData::to_doc;
    let _: fn(&GameData, StableKey) -> Option<&str> = GameData::image_path;

    let _: fn(&GameData, &Build, &Scenario) -> Vec<Violation> = legality::check;
    let _: fn(&GameData, &Build, SlotId, &Pick) -> Vec<Violation> = legality::check_slot;
    let _: fn(&GameData, &Build, CapacityMode) -> FormaReport = legality::min_forma;
    let _: fn(&GameData, &Build, CapacityMode, FormaTypes) -> FormaReport =
        legality::min_forma_with;
    let _: fn(&GameData, &Build) -> SmallVec<[(SetId, u8); 4]> = legality::set_counts;
    let _: fn(&GameData, WeaponId, &Riven) -> Vec<TraitRange> = legality::riven_range;

    let _: fn(&Bundle) -> Vec<u8> = encode_bundle;
    let _: fn(&[u8]) -> Result<Bundle, DataError> = decode_bundle;
}

/// The additions of the contract review: retired keys (API-5), the bundle header (API-2, DC-28), riven generation
/// (API-12), capacity inputs and the minimum-Forma model (API-13), valence fusion and N_cav (API-16).
#[test]
fn signatures_of_the_contract_review() {
    let _: fn(&GameData, StableKey) -> bool = GameData::is_retired;
    let _: fn(&[u8]) -> Result<BundleHeader, DataError> = read_header;
    let _: fn(&GameData, WeaponId) -> Option<RivenClass> = legality::riven_class;
    let _: fn(&GameData, WeaponId) -> Vec<&RivenTraitDef> = legality::riven_traits_for;
    let _: fn(&GameData, &Build, SlotId, &Pick) -> Option<SlotCost> = legality::slot_cost;
    let _: fn(SlotCost, Option<Polarity>) -> i16 = legality::slot_charge;
    let _: fn(&SlotCost) -> i16 = SlotCost::min_charge;
    fn capacity_model<'a>(_: std::marker::PhantomData<&'a ()>) {
        let _: fn(&'a GameData, &'a Build, CapacityMode, FormaTypes) -> CapacityModel<'a> =
            CapacityModel::<'a>::new;
        let _: fn(&CapacityModel<'a>, &[(SlotId, SlotCost)]) -> FormaReport =
            CapacityModel::<'a>::report;
    }
    capacity_model(std::marker::PhantomData);
    let _: fn(u16, u16) -> u16 = legality::fuse_valence;
    let _: fn(&GameData, &Build) -> u8 = legality::shard_cav;

    // `TraitRole` is the API type, so `CalcReport.riven` copies a `TraitRange` unchanged (API-14).
    let _: wfd_model::api::TraitRole = TraitRole::Negative;
    let cost = SlotCost {
        drain: 18,
        polarity: PickPolarity::Unknown,
        default_drain: true,
    };
    assert_eq!(cost, cost.clone());
    assert_ne!(
        PickPolarity::Polarized(Polarity::Madurai),
        PickPolarity::Unpolarized
    );
}

/// The bundle header is plain data with the postcard layout of pipeline.md §8.2 (field order frozen).
#[test]
fn bundle_header_is_plain_data() {
    let h = BundleHeader {
        schema_version: BUNDLE_SCHEMA_VERSION,
        data_version: "2026.10.28.09.54+0a1b2c3d".into(),
        content_hash: [7; 32],
        created_at: "2026-11-27T00:00:00Z".into(),
        game_baseline: "Hotfix 44.1.4".into(),
        licences: Vec::new(),
        counts: BundleCounts {
            weapons: 911,
            items: 2_000,
            enemies: 1_019,
            frames: 126,
            stable_keys: 4_100,
        },
    };
    assert_eq!(h.clone(), h);
    let json = serde_json::to_value(&h).unwrap();
    let back: BundleHeader = serde_json::from_value(json).unwrap();
    assert_eq!(back, h);
    assert_eq!(BundleCounts::default().weapons, 0);
}

fn mini() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/mini")
}

fn first<T: serde::de::DeserializeOwned>(file: &str) -> T {
    let path = mini().join(file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut v: Vec<T> =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    v.remove(0)
}

/// `Entity` reads key, uniqueName, name and image from the record it wraps; enemies have no image (server-api.md
/// §10 serves weapons, items, frames and parts; enemy.rs: no DE export).
#[test]
fn entity_accessors_read_the_record() {
    let weapon: Weapon = first("weapons/primary.json");
    let item: Item = first("items/weapon_mod.json");
    let enemy: Enemy = first("enemies/grineer.json");
    let frame: Frame = first("frames.json");
    let modular: ModularTables = {
        let text = std::fs::read_to_string(mini().join("modular.json")).unwrap();
        serde_json::from_str(&text).unwrap()
    };
    let grip = &modular.grips[0];

    let cases = [
        (
            Entity::Weapon(&weapon),
            EntityKind::Weapon,
            weapon.key,
            &weapon.unique_name,
            &weapon.name,
            weapon.image.as_deref(),
        ),
        (
            Entity::Item(&item),
            EntityKind::Item,
            item.key,
            &item.unique_name,
            &item.name,
            item.image.as_deref(),
        ),
        (
            Entity::Enemy(&enemy),
            EntityKind::Enemy,
            enemy.key,
            &enemy.unique_name,
            &enemy.name,
            None,
        ),
        (
            Entity::Frame(&frame),
            EntityKind::Frame,
            frame.key,
            &frame.unique_name,
            &frame.name,
            frame.image.as_deref(),
        ),
        (
            Entity::Part(0, grip),
            EntityKind::Part,
            grip.key,
            &grip.unique_name,
            &grip.name,
            grip.image.as_deref(),
        ),
    ];
    for (e, kind, key, unique_name, name, image) in cases {
        assert_eq!(e.kind(), kind, "{e:?}");
        assert_eq!(e.key(), key, "{e:?}");
        assert_eq!(e.unique_name(), unique_name, "{e:?}");
        assert_eq!(e.name(), name, "{e:?}");
        assert_eq!(e.image(), image, "{e:?}");
    }
    assert!(
        weapon.image.is_some(),
        "the fixture weapon has an image, so the test reads a value"
    );
}

/// The default query has no filter; the skeleton's helper types keep their derives.
#[test]
fn search_types() {
    let q = SearchQuery::default();
    assert!(q.text.is_empty() && q.kinds.is_empty() && q.item_kinds.is_empty());
    assert_eq!((q.faction, q.stat, q.limit), (None, None, None));
    let hit = SearchHit {
        key: StableKey(1204),
        kind: EntityKind::Weapon,
        score: 3,
    };
    assert_eq!(hit.clone(), hit);
    let r = TraitRange {
        trait_id: "critical_chance".into(),
        role: Some(TraitRole::Positive),
        range: Some(wfd_model::api::ValueRange { min: 0.5, max: 1.2 }),
    };
    assert_eq!(r.clone(), r);
}

/// `DataError` messages name what failed (logged at startup, pipeline.md §9.2).
#[test]
fn data_error_messages() {
    let e = DataError::SchemaVersion {
        found: 2,
        expected: BUNDLE_SCHEMA_VERSION,
    };
    assert_eq!(
        e.to_string(),
        format!("bundle schema version 2, this binary reads {BUNDLE_SCHEMA_VERSION}")
    );
    let e = DataError::Invalid(vec![
        "duplicate key 1204".into(),
        "weapon 3 has id 4".into(),
    ]);
    assert_eq!(
        e.to_string(),
        "game data failed validation: duplicate key 1204; weapon 3 has id 4"
    );
    let e = DataError::Io {
        path: PathBuf::from("x/manifest.json"),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
    };
    assert_eq!(e.to_string(), "x/manifest.json: gone");
    assert!(std::error::Error::source(&e).is_some());
}
