//! Serde round trip of every API type, and the generated TypeScript accepts exactly what serde writes.
//!
//! Each root type of `codegen::typescript_roots` has one or more fully populated samples. For every sample the test
//! (1) parses it (unknown fields are errors, so nothing in a sample is silently ignored), (2) serializes it, parses
//! that again and checks equality and a stable JSON form, and (3) checks the written JSON against the generated
//! declarations. Coverage: every object literal reachable from the roots in the generated TypeScript must have been
//! matched by some sample with every one of its fields present (every optional field set, every data-carrying enum
//! arm used). Documents (`CalcInput`, tested by T01a) are exempt.

use super::ts::{Checker, Field, generated_defs, normalize};
use crate::StableKey;
use crate::api::*;
use crate::scenario::CalcInput;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn msg() -> Value {
    json!({ "key": "api.not_found", "args": { "what": "weapon", "id": 7.0 }, "severity": "error" })
}

fn input() -> Value {
    json!({ "v": 1, "build": { "weapon": 7 }, "scenario": { "enemy": { "enemy": 9, "level": 30 } } })
}

fn attack_key() -> Value {
    json!({ "slug": "normal_attack", "idx": 0 })
}

fn per_damage_type(x: f64) -> Value {
    let mut m = serde_json::Map::new();
    for t in crate::DamageType::ALL {
        m.insert(t.name().into(), json!(x));
    }
    Value::Object(m)
}

fn falloff() -> Value {
    json!({ "start": 10.0, "end": 20.0, "min_factor": 0.5 })
}

fn param() -> Value {
    json!({ "key": "element", "kind": "choice", "min": 0.0, "max": 1.0,
            "choices": [{ "value": "HEA", "label": "param.choice.HEA" }], "default": "HEA", "label": "param.element" })
}

fn stat_block() -> Value {
    json!({ "cc": 0.12, "cd": 2.0, "sc": 0.26, "multishot": 1.0, "fire_rate": 8.75, "efr": 8.0, "charge_time": 0.5,
            "burst_delay": 0.05, "radius": 3.0, "punch_through": 0.5, "ammo_cost": 1.0, "falloff": falloff(),
            "forced": ["SLA"] })
}

fn summary() -> Value {
    json!({ "mean": 1.0, "sd": 0.1, "ci95": [0.9, 1.1], "ci99": [0.8, 1.2], "median": 1.0, "p5": 0.8, "p25": 0.9,
            "p75": 1.1, "p95": 1.2, "min": 0.5, "max": 1.5 })
}

fn ttk_ev() -> Value {
    json!({ "band": "killed", "seconds": 3.0, "kappa": 0.1, "shots_to_kill": 30, "ammo_to_kill": 30.0,
            "shield_regen_possible": true, "breakdown": [{ "source": "direct.IMP", "damage": 10.0 }] })
}

/// EV TTK of a build that never kills (E-8.26 band 2): no T_EV, no κ.
fn ttk_never() -> Value {
    json!({ "band": "never", "seconds": null, "shield_regen_possible": false })
}

fn view_ev() -> Value {
    json!({ "dps_burst": 1.0, "dps_sustained": 0.9, "dps_window": 0.95, "per_shot": 2.0, "per_magazine": 120.0,
            "per_type": { "IMP": 1.0, "COR": 2.0 }, "per_source": [{ "source": "dot.SLA", "dps": 0.5 }],
            "stacks": [{ "status": "COR", "mean": 1.0, "p_any": 0.5, "p_cap": 0.1 }], "distinct_mean": 2.0,
            "crit_tiers": [0.1, 0.2, 0.3, 0.2, 0.1, 0.05, 0.03, 0.02],
            "crit_tiers_steady": [0.2, 0.2, 0.2, 0.2, 0.1, 0.05, 0.03, 0.02], "series": [1.0, 0.9],
            "uptime": [{ "status": "COR", "uptime": [0.5, 1.0], "mean_stacks": [1.0, 2.5] }], "pending_dot": 0.5,
            "ttk": ttk_ev(), "hit_fresh": 2.0, "hit_infight": 2.5, "ammo_window": 100.0 })
}

/// EV outputs of a gauge-gated attack (08 E-8.14: no steady metrics) on a never-killed target.
fn view_ev_gauge_gated() -> Value {
    json!({ "dps_window": 0.95, "per_type": {}, "per_source": [], "stacks": [], "distinct_mean": 0.0,
            "crit_tiers": [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], "series": [0.95], "pending_dot": 0.0,
            "ttk": ttk_never(), "hit_fresh": 2.0, "ammo_window": 1.0 })
}

fn view_mc() -> Value {
    json!({ "runs": 1000, "seed": 9007199254740991u64, "dps_window": summary(),
            "series": [{ "mean": 1.0, "p10": 0.5, "p90": 1.5 }],
            "uptime": [{ "status": "COR", "uptime": [1.0], "mean_stacks": [2.0] }],
            "crit_hist": [0.1, 0.2, 0.3, 0.2, 0.1, 0.05, 0.03, 0.02], "breakdown": [{ "source": "heat", "dps": 1.0 }],
            "pending_dot": 0.0,
            "ttk": { "kill_probability": 0.9, "median": 3.0, "mean_killed": 3.1, "p5": 2.0, "p95": null,
                     "shots_to_kill": 30.0, "ammo_to_kill": 30.0, "shield_regen_possible": false,
                     "breakdown": [{ "source": "magnetic_break", "damage": 1.0 }] },
            "parity": { "queue_dps": 1.0, "queue_dps_min": 0.5, "queue_dps_max": 2.0, "avg_direct": 1.0,
                        "avg_status": 0.5, "avg_total": 1.5, "median_total": 1.4, "ttk": null } })
}

