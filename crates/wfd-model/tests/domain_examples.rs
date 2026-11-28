//! Acceptance of T01a (work-breakdown.md): every TOML block of domain-model.md §14-§16 parses into the authoring
//! types; the §16.13 one-line encodings are expressible; `load_curated_dir` loads a tree and reports bad files.
//!
//! The blocks are read from `docs/architecture/domain-model.md` at test time, so the test follows the document.

use std::path::{Path, PathBuf};
use wfd_model::curated::{
    CuratedItemFile, CuratedKind, IncarnonFamilyFile, MultGroupsFile, RivenTraitsFile,
    load_curated_dir, load_curated_files, parse_toml,
};
use wfd_model::*;

fn doc() -> String {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/architecture/domain-model.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Fenced blocks of a language, with the heading they appear under.
fn fenced_blocks(text: &str, lang: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut heading = String::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.starts_with('#') {
            heading = line.trim_start_matches('#').trim().to_owned();
        } else if let Some(rest) = line.strip_prefix("**") {
            // Bold example labels: "**16.1 Serration** (…)".
            if let Some((label, _)) = rest.split_once("**")
                && label.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                heading = label.trim().to_owned();
            }
        }
        if line.trim() == format!("```{lang}") {
            let mut body = String::new();
            for l in lines.by_ref() {
                if l.trim() == "```" {
                    break;
                }
                body.push_str(l);
                body.push('\n');
            }
            out.push((heading.clone(), body));
        }
    }
    out
}

/// The section number of a heading such as "14. Multiplier groups" or "15.1 File header".
fn section(heading: &str) -> f64 {
    heading
        .split_whitespace()
        .next()
        .and_then(|n| n.trim_end_matches('.').parse().ok())
        .unwrap_or(0.0)
}

/// Parses one block into the type its shape names; returns a short description.
fn parse_block(heading: &str, body: &str) -> String {
    let path = PathBuf::from(format!("domain-model.md § {heading}"));
    if body.contains("[[group]]") {
        let f: MultGroupsFile = parse_toml(&path, body).unwrap_or_else(|e| panic!("{e}"));
        assert!(!f.group.is_empty());
        format!("mult groups ({})", f.group.len())
    } else if body.contains("[[trait]]") {
        let f: RivenTraitsFile = parse_toml(&path, body).unwrap_or_else(|e| panic!("{e}"));
        assert!(!f.traits.is_empty());
        format!("riven traits ({})", f.traits.len())
    } else if body.contains("\nfamily = ")
        || body.starts_with("family = ")
        || body.contains("[[perk]]")
    {
        let f: IncarnonFamilyFile = parse_toml(&path, body).unwrap_or_else(|e| panic!("{e}"));
        format!("incarnon family {} ({} perks)", f.family, f.perks.len())
    } else {
        let f: CuratedItemFile = parse_toml(&path, body).unwrap_or_else(|e| panic!("{e}"));
        format!("item {} ({} effects)", f.name, f.effects.len())
    }
}

#[test]
fn every_toml_block_of_sections_14_to_16_parses() {
    let text = doc();
    let blocks: Vec<_> = fenced_blocks(&text, "toml")
        .into_iter()
        .filter(|(h, _)| (14.0..17.0).contains(&section(h)))
        .collect();
    // §14 mult groups, §15.1 header, §16.1-16.12 (16.6 is the riven-trait table).
    assert_eq!(
        blocks.len(),
        14,
        "TOML blocks found: {:?}",
        blocks.iter().map(|b| &b.0).collect::<Vec<_>>()
    );
    for (heading, body) in &blocks {
        let what = parse_block(heading, body);
        eprintln!("{heading}: {what}");
    }
}

#[test]
fn json_riven_of_16_6_parses() {
    let text = doc();
    let blocks: Vec<_> = fenced_blocks(&text, "json")
        .into_iter()
        .filter(|(h, _)| h.starts_with("16"))
        .collect();
    assert_eq!(blocks.len(), 1);
    let entry: SlotEntry = serde_json::from_str(&blocks[0].1).unwrap();
    assert_eq!(entry.slot, SlotId::Gen(7));
    match &entry.content {
        SlotContent::Riven(r) => {
            assert_eq!(r.class, RivenClass::Pistol);
            assert_eq!(r.traits.len(), 4);
            assert_eq!(&*r.traits[3].trait_id, "zoom");
            assert_eq!(r.traits[3].value, -0.793);
        }
        other => panic!("{other:?}"),
    }
    let back = serde_json::to_string(&entry).unwrap();
    assert_eq!(serde_json::from_str::<SlotEntry>(&back).unwrap(), entry);
}

fn item(heading_prefix: &str) -> CuratedItemFile {
    let text = doc();
    let (h, body) = fenced_blocks(&text, "toml")
        .into_iter()
        .find(|(h, _)| h.starts_with(heading_prefix))
        .unwrap_or_else(|| panic!("no block under {heading_prefix}"));
    parse_toml(&PathBuf::from(h), &body).unwrap()
}

