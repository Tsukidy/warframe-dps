//! T03b acceptance (work-breakdown.md §4 T03b): the rule tables of `data/curated/rules/` that T03b owns load into the
//! authoring types; every toggle id of spec 00 §6.1 has a descriptor; every slot id of 09 §2.1 appears in a layout;
//! plus the cross-table checks that keep the tables consistent with each other and with the model.
//!
//! Run: `cargo test -p wfd-model --features curated --test rules_tables`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use wfd_model::curated::*;
use wfd_model::*;

fn curated_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/curated")
}

/// The tables T03b owns (work-breakdown.md §8; `game_constants.toml` named by the contract review, DC-19).
const OWNED: [&str; 12] = [
    "mult_groups.toml",
    "slot_layout.toml",
    "riven_traits.toml",
    "riven_formula.toml",
    "body_parts.toml",
    "special_dr.toml",
    "toggles.toml",
    "factions.toml",
    "categories.toml",
    "helminth.toml",
    "frames.toml",
    "game_constants.toml",
];

fn load<T: CuratedFile>(name: &str) -> T {
    let path = curated_root().join("rules").join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    parse_curated::<T>(Path::new(name), &text).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn every_owned_table_loads_into_the_authoring_types() {
    for name in OWNED {
        assert!(
            curated_root().join("rules").join(name).is_file(),
            "data/curated/rules/{name} is missing"
        );
    }
    // The whole tree, through the loader every consumer uses (pipeline.md §6.1).
    let tree = load_curated_dir(&curated_root()).unwrap_or_else(|e| panic!("{e}"));
    let r = &tree.rules;
    assert!(r.mult_groups.is_some());
    assert!(r.slot_layout.is_some());
    assert!(r.riven_traits.is_some());
    assert!(r.riven_formula.is_some());
    assert!(r.body_parts.is_some());
    assert!(r.special_dr.is_some());
    assert!(r.toggles.is_some());
    assert!(r.factions.is_some());
    assert!(r.categories.is_some());
    assert!(r.helminth.is_some());
    assert!(r.frames.is_some());
    assert!(r.game_constants.is_some());
}

/// Every file states its sources.
#[test]
fn every_file_cites_sources() {
    fn check(name: &str, sources: &[Box<str>]) {
        assert!(!sources.is_empty(), "{name}: file-level `sources` is empty");
    }
    check(
        "mult_groups",
        &load::<MultGroupsFile>("mult_groups.toml").sources,
    );
    check(
        "slot_layout",
        &load::<SlotLayoutFile>("slot_layout.toml").sources,
    );
    check(
        "riven_traits",
        &load::<RivenTraitsFile>("riven_traits.toml").sources,
    );
    check(
        "riven_formula",
        &load::<RivenFormulaFile>("riven_formula.toml").sources,
    );
    check(
        "body_parts",
        &load::<BodyPartsFile>("body_parts.toml").sources,
    );
    check(
        "special_dr",
        &load::<SpecialDrFile>("special_dr.toml").sources,
    );
    check("toggles", &load::<TogglesFile>("toggles.toml").sources);
    check("factions", &load::<FactionsFile>("factions.toml").sources);
    check(
        "categories",
        &load::<CategoriesFile>("categories.toml").sources,
    );
    check("helminth", &load::<HelminthFile>("helminth.toml").sources);
    check("frames", &load::<FramesFile>("frames.toml").sources);
    check(
        "game_constants",
        &load::<GameConstantsFile>("game_constants.toml").sources,
    );
}

// ---------------------------------------------------------------------------------------------------------------
// Toggles (00 §6.1, domain-model.md §9.3)
// ---------------------------------------------------------------------------------------------------------------

/// The descriptor `field` for a `TOGGLE_MAP` place (convention of rules/toggles.toml).
fn field_of(place: &TogglePlace) -> String {
    let prefixed = |prefix: &str, paths: &str| {
        paths
            .split('|')
            .map(|p| format!("{prefix}{p}"))
            .collect::<Vec<_>>()
            .join("|")
    };
    match place {
        TogglePlace::Scenario(p) => (*p).to_owned(),
        TogglePlace::Build(p) => prefixed("build.", p),
        TogglePlace::Buffs => "buffs".to_owned(),
        TogglePlace::OptRequest(p) => prefixed("opt.", p),
    }
}

/// The JSON value at a dotted path, `None` if absent (a path through a list is not followed).
fn at<'a>(v: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    path.split('.').try_fold(v, |v, k| v.get(k))
}

#[test]
fn every_toggle_of_spec_00_has_a_descriptor() {
    let f = load::<TogglesFile>("toggles.toml");
    let ids: Vec<&str> = f.toggle.iter().map(|t| &*t.id.0).collect();
    let expected: Vec<String> = (1..=43).map(|i| format!("T-{i:02}")).collect();
    assert_eq!(ids, expected, "T-01 … T-43 in order, each once (00 §6.1)");
    let map: BTreeMap<&str, &TogglePlace> = TOGGLE_MAP.iter().map(|(id, p, _)| (*id, p)).collect();
    for t in &f.toggle {
        let id = &*t.id.0;
        let place = map
            .get(id)
            .unwrap_or_else(|| panic!("{id} not in TOGGLE_MAP"));
        assert_eq!(&*t.field, field_of(place), "{id}: field");
        assert_eq!(
            t.label.as_str(),
            format!("toggle.{id}.label"),
            "{id}: label key"
        );
        assert_eq!(
            t.help.as_str(),
            format!("toggle.{id}.help"),
            "{id}: help key"
        );
        assert!(!t.group.0.is_empty(), "{id}: group");
        for o in &t.control.options {
            assert_eq!(
                o.label.as_str(),
                format!("toggle.{id}.opt.{}", o.value),
                "{id}: option key"
            );
        }
        if let (Some(lo), Some(hi)) = (t.control.min, t.control.max) {
            assert!(lo <= hi, "{id}: min > max");
        }
        if t.control.kind == ControlType::Bool {
            assert!(t.control.options.is_empty(), "{id}: a bool has no options");
        }
    }
}