fn view_report() -> Value {
    json!({ "view": "layered", "ev": view_ev(), "mc": view_mc() })
}

fn score() -> Value {
    json!({ "band": 0, "primary": 1.0, "tie": "-Infinity", "tie2": "Infinity" })
}

fn ranked() -> Value {
    json!({ "input": input(), "code": "wfd1-x", "score": 1.0, "key": score(), "ttk": ttk_ev(), "mc": summary(),
            "ev_mc_deviation": true, "gap_classes": ["G-2"], "ev_rank": 2, "mix": ["COR", "HEA"],
            "draft_items": [3] })
}

/// A TTK-metric build that never kills: score T_EV = +∞ (08 E-8.21, E-8.26 band 2; engine.md §7.7).
fn ranked_never() -> Value {
    json!({ "input": input(), "code": "wfd1-y", "score": "Infinity",
            "key": { "band": 2, "primary": 120.5, "tie": 0.25, "tie2": 0.0 }, "ttk": ttk_never(),
            "ev_mc_deviation": false, "mix": ["IMP"], "draft_items": [] })
}

fn opt_result(proof: Value, condition_sensitivity: Value) -> Value {
    json!({
        "best": ranked(), "alternatives": [ranked(), ranked_never()], "per_mix": [ranked()], "proof": proof,
        "leave_one_out": [{ "slot": "gen0", "content": { "item": { "item": 3301, "rank": 5, "params": { "value": 0.3 } } },
                            "score": 0.9 },
                          { "slot": "gen2", "content": { "item": { "item": 3302 } }, "score": "Infinity" }],
        "next_best": [
            { "slot": "gen1", "content": { "riven": { "class": "rifle", "traits": [{ "trait": "cc", "value": 1.5 }],
                                                       "rank": 8, "polarity": "madurai", "drain": 18, "family": "braton" } },
              "score": 0.8 },
            { "slot": "inc2", "content": { "perk": { "index": 1 } }, "score": 0.7 },
            { "slot": "shard0", "content": { "shard": { "item": 7, "tauforged": true } }, "score": 0.6 },
            { "slot": "subsumed", "content": { "subsumed": { "ability": 40, "replaced": "Iron Skin" } }, "score": 0.5 }
        ],
        "condition_sensitivity": condition_sensitivity,
        "buff_table": { "rows": [{ "buffs": [40], "best": ranked(), "buff_value": 0.1, "rebuild_gain": 0.05 },
                                 { "buffs": [41], "best": ranked_never(), "buff_value": "-Infinity", "rebuild_gain": null },
                                 { "buffs": [40, 41], "best": ranked_never(), "buff_value": null, "rebuild_gain": null }] },
        "excluded": [{ "key": 3390, "reason": msg() }],
        "stats": { "evaluated": 1, "bounded": 2, "pruned": 3, "leaves": 4, "phases": [{ "phase": "main_search", "ms": 10 }],
                   "threads": 4, "reproducible": true, "elapsed_ms": 20 },
        "capacity": { "used": 58, "capacity": 60, "forma": 3, "umbra_forma": 1, "mode": "any_forma" }
    })
}

