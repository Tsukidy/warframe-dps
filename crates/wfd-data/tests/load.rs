//! Loading game data (pipeline.md §8.2, §8.3; testing.md §4 "Bundle"): the JSON → bundle → `GameData` round trip on
//! the mini fixture, the index lookups, and the checks of the file format. Every test needs a T04 body and is ignored
//! until T04 implements it (work-breakdown.md T02b, T04); T04 removes the `#[ignore]`s and adds the legality cases,
//! the `x`-map decode and the TV-09 vectors of its acceptance.

use std::path::{Path, PathBuf};
use wfd_data::{DataError, Entity, GameData, decode_bundle, encode_bundle, read_header};
use wfd_model::*;

fn mini() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/mini")
}

fn mini_data() -> GameData {
    GameData::from_json_dir(&mini()).unwrap_or_else(|e| panic!("{}: {e}", mini().display()))
}

/// JSON → bundle bytes → bundle and `GameData` give back the same bundle (testing.md §4 "Bundle").
#[test]
#[ignore = "T04: from_json_dir, encode_bundle, decode_bundle, read_header, from_bytes"]
fn mini_fixture_round_trips_through_the_bundle_format() {
    let gd = mini_data();
    let b = gd.bundle();
    assert!(
        !b.weapons.is_empty()
            && !b.items.is_empty()
            && !b.enemies.is_empty()
            && !b.frames.is_empty()
    );
    let bytes = encode_bundle(b);
    assert!(bytes.starts_with(b"WFDPSB01"), "pipeline.md §8.2 magic");
    assert_eq!(&decode_bundle(&bytes).unwrap(), b);
    assert_eq!(GameData::from_bytes(&bytes).unwrap().bundle(), b);
    assert_eq!(encode_bundle(b), bytes, "encoding is deterministic");
    // The header alone lists the bundle (API-2): the manifest's versions and the record counts.
    let h = read_header(&bytes).unwrap();
    assert_eq!(h.schema_version, BUNDLE_SCHEMA_VERSION);
    assert_eq!(h.data_version, b.manifest.data_version);
    assert_eq!(h.created_at, b.manifest.created_at);
    assert_eq!(h.game_baseline, b.manifest.game_baseline);
    assert_eq!(h.licences, b.manifest.licences);
    assert_eq!(h.counts.weapons as usize, b.weapons.len());
    assert_eq!(h.counts.stable_keys as usize, b.stable_keys.len());
}

/// Every stable key and uniqueName of the bundle finds its record, and the record carries that key and name.
#[test]
#[ignore = "T04: by_key, by_unique_name, is_retired"]
fn every_stable_key_is_indexed() {
    let gd = mini_data();
    for (key, unique_name) in &gd.bundle().stable_keys {
        let by_key = gd.by_key(*key).unwrap_or_else(|| panic!("by_key({key:?})"));
        let by_name = gd
            .by_unique_name(unique_name.as_str())
            .unwrap_or_else(|| panic!("by_unique_name({unique_name:?})"));
        assert_eq!(by_key, by_name);
        assert_eq!(by_key.key(), *key);
        assert_eq!(by_key.unique_name(), unique_name);
        assert_eq!(gd.image_path(*key), by_key.image());
    }
    assert_eq!(gd.by_key(StableKey(LOCAL_KEY_BASE + 12_345)), None);
    assert!(!gd.is_retired(StableKey(LOCAL_KEY_BASE + 12_345)));
    assert_eq!(gd.by_unique_name("wfd:nothing/at_all"), None);
    // Retired keys name no record and are reported as retired, not unknown (API-5).
    for (key, _) in &gd.bundle().retired_keys {
        assert_eq!(gd.by_key(*key), None, "{key:?}");
        assert!(gd.is_retired(*key));
    }
    // A kitgun chamber shares its weapon record's key (data/fixtures/README.md).
    for chamber in &gd.bundle().modular.chambers {
        assert!(
            matches!(gd.by_key(chamber.key), Some(Entity::Weapon(w)) if w.id == chamber.weapon)
        );
    }
}

/// A file without the magic, and a bundle of another schema version, are rejected (pipeline.md §8.2, §9.2 "Startup").
#[test]
#[ignore = "T04: decode_bundle, read_header"]
fn foreign_files_are_rejected() {
    assert!(matches!(
        decode_bundle(b"PK\x03\x04 not a bundle at all"),
        Err(DataError::BadMagic)
    ));
    let mut bytes = encode_bundle(mini_data().bundle());
    // file = magic (8 bytes) | u32 schema_version | … (pipeline.md §8.2); all-ones reads the same in either byte order.
    bytes[8..12].copy_from_slice(&[0xFF; 4]);
    assert!(matches!(
        decode_bundle(&bytes),
        Err(DataError::SchemaVersion {
            found: u32::MAX,
            expected: BUNDLE_SCHEMA_VERSION
        })
    ));
    assert!(matches!(
        read_header(&bytes),
        Err(DataError::SchemaVersion {
            found: u32::MAX,
            ..
        })
    ));
    assert!(matches!(
        read_header(b"PK\x03\x04"),
        Err(DataError::BadMagic)
    ));
    assert!(GameData::from_bytes(&bytes).is_err());
}

/// The embedded bundle loads (build.rs: `WFD_DATA_DIR`, else `data/generated/`, else the mini fixture with a
/// `cargo:warning`; overview.md AD-11, ruling T02c-R5).
#[test]
#[ignore = "T04: build.rs embedding and GameData::embedded"]
fn embedded_bundle_loads() {
    let gd = GameData::embedded().unwrap();
    assert_eq!(gd.bundle().manifest.schema_version, BUNDLE_SCHEMA_VERSION);
    assert!(!gd.bundle().weapons.is_empty());
}

/// Every scenario fixture resolves on the mini bundle, and `to_doc` gives the document back in canonical form
/// (domain-model.md §9.2; violations never block, 09 §1.5).
#[test]
#[ignore = "T04: resolve, to_doc"]
fn scenario_fixtures_resolve_and_convert_back() {
    let gd = mini_data();
    let dir = mini().join("../scenarios");
    let mut n = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "json") {
            continue;
        }
        let input: CalcInput =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let (r, warnings) = gd
            .resolve(&input)
            .unwrap_or_else(|issues| panic!("{}: {issues:?}", path.display()));
        assert!(warnings.is_empty(), "{}: {warnings:?}", path.display());
        let doc = gd.to_doc(&r.build, &r.scenario);
        let (again, _) = gd.resolve(&doc).unwrap();
        assert_eq!(
            (&again.build, &again.scenario),
            (&r.build, &r.scenario),
            "{}",
            path.display()
        );
        n += 1;
    }
    assert!(n > 0);
}