/// A descriptor default is the value the default document holds at its field (rules/toggles.toml header).
#[test]
fn toggle_defaults_match_the_default_document() {
    let f = load::<TogglesFile>("toggles.toml");
    let doc =
        serde_json::to_value(ScenarioDoc::against(EnemySetup::new(StableKey(9), 30))).unwrap();
    let null = serde_json::Value::Null;
    for t in &f.toggle {
        let id = &*t.id.0;
        let place = TOGGLE_MAP.iter().find(|(i, _, _)| *i == id).unwrap().1;
        // The buff toggles are off by default (rules/toggles.toml header): null for a value (T-13 … T-18), false for
        // the T-19 bool (Phoenix Talons has no value param).
        if let TogglePlace::Buffs = place {
            match t.control.kind {
                ControlType::OptionalNumber => assert_eq!(t.default, null, "{id}: buff default"),
                ControlType::Bool => {
                    assert_eq!(t.default, serde_json::json!(false), "{id}: buff default")
                }
                k => panic!("{id}: buff toggle with control {k:?}"),
            }
            continue;
        }
        let TogglePlace::Scenario(paths) = place else {
            continue;
        };
        // T-23: the enemy is the user's pick (documented exception).
        if id == "T-23" {
            assert_eq!(t.default, null, "T-23 default");
            continue;
        }
        let paths: Vec<&str> = paths.split('|').collect();
        // T-37: the caster of a buff pick; absent from a document without buffs. Its default is the meaning of an
        // absent caster for the T-13 Roar buff (`default_caster = ally`, 00 §6.1 T-37).
        if paths == ["buffs.caster"] {
            assert_eq!(t.default, serde_json::json!("ally"), "{id}");
            continue;
        }
        if paths.len() == 1 {
            let want = at(&doc, paths[0]).unwrap_or(&null);
            assert_eq!(
                &t.default, want,
                "{id}: default vs ScenarioDoc at {}",
                paths[0]
            );
            continue;
        }
        let obj = t
            .default
            .as_object()
            .unwrap_or_else(|| panic!("{id}: a multi-path default is an object keyed by path"));
        for k in obj.keys() {
            assert!(
                paths.contains(&k.as_str()),
                "{id}: default key {k} is not one of its paths"
            );
        }
        for p in paths {
            let got = obj.get(p).unwrap_or(&null);
            match at(&doc, p) {
                Some(want) => assert_eq!(got, want, "{id}: default of {p}"),
                // Paths through `buffs` (params and caster of buff picks) are absent from a document without buffs.
                None => assert!(
                    p.starts_with("buffs."),
                    "{id}: {p} not in the default document"
                ),
            }
        }
    }
}

/// Generic controls write a value of their own shape at one path; structured values are `custom` (rules/toggles.toml
/// header, `ControlType`; CONTRACTS.md T03b review fixes).
#[test]
fn toggle_controls_match_their_values() {
    let f = load::<TogglesFile>("toggles.toml");
    let custom: Vec<&str> = f
        .toggle
        .iter()
        .filter(|t| t.control.kind == ControlType::Custom)
        .map(|t| &*t.id.0)
        .collect();
    // T-08 and T-09 are optional_number with the one-member-object convention (ControlType::OptionalNumber, ruling
    // T03b-R6); T-36's `opt.` field is written only by the optimizer page (T03b-R5).
    assert_eq!(
        custom,
        [
            "T-05", "T-22", "T-23", "T-24", "T-25", "T-26", "T-29", "T-30", "T-32", "T-33", "T-35",
            "T-36", "T-37", "T-38", "T-41", "T-43"
        ],
        "toggles with a structured value"
    );
    // Every `opt.` field is custom: calculator panels never write the optimizer request (server-api.md §5.6).
    for t in &f.toggle {
        if t.field.starts_with("opt.") {
            assert_eq!(t.control.kind, ControlType::Custom, "{}", &*t.id.0);
        }
    }
    // The one-member objects of the OptionalNumber convention: the model value at the field is null (off) or an
    // object with one numeric member, and `{}` is on with the member's model default.
    let toggles = Toggles {
        stealth: Some(Stealth { rank: Some(30) }),
        heavy: Some(Heavy { start_combo: 13 }),
        ..Toggles::default()
    };
    let v = serde_json::to_value(&toggles).unwrap();
    for (id, path, member) in [
        ("T-08", "stealth", "rank"),
        ("T-09", "heavy", "start_combo"),
    ] {
        let t = f.toggle.iter().find(|t| &*t.id.0 == id).unwrap();
        assert_eq!(t.control.kind, ControlType::OptionalNumber, "{id}");
        assert_eq!(&*t.field, format!("toggles.{path}"), "{id}");
        let obj = v[path].as_object().unwrap();
        assert_eq!(obj.len(), 1, "{id}: one member");
        assert!(obj[member].is_number(), "{id}: numeric member {member}");
    }
    assert_eq!(
        serde_json::from_value::<Heavy>(serde_json::json!({})).unwrap(),
        Heavy {
            start_combo: Heavy::DEFAULT_START_COMBO
        }
    );
    assert_eq!(
        serde_json::from_value::<Stealth>(serde_json::json!({})).unwrap(),
        Stealth { rank: None }
    );
    for t in &f.toggle {
        let id = &*t.id.0;
        let k = t.control.kind;
        if k == ControlType::Custom {
            continue;
        }
        assert!(
            !t.field.contains('|'),
            "{id}: a generic control has one path"
        );
        let d = &t.default;
        match k {
            ControlType::Bool => assert!(d.is_boolean(), "{id}: bool default {d}"),
            ControlType::Number | ControlType::Percent => {
                assert!(d.is_number(), "{id}: number default {d}")
            }
            ControlType::OptionalNumber => {
                assert!(d.is_number() || d.is_null(), "{id}: optional default {d}")
            }
            ControlType::Select => {
                assert!(!t.control.options.is_empty(), "{id}: a select has options");
                match d {
                    serde_json::Value::Null => {} // optional select: null = off (T-34)
                    serde_json::Value::String(v) => assert!(
                        t.control.options.iter().any(|o| *o.value == **v),
                        "{id}: default {v} is not an option"
                    ),
                    _ => panic!("{id}: select default {d}"),
                }
            }
            ControlType::Custom => unreachable!(),
        }
        if matches!(
            k,
            ControlType::Number | ControlType::Percent | ControlType::OptionalNumber
        ) {
            let (Some(lo), Some(hi)) = (t.control.min, t.control.max) else {
                panic!("{id}: a number control has bounds");
            };
            if let Some(x) = d.as_f64() {
                assert!(
                    (lo..=hi).contains(&x),
                    "{id}: default {x} outside {lo}..{hi}"
                );
            }
        }
    }
    // frontend.md §6 row 20 detail: Strength is a percent field (shows 100 %).
    let t20 = f.toggle.iter().find(|t| &*t.id.0 == "T-20").unwrap();
    assert_eq!(t20.control.kind, ControlType::Percent);
}

// ---------------------------------------------------------------------------------------------------------------
// Slot layouts (09 §2)
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn every_slot_id_of_09_appears_in_a_layout() {
    let f = load::<SlotLayoutFile>("slot_layout.toml");
    let mut seen = BTreeSet::new();
    let mut classes = BTreeSet::new();
    for l in &f.layout {
        assert!(classes.insert(l.class), "{:?}: two layouts", l.class);
        assert!(!l.source.is_empty(), "{:?}: no source", l.class);
        let mut ids = BTreeSet::new();
        for s in &l.slots {
            assert!(s.id.is_valid(), "{:?}: {}", l.class, s.id);
            assert!(ids.insert(s.id), "{:?}: {} twice", l.class, s.id);
            assert!(
                !s.accepts.is_empty(),
                "{:?}: {} accepts nothing",
                l.class,
                s.id
            );
            seen.insert(s.id);
        }
    }
    for id in SlotId::all() {
        assert!(seen.contains(&id), "slot {id} of 09 §2.1 is in no layout");
    }
    let all_classes = [
        EquipmentClass::Primary,
        EquipmentClass::Secondary,
        EquipmentClass::Melee,
        EquipmentClass::KitgunPrimary,
        EquipmentClass::KitgunSecondary,
        EquipmentClass::Zaw,
        EquipmentClass::ExaltedRanged,
        EquipmentClass::ExaltedMelee,
        EquipmentClass::PseudoExaltedMelee,
        EquipmentClass::Necramech,
        EquipmentClass::Archgun,
        EquipmentClass::Archmelee,
        EquipmentClass::Robotic,
        EquipmentClass::Warframe,
    ];
    for c in all_classes {
        assert!(classes.contains(&c), "{c:?} has no layout");
    }
}