/// (TypeScript root name, sample) for every root.
fn samples() -> Vec<(&'static str, Value)> {
    let proven =
        json!({ "proven_optimal": { "scope": "main_given_context", "assumes_fast_error": 0.005 } });
    let gap = json!({ "gap": { "upper_bound": score(), "relative": 0.01 } });
    let gap_inf = json!({ "gap": { "upper_bound": score(), "relative": "Infinity" } });
    let part = json!({ "key": 1, "name": "Gaze", "image": false, "display": { "fire_rate": 0.5, "note": "x" } });
    let condition = |ov: Value| {
        json!({ "effect": "/Lotus/X#co", "item": 3305, "class": ["a", "c"], "treatment": "expected", "value": 0.3,
                "override": ov })
    };
    vec![
        ("Message", msg()),
        (
            "ErrorResponse",
            json!({ "error": msg(), "issues": [msg()] }),
        ),
        (
            "Meta",
            json!({
                "app_id": "warframe-dps", "app_version": "0.1.0", "api_version": 1, "schema_version": 1,
                "data_version": "2026.10.28+abcd1234", "game_baseline": "Hotfix 44.1.4",
                "bundle": { "id": "embedded", "source": "embedded", "created_at": "2026-11-24T00:00:00Z" },
                "sources": [{ "source": "wiki", "revision": "123", "fetched_at": "2026-11-23T00:00:00Z" }],
                "licences": [{ "source": "wiki", "licence": "CC BY-NC-SA 3.0",
                               "url": "https://creativecommons.org/licenses/by-nc-sa/3.0/", "attribution": "Warframe Wiki" }],
                "coverage": { "weapon_mod": { "verified": 3, "unmodelled": 2 } },
                "config": { "optimizer_threads": 3, "interactive_threads": 4, "mc_runs_default": 1000, "images_fetch": false },
                "local_keys": false, "features": { "refresh": true }
            }),
        ),
        (
            "Catalog",
            json!({
                "data_version": "dv",
                "weapons": [{ "key": 1204, "name": "Braton Prime", "slot": "primary", "class": "rifle", "traits": ["prime"],
                              "image": true, "incarnon": false, "modular": "kitgun", "progenitor": false, "mastery": 8 }],
                "items": [{ "key": 3301, "name": "Serration", "kind": "weapon_mod", "polarity": "madurai", "drain": 4,
                            "max_rank": 10, "rarity": "common", "set": 0, "image": true, "status": "verified",
                            "stats": ["base_dmg"], "exilus": false, "color": "crimson", "subsumable": false }],
                "enemies": [{ "key": 9001, "name": "Lancer", "faction": "grineer", "base_level": 1, "layers": ["HP", "AR"] }],
                "frames": [{ "key": 5, "name": "Rhino Prime", "variant_of": 6, "polarities": ["madurai"],
                             "aura_polarities": ["any"], "exilus_polarity": "naramon", "aura_slots": 1,
                             "abilities": [{ "name": "Roar", "self_buff": 40 }], "helminth_host": 40, "image": true }],
                "factions": [{ "slug": "grineer", "name": "Grineer" }],
                "sets": [{ "id": 0, "name": "Vigilante", "members": [1, 2] }],
                "names": { "/Lotus/X": "X" }
            }),
        ),
        (
            "WeaponDetail",
            json!({
                "key": 1204, "unique_name": "/Lotus/Weapons/Tenno/Rifle/BratonPrime", "name": "Braton Prime",
                "slot": "primary", "class": "rifle", "trigger": "auto",
                "magazine": { "size": 75.0, "ammo_max": "Infinity", "infinite": false,
                              "battery": { "delay": 1.0, "delay_empty": 2.0, "rate": 10.0 } },
                "reload": { "kind": "standard", "time": 2.15 },
                "attacks": [{
                    "key": attack_key(), "name": "Normal Attack", "roles": ["normal"], "shot": "hit_scan", "parent": 0,
                    "base": per_damage_type(1.0), "cc": 0.12, "cd": 2.0, "sc": 0.26, "multishot": 1.0,
                    "fire": { "trigger": "charge", "rate": "Infinity", "charge_time": 0.5,
                              "burst": { "count": 3, "delay": 0.05 } },
                    "falloff": falloff(), "radius": 3.0, "forced": ["IMP"], "is_heavy": true, "incarnon_charges": 1,
                    "co": { "behaviour": "adding", "ratio": 1.0, "row": 12 }
                }],
                "groups": [{ "members": [0, 1], "combinable": true, "summed": true }],
                "incarnon": { "tiers": [{ "tier": 2, "perks": [{ "index": 0, "key": 77, "name": "Haven Foray",
                                                                  "effects_text": [msg()] }] }] },
                "modular": { "kind": "kitgun", "grips": [part.clone()], "loaders": [part.clone()], "links": [part] },
                "progenitor": { "elements": ["HEA"], "min": 0.25, "max": 0.6, "default": 0.6 },
                "stances": [{ "key": 300, "name": "Crimson Dervish",
                              "combos": [{ "key": "neutral", "kind": "neutral", "name": "Combo", "source": "stance" }] }],
                "melee": { "class": "sword", "plans": ["light_loop", "heavy_loop"], "heavy": 1, "ability_combo": false },
                "zoom": [{ "name": "2x", "effects_text": [msg()] }],
                "slot_layout": { "slots": ["gen0", "exilus"], "arcane_slots": 1 },
                "polarities": ["madurai"], "exilus_polarity": "naramon", "max_rank": 30, "mastery": 14,
                "display": { "accuracy": 28.6, "noise": "alarming" }, "image": true,
                "provenance": [{ "source": "wiki", "revision": "1" }]
            }),
        ),
        (
            "ItemDetail",
            json!({
                "key": 3301, "unique_name": "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod", "name": "Serration",
                "kind": "weapon_mod", "polarity": "madurai", "drain": 4, "max_rank": 10, "rarity": "common",
                "set": { "id": 0, "name": "Vigilante", "members": [1], "bonus_text": [msg()] },
                "params": [param()],
                "effects": [{
                    "id": "dmg",
                    "target": { "stat": "base_dmg", "element": "HEA", "faction": "grineer", "mult": "G",
                                "mechanic": "extra_hit" },
                    "value_max": 1.65, "per": ["stacks"],
                    "condition": { "class": "c", "trigger": "on_kill", "chance": 1.0,
                                   "stacks": { "max": 3, "duration": 10.0 } },
                    "scope_text": [msg()], "note": "n"
                }],
                "status": "verified", "sources": ["W:Serration"], "upstream_text": "+165% Damage",
                "incompatible": [3390], "compat": { "slots": ["gen"], "trigger": "semi", "categories": ["Rifle"] },
                "image": true
            }),
        ),
        (
            "EnemyDetail",
            json!({
                "key": 9001, "name": "Lancer", "faction": "grineer", "base_level": 1,
                "base": { "health": 100.0, "shield": 0.0, "armor": 100.0, "overguard": 0.0 },
                "eximus": { "health": 200.0, "shield": 10.0, "available": true },
                "parts": [{ "name": "head", "mult": 2.0, "weak_point": true, "head": true, "head_crit": true }],
                "immune": ["VIR"], "caps": { "COR": 10 }, "innate_dr": { "OG": 0.0, "SH": 0.0, "HP": 0.1 },
                "status_dr": 0.0, "special_dr": { "kind": "attenuation", "note": msg() }, "flags": ["boss"],
                "party_scaled": false, "faction_mods": per_damage_type(1.0)
            }),
        ),
        (
            "ScenarioSchema",
            json!({
                "toggles": [{
                    "id": "T-03", "field": "toggles.hit_location", "group": "general", "label": "toggle.T-03.label",
                    "help": "toggle.T-03.help",
                    "control": { "kind": "select", "min": 0.0, "max": 1.0, "step": 0.1,
                                 "options": [{ "value": "body", "label": "toggle.T-03.body" }] },
                    "default": "body",
                    "visible_if": { "weapon_slot": ["melee"], "attack_role": ["normal"], "trait": ["prime"] },
                    "advanced": false
                }],
                "buffs": [{ "key": 40, "id": "roar", "name": "Roar", "group": "ability", "toggle": "T-13",
                            "inspiration_key": "o.r", "strength_scaled": true, "subsumable": true, "params": [param()] }],
                "melee_plans": [{ "id": "heavy_loop", "label": "plan.heavy_loop", "help": "plan.heavy_loop.help",
                                  "params": [{ "key": "policy", "kind": "select", "default": "full_regen",
                                               "options": ["full_regen", "back_to_back"] }] }],
                "custom_stats": [{ "id": "base", "key": 500, "label": "custom.base", "unit": "percent", "min": 0.0,
                                   "max": 10.0, "inspiration_key": "p.a" }],
                "riven_traits": [{ "id": "cc", "label": "riven.cc", "unit": "percent", "classes": ["rifle"],
                                   "element": "HEA", "range": { "min": 0.1, "max": 1.5 } }],
                "slot_layouts": { "primary": { "slots": ["gen0", "exilus"] } },
                "views": [{ "view": "armor", "label": "view.armor", "help": "view.armor.help" }],
                "conditions": { "modes": ["all_max", "simulated"],
                                "classes": [{ "id": "a", "label": "cond.a", "help": "cond.a.help" }] }
            }),
        ),
        (
            "CandidatesRequest",
            json!({ "input": input(), "slot": "gen0", "query": "serr", "stat": "base_dmg" }),
        ),
        (
            "Candidate",
            json!({ "key": 1, "legal": false, "reason": msg() }),
        ),
        ("Candidate", json!({ "key": 2, "legal": true })),
        (
            "CalcRequest",
            json!({ "input": input(), "want": { "mc": { "runs": 100, "seed": 42, "series_bin_s": 0.5 }, "parity": true,
                                                "breakdown": true, "trace": true, "views": ["armor"] } }),
        ),
        (
            "CalcReport",
            json!({
                "data_version": "dv", "code": "wfd1-x",
                "weapon": {
                    "attacks": [{ "key": attack_key(), "name": "Normal Attack", "base": stat_block(),
                                  "modded": stat_block(), "per_type": { "IMP": 1.0 }, "total": 1.0 }],
                    "magazine": { "base": 60.0, "modded": 90.0 }, "incarnon_magazine": 30.0,
                    "reload": { "base": 2.0, "modded": 1.5 },
                    "extra": { "zoom": { "base": 1.0, "modded": 2.0 },
                               "ammo_max": { "base": "Infinity", "modded": "Infinity" } }
                },
                "enemy": { "level": 100, "pools": { "OG": 0.0, "SH": 0.0, "HP": 73621.375 }, "armor0": 1355.0,
                           "armor_eff": 1355.0, "dr": 0.8, "innate_dr": { "OG": 0.0, "SH": 0.0, "HP": 0.0 },
                           "immune": ["VIR"], "special_dr": msg(), "faction_mods": { "COR": 1.5 }, "party": 1,
                           "caps": { "VIR": 10 } },
                "attacks": [{ "key": attack_key(), "name": "Normal Attack",
                              "views": [view_report(), { "view": "health", "ev": view_ev_gauge_gated() }] }],
                "groups": [{ "members": [0, 1], "combined": false, "views": [view_report()] }],
                "conditions": [
                    condition(json!({ "force": true })),
                    condition(json!({ "stacks": 2 })),
                    condition(json!({ "uptime": { "u": 0.5, "stacks": 2 } }))
                ],
                "touched_rows": ["D-023"],
                "violations": [{ "key": "cap.exceeded", "severity": "illegal", "slots": ["gen3"],
                                 "args": { "used": 72.0 } }],
                "capacity": { "used": 72, "capacity": 60, "forma": null, "umbra_forma": null, "mode": { "budget": 60 },
                              "forma_applied_min": 2,
                              "warframe": { "used": 58, "capacity": 60, "forma": 1, "umbra_forma": 1 } },
                "trace": [{ "probe": "attack[normal_attack].cc", "equation": "E-5", "value": 0.3,
                            "inputs": [{ "probe": "attack[normal_attack].cc_rel", "value": 1.5 }],
                            "contributions": [{ "effect": "/Lotus/X#cc", "value": 1.5 }], "rows": ["D-001"] }],
                "warnings": [msg()], "timing_us": 1234
            }),
        ),
        (
            "TimelineRequest",
            json!({ "input": input(), "attack": attack_key(), "view": "armor", "seed": 5, "run": 2, "limit": 100 }),
        ),
        (
            "TimelineRequest",
            json!({ "input": input(), "attack": { "group": 0 }, "view": "health" }),
        ),
        (
            "Timeline",
            json!({
                "duration": 20.0,
                "hits": [{ "n": 1, "t": 0.0, "damage": 100.0, "crit_tier": 1, "crit_mult": 2.0, "armor_eff": 100.0,
                           "armor_reduction": 0.25, "stance_mult": 1.0, "heavy_combo_mult": 1.0,
                           "extra_hits": [{ "kind": "xata", "damage": 5.0 }], "statuses_applied": ["SLA"],
                           "statuses_added": ["HEA"], "tick_values": { "SLA": 3.0 }, "stacks": { "SLA": 1 },
                           "flags": ["first", "after_reload"] }],
                "reloads": [{ "n": 1, "start": 5.0, "end": 7.0 }],
                "heat_episodes": [{ "ignition": 1.0, "end": 7.0 }], "truncated": false
            }),
        ),
        (
            "ExplainRequest",
            json!({ "input": input(), "probe": "attack[normal_attack].cc" }),
        ),
        (
            "Explanation",
            json!({ "probe": "attack[normal_attack].cc", "value": 0.3, "equation": "E-5", "formula": msg(),
                    "inputs": [{ "probe": "attack[normal_attack].cc_rel", "label": msg(), "value": 1.5 }],
                    "contributions": [{ "effect": "/Lotus/X#cc", "item": 3303, "label": msg(), "value": 1.5 }],
                    "rows": [{ "id": "D-001", "text": msg() }] }),
        ),
        (
            "EncodeResponse",
            json!({ "code": "wfd1-x", "path": "/?b=wfd1-x" }),
        ),
        ("DecodeRequest", json!({ "text": "wfd1-x" })),
        (
            "DecodeResponse",
            json!({ "input": input(), "source": "inspiration_lz",
                    "report": { "unmapped": [{ "field": "m.3", "value": "Foo" }], "converted": [msg()],
                                "defaults_applied": [msg()], "retired_items": [5], "warnings": [msg()] } }),
        ),
        (
            "ExportInspirationResponse",
            json!({ "url": "https://example.invalid/?p=x&n=1" }),
        ),
        (
            "OptRequest",
            json!({
                "input": input(),
                "objective": { "metric": "ttk_noarmor", "view": "auto",
                               "attacks": [{ "attack": attack_key(), "weight": 1.0 }],
                               "heavy": { "start_combo": 12 }, "window_s": 20.0 },
                "constraints": {
                    "locked": ["gen0"], "must_include": [1], "prohibited": [2],
                    "elements": { "require": ["COR"], "forbid": ["VIR"] }, "rules": ["use_weak_point", "no_faction"],
                    "capacity": { "mode": { "budget": 60 }, "umbra_forma": true, "ranks": "allow_lower" },
                    "riven": { "search": { "traits": 3, "allow_negative": true, "quality": 0.5 } }
                },
                "groups": { "exilus": true, "weapon_arcanes": true, "stance": false, "melee_plan": false,
                            "incarnon": true, "frame_mods": 2, "frame_exilus": false, "aura": true,
                            "frame_arcanes": false,
                            "shards": { "owned": [{ "item": 7, "tauforged": true, "count": 2 }] },
                            "modular_parts": true, "progenitor": true },
                "buffs": { "allowed": [{ "buff": 40, "params": { "value": 0.3 },
                                         "caster": { "ally": { "strength": 2.0 } } }],
                           "portfolio": true, "strength": 2.5 },
                "search": { "mode": "balanced", "certify": true, "time_limit_s": 60.0, "node_budget": 1000000,
                            "keep": 32, "mc_confirm": { "finalists": 12, "runs": 1000, "seed": 0 }, "threads": 4,
                            "context_beam": 4, "preset": "inspiration_compatible" }
            }),
        ),
        ("JobAccepted", json!({ "job": "opt-7f3a9c" })),
        (
            "OptEvent",
            json!({ "event": "started", "job": "opt-1",
                    "plan": { "positions": [{ "group": "main", "pool": 36 }], "composition_sets": 2154,
                              "excluded": [{ "key": 3390, "reason": msg() }] } }),
        ),
        (
            "OptEvent",
            json!({ "event": "phase", "phase": "main_search", "index": 3, "total": 9, "message": msg() }),
        ),
        (
            "OptEvent",
            json!({ "event": "progress", "phase": "main_search", "fraction": 0.42, "evaluated": 18234000,
                    "bounded": 4410021, "pruned": 3902112, "rate_per_s": 1520000.0, "elapsed_ms": 12000,
                    "eta_s": 16.5, "gap": null }),
        ),
        (
            "OptEvent",
            json!({ "event": "progress", "phase": "local_search", "fraction": null, "evaluated": 1, "bounded": 0,
                    "pruned": 0, "rate_per_s": 1.0, "elapsed_ms": 1, "eta_s": null, "gap": "Infinity" }),
        ),
        (
            "OptEvent",
            json!({ "event": "incumbent", "rank": 1, "build": ranked() }),
        ),
        (
            "OptEvent",
            json!({ "event": "finalist", "build": ranked() }),
        ),
        ("OptEvent", json!({ "event": "log", "message": msg() })),
        (
            "OptEvent",
            json!({ "event": "done", "result": opt_result(proven, json!(0.5)) }),
        ),
        (
            "OptEvent",
            json!({ "event": "cancelled", "result": opt_result(gap_inf, json!("Infinity")) }),
        ),
        ("OptEvent", json!({ "event": "cancelled", "result": null })),
        ("OptEvent", json!({ "event": "failed", "error": msg() })),
        ("OptResult", opt_result(json!("heuristic"), Value::Null)),
        (
            "JobStatus<OptResult>",
            json!({ "job": "opt-1", "state": "done", "started_at": "2026-11-24T00:00:00Z",
                    "last_progress": { "event": "progress", "fraction": null },
                    "result": opt_result(gap, json!(0.5)),
                    "error": msg() }),
        ),
        (
            "DataStatus",
            json!({ "active": { "id": "embedded", "source": "embedded", "data_version": "dv", "game_baseline": "b",
                                "created_at": "t" },
                    "bundles": [{ "id": "wfdata-x", "data_version": "dv", "created_at": "t", "size": 1000,
                                  "active": false }],
                    "refresh_job": "data-1" }),
        ),
        (
            "DataCheck",
            json!({ "changed": true, "pe_build_label": "l", "wiki_changed_modules": ["Module:Weapons/data"],
                    "wfcd_latest_tag": "v1", "checked_at": "t" }),
        ),
        ("RefreshRequest", json!({ "curated": "embedded" })),
        ("ActivateRequest", json!({ "bundle": "embedded" })),
        (
            "DataEvent",
            json!({ "event": "step", "step": "fetch", "done": 3, "total": 60 }),
        ),
        ("DataEvent", json!({ "event": "log", "message": msg() })),
        (
            "DataEvent",
            json!({ "event": "report", "bundle_id": "b",
                    "report": { "diff": { "weapons_changed": 1, "items_changed": 2, "enemies_changed": 3,
                                          "samples": [{ "path": "p", "old": "a", "new": "b" }] },
                                "new_items": [1], "stale": [2], "auto_patched": [3], "conflicts": [msg()],
                                "errors": [msg()], "warnings": [msg()] } }),
        ),
        ("DataEvent", json!({ "event": "done", "bundle_id": "b" })),
        ("DataEvent", json!({ "event": "cancelled" })),
        ("DataEvent", json!({ "event": "failed", "error": msg() })),
        ("CalcInput", input()),
    ]
}

