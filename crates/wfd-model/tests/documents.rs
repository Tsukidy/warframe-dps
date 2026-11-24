//! Acceptance of T01a (work-breakdown.md): serde round trip of every document type; `SeedSel` accepts `"auto"` and
//! 0 … 2^53 − 1 and rejects 2^53; `StatKey::parse` round-trips every key; every bundle type survives the readable
//! (JSON) and the compact (postcard) encodings.

use serde::{Serialize, de::DeserializeOwned};
use smallvec::smallvec;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::Path;
use wfd_model::*;

fn json_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(v: &T) -> String {
    let text = serde_json::to_string(v).unwrap();
    let back: T = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}\n{text}"));
    assert_eq!(&back, v, "JSON round trip\n{text}");
    let pretty = serde_json::to_string_pretty(v).unwrap();
    assert_eq!(
        &serde_json::from_str::<T>(&pretty).unwrap(),
        v,
        "pretty JSON round trip"
    );
    text
}

fn postcard_round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(v: &T) {
    let bytes = postcard::to_stdvec(v).unwrap();
    let back: T = postcard::from_bytes(&bytes).unwrap_or_else(|e| panic!("postcard: {e}"));
    assert_eq!(&back, v, "postcard round trip");
}

fn server_api_example() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/architecture/server-api.md");
    let text = std::fs::read_to_string(path).unwrap();
    let start = text
        .find("Example (Braton Prime")
        .expect("example paragraph");
    let rest = &text[start..];
    let open = rest.find("```json").expect("json block") + "```json".len();
    let close = rest[open..].find("```").expect("end of block");
    rest[open..open + close].to_owned()
}

#[test]
fn server_api_calc_input_example_parses() {
    let input: CalcInput = serde_json::from_str(&server_api_example()).unwrap();
    assert_eq!(input.v, DOC_VERSION);
    assert_eq!(input.build.weapon, StableKey(1204));
    assert_eq!(input.build.slots.len(), 8);
    assert_eq!(input.build.slots[0].slot, SlotId::Gen(0));
    assert_eq!(input.scenario.enemy.level, 100);
    assert!(input.scenario.enemy.steel_path);
    assert_eq!(input.scenario.toggles, Toggles::default());
    assert_eq!(input.scenario.eval, EvalSettings::default());
    assert_eq!(input.scenario.conditions.mode, ConditionMode::AllMax);
    assert!(input.x.is_empty());
    json_round_trip(&input);
}

