//! Acceptance of T03 (work-breakdown.md): every file of `data/fixtures/` deserializes into model types, the mini
//! bundle assembles into a consistent `Bundle` (dense ids, references in range, stable keys and names complete,
//! JSON and postcard round trips), every record cites a source, every row of the domain-model.md §17 coverage
//! matrix has its fixture, and the spec 00 §3.8 inputs (Braton Prime, Lancer, the eight mods) carry exactly the
//! spec values.
//!
//! The directory layout is pipeline.md §8.1 (`data/generated/`); `data/fixtures/README.md` states the per-file
//! shapes this test reads and the merge rule of `extra/*.json`.

use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use wfd_model::*;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures")
}

fn read<T: DeserializeOwned>(path: &Path) -> T {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `dir/*.json` in file-name order.
fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect(),
        Err(_) => Vec::new(),
    };
    v.sort();
    v
}

/// Record files of the §8.1 layout, in the order that defines the dense ids.
const WEAPON_FILES: [&str; 8] = [
    "primary",
    "secondary",
    "melee",
    "archgun",
    "archmelee",
    "exalted",
    "companion",
    "modular",
];
const ITEM_FILES: [&str; 10] = [
    "weapon_mod",
    "weapon_arcane",
    "frame_mod",
    "aura",
    "frame_arcane",
    "shard",
    "stance",
    "incarnon",
    "buff",
    "custom",
];

/// One `extra/<task>.json` file (README.md "Extension files", CONTRACTS.md T03 fix): record lists appended after the
/// base files, names and stable keys added, and `rules` / `modular` extended by [`deep_merge`].
#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
struct Extra {
    weapons: Vec<Weapon>,
    items: Vec<Item>,
    enemies: Vec<Enemy>,
    sets: Vec<SetDef>,
    frames: Vec<Frame>,
    combo_tables: Vec<ComboTable>,
    stances: Vec<Stance>,
    names: BTreeMap<NameKey, Box<str>>,
    stable_keys: Vec<(StableKey, UniqueName)>,
    /// Extension of `rules.json` (the readable form of `Rules`), merged by [`deep_merge`].
    rules: Option<serde_json::Value>,
    /// Extension of `modular.json` (`ModularTables`), merged by [`deep_merge`].
    modular: Option<serde_json::Value>,
}

/// The extensible lists of `rules` and `modular` (readable JSON paths) and the natural keys of their rows
/// (README.md "Extension files" rule 3). Each inner slice is one key: the listed fields together; a field `"0"`
/// indexes a tuple row (`[uniqueName, value]`); an empty key is the row itself (a list of names); a field `f[]`
/// makes every element of the row's list `f` a key (a faction belongs to one `faction_heads` row). After a merge no
/// two rows of a list share a key, so an extension adds rows but never a second row for an existing key.
const LIST_KEYS: &[(&str, &[&[&str]])] = &[
    ("rules.factions", &[&["id"], &["slug"]]),
    ("rules.scaling", &[&["name"]]),
    ("rules.mult_groups", &[&["id"]]),
    ("rules.slot_layouts", &[&["class"]]),
    ("rules.body_parts.faction_heads", &[&["factions[]"]]),
    ("rules.special_dr", &[&["id"]]),
    ("rules.riven_traits", &[&["id"]]),
    (
        "rules.riven_formula.layouts",
        &[&["positives", "negatives"]],
    ),
    ("rules.toggles", &[&["id"]]),
    ("rules.melee.heavy_class", &[&["class"]]),
    ("rules.melee.heavy_kappa", &[&["0"]]),
    (
        "rules.melee.attack_triggers",
        &[&["weapons", "trigger", "attack", "attack_idx"]],
    ),
    ("rules.melee.stance_co", &[&["stance", "class"]]),
    // (class, weapons): the real 9-E table has two `exalted_weapon` rows told apart by `weapons` (DC-20).
    ("rules.melee.finishers", &[&["class", "weapons"]]),
    ("rules.melee.combo_specials", &[&["0"]]),
    ("rules.helminth.restricted", &[&[]]),
    ("rules.helminth.hosts", &[&["frame"]]),
    ("rules.categories", &[&["bit"], &["name"]]),
    ("rules.tags", &[&["id"], &["name"]]),
    ("modular.chambers", &[&["unique_name"], &["key"]]),
    ("modular.grips", &[&["unique_name"], &["key"]]),
    ("modular.loaders", &[&["unique_name"], &["key"]]),
    ("modular.strikes", &[&["unique_name"], &["key"]]),
    ("modular.links", &[&["unique_name"], &["key"]]),
];

/// The keys of one row under one key spec of [`LIST_KEYS`].
fn row_keys(row: &serde_json::Value, key: &[&str]) -> Vec<String> {
    let field = |f: &str| match row {
        serde_json::Value::Array(a) => f.parse::<usize>().ok().and_then(|i| a.get(i)).cloned(),
        _ => row.get(f).cloned(),
    };
    match key {
        [] => vec![row.to_string()],
        [f] if f.ends_with("[]") => match field(&f[..f.len() - 2]) {
            Some(serde_json::Value::Array(a)) => a.iter().map(|v| v.to_string()).collect(),
            other => vec![format!("{other:?}")],
        },
        _ => vec![
            serde_json::Value::Array(
                key.iter()
                    .map(|f| field(f).unwrap_or(serde_json::Value::Null))
                    .collect(),
            )
            .to_string(),
        ],
    }
}