/// Parses, writes, parses again; returns what serde writes.
fn round_trip<T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug>(
    name: &str,
    v: &Value,
    exact: bool,
) -> Value {
    let a: T = serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("{name}: {e}\n{v:#}"));
    let written = serde_json::to_value(&a).unwrap_or_else(|e| panic!("{name}: {e}"));
    let b: T = serde_json::from_value(written.clone())
        .unwrap_or_else(|e| panic!("{name} (written): {e}\n{written:#}"));
    assert_eq!(a, b, "{name}: round trip");
    assert_eq!(
        serde_json::to_value(&b).unwrap(),
        written,
        "{name}: unstable JSON form"
    );
    if !exact {
        return written;
    }
    assert_eq!(
        super::normalize_numbers(&written),
        super::normalize_numbers(&expand(name, v)),
        "{name}: serde writes something else than the sample (an omitted default or a renamed field)"
    );
    written
}

/// The sample with the parts serde fills in (documents expand their defaults; only `CalcInput` is affected).
fn expand(name: &str, v: &Value) -> Value {
    fn fill(v: &Value) -> Value {
        match v {
            Value::Object(m) => {
                let mut out = serde_json::Map::new();
                for (k, x) in m {
                    if k == "input" && x.get("v").is_some() {
                        let ci: CalcInput = serde_json::from_value(x.clone()).unwrap();
                        out.insert(k.clone(), serde_json::to_value(ci).unwrap());
                    } else {
                        out.insert(k.clone(), fill(x));
                    }
                }
                Value::Object(out)
            }
            Value::Array(a) => Value::Array(a.iter().map(fill).collect()),
            x => x.clone(),
        }
    }
    if name == "CalcInput" {
        let ci: CalcInput = serde_json::from_value(v.clone()).unwrap();
        return serde_json::to_value(ci).unwrap();
    }
    fill(v)
}