/// Element order of 01 §4.10.1 / E-1.8 (domain-model.md §13; CONTRACTS.md T01d review fix 2).
#[test]
fn slot_kinds_and_element_order() {
    let f = load::<SlotLayoutFile>("slot_layout.toml");
    for l in &f.layout {
        for s in &l.slots {
            let want = match s.id {
                SlotId::Gen(i) => Some(i),
                SlotId::Exilus => Some(8),
                SlotId::Arcane(_) | SlotId::Inc(_) => Some(9),
                _ => None,
            };
            assert_eq!(s.element_order, want, "{:?} {}", l.class, s.id);
            let kinds: Vec<SlotKind> = s.accepts.iter().copied().collect();
            match s.id {
                SlotId::Gen(_) | SlotId::FGen(_) => {
                    assert_eq!(kinds[0], SlotKind::General, "{:?} {}", l.class, s.id)
                }
                SlotId::Exilus | SlotId::FExilus => assert_eq!(kinds, [SlotKind::Exilus]),
                SlotId::Stance => assert_eq!(kinds, [SlotKind::Stance]),
                SlotId::Arcane(_) | SlotId::FArcane(_) => assert_eq!(kinds, [SlotKind::Arcane]),
                SlotId::Inc(_) => assert_eq!(kinds, [SlotKind::IncarnonPerk]),
                SlotId::Aura(_) => assert_eq!(kinds, [SlotKind::Aura]),
                SlotId::Shard(_) => assert_eq!(kinds, [SlotKind::Shard]),
                SlotId::Subsumed => assert_eq!(kinds, [SlotKind::Subsumed]),
            }
            if matches!(s.id, SlotId::FArcane(_)) {
                assert_eq!(s.arcane_family, Some(ArcaneFamily::Warframe));
            }
            if s.arcane_family.is_some() {
                assert!(matches!(s.id, SlotId::Arcane(_) | SlotId::FArcane(_)));
            }
            assert_eq!(
                s.id.is_warframe(),
                l.class == EquipmentClass::Warframe,
                "{}",
                s.id
            );
        }
        let riven = l.slots.iter().any(|s| s.accepts.contains(&SlotKind::Riven));
        // 09 E-9.50: no rivens on exalted (incl. pseudo-exalted and Necramech) weapons, archmelee or warframes.
        let no_riven = matches!(
            l.class,
            EquipmentClass::ExaltedRanged
                | EquipmentClass::ExaltedMelee
                | EquipmentClass::PseudoExaltedMelee
                | EquipmentClass::Necramech
                | EquipmentClass::Archmelee
                | EquipmentClass::Warframe
        );
        assert_eq!(riven, !no_riven, "{:?}: riven acceptance", l.class);
        let fixed_stance = l.slots.iter().any(|s| s.id == SlotId::Stance && s.fixed);
        assert_eq!(
            fixed_stance,
            matches!(
                l.class,
                EquipmentClass::ExaltedMelee | EquipmentClass::PseudoExaltedMelee
            ),
            "{:?}: stance_fixed (09 E-9.2)",
            l.class
        );
    }
}