/// Checks that no list of [`LIST_KEYS`] under `root` (the readable JSON of `Rules` or `ModularTables`, named
/// `what`) has two rows with the same key.
fn check_list_keys(root: &serde_json::Value, what: &str) -> Result<(), String> {
    for (path, keys) in LIST_KEYS {
        let Some(rest) = path.strip_prefix(what).and_then(|p| p.strip_prefix('.')) else {
            continue;
        };
        let Some(serde_json::Value::Array(rows)) = rest.split('.').try_fold(root, |v, k| v.get(k))
        else {
            continue;
        };
        for key in *keys {
            let mut seen = BTreeSet::new();
            for row in rows {
                for k in row_keys(row, key) {
                    if !seen.insert(k.clone()) {
                        return Err(format!(
                            "{path}: two rows with the key {key:?} = {k}; an extension adds rows, it never \
                             overrides one"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// The merge rule of `extra/<task>.json` for `rules` and `modular` (README.md, CONTRACTS.md T03 fix): objects merge
/// key by key, a key absent or `null` in the base takes the extension's value, the lists of [`LIST_KEYS`] and the
/// citation lists (`…sources`) are concatenated (base first, then the extension), and any other collision (a scalar,
/// a map entry that already exists, any other list such as the fixed-size `status` table or the `co_default` flags)
/// is an error: an extension adds rows and keys, it never overrides one. List tables (`factions`, `toggles`,
/// `mult_groups`, `melee.heavy_class`, `body_parts.faction_heads`, `helminth.restricted`, `modular.grips` …)
/// therefore append, map tables (`game_constants`) gain new keys, and scalar fields (`riven_formula.disposition`, the
/// `co_default` rules) cannot be changed. [`merge_section`] then rejects a duplicate key ([`check_list_keys`]). Dense
/// ids inside appended rows (`Faction.id`, `TagDef.id`, `CategoryDef.bit`) must continue the base sequence; the id
/// test checks them.
fn deep_merge(
    base: &mut serde_json::Value,
    ext: serde_json::Value,
    path: &str,
) -> Result<(), String> {
    use serde_json::Value;
    match (base, ext) {
        (Value::Object(b), Value::Object(e)) => {
            for (k, v) in e {
                let p = format!("{path}.{k}");
                match b.get_mut(&k) {
                    None | Some(Value::Null) => {
                        b.insert(k, v);
                    }
                    Some(slot) => deep_merge(slot, v, &p)?,
                }
            }
            Ok(())
        }
        (Value::Array(b), Value::Array(e))
            if path.ends_with(".sources") || LIST_KEYS.iter().any(|(p, _)| *p == path) =>
        {
            b.extend(e);
            Ok(())
        }
        (Value::Array(_), Value::Array(_)) => Err(format!(
            "{path}: not an extensible list (README.md \"Extension files\" rule 3)"
        )),
        (_, _) => Err(format!(
            "{path}: an extension cannot override an existing value"
        )),
    }
}

/// Merges `ext` into `section` of the bundle through its readable JSON form.
fn merge_section<T: serde::Serialize + DeserializeOwned>(
    section: &mut T,
    ext: serde_json::Value,
    what: &str,
) -> Result<(), String> {
    let mut v = serde_json::to_value(&*section).map_err(|e| e.to_string())?;
    if !ext.is_object() {
        return Err(format!("{what}: the extension must be an object"));
    }
    deep_merge(&mut v, ext, what)?;
    check_list_keys(&v, what)?;
    *section = serde_json::from_value(v).map_err(|e| format!("{what}: {e}"))?;
    Ok(())
}

/// Applies one extension file to the bundle (README.md "Extension files").
fn apply_extra(
    b: &mut Bundle,
    names: &mut BTreeMap<NameKey, Box<str>>,
    x: Extra,
) -> Result<(), String> {
    b.weapons.extend(x.weapons);
    for it in &x.items {
        *b.manifest
            .coverage
            .0
            .entry(it.kind.slug().into())
            .or_default()
            .entry(it.status)
            .or_default() += 1;
    }
    b.items.extend(x.items);
    b.enemies.extend(x.enemies);
    b.sets.extend(x.sets);
    b.frames.extend(x.frames);
    b.combo_tables.extend(x.combo_tables);
    b.stances.extend(x.stances);
    for (k, v) in x.names {
        if names.insert(k.clone(), v).is_some() {
            return Err(format!("names: {k} exists"));
        }
    }
    b.stable_keys.extend(x.stable_keys);
    if let Some(r) = x.rules {
        merge_section(&mut b.rules, r, "rules")?;
    }
    if let Some(m) = x.modular {
        merge_section(&mut b.modular, m, "modular")?;
    }
    Ok(())
}

fn load_mini() -> Bundle {
    let dir = fixtures().join("mini");
    let mut weapons = Vec::new();
    for f in WEAPON_FILES {
        weapons.extend(read::<Vec<Weapon>>(&dir.join(format!("weapons/{f}.json"))));
    }
    let mut items = Vec::new();
    for f in ITEM_FILES {
        items.extend(read::<Vec<Item>>(&dir.join(format!("items/{f}.json"))));
    }
    let mut enemies = Vec::new();
    for p in json_files(&dir.join("enemies")) {
        enemies.extend(read::<Vec<Enemy>>(&p));
    }
    let mut names: BTreeMap<NameKey, Box<str>> = read(&dir.join("names/en.json"));
    let mut b = Bundle {
        manifest: read(&dir.join("manifest.json")),
        weapons,
        items,
        sets: read(&dir.join("sets.json")),
        stances: read(&dir.join("stances.json")),
        modular: read(&dir.join("modular.json")),
        combo_tables: read(&dir.join("combo_tables.json")),
        frames: read(&dir.join("frames.json")),
        enemies,
        rules: read(&dir.join("rules.json")),
        names: NameTable::default(),
        stable_keys: read(&dir.join("stable_keys.json")),
        // The mini fixture has no retired key (`Bundle.retired_keys`, API-5 of the contract review).
        retired_keys: Vec::new(),
    };
    for p in json_files(&dir.join("extra")) {
        let x: Extra = read(&p);
        apply_extra(&mut b, &mut names, x).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    }
    b.names.0.insert("en".into(), names);
    b
}

fn key_of(b: &Bundle, un: &str) -> StableKey {
    b.stable_keys
        .iter()
        .find(|(_, u)| u.as_str() == un)
        .unwrap_or_else(|| panic!("no stable key for {un}"))
        .0
}

#[test]
fn every_fixture_file_deserializes() {
    // Deserialization is the schema check (pipeline.md §8.2): load_mini reads every record file with its model type.
    let b = load_mini();
    assert!(b.weapons.len() >= 12, "weapons: {}", b.weapons.len());
    assert!(b.items.len() >= 70, "items: {}", b.items.len());
    assert!(b.enemies.len() >= 6, "enemies: {}", b.enemies.len());
    assert_eq!(b.manifest.schema_version, BUNDLE_SCHEMA_VERSION);
    // No stray JSON file in the bundle directory that the layout does not name.
    let dir = fixtures().join("mini");
    let known: BTreeSet<&str> = [
        "manifest.json",
        "sets.json",
        "stances.json",
        "combo_tables.json",
        "modular.json",
        "frames.json",
        "rules.json",
        "stable_keys.json",
    ]
    .into_iter()
    .collect();
    for p in json_files(&dir) {
        let n = p.file_name().unwrap().to_str().unwrap();
        assert!(known.contains(n), "unexpected file {n}");
    }
}

#[test]
fn bundle_round_trips_in_both_encodings() {
    let b = load_mini();
    let text = serde_json::to_string(&b).unwrap();
    let back: Bundle = serde_json::from_str(&text).unwrap();
    assert_eq!(back, b, "JSON round trip");
    let bytes = postcard::to_stdvec(&b).unwrap();
    let back: Bundle = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, b, "postcard round trip");
}

#[test]
fn ids_are_dense_and_references_are_in_range() {
    let b = load_mini();
    for (i, w) in b.weapons.iter().enumerate() {
        assert_eq!(w.id.idx(), i, "weapon id of {}", w.unique_name);
    }
    for (i, x) in b.items.iter().enumerate() {
        assert_eq!(x.id.idx(), i, "item id of {}", x.unique_name);
    }
    for (i, e) in b.enemies.iter().enumerate() {
        assert_eq!(e.id.idx(), i, "enemy id of {}", e.unique_name);
    }
    for (i, s) in b.sets.iter().enumerate() {
        assert_eq!(s.id.idx(), i);
    }
    for (i, s) in b.stances.iter().enumerate() {
        assert_eq!(s.id.idx(), i);
    }
    for (i, t) in b.combo_tables.iter().enumerate() {
        assert_eq!(t.id.idx(), i);
    }
    for (i, f) in b.frames.iter().enumerate() {
        assert_eq!(f.id.idx(), i);
    }
    for (i, f) in b.rules.factions.iter().enumerate() {
        assert_eq!(f.id.idx(), i);
    }
    for (i, t) in b.rules.tags.iter().enumerate() {
        assert_eq!(t.id.idx(), i, "tag {}", t.name);
    }
    for (i, c) in b.rules.categories.iter().enumerate() {
        assert_eq!(c.bit as usize, i, "category {}", c.name);
    }
    // Records are sorted by uniqueName within each file (pipeline.md §8.1; `record_files_are_sorted`), so ids follow
    // file order.
    let n_items = b.items.len();
    let n_weapons = b.weapons.len();
    let n_tags = b.rules.tags.len();
    let in_items = |id: ItemId, what: &str| assert!(id.idx() < n_items, "{what}: item {id:?}");
    for x in &b.items {
        x.incompat.iter().for_each(|i| in_items(*i, "incompat"));
        if let Some(s) = x.set {
            assert!(
                b.sets[s.idx()].members.contains(&x.id),
                "{} not a member of its set",
                x.unique_name
            );
        }
        if let ItemKind::Stance { stance } = &x.kind {
            assert_eq!(b.stances[stance.idx()].item, x.id);
        }
        x.compat
            .weapons
            .iter()
            .for_each(|w| assert!(w.idx() < n_weapons));
        x.compat
            .tags
            .iter()
            .chain(&x.compat.forbid_tags)
            .for_each(|t| assert!(t.idx() < n_tags));
    }
    for s in &b.sets {
        for m in &s.members {
            assert_eq!(b.items[m.idx()].set, Some(s.id), "set member {m:?}");
        }
    }
    for st in &b.stances {
        in_items(st.item, "stance item");
        assert!(st.table.idx() < b.combo_tables.len());
        assert_eq!(
            b.combo_tables[st.table.idx()].source,
            ComboSource::Stance(st.id)
        );
    }
    for w in &b.weapons {
        for a in &w.attacks {
            if let Some(p) = a.parent {
                assert!(p.idx() < w.attacks.len(), "{} parent", w.unique_name);
            }
            assert!(a.key.idx as usize == w.attacks.iter().position(|x| x == a).unwrap());
        }
        for g in &w.groups {
            g.members
                .iter()
                .for_each(|m| assert!(m.idx() < w.attacks.len()));
        }
        w.compat_tags.iter().for_each(|t| assert!(t.idx() < n_tags));
        if let Some(inc) = &w.incarnon {
            // `tiers[t − 1][i]` is the perk with `tier = t` and `index = i` (`SlotContent::Perk.index` indexes the
            // same list, build.rs), so every evolution lists its perks completely and in order.
            for (t, tier) in inc.tiers.iter().enumerate() {
                for (i, p) in tier.iter().enumerate() {
                    in_items(*p, "incarnon tier");
                    match &b.items[p.idx()].kind {
                        ItemKind::IncarnonPerk {
                            family,
                            tier: pt,
                            index,
                        } => {
                            assert_eq!(*family, inc.family);
                            assert_eq!(
                                (*pt as usize, *index as usize),
                                (t + 1, i),
                                "{}: perk {} at tiers[{t}][{i}]",
                                w.unique_name,
                                b.items[p.idx()].unique_name
                            );
                        }
                        k => panic!("incarnon tier holds {k:?}"),
                    }
                }
            }
        }
        if let Some(m) = &w.melee {
            assert!(m.stanceless.idx() < b.combo_tables.len());
            if let Some(u) = m.unique {
                assert!(matches!(
                    b.combo_tables[u.idx()].source,
                    ComboSource::Unique { .. }
                ));
            }
            if let Some(h) = &m.heavy {
                let a = &w.attacks[h.attack.idx()];
                assert!(
                    a.roles.contains(AttackRoles::HEAVY),
                    "{} heavy",
                    w.unique_name
                );
                assert!(matches!(a.synthesized, Some(Synthesized::HeavyFrom { .. })));
            }
        }
        if let Some(m) = &w.modular {
            let n = match m.kind {
                ModularKind::Kitgun => b.modular.chambers.len(),
                ModularKind::Zaw => b.modular.strikes.len(),
            };
            assert!((m.base as usize) < n);
        }
    }
    for c in &b.modular.chambers {
        assert!(c.weapon.idx() < n_weapons);
        assert_eq!(b.weapons[c.weapon.idx()].unique_name, c.unique_name);
        c.by_grip
            .iter()
            .for_each(|g| assert!((g.grip as usize) < b.modular.grips.len()));
    }
    let n_scaling = b.rules.scaling.len();
    for e in &b.enemies {
        assert!(e.faction.idx() < b.rules.factions.len());
        if let Some(f) = e.damage_faction {
            assert!(f.idx() < b.rules.factions.len());
        }
        assert!((e.scaling as usize) < n_scaling);
        if let Some(s) = e.special_dr {
            assert!((s as usize) < b.rules.special_dr.len());
        }
        assert_eq!(
            e.parts.iter().filter(|p| p.is_default).count(),
            1,
            "{} default part",
            e.unique_name
        );
    }
    for f in &b.rules.factions {
        f.bane_of
            .iter()
            .for_each(|x| assert!(x.idx() < b.rules.factions.len()));
        assert!((f.scaling_family as usize) < n_scaling);
    }
    // Every multiplier group an effect names exists.
    let groups: BTreeSet<&str> = b.rules.mult_groups.iter().map(|g| &*g.id).collect();
    let all_effects = b
        .items
        .iter()
        .flat_map(|i| &i.effects)
        .chain(b.sets.iter().flat_map(|s| &s.bonus))
        .chain(b.weapons.iter().flat_map(|w| &w.passives));
    for e in all_effects {
        if let Target::Mult(g) = &e.target {
            assert!(groups.contains(&**g), "unknown multiplier group {g}");
        }
        if let Target::Faction(FactionSel::Of(v)) = &e.target {
            for s in v {
                assert!(
                    b.rules.factions.iter().any(|f| f.slug == *s),
                    "unknown faction {s}"
                );
            }
        }
    }
}

#[test]
fn stable_keys_and_names_are_complete() {
    let b = load_mini();
    let mut by_key = BTreeMap::new();
    let mut by_name = BTreeMap::new();
    for (k, u) in &b.stable_keys {
        assert!(!k.is_local(), "global key {k} below LOCAL_KEY_BASE");
        assert!(by_key.insert(*k, u.clone()).is_none(), "key {k} twice");
        assert!(by_name.insert(u.clone(), *k).is_none(), "{u} twice");
    }
    let check = |key: StableKey, un: &UniqueName| {
        assert_eq!(by_name.get(un), Some(&key), "stable key of {un}");
    };
    b.weapons.iter().for_each(|w| check(w.key, &w.unique_name));
    b.items.iter().for_each(|x| check(x.key, &x.unique_name));
    b.enemies.iter().for_each(|e| check(e.key, &e.unique_name));
    b.frames.iter().for_each(|f| check(f.key, &f.unique_name));
    let m = &b.modular;
    m.chambers.iter().for_each(|c| check(c.key, &c.unique_name));
    m.strikes.iter().for_each(|c| check(c.key, &c.unique_name));
    for p in m.grips.iter().chain(&m.loaders).chain(&m.links) {
        check(p.key, &p.unique_name);
    }

    let en = &b.names.0["en"];
    let mut keys: Vec<&NameKey> = Vec::new();
    for w in &b.weapons {
        keys.push(&w.name);
        keys.extend(w.attacks.iter().map(|a| &a.name));
        keys.extend(w.zoom.iter().map(|z| &z.name));
    }
    keys.extend(b.items.iter().map(|x| &x.name));
    keys.extend(b.sets.iter().map(|x| &x.name));
    keys.extend(b.enemies.iter().map(|x| &x.name));
    keys.extend(b.rules.factions.iter().map(|x| &x.name));
    for f in &b.frames {
        keys.push(&f.name);
        keys.extend(f.abilities.iter().map(|a| &a.name));
    }
    for t in &b.combo_tables {
        keys.extend(t.combos.iter().map(|c| &c.name));
    }
    keys.extend(
        m.grips
            .iter()
            .chain(&m.loaders)
            .chain(&m.links)
            .map(|p| &p.name),
    );
    for k in keys {
        assert!(en.contains_key(k), "no English name for {k}");
    }
}

#[test]
fn every_record_cites_a_source() {
    let b = load_mini();
    let sources_md = std::fs::read_to_string(fixtures().join("mini/SOURCES.md")).unwrap();
    for w in &b.weapons {
        assert!(!w.provenance.sources.is_empty(), "{}", w.unique_name);
        assert!(
            sources_md.contains(&format!("`{}`", w.unique_name)),
            "{}",
            w.unique_name
        );
    }
    for x in &b.items {
        assert!(!x.sources.is_empty(), "{}", x.unique_name);
        assert!(
            sources_md.contains(&format!("`{}`", x.unique_name)),
            "{}",
            x.unique_name
        );
    }
    for un in b
        .enemies
        .iter()
        .map(|e| &e.unique_name)
        .chain(b.frames.iter().map(|f| &f.unique_name))
        .chain(b.sets.iter().map(|s| &s.unique_name))
    {
        assert!(sources_md.contains(&format!("`{un}`")), "{un}");
    }
    for s in &b.rules.status {
        assert!(!s.source.is_empty());
    }
    for g in &b.rules.mult_groups {
        assert!(!g.sources.is_empty(), "{}", g.id);
    }
    // No record is taken from the inspiration bundle (CLAUDE.md): no citation names it.
    let all = std::fs::read_dir(fixtures().join("mini"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<String>();
    assert!(
        !all.contains("min.pretty.js") && !all.contains("JS L"),
        "inspiration cited as a source"
    );
}

/// Backticked uniqueNames of each row of `mini/COVERAGE.md` (domain-model.md §17 rows).
#[test]
fn every_coverage_row_has_a_fixture() {
    let b = load_mini();
    let text = std::fs::read_to_string(fixtures().join("mini/COVERAGE.md")).unwrap();
    let mut known: BTreeSet<&str> = BTreeSet::new();
    known.extend(b.weapons.iter().map(|w| w.unique_name.as_str()));
    known.extend(b.items.iter().map(|w| w.unique_name.as_str()));
    known.extend(b.enemies.iter().map(|w| w.unique_name.as_str()));
    known.extend(b.sets.iter().map(|w| w.unique_name.as_str()));
    known.extend(b.frames.iter().map(|w| w.unique_name.as_str()));
    let m = &b.modular;
    known.extend(m.chambers.iter().map(|w| w.unique_name.as_str()));
    known.extend(m.strikes.iter().map(|w| w.unique_name.as_str()));
    known.extend(
        m.grips
            .iter()
            .chain(&m.loaders)
            .chain(&m.links)
            .map(|w| w.unique_name.as_str()),
    );
    let spec = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/architecture/domain-model.md"),
    )
    .unwrap();
    let section = &spec[spec.find("## 17. Coverage matrix").unwrap()..spec.find("## 18.").unwrap()];
    let spec_rows: Vec<&str> = section
        .lines()
        .filter(|l| {
            l.starts_with("| ") && !l.starts_with("| Spec mechanic") && !l.starts_with("|---")
        })
        .map(|l| {
            l.split(" | ")
                .next()
                .unwrap()
                .trim_start_matches("| ")
                .trim()
        })
        .collect();
    let mut rows = 0;
    for line in text
        .lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("|---"))
    {
        let cells: Vec<&str> = line.split(" | ").collect();
        if cells[0] == "| Spec mechanic (domain-model.md §17)" {
            continue;
        }
        rows += 1;
        let row = cells[0].trim_start_matches("| ").trim();
        assert!(
            spec_rows.contains(&row),
            "coverage row not in domain-model.md §17: {row}"
        );
        let names: Vec<&str> = cells[1].split('`').skip(1).step_by(2).collect();
        assert!(!names.is_empty(), "row without fixture: {row}");
        for n in names {
            assert!(
                known.contains(n),
                "row {row}: fixture {n} not in the mini bundle"
            );
        }
    }
    assert_eq!(rows, spec_rows.len(), "every §17 row is covered");
}

fn scenarios() -> Vec<(PathBuf, CalcInput)> {
    let dir = fixtures().join("scenarios");
    json_files(&dir)
        .into_iter()
        .chain(json_files(&dir.join("real")))
        .map(|p| {
            let c: CalcInput = read(&p);
            (p, c)
        })
        .collect()
}

fn doc_keys(c: &CalcInput) -> Vec<StableKey> {
    let mut v = vec![c.build.weapon, c.scenario.enemy.enemy];
    for s in &c.build.slots {
        match &s.content {
            SlotContent::Item { item, .. } | SlotContent::Shard { item, .. } => v.push(*item),
            SlotContent::Subsumed { ability, .. } => v.push(*ability),
            SlotContent::Riven(_) | SlotContent::Perk { .. } => {}
        }
    }
    v.extend(c.scenario.buffs.iter().map(|b| b.buff));
    if let Some(k) = &c.build.variant.kitgun {
        v.extend([k.grip, k.loader]);
    }
    if let Some(z) = &c.build.variant.zaw {
        v.extend([z.grip, z.link]);
    }
    v
}

#[test]
fn every_scenario_decodes_and_resolves() {
    let b = load_mini();
    let keys: BTreeMap<StableKey, &UniqueName> =
        b.stable_keys.iter().map(|(k, u)| (*k, u)).collect();
    let names: BTreeSet<&UniqueName> = keys.values().copied().collect();
    let all = scenarios();
    let found: BTreeSet<String> = all
        .iter()
        .map(|(p, _)| {
            p.strip_prefix(fixtures().join("scenarios"))
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    for required in [
        "braton_prime_lancer_l100_sp.json",
        "real/braton_prime_lancer_l100_sp.json",
    ] {
        assert!(found.contains(required), "missing scenario {required}");
    }
    assert!(all.len() >= 7, "scenarios: {found:?}");
    for (p, c) in &all {
        assert_eq!(c.v, DOC_VERSION, "{}", p.display());
        let text = serde_json::to_string(c).unwrap();
        assert_eq!(&serde_json::from_str::<CalcInput>(&text).unwrap(), c);
        let real = p.parent().unwrap().ends_with("real");
        for k in doc_keys(c) {
            if real {
                // Every key is session-local and carried by uniqueName (AD-18), so the document decodes on the
                // real bundle whatever its global keys are.
                assert!(k.is_local(), "{}: global key {k}", p.display());
                let un =
                    c.x.get(&k)
                        .unwrap_or_else(|| panic!("{}: no x entry for {k}", p.display()));
                assert!(names.contains(un), "{}: {un} unknown", p.display());
            } else {
                assert!(
                    keys.contains_key(&k),
                    "{}: key {k} not in the mini bundle",
                    p.display()
                );
            }
        }
    }
    // The scenario kinds of work-breakdown.md T03.
    let weapon_of = |c: &CalcInput| {
        let k = if c.build.weapon.is_local() {
            key_of(&b, c.x[&c.build.weapon].as_str())
        } else {
            c.build.weapon
        };
        b.weapons.iter().find(|w| w.key == k).unwrap()
    };
    let enemy_of = |c: &CalcInput| {
        b.enemies
            .iter()
            .find(|e| e.key == c.scenario.enemy.enemy)
            .unwrap()
    };
    // Every slot exists in the weapon's layout (09 §2.1) and holds a content kind the slot accepts (E-9.3); a perk
    // index points into the evolution's perk list (build.rs `SlotContent::Perk`).
    for (p, c) in &all {
        let w = weapon_of(c);
        let class = serde_json::to_value(w.slot).unwrap();
        let layout = b
            .rules
            .slot_layouts
            .iter()
            .find(|l| serde_json::to_value(l.class).unwrap() == class)
            .unwrap_or_else(|| panic!("{}: no slot layout for {class}", p.display()));
        let item_of = |k: StableKey| {
            let k = if k.is_local() {
                key_of(&b, c.x[&k].as_str())
            } else {
                k
            };
            b.items.iter().find(|x| x.key == k).unwrap()
        };
        for s in &c.build.slots {
            let def = layout
                .slots
                .iter()
                .find(|d| d.id == s.slot)
                .unwrap_or_else(|| panic!("{}: slot {:?} not in the layout", p.display(), s.slot));
            let kind = match &s.content {
                SlotContent::Item { item, .. } => match &item_of(*item).kind {
                    ItemKind::WeaponMod { exilus, .. } if s.slot == SlotId::Exilus => {
                        assert!(
                            *exilus,
                            "{}: non-exilus mod in the exilus slot",
                            p.display()
                        );
                        SlotKind::Exilus
                    }
                    ItemKind::WeaponMod { .. } | ItemKind::FrameMod { .. } => SlotKind::General,
                    ItemKind::Stance { .. } => SlotKind::Stance,
                    ItemKind::WeaponArcane { .. } | ItemKind::FrameArcane => SlotKind::Arcane,
                    ItemKind::Aura => SlotKind::Aura,
                    k => panic!("{}: {k:?} in slot {:?}", p.display(), s.slot),
                },
                SlotContent::Riven(_) => SlotKind::Riven,
                SlotContent::Shard { .. } => SlotKind::Shard,
                SlotContent::Subsumed { .. } => SlotKind::Subsumed,
                SlotContent::Perk { index } => {
                    let SlotId::Inc(t) = s.slot else {
                        panic!("{}: perk in slot {:?}", p.display(), s.slot)
                    };
                    let inc = w
                        .incarnon
                        .as_ref()
                        .expect("perk on a weapon without Incarnon");
                    assert!(
                        (*index as usize) < inc.tiers[t as usize - 1].len(),
                        "{}: inc{t} index {index} out of range",
                        p.display()
                    );
                    SlotKind::IncarnonPerk
                }
            };
            assert!(
                def.accepts.contains(&kind),
                "{}: slot {:?} does not accept {kind:?}",
                p.display(),
                s.slot
            );
        }
    }
    let globals: Vec<&CalcInput> = all
        .iter()
        .filter(|(p, _)| !p.parent().unwrap().ends_with("real"))
        .map(|(_, c)| c)
        .collect();
    assert!(globals.iter().any(|c| {
        weapon_of(c).incarnon.is_some()
            && c.build
                .slots
                .iter()
                .any(|s| matches!(s.content, SlotContent::Perk { .. }))
    }));
    assert!(globals.iter().any(|c| weapon_of(c).melee.is_some()
        && c.build.slots.iter().any(|s| s.slot == SlotId::Stance)
        && c.scenario.toggles.melee_plan != MeleePlanSel::Auto));
    assert!(
        globals
            .iter()
            .any(|c| weapon_of(c).traits.contains(WeaponTraits::KUVA)
                && c.build.variant.progenitor.is_some())
    );
    assert!(globals.iter().any(|c| enemy_of(c).shield > 0.0));
    // T-06 and T-07 (and an ability strip) on an armored target (open-items.md §3 issue 10).
    assert!(globals.iter().any(|c| c.scenario.toggles.armor_strip > 0.0
        && c.scenario.toggles.external_viral.is_some()
        && enemy_of(c).armor > 0.0));
    assert!(
        globals
            .iter()
            .any(|c| c.scenario.enemy.eximus && enemy_of(c).eximus.og && !enemy_of(c).eximus.off)
    );
}

#[test]
fn spec_00_3_8_inputs_have_exactly_the_spec_values() {
    let b = load_mini();
    let c: CalcInput = read(&fixtures().join("scenarios/braton_prime_lancer_l100_sp.json"));
    let w = b.weapons.iter().find(|w| w.key == c.build.weapon).unwrap();
    assert_eq!(
        w.unique_name.as_str(),
        "/Lotus/Weapons/Tenno/Rifle/BratonPrime"
    );
    assert_eq!(b.names.get("en", &w.name), Some("Braton Prime"));
    // Braton Prime "Normal Attack": IMP 1.75, PUN 12.25, SLA 21 (B' = 35); CC0 0.12; CDM0 2.0; SC0 0.26; FR 9.58;
    // MS0 1; Mag 75; Reload 2.15 s; Auto; Hit-Scan (00 §3.8 Inputs).
    let a = &w.attacks[0];
    assert_eq!(b.names.get("en", &a.name), Some("Normal Attack"));
    let mut base = [0.0; N_DT];
    base[DamageType::IMP.index()] = 1.75;
    base[DamageType::PUN.index()] = 12.25;
    base[DamageType::SLA.index()] = 21.0;
    assert_eq!(a.base, base);
    assert_eq!(a.base.iter().sum::<f64>(), 35.0);
    assert_eq!((a.cc, a.cd, a.sc, a.multishot), (0.12, 2.0, 0.26, 1.0));
    assert_eq!(a.fire.rate, 9.58);
    assert_eq!(a.fire.trigger, Trigger::Auto);
    assert_eq!(w.trigger, Trigger::Auto);
    assert_eq!(a.shot, ShotType::HitScan);
    assert_eq!(w.magazine.as_ref().unwrap().size, 75.0);
    assert_eq!(w.reload, ReloadSpec::Standard { time: 2.15 });
    assert_eq!(a.co.behaviour, CoBehaviour::Adding);

    // Mods at max rank, in slots 1..8 (Infected Clip slot 5, Stormbringer slot 6).
    let expect: [(&str, Target, f64); 8] = [
        (
            "Serration",
            Target::Stat(StatKey::parse("base_dmg").unwrap()),
            1.65,
        ),
        (
            "Split Chamber",
            Target::Stat(StatKey::parse("multishot").unwrap()),
            0.90,
        ),
        (
            "Point Strike",
            Target::Stat(StatKey::parse("cc_rel").unwrap()),
            1.50,
        ),
        (
            "Vital Sense",
            Target::Stat(StatKey::parse("cd_rel").unwrap()),
            1.20,
        ),
        (
            "Infected Clip",
            Target::Element(ElemSel::Type(DamageType::TOX)),
            0.90,
        ),
        (
            "Stormbringer",
            Target::Element(ElemSel::Type(DamageType::ELE)),
            0.90,
        ),
        (
            "Primed Bane of Grineer",
            Target::Faction(FactionSel::Of(smallvec::smallvec!["grineer".into()])),
            0.55,
        ),
        (
            "Speed Trigger",
            Target::Stat(StatKey::parse("fire_rate").unwrap()),
            0.60,
        ),
    ];
    assert_eq!(c.build.slots.len(), 8);
    for (i, (slot, (name, target, value))) in c.build.slots.iter().zip(expect).enumerate() {
        assert_eq!(slot.slot, SlotId::Gen(i as u8));
        let SlotContent::Item { item: k, rank, .. } = &slot.content else {
            panic!("slot {i}")
        };
        assert_eq!(*rank, None, "max rank");
        let it = b.items.iter().find(|x| x.key == *k).unwrap();
        assert_eq!(b.names.get("en", &it.name), Some(name));
        assert_eq!(it.effects.len(), 1, "{name}");
        let e = &it.effects[0];
        assert_eq!(e.target, target, "{name}");
        match &e.value {
            ValueExpr::Ranked { max, rank, .. } => {
                assert_eq!(*max, value, "{name}");
                assert_eq!(*rank, RankCurve::Linear, "{name}");
            }
            v => panic!("{name}: {v:?}"),
        }
        assert_eq!(e.when, Cond::Always, "{name}: unconditional");
        if matches!(e.target, Target::Element(_)) {
            assert_eq!(e.order, Some(ElemOrder::Slot), "{name}");
        }
    }
    let ranks: Vec<u8> = c
        .build
        .slots
        .iter()
        .map(|s| match &s.content {
            SlotContent::Item { item, .. } => {
                b.items.iter().find(|x| x.key == *item).unwrap().max_rank
            }
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(ranks, [10, 5, 5, 5, 5, 5, 10, 5]);

    // Lancer (Grineer): base health 100, base armor 100, BaseLevel 1, head 3.0×; level 100, Steel Path, no shield,
    // no overguard, body hits.
    let e = b
        .enemies
        .iter()
        .find(|e| e.key == c.scenario.enemy.enemy)
        .unwrap();
    assert_eq!(e.unique_name.as_str(), "wfd:enemy/grineer/lancer");
    assert_eq!(b.rules.factions[e.faction.idx()].slug.as_ref(), "grineer");
    assert_eq!(
        (e.health, e.armor, e.shield, e.overguard),
        (100.0, 100.0, 0.0, 0.0)
    );
    assert_eq!(e.base_level, 1);
    let head = e.parts.iter().find(|p| p.is_head).unwrap();
    assert_eq!((head.part_mult, head.headcrit), (3.0, 2.0));
    assert_eq!(
        b.rules.scaling[e.scaling as usize].health,
        ScalingCurve {
            c1: 0.015,
            e1: 2.12,
            c2: 10.7332,
            e2: 0.72,
            lo: 70.0,
            hi: 80.0,
            q_from_level_one: false
        }
    );
    let gf = &b.rules.factions[e.faction.idx()].type_mods;
    assert_eq!(
        (gf[DamageType::IMP.index()], gf[DamageType::COR.index()]),
        (1.5, 1.5)
    );
    assert_eq!(gf.iter().filter(|x| **x != 1.0).count(), 2);
    let en = &c.scenario.enemy;
    assert_eq!(
        (en.level, en.steel_path, en.eximus, en.party),
        (100, true, false, 1)
    );
    assert_eq!(c.scenario.toggles, Toggles::default());
    assert_eq!(c.scenario.conditions.mode, ConditionMode::AllMax);
    assert!(c.scenario.buffs.is_empty());
    assert_eq!(b.rules.game_constants["E-5.4.steel_path_mult"], 2.5);

    // The real-bundle twin is the same document with session-local keys carried by uniqueName.
    let r: CalcInput = read(&fixtures().join("scenarios/real/braton_prime_lancer_l100_sp.json"));
    assert_eq!(r.x[&r.build.weapon].as_str(), w.unique_name.as_str());
    assert_eq!(
        r.scenario,
        c.scenario.clone().with_enemy_key(r.scenario.enemy.enemy)
    );
    for (s, t) in r.build.slots.iter().zip(&c.build.slots) {
        let (SlotContent::Item { item: a, .. }, SlotContent::Item { item: b2, .. }) =
            (&s.content, &t.content)
        else {
            unreachable!()
        };
        assert_eq!(key_of(&b, r.x[a].as_str()), *b2);
    }
}

#[test]
fn stance_tables_follow_e_7_0() {
    // TV-07-01: Guiding Light (Blind Justice Neutral) has D_K = 2.6, N_K = 8, ΣS = 19.0. TV-07-12: Blind Justice
    // Forward Block input 3 (index 2) is Dmg [200, 400, 100], Types [Sweep, "", Slam], Procs with a hole → no procs.
    // TV-07-08: Guiding Light strike 1 is S = 3 with S^phys_SLA = 1.0.
    let b = load_mini();
    let bj = b
        .combo_tables
        .iter()
        .find(|t| matches!(t.source, ComboSource::Stance(_)))
        .unwrap();
    let neutral = bj
        .combos
        .iter()
        .find(|c| c.kind == ComboType::Neutral)
        .unwrap();
    assert_eq!(b.names.get("en", &neutral.name), Some("Guiding Light"));
    assert_eq!(neutral.duration, Some(2.6));
    assert_eq!(neutral.strikes.len(), 8);
    assert_eq!(neutral.strikes.iter().map(|s| s.mult).sum::<f64>(), 19.0);
    // `Strike.phys` is ordered SLA, IMP, PUN (weapon.rs, domain-model.md §4; E-7.0 step 5).
    const SLA: usize = 0;
    const PUN: usize = 2;
    let mut slash_only = [0.0; 3];
    slash_only[SLA] = 1.0;
    assert_eq!(
        (neutral.strikes[0].mult, neutral.strikes[0].phys),
        (3.0, slash_only)
    );
    // Stanceless Dagger input 0: PunctureMultiplier [2] (W:Module:Stances/data) → S^phys_PUN = 1.0.
    let dagger = b
        .combo_tables
        .iter()
        .find(|t| t.source == ComboSource::Stanceless(MeleeClass::Dagger))
        .unwrap();
    let d0 = &dagger
        .combos
        .iter()
        .find(|c| c.kind == ComboType::Neutral)
        .unwrap()
        .strikes[0];
    let mut puncture_only = [0.0; 3];
    puncture_only[PUN] = 1.0;
    assert_eq!(d0.phys, puncture_only);
    let fb = bj
        .combos
        .iter()
        .find(|c| c.kind == ComboType::ForwardBlock)
        .unwrap();
    let input3: Vec<&Strike> = fb.strikes.iter().filter(|s| s.input == 2).collect();
    assert_eq!(
        input3.iter().map(|s| (s.mult, s.ty)).collect::<Vec<_>>(),
        [
            (2.0, StrikeType::Sweep),
            (4.0, StrikeType::Plain),
            (1.0, StrikeType::Slam)
        ]
    );
    assert!(input3.iter().all(|s| s.procs.is_empty() && s.cc.is_empty()));
    // The heavy of Nikana Prime: class table 9-A Nikana 5× Bleed, σ = 1 (07 E-7.40), procs of the stance entry.
    let nik = b
        .weapons
        .iter()
        .find(|w| {
            w.unique_name.as_str() == "/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana"
        })
        .unwrap();
    let h = nik.melee.as_ref().unwrap().heavy.as_ref().unwrap();
    for hits in &h.hits {
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].mult, 5.0);
        assert_eq!(&hits[0].procs[..], &[StatusType::SLA, StatusType::KND]);
    }
}

#[test]
fn reference_weapons_parse() {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RefFile {
        schema: u32,
        weapons: BTreeMap<String, String>,
    }
    let text = std::fs::read_to_string(fixtures().join("reference_weapons.toml")).unwrap();
    let r: RefFile = toml::from_str(&text).unwrap();
    assert_eq!(r.schema, 1);
    for (class, un) in &r.weapons {
        assert!(un.starts_with("/Lotus/"), "{class}: {un}");
    }
    // Keys: the WeaponClass names, plus the documented extras; every WeaponClass has an entry except the
    // out-of-scope `amp` (00 §1.3; the file's header states both lists).
    const EXTRAS: [&str; 3] = ["glaive", "kitgun", "zaw"];
    const OUT_OF_SCOPE: [&str; 1] = ["amp"];
    let classes: Vec<WeaponClass> = [
        WeaponClass::Rifle,
        WeaponClass::Shotgun,
        WeaponClass::Sniper,
        WeaponClass::Bow,
        WeaponClass::Launcher,
        WeaponClass::Speargun,
        WeaponClass::Pistol,
        WeaponClass::DualPistols,
        WeaponClass::Thrown,
        WeaponClass::ShotgunSidearm,
        WeaponClass::Melee,
        WeaponClass::ArchGun,
        WeaponClass::ArchMelee,
        WeaponClass::ExaltedRanged,
        WeaponClass::ExaltedMelee,
        WeaponClass::CompanionWeapon,
        WeaponClass::Amp,
        WeaponClass::Crossbow,
        WeaponClass::DualShotguns,
        WeaponClass::Tome,
    ]
    .into_iter()
    .inspect(|c| {
        // Exhaustive: a new WeaponClass variant fails to compile here until the list above has it.
        match c {
            WeaponClass::Rifle
            | WeaponClass::Shotgun
            | WeaponClass::Sniper
            | WeaponClass::Bow
            | WeaponClass::Launcher
            | WeaponClass::Speargun
            | WeaponClass::Pistol
            | WeaponClass::DualPistols
            | WeaponClass::Thrown
            | WeaponClass::ShotgunSidearm
            | WeaponClass::Melee
            | WeaponClass::ArchGun
            | WeaponClass::ArchMelee
            | WeaponClass::ExaltedRanged
            | WeaponClass::ExaltedMelee
            | WeaponClass::CompanionWeapon
            | WeaponClass::Amp
            | WeaponClass::Crossbow
            | WeaponClass::DualShotguns
            | WeaponClass::Tome => {}
        }
    })
    .collect();
    let names: Vec<String> = classes
        .iter()
        .map(|c| {
            serde_json::to_value(c)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    for key in r.weapons.keys() {
        assert!(
            names.contains(key) || EXTRAS.contains(&key.as_str()),
            "{key} is neither a WeaponClass nor a documented extra"
        );
    }
    for n in &names {
        assert!(
            r.weapons.contains_key(n) || OUT_OF_SCOPE.contains(&n.as_str()),
            "no reference weapon for class {n}"
        );
    }
    assert!(text.contains("`amp` has no entry"));
}

#[test]
fn record_files_are_sorted() {
    // pipeline.md §8.1: records sorted by `uniqueName` within a file; dense ids depend on it (README.md).
    fn sorted<T>(what: &str, v: &[T], un: impl Fn(&T) -> &str) {
        let names: Vec<&str> = v.iter().map(un).collect();
        let mut s = names.clone();
        s.sort_unstable();
        assert_eq!(names, s, "{what} is not sorted by uniqueName");
    }
    let dir = fixtures().join("mini");
    for f in WEAPON_FILES {
        let v: Vec<Weapon> = read(&dir.join(format!("weapons/{f}.json")));
        sorted(f, &v, |w| w.unique_name.as_str());
    }
    for f in ITEM_FILES {
        let v: Vec<Item> = read(&dir.join(format!("items/{f}.json")));
        sorted(f, &v, |x| x.unique_name.as_str());
    }
    for p in json_files(&dir.join("enemies")) {
        let v: Vec<Enemy> = read(&p);
        sorted(&p.display().to_string(), &v, |e| e.unique_name.as_str());
    }
    let sets: Vec<SetDef> = read(&dir.join("sets.json"));
    sorted("sets.json", &sets, |s| s.unique_name.as_str());
    let frames: Vec<Frame> = read(&dir.join("frames.json"));
    sorted("frames.json", &frames, |f| f.unique_name.as_str());
    let m: ModularTables = read(&dir.join("modular.json"));
    sorted("modular chambers", &m.chambers, |c| c.unique_name.as_str());
    sorted("modular strikes", &m.strikes, |c| c.unique_name.as_str());
    for (what, parts) in [
        ("grips", &m.grips),
        ("loaders", &m.loaders),
        ("links", &m.links),
    ] {
        sorted(what, parts, |p| p.unique_name.as_str());
    }
    let keys: Vec<(StableKey, UniqueName)> = read(&dir.join("stable_keys.json"));
    assert!(
        keys.windows(2).all(|w| w[0].0 < w[1].0),
        "stable_keys.json is not sorted by key"
    );
}

#[test]
fn toggle_descriptors_match_the_toggle_map_and_the_scenarios() {
    // Every descriptor names a toggle of TOGGLE_MAP and its field path; every toggle field a scenario changes from
    // its default, and every buff caster a scenario sets, has a descriptor (README.md: rules.json holds the toggles
    // the scenarios use).
    let b = load_mini();
    let field_of = |place: &TogglePlace| match place {
        TogglePlace::Scenario(p) => (*p).to_owned(),
        TogglePlace::Build(p) => p
            .split('|')
            .map(|x| format!("build.{x}"))
            .collect::<Vec<_>>()
            .join("|"),
        TogglePlace::Buffs => "buffs".to_owned(),
        TogglePlace::OptRequest(p) => format!("opt.{p}"),
    };
    for d in &b.rules.toggles {
        let (_, place, _) = TOGGLE_MAP
            .iter()
            .find(|(id, _, _)| *id == &*d.id.0)
            .unwrap_or_else(|| panic!("descriptor {} not in TOGGLE_MAP", &*d.id.0));
        assert_eq!(&*d.field, field_of(place), "field of {}", &*d.id.0);
    }
    let described: BTreeSet<String> = b
        .rules
        .toggles
        .iter()
        .flat_map(|d| d.field.split('|').map(str::to_owned).collect::<Vec<_>>())
        .collect();
    let has = |id: &str| b.rules.toggles.iter().any(|d| &*d.id.0 == id);
    let default = serde_json::to_value(Toggles::default()).unwrap();
    for (p, c) in scenarios() {
        // Buff picks (00 §6.1): a caster other than the buff's default is T-37 for the T-13 buff and the per-buff
        // caster of T-33 otherwise; an ally Strength or Duration other than 1.0 is T-33; params are the buff's own
        // inputs (T-32 for the spot params), described by the item, not by a toggle.
        for pick in &c.scenario.buffs {
            let Some(caster) = &pick.caster else { continue };
            let item = b
                .items
                .iter()
                .find(|x| x.key == pick.buff)
                .unwrap_or_else(|| panic!("{}: buff {:?}", p.display(), pick.buff));
            let ItemKind::Buff {
                toggle,
                default_caster,
                ..
            } = &item.kind
            else {
                panic!("{}: {:?} is not a buff", p.display(), pick.buff)
            };
            let t13 = toggle.as_ref().is_some_and(|t| &*t.0 == "T-13");
            let mut need = Vec::new();
            if *caster != default_caster.caster() {
                need.push(if t13 { "T-37" } else { "T-33" });
            }
            if let Caster::Ally { strength, duration } = caster
                && (*strength != 1.0 || *duration != 1.0)
            {
                need.push("T-33");
            }
            for id in need {
                assert!(
                    has(id),
                    "{}: buff {:?} sets its caster ({id}) but rules.json has no {id} descriptor",
                    p.display(),
                    pick.buff
                );
            }
        }
        let t = serde_json::to_value(&c.scenario.toggles).unwrap();
        for (k, v) in t.as_object().unwrap() {
            if default.get(k) != Some(v) {
                assert!(
                    described.contains(&format!("toggles.{k}")),
                    "{}: toggles.{k} is set but rules.json has no descriptor for it",
                    p.display()
                );
            }
        }
    }
    for id in ["T-06", "T-07", "T-30", "T-33", "T-37"] {
        assert!(has(id), "{id}");
    }
}

#[test]
fn extension_files_merge_by_the_documented_rule() {
    // The format of `extra/<task>.json` (README.md "Extension files"): records append, rule tables and modular
    // parts extend through `deep_merge` with new keys only, existing values and keys are never overridden.
    let base = load_mini();
    for (what, v) in [
        ("rules", serde_json::to_value(&base.rules).unwrap()),
        ("modular", serde_json::to_value(&base.modular).unwrap()),
    ] {
        check_list_keys(&v, what).unwrap();
    }
    let mut names = base.names.0["en"].clone();
    let mut b = base.clone();
    let mut toggle = serde_json::to_value(&base.rules.toggles[0]).unwrap();
    toggle["id"] = "T-04".into();
    toggle["field"] = "toggles.co_multiplicative".into();
    let mut heavy = serde_json::to_value(&base.rules.melee.heavy_class[0]).unwrap();
    heavy["class"] = "polearm".into();
    let mut grip = serde_json::to_value(&base.modular.grips[0]).unwrap();
    grip["unique_name"] = "wfd:test/grip".into();
    grip["key"] = 6499.into();
    let mut item = base.items[0].clone();
    item.status = EffectStatus::Draft;
    let ext: Extra = serde_json::from_value(serde_json::json!({
        "items": [item],
        "names": { "wfd:test/grip": "Test grip" },
        "rules": {
            "game_constants": { "test.constant": 1.5 },
            "toggles": [toggle],
            "tags": [{ "id": base.rules.tags.len(), "name": "TEST_TAG" }],
            "melee": { "heavy_class": [heavy] },
            "body_parts": { "sources": ["test citation"] }
        },
        "modular": { "grips": [grip] }
    }))
    .unwrap();
    apply_extra(&mut b, &mut names, ext).unwrap();
    assert_eq!(b.rules.game_constants["test.constant"], 1.5);
    assert_eq!(
        b.rules.game_constants.len(),
        base.rules.game_constants.len() + 1
    );
    assert_eq!(b.rules.toggles.len(), base.rules.toggles.len() + 1);
    assert_eq!(
        b.rules.toggles[..base.rules.toggles.len()],
        base.rules.toggles[..]
    );
    assert_eq!(&*b.rules.toggles.last().unwrap().id.0, "T-04");
    assert_eq!(b.rules.tags.last().unwrap().name.as_ref(), "TEST_TAG");
    assert_eq!(
        b.rules.melee.heavy_class.len(),
        base.rules.melee.heavy_class.len() + 1
    );
    assert_eq!(
        b.modular.grips.last().unwrap().unique_name.as_str(),
        "wfd:test/grip"
    );
    assert_eq!(names.len(), base.names.0["en"].len() + 1);
    // Rule 5: the coverage counts the extension's items.
    assert_eq!(b.manifest.coverage, coverage_of(&b.items));
    // Collisions are errors: an existing constant, a scalar of a struct, an existing name, a fixed-size table, a
    // list that is not a table, and a second row for an existing key of every kind of keyed list.
    let row = |v: serde_json::Value, f: &str, x: serde_json::Value| {
        let mut v = v;
        v[f] = x;
        v
    };
    fn r<T: serde::Serialize>(v: &T) -> serde_json::Value {
        serde_json::to_value(v).unwrap()
    }
    let conflicts = [
        serde_json::json!({ "rules": { "game_constants": { "E-5.4.steel_path_mult": 3.0 } } }),
        serde_json::json!({ "rules": { "riven_formula": { "disposition": 1.0 } } }),
        serde_json::json!({ "names": { "wfd:test/grip": "Again" } }),
        serde_json::json!({ "rules": { "status": [r(&base.rules.status[0])] } }),
        serde_json::json!({ "rules": { "co_default": { "other": { "flags": ["direct_target_only"] } } } }),
        serde_json::json!({ "rules": [] }),
        // A second descriptor for T-01 (same id, other field).
        serde_json::json!({ "rules": { "toggles": [row(r(&base.rules.toggles[0]), "field", "toggles.x".into())] } }),
        // A second 9-A row for the class of the first one.
        serde_json::json!({ "rules": { "melee": { "heavy_class": [row(r(&base.rules.melee.heavy_class[0]), "source", "x".into())] } } }),
        serde_json::json!({ "rules": { "melee": { "stance_co": [row(r(&base.rules.melee.stance_co[0]), "source", "x".into())] } } }),
        serde_json::json!({ "rules": { "mult_groups": [row(r(&base.rules.mult_groups[0]), "stage", "G".into())] } }),
        serde_json::json!({ "rules": { "special_dr": [r(&base.rules.special_dr[0])] } }),
        serde_json::json!({ "rules": { "riven_traits": [r(&base.rules.riven_traits[0])] } }),
        serde_json::json!({ "rules": { "slot_layouts": [r(&base.rules.slot_layouts[0])] } }),
        serde_json::json!({ "rules": { "scaling": [r(&base.rules.scaling[0])] } }),
        serde_json::json!({ "rules": { "riven_formula": { "layouts": [r(&base.rules.riven_formula.layouts[0])] } } }),
        serde_json::json!({ "rules": { "helminth": { "restricted": [r(&base.rules.helminth.restricted[0])] } } }),
        serde_json::json!({ "rules": { "body_parts": { "faction_heads": [r(&base.rules.body_parts.faction_heads[0])] } } }),
        // A new id but an existing slug / name.
        serde_json::json!({ "rules": { "factions": [row(r(&base.rules.factions[0]), "id", base.rules.factions.len().into())] } }),
        serde_json::json!({ "rules": { "tags": [{ "id": base.rules.tags.len() + 1, "name": &*base.rules.tags[0].name }] } }),
        serde_json::json!({ "rules": { "categories": [row(r(&base.rules.categories[0]), "bit", base.rules.categories.len().into())] } }),
        serde_json::json!({ "modular": { "grips": [r(&base.modular.grips[0])] } }),
        serde_json::json!({ "modular": { "chambers": [r(&base.modular.chambers[0])] } }),
    ];
    for c in conflicts {
        let mut b2 = b.clone();
        let mut n2 = names.clone();
        let ext: Extra = serde_json::from_value(c.clone()).unwrap();
        assert!(apply_extra(&mut b2, &mut n2, ext).is_err(), "{c} merged");
    }
    // Every keyed list of LIST_KEYS exists in the readable form (a renamed field would silently skip the check).
    for (path, _) in LIST_KEYS {
        let (what, rest) = path.split_once('.').unwrap();
        let root = if what == "rules" {
            serde_json::to_value(&base.rules).unwrap()
        } else {
            serde_json::to_value(&base.modular).unwrap()
        };
        assert!(
            rest.split('.')
                .try_fold(&root, |v, k| v.get(k))
                .is_some_and(|v| v.is_array()),
            "{path}"
        );
    }
    // An unknown top-level key is rejected (deny_unknown_fields).
    assert!(serde_json::from_value::<Extra>(serde_json::json!({ "weapon": [] })).is_err());
}

/// `Manifest.coverage` computed from items: count per `ItemKind::slug()` per `EffectStatus`.
fn coverage_of(items: &[Item]) -> Coverage {
    let mut c = Coverage::default();
    for it in items {
        *c.0.entry(it.kind.slug().into())
            .or_default()
            .entry(it.status)
            .or_default() += 1;
    }
    c
}

#[test]
fn manifest_coverage_counts_the_items() {
    // bundle.rs `Manifest.coverage`: "Counts per EffectStatus per item kind" (data/fixtures/tools/coverage.py
    // writes it).
    let b = load_mini();
    assert_eq!(
        b.manifest.coverage,
        coverage_of(&b.items),
        "manifest.json coverage differs from the item files (python3 -I data/fixtures/tools/coverage.py --write)"
    );
    // The fixture follows the spec commit it names (README.md "Tools").
    let spec = b
        .manifest
        .sources
        .iter()
        .find(|s| &*s.source == "spec")
        .expect("spec source stamp");
    assert_eq!(spec.revision, b.manifest.spec_rev);
}

/// Every effect of the bundle in its readable JSON form (items, set bonuses, weapon and attack passives, zoom
/// levels).
fn effect_values(b: &Bundle) -> Vec<serde_json::Value> {
    let mut all: Vec<&EffectDef> = Vec::new();
    for x in &b.items {
        all.extend(&x.effects);
    }
    for s in &b.sets {
        all.extend(&s.bonus);
    }
    for w in &b.weapons {
        all.extend(&w.passives);
        for a in &w.attacks {
            all.extend(&a.passives);
        }
        for z in &w.zoom {
            all.extend(&z.effects);
        }
    }
    all.into_iter()
        .map(|e| serde_json::to_value(e).unwrap())
        .collect()
}

/// `true` when some object inside `vs` has `key` with a value that satisfies `pred`.
fn any_key(vs: &[serde_json::Value], key: &str, pred: &dyn Fn(&serde_json::Value) -> bool) -> bool {
    fn walk(v: &serde_json::Value, key: &str, pred: &dyn Fn(&serde_json::Value) -> bool) -> bool {
        match v {
            serde_json::Value::Object(m) => {
                m.get(key).is_some_and(pred) || m.values().any(|x| walk(x, key, pred))
            }
            serde_json::Value::Array(a) => a.iter().any(|x| walk(x, key, pred)),
            _ => false,
        }
    }
    vs.iter().any(|v| walk(v, key, pred))
}

/// How the coverage test checks one backticked token of the §17 Encoding column: `Ok(true)` when the fixture
/// exercises it, `Err(reason)` for a token that is not fixture data (engine functions, model tables) or has no sourced
/// catalogue user; `None` for an unknown token, so a new §17 encoding fails until this table covers it.
fn encoding_check(tok: &str, b: &Bundle, sc: &[CalcInput]) -> Option<Result<bool, &'static str>> {
    use serde_json::Value;
    let e = effect_values(b);
    let stat_is = |s: &str| any_key(&e, "stat", &|v| v.as_str() == Some(s));
    let stat_prefix = |s: &str| {
        any_key(&e, "stat", &|v| {
            v.as_str().is_some_and(|x| x.starts_with(s))
        })
    };
    let has = |k: &str| any_key(&e, k, &|v| !v.is_null());
    let eq = |k: &str, s: &str| any_key(&e, k, &|v| v.as_str() == Some(s));
    let per_has = |pred: &dyn Fn(&Value) -> bool| {
        any_key(&e, "per", &|v| {
            v.as_array().is_some_and(|a| a.iter().any(pred))
        })
    };
    let attacks = || b.weapons.iter().flat_map(|w| &w.attacks);
    let group_used = |stage: MultStage| {
        b.rules
            .mult_groups
            .iter()
            .any(|g| g.stage == stage && eq("mult", &g.id))
    };
    let r = match tok {
        "stat = \"base_dmg\"" => stat_is("base_dmg"),
        "phys.*" => stat_prefix("phys."),
        "cc_*" => ["cc_rel", "cc_abs", "cc_floor", "cc_base_add", "cc_final_mul"]
            .iter()
            .all(|s| stat_is(s)),
        "cd_*" => ["cd_rel", "cd_abs", "cd_base_add", "cd_base_cond", "cd_base_cond_thr", "cd_field_mul"]
            .iter()
            .all(|s| stat_is(s)),
        "sc_*" => ["sc_rel", "sc_abs", "sc_base_add", "sc_mul"]
            .iter()
            .all(|s| stat_is(s)),
        "co_*" => ["co_per_status", "co_per_cold_stack", "co_flat"]
            .iter()
            .all(|s| stat_is(s)),
        "forced.*" => attacks().any(|a| !a.forced.is_empty()) || stat_prefix("forced."),
        "status_dmg_type.*" => stat_prefix("status_dmg_type."),
        "stack_cap.*" => stat_prefix("stack_cap."),
        "magazine(_flat)" => stat_is("magazine") && stat_is("magazine_flat"),
        "base_dmg_universal" | "incarnon_flat" | "co_per_cold_stack" | "co_flat" | "wp_dmg" | "hs_mult"
        | "tier_raise" | "multishot" | "lock_multishot" | "status_dmg" | "status_dur" | "dot_vuln" | "vuln_add"
        | "strip_ability" | "strip_aura" | "fire_rate" | "lock_fire_rate" | "charge_rate" | "reload_speed"
        | "ammo_eff" | "attack_speed" | "initial_combo" | "combo_dur" | "heavy_eff" | "wind_up" | "heat_inherit"
        | "phoenix_talons" | "heat_inherit_faction" | "doughty_rank" => stat_is(tok),
        "per_event = \"any_input\"" => eq("per_event", "any_input"),
        // The gate of 05 §13.1 needs a gated and an ungated target debuff (a `vuln_add`, `strip_ability` or
        // `target_vuln` group): Molecular Prime and Terrify against Magus Accelerant.
        "ability_source" => {
            let debuff = |x: &Item| {
                let v: Vec<serde_json::Value> = x.effects.iter().map(|e| serde_json::to_value(e).unwrap()).collect();
                any_key(&v, "stat", &|s| s == "vuln_add" || s == "strip_ability")
                    || b.rules
                        .mult_groups
                        .iter()
                        .any(|g| g.stage == MultStage::TargetVuln && any_key(&v, "mult", &|m| m == &*g.id))
            };
            let source = |want: bool| {
                b.items.iter().any(|x| {
                    matches!(x.kind, ItemKind::Buff { ability_source, .. } if ability_source == want) && debuff(x)
                })
            };
            source(true) && source(false)
        }
        "Frame.passives" => b.frames.iter().any(|f| !f.passives.is_empty()),
        "element" | "faction" | "mult" | "vuln" | "stacks" => has(tok),
        "ElemOrder" => ["slot", "innate", "non_combining", "warframe_arcane"]
            .iter()
            .all(|o| eq("order", o)),
        "order = \"non_combining\"" => eq("order", "non_combining"),
        "element = \"target_weakness\"" => eq("element", "target_weakness"),
        "mult = \"slam\"" => eq("mult", "slam"),
        "\"finisher\"" => eq("mult", "finisher"),
        "health_below" | "spot" => has(tok),
        "EffectDef.spot" => has("spot"),
        "Cond::Shot" => has("shot"),
        "Cond::Attack" => {
            return Some(Err(
                "no catalogue user in the fixture: Prelude of Might is the cd_base_cond rows since contract review \
                 ENG-R7, and the role filter's user, Pain Points, needs the Athodai (T21); domain_examples.rs \
                 covers both predicate forms",
            ));
        }
        "scope.form" => any_key(&e, "scope", &|s| s.get("form").is_some()),
        "per = combo_tier_above_one" => per_has(&|p| p == "combo_tier_above_one"),
        "Per::Strength" => per_has(&|p| p == "strength"),
        "Per::ReloadedFraction" => per_has(&|p| p == "reloaded_fraction"),
        "Per::Resolved" => per_has(&|p| p.get("resolved").is_some()),
        "Per::Resolved { Pow }" => per_has(&|p| p.pointer("/resolved/curve/pow").is_some()),
        "TriggerId::OnPlainHit" => eq("trigger", "on_plain_hit"),
        "TriggerId::CountWithin" => has("count_within"),
        "while_aiming" => eq("trigger", "while_aiming"),
        "proc_on_event" | "extra_hit" | "repeat_hit" | "random_multiplier" | "counter_bonus" => eq("mechanic", tok),
        "convert_damage { to = param }" => any_key(&e, "params", &|p| p.pointer("/to/param").is_some()),
        "Spot" => group_used(MultStage::Spot),
        "TargetVuln" => group_used(MultStage::TargetVuln),
        "ScVuln" => group_used(MultStage::ScVuln),
        "OverguardOnly" => group_used(MultStage::OverguardOnly),
        "DotOnly" => {
            return Some(Err(
                "no sourced catalogue member: V_sd is the `dot_vuln` slot (custom p.o, 03 §9.4) and the Mecha / \
                 Vauban second dips are `dot = \"twice\"` G groups (Mecha Empowered in the fixture)",
            ))
        }
        "scope.hit.outcome" => {
            return Some(Err(
                "R-2 construct without a catalogue user since D-410 (domain-model.md §11.1, §17); unit tests in \
                 effect.rs",
            ))
        }
        "assemble" => return Some(Err("engine function (kitgun and zaw assembly), not fixture data")),
        "Faction.bane_of" => b.rules.factions.iter().any(|f| !f.bane_of.is_empty()),
        "Attack.co" => [CoBehaviour::Adding, CoBehaviour::Multiplying, CoBehaviour::None]
            .iter()
            .all(|k| attacks().any(|a| a.co.behaviour == *k)),
        "Weapon.progenitor" => b.weapons.iter().any(|w| w.progenitor.is_some()),
        "Weapon.exalted" => b.weapons.iter().any(|w| w.exalted.is_some()),
        "Weapon.zoom" => b.weapons.iter().any(|w| !w.zoom.is_empty()),
        "VariantDoc.progenitor" => sc.iter().any(|c| c.build.variant.progenitor.is_some()),
        "toggles.zoom_level" => sc.iter().any(|c| c.scenario.toggles.zoom_level.is_some()),
        "external.VIR" => sc.iter().any(|c| c.scenario.toggles.external_viral.is_some()),
        "MeleePlanSel" => sc.iter().any(|c| c.scenario.toggles.melee_plan != MeleePlanSel::Auto),
        "BuffPick.caster" => sc
            .iter()
            .any(|c| c.scenario.buffs.iter().any(|p| p.caster.is_some())),
        "BodyPart.tier_add" => b.enemies.iter().any(|en| en.parts.iter().any(|p| p.tier_add > 0)),
        "Attack.lingering" => attacks().any(|a| a.lingering.is_some()),
        "explosion_delay" => attacks().any(|a| a.explosion_delay.is_some()),
        "embed_delay" => attacks().any(|a| a.embed_delay.is_some()),
        "INCARNON_FORM" => attacks().any(|a| a.roles.contains(AttackRoles::INCARNON_FORM)),
        "IncarnonDef" => b.weapons.iter().any(|w| w.incarnon.is_some()),
        "ComboTable" => !b.combo_tables.is_empty(),
        "MeleeData.unique" => b.weapons.iter().any(|w| w.melee.as_ref().is_some_and(|m| m.unique.is_some())),
        "Strike" => b
            .combo_tables
            .iter()
            .flat_map(|t| &t.combos)
            .flat_map(|c| &c.strikes)
            .any(|s| s.phys.iter().any(|x| *x != 0.0)),
        // A synthesized heavy with σ ≠ 1 (TV-07-10) besides the σ = 1 ones.
        "HeavyDef" => b.weapons.iter().filter_map(|w| w.melee.as_ref()?.heavy.as_ref()).any(|h| {
            matches!(h.source, HeavySource::ClassTable { sigma } if sigma.iter().any(|s| *s != 1.0))
        }),
        // Count-based bonus (`per = set_piece`) and a member scale (Sacrificial).
        "SetDef" => {
            b.sets.iter().any(|s| s.member_scale.is_some())
                && b.sets.iter().flat_map(|s| &s.bonus).any(|e| {
                    serde_json::to_value(e).unwrap()["per"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|p| p == "set_piece"))
                })
        }
        "ParamSpec" => b.items.iter().any(|x| !x.params.is_empty()),
        "ModularTables" => !b.modular.chambers.is_empty() && !b.modular.strikes.is_empty(),
        "Toggles" | "TOGGLE_MAP" => {
            !b.rules.toggles.is_empty()
                && b.rules
                    .toggles
                    .iter()
                    .all(|d| TOGGLE_MAP.iter().any(|(id, _, _)| *id == &*d.id.0))
        }
        _ => return None,
    };
    Some(Ok(r))
}

#[test]
fn every_coverage_encoding_is_exercised() {
    // Each backticked token of the domain-model.md §17 Encoding column is checked against the fixture (open-items.md
    // §3 issue 7): naming one record per row is not enough.
    let b = load_mini();
    let sc: Vec<CalcInput> = scenarios().into_iter().map(|(_, c)| c).collect();
    let spec = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/architecture/domain-model.md"),
    )
    .unwrap();
    let section = &spec[spec.find("## 17. Coverage matrix").unwrap()..spec.find("## 18.").unwrap()];
    let mut missing = Vec::new();
    let mut gaps = Vec::new();
    let mut n = 0;
    for line in section.lines().filter(|l| {
        l.starts_with("| ") && !l.starts_with("| Spec mechanic") && !l.starts_with("|---")
    }) {
        let cells: Vec<&str> = line.split(" | ").collect();
        let row = cells[0].trim_start_matches("| ");
        for tok in cells[1].split('`').skip(1).step_by(2) {
            n += 1;
            match encoding_check(tok, &b, &sc) {
                None => missing.push(format!("{row}: no check for `{tok}`")),
                Some(Ok(true)) => {}
                Some(Ok(false)) => missing.push(format!("{row}: `{tok}` not in the fixture")),
                Some(Err(reason)) => gaps.push(format!("{tok}: {reason}")),
            }
        }
    }
    assert!(n > 80, "tokens: {n}");
    assert!(missing.is_empty(), "{missing:#?}");
    // The documented non-data tokens (COVERAGE.md lists them); a new one needs a decision, not a silent pass.
    // `Cond::Attack` joined them in contract review pass 3 (CONTRACTS.md, ENG-R7 and DC-10).
    assert_eq!(gaps.len(), 4, "{gaps:#?}");
}

#[test]
fn spec_encodings_of_the_review_fixes() {
    // open-items.md §3 and CONTRACTS.md T01d "Left for other owners": each record follows the spec, not the stale
    // architecture row it was first copied from.
    let b = load_mini();
    let item = |un: &str| {
        b.items
            .iter()
            .find(|x| x.unique_name.as_str() == un)
            .unwrap()
    };
    let eff = |un: &str| serde_json::to_value(&item(un).effects).unwrap();
    // Sniper zoom crit damage is ΣCD_rel (02 §4.3, EC-02-17).
    let rubico = b
        .weapons
        .iter()
        .find(|w| w.unique_name.as_str().ends_with("FiveShotSniper"))
        .unwrap();
    for z in &rubico.zoom {
        assert_eq!(
            z.effects[0].target,
            Target::Stat(StatKey::parse("cd_rel").unwrap())
        );
    }
    // Spiteful Defilement is ΣCD_abs (D-067 amended, 02 §4.3).
    assert_eq!(
        eff("wfd:perk/phenmor/spiteful_defilement")[0]["stat"],
        "cd_abs"
    );
    // Melee Doughty: the registry row `doughty_rank` holds b_r = 0.5 + 0.1·rank; the crit stage computes 02 E-2.5a
    // per instance (ruling C-37 of the contract review; it was a draft ΣCD_abs line before).
    let d = item("/Lotus/Upgrades/CosmeticEnhancers/Offensive/CritDamageForPunctureStatus");
    assert_eq!(d.status, EffectStatus::Verified);
    assert_eq!(d.effects.len(), 1);
    assert_eq!(
        d.effects[0].target,
        Target::Stat(StatKey::parse("doughty_rank").unwrap())
    );
    assert!(d.effects[0].per.is_empty());
    let br = eff(d.unique_name.as_str())[0]["value"].clone();
    assert_eq!(
        br,
        serde_json::json!({ "max": 1.0, "rank": { "table": [0.5, 0.6, 0.7, 0.8, 0.9, 1.0] } })
    );
    // Primary Deadhead: shared_lose_one (04 §4.2).
    let dh = &item("/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnNoMelee").effects[0];
    assert_eq!(
        dh.stacks.as_ref().unwrap().stack_timer,
        StackTimer::SharedLoseOne
    );
    // Magus Accelerant: a user count without a limit (ruling C-4).
    let ma = item("wfd:buff/magus_accelerant").effects[0]
        .stacks
        .clone()
        .unwrap();
    assert!(ma.unbounded);
    // Primary Plated Round: duration by rank 3 / 4 / 5 / 6 / 8 / 10 s (04 §5.4).
    let pr = item("/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamagePerAmmoOnReload")
        .effects[0]
        .stacks
        .clone()
        .unwrap();
    assert_eq!(
        pr.duration_rank,
        Some(RankCurve::Table(vec![3.0, 4.0, 5.0, 6.0, 8.0, 10.0]))
    );
    // Laetum Overwhelming Attrition: on_plain_hit stacks, +4.00 × 3, 10 s, shared_lose_one (04 §5.12, TV-04-47).
    let la = &item("wfd:perk/laetum/overwhelming_attrition").effects[0];
    assert_eq!(la.when, Cond::trigger(TriggerId::OnPlainHit)); // hit_unit = pellet
    assert_eq!(la.per.as_slice(), &[Per::Stack]);
    let st = la.stacks.as_ref().unwrap();
    assert_eq!(
        (st.max, st.duration, st.stack_timer),
        (3, 10.0, StackTimer::SharedLoseOne)
    );
    assert!(la.scope.hit.outcome.is_none());
    // Sonar is inherited by DoT (D-095); the slam group exists for Nira's set.
    let g = |id: &str| b.rules.mult_groups.iter().find(|g| &*g.id == id).unwrap();
    assert_eq!(g("sonar").dot, DotInheritance::Inherit);
    assert_eq!(g("slam").stage, MultStage::G);
    // Position 9 of 01 §4.10.1 for the weapon arcane and Incarnon perk slots; warframe slots have none.
    for l in &b.rules.slot_layouts {
        for s in &l.slots {
            let want = match s.id {
                SlotId::Arcane(_) | SlotId::Inc(_) => Some(9),
                SlotId::Gen(n) => Some(n),
                SlotId::Exilus => Some(8),
                _ => None,
            };
            assert_eq!(s.element_order, want, "{:?} {:?}", l.class, s.id);
        }
    }
    // Theorem Demulcent: a warframe-arcane element (TV-01-38).
    let td = &item("/Lotus/Upgrades/CosmeticEnhancers/Offensive/WeaponDamageOnResidualContact")
        .effects[0];
    assert_eq!(td.order, Some(ElemOrder::WarframeArcane));
    // Machete heavy (TV-07-10): σ_1 = 6/5, strikes 1.5, 1.5, 1.5 (Bleed), 1.5; heavy 2 one 500 % Knockdown.
    let m = b
        .weapons
        .iter()
        .find(|w| w.unique_name.as_str().ends_with("/Machete"))
        .unwrap();
    let h = m.melee.as_ref().unwrap().heavy.as_ref().unwrap();
    assert_eq!(h.source, HeavySource::ClassTable { sigma: [1.2, 1.0] });
    let hits: Vec<(f64, Vec<StatusType>)> = h.hits[0]
        .iter()
        .map(|s| (s.mult, s.procs.to_vec()))
        .collect();
    assert_eq!(
        hits,
        [
            (1.5, vec![]),
            (1.5, vec![]),
            (1.5, vec![StatusType::SLA]),
            (1.5, vec![])
        ]
    );
    assert_eq!(
        (
            h.hits[1].len(),
            h.hits[1][0].mult,
            h.hits[1][0].procs.to_vec()
        ),
        (1, 5.0, vec![StatusType::KND])
    );
    let table = &b.combo_tables[m.melee.as_ref().unwrap().stanceless.idx()];
    let direct: f64 = table
        .combos
        .iter()
        .find(|c| c.kind == ComboType::Heavy)
        .unwrap()
        .strikes
        .iter()
        .filter(|s| s.input == 0 && s.ty != StrikeType::Slam)
        .map(|s| s.mult)
        .sum();
    let total = b
        .rules
        .melee
        .heavy_class
        .iter()
        .find(|r| r.class == MeleeClass::Machete)
        .unwrap()
        .total[0]
        .unwrap();
    assert_eq!((direct, total, total / direct), (5.0, 6.0, 1.2));
}

/// A curated rule table of `data/curated/rules/` in its authoring type.
fn curated_rules<T: wfd_model::curated::CuratedFile>(name: &str) -> T {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/curated/rules")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    wfd_model::curated::parse_curated(Path::new(name), &text).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn rule_rows_follow_the_curated_tables() {
    // `rules.json` rows are copies of the curated rule tables of T03b (contract review T03b-R6, DC-23, DC-19): every
    // descriptor, group, layout, trait and constant the fixture carries equals its curated row (citations aside), so
    // tests built on the fixture see the real values.
    use wfd_model::curated::*;
    let b = load_mini();
    let r = &b.rules;
    let toggles = curated_rules::<TogglesFile>("toggles.toml").toggle;
    for t in &r.toggles {
        let real = toggles.iter().find(|x| x.id == t.id).unwrap();
        assert_eq!(t, real, "toggle {}", t.id);
    }
    let groups = curated_rules::<MultGroupsFile>("mult_groups.toml").group;
    for g in &r.mult_groups {
        let real = groups
            .iter()
            .find(|x| x.id == g.id)
            .unwrap_or_else(|| panic!("group {}", g.id));
        let strip = |g: &MultGroupDef| MultGroupDef {
            sources: Vec::new(),
            ..g.clone()
        };
        assert_eq!(strip(g), strip(real), "group {}", g.id);
    }
    let layouts = curated_rules::<SlotLayoutFile>("slot_layout.toml").layout;
    for l in &r.slot_layouts {
        let real = layouts.iter().find(|x| x.class == l.class).unwrap();
        assert_eq!(l.slots, real.slots, "layout {:?}", l.class);
    }
    // Body parts: the fixture rows are the curated rows restricted to the fixture's factions, plus the fallback row.
    let bp = curated_rules::<BodyPartsFile>("body_parts.toml").rules;
    assert_eq!(r.body_parts.head_wp_factor, bp.head_wp_factor);
    for row in &r.body_parts.faction_heads {
        let real = bp
            .faction_heads
            .iter()
            .find(|x| {
                if row.factions.is_empty() {
                    x.factions.is_empty()
                } else {
                    row.factions.iter().all(|f| x.factions.contains(f))
                }
            })
            .unwrap_or_else(|| panic!("{:?}", row.factions));
        assert_eq!(
            (row.head_mult, row.headcrit),
            (real.head_mult, real.headcrit),
            "{:?}",
            row.factions
        );
    }
    assert_eq!(
        r.body_parts
            .faction_heads
            .iter()
            .filter(|x| x.factions.is_empty())
            .count(),
        1,
        "the fallback row of every other faction"
    );
    let h = curated_rules::<HelminthFile>("helminth.toml").rules;
    assert_eq!(r.helminth.restricted, h.restricted);
    for host in &r.helminth.hosts {
        assert!(h.hosts.contains(host), "{host:?}");
        assert!(b.frames.iter().any(|f| f.unique_name == host.frame));
    }
    assert!(!r.helminth.hosts.is_empty());
    // Riven traits: every field but the splice recipes (the fixture has no recipe traits) and the label.
    let traits = curated_rules::<RivenTraitsFile>("riven_traits.toml").traits;
    for t in &r.riven_traits {
        let real = traits.iter().find(|x| x.id == t.id).unwrap();
        let strip = |t: &RivenTraitDef| RivenTraitDef {
            recipes: Vec::new(),
            label: None,
            ..t.clone()
        };
        assert_eq!(strip(t), strip(real), "riven trait {}", t.id);
    }
    let dr = curated_rules::<SpecialDrFile>("special_dr.toml").dr;
    for d in &r.special_dr {
        let real = dr.iter().find(|x| x.id == d.id).unwrap();
        let strip = |d: &SpecialDrDef| SpecialDrDef {
            sources: Vec::new(),
            ..d.clone()
        };
        assert_eq!(strip(d), strip(real), "special DR {}", d.id);
    }
    let constants: BTreeMap<Box<str>, f64> =
        curated_rules::<GameConstantsFile>("game_constants.toml")
            .constant
            .into_iter()
            .map(|c| (c.key, c.value))
            .collect();
    assert_eq!(r.game_constants, constants);
    // The curated rows satisfy the extension keys of LIST_KEYS (README.md rule 3), so the Rules T18 emits from them
    // merge by the same rule: e.g. the two `exalted_weapon` rows of the 9-E finisher table differ by `weapons`
    // (contract review DC-20). Only the tables whose authoring row is the bundle row are checked here.
    fn v<T: serde::Serialize>(x: &T) -> serde_json::Value {
        serde_json::to_value(x).unwrap()
    }
    let melee = |name: &str| -> serde_json::Value {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/curated/rules/melee")
            .join(name);
        let text = std::fs::read_to_string(&path).unwrap();
        let t: toml::Value = toml::from_str(&text).unwrap();
        serde_json::to_value(&t["row"]).unwrap()
    };
    let root = serde_json::json!({
        "mult_groups": v(&groups),
        "toggles": v(&toggles),
        "riven_traits": v(&traits),
        "slot_layouts": v(&layouts),
        "special_dr": v(&dr),
        "body_parts": { "faction_heads": v(&bp.faction_heads) },
        "helminth": { "restricted": v(&h.restricted), "hosts": v(&h.hosts) },
        "melee": {
            "heavy_class": melee("heavy_class.toml"),
            "attack_triggers": melee("attack_triggers.toml"),
            "stance_co": melee("stance_co.toml"),
            "finishers": melee("finishers.toml"),
        },
    });
    check_list_keys(&root, "rules").unwrap();
}

#[test]
fn contract_review_encodings() {
    // The fixture records of the whole-contract review, pass 1 (CONTRACTS.md "Contract review fixes (data)").
    let b = load_mini();
    let item = |un: &str| {
        b.items
            .iter()
            .find(|x| x.unique_name.as_str() == un)
            .unwrap_or_else(|| panic!("no item {un}"))
    };
    let json = |v: &EffectDef| serde_json::to_value(v).unwrap();
    // DC-1: Melee Animosity counts every light, slide, aerial or wall input (04 §5.4, 07 §7.5).
    let an = json(
        &item("/Lotus/Upgrades/CosmeticEnhancers/Offensive/HeavyCritOnRegularMelee").effects[0],
    );
    assert_eq!(an["params"]["per_event"], "any_input");
    // DC-22: Piercing Navigator's user count defaults to 0 (04 §5.3); "+3x" binds as n2 (domain-model.md §15.2).
    let pn = &item("wfd:buff/piercing_navigator").effects[0];
    assert_eq!(pn.stacks.as_ref().unwrap().default_count, Some(0));
    let cr = json(
        &item(
            "/Lotus/Upgrades/CosmeticEnhancers/Utility/AbilityStrengthAndCritDamageWhenInvisible",
        )
        .effects[0],
    );
    assert_eq!(
        (cr["value"]["bind"].clone(), cr["value"]["max"].clone()),
        ("n2".into(), 3.0.into())
    );
    // DC-14: Toxic Lash carries no status-duration term; Saryn's passive is the frame's (04 E-4.14, D-037).
    let tl = json(&item("wfd:buff/toxic_lash").effects[0]);
    assert!(
        tl["params"]
            .get("status_duration_add")
            .is_none_or(|v| v == 0.0),
        "{tl}"
    );
    let saryn = b
        .frames
        .iter()
        .find(|f| f.unique_name.as_str() == "/Lotus/Powersuits/Saryn/Saryn")
        .unwrap();
    assert_eq!(
        saryn.passives.iter().map(json).collect::<Vec<_>>(),
        [serde_json::json!({ "stat": "status_dur", "value": 0.25 })]
    );
    assert!(
        saryn
            .abilities
            .iter()
            .any(|a| a.self_buff == Some(item("wfd:buff/toxic_lash").id))
    );
    // DC-23 (e): a Helminth host frame (Rhino: Roar, 09 E-9.32 H3).
    let rhino = b
        .frames
        .iter()
        .find(|f| f.unique_name.as_str() == "/Lotus/Powersuits/Rhino/Rhino")
        .unwrap();
    assert_eq!(rhino.helminth_host, Some(item("wfd:buff/roar").id));
    // DC-16: the source kind of every buff (04 §4.2; §5.7 "Source kind"): abilities and augments true; arcanes,
    // auras, focus and companions false.
    for x in b.items.iter().filter(|x| x.kind.slug() == "buff") {
        let ItemKind::Buff { ability_source, .. } = &x.kind else {
            unreachable!()
        };
        let want = !matches!(
            x.unique_name.as_str(),
            "wfd:buff/flat_crit_chance"
                | "wfd:buff/magus_accelerant"
                | "wfd:buff/mandonel_field"
                | "wfd:buff/mecha_empowered"
                | "wfd:buff/phoenix_talons"
                | "wfd:buff/smeeta_charm_crit"
        );
        assert_eq!(*ability_source, want, "{}", x.unique_name);
    }
    // M-2: the custom stat heatInheritFaction (03 §10.3 F_inh), 27 custom stats in all (00 §6.3).
    let hf = item("wfd:custom/heatinheritfaction");
    assert_eq!(
        hf.effects[0].target,
        Target::Stat(StatKey::parse("heat_inherit_faction").unwrap())
    );
    assert_eq!(
        b.items
            .iter()
            .filter(|x| x.kind.slug() == "custom_stat")
            .count(),
        27
    );
    // C-38: Theorem Demulcent is four per-element values, default 0 (04 §5.5; 01 §4.10.1: 0 holds no position).
    let td = item("/Lotus/Upgrades/CosmeticEnhancers/Offensive/WeaponDamageOnResidualContact");
    assert_eq!(td.status, EffectStatus::Verified);
    let elems: Vec<serde_json::Value> = td
        .effects
        .iter()
        .map(|e| json(e)["element"].clone())
        .collect();
    assert_eq!(elems, ["HEA", "COL", "ELE", "TOX"]);
    for (e, p) in td.effects.iter().zip(&td.params) {
        assert_eq!(e.order, Some(ElemOrder::WarframeArcane));
        assert_eq!(e.value, ValueExpr::Param(p.key.clone()));
        assert_eq!(p.default, ParamValue::Num(0.0));
    }
    // T03b-R6: target-vulnerability groups never enter the DoT seed (05 §13).
    for g in b
        .rules
        .mult_groups
        .iter()
        .filter(|g| g.stage == MultStage::TargetVuln)
    {
        assert_eq!(g.dot, DotInheritance::None, "{}", g.id);
    }
    // API-12: the companion vintage trait of the fixture's riven traits (09 §12.3).
    let zoom = b
        .rules
        .riven_traits
        .iter()
        .find(|t| &*t.id == "zoom")
        .unwrap();
    assert!(zoom.vintage_companion);
    // T02c-R5: a fixture bundle names itself, so `xtask dist` can refuse to package it (AD-11).
    assert!(b.manifest.data_version.starts_with("fixture-"));
}

#[test]
fn contract_review_pass3_encodings() {
    // CONTRACTS.md "Contract review fixes (pass 3)": each record follows domain-model.md §16 and the spec.
    let b = load_mini();
    let item = |un: &str| {
        b.items
            .iter()
            .find(|x| x.unique_name.as_str() == un)
            .unwrap_or_else(|| panic!("{un}"))
    };
    let stat = |s: &str| Target::Stat(StatKey::parse(s).unwrap());
    // DC-11: Electric Shield's Electricity scales no DoT (04 §5.6, 03 §9.3; §16.13).
    let es = item("wfd:buff/electric_shield");
    assert_eq!(
        es.effects[0].target,
        Target::Element(ElemSel::Type(DamageType::ELE))
    );
    assert_eq!(es.effects[0].dot_scaling, Some(false));
    // P3-3: the same holds for every element line of every buffing field (a buff with a `cd_field_mul` line: Electric
    // Shield, the Mandonel field), whose elements 03 §9.3 excludes from E^dot.
    let fields: Vec<_> = b
        .items
        .iter()
        .filter(|x| matches!(x.kind, ItemKind::Buff { .. }))
        .filter(|x| x.effects.iter().any(|e| e.target == stat("cd_field_mul")))
        .collect();
    assert!(fields.len() >= 2, "{}", fields.len());
    for x in &fields {
        let lines: Vec<_> = x
            .effects
            .iter()
            .filter(|e| {
                matches!(e.target, Target::Element(_))
                    || matches!(e.target, Target::Stat(StatKey::ElemNc(_)))
            })
            .collect();
        assert!(!lines.is_empty(), "{}", x.unique_name.as_str());
        for e in lines {
            assert_eq!(e.dot_scaling, Some(false), "{}", x.unique_name.as_str());
        }
    }
    // DC-12: custom p.k is a target-side factor of M_sc,t (02 E-2.11, 04 §10): the `sc_vuln` group `custom_sc_vuln`;
    // the weapon-side `sc_mul` (M_sc,w) is the Mandonel field's ×2.
    let pk = item("wfd:custom/status_chance_mult");
    let Target::Mult(g) = &pk.effects[0].target else {
        panic!("{:?}", pk.effects[0].target)
    };
    let group = b.rules.mult_groups.iter().find(|x| x.id == *g).unwrap();
    assert_eq!(
        (&*group.id, group.stage),
        ("custom_sc_vuln", MultStage::ScVuln)
    );
    let mf = item("wfd:buff/mandonel_field");
    assert!(
        mf.effects
            .iter()
            .any(|e| e.target == stat("sc_mul") && e.value == ValueExpr::Const(1.0))
    );
    assert!(
        !b.items
            .iter()
            .any(|x| x.unique_name.as_str() != "wfd:buff/mandonel_field"
                && x.effects.iter().any(|e| e.target == stat("sc_mul")))
    );
    // ENG-R7, NEW-CR-8: Prelude of Might is CDM0c and θ_P (02 E-2.5, 04 §5.12), unconditional; CDM0+ is Critical
    // Parallel's unconditional line.
    let pm = item("wfd:perk/furis/prelude_of_might");
    assert_eq!(pm.effects.len(), 2);
    assert_eq!(
        (&pm.effects[0].target, &pm.effects[0].value),
        (&stat("cd_base_cond"), &ValueExpr::Const(3.0))
    );
    assert_eq!(
        (&pm.effects[1].target, &pm.effects[1].value),
        (&stat("cd_base_cond_thr"), &ValueExpr::Const(0.40))
    );
    assert!(pm.effects.iter().all(|e| e.when.is_always()));
    let cp = item("wfd:perk/braton/critical_parallel");
    assert!(
        cp.effects
            .iter()
            .any(|e| e.target == stat("cd_base_add") && e.when.is_always())
    );
    // N-8: abilityCombo's parameter is T-11's range, 0 = off.
    let ac = item("wfd:custom/abilitycombo");
    assert_eq!(
        ac.params[0].kind,
        ParamKind::Number {
            min: 0.0,
            max: 12.0,
            step: 1.0
        }
    );
}

trait WithEnemyKey {
    fn with_enemy_key(self, k: StableKey) -> Self;
}

impl WithEnemyKey for ScenarioDoc {
    fn with_enemy_key(mut self, k: StableKey) -> Self {
        self.enemy.enemy = k;
        self
    }
}

/// The item of the mini bundle with this uniqueName.
fn item_id(b: &Bundle, un: &str) -> ItemId {
    b.items
        .iter()
        .find(|i| i.unique_name.as_str() == un)
        .unwrap_or_else(|| panic!("no item {un}"))
        .id
}

/// A resolved build of `weapon` with every slot empty.
fn empty_build(weapon: WeaponId) -> Build {
    Build {
        weapon,
        item: WeaponItem {
            rank: 30,
            catalyst: true,
            capacity_override: None,
            polarities: BTreeMap::new(),
        },
        variant: Variant::default(),
        r#gen: Default::default(),
        exilus: Slot::Empty,
        stance: None,
        stance_rank: None,
        combo: ComboType::Neutral,
        arcanes: [None, None],
        genesis_installed: false,
        incarnon: [None; 4],
        frame: FrameLoadout::default(),
        companion_set_mods: Default::default(),
        companion_kind: CompanionKind::None,
    }
}

fn set_mod(item: ItemId) -> Slot {
    Slot::Mod {
        item,
        rank: 0,
        params: Default::default(),
    }
}

/// TV-09-09 (09 E-9.40, E-9.41) on the model's one set count (`wfd_model::sets::set_counts`, API-6 of the contract
/// review): distinct identities over warframe, active weapon and declared robotic-weapon pieces; the noncounting rule;
/// arcane slots hold no set mods.
#[test]
fn set_counts_follow_tv_09_09() {
    let mut b = load_mini();
    let v = |s: &str| format!("/Lotus/Upgrades/Mods/Sets/Vigilante/{s}");
    let armaments = item_id(&b, &v("PrimaryVigilanteArmamentsMod"));
    let mut build = empty_build(WeaponId(0));
    for (i, m) in [
        "PrimaryVigilanteArmamentsMod",
        "PrimaryVigilanteOffenseMod",
        "PrimaryVigilanteFervorMod",
        "PrimaryVigilanteSuppliesMod",
    ]
    .iter()
    .enumerate()
    {
        build.r#gen[i] = set_mod(item_id(&b, &v(m)));
    }
    build.frame.r#gen[0] = set_mod(item_id(&b, &v("WarframeVigilantePursuitMod")));
    build.frame.exilus = set_mod(item_id(&b, &v("WarframeVigilanteVigorMod")));
    build.companion_set_mods.push(armaments);
    build.companion_kind = CompanionKind::Sentinel;
    let vigilante = b.items[armaments.idx()].set.unwrap();
    let count = |b: &Bundle, build: &Build| -> Vec<(SetId, u8)> {
        wfd_model::sets::set_counts(b, build).to_vec()
    };
    assert_eq!(count(&b, &build), [(vigilante, 6)], "the primary");
    let mut no_robotic = build.clone();
    no_robotic.companion_set_mods.clear();
    assert_eq!(
        count(&b, &no_robotic),
        [(vigilante, 6)],
        "robotic piece removed"
    );
    let mut no_frame = build.clone();
    no_frame.frame = FrameLoadout::default();
    assert_eq!(
        count(&b, &no_frame),
        [(vigilante, 4)],
        "warframe mods removed"
    );

    // Other sets are counted separately and sorted by set id; an arcane slot holds no set mod.
    let gladiator = item_id(
        &b,
        "/Lotus/Upgrades/Mods/Sets/Gladiator/MeleeGladiatorMightMod",
    );
    build.exilus = set_mod(gladiator);
    build.arcanes[0] = Some((
        item_id(&b, "/Lotus/Upgrades/Mods/Sets/Nira/NiraMeleeMod"),
        5,
    ));
    let g = b.items[gladiator.idx()].set.unwrap();
    assert!(g < vigilante);
    assert_eq!(count(&b, &build), [(g, 1), (vigilante, 6)]);

    // `set_noncounting` (Mecha Empowered in the real data): a listed member never counts.
    let vigor = item_id(&b, &v("WarframeVigilanteVigorMod"));
    b.sets[vigilante.idx()].noncounting.push(vigor);
    assert_eq!(count(&b, &build), [(g, 1), (vigilante, 5)]);
    assert!(count(&b, &empty_build(WeaponId(0))).is_empty());
}