fn round_trip_as(name: &str, v: &Value, exact: bool) -> Value {
    match name {
        "Message" => round_trip::<crate::message::Message>(name, v, exact),
        "ErrorResponse" => round_trip::<ErrorResponse>(name, v, exact),
        "Meta" => round_trip::<Meta>(name, v, exact),
        "Catalog" => round_trip::<Catalog>(name, v, exact),
        "WeaponDetail" => round_trip::<WeaponDetail>(name, v, exact),
        "ItemDetail" => round_trip::<ItemDetail>(name, v, exact),
        "EnemyDetail" => round_trip::<EnemyDetail>(name, v, exact),
        "ScenarioSchema" => round_trip::<ScenarioSchema>(name, v, exact),
        "CandidatesRequest" => round_trip::<CandidatesRequest>(name, v, exact),
        "Candidate" => round_trip::<Candidate>(name, v, exact),
        "CalcRequest" => round_trip::<CalcRequest>(name, v, exact),
        "CalcReport" => round_trip::<CalcReport>(name, v, exact),
        "TimelineRequest" => round_trip::<TimelineRequest>(name, v, exact),
        "Timeline" => round_trip::<Timeline>(name, v, exact),
        "ExplainRequest" => round_trip::<ExplainRequest>(name, v, exact),
        "Explanation" => round_trip::<Explanation>(name, v, exact),
        "EncodeResponse" => round_trip::<EncodeResponse>(name, v, exact),
        "DecodeRequest" => round_trip::<DecodeRequest>(name, v, exact),
        "DecodeResponse" => round_trip::<DecodeResponse>(name, v, exact),
        "ExportInspirationResponse" => round_trip::<ExportInspirationResponse>(name, v, exact),
        "OptRequest" => round_trip::<OptRequest>(name, v, exact),
        "JobAccepted" => round_trip::<JobAccepted>(name, v, exact),
        "OptEvent" => round_trip::<OptEvent>(name, v, exact),
        "OptResult" => round_trip::<OptResult>(name, v, exact),
        "JobStatus<OptResult>" => round_trip::<JobStatus<OptResult>>(name, v, exact),
        "DataStatus" => round_trip::<DataStatus>(name, v, exact),
        "DataCheck" => round_trip::<DataCheck>(name, v, exact),
        "RefreshRequest" => round_trip::<RefreshRequest>(name, v, exact),
        "ActivateRequest" => round_trip::<ActivateRequest>(name, v, exact),
        "DataEvent" => round_trip::<DataEvent>(name, v, exact),
        "CalcInput" => round_trip::<CalcInput>(name, v, exact),
        other => panic!("no Rust type mapped for root {other}"),
    }
}