#[test]
fn missing_fields_take_the_documented_defaults() {
    let input: CalcInput = serde_json::from_str(
        r#"{"v":1,"build":{"weapon":7},"scenario":{"enemy":{"enemy":9,"level":30}}}"#,
    )
    .unwrap();
    assert!(input.build.weapon_item.catalyst);
    assert_eq!(input.build.weapon_item.genesis_installed, None);
    assert!(input.build.warframe.reactor);
    assert_eq!(input.scenario.enemy.party, 1);
    let t = &input.scenario.toggles;
    assert_eq!(t.hit_location, HitLocation::Body);
    assert_eq!(t.strength, 1.0);
    assert!(t.unique_dr);
    assert_eq!(t.capacity, CapacityMode::AnyForma);
    assert_eq!(t.melee_plan, MeleePlanSel::Auto);
    assert_eq!(t.co_status_count.mode, CoCountMode::Expected);
    let e = &input.scenario.eval;
    assert_eq!(
        (
            e.window_s,
            e.ttk_horizon_s,
            e.mc_runs,
            e.seed,
            e.series_bin_s
        ),
        (20.0, 60.0, 1000, SeedSel::Fixed(0), 1.0)
    );
    assert_eq!(input.scenario.profile, Profile::Game);
    // Compact form: defaults are omitted where the document type allows it.
    let text = serde_json::to_string(&input.build).unwrap();
    assert_eq!(text, r#"{"weapon":7,"variant":{},"slots":[]}"#);
}

#[test]
fn unknown_fields_are_rejected() {
    for bad in [
        r#"{"v":1,"build":{"weapon":7,"wepon":1},"scenario":{"enemy":{"enemy":9,"level":30}}}"#,
        r#"{"v":1,"build":{"weapon":7},"scenario":{"enemy":{"enemy":9,"level":30},"toggles":{"strenght":2}}}"#,
        r#"{"v":1,"build":{"weapon":7},"scenario":{"enemy":{"enemy":9,"level":30}},"y":{}}"#,
        r#"{"v":1,"build":{"weapon":7,"slots":[{"slot":"gen9","content":{"item":{"item":1}}}]},"scenario":{"enemy":{"enemy":9,"level":30}}}"#,
        r#"{"v":1,"build":{"weapon":7,"slots":[{"slot":"gen0","content":{"item":{"item":1,"rnk":2}}}]},"scenario":{"enemy":{"enemy":9,"level":30}}}"#,
    ] {
        assert!(
            serde_json::from_str::<CalcInput>(bad).is_err(),
            "accepted {bad}"
        );
    }
}

#[test]
fn seed_selection_bounds() {
    let parse = |s: &str| serde_json::from_str::<SeedSel>(s);
    assert_eq!(parse("\"auto\"").unwrap(), SeedSel::Auto);
    assert_eq!(parse("0").unwrap(), SeedSel::Fixed(0));
    assert_eq!(
        parse("9007199254740991").unwrap(),
        SeedSel::Fixed((1 << 53) - 1)
    );
    assert_eq!(MAX_SEED, 9_007_199_254_740_991);
    for bad in [
        "9007199254740992",
        "18446744073709551615",
        "-1",
        "1.5",
        "\"Auto\"",
        "null",
        "true",
    ] {
        assert!(parse(bad).is_err(), "accepted {bad}");
    }
    assert_eq!(serde_json::to_string(&SeedSel::Auto).unwrap(), "\"auto\"");
    assert_eq!(serde_json::to_string(&SeedSel::Fixed(42)).unwrap(), "42");
    for s in [SeedSel::Auto, SeedSel::Fixed(0), SeedSel::Fixed(MAX_SEED)] {
        json_round_trip(&s);
        postcard_round_trip(&s);
    }
    let eval: EvalSettings = serde_json::from_str(r#"{"seed":"auto"}"#).unwrap();
    assert_eq!(eval.seed, SeedSel::Auto);
    assert!(serde_json::from_str::<EvalSettings>(r#"{"seed":9007199254740992}"#).is_err());
}

#[test]
fn stat_key_parse_round_trips_every_key() {
    assert_eq!(StatKey::all().count(), N_STATS);
    for k in StatKey::all() {
        let name = k.name();
        assert_eq!(StatKey::parse(&name), Some(k), "{name}");
        assert_eq!(name.parse::<StatKey>().unwrap(), k);
        assert_eq!(serde_json::to_string(&k).unwrap(), format!("\"{name}\""));
        postcard_round_trip(&k);
        assert_eq!(k.meta().toml, name.split('.').next().unwrap());
    }
    for bad in [
        "",
        "cc",
        "forced.XYZ",
        "vuln_add.HP",
        "vuln_add.XX.HEA",
        "xh.8",
        "mech.-1",
        "display.speed",
    ] {
        assert!(StatKey::parse(bad).is_none(), "{bad}");
    }
}

fn full_build_doc() -> BuildDoc {
    let mut polarities = BTreeMap::new();
    polarities.insert(SlotId::Gen(3), Polarity::Madurai);
    polarities.insert(SlotId::Exilus, Polarity::Naramon);
    let mut params = BTreeMap::new();
    params.insert(ParamKey::new("element"), ParamValue::Choice("HEA".into()));
    let mut wf_pol = BTreeMap::new();
    wf_pol.insert(SlotId::Aura(0), Polarity::Any);
    BuildDoc {
        weapon: StableKey(12),
        weapon_item: WeaponItemDoc {
            rank: Some(40),
            catalyst: false,
            capacity_override: Some(76),
            polarities,
            genesis_installed: Some(true),
        },
        variant: VariantDoc {
            kitgun: Some(KitgunPicks {
                grip: StableKey(1),
                loader: StableKey(2),
            }),
            zaw: Some(ZawPicks {
                grip: StableKey(3),
                link: StableKey(4),
            }),
            progenitor: Some(ProgenitorPick {
                element: DamageType::TOX,
                v10: 600,
            }),
        },
        slots: vec![
            SlotEntry {
                slot: SlotId::Gen(0),
                content: SlotContent::Item {
                    item: StableKey(5),
                    rank: Some(3),
                    params,
                },
            },
            SlotEntry {
                slot: SlotId::Gen(7),
                content: SlotContent::Riven(Riven {
                    class: RivenClass::Pistol,
                    traits: [
                        RivenTrait {
                            trait_id: "critical_damage".into(),
                            value: 1.114,
                        },
                        RivenTrait {
                            trait_id: "zoom".into(),
                            value: -0.793,
                        },
                    ]
                    .into_iter()
                    .collect(),
                    rank: Some(8),
                    polarity: Some(Polarity::Vazarin),
                    drain: Some(18),
                    family: Some("Furis".into()),
                }),
            },
            SlotEntry {
                slot: SlotId::Stance,
                content: SlotContent::Item {
                    item: StableKey(6),
                    rank: None,
                    params: BTreeMap::new(),
                },
            },
            SlotEntry {
                slot: SlotId::Inc(3),
                content: SlotContent::Perk { index: 1 },
            },
            SlotEntry {
                slot: SlotId::Shard(4),
                content: SlotContent::Shard {
                    item: StableKey(8),
                    tauforged: true,
                },
            },
            SlotEntry {
                slot: SlotId::Subsumed,
                content: SlotContent::Subsumed {
                    ability: StableKey(9),
                    replaced: Some("Ripper's Wail".into()),
                },
            },
        ],
        stance_combo: Some(ComboKey::new("forward_block")),
        warframe: WarframeDoc {
            frame: Some(StableKey(10)),
            reactor: false,
            polarities: wf_pol,
            frame_complete: true,
        },
        companion_set_mods: vec![StableKey(11)],
        companion_kind: CompanionKind::Sentinel,
    }
}

fn full_scenario_doc() -> ScenarioDoc {
    let mut overrides = BTreeMap::new();
    overrides.insert(EffectKey::new("/Lotus/A#co"), CondOverride::Force(false));
    overrides.insert(EffectKey::new("/Lotus/B#e0"), CondOverride::Stacks(2));
    overrides.insert(
        EffectKey::new("/Lotus/C#dmg"),
        CondOverride::Uptime {
            u: 0.5,
            stacks: Some(1),
        },
    );
    overrides.insert(
        EffectKey::new("/Lotus/D#dmg"),
        CondOverride::Uptime {
            u: 0.25,
            stacks: None,
        },
    );
    let mut custom = BTreeMap::new();
    custom.insert("crit_chance".into(), 0.5);
    let mut params = BTreeMap::new();
    params.insert(ParamKey::new("value"), ParamValue::Num(0.3));
    ScenarioDoc {
        enemy: EnemySetup {
            enemy: StableKey(900),
            level: 165,
            steel_path: true,
            eximus: true,
            party: 4,
            mission: Some(MissionModifier::ArchonHunt),
            context_overguard: true,
        },
        toggles: Toggles {
            hit_location: HitLocation::WeakPoint,
            co_multiplicative: true,
            co_status_count: CoStatusCount {
                mode: CoCountMode::Manual,
                n: Some(5),
            },
            external_viral: Some(10),
            armor_strip: 0.5,
            stealth: Some(Stealth { rank: 30 }),
            heavy: Some(Heavy { start_combo: 12 }),
            light_combo: Some(6),
            ability_combo: Some(3),
            sniper_combo: Some(1.5),
            strength: 2.5,
            wp_cc_multiplicative: true,
            combine_groups: smallvec![0, 2],
            distance_m: 25.0,
            unique_dr: false,
            capacity: CapacityMode::Budget(3),
            melee_plan: MeleePlanSel::HeavyLoop {
                policy: HeavyPolicy::BackToBack,
                heavy_cancel: Some(0.3),
            },
            zoom_level: Some(1),
        },
        buffs: vec![
            BuffPick {
                buff: StableKey(50),
                params: params.clone(),
                caster: Caster::SelfCast,
            },
            BuffPick {
                buff: StableKey(51),
                params,
                caster: Caster::Ally { strength: 3.0 },
            },
        ],
        custom,
        conditions: ConditionPolicy {
            mode: ConditionMode::Simulated,
            overrides,
        },
        eval: EvalSettings {
            window_s: 30.0,
            ttk_horizon_s: 90.0,
            mc_runs: 5000,
            seed: SeedSel::Auto,
            series_bin_s: 0.5,
        },
        profile: Profile::Compat(CompatFlags::empty()),
    }
}

#[test]
fn every_document_type_round_trips() {
    let build = full_build_doc();
    let scenario = full_scenario_doc();
    let mut x = BTreeMap::new();
    x.insert(
        StableKey(LOCAL_KEY_BASE + 3),
        UniqueName::new("/Lotus/Weapons/Tenno/LongGuns/NewRifle/NewRifleWeapon"),
    );
    let input = CalcInput {
        v: DOC_VERSION,
        build: build.clone(),
        scenario: scenario.clone(),
        x,
    };
    let text = json_round_trip(&input);
    assert!(
        text.contains(
            r#""x":{"2147483651":"/Lotus/Weapons/Tenno/LongGuns/NewRifle/NewRifleWeapon"}"#
        ),
        "{text}"
    );
    assert!(
        text.contains(r#""caster":{"ally":{"strength":3.0}}"#),
        "{text}"
    );
    assert!(text.contains(r#""seed":"auto""#), "{text}");
    assert!(text.contains(r#""profile":{"compat":[]}"#), "{text}");
    assert!(text.contains(r#""capacity":{"budget":3}"#), "{text}");
    assert!(StableKey(LOCAL_KEY_BASE + 3).is_local() && !StableKey(LOCAL_KEY_BASE - 1).is_local());
    json_round_trip(&build);
    json_round_trip(&scenario);
    json_round_trip(&scenario.enemy);
    json_round_trip(&scenario.toggles);
    json_round_trip(&scenario.conditions);
    json_round_trip(&scenario.eval);
    for slot in &build.slots {
        json_round_trip(slot);
    }

    for plan in [
        MeleePlanSel::Auto,
        MeleePlanSel::LightLoop,
        MeleePlanSel::SlideLoop,
        MeleePlanSel::HeavyLoop {
            policy: HeavyPolicy::FullRegen,
            heavy_cancel: None,
        },
        MeleePlanSel::BuildSpend { k: 12.0 },
        MeleePlanSel::Tennokai,
        MeleePlanSel::SlamLoop { t_slam: 1.0 },
        MeleePlanSel::HeavySlamLoop { t_slam: 2.0 },
        MeleePlanSel::GlaiveCycle {
            throw: ThrowKind::Quick,
        },
        MeleePlanSel::CycleLoop,
        MeleePlanSel::DataAttackLoop { t_cyc: Some(0.8) },
    ] {
        json_round_trip(&plan);
    }
    let plan: MeleePlanSel =
        serde_json::from_str(r#"{"heavy_loop":{"policy":"full_regen"}}"#).unwrap();
    assert_eq!(
        plan,
        MeleePlanSel::HeavyLoop {
            policy: HeavyPolicy::FullRegen,
            heavy_cancel: None
        }
    );
    for c in [
        CapacityMode::Ignore,
        CapacityMode::Current,
        CapacityMode::AnyForma,
        CapacityMode::Budget(0),
    ] {
        json_round_trip(&c);
    }
    assert_eq!(
        serde_json::to_string(&CapacityMode::AnyForma).unwrap(),
        "\"any_forma\""
    );

    let violation: Violation = serde_json::from_str(
        r#"{"key":"cap.exceeded","severity":"illegal","slots":["gen3","exilus"],"args":{"used":64,"cap":"60"}}"#,
    )
    .unwrap();
    assert_eq!(violation.severity, RuleSeverity::Illegal);
    assert_eq!(
        violation.slots.as_slice(),
        &[SlotId::Gen(3), SlotId::Exilus]
    );
    json_round_trip(&violation);
}

#[test]
fn resolved_forms_round_trip() {
    let mut frame = FrameLoadout {
        frame: Some(FrameId(3)),
        ..FrameLoadout::default()
    };
    frame.r#gen[0] = Slot::Mod {
        item: ItemId(4),
        rank: 5,
        params: smallvec![],
    };
    frame.auras[0] = Slot::Mod {
        item: ItemId(5),
        rank: 5,
        params: smallvec![],
    };
    frame.shards[1] = Some(ShardPick {
        item: ItemId(6),
        tauforged: true,
    });
    frame.subsumed = Some(Subsumed {
        ability: ItemId(7),
        replaced: None,
    });
    let mut gen_slots: [Slot; 8] = Default::default();
    gen_slots[1] = Slot::Mod {
        item: ItemId(1),
        rank: 10,
        params: smallvec![(ParamKey::new("element"), ParamValue::Choice("COL".into()))],
    };
    gen_slots[7] = Slot::Riven(Riven {
        class: RivenClass::Melee,
        traits: [RivenTrait {
            trait_id: "critical_chance".into(),
            value: 2.2,
        }]
        .into_iter()
        .collect(),
        rank: None,
        polarity: None,
        drain: None,
        family: None,
    });
    let build = Build {
        weapon: WeaponId(2),
        item: WeaponItem {
            rank: 30,
            catalyst: true,
            capacity_override: None,
            polarities: BTreeMap::new(),
        },
        variant: Variant {
            kitgun: Some((0, 1)),
            zaw: None,
            progenitor: None,
        },
        r#gen: gen_slots,
        exilus: Slot::Empty,
        stance: Some(StanceId(1)),
        stance_rank: Some(3),
        combo: ComboType::Forward,
        arcanes: [Some((ItemId(9), 5)), None],
        genesis_installed: true,
        incarnon: [Some(0), None, Some(2), None],
        frame,
        companion_set_mods: smallvec![ItemId(12)],
        companion_kind: CompanionKind::Kavat,
    };
    let scenario = Scenario {
        enemy: EnemyInstance {
            enemy: EnemyId(4),
            level: 100,
            steel_path: true,
            eximus: false,
            party: 1,
            mission: None,
            context_overguard: false,
        },
        toggles: Toggles::default(),
        buffs: smallvec![BuffInstance {
            item: ItemId(30),
            params: smallvec![(ParamKey::new("value"), ParamValue::Num(0.45))],
            caster: Caster::SelfCast,
        }],
        custom: smallvec![(ItemId(31), 0.25)],
        conditions: ConditionPolicy::default(),
        eval: EvalSettings::default(),
        profile: Profile::Game,
    };
    let resolved = Resolved {
        build,
        scenario,
        violations: vec![],
    };
    json_round_trip(&resolved);
    postcard_round_trip(&resolved.build);
}

// ---------------------------------------------------------------------------------------------------------------
// A fully populated bundle: every bundle type through JSON and postcard.
// ---------------------------------------------------------------------------------------------------------------

fn eff(toml_text: &str) -> EffectDef {
    toml::from_str(toml_text).unwrap_or_else(|e| panic!("{e}\n{toml_text}"))
}

fn effects_of_every_kind() -> Vec<EffectDef> {
    vec![
        eff(
            "id = \"e0\"\nstat = \"base_dmg\"\nvalue = { max = 1.65, rank = { table = [0.15, 0.3, 1.65] } }",
        ),
        eff(
            "element = { param = \"element\" }\norder = \"slot\"\nvalue = { variants = { furis = 28.0, mk1_furis = 34.0 } }\nclass = [\"static\"]\nnote = \"UNVERIFIED\"",
        ),
        eff(
            "faction = [\"grineer\", \"kuva_grineer\"]\nvalue = { scalar = \"set_piece\", values = [0.1, 0.2] }",
        ),
        eff(
            "mult = \"eclipse\"\nvalue = 0.3\nwhen = { any = [{ target = \"frozen\" }, { shot = \"first_of_magazine\" }] }\nstacks = { max = 3, gain = 1, duration = 4.0, stack_timer = \"shared_lose_all\", refresh = \"no_refresh_while_active\", decay = { idle = { t_idle = 5.0, rate = 0.01 } }, consume_on = { next_shots = 2 }, reset_on = { on = \"on_miss\", count = 1 } }",
        ),
        eff(
            "stat = \"vuln_add.SH.MAG\"\nvalue = 0.5\nvuln = { types = [\"MAG\"], layers = [\"SH\"] }\nscope = { weapon = { classes = [\"bow\"], melee_classes = [\"glaive\"], categories = [\"Daggers\"], tags = [\"AOE\"], traits = [\"beam\"], exalted = \"never\" }, attacks = { any_of = [\"heavy\"], none_of = [\"slam\"] }, form = \"incarnon\", hit = { kinds = [\"direct\", \"dot_tick\"], body_part = [\"head\"], pellet = \"extra_only\", outcome = { crit = { at_least = 2 }, status = \"some\" } }, layers = [\"HP\"] }",
        ),
        eff("mechanic = \"extra_hit\"\nvalue = 0.26\n[params]\nelement = \"VOI\""),
        eff(
            "mechanic = \"proc_on_event\"\nvalue = 0.3\n[params]\non = \"hit\"\nproc = { random = [\"HEA\", \"COL\"] }\ncount = 2",
        ),
        eff(
            "mechanic = \"convert_damage\"\nvalue = 1.0\n[params]\nfrom = [\"IMP\"]\nto = \"PUN\"\nfraction = 0.5",
        ),
        eff("mechanic = \"repeat_hit\"\nvalue = 1.0\n[params]\non = { crit_tier_at_least = 1 }"),
        eff(
            "mechanic = \"flat_damage_on_proc\"\nvalue = 750.0\n[params]\nstatuses = [\"HEA\", \"ELE\"]\nscaling = \"strength\"",
        ),
        eff(
            "mechanic = \"proc_after_ticks\"\nvalue = 1.0\n[params]\ncombined = \"GAS\"\nticks = 3",
        ),
        eff(
            "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cd_rel\"\nper_event = \"hit\"\nmax = 100\nreset_on = { crit_tier_at_least = 2 }\nreset_after = 6\nhit_unit = \"shot\"",
        ),
        eff(
            "mechanic = \"random_multiplier\"\nvalue = 0.5\n[params]\nfactor = 21.0\non = { crit_tier_eq = 0 }",
        ),
        eff(
            "mechanic = \"spawn_attack\"\nvalue = 0.5\n[params]\non = \"bounce\"\nfraction = 0.5\ntemplate = { radial = true, can_crit = true }",
        ),
        eff(
            "stat = \"cc_rel\"\nvalue = 1.0\nwhen = { trigger = { next_shot_after = { on_status = \"COL\" } }, chance = 0.5, hit_unit = \"instance\" }",
        ),
        eff(
            "stat = \"cc_rel\"\nvalue = 1.0\nwhen = { trigger = { on_melee_input = \"slide\" } }\nper = [{ status_stacks = \"COR\" }, { status_present = \"VIR\" }, \"strength\", \"sniper_combo\", \"combo_tier\", \"distinct_status\"]",
        ),
        eff(
            "stat = \"cd_rel\"\nvalue = 1.0\nwhen = { all = [{ attack = { stat_at_least = { stat = \"sc_of.SLA\", x = 0.5 } } }, { target = { has_status = \"HEA\" } }, { target = { status_count_at_least = 3 } }, { target = \"incapacitated\" }, { trigger = { combo = 4 } }, { trigger = { on_melee_hit_count = 20 } }] }",
        ),
    ]
}

fn attack(slug: &str, idx: u8) -> Attack {
    let mut base = [0.0; N_DT];
    base[DamageType::IMP.index()] = 12.0;
    base[DamageType::SLA.index()] = 23.5;
    Attack {
        key: AttackKey {
            slug: slug.into(),
            idx,
        },
        name: NameKey::new(format!("/Lotus/W#{slug}")),
        roles: AttackRoles::NORMAL | AttackRoles::BEAM,
        shot: ShotType::HitScan,
        parent: None,
        base,
        cc: 0.12,
        cd: 2.0,
        sc: 0.26,
        multishot: 1.0,
        fire: FireSpec {
            trigger: Trigger::Held,
            rate: f64::MAX,
            charge_time: Some(0.5),
            charge_fixed: true,
            partial_charge: Some(PartialCharge {
                release_fraction: 0.33,
            }),
            burst: Some(Burst {
                count: 3,
                delay: 0.05,
            }),
            burst_reload_delay: Some(0.2),
            spool: Some(Spool {
                shots: Some(9),
                start_frac: 0.2,
                step: None,
            }),
            held: Some(Held {
                tick_rate: 12.0,
                ramp_start: 0.2,
                ramp_time: 0.6,
            }),
            semi_cap: Some(8.0),
            kohm: Some(KohmSpec {
                max_pellets: 12,
                start_frac: 0.2,
            }),
            peacemaker: Some(PeacemakerSpec {
                ramp_start: 0.25,
                ramp_time: 3.5,
                burst: 2,
                step: 0.075,
                max_bursts: 20,
            }),
            gauge: Some(GaugeSpec { charges: 3 }),
        },
        ammo_cost: 1.0,
        magazine_override: Some(20.0),
        reload_override: None,
        falloff: Some(Falloff {
            start: 10.0,
            end: 30.0,
            min_factor: 0.4,
        }),
        radius: Some(3.0),
        punch_through: 0.0,
        projectile_speed: Some(80.0),
        range: Some(300.0),
        forced: smallvec![StatusType::SLA],
        headshot: HeadshotRule::Normal,
        co: CoSpec {
            behaviour: CoBehaviour::Multiplying,
            ratio: 0.2,
            co_ref: CoRef::Fixed,
            fixed_abs: Some(200.0),
            flags: CoFlags::NO_COMBO | CoFlags::DOT_RECALC_PER_TICK,
            ms_proj_factor: Some(0.65),
            off_with: smallvec![ItemId(1)],
            row: Some(114),
            mproj: MprojMode::TwoBucket,
        },
        sniper_combo: Some(SniperComboSpec {
            min: Some(3.0),
            reset: 2.0,
        }),
        is_heavy: Some(false),
        explosion_delay: Some(1.3),
        embed_delay: None,
        lingering: Some(Lingering {
            effect_duration: 5.0,
            tick_rate: 1.0,
        }),
        synthesized: Some(Synthesized::HeavyFrom {
            normal: AttackIdx(0),
        }),
        compression_eff: Some(1.0),
        passives: effects_of_every_kind(),
    }
}

fn strike() -> Strike {
    Strike {
        input: 1,
        entry: 0,
        mult: 2.5,
        ty: StrikeType::Spin360,
        phys: [1.0, 0.0, 0.25],
        procs: smallvec![StatusType::SLA, StatusType::LIF],
        cc: smallvec![CrowdControl::Ragdoll],
        status_mult: 1.5,
    }
}

fn heavy_strike() -> HeavyStrike {
    HeavyStrike {
        mult: 6.0,
        procs: smallvec![StatusType::SLA],
        cc: smallvec![],
        spawns: Some(AttackIdx(2)),
    }
}

fn trigger_row() -> MeleeTriggerRow {
    MeleeTriggerRow {
        weapons: vec![UniqueName::new("/Lotus/Weapons/Syam")],
        trigger: MeleeTriggerSource::Heavy { h: Some(1) },
        attack: "First Shockwave".into(),
        attack_idx: Some(AttackIdx(2)),
        count: TriggerCount::Fixed(1),
        scaling: TriggerScaling::AsData,
        heavy_combo: true,
        heavy_bonus: false,
        source: "07 §9.7".into(),
    }
}

fn weapon() -> Weapon {
    let mut display = WeaponDisplay::default();
    display
        .values
        .insert("accuracy".into(), DisplayValue::Num(25.0));
    display
        .values
        .insert("noise".into(), DisplayValue::Text("Alarming".into()));
    Weapon {
        id: WeaponId(0),
        key: StableKey(1204),
        unique_name: UniqueName::new("/Lotus/Weapons/Tenno/Rifle/BratonPrime"),
        name: NameKey::new("/Lotus/Weapons/Tenno/Rifle/BratonPrime"),
        slot: WeaponSlot::Primary,
        class: WeaponClass::Rifle,
        categories: CategorySet::bit(0) | CategorySet::bit(63),
        compat_tags: smallvec![TagId(3)],
        traits: WeaponTraits::PRIME | WeaponTraits::INCARNON,
        family: Some(FamilyId(0)),
        riven_class: Some(RivenClass::Rifle),
        riven_family: Some("Braton".into()),
        trigger: Trigger::Auto,
        magazine: Some(MagazineSpec {
            size: 75.0,
            ammo_max: Some(600.0),
            infinite: false,
            battery: Some(Battery {
                delay: 0.25,
                delay_empty: 0.5,
                rate: 50.0,
            }),
        }),
        reload: ReloadSpec::PerShell {
            start_end: 0.5,
            per_shell: 0.3,
        },
        attacks: smallvec![attack("normal_attack", 0), attack("incarnon_form", 1)],
        groups: smallvec![AttackGroup {
            members: smallvec![AttackIdx(0), AttackIdx(1)],
            combinable: true,
            summed: false,
            volley: true
        }],
        zoom: smallvec![ZoomLevel {
            name: NameKey::new("zoom1"),
            effects: effects_of_every_kind()
        }],
        melee: Some(MeleeData {
            class: MeleeClass::TwoHandedNikana,
            family: Some("Nikana".into()),
            fr_base: 0.833333,
            wind_up: 0.7,
            combo_dur: None,
            heavy_swing: Some(1.2),
            follow_through: 0.6,
            range: 2.5,
            stanceless: ComboTableId(0),
            unique: Some(ComboTableId(1)),
            fixed_stance: Some(StanceId(0)),
            heavy: Some(HeavyDef {
                attack: AttackIdx(1),
                hits: [
                    smallvec![heavy_strike(), heavy_strike()],
                    smallvec![heavy_strike()],
                ],
                kappa: 0.5416666666666666,
                source: HeavySource::ClassTable { sigma: [1.2, 1.0] },
            }),
            triggers: smallvec![trigger_row()],
            specials: ComboSpecials {
                k_tier: Some(11),
                m_cap: Some(11),
                m_bcap: Some(11),
                combo_dur: Some(5.0),
                initial_combo: Some(30.0),
            },
            ability_combo: true,
        }),
        incarnon: Some(IncarnonDef {
            family: FamilyId(0),
            variant: "furis".into(),
            native: false,
            tiers: [
                smallvec![ItemId(0)],
                smallvec![ItemId(1), ItemId(2)],
                smallvec![],
                smallvec![],
                smallvec![],
            ],
        }),
        modular: Some(ModularRef {
            kind: ModularKind::Kitgun,
            base: 0,
        }),
        progenitor: Some(ProgenitorSpec {
            elements: smallvec![DamageType::IMP, DamageType::HEA],
            min: 0.25,
            max: 0.6,
            default: 0.6,
        }),
        exalted: Some(ExaltedScaling::Partial(0.5)),
        passives: effects_of_every_kind(),
        polarities: smallvec![Polarity::Madurai, Polarity::Umbra],
        exilus_polarity: Some(Polarity::Naramon),
        stance_polarity: None,
        max_rank: 40,
        mastery: 14,
        release: Some("2012-10-25".into()),
        image: Some("/Lotus/Interface/Icons/BratonPrime.png".into()),
        display,
        provenance: Provenance {
            sources: vec![SourceRef {
                group: "attacks".into(),
                source: "wiki".into(),
                revision: "123456".into(),
            }],
            overridden: vec!["trigger".into()],
        },
    }
}

fn param_specs() -> Vec<ParamSpec> {
    vec![
        ParamSpec {
            key: ParamKey::new("value"),
            kind: ParamKind::Percent {
                min: 0.0,
                max: 99.99,
            },
            default: ParamValue::Num(0.3),
            label: None,
            helper: Some(ParamHelper {
                formula: "rank_value * strength".into(),
                rank_values: vec![0.1, 0.5],
            }),
        },
        ParamSpec {
            key: ParamKey::new("count"),
            kind: ParamKind::Number {
                min: 1.0,
                max: 10.0,
                step: 1.0,
            },
            default: ParamValue::Num(10.0),
            label: None,
            helper: None,
        },
        ParamSpec {
            key: ParamKey::new("element"),
            kind: ParamKind::Choice(vec![
                Choice {
                    value: "HEA".into(),
                    label: None,
                },
                Choice {
                    value: "COL".into(),
                    label: None,
                },
            ]),
            default: ParamValue::Choice("HEA".into()),
            label: None,
            helper: None,
        },
        ParamSpec {
            key: ParamKey::new("on"),
            kind: ParamKind::Bool,
            default: ParamValue::Bool(true),
            label: None,
            helper: None,
        },
    ]
}

fn items() -> Vec<Item> {
    let kinds = [
        ItemKind::WeaponMod {
            exilus: true,
            augment_of: Some(UniqueName::new("/Lotus/Powersuits/Rhino")),
        },
        ItemKind::Stance {
            stance: StanceId(0),
        },
        ItemKind::WeaponArcane {
            class: ArcaneClass::Akimbo,
        },
        ItemKind::FrameMod { exilus: false },
        ItemKind::Aura,
        ItemKind::FrameArcane,
        ItemKind::Shard {
            color: ShardColor::Violet,
            tauforged_factor: 1.5,
        },
        ItemKind::IncarnonPerk {
            family: FamilyId(0),
            tier: 2,
            index: 0,
        },
        ItemKind::Buff {
            group: BuffGroup::Ability,
            toggle: Some(ToggleId::new("T-13")),
            strength_scaled: true,
            subsumable: true,
            inspiration_key: Some("o.r".into()),
        },
        ItemKind::CustomStat {
            inspiration_key: "p.a".into(),
        },
    ];
    kinds
        .into_iter()
        .enumerate()
        .map(|(i, kind)| Item {
            id: ItemId(i as u32),
            key: StableKey(3300 + i as u32),
            unique_name: UniqueName::new(format!("/Lotus/Upgrades/Mods/Test{i}")),
            name: NameKey::new(format!("/Lotus/Upgrades/Mods/Test{i}")),
            kind,
            compat: CompatRule {
                slot_type: Some(ModSlotType::Primary),
                categories: CategorySet::bit(4),
                weapons: smallvec![WeaponId(0)],
                tags: smallvec![TagId(1)],
                forbid_tags: smallvec![TagId(2)],
                exalted_lift: if i % 2 == 0 {
                    ExaltedLift::AllExalted
                } else {
                    ExaltedLift::Only(vec![WeaponId(0)])
                },
                exalt_req: smallvec![TagId(5)],
                trigger: Some(Trigger::Semi),
                min_base_magazine: Some(6.0),
                magazine_exempt: smallvec![WeaponId(0)],
            },
            incompat: smallvec![ItemId(1)],
            polarity: Some(Polarity::Vazarin),
            base_drain: -2,
            max_rank: 10,
            rarity: Rarity::Legendary,
            set: Some(SetId(0)),
            params: param_specs(),
            effects: effects_of_every_kind(),
            status: [
                EffectStatus::Verified,
                EffectStatus::Auto,
                EffectStatus::Draft,
                EffectStatus::Stale,
                EffectStatus::Unmodelled,
                EffectStatus::NoDamageEffect,
            ][i % 6],
            effect_tier: Some(
                [
                    EffectTier::StatLine,
                    EffectTier::SpecTable,
                    EffectTier::Curated,
                ][i % 3],
            ),
            auto_patched: i == 3,
            sources: vec!["W:Serration".into()],
            upstream_text: Some("+165% Damage".into()),
            inspiration_name: None,
            image: None,
        })
        .collect()
}

fn scaling_curve() -> ScalingCurve {
    ScalingCurve {
        c1: 0.015,
        e1: 2.12,
        c2: 10.7332,
        e2: 0.72,
        lo: 70.0,
        hi: 80.0,
        q_from_level_one: false,
    }
}

fn rules() -> Rules {
    let status = std::array::from_fn(|i| {
        let s = StatusType::from_index(i).unwrap();
        let mut params = BTreeMap::new();
        params.insert("per_stack".into(), 0.05);
        StatusRule {
            status: s,
            duration: Some(6.0),
            sd_scales: true,
            cap: if i % 2 == 0 { Some(10) } else { None },
            stacking: [
                StatusStacking::Independent,
                StatusStacking::Single,
                StatusStacking::Pool,
                StatusStacking::Fuse,
                StatusStacking::Permanent,
            ][i % 5],
            replace_oldest: true,
            first_tick: Some(1.0),
            interval: Some(1.0),
            k_dot: Some(0.35),
            tick_type: Some(DamageType::CIN),
            counts_in_n: true,
            params,
            source: "03 §4.1".into(),
        }
    });
    let mut type_mods = [1.0; N_DT];
    type_mods[DamageType::IMP.index()] = 1.5;
    let mut constants = BTreeMap::new();
    constants.insert("E-5.4.steel_path_mult".into(), 2.5);
    let mut stance_cells = BTreeMap::new();
    stance_cells.insert(ComboType::Neutral, vec![1.0]);
    stance_cells.insert(ComboType::Heavy, vec![0.4, 0.6]);
    let mut finisher_kinds = BTreeMap::new();
    finisher_kinds.insert(FinisherKind::Stealth, vec![8.0, 8.0]);
    Rules {
        status,
        factions: vec![Faction {
            id: FactionId(0),
            slug: "grineer".into(),
            name: NameKey::new("faction.grineer"),
            type_mods,
            bane_of: smallvec![FactionId(0)],
            weakness: Some(DamageType::COR),
            scaling_family: 0,
        }],
        scaling: vec![ScalingClass {
            name: "grineer".into(),
            health: scaling_curve(),
            shield: scaling_curve(),
            armor: scaling_curve(),
            overguard: ScalingCurve {
                q_from_level_one: true,
                ..scaling_curve()
            },
        }],
        mult_groups: vec![MultGroupDef {
            id: "custom_vuln".into(),
            stage: MultStage::TargetVuln,
            dot: DotInheritance::Twice,
            twice_on_two_bucket: true,
            vuln: Some(VulnMask {
                types: TypeSel::Set(DamageTypeSet::HEA),
                layers: LayerSet::HP,
            }),
            sources: vec!["04 §3".into()],
        }],
        slot_layouts: vec![SlotLayout {
            class: EquipmentClass::Melee,
            slots: vec![
                SlotDef {
                    id: SlotId::Gen(0),
                    accepts: smallvec![SlotKind::General, SlotKind::Riven],
                    arcane_family: None,
                    element_order: Some(0),
                    fixed: false,
                },
                SlotDef {
                    id: SlotId::Arcane(0),
                    accepts: smallvec![SlotKind::Arcane],
                    arcane_family: Some(ArcaneFamily::Melee),
                    element_order: None,
                    fixed: false,
                },
            ],
            source: "09 §2.2".into(),
        }],
        co_default: CoDefaults {
            melee_radial: CoSpec {
                behaviour: CoBehaviour::None,
                ..CoSpec::default()
            },
            projectile_radial_child: CoSpec::default(),
            hitscan_radial_child: CoSpec {
                co_ref: CoRef::Parent,
                ..CoSpec::default()
            },
            beam_aoe: CoSpec::default(),
            embedded_cloud: CoSpec::default(),
            projectile_child: CoSpec {
                co_ref: CoRef::Uncharged,
                ..CoSpec::default()
            },
            standalone_aoe: CoSpec::default(),
            other: CoSpec::default(),
        },
        body_parts: BodyPartRules {
            faction_heads: vec![FactionHeadRule {
                factions: vec!["grineer".into()],
                head_mult: 3.0,
                headcrit: 2.0,
                source: "05 §4.3".into(),
            }],
            head_wp_factor: 1.5,
            sources: vec![],
        },
        special_dr: vec![SpecialDrDef {
            id: "eidolon".into(),
            attenuation: Some(Attenuation {
                p_att: Some(2.0),
                layers: LayerSet::SH | LayerSet::HP,
            }),
            dps_dr: Some(vec![[1000.0, 0.5]]),
            adaptation: Some(Adaptation {
                kind: AdaptationKind::Sentient,
                res: vec![0.1, 0.2],
                params: BTreeMap::new(),
            }),
            sources: vec![],
        }],
        riven_traits: vec![RivenTraitDef {
            id: "critical_chance".into(),
            pe_tag: Some("WeaponCritChanceMod".into()),
            input: RivenUnit::Percent,
            classes: smallvec![RivenClass::Rifle],
            template: Some(eff("stat = \"cc_rel\"")),
            label: None,
            coeff: [(RivenColumn::Rifle, 0.016666), (RivenColumn::Melee, 0.02)]
                .into_iter()
                .collect(),
            sign: RivenSign::PositiveOnlyUnverified,
            spliced: true,
            recipes: vec![["damage".into(), "zoom".into()]],
            inspiration_key: Some("crit_chance".into()),
        }],
        riven_formula: RivenFormula {
            scale: 10.0,
            disposition: 1.2,
            quality_min: 0.9,
            quality_span: 0.2,
            layouts: vec![RivenLayoutRow {
                positives: 2,
                negatives: 1,
                l_pos: 1.2375,
                l_neg: Some(-0.495),
            }],
            max_rank: 8,
            default_drain_at_max: 18,
            sources: vec![],
        },
        toggles: vec![ToggleDescriptor {
            id: ToggleId::new("T-05"),
            field: "toggles.co_status_count".into(),
            group: ToggleGroup("conditions".into()),
            label: serde_json::from_str("\"toggle.t05.label\"").unwrap(),
            help: serde_json::from_str("\"toggle.t05.help\"").unwrap(),
            control: ControlKind {
                kind: ControlType::Select,
                min: Some(1.0),
                max: Some(50.0),
                step: Some(1.0),
                options: vec![ControlOption {
                    value: "expected".into(),
                    label: serde_json::from_str("\"opt.expected\"").unwrap(),
                }],
            },
            default: serde_json::json!({ "mode": "expected", "n": null, "list": [1, 2.5, "x", true] }),
            visible_if: Some(VisibilityRule {
                weapon_slot: vec![WeaponSlot::Melee],
                attack_role: AttackRoles::HEAVY,
                traits: WeaponTraits::EXALTED,
            }),
            advanced: true,
        }],
        melee: MeleeRules {
            heavy_class: vec![HeavyClassRow {
                class: MeleeClass::Machete,
                total: [Some(6.0), Some(5.0)],
                breakdown: [vec![heavy_strike()], vec![]],
                wind_up: Some(0.7),
                source: "W:Melee".into(),
            }],
            heavy_kappa: vec![(UniqueName::new("/Lotus/Weapons/Hirudo"), 0.5416666666666666)],
            attack_triggers: vec![trigger_row()],
            stance_co: vec![StanceCoRow {
                stance: "Blind Justice".into(),
                class: MeleeClass::Sword,
                first_of_class: true,
                cells: stance_cells,
                source: "9-D".into(),
            }],
            finishers: vec![FinisherRow {
                class: MeleeClass::Sword,
                kinds: finisher_kinds,
                source: "9-E".into(),
            }],
            combo_specials: vec![(
                UniqueName::new("/Lotus/Weapons/VenkaPrime"),
                ComboSpecials {
                    m_cap: Some(13),
                    ..ComboSpecials::default()
                },
            )],
        },
        helminth: HelminthRules {
            restricted: vec![UniqueName::new("wfd:buff/roar")],
            hosts: vec![HelminthHost {
                frame: UniqueName::new("/Lotus/Powersuits/Rhino"),
                ability: UniqueName::new("wfd:buff/roar"),
            }],
            sources: vec![],
        },
        categories: vec![CategoryDef {
            bit: 4,
            name: "Assault Rifle".into(),
            label: None,
        }],
        tags: vec![TagDef {
            id: TagId(1),
            name: "AOE".into(),
        }],
        game_constants: constants,
    }
}

fn bundle() -> Bundle {
    let mut caps = [None; N_ST];
    caps[StatusType::COL.index()] = Some(4);
    let mut en = BTreeMap::new();
    en.insert(
        NameKey::new("/Lotus/Weapons/Tenno/Rifle/BratonPrime"),
        "Braton Prime".into(),
    );
    let mut names = BTreeMap::new();
    names.insert("en".into(), en);
    let mut coverage = BTreeMap::new();
    coverage.insert(
        "weapon_mod".into(),
        [
            (EffectStatus::Verified, 12u32),
            (EffectStatus::Unmodelled, 3),
        ]
        .into_iter()
        .collect(),
    );
    let mut magazine_by_tier = BTreeMap::new();
    magazine_by_tier.insert("Med".into(), 9.0);
    let part = |kind| ModularPart {
        key: StableKey(70),
        unique_name: UniqueName::new("/Lotus/Weapons/SolarisUnited/Primary/Grip"),
        name: NameKey::new("grip"),
        kind,
        slot: Some(WeaponSlot::Secondary),
        hand: Some(ZawHand::TwoHanded),
        damage: 12.0,
        speed: -0.08,
        cc: 0.03,
        cd: 0.0,
        sc: 0.0,
        reload: Some(1.2),
        magazine_tier: Some("Med".into()),
        image: None,
    };
    Bundle {
        manifest: Manifest {
            schema_version: BUNDLE_SCHEMA_VERSION,
            data_version: "2026.10.28.09.54+0123abcd".into(),
            game_baseline: "Hotfix 44.1.4".into(),
            created_at: "2026-11-24T00:00:00Z".into(),
            sources: vec![SourceStamp {
                source: "wiki".into(),
                id: "Module:Weapons/data/primary".into(),
                revision: "123456".into(),
                timestamp: None,
            }],
            curated_rev: "abc".into(),
            spec_rev: "def".into(),
            licences: vec![LicenceNotice {
                source: "wiki".into(),
                licence: "CC BY-NC-SA 3.0".into(),
                url: "https://creativecommons.org/licenses/by-nc-sa/3.0/".into(),
                attribution: "wiki.warframe.com".into(),
                modules: vec![("Module:Weapons/data/primary".into(), 123_456)],
            }],
            coverage: Coverage(coverage),
        },
        weapons: vec![weapon()],
        items: items(),
        sets: vec![SetDef {
            id: SetId(0),
            unique_name: UniqueName::new("/Lotus/Upgrades/Mods/Sets/Vigilante/VigilanteSetMod"),
            name: NameKey::new("set.vigilante"),
            members: smallvec![ItemId(0), ItemId(1)],
            noncounting: smallvec![],
            bonus: effects_of_every_kind(),
            member_scale: Some(vec![1.0, 1.25]),
        }],
        stances: vec![Stance {
            id: StanceId(0),
            item: ItemId(1),
            class: MeleeClass::Nikana,
            table: ComboTableId(0),
        }],
        modular: ModularTables {
            chambers: vec![KitgunChamber {
                key: StableKey(71),
                unique_name: UniqueName::new("/Lotus/Weapons/SolarisUnited/Secondary/Catchmoon"),
                name: NameKey::new("catchmoon"),
                slot: WeaponSlot::Secondary,
                weapon: WeaponId(0),
                by_grip: vec![ChamberGrip {
                    grip: 0,
                    damage: [1.0; N_DT],
                    fire_rate: 2.0,
                    charge_time: None,
                    range: Some(10.0),
                }],
                magazine_by_tier,
                cc: 0.21,
                cd: 1.5,
                sc: 0.2,
            }],
            grips: vec![
                part(ModularPartKind::KitgunGrip),
                part(ModularPartKind::ZawGrip),
            ],
            loaders: vec![part(ModularPartKind::KitgunLoader)],
            strikes: vec![ZawStrike {
                key: StableKey(72),
                unique_name: UniqueName::new(
                    "/Lotus/Weapons/Ostron/Melee/ModularMelee01/Tip/TipOne",
                ),
                name: NameKey::new("balla"),
                damage: [2.0; N_DT],
                class: [MeleeClass::Dagger, MeleeClass::Staff],
                multi: [
                    ZawMulti {
                        base: 1.0,
                        leap: 2.0,
                        slide: 2.142857,
                        wall: 3.0,
                    },
                    ZawMulti {
                        base: 1.1,
                        leap: 2.0,
                        slide: 2.142857,
                        wall: 3.0,
                    },
                ],
                cc: 0.18,
                cd: 2.0,
                sc: 0.1,
                speed: 0.083,
                plague_split: true,
                image: None,
            }],
            links: vec![part(ModularPartKind::ZawLink)],
        },
        combo_tables: vec![
            ComboTable {
                id: ComboTableId(0),
                source: ComboSource::Stanceless(MeleeClass::Nikana),
                weapon_type: Some(MeleeClass::Nikana),
                combos: smallvec![Combo {
                    kind: ComboType::Neutral,
                    key: ComboKey::new("neutral"),
                    name: NameKey::new("combo.neutral"),
                    duration: Some(3.3),
                    from_weapon: true,
                    n_inputs: 4,
                    strikes: vec![strike(), strike()],
                }],
            },
            ComboTable {
                id: ComboTableId(1),
                source: ComboSource::Unique {
                    name: "Nepheri".into(),
                },
                weapon_type: None,
                combos: smallvec![],
            },
        ],
        frames: vec![Frame {
            id: FrameId(0),
            key: StableKey(80),
            unique_name: UniqueName::new("/Lotus/Powersuits/Jade/Jade"),
            name: NameKey::new("jade"),
            variant_of: None,
            polarities: smallvec![Polarity::Madurai],
            aura_polarities: smallvec![Polarity::Any, Polarity::Any],
            exilus_polarity: None,
            aura_slots: 2,
            abilities: smallvec![FrameAbility {
                name: NameKey::new("ability"),
                self_buff: Some(ItemId(8))
            }],
            helminth_host: Some(ItemId(8)),
            image: None,
        }],
        enemies: vec![Enemy {
            id: EnemyId(0),
            key: StableKey(9001),
            unique_name: UniqueName::new("wfd:enemy/grineer/lancer"),
            name: NameKey::new("wfd:enemy/grineer/lancer"),
            faction: FactionId(0),
            damage_faction: Some(FactionId(0)),
            base_level: 1,
            spawn_level: Some(10),
            health: 100.0,
            shield: 0.0,
            armor: 100.0,
            overguard: 0.0,
            eximus: EximusData {
                health: Some(300.0),
                shield: None,
                armor: Some(50.0),
                overguard: Some(12.0),
                off: false,
                og: true,
            },
            sp_health_bonus: 1800.0,
            sp_shield_bonus: 0.0,
            archimedea_health_bonus: 0.0,
            parts: smallvec![BodyPart {
                name: NameKey::new("Head"),
                column: PartColumn::WeakPoint,
                part_mult: 3.0,
                is_weak_point: true,
                is_head: true,
                headcrit: 2.0,
                tier_add: 1,
                crit_disabled: false,
                wp_bonus_ok: true,
                part_dr: 0.3,
                status_ok: true,
                no_armor: false,
                no_faction_mod: false,
                is_default: false,
            }],
            immune: StatusSet::VIR,
            caps,
            type_mod_override: Some([1.0; N_DT]),
            resist: Some([0.0; N_DT]),
            innate_dr: [0.0, 0.2, 0.9],
            status_dr: 0.5,
            dr_class: DrClass {
                radial_or_melee: 0.5,
                non_melee: 0.0,
                radial: 0.0,
            },
            dr_class_og: DrClass::default(),
            special_dr: Some(0),
            party_scaling: Some([1.0, 1.5, 2.0, 3.0]),
            defaults: EnemyDefaults {
                steel_path: true,
                eximus: false,
                mission: Some(MissionModifier::Empowered),
            },
            overguard_context: Some("Only in The Undercroft".into()),
            flags: EnemyFlags::BOSS | EnemyFlags::TOXIN_NULLIFY,
            scaling: 0,
            image: None,
        }],
        rules: rules(),
        names: NameTable(names),
        stable_keys: vec![(
            StableKey(1204),
            UniqueName::new("/Lotus/Weapons/Tenno/Rifle/BratonPrime"),
        )],
    }
}

#[test]
fn a_fully_populated_bundle_round_trips_in_both_encodings() {
    let b = bundle();
    json_round_trip(&b);
    postcard_round_trip(&b);
    assert_eq!(
        b.names.get(
            "de",
            &NameKey::new("/Lotus/Weapons/Tenno/Rifle/BratonPrime")
        ),
        Some("Braton Prime")
    );
    // The readable form uses the compact authoring shapes.
    let text = serde_json::to_string(&b.weapons[0].attacks[0].roles).unwrap();
    assert_eq!(text, r#"["normal","beam"]"#);
    let text = serde_json::to_string(&b.rules.toggles[0].default).unwrap();
    assert!(text.starts_with(r#"{"list""#), "{text}");
}

#[test]
fn infinite_fire_rate_survives_postcard() {
    // Lanka's FireRate is math.huge (06 §4.1); JSON has no infinity, the compact form keeps it.
    let mut a = attack("normal_attack", 0);
    a.fire.rate = f64::INFINITY;
    postcard_round_trip(&a);
}