#[test]
fn encoded_examples_have_the_documented_meaning() {
    // 16.1 Serration: base-damage bucket, linear rank curve bound to n1%.
    let serration = item("16.1");
    assert_eq!(serration.kind, CuratedKind::WeaponMod);
    assert_eq!(serration.status, EffectStatus::Verified);
    let e = &serration.effects[0];
    assert_eq!(e.target, Target::Stat(StatKey::BaseDmg));
    assert_eq!(
        e.value,
        ValueExpr::Ranked {
            max: 1.65,
            rank: RankCurve::Linear,
            bind: Some(Binding("n1%".into()))
        }
    );

    // 16.2 Galvanized Aptitude: CO effect with stacks on kill.
    let ga = item("16.2");
    let co = ga.effects.iter().find(|e| &*e.id == "co").unwrap();
    assert_eq!(co.target, Target::Stat(StatKey::CoPerStatus));
    assert_eq!(co.per.as_slice(), &[Per::Stack]);
    assert!(
        matches!(&co.when, Cond::Trigger { trigger: TriggerId::OnKill, chance, .. } if *chance == 1.0)
    );
    let stacks = co.stacks.as_ref().unwrap();
    assert_eq!(
        (stacks.max, stacks.duration, stacks.stack_timer),
        (2, 20.0, StackTimer::SharedLoseOne)
    );
    assert_eq!(stacks.bind_max, Some(Binding("n4".into())));

    // 16.4 Primary Deadhead: item-level scope, two effects.
    let dh = item("16.4");
    assert_eq!(
        dh.scope.as_ref().unwrap().weapon.slots.as_slice(),
        &[WeaponSlot::Primary]
    );
    assert_eq!(dh.effects[1].target, Target::Stat(StatKey::HsMult));

    // 16.5 Hunter Munitions: proc_on_event with an exclusive group.
    let hm = item("16.5");
    match &hm.effects[0].target {
        Target::Mechanic(Mechanic::ProcOnEvent(p)) => {
            assert_eq!(p.on, ProcEvent::CritInstance);
            assert_eq!(p.proc, ProcSel::Status(StatusType::SLA));
            assert_eq!(p.exclusive_group.as_deref(), Some("forced_slash"));
        }
        other => panic!("{other:?}"),
    }

    // 16.7 Roar: buff with a percent parameter and a helper.
    let roar = item("16.7");
    assert_eq!(roar.kind, CuratedKind::Buff);
    assert_eq!(roar.toggle, Some(ToggleId::new("T-13")));
    assert_eq!(
        roar.params[0].kind,
        ParamKind::Percent {
            min: 0.0,
            max: 99.99
        }
    );
    assert_eq!(roar.params[0].default, ParamValue::Num(0.30));
    assert_eq!(roar.effects[0].target, Target::Faction(FactionSel::Any));
    assert_eq!(
        roar.effects[0].value,
        ValueExpr::Param(ParamKey::new("value"))
    );

    // 16.8 Xata's Whisper: extra hit.
    let xata = item("16.8");
    match &xata.effects[0].target {
        Target::Mechanic(Mechanic::ExtraHit(x)) => {
            assert_eq!(
                (x.element, x.part, x.status),
                (DamageType::VOI, XhPart::Twice, XhStatus::RollWeaponSc)
            );
            assert_eq!(
                x.triggers.as_slice(),
                &[XhTrigger::PelletHit, XhTrigger::BlastDetonation]
            );
        }
        other => panic!("{other:?}"),
    }

    // 16.9 Violet shard: linear value over shard counts, innate element order.
    let violet = item("16.9");
    let e = &violet.effects[0];
    assert_eq!(e.target, Target::Element(ElemSel::Type(DamageType::ELE)));
    assert_eq!(e.order, Some(ElemOrder::Innate));
    match &e.value {
        ValueExpr::Linear { base, add } => {
            assert_eq!(**base, ValueExpr::Const(0.30));
            assert_eq!(add[0].coeff, 0.10);
            assert_eq!(
                add[0].per.as_slice(),
                &[Per::ShardCount(
                    [ShardColor::Crimson, ShardColor::Azure, ShardColor::Violet]
                        .into_iter()
                        .collect()
                )]
            );
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        e.scope.weapon.slots.as_slice(),
        &[WeaponSlot::Primary, WeaponSlot::ArchGun]
    );

    // 16.10 Furis line: per-variant values and the per-weapon perk rows CDM0c and θ_P (unconditional; P3-6).
    let text = doc();
    let (_, body) = fenced_blocks(&text, "toml")
        .into_iter()
        .find(|(h, _)| h.starts_with("16.10"))
        .unwrap();
    let furis: IncarnonFamilyFile = parse_toml(Path::new("16.10"), &body).unwrap();
    assert_eq!(furis.perks.len(), 3);
    assert_eq!(furis.perks[0].effects[0].scope.form, FormSel::Normal);
    // Prelude of Might: CDM0c and θ_P of 02 E-2.5 as their own registry rows, unconditional (the crit stage applies
    // the strict CC_i < θ_P test per instance; contract review ENG-R7, NEW-CR-8).
    let prelude = &furis.perks[2].effects;
    assert_eq!(prelude.len(), 2);
    assert_eq!(prelude[0].target, Target::Stat(StatKey::CdBaseCond));
    assert_eq!(prelude[0].value, ValueExpr::Const(3.0));
    assert_eq!(prelude[1].target, Target::Stat(StatKey::CdBaseCondThr));
    assert_eq!(prelude[1].value, ValueExpr::Const(0.40));
    assert!(prelude.iter().all(|e| e.when.is_always()));

    // 16.12 Arcane Avenger: external trigger with chance and a single refreshing stack.
    let avenger = item("16.12");
    assert!(matches!(
        &avenger.effects[0].when,
        Cond::Trigger { trigger: TriggerId::OnDamaged, chance, bind_chance: Some(_), .. } if *chance == 0.21
    ));
    assert_eq!(
        avenger.effects[0].stacks.as_ref().unwrap().refresh,
        Refresh::Refresh
    );
}

/// The one-line encodings of domain-model.md §16.13, written as `[[effect]]` tables (one per row).
const TABLE_16_13: &[(&str, &str)] = &[
    (
        "Primed Chamber",
        r#"mult = "chamber"
value = 1.00
when = { shot = "first_of_magazine" }"#,
    ),
    (
        "Synth Charge",
        r#"mult = "synth_charge"
value = 2.00
when = { shot = { last_of_magazine = { min_base_magazine = 6 } } }
scope = { attacks = { none_of = ["beam", "incarnon_form", "whole_magazine"] } }"#,
    ),
    (
        "Eclipse",
        r#"mult = "eclipse"
value = { param = "value" }"#,
    ),
    (
        "Nourish",
        r#"element = "VIR"
order = "non_combining"
value = { param = "value" }"#,
    ),
    (
        "Flat crit chance",
        r#"stat = "cc_abs"
value = { param = "value" }"#,
    ),
    (
        "Phoenix Talons",
        r#"stat = "phoenix_talons"
value = 0.30"#,
    ),
    (
        "Secondary Shiver",
        r#"stat = "co_per_cold_stack"
value = 0.45
scope = { weapon = { slots = ["secondary"] } }
class = ["a"]"#,
    ),
    (
        "Kunai Incarnon perk",
        r#"stat = "co_flat"
value = 1.0
when = { target = { health_below = 0.5 } }"#,
    ),
    (
        // ΣCD_abs per D-067 (amended) and 02 §4.3 (OQ-406 closed); T01d brought §16.13 in line.
        "Spiteful Defilement",
        r#"stat = "cd_abs"
value = 1.0
when = { target = { distinct_below = 3 } }"#,
    ),
    (
        "Vigilante set",
        r#"stat = "tier_raise"
value = 0.05
per = ["set_piece"]"#,
    ),
    (
        "Gladiator set",
        r#"stat = "cc_rel"
value = 0.10
per = ["set_piece", "combo_tier_above_one"]"#,
    ),
    (
        "Primary Acuity WP",
        r#"stat = "wp_dmg"
value = 3.5"#,
    ),
    (
        "Primary Acuity CC",
        r#"stat = "cc_rel"
value = 3.5
scope = { hit = { body_part = ["weak_point"] } }"#,
    ),
    (
        "Primary Acuity lock",
        r#"stat = "lock_multishot"
value = 1.0"#,
    ),
    (
        "Semi-Rifle Cannonade",
        r#"stat = "lock_fire_rate"
value = 1.0"#,
    ),
    (
        "Killing Blow",
        r#"stat = "base_dmg"
value = 1.20
scope = { attacks = { any_of = ["heavy"] } }"#,
    ),
    (
        // Ruling C-37 (contract review M-5): the rank factor b_r of 02 E-2.5a; the crit stage computes doughty.
        "Melee Doughty",
        r#"stat = "doughty_rank"
value = { max = 1.0, rank = { table = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0] } }"#,
    ),
    (
        // §16.13 plus the X-RANK duration table of 09 E-9.13 (3, 4, 5, 6, 8, 10 s).
        "Primary Plated Round",
        r#"stat = "base_dmg"
value = 0.15
per = [{ resolved = { stat = "magazine_normal_form", curve = { pow = { scale = 5.0, exp = 0.5 } } } }, "reloaded_fraction"]
when = { trigger = "on_reload" }
stacks = { max = 1, duration = 10.0, duration_rank = { table = [3.0, 4.0, 5.0, 6.0, 8.0, 10.0] } }"#,
    ),
    (
        // 04 §5.12 after its second tail round (D-410, TV-04-47): an on-hit buff from `on_plain_hit`, not a gate.
        "Laetum Overwhelming Attrition",
        r#"stat = "base_dmg"
value = 4.0
per = ["stack"]
when = { trigger = "on_plain_hit", hit_unit = "pellet" }
stacks = { max = 3, duration = 10.0, stack_timer = "shared_lose_one" }
class = ["b"]"#,
    ),
    (
        // T-32 (02 E-2.10): a `Spot`-stage source naming the params of L_σ and ν_σ.
        "Sonar",
        r#"mult = "sonar"
value = { param = "value" }
per = ["strength"]
spot = { location = "spot_location", nodes = "spot_nodes" }"#,
    ),
    (
        // 04 `hits_within(4, 0.05)` (§5.3).
        "Final Tap",
        r#"stat = "base_dmg"
value = 1.0
when = { trigger = { count_within = { of = "on_hit", n = 4, window = 0.05 } }, hit_unit = "pellet" }
stacks = { max = 1, duration = 0.2 }
class = ["b"]"#,
    ),
    (
        "Phenmor Lingering Judgement",
        r#"stat = "hs_mult"
value = 0.50
when = { trigger = { count_within = { of = "on_headshot", n = 2, window = 2.0 } } }
stacks = { max = 1, duration = 8.0 }"#,
    ),
    (
        // One stack per light, slide, aerial or wall input (04 §5.4, 07 §7.5; contract review DC-1).
        "Melee Animosity",
        r#"mechanic = "counter_bonus"
value = 0.42
[params]
stat = "cc_rel"
per_event = "any_input"
max = 10
reset_on = "heavy"
applies_to = { attacks = { any_of = ["heavy"] } }"#,
    ),
    (
        "Power Throw",
        r#"mechanic = "counter_bonus"
value = 1.00
[params]
stat = "base_dmg_universal"
per_event = "throw"
max = 3
reset_on = "other_input""#,
    ),
    (
        // §16.13: `stat = "vuln_add"`, value 0.65 per application (`per = ["stack"]`, stacks = user count with no
        // limit, default 1, class (c)), `vuln = { types = ["HEA"] }`. 04 §5.5: "permanent, no limit" (ruling C-4:
        // `unbounded`, `max` = the default count).
        "Magus Accelerant",
        r#"stat = "vuln_add"
value = 0.65
per = ["stack"]
when = { trigger = "on_ability_cast" }
stacks = { max = 1, unbounded = true }
class = ["c"]
vuln = { types = ["HEA"] }"#,
    ),
    (
        // §16.13: `stat = "vuln_add"`, value 2.00, `vuln = { types = { param = "element" } }`, class (c), 6 s.
        "Theorem Contagion",
        r#"stat = "vuln_add"
value = 2.00
when = { trigger = "on_kill" }
stacks = { max = 1, duration = 6.0 }
class = ["c"]
vuln = { types = { param = "element" } }"#,
    ),
    (
        "Chromatic Blade",
        r#"mechanic = "convert_damage"
value = 1.0
[params]
from = "all"
to = { param = "element" }"#,
    ),
    (
        "Reactive Storm",
        r#"mechanic = "convert_damage"
value = 1.0
[params]
from = "all"
to = "target_weakness""#,
    ),
    (
        "Sentient Incision",
        r#"element = "target_weakness"
order = "non_combining"
value = 0.40"#,
    ),
    (
        "Corrosive Projection",
        r#"stat = "strip_aura"
value = 0.18"#,
    ),
    (
        "Emerald shard Corrosion cap",
        r#"stat = "stack_cap.COR"
value = 2
tauforged_values = [3]"#,
    ),
    (
        "Secondary Fortifier",
        r#"mult = "fortifier"
value = 7.0"#,
    ),
    (
        "Mecha Empowered",
        r#"mult = "mecha"
value = 1.0
when = { target = "marked" }"#,
    ),
    (
        "Melee Careen",
        r#"mult = "careen"
value = 1.5
when = { target = "frozen" }"#,
    ),
    (
        "Devouring Attrition",
        r#"mechanic = "random_multiplier"
value = 0.5
[params]
factor = 21.0
on = { crit_tier_eq = 0 }"#,
    ),
    (
        "Galvanized Scope",
        r#"stat = "cc_rel"
value = 1.20
per = ["stack"]
when = { trigger = "while_aiming" }
stacks = { max = 2, duration = 12.0, stack_timer = "independent", gain_on = { trigger = "on_weak_point_hit" } }"#,
    ),
    (
        "Melee Duplicate",
        r#"mechanic = "repeat_hit"
value = 1.0
[params]
on = { crit_tier_eq = 1 }"#,
    ),
    (
        "Custom p.b",
        r#"stat = "co_per_status"
value = { param = "value" }"#,
    ),
    (
        "Custom p.x",
        r#"mult = "custom_vuln"
value = { param = "value" }"#,
    ),
    (
        "Toxic Lash",
        r#"mechanic = "extra_hit"
value = { param = "value" }
[params]
element = "TOX"
part = "once"
status = "forced"
melee_factor = 2.0"#,
    ),
    // Rows added by the contract review (CONTRACTS.md, "Contract review fixes (model-core)").
    (
        "Volatile Rebound",
        r#"stat = "glaive_volatile_chance"
value = { max = 1.0, rank = "linear" }"#,
    ),
    (
        "Argon Scope",
        r#"stat = "cc_rel"
value = 1.35
per = ["stack"]
when = { trigger = "while_aiming" }
stacks = { max = 1, duration = 9.0, gain_on = { trigger = "on_weak_point_hit" } }"#,
    ),
    (
        "Double Tap",
        // 04 §3.2 `1 + 0.20·min(20, H − 1)`, H including this shot's hitting pellets (TV-04-25: ×1.4, ×2.0, ×2.6).
        r#"mult = "double_tap"
value = { base = -0.20, add = [{ coeff = 0.20, per = ["stack"] }] }
when = { trigger = "on_hit", hit_unit = "pellet" }
stacks = { max = 21, duration = 2.0, stack_timer = "shared_lose_all", read = "after_shot", reset_on = { on = "on_miss" } }"#,
    ),
    (
        "Piercing Navigator",
        r#"stat = "cc_abs"
value = 0.50
per = ["stack"]
stacks = { max = 5, default_count = 0 }"#,
    ),
    (
        "Spring-Loaded Broadhead",
        r#"mult = "spring_loaded_broadhead"
value = 0.40
scope = { hit = { min_distance = 15.0 } }"#,
    ),
    (
        "Master's Edge",
        r#"stat = "base_dmg"
value = 0.60
scope = { hit = { flags = ["tennokai"] } }"#,
    ),
    (
        "Opportunity's Reach",
        r#"stat = "tennokai_window"
value = 4.0"#,
    ),
    (
        "Discipline's Merit",
        r#"stat = "tennokai_every"
value = 4.0"#,
    ),
    (
        "Primary Compression (multiplies)",
        r#"mult = "primary_compression"
value = { max = 0.8, rank = { table = [0.4, 0.48, 0.56, 0.64, 0.72, 0.8] } }
per = [{ resolved = { stat = "radius", curve = { pow = { scale = 1.0, exp = 1.0 } } } }, { compression_eff = "multiplies" }]
when = { trigger = "while_aiming" }"#,
    ),
    (
        "Electric Shield",
        r#"element = "ELE"
order = "non_combining"
value = 0.50
dot_scaling = false"#,
    ),
    (
        "Grave Spirit vs Doomed",
        r#"stat = "cd_rel"
value = 1.00
per = ["strength"]
when = { target = { debuffed = "doomed" } }"#,
    ),
    (
        "Eximus Advantage",
        r#"stat = "base_dmg"
value = 6.0
when = { all = [{ trigger = "on_headshot" }, { target = "eximus" }] }
stacks = { max = 1, duration = 10.0 }"#,
    ),
    (
        "Pain Points",
        r#"stat = "wp_dmg"
value = 0.60
per = ["stack"]
when = { all = [{ trigger = "on_weak_point_hit" }, { attack = { roles = { any_of = ["normal"] } } }] }
stacks = { max = 10, duration = 5.0, stack_timer = "shared_lose_all" }"#,
    ),
    (
        "Combat Reload",
        r#"stat = "reload_speed"
value = 1.20
when = { trigger = { count_within = { of = "on_headshot", n = 5, window = 0.0 } }, hit_unit = "pellet" }
stacks = { max = 1, duration = 3.0 }
class = ["b"]"#,
    ),
    (
        "Mecha Empowered uptime",
        r#"mult = "mecha"
value = 1.5
when = { target = "marked" }
uptime = { on = { scalar = "set_piece", values = [3.0, 6.0, 9.0, 12.0] }, period = { scalar = "set_piece", values = [60.0, 45.0, 30.0, 15.0] } }"#,
    ),
    (
        "Melee Afflictions",
        r#"mechanic = "stack_present_statuses"
value = { max = 6.0, rank = "linear" }
[params]
statuses = ["KND", "LIF"]
ragdoll = true"#,
    ),
    (
        "Secondary Encumber",
        r#"mechanic = "proc_on_event"
value = 0.24
[params]
on = "proc"
proc = { random = ["IMP", "PUN", "SLA", "HEA", "COL", "ELE", "TOX", "BLA", "COR", "MAG", "VIR", "GAS", "RAD"] }
limit = "per_event""#,
    ),
    (
        "Primary Debilitate",
        r#"mechanic = "proc_after_ticks"
value = 1.0
[params]
combined = ["BLA", "RAD", "GAS", "MAG", "VIR", "COR"]
stacks_at_least = 10"#,
    ),
    (
        "Gotva Prime passive",
        r#"stat = "cc_set"
value = 3.0
when = { trigger = "on_any_status", chance = 0.15 }
stacks = { max = 1, consume_on = "next_shot" }"#,
    ),
    (
        "Meticulous Aim",
        r#"stat = "part_body_pen"
value = 0.45"#,
    ),
    (
        "Shattering Impact",
        r#"stat = "strip_flat_per_hit"
value = { max = 6.0, rank = "linear" }
scope = { weapon = { slots = ["melee"] } }"#,
    ),
    (
        "Polarize",
        r#"stat = "strip_points"
value = 400.0
per = ["strength"]"#,
    ),
    (
        "Sharpened Claws",
        r#"stat = "strip_companion"
value = 1.20"#,
    ),
    (
        "Resupply",
        r#"mechanic = "extra_hit"
value = { param = "value" }
scope = { weapon = { slots = ["primary", "secondary"], none_traits = ["battery"] }, form = "normal" }
[params]
element = "HEA"
status = "forced""#,
    ),
    (
        "Critical Precision",
        r#"stat = "cc_rel"
value = 0.10
per = ["stack"]
when = { trigger = "on_headshot" }
stacks = { max = 50 }
scope = { attacks = { triggers = ["burst"] } }"#,
    ),
    (
        "Prismatic Gem SD",
        r#"stat = "status_dur"
value = 1.00
per = ["ability_duration"]"#,
    ),
    (
        "Enraged",
        r#"stat = "base_dmg"
value = 3.50
scope = { weapon = { weapons = ["<uniqueName of Hysteria>"] } }"#,
    ),
    (
        "Custom heatInheritFaction",
        r#"stat = "heat_inherit_faction"
value = { param = "value" }"#,
    ),
    (
        "Arcane Pistoleer",
        r#"stat = "infinite_ammo"
value = 1.0
when = { trigger = "on_weak_point_kill", chance = 0.6 }
stacks = { max = 1, duration = 12.0 }"#,
    ),
    (
        "Dante passive",
        r#"mult = "dante_passive"
value = 0.5"#,
    ),
    (
        "Internal Bleeding",
        r#"mechanic = "proc_on_event"
value = 0.35
[params]
on = "proc"
on_status = "IMP"
proc = "SLA"
chance_if = [{ fire_rate_below = 2.5, factor = 2.0 }]
exclusive_group = "forced_slash""#,
    ),
    (
        "Blood Rush heavy doubling",
        r#"stat = "cc_rel"
value = 0.40
per = ["combo_tier_above_one"]
scope_factor = [{ when = { attacks = { any_of = ["heavy"] } }, factor = 2.0 }]"#,
    ),
    (
        "Nightwatch Napalm",
        r#"mechanic = "spawn_attack"
value = 1.0
[params]
on = "detonation"
template = { base = { HEA = 150.0 }, sc = 0.68, forced = ["HEA"], proc_duration = 3.0, lingering = { effect_duration = 6.0, tick_rate = 1.0 } }"#,
    ),
];

