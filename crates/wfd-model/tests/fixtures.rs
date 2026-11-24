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

/// One `extra/<task>.json` file: records appended after the base files (README.md).
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
    };
    for p in json_files(&dir.join("extra")) {
        let x: Extra = read(&p);
        b.weapons.extend(x.weapons);
        b.items.extend(x.items);
        b.enemies.extend(x.enemies);
        b.sets.extend(x.sets);
        b.frames.extend(x.frames);
        b.combo_tables.extend(x.combo_tables);
        b.stances.extend(x.stances);
        names.extend(x.names);
        b.stable_keys.extend(x.stable_keys);
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
    // Records are sorted by uniqueName within each file (pipeline.md §8.1), so ids follow file order.
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
            for tier in &inc.tiers {
                for p in tier {
                    in_items(*p, "incarnon tier");
                    match &b.items[p.idx()].kind {
                        ItemKind::IncarnonPerk { family, .. } => assert_eq!(*family, inc.family),
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
    assert_eq!(
        (neutral.strikes[0].mult, neutral.strikes[0].phys),
        (3.0, [0.0, 0.0, 1.0])
    );
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
    assert!(r.weapons.len() >= 10);
    for (class, un) in &r.weapons {
        assert!(un.starts_with("/Lotus/"), "{class}: {un}");
    }
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
