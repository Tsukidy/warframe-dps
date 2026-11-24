Status: reviewed (spec)

# Damage-calculation spec, part 05: enemies

This document defines the target: what an enemy is in data, how its pools scale with level, Steel Path, Eximus,
squad size and mission modifiers, and everything the target does to an incoming damage instance (stage ST-8) and
how that instance is routed through overguard, shields and health (stage ST-9). It also supplies the per-enemy data
that other documents consume: body parts and head-crit capability (`02`), status immunities and per-enemy status
caps (`02`, `03`).

Related documents: [00-overview](00-overview.md) (master: notation, E-12, E-15, CD-xx, register, toggles),
[01-damage-composition](01-damage-composition.md) (weapon-side W_t, faction bucket F),
[02-crit-status-multishot](02-crit-status-multishot.md) (body part E-7, crit, proc weights),
[03-status-effects](03-status-effects.md) (Viral, Magnetic, Corrosive and Heat strip values, caps rule CD-07),
[04-conditional-bonuses-and-buffs](04-conditional-bonuses-and-buffs.md) (vulnerability and strip catalogue, M_OG),
[06-ranged-firing-model](06-ranged-firing-model.md), [07-melee](07-melee.md),
[08-evaluation-and-outputs](08-evaluation-and-outputs.md) (EV and MC algorithms, TTK procedure),
[09-build-rules](09-build-rules.md).

Contents:

- [1. Scope, stages and citation keys](#1-scope-stages-and-citation-keys)
- [2. Inputs](#2-inputs)
- [3. Outputs and symbols added](#3-outputs-and-symbols-added)
- [4. Enemy data model](#4-enemy-data-model)
- [5. Faction damage-type modifiers](#5-faction-damage-type-modifiers)
- [6. Level scaling (NP-2)](#6-level-scaling-np-2)
- [7. Base-value modifiers: Steel Path, Eximus, squad size, mission modifiers](#7-base-value-modifiers-steel-path-eximus-squad-size-mission-modifiers)
- [8. Overguard pool](#8-overguard-pool)
- [9. Armor](#9-armor)
- [10. Shields](#10-shields)
- [11. Overguard layer](#11-overguard-layer)
- [12. Health layer and the Viral stage](#12-health-layer-and-the-viral-stage)
- [13. Vulnerability stage](#13-vulnerability-stage)
- [14. Innate, status and class-conditional damage reduction](#14-innate-status-and-class-conditional-damage-reduction)
- [15. Unique damage reduction and T-28](#15-unique-damage-reduction-and-t-28)
- [16. The enemy-side chain of one instance (ST-8)](#16-the-enemy-side-chain-of-one-instance-st-8)
- [17. Layer routing, overflow and shield gate (ST-9)](#17-layer-routing-overflow-and-shield-gate-st-9)
- [18. Layer views](#18-layer-views)
- [19. Notes for the evaluators](#19-notes-for-the-evaluators)
- [20. Worked example](#20-worked-example)
- [21. Edge cases](#21-edge-cases)
- [22. Inspiration differences](#22-inspiration-differences)
- [23. Open questions](#23-open-questions)
- [24. Test vectors](#24-test-vectors)
- [Review notes](#review-notes)

---

## 1. Scope, stages and citation keys

### 1.1 Scope (from 00 section 7.1)

Owns: enemy data model (including status immunities and per-enemy status caps as data); faction damage-type table
and overrides; level scaling (NP-2), Steel Path (including flat bonuses), Eximus, overguard scaling, party scaling;
armor (DR, clamp, floor, E-15 strip combination including ability strips, Corrosive Projection, flat base-armor
strips, Hydroid); per-type minimum 1; shields (Toxin bypass and exceptions, gate, recharge for TTK); overguard
(modifiers, caps, Fortifier); health (Viral stage); vulnerability stage; innate DR; unique DR (attenuation, flat DR,
adaptation, Orphix) and T-28; layer order and overflow (CD-08); body-part data and head-crit capability supplied to
`02`.

Not here: status values and timers (`03`: Viral and Magnetic multipliers, Corrosive and Heat strip functions, stack
caps as a rule), the source catalogue of vulnerabilities, strips and M_OG (`04`), the TTK procedure and the EV
expectation algorithm (`08`).

### 1.2 Stages owned

- ST-8 (enemy side per type), section 16.
- ST-9 (layer routing and depletion), section 17. The TTK procedure that drives ST-9 belongs to `08`.
- Pool construction (no stage number; runs once per scenario), sections 6 to 8.

### 1.3 Citation keys added by this document

Keys of 00 section 0.2 apply. In addition:

| Key | File |
|---|---|
| `G:slider` | `research/external/cache/wiki-gadgets/enemyinfoboxslider.js` (MediaWiki:Gadget-enemyinfoboxslider.js, the wiki's own scaling implementation, edited 2026-10-07) |
| `M:infobox` | `research/external/cache/wiki-modules/raw/Enemies_infobox.lua` (Module:Enemies/infobox, 2026-10-08) |
| `M:enemies/<f>` | `research/external/cache/wiki-modules/raw/Enemies_data_<f>.lua` (Module:Enemies/data/<f>) |
| `M:dt` | `research/external/cache/wiki-modules/raw/DamageTypes_data.lua` (Module:DamageTypes/data, 2026-11-13) |

Numbers in this document were computed with `python3` (numpy 2.5 float32 for NP-2). Equations are numbered `E-5.k`.
New discrepancies use the overview range `D-501` ... `D-599` (the brief calls this range "D-05-xx"); new open
questions use `OQ-501` ...; edge cases `EC-05-nn`; test vectors `TV-05-nn`.

---

## 2. Inputs

| Symbol | Meaning | Owner |
|---|---|---|
| `W_t(k)` | weapon-side damage of type t for crit tier k, after quantization, Y, (1 + F), G, Part, c_k (E-12) | `01`, `02`, `04` |
| `part`, `headcrit` | body-part factor and head-crit factor of the instance (E-7) | `02` (data from here) |
| `ms` | modded multishot (E-10), used by attenuation and DPS-scaled DR | `02` |
| merged beam count b of a beam tick (CD-10) | used by DPS-scaled DR (E-5.22) | `02` |
| `fire_rate` | modded fire rate, used by DPS-scaled DR | `06` |
| melee hits per second of the stance schedule | used by DPS-scaled DR for melee attacks (E-5.22) | `07` |
| weapon class (primary, secondary, melee, arch-gun ...) and radial flag of every direct and radial instance; inducing instance of every DoT tick, Blast stack hit and extra hit | instance class of section 14 | `06`, `07`, `02`, `03`, `04` |
| `stacks[s]` | live stacks on the target (Viral, Magnetic, Corrosive, Cold, Puncture ...) | `03`, `08` |
| `viral_mult(n)`, `mag_mult(n)` | Vir(n), Mag(n) | `03` |
| `corr_strip(n)` | c_corr(n) | `03` |
| `heat_step` k_h(τ), `heat_strip` | Heat strip step counter (target state) and `heat_strip = 1 − 0.5^(k_h/4)` (03 E-3.8) | `03` |
| `og_present` | rule that decides when the overguard-dependent caps apply (03 E-3.2) | `03` |
| `vuln` entries | value, multiplicative or additive group, layer mask, type mask, class condition, source kind (Warframe ability or not, section 13) | `04` (catalogue) |
| ability strips (`04` §5.7), Corrosive Projection count and aura strength (`04` §5.8), flat base-armor strips and point strips (Shattering Impact, Metal Auger, Polarize, Sharpened Claws: `04` §5.13) | per-source values at the caster | `04` |
| Shield Disruption (0.18 per aura, cap 0.80) and ability shield strips | E-5.16; values cited there (`W:Shield`) | this document |
| `M_OG` | overguard-only weapon multiplier (Secondary Fortifier); not the overguard level multiplier Λ_OG | `04` |
| T-03, T-06, T-07, T-15, T-23, T-26 (`dmgVulnerability`), T-28 | toggles | `00` section 6 |

## 3. Outputs and symbols added

Outputs: the scaled pools `pool[ℓ]` (OG, SH, HP), `armor_init` A0, `armor_eff` A_eff, the per-layer per-type
enemy-side damage D^ℓ_t of every instance, routed pool changes, and the per-enemy data records of section 4.

Symbols added (prefixed per 00 section 0.3):

| ID | Symbol | Unit | Meaning | Defined in |
|---|---|---|---|---|
| `scale_f1`, `scale_f2` | f1, f2 | factor | endpoint curves `1 + C_i·q^E_i` (binary32; C_i, E_i are the table coefficients of 6.1) | E-5.2 |
| `scale_t` | t_lvl | fraction ∈ [0,1] | position inside the transition band | E-5.2 |
| `scale_s` | s_lvl | weight ∈ [0,1] | smoothstep weight | E-5.2 |
| `scale_mult[stat]` | Λ_H, Λ_S, Λ_A, Λ_OG | factor | level multiplier per stat (Λ_OG is not the overview's M_OG) | E-5.2 |
| `enemy_base[stat]` | B_H, B_S, B_A, B_OG | HP, armor | base value after SP, Eximus, squad and mission modifiers, rounded once to binary32 | E-5.4 |
| `enemy_exi_d` | d_exi | level | Eximus step variable `L − L0` | E-5.5 |
| `enemy_exi_k` | k_exi(d_exi) | factor | Eximus base step factor | E-5.5 |
| `enemy_exi_c` | c_exi | factor | Eximus base coefficient 0.25 or 0.375 | E-5.5 |
| `enemy_party_size` | n_party | count | squad size of the scenario (T-23; 1..4, 1..6 in Icebind) | E-5.4 |
| `enemy_party_mult` | P_party | factor | squad-size health multiplier | E-5.6 |
| `armor_base` | A_base | armor | base armor at BaseLevel (data) | 4.1 |
| `armor_flat_strip` | s_flat | armor | base-armor points removed so far (Shattering Impact, Metal Auger) | E-5.10 |
| `armor_flat_factor` | f_flat | fraction | `max(0, B_A − s_flat) / B_A` | E-5.10 |
| `armor_add_strip` | Σ_add | fraction | additive total-armor strip group | E-5.10 |
| `armor_pts_strip` | P_pts | armor | point strips (Polarize) | E-5.10 |
| `armor_hydroid` | – | bool | target damaged by Hydroid (Corrosive variant) | E-5.12 |
| `armor_mult` | m_arm | factor | `1 − DR(A_eff)` | E-5.9 |
| `type_res[t]` | ρ^res_t | fraction | enemy-specific type resistance (data); not the overview's composition ratio r_t | E-5.1 |
| `phi_eff[ℓ][t]` | φ'_t | factor | φ_t combined with every type resistance | E-5.1 |
| `shield_bypass` | B_SH | set of types | types that skip SH | E-5.15 |
| `shield_gate_factor` | g | factor | 0.05 / 0 / 1 for HP damage during the 0.1 s gate | E-5.25 |
| `dr_class[c]` | – | fraction | class-conditional flat DR (radial_or_melee, non_melee, radial) | E-5.20 |
| `dr_total[ℓ]` | R_ℓ | factor | product of all non-armor DR factors on layer ℓ | E-5.20 |
| `att_frac` | p_att | fraction | MDPI cap as a fraction of max health | E-5.21 |
| `att_cap` | C_att | HP | `p_att · HP_max` | E-5.21 |
| `att_in` | X_att | HP | summed enemy-side damage of one instance on one layer, input of Att | E-5.21 |
| `att_ms` | MS_att | factor | multishot term of the attenuation formula | E-5.21 |
| `dpsdr_in` | X_dps | HP/s | average non-crit DPS input of the Orphix formula | E-5.22 |
| `dpsdr_mod` | M_dps | factor | DPS-scaled DR multiplier (Orphix) | E-5.22 |
| `adapt_res[t]` | – | fraction | adaptation resistance on type t (layered pass only) | E-5.23 |
| `vuln[ℓ][t]` | V_ℓ,t | factor | vulnerability on layer ℓ and type t; a local refinement of the overview's `vuln` (same ID, new shape), proposed to the manager in OQ-523 | E-5.19 |
| `route_*` | Z, a_sh, H_rt, gate_until | HP, fraction, HP, s | local variables of the routing function | E-5.25 |

---

## 4. Enemy data model

### 4.1 Enemy record

Primary source: wiki `Module:Enemies/data/<faction>` (12 live modules, 1020 records counted with `python3` over
`research/external/cache/poc/out/Enemies_data_*.json` without `pre-U36`; `DS:wiki` §6 counts 1019). WFCD `Enemy.json`
is stale (Damage 2.0 classes, no base level, no overguard) and MUST NOT be used (`DS:wfcd` §6); the DE Public Export
has no enemies (`DS:pe` §1.6); `Enemies/data/pre-U36` is a frozen archive and MUST NOT be used (`DS:wiki` §6). The
inspiration's `enemies-min.json` is a cross-check only (CLAUDE.md). Fields without a module source are hand-authored,
each row with a citation (sections 4.4, 4.5).

| Field | Type | Source | Default and rule |
|---|---|---|---|
| `id` | string | `<module>:<key>`: the module file's faction suffix and the record's key, e.g. `infestation:Juggernaut` | unique; the key alone is not (`Death Scuttler` is a key in `themurmur` and `unaffiliated`) |
| `internal_name` | string or none | `General.InternalName` | not a key: empty on 186 records and shared by several (`Heavy Gunner` and `[Wiki Dummy]`; `The Fragmented` and `The Fragmented One`; Gyre and Kosma Raiders) |
| `name`, `display_name` | string | `General.Name`; `display_name` = the name, or the module key when two records share the name (4.6) | |
| `link` | string | `General.Link` (`Page` or `Page#Variant`) | join key for wiki tables (4.3) |
| `faction` | Faction | `General.Faction`, normalised per 4.2 | drives scaling class (6.2) and the bane bucket (`01`) |
| `damage_faction` | Faction | `General.FactionDamageOverride` if the key exists, else `faction` (4.2) | drives φ (section 5); `M:infobox` L607 |
| `scaling_class` | {Grineer, Corpus, Infested, Orokin, Techrot, Default} × {health, shield} | `Stats.FactionScaling`, else from `faction` (6.2) | `M:infobox` L451 |
| `enemy_type` | string | `General.Type` (`"Boss"`, `"Heavy Ranged"` ...) | free text; used only through `boss` |
| `boss` | bool | `General.Type` exactly one of `"Boss"` (62), `"Field Boss"` (33), `"Grand Boss"` (5), `"Secret Boss"` (2) | per-record override list in 4.4; drives only the sourced boss caps of 4.4 and `eximus_capable` |
| `base_level` L0 | int ≥ 1 | `Stats.BaseLevel` | 1 if absent (`M:infobox` L435); the Archons have only `SpawnLevel` |
| `spawn_level` | int | `Stats.SpawnLevel` | UI default level; else L0 |
| `health`, `shield`, `armor` (A_base) | f64 ≥ 0 | `Stats.Health`, `Stats.Shield`, `Stats.Armor` | 0 = layer absent |
| `overguard` | f64 ≥ 0 | `Stats.Overguard` (12 entries) | 0; semantics in section 8 |
| `overguard_context` | none or text | hand-authored | none; Dax Malleus "Only in The Undercroft/The Circuit" (`W:Overguard` L58): the record's overguard is used only when the scenario option `context_overguard` is on (default off) |
| `eximus_health`, `eximus_shield`, `eximus_armor`, `eximus_overguard` | f64 or none | `Stats.EximusHealth`, `EximusShield` (also typo `EximusSHield`), `EximusArmor`, `EximusOverguard` | none; 0 means none (`G:slider` L129-138) |
| `sp_health_bonus`, `sp_shield_bonus` | f64 | `Stats.SteelPathHealthBonus`, `SteelPathShieldBonus` | 0; commented-out module lines are not data (`M:enemies/themurmur` L66, L91) |
| `archimedea_health_bonus` | f64 | `Stats.ArchimedeaHealthBonus` (3 records) | 0 |
| `default_steel_path`, `default_eximus`, `default_mission` | bool, bool, none/empowered/icebind | `General.SteelPathDefault` (4 records), `EximusDefault` (1: Prosecutor), `EmpoweredDefault` (13), `IcebindDefault` (0 today) | scenario defaults when the user picks the enemy (`M:infobox` L447-450; the gadget pre-checks its boxes from them, `G:slider` L97-100); never forced |
| `eximus_capable` | bool | `W:Eximus/Compatibilities` (transcluded by `W:Eximus`; cached `Eximus__Compatibilities.wiki`): a record is capable when a row links to its `link` base page; the row's ticked kinds are stored as `eximus_kinds` | the page is tagged UpdateMe for Murmur and Surge; records of factions the page does not cover fall back to: false when `boss`, faction Sentient or Stalker (`W:Sentient` trivia), else true, with a data warning; cross-check: the inspiration's `eximusOff` flags |
| `eximus_og` | bool | hand-authored | true; false for records listed in the Archwing section of `W:Eximus/Compatibilities` (L836-) and for Warden and Prosecutor records (`W:Eximus` L179) |
| `parts[]`, `default_part` | Part list, part name | section 4.3 | |
| `status_immune` | set of StatusType or ALL | section 4.4 | empty |
| `status_caps` | map StatusType → count, plus `all` | section 4.4 | none |
| `no_freeze` | bool | hand-authored | false; true for Demolishers (`W:Demolisher` L85). Named as `03` consumes it |
| `vip` | bool | hand-authored | false; Puncture cap 3 while overguard is present (U36 fix, `F:status` §4.2). No source lists the VIP enemies yet: every record is false (OQ-515). Named as `03` consumes it |
| `toxin_blocked` | {none, absorb, nullify} | hand-authored | none; Hounds `absorb`, Treasurer `nullify` (10.2) |
| `shield_regen` | {normal, none_after_break, none} | hand-authored | normal; Amalgams `none_after_break`, Treasurer `none` (`F:enemy` §4.5) |
| `shield_weapon_immune` | {none, all, displayed_type} | hand-authored | none; Eidolons `all`, Profit-Taker Orb `displayed_type` (10.6) |
| `shield_strip_immune` | bool | hand-authored | false; Eidolons and Profit-Taker Orb true (10.3) |
| `armor_strip_immune` | {none, flat_only, all} | hand-authored | none; Archons in Archon Hunts and Profit-Taker Orb `all`, Commandeered Primes `flat_only` (9.6) |
| `ability_debuff_immune` | {none, all, dispelled} | hand-authored | none; section 13.1 |
| `type_res[t]` | fraction per DamageType | hand-authored | 0 (section 4.5) |
| `innate_dr[SH]`, `innate_dr[HP]`, `innate_dr[OG]` | fraction | hand-authored | 0 (section 4.5); OG entry is 0 for every enemy unless a row says otherwise |
| `status_dr` | fraction | hand-authored | 0 (Archons 0.5) |
| `dr_class[c]` | fraction per class c ∈ {radial_or_melee, non_melee, radial} | hand-authored | 0 |
| `dr_class_og[c]` | fraction per class, on the overguard layer only | hand-authored | 0 (Guardian Eximus radial 0.9) |
| `attenuation` | none, unknown, or {p_att, layers ⊆ {SH, HP}} | hand-authored | none (15.1) |
| `dps_dr` | none or table | hand-authored | none (Orphix, 15.3) |
| `adaptation` | none or {kind, res[1..4], extra, gates, hit_caps} | hand-authored | none (15.4) |
| `party_scaling` | none or table n → P_party | hand-authored | none (Demolisher family, 7.4) |
| `is_object` | bool | `General.Faction == "Objects"` | false (EC-05-60) |
| `adversary` | bool | hand-authored (Kuva Lich, Sister of Parvos, Technocyte Coda) | false; no module record exists for them (4.5); health per rank is a data gap (OQ-526) |

Every enemy-specific field above is data, not engine code: the engine never branches on an enemy's name.

**Keys of hand-authored rows.** Every hand-authored row of sections 4.3 to 4.5 is stored as an explicit list of `id`
values, never as a name pattern or a free-text group. A row that names a group lists its members once, in the data
build, with the citation for the group. A row whose enemy has no module record (Kuva Lich, Sister of Parvos,
Technocyte Coda, Kuva Lich Thralls, Sister Hounds, Skittergirl: 0 records in the live modules, `python3` search) is
kept as a **data gap**: it attaches to nothing until a base record (health, shield, armor, base level) is hand-authored
with a citation (OQ-526). The group memberships used by this document:

| Group | Members (`id`) | Source |
|---|---|---|
| Demolisher family | every record whose key starts with `Demolisher ` or `Demolyst` (22): `corpus:` Demolisher Anti MOA, Demolisher Bursa, Demolisher Hyena, Demolisher Machinist, Demolyst, Demolyst Heqet, Demolyst MOA, Demolyst Machinist, Demolyst Osprey, Demolyst Satyr; `grineer:` Demolisher Bailiff, Devourer, Expired, Heavy Gunner, Kuva Guardian, Nox; `infestation:` Demolisher Boiler, Charger, Juggernaut, Thrasher; `themurmur:` Demolisher Bonewidow, Demolisher Voidrig | `W:Demolisher` L1-7 ("Demolishers ... The Amalgams ... are called Demolysts"; the pulse exception names the two Necramechs as Demolishers) |
| Enemy Necramechs | `themurmur:` Rogue Voidrig, Rogue Voidrig (Deimos), Rogue Voidrig (Descendia), Rogue Bonewidow, Deimos Rogue Bonewidow, Rogue Bonewidow (Descendia), Demolisher Voidrig, Demolisher Bonewidow | `W:Damage/Radiation_Damage` L27 ("enemy Necramechs") |
| Sentient adaptation | `sentient:` Aerolyst, Anu Mantalyst, Anu Pyrolyst, Battalyst, Decaying Battalyst, Conculyst, Decaying Conculyst, Mimic, Ortholyst, Summulyst, Symbilyst, Anu Symbilyst (their pages transclude `Sentient#Damage Adaptation`); not: Tyro Battalyst, Tyro Conculyst ("Tyro ... don't possess any innate damage adaptation", `W:Sentient` L155), Brachiolysts (L77), Choralyst, Oculyst, Gyrix, Ionyx, Dactolyst, Hunhullus, Typholyst, Ropalolyst, Splintrix, Anu Interference Drone, Eradicyst (no adaptation section), Eidolons, Vomvalyst, Orphix, Condrix (own mechanics) | `W:Sentient` L293-323; pages cached 2026-11-24 |
| Juggernauts (90 %) | `infestation:` Juggernaut, Juggernaut Behemoth, Deimos Juggernaut; not Polyp-Hog Juggernaut (no source) | `W:Juggernaut` L31; `W:Enemy_Body_Parts` L670, L697; `W:Damage_Reduction` "Juggernaut (Behemoth)" |
| Acolytes | `stalker:` Angst, Malice, Mania, Misery, Torment, Violence | `W:Acolytes` |
| Archons | `narmer:` Archon Amar, Archon Boreal, Archon Nira | `W:Archon` |
| Eidolons | `sentient:` Eidolon Teralyst, Eidolon Gantulyst, Eidolon Hydrolyst | `W:Eidolon_Teralyst` |
| VIPs (`vip`) | none known | OQ-515 |

**Eximus kind.** Entropic and Guardian are Eximus kinds, not records, and T-23 has only an Eximus boolean. Until
OQ-523 adds a kind input, the scenario carries the fixed default `eximus_kind = generic`: the Entropic and Guardian rows
of sections 4.5, 7 and 13.1 are data for a later toggle and are inactive.

### 4.2 Faction strings and normalisation

`General.Faction` values in the 12 live modules (count): Corpus 308, Grineer 253, Infested 65, Corpus Amalgam 44,
Narmer 42, Sentient 33, Kuva Grineer 29, Orokin 29, Anarchs 29, Infested Deimos 28, The Murmur 27, Predator 24,
Prey 24, Neutral 20, Techrot 13, Unknown 12, Scaldra 11, Duviri 11, Stalker 9, Tenno 5, Objects 2, `?` 2 (counted
with `python3` over `research/external/cache/poc/out/Enemies_data_*.json`).

Mapping to the overview's Faction enum (00 section 2.2):

| Data string | Faction | Note |
|---|---|---|
| `Grineer`, `Kuva Grineer`, `Corpus`, `Corpus Amalgam`, `Infested`, `Infested Deimos`, `Orokin`, `Sentient`, `Narmer`, `Scaldra`, `Techrot`, `Anarchs`, `Stalker`, `Tenno`, `Zariman` | same name | `Orokin` is the Corrupted faction |
| `The Murmur` | Murmur | |
| `Neutral`, `Predator`, `Prey`, `Unknown`, `Duviri`, `?` | Neutral | no damage table exists for these strings in `M:dt` |
| `Objects` | Neutral with `is_object = true` | `W:Damage/Object` |

`FactionDamageOverride` (152 entries): `""` (126) → Neutral (the infobox passes the empty string to
`DamageTypes.healthMod`, which renders no modifiers; `M:infobox` L607; `F:enemy` §2); `"Grineer"` (5), `"Corpus"`
(1), `"The Murmur"` (2), `"Zariman"` (12) → that faction; an internal path such as
`/Lotus/Types/Enemies/Corpus/Gamemodes/PurgatoryWarriorRanged` (6) → Neutral with a data warning (the wiki renders
no modifiers; UNVERIFIED that the game does the same, EC-05-07).

### 4.3 Body parts and head-crit capability (supplied to `02`)

Part record:

| Field | Meaning | Default |
|---|---|---|
| `name` | hitbox name | |
| `column` | `general`, `weak_point` or `other` (column of the source table; Multis parts per rule M below) | `general` |
| `part_mult` (P_part) | built-in damage multiplier | 1.0 |
| `is_weak_point` | weak-point flag (WP damage bonuses, WP crit chance, on-weak-point effects, gate bypass) | false |
| `is_head` | headshot flag (headshot-multiplier bonuses, k_wp = 1.5) | false |
| `headcrit` (h) | factor on the weapon's crit multiplier, `c_k = 1 + k·(h·CDM − 1)`; values outside the overview's {1, 2} (4, 1.5, 0.2) are `02` OQ-203 | 1.0 |
| `tier_add` | critical promotion: crit tier + `tier_add` (Rogue Culverin canisters, Scaldra backpacks) | 0 |
| `crit_disabled` | hits on the part never crit (tier forced to 0): Scaldra TI-92 turret "downgrades all hits to non-crits" (`W:Enemy_Body_Parts` L925, L939) | false |
| `wp_bonus_ok` | weak-point damage bonuses ΣWP apply on the part | true; false on H-09 Efervon Tank (Techrot) growths and proboscis ("not increased by Acuity mods", L38, L940) |
| `part_dr` | flat damage reduction of instances that hit the part, a factor `(1 − part_dr)` inside R_ℓ (E-5.20) | 0; Scaldra Dedicant backpack and abdominal flask 0.30 (L919, footnote ⁴ L937); breaking the part is out of scope (EC-05-83) |
| `status_ok` | the part accepts status procs | true |
| `no_armor`, `no_faction_mod` | non-body hitboxes (Symbilyst shield, Aerolyst canisters, Efervon growths): no armor (E-5.18) and φ'_t = 1 on SH and HP (E-5.1) | false |
| `is_default` | the part used for `body` hits and every radial instance (selection rules below) | |

**Sources and ingestion**, in priority order (`W:Enemy_Body_Parts`, edited for U44; `F:core` §10). The data build
reads them; every derived part keeps its source (table row, Multis string, faction default, hand row).

1. **Table** (T): the "Special Body Part Multipliers" tables (`W:Enemy_Body_Parts` L256-997) are parsed by the data
   build (they are not in `DS:wiki` or the overview §1.5 plan; this document adds them, OQ-530). Column semantics
   (L259-266): `General` = bodyshot parts with P ≤ 1; `Weak Point` = weak point and headshot with the critical-headshot
   bonus unless marked `*` (no headcrit) or `†` (not a headshot); `Other` = bodyshot parts with P > 1 or a special
   interaction. Flags: General → `is_weak_point = is_head = false`, h = 1; Weak Point → `is_weak_point = true`,
   `is_head = !†`, h = 2 unless `*`; Other → `is_weak_point = is_head = false`, h = 1. Footnote markers (¹ ² ³ ...) are
   table-local; the build maps each footnote text to fields by a hand-written table that cites the line: "Bodyshot, but
   applies critical headshot bonus" → h = 2; "... at 4x base mult" → h = 4; "Promotes hits to next crit tier" →
   `tier_add = 1`; "Not a weak point or headshot, but grants headcrit bonus" → h = 2 on a General part;
   "Non-weakpoint headshot location" → `is_head = true`, `is_weak_point = false`; "No armor, downgrades all hits to
   non-crits" → `no_armor`, `crit_disabled`; "Damage reduction of 30%" → `part_dr = 0.30`; "not affected by Acuity" →
   `wp_bonus_ok = false`. Row join: the unit cell's `{{E|X}}` / `[[X]]` link target X matches records whose `link`
   equals X (exact match first), else records whose `link` base page (before `#`) equals X ("Stats apply to all
   variants unless stated otherwise, including Corrupted and Narmer versions", L268), else records whose `name` is X.
   A row with no matching record is reported. Unit-cell notes ("Innate 90% Damage Reduction", "80% Toxin Resistance",
   "60% damage reduction from sources in front of it") are reported as candidates for section 4.5; they are never
   applied automatically.
2. **Multis** (M), only for records without a table row: `Stats.Multis` strings in `M:enemies` (717 of 1020 records;
   `"Head: 3.0x"` 447 times, `""` 81, `"Head: 1.0x"` 42, `"Fanny Pack: 3.0x"` 42, 89 distinct strings, `python3` over
   the live modules). The 303 records without Multis (their `--Multis = { "?" }` placeholder lines are commented out)
   use the faction default. Grammar `<name>:<spaces><number>[x|X][ (<note>)]` (accepts `"Head: 3.0"`, `"Body:0.5x"`,
   `"Shoulders: 3.0x (Incarnon Compatible)"`). Flag rules (Decision, UNVERIFIED): name `Head` → `is_head`,
   `is_weak_point`, h from the faction default table below; any other name with P > 1 → `is_weak_point` only, h = 1,
   column `weak_point`; P ≤ 1 → `general` part. Hand-mapped strings: `"Head: +1 Crit Tier, 1.5x Critical Damage"`
   (5 records, Dax) → Head with P = 1 (the string gives no damage multiplier; Decision), `tier_add = 1`, h = 1.5;
   `"Energy Emissions (elbows): 1.5x, +1 Crit Tier"` → weak point P = 1.5, `tier_add = 1`; `"Stealth/Finisher: 16.0x"`
   and `"Stealth: 4.0x"` are not parts (finisher data for `04`); `"All: 0x"`, `"None"`, `""`, `"Head: ?x"` and any
   other unparsable string → no part data, faction default, data warning.
3. **Faction defaults** (F), when neither source covers the enemy (`W:Enemy_Body_Parts` L13-14 and L43-46):

| Faction | Head P | `is_head` | head `headcrit` h | Note |
|---|---|---|---|---|
| Grineer, Kuva Grineer, Orokin, Narmer, Anarchs, Scaldra | 3.0 | yes | 2 | headcrit "activates reliably" |
| Infested, Infested Deimos, Sentient | 3.0 | yes | 2 | "most" units |
| Corpus, Corpus Amalgam | 3.0 | yes | 1 | no headcrit on humanoids and ground units |
| Murmur, Techrot | 1.5 | yes | 1 | "usual multiplier closer to 1.5x"; "most"/"much of" cannot be headcrit |
| all other factions | 3.0 | yes | 1 | Decision, UNVERIFIED (no source lists them as headcrit-capable) |

A faction default adds `{Head, weak_point}` and `{Body, P = 1, h = 1, general}`.

4. **Hand rows** (H) override the result per `id`, each with its citation: Juggernauts (group of 4.1): body h = 2
   and `status_ok = false` on every part except maw and belly; maw and belly P = 10, h = 4, `status_ok = true`
   (`W:Juggernaut` L31-37; the maw accepts status only while open: Decision `status_ok = true`, UNVERIFIED, EC-05-84);
   Demolisher Juggernaut boils P = 0.5, h = 1, `status_ok = true` (L38-39); Ropalolyst head h = 0.2 (L46, table
   footnote L812); Hemocyte: outer skin P = 0 ("invulnerable to all weapons", `W:Hemocyte` L19), head weak spots
   `is_weak_point`, P = 1, h = 2, Grineer head chest weak spot P = 2, h = 2 (L29-30), `default_part` = head weak spot;
   Lephantis: h = 2 on every part (`W:Lephantis` L29), Body P = 0, Mouths P = 1 (table L676),
   `default_part` = Mouths; Jordas Golem: Body P = 0, Engines P = 1 with headcrit (table L674; `F:core` §10),
   `default_part` = Engines; Eidolons: body P = 2, h = 2; head P = 3, h = 2 (`W:Eidolon_Teralyst` L77; the table L807
   lists Body 1x and Head 1x, a conflict; Decision: the per-enemy page, UNVERIFIED, EC-05-49); Isolator Bursa:
   adds the part Front (P = 0.4, `general`) from its Multis `"Front: 0.4x"` (`W:Isolator_Bursa` infobox) to the table
   row and makes it the `default_part`; it is the table's "60% damage reduction from sources in front of it" (L500)
   expressed as a part (Decision, UNVERIFIED). U44 non-head weak points
   (MOA rears except Anti, Disc and Fusion, Bursa panels, Severed Warden arms, Scaldra Harbinger balloon, Amalgam
   Heqet backpack, Lumbering Fragment, `W:Enemy_Body_Parts` L41) come from the table's `†` marks.

The Juggernaut maw and belly are in the table's Other column ("Bodyshot, but applies critical headshot bonus at 4x",
L670-679), while `W:Juggernaut` L32-33 calls them "weakpoints". Decision: the table's flags (bodyshot: no weak-point
or headshot bonuses), because it states the hitbox flags explicitly; UNVERIFIED, D-509. The Leaping Thrasher back
skull (3x) is likewise a bodyshot (L40, L609).

**Part selection** for T-03 (`02` consumes it), deterministic:

- `head`: the part named `Head` with `is_head`; else the `is_head` part with the highest P (ties: table order). Nox:
  Head (3x), not Helmet (2x), which comes first in the row (L330). No head part → `body`, result flag `no_head_part`.
- `weak_point`: the non-head weak point with the highest P (ties: table order). If none exists, the `other` part with
  the highest P > 1 (Juggernaut maw, Leaping Thrasher back skull) is used as a bodyshot with flag
  `special_part_not_weak_point` (no weak-point bonuses, no gate bypass); if that does not exist either, `body` with
  flag `no_weak_point` (the UI marks the option). Decision, UNVERIFIED; a named-part selector is proposed in OQ-523.
- `body`: the part with `is_default`: the hand row's `default_part` if any; else the `general` part named `Body`;
  else the first `general` part with P = 1 in table order (MOA: Legs); else the first part in table order. Enemies
  with several General parts (Scaldra Dedicant and Screamer: Upper Body 2x with headcrit, Arms 1x, Legs 1x; Deimos
  Charger: Body 1x, Carapace 0.05x, Face 0.2x; Bailiff: Body 1x, Armor Plates 0.5x) therefore use Body, or Arms for
  the Dedicant and Screamer (first P = 1 part; Decision, UNVERIFIED: a centre-mass hit on them lands on the 2x Upper
  Body, which the named-part selector of OQ-523 would expose).
- Radial instances always use the `body` part's P_part and h, never a head or weak point (`F:weapon` §10;
  `W:Eidolon_Teralyst` L77: "AOE weapons will not benefit from the 3x headshot multiplier ... Body hits with AOE
  weapons will still benefit from the 2x body part multiplier and 2x critical damage bonus"). E-7 writes `Part = 1`
  for body and radial hits; that equals `P_body` for every enemy except the hand rows above (OQ-524).

### 4.4 Status immunities and per-enemy status caps (supplied to `02` and `03`)

Immunity is a set of StatusType values or ALL. Sources: `Stats.ProcResists` in `M:enemies` (71 records carry the key,
47 non-empty, in the live modules; for example Demolishers `{"Radiation"}`, Deimos units `{"Viral"}`, The Fragmented `{"Cold",
"Viral"}`), plus the hand-authored rows below (keyed by `id` per 4.1). `02` removes immune types from the proc weights
and renormalises (`F:status` §2.2); `03` applies no effect and creates no DoT for an immune type. Immunity to a status
does not remove the damage of that type (`F:status` §5); damage immunity is `type_res = 1`.

| Enemy or group | Immunity | Source |
|---|---|---|
| Arbitration Shield Drone, Desert Skate, Exploiter Orb, Profit-Taker Orb, Regulator, Ropalolyst, Eidolons including Vomvalysts, Tusk Thumper, Tusk Thumper Bull, Tusk Thumper Doma | ALL | `W:Status_Effect` L838-850 "Full Immunity" (section tagged UpdateMe); Eidolons: `W:Eidolon_Teralyst` L81 says "immune to all Status Effects, except for Void and Tau" (conflict D-511; decision ALL, UNVERIFIED, OQ-534) |
| Hemocyte | ALL | `W:Hemocyte` L19 (the weak spots are "immune to all Status Effects"), L89 ("While the Hemocyte is immune to status effects, its armor can still be reduced"); the module record has no `ProcResists`. Its outer skin takes no weapon damage (L19): parts in 4.3 |
| Viral partial-immunity list: Ambulas, Basal Diploid, Deimos Carnis, Deimos Genetrix, Deimos Jugulus, Deimos Leaping Thrasher, Deimos Saxum, Deimos Therid, Demolisher Boiler, Demolisher Charger, Demolisher Juggernaut, Demolisher Leaping Thrasher, Feral Diploid, Juggernaut, Leaping Thrasher, Mitosid, Undying Flyer | VIR | `W:Status_Effect` L853-870 (the plain Leaping Thrasher is listed separately, L868); the module's `ProcResists` agree |
| Juggernauts (group of 4.1) | ALL on every part except maw and belly (`status_ok = false`); maw and belly accept status | `W:Juggernaut` L31-33 |
| Infested Demolishers | VIR | `W:Demolisher` L82 |
| Demolisher Juggernaut, Demolisher Charger | VIR (status) and `type_res[VIR] = type_res[TOX] = 1` (damage) | `W:Demolisher` L82 |
| Ambulas, Deimos Carnis/Rex, Genetrix, Jugulus, Leaping Thrasher (Deimos), Therid, Saxum/Rex, Demolisher Boiler/Thrasher, Tusk Bolkor, Tusk Firbolg, Techrot Babau | VIR | `W:Damage/Viral_Damage` L58-72 (`F:status` §4.13) |
| The Fragmented, Leaping Thrasher, Scaldra TI-92 | VIR ("Immune" under "Sources of Viral Resistances") | `W:Damage/Viral_Damage` L150-156; The Fragmented also COL (`W:Damage/Cold_Damage` L292; `ProcResists`) |
| Alkonost (Amalgam) | ALL | `F:enemy` §9.2 |
| Anu Pyrolyst, Choralyst | ALL | page infobox `procresistance = All` (`W:Anu_Pyrolyst` L14, `W:Choralyst` L14); a legacy template parameter (the same infoboxes still use pre-U36 `ferritearmor`, and `W:Mimic` L66 notes the drones are "no longer immune"), so UNVERIFIED (OQ-534); the inspiration agrees for Anu Pyrolyst |
| Manics, Manic Bombards | RAD | `W:Damage/Radiation_Damage` L31-34 (`F:status` §4.12) |
| Enemy Necramechs (group of 4.1) | none from this rule; `F:enemy` §9.2's "immune to Knockdown, Stagger and status effects" comes from `W:Necramech` L102, which describes player Necramechs, while `W:Damage/Radiation_Damage` L27 gives enemy Necramechs a Radiation cap of 4; the Demolisher Bonewidow and Demolisher Voidrig records have no `ProcResists` in the module (the Radiation `ProcResists` belong to the Grineer, Corpus and Infested Demolishers and the Demolysts) | Decision: not immune, RAD cap 4 (UNVERIFIED, OQ-520) |

The inspiration's data has further partial immunities that no wiki page or module record supports (`W:Status_Effect`
L731 sends "individual status effects" to the damage-type pages; their "Sources of ... Resistances" sections were
checked for every type with one): Heat on Railgun, Vapos Railgun, Taro Railgun, Shockwave, Fusion, Vapos Fusion and
Anti MOAs and the Fire Prosecutor; Impact on Remech Osprey and Scyto Raknoid; Viral, Impact and Magnetic on Kyta
Raknoid; six types each on Corrupted Drone and Orbital Strike Drone; Slash on Tomb Guardian; Toxin on Techrot Babau;
ALL on Lephantis, Orphix and Wolf of Saturn Six. They are not applied (D-510, Compat). The inspiration's
Aurax Atloc Raknoid immunity (Viral, Corrosive, Magnetic) equals its module `ProcResists` and is applied.

Per-enemy caps are data; the replacement rule, the extension to DoT instances and the `og_present` condition are
`03`'s (CD-07, `03` E-3.2). Rows are keyed by `id` (4.1):

| Group | Cap | Source |
|---|---|---|
| Acolytes (group of 4.1) | every status 4, IMP 3 | `W:Acolytes` L17 |
| Archons (group of 4.1, Archon Hunt) | every status 4 | `W:Archon_Boreal` L43; `W:Archon_Amar` L47 |
| Kuva Liches, Sisters of Parvos, Technocyte Coda | every status 4; IMP not capped below the normal 5 (U27.3 allowed 6, the global Impact cap became 5 in U33.6) | U27.3 note on `W:Bosses` L925 (`F:status` §2.4); data gap: no records (4.1) |
| records with `boss = true` not listed above | COL 4 (cannot freeze) and BLA 4 (detonation at 4); every other status keeps its normal cap | `W:Damage/Cold_Damage` L27, L364 ("Bosses, such as Alad V or Kela De Thaym"); `W:Damage/Calculation` L643-645 (Blast "bosses cap at 4"); no current page caps the other statuses of ordinary bosses (`F:status` open question 4; OQ-531) |
| Liches, Acolytes, enemy Necramechs, Sisters, Hounds | RAD 4 | `W:Damage/Radiation_Damage` L27 |
| any enemy while `og_present` (`03` E-3.2) | COL 4 (cannot freeze) | `W:Overguard` "can normally only receive a maximum of 4 Cold procs" (U36) |
| `vip` enemies while `og_present` | PUN 3 | U36 "intended 3" (`F:status` §4.2); no VIP list (OQ-515) |
| Demolisher family | COL not capped but `no_freeze = true` (10 stacks slow by 90 % instead) | `W:Demolisher` L85 |
| Void Angel, Demolisher Voidrig, H-09 Efervon Tank (4), Zealoid Prelate (10) | only in the inspiration's data (`maxProcStacks`) | not applied without a wiki source (UNVERIFIED, OQ-515) |

The `boss` flag is the exact `General.Type` set of 4.1 with a per-`id` override list (empty today). The draft's rule
"every status 4 on every `Boss`" was dropped: it capped Viral, Corrosive, Magnetic, Heat and Slash on 62 records
(Commandeered Primes, Hyenas, Jackal, Vay Hek, Kela ...) on the strength of a note about Kuva Liches only.

Emerald Archon Shard raises the Corrosive cap of capped enemies (4 → 6 or 7; `F:buffs` §5); `03` owns the rule.

### 4.5 Special damage-reduction data (hand-authored)

The wiki modules have no DR field (`DS:wiki` §6), so this table is the data; every row is stored as an `id` list
(4.1). Values are from wiki pages; rows that only the inspiration supports are listed in D-502 and are not used. The
data build SHOULD also scan the unit cells of `W:Enemy_Body_Parts` for "Damage Reduction" and "Resistance" and the
enemy pages' infobox parameters `protoshield` and `procresistance`, and report every hit that has no row here.

| Enemy (`id` list) | Field | Value | Source (update) |
|---|---|---|---|
| Archons (group of 4.1) in Archon Hunts | `innate_dr[HP]`, `innate_dr[SH]`; `status_dr`; `armor_strip_immune = all`; `ability_debuff_immune = all`; caps 4; damage_faction Narmer | 0.20; 0.50 | `W:Archon` L32; `W:Archon_Hunt` L117-119 ("immune to most Warframe abilities": `all` is a Decision, UNVERIFIED); U36.0.8 Narmer |
| Juggernauts (group of 4.1) | `innate_dr[HP]`; `dr_class[radial_or_melee]` | 0.90; 0.50 | `W:Juggernaut` L31, L36 |
| Demolisher Juggernaut | `innate_dr[HP]` | 0.20 | `W:Juggernaut` L38 |
| Demolisher Bursa | `innate_dr[HP]` | 0.20 | `W:Bursa` L30 |
| Kuva Lich Thralls (data gap: no record) | `innate_dr[HP]` | 0.25 ("on top of their armor") | `W:Damage_Reduction` L284 |
| Treasurer | `innate_dr[SH]`; `innate_dr[HP]`; `toxin_blocked = nullify`; `shield_regen = none` | 0.99; 0.98 | `W:Treasurer` L30-32. The page was last edited 2025-11-01, after U40.0.2 (its own patch history quotes the U40.0.2 and U40 attenuation notes, L88-96), but no U40 note restates the 99 %/98 % values: UNVERIFIED, OQ-514 |
| Aurax Atloc Raknoid | `innate_dr[HP]`, `innate_dr[SH]` | 0.80 | `W:Aurax_Atloc_Raknoid` L13 and `W:Damage_Reduction` L301 (80 %); `W:Enemy_Body_Parts` L541 says 70 % (D-512) |
| Kyta Raknoid | `innate_dr[SH]` | 0.25 | `W:Damage_Reduction` L302 |
| Scyto Raknoid | `innate_dr[HP]`, `innate_dr[SH]` | 0.50 | `W:Damage_Reduction` L303 |
| Skittergirl (data gap: no record) | `innate_dr[HP]`, `innate_dr[SH]` | 0.90 (U40.0.3; attenuation removed) | `W:Damage_Reduction` L317-318 |
| Hyekka, Hyekka Master, Kuva Hyekka, Kuva Hyekka Master, Nightwatch Hyekka Master | `type_res[HEA]` | 0.80 | `W:Damage_Reduction` L278 ("Hyekkas and Hyekka Masters"); `W:Damage/Heat_Damage` L365; the variants by link (Decision) |
| Toxic Ancient | `type_res[TOX]` | 0.80 | `W:Toxic_Ancient` L29; `W:Damage/Toxin_Damage` L275; `W:Enemy_Body_Parts` L661 |
| Summulyst | `innate_dr[HP]`, `innate_dr[SH]` | 0.30 | `W:Summulyst` L24 ("innate 30% damage resistance, separate from armor and their Sentient adaptation"); `W:Enemy_Body_Parts` L760 |
| Symbilyst, Anu Symbilyst | `innate_dr[HP]`, `innate_dr[SH]` | 0.15 | `W:Symbilyst` L14; `W:Enemy_Body_Parts` L762 |
| Choralyst | `innate_dr[HP]`, `innate_dr[SH]` | 0.25 | `W:Enemy_Body_Parts` L748 only (the unit page has no DR text): UNVERIFIED |
| Mimic, Ortholyst | `innate_dr[HP]`, `innate_dr[SH]` | 0.15 | `W:Enemy_Body_Parts` L752, L756 only: UNVERIFIED |
| Scaldra Screamer | `innate_dr[HP]`, `innate_dr[SH]` | 0.20 | `W:Enemy_Body_Parts` L921 only: UNVERIFIED |
| Amalgam Machinist | `innate_dr[SH]` | 0.50 | `W:Amalgam_Machinist` L24 ("shields have 50% damage reduction"); `W:Enemy_Body_Parts` L781 ("50% Damage Reduction with shields up"); a DR on the shield layer is only active while shields are up, so the two agree; the inspiration has the same value |
| Scaldra Dedicant, Scaldra Screamer | `part_dr` 0.30 on Backpack and Abdominal Flask | | `W:Enemy_Body_Parts` L919-921, footnote ⁴ L937 (4.3) |
| Isolator Bursa | no DR row: its "60% damage reduction from sources in front of it" (L500) is the Multis part `"Front: 0.4x"`, used as its default part (4.3) | | `W:Isolator_Bursa` infobox |
| Entropic Eximus (Eximus kind, inactive until OQ-523) | `dr_class[non_melee]`; Eximus base armor 50; `ability_debuff_immune = all` while the orb is active | 0.90 (U44; double application fixed in U44.0.2) | `W:Eximus` L89-95, L104; U44.0.1 clarification L260-263 ("Damage from non-melee sources is reduced by 90%"); U44.0.2 L246-247 |
| Guardian Eximus (itself; Eximus kind, inactive until OQ-523) | `dr_class[radial]` and `dr_class_og[radial]` | 0.90 | `W:Eximus` L116-118 (2026-11-16; conflicts with `W:Damage_Reduction` L274, EC-05-21) |
| Hemocyte | `attenuation {p_att = 0.05, layers {HP}}` (MDPS 0.20 documented only); status ALL (4.4); parts (4.3) | | `W:Hemocyte` L19-35, L79, L89 |
| Lephantis | `attenuation {0.05, {HP}}` | | `W:Lephantis` L28 |
| Eidolons (group of 4.1) | `attenuation {2.00, {SH, HP}}` (MDPS 1.00 documented only); `shield_weapon_immune = all`; `shield_strip_immune`; `ability_debuff_immune = all` | | `W:Eidolon_Teralyst` L75 (U40.0.3 "greatly increased the MDPI"), L81 (Shield Disruption "has no effect on its shields"), L91, L93, L121 |
| Profit-Taker Orb | `shield_weapon_immune = displayed_type`; `shield_strip_immune`; `armor_strip_immune = all` | | `W:Profit-Taker_Orb` L59-66, L112 ("immune to the effects of Shield Disruption, and any armor-reducing effect (e.g. Corrosive Projection, Shattering Impact and Seeking Shuriken)") |
| Orphix | `dps_dr = orphix` | 15.3 | `W:Orphix` L30-44 |
| Sentient adaptation group (4.1) | `adaptation {sentient}` | 15.4 | `W:Sentient` L293-323 |
| Shadow Stalker | `adaptation {shadow_stalker}` | 15.4 | `W:Shadow_Stalker` L34-44 |
| Amalgams (the 44 records of faction Corpus Amalgam) | `shield_regen = none_after_break` | | `F:enemy` §4.5 |
| Hounds (Sisters of Parvos; data gap: no record) | `toxin_blocked = absorb`; RAD cap 4; abilities last 1/4 as long (flag only) | | `W:Hound` L35 (infobox `protoshield = 1`, L11) |
| Demolisher family (4.1) | `party_scaling = {1: 1.0, 2: 1.5, 3: 2.0, 4: 3.0}` on health; `no_freeze`; `ability_debuff_immune = dispelled` except Demolisher Bonewidow and Demolisher Voidrig | | `W:Demolisher` L5, L7, L85 |
| Scaldra Screamer | `ability_debuff_immune = dispelled` | | `W:Scaldra_Screamer` L8 ("pulse out a red aura every 5 seconds ... dispelling and disabling all" abilities) |
| Enemies keeping standardised U40 attenuation with unknown p (Acolytes, Adversaries, Archons, Infested Oni, Legacyte, Mocking Whispers, Oraxia, Protea, Coda Duets, The Fragmented, Void Angel, Zealoid Prelate, Lich Thralls, Juggernauts) | `attenuation = unknown` | not modelled, result flagged | `F:enemy` §9.1; OQ-510 |

### 4.6 Data hygiene

- Legacy health classes in `M:dt` (`Shield` with Magnetic +75 %, `Indifferent Facade`, `Cloned Flesh`, `Ferrite
  Armor` ...) are pre-U36 and MUST NOT be used (`DS:wiki` §7; `W:Damage/The_Murmur` L2 "preceded by Indifferent
  Facade"). Only entries with `Introduced` ≥ 31.5 that name a faction or the Overguard layer are used (table 5.1).
- The Shield-class `Bypass = {"Toxin", {"True", 50}}` is legacy; the post-U36 bypass set is section 10.2.
- Typos are normalised (`EximusSHield`). Placeholder values (Kuva Lich and Sister 1/1/1, Narmer Deacon −1 in the
  inspiration's data, `I:enemy` §10) do not exist in the wiki modules; any non-positive base value is treated as 0
  with a warning.
- U40 changed many base health values (Treasurer 2,500, Tusk Thumper 9,000, Juggernaut 900, Techrot Babau 10,000 ...;
  `F:enemy` §9.1); the data build MUST read the current modules, not a snapshot older than U40.0.3.
- A record whose `Stats` is not a table is skipped with a warning (`grineer:Grineer Queens` has `Stats = {}` as a
  list, 1 record).
- Duplicate display names: 14 `General.Name` values are shared by two or three records (Cannon Battery, Condor
  Dropship, Senta Turret, Bailiff/Kuva Bailiff, Heavy Gunner/[Wiki Dummy], Kuva Larvling, H-09 Apex, H-09 Efervon Tank,
  Rogue Voidrig (7,500/500 and 600/300), Void Angel, Hollow Thrax Centurion, Hollow Thrax Legatus, Necramite, and two
  records without a name), counted with `python3`. The UI lists them by `display_name` (the module key), and links and
  saved builds store `id`. `[Wiki Dummy]` is a test record and is hidden.
- Context-only overguard (`overguard_context`, 4.1): Dax Malleus's `Stats.Overguard = 8` exists "Only in The
  Undercroft/The Circuit" (`W:Overguard` L58); Nox's `Overguard = 5` is ordinary (removed when its helmet breaks,
  out of scope, EC-05-24).

---

## 5. Faction damage-type modifiers

### 5.1 Table (post-U36)

Two magnitudes exist: vulnerable ×1.5, resistant ×0.5 (`W:Damage/Overview_Table` L3: "As of Update 36, Vulnerable
+ = x1.5 and Resistant - = x0.5"). The table below equals `M:dt` `Health` entries with `Introduced` ≥ 31.5 cell by
cell (checked with `python3`; `F:enemy` §2; `DS:wiki` §7). Empty cell = ×1.0. Types not shown (Tau, True,
Cinematic, Finisher) are ×1.0 for every faction.

| damage_faction | IMP | PUN | SLA | HEA | COL | ELE | TOX | BLA | RAD | GAS | MAG | VIR | COR | VOI | Update |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Grineer | 1.5 | | | | | | | | | | | | 1.5 | | U36 |
| Kuva Grineer | 1.5 | | | 0.5 | | | | | | | | | 1.5 | | U36 |
| Corpus | | 1.5 | | | | | | | | | 1.5 | | | | U36 |
| Corpus Amalgam | | | | | | 1.5 | | 0.5 | | | 1.5 | | | | U36 |
| Infested | | | 1.5 | 1.5 | | | | | | | | | | | U36 |
| Infested Deimos | | | | | | | | 1.5 | | 1.5 | | 0.5 | | | U36 |
| Orokin (Corrupted) | | 1.5 | | | | | | | 0.5 | | | 1.5 | | | U36 |
| Sentient | | | | | 1.5 | | | | 1.5 | | | | 0.5 | | U36 |
| Narmer | | | 1.5 | | | | 1.5 | | | | 0.5 | | | | U36 (Archon Hunt Archons: U36.0.8) |
| Murmur | | | | | | 1.5 | | | 1.5 | | | 0.5 | | | U35.5 |
| Zariman | | | | | | | | | | | | | | 1.5 | U36 |
| Scaldra | 1.5 | | | | | | | | | 0.5 | | | 1.5 | | U38 |
| Techrot | | | | | 0.5 | | | | | 1.5 | 1.5 | | | | U38 |
| Anarchs | 1.5 | | | | | 1.5 | | | 0.5 | | | | | | U41 |
| Stalker, Tenno, Neutral | | | | | | | | | | | | | | | `W:Damage/Tenno`; Acolytes neutral since U36 (`W:Acolytes` L61) |
| Overguard layer (any faction) | | | | | | | | | | | | | | 1.5 | U31.5 (`W:Overguard`) |

Object health: no modifiers, no crits, no status (`W:Damage/Object` L1).

### 5.2 Resolution

φ_t is looked up with `damage_faction` (4.1), not `faction`: Corrupted Jackal (Orokin) takes Corpus modifiers;
Penta Ranger, Quanta Ranger, Rana Del, Wolf of Saturn Six and Saturn Six Fugitive take Grineer; Thrax and Void Angels
take Zariman; Death Scuttler and Scarlet Nexifera take Murmur; Warframe Specters, drones, Railjack fighters, Infested
Hives, Jordas Golem and Infested Mesa are neutral (`F:enemy` §2, `M:infobox` L607). Which faction a bane mod follows
is `01`'s rule; this document supplies both fields.

### 5.3 Combination with type resistances

```text
E-5.1  Effective type modifier on SH and HP (not OG)
  φ'_t = φ_t(damage_faction) · Π_i (1 − ρ^res_t,i)        clamped to ≥ 0
         ρ^res_t,i ∈ { type_res[t] (data), adapt_res[t] (15.4, layered pass), external type-resistance effects }
  φ'_t = 1 on a part with no_faction_mod (non-body hitboxes "lack ... resistances and weaknesses", W:Enemy_Body_Parts L37)
  OG:   φ^OG_t = 1.5 if t = VOI else 1                     (no faction, no type resistance)
```

Sources: "Sources of damage type resistance stack multiplicatively" and "can change these modifiers down to a
minimum of −100 % (0x)" (`W:Damage_Type_Modifier` L9, L13; U30.5 multiplicative change, `F:enemy` §2). Overguard is
not affected by "Damage type resistance effects such as Adaptation" (`W:Overguard` mechanics). φ applies per damage
type after quantization and before armor (`W:Damage/Calculation` §Unarmored Enemies, L251-262).

Worked check (`F:enemy` §2): Kuva Heavy Gunner hit for 100 HEA + 100 COR + 100 IMP takes 50 + 150 + 150 = 350 before
armor (TV-05-11).

---

## 6. Level scaling (NP-2)

### 6.1 Curve coefficients

Each stat multiplier blends two endpoint curves `f_i = 1 + C_i·q^E_i` with a smoothstep over [Lo, Hi]
(`W:Enemy_Level_Scaling` L14-103, edited 2026-07-29; formulas from U36, smoothstep from U27.2).

Health (`W:Enemy_Level_Scaling` health tabs; `M:infobox` L15-76):

| Health class | C1 | E1 | C2 | E2 | Factions | Note |
|---|---|---|---|---|---|---|
| Grineer | 0.015 | 2.12 | 10.7332 | 0.72 | Grineer, Kuva Grineer, Scaldra | page "Grineer, Scaldra"; module agrees |
| Corpus | 0.015 | 2.12 | 13.4165 | 0.55 | Corpus, Corpus Amalgam | |
| Infested | 0.0225 | 2.12 | 16.0998 | 0.72 | Infested, Infested Deimos | module 16.100; D-044 decision: page value, UNVERIFIED (OQ-502) |
| Orokin | 0.015 | 2.10 | 10.7332 | 0.685 | Orokin, Anarchs | page tab "Anarchs, Corrupted"; module; the page text also lists Anarchs with Default (D-044: module) |
| Techrot | 0.02 | 2.12 | 15.0998 | 0.70 | Techrot | |
| Default | 0.015 | 2.00 | 10.7332 | 0.50 | every other faction (Sentient, Murmur, Narmer, Zariman, Stalker, Tenno, Neutral) | Narmer, Zariman, Stalker not named on the page; module fallback (UNVERIFIED, OQ-503) |

Shield (`W:Enemy_Level_Scaling` shield tabs; `M:infobox` L79-120):

| Shield class | C1 | E1 | C2 | E2 | Factions | Note |
|---|---|---|---|---|---|---|
| Corpus | 0.02 | 1.76 | 2.0 | 0.76 | Corpus, Corpus Amalgam | |
| Corrupted | 0.02 | 1.75 | 2.0 | 0.75 | Orokin, Anarchs | page; the module keys this class `"Corrupted"` while units have Faction `"Orokin"`, so the wiki slider shows Default (1.6) for Orokin units (module bug, `F:enemy` §7.3); D-044: page. Anarchs are not affected: the module has its own `"Anarchs"` shield key with the same 2.0 (`M:infobox` L104-109) |
| Techrot | 0.02 | 1.76 | 3.5 | 0.76 | Techrot | module E1 1.75; D-044: page (OQ-503) |
| Default | 0.02 | 1.75 | 1.6 | 0.75 | every other faction, including Grineer and Sentient | page "Grineer and Sentient" |

Armor, every faction: C1 0.005, E1 1.75, C2 0.4, E2 0.75. Overguard, every enemy: C1 0.0015, E1 4, C2 260, E2 0.9.

Transition bounds: health, shield, armor Lo = 70, Hi = 80 on q = L − L0; overguard Lo = 45, Hi = 50 on q = L − 1
(section 8).

### 6.2 Scaling class

`scaling_class = Stats.FactionScaling` if present, else the class of `faction` per the tables (`M:infobox` L451).
Overrides in the data: Jordas Golem → Default; H-09 Apex and H-09 Efervon Tank (Techrot) → Corpus
(`M:enemies/infestation` L2037, `M:enemies/techrot` L384, L413). Scaling never uses `damage_faction`.

### 6.3 Exact binary32 algorithm

`f32(x)` rounds to IEEE 754 binary32, round-to-nearest-even. Every arithmetic operation below is done in binary32,
in the order written (the page's requirement: "coefficients, exponents, power results, and intermediate arithmetic
results must be evaluated in binary32 in the displayed order", `W:Enemy_Level_Scaling` L42, L103).

```text
E-5.2  Level multiplier of one stat (health, shield, armor: Lo = 70, Hi = 80; overguard: Lo = 45, Hi = 50)
  q      = L − L0                     (overguard: q = L − 1);  q is an integer ≥ 0, f32(q) is exact
  pw_i   = f32( pow( f64(f32(q)), f64(f32(E_i)) ) )          (pow on binary32 inputs, rounded once; Decision, OQ-504)
  f_i    = f32( 1 + f32( f32(C_i) · pw_i ) )                 i = 1, 2; pow(0, E) = 0, so q = 0 gives f_i = 1
  t_lvl  = clamp( f32( f32(f32(q) − f32(Lo)) / f32(Hi − Lo) ), 0, 1 )
  s_lvl  = f32( f32(t_lvl · t_lvl) · f32( 3 − f32(2 · t_lvl) ) )
  Λ      = f32( f1 + f32( f32(f2 − f1) · s_lvl ) )
         Both curves are evaluated at every level. MUST NOT be rewritten as f1·(1 − s_lvl) + f2·s_lvl and MUST NOT
         substitute f1 when s_lvl = 0 or f2 when s_lvl = 1 (W:Enemy_Level_Scaling L103).
E-5.3  Scaled value
  value(B, Λ) = f32( f32(B) · Λ )                            (W:Enemy_Level_Scaling L28-40)
  HP_max = value(B_H, Λ_H);   SH_max = value(B_S, Λ_S)        (B from E-5.4)
  A_raw  = value(B_A, Λ_A)                                    (armor continues in E-5.8)
  OG_max = value(B_OG, Λ_OG)                                  (section 8; Λ_OG is not the Fortifier factor M_OG)
```

The f32 blend matters at high level: at q = 992 on the Orokin health curve, f1 = 29,429.3203125 and
f2 = 1,212.5455322265625, and E-5.2 gives Λ = 1,212.544921875 instead of f2 (TV-05-01). For overguard at q = 9998 the
f32 result 1,048,576 is 1.3 % above the f64 value 1,034,893.33 because f1 ≈ 1.5·10^13 swamps binary32 (TV-05-07).
Whether the game really evaluates in this order is stated only by the wiki (UNVERIFIED, OQ-026; D-045).

### 6.4 Rounding and precision of pools

- Health, shield and overguard keep the binary32 value of E-5.3 exactly; they are not rounded to integers or to 0.01
  (the gadget's `Math.round(x·100)/100` is display only, `G:slider` L212, L234). The UI displays them with U-5.
- Armor is floored after E-5.3 (`G:slider` L223 `Math.floor(base_armor * armor_multi)`; UNVERIFIED that the game
  floors, OQ-026; the difference to rounding is at most 1 armor point).
- Pools are stored as f64 from here on (U-4); depletion arithmetic is f64.

### 6.5 Domain

L is the final enemy level including the Steel Path offset (the SP level bonus is part of the displayed level;
`F:enemy` §8.1; the inspiration's level box works the same way, `I:enemy` §1.2). The engine requires
`L0 ≤ L ≤ 9999` (cap 9999 for regular and Eximus enemies since U25.0.7; Void Fissures can exceed it,
`W:Enemy_Level_Scaling` L8, L522-523). A scenario with L < L0 is rejected with an error (the gadget ignores such
input, `G:slider` L110-112); the UI clamps to the range and defaults to `spawn_level`. At L = L0 (q = 0) every
multiplier is exactly 1 (TV-05-30).

---

## 7. Base-value modifiers: Steel Path, Eximus, squad size, mission modifiers

### 7.1 Order

The base values are modified before the level multiplier, in the order of the wiki gadget (`G:slider` L120-205),
computed in f64 and rounded once to binary32 when E-5.3 applies `f32(B)`:

```text
E-5.4  Base values (scenario: steel_path, eximus, n_party, mission modifier)
  n_party ∈ 1..4;  1..6 when mission = icebind
  B_H ← (eximus and eximus_health > 0) ? eximus_health : health
  B_S ← (eximus and eximus_shield > 0) ? eximus_shield : shield
  B_A ← (eximus and eximus_armor  > 0) ? eximus_armor  : armor          (Entropic Eximus: 50, W:Eximus L92; inactive, 4.1)
  if steel_path:           B_H ← B_H + sp_health_bonus;  B_S ← B_S + sp_shield_bonus
  if mission = empowered:  B_H ← B_H + archimedea_health_bonus
  if eximus:               (B_H, B_S) ← eximus_base(B_H, B_S, B_A, d_exi = L − L0)     (E-5.5)
  if steel_path:           B_H ← 2.5·B_H;  B_S ← 2.5·B_S                 (armor unchanged since U36)
  if mission = empowered or archon_hunt:  B_H ← (2 + 0.5·n_party)·B_H;  B_S ← (2 + 0.5·n_party)·B_S
  if mission = icebind:    B_H ← (3 + n_party)·B_H;  B_S ← (3 + n_party)·B_S;  B_A ← (3 + n_party)·B_A
  B_H ← P_party · B_H                                                  (E-5.6)
```

Sources: Eximus overrides `G:slider` L129-138; SP flat bonuses L140-148 (UNVERIFIED order, D-055, OQ-505); Archimedea
bonus L150-152; Eximus L155-183; SP ×2.5 L185-188 (U36: "Steel Path Shields are now multiplied by 2.5x, in place of
the previous 6.25x"; "Steel Path no longer increases Armor values", `W:The_Steel_Path` L411-412); Empowered L190-195;
Icebind L198-204 ("4.0 / 5.0 / 6.0 / 7.0 / 8.0 / 9.0x"). Party size: the gadget accepts 1-6 players (L384); The
Icebind admits six players ("up to four players, six for The Icebind", `W:Coaction_Drift` L36); Archon Hunt and
Archimedea squads are four players (the Archon Hunt bonus stacks "to a maximum of 300 %", `W:Archon_Hunt` L57). The
party multiplier position is a Decision (multiplicative, so the order only changes binary32 rounding by at most 1 ulp;
UNVERIFIED).

### 7.2 Steel Path

- Health and shield ×2.5 of the base; armor and overguard unchanged (U36; `F:enemy` §8.1; overguard: neither the
  page nor the gadget applies it, OQ-009).
- The level offset (+100; +50 in Archwing, Empyrean and Railjack; +20 in Duviri with no health or shield change;
  Eidolons 110/120/130) is part of the level the user enters (`W:The_Steel_Path` L10-12, L59-64). The engine adds
  nothing. A Duviri Steel Path enemy is entered with `steel_path = false` and the higher level (EC-05-15).
- The Simulacrum (U35.5) and Undercroft (U33) notes that SP adds +250 % armor predate U36 and are outdated
  (`W:The_Steel_Path` L505; `F:enemy` §8.1).
- Flat base-health bonuses (U40 patch notes on `W:Damage_Reduction` L425-510, module data): Juggernaut +1,800;
  Juggernaut Behemoth +3,000; Juggernaut Demolisher +2,000 (L438-440: in the note the +2,000 is a sub-bullet of the
  Juggernaut Demolisher only; the Demolisher Bonewidow and Demolisher Voidrig lines of the module are commented out as
  non-functional, `M:enemies/themurmur` L66, L91, and are not data); The Fragmented +150,000; Anarch Commandeered
  Primes +10,000 health and +10,000 shield; two Sentient bosses +440,000 and +170,000. They are added before ×2.5
  (D-055).
- Legacyte: the module carries `SteelPathHealthBonus = 7500` and `ArchimedeaHealthBonus = 7500` with the comment
  "Doesn't seem to have this in practice despite U40 Patchnotes" (`M:enemies/techrot` L43-44); the U40 note lists the
  bonus (`W:Damage_Reduction` L484-487). Decision: apply the module value (patch note and data agree; the comment is an
  unrecorded observation), UNVERIFIED, OQ-532.
- Techrot Babau: U40.0.3 "increased the Temporal Archimedea base health bonus from 1,000 to 9,000"
  (`W:Damage_Reduction` L321). The module has no `ArchimedeaHealthBonus` on `techrot:Techrot Babau`; it has a separate
  record `techrot:Techrot Babau (Temporal Archimedea Survival)` with `Health = 9000` and `EmpoweredDefault`
  (`M:enemies/techrot` L80-104), which is neither 10,000 + 9,000 nor a bonus. Data gap: both records are used as the
  module gives them, and the Archimedea record carries a data warning (OQ-532).

### 7.3 Eximus

```text
E-5.5  Eximus base increase (G:slider L155-183; W:Enemy_Level_Scaling L161-193 health, L240-256 shields; U32 raised Eximus health)
  d_exi    = L − L0                                (gadget `lvl_diff`; Decision per D-043, UNVERIFIED, OQ-010/OQ-501)
  k_exi(d) = 1                          d ≤ 15
           = 1 + 0.025·(d − 15)         15 < d ≤ 25
           = 1.25 + 0.125·(d − 25)      25 < d ≤ 35
           = 2.5 + (2/15)·(d − 35)      35 < d ≤ 50
           = 4.5 + 0.03·(d − 50)        50 < d ≤ 100
           = 6                          d > 100          (continuous at every boundary)
  c_exi    = 0.25 if (B_S > 0 or B_A > 0) else 0.375  (B_S after the SP shield bonus, B_A after the Eximus override)
  B_H'     = max( 1.1·B_H, c_exi·(B_H + 900)·k_exi(d_exi) )
  B_S'     = B_S · max(1.1, k_exi(d_exi))             (equals the page's piecewise max for every d)
```

- Eximus use their own faction's curves; the Codex health is not the Eximus base (`W:Enemy_Level_Scaling` L162).
- Armor: no Eximus multiplier, only the `eximus_armor` override (`F:enemy` §8.2).
- Which records can be Eximus: `eximus_capable` (4.1), from `W:Eximus/Compatibilities`. The Eximus toggle is disabled
  for records that are not capable.
- Overguard: section 8. Archwing, Warden and Prosecutor Eximus have no overguard (`W:Eximus` L179); Archwing units are
  the records listed in the Archwing section of `W:Eximus/Compatibilities` (`eximus_og = false`).
- Prosecutors have "10x base Health rather than Overguard" and Wardens "+1.2 base Health" (`W:Eximus` L164, L169).
  The module's `grineer:Prosecutor` record already holds the 10× base (Health 1,500 against Guardsman 150, L0 10) and
  has `EximusDefault`, so the wiki gadget opens it with Eximus checked and applies E-5.5 to the 1,500 base. Decision:
  follow the gadget (the record is an Eximus record whose scenario default is `eximus = true`; E-5.5 applies; no
  overguard); whether the game's 10× already includes the Eximus base increase is UNVERIFIED (OQ-533). Wardens have no
  separate record; the Eximus toggle on a Warden record applies E-5.5 without overguard, and "+1.2 base Health" is not
  modelled (UNVERIFIED, EC-05-19).
- The page writes the step variable as `x`, which elsewhere on the same page is the current level (`f_1(x) = 1 +
  0.015(x − Base Level)^2.12`); the inspiration also uses the absolute level. The gadget uses `L − L0`. TV-05-05 gives
  both readings (OQ-501).

### 7.4 Squad size

```text
E-5.6  P_party = party_scaling(n_party) if the enemy has party_scaling, else 1
  Demolisher family: P_1 = 1, P_2 = 1.5, P_3 = 2, P_4 = 3; health only    (W:Demolisher L7: +50/+100/+200 %)
```

Regular enemies do not scale with squad size; "Known exceptions are Demolishers, Acolytes and Archons"
(`W:Enemy_Level_Scaling` L8). The Archon exception is the Archon Hunt rule "Enemy Health and Shields increased by 100 %
and an additional 50 % for each squad member" (`W:Archon_Hunt` L57), which applies to every enemy of the mission:
it is the `archon_hunt` mission modifier of 7.5, not a per-enemy `party_scaling` (Decision). Acolytes scale "by squad
size" by an unknown amount (`W:Acolytes` L337): P_party = 1 with a "squad scaling not modelled" flag (OQ-518). The
overview's T-23 limits party size to "squad-scaled enemies only"; with the mission modifiers it also feeds every enemy
of an Archon Hunt, Archimedea or Icebind mission (rewording proposed in OQ-523).

### 7.5 Mission modifiers (optional scenario field, default none)

Not part of T-23 today; proposed in OQ-523; the inspiration has none of them (D-507). Values from `G:slider` and
mission pages:

| Modifier | Effect | Source |
|---|---|---|
| `empowered` (Deep/Temporal Archimedea) | B_H += archimedea_health_bonus; B_H, B_S ×(2 + 0.5·n_party), n_party ≤ 4 | `G:slider` L150-152, L190-195 |
| `archon_hunt` | B_H, B_S ×(2 + 0.5·n_party), n_party ≤ 4: "increased by 100 % and an additional 50 % for each squad member, stacking to a maximum of 300 %" | `W:Archon_Hunt` L57 |
| `icebind` | B_H, B_S, B_A ×(3 + n_party), n_party ∈ 1..6; armor still clamps to 2700 | `G:slider` L198-204, L384; `W:Coaction_Drift` L36 |
| `bolstered_belligerents` | OG_max = 0.5·HP_max for every enemy (Techrot Babau 75 % less, U38.5) | `W:Overguard` enemy list; `F:enemy` §8.3 |

---

## 8. Overguard pool

```text
E-5.7 Overguard base and pool
  B_OG = eximus ? (eximus_og ? (eximus_overguard > 0 ? eximus_overguard : 12) : 0)
                : (overguard_context = none or context_overguard ? overguard : 0)
  q_OG = L − 1                              (D-042 decision; gadget uses L − L0; UNVERIFIED, OQ-009)
  OG_max = value(B_OG, Λ_OG(q_OG))           E-5.2 with C1 0.0015, E1 4, C2 260, E2 0.9, Lo 45, Hi 50
  no Steel Path factor, no squad factor; mission bolstered_belligerents replaces OG_max by 0.5·HP_max
```

Sources: base 12 for every Eximus (`W:Enemy_Level_Scaling` overguard section; `G:slider` L135); overrides in data
(Scaldra Dedicant `Overguard = 22`, `EximusOverguard = 24`, `M:enemies/scaldra` L146-147); `q = Current Level − 1`
(`W:Enemy_Level_Scaling` L25). The Overguard page supports the L − 1 reading: overguard "will scale by level but will
be the same amount across all Eximus enemy types" (`W:Overguard` L41), which only holds if base level does
not enter. Non-Eximus enemies with `Stats.Overguard` (Thrax Centurion 15, Thrax Legatus 10, Kuva Trokarian 2, Nox 5,
Dax Malleus 8 in its context ...) scale with the same formula from their own base, as the gadget does (`G:slider`
L124, L233-234); UNVERIFIED (`DS:wiki` open question; OQ-519). Example: Eximus Heavy Gunner at L 58:
118,709.6015625 with q = 57 versus 105,505.953125 with q = 50 (TV-05-05).

Overguard sources outside the scaled pool (Ancient Protector grants 9× its health, Severed Warden, Nox helmet and
Dedicant ampule removal, Void Armageddon) are out of scope (EC-05-24).

---

## 9. Armor

### 9.1 Initial armor

```text
E-5.8  A0 = clamp( floor(A_raw), 200, 2700 )   if B_A > 0
       A0 = 0                                   if B_A = 0
       A_raw from E-5.3.
```

Sources: U36 "Armor has a maximum cap of 2700 Armor (90 % Damage Reduction). Armor has a minimum cap of 200 Armor"
(`W:Enemy_Level_Scaling` L263-265, L466-467); floor and the `armor > 0` guard (`G:slider` L223-227). The floor of 200
applies to the initial value only; strips may take armor below 200 (`W:Enemy_Level_Scaling` L265). The cap is reached
only from a base-armor-dependent level offset. First q with `floor(B_A·Λ_A(q)) ≥ 2700` (`python3`, E-5.2; before any
Icebind factor):

| Base armor | 500 | 300 | 200 | 150 | 100 | 50 | 10 | 5 |
|---|---|---|---|---|---|---|---|---|
| first q at 2700 | 49 | 68 | 99 | 149 | 262 | 676 | 5892 | never (q ≤ 9998) |

Low-armor units are common (Lancer 100, Scaldra Flayer 10): a Lancer at L 100 has A0 = 1355 (overview §3.8), and a
Scaldra Flayer sits at the 200 floor up to q = 173 (TV-05-21).

### 9.2 Damage reduction

```text
E-5.9  DR(A) = 0.9·sqrt(A / 2700)     0 ≤ A ≤ 2700          (= sqrt(3A)/100)
       DR(A) = A / (A + 300)          A > 2700               ("exceptional conditions" only)
       armor_mult m_arm = 1 − DR(A_eff)
```

Source: `W:Armor` L33-39 (U36 "Altered the formula for Armor Reduction"); the branches meet at 2700 (0.9). Level
scaling cannot produce A > 2700 because of the clamp; the second branch exists for data or modes that bypass the clamp
(EC-05-28). f64 arithmetic.

### 9.3 Effective armor

E-15 of the overview, with every term defined:

```text
E-5.10  Effective armor at the time an instance lands (state σ before the instance, CD-04; DoT ticks: at tick time, CD-05)
  if armor_strip_immune = all:  A_eff = A0                                                (9.6)
  f_flat = (B_A > 0) ? max(0, B_A − s_flat) / B_A : 0                                     (9.4; 1 if flat-immune)
  Σ_add  = Σ_j a_j·Str_j                ability strips of TOTAL armor (U32), each listed % × caster Strength (04 §5.7);
                                        0 when ability_debuff_immune = all (13.1)
         + Σ_CP 0.18·aura_str           every Corrosive Projection in the squad (0.03 per rank, U27.2); aura_str = 1.15
                                        for the wearer's own Coaction Drift (Aura Strength only, group aura)
         + T-07                          external ability-type strip (00 section 6); 0 when ability_debuff_immune = all
         + companion % strips            (Sharpened Claws 120 %, additive with CP per its page; OQ-508)
  A_tot  = max( 0, A0 · f_flat · max(0, 1 − Σ_add) − P_pts )                              (P_pts: Polarize 400·Str; order UNVERIFIED, OQ-507)
  A_eff  = A_tot · (1 − c_heat(τ)) · (1 − c_corr(n_COR))
         c_heat(τ) = heat_strip(k_h(τ)) = 1 − 0.5^(k_h(τ)/4)    (03 E-3.8; ramp up after ignition, step down after the
                     episode ends, D-026). k_h is target state kept by 03 independently of live Heat stacks; c_heat = 0
                     only if the target is Heat-status-immune or k_h(τ) = 0
         c_corr = 0 if Corrosive-immune or n_COR = 0; else corr_strip(n) of 03, or the Hydroid variant E-5.12
```

Sources: `W:Armor` L334-336 ("Warframe abilities are based on the total"; Heat and Corrosive on the current value);
U32 note (`W:Armor` L598-620: "ALL Armor Stripping Warframe abilities now apply to the TOTAL armor value");
`W:Corrosive_Projection` L31-46 (additive with itself and with Terrify and Fusion Strike; multiplicative with Heat and
Corrosive; worked example `(1 − 50 %)·[1 − (20 % + 6 %·10)]·(1 − 18 %·4)`), L58 (strip capped at 100 % before U27.2,
hence the `max(0, ·)`); `W:Coaction_Drift` L30-35; `F:buffs` §3; `F:enemy` §3.3; Heat steps `W:Damage/Heat_Damage`
§Armor Stripping (via `03` §10.4). Corrosive Projection and abilities act on the clamped value (`F:enemy` §3.2,
derived: "clamps an enemy's initial armor").

The ability strip list with percentages is `04`'s catalogue (§5.7: Terrify 60 %, Fusion Strike 50 %, Sonic Boom 70 %,
Avalanche 60 %, Pulverize 50 %, Regurgitate 75 %, Pillage 25 % shields first, Fracturing Crush 75 %, Psychic Bolts
80 %, Reckoning 60 %, Tharros Strike 50 %, Abating Link 60 %, Seeking Shuriken 70 %, Gaze 50 %, Rakhali's Cavalry
35 %/hit, Temporal Erosion 10 %/hit, Bastille and Ophanim Eyes 10 %/s, Smite 100 %, Unairu Caustic Strike 100 %).

- **Time- and hit-based ability strips** (Bastille, Ophanim Eyes per second; Rakhali's Cavalry, Temporal Erosion per
  hit): their reached value is a user input in Σ_add (default: the catalogue maximum, as `04`'s class (u) debuffs);
  their growth over time is not modelled (Decision, UNVERIFIED).
- **Shields first** (Pillage, Polarize): "If the affected enemy has both shield and armor, shield is removed first"
  (`W:Armor` L408-410 Pillage, L438-440 Polarize; "Shields must be completely drained ... before a new Polarize cast
  can deplete armor", `W:Polarize` via `04` §5.13). Decision: on a target with SH_max > 0 these casts act on the shield
  (Pillage as an ability shield strip in E-5.16, Polarize as 400·Str shield points removed at fight start) and add
  nothing to Σ_add or P_pts; on a target without shields they act on armor. The `armor` and `health` views assume
  shields and overguard are gone (section 18), so there they act on armor. UNVERIFIED (OQ-507).

### 9.4 Flat base-armor strips

Shattering Impact removes `rank + 1` base armor per melee hit that deals Impact (6 at rank 5), whether or not Impact
procs and only if the weapon's quantized Impact is > 0; Amalgam Argonak Metal Auger removes 6 per single-dagger
instance, including the dagger's status ticks; they stack additively with each other (`W:Shattering_Impact` notes;
`W:Amalgam_Argonak_Metal_Auger` notes; `W:Armor` L505-509). They act on base armor, "applied before
percentage-based base armor strips, such as Corrosive Projection" (`W:Shattering_Impact`).

```text
E-5.11 s_flat(τ) = Σ over earlier qualifying hits of their per-hit points (permanent; never decays)
       f_flat    = max(0, B_A − s_flat) / B_A
```

The post-U36 clamp makes the result ambiguous: recomputing `clamp(floor((B_A − s_flat)·Λ_A), 200, 2700)` would leave a
clamped enemy at 2700 for most of the strip and would never take an enemy below 200, which contradicts "armor can
still be decreased below 200 through all normal means" (`W:Enemy_Level_Scaling` L265). Decision: the proportional
form above, which keeps "the same number of hits to completely drain the enemy's armor, regardless of its level"
(`W:Shattering_Impact`). UNVERIFIED (OQ-506). Example: base 500, 30 rank-5 hits → f_flat = 0.64, A = 1728 on a
clamped enemy instead of 2700 (TV-05-10).

Qualifying hits are counted by the schedule (`06`, `07`); the per-hit values come from `04` §5.13. A qualifying
Shattering Impact hit is every damage instance of a melee weapon whose quantized Impact is > 0: each stance hit, each
gunblade or Arum Spinosa projectile that carries Impact (one count per projectile instance), slams (the pure-Impact
slams of Galatine and Destreza included), and each Melee Duplicate copy (it is a full hit, `04` §6.4; "doubles the armor
strip", `W:Shattering_Impact` tips); not the charged throw of a thrown melee weapon ("doesn't reduce enemy armor when
used with the charge attacks of thrown weapons", U18.1.4), and not radial instances without Impact. Shattering Impact
works on status-immune enemies and does not affect Archons or Commandeered Primes (`W:Shattering_Impact` notes;
`armor_strip_immune`). Metal Auger counts every instance of the dagger including its status ticks (`04` §5.13).

### 9.5 Corrosive variants

```text
E-5.12  Hydroid passive (target damaged by Hydroid; U34 rework): first Corrosive stack strips 50 % instead of 26 %
  c_corr_H(n) = min(1, 0.50 + 0.06·(n − 1))   n ≥ 1;   0 for n = 0
```

Source: `W:Armor` L417, L552-556 ("Corrosion Status can reach 100 % Armor Reduction with full stacks ... the Corrosive
Status Effects can be applied from any source"). `armor_hydroid` is a scenario flag (proposed toggle, OQ-523). The
Emerald Archon Shard raises the Corrosive stack cap (+2, Tauforged +3; 14 stacks give `min(1, 0.20 + 0.84) = 1`); the
cap is `03`'s rule, the clamp to 1 is in `corr_strip` (00 section 2.4).

### 9.6 Armor-strip immunity

`armor_strip_immune = all`: A_eff = A0 for every source (Archons in Archon Hunts: "Archons are immune to armor
reduction", `W:Archon_Hunt` L119; `W:Archon_Boreal` L43 "armor which cannot be removed"; Profit-Taker Orb: "immune to
... any armor-reducing effect (e.g. Corrosive Projection, Shattering Impact and Seeking Shuriken)",
`W:Profit-Taker_Orb` L112). Corrosive procs still apply as statuses (they count for CO and caps) but strip nothing.
`flat_only`: f_flat = 1 (Commandeered Primes, `W:Shattering_Impact`). Sharpened Claws "cannot armor strip Kuva
Liches" (`W:Sharpened_Claws`): `04` gates that source. The inspiration has no immunity (D-506).

### 9.7 Per-type minimum of 1

```text
E-5.13  On the health layer, for each type t with arm_t < 1 (A_eff > 0 and t armor-reduced) and D_t > 0:
         D_t ← max(1, D_t)
         applied after φ', Vir, V and m_arm, before innate/class/status DR and attenuation (E-5.24)
```

Source: "When damage is reduced from armor, each damage type has a minimum damage of 1. E.g. a Braton will always do a
minimum of 3 damage against an armored target ... Non-Armor sources of damage reduction do not have such a behavior"
(`W:Armor` L41). Applies to every instance class that armor reduces (direct, radial, extra hit, typed DoT ticks), never
to TRU, FIN, CIN, the shield or overguard layer, or the `health` view. A type quantized to zero has D_t = 0 and gets no
floor. With A_eff = 0 there is no armor reduction and no floor (Decision; the inspiration floors whenever base armor
> 0, D-503). Position relative to V and Vir: Decision per E-12 (all multiplicative HP terms first), UNVERIFIED (OQ-517).

---

## 10. Shields

### 10.1 Damage to shields

```text
E-5.14  D^SH_t = W_t · φ'_t · Mag(n_MAG) · V_SH,t                 t ∉ B_SH
        then × R_SH (E-5.20), then attenuation if SH ∈ attenuation.layers (E-5.21)
```

Faction modifiers apply to shields since U36 (no separate shield class; `F:enemy` §4.1; `W:Damage` intro). Armor
never reduces shield damage and Viral never applies (`F:enemy` §1). Magnetic: `mag_mult` of `03` (×2.0 to ×4.25),
also stops shield regeneration. Slash Bleed (Cinematic) damages shields since U27.2 (`W:Shield` L553).

### 10.2 Bypass set and exceptions

```text
E-5.15  B_SH = {TOX, TRU}   if toxin_blocked = none
        B_SH = {TRU}        if toxin_blocked = absorb   (Hounds; True bypass UNVERIFIED: W:Hound L35 names only Toxin)
        B_SH = ∅            if toxin_blocked = nullify  (Treasurer: nothing bypasses its shield)
        Types in B_SH skip the shield and go to health while SH > 0; they do not skip overguard.
        toxin_blocked = nullify (Treasurer): Toxin DoT ticks deal 0 while SH > 0; direct Toxin and True damage are
        absorbed by the shield like any other type.
```

Sources: "Shields absorb every Damage but Toxin" (`W:Shield` L6, L29); Toxin "ignores enemy and player Shields, but not
Overguard" (`W:Damage/Toxin_Damage` L13); True `Bypass = {"Armor", "Shield"}` in `M:dt` (`F:enemy` §1); Treasurer
"impenetrable by methods that would normally bypass shields (such as Toxin damage) ... Toxin procs have no effect
(dealing 0 damage) while their shields are up" (`W:Treasurer` L30): True damage is such a method in this spec (its
`M:dt` `Bypass` lists "Shield"), so it is blocked too; Hounds "cannot be bypassed by Toxin damage or procs"
(`W:Hound` L35), which says nothing about True damage; `W:Damage/Calculation` L289 names the same two. The list of
`toxin_blocked` enemies is incomplete (OQ-029); the Hound infobox carries `protoshield = 1`, which the data build
SHOULD scan for on enemy pages.

### 10.3 External shield strips (optional input, default none)

Ability shield strips take a percentage of the current shield (diminishing returns), the opposite of armor strips;
Shield Disruption takes 18 % per aura, 72 % for four, hard cap 80 % with Coaction Drift (`W:Shield` L357-415;
`F:enemy` §4.4).

```text
E-5.16  SH_max' = SH_max · (1 − min(0.80, Σ 0.18·aura_str)) · Π_j (1 − s_j·Str_j)      (applied at fight start)
        the aura term is 0 when shield_strip_immune; each ability term s_j is 0 when shield_strip_immune or
        ability_debuff_immune = all (13.1); Polarize shield points (9.3): SH_max' ← max(0, SH_max' − 400·Str)
```

The ordering of the aura against ability casts is not documented; Decision, UNVERIFIED (OQ-523 proposes the toggle).
Shield Disruption "has no effect" on Eidolon shields (`W:Eidolon_Teralyst` L81) and the Profit-Taker Orb is "immune to
the effects of Shield Disruption" (`W:Profit-Taker_Orb` L112): `shield_strip_immune`.

### 10.4 Shield gate

Enemy shield gate: "Enemies have a shield gate that lasts 0.1 seconds, during which only 5 % of the damage dealt will
damage their health. However, targeting weakspots will completely bypass their shield gate. Some area of effect
attacks (like slam attacks) do not benefit from the damage bypass and instead will have the damage instance completely
blocked. Additional damage instance associated with the attack such as status effects or Xata's Whisper can still
damage the enemy" (`W:Shield` L341; enemy gating since U27.2). Overguard has no gate on enemies (`W:Overguard`). The
gate is part of ST-9 (E-5.25).

### 10.5 Recharge (TTK only)

U36: recharge delay depends on accumulated shield damage between a minimum and a maximum; a break counts as 200 % (the
maximum); the rate grows with level and accelerates while recharging (`W:Shield` L437-450); "Enemy shields take a
maximum of 3 seconds to begin regenerating" (`W:Shield` L108). No rate formula is published.

Decision: shields do not regenerate in either evaluator. MC TTK sets the flag `shield_regen_possible` when, with
`shield_regen = normal`, SH < SH_max and no live Magnetic stack, the time between two consecutive instances that reach
the target exceeds 3 s. Amalgam shields never regenerate once fully removed, Treasurer shields never (`F:enemy` §4.5).
UNVERIFIED (OQ-525).

### 10.6 Shields that weapons cannot damage

Eidolon shields are "invulnerable to all sources of damage, it can only be harmed with Tenno Operator sourced Void
damage" (`W:Eidolon_Teralyst` L121); Xaku's Void abilities "have no effect on the Eidolon Teralyst shields" (L93). The
Profit-Taker Orb's shield "is only vulnerable to that damage [the type displayed on its forehead], while every other
damage type will be rendered useless", and the type cycles (`W:Profit-Taker_Orb` L59-66). Operator damage and the
phase mechanics are out of scope (00 section 1.3); the engine handles the weapon side only:

```text
E-5.26  shield_weapon_immune = all:             D^SH_t = 0 for every weapon instance and type
         shield_weapon_immune = displayed_type: D^SH_t = 0 unless t = pt_shield_type (scenario input, default none)
         shield view (18): "immune" (or only the allowed type's damage); B_SH types still go to health while SH > 0
         layered pass: scenario option shield_removed_externally (default on for both values) starts the pass with
         SH = 0 ("shields removed by operator / phase skipped") and flags the result; off: the shield is routed with
         the rule above (an Eidolon then never loses its shield to weapons)
```

Decision, UNVERIFIED for the Profit-Taker type set (EC-05-79). The Profit-Taker's body "can only be damaged using a
Gravimag-enhanced Archgun" (L80): out of scope, flagged.

---

## 11. Overguard layer

```text
E-5.17 D^OG_t = W_t · φ^OG_t · Mag(n_MAG) · V_OG,t · M_OG · R_OG                every type, including TOX, TRU, CIN
        φ^OG_t = 1.5 for VOI, else 1;   R_OG = (1 − innate_dr[OG]) · Π_{c ∈ classes} (1 − dr_class_og[c])
        no armor, no faction modifier, no type resistance or adaptation, no Viral, no status DR, no attenuation
```

Sources: `W:Overguard` mechanics: neutral except Void +50 %; Magnetic amplifies overguard like shields; "Damage
Vulnerability applies normally"; "not affected by Damage Reduction from armor or abilities such as Splinter Storm or
Damage type resistance effects"; Secondary Fortifier "up to 8x" (M_OG, value in `04`; ×3 at rank 0, `F:addmul` §5);
Viral on overguard was a bug fixed in U31.6 (`F:status` §4.13). Toxin does not bypass it (`W:Damage/Toxin_Damage` L13).
DoT ticks damage overguard (`F:status` §5: "DoT ticks damage OG"; whether the game routes them to OG or health is
UNVERIFIED, OQ-011). Guardian Eximus' own AoE reduction "does apply to the Overguard of the Guardian Eximus"
(`W:Eximus` L118).

Overguard state rules: Cold cap 4 and VIP Puncture cap 3 while `og_present` (rule `03` E-3.2, data 4.4);
crowd-control immunity is irrelevant to damage; statuses still apply and DoTs tick (`W:Overguard` enemy section).
Effects that remove overguard outright (Smite, Null Audit 50 %) and companion overguard bonuses (Assassin Posture,
Unleashed +300 %) are out of scope (companion and ability damage, 00 section 1.3), EC-05-25.

---

## 12. Health layer and the Viral stage

```text
E-5.18 D^HP_t = W_t · φ'_t · Vir(n_VIR) · V_HP,t · arm_t
        arm_t  = m_arm = 1 − DR(A_eff)   if A_eff > 0 and t ∉ {TRU, FIN, CIN} and the part has no_armor = false
        arm_t  = 1                       otherwise
        then the minimum of 1 (E-5.13), then × R_HP (E-5.20), then attenuation (E-5.21)
```

Sources: `W:Damage/Calculation` §Armored Enemies (L278-289); `W:Damage/Viral_Damage` (Viral multiplies health damage
including armored health, not shields, not overguard; `F:enemy` §6); bypass of armor by True, Finisher and Cinematic
(`M:dt` `Bypass`; `W:Damage/True_Damage`; `W:Damage/Cinematic_Damage` L9-13; ground finishers U36, `F:melee` §8).
Vir(n) is `viral_mult` of `03` with n_VIR = own stacks plus T-06 external stacks, capped by the Viral cap including
enemy caps (00 T-06); Vir = 1 for a Viral-immune target. The hit that applies a Viral proc does not benefit from it,
and sibling pellets do not share it (CD-04; `F:status` §4.13).

---

## 13. Vulnerability stage

```text
E-5.19  V_ℓ,t = Π_{e ∈ Mul(ℓ,t,c)} (1 + v_e) · (1 + Σ_{e ∈ Add(ℓ,t,c)} v_e)
        Mul, Add: the active multiplicative and additive-group vulnerability entries whose layer mask contains ℓ,
        whose type mask contains t (all types unless stated), and whose class condition matches the instance class c
        (melee only: Paralysis; Void only: The Vast Untime; Heat or Puncture only: Magus Accelerant / Destruct)
```

Sources: `W:Damage_Vulnerability` mechanics ("stack multiplicatively with damage bonuses and buffs, including other
sources of Damage Vulnerability"; "some specific Damage Vulnerability effects stack additively"; "can be conditional
such as applying to Melee attacks only or specific damage types"; "does not 'double dip' in Damage over Time");
`W:Damage_Vulnerability` §Notes L96-104 and `F:buffs` §2 (additive group: Atomi-Barrage, Containment Wall, Petrify,
Jade's Judgement, Prey of Dynar, Magus Accelerant, Magus Destruct, Theorem Contagion; `F:buffs` line 445).

Rules:

- The catalogue (values, Strength scaling, group, masks, conditions) is `04`'s. The stage applies V exactly once per
  damage instance on the layer it lands on: direct, radial, extra hit, and every DoT tick at tick time (CD-05). It never
  enters the weapon side or the DoT seed.
- Layer masks: since U37 the vulnerability of The Vast Untime, Magnetize, Beguiling Lantern augment, Firing Line, Mass
  Vitrify, Splinter Storm, Blazing Chakram and Molecular Prime also applies to shields and overguard (`F:enemy` §5).
  An entry without an explicit mask applies to health only (Decision; the Overguard page's "applies normally" conflicts,
  UNVERIFIED, OQ-523). The custom stat `dmgVulnerability` (T-26) is multiplicative on every layer and type (it is in
  the inspiration's per-hit product for every layer, `I:queue` §3.6).
- Mecha Empowered and the Vauban passive are not vulnerabilities here: they behave like faction damage and live in
  `M_dot` / `G` (`04`, D-041).

### 13.1 Immunity to Warframe-ability debuffs

Some enemies ignore debuffs cast by Warframe abilities. The field `ability_debuff_immune` (4.1) gates every `04`
catalogue entry whose source is a Warframe ability (including Helminth-subsumed abilities and augments): ability
vulnerabilities in E-5.19, ability armor strips and T-07 in Σ_add (E-5.10), ability shield strips in E-5.16. Entries
from mods, arcanes, auras (Corrosive Projection, Shield Disruption), focus, companions and statuses are not gated by
it. `04` marks each entry's source kind (requested in OQ-523).

| Value | Effect | Enemies (4.5) |
|---|---|---|
| `none` | no gate | default |
| `all` | gated entries contribute nothing (V factor 1, strip 0) | Eidolons ("immune to all debuffing effects of Warframe powers", `W:Eidolon_Teralyst` L91; Corrosive Projection still works, L81); Archons in Archon Hunts ("immune to most Warframe abilities", `W:Archon_Hunt` L117, Decision `all`, UNVERIFIED); Entropic Eximus while its orb is active (`W:Eximus` L95, L104; inactive until OQ-523) |
| `dispelled` | entries apply, the result carries the flag `ability_dispel_5s` (a pulse every 5 s dispels abilities on the target; re-casting is the user's scenario) | Demolisher family except Demolisher Bonewidow and Demolisher Voidrig (`W:Demolisher` L5); Scaldra Screamer (`W:Scaldra_Screamer` L8) |

Hounds' "effects being reduced to 1/4th of their duration" (`W:Hound` L35) is a flag only (data gap: no record).

---

## 14. Innate, status and class-conditional damage reduction

```text
E-5.20  R_ℓ = (1 − innate_dr[ℓ]) · Π_{c ∈ classes(instance)} (1 − dr_class[c]) · (status instance ? (1 − status_dr) : 1)
                                                                                     ℓ ∈ {SH, HP}
        R_OG = (1 − innate_dr[OG]) · Π_{c ∈ classes(instance)} (1 − dr_class_og[c])  (no status DR on OG)
        classes(instance), from the instance's weapon class and radial flag:
          radial_or_melee  radial instance, or any instance of a melee weapon (matched once)
          radial           radial instance
          non_melee        instance of a primary, secondary or arch-gun weapon, or a ranged instance of a glaive or
                           gunblade (thrown glaive, gunblade shot)
        inheritance: a DoT tick (incl. the forced DoT of an extra hit), a Blast stack hit and an extra hit (Xata's
          Whisper, Toxic Lash, ...) take the weapon class and radial flag of the instance that induced them; a Magnetic
          break proc and any other special instance without an inducing weapon instance has no class (no dr_class)
        status instance: DoT tick (E-13), Blast stack hit, Magnetic break proc, forced DoT of an extra hit
        part_dr: an instance that hits a part with part_dr (4.3) multiplies R_ℓ by (1 − part_dr); radial instances use
          the default part, DoT ticks inherit the inducing part (Part_dot)
```

Sources: "Any source of damage reduction, aside from armor, will decrease damage done to health and shields. Sources
of damage reduction (including armor) stack multiplicatively with each other" (`W:Damage_Reduction` L101); "Damage
Reduction has no affect on ... Overguard" (L53); per-enemy values in 4.5. R applies after the minimum of 1 (CD-09,
E-12); there is no minimum after it (`W:Armor` L41). Juggernaut "additional 50 % damage reduction against Area of
Affect and melee attacks" is one factor even when an instance is both (Decision). The `non_melee` class follows the
U44.0.1 note "Damage from non-melee sources is reduced by 90%" (`W:Eximus` L262; it includes arch-guns) and the page's
list "Primary and Secondary weapons ... ranged melee attacks from Glaives and Gunblades" (L90-93). The inheritance rule
for DoT ticks, Blast stacks and extra hits is not documented: Decision, UNVERIFIED (OQ-535). Examples: a Slash tick
from a melee hit on a Juggernaut is `radial_or_melee` (R_HP = 0.1·0.5 = 0.05 with T-28 on); a Toxin tick from a rifle
on an Entropic Eximus is `non_melee` (×0.1) (TV-05-26).

## 15. Unique damage reduction and T-28

### 15.1 Attenuation (max damage per instance)

```text
E-5.21  For an enemy with attenuation {p_att, layers}, on a layer ℓ ∈ layers, per damage instance:
  X_att  = Σ_t D^ℓ_t                         (after every modifier, including armor, Viral, V and R_ℓ; CD-09)
  C_att  = p_att · HP_max                    (HP_max = scaled max health of E-5.3 after SP, Eximus, squad, mission;
                                              shields, armor and overguard are not counted)
  Att(X_att) = X_att · C_att / (X_att · MS_att + C_att);   Att(0) = 0
  MS_att = ms (E-10, modded, fractional allowed) for non-beam direct and radial instances;
           1 for merged beam ticks (CD-10), DoT ticks, extra hits and special instances (Decision, UNVERIFIED)
  each type is scaled by Att(X_att)/X_att (no scaling when X_att = 0)
```

Sources: `W:Hemocyte` L23-35 (formula and worked example: 12,000 incoming, multishot 2, max health 2,472,207,
p = 5 % → 10,049); U40 patch notes on `W:Damage_Reduction` L391-412 ("Damage Attenuation values are now proportional
to Max Health", "Max Health for this calculation only uses the Health value", MDPS per player); U34 "Modifiers now
apply before Damage Attenuation" (CD-09). The inspiration's disabled `getArchonDR` has the same form with a shotgun
factor (`JS L13265-13268`: `x/(x + 1)·460000/ms` with `x = dmg·ms/460000`).

`p_att` is known only for Hemocyte (0.05), Lephantis (0.05) and Eidolons (2.00) (4.5). Every other attenuated enemy
gets no attenuation and the result carries the flag `attenuation_unknown` (OQ-510, OQ-022). The pre-U40 absolute
values on `W:Archon` (MDPT 460,000, MDPS 935,000, λ = 2/3, shotgun compensation 0.7) are not used: U40 replaced them by
health-proportional values that are not published.

### 15.2 Max damage per second

Not modelled: "MaxDamagePerSecondHealthPercent not formulated yet" (`W:Hemocyte` L76, L80). Documented fractions
(Hemocyte 0.20, Eidolons 1.00) are stored as data for a later model. Flag as in 15.1.

### 15.3 DPS-scaled damage reduction (Orphix)

```text
E-5.22  M_dps(X) = 1                     X ≤ 1000
                 = 0.7  + 300/X          1000 < X ≤ 2500
                 = 0.5  + 800/X          2500 < X ≤ 5000
                 = 0.2  + 2300/X         5000 < X ≤ 10000
                 = 0.02 + 4100/X         X > 10000            (continuous at every boundary)
        X_dps = (Σ_t w_t · φ_t) · rate · ms_dps            per attack: non-crit (k = 0), quantized, with the
                                                            faction modifier; W already holds the body part (E-12)
          w_t    = W_t(0) per pellet (non-beam); W_t(0)/b per beam for a merged beam tick (W of a merged tick already
                   holds the beam count b, CD-10, so dividing avoids counting multishot twice)
          rate   = modded fire_rate (06) for ranged attacks; hits per second of the 07 schedule (mean over the combo)
                   for melee attacks
          ms_dps = ms (E-10) for ranged attacks; 1 for melee
        every instance of that attack on the Orphix is multiplied by M_dps(X_dps); crit applies on top (in W_t(k))
```

Source: `W:Orphix` L30-44 ("DPS excluding Critical Hits ... Critical Hits are applied after DR is calculated ... the
average DPS is calculated using Health Modifiers and is quantized"); the inspiration's disabled `orphixDR` is identical
(`JS L13118-13124`). Choices: `fire_rate` is the modded fire rate (not the reload-averaged rate), and the multiplier also
applies to DoT ticks and extra hits of the attack (Decision, UNVERIFIED, OQ-512). Condrix has "DR like Orphix" with no
published formula (`F:enemy` §9.2): not modelled, flagged.

### 15.4 Adaptation (Sentients, Shadow Stalker)

Only in the `layered` pass (health depletes); the layer views use a fresh, unadapted target.

```text
E-5.23  State: adapted list a[1..4] (types), dmg_by_type[t] (health damage taken after all modifiers), gates g = 0.25,
        0.45, 0.65, 0.80 of HP_max lost, each crossed once.
  After an instance is applied: for each gate newly crossed (HP lost ≥ g·HP_max), append to a the type with the largest
  dmg_by_type among types not yet in a and not in {TRU, FIN, CIN}; ties: lowest DamageType enum index (Decision).
  Resistance on type a[j]:
    Sentient:        adapt_res = 1 − (1 − ρ_j)·(1 − 0.40),  ρ = (0.90, 0.80, 0.75, 0.70)
    Shadow Stalker:  adapt_res = ρ_j = 0.95
  Reset: a VOI status proc on the target from any in-scope source (a weapon's own Void procs, the Void proc of a
  Xata's Whisper extra hit, T-15), plus out-of-scope sources the user may mark (Void Beam, Paracesis rank 40 hit,
  Shedu pulse, Radial Howl; The Vast Untime for Shadow Stalker): ρ parts removed; the Sentient 40 % parts stay;
  later gates can adapt again.
  Per-instance cap (Sentients): health removed by one instance ≤ 0.25·HP_max while HP ≥ 0.75·HP_max; 0.20 while
  0.35 ≤ HP/HP_max < 0.75; 0.15 while 0.20 ≤ HP/HP_max < 0.35; no cap below 0.20 (HP before the instance).
```

Sources: `W:Sentient` L293-323 (gates, 90/80/75/70 %, "a 40 % resistance to the adapted damage type at each adaptation,
which stacks multiplicatively ... cannot be removed", per-hit maxima, no adaptation to True, Finisher, Cinematic
including Bleed, the hit that crosses a gate takes full damage); `W:Shadow_Stalker` L34-44 (95 %, same gates, Vast
Untime also resets); the Xata's Whisper hit resets "upon triggering a Status Effect" (`W:Xata's_Whisper` L18); the
in-game Void description "Highly effective against enemies with Overguard Health and Sentients as it resets their
damage adaptation" (`W:Damage/Void_Damage` L10-11; quoted on `W:Overguard` L212) makes any Void status proc the
trigger. `W:Damage/Void_Damage` L17 excepts "Xaku's abilities", which contradicts `W:Sentient` L311 and
`W:Xata's_Whisper` L18; Decision: the Xata's Whisper Void proc resets (two specific pages against one parenthesis),
UNVERIFIED, D-513, OQ-536.
Brachiolysts, Tyro Conculysts and fledgling Sentients do not adapt (`W:Sentient` L77, L155); Eidolons and Orphix are
excluded (their own mechanics). The accumulator scope, the tie rule and the band edges of the
per-hit cap are not documented (UNVERIFIED, OQ-511). adapt_res enters φ' (E-5.1) on SH and HP.

### 15.5 T-28

T-28 follows the overview's definition ("off: attenuation, flat enemy DR and adaptation are ignored", 00 §6.1) and the
register's list of the unique DR it controls (D-047: "attenuation ..., flat DR (Archons 20 % + 50 % status,
Juggernaut 90 %), Sentient adaptation, Orphix DPS-based DR"). T-28 off removes:

- attenuation (15.1) and the `attenuation_unknown` flag;
- DPS-scaled DR (15.3);
- adaptation and its per-hit caps (15.4);
- every flat enemy DR: `innate_dr[ℓ]` on every layer, `status_dr`, `dr_class`, `dr_class_og` and `part_dr`
  (R_ℓ = 1, E-5.20).

It keeps type modifiers and type resistances (`type_res`, E-5.1: they are damage-type modifiers, not DR), body parts,
status immunities and caps, `toxin_blocked`, `shield_weapon_immune`, strip immunities and the mission and squad
multipliers.

T-28 off is **not** inspiration parity. The inspiration disables only its `unique` functions (`unique = "notset"`,
`JS L7690`) and always applies its own `innateDR` data (`I:enemy` §5.2, §8; 32 enemies), whose values differ from
section 4.5: it has innate DR on Acolytes (0.625), Hemocyte (0.7), Lephantis (0.6), the Demolishers (0.2, Corpus shields
0.5), the Profit-Taker Orb (shield 0.5), Archons, Treasurer, the Raknoids and Amalgam Machinist, and none on
Juggernauts, Kuva Lich Thralls, Skittergirl, the Sentient drones or Scaldra Screamer. With T-28 off our engine applies
none of them; with T-28 on it applies section 4.5 plus attenuation and adaptation. Neither setting matches the
inspiration on these enemies, so oracle comparisons MUST avoid every enemy that has flat DR, type resistance,
attenuation, adaptation or DPS-scaled DR in either program's data (list in OQ-522). The overview's parenthesis
"(inspiration behaviour)" on T-28 is therefore inaccurate; OQ-522 asks the manager to remove it.

---

## 16. The enemy-side chain of one instance (ST-8)

For one damage instance with weapon-side vector W_t(k) (direct, radial, extra hit) or tick value (E-13), crit tier k,
body part, instance class c, landing at time τ on a target in state σ (stacks, A_eff, adaptation, pools) just before the
instance (CD-04):

```text
E-5.24  Per layer ℓ, per type t (order fixed; commutative products may be reordered in code)
  OG : D = W_t · φ^OG_t · Mag(n_MAG) · V_OG,t · M_OG · R_OG                                        (E-5.17)
  SH : D = W_t · φ'_t · Mag(n_MAG) · V_SH,t · R_SH                       t ∉ B_SH                   (E-5.14)
       D = 0 for weapon instances of a type the shield is immune to (shield_weapon_immune, E-5.26)
  HP : D = W_t · φ'_t · Vir(n_VIR) · V_HP,t · arm_t;  if arm_t < 1 and D > 0: D = max(1, D);  D = D · R_HP   (E-5.18)
  T-28 off: R_ℓ = R_OG = 1 and no attenuation, M_dps or adaptation (15.5)
  then, per layer, if the enemy has attenuation on ℓ and T-28 is on:
       D_t ← D_t · Att(X_att,ℓ)/X_att,ℓ,  X_att,ℓ = Σ_t D_t over every type of the layer formula
       (on HP: all types, bypass types included; on SH: t ∉ B_SH)                                    (E-5.21)
  then, if the enemy has dps_dr and T-28 is on:  D_t ← D_t · M_dps(X_dps)                          (E-5.22)
  DoT ticks: SLA (Cinematic) uses φ = 1 and arm = 1 on SH and HP; the other ticks use their own type (E-13);
             Treasurer (toxin_blocked = nullify) Toxin ticks are 0 while SH > 0
  Part flags (4.3): φ' = 1 on no_faction_mod parts; arm = 1 on no_armor parts; part_dr inside R_ℓ; crit_disabled and
             status_ok are applied by 02 (tier forced to 0, no procs)
  Object targets (is_object): φ = 1, no armor, no Viral; crit and status are disabled by 02
```

Equivalence with the overview: this is E-12 with φ replaced by φ' (type resistances), innate DR extended to R_ℓ, and
M_dps added; with no type resistance, no class or status DR and no unique DR it is E-12 term for term. The position of
attenuation after every modifier is CD-09. M_dps commutes with everything before attenuation; it is placed after
attenuation only for definiteness (no enemy has both).

Precision: f64. The non-commutative steps are the minimum of 1 (after the multiplicative HP terms, before R_HP) and
attenuation (after everything).

---

## 17. Layer routing, overflow and shield gate (ST-9)

The `layered` pass of both evaluators (TTK) routes each instance through the live pools. Order OG → SH → HP (CD-08;
`W:Overguard`, `W:Shield`).

```text
E-5.25  route(instance at time τ):            state: OG, SH, HP; gate_until = −∞ at the start of the pass
  D^ℓ_t are the E-5.24 values (attenuation already applied per layer on the whole instance)
  if OG > 0:
      Z = Σ_t D^OG_t                          (every type; Toxin and True do not bypass overguard)
      OG ← OG − Z
      if OG ≤ 0: OG ← 0; emit "OG broken" (Magnetic break proc, 03)      (overflow discarded: CD-08, D-049)
      return                                   (shield and health untouched by this instance)
  h_t = 0 for every type
  if SH > 0:
      h_t = D^HP_t for t ∈ B_SH                (bypass types go to health and are never gated; Decision)
      Z = Σ_{t ∉ B_SH} D^SH_t
      if Z < SH:  SH ← SH − Z
      else:                                    (Z ≥ SH: the shield reaches 0, including Z = SH exactly)
          a_sh = SH / Z;  SH ← 0;  gate_until ← τ + 0.1;  emit "SH broken" (Magnetic break proc, 03)
          h_t = g(instance) · (1 − a_sh) · D^HP_t   for t ∉ B_SH        (fractional overflow; Decision)
  else:
      g' = (τ < gate_until) ? g(instance) : 1
      h_t = D^HP_t for t ∈ B_SH;   h_t = g' · D^HP_t for t ∉ B_SH
  H_rt = Σ_t h_t
  if the enemy has Sentient adaptation (T-28 on) and H_rt > cap(HP/HP_max)·HP_max:          (15.4)
      h_t ← h_t · cap(HP/HP_max)·HP_max / H_rt;   H_rt ← cap(HP/HP_max)·HP_max
  HP ← HP − H_rt;  dmg_by_type[t] += h_t for every t;  process gates newly crossed (15.4)
  if HP ≤ 0: kill at τ

  g(instance) = 1     weak-point hit (T-03 head or weak_point on a direct instance), DoT tick, extra hit, special instance
              = 0     slam or heavy-slam radial instance
              = 0.05  every other instance
```

Sources and decisions:

- Overguard first, no gate on enemies, "prevents ... damage spillover" (`M:dt` Overguard text; `W:Overguard`): overflow
  of the breaking instance is discarded (CD-08). UNVERIFIED (OQ-011).
- Shield gate 0.1 s at 5 %, weak points bypass, slams fully blocked, status effects and Xata's Whisper not gated
  (`W:Shield` L341). The breaking instance's own overflow reaches health at the gate factor (CD-08: "reaches HP at 5 %
  during the 0.1 s gate, at 100 % for weak-point hits"). Which radial instances other than slams are blocked is not
  listed; Decision: only slam and heavy-slam radials get 0 (UNVERIFIED, OQ-509). An instance that brings the shield to
  exactly 0 is a break (gate and Magnetic break proc), with no overflow (`a_sh = 1`).
- Fractional overflow: the shield absorbs the fraction `a_sh` of every non-bypass type of the breaking instance and
  the rest continues to health with health-layer multipliers (armor, Viral, minimum of 1 already in D^HP). The game's
  rule is not documented (UNVERIFIED, OQ-509).
- Attenuation and routing: each layer's attenuation is computed once per instance from that layer's whole-instance sum
  (E-5.24), and the routed parts (bypass types while shields are up, the overflow remainder) are fractions of those
  attenuated values; Att(0) = 0. Decision, UNVERIFIED (OQ-509).
- Per-type shares (`dmg_by_type`, the damage breakdown): each type's `h_t` after its own gate factor, overflow fraction,
  attenuation and the proportional adaptation cap.
- Toxin while shields are up: direct Toxin and Toxin ticks go to health with full damage; health can reach 0 with
  shields up (`F:enemy` §4.2; `W:Enemy_Level_Scaling` EHP section; the inspiration does the same, `I:enemy` §9.2).
- DoT ticks route by the same function with their single type: Cinematic ticks hit OG, then SH, then HP; Toxin ticks
  skip SH but not OG (CD-08 "status ticks are not gated"; `F:status` §5).
- The EV evaluator walks the same routing with expected damage and without the gate (`g = g' = 1`, 00 section 4.2).
- Shield regeneration: none (10.5). Overguard never regenerates in scope. Shields that weapons cannot damage: E-5.26.

## 18. Layer views

The four non-depleting views of 00 section 4.1, enemy side:

| View | Exists when | Instance damage |
|---|---|---|
| `overguard` | OG_max > 0 | Σ_t D^OG_t (every type) |
| `shield` | SH_max > 0 | Σ_{t ∉ B_SH} D^SH_t (bypass types excluded; the inspiration's Shield panel tooltip says the same, `I:feat` §12); "immune" when `shield_weapon_immune = all` (E-5.26) |
| `armor` | A0 > 0 | Σ_t D^HP_t with E-5.18 (armor, minimum of 1) for every type, as if shields and overguard were gone |
| `health` | always | Σ_t D^HP_t with arm_t = 1 and no minimum of 1 ("100 % armor stripped") |

All views share one status timeline (00 section 4.1). Caps that depend on overguard (Cold 4, VIP Puncture 3) follow
`03` E-3.2, which owns that rule: in every non-depleting view and in the EV evaluator `og_present` is true when the
enemy spawns with OG_max > 0, so those caps apply for the whole window; in the layered pass `og_present` is the live
state (OG > 0 at the moment of application). This document supplies only OG_max and the `vip` flag. (The draft's own
view rule, "caps of a target whose overguard is gone", is withdrawn; `08` §4.5 adopted it and records the conflict as
OQ-804, so `08` must follow `03`; OQ-529.) Unique DR in views: attenuation and M_dps apply per instance (they need no
depletion); adaptation and per-hit caps do not (15.4). The inspiration's Health and Armor panels on a shielded target
with a Toxin attack show only the Toxin part (D-504, OQ-527).

## 19. Notes for the evaluators

`08` owns the algorithms; these properties of this document bind them:

- Nonlinear in the target state (CD-13): DR(A_eff) (square root of a product of strip terms), the minimum of 1, Vir and
  Mag, attenuation (concave in X_att), M_dps, the shield-break fraction. EV MUST take expectations over the stack and
  crit distributions (per crit tier and per stack state), never apply these functions to expected inputs. The minimum
  of 1 is applied per crit tier.
- c_heat(τ) depends on the Heat step counter k_h(τ) of `03` E-3.8, which steps up after ignition and steps down after
  the Heat episode ends; A_eff is therefore time-dependent even with constant stacks and keeps a decay tail of up to
  `4·1.5·(1 + SD*)` s after the last Heat proc expires.
- s_flat grows with the count of qualifying hits (monotone, permanent); in EV window mode it is the expected count of
  earlier qualifying hits, and f_flat is linear in it.
- Pool construction (sections 6 to 8) runs once per scenario and is deterministic.

---

## 20. Worked example

Corrupted Heavy Gunner (`M:enemies/orokin` L256-288: Faction Orokin, Health 700, EximusHealth 700, Armor 500, no
shield, BaseLevel 8, head 3.0×), level 1000, Steel Path, not Eximus. `F:enemy` §10; `I:enemy` §11.

1. q = 1000 − 8 = 992 (the SP offset is inside the 1000). Health class Orokin.
2. E-5.4: B_H = 700 (no SP flat bonus) × 2.5 = 1750. B_A = 500.
3. E-5.2 (Orokin health): pw1 = f32(992^2.1), f1 = 29,429.3203125; f2 = 1,212.5455322265625; t = 1, s = 1;
   Λ_H = f32(f1 + f32(f32(f2 − f1)·1)) = 1,212.544921875.
4. E-5.3: HP_max = f32(1750 × 1,212.544921875) = **2,121,953.5** (the wiki's value).
5. Armor: Λ_A = 71.7039794921875; A_raw = f32(500 × Λ_A) = 35,851.98828125; floor 35,851; clamp → **A0 = 2700**;
   DR = 0.9, m_arm = 0.1.
6. Why the inspiration shows 2,121,940 (`I:enemy` §11; `JS L8039-8125`): (a) its coefficient is `24·√5/5 =
   10.7331263`, not 10.7332: −14.5601 health; (b) it evaluates in f64 without the f32 blend: the f64 value with
   10.7332 is 2,121,954.5691, and the f32 blend costs −1.0691; (c) it rounds to an integer:
   `round(round(2,121,940.0090·100)/100) = 2,121,940`. 2,121,953.5 − 2,121,940 = 13.5 = 14.5601 − 1.0691 + 0.0090.
   Armor: it rounds (35,852) where the gadget floors (35,851); both clamp to 2700 (D-045).
7. Effective health against one neutral type with no strip: 2,121,953.5 / 0.1 = 21,219,535; Puncture or Viral (×1.5)
   14,146,357; with 10 Corrosive stacks A_eff = 540, m_arm = 0.597508, neutral EHP 3,551,340 (`F:enemy` §10's
   3,551,282 is an arithmetic slip; rounding m_arm to 0.5975 would give 3,551,387).
8. As an Eximus: d_exi = 992, k_exi = 6, c_exi = 0.25 (armored), B_H = max(770, 0.25·1600·6) = 2400, ×2.5 = 6000,
   HP_max = 7,275,269.5; OG: q_OG = 999, Λ_OG = 130,176 (binary32), OG_max = **1,562,112** (q = 992 would give
   1,552,896) (TV-05-07).

---

## 21. Edge cases

Each item is specified above (section in brackets) or marked out of scope with the reason.

| ID | Rule | Source |
|---|---|---|
| EC-05-01 | Scaling uses q = L − L0, and the 70-80 smoothstep is in q: a base-level-8 enemy blends between levels 78 and 88 [6.1] | `W:Enemy_Level_Scaling` L20-25; `G:slider` L101-107; `F:enemy` §7.1 |
| EC-05-02 | Base level comes from data for every enemy; missing → 1 (the inspiration has `b_lvl` on only 36 of 498 entries, D-505) [4.1] | `M:infobox` L435; `I:enemy` §1.1 |
| EC-05-03 | Overguard uses q = L − 1 (page) not L − L0 (gadget); the two readings differ by up to ×2 or more below L0 + 50 (e.g. +85 % at L 50 and ×2.2 at L 40 for L0 = 8, Eximus base 12) and by 12.5 % at L 58 [8] | D-042; OQ-009; `python3` with E-5.2 |
| EC-05-04 | Binary32 order changes results: no f1(1 − s) + f2·s rewrite, no endpoint substitution; overguard at L 9999 is 1.3 % off f64 [6.3] | `W:Enemy_Level_Scaling` L42, L103 |
| EC-05-05 | Level cap 9999 (Eximus too since U25.0.7); Void Fissures can exceed it, outside the engine domain [6.5] | `W:Enemy_Level_Scaling` L8 |
| EC-05-06 | Faction modifiers follow `FactionDamageOverride`, not `Faction` (Corrupted Jackal → Corpus; Thrax, Void Angels → Zariman; Wolf of Saturn Six → Grineer) [5.2] | `M:infobox` L607; `F:enemy` §2 |
| EC-05-07 | An override that is an internal path (6 entries, e.g. Errant Specter) is treated as Neutral with a data warning [4.2] | `F:enemy` §2 (data-entry errors, UNVERIFIED) |
| EC-05-08 | Warframe Specters: the data overrides to neutral, `W:Damage/Orokin` L2 says Orokin modifiers; Decision: data (neutral), UNVERIFIED [5.2] | OQ-521 |
| EC-05-09 | Sub-factions differ from their parent: Kuva Grineer resist Heat, Corpus Amalgam resist Blast and are weak to Magnetic, Infested Deimos resist Viral [5.1] | `M:dt` |
| EC-05-10 | Acolytes and the Stalker faction are neutral since U36 [5.1] | `W:Acolytes` L61 |
| EC-05-11 | Type resistances multiply φ and floor at ×0; φ never applies to OG (Void ×1.5 only) or to TRU, CIN, FIN, TAU [5.3] | `W:Damage_Type_Modifier` L9-13 |
| EC-05-12 | Legacy damage classes (Shield +75 % Magnetic, Indifferent Facade, Cloned Flesh ...) and the pre-U36 enemy module are never used [4.6] | `DS:wiki` §6-7 |
| EC-05-13 | Module coefficient conflicts: Infested C2 16.0998/16.1, Techrot shield E1 1.76/1.75, Corrupted shield key bug for Orokin units only (1.6 shown, 2.0 on the page; Anarchs have their own key with 2.0), Anarchs health curve; page values chosen [6.1] | D-044; OQ-502, OQ-503 |
| EC-05-14 | Steel Path: health and shield ×2.5 of the base, no armor change (U36), no overguard change; Simulacrum/Undercroft "+250 % armor" notes are outdated [7.2] | `W:The_Steel_Path` L411-412, L505 |
| EC-05-15 | SP level offsets are part of the entered level; Duviri SP is +20 levels with no health or shield change (enter SP off); Archwing/Railjack +50; Eidolons 110/120/130 [7.2] | `W:The_Steel_Path` L10-12, L59-64 |
| EC-05-16 | SP flat base-health (and shield) bonuses are added before ×2.5 (gadget order; UNVERIFIED); no bonus for Demolisher Necramechs (commented out); the Legacyte bonus is applied although the module doubts it; Babau's Archimedea bonus exists only as a separate record [7.2] | D-055; `M:enemies/techrot` L43-44; OQ-532 |
| EC-05-17 | Eximus base health depends on whether the unit has shields or armor (0.25 vs 0.375) and on d_exi; Codex health is not the Eximus base [7.3] | `W:Enemy_Level_Scaling` L161-193 |
| EC-05-18 | Eximus step variable: level difference (gadget, decision) vs absolute level (page notation, inspiration) [7.3] | OQ-501 |
| EC-05-19 | Archwing, Warden and Prosecutor Eximus have no overguard; the Prosecutor record already holds 10× base health and defaults to Eximus (E-5.5 applied on top, as the gadget does; OQ-533); Warden "+1.2 base Health" not modelled (UNVERIFIED) [7.3] | `W:Eximus` L164, L169, L179; `M:infobox` L447 |
| EC-05-20 | Entropic Eximus (U44): base armor 50, 90 % DR against non-melee damage (primary, secondary, arch-gun, glaive and gunblade ranged; DoT ticks inherit), immune to Warframe abilities while its orb is active; orb blocks radial attacks (not modelled: single target); inactive until an Eximus-kind input exists [4.5, 14] | `W:Eximus` L89-106, L262; OQ-523 |
| EC-05-21 | Guardian Eximus: 90 % DR to AoE for itself including its own OG (Eximus page) vs 90 % to all types for itself and allies (Damage Reduction page); ally aura out of scope by default [4.5] | `W:Eximus` L116-118; OQ-513 |
| EC-05-22 | Squad size scales Demolisher-family health (+50/+100/+200 %); Archon scaling is the Archon Hunt mission modifier; Acolytes scale by an unknown amount [7.4] | `W:Demolisher` L7; `W:Archon_Hunt` L57; OQ-518 |
| EC-05-23 | Mission modifiers: Archimedea Empowered and Archon Hunt ×(2 + 0.5·n_party) health and shield (n_party ≤ 4), Icebind ×(3 + n_party) including armor with n_party up to 6, Bolstered Belligerents OG = 50 % health [7.5] | `G:slider` L190-204, L384; `W:Archon_Hunt` L57; `W:Coaction_Drift` L36 |
| EC-05-24 | Overguard granted by other units (Ancient Protector 9× its health, Severed Warden), helmet or ampule removal (Nox, Dedicant), Void Armageddon: out of scope (needs a second enemy or an action) [8] | `W:Overguard` |
| EC-05-25 | Overguard: no armor, no faction (except Void ×1.5), no DR, no Viral; Magnetic and vulnerability apply; Secondary Fortifier M_OG; Smite, Null Audit, companion overguard bonuses out of scope [11] | `W:Overguard` |
| EC-05-26 | Base armor 0 stays 0; armor floor 200 applies to the initial value only; strips go below 200 [9.1] | `W:Enemy_Level_Scaling` L265; `G:slider` L224-227 |
| EC-05-27 | Armor is floored after scaling (gadget), the inspiration rounds (≤ 1 point difference) [6.4] | D-045 |
| EC-05-28 | Armor above 2700 uses A/(A + 300); unreachable through level scaling because of the clamp [9.2] | `W:Armor` L37-39 |
| EC-05-29 | Ability strips use total armor and add with Corrosive Projection and T-07; Heat and Corrosive multiply; the additive group is clamped at 0 [9.3] | `W:Corrosive_Projection` L31-58 |
| EC-05-30 | Corrosive Projection: 18 % per aura, 4 auras 72 %; Coaction Drift ×1.15 Aura Strength only (group aura); cap with Coaction Drift unknown [9.3] | `W:Coaction_Drift` L30-35; `F:enemy` open question 19 |
| EC-05-31 | Heat strip ramps (0.5^(1/4) per 0.5·(1 + SD) s) and steps back down after the Heat episode ends (4 steps of 1.5·(1 + SD) s); `03` owns the step counter k_h, this document uses heat_strip(k_h(τ)) at instance time, also after the last Heat stack expired [9.3] | D-026; `03` E-3.8 |
| EC-05-32 | Corrosive reaches 100 % only with 14 stacks (Emerald shards) or the Hydroid passive (first stack 50 %, any source, U34) [9.5] | `W:Armor` L417, L552-556 |
| EC-05-33 | Flat base-armor strips (Shattering Impact rank + 1 per Impact melee hit, Metal Auger 6 per dagger instance including its status ticks) act on base armor, before percentage strips, permanently; proportional model on the clamped value [9.4] | `W:Shattering_Impact`; OQ-506 |
| EC-05-34 | Shattering Impact needs quantized Impact > 0 (Galatine-type weapons get nothing except pure-Impact slams); works on status-immune enemies; not on Archons or Commandeered Primes [9.4] | `W:Shattering_Impact` notes |
| EC-05-35 | Polarize removes 400·Str armor points, Pillage 25 %·Str of armor, both only once shields are gone (shields first); order against percentage strips UNVERIFIED [9.3] | `W:Armor` L408-410, L438-440; OQ-507 |
| EC-05-36 | Sharpened Claws (companion) strips 120 %, additive with Corrosive Projection per its page, multiplicative per the Corrosive Projection page; not on Kuva Liches [9.3] | `W:Sharpened_Claws`; OQ-508 |
| EC-05-37 | Archons in Archon Hunts are immune to every armor strip [9.6] | `W:Archon_Hunt` L119 |
| EC-05-38 | Minimum of 1 per armor-reduced type, only after armor, never after other DR, never on SH or OG; Braton-type weapons deal at least 3 to armor [9.7] | `W:Armor` L41 |
| EC-05-39 | Toxin bypasses shields (direct and ticks) but not overguard and not armor; health can die with shields up [10.2, 17] | `W:Damage/Toxin_Damage` L13, L317 |
| EC-05-40 | Toxin does not bypass Treasurer or Hound shields; nothing bypasses the Treasurer's shield (True included); Treasurer Toxin procs deal 0 while shields are up; the full list is unknown [10.2] | `W:Treasurer` L30; `W:Hound` L35; OQ-029 |
| EC-05-41 | True damage bypasses armor and shields, has no faction modifier, does not bypass non-armor DR; its overguard behaviour is inferred (damages OG) [10.2, 11] | `W:Damage/True_Damage`; OQ-516 |
| EC-05-42 | Shield strips by abilities take a percentage of current shields (diminishing), Shield Disruption caps at 80 % with Coaction Drift [10.3] | `W:Shield` L357-415 |
| EC-05-43 | Shield gate: 0.1 s at 5 % to health after a full break; weak points bypass; slam radials blocked; DoT ticks and extra hits not gated; no gate on overguard; EV ignores the gate [17] | `W:Shield` L341; CD-08 |
| EC-05-44 | Shield recharge is not modelled; MC flags gaps > 3 s; Magnetic stops regeneration; Amalgam and Treasurer shields do not regenerate [10.5] | `W:Shield` L108, L437-450 |
| EC-05-45 | Magnetic multiplies shield and overguard damage, Viral only health (including armored health); a hit never benefits from the Viral or Magnetic proc it applies [10.1, 12] | CD-04; `F:status` §4.11, §4.13 |
| EC-05-46 | Vulnerability multiplies once per instance, never on the weapon side, once on each DoT tick at tick time; additive group; type- and class-conditional entries; U37 list also on SH and OG [13] | `W:Damage_Vulnerability` |
| EC-05-47 | Non-armor DR stacks multiplicatively, applies to SH and HP, never to OG (except Guardian's own AoE reduction) [14] | `W:Damage_Reduction` L53, L101 |
| EC-05-48 | Archons: 20 % flat DR, 50 % status-damage DR, 4-stack caps, armor-strip immunity, Warframe-ability debuff immunity (Decision for "most"), Narmer modifiers (U36.0.8), level 150 [4.5, 13.1] | `W:Archon` L32; `W:Archon_Boreal` L43; `W:Archon_Hunt` L117-119 |
| EC-05-49 | Eidolons: attenuation on shields and health (MDPI 200 %, MDPS 100 % of max health), body 2× with 2× crit damage, head 3×, radial gets the body values only; body-part table conflict; shields immune to weapons (E-5.26) and to Shield Disruption; ability debuffs ignored; status immunity conflict (D-511) [4.3, 4.4, 4.5, 10.6, 13.1] | `W:Eidolon_Teralyst` L75-93, L121; OQ-524, OQ-534 |
| EC-05-50 | Juggernaut: 90 % DR (Demolisher variant 20 %), +50 % DR against AoE and melee (DoT ticks of melee or radial hits included), maw and belly 10× with 4× crit damage and the only status-accepting parts, bodyshots per the body-part table (D-509), other parts 2× crit damage; Demolisher boils 0.5× without the crit bonus [4.3, 4.5, 14] | `W:Juggernaut` L30-39; `W:Enemy_Body_Parts` L670-679 |
| EC-05-51 | Demolishers: Infested ones Viral-immune; Demolisher Juggernaut and Charger immune to Viral and Toxin damage; no freeze at 10 Cold (slow 90 %); attenuation removed in U40.0.2 [4.4] | `W:Demolisher` L82-85; `W:Damage_Reduction` L330 |
| EC-05-52 | Attenuation: after every modifier, health-proportional cap from max health only, multishot in the denominator penalises shotguns; only Hemocyte, Lephantis, Eidolons have known fractions; MDPS not modelled [15.1, 15.2] | `W:Hemocyte` L23-35; OQ-510 |
| EC-05-53 | Attenuation was removed in U40.0.2 from Scaldra Dedicants, Demolishers, Jugulus, Saxum, Rogue Necramechs, Amalgams, Empyrean units, Errant Specters, Gruzzling, Necramite, Hounds, Babau, Treasurer, Tusk Thumpers; U40.0.3 Skittergirl gets 90 % DR instead [4.5] | `F:enemy` §9.1; `W:Damage_Reduction` L313-354 |
| EC-05-54 | Orphix: DPS-scaled multiplier on non-crit DPS with health modifiers, crits after; Condrix not modelled [15.3] | `W:Orphix` L30-44 |
| EC-05-55 | Sentient adaptation: 4 gates, 90/80/75/70 % plus a permanent 40 % per adaptation, no adaptation to TRU/FIN/CIN, gate-crossing hit unreduced, per-hit caps 25/20/20/15 %; Xata's Whisper and other Void sources reset the removable part; some Sentients do not adapt [15.4] | `W:Sentient` L293-323 |
| EC-05-56 | Shadow Stalker adapts at 95 % per type, same gates, also reset by The Vast Untime; no 40 % layer stated [15.4] | `W:Shadow_Stalker` L34-44 |
| EC-05-57 | Status caps on special enemies: Acolytes 4 (Impact 3), Archons and Liches 4, Radiation 4 on Liches, Acolytes, Necramechs, Sisters, Hounds; Cold 4 while `og_present`; VIP Puncture 3 while `og_present`; other bosses only Cold 4 and Blast 4 [4.4] | CD-07; `03` E-3.2; `W:Damage/Cold_Damage` L27; OQ-531 |
| EC-05-58 | Status immunity removes the type from proc weights but not its damage; damage immunity is type_res = 1 [4.4] | `F:status` §5 |
| EC-05-59 | Treasurer DR 99 % shield and 98 % health: the page was edited after U40.0.2, but no U40 note restates the values [4.5] | OQ-514 |
| EC-05-60 | Object health (Faction "Objects"): no crits, no status, no modifiers; hitscan does not quantize against it (`01`); not a normal target [16] | `W:Damage/Object` L1; `F:core` §14 |
| EC-05-61 | Non-body hitboxes (Symbilyst shield, Aerolyst canisters, Efervon growths): no status, no armor, no φ, crits apply [4.3] | `W:Enemy_Body_Parts` L36-38 |
| EC-05-62 | Head-crit only on parts flagged for it: none on Corpus humanoids and ground units, most Murmur and Techrot; MOA fanny pack 3× without; Jordas engines 1× with; Juggernaut 4×; Ropalolyst head 0.2×; critical promotion parts add a tier [4.3] | `W:Enemy_Body_Parts` L43-50 |
| EC-05-63 | U44: MOA rears (except Anti, Disc, Fusion), Bursa panels, Lumbering Fragment, Severed Warden arms, Scaldra Harbinger balloon, Amalgam Heqet backpack are weak points but not heads (k_wp = 1.0, no headshot bonuses) [4.3] | `W:Enemy_Body_Parts` L41 |
| EC-05-64 | Murmur and Techrot heads are about 1.5×, other factions 3× (3× since U32; older pages say 2×) [4.3] | `W:Enemy_Body_Parts` L13-14; `F:core` §10 |
| EC-05-65 | Radial damage uses the body part (P_body, h_body), never the head [4.3] | `F:weapon` §10; `W:Eidolon_Teralyst` L77 |
| EC-05-66 | Enemy Necramechs: the "immune to status effects" text is about player Necramechs; enemy ones take status with Radiation capped at 4; Demolisher Necramechs have no `ProcResists` in the module [4.4] | `W:Necramech` L102; `W:Damage/Radiation_Damage` L27; OQ-520 |
| EC-05-67 | Deimos Jugulus regenerates 2 % of max health per second after 10 s without damage: not modelled (continuous fire); out of scope [—] | `F:enemy` §9.2 |
| EC-05-68 | Kuva Lich Thralls: 25 % DR on top of armor [4.5] | `W:Damage_Reduction` L284 |
| EC-05-69 | Hyekka and Hyekka Master resist 80 % Heat (type_res) [4.5] | `W:Damage_Reduction` L278 |
| EC-05-70 | Raknoids: Aurax Atloc 80 % DR, Kyta 25 % on shields only, Scyto 50 % [4.5] | `W:Damage_Reduction` L300-303 |
| EC-05-71 | Prosecutor aura (allies −100 % to physical and primary elements, −85 % to combined, +100 % to its element; Void always hurts): out of scope (needs a second enemy); a proposed toggle could model it as type_res on the target [—] | `W:Guardsman`; `F:enemy` §9.2 |
| EC-05-72 | Ancient Disruptor aura (90 % DR to ability damage, also reduces Xata's Whisper per `F:buffs` §1.3) and Grustrag Bolt (DR against Grineer attackers): out of scope (aura from another enemy; enemy-versus-enemy) [—] | `W:Damage_Reduction` L171-172 |
| EC-05-73 | Kuva Lich, Sister, Coda, their Thralls and Hounds, and Skittergirl have no module record; their hand-authored rows are data gaps until a cited base record exists; adversary health scales per rank and level (U40 multipliers), not researched [4.1] | OQ-526 |
| EC-05-74 | U40 base-health changes (Treasurer 2,500, Juggernaut 900, Babau 10,000 in U40.0.3 ...) must come from current modules [4.6] | `F:enemy` §9.1 |
| EC-05-75 | Damage dealt to an immune or 0× type still counts as an instance (crit rolls, procs of other types); a fully nullified instance on health deals 0, never 1 (no floor without armor reduction) [9.7] | `W:Armor` L41 |
| EC-05-76 | The inspiration's "decrease armor" option is skipped on Steel Path except at the clamps; ours is T-07 in the additive group on the clamped value [9.3] | D-051 |
| EC-05-77 | DoT ticks: target-side terms (Viral, Magnetic, A_eff, φ, V) evaluated when each tick lands; innate DR once [16] | CD-05; D-031, D-038 |
| EC-05-78 | Void resistance: the U29.1 note says Eidolons, Amalgams and certain bosses keep a 50 % Void resistance; the post-U36 faction tables and the Void page ("neutral to all other health types") have none; not applied (UNVERIFIED) [5.1] | `W:Damage/Void_Damage` L15, L110-112 |
| EC-05-79 | Shields that weapons cannot damage: Eidolon shields (Operator Void only) and the Profit-Taker shield (displayed type only) get D^SH = 0 for weapon instances, an "immune" shield view and a layered pass that starts with SH = 0 by default (E-5.26); phase mechanics (Thrax and Vomvalyst Void forms, Jack O'Naut, Arctic Eximus globe, the Profit-Taker's type cycling and Gravimag phase) stay out of scope [10.6] | `W:Eidolon_Teralyst` L93, L121; `W:Profit-Taker_Orb` L59-66, L80; `W:Xata's_Whisper` L30; `W:Juggernaut` L40; `W:Eximus` L38 |
| EC-05-80 | Enemy damage and affinity scaling: out of scope (no effect on player damage) [—] | `W:Enemy_Level_Scaling` L325-369; `F:enemy` §7.6 |
| EC-05-81 | Hemocyte: immune to all statuses; its outer skin takes no weapon damage (P 0), so body hits use the head weak spot as default part [4.3, 4.4] | `W:Hemocyte` L19, L89 |
| EC-05-82 | Enemies immune to Warframe-ability debuffs (Eidolons, Archons, Entropic Eximus) ignore ability vulnerabilities, ability armor and shield strips and T-07; Demolishers and Scaldra Screamers dispel them every 5 s (flag) [13.1] | `W:Eidolon_Teralyst` L91; `W:Archon_Hunt` L117; `W:Demolisher` L5 |
| EC-05-83 | Breakable parts (Scaldra Dedicant backpack and flask DR 30 % until broken, Nox helmet, Dedicant ampule) are modelled unbroken; breaking is out of scope (needs an action) [4.3] | `W:Enemy_Body_Parts` L937-938 |
| EC-05-84 | Juggernaut maw accepts status only while open; modelled as always accepting (UNVERIFIED) [4.3] | `W:Juggernaut` L33 |
| EC-05-85 | Parts with a distinct health pool (Scaldra TI-92 turret: no armor, all hits non-crit; H-09 Efervon Tank proboscis) are recorded with their flags; damaging them as a separate target is out of scope [4.3] | `W:Enemy_Body_Parts` L939-940 |
| EC-05-86 | Records without a body-part table row use Multis; records without Multis (303) use the faction default; a Multis string without a damage multiplier (Dax "+1 Crit Tier, 1.5x Critical Damage") gets P = 1 [4.3] | `W:Enemy_Body_Parts`; `M:enemies` |
| EC-05-87 | T-03 on an enemy without a head part or a weak point falls back to the highest special part (Juggernaut maw) or the body, with a flag [4.3] | Decision |
| EC-05-88 | Shield break with damage exactly equal to the remaining shield is a break (gate, Magnetic break proc) with zero overflow [17] | Decision |
| EC-05-89 | Duplicate enemy names and empty or shared InternalNames: the record key is `<module>:<key>`; Grineer Queens (list `Stats`) is skipped [4.1, 4.6] | `python3` over `M:enemies` |
| EC-05-90 | Dax Malleus overguard exists only in the Undercroft and the Circuit (scenario option, default off) [4.1, 8] | `W:Overguard` L58 |
| EC-05-91 | Party size 5 or 6 is valid only with the Icebind mission modifier [7.1] | `W:Coaction_Drift` L36; `G:slider` L198, L384 |
| EC-05-92 | Isolator Bursa frontal 60 % reduction is its Front part (0.4×), the default body part [4.3] | `W:Enemy_Body_Parts` L500; `W:Isolator_Bursa` infobox |

## 22. Inspiration differences

Register rows this document owns or shares (00 section 5.2), restated with the engine behaviour, followed by new rows.
"Compat" rows switch in the compatibility profile only (00 section 4.6).

| ID | Topic | Inspiration (source) | Engine (this document) | Toggle |
|---|---|---|---|---|
| D-013 | Head-crit | every headshot crit uses 2·CDM (`JS L8319-8328`) | per-part `headcrit` data (4.3), consumed by `02` | — |
| D-026 | Heat armor strip | flat 50 % while any Heat stack is live (`JS L10815`) | A_eff uses `heat_strip(τ)` of `03` at instance time (E-5.10) | Compat |
| D-030, D-031 | Own-hit and tick timing | a hit's own Corrosive/Heat strips its own armor; DoT target terms frozen at the inducing hit (`JS L10093-10136`, L10820) | state before the instance (CD-04); tick-time evaluation (CD-05) | Compat |
| D-038 | Innate DR on DoT | `(1 − innateDR)²` on most ticks, cubed on armor (`JS L10885`, L9202) | R_ℓ once per instance (E-5.20) | Compat |
| D-039 | DoT on shields | ×3 (`Ob`, `JS L10576`) | ×1 | Compat |
| D-040 | Armor-layer DoT faction | type modifier twice (`JS L9198-9202`) | once | Compat |
| D-042 | Overguard scaling | f1 below Δ 45, f2 from Δ 45, no smoothstep, Δ = L − L0 (`JS L7617-7636`); 95,962 at Δ 45 where the wiki has 73,823.25 (L0 = 1) | E-5.2 smoothstep 45-50, q = L − 1 (section 8) | Compat |
| D-043 | Eximus | default curves × step factor on absolute level; armor × up to 1.625; b > 100 pure high curve (`JS L7637-7686`) | E-5.4/E-5.5: faction curves, base increase with d_exi = L − L0, no armor change | Compat |
| D-044 | Curves | Techrot 0.015/2.1, 13.64/0.72, shield 3.4106; Scaldra, Amalgam, Anarchs on default; Sentient/Murmur/Neutral shield f2 2.0; coefficients `k·√5/5` (`JS L8048-8095`) | table 6.1 (page values; Anarchs per module) | Compat |
| D-045 | Arithmetic | f64 `f1(1 − s) + f2·s`, `round(round(x·100)/100)` to integer, armor `round` (`JS L8096-8125`) | binary32 E-5.2/E-5.3, no rounding, armor `floor` (TV-05-01 decomposes the 13.5 difference) | Compat |
| D-046 | Faction table | Sentient +Magnetic; Amalgam lacks Magnetic; Scaldra resists Toxin; no overrides; " Infested", " Sentient", UNKNOWN, Infestation match nothing (`JS L5503-5518`) | table 5.1 with `damage_faction` (5.2) | — |
| D-047 | Unique DR | disabled for all enemies (`unique = "notset"`, `JS L7690`); its `innateDR` data stays on | section 15 where documented; T-28 off removes attenuation, M_dps, adaptation and every flat DR (15.5), which is not inspiration parity (OQ-522) | User (T-28) |
| D-048 | Shield gate | absent (`I:enemy` §5.3) | E-5.25 in MC TTK | — |
| D-049 | Overflow | discarded on every layer (`applyDamageLayer`, `JS L13805-13809`) | OG overflow discarded; SH overflow fractional and gated (E-5.25) | Compat |
| D-050 | Toxin exceptions | none | `toxin_blocked` (10.2) | — |
| D-051 | Armor strip option and CP | option on base armor before clamps, skipped on SP except at the clamps, armor in (0, 200] reset to 200·(1 − d); CP multiplies (`JS L7698-7712`, L8113-8116, L10805-10820) | E-5.10: T-07 and every CP in the additive group on the clamped A0 | User (T-07) + Compat |
| D-052 | Enemy-specific resistance | `elemRes` skipped on armor-layer direct hits; armorFull uses the Toxin-only map (`JS L11364-11374`; `I:queue` QM16-QM17) | `type_res` applied on SH and HP for every instance (E-5.1) | — |
| D-053 | Status caps (with `03`) | `maxProcStacks` caps per-hit procs and live stacks, not DoT damage (`JS L10011-10031`, L10119-10127) | data 4.4; rule CD-07 | Compat |
| D-055 | SP flat bonuses | not modelled (`I:enemy` §2.6) | added before ×2.5 (E-5.4) | — |

New rows (range D-501 ... D-599):

| ID | Topic | Inspiration behaviour | Game behaviour (source) | Decision | Toggle |
|---|---|---|---|---|---|
| D-501 | Corpus "crewmans-rangers" resistances | `elemRes` 0.75 to Cold, Blast, Magnetic, Viral on 19 Corpus units, applied to damage and DoT (`I:enemy` §6; `JS L9110-9119`, L10889-10893) | no wiki source found (`I:model` open question 2) | not applied; `type_res` only from wiki pages (UNVERIFIED, OQ-528) | Compat |
| D-502 | Innate DR values | `innateDR` data on 32 enemies, e.g. Acolytes 0.625, Hemocyte 0.7, Lephantis 0.6, Demolishers 0.2 (Corpus ones shield 0.5), Profit-Taker shield 0.5; none on Juggernauts, Thralls, Skittergirl, Sentient drones, Scaldra Screamer (`I:enemy` §10) | Acolytes, Hemocyte, Lephantis use attenuation, not flat DR; Demolishers lost attenuation in U40.0.2 and have no documented flat DR except Demolisher Juggernaut and Bursa 20 % (`W:Damage_Reduction` L426-500; `W:Juggernaut` L38; `W:Bursa` L30); Juggernauts 90 %, Sentient drones, Scaldra Screamer per 4.5 | table 4.5 (wiki values only); Amalgam Machinist shield 0.5 agrees | Compat |
| D-503 | Minimum of 1 per type | floor on the per-type armor-layer value after `(1 − innateDR.armor)`, before crit and per-hit multipliers, whenever base armor > 0 (`JS L9801-9817`) | floor after armor only; non-armor DR has no floor (`W:Armor` L41) | E-5.13: after the multiplicative HP terms, before R_HP, only when A_eff > 0 (UNVERIFIED position, OQ-517) | Compat |
| D-504 | Health and armor panels on shielded targets | with a Toxin attack the `health` and `armor` maps hold only Toxin (`JS L9265-9283`; `I:queue` QM16) | not a game rule (a view definition) | views of section 18 use every type | Compat |
| D-505 | Base level data | `b_lvl` on 36 of 498 enemies, default 1 (`I:enemy` §1.1) | `BaseLevel` on 1013 of 1019 module entries (`DS:wiki` §6) | module data; oracle tests use enemies whose base levels agree | — |
| D-506 | Armor-strip immunity | none: Archons are stripped by Corrosive, Heat, CP and the option (`JS L10805-10822`) | Archons are immune to armor reduction (`W:Archon_Hunt` L119) | `armor_strip_immune` (9.6) | — |
| D-507 | Archon Hunt and other mission health multipliers | not modelled | ×(2 + 0.5·n_party) on health and shields in Archon Hunts; Archimedea and Icebind multipliers (`W:Archon_Hunt` L57; `G:slider`) | optional mission field (7.5), default none | — |
| D-508 | Hemocyte, Eidolon and Profit-Taker special defences | Hemocyte `immun.status = all`; no shield immunity to weapons; no ability-debuff immunity | Hemocyte immune to all statuses, outer skin invulnerable (`W:Hemocyte` L19, L89); Eidolon shields only take Operator Void damage (`W:Eidolon_Teralyst` L121), Profit-Taker shield only its displayed type (`W:Profit-Taker_Orb` L59-66); Eidolons and Archons ignore Warframe-ability debuffs (`W:Eidolon_Teralyst` L91; `W:Archon_Hunt` L117) | 4.4, E-5.26, 13.1 | — |
| D-509 | Juggernaut maw and belly flags | not modelled (headshot option uses the generic head) | body-part table: bodyshots with 4× critical headshot bonus (`W:Enemy_Body_Parts` L670-679); `W:Juggernaut` L32-33: "weakpoints" (wiki-internal conflict) | table flags; UNVERIFIED | — |
| D-510 | Partial status immunities without a wiki source | Heat on Railgun, Shockwave, Fusion and Anti MOAs and Fire Prosecutor; Impact on Remech Osprey, Scyto Raknoid; Viral/Impact/Magnetic on Kyta Raknoid; six types on Corrupted and Orbital Strike Drones; Slash on Tomb Guardian; Toxin on Techrot Babau; all on Lephantis, Orphix, Wolf of Saturn Six (`enemies-min.json` `immun.status`) | no per-type page or module record lists them (4.4) | not applied; renormalised proc weights differ on these enemies | Compat |
| D-511 | Eidolon status immunity | all (`immun.status = all`) | `W:Status_Effect` L838-850: full immunity; `W:Eidolon_Teralyst` L81: "immune to all Status Effects, except for Void and Tau" (wiki-internal conflict) | ALL; UNVERIFIED (OQ-534) | — |
| D-512 | Aurax Atloc Raknoid DR | innateDR 0.8 | 80 % (`W:Aurax_Atloc_Raknoid` L13, `W:Damage_Reduction` L301) vs 70 % (`W:Enemy_Body_Parts` L541) (wiki-internal conflict) | 0.80 (per-enemy and per-mechanic page against a summary table, 00 §1.4) | — |
| D-513 | Adaptation reset by Xaku's Void | adaptation not modelled (unique DR off) | Xata's Whisper resets (`W:Sentient` L311, `W:Xata's_Whisper` L18) vs "with the exception of Xaku's abilities" (`W:Damage/Void_Damage` L17) (wiki-internal conflict) | resets on its Void proc; UNVERIFIED (OQ-536) | — |

Features the inspiration lacks entirely and that are off by default here (no register row needed; oracle scenarios
leave them off): flat base-armor strips, Polarize points, Hydroid passive, squad Corrosive Projection copies, external
shield strips, mission modifiers, shield regeneration flag, Sentient adaptation (D-047 covers unique DR).

## 23. Open questions

Overview questions this document depends on: OQ-008 (caps on DoTs), OQ-009 (overguard q and SP), OQ-010 (Eximus
variable), OQ-011 (overflow), OQ-022 (attenuation parameters), OQ-026 (f32 and armor floor), OQ-029 (Toxin-blocking
shields). New ones:

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-501 | Eximus step factor k_exi: level difference (gadget) or absolute level (page notation `x` = current level elsewhere on the page; inspiration)? | L − L0 | Eximus Heavy Gunner (L0 8) at level 58, normal path: health 82,304.54 (d = 50) vs 86,694.12 (absolute 58); also at level 23 (d = 15 vs 23): 0.25·1200·1 vs 0.25·1200·1.2 base |
| OQ-502 | Infested health C2: 16.0998 (page) or 16.1 (module; the Hemocyte page's example 2,472,207 uses 16.1 while the same page's calculator uses 16.0998) | 16.0998 | Level-117 Hemocyte health readout: 2,472,176.75 vs 2,472,207.75; or any Infested unit's health at a high level |
| OQ-503 | Techrot shield E1 (1.76/1.75), Corrupted (Orokin) shield C2 (2.0 page / 1.6 via the module key bug; Anarchs have their own key with 2.0), health curves for Narmer, Zariman, Stalker (Default?) and Anarchs (Orokin vs Default) | page; module for Anarchs health | in-game health and shield readouts (Simulacrum enemy info with a level slider, or the Codex at a fixed level) of one unit per class at q ≥ 100 |
| OQ-504 | Does the game's binary32 pow match "pow on binary32 inputs rounded once"? Is the base product `f32(f32(B)·M)` with B rounded once after the SP and Eximus factors? | yes | readouts at levels where the two choices differ by one ulp (search with the python model); low priority |
| OQ-505 | SP flat health bonus before or after ×2.5 (D-055) | before | Juggernaut at level 115 SP: 2,999,871.25 (before) vs 1,799,922.63 (after) (TV-05-09) |
| OQ-506 | Flat base-armor strips after the U36 clamp: proportional on the clamped value, recomputed with the clamp, or subtracted after scaling? | proportional | Shattering Impact rank 5 on a level-200 Heavy Gunner (base 500, clamped): armor icon or damage numbers after 10, 40 and 80 hits (proportional: 2376, 1404, 108) |
| OQ-507 | Polarize points: subtracted before or after percentage ability strips and CP? | after the additive group, before status | Polarize + Terrify on a clamped Grineer: damage numbers both cast orders |
| OQ-508 | Sharpened Claws: additive with Corrosive Projection (its page, with a worked example) or multiplicative (Corrosive Projection page)? | additive | rank-1 Sharpened Claws plus one Corrosive Projection: 22 % armor remaining (additive) vs 32.8 % (multiplicative) |
| OQ-509 | Shield-break overflow: fraction of the breaking instance, gate factor on its overflow, gate on Toxin, which radials are fully blocked | fractional, 5 % (weak point 100 %), Toxin never gated, only slam radials blocked | one-shot tests on a shielded Corpus unit with exact damage numbers: body vs head; a rocket explosion vs a slam |
| OQ-510 | Attenuation fractions for Archons, Acolytes, Liches and other bosses; MDPS model; MS_att for beams and DoT; layers | known fractions only; MS_att = ms for non-beam hits, 1 otherwise | patch notes or developer statements; damage numbers against an Archon with a single sniper hit at two damage levels |
| OQ-511 | Adaptation accumulator (since spawn or since last gate), tie rule, per-hit cap bands | since spawn; lowest enum index; bands as written | Sentient fight logs with two damage types in known proportions |
| OQ-512 | Orphix DPS: modded or effective fire rate; does the multiplier reach DoT ticks? | modded FR; yes | Orphix damage numbers with and without a reload-heavy weapon at equal burst DPS |
| OQ-513 | Guardian Eximus: AoE-only DR for itself (Eximus page, 2026-11-16) or all-type DR for itself and allies (Damage Reduction page) | Eximus page | Guardian Eximus hit with a hitscan rifle and with an explosion |
| OQ-514 | Treasurer 99 % shield and 98 % health DR after U40 (the page was edited 2025-11-01, after U40.0.2, but no U40 note restates the values) | apply | Treasurer damage numbers at a known level |
| OQ-515 | Status caps not on wiki pages (Void Angel, Demolisher Voidrig, H-09 Efervon Tank 4; Zealoid Prelate 10); Evensong's "enemies with overguard usually capped at 4" vs Overguard page (Cold only); which enemies are VIPs (`vip`) | not applied; Cold only; no VIPs | stack counters on those enemies in the Simulacrum |
| OQ-516 | Does True damage bypass overguard? | no (damages OG) | Basmu pulse against an Eximus |
| OQ-517 | Minimum of 1: before or after vulnerability and Viral; applies when A_eff = 0? | after all multiplicative HP terms; no | a 3-type low-damage weapon against a clamped target with Molecular Prime on and off |
| OQ-518 | Squad scaling of Acolytes (amount unknown); is the Archons' squad scaling exactly the Archon Hunt modifier; does Demolisher squad scaling touch shields? | Acolytes none (flag); Archons via `archon_hunt`; health only | health readouts in 1- and 4-player squads |
| OQ-519 | Meaning of `Stats.Overguard` on non-Eximus units (Thrax 15): base scaled like Eximus, or a fixed value? | scaled base | Thrax Centurion overguard at two levels |
| OQ-520 | Enemy Necramech status immunity vs Radiation cap 4 (`03` currently lists Necramechs as immune to all) | not immune, Radiation 4 | Simulacrum status test on a Rogue Voidrig |
| OQ-521 | Warframe Specter modifiers: neutral (data) or Orokin (Damage/Orokin page)? | neutral | Puncture vs Impact damage numbers on an Excalibur Specter |
| OQ-522 | T-28 wording: off removes attenuation, M_dps, adaptation and every flat DR as 00 §6.1 and D-047 say (15.5); the parenthesis "(inspiration behaviour)" is wrong because the inspiration always applies its own `innateDR` table. Oracle-unsafe enemies in both settings: Archons, Acolytes, Juggernauts, Demolishers, Demolysts, Treasurer, Raknoids (Aurax Atloc, Kyta, Scyto), Amalgam Machinist, Hemocyte, Lephantis, Eidolons, Profit-Taker Orb, Hyekkas, Toxic Ancient, Sentient drones (Choralyst, Mimic, Ortholyst, Summulyst, Symbilyst), Scaldra Screamer and Dedicant, Orphix, the Sentient adaptation group, Shadow Stalker, Kuva Lich Thralls, Skittergirl | 00 wording; remove all flat DR when off | manager decision: drop "(inspiration behaviour)" from T-28, or add a third value `inspiration` that applies the inspiration's `innateDR` table in the compatibility profile |
| OQ-523 | Proposed changes for 00 section 6 and the catalogue: new toggles mission modifier (7.5), Eximus kind (Entropic, Guardian; default `generic`), `armor_hydroid`, squad Corrosive Projection count and Coaction Drift, external shield strip (E-5.16), flat base-armor strip rate, target inside a Guardian or Prosecutor aura, `context_overguard`, `shield_removed_externally` and `pt_shield_type` (E-5.26), a named-part selector for T-03 (4.3); T-23 rewording: party size 1..4 (1..6 in Icebind) also feeds the mission modifiers of every enemy, not only squad-scaled enemies; the overview symbol `vuln` refined to `vuln[ℓ][t]`; a `source_kind` column (Warframe ability or not) in `04`'s vulnerability and strip catalogue for 13.1; default layer mask of vulnerabilities without a stated mask | not toggles yet; vulnerability default mask {HP}; `vuln[ℓ][t]` used locally | manager decision; for the mask, damage numbers on an Eximus' overguard under Petrify |
| OQ-524 | E-7 writes Part = 1 for body and radial hits; enemies with P_body ≠ 1 or h_body ≠ 1 (Eidolons 2× and 2×, Juggernaut h 2, Isolator Bursa Front 0.4, Hemocyte and Lephantis default parts) need E-7 to use P_body and h_body; headcrit values outside {1, 2} (4, 1.5, 0.2) are `02` OQ-203; Eidolon body-part table conflict (1× vs 2×) | P_body, h_body from 4.3 | manager decision for `02`; Eidolon damage numbers on body and synovia |
| OQ-525 | Enemy shield recharge delay and rate (U36) | no regeneration, flag | frame-timed shield recovery of a Corpus Crewman at two levels |
| OQ-526 | Base records (health, shield, armor, base level) for the enemies without a module record (Kuva Lich, Sister of Parvos, Technocyte Coda, Kuva Lich Thralls, Sister Hounds, Skittergirl) and adversary health per rank and level after U40 | data gap; their 4.4/4.5 rows attach to nothing; adversary records flagged | wiki pages with infobox values, patch notes, or readouts at known rank and level |
| OQ-527 | Health and armor views on a shielded target with a Toxin attack: our views use all types (D-504); confirm this is the desired output | all types | user decision |
| OQ-528 | Source of the inspiration's 0.75 Cold/Blast/Magnetic/Viral resistance on Corpus crewmen and rangers | not applied | wiki page or in-game numbers on a Railjack Corpus crewman |
| OQ-529 | Overguard-state caps in the shared view timeline: this document now defers to `03` E-3.2 (views and EV: caps apply when the enemy spawns with overguard; layered pass: live state); `08` §4.5 still holds the draft's rule (OQ-804) | `03` E-3.2 | manager: align `08`; game check: Cold stack count on an Eximus before and after its overguard breaks |
| OQ-530 | Ingestion of the `W:Enemy_Body_Parts` tables (not in `DS:wiki` or 00 §1.5): column flags, footnote mapping, join by link (4.3); Multis flag rules for non-Head parts | parse the tables; Multis rule M | `docs/data/` owner adds the source; spot checks of headcrit (orange/red numbers) on a MOA fanny pack and a Scaldra backpack |
| OQ-531 | Caps of ordinary bosses (`boss = true`): only Cold 4 and Blast 4, or every status 4 like Liches? | Cold and Blast only | Viral and Corrosive stack counters on Vay Hek or a Commandeered Prime in the Simulacrum |
| OQ-532 | Legacyte SP/Archimedea +7,500 (module doubts it); Techrot Babau Archimedea bonus 9,000 (separate record with Health 9,000) | apply Legacyte; Babau records as given, flagged | health readouts of Legacyte on Steel Path and of Babau in Temporal Archimedea at a known level |
| OQ-533 | Prosecutor (Eximus record with 10× base): does the Eximus base increase E-5.5 apply on top of the 10×? | yes (gadget) | Prosecutor health readout at two levels with d_exi ≤ 15 and d_exi > 50 |
| OQ-534 | Status immunity of Eidolons (all, or all except Void and Tau), Anu Pyrolyst and Choralyst (legacy infobox `procresistance = All`) | ALL for all three | Void and Tau procs on an Eidolon; any proc on a Choralyst |
| OQ-535 | Instance class of DoT ticks, Blast stacks and extra hits for class DR (Juggernaut, Entropic) | inherit the inducing instance's weapon class and radial flag | Slash tick numbers from a melee hit vs a rifle hit on a Juggernaut |
| OQ-536 | Does Xaku's Xata's Whisper Void proc reset Sentient adaptation (Sentient and Xata pages) or not (Void page)? | resets | Xata's Whisper on an adapted Battalyst: damage before and after a Void proc |

## 24. Test vectors

All numbers were computed with `python3` (numpy float32 for E-5.2/E-5.3 exactly as written in 6.3). Scaling results
MUST match bit for bit (binary32). f64 results: relative tolerance 1e-12 unless stated.

**TV-05-01 Corrupted Heavy Gunner health, L 1000, Steel Path (normal and inspiration paths).**
Inputs: Orokin health class (0.015, 2.10, 10.7332, 0.685); health 700; L0 8; L 1000; steel_path; not Eximus; n = 1.
Expected: q = 992; f1 = 29429.3203125; f2 = 1212.5455322265625; t_lvl = 1; s_lvl = 1.0; Λ_H = 1212.544921875;
B_H = 1750; HP_max = **2121953.5** (binary32; equals the wiki). Normal path (B_H = 700): 848781.4375.
Reference values (not engine outputs): f64 evaluation with 10.7332: 2121954.5691025197; inspiration (coefficient
24·√5/5 = 10.733126291998991, f64, `round(round(x·100)/100)`): 2121940.009042097 → **2121940**. Decomposition of
2121953.5 − 2121940 = 13.5: coefficient +14.560060, binary32 blend −1.069103, inspiration rounding +0.009042.

**TV-05-02 Armor at the cap, Corrosive strip.**
Inputs: armor 500, q = 992 (same enemy). Expected: Λ_A = 71.7039794921875; A_raw = 35851.98828125; floor 35851;
A0 = **2700**; DR = 0.9; m_arm = 0.1. With 10 Corrosive stacks (c_corr = 0.80): A_eff = 540, DR = 0.4024922359499621,
m_arm = **0.5975077640500379** (damage ×5.975 vs unstripped). With 1 stack (0.26): A_eff = 1998, m_arm =
0.22579072596616356. Inspiration armor (reference): `round(round(35851.98056·100)/100) = 35852` → 2700.

**TV-05-03 Lancer, L 100, Steel Path (overview 3.8 cross-check).**
Inputs: Grineer health class; health 100; armor 100; L0 1; L 100; SP. Expected: q = 99; health f1 = 256.17474365234375,
f2 = 294.4855041503906, s_lvl = 1, Λ_H = 294.4855041503906, HP_max = **73621.375**; armor Λ_A = 13.554122924804688,
A_raw = 1355.412353515625 → A0 = **1355**; DR = 0.6375735251718033; m_arm = 0.3624264748281967.

**TV-05-04 Smoothstep inside the transition.**
Inputs: Grineer health class, health 100, L0 1, not SP. (a) q = 72: t_lvl = 0.2, s_lvl = 0.10400000214576721,
f1 = 130.9080047607422, f2 = 234.35047912597656, Λ_H = 141.666015625, HP_max = **14166.6015625** (f64 reference
14166.608180127823). (b) q = 75: s_lvl = 0.5, f1 = 142.6514129638672, f2 = 241.31085205078125, Λ_H = 191.98114013671875,
HP_max = **19198.11328125** (f64 19198.115515315218).

**TV-05-05 Eximus Heavy Gunner, L 58, normal path.**
Inputs: Grineer; health 300 (EximusHealth 300); armor 500; no shield; L0 8; L 58; Eximus. Expected: d_exi = 50, k_exi = 4.5,
c_exi = 0.25, B_H = max(330, 0.25·1200·4.5) = 1350; Λ_H = 60.96632766723633 (s_lvl = 0, q = 50); HP_max = **82304.5390625**;
non-Eximus: 18289.8984375. Armor: A_raw = 2850.376953125 → 2850 → A0 = **2700**. Overguard: B_OG = 12; q_OG = 57:
f1 = 15835.001953125, f2 = 9892.466796875, s_lvl = 1, Λ_OG = 9892.466796875, OG_max = **118709.6015625**; with q = L − L0
= 50: 105505.953125. Alternative Eximus reading (absolute level 58, OQ-501): k_exi = 4.74, B_H = 1422, HP_max = 86694.1171875.
Inspiration reference: health 300·4.74·(1 + 0.015·50²) = 54747; overguard at Δ = 50: 105505.96.

**TV-05-06 Overguard smoothstep.**
Inputs: Eximus, B_OG = 12. (a) q_OG = 44: s_lvl = 0, Λ_OG = 5623.14404296875, OG = 67477.7265625. (b) q_OG = 45: s_lvl = 0,
Λ_OG = 6151.9375, OG = **73823.25** (inspiration at Δ = 45 uses f2: 95,962). (c) q_OG = 46: s_lvl = 0.10400000214576721,
Λ_OG = 6866.88232421875, OG = 82402.5859375. (d) q_OG = 47: s_lvl = 0.35200002789497375, f1 = 7320.521484375,
f2 = 8315.9833984375, Λ_OG = 7670.92431640625, OG = **92051.09375**. (e) q_OG = 50: OG = 105505.953125.

**TV-05-07 Binary32 at extreme levels.**
(a) Corrupted Heavy Gunner Eximus, L 1000, SP: d_exi = 992, k_exi = 6, c_exi = 0.25, B_H = max(770, 0.25·1600·6) = 2400, ×2.5 = 6000,
HP_max = **7275269.5**. q_OG = 999: f1 = 1494008960.0, f2 = 130192.375, Λ_OG = 130176.0, OG_max = **1562112.0** (f64
reference 1562308.7647733688); q = 992 instead: 1552896.0. (b) q_OG = 9998: f1 = 14988003508224.0, f2 = 1034893.125,
Λ_OG = 1048576.0, OG = **12582912.0** (f64 12418719.9375, +1.3 %).

**TV-05-08 Shielded Corpus unit, L 200, Steel Path.**
Inputs: Elite Crewman: Corpus, health 110, shield 150, L0 1, L 200, SP. Expected: q = 199; Λ_H = 247.6087646484375,
HP_max = **68092.4140625**; Corpus shield class f1 = 223.33680725097656, f2 = 112.72704315185547, Λ_S = 112.72704315185547,
SH_max = **42272.640625**.

**TV-05-09 Steel Path flat bonus order.**
Inputs: Juggernaut: Infested health class (C2 16.0998), health 900, armor 200, L0 15, L 115, SP, sp_health_bonus 1800.
Expected: q = 100; Λ_H = 444.42535400390625; B_H = (900 + 1800)·2.5 = 6750; HP_max = **2999871.25**. Alternatives
(reference only): bonus after ×2.5: 1799922.625; no bonus: 999957.0625. Armor: A_raw = 2729.822265625 → 2729 → 2700;
innate_dr[HP] = 0.9; a melee or radial instance additionally ×0.5 (T-28 on).

**TV-05-10 Strip combinations on A0 = 2700.**
| Strips | A_eff | DR | m_arm |
|---|---|---|---|
| 1 Corrosive Projection + Terrify (100 % Str): Σ_add = 0.78 | 594 | 0.4221374183841087 | 0.5778625816158913 |
| same + 10 Corrosive | 118.8 | 0.18878559267062728 | 0.8112144073293728 |
| same + 10 Corrosive + Heat at full strip (c_heat 0.5) | 59.4 | 0.13349157276772194 | 0.8665084272322781 |
| 4 Corrosive Projections + full Heat + 10 Corrosive | 75.6 | 0.15059880477613363 | 0.8494011952238664 |
| Hydroid, 1 Corrosive stack (0.50) | 1350 | 0.6363961030678928 | 0.36360389693210715 |
| Hydroid, 10 Corrosive stacks (min(1, 1.04)) | 0 | 0 | 1 (no armor, no minimum of 1) |
| Shattering Impact rank 5, 30 hits, base armor 500 (f_flat = 0.64) | 1728 | 0.72 | 0.28 |
| `armor_strip_immune = all` with any of the above | 2700 | 0.9 | 0.1 |

**TV-05-11 Faction modifiers and overguard.**
(a) Kuva Grineer health, no armor, W = HEA 100, COR 100, IMP 100 (non-crit, no V, Vir = 1): D^HP = 50 + 150 + 150 =
**350**. (b) Same instance on overguard: 100 + 100 + 100 = **300**; with a VOI 100 component instead of HEA: 150 + 100
+ 100 = 350. (c) Corpus shield, W = PUN 800, IMP 400, TOX 200, Mag 0: D^SH = 1200 + 400 = **1600**; TOX goes to health:
200.

**TV-05-12 Per-type minimum of 1.**
Inputs: Orokin target, A_eff = 2700 (m_arm 0.1), W = IMP 2, PUN 5, SLA 3 (non-crit), no V, Vir = 1, no DR.
Before the floor: 0.2, 0.75, 0.3; after: **1, 1, 1** (Σ = 3). With innate_dr[HP] = 0.5: **0.5** each (no floor after
non-armor DR). With A_eff = 0: 2, 7.5, 3 (no armor, no floor). On the `health` view: 2, 7.5, 3.

**TV-05-13 Full health chain.**
Inputs: Grineer, A_eff = 540, 10 Viral stacks (Vir = 4.25), Molecular Prime (+1.0, multiplicative, all layers),
innate_dr[HP] = 0.2, W = IMP 100, SLA 50. Expected: m_arm = 0.5975077640500379; D_IMP = 100·1.5·4.25·2·m_arm =
761.8223991637983; D_SLA = 253.9407997212661; Σ = 1015.7631988850644; after R_HP: **812.6105591080516**.
Vulnerability group: W = IMP 100 on Grineer health without armor, Molecular Prime (×2) plus Petrify (+0.5) and
Containment Wall (+0.25) in the additive group: V_HP = 2·1.75 = 3.5, D = **525**; on overguard with only Molecular Prime
in the OG mask: 100·2 = 200.

**TV-05-14 Shield break, overflow and gate (MC layered pass).**
Inputs: Corpus target, SH = 1000 remaining, HP = 5000, no armor, no stacks, no V, instance W = IMP 400, PUN 800, TOX 200
(body hit, direct, not slam). Expected: TOX to health 200 (not gated); Z = 1600 > SH, a_sh = 0.625, SH = 0;
overflow Σ D^HP = 1600 × 0.375 = 600; g = 0.05 → 30. HP = 5000 − 200 − 30 = **4770**; gate open until τ + 0.1. Same
instance as a head hit (D values as given, g = 1): HP = **4200**. A second body instance 0.05 s later with D^HP Σ = 1000
(no Toxin): HP − 50. A Slash tick in the gate: full damage. EV TTK has no gate (g = 1), so the body hit also gives
HP = 4200.

**TV-05-15 Attenuation (Hemocyte example).**
Inputs: incoming X = 12000 (1000 × crit 3 × 2 crit bonus × 2 weak point), ms = 2, p_att = 0.05. (a) HP_max =
2,472,207 (the page's value): C_att = 123610.35, Att = 12000·123610.35 / (24000 + 123610.35) = **10048.917301530684**
(page: 10,049). (b) HP_max from E-5.3 with C2 16.0998 at L 117 (L0 1): 2472176.75 → C_att = 123608.8375, Att =
10048.897309417533. With C2 16.1: HP_max = 2472207.75.

**TV-05-16 DPS-scaled DR (Orphix).**
M_dps(800) = 1; M_dps(1000) = 1; M_dps(2500) = 0.82; M_dps(4000) = **0.7**; M_dps(5000) = 0.66; M_dps(10000) = 0.43;
M_dps(12000) = 0.3616666666666667; M_dps(100000) = 0.061. A non-crit direct instance of 500 health damage from an
attack with X_dps = 4000 deals 350; a yellow crit of the same instance (CDM 2) deals 700.

**TV-05-17 Adaptation multipliers (Sentient, health, Cold weapon).**
Sentient φ_COL = 1.5. After the first adaptation to Cold: φ' = 1.5·(1 − 0.9)·(1 − 0.4) = **0.09**. After a Xata's
Whisper reset: 1.5·(1 − 0.4) = **0.9**. Second adaptation to Radiation: φ'_RAD = 1.5·(1 − 0.8)·(1 − 0.4) = 0.18.
Shadow Stalker (neutral) after adapting to Slash: 1·(1 − 0.95) = 0.05. Per-instance cap at full health with
HP_max = 100,000: at most 25,000 removed by one instance.

**TV-05-18 Squad scaling.**
Demolisher-family enemy, health 1000, any class, q = 0 (Λ_H = 1): n_party = 1 → 1000; 2 → 1500; 3 → 2000; 4 →
**3000**. A regular enemy with n_party = 4: 1000. Archon Hunt mission modifier, n_party = 4, health 1000: B_H ×4 → 4000
(also on the Archon itself; 7.4).

**TV-05-19 Eximus with shields, and a unit without shield or armor.**
(a) Elite Crewman (Corpus, health 110, shield 150, no armor, L0 1), L 50, Eximus, normal path: d_exi = 49,
k_exi = 4.366666666666667, c_exi = 0.25 (shielded); B_H = max(121, 0.25·1010·4.366666666666667) = 1102.5833333333335;
B_S = 150·max(1.1, 4.366666666666667) = 655.0000000000001 (f32 655); q = 49: Λ_H = 58.45220947265625,
Λ_S = 19.87013816833496; HP_max = **64448.43359375**, SH_max = **13014.9404296875** (non-Eximus: 6429.7431640625 and
2980.520751953125). At L 11 (d_exi = 10): B_H = max(121, 252.5) = 252.5, B_S = 150·1.1 = 165.
(b) Charger (Infested, health 80, no shield, no armor, L0 1), L 30, Eximus: d_exi = 29, k_exi = 1.75, c_exi = 0.375;
B_H = max(88, 0.375·980·1.75) = 643.125; q = 29: Λ_H = 29.344324111938477; HP_max = **18872.068359375** (non-Eximus
2347.5458984375).

**TV-05-20 Steel Path flat health and shield bonuses (Commandeered Ash Prime).**
Inputs: Anarchs (health class Orokin per module, shield key Anarchs 0.02/1.75/2.0/0.75), health 15000, shield 1750,
armor 50, L0 1, L 170, SP, sp_health_bonus = sp_shield_bonus = 10000; not Eximus-capable (`boss`). Expected: q = 169;
Λ_H = 361.4388427734375, Λ_S = 94.74433135986328; B_H = (15000 + 10000)·2.5 = 62500, B_S = (1750 + 10000)·2.5 = 29375;
HP_max = **22589928.0**, SH_max = **2783114.75** (without the flat bonuses 13553957.0 and 414506.4375). Armor:
Λ_A = 19.74886703491211, A_raw = 987.443359375 → A0 = 987; `armor_strip_immune = flat_only`.

**TV-05-21 Armor floor and absent armor.**
Scaldra Flayer (armor 10, L0 1): q = 0 → A_raw 10 → A0 = **200**; q = 99 → A_raw 135.54122924804688 → **200**;
q = 173 → 200.80711364746094 → 200; q = 174 → 201.63372802734375 → **201**. Elite Crewman (armor absent): A0 = **0**
at every level, no armor reduction, no minimum of 1. Lancer (armor 100) at q = 0: 100 → **200**.

**TV-05-22 FactionDamageOverride.**
W = PUN 100, IMP 100, VIR 100 (non-crit, `health` view). Corrupted Jackal (Faction Orokin, override `"Corpus"`):
150 + 100 + 100 = **350** (with its own Orokin table it would be 400). Jordas Golem (override `""` → Neutral):
**300**. An override that is an internal path (Errant Specter): Neutral, 300, with a data warning.

**TV-05-23 Type resistance.**
(a) Hyekka (Grineer, type_res[HEA] = 0.8), `health` view, W = HEA 100, IMP 100: 100·1·0.2 + 100·1.5 = **170**
(the HEA part is 19.999999999999996 in f64).
(b) Demolisher Juggernaut (Infested, type_res[TOX] = type_res[VIR] = 1), A_eff = 200 (m_arm = 0.7550510257216821),
W = TOX 100, HEA 100: D_TOX = 0 (φ' = 0; no minimum of 1 because D = 0); D_HEA = 150·m_arm = 113.25765385825233; after
innate_dr[HP] = 0.2: **90.60612308660187** (T-28 on).

**TV-05-24 Overguard break, ticks on overguard.**
Eximus with OG = 1000 remaining, SH 0, HP 5000. Direct body instance W = IMP 800, TOX 400, VOI 200 (no Magnetic, no
V): D^OG = 800 + 400 + 300 = 1500 → OG = 0, "OG broken", overflow 500 discarded, HP stays **5000**. Before the break,
a Slash tick of 100 and a Toxin tick of 100 each remove 100 from OG (Toxin does not bypass overguard).

**TV-05-25 Toxin-blocking shields.**
(a) Treasurer (`nullify`, innate_dr[SH] 0.99, Corpus φ = 1 for these types), SH > 0, instance TOX 200, TRU 100,
IMP 300: B_SH = ∅, D^SH = 600·0.01 = **6**, health receives **0**; a Toxin tick while SH > 0 deals **0**.
(b) A record with `toxin_blocked = absorb` (Hound row; no Hound record exists, so the vector uses a Corpus record with
the flag), same instance, no DR: D^SH = 200 + 300 = **500**, health receives TRU **100**.

**TV-05-26 Class and status DR (T-28 on).**
Juggernaut (innate_dr[HP] 0.9, dr_class[radial_or_melee] 0.5), health damage after armor 1000 per instance: melee hit
**50**; radial hit **50**; rifle hit **100**; a Slash tick from a melee hit (inherits melee) ×0.05; from a rifle hit
×0.1. Archon (innate 0.2, status_dr 0.5): direct 1000 → **800**, Slash tick 1000 → **400**. Entropic Eximus formula
(inactive kind): a Toxin tick of 1000 from a rifle → 100; from a melee hit → 1000. With T-28 off every value above is
1000 (TV-05-39).

**TV-05-27 Point strips, shield strips, flat strip on an unclamped enemy.**
(a) A0 = 2700, no shields, one Polarize (Str 1): A_tot = **2300**, m_arm = 0.16933761370819245; with Terrify (0.60):
2700·0.4 − 400 = **680**, m_arm = 0.5483364083745514. On a shielded target Polarize removes 400 shield points instead.
(b) SH_max = 10000, four Shield Disruptions each worn with Coaction Drift: Σ = 4·0.18·1.15 = 0.828 → capped 0.80 →
SH_max' = **2000**; plus one ability shield strip s = 0.5 (Str 1): **1000**. Eidolon or Profit-Taker
(`shield_strip_immune`): 10000.
(c) Lancer base armor 100, q = 99, A0 = 1355; 5 rank-5 Shattering Impact hits (s_flat = 30): f_flat = 0.7,
A_eff = **948.5** (f64 948.4999999999999), m_arm = 0.46656771751233506.

**TV-05-28 Icebind and Empowered.**
Lancer base armor 100: q = 10: Λ_A = 1.2811706066131592; no mission: A_raw 128.1170654296875 → **200**; icebind
n_party = 1 (×4): 512.46826171875 → **512**; n_party = 4 (×7): 896.8193969726562 → **896**; n_party = 6 (×9):
1153.0535888671875 → **1153**; q = 99, n_party = 1: 5421.6494140625 → **2700**. Empowered, health 1000, q = 0,
no Archimedea bonus: n_party = 1..4 → **2500, 3000, 3500, 4000**. n_party = 5 is rejected unless mission = icebind.

**TV-05-29 Attenuation on a multi-type instance.**
HP_max = 2,000,000, p_att = 0.05 → C_att = 100000. Instance on HP: IMP 30000, HEA 10000 (X_att = 40000).
Direct, ms = 2: Att = 22222.222222222223; IMP **16666.666666666664**, HEA **5555.555555555556**. The same X_att as a
DoT tick or merged beam tick (MS_att = 1): Att = **28571.428571428572** (IMP 21428.57142857143, HEA 7142.857142857143).
X_att = 0: Att = 0.

**TV-05-30 Level domain.**
q = 0 (L = L0): f1 = f2 = 1, Λ = **1.0** for every stat, HP_max = health. L < L0: the scenario is rejected. Lancer
(health 100, L0 1) at L 9999, not SP: q = 9998, f1 = 4528003.0, f2 = 8141.794921875, s_lvl = 1, Λ_H = **8142.0** (the
binary32 blend rounds f2 − f1), HP_max = **814200.0**.

**TV-05-31 Sentient gate crossing and per-instance cap (layered pass, T-28 on).**
HP_max = 100000. HP = 76000 (24 % lost), a Cold instance with D^HP = 5000: cap at HP/HP_max = 0.76 is 25000 (not
binding); HP → 71000 (29 % lost): gate 0.25 crossed after the instance, which deals its full **5000**; a[1] = COL.
The next Cold instance with the same pre-adaptation 5000 deals 5000·(1 − 0.9)·(1 − 0.4) = **300**. Caps at band edges:
HP/HP_max = 0.75 → 25000; 0.7499 → 20000; 0.35 → 20000; 0.3499 → 15000; 0.20 → 15000; 0.1999 → none. At full health
an instance of 40000 removes **25000**.

**TV-05-32 Part selection.**
Nox (table: Body 1x; Weak Point Helmet 2x, Head 3x): `head` = Head, P 3, h 2. MOA (table: Legs 1x, Gun 0.5x; Head 1x*,
Fanny Pack 3x*†): `head` = Head, P 1, h 1; `weak_point` = Fanny Pack, P 3, h 1, k_wp = 1.0; `body` = Legs, P 1.
Juggernaut: `head` → body (flag `no_head_part`); `weak_point` → Maw, P 10, h 4, bodyshot (flag
`special_part_not_weak_point`). Jordas Golem: `body` = Engines, P 1 (not Body 0x). Isolator Bursa: `body` = Front,
P 0.4. Radial hit on an Eidolon body (P 2, h 2): non-crit ×**2**; yellow crit with CDM 2: c = 1 + (2·2 − 1) = 4,
×**8**.

**TV-05-33 Heat strip decay tail (03 E-3.8, SD = 0).**
A0 = 2700; the Heat episode ends at t_exp with k_h = 4. A_eff at t_exp + 0, 1.5, 3, 4.5, 6 s (a step at time t applies
to instances at t): **1350.0, 1605.4296052536733, 1909.1883092036785, 2270.420321185029, 2700.0**; m_arm =
0.36360389693210715, 0.3060051285664267, 0.24319322627165685, 0.1746963611157959, 0.1. No Heat stack is live during
this tail; the draft's "0 without live Heat" would have given 2700 (m_arm 0.1) at once.

**TV-05-34 Overguard-dependent Cold cap in the views (03 E-3.2).**
Eximus Heavy Gunner (OG_max > 0), pure Cold weapon applying 10 Cold procs within their duration: in every
non-depleting view and in EV, n_COL is capped at **4**, cold_cd = 0.10 + 0.05·3 = **0.25**, never Frozen. In the
layered pass the cap is 4 while OG > 0 and 10 after the overguard breaks. The same unit without Eximus: cap 10.

**TV-05-35 Boss caps.**
Councilor Vay Hek (`boss`, not listed): VIR cap **10**, COR cap 10, COL cap **4**, Blast detonates at **4**. Angst
(Acolyte): every status 4, IMP 3. Kuva Lich: no record (data gap).

**TV-05-36 Orphix DPS input for beams and melee.**
Beam attack, per-beam non-crit damage after φ 100, merged tick of b = 2 beams (W = 200), FR 12, ms 2:
X_dps = (200/2)·12·2 = **2400**, M_dps = **0.825** (counting multishot twice would give 4800 and 0.6666666666666666).
Melee attack, 500 per hit, 2 hits/s: X_dps = **1000**, M_dps = 1.

**TV-05-37 Exact shield break.**
Corpus target, SH = 1600 remaining, instance IMP 400, PUN 800 (D^SH = 400 + 1200 = 1600 = SH): a break, a_sh = 1,
overflow 0, SH = 0, gate open until τ + 0.1, "SH broken" emitted (Magnetic break proc if n_MAG > 0).

**TV-05-38 Ability-debuff immunity.**
Molecular Prime (+1.00, multiplicative) on an Eidolon (`ability_debuff_immune = all`): V = **1**; on a Lancer: V_HP = 2.
Terrify (0.60) plus one Corrosive Projection (0.18) on an Eidolon with A0 = A: Σ_add = **0.18** (only the aura), A_eff =
0.82·A; on a Lancer Σ_add = 0.78.

**TV-05-39 T-28 off.**
Juggernaut melee hit, health damage after armor 1000: T-28 on **50**, off **1000**. Archon Slash tick 1000: on **400**,
off **1000**. Hemocyte instance X_att = 12000, ms 2, HP_max 2,472,207: on 10048.917301530684, off **12000**. Hyekka
Heat (type_res) is unchanged by T-28: 20 from W = HEA 100.

**TV-05-40 Shields that weapons cannot damage.**
Eidolon Teralyst (`shield_weapon_immune = all`): every weapon instance has D^SH = 0; the `shield` view shows "immune";
the layered pass starts with SH = 0 by default (flag `shield_phase_skipped`). Profit-Taker Orb with
`pt_shield_type = MAG`: only the Magnetic part damages the shield (Corpus φ_MAG 1.5); with the default (none): 0.

**TV-05-41 Hemocyte.**
Status immunity ALL: proc weights are empty, no procs, no DoT, no Viral or Corrosive; Corrosive Projection and ability
strips still reduce its armor (L89). `body` → head weak spot (P 1, h 2); the outer skin (P 0) is never selected.

## Review notes

Two verifier reviews were applied in this revision. Issues rejected or applied only in part, one line each:

- Prosecutor "already Eximus, toggle disabled": not applied; the module gives the record `EximusDefault` and the wiki gadget applies E-5.5 to its 1,500 base, and no source says the 10× already contains the Eximus increase (7.3, OQ-533).
- Remove arch-guns from `non_melee`: not applied; the U44.0.1 patch note "Damage from non-melee sources is reduced by 90%" (`W:Eximus` L262) includes them, so the class keeps them with that citation.
- Test vector "SP shield flat bonus combined with Eximus": not possible; the only records with `SteelPathShieldBonus` are Commandeered Primes, which are bosses and cannot be Eximus (TV-05-20 and TV-05-19 cover the two parts).
- "Babau's Archimedea bonus is absent from the module": inaccurate; the module models it as a separate record with Health 9,000 (`M:enemies/techrot` L80-104), so it is recorded as a data gap (OQ-532) instead of a hand-authored bonus that could double-count.
- "04 holds none of the strip values": outdated; `04` §5.13 now carries Shattering Impact, Metal Auger, Polarize and Sharpened Claws, so only Shield Disruption stays in this document (E-5.16).
- New DR fields for Isolator Bursa (frontal) and Amalgam Machinist (conditional): not added; the Bursa's 60 % is its Multis part "Front: 0.4x" (4.3) and the Machinist's own page puts the 50 % on its shields (`innate_dr[SH]`).
- Multis counts "801 'Head: 3.0x'" and "553 '?'": these include the pre-U36 archive and commented-out `--Multis = { "?" }` lines, which are not data; the live modules give 447 and 303 records without Multis.
- T-28 option (b) (keep flat DR when off): not chosen; option (a) follows 00 §6.1 and D-047, and OQ-522 asks the manager to drop "(inspiration behaviour)".