/// 09 E-9.2 (n_gen, has_exilus, has_stance, stance_fixed), the slot-count table of 09 §2.2 and E-9.30 (arcane slots
/// and families), class by class.
#[test]
fn slot_counts_and_arcane_families_per_class() {
    use ArcaneFamily as F;
    use EquipmentClass as C;
    // (class, exilus, stance, families of arcane0, arcane1 …; `None` = the weapon's slot class, E-9.30 exalted row)
    type Row = (C, bool, bool, &'static [Option<F>]);
    let want: [Row; 13] = [
        (C::Primary, true, false, &[Some(F::Primary)]),
        (C::Secondary, true, false, &[Some(F::Secondary)]),
        (C::Melee, true, true, &[Some(F::Melee)]),
        (
            C::KitgunPrimary,
            true,
            false,
            &[Some(F::Primary), Some(F::Kitgun)],
        ),
        (
            C::KitgunSecondary,
            true,
            false,
            &[Some(F::Secondary), Some(F::Kitgun)],
        ),
        (C::Zaw, true, true, &[Some(F::Melee), Some(F::Zaw)]),
        (C::ExaltedRanged, true, false, &[None]),
        (C::ExaltedMelee, true, true, &[Some(F::Melee)]),
        (C::PseudoExaltedMelee, false, true, &[Some(F::Melee)]),
        (C::Necramech, false, false, &[]),
        (
            C::Archgun,
            false,
            false,
            &[Some(F::Primary), Some(F::Secondary)],
        ),
        (C::Archmelee, false, false, &[]),
        (C::Robotic, false, false, &[]),
    ];
    let f = load::<SlotLayoutFile>("slot_layout.toml");
    let layout = |c: C| f.layout.iter().find(|l| l.class == c).unwrap();
    for (c, exilus, stance, arcanes) in want {
        let l = layout(c);
        let gens: Vec<u8> = l
            .slots
            .iter()
            .filter_map(|s| match s.id {
                SlotId::Gen(i) => Some(i),
                _ => None,
            })
            .collect();
        assert_eq!(gens, (0..8).collect::<Vec<u8>>(), "{c:?}: n_gen = 8");
        let has = |id: SlotId| l.slots.iter().any(|s| s.id == id);
        assert_eq!(has(SlotId::Exilus), exilus, "{c:?}: has_exilus");
        assert_eq!(has(SlotId::Stance), stance, "{c:?}: has_stance");
        let got: Vec<(SlotId, Option<F>)> = l
            .slots
            .iter()
            .filter(|s| matches!(s.id, SlotId::Arcane(_)))
            .map(|s| (s.id, s.arcane_family))
            .collect();
        let want: Vec<(SlotId, Option<F>)> = arcanes
            .iter()
            .enumerate()
            .map(|(i, fam)| (SlotId::Arcane(i as u8), *fam))
            .collect();
        assert_eq!(got, want, "{c:?}: arcane slots and families (E-9.30)");
        assert!(
            l.slots.iter().all(|s| !s.id.is_warframe()),
            "{c:?}: warframe slot on a weapon"
        );
    }
    // Warframe: 8 general, 1 exilus, aura 1 (Jade 2: aura1 exists, frames.toml gates it), 2 warframe arcanes,
    // 5 shards, 1 subsumed ability (E-9.2).
    let w = layout(C::Warframe);
    let ids: Vec<SlotId> = w.slots.iter().map(|s| s.id).collect();
    let mut want: Vec<SlotId> = (0..8).map(SlotId::FGen).collect();
    want.extend([SlotId::FExilus, SlotId::Aura(0), SlotId::Aura(1)]);
    want.extend([SlotId::FArcane(0), SlotId::FArcane(1)]);
    want.extend((0..5).map(SlotId::Shard));
    want.push(SlotId::Subsumed);
    assert_eq!(ids, want, "warframe slots (09 §2.2)");
}

// ---------------------------------------------------------------------------------------------------------------
// Multiplier groups (04 §3, 05 E-5.19, domain-model.md §14)
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn multiplier_groups() {
    let f = load::<MultGroupsFile>("mult_groups.toml");
    let mut ids = BTreeMap::new();
    for g in &f.group {
        assert!(ids.insert(&*g.id, g).is_none(), "{}: twice", g.id);
        assert!(!g.sources.is_empty(), "{}: no sources", g.id);
        assert_eq!(
            g.vuln.is_some(),
            g.stage == MultStage::TargetVuln,
            "{}: `vuln` exactly on target_vuln groups",
            g.id
        );
        if g.stage == MultStage::TargetVuln {
            // V never enters the DoT seed (05 §13).
            assert_eq!(g.dot, DotInheritance::None, "{}", g.id);
        }
        if g.twice_on_two_bucket {
            assert_eq!(g.stage, MultStage::GF, "{}", g.id);
        }
    }
    // Group ids the effect DSL names in domain-model.md §16-17 and the mini fixture.
    for id in [
        "eclipse",
        "chamber",
        "synth_charge",
        "mecha",
        "careen",
        "slam",
        "finisher",
        "fortifier",
        "sonar",
        "detect_vulnerability",
        "molecular_prime",
        "custom_vuln",
        "custom_sc_vuln",
        "pageflight",
        "dante_passive",
    ] {
        assert!(ids.contains_key(id), "group {id} missing");
    }
    // M_sc,t of 02 E-2.11: custom p.k, Pageflight and Dante's passive are target-side factors, one group each, outside
    // the DoT seed (contract review DC-12).
    for id in ["custom_sc_vuln", "pageflight", "dante_passive"] {
        assert_eq!(
            (ids[id].stage, ids[id].dot),
            (MultStage::ScVuln, DotInheritance::None),
            "{id}"
        );
    }
    // Devouring Attrition is `random_multiplier`, whose factor the engine places in G; no group names it, so no
    // deterministic `mult` encoding of the ×21 exists (domain-model.md §16.13; contract review DC-25).
    assert!(!ids.contains_key("devouring_attrition"));
    // D-095: Sonar is inherited by DoT through Part_dot = Part (stage Spot, dot inherit).
    let sonar = ids["sonar"];
    assert_eq!(
        (sonar.stage, sonar.dot),
        (MultStage::Spot, DotInheritance::Inherit)
    );
    assert_eq!(ids["mecha"].dot, DotInheritance::Twice);
    assert_eq!(ids["eclipse"].stage, MultStage::GF);
    assert_eq!(ids["fortifier"].stage, MultStage::OverguardOnly);
    let all_layers: LayerSet = LayerSet::OG | LayerSet::SH | LayerSet::HP;
    assert_eq!(ids["custom_vuln"].vuln.as_ref().unwrap().layers, all_layers);
    assert_eq!(
        ids["molecular_prime"].vuln.as_ref().unwrap().layers,
        all_layers
    );
    assert_eq!(
        ids["strangledome"].vuln.as_ref().unwrap().layers,
        LayerSet::HP
    );
}

// ---------------------------------------------------------------------------------------------------------------
// Rivens (04 §8, 09 §12)
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn riven_formula_constants() {
    let f = load::<RivenFormulaFile>("riven_formula.toml").formula;
    assert_eq!(
        (f.scale, f.disposition, f.quality_min, f.quality_span),
        (10.0, 1.2, 0.9, 0.2)
    );
    assert_eq!((f.max_rank, f.default_drain_at_max), (8, 18));
    let l: Vec<(u8, u8, f64, Option<f64>)> = f
        .layouts
        .iter()
        .map(|r| (r.positives, r.negatives, r.l_pos, r.l_neg))
        .collect();
    assert_eq!(
        l,
        [
            (2, 0, 0.99, None),
            (2, 1, 1.2375, Some(-0.495)),
            (3, 0, 0.75, None),
            (3, 1, 0.9375, Some(-0.75)),
        ]
    );
    // 04 §8.2 worked check (TV-04-13): Rifle Critical Chance, 2+1, rank 8, u = 0 and u = 1, in binary32.
    let traits = load::<RivenTraitsFile>("riven_traits.toml");
    let cc = traits
        .traits
        .iter()
        .find(|t| &*t.id == "critical_chance")
        .unwrap();
    let b = cc.coeff[&RivenColumn::Rifle] as f32;
    let v = |u: f32| {
        let w = 0.9f32 + 0.2f32 * u;
        b * f.scale as f32 * 9.0f32 * f.disposition as f32 * 1.2375f32 * w
    };
    assert!((v(0.0) - 2.004_670_1).abs() < 2e-6, "{}", v(0.0));
    assert!((v(1.0) - 2.450_152_4).abs() < 2e-6, "{}", v(1.0));
}

#[test]
fn riven_traits() {
    let f = load::<RivenTraitsFile>("riven_traits.toml");
    let ids: BTreeSet<&str> = f.traits.iter().map(|t| &*t.id).collect();
    assert_eq!(ids.len(), f.traits.len(), "trait ids are unique");
    for t in &f.traits {
        assert!(!t.coeff.is_empty(), "{}: no coefficient column", t.id);
        if t.spliced {
            assert_eq!(
                t.sign,
                RivenSign::PositiveOnly,
                "{}: spliced traits are positive (09 E-9.55 T3)",
                t.id
            );
            assert!(!t.recipes.is_empty(), "{}: splice recipe (09 §12.3)", t.id);
        }
        for [a, b] in &t.recipes {
            assert!(
                ids.contains(&**a) && ids.contains(&**b),
                "{}: recipe {a} + {b}",
                t.id
            );
        }
        if let Some(tpl) = &t.template {
            assert!(
                matches!(tpl.value, ValueExpr::Param(_)),
                "{}: value enters as the param",
                t.id
            );
        }
    }
    // The inspiration's trait keys (09 §2.4) map to traits, one or more (Damage / Melee Damage, Fire Rate / Attack
    // Speed share `base` and `speed`).
    let keys: BTreeSet<&str> = f
        .traits
        .iter()
        .filter_map(|t| t.inspiration_key.as_deref())
        .collect();
    for k in [
        "crit_chance",
        "crit_mult",
        "multishot",
        "status_chance",
        "base",
        "speed",
        "Heat",
        "Cold",
        "Electricity",
        "Toxin",
        "Impact",
        "Puncture",
        "Slash",
        "addBlast",
        "addCorrosive",
        "addGas",
        "addMagnetic",
        "addRadiation",
        "addViral",
        "status_damage",
        "Corpus",
        "Grineer",
        "Infested",
        "Scaldra",
        "Techrot",
        "crit_chance_weakp",
        "mult_for_head",
        "ammoEff",
        "ammoCapacity",
        "magazineSize",
        "reloadTime",
        "shot_speed",
        "comboDuration",
        "initialCombo",
        "melee_combo_eff",
        "range",
        "punch_through",
        "zoom",
        "recoil",
        "status_duration",
    ] {
        assert!(
            keys.contains(k),
            "inspiration key {k} (09 §2.4) has no trait"
        );
    }
    // The domain-model.md §16.6 riven decodes against the table.
    for id in ["critical_damage", "multishot", "status_damage", "zoom"] {
        assert!(ids.contains(id), "{id}");
    }
    // 09 §12.3 (W:Riven_Mods L921-922): the companion vintage traits, which riven generation never produces on a
    // companion riven (contract review API-12); no spliced trait is flagged (09 §12.3 lists the traits only).
    let vintage: BTreeSet<&str> = f
        .traits
        .iter()
        .filter(|t| t.vintage_companion)
        .map(|t| &*t.id)
        .collect();
    assert_eq!(
        vintage,
        BTreeSet::from([
            "zoom",
            "recoil",
            "ammo_maximum",
            "combo_duration",
            "critical_chance_slide",
            "finisher_damage",
            "heavy_attack_efficiency",
            "initial_combo",
            "chance_to_gain_combo_count",
        ])
    );
    for t in f.traits.iter().filter(|t| t.vintage_companion) {
        assert!(!t.spliced, "{}", t.id);
    }
    // The whole 04 §8.3 table: columns Rifle, Shotgun, Pistol / Kitgun, Archgun, Melee / Zaw; `None` = cannot roll.
    // "Damage / Melee Damage" and "Fire Rate / Attack Speed" are two traits each (ranged and melee halves).
    const N: Option<f64> = None;
    let all = |b: f64| [Some(b); 5];
    let ranged = |b: f64| [Some(b), Some(b), Some(b), Some(b), N];
    let melee = |b: f64| [N, N, N, N, Some(b)];
    let row =
        |r: f64, s: f64, p: f64, a: f64, m: Option<f64>| [Some(r), Some(s), Some(p), Some(a), m];
    let want: Vec<(&str, [Option<f64>; 5])> = vec![
        (
            "damage",
            [Some(0.0183333), Some(0.0183), Some(0.0244), Some(0.0111), N],
        ),
        ("melee_damage", melee(0.0183)),
        (
            "critical_chance",
            row(0.016666, 0.01, 0.016666, 0.0111, Some(0.02)),
        ),
        (
            "critical_damage",
            row(0.013333, 0.01, 0.01, 0.0089, Some(0.01)),
        ),
        ("multishot", row(0.01, 0.0133, 0.0133, 0.0067, N)),
        ("fire_rate", row(0.00667, 0.01, 0.0083, 0.00667, N)),
        ("attack_speed", melee(0.0061)),
        ("status_chance", row(0.01, 0.01, 0.01, 0.0067, Some(0.01))),
        ("status_duration", all(0.01111)),
        ("heat", row(0.01, 0.01, 0.01, 0.0133, Some(0.01))),
        ("cold", row(0.01, 0.01, 0.01, 0.0133, Some(0.01))),
        ("electricity", row(0.01, 0.01, 0.01, 0.0133, Some(0.01))),
        ("toxin", row(0.01, 0.01, 0.01, 0.0133, Some(0.01))),
        (
            "impact",
            row(0.013333, 0.013333, 0.013333, 0.01, Some(0.0133)),
        ),
        (
            "slash",
            row(0.013333, 0.013333, 0.013333, 0.01, Some(0.0133)),
        ),
        (
            "puncture",
            row(0.01333, 0.01333, 0.01333, 0.01, Some(0.0133)),
        ),
        ("damage_to_corpus", all(0.005)),
        ("damage_to_grineer", all(0.005)),
        ("damage_to_infested", all(0.005)),
        (
            "magazine_capacity",
            row(0.005555, 0.005555, 0.005555, 0.0067, N),
        ),
        ("reload_speed", row(0.005555, 0.005555, 0.005555, 0.0111, N)),
        ("ammo_maximum", row(0.00555, 0.01, 0.01, 0.0111, N)),
        (
            "projectile_speed",
            [Some(0.01), Some(0.01), Some(0.01), N, N],
        ),
        ("punch_through", ranged(0.03)),
        ("recoil", ranged(-0.01)),
        ("zoom", row(0.006666, 0.004666, 0.0089, 0.006666, N)),
        ("range", melee(0.02158)),
        ("combo_duration", melee(0.09)),
        ("initial_combo", melee(0.27224)),
        ("additional_combo_count_chance", melee(0.00653)),
        ("chance_to_gain_combo_count", melee(-0.01165)),
        ("critical_chance_slide", melee(0.013334)),
        ("finisher_damage", melee(0.0133)),
        ("heavy_attack_efficiency", melee(0.00816)),
        // Spliced traits (04 §8.3 last paragraph).
        ("blast", all(0.01)),
        ("corrosive", all(0.01)),
        ("gas", all(0.01)),
        ("magnetic", all(0.01)),
        ("radiation", all(0.01)),
        ("viral", all(0.01)),
        ("damage_to_orokin", all(0.005)),
        ("damage_to_scaldra", all(0.005)),
        ("damage_to_techrot", all(0.005)),
        ("status_damage", all(0.01)),
        ("weak_point_damage", ranged(0.025)),
        ("weak_point_critical_chance", ranged(0.0275)),
        ("ammo_efficiency", ranged(-0.005)),
        (
            "reload_while_holstered",
            [Some(0.01), Some(0.01), Some(0.01), N, N],
        ),
        ("heavy_attack_wind_up", melee(0.0133)),
        ("melee_damage_on_heavy", melee(0.0133)),
        ("slam_attack_damage", melee(0.0133)),
        ("parry_angle", melee(0.009)),
    ];
    let cols = [
        RivenColumn::Rifle,
        RivenColumn::Shotgun,
        RivenColumn::Pistol,
        RivenColumn::Archgun,
        RivenColumn::Melee,
    ];
    assert_eq!(
        ids,
        want.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>(),
        "trait ids vs 04 §8.3"
    );
    for (id, b) in &want {
        let t = f.traits.iter().find(|t| &*t.id == *id).unwrap();
        for (col, want) in cols.iter().zip(b) {
            assert_eq!(t.coeff.get(col).copied(), *want, "{id} {col:?} (04 §8.3)");
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Factions, body parts, categories, Helminth, frames
// ---------------------------------------------------------------------------------------------------------------

fn faction_slugs() -> Vec<Box<str>> {
    load::<FactionsFile>("factions.toml")
        .faction
        .into_iter()
        .map(|f| f.slug)
        .collect()
}

#[test]
fn factions() {
    let f = load::<FactionsFile>("factions.toml");
    let slugs: Vec<&str> = f.faction.iter().map(|x| &*x.slug).collect();
    assert_eq!(
        slugs,
        [
            "grineer",
            "kuva_grineer",
            "corpus",
            "corpus_amalgam",
            "infested",
            "infested_deimos",
            "orokin",
            "sentient",
            "narmer",
            "murmur",
            "zariman",
            "scaldra",
            "techrot",
            "anarchs",
            "stalker",
            "tenno",
            "neutral"
        ],
        "the 17 factions of 00 §2.2"
    );
    let mut names = BTreeSet::new();
    for x in &f.faction {
        assert!(
            x.bane_of.contains(&x.slug),
            "{}: bane_of holds itself",
            x.slug
        );
        for b in &x.bane_of {
            assert!(slugs.contains(&&**b), "{}: bane_of {b}", x.slug);
        }
        assert!(
            [
                "grineer", "corpus", "infested", "orokin", "techrot", "default"
            ]
            .contains(&&*x.scaling),
            "{}: scaling {}",
            x.slug,
            x.scaling
        );
        for n in &x.wiki_names {
            assert!(
                names.insert(n.clone()),
                "data string {n:?} maps to two factions"
            );
        }
        assert!(!x.sources.is_empty(), "{}: sources", x.slug);
    }
    let bane = |s: &str| {
        f.faction
            .iter()
            .find(|x| &*x.slug == s)
            .unwrap()
            .bane_of
            .clone()
    };
    // D-008, 01 Table 4.15-B.
    assert!(bane("kuva_grineer").iter().any(|b| &**b == "grineer"));
    assert!(bane("corpus_amalgam").iter().any(|b| &**b == "corpus"));
    assert!(bane("infested_deimos").iter().any(|b| &**b == "infested"));
    assert!(!bane("orokin").iter().any(|b| &**b == "grineer"));
    assert!(!bane("techrot").iter().any(|b| &**b == "infested"));
    // Weakness by damage_faction (01 Table 4.10-A; none for Stalker, Tenno, Neutral) and scaling class (05 §6.1:
    // health class, the shield class follows from it).
    use DamageType as D;
    let want: [(&str, Option<DamageType>, &str); 17] = [
        ("grineer", Some(D::COR), "grineer"),
        ("kuva_grineer", Some(D::COR), "grineer"),
        ("corpus", Some(D::MAG), "corpus"),
        ("corpus_amalgam", Some(D::MAG), "corpus"),
        ("infested", Some(D::HEA), "infested"),
        ("infested_deimos", Some(D::GAS), "infested"),
        ("orokin", Some(D::VIR), "orokin"),
        ("sentient", Some(D::RAD), "default"),
        ("narmer", Some(D::TOX), "default"),
        ("murmur", Some(D::RAD), "default"),
        ("zariman", Some(D::COR), "default"),
        ("scaldra", Some(D::COR), "grineer"),
        ("techrot", Some(D::MAG), "techrot"),
        ("anarchs", Some(D::ELE), "orokin"),
        ("stalker", None, "default"),
        ("tenno", None, "default"),
        ("neutral", None, "default"),
    ];
    for (slug, weakness, scaling) in want {
        let x = f.faction.iter().find(|x| &*x.slug == slug).unwrap();
        assert_eq!(x.weakness, weakness, "{slug}: weakness (01 Table 4.10-A)");
        assert_eq!(&*x.scaling, scaling, "{slug}: scaling class (05 §6.1)");
    }
}

#[test]
fn body_part_defaults() {
    let r = load::<BodyPartsFile>("body_parts.toml").rules;
    assert_eq!(r.head_wp_factor, 1.5);
    let slugs = faction_slugs();
    let n = r.faction_heads.len();
    assert!(n >= 2);
    for (i, row) in r.faction_heads.iter().enumerate() {
        assert_eq!(
            row.factions.is_empty(),
            i == n - 1,
            "only the last row is the fallback"
        );
        for s in &row.factions {
            assert!(slugs.contains(s), "unknown faction {s}");
        }
        assert!(!row.source.is_empty());
    }
    let head = |s: &str| {
        let row = r
            .faction_heads
            .iter()
            .find(|x| x.factions.iter().any(|f| &**f == s) || x.factions.is_empty())
            .unwrap();
        (row.head_mult, row.headcrit)
    };
    assert_eq!(head("grineer"), (3.0, 2.0));
    assert_eq!(head("corpus"), (3.0, 1.0));
    assert_eq!(head("murmur"), (1.5, 1.0));
    assert_eq!(head("stalker"), (3.0, 1.0));
}

#[test]
fn categories_and_chains() {
    let f = load::<CategoriesFile>("categories.toml");
    let mut bits = BTreeSet::new();
    let mut names = BTreeSet::new();
    for c in &f.category {
        assert!(c.bit < 64, "{}: bit {}", c.name, c.bit);
        assert!(bits.insert(c.bit), "bit {} twice", c.bit);
        assert!(names.insert(&*c.name), "{} twice", c.name);
    }
    for row in &f.chain {
        assert!(!row.source.is_empty());
        for n in &row.chain {
            assert!(
                names.contains(&**n),
                "chain {}/{:?}: {n} is not a category",
                row.slot,
                row.class
            );
        }
    }
    for t in f.type_alias.values() {
        assert!(names.contains(&**t), "alias target {t} is not a category");
    }
    for c in [
        "Dagger",
        "Staff",
        "Two-Handed Nikana",
        "Heavy Scythe",
        "Claws",
    ] {
        assert!(f.stance_tag.contains_key(c), "stance tag of {c}");
    }
    // stance_tag maps melee classes (09 §3.2); the EC-09-21 renames are a table of their own.
    for (class, tag) in &f.stance_tag {
        assert!(
            class.chars().any(|ch| ch.is_lowercase()),
            "{class}: a stance_tag key is a melee class, not a tag"
        );
        assert!(tag.ends_with("_STANCE"), "{class}: {tag}");
    }
    let renames: Vec<(&str, &str)> = f
        .stance_tag_rename
        .iter()
        .map(|(a, b)| (&**a, &**b))
        .collect();
    assert_eq!(
        renames,
        [
            ("BAYONET", "BAYONET_STANCE"),
            ("SCYTHE_STANCE", "SCYTHES_STANCE")
        ],
        "09 §3.2, EC-09-21"
    );
    for (_, to) in &renames {
        assert!(
            f.stance_tag.values().any(|t| &**t == *to),
            "rename target {to} is a class tag"
        );
    }
}

#[test]
fn helminth_and_frames() {
    let h = load::<HelminthFile>("helminth.toml").rules;
    let r: BTreeSet<&str> = h.restricted.iter().map(|u| u.0.as_ref()).collect();
    assert_eq!(
        r,
        BTreeSet::from([
            "wfd:buff/eclipse",
            "wfd:buff/roar",
            "wfd:buff/xatas_whisper"
        ])
    );
    assert_eq!(h.hosts.len(), 9, "Host of 09 E-9.32 H3");
    let frames = load::<FramesFile>("frames.toml");
    let variants: BTreeMap<&str, Option<&str>> = frames
        .frame
        .iter()
        .map(|f| (f.id.0.as_ref(), f.variant_of.as_ref().map(|v| v.0.as_ref())))
        .collect();
    assert_eq!(variants.len(), frames.frame.len(), "one row per frame");
    for host in &h.hosts {
        assert!(
            host.frame.0.starts_with("/Lotus/Powersuits/"),
            "{}",
            host.frame.0
        );
        assert!(
            host.ability.0.starts_with("wfd:buff/"),
            "{}",
            host.ability.0
        );
        assert_eq!(
            variants.get(host.frame.0.as_ref()).copied().flatten(),
            None,
            "a host is a base frame"
        );
    }
    for f in &frames.frame {
        assert!(!f.sources.is_empty());
        if let Some(base) = &f.variant_of {
            assert_eq!(
                variants.get(base.0.as_ref()).copied().flatten(),
                None,
                "{}: variant of a variant",
                f.id.0
            );
        }
    }
    let jade = frames
        .frame
        .iter()
        .filter(|f| f.aura_slots.is_some())
        .collect::<Vec<_>>();
    assert_eq!(jade.len(), 1);
    assert_eq!(
        (jade[0].id.0.as_ref(), jade[0].aura_slots),
        ("/Lotus/Powersuits/Choir/Choir", Some(2))
    );
    // Every Host frame with a Prime has it linked (H6): Rhino Prime → Rhino.
    assert_eq!(
        variants
            .get("/Lotus/Powersuits/Rhino/RhinoPrime")
            .copied()
            .flatten(),
        Some("/Lotus/Powersuits/Rhino/Rhino")
    );
    // Frame passives (04 §5.11, 03 §5; contract review DC-14): Ash SD_SLA +0.50 and A_sd,SLA +0.25, Lavos SD +1.00 ×
    // Ability Duration, Saryn SD +0.25; each variant carries the list of its base frame (`Frame.passives` doc).
    let passives = |id: &str| -> Vec<serde_json::Value> {
        let f = frames
            .frame
            .iter()
            .find(|f| &*f.id.0 == id)
            .unwrap_or_else(|| panic!("{id}: no row"));
        f.passives
            .iter()
            .map(|e| serde_json::to_value(e).unwrap())
            .collect()
    };
    let ps = "/Lotus/Powersuits/";
    assert_eq!(
        passives(&format!("{ps}Saryn/Saryn")),
        [serde_json::json!({ "stat": "status_dur", "value": 0.25 })]
    );
    assert_eq!(
        passives(&format!("{ps}Ninja/Ninja")),
        [
            serde_json::json!({ "stat": "status_dur_type.SLA", "value": 0.5 }),
            serde_json::json!({ "stat": "status_dmg_type.SLA", "value": 0.25 })
        ]
    );
    assert_eq!(
        passives(&format!("{ps}Alchemist/Alchemist")),
        [serde_json::json!({ "stat": "status_dur", "value": 1.0, "per": ["ability_duration"] })]
    );
    let mut with_passives = 0;
    for f in &frames.frame {
        let own = passives(&f.id.0);
        with_passives += usize::from(!own.is_empty());
        match &f.variant_of {
            Some(base) if variants.contains_key(base.0.as_ref()) => {
                assert_eq!(
                    own,
                    passives(&base.0),
                    "{}: the passives of its base",
                    f.id.0
                )
            }
            Some(_) => assert!(own.is_empty(), "{}: base frame has no row", f.id.0),
            None => {}
        }
        for e in &f.passives {
            assert!(
                e.when.is_always(),
                "{}: a passive applies with the frame",
                f.id.0
            );
        }
    }
    assert_eq!(with_passives, 6, "Ash, Lavos, Saryn and their Primes");
}

// ---------------------------------------------------------------------------------------------------------------
// Special DR and enemy hand rows (05 §4.4-4.5, §15)
// ---------------------------------------------------------------------------------------------------------------

/// Wiki sub-modules of 05 §4.1 ids (`Module:Enemies/data/<module>`, the live ones).
const ENEMY_MODULES: [&str; 12] = [
    "anarchs",
    "corpus",
    "grineer",
    "infestation",
    "narmer",
    "orokin",
    "scaldra",
    "sentient",
    "stalker",
    "techrot",
    "themurmur",
    "unaffiliated",
];

#[test]
fn special_dr_rows() {
    let f = load::<SpecialDrFile>("special_dr.toml");
    let mut defs = BTreeSet::new();
    for d in &f.dr {
        assert!(defs.insert(&*d.id), "{}: twice", d.id);
        assert!(!d.sources.is_empty(), "{}: sources", d.id);
        let n =
            d.attenuation.is_some() as u8 + d.dps_dr.is_some() as u8 + d.adaptation.is_some() as u8;
        assert_eq!(n, 1, "{}: exactly one kind of special DR", d.id);
        if let Some(a) = &d.attenuation {
            assert!(
                !a.layers.contains(LayerSet::OG),
                "{}: attenuation layers ⊆ {{SH, HP}} (E-5.21)",
                d.id
            );
        }
    }
    // E-5.22 through the reader T07 uses (`SpecialDrDef::dps_multiplier`, the documented reading of `dps_dr`): the
    // points reproduce M_dps at the band edges, inside the bands and far beyond the last point.
    let orphix = f.dr.iter().find(|d| &*d.id == "orphix").unwrap();
    let e522 = |x: f64| match x {
        x if x <= 1000.0 => 1.0,
        x if x <= 2500.0 => 0.7 + 300.0 / x,
        x if x <= 5000.0 => 0.5 + 800.0 / x,
        x if x <= 10000.0 => 0.2 + 2300.0 / x,
        x => 0.02 + 4100.0 / x,
    };
    for x in [
        0.0, 1.0, 500.0, 1000.0, 1800.0, 2500.0, 4000.0, 5000.0, 7000.0, 10000.0, 15000.0, 20000.0,
        80000.0, 1e7,
    ] {
        let m = orphix.dps_multiplier(x).unwrap();
        assert!(
            (m - e522(x)).abs() < 1e-12,
            "M_dps({x}) = {m} ≠ {}",
            e522(x)
        );
        assert!(m > 0.0, "M_dps({x}) > 0");
    }
    assert_eq!(
        f.dr.iter()
            .find(|d| &*d.id == "hemocyte")
            .unwrap()
            .dps_multiplier(5000.0),
        None
    );
    // 05 §15.2: the documented MDPS fractions are stored (not applied).
    let mdps = |id: &str| {
        f.dr.iter()
            .find(|d| &*d.id == id)
            .and_then(|d| d.attenuation.as_ref())
            .and_then(|a| a.mdps_frac)
    };
    assert_eq!(
        (mdps("hemocyte"), mdps("eidolon"), mdps("lephantis")),
        (Some(0.20), Some(1.0), None)
    );
    // Rows: known ids, known definitions, and no value field set twice for one enemy, so rows merge order-free
    // (`flags` and `immune` are sets and unite; rules/special_dr.toml header).
    let mut set: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (i, row) in f.enemy.iter().enumerate() {
        assert!(!row.sources.is_empty(), "row {i}: sources");
        if let Some(id) = &row.special_dr {
            assert!(defs.contains(&**id), "row {i}: special_dr {id}");
        }
        let has_caps = row.cap_all.is_some() || !row.caps.is_empty();
        assert_eq!(
            has_caps,
            row.flags.contains(EnemyFlags::STATUS_CAPPED),
            "row {i}: status_capped flag"
        );
        let fields = [
            ("immune", row.immune.is_some()),
            ("caps", has_caps),
            ("innate_dr", !row.innate_dr.is_empty()),
            ("status_dr", row.status_dr.is_some()),
            ("type_res", !row.type_res.is_empty()),
            ("dr_class", row.dr_class.is_some()),
            ("dr_class_og", row.dr_class_og.is_some()),
            ("flags", !row.flags.is_empty()),
            ("special_dr", row.special_dr.is_some()),
            ("party_scaling", row.party_scaling.is_some()),
            ("overguard_context", row.overguard_context.is_some()),
            ("eximus_capable", row.eximus_capable.is_some()),
            ("eximus_og", row.eximus_og.is_some()),
            ("bolstered_og_factor", row.bolstered_og_factor.is_some()),
        ];
        assert!(fields.iter().any(|(_, s)| *s), "row {i} sets nothing");
        for e in &row.enemies {
            let (module, key) = e
                .split_once(':')
                .unwrap_or_else(|| panic!("{e}: not <module>:<key>"));
            assert!(ENEMY_MODULES.contains(&module), "{e}: module {module}");
            assert!(!key.is_empty(), "{e}");
            for (name, present) in fields {
                if present
                    && name != "flags"
                    && name != "immune"
                    && let Some(j) = set.insert((e, name), i)
                {
                    panic!("{e}: `{name}` set by rows {j} and {i}");
                }
            }
        }
        for v in row.innate_dr.values().chain(row.type_res.values()) {
            assert!((0.0..=1.0).contains(v), "row {i}: fraction {v}");
        }
    }
    // 05 §4.5 spot checks.
    let row_of = |id: &'static str| {
        f.enemy
            .iter()
            .filter(move |r| r.enemies.iter().any(|e| &**e == id))
    };
    let jug = row_of("infestation:Juggernaut")
        .find(|r| !r.innate_dr.is_empty())
        .unwrap();
    assert_eq!(jug.innate_dr[&Layer::HP], 0.90);
    assert_eq!(jug.dr_class.as_ref().unwrap().radial_or_melee, 0.50);
    let archon = row_of("narmer:Archon Boreal").next().unwrap();
    assert_eq!((archon.cap_all, archon.status_dr), (Some(4), Some(0.50)));
    assert!(row_of("corpus:Treasurer").any(|r| r.flags.contains(EnemyFlags::TOXIN_NULLIFY)));
    // 05 §4.1 `bolstered_og_factor` (E-5.7): both Techrot Babau records 0.25, no other row (contract review M-17).
    for babau in [
        "techrot:Techrot Babau",
        "techrot:Techrot Babau (Temporal Archimedea Survival)",
    ] {
        assert_eq!(
            row_of(babau)
                .filter_map(|r| r.bolstered_og_factor)
                .collect::<Vec<_>>(),
            [0.25],
            "{babau}"
        );
    }
    assert_eq!(
        f.enemy
            .iter()
            .filter(|r| r.bolstered_og_factor.is_some())
            .count(),
        1
    );
    // E-5.27: no Guardian Eximus aura on a Demolisher-family record (05 §4.1 group: every key starting with
    // `Demolisher ` or `Demolyst`, 22 records; contract review M-16), and on no other row.
    let guardian_immune: BTreeSet<&str> = f
        .enemy
        .iter()
        .filter(|r| r.flags.contains(EnemyFlags::GUARDIAN_AURA_IMMUNE))
        .flat_map(|r| r.enemies.iter().map(|e| &**e))
        .collect();
    assert_eq!(guardian_immune.len(), 22, "{guardian_immune:?}");
    for e in &guardian_immune {
        let key = e.split_once(':').unwrap().1;
        assert!(
            key.starts_with("Demolisher ") || key.starts_with("Demolyst"),
            "{e}"
        );
    }
    // 05 §4.5 last row (T03b-R3 as the spec round resolved it): Protea is the quest boss `corpus:Protea Specter`.
    assert!(
        row_of("corpus:Protea Specter")
            .any(|r| r.special_dr.as_deref() == Some("attenuation_unknown"))
    );
    // Data gaps (Kuva Liches, Sisters, Hounds, Technocyte Coda, Thralls, Skittergirl) attach to nothing.
    let gaps: Vec<_> = f.enemy.iter().filter(|r| r.enemies.is_empty()).collect();
    assert_eq!(gaps.len(), 6);
    // 05 §4.4 (updated 2026-11-27): Sisters, Hounds and Kuva Liches: every status 4, IMP 5.
    for g in gaps.iter().filter(|r| r.cap_all.is_some()) {
        assert_eq!(
            (g.cap_all, g.caps.get(&StatusType::IMP).copied()),
            (Some(4), Some(5))
        );
    }
    // Sister Hounds: 25 % SH/HP DR, Toxin absorbed, abilities 1/4 as long (05 §4.5 Hounds row, §13.1 flag).
    assert!(gaps.iter().any(|r| {
        r.flags
            .contains(EnemyFlags::TOXIN_BLOCKED | EnemyFlags::ABILITY_DURATION_QUARTER)
            && r.innate_dr.get(&Layer::SH) == Some(&0.25)
    }));
    // Adversaries (05 §4.1: Kuva Lich, Sister of Parvos, Technocyte Coda) keep U40 attenuation with unknown p
    // (05 §4.5 last row).
    let adversaries: Vec<_> = gaps
        .iter()
        .filter(|r| r.flags.contains(EnemyFlags::ADVERSARY))
        .collect();
    assert_eq!(adversaries.len(), 3);
    for r in adversaries {
        assert_eq!(
            r.special_dr.as_deref(),
            Some("attenuation_unknown"),
            "{:?}",
            r.sources
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Game constants (rules/game_constants.toml; contract review DC-19)
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn game_constants() {
    let f = load::<GameConstantsFile>("game_constants.toml");
    let mut map = BTreeMap::new();
    for c in &f.constant {
        // `"<equation id>.<name>"` (`Rules.game_constants` doc): an equation id of the spec, then a snake_case name.
        let (eq, name) = c
            .key
            .rsplit_once('.')
            .unwrap_or_else(|| panic!("{}: no name", c.key));
        assert!(
            eq.starts_with("E-")
                && eq[2..].split('.').count() == 2
                && eq[2..].split('.').all(|p| p.parse::<u32>().is_ok()),
            "{}: equation id",
            c.key
        );
        assert!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
            "{}: name",
            c.key
        );
        assert!(c.value.is_finite(), "{}", c.key);
        assert!(!c.source.is_empty(), "{}: source", c.key);
        assert!(map.insert(&*c.key, c.value).is_none(), "{}: twice", c.key);
    }
    // 05 E-5.4 (Steel Path ×2.5) and E-5.5: the points of k_exi reproduce the slopes of the spec's bands and the
    // constant 6 beyond d = 100.
    assert_eq!(map["E-5.4.steel_path_mult"], 2.5);
    let d = |i: usize| map[format!("E-5.5.k_exi_d{i}").as_str()];
    let k = |i: usize| map[format!("E-5.5.k_exi_k{i}").as_str()];
    let slopes = [0.025, 0.125, 2.0 / 15.0, 0.03];
    for (i, s) in slopes.iter().enumerate() {
        assert!(
            ((k(i + 1) - k(i)) / (d(i + 1) - d(i)) - s).abs() < 1e-12,
            "band {i}"
        );
    }
    assert_eq!((d(0), k(0), d(4), k(4)), (15.0, 1.0, 100.0, 6.0));
    assert_eq!(
        (
            map["E-5.5.c_exi_defended"],
            map["E-5.5.c_exi_undefended"],
            map["E-5.5.health_offset"],
            map["E-5.5.base_floor_mult"]
        ),
        (0.25, 0.375, 900.0, 1.1)
    );
}
