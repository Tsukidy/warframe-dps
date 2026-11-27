//! Acceptance of T03c (work-breakdown.md): the melee hand tables of `data/curated/rules/melee/` (07 §4.4) load into
//! the authoring types; table 9-D has the 63 rows of `W:Condition_Overload_(Mechanic)` §Stances (07 §7.6); every
//! table 9-A class is a `MeleeClass` and the TV-07-10 inputs are in 9-A and 9-B; the 9-C, 9-E and combo-special rows
//! carry the values 07 §9.7, §9.11 and §6 state; every row cites its source line; the tables convert into the bundle
//! form `MeleeRules` and survive its JSON and postcard round trips.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use wfd_model::curated::{
    AttackTriggersFile, ComboSpecialsFile, CuratedMeleeRules, FinishersFile, HeavyClassFile,
    HeavyKappaFile, StanceCoFile, load_curated_files,
};
use wfd_model::*;

const FILES: [&str; 6] = [
    "heavy_class.toml",
    "heavy_kappa.toml",
    "attack_triggers.toml",
    "stance_co.toml",
    "finishers.toml",
    "combo_specials.toml",
];

fn curated_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/curated")
}

/// Loads the six files through the curated loader (paths relative to `data/curated`, so its path rules apply).
fn melee() -> CuratedMeleeRules {
    let root = curated_root();
    let files = FILES.iter().map(|f| {
        let rel = Path::new("rules/melee").join(f);
        let text = std::fs::read_to_string(root.join(&rel))
            .unwrap_or_else(|e| panic!("{}: {e}", rel.display()));
        (rel, text)
    });
    match load_curated_files(files) {
        Ok(tree) => tree.rules.melee,
        Err(e) => panic!("melee tables do not load:\n{e}"),
    }
}