#[test]
fn every_row_of_table_16_13_is_expressible() {
    for (name, body) in TABLE_16_13 {
        let e: EffectDef = toml::from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
        // A declared class must equal the derived one (domain-model.md §11.1, §12.1).
        if let Some(declared) = e.class {
            assert_eq!(
                e.derived_class(),
                declared,
                "{name}: declared vs derived class"
            );
        }
        // Readable and compact round trips.
        let t = toml::to_string(&e).unwrap();
        assert_eq!(
            toml::from_str::<EffectDef>(&t).unwrap(),
            e,
            "{name}: TOML round trip\n{t}"
        );
        let j = serde_json::to_string(&e).unwrap();
        assert_eq!(
            serde_json::from_str::<EffectDef>(&j).unwrap(),
            e,
            "{name}: JSON round trip"
        );
        let p = postcard::to_stdvec(&e).unwrap();
        assert_eq!(
            postcard::from_bytes::<EffectDef>(&p).unwrap(),
            e,
            "{name}: postcard round trip"
        );
    }
}

fn row(name: &str) -> EffectDef {
    let (_, body) = TABLE_16_13
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("no row {name}"));
    toml::from_str(body).unwrap()
}

#[test]
fn gain_events_filter_by_attack_role_and_by_pellets_of_one_shot() {
    // Contract review DC-10 (pass 3): Pain Points gains only on weak-point hits of primary fire (an attack-role filter
    // at the event), Combat Reload on 5 headshot pellets of one shot (`count_within` with window 0, per pellet).
    let pp = row("Pain Points");
    match &pp.when {
        Cond::All(cs) => {
            assert!(matches!(
                &cs[0],
                Cond::Trigger {
                    trigger: TriggerId::OnWeakPointHit,
                    ..
                }
            ));
            match &cs[1] {
                Cond::Attack(AttackPred::Roles(f)) => {
                    assert_eq!(f.any_of, AttackRoles::NORMAL);
                    assert!(f.none_of.is_empty());
                }
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
    let cr = row("Combat Reload");
    match &cr.when {
        Cond::Trigger {
            trigger: TriggerId::CountWithin { of, n, window },
            hit_unit,
            ..
        } => {
            assert_eq!((**of).clone(), TriggerId::OnHeadshot);
            assert_eq!((*n, *window, *hit_unit), (5, 0.0, HitUnit::Pellet));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn vulnerability_rows_spread_over_the_vuln_add_row() {
    // domain-model.md §16.13, §17: `vuln_add` with a `vuln` mask (54 slots); a mask without `layers` is health only
    // (05 §13, 04 §4.2, §5.7: Magus and Theorem are HP only).
    let magus = row("Magus Accelerant");
    assert_eq!(magus.target, Target::VulnAdd);
    let mask = magus.vuln.as_ref().unwrap();
    assert_eq!(mask.types, TypeSel::Set(DamageTypeSet::HEA));
    assert_eq!(mask.layers, LayerSet::HP);
    assert_eq!(magus.derived_class(), ClassSet::C);
    let theorem = row("Theorem Contagion");
    assert_eq!(theorem.target, Target::VulnAdd);
    assert_eq!(
        theorem.vuln.as_ref().unwrap().types,
        TypeSel::Param(ParamKey::new("element"))
    );
    assert_eq!(theorem.vuln.as_ref().unwrap().layers, LayerSet::HP);
    // Readable form keeps the row name.
    let t = toml::to_string(&theorem).unwrap();
    assert!(t.contains("stat = \"vuln_add\""), "{t}");

    // An explicit mask and the every-type default (04 §4.2 `type_mask` default `all`).
    let all: EffectDef = toml::from_str(
        "stat = \"vuln_add\"\nvalue = 1.0\nvuln = { layers = [\"OG\", \"SH\", \"HP\"] }",
    )
    .unwrap();
    let mask = all.vuln.unwrap();
    assert_eq!(mask.types, TypeSel::Set(DamageTypeSet::all()));
    assert_eq!(mask.layers, LayerSet::all());
    // One slot without a mask is still a plain stat target.
    let one: EffectDef = toml::from_str("stat = \"vuln_add.SH.MAG\"\nvalue = 0.5").unwrap();
    assert!(matches!(one.target, Target::Stat(StatKey::VulnAdd(_))));
    // A TargetVuln multiplier group may carry a mask.
    let group: EffectDef =
        toml::from_str("mult = \"custom_vuln\"\nvalue = 1.0\nvuln = { types = [\"COR\"] }")
            .unwrap();
    assert!(group.vuln.is_some());

    for (bad, why) in [
        ("stat = \"vuln_add\"\nvalue = 0.65", "the row needs a mask"),
        (
            "stat = \"vuln_add.HP.HEA\"\nvalue = 0.65\nvuln = { types = [\"PUN\"] }",
            "a single slot takes no mask (it could contradict it)",
        ),
        (
            "stat = \"vuln_add.HP.HEA\"\nvalue = 0.65\nvuln = { types = [\"HEA\"] }",
            "a single slot takes no mask even when it agrees",
        ),
        (
            "stat = \"cc_rel\"\nvalue = 1.0\nvuln = { types = [\"HEA\"] }",
            "a mask only belongs to vulnerabilities",
        ),
        (
            "stat = \"vuln_add.HP\"\nvalue = 1.0",
            "incomplete slot name",
        ),
    ] {
        assert!(toml::from_str::<EffectDef>(bad).is_err(), "{why}: {bad}");
    }
}

#[test]
fn rank_dependent_fields_have_rank_curves() {
    // 09 E-9.13 / X-RANK: Plated Round duration table, Secondary Enervate `r + 1` big crits, trigger chances.
    let pr = row("Primary Plated Round");
    let st = pr.stacks.as_ref().unwrap();
    assert_eq!(st.duration, 10.0);
    assert_eq!(
        st.duration_rank,
        Some(RankCurve::Table(vec![3.0, 4.0, 5.0, 6.0, 8.0, 10.0]))
    );
    assert_eq!(st.max_rank, None);
    let enervate: EffectDef = toml::from_str(
        "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cc_abs\"\nper_event = \"hit\"\nunbounded = true\nreset_on = { crit_tier_at_least = 2 }\nreset_after = 6\nreset_after_rank = \"linear\"",
    )
    .unwrap();
    match &enervate.target {
        Target::Mechanic(Mechanic::CounterBonus(p)) => {
            assert_eq!(p.reset_after, Some(6));
            assert_eq!(p.reset_after_rank, Some(RankCurve::Linear));
            assert_eq!(p.max_rank, None);
        }
        other => panic!("{other:?}"),
    }
    let back = toml::to_string(&enervate).unwrap();
    assert_eq!(toml::from_str::<EffectDef>(&back).unwrap(), enervate);
    assert!(
        toml::from_str::<EffectDef>(
            "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cc_abs\"\nper_event = \"hit\"\nmax = 100\nreset_after_rank = \"linear\""
        )
        .is_err(),
        "reset_after_rank without reset_after"
    );
    let avenger: EffectDef = toml::from_str(
        "stat = \"cc_rel\"\nvalue = 0.45\nwhen = { trigger = \"on_damaged\", chance = 0.21, chance_rank = \"linear\" }",
    )
    .unwrap();
    assert!(matches!(
        &avenger.when,
        Cond::Trigger {
            chance_rank: Some(RankCurve::Linear),
            ..
        }
    ));
    assert!(
        toml::from_str::<EffectDef>(
            "stat = \"cc_rel\"\nvalue = 0.45\nwhen = { chance_rank = \"linear\" }"
        )
        .is_err()
    );
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn load_curated_dir_loads_a_tree() {
    let text = doc();
    let blocks = fenced_blocks(&text, "toml");
    let find = |p: &str| {
        blocks
            .iter()
            .find(|(h, _)| h.starts_with(p))
            .unwrap()
            .1
            .clone()
    };
    let root = scratch("curated_ok");
    write(&root, "weapon_mod/rifle/serration.toml", &find("16.1"));
    write(
        &root,
        "weapon_mod/rifle/galvanized_aptitude.toml",
        &find("16.2"),
    );
    write(&root, "buff/ability/roar.toml", &find("16.7"));
    write(
        &root,
        "shard/violet/primary_electricity.toml",
        &find("16.9"),
    );
    write(&root, "incarnon/secondary/furis.toml", &find("16.10"));
    write(&root, "rules/mult_groups.toml", &find("14."));
    write(&root, "rules/riven_traits.toml", &find("16.6"));
    write(
        &root,
        "rules/melee/heavy_kappa.toml",
        "sources = [\"07 E-7.41\"]\n[[row]]\nweapon = \"/Lotus/Weapons/Tenno/Melee/Hirudo\"\nkappa = 0.5416666666666666\n",
    );
    write(&root, "overrides/attacks/ignored.toml", "anything = 1\n");
    // Files of other tasks at the top (pipeline.md §6.3 T19, §6.1 T01b) and hidden files are skipped.
    write(&root, "priority.toml", "anything = 1\n");
    write(&root, "schema.json", "{}\n");
    write(&root, "weapon_mod/.gitkeep", "");
    let tree = load_curated_dir(&root).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(tree.items.len(), 4);
    assert_eq!(tree.items[0].0, Path::new("buff/ability/roar.toml"));
    assert_eq!(tree.incarnon.len(), 1);
    assert_eq!(tree.rules.mult_groups.as_ref().unwrap().group.len(), 3);
    assert!(tree.rules.riven_traits.is_some());
    assert_eq!(tree.rules.melee.heavy_kappa.as_ref().unwrap().row.len(), 1);
    assert!(tree.rules.toggles.is_none());
}

#[test]
fn load_curated_dir_reports_every_bad_file() {
    let text = doc();
    let serration = fenced_blocks(&text, "toml")
        .into_iter()
        .find(|(h, _)| h.starts_with("16.1"))
        .unwrap()
        .1;
    let root = scratch("curated_bad");
    write(&root, "weapon_mod/rifle/ok.toml", &serration);
    write(&root, "aura/wrong_kind.toml", &serration);
    write(
        &root,
        "weapon_mod/rifle/unknown_field.toml",
        &format!("{serration}bogus = 1\n"),
    );
    write(
        &root,
        "weapon_mod/rifle/bad_stat.toml",
        &serration.replace("base_dmg", "base_dmgg"),
    );
    write(&root, "rules/not_a_table.toml", "x = 1\n");
    write(&root, "mods/elsewhere.toml", "x = 1\n");
    write(&root, "mods/other.toml", "x = 1\n");
    write(&root, "weapon_mod/rifle/backup.toml.bak", &serration);
    write(&root, "weapon_mod/rifle/README.md", "notes\n");
    write(
        &root,
        "weapon_mod/rifle/future_schema.toml",
        &serration.replace("schema = 1", "schema = 2"),
    );
    write(&root, "stray.toml", "x = 1\n");
    let errors = load_curated_dir(&root).unwrap_err();
    let paths: Vec<String> = errors
        .0
        .iter()
        .map(|e| e.path.display().to_string())
        .collect();
    assert_eq!(errors.0.len(), 9, "{errors}");
    for expected in [
        "aura/wrong_kind.toml",
        "weapon_mod/rifle/unknown_field.toml",
        "weapon_mod/rifle/bad_stat.toml",
        "weapon_mod/rifle/backup.toml.bak",
        "weapon_mod/rifle/README.md",
        "weapon_mod/rifle/future_schema.toml",
        "rules/not_a_table.toml",
        "mods",
        "stray.toml",
    ] {
        assert!(
            paths.iter().any(|p| p == expected),
            "missing {expected} in {paths:?}"
        );
    }
}

#[test]
fn load_curated_files_loads_an_in_memory_tree() {
    // The in-app refresh passes the embedded tree (pipeline.md §2 `InputTree::Embedded`) as (path, content) pairs.
    let text = doc();
    let blocks = fenced_blocks(&text, "toml");
    let find = |p: &str| {
        blocks
            .iter()
            .find(|(h, _)| h.starts_with(p))
            .unwrap()
            .1
            .clone()
    };
    let files = vec![
        (
            PathBuf::from("weapon_mod/rifle/serration.toml"),
            find("16.1"),
        ),
        (PathBuf::from("rules/mult_groups.toml"), find("14.")),
        (PathBuf::from("priority.toml"), "not even TOML [".to_owned()),
        (PathBuf::from("overrides/attacks/x.toml"), "[".to_owned()),
        (
            PathBuf::from("incarnon/secondary/furis.toml"),
            find("16.10"),
        ),
    ];
    let tree = load_curated_files(files.clone()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(tree.items.len(), 1);
    assert_eq!(tree.incarnon.len(), 1);
    assert!(tree.rules.mult_groups.is_some());

    // A second copy of a rule table, a path outside the tree and an unknown directory are errors.
    let mut bad = files;
    bad.push((PathBuf::from("rules/mult_groups.toml"), find("14.")));
    bad.push((PathBuf::from("../escape.toml"), find("16.1")));
    bad.push((PathBuf::from("rules/extra/x.toml"), "x = 1".to_owned()));
    let errors = load_curated_files(bad).unwrap_err();
    let msgs: Vec<String> = errors.0.iter().map(|e| e.to_string()).collect();
    assert_eq!(errors.0.len(), 3, "{errors}");
    assert!(
        msgs.iter().any(|m| m.contains("duplicate rule table")),
        "{msgs:?}"
    );
    assert!(
        msgs.iter().any(|m| m.starts_with("rules/extra:")),
        "{msgs:?}"
    );

    // A missing tree is one error naming the directory.
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no_such_curated_dir");
    let errors = load_curated_dir(&missing).unwrap_err();
    assert_eq!(errors.0.len(), 1);
    assert_eq!(errors.0[0].path, missing);
}

/// Ruling C-4 (CONTRACTS.md T01d): `max_stacks = none` of 04 §4.2. Event-driven counters without a cap until their
/// reset (Secondary Enervate, Hata-Satya) are `counter_bonus` with `unbounded = true`; user counts without a limit
/// (Magus Accelerant, 04 §5.5) are `stacks` with `unbounded = true`, `max` being the default count.
#[test]
fn unbounded_counters_and_user_counts() {
    let enervate: EffectDef = toml::from_str(
        "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cc_abs\"\nper_event = \"hit\"\nunbounded = true\nreset_on = { crit_tier_at_least = 2 }\nreset_after = 6\nhit_unit = \"shot\"",
    )
    .unwrap();
    match &enervate.target {
        Target::Mechanic(Mechanic::CounterBonus(p)) => {
            assert!(p.unbounded);
            assert_eq!(p.max, 0);
            assert_eq!(p.hit_unit, HitUnit::Shot);
        }
        other => panic!("{other:?}"),
    }
    let back = toml::to_string(&enervate).unwrap();
    assert!(
        back.contains("unbounded = true") && !back.contains("max ="),
        "{back}"
    );
    assert_eq!(toml::from_str::<EffectDef>(&back).unwrap(), enervate);
    // Hata-Satya: per pellet, reset by a reload.
    let hata: EffectDef = toml::from_str(
        "mechanic = \"counter_bonus\"\nvalue = 0.012\n[params]\nstat = \"cc_rel\"\nper_event = \"hit\"\nunbounded = true\nreset_on = \"reload\"",
    )
    .unwrap();
    assert!(matches!(
        &hata.target,
        Target::Mechanic(Mechanic::CounterBonus(CounterBonusParams {
            reset_on: Some(CounterReset::Reload),
            unbounded: true,
            ..
        }))
    ));
    for bad in [
        // `unbounded` with a cap, without a reset, or with a per-rank cap.
        "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cc_abs\"\nper_event = \"hit\"\nunbounded = true\nmax = 10\nreset_on = \"heavy\"",
        "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cc_abs\"\nper_event = \"hit\"\nunbounded = true",
        "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cc_abs\"\nper_event = \"hit\"\nunbounded = true\nreset_on = \"heavy\"\nmax_rank = \"linear\"",
        // Neither a cap nor `unbounded`.
        "mechanic = \"counter_bonus\"\nvalue = 0.1\n[params]\nstat = \"cc_abs\"\nper_event = \"hit\"\nreset_on = \"heavy\"",
    ] {
        assert!(toml::from_str::<EffectDef>(bad).is_err(), "accepted {bad}");
    }
    // Magus Accelerant: user count of applications, default 1, no limit.
    let magus: EffectDef = toml::from_str(
        "stat = \"vuln_add\"\nvalue = 0.65\nper = [\"stack\"]\nvuln = { types = [\"HEA\"] }\nstacks = { max = 1, unbounded = true }",
    )
    .unwrap();
    let st = magus.stacks.as_ref().unwrap();
    assert!(st.unbounded && st.max == 1);
    assert_eq!(
        toml::from_str::<EffectDef>(&toml::to_string(&magus).unwrap()).unwrap(),
        magus
    );
    // Melee Animosity-style trigger counted per melee input (04 §4.1 `hit_unit = input`).
    let per_input: EffectDef = toml::from_str(
        "stat = \"cc_rel\"\nvalue = 0.42\nwhen = { trigger = \"on_hit\", hit_unit = \"input\" }",
    )
    .unwrap();
    assert!(matches!(
        &per_input.when,
        Cond::Trigger {
            hit_unit: HitUnit::Input,
            ..
        }
    ));
}