#[test]
fn every_api_type_round_trips_and_matches_its_typescript() {
    let defs = generated_defs();
    let roots = crate::api::codegen::typescript_roots().unwrap();
    let samples = samples();
    let mut checker = Checker::new(&defs);
    checker.no_cover.insert("CalcInput".into());
    let mut problems = Vec::new();
    for (name, v) in &samples {
        assert!(
            roots.iter().any(|r| r == name),
            "sample for {name}, which is not a root"
        );
        let written = round_trip_as(name, v, true);
        problems.extend(checker.check(name, &written, name, true));
        // The opposite direction: optional fields absent and lists empty. serde must accept what TypeScript calls
        // optional, and TypeScript must accept what serde then writes (omitted, null or empty).
        let min = checker.minimize(name, &written);
        let written = round_trip_as(name, &min, false);
        problems.extend(checker.check(name, &written, &format!("{name} (minimal)"), false));
    }
    for r in &roots {
        assert!(
            samples.iter().any(|(n, _)| n == r),
            "no sample for root {r}"
        );
    }
    assert!(
        problems.is_empty(),
        "TypeScript does not accept serde's JSON:\n{}",
        problems.join("\n")
    );

    // Coverage: every object literal reachable from the roots, with every field.
    let mut reachable: BTreeMap<String, Vec<Field>> = BTreeMap::new();
    for r in &roots {
        checker.reachable_objects(r, &mut reachable);
    }
    let mut missing = Vec::new();
    for (obj, fields) in &reachable {
        if super::ts::mapped(obj).is_some() {
            continue;
        }
        match checker.seen.get(obj) {
            None => missing.push(format!("never matched: {obj}")),
            Some(seen) => {
                for f in fields {
                    if !seen.contains(&f.name) {
                        missing.push(format!(
                            "field `{}` never present in: {}",
                            f.name,
                            normalize(obj)
                        ));
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "samples do not cover:\n{}",
        missing.join("\n")
    );
}

#[test]
fn engine_outputs_round_trip() {
    // EvOut and McOut are not route bodies (no TypeScript root) but are API DTOs the engine re-exports.
    let ev = json!({ "views": [["armor", view_ev()]] });
    let back: EvOut = serde_json::from_value(ev.clone()).unwrap();
    assert_eq!(
        super::normalize_numbers(&serde_json::to_value(&back).unwrap()),
        super::normalize_numbers(&ev)
    );
    let mc = json!({ "views": [["layered", view_mc()]], "runs": 1000, "seed": 7 });
    let m: McOut = serde_json::from_value(mc).unwrap();
    let again: McOut = serde_json::from_value(serde_json::to_value(&m).unwrap()).unwrap();
    assert_eq!(m, again);
    assert_eq!(m.views[0].0, crate::View::Layered);
}

#[test]
fn optional_and_nullable_parity_ttk() {
    // ParityOut.ttk: absent = not computed, null = not killed, a number = the TTK.
    let base = json!({ "queue_dps": 1.0, "queue_dps_min": 0.5, "queue_dps_max": 2.0, "avg_direct": 1.0,
                       "avg_status": 0.5, "avg_total": 1.5, "median_total": 1.4 });
    let p: ParityOut = serde_json::from_value(base.clone()).unwrap();
    assert_eq!(p.ttk, None);
    assert!(serde_json::to_value(&p).unwrap().get("ttk").is_none());
    let mut v = base.clone();
    v["ttk"] = Value::Null;
    let p: ParityOut = serde_json::from_value(v).unwrap();
    assert_eq!(p.ttk, Some(None));
    assert_eq!(serde_json::to_value(&p).unwrap()["ttk"], Value::Null);
    let mut v = base;
    v["ttk"] = json!(4.5);
    let p: ParityOut = serde_json::from_value(v).unwrap();
    assert_eq!(p.ttk, Some(Some(4.5)));
}

#[test]
fn large_integers_and_infinities() {
    // u64 fields are capped at 2^53 − 1 in both directions (domain-model.md §1).
    let mut req = json!({ "input": input(), "want": { "mc": { "seed": 9007199254740992u64 } } });
    assert!(serde_json::from_value::<CalcRequest>(req.clone()).is_err());
    req["want"]["mc"]["seed"] = json!(9007199254740991u64);
    assert!(serde_json::from_value::<CalcRequest>(req).is_ok());
    // Score keys of bounds and thresholds carry ±∞ (engine.md §7.9), written as strings.
    let s = Score {
        band: 2,
        primary: f64::NEG_INFINITY,
        tie: f64::INFINITY,
        tie2: 0.0,
    };
    let v = serde_json::to_value(s).unwrap();
    assert_eq!(
        v,
        json!({ "band": 2, "primary": "-Infinity", "tie": "Infinity", "tie2": 0.0 })
    );
    assert_eq!(serde_json::from_value::<Score>(v).unwrap(), s);
}

#[test]
fn per_damage_type_is_a_full_record() {
    // Every type maps to its own field by name: value i at index i, under the JSON key of its spec id.
    let mut v = [0.0; crate::N_DT];
    for (i, x) in v.iter_mut().enumerate() {
        *x = i as f64;
    }
    let r = PerDamageType::from_vec(&v);
    assert_eq!(r.to_vec(), v);
    let json = serde_json::to_value(r).unwrap();
    assert_eq!(json.as_object().unwrap().len(), crate::N_DT);
    for t in crate::DamageType::ALL {
        assert_eq!(r.get(*t), t.index() as f64, "{t}");
        assert_eq!(json[t.name()], json!(t.index() as f64), "{t}");
    }
    // A record with a missing type is rejected.
    let mut partial = json.clone();
    partial.as_object_mut().unwrap().remove("TAU");
    assert!(serde_json::from_value::<PerDamageType>(partial).is_err());
}

#[test]
fn non_finite_values_round_trip() {
    // TTK scores are T_EV seconds, +∞ for a build that never kills (08 E-8.21, E-8.26 band 2): written "Infinity",
    // never `null` (which would not read back).
    let mut b: RankedBuild = serde_json::from_value(ranked()).unwrap();
    b.score = f64::INFINITY;
    let v = serde_json::to_value(&b).unwrap();
    assert_eq!(v["score"], json!("Infinity"));
    assert_eq!(serde_json::from_value::<RankedBuild>(v).unwrap(), b);
    // A NaN score is an error, not a silent `null`.
    b.score = f64::NAN;
    assert!(serde_json::to_value(&b).is_err());

    // Weapon-panel rows: `ammo_max` of a robotic weapon (06 §11.4).
    let row = BaseModded {
        base: f64::INFINITY,
        modded: f64::INFINITY,
    };
    let v = serde_json::to_value(row).unwrap();
    assert_eq!(v, json!({ "base": "Infinity", "modded": "Infinity" }));
    assert_eq!(serde_json::from_value::<BaseModded>(v).unwrap(), row);

    // κ exists only in band `killed`; a never-killed TTK has neither T_EV nor κ.
    let t: TtkEv = serde_json::from_value(ttk_never()).unwrap();
    assert_eq!((t.band, t.seconds, t.kappa), (TtkBand::Never, None, None));
    assert!(serde_json::to_value(&t).unwrap().get("kappa").is_none());

    // Score-valued fields of the optimizer result.
    let gap = Proof::Gap {
        upper_bound: serde_json::from_value(score()).unwrap(),
        relative: f64::INFINITY,
    };
    let v = serde_json::to_value(&gap).unwrap();
    assert_eq!(v["gap"]["relative"], json!("Infinity"));
    assert_eq!(serde_json::from_value::<Proof>(v).unwrap(), gap);
    b.score = f64::INFINITY;
    let row = BuffScenarioRow {
        buffs: vec![StableKey(41)],
        best: b,
        buff_value: Some(f64::NEG_INFINITY),
        rebuild_gain: None,
    };
    let v = serde_json::to_value(&row).unwrap();
    assert_eq!(v["buff_value"], json!("-Infinity"));
    assert_eq!(v["rebuild_gain"], Value::Null);
    assert_eq!(serde_json::from_value::<BuffScenarioRow>(v).unwrap(), row);
}

#[test]
fn search_params_defaults_and_off() {
    // Absent `time_limit_s` and `mc_confirm` take the optimizer.md §11 defaults; `null` switches them off.
    let required = json!({ "mode": "exact", "certify": false, "keep": 32, "context_beam": 4, "preset": "default" });
    let p: SearchParams = serde_json::from_value(required.clone()).unwrap();
    assert_eq!(p, SearchParams::default());
    assert_eq!(p.time_limit_s, Some(SearchParams::DEFAULT_TIME_LIMIT_S));
    assert_eq!(p.time_limit_s, Some(120.0));
    assert_eq!(
        p.mc_confirm,
        Some(McConfirm {
            finalists: 12,
            runs: 1000,
            seed: 0
        })
    );
    let mut off = required;
    off["time_limit_s"] = Value::Null;
    off["mc_confirm"] = Value::Null;
    let q: SearchParams = serde_json::from_value(off).unwrap();
    assert_eq!((q.time_limit_s, q.mc_confirm.clone()), (None, None));
    // Both are always written, so the meaning survives a round trip.
    let w = serde_json::to_value(&q).unwrap();
    assert_eq!(
        (&w["time_limit_s"], &w["mc_confirm"]),
        (&Value::Null, &Value::Null)
    );
    assert_eq!(serde_json::from_value::<SearchParams>(w).unwrap(), q);
    let w = serde_json::to_value(&p).unwrap();
    assert_eq!(w["time_limit_s"], json!(120.0));
    assert_eq!(serde_json::from_value::<SearchParams>(w).unwrap(), p);
}

#[test]
fn candidate_reason_iff_blocked() {
    let legal = Candidate::legal(StableKey(2));
    assert!(legal.is_legal() && legal.reason().is_none());
    assert_eq!(
        serde_json::to_value(&legal).unwrap(),
        json!({ "key": 2, "legal": true })
    );
    let m: crate::message::Message = serde_json::from_value(msg()).unwrap();
    let blocked = Candidate::blocked(StableKey(1), m.clone());
    assert_eq!(blocked.key(), StableKey(1));
    assert_eq!(blocked.reason(), Some(&m));
    assert!(!blocked.is_legal());
    // The rule is checked when reading.
    for bad in [
        json!({ "key": 1, "legal": false }),
        json!({ "key": 1, "legal": true, "reason": msg() }),
        json!({ "key": 1, "legal": true, "extra": 0 }),
    ] {
        assert!(
            serde_json::from_value::<Candidate>(bad.clone()).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn json_schemas_of_api_types_generate() {
    for (name, schema) in [
        ("CalcRequest", schemars::schema_for!(CalcRequest)),
        ("CalcReport", schemars::schema_for!(CalcReport)),
        ("OptRequest", schemars::schema_for!(OptRequest)),
        ("OptEvent", schemars::schema_for!(OptEvent)),
        ("Catalog", schemars::schema_for!(Catalog)),
        ("ScenarioSchema", schemars::schema_for!(ScenarioSchema)),
        ("DataEvent", schemars::schema_for!(DataEvent)),
        ("JobStatus", schemars::schema_for!(JobStatus<OptResult>)),
    ] {
        let text = serde_json::to_string(&schema).unwrap();
        assert!(text.len() > 200, "{name}: {text}");
    }
}