fn heavy_class() -> HeavyClassFile {
    melee().heavy_class.expect("rules/melee/heavy_class.toml")
}
fn heavy_kappa() -> HeavyKappaFile {
    melee().heavy_kappa.expect("rules/melee/heavy_kappa.toml")
}
fn triggers() -> AttackTriggersFile {
    melee()
        .attack_triggers
        .expect("rules/melee/attack_triggers.toml")
}
fn stance_co() -> StanceCoFile {
    melee().stance_co.expect("rules/melee/stance_co.toml")
}
fn finishers() -> FinishersFile {
    melee().finishers.expect("rules/melee/finishers.toml")
}
fn specials() -> ComboSpecialsFile {
    melee()
        .combo_specials
        .expect("rules/melee/combo_specials.toml")
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Line numbers cited as `L<n>` (or `L<n>-<m>`) in a source string.
fn cited_lines(source: &str) -> Vec<u32> {
    let b = source.as_bytes();
    let mut out = Vec::new();
    for i in 0..b.len() {
        let starts_word = i == 0 || !b[i - 1].is_ascii_alphanumeric();
        if b[i] == b'L' && starts_word && b.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            let digits: String = source[i + 1..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            out.push(digits.parse().unwrap());
        }
    }
    out
}

fn assert_cites_line(table: &str, i: usize, source: &str) {
    assert!(
        !cited_lines(source).is_empty(),
        "{table} row {i}: source must cite its line (`L<n>`): {source:?}"
    );
}

fn weapon_names(ws: &[UniqueName]) -> Vec<&str> {
    ws.iter().map(UniqueName::as_str).collect()
}

// ---- all files ----

fn check_header(name: &str, schema: u32, sources: &[Box<str>], notes: &str) {
    assert_eq!(schema, wfd_model::curated::CURATED_SCHEMA, "{name}");
    assert!(!sources.is_empty(), "{name}: file-level sources");
    assert!(!notes.trim().is_empty(), "{name}: notes");
}

#[test]
fn every_file_loads_with_header_and_sources() {
    let m = melee();
    let a = m.heavy_class.expect("heavy_class.toml");
    check_header("heavy_class", a.schema, &a.sources, &a.notes);
    let b = m.heavy_kappa.expect("heavy_kappa.toml");
    check_header("heavy_kappa", b.schema, &b.sources, &b.notes);
    let c = m.attack_triggers.expect("attack_triggers.toml");
    check_header("attack_triggers", c.schema, &c.sources, &c.notes);
    let d = m.stance_co.expect("stance_co.toml");
    check_header("stance_co", d.schema, &d.sources, &d.notes);
    let e = m.finishers.expect("finishers.toml");
    check_header("finishers", e.schema, &e.sources, &e.notes);
    let f = m.combo_specials.expect("combo_specials.toml");
    check_header("combo_specials", f.schema, &f.sources, &f.notes);
}

#[test]
fn every_row_cites_its_source_line() {
    for (i, r) in heavy_class().row.iter().enumerate() {
        assert_cites_line("9-A", i, &r.source);
    }
    for (i, r) in heavy_kappa().row.iter().enumerate() {
        assert_cites_line("9-B", i, &r.source);
    }
    for (i, r) in triggers().row.iter().enumerate() {
        assert_cites_line("9-C", i, &r.source);
    }
    for (i, r) in stance_co().row.iter().enumerate() {
        assert_cites_line("9-D", i, &r.source);
    }
    for (i, r) in finishers().row.iter().enumerate() {
        assert_cites_line("9-E", i, &r.source);
    }
    for (i, r) in specials().row.iter().enumerate() {
        assert_cites_line("specials", i, &r.source);
    }
}

// ---- 9-A (07 §9.1) ----

#[test]
fn heavy_class_rows_are_the_classes_of_table_9a() {
    let f = heavy_class();
    // Every row's class is a `MeleeClass` by type; the set is the 07 table 9-A classes with values: every class of
    // the enum except Bayonet ("–", no row: sigma = 1), the data class "Exalted Weapon" and archmelee.
    let got: Vec<MeleeClass> = f.row.iter().map(|r| r.class).collect();
    let set: BTreeSet<MeleeClass> = got.iter().copied().collect();
    assert_eq!(set.len(), got.len(), "one row per class");
    let want: BTreeSet<MeleeClass> = MeleeClass::ALL
        .iter()
        .copied()
        .filter(|c| {
            !matches!(
                c,
                MeleeClass::Bayonet | MeleeClass::ExaltedWeapon | MeleeClass::ArchMelee
            )
        })
        .collect();
    assert_eq!(set, want);
    assert_eq!(f.row.len(), 27);
    for r in &f.row {
        // The class name round-trips through the wiki `Class` string (07 table 9-A column 1).
        assert_eq!(MeleeClass::from_wiki(r.class.wiki_name()), Some(r.class));
        let w = r
            .wind_up
            .unwrap_or_else(|| panic!("{:?}: wind-up", r.class));
        assert!((0.3..=1.2).contains(&w), "{:?}: wind-up {w}", r.class);
        for h in 0..2 {
            let t = r.total[h].unwrap_or_else(|| panic!("{:?}: total {h}", r.class));
            assert!(t > 0.0);
            let b = &r.breakdown[h];
            if !b.is_empty() {
                let sum: f64 = b.iter().map(|s| s.mult).sum();
                assert!(
                    close(sum, t),
                    "{:?} heavy {}: breakdown {sum} != total {t}",
                    r.class,
                    h + 1
                );
                assert!(b.len() > 1, "{:?}: a breakdown is a hit split", r.class);
                for s in b {
                    // 9-A breakdowns carry Bleed procs only (E-7.40 rule 2), crowd control in `cc`.
                    assert!(
                        s.procs.iter().all(|p| *p == StatusType::SLA),
                        "{:?}",
                        r.class
                    );
                    assert!(s.spawns.is_none());
                }
            }
        }
    }
}

#[test]
fn tv_07_10_inputs_are_in_9a_and_9b() {
    let f = heavy_class();
    let row = |c: MeleeClass| f.row.iter().find(|r| r.class == c).unwrap();
    // Machete heavy 1: table 6x with the split 150 % Bleed + 3 x 150 % (four strikes, like the stance data, so
    // sigma_1 = 6/5 = 1.2 scales the stance strikes); heavy 2: 5x.
    let m = row(MeleeClass::Machete);
    assert_eq!(m.total, [Some(6.0), Some(5.0)]);
    let mults: Vec<f64> = m.breakdown[0].iter().map(|s| s.mult).collect();
    assert_eq!(mults, [1.5, 1.5, 1.5, 1.5]);
    assert_eq!(m.breakdown[0][0].procs.as_slice(), [StatusType::SLA]);
    assert!(m.breakdown[0][1..].iter().all(|s| s.procs.is_empty()));
    assert!(m.breakdown[1].is_empty());
    // Sparring heavy 1: 12x (one direct strike scaled to 1200 %); heavy 2: 15x = 3 x 300 % + 2 x 300 % Bleed.
    let s = row(MeleeClass::Sparring);
    assert_eq!(s.total, [Some(12.0), Some(15.0)]);
    assert!(s.breakdown[0].is_empty());
    let h2: Vec<(f64, bool)> = s.breakdown[1]
        .iter()
        .map(|x| (x.mult, x.procs.as_slice() == [StatusType::SLA]))
        .collect();
    assert_eq!(
        h2,
        [
            (3.0, false),
            (3.0, false),
            (3.0, false),
            (3.0, true),
            (3.0, true)
        ]
    );
    // Hirudo kappa = 6.5 / 12 (Sparring): heavy 1 H = 12 x kappa = 6.5.
    let k = heavy_kappa();
    let hirudo = k
        .row
        .iter()
        .find(|r| {
            r.weapon.as_str()
                == "/Lotus/Weapons/Infested/Melee/InfWFAccompanyingSparring/InfestedKogake"
        })
        .expect("Hirudo in 9-B");
    assert!(close(hirudo.kappa, 6.5 / 12.0));
    assert!(close(12.0 * hirudo.kappa, 6.5));
    assert!(close(3.0 * hirudo.kappa, 1.625));
}

#[test]
fn heavy_class_spot_values() {
    let f = heavy_class();
    let row = |c: MeleeClass| f.row.iter().find(|r| r.class == c).unwrap();
    // Dagger heavy 1: 250 % Knockback, Bleed + 250 % (Knockback is crowd control, not an Impact proc).
    let d = row(MeleeClass::Dagger);
    assert_eq!(d.breakdown[0][0].procs.as_slice(), [StatusType::SLA]);
    assert_eq!(d.breakdown[0][0].cc.as_slice(), [CrowdControl::Knockback]);
    assert!(d.breakdown[0][1].procs.is_empty() && d.breakdown[0][1].cc.is_empty());
    // Assault Saw heavy 1: 3 x 100 % + 3 x 100 % Bleed.
    let a = row(MeleeClass::AssaultSaw);
    let bleeds = a.breakdown[0]
        .iter()
        .filter(|s| s.procs.contains(&StatusType::SLA))
        .count();
    assert_eq!((a.breakdown[0].len(), bleeds), (6, 3));
    assert_eq!(row(MeleeClass::DualNikanas).total, [Some(12.0), Some(6.0)]);
    assert_eq!(row(MeleeClass::HeavyScythe).total, [Some(6.0), Some(12.0)]);
    assert_eq!(row(MeleeClass::BladeAndWhip).total, [Some(4.0), Some(12.0)]);
    assert_eq!(row(MeleeClass::Rapier).total, [Some(4.5), Some(4.5)]);
    assert_eq!(row(MeleeClass::Glaive).total, [Some(2.0), Some(3.0)]);
    assert_eq!(row(MeleeClass::Hammer).wind_up, Some(1.2));
    assert_eq!(row(MeleeClass::Nikana).wind_up, Some(0.5));
}

// ---- 9-B (07 E-7.41) ----

#[test]
fn heavy_kappa_members_of_e_7_41() {
    let k = heavy_kappa();
    let got: BTreeMap<&str, f64> = k.row.iter().map(|r| (r.weapon.as_str(), r.kappa)).collect();
    assert_eq!(got.len(), k.row.len(), "one row per weapon");
    let want = [
        (
            "/Lotus/Weapons/Infested/Melee/InfWFAccompanyingSparring/InfestedKogake",
            6.5 / 12.0,
        ),
        (
            "/Lotus/Weapons/Infested/InfestedLich/Melee/CodaHirudo",
            6.502857 / 12.0,
        ),
        (
            "/Lotus/Weapons/Grineer/Melee/GrnSparring/GrnSpiderSparring/GrnSpiderSparring",
            10.699482 / 12.0,
        ),
        (
            "/Lotus/Weapons/Tenno/Melee/SwordsAndBoards/MortiforShieldAndSword/MortiforShieldAndSword",
            12.0 / 5.0,
        ),
    ];
    assert_eq!(got.len(), want.len());
    for (w, kappa) in want {
        let v = got.get(w).unwrap_or_else(|| panic!("{w} missing"));
        assert!((v - kappa).abs() < 1e-7, "{w}: {v} != {kappa}");
    }
}

// ---- 9-C (07 §9.7) ----

#[test]
fn attack_triggers_follow_table_9c() {
    let f = triggers();
    assert!(!f.row.is_empty());
    for (i, r) in f.row.iter().enumerate() {
        assert!(!r.weapons.is_empty(), "9-C row {i}: names its weapons");
        for w in &r.weapons {
            assert!(w.as_str().starts_with("/Lotus/"), "9-C row {i}: {w}");
        }
        assert!(!r.attack.trim().is_empty());
        assert!(
            r.attack_idx.is_none(),
            "resolved by the pipeline, not authored"
        );
        match &r.trigger {
            MeleeTriggerSource::Heavy { h } => {
                assert!(h.is_none_or(|h| h == 1 || h == 2), "9-C row {i}: h");
                // Heavy rows: x M_h and spend (07 §9.7 default), except the Exalted Blade heavy waves (no M_h, E-7.43).
                assert_eq!(
                    r.heavy_combo,
                    r.attack.as_ref() != HEAVY_WAVES,
                    "9-C row {i}"
                );
            }
            MeleeTriggerSource::ComboRanged { combo, input } => {
                assert!(input.is_none());
                assert!(
                    !matches!(combo, ComboType::Heavy | ComboType::HeavySlam),
                    "heavies use the heavy trigger"
                );
                assert!(!r.heavy_combo && !r.heavy_bonus, "9-C row {i}: light rows");
            }
            MeleeTriggerSource::Slide => assert!(!r.heavy_combo && !r.heavy_bonus),
        }
        if r.scaling == TriggerScaling::Wave {
            assert!(r.attack.as_ref() == LIGHT_WAVES || r.attack.as_ref() == HEAVY_WAVES);
        }
    }
    // Every weapon of 07 table 9-C with a default trigger is covered (PE uniqueNames).
    let covered: BTreeSet<&str> = f
        .row
        .iter()
        .flat_map(|r| r.weapons.iter().map(UniqueName::as_str))
        .collect();
    for w in [
        "/Lotus/Weapons/Tenno/Melee/Swords/DaxDuviriKatana/DaxDuviriKatanaWeapon", // Syam
        "/Lotus/Weapons/Infested/Melee/InfWarfan/InfWarfanWeapon",                 // Arum Spinosa
        "/Lotus/Weapons/Tenno/Melee/Warfan/TnBrokenFrameWarfan/TnBrokenFrameWarfanWeapon", // Quassus
        "/Lotus/Weapons/Tenno/Melee/Warfan/PrimeQuassus/PrimeQuassusWeapon", // Quassus Prime
        "/Lotus/Weapons/Tenno/Melee/SwordsAndBoards/DaxDuviriMaceShieldWeapon", // Argo & Vel
        "/Lotus/Weapons/Tenno/Melee/Polearms/DaxDuviriPolearm/DaxDuviriPolearmWeapon", // Edun
        "/Lotus/Weapons/Corpus/Melee/ShieldAndSword/CrpHammerShield/CrpHammerShield", // Tenet Agendus
        "/Lotus/Weapons/Corpus/Melee/CrpBriefcaseScythe/CrpBriefcaseScythe", // Tenet Grigori
        "/Lotus/Weapons/Infested/Melee/InfEmbolistScythe/InfestedScythe",    // Caustacyst
        "/Lotus/Weapons/Infested/InfestedLich/Melee/CodaCaustacyst/CodaCaustacyst", // Coda Caustacyst
        "/Lotus/Weapons/Tenno/Melee/SunDialAxe/SundialAxeWeapon",                   // Zenistar
        "/Lotus/Weapons/Tenno/Archwing/Melee/ExaltedArchScythe/ExaltedAWScytheWeapon", // Corufell
        "/Lotus/Weapons/Tenno/Archwing/Melee/PrimeCorufell/PrimeCorufellScytheWeapon", // Corufell Prime
        "/Lotus/Weapons/Archon/Melee/DualDaggers/ArchonDualDaggersPlayerWep",          // Nepheri
        "/Lotus/Weapons/Archon/Melee/Whip/ArchonWhipPlayerWep",                        // Verdilac
        "/Lotus/Weapons/Tenno/Melee/Swords/TnTwoHandedKatana/TnTwoHandedKatana",       // Tatsu
        "/Lotus/Weapons/Tenno/Melee/Swords/PrimeTatsu/PrimeTatsuWeapon", // Tatsu Prime
        "/Lotus/Weapons/Tenno/Melee/Gunblade/TnoGunblade",               // Redeemer
        "/Lotus/Weapons/Tenno/Melee/Gunblade/RedeemerPrime/RedeemerPrimeWep", // Redeemer Prime
        "/Lotus/Weapons/Tenno/Melee/Gunblade/GunbladeAutomatic/TnoGunbladeAutomatic", // Sarpa
        "/Lotus/Weapons/Grineer/Melee/GunBlade/GrnGunBlade/GrnGunblade", // Vastilok
        "/Lotus/Weapons/Corpus/Melee/Gunblade/CrpGunBlade/CrpGunbladeWeapon", // Stropha
        "/Lotus/Powersuits/Excalibur/DoomSword",                         // Exalted Blade
        "/Lotus/Powersuits/Excalibur/DoomSwordPrime",
        "/Lotus/Powersuits/Excalibur/DoomSwordUmbra",
        "/Lotus/Powersuits/Pacifist/PacifistFist", // Desert Wind
        "/Lotus/Powersuits/Pacifist/PacifistFistPrime",
    ] {
        assert!(covered.contains(w), "9-C: {w} has no row");
    }
}

#[test]
fn attack_triggers_spot_values() {
    let f = triggers();
    let rows_of = |w: &str| -> Vec<&MeleeTriggerRow> {
        f.row
            .iter()
            .filter(|r| r.weapons.iter().any(|x| x.as_str() == w))
            .collect()
    };
    // Syam: heavy 1 First Shockwave, heavy 2 Second Shockwave, 1 each, as_data (07 table 9-C).
    let syam = rows_of("/Lotus/Weapons/Tenno/Melee/Swords/DaxDuviriKatana/DaxDuviriKatanaWeapon");
    let got: Vec<(Option<u8>, &str)> = syam
        .iter()
        .map(|r| match r.trigger {
            MeleeTriggerSource::Heavy { h } => (h, r.attack.as_ref()),
            _ => panic!("Syam: heavy rows only"),
        })
        .collect();
    assert_eq!(
        got,
        [(Some(1), "First Shockwave"), (Some(2), "Second Shockwave")]
    );
    assert!(syam.iter().all(|r| r.count == TriggerCount::Fixed(1)
        && r.scaling == TriggerScaling::AsData
        && r.heavy_combo
        && !r.heavy_bonus));
    // Quassus: counts by Multishot.
    let q = rows_of("/Lotus/Weapons/Tenno/Melee/Warfan/PrimeQuassus/PrimeQuassusWeapon");
    assert_eq!(q.len(), 2);
    assert!(q.iter().all(|r| r.count == TriggerCount::Multishot));
    // Gunblades: heavy shots get heavy_bonus when hitscan (Redeemer, Sarpa, Vastilok), not Stropha (E-7.45, OQ-717).
    let heavy_bonus = |w: &str| {
        rows_of(w)
            .iter()
            .find(|r| matches!(r.trigger, MeleeTriggerSource::Heavy { h: None }))
            .map(|r| (r.heavy_combo, r.heavy_bonus, r.scaling, r.count.clone()))
    };
    let hitscan = Some((true, true, TriggerScaling::Scaled, TriggerCount::Hits));
    assert_eq!(
        heavy_bonus("/Lotus/Weapons/Tenno/Melee/Gunblade/TnoGunblade"),
        hitscan
    );
    assert_eq!(
        heavy_bonus("/Lotus/Weapons/Grineer/Melee/GunBlade/GrnGunBlade/GrnGunblade"),
        hitscan
    );
    assert_eq!(
        heavy_bonus("/Lotus/Weapons/Corpus/Melee/Gunblade/CrpGunBlade/CrpGunbladeWeapon"),
        Some((true, false, TriggerScaling::Scaled, TriggerCount::Hits))
    );
    // Exalted Blade: light waves on Neutral, Forward, Block, Aerial and the slide; two heavy waves per heavy.
    let eb = rows_of("/Lotus/Powersuits/Excalibur/DoomSwordUmbra");
    let light: BTreeSet<String> = eb
        .iter()
        .filter(|r| r.attack.as_ref() == LIGHT_WAVES)
        .map(|r| format!("{:?}", r.trigger))
        .collect();
    assert_eq!(light.len(), 5);
    let heavy: Vec<_> = eb
        .iter()
        .filter(|r| r.attack.as_ref() == HEAVY_WAVES)
        .collect();
    assert_eq!(heavy.len(), 1);
    assert_eq!(heavy[0].count, TriggerCount::Fixed(2));
    assert!(!heavy[0].heavy_combo && heavy[0].heavy_bonus);
}

// ---- 9-D (07 §7.6) ----

#[test]
fn stance_co_has_63_rows() {
    let f = stance_co();
    assert_eq!(f.row.len(), 63, "07 §7.6: 63 rows");
    let names: BTreeSet<&str> = f.row.iter().map(|r| r.stance.as_ref()).collect();
    assert_eq!(names.len(), 63, "one row per stance");
    // Rows are in page order (cached L582-706, one row every second line).
    let lines: Vec<u32> = f.row.iter().map(|r| cited_lines(&r.source)[0]).collect();
    assert!(lines.windows(2).all(|w| w[0] < w[1]));
    assert!(lines.iter().all(|l| (572..=708).contains(l)));
    for r in &f.row {
        assert!(r.source.starts_with("W:Condition_Overload_(Mechanic) L"));
    }
}

#[test]
fn stance_co_shared_columns_and_classes() {
    let f = stance_co();
    let shared = [
        ComboType::Slide,
        ComboType::Slam,
        ComboType::Finisher,
        ComboType::Wall,
        ComboType::Aerial,
        ComboType::Heavy,
    ];
    let own = [
        ComboType::Neutral,
        ComboType::Forward,
        ComboType::ForwardBlock,
        ComboType::Block,
    ];
    let mut seen: BTreeSet<MeleeClass> = BTreeSet::new();
    let mut exalted = Vec::new();
    for r in &f.row {
        for (k, v) in &r.cells {
            assert!(shared.contains(k) || own.contains(k), "{}: {k:?}", r.stance);
            assert_eq!(v.len(), 1, "{}: one ratio per combo", r.stance);
            assert!(v[0] > 0.0 && v[0] < 3.0, "{}: {k:?} = {}", r.stance, v[0]);
        }
        if r.class == MeleeClass::ExaltedWeapon {
            assert!(
                !r.first_of_class,
                "{}: exalted rows are their own",
                r.stance
            );
            exalted.push(r.stance.as_ref());
            continue;
        }
        // The first row of each class carries the shared columns (E-7.21); later rows of the class never do.
        let first = seen.insert(r.class);
        assert_eq!(r.first_of_class, first, "{}", r.stance);
        if !first {
            assert!(
                r.cells
                    .keys()
                    .all(|k| own.contains(k) || *k == ComboType::Slide),
                "{}: shared columns only on the class's first row",
                r.stance
            );
        }
    }
    assert_eq!(
        exalted,
        [
            "Serene Storm",
            "Razorwing",
            "Exalted Blade",
            "Primal Fury",
            "Ravenous Wraith",
            "Hysteria"
        ]
    );
    // 27 classes have stances (every 9-A class).
    let nine_a: BTreeSet<MeleeClass> = heavy_class().row.iter().map(|r| r.class).collect();
    assert_eq!(seen, nine_a);
}

#[test]
fn stance_co_spot_values() {
    let f = stance_co();
    let row = |s: &str| f.row.iter().find(|r| r.stance.as_ref() == s).unwrap();
    let cell = |s: &str, k: ComboType| row(s).cells.get(&k).map(|v| v[0]);
    // TV-07-29 / 04 E-4.6: a stanceless Dual Daggers heavy uses the class's first row, 0.40.
    assert_eq!(row("Gnashing Payara").class, MeleeClass::DualDaggers);
    assert_eq!(cell("Gnashing Payara", ComboType::Heavy), Some(0.40));
    // Exalted Blade 1.22-1.30 (07 E-7.21).
    let eb: Vec<f64> = row("Exalted Blade").cells.values().map(|v| v[0]).collect();
    assert!(eb.iter().all(|v| (1.22..=1.30).contains(v)), "{eb:?}");
    assert_eq!(cell("Exalted Blade", ComboType::Slide), Some(1.22));
    // Blank cells have no entry (E-7.21 reads them as 1.0).
    assert_eq!(cell("Coiling Viper", ComboType::Neutral), None);
    assert_eq!(cell("Bleeding Willow", ComboType::Forward), Some(0.80));
    assert_eq!(cell("Mountain's Edge", ComboType::Heavy), Some(2.33));
    assert_eq!(
        cell("Decisive Judgement", ComboType::ForwardBlock),
        Some(0.65)
    );
    // Blind Justice fills only Slide among the shared columns (page L644).
    assert_eq!(cell("Blind Justice", ComboType::Slide), Some(1.0));
}

// ---- 9-E (07 §9.11) ----

#[test]
fn finishers_follow_table_9e() {
    let f = finishers();
    let mut class_rows = BTreeSet::new();
    for r in &f.row {
        assert_eq!(r.kinds.len(), 5, "{:?}: every finisher kind", r.class);
        for (k, v) in &r.kinds {
            assert!(
                !v.is_empty() && v.iter().all(|x| *x > 0.0),
                "{:?} {k:?}",
                r.class
            );
        }
        if r.class == MeleeClass::ExaltedWeapon {
            assert!(!r.weapons.is_empty(), "exalted rows name their weapons");
        } else {
            assert!(r.weapons.is_empty());
            assert!(
                class_rows.insert(r.class),
                "{:?}: one row per class",
                r.class
            );
        }
    }
    let row = |c: MeleeClass| f.row.iter().find(|r| r.class == c).unwrap();
    let k = |c: MeleeClass, kind: FinisherKind| row(c).kinds[&kind].clone();
    // 07 §9.11 examples.
    assert_eq!(k(MeleeClass::Sword, FinisherKind::Ground), [10.0]);
    assert_eq!(k(MeleeClass::Sword, FinisherKind::Stealth), [8.0, 8.0]);
    assert_eq!(k(MeleeClass::Sword, FinisherKind::Back), [6.0, 2.0]);
    assert_eq!(k(MeleeClass::Sword, FinisherKind::Front), [8.0]);
    assert_eq!(k(MeleeClass::Sword, FinisherKind::Universal), [8.0]);
    assert_eq!(k(MeleeClass::Hammer, FinisherKind::Stealth), [4.0, 8.0]);
    assert_eq!(k(MeleeClass::Hammer, FinisherKind::Back), [12.0, 12.0]);
    assert_eq!(k(MeleeClass::Claws, FinisherKind::Ground), [5.0; 6]);
    let valkyr = f
        .row
        .iter()
        .find(|r| weapon_names(&r.weapons).contains(&"/Lotus/Powersuits/Berserker/BerserkerMelee"))
        .expect("Valkyr Talons row");
    assert_eq!(
        valkyr.kinds[&FinisherKind::Ground],
        [10.0, 10.0, 10.0, 10.0, 10.0, 4.0]
    );
    let ground_total: f64 = valkyr.kinds[&FinisherKind::Ground].iter().sum();
    assert!(close(ground_total, 54.0));
    assert!(f.row.iter().any(|r| {
        weapon_names(&r.weapons).contains(&"/Lotus/Powersuits/Excalibur/DoomSword")
            && r.kinds[&FinisherKind::Ground] == [12.0]
    }));
}

// ---- combo specials (07 §6) ----

#[test]
fn combo_specials_of_07_section_6() {
    let f = specials();
    let get = |w: &str| {
        f.row
            .iter()
            .find(|r| r.weapon.as_str() == w)
            .unwrap_or_else(|| panic!("{w}"))
            .specials()
    };
    let names: BTreeSet<&str> = f.row.iter().map(|r| r.weapon.as_str()).collect();
    assert_eq!(names.len(), f.row.len(), "one row per weapon");
    // E-7.10, E-7.11: Venka Prime M_cap 13 (M_bcap stays 12); Dex Nikana k_tier 11, M_cap 11, M_bcap 11.
    assert_eq!(
        get("/Lotus/Weapons/Tenno/Melee/PrimeVenKa/PrimeVenkaClaws"),
        ComboSpecials {
            m_cap: Some(13),
            ..ComboSpecials::default()
        }
    );
    assert_eq!(
        get("/Lotus/Weapons/Tenno/Melee/Swords/KatanaAndWakizashi/Dex2023Nikana/Dex2023Nikana"),
        ComboSpecials {
            k_tier: Some(11),
            m_cap: Some(11),
            m_bcap: Some(11),
            ..ComboSpecials::default()
        }
    );
    // E-7.15: innate Initial Combo.
    for (w, ic) in [
        (
            "/Lotus/Weapons/Syndicates/CephalonSuda/Melee/CSHeliocor",
            20.0,
        ),
        ("/Lotus/Weapons/Tenno/Melee/Fist/FuraxWraith", 20.0),
        ("/Lotus/Weapons/Tenno/Melee/PrimeFragor/PrimeFragor", 30.0),
    ] {
        assert_eq!(
            get(w),
            ComboSpecials {
                initial_combo: Some(ic),
                ..ComboSpecials::default()
            }
        );
    }
    assert_eq!(f.row.len(), 5);
}

// ---- bundle form ----

#[test]
fn tables_convert_to_melee_rules_and_round_trip() {
    let rules = MeleeRules {
        heavy_class: heavy_class().row,
        heavy_kappa: heavy_kappa()
            .row
            .into_iter()
            .map(|r| (r.weapon, r.kappa))
            .collect(),
        attack_triggers: triggers().row,
        stance_co: stance_co().row,
        finishers: finishers().row,
        combo_specials: specials()
            .row
            .iter()
            .map(|r| (r.weapon.clone(), r.specials()))
            .collect(),
    };
    let json = serde_json::to_string(&rules).unwrap();
    let back: MeleeRules = serde_json::from_str(&json).unwrap();
    assert_eq!(back, rules, "JSON round trip");
    let bytes = postcard::to_allocvec(&rules).unwrap();
    let back: MeleeRules = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, rules, "postcard round trip");
}
