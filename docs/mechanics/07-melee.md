Status: draft (spec)

# Damage-calculation spec, part 07: melee

This document defines how a melee attack turns its stats into a timed sequence of hits with per-hit multipliers,
forced procs and combo state, and every damage rule that exists only for melee. Both evaluators of
`00-overview.md` section 4 consume the same deterministic melee schedule defined here (overview 4.1: "the schedule
never depends on random draws"). It follows the template of overview 0.4.

Related documents: [00-overview.md](00-overview.md) (notation, master formulas E-1 to E-14, register, toggles T-09,
T-10, T-11), [01-damage-composition.md](01-damage-composition.md) (base vector, stance physical bonus inside E-1.9,
Incarnon flat, base bucket A including the heavy-only members), [02-crit-status-multishot.md](02-crit-status-multishot.md)
(crit chance buckets, the heavy doubling hook, proc rolls and proc-name mapping), [03-status-effects.md](03-status-effects.md)
(Lifted and Knockdown durations, DoT ticks), [04-conditional-bonuses-and-buffs.md](04-conditional-bonuses-and-buffs.md)
(CO family and the stance formula E-4.6, `G`/`G_F` membership, melee-projectile buckets, effect values, stealth E-4.16,
finishers E-4.17, Melee Duplicate E-4.15), [05-enemies.md](05-enemies.md), [06-ranged-firing-model.md](06-ranged-firing-model.md)
(falloff Φ(d), the shared rules for timing stats under conditional buffs, 5.9), `08-evaluation-and-outputs.md` (how the
schedule is walked), `09-build-rules.md` (stance slot legality, Incarnon tier choice).

Contents:

- [1. Scope and stages](#1-scope-and-stages)
- [2. Inputs](#2-inputs)
- [3. Outputs and symbols added](#3-outputs-and-symbols-added)
- [4. Data model](#4-data-model)
- [5. Modded timing stats](#5-modded-timing-stats)
- [6. The combo counter](#6-the-combo-counter)
- [7. The melee damage instance](#7-the-melee-damage-instance)
- [8. The melee schedule generator](#8-the-melee-schedule-generator)
- [9. Weapon families](#9-weapon-families)
- [10. Worked examples](#10-worked-examples)
- [11. Edge cases](#11-edge-cases)
- [12. Inspiration differences](#12-inspiration-differences)
- [13. Open questions](#13-open-questions)
- [14. Test vectors](#14-test-vectors)

Citation keys are those of overview 0.2. Wiki pages are cited as `W:<Page>` with line numbers of the cached raw copy
in `research/external/cache/wiki/` where given. `M:Stances` and `M:Modular` are the renderer modules
`research/external/cache/wiki-modules/raw/{Stances,Modular}.lua`; `D:Stances` is `Module:Stances/data` and `D:<weapon>`
a record of `Module:Weapons/data/melee` or `/modular`, both as converted by mlua into `research/external/cache/poc/out/`
(module revisions of 2026-10-02 and 2026-10-22). `PE:<weapon>` is a record of the DE Public Export
`research/external/cache/public-export/manifests/ExportWeapons_en.json` (build label 2026.10.28). Counts and medians
quoted from these files were computed with python3 in this session; the scripts are not part of the repository.

---

## 1. Scope and stages

**Owns** (overview 7.1, row `07`): the melee schedule generator; attack speed; stance combos (durations, hit
multipliers, physical bonuses, forced procs, CO ratio data) and stanceless defaults; combo counter (gain, tiers, caps,
duration, decay, Initial Combo, regeneration); heavy attacks (damage, heavy and combo multipliers, Incarnon-flat
exception, crit doubling, heavy attack efficiency, wind-up, Tennokai); slams and heavy slams; slide attacks; glaives;
gunblades; whips; zaw assembly; exalted melee and the ability combo (T-11); melee-only multipliers in `G`. By
delegation from `04` (section 7.2 there): the finisher class multiplier table and the finisher schedule.

**Not owned**: values of arcanes, mods and buffs (`04`; this document names the bucket hooks and the gates), the
stealth formula (`04` E-4.16; this document only marks which instances are stealth-eligible melee hits), the CO formula
(`04` E-4.3, E-4.6; this document owns the per-input ratio data r_in), proc-name mapping and rolls (`02`, `03`), status
durations including the Lifted duration rule (`03` 8.9; this document supplies the combo multiplier of the hit),
falloff Φ(d) (`06`), the evaluation algorithms (`08`), stance and Incarnon legality (`09`).

**Stages.** No stage of overview 3.1 is owned outright. This document supplies: the melee hit multiplier S_j and the
heavy-attack base vector to ST-1 (section 7.2); the stance physical bonus `stance_phys[t]` to ST-2 (`01` E-1.9); the
melee members of `G` (combo multiplier, Slam Damage, ability combo) to ST-5; the wave falloff factor to `G_F` (ST-4);
the instance flags (heavy, slide, Tennokai, slam, radial) that gate buckets in `01`, `02`, `04`; the forced procs of
stance and heavy hits to ST-10 (`02` rolls them); zaw assembled stats to ST-1 and E-4 to E-8; and the schedule consumed
by `08`.

---

## 2. Inputs

| Input | Symbol | Owner | Notes |
|---|---|---|---|
| Melee weapon record: attacks (Normal Attack, Slam Attack, Heavy Slam Attack, throws, explosions, ranged attacks, weapon-unique attacks), `Class`, `HeavyAttack`, `SlideAttack`, `WindUp`, `ComboDur`, `FollowThrough`, `Family`, `Traits` | – | data pipeline (`DS:wiki` §3) | `D:<weapon>` |
| Public Export melee fields `fireRate`, `heavyAttackDamage`, `totalDamage`, `windUp`, `comboDuration` | – | data pipeline (`DS:pe`) | cross-checks and the base attack speed (5.1) |
| `Module:Stances/data` after `addSharedCombos` | – | data pipeline | section 4.1 |
| Zaw parts (`Module:Modular/data` `Zaw.Strike/Grip/Link`) | – | data pipeline | section 9.9 |
| Equipped stance mod and selected stance combo | – | build (`09` legality) | default combo: Neutral |
| Base vector parts Bw_t, V_t, X_t, B', B_ref of each attack | `base_weapon[t]`, `progenitor[t]`, `incarnon_flat[t]` | `01` | X: Incarnon flat |
| Bucket sums: attack speed ΣAS, wind-up speed ΣWU, combo duration (percent Σpct and flat Σflat), Initial Combo, heavy attack efficiency, additional combo count chance, Relentless Combination chance, Slam Damage ΣSlam, Tennokai chance modifiers | – | `04` (values, condition class T-02) | timing stats follow `06` 5.9 |
| T-01 `t_win`, T-09 heavy and start multiplier, T-10 light-attack combo multiplier, T-11 ability combo, T-20 Strength, T-25 zaw grip and link, T-27 distance | – | overview 6.1 | |
| Melee plan of the attack (section 8.1) | – | scenario (proposed toggle, OQ-722) | default per attack kind |

---

## 3. Outputs and symbols added

Overview symbols used with their overview meaning: `attack_speed` AS, `combo_count` c, `combo_mult` M(c),
`stance_hit_mult` S_j, `stance_phys[t]` S^phys_t, `t_win`, `distance` d. `04`'s `co_stance_ratio` r_in is filled from
the data defined here.

**Symbols added** (overview 0.3: domain prefixes `melee_`, `combo_`, `stance_`, `heavy_`, `slam_`, `ability_`, `wave_`,
`zaw_`, `glaive_`, `tennokai_`):

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `melee_fr_base` | FR_w | 1/s | base attack speed of the weapon, snapped to k/60 within tolerance (E-7.1) |
| `melee_as_sum` | ΣAS | bonus | attack-speed bucket (additive) |
| `stance_dur` | D_C | s | duration of combo C at attack speed 1.0 (data or fallback, E-7.3) |
| `stance_strikes` | N_C | count | number of strikes of one loop of combo C (Σ Hits over all entries) |
| `melee_strike_dt` | Δ | s | time between two strikes of a light loop, `D_C / (AS·N_C)` |
| `heavy_mult` | H_j | factor | hit multiplier of strike j of a synthesized heavy attack (E-7.40) |
| `heavy_index` | h | 1 or 2 | first or second heavy attack of the alternating pair |
| `heavy_windup_base`, `heavy_windup_sum`, `heavy_windup` | W0, ΣWU, W | s, bonus, s | wind-up (E-7.4) |
| `heavy_swing` | T_sw | s | heavy swing time at AS 1.0 (data or fallback, E-7.5) |
| `heavy_weapon_factor` | κ_w | factor | per-weapon heavy multiplier factor (E-7.41), 1 by default |
| `combo_tier_size`, `combo_cap`, `combo_bonus_cap` | k_tier, M_cap, M_bcap | count, factor, factor | 20, 12, 12 by default (E-7.10) |
| `combo_mult_bonus` | M_b | factor | combo multiplier read by combo-scaled bonuses (E-7.11) |
| `combo_dur_base`, `combo_dur` | D0_cb, D_cb | s | combo duration before and after mods (E-7.12) |
| `combo_initial` | IC | count | Initial Combo (E-7.15) |
| `combo_eff` | η | fraction | heavy attack efficiency, clamped to [0, 0.9] (E-7.16) |
| `combo_regen_rate` | ρ_IC | count/s | Initial Combo regeneration, 40 |
| `combo_gain` | g | count | combo points a hit adds before the additional-chance roll (E-7.13) |
| `combo_acc` | p_acc | fraction | additional combo count chance (additive bucket) |
| `combo_rc` | p_RC | fraction | Relentless Combination chance per melee Slash proc and per tick |
| `combo_decay` | δ_PS | count | Power Spike decay per expiry (20 / 15 / 10 / 5) |
| `combo_expected` | c̄ | count | expected-path combo count stored in the schedule (6.9) |
| `slam_dmg_sum` | ΣSlam | bonus | Slam Damage bucket; factor `M_slam = 1 + ΣSlam` |
| `ability_combo` | M_ab | factor | ability combo multiplier of the four ability-combo exalted weapons (E-7.44) |
| `wave_falloff` | φ_w | factor | Exalted Blade wave falloff, `max(0, 1 − d/70)` (E-7.43) |
| `tennokai_chance`, `tennokai_every` | p_tk, n_tk | fraction, count | Tennokai window chance and the deterministic input spacing (E-7.17) |
| `tennokai_window` | T_tk | s | Tennokai window, 2 s (4 s with Opportunity's Reach) |
| `heavy_animosity_stacks` | n_anim | count | Melee Animosity stacks carried by a heavy (normal inputs since the last connecting heavy, ≤ 10) |
| `glaive_throw_ct` | CT_g | s | modded charge time of a charged glaive or thrown-hammer throw (E-7.6) |
| `zaw_total` | B_zaw | HP | assembled zaw base damage (E-7.42) |
| `melee_plan` | P | enum | `light_loop`, `heavy_loop`, `build_spend`, `tennokai`, `slide_loop`, `slam_loop`, `heavy_slam_loop`, `glaive_cycle` (8.1) |

Outputs: the melee schedule (8.2) with per-hit S_j, S^phys_t, forced procs, flags, c̄ and M; the synthesized heavy
and slide attacks (9.1, 9.3); zaw assembled attacks (9.9); the per-hit G factors of section 7.4; steady-state
quantities for `dps_burst` (8.6).

---

## 4. Data model

### 4.1 Stance records and their normalisation

`Module:Stances/data` has 135 entries (`F:melee` §2.2; recounted: 64 PvE entries including the `__Legend` table,
28 stanceless class tables with `StancelessStance = true`, 22 `UniqueToWeapon` tables, 21 `ConclaveOnly` tables). The
module returns `addSharedCombos(StanceData)` (`D:Stances` raw line 11625), which copies the class table's `Heavy`,
`Slide`, `Aerial`, `Wall`, `Finisher` and `Slam` combos (`SHARED_COMBOS`, raw line 4) into every table that lacks them
(Ravenous Wraith excluded) and sets `FromWeapon = true` on the copied object, which is the class table's own object.
The data pipeline MUST evaluate the module in Lua (mlua, `DS:wiki` §9); a literal-table parser returns the wrong table
(`DS:wiki` §9 PoC 2).

Each combo has `Name`, optional `Duration` (s at attack speed 1.0: "Length is the base duration of a particular combo
when melee attack speed is at 1.0", `W:Stance` line 690) and `Attacks[]`. One element of `Attacks[]` is one **input**
(one key press); it holds aligned arrays, one element per **entry**:

| Field | Meaning | Default when absent |
|---|---|---|
| `Dmg[e]` | hit multiplier in percent (100 = weapon damage) | required |
| `Hits[e]` | number of **strikes** of this entry | 1 |
| `Procs[e]` | forced procs: a name, a list of names, or `""` | none |
| `Types[e]` | `""`, `360`, `Sweep`, `Thrust` (melee strike), `Ranged` (projectile or wave), `Slam` (radial slam part) | `""` |
| `SlashMultiplier[e]`, `ImpactMultiplier[e]`, `PunctureMultiplier[e]` | stance physical factor (2 = +100 %) | 1 |
| `StatusMultiplier[e]` | undocumented (6 uses) | 1 |
| `Shape`, `Note` | display only | – |

```text
E-7.0  Normalisation of one input u with n = |Dmg_u| entries (data pipeline; every step writes a data-report line)
  1. Missing array → defaults above.  Array shorter than n → pad with defaults.
  2. Procs given as a map with numeric-string keys (a Lua table with holes) → list; missing indices = no proc.
  3. Procs longer than n → the extra names are appended to entry n's proc list.
     Any other array longer than n → extra elements dropped.
  4. Proc list of an entry: "" removed; a single string becomes a one-element list.
  5. S^phys_t of an entry = (factor_t − 1) for t ∈ {SLA, IMP, PUN}.
  6. Every entry expands into Hits[e] strikes with identical fields, in entry order; the strike list of a combo is the
     concatenation over inputs. N_C = total strike count (all types, Slam and Ranged included).
```

Data cases handled by E-7.0 (cached module, recounted this session): Blind Justice Forward Block input 3 `Procs` with a
hole (the unquoted `Knockdown` evaluates to nil; `03` 4.2 maps it to "no proc"); Bullet Dance Neutral input 5 physical
arrays of length 2 for 1 entry; Crimson Dervish, Gaia's Tragedy, Rending Crane, Seismic Palm, Twirling Spire short
arrays; Galeforce Dawn Forward Block input 2 `Types` of length 1 for 4 entries; Hysteria Block, Forward and Neutral
and Ravenous Wraith Neutral `Procs` of length 2 for 1 entry (step 3 gives `[Slash, Stagger]` and `[Bleed, Big Stagger]`
on that entry); Galeforce Dawn and Heavy Scythe Slide empty `Procs`. Crimson Dervish `Block` is byte-identical to its
`Neutral` (`F:melee` §2.2): kept as data. Dmg values not multiple of 50 exist (40, 125, 166, 180, 185, 215, 220, 225,
555): kept exact. `StatusMultiplier` (Blind Justice, Decisive Judgement, Tranquil Cleave; values 1.25, 1.5, 2) is
stored and not applied (Decision, UNVERIFIED, OQ-714); `02` EC-02-53 waits on this document.

Proc names map to status types by `03` table 4.2 (with `02` 4.7.2). Names in stance data that `03` 4.2 does not list:
`Finisher` (Caustacyst heavy projectile; opens the target to a finisher) and `Disarmed` are crowd control only
(proposal to `03`, OQ-715).

### 4.2 Resolving the combo tables of a weapon

```text
E-7.0b Combo table of combo type K ∈ {Neutral, Forward, Block, Forward Block, Slide, Aerial, Wall, Finisher, Slam,
                                       Heavy, Heavy Slam} for weapon w with equipped stance mod S (or none)
  U = the UniqueToWeapon table whose key equals w's name, else w's data Family; none otherwise
  if U has K and U.K has no FromWeapon flag:                    use U.K   (weapon-unique override)
  elif S is equipped, S has K and S.K has no FromWeapon flag:   use S.K   (the four stance-specific combos, and shared
                                                                          combos the stance defines itself)
  else:                                                         use T.K,  T = the stanceless table of w's Class
  Conclave-only stances are never used.  Heavy Slam exists only as a unique override (Sampotes, Tenet Exec).
```

Sources: `W:Stance` lines 20-77 (four stance-specific combos; Heavy, Slide, Aerial, Wall, Ground Finisher shared by the
weapon class), `W:Melee` line 270 ("Aerial attacks, wall attacks, heavy attacks, and slam attacks derive their stats from
the weapon's class, and are unchanged by stance mods"), `W:Stance` lines 678-684 (weapons with unique combos override
the stance: Sigma & Octantis and Cobra & Crane aerial, Nepheri neutral, Korumm and Verdilac block, Argo & Vel heavy),
the `FromWeapon` semantics of `addSharedCombos` (raw lines 10-27). Family matching covers Quassus Prime → Quassus,
Coda Caustacyst → Caustacyst, Tatsu Prime → Tatsu, Cobra & Crane Prime → Cobra & Crane (`D:<weapon>` `Family`); name
matching covers Telos Boltace (Family `Boltace`).

Stances that define shared combos themselves (no `FromWeapon`): the exalted stances Exalted Blade (all but Heavy),
Hysteria, Primal Fury, Serene Storm and Ravenous Wraith (their classes have no stanceless table), and the Slide of
Blind Justice, Decisive Judgement, Tranquil Cleave and Four Riders (plus the Blind Justice Wall); the data wins over the
"shared by class" statement for these (Decision).

Stance switching mid-combo is seamless since U26 (`W:Stance` line 16), so every combo is evaluated as an independent
loop that restarts at its first input (Decision).

### 4.3 Melee weapon fields used

| Field | Use | Notes |
|---|---|---|
| `Normal Attack` | the attack of light, slide, aerial, wall and heavy hits (its damage map, CC0, CDM0, SC0) | heavies and slides have no separate crit or status data (W:Melee §Heavy Attack Damage) |
| `FireRate` of `Normal Attack` | FR_src for E-7.1 when the weapon does not join to PE | |
| `Slam Attack`, `Heavy Slam Attack` | the two slam attacks: damage already 2× and 3× weapon damage, own SC, `ForcedProcs`, `Falloff`, `Range` | Nikana Prime 396 = 2 × 198, 594 = 3 × 198 (`D:Nikana Prime`) |
| `WindUp` | W0 | present on all 250 records |
| `ComboDur` | D0_cb | present on 249 of 250 records; absent on Xoris (infinite, 6.4) |
| `HeavyAttack`, `SlideAttack` | cross-checks only (9.1, 9.3) | semantics inconsistent across classes |
| `FollowThrough`, `MeleeRange`, `SweepRadius`, `BlockAngle` | multi-target and display only | |
| throws, explosions, `Ranged Attack`, weapon-unique attacks | sections 9.4 to 9.7 | |

### 4.4 Hand-authored tables

| Table | Content | Source |
|---|---|---|
| 9-A class heavy table | per class: first and second heavy multiplier, explicit per-hit breakdown where given, typical wind-up | `W:Melee` §Heavy Attack Damage (cached lines 585-637) |
| 9-B weapon heavy factors κ_w | computed by the rule of E-7.41; current members listed there | `D:<weapon>`, `PE:<weapon>` |
| 9-C melee attack triggers | which input, strike or heavy spawns which data attack, count, scaling, flags | section 9.7 |
| 9-D stance CO ratios r_in | `W:Condition_Overload_(Mechanic)` §Stances, 63 rows | section 7.6 |
| 9-E finisher class multipliers | `W:Finisher` §All Finishers Multiplier Comparison | section 9.11 |
| combo specials | k_tier, M_cap, M_bcap, D0_cb overrides, innate Initial Combo | section 6 |

### 4.5 Durations and fallbacks

`Duration` coverage of the evaluated module (`F:melee` §2.2, recounted): the stance-specific combos of the 63 PvE
stances (exalted stances included) 250 of 251 have it (missing: Gleaming Talon Block; Wise Razor has no Block combo);
the shared combos of the 28 stanceless tables: Neutral 11 of 27 (Bayonet has no Neutral), Slide 21 of 28, Heavy 2 of 28; distinct heavy combos with `Duration`: 6 (Dual Nikanas 4.4, Dual Swords
3.3, Edun 3.3, Hysteria 2, Sampotes 4.2, Syam 3.6). Stanceless classes with no Neutral duration: Assault Saw, Claws,
Dagger, Dual Daggers, Fist, Glaive, Hammer, Machete, Nikana, Polearm, Rapier, Sparring, Staff, Sword, Tonfa, Warfan
(and Bayonet, which has no Neutral combo). The wiki states the lengths are "approximate and are only accurate within a
few tenths of a second" (`M:Stances` line 557). Fallbacks: E-7.3 (loops) and E-7.5 (heavy swing); overview OQ-021.

---

## 5. Modded timing stats

Timing stats under conditional buffs follow `06` 5.9 exactly (all_max and none: constant; `simulated`: deterministic
own-event triggers switch with the schedule's events, random and external triggers use the class-(c) uptime fraction
of their maximum). Arcane Strike (15 % on hit) and Berserker Fury (on melee kill) are therefore constant fractions under
`simulated`.

### 5.1 Base attack speed

```text
E-7.1  k = round(60 · FR_src)
       FR_w = k / 60   if |FR_src − k/60| ≤ tol,   else FR_w = FR_src
       FR_src, tol = PE fireRate, 1e-6          when the weapon joins to the Public Export
                   = D:<weapon> Normal Attack FireRate, 0.005   otherwise (the wiki rounds to 2-3 decimals)
       zaw: FR_src = grip.Speed + strike.Speed + link.Speed (9.9), tol 0.005
```

"All base weapon Attack Speeds are in fractions of X / 60" (`W:Attack_Speed` line 32). Check (this session): the PE
`fireRate` of 221 of the 222 melee weapons that join to the wiki module is within 1 ulp (f32) of k/60; Spinnerex is
1.36 in the PE (k/60 would be 1.36667), so the PE value is kept. `round(60·FR)` of the wiki value equals the PE k for
every weapon except Ripkas (wiki 0.883 → 53, PE 0.83333 → 50; the PE wins). Nikana Prime
1.08 → 65/60; Skana 0.833 → 50/60. Whether the game's internal value is the exact quotient or its f32 neighbour is not
documented; the difference is below 1e-6 relative (UNVERIFIED, OQ-713).

### 5.2 Attack speed and light-loop timing

```text
E-7.2  AS = max(0.05, FR_w · (1 + ΣAS))                    ΣAS additive over every source (values in 04)
E-7.3  T_loop(C) = D_C / AS ;   Δ = T_loop / N_C
       D_C = Duration of combo C                                                    (data)
           | Duration of another combo of the same table with an identical strike list (data)
           | N_C · τ_class                                                           (fallback, UNVERIFIED)
       τ_class = median over every PvE stance-specific combo with Duration of the weapon's class (Neutral, Forward,
                 Block, Forward Block of non-stanceless, non-unique, non-Conclave tables) of Duration / N_combo;
                 classes with no such combo: the median over all classes (0.40 s per strike in the current data)
```

`W:Attack_Speed` lines 13-16: `Modified Animation Length = Base Attack Animation Length / [Base Weapon Attack Speed ×
(1 + Attack Speed Bonuses)]`; the bonuses are additive with each other (`F:melee` §1.1: Arcane Strike "additive to mods
like Fury", Excalibur passive "additive to mods such as Pressure Point and Fury"). The 0.05 floor mirrors `06` D-603
(Decision; Spoiled Strike −0.20 is the only negative source in the catalogue). Attack speed speeds stance combos,
slide, aerial and wall attacks and the heavy **swing**, not the heavy wind-up and not glaive or Wolf Sledge charged
throws (`W:Attack_Speed` line 30; `W:Melee` line 705: "Increasing melee attack speed does not reduce the wind-up time;
rather, it reduces the interval between Heavy Attacks"). Finisher animations use only attack-speed mods, capped at 33 %
faster at +50 % (`W:Attack_Speed` line 30; finisher mode only, 9.11).

Current τ_class values (s per strike at AS 1.0; this session, from the cached module): Assault Saw 0.82875, Dagger
0.355, Claws 0.348571, Dual Daggers 0.30875, Fist 0.340833, Glaive 0.425, Hammer 0.54125, Machete 0.570833, Nikana
0.4625, Polearm 0.24375, Rapier 0.428571, Sparring 0.335833, Staff 0.5125, Sword 0.45, Tonfa 0.257323, Warfan
0.275538; global median 0.40 over 250 combos. The pipeline recomputes them from the module (they change when the wiki
adds data). Fallback durations are marked in the result so the UI can flag them.

Strike timing inside a loop (Decision, UNVERIFIED, OQ-703): strike k of a light phase that starts at t0 lands at
`t0 + k·Δ` (k = 0, 1, …; first strike at the phase start), strikes are evenly spaced over the loop regardless of inputs
and entries, and the loop repeats without a gap. The wiki gives only the total length per combo. With
attack-speed changes inside a phase (`simulated` deterministic triggers) Δ is evaluated at each strike from AS at that
strike.

### 5.3 Heavy wind-up and swing

```text
E-7.4  W = W0 / (1 + ΣWU)                      ΣWU additive (Killing Blow 0.60, Amalgam Organ Shatter 0.60, Melee
                                               Elementalist 0.60, Swift Momentum aura 0.30, Kullervo passive 1.00,
                                               riven splice; values in 04)
       Tennokai heavy:  W_tk = W               (Decision, UNVERIFIED, OQ-708)
       heavy slams: no wind-up
E-7.5  swing time of one heavy h:   T_sw / AS
       T_sw = Duration of the weapon's Heavy combo / number of heavy attacks in that combo   (data)
            | τ_heavy = median of that quotient over every heavy combo with Duration (1.95 s today)  (fallback)
```

Sources: `W:Melee` lines 703-709 (wind-up "Time a Heavy Attack must charge up before it activates"; "There is no cap
to Wind Up … due to diminishing returns"; "Heavy Slams do not have any wind up"); the division form is the only form
consistent with "no cap" plus diminishing returns (`F:melee` §4.3), UNVERIFIED (OQ-707). Example: Glaive Prime W0 1.2 s
with Killing Blow → 0.75 s. `W:Melee` line 723: "The Wind Up Speed of Tennokai attacks is not affected by Wind Up Speed
bonuses from other sources" and Tennokai "increases its Wind Up Speed" (line 722) without a value: the Decision keeps the
normal modded wind-up. Hysteria: wind-up mods on the regular melee weapon also apply to Valkyr Talons and stack with
those on the talons (`W:Hysteria` lines 40-41).

T_sw fallback: the six heavy combos with `Duration` give per-heavy quotients 2.2 (Dual Nikanas), 1.65 (Dual Swords), 3.3
(Edun, one attack), 1.0 (Hysteria), 2.1 (Sampotes), 1.8 (Syam); median 1.95. The Stance page defines Duration as a
length that scales with attack speed, so it excludes the wind-up, which does not (Decision, UNVERIFIED, OQ-701). The
first damage frame of a heavy is taken at the end of the wind-up (Decision, OQ-701).

### 5.4 Throw and charge times

```text
E-7.6  charged glaive or thrown-hammer throw:  CT_g = ChargeTime_attack / (1 + ΣWU)       (data ChargeTime, e.g. 1.192 s)
       quick throw: cycle 1 / FireRate_attack (data FireRate of the "Throw" attack; not scaled by AS)
```

Glaive throws are "governed by Heavy Attack Wind Up Speed (1.2 s base since U30.5), not attack speed" (`F:melee` §11;
`W:Attack_Speed` line 30; `W:Glaive` line 127). The use of the attack's data `ChargeTime` and `FireRate` as cycle times
is a Decision (UNVERIFIED, OQ-711): `D:Glaive Prime` gives Charged Throw ChargeTime 1.192 and FireRate 0.833 (= 1/1.2),
Throw FireRate 1.25 (the weapon's attack speed).

---

## 6. The combo counter

### 6.1 State

The combo state of one melee weapon (exalted weapons keep their own, 9.8) is `(c, t_x, regen)`: the count c ≥ 0, the
expiry time t_x of the combo timer, and the regeneration segment (start time, start value) while c is below IC after a
heavy. The fresh state at t = 0 is `c = IC` (overview 4.1) unless T-09 sets a start multiplier (8.5); the timer is not
running until the first gain or heavy.

### 6.2 Tiers and caps

```text
E-7.10 M(c) = clamp( floor(c / k_tier) + 1, 1, M_cap )
       default k_tier = 20, M_cap = 12;  Venka Prime: M_cap = 13;  Dex Nikana: k_tier = 11, M_cap = 11
E-7.11 M_b = min(M(c), M_bcap)     M_bcap = 12 (11 for Dex Nikana)          combo-scaled bonuses (6.8)
```

Sources: `W:Melee_Combo` line 59 and its table ("2.0x after 20 consecutive hits … increased by 1.0x every consecutive
20 hits until 220 … cap at 12.0x"), lines 93-94 (Venka Prime 13× at 240; Dex Nikana 11 hits per tier, 11× at 110),
`W:Dex_Nikana` line 9, `W:Venka_Prime` line 44 ("mods that scale with combo multiplier, such as Blood Rush, Gladiator
mods or Weeping Wounds, will stop scaling at the usual 12.0x"). A heavy at 0 combo is allowed and gets M = 1 (`W:Melee`
line 705 "Performing a Heavy Attack is possible when the combo counter is zero"). The count itself is not capped
(Decision, UNVERIFIED, OQ-705): heavy attack efficiency is defined on "current combo" (`W:Melee_Combo` line 192) and
Melee Crescendo's Initial Combo is unbounded (`F:melee` §3.4).

**Normal attacks are not multiplied by M(c)** (`W:Melee_Combo` line 13). M multiplies heavy attacks, heavy slams,
glaive heavy explosions and heavy projectiles (7.4); M_b feeds the combo-scaled bonuses; M is the Lifted duration
argument of `03` 8.9.

### 6.3 Gain

```text
E-7.13 Gain of one damage instance i on the single target (applied after the instance, 6.1 order rule):
  stance strike (light, slide, aerial, wall; Slam-typed entries excluded, 7.1):  g = floor(Dmg_e / 100 + 0.5)
  gunblade shot (one Ranged stance strike):     g = 1 per pellet that hits                 (Decision, UNVERIFIED)
  Rauta pellet:                                  g = 2 per pellet, at most 28 per shot
  Exalted Blade or Desert Wind light wave:       g of its Ranged entry by the stance rule   (Decision, UNVERIFIED)
  thrown glaive or thrown-hammer direct hit:     g = 1                                      (Decision, UNVERIFIED)
  normal slam (radial, per enemy):               g = 1
  heavy attack hit, heavy slam, glaive explosion (heavy or Volatile), heavy projectile, heavy wave,
  Melee Duplicate copy, extra hit, DoT tick:     g = 0
  extra point:  every instance with g > 0 adds X = floor(p_acc) + Bernoulli(frac(p_acc)) more points
  Relentless Combination: each Slash proc applied by a melee instance, and each tick of such a proc, adds
                          Bernoulli(p_RC) points, each such point also rolling X
  riven curse "Chance to Gain Combo Count" −x (negative only): every gain event above happens with probability
                          1 − x (its g and X together; Decision, UNVERIFIED)
  no gain at all when D_cb is undefined (E-7.12, raw duration ≤ 0)
```

Sources: `W:Melee_Combo` line 26 ("100 % stance damage multiplier = 1 point … a 300 % damage multiplier adds 3 combo
points per hit. Only successful strikes against an enemy will award combo points"), line 36 (Rauta 2 per pellet, 28
max), lines 38-52 (Additional Combo Count Chance "starts at +0 % and benefits additively"; sources Enduring Strike,
Guardian Derision, Quickening, Relentless Combination, True Punishment, Exodia Triumph, Exodia Valor); `W:Melee` line 63
(slams: "Melee combo is gained for each enemy struck") and line 705 ("connecting with a Heavy Attack does not add to the
combo counter"); `W:Volatile_Rebound` lines 28-29 (explosions add no combo; "only the thrown weapon's direct hits on
enemies count"), which contradicts `W:Relentless_Combination` line 29 ("the combo counter does not interact with
Glaives"); the Volatile pages are explicit and win (Decision). `W:True_Punishment` line 29 (the Redeemer "shoots 10
pellets with each shot, resulting in +10 hit combo"). `W:Relentless_Combination` lines 24-29 (+1 per Slash proc and per
Bleed tick at max rank, melee only, exalted weapons and Exalted Blade waves included). `W:Extra_Hit` (extra hits give no
melee combo; Melee Duplicate is listed there, so its copy gains nothing, Decision).

Rounding: `M:Stances` `calcTotalComboGain` (lines 335-349) rounds each entry half up, so 50 % gives 1 point and 250 %
gives 3; this is the wiki's tabulation rule, UNVERIFIED against the game (OQ-704). The extra-point rule (one extra point
per gaining hit, not per point) follows "chance of additional combo count per hit" (`W:Quickening` line 2) and
`W:Melee_Combo` line 39 ("awarding an extra combo point either on hit"); `W:True_Punishment` line 27 ("pairs well with
stances that have high damage multipliers") hints at a per-point rule (UNVERIFIED, OQ-704). Above 100 % the
floor-plus-Bernoulli rule follows the game's convention for other chances above 1 (Decision, UNVERIFIED).

Gains apply per enemy struck; in the single-target default every gain event is counted once. Follow-through does not
scale combo gain (`F:melee` §10).

### 6.4 Duration, expiry and decay

```text
E-7.12 D_raw = D0_cb · (1 + Σpct) + Σflat
       D_cb  = undefined (no combo gain; heavies still spend)   if D_raw ≤ 0
             = max(0.1, D_raw)                                   otherwise
       D0_cb = data ComboDur (5; Guandao Prime 6, Pulmonars 9, Vitrica 10); Xoris: ∞
E-7.14 Timer: every gain event at τ (g + X > 0) and every heavy execution (hit or not) sets t_x = τ + D_cb.
       The timer does not run during a heavy wind-up (t_x is shifted by the wind-up length).
       At t_x without Power Spike:  c ← IC (regeneration state cleared)
       At t_x with Power Spike:     c ← max(IC, c − δ_PS);  t_x ← t_x + D_cb  (repeats while c > IC)
```

Sources: `W:Melee_Combo` lines 98-101 ("A zero or negative combo duration prevents increasing the combo counter";
"cannot be reduced to below 0.1 seconds"), lines 119-131 (Power Spike 20 / 15 / 10 / 5 per reset, 220 s or 44 ticks
from 220 at 5 s; Xoris infinite; Guandao Prime 6 s, Pulmonars 9 s, Vitrica 10 s), line 286 ("the combo duration timer
resets on heavy attack regardless of whether enemies are struck"); `W:Melee` line 705 ("While winding-up, the combo
counter decay is paused"); `W:Corrupt_Charge` line 29 and `W:True_Punishment` line 26 ("The Combo Duration penalty is
calculated from the melee weapon's base value. Meaning it always subtracts exactly 2.5 seconds"); `W:Body_Count`
line 28-29 (5 + 12 = 17 s; with Drifting Contact and Gladiator Rush 33 s). U35.1 fixed gaining combo with a negative
duration and heavies not spending it then; U35.5 fixed "if you had any Heavy Attack Efficiency and your Combo Duration
was zero, you would immediately get all of your Initial Combo back instead of it regenerating" (`W:Melee` lines 1151,
1161). The order "percent on base, then flat" is UNVERIFIED (OQ-706). The floor of the Power Spike decay at IC is a
Decision (IC is "a minimum value of combo points when idle", `W:Melee_Combo` line 135). Pause of the timer while
holstered (Tenet Livia, Tenet Grigori, Incarnon "Standoff" / "Abiding Hold" perks) and the Combo Killer reset are
outside the single-weapon schedule (EC-07-24).

### 6.5 Initial Combo and regeneration

```text
E-7.15 IC = max(0, Σ sources)                 no upper cap (Decision)
       sources: Corrupt Charge 30, Covert Lethality 16, Galvanized Reflex 20 per stack (4 stacks), Ready Steel aura 24,
       Melee Crescendo 6 per stack, innate Synoid Heliocor 20, Furax Wraith 20, Fragor Prime 30, Incarnon perks,
       riven Initial Combo (positive only); values and condition classes in 04
       Regeneration: while c < IC after a heavy:  c(t) = min(IC, c_after + ρ_IC·(t − t_exec)),  ρ_IC = 40 /s
                     plus any gains in between (gains add on top; regeneration stops when c ≥ IC)
```

Sources: `W:Melee_Combo` line 135 ("Initial Combo grants a minimum value of combo points when idle or after a combo
reset. Heavy attacks spend initial combo, which regenerates at a rate of 40 combo points per second"), lines 196-198
(3 s to rebuild 7× = 120 points, 5.5 s for 12× = 220; 1.5 s / 2.75 s at 50 %), `W:Corrupt_Charge` line 28 ("rapidly
regenerate to full within 2 seconds"), `F:melee` §3.4 (source list). Regeneration continues during a wind-up (Decision,
UNVERIFIED, OQ-707: only the decay is documented as paused).

### 6.6 Heavy consumption and heavy attack efficiency

```text
E-7.16 At the execution of a heavy (end of the wind-up), with c⁻ the count at that moment:
         M_h = M(c⁻)                                   used by every hit of this heavy
         c⁺  = c⁻            Tennokai heavy (6.7)
         c⁺  = η · c⁻        otherwise,   η = clamp(ΣHAE, 0, 0.9)
         t_x = t_exec + D_cb ;  if c⁺ < IC: start regeneration (E-7.15)
       Heavy slams spend the same way (Furax Incarnon form's +0.90 applies to heavy slams only).
       Glaive heavy explosions spend; Volatile explosions do not.
```

Sources: `W:Melee_Combo` line 192 (40 % efficiency at 160 retains 64; "capped at 90 %"), `W:Melee` line 557
(heavies "consume Melee Combo upon execution"), `W:Glaive` line 58 (the explosion "consumes the combo meter"),
`W:Volatile_Rebound` line 30, `W:Volatile_Quick_Return` line 30 (Volatile explosions do not consume), `F:melee` §4.2
(sources: Reflex Coil 0.60, Galvanized Reflex 0.50, Focus Energy / Focus Radon 0.40, riven, Kullervo passive 0.75,
Incarnon perks; additivity implied by the single cap, UNVERIFIED). Consecutive heavies each spend: the `W:Melee` tip
(line 954) that a second heavy chained right after the first keeps the combo describes a bug fixed in U33.6 ("Fixed
consecutive Heavy Attacks not resetting the Melee Combo multiplier", `W:Melee` line 1270).

### 6.7 Tennokai

```text
E-7.17 Enabled by any Tennokai mod on the weapon.
       Qualifying event: one per input that strikes the target (multi-strike inputs count once; every gunblade shot
       counts; Exalted Blade and Serene Storm waves count; heavy attacks do not; a heavy slide (second heavy started
       while sliding) does)
       Window chance per qualifying event: p_tk = 0.15 · (1.5 with Dreamer's Wrath) ; window T_tk = 2 s (4 s with
       Opportunity's Reach); Discipline's Merit: a window after every 4th qualifying event instead of the chance
       A heavy or heavy slam executed inside a window is a Tennokai heavy: no combo spent (E-7.16), Tennokai-only
       bonuses apply (Master's Edge, Truth's Flame, Condition's Perfection, Dreamer's Wrath; values in 04)
       Schedule (deterministic, 8.3): n_tk = 4 with Discipline's Merit, else ceil(1 / p_tk) (7, or 5 with Dreamer's Wrath)
```

Sources: `W:Melee` lines 719-742, `F:melee` §4.2 (Discipline's Merit "every 4 hits instead of random"; Dreamer's
Wrath ×1.5 → 22.5 %; Opportunity's Reach 4.0 s). The window timer is paused while a glaive is in flight (`W:Melee`
patch U37 line 1027). `ceil(1/p)` replaces the geometric waiting time by a fixed spacing so the schedule stays
deterministic (Decision; the MC evaluator uses the same spacing, so its Tennokai heavies are not random; OQ-709).

### 6.8 Combo-scaled bonuses

Blood Rush (`0.40·(M_b − 1)` relative crit chance), Weeping Wounds (`0.40·(M_b − 1)` relative status chance), the
Gladiator set (`0.10·N_glad·(M_b − 1)`), Secondary Outburst (consumes the combo on a weapon swap, external), Melee
Doughty through Weeping Wounds, Lifted duration, Iron Staff range: values and buckets in `02`, `03`, `04`. This document
fixes which M_b an instance uses:

```text
E-7.18 light, slide, aerial, wall instances, throws, waves:  M_b(c⁻), c⁻ = expected or live count just before the
                                                             instance (before its own gain)
       heavy hits, heavy slam, glaive heavy explosion, heavy projectiles and waves: M_b(c⁻) with c⁻ taken at the
                                                             heavy's execution, before consumption (Decision, UNVERIFIED, OQ-724)
       T-10 set to value V:  light-type instances use M_b = min(V, M_bcap) instead of the count
```

Thrown glaive attacks "are affected by the combo counter and so by Blood Rush" (`F:melee` §5). Whether a heavy's
Blood Rush reads the count before or after it is spent is not documented (`F:melee` open 4, `F:addmul` open 12); the
pre-consumption reading matches the inspiration's heavy path (`JS L8218-8224`), so it adds no register row.

### 6.9 Expected path and the Monte Carlo count

The schedule (section 8) stores the **expected-path** count c̄: every random gain (X, Relentless Combination) enters
with its expectation (p_acc per gaining instance, p_RC·(1 + p_acc) per Slash proc and per tick, with the expected Slash
proc rate of `02`; every gain times 1 − x under the riven curse). Plan decisions that depend on the count (the heavy trigger of `build_spend`, regeneration waits) use
c̄, so the schedule is deterministic (overview 4.1). The EV evaluator uses M(c̄) and M_b(c̄) (mean-field; the gap to
the exact expectation over the Poisson-binomial count is a documented EV approximation class for `08`). The MC
evaluator tracks the live count c along the same schedule with its own draws (mechanic ids `combo_acc`, `combo_rc`)
and uses M(c) and M_b(c) of the live count for damage and bonuses; the hit times do not change.

### 6.10 Ability combo counter (Blade Storm, Landslide, Shattered Lash, Whipclaw)

```text
E-7.44 M_ab = clamp(floor(c_ab / 20) + 1, 1, 12)     c_ab: +20 per recast within 5 s of the previous cast
       Damage of these four exalted weapons × M_ab (a G member).  T-11 sets M_ab directly.  Not for any other weapon.
```

Sources: `W:Exalted_Weapon` lines 352-358 (U38.5 Techrot Encore: "Recasting their respective Abilities within 5 seconds
of the previous cast will grant you 20 combo points which will increase the Combo Multiplier by 1x"; cap 12×; "Ability
Damage now scales 1:1 with this new Ability Combo Counter"; Blood Rush, Weeping Wounds, Body Count and Power Spike work
with it), `W:Melee_Combo` §Ability Combo Counter (duration also scaled by Ability Duration, "verification needed").
Recasting is an external event: without T-11 the engine uses M_ab = 1 (Decision). Combo-scaled bonuses of these weapons
read M_b = min(M_ab, 12). "Multiplier Ability Damage" abilities (1 : 0.25) are abilities, out of scope (overview 1.3).

---

## 7. The melee damage instance

### 7.1 Instance kinds

| Kind | Produced by | Damage source | Notes |
|---|---|---|---|
| `light` | a stance strike of a Neutral, Forward, Block or Forward Block loop (type `""`, `360`, `Sweep`, `Thrust`) | Normal Attack × S_j | |
| `slide`, `aerial`, `wall` | strikes of the shared Slide, Aerial, Wall combos | Normal Attack × S_j | slide-only bonuses gate on `slide` |
| `heavy` | strikes of a synthesized heavy attack (9.1) | Normal Attack, base E-7.20b | |
| `ranged` | a `Ranged` stance strike: gunblade shot, wave, or a mapped weapon-unique attack (9.5, 9.7, 9.8) | the mapped attack or Normal Attack × S_j | |
| `slam`, `heavy_slam` | the `Slam Attack` and `Heavy Slam Attack` data attacks | data (radial) | 9.2 |
| `throw`, `explosion` | glaive and thrown-weapon data attacks | data | 9.4 |
| `slam_part` | a `Slam`-typed stance entry | none in the single-target metric | below |

`Slam`-typed stance entries are the radial part of a combo slam. They are excluded from single-target damage, procs and
combo gain: `M:Stances` `calcTotalDmgMulti` (lines 276-294) excludes them because "a combo with a slam attack will only
produce one damage instance on direct hit of a target; the radial slam is not applied on direct hit", and since U35.5
"Slam attacks during combos deal flat damage amounts and do not scale with Mods" (`W:Melee` line 1104). They still occupy
their strike slot in the loop timing (E-7.3). An input made only of `Slam` entries (the stanceless Hammer Neutral,
Forward, Block and Forward Block inputs 1 and 3, Azothane Block, Galeforce Dawn Forward Block input 2) therefore deals
no modelled damage to the target; this follows the wiki rule but is UNVERIFIED (OQ-710).

### 7.2 Base vector of a melee instance

```text
E-7.20 Instance j of a light-type kind (light, slide, aerial, wall, light waves) with hit multiplier S_j = Dmg_j/100:
         B'_{j,t}  = S_j · (Bw_t + V_t + X_t)          B_ref,j = S_j · B_ref
E-7.20b Strike j of a synthesized heavy attack with heavy multiplier H_j (E-7.40):
         B'_{j,t}  = H_j · (Bw_t + V_t) + X_t           B_ref,j = H_j · B_ref
E-7.20c Data attacks (slams, heavy slams, throws, explosions, Ranged Attack, mapped attacks):
         B'_{j,t}  = their own E-1 base (01) · s_j      s_j = Dmg_j/100 for gunblade shots and mapped attacks marked
                                                        `scaled` in table 9-C, else 1
```

Everything after ST-1 follows `01` and `04` with B' := B'_j and B_ref := B_ref,j: the composition r_t uses the strike's
stance physical bonus S^phys_{j,t} (`01` E-1.9; composition and quantization per strike when it differs, `01`
EC-01-34); the base scalar is E-11 / `04` E-4.3 with the CO term multiplied by r_in (7.6). Because `S_j` is inside the
base, it is **not** a member of G for that instance (it would otherwise apply twice). For X = 0, E-7.20 with E-4.3 is
exactly `04` E-4.6.

Sources: `W:Stance` line 88 (`Physical Damage = Base Physical × (1 + Stance Physical Bonus) × (1 + Modded Physical
Bonus) × Combo Damage Multiplier On Hit`), `W:Melee` lines 638-656 (`[Base × (1 + Damage Mods) × (1 + Elemental Mods)] ×
Heavy Attack Multiplier × Combo Multiplier`, Paracesis `222 × 2.2 × 1.9 × 6 × 7 = 38,974.32`), line 681 ("Increased base
damage from Incarnon Genesis is not multiplied by the Heavy Attack Multiplier, but is multiplied by the combo
multiplier"). The light multiplier S_j multiplies the Incarnon flat (no source says otherwise; Decision). Because X_t is
split proportionally to `Bw_t + V_t` (`01` E-1.3, D-005), E-7.20b leaves the ratios `B'_{j,t}/B'_j` unchanged, so q_t of
a heavy strike equals q_t of a light strike with the same physical bonus.

### 7.3 Faction, DoT and extra hits

The melee instance is an ordinary direct instance of E-12 for everything after ST-5. Its DoT seeds carry its full
weapon side, so the stance multiplier, heavy multiplier, M(c) and stealth are inherited by its ticks (`W:Damage_over_Time`
lists the melee combo multiplier and stealth among the scaled terms, `F:status`; Reaper Prime heavy Slash example
`200 × 2.2 × 1.3 × 6 × 5 × 1.3 × 0.35 × 6 = 46,846.8`, `W:Melee` lines 660-677). Extra hits (Toxic Lash ×2 on melee,
Xata's Whisper) and Melee Duplicate follow `04` §6.

### 7.4 Melee members of G and G_F

| Factor | Applies to | Value | Source |
|---|---|---|---|
| combo M_h = M(c⁻) | heavy strikes, heavy slam, glaive heavy explosion, heavy projectiles of table 9-C (non-CO part only for `no_combo` rows, `04` co_omit) | E-7.10 | `W:Melee` line 557; `W:Glaive` line 58 |
| Slam Damage `M_slam = 1 + ΣSlam` | slam and heavy slam instances (and the Hysteria opening heavy, a slam) | Seismic Wave 2.00, Nira's set 1.50 per piece, riven splice (04) | `W:Melee` lines 177-183 ("multiplicative with other modifiers to melee damage"); Nira's double application fixed U39 |
| ability combo M_ab | Shadow Clones (Blade Storm), Landslide Fists, Shattered Lash, Whipclaw | E-7.44, T-11 | `W:Exalted_Weapon` lines 352-358 |
| Exalted Blade wave falloff φ_w | Exalted Blade light and heavy waves | E-7.43; member of `G_F` (range falloff) | `W:Exalted_Blade` line 23 |
| stance and heavy multipliers | – | inside the base (E-7.20), not in G | 7.2 |

Not here: Melee Careen, Valkyr Rage, Excalibur passive, Steel Charge, Incarnon transformation "+100 % Melee Damage"
(04: A or G by `04`'s default rule), stealth (`04` E-4.16), finisher damage (`04` E-4.17).

### 7.5 Instance flags that gate other documents

| Flag | Set on | Consumers |
|---|---|---|
| `heavy` | heavy strikes, heavy slams, glaive heavy explosions (standing throws), heavy projectiles marked `heavy_bonus` in 9-C | `01` A heavy-only members (Killing Blow, Melee Assimilation, riven "Melee Damage on Heavy Attack": heavy attacks and heavy slams, `W:Melee` line 791); `02` heavy crit doubling; `04` heavy gates (Empowered Blades, Melee Animosity consumption, Killing Blow wind-up) |
| `tennokai` | heavies executed in a Tennokai window | `04` Tennokai effects; no consumption (E-7.16) |
| `slide` | slide strikes | slide-only crit chance (Maiming Strike family, riven), `02` |
| `slam` | slams, heavy slams | Slam Damage; CO `none` (`04` D2); Melee Duplicate triggers (`04` E-4.15) |
| `radial` | slams, heavy slams, explosions, radial children | `Part = 1` (overview E-7) |
| `melee_hit` | every instance of this document except DoT and extra hits | stealth eligibility (T-08, `04` E-4.16), Valkyr Rage hit count, `on_melee_hit` triggers |
| `input_start` | first strike of each input | Melee Animosity stacking, Tennokai qualifying events |

**Heavy crit doubling** (the hook of `02` 4.2): on `heavy` instances the relative crit chance of the True Steel family
(True Steel, Sacrificial Steel with its set scaling, Galvanized Steel's base +1.10, Amalgam Ripkas True Steel) and of
melee riven Critical Chance enters ΣCC_rel a second time (`W:Melee` line 746: "Melee critical chance from Riven Mods and
the variants of True Steel apply their bonuses twice on Heavy Attacks"; Galvanized Steel heavy = +2.20 after the
U37.0.4 fix, `F:addmul` §2). Heavy slams count as heavy for the doubling (Decision, UNVERIFIED, OQ-712). Glaive
explosions after an airborne throw do not double; Volatile explosions always double (bug, `W:Glaive_(Weapon_Type)`
lines 44-45). Custom crit chance (T-26 p.c) is added once (D-094).

Melee Animosity (`W:Melee_Animosity` §Notes): each light, slide, aerial or wall **input** adds one stack (≤ 10) for the
next heavy regardless of the number of strikes or enemies; slams and heavies add none; the stacks are consumed only by
a heavy that connects (always, in the single-target metric). The schedule stores n_anim on every heavy (8.2).

### 7.6 Stance CO ratios r_in

Table 9-D imports `W:Condition_Overload_(Mechanic)` §Stances (cached lines 572-708; 63 rows; the page footnote cites a
community spreadsheet). Columns Neutral, Forward, Forward Block, Block map to the stance's own combos; columns Slide
Attack, Ground Slam Strike, Ground Finisher, Wall Attack, Air Attack, Heavy Attack are filled only on the first row of
each class because those combos are shared.

```text
E-7.21 r_in of instance j:
  stance-specific combos:   the row of the equipped stance and the column of the combo; blank cell → 1.0
  shared combos (Slide, Wall, Aerial, Heavy, Finisher): the class's first row (filled one) for that column, for every
                            stance of the class and for stanceless weapons; blank → 1.0
  exalted stances: their own row (Exalted Blade 1.22-1.30, Serene Storm, Primal Fury, Ravenous Wraith, Hysteria, Razorwing)
  slams: unused (CO behaviour `none`, 04 D2; the "Ground Slam Strike" column predates the U35.5 radial slams)
  stanceless stance-specific combos and weapon-unique combos: 1.0
```

The shared-column rule is a Decision (UNVERIFIED): the page lists those columns once per class because the combos are
identical; `04` 2.12 says "blank cells and stanceless combos use 1.0", which this refines for shared combos (OQ-716).
r_in multiplies only the CO term (`04` E-4.6); per-hit versus per-input application is `04`'s OQ-401.

### 7.7 Forced procs of melee strikes

Each strike carries the normalised proc list of its entry (E-7.0); heavy strikes carry the procs of their heavy entry
(E-7.40); data attacks carry `ForcedProcs`. `02` E-2.14 counts them per instance; stance and heavy hits also roll normal
status (`W:Melee` line 679, `W:Stance` line 84 "100% chance to initiate regardless of the weapon's actual status
chance").

---

## 8. The melee schedule generator

### 8.1 Plans

A plan turns one selected melee attack into a repeating action sequence. Defaults: Normal Attack → `light_loop` of the
selected stance combo (Neutral when none is selected); synthesized Heavy Attack (T-09 on, or selected) → `heavy_loop`;
Slide Attack → `slide_loop`; Slam Attack → `slam_loop`; Heavy Slam Attack → `heavy_slam_loop`; glaive throws and
explosions → `glaive_cycle`. `build_spend` and `tennokai` are selectable (proposed toggle, OQ-722).

| Plan | Sequence | Parameters (default) |
|---|---|---|
| `light_loop(C)` | combo C repeated, strikes at `t0 + k·Δ` | C (Neutral) |
| `slide_loop` | the shared Slide combo repeated like a light loop | – |
| `heavy_loop` | heavies back to back, alternating h = 1, 2 | start count (T-09), policy `full_regen` or `back_to_back` (`full_regen`) |
| `build_spend(K)` | light loop until M(c̄) ≥ K, then one heavy; the light loop restarts at input 1 | K = M_cap (12) |
| `tennokai` | n_tk qualifying inputs of a light loop, then one Tennokai heavy; repeat | n_tk (E-7.17) |
| `slam_loop` | one normal slam every T_slam | T_slam (1.0 s, UNVERIFIED, OQ-711) |
| `heavy_slam_loop` | one heavy slam every max(T_slam, regeneration wait) | T_slam (1.0 s) |
| `glaive_cycle` | throw (quick or charged), hit, heavy detonation, catch, repeat | throw kind (charged) |

### 8.2 Output records

| Record | Fields |
|---|---|
| hit | time t; attack id; kind (7.1); input index u and strike index e of the combo; flags of 7.5; S_j (or H_j, s_j); S^phys_{j,t}; proc names; r_in; heavy index h; c̄⁻ and M(c̄⁻), M_b(c̄⁻); g and expected extra gain; n_anim (heavies); pellet count rule (gunblade shots) |
| heavy | input time, execution time, h, Tennokai flag, c̄⁻, c̄⁺, M_h |
| combo | time and kind of every expected-path change: gain, expiry, Power Spike decay, heavy spend, regeneration start and end |

The window rule of `06` 6.3 applies: a hit belongs to the window when `t ≤ t_win + ε` (ε = 1e-9), closed interval.

### 8.3 Algorithm

```text
E-7.30 Melee schedule generator (times in s, f64, ε = 1e-9); horizon H = max(t_win, t_ttk_max)
  state: t (cursor), c̄ (= start count, 8.5), timer t_x (not running), regeneration segment, h = 1, u_since_heavy = 0
  advance(c̄, τ): apply, in time order up to τ, regeneration (E-7.15), expiry and Power Spike decay (E-7.14)

  LIGHT PHASE (light_loop, slide_loop, and the light parts of build_spend and tennokai), starting at t0:
    for k = 0, 1, 2, …:   τ = t0 + k·Δ  (Δ with AS at τ);  e = k mod N_C;  stop the phase when the plan says so
      if τ > H + ε: end
      advance(c̄, τ);  emit hit(s) of strike e (kind per 7.1; Ranged strikes expand per 9.5, 9.7, 9.8)
      c̄ += g_e + E[X]·[g_e > 0];  if g_e > 0: t_x = τ + D_cb;  count Tennokai and Animosity inputs at input starts
  expected Relentless Combination gains (6.9) enter c̄ at the expected Slash proc times and their tick times

  HEAVY STEP at input time τ_in (heavy_loop, build_spend, tennokai):
    τ_ex = τ_in + W (W_tk for Tennokai);  if τ_ex > H + ε: end
    advance(c̄, τ_ex) with the timer frozen on [τ_in, τ_ex];  M_h = M(c̄⁻)
    emit the strikes of heavy h at τ_ex + i·(T_sw/AS)/n_h, i = 0 … n_h − 1, all with M_h and n_anim = min(10, u_since_heavy)
    c̄⁺ = c̄⁻ (Tennokai) or η·c̄⁻;  t_x = τ_ex + D_cb;  start regeneration if c̄⁺ < IC;  h ← 3 − h;  u_since_heavy = 0
    next action time τ_next = τ_ex + T_sw/AS
      heavy_loop with policy full_regen:  τ_next = max(τ_next, τ_ex + max(0, IC − c̄⁺)/ρ_IC − W)
  heavy_loop:       τ_in(0) = 0; each next heavy input at τ_next
  build_spend(K):   light phase from t0 = 0; end it after the strike that makes M(c̄) ≥ K; heavy input at the time
                    of the next strike slot; after the heavy a new light phase starts at τ_next at input 1
  tennokai:         light phase; end it after the last strike of the input that holds the n_tk-th qualifying event of
                    the phase; heavy input at the next strike slot; Tennokai heavy; new light phase at τ_next at input 1
  slam_loop:        slams at 0, T_slam, 2·T_slam, …;  each gains 1
  heavy_slam_loop:  heavy slams at τ_0 = 0, τ_{k+1} = max(τ_k + T_slam, τ_k + max(0, IC − c̄⁺_k)/ρ_IC);
                    each spends per E-7.16 (no wind-up)
  glaive_cycle:     section 9.4
```

Rationale and status: the heavy cycle (wind-up + swing) and the regeneration wait follow `F:melee` §4.3
(`max(windup + swing / AS, regen_time)`); the decision that the player starts the wind-up so that execution coincides
with full regeneration (`− W`) relies on regeneration during wind-up (OQ-707). Strict alternation of the two heavies from
heavy 1 at the plan start, never reset, is a Decision (`W:Melee` line 574: "a first and second Heavy Attack, repeating in
sequence"; reset after idle undocumented, OQ-702). Light-phase restart at input 1 after a heavy is a Decision (OQ-703).

### 8.4 Melee Duplicate, waves and other inserted instances

Melee Duplicate copies (`04` E-4.15), Exalted Blade waves and mapped projectile instances are emitted at their parent's
time, after the parent, in emission order (CD-04 ordering). Projectile flight time is 0 at the default distance d = 0
(`06` E-6.22 applies at d > 0).

### 8.5 Start state, T-09 and T-10

```text
E-7.31 start count:  c(0) = IC                                         default
                     c(0) = max(IC, k_tier · (M_start − 1))            T-09 on, M_start ∈ [1, M_cap] (default min(12, M_cap))
       T-09 on: attacks without an explicit heavy flag use the heavy_loop plan; attacks with a data role (slam, heavy
                slam, throw, explosion) keep their own plan (the inspiration's `isHeavy` auto-mode, I:feat §2)
       T-10 = V: E-7.18 for light-type instances; the count still evolves (used by heavies and Lifted)
```

T-09 values above M_cap are clamped (D-063).

### 8.6 Steady-state quantities (`dps_burst`, `dps_sustained`)

```text
E-7.37 light_loop / slide_loop:  hits per s = N_C / T_loop;  Σ S per s = Σ_e S_e / T_loop
         steady M_b = min(M_bcap, M_cap)  if Σ_e g_e > 0, D_cb defined and Δ < D_cb;  else M_b(IC)
       heavy-type plans: run E-7.30 until the expected-path state at a heavy execution (c̄⁻ rounded to 1e-9, h, phase)
         repeats, or for 1000 heavies; dps_burst = damage over the last full period / period length
       melee has no reload: dps_sustained = dps_burst (overview 4.1)
```

`08` consumes these; with T-10 set, steady M_b = min(V, M_bcap).

---

## 9. Weapon families

### 9.1 Heavy attacks: the synthesized attack and its hit table

The heavy attack is not a data attack. The engine synthesizes it from the weapon's Normal Attack (types, CC0, CDM0, SC0)
and a hit table:

```text
E-7.40 Hit list of heavy h ∈ {1, 2} of weapon w
  1. Weapon-unique heavy (E-7.0b, unique table without FromWeapon): its entries as given, H = Dmg/100 (σ_h = 1);
     Ranged entries spawn table 9-C attacks (9.7); Slam entries excluded (7.1).
  2. Otherwise the class's stanceless Heavy combo, attack h; direct strikes = non-Slam strikes:
       if table 9-A gives an explicit breakdown for (class, h) whose strike count differs from the direct strikes:
            use the 9-A breakdown (multipliers, Bleed procs; no other procs)
       σ_h = T_{class,h} / Σ_direct (Dmg·Hits/100)       T_{class,h} from table 9-A; σ_h = 1 if the class has no row
       H_j = σ_h · Dmg_j / 100 ;  procs from the stance entry
  3. Weapon factor:  H_j ← κ_w · H_j        (E-7.41; κ_w = 1 unless listed)
  A heavy combo with one attack uses it for h = 1 and h = 2 (Edun).
  Glaives (9.4) and gunblades (9.5) use their own rules.
```

Table 9-A (source `W:Melee` §Heavy Attack Damage, cached lines 585-637; "Typical" wind-up is the table's, the weapon's
`WindUp` field wins):

| Class | Heavy 1 | Heavy 2 | Typical wind-up (s) | Stance data (direct sum) |
|---|---|---|---|---|
| Assault Saw | 6× (3 × 100 % + 3 × 100 % Bleed) | 6× | 1.0 | 600 / 600 |
| Blade and Whip | 4× | 12× (3 × 400 %) | 0.4 | 400 / 1200 |
| Claws | 5× Bleed | 5× Bleed | 0.6 | 500 / 500 |
| Dagger | 5× (250 % Knockback, Bleed + 250 %) | 5× (250 % Bleed + 250 %) | 0.4 | 500 / 500 |
| Dual Daggers | 5× (250 % Bleed + 250 % Bleed) | same | 0.5 | 500 / 500 |
| Dual Nikanas | 12× (3 × 200 % + 600 %) | 6× (100 % + 200 % + 300 %) | 1.0 | 1200 / 600 |
| Dual Swords | 5× (250 % + 250 %) | 5× (250 % + 250 %) | 0.7 | 500 / 500 |
| Fist | 5× | 5× | 0.6 | 500 / 500 |
| Glaive | 2× | 3× | 1.2 | not used (9.4) |
| Gunblade | 5× | 5× | 0.4 | not used (9.5) |
| Hammer | 6× | 6× | 1.2 | 600 / 600 |
| Heavy Blade | 6× | 6× | 1.1 | 600 / 600 |
| Heavy Scythe | 6× (3 × 200 %) | 12× (2 × 600 %) | 1.0 | 600 / 1200 |
| Machete | 6× (150 % Bleed + 3 × 150 %) | 5× | 0.7 | **500** / 500 |
| Nikana | 5× Bleed | 5× Bleed | 0.5 | 500 / 500 |
| Nunchaku | 5× | 5× | 0.5 | 500 / 500 |
| Polearm | 6× | 6× | 0.9 | 600 / 600 |
| Rapier | 4.5× Bleed | 4.5× Bleed | 0.5 | 450 / 450 |
| Scythe | 6× Bleed | 6× Bleed | 1.0 | 600 / 600 |
| Sparring | 12× | 15× (3 × 300 % + 2 × 300 % Bleed) | 0.5 | **500** (+400 Slam) / **400** (4 strikes) |
| Staff | 5× | 5× | 0.5 | 500 / 500 |
| Sword | 5× | 5× | 0.6 | 500 / 500 |
| Sword and Shield | 5× | 5× | 0.7 | 500 / 500 |
| Tonfa | 5× (250 % Bleed + 250 % Bleed) | same | 0.7 | 500 / 500 |
| Two-Handed Nikana | 6× Bleed | 6× (300 % Bleed + 300 % Bleed) | 0.7 | 600 / 600 |
| Warfan | 5× (250 % Bleed + 250 % Bleed) | 5× Bleed | 0.5 | 500 / 500 |
| Whip | 4.5× Bleed | 4.5× Bleed | 0.4 | 450 / 450 |
| Bayonet | – | – | – | 1000 / 500 (stance data only, σ = 1) |

The last column (this session, from `D:Stances`) shows that the stance data and the class table agree except Machete
heavy 1 (σ_1 = 6/5 = 1.2, four strikes of 150 %, matching the table's breakdown) and Sparring (heavy 1: one direct
strike scaled to 1200 %; heavy 2: the table breakdown, five strikes of 300 % of which the last two carry Bleed).

**Conflict with the per-weapon `HeavyAttack` field.** `D:<weapon>` `HeavyAttack` and `PE:<weapon>` `heavyAttackDamage`
divided by the Normal Attack total give the class multiplier for every single-strike class checked, but not for
multi-strike classes: Dagger and Dual Daggers 2.0 (table 5×), Dual Swords, Tonfa, Warfan 4.0 (table 5×), Heavy Scythe
18.0 in PE (= 6 + 12), gunblades 4 × the shot damage in the wiki and 5.0 in PE (this session; e.g. Karyst 546 = 2 × 273,
Dual Ether 720 = 4 × 180, Hespar 5040 = 18 × 280, Redeemer Prime wiki 3200 = 4 × 800, PE 1060 = 5 × 212). The meaning of
these display values for multi-strike heavies is undocumented, so the engine uses the per-mechanic page's table and the
stance data (overview 1.4: per-mechanic page with worked examples over summary data) and reports the difference as a
data warning; the conflict is UNVERIFIED (OQ-723). The wiki `HeavyAttack` values that deviate from the class on
single-strike weapons (Cadus 850, Coda Caustacyst 1560, Wolf Sledge 1036, Prisma Machete 756, Amanata absent) are
contradicted by PE (650, 1710, 1554, 1158, 756 = class value) and ignored.

```text
E-7.41 κ_w = r_w / r_mode(class)   when all of these hold, else 1:
         r_w = PE heavyAttackDamage / PE totalDamage, and the wiki HeavyAttack / Normal total agrees within 0.5 %
         the weapon uses the class heavy table (rule 2 of E-7.40) and has no heavy row in table 9-C (Corufell excluded)
         r_mode(class) = most frequent r over the class's weapons that use the class table; |r_w − r_mode| > 0.5 %
       current members (this session): Hirudo 6.5/12, Coda Hirudo 6.502857/12, Korrudo 10.699482/12, Tak & Lug 12/5
```

`W:Melee` line 623 (HTML comment): "Hirudo and Korrudo differ, 6.5x and 10.7x respectively". κ_w applies to both heavies
(Decision, UNVERIFIED for heavy 2, OQ-723).

**Heavy damage, crit and status.** Each strike j of the heavy is one instance with base E-7.20b, M_h in G, the
`heavy` flag, its proc list, and the Normal Attack's CC0, CDM0, SC0 with the heavy-only buckets (7.5). Heavies roll
normal status (`W:Melee` line 679). The heavy's damage types are those of the Normal Attack: the heavy is defined as
a multiple of weapon damage (`W:Melee` line 567 onward); the per-weapon total has no type split (`DS:wiki` open
question 1), so no other split is available.

### 9.2 Slams and heavy slams

```text
E-7.32 Slam Attack and Heavy Slam Attack are data attacks (kind slam / heavy_slam): base from data (already 2× and 3×
       the weapon damage, single type: Impact for slams, Blast for heavy slams by default, weapon overrides in data),
       own SC and ForcedProcs (heavy slam: Lifted), radial (Part = 1), Falloff from data (0.5 / 0.3 at the edge).
       single target at the epicentre: d_aoe = 0, Φ = 1 (06 8.2)
       G factors: M_slam = 1 + ΣSlam (both);  M_h = M(c⁻) and spending (heavy slam only)
       CO: none (04 D2).  Combo gain: slam 1, heavy slam 0.  Heavy-only A members and heavy crit doubling: heavy slam.
```

Sources: `W:Melee` lines 63-65 (2× weapon damage, 3× for heavy slams, single element, falloff 100 % → 50 % (70 % heavy),
radial since U35.5), lines 686-692 (heavy slam: combo multiplier, Lifted, no wind-up), lines 177-183 (Slam Damage),
lines 1092-1104 (U35.5: slams entirely radial and scalable with mods), `W:Condition_Overload` notes via `F:melee` §6.
Arca Titron's Slam Capacitor adds base damage, not Slam Damage (`W:Melee` line 183): a member of A on slams of that
weapon (`04` catalogue). Slam radius scales 50-150 % with drop height and per class (`W:Melee` lines 67-170): only the
multi-target mode uses it. The Lifted duration from a heavy slam or heavy attack depends on M (`03` 8.9 owns the rule;
this document supplies M_h). Cross-check data: PE `slamRadialDamage` and `heavySlamRadialDamage` (2 × and 3 × B) equal
the wiki slam attacks; PE `slamAttack` and `heavySlamAttack` (3 × and 4 × B on Nikana Prime) are the pre-U35.5 direct-hit
values and are not used. Sampotes and Tenet Exec have weapon-unique `Heavy Slam` combos (12 × 500 % and 9 × 300 % Slam
entries, Duration 3.3 s and 2.5 s): shockwave patterns for the multi-target mode; the single-target heavy slam is the data
attack once (Decision, UNVERIFIED, OQ-710).

### 9.3 Slide, aerial and wall attacks

The slide attack is synthesized from the shared Slide combo (E-7.0b): its strikes are `slide` instances with
`S_j = Dmg_j/100` (Nikana Slide 200 % = Nikana Prime `SlideAttack` 396 = 2 × 198, `D:Nikana Prime`). `SlideAttack`
fields that disagree with the combo (Sword class: 15 weapons at 1.0 × B, Skana at 0.625 × B = 75/120; Heavy Blade:
Zenistar 0.872 × B; War and War Prime without the field) are reported and ignored. Slide durations exist for 21 of 28
stanceless and 48 of 63 PvE shared tables; missing ones use E-7.3. Pressure Point and elemental mods apply (`W:Melee` line 47); "Critical Chance for Slide Attack" mods and riven
lines gate on `slide`. The second heavy can be started from a slide (heavy slide: second heavy, Tennokai qualifying).
Aerial combos (mostly without Duration) and wall attacks (class multipliers 1.67×-3×, `W:Melee` lines 185-250; zaw
`Multi.Wall`) are data for completeness; no default plan uses them (`F:melee` §9 recommendation).

### 9.4 Glaives and thrown weapons

| Attack (data) | Kind | Rules |
|---|---|---|
| `Throw`, `Charged Throw` | throw (direct, `Thrown`) | own CC, SC, ForcedProcs; CO applies (U28.0.6); combo gain 1; M_b for Blood Rush; Power Throw and the two-bucket rule (`04` 3.4); no heavy flag |
| `Throw Recall Explosion`, `Charged Throw Recall Explosion` | explosion, heavy | the heavy-input detonation: radial (Part = 1, no falloff since U29.5), × M_h, spends combo, heavy-only bonuses and crit doubling (standing throw), CO `none`, two-bucket (`04`) |
| `Throw Bounce Explosion`, `Charged Throw Bounce Explosion` | explosion, not heavy | Volatile Rebound / Volatile Quick Return explosions (50 % of the heavy explosion): no combo multiplier, no spending, no gain, no forced procs except data ForcedProcs, crit doubling always (bug) |
| `Slam Attack`, `Heavy Slam Attack` | 9.2 | |
| `Normal Attack` | melee swings with the stance loop | |

```text
E-7.36 glaive_cycle (charged throw, default):  cycle n starts at τ_n
         release at τ_n + CT_g;  throw hit at release (+ d/v at d > 0);  heavy detonation at the throw hit time:
         M_h = M(c⁻) then spend (E-7.16);  Volatile explosion (if equipped, chance per bounce: one per throw on the
         single target, Decision) at the throw hit time, before the detonation
         τ_{n+1} = max(release, τ_n + CT_g) + regeneration wait as in heavy_loop full_regen (no wind-up term)
       quick-throw variant: release at τ_n, cycle 1 / FireRate_Throw
```

Sources: `W:Glaive` lines 12-25, 57-58 (heavy attack in flight explodes, "counted as a Heavy Attack and consumes the
combo meter. It benefits from heavy attack mods and the combo multiplier"; "headshot multiplier of 1x"), patch U30.5
(wind-up 0.6 → 1.2 s; quick throws ~50 % of charged, cached lines 121-140), `W:Melee` line 748 ("Glaive type weapons
apply these effects to the explosion of a Glaive Heavy attack only, and not to the weapon's damage from direct hits"),
`W:Volatile_Rebound` lines 28-31 and `W:Volatile_Quick_Return` lines 30-31 (50 % of heavy damage, no combo multiplier, no
spending, no guaranteed procs), `W:Glaive_(Weapon_Type)` lines 44-45 (crit doubling bugs), `F:melee` §11 (class heavy
2×/3× and the Stances/data glaive heavy "Falling Star" 3 × 500 % conflict with the data explosions; the data attacks
win). The flight and catch times are not documented: the cycle uses only the data charge time and fire rate (Decision,
UNVERIFIED, OQ-711). Melee Animosity: ground throws benefit but do not consume stacks (bug, `F:melee` §11): the throw
is not `heavy`, so it never consumes. Wolf Sledge and Sigma & Octantis (shield throw) use this section with their own
throw attacks (`F:melee` §12).

### 9.5 Gunblades

```text
E-7.45 A Ranged stance strike of a gunblade fires one shot of the weapon's "Ranged Attack":
         pellets = floor(MS) + Bernoulli(frac(MS)) (02 E-10; MS from the attack's Multishot), each pellet one instance
         base per pellet = E-1 base of Ranged Attack × s_j,  s_j = Dmg_j / 100         (Decision, UNVERIFIED)
         falloff Φ(d) of the Ranged Attack (06 E-6.20), hitscan
         pellets are sequential for the CO count (04 2.3)
         combo gain 1 per pellet; one Tennokai qualifying event per shot (multishot does not add events)
       Heavy (Full Bore): the stanceless Gunblade Heavy combo as shots: heavy 1 = 2 shots at 400 % (+2 Slam entries,
         excluded), heavy 2 = 1 shot at 400 % (+1 Slam entry); each shot × M_h; σ = 1 (class 5× not used)
       Melee strikes of a gunblade loop are ordinary light strikes.
```

Sources: `F:melee` §12 (shots use no ammo; each shot counts as a Tennokai hit, multishot does not; pellets hit
sequentially for CO; U28.0.6 "Gunblade Heavy Attacks will now expel Melee Combo as intended"), `W:Gunblade` (no ammo,
falloff), `W:Melee` line 728 (gunblade exception for Tennokai), `W:True_Punishment` line 29. Evidence for `s_j`
scaling the shot: `D:<weapon>` `HeavyAttack` of Redeemer Prime, Stropha and Vastilok equals exactly 4 × (Ranged Attack
total × Multishot) (3200, 2800, 2484), i.e. one 400 % heavy shot; the High Noon and Bullet Dance CO ratios are 100 % on
every input. PE gives 5 × the melee base instead (OQ-723). Whether melee-only mods apply to shots: they do (the shot is
an attack of a melee weapon; no source says otherwise; Decision). Corufell's heavy is a gunblade-like projectile (table
9-C).

### 9.6 Whips and other classes

Whips have no class-specific damage mechanic beyond the class tables (heavy 4.5× Bleed, wind-up 0.4 s, follow-through
0.5; `F:melee` §12). Blade and Whip heavy 4× / 12×. Heavy Scythe, Dual Nikanas and Sparring have large second heavies
(table 9-A). Follow-through (`W:Melee` lines 814-940: n-th enemy takes FT^(n−1); exceptions Volnus with a stance mod,
Ironbride, Magistar, Nami Solo, Dorrclave) is multi-target only and is kept as data.

### 9.7 Weapon-unique melee attacks (table 9-C)

A row of table 9-C maps a trigger to a data attack. Fields: trigger (a `Ranged` entry of a named combo, a heavy h, the
slide, a block combo), count per trigger (the entry's Hits, or the attack's Multishot), `scaled` (s_j = Dmg/100) or
`as_data` (s_j = 1; the stance Dmg% is the attack's damage relative to the Normal total and serves as a consistency
check), heavy flags (`heavy_combo`: × M_h and spends with its heavy; `heavy_bonus`: heavy-only A members and crit
doubling), CO behaviour (`04` catalogue). Default for rows from heavies: `heavy_combo` yes, `heavy_bonus` no ("Non-hitscan
projectiles created by Heavy Attacks … will often not be counted for bonuses like Killing Blow, except while the user is
in the Heavy Attack Wind Up animation", `W:Melee` line 750; Decision, UNVERIFIED, OQ-717).

| Weapon | Trigger | Data attack | Count, scaling | Check |
|---|---|---|---|---|
| Syam | heavy 1 / heavy 2 Ranged | `First Shockwave` 500 Heat / `Second Shockwave` 1500 Heat | 1, as_data | 185 % ≈ 500/270, 555 % ≈ 1500/270 |
| Arum Spinosa | heavy 1 / 2 Ranged | `First / Second Heavy Attack - Toxic Spines` | 18 / 9 (Multishot), as_data | Hits 18 / 9; CO no_combo (`04`) |
| Quassus, Quassus Prime | heavy 1 / 2 Ranged | `First / Second Heavy Attack - Ethereal Daggers` | Multishot (12 / 6, Prime 18 / 9), as_data | CO no_combo |
| Argo & Vel | heavy Ranged | `Heavy Attack Glaive` (+ `AoE` child) | 1, as_data | CO no_combo |
| Edun | heavy Ranged (one heavy combo attack, used for h = 1 and 2) | `Polearm Throw` (+ `Polearm Explosion`) | 1, as_data | 220 % ≈ 480/220; CO ratio 0.25 |
| Tenet Agendus | heavy 1 Ranged | `Energy Disk` | 1, as_data | CO multiplying 0.2 |
| Tenet Grigori | heavy 2 Ranged | `Energy Disk` | 1, as_data | universal-only bucket (`04`) |
| Caustacyst, Coda Caustacyst | heavy Ranged | `Corrosive Wave` (+ `Corrosive Pool`) | 1, as_data | |
| Zenistar | heavy Ranged | `Disc Impact`, `Disc Explosion` (deployed disc attacks: multi-target, kept as data) | 1, as_data | CO ratio 10.4 |
| Corufell, Corufell Prime | heavy | `Charged Projectile` | 1, as_data | CO multiplying |
| Nepheri | Neutral input 2 Ranged | `Fireballs` | 4 (Hits), as_data | 50 % ≈ 131/261 |
| Verdilac | Block Ranged | `Energy Wave` | Hits, as_data | |
| Tatsu, Tatsu Prime | Slide Ranged | `Soul Swarm Projectile` | 5 (Hits), as_data | 40 % ≈ 96/230 |
| gunblades | Ranged entries | `Ranged Attack` | 1 shot, scaled | 9.5 |
| Exalted Blade, Desert Wind | Ranged entries | none: Normal Attack × S_j (waves, 9.8) | 1, wave | |
| Ack & Brunt, Ceramic Dagger, Okina, Hate, Innodem, Destreza (Incarnon), Vinquibus, Korumm, Telos Boltace, Vitrica, Sigma & Octantis, Orvius | Incarnon form, conditional or multi-target triggers | their data attacks | – | not in default plans (EC-07-52) |

The trigger column for Syam, Edun, Zenistar, Caustacyst and the Incarnon rows is the pipeline's curation task; the
values above come from `D:Stances` Ranged entries and the attack names of `D:<weapon>` (this session).

### 9.8 Exalted melee weapons

```text
E-7.43 Base: m_str = Str (01 E-1.1 model 1) for Exalted Blade, Valkyr Talons, Iron Staff, Desert Wind, Shadow Claws,
       Diwata; the exalted weapon keeps its own combo state (6.1) and its own mods.
       Exalted Blade light waves: every Ranged entry of the Exalted Blade stance (and one wave per slide strike, with
         the slide strike's Dmg, Decision) is one `ranged` instance: E-7.20 with S = Dmg/100, falloff
         φ_w = max(0, 1 − d/70) in G_F, separate crit and status rolls, combo gain (E-7.13), Tennokai events
       Exalted Blade heavy: the Sword heavy table (blade strikes × M_h) plus 2 waves per heavy, each with base
         H_j·B_ref (E-7.20b without M_h), φ_w, Lifted forced, weak points possible (projectile), no M_h, no gain;
         both waves hit the single target (Decision, UNVERIFIED)
       Exalted Blade and Desert Wind normal slam: CC0 = 0 and no M (as every normal slam)
       Desert Wind waves: Ranged entries of Serene Storm, no falloff (≤ 20 m), line of sight
```

Sources: `W:Exalted_Blade` lines 17-40 (100/125/200/250 × Strength; normal, slide and aerial attacks emit waves with
"the same base damage as the attacks that release them"; separate entities; waves add combo and trigger Tennokai; "lose
40 % of their damage per second; Wave damage falls off linearly with distance from 100 % to 0 % between 0 and 70 m";
lose 20 % per enemy hit (multi-target); heavy: 2 waves in a cross, projectiles that hit weak points, do not lose damage
per enemy, "do not scale off combo counter"; slam radial "does not have a critical chance, is not affected by the Melee
Combo Counter"), `W:Serene_Storm` lines 30-50 (waves 20 m, no falloff, add combo, trigger Tennokai), `W:Exalted_Weapon`
lines 94-96 and 285 (pseudo-exalted weapons have no heavies; waves gain combo since U38.5), `F:melee` §13. The two wave
statements of `W:Exalted_Blade` differ (40 %/s at 30 m/s is 1.33 %/m, the linear rule 1.43 %/m); the explicit linear
distance rule is used (Decision). `D:Stances` encodes Exalted Blade waves as `Ranged` entries with their own percentages
(e.g. Neutral input 2: 300 % blade, 100 % wave), which overrides "same base damage" for light waves (Decision).
`W:Desert_Wind` line 24 says "Melee and wave damage can be increased by the combo counter"; the general rule (normal
attacks are not multiplied) is kept (Decision, UNVERIFIED, OQ-718).

Valkyr Talons (Hysteria): the Hysteria heavy combo's first heavy is a Slam entry ("considered a slam, and thus benefits
from Seismic Wave", `W:Hysteria` line 73): it is modelled as a heavy slam-type instance at the epicentre (H = 5.0, ×
M_h, × M_slam, radial) rather than excluded (Decision, the page is explicit); the second heavy is 5 strikes of 300 %
with Slash procs on strikes 1 and 3. Slide: 6 strikes (`W:Hysteria` line 44). Valkyr Rage passive gains per hit
(`04`). Iron Staff gains +0.5 m range per tier (no damage effect). Diwata has no heavies (`W:Melee` line 563).
Shattered Lash, Whipclaw, Landslide Fists and Shadow Clones use E-7.44 and have no heavies, no blocking, no Tennokai
(`W:Melee` line 730). Garuda Talons are a normal claw weapon (no Strength scaling, any Claw stance).

### 9.9 Zaw assembly

```text
E-7.42 Zaw = strike S + grip G + link L;  hand type τ = G.Type (OneHanded | TwoHanded); class = S.Type[τ]
  B_zaw    = (Σ_t S.Damage_t + G.Damage + L.Damage) · S.Multi[τ].Base
  B0_t     = S.Damage_t · B_zaw / Σ_t S.Damage_t                 every strike except
  B0_t     = S.Damage_t + (B_zaw − Σ_t S.Damage_t) / n_types       Plague Keewar, Plague Kripath (n_types = 4)
  CC0 = S.CritChance + L.CritChance ;  CDM0 = S.CritMultiplier ;  SC0 = S.StatusChance + L.StatusChance
  FR_src = G.Speed + L.Speed + S.Speed  (then E-7.1)
  Slide multiplier S.Multi[τ].Slide, wall S.Multi[τ].Wall, slam ("Leap") S.Multi[τ].Leap = 2
  Heavy, wind-up, combo duration, stance tables: those of `class` (table 9-A typical wind-up, D0_cb = 5)
```

Sources: `W:Zaw` lines 192-200 (grip sets base speed and damage, the distribution keeps the strike's ratio), lines
580-631 ("Final stat values are calculated by adding values of the Strike, Grip, and Link. For Two-Handed Zaws, the
Damage value is further multiplied by a factor … depending on the Strike"), `M:Modular` lines 384-450 (the damage, per-type
and slide formulas, including the Plague-strike split `strike_t + (total − base)/4`) and lines 530-630 (crit, crit
multiplier, status and speed sums), `D:Modular` `Zaw` (strike `Multi` tables, grip and link values). The zaw slam and
heavy slam attacks are not in the data: they are synthesized as `Slam Attack` = Leap × B_zaw Impact with forced Impact
and `Heavy Slam Attack` = 3 × B_zaw Blast with forced Lifted, the strike's CC and CDM, SC 0.10 and the class falloff
(Decision, UNVERIFIED, OQ-719). Exodia arcanes (zaw only) are `04`'s; the Exodia Contagion aerial projectile ("2× Zaw
base", `W:Condition_Overload_(Mechanic)` melee table) is an aim-glide attack outside the default plans.

### 9.10 Incarnon melee form

A melee Incarnon weapon enters its form with a heavy attack at combo tier 6× or higher (5× Innodem and Praedos, 3× with
some evolutions); Genesis weapons then get +100 % Melee Damage, +20 % sprint and parkour speed for 180 s; native Void
weapons typically +3 range and "+40 Attack Speed" for 90 s; the transformation does not reset the combo beyond the
heavy's spending; a Tennokai heavy activates it without loss (`W:Melee_Combo` lines 241-251, `W:Melee` line 732,
`F:mods` §9). The engine treats the form as a scenario state active for the whole window when the user selects it
(Decision; activation cost not simulated, OQ-720). The +100 % Melee Damage enters A by `04`'s default rule; the "+40
Attack Speed" is read as ΣAS += 0.40 (unit UNVERIFIED, OQ-720). Incarnon flat damage on heavies: E-7.20b.

### 9.11 Finishers (optional mode)

Finishers are not in the default metric (`04` 7.2). Table 9-E is `W:Finisher` §All Finishers Multiplier Comparison
(cached lines 458-566); each class lists ground, stealth, back, front and universal finisher multipliers with their hit
split, e.g. Sword 10× / 16× (8× + 8×) / 8× (6× + 2×) / 8× / 8×, Hammer 12× / 12× (4× + 8×) / 24× (12× × 2) / 12× / 12×,
Claws ground 30× (5× × 6), Valkyr Talons ground 54× (10× × 5 + 4×); Exalted Blade's ability page gives 400 % and 3200 %
instead (`W:Exalted_Blade` line 41; conflict, OQ-721). A finisher mode schedules one finisher of the chosen type every
user-set interval (default none); each hit of the split is one instance of `04` E-4.17 with M_class = that hit's
multiplier.

---

## 10. Worked examples

Arithmetic checked with `python3` (f64; numpy f32 for E-4). Inputs not stated are zero.

**10.1 Light loop and combo over 20 s.** Nikana Prime (FR 1.08 → 65/60), Blind Justice "Guiding Light" (Duration 2.6 s;
inputs 300 % (Slash ×2, Knockback), 200 % (Slash ×2), 2 × 200 %, 2 × 200 %, 200 % (Bleed) + 400 % (Knockdown): N = 8,
ΣS = 19.0, gains 3, 2, 2, 2, 2, 2, 2, 4 = 19 per loop). Primed Fury 0.55 + Gladiator Vice 0.30 + Arcane Strike 0.60 →
ΣAS = 1.45, AS = 2.654167, T_loop = 0.979592 s, Δ = 0.122449 s, 8.1667 hits/s, ΣS 19.3958 per s. IC 0, every strike
connects: 164 strikes in [0, 20] (k = 0 … 163); the count reaches 20 (2×) after the strike at 0.9796 s and 220 (12×)
after the strike at 11.2653 s; 389 at 20 s. Blood Rush with True Steel on a 0.28 base: hit-averaged crit chance over the
window `mean(0.28·(1 + 1.2 + 0.4·(M_b(c⁻) − 1))) = 1.466244`, versus 1.848 if 12× were assumed from t = 0 (the
inspiration's default, D-062). `F:melee` §3.5 reported 11.42 s with FR 1.08 and the first strike at Δ instead of 0
(11.3001 + 0.1228 = 11.4229 s); this spec puts the first strike at t = 0 (TV-07-02).

**10.2 Heavy damage.** Fragor Prime (IMP 189, PUN 40.5, SLA 40.5; B = 270; CC0 0.40; CDM0 2.5), Hammer heavy 1 600 %
(H = 6), Primed Pressure Point 1.65 + Killing Blow 1.20 (heavy only) → A = 2.85, True Steel 1.20 (doubled on the heavy),
Organ Shatter 0.90. q = 22/32, 5/32, 5/32 (Σ = 1). Y = 6 × 270 × 3.85 = 6237 per M; at M = 8: 49,896 (non-crit). CC =
0.40 × (1 + 2.40) = 1.36; CDM = Q(2.5) × 1.9 = 2.50061059 × 1.9 = 4.75116012; E[c] = 6.10157776; expected damage
304,444.32 (the unquantized 304,365.6 of `F:melee` §4.4 omits E-4). Without the doubling CC would be 0.88 (TV-07-03).

**10.3 Initial-combo heavy loop.** Same weapon, IC = 30 (innate) + 30 (Corrupt Charge) + 80 (Galvanized Reflex ×4) =
140, η = 0.50, W = 1.2 / 1.6 = 0.75 s, Primed Fury → AS = 0.8 × 1.55 = 1.24, T_sw = 1.95 s (fallback) → 1.572581 s, D_cb =
5 × 0.5 = 2.5 s, T-09 start 12× → c(0) = 220. Executions at 0.75 (220, 12×), 3.072581 (140, 8×), then every 2.322581 s at
8× (swing-limited: 1.5726 > 70/40 − 0.75 = 1.0); 9 heavies in 20 s, ΣM = 76. With ΣAS = 2.0 (AS = 2.4, swing 0.8125 s)
the policies differ: `full_regen` waits (period 1.75 s, 12 heavies, ΣM 100), `back_to_back` fires every 1.5625 s at
7× after the second heavy (13 heavies, ΣM 97) (TV-07-05).

---

## 11. Edge cases

| ID | Rule | Source |
|---|---|---|
| EC-07-01 | Normal (light, slide, aerial, wall) attacks are never multiplied by the combo multiplier; it multiplies heavies, heavy slams, glaive heavy explosions and heavy projectiles, and feeds combo-scaled bonuses. | `W:Melee_Combo` line 13; 6.2, 7.4 |
| EC-07-02 | Venka Prime reaches 13× at 240, but Blood Rush, Weeping Wounds and Gladiator stop at 12× (M_bcap). Dex Nikana: 11 points per tier, cap 11×. | `W:Melee_Combo` lines 93-94; `W:Venka_Prime` line 44; E-7.10, E-7.11 |
| EC-07-03 | A heavy at combo below 20 is allowed and uses M = 1. | `W:Melee` line 705 |
| EC-07-04 | Heavy attacks add no combo; slams add 1 per enemy; heavy slams add none; blocking (+1 per blocked projectile) is out of scope. | `W:Melee` lines 63, 705; `W:Melee_Combo` line 31; E-7.13 |
| EC-07-05 | Combo gain per stance strike = round-half-up(Dmg/100) (50 % → 1, 40 % → 0, 250 % → 3), per enemy struck; Swirling Tiger "Dancing Hunter" last hit and Homing Fang "Cutting Arches" first hit are documented exceptions without values: they use the rule (UNVERIFIED). | `W:Melee_Combo` lines 26-28; `M:Stances` lines 335-349; OQ-704 |
| EC-07-06 | Additional Combo Count Chance: additive, +1 per gaining hit with chance p (floor + Bernoulli above 1); sources Quickening 0.20, True Punishment 1.00 (with −50 % base duration), Enduring Strike (Lifted targets), Guardian Derision (blocking), Exodia Triumph, Exodia Valor (Lifted, zaws). | `W:Melee_Combo` lines 38-52; `W:Quickening`; `W:True_Punishment`; E-7.13 |
| EC-07-07 | Relentless Combination: +1 (chance) per melee Slash proc and per Bleed tick; works on exalted weapons, Exalted Blade waves and thrown glaives; benefits from Additional Combo Count Chance. | `W:Relentless_Combination` lines 24-31 |
| EC-07-08 | Rauta: 2 combo points per pellet, at most 28 per shot. | `W:Melee_Combo` line 36 |
| EC-07-09 | Gunblades: each shot pellet adds 1 combo; each shot (not each pellet) is a Tennokai event; pellets are sequential for the CO count. | 9.5 |
| EC-07-10 | Volatile glaive explosions add no combo; thrown direct hits do (this beats Relentless Combination's "the combo counter does not interact with Glaives"). | `W:Volatile_Rebound` lines 28-29; 6.3 |
| EC-07-11 | Melee Duplicate copies and extra hits add no combo. | `W:Extra_Hit`; E-7.13 |
| EC-07-12 | Combo duration: `D0·(1 + Σpct) + Σflat`; percent penalties act on the base (Corrupt Charge and True Punishment subtract exactly 2.5 s each from 5); minimum 0.1 s; raw ≤ 0 prevents gain (heavies still spend; U35.1); Body Count +12, Drifting Contact +10, Gladiator Rush +6 (33 s together), Primary/Secondary Dexterity +7.5 each, Swift Momentum +6, Combo Killer +5, Amalgam Furax Body Count +15, Melee Guidance −6 self; riven combo duration ±. | `W:Melee_Combo` lines 96-131; `W:Corrupt_Charge` line 29; `W:Body_Count` line 29; E-7.12 |
| EC-07-13 | Base combo duration exceptions: Guandao Prime 6 s, Pulmonars 9 s, Vitrica 10 s, Xoris infinite. | `W:Melee_Combo` lines 122-131 |
| EC-07-14 | The combo timer is refreshed by every gaining hit and by every heavy (even one that hits nothing) and is paused during the wind-up. | `W:Melee_Combo` line 286; `W:Melee` line 705 |
| EC-07-15 | Power Spike: at each expiry the count loses 20/15/10/5 instead of resetting, floored at IC (Decision); 220 → 0 takes 44 expiries (220 s at 5 s, 748 s at 17 s). | `W:Melee_Combo` line 119; `W:Focus/Naramon` lines 23-27 |
| EC-07-16 | Initial Combo is the count after any reset and the minimum when idle; heavies spend it; it regenerates at 40 points/s; Corrupt Charge "rapidly regenerate[s] to full within 2 seconds". Innate: Synoid Heliocor 20, Furax Wraith 20, Fragor Prime 30. No cap (Melee Crescendo is unbounded). | `W:Melee_Combo` line 135; `W:Corrupt_Charge` line 28; D-705 |
| EC-07-17 | Heavy attack efficiency keeps `η·c` (40 % of 160 → 64), capped at 90 %; Furax Incarnon +90 % applies to heavy slams only; Skana Incarnon +80 % with overshields; Corufell / Dual Keres +20 % with their warframe; conditional Incarnon perks per `04`. | `W:Melee_Combo` lines 192-235 |
| EC-07-18 | Heavy with zero combo duration and HAE: Initial Combo regenerates over time, not instantly (U35.5 fix). | `W:Melee` line 1151 |
| EC-07-19 | Chaining a second heavy immediately does not keep the combo (bug fixed U33.6; the `W:Melee` tip is outdated). | `W:Melee` lines 954, 1270 |
| EC-07-20 | Tennokai: 15 % per qualifying input (22.5 % with Dreamer's Wrath), every 4th with Discipline's Merit, window 2 s (4 s with Opportunity's Reach); a Tennokai heavy or heavy slam spends no combo; multi-strike inputs count once; gunblade shots, Exalted Blade and Serene Storm waves count; heavies do not; a heavy slide does; the window pauses while a glaive flies; pseudo-exalted weapons and Diwata cannot use Tennokai. | `W:Melee` lines 719-742; `F:melee` §4.2; E-7.17 |
| EC-07-21 | Tennokai heavies ignore other wind-up speed bonuses; the Tennokai wind-up value is undocumented (engine: normal modded wind-up). | `W:Melee` line 723; OQ-708 |
| EC-07-22 | Attack speed speeds combos, slides, aerials and the heavy swing, not the wind-up; glaive and Wolf Sledge charged throws use only wind-up speed. | `W:Attack_Speed` line 30; `W:Melee` line 705 |
| EC-07-23 | Base attack speeds are k/60 values; the displayed value is rounded (Ripkas: wiki 0.883 vs PE 50/60). | `W:Attack_Speed` line 32; E-7.1 |
| EC-07-24 | Out of the single-weapon schedule: combo pause while holstered (Tenet Livia, Tenet Grigori, Anku, Ack & Brunt, Furax, Okina, Obex Incarnon "Standoff"/"Abiding Hold"), Combo Killer's glaive timer reset on a secondary kill, Secondary Outburst consuming the combo, Melee Crescendo stacks (user input to `04`). | `W:Melee_Combo` lines 120-132 |
| EC-07-25 | Heavy damage `[B·(1 + A)·(1 + Elem)]·H·M`; the Incarnon flat is ×M but not ×H. | `W:Melee` lines 638-681; E-7.20b |
| EC-07-26 | Heavy-only base damage (Killing Blow, Melee Assimilation, riven splice) is additive with Pressure Point and applies to heavies and heavy slams only. | `W:Melee` line 791; `01` Table 4.8-A |
| EC-07-27 | True Steel family and melee riven crit chance count twice on heavies (Galvanized Steel heavy +2.20 after U37.0.4); heavy slams count as heavies (Decision); glaive airborne-throw explosions miss the doubling, Volatile explosions always get it (also Wolf Sledge). | `W:Melee` line 746; `W:Glaive_(Weapon_Type)` lines 44-45; 7.5 |
| EC-07-28 | Melee Animosity: one stack per normal input (not per strike or enemy), max 10, consumed only by a heavy that connects; slams and heavies add none; ground glaive throws benefit without consuming (bug). | `W:Melee_Animosity` §Notes; `F:melee` §11 |
| EC-07-29 | Dreamer's Wrath +32 % crit damage applies to Tennokai heavies only (additive with Organ Shatter); Condition's Perfection, Master's Edge, Truth's Flame are Tennokai-gated (values `04`). | `F:melee` §4.1-4.2 |
| EC-07-30 | Heavies roll normal status besides their forced procs; class forced Bleed: Claws, Dual Daggers, Nikanas, Two-Handed Nikanas, Rapiers, Scythes, Tonfas, Warfans, Whips, and per table 9-A. | `W:Melee` line 679; `F:status` §4.3 |
| EC-07-31 | Two heavies alternate; the second can be started from a slide. | `W:Melee` line 574; 8.3 |
| EC-07-32 | Heavy multiplier sources conflict (class table vs stance data vs `HeavyAttack` vs PE); resolution E-7.40, E-7.41; Machete heavy 1 is rescaled to 150 % strikes; Sparring uses the class table (12× / 15×); weapon factors Hirudo 6.5/12, Coda Hirudo, Korrudo 10.7/12, Tak & Lug 12/5. | 9.1; OQ-723 |
| EC-07-33 | Non-hitscan heavy projectiles (Destreza Incarnon, Syam) often miss heavy-only bonuses unless still in wind-up: table 9-C rows default to `heavy_bonus = no`. | `W:Melee` line 750; OQ-717 |
| EC-07-34 | Stance physical bonuses multiply with physical mods (`(1 + S^phys)(1 + P)`), per strike; composition and quantization per strike. | `W:Stance` lines 87-88; `01` EC-01-34 |
| EC-07-35 | Forced stance procs are guaranteed regardless of status chance and damage types; crowd-control names (Ragdoll, Stagger, Big Stagger, Stun, Impair, Disarmed, Finisher) create no status. | `W:Stance` line 84; `03` 4.2; OQ-715 |
| EC-07-36 | Combo-embedded `Slam` entries deal flat, unmoddable radial damage (U35.5) and are excluded from single-target damage, procs and gain; Slam-only inputs (stanceless Hammer) therefore deal no modelled damage; the Hysteria opening heavy is the exception (9.8). | `M:Stances` lines 276-294; `W:Melee` line 1104; OQ-710 |
| EC-07-37 | Slams: 2× (heavy 3×) weapon damage already in the data, one type, radial, falloff to 50 % (70 %), own status chance, forced Impact (Lifted for heavy slams), radius 50-150 % by drop height; no CO; Slam Damage ×(1 + ΣSlam) multiplicative with Pressure Point (Nira's double application fixed U39); Arca Titron's Slam Capacitor is base damage. | `W:Melee` lines 63-183, 1092-1104; 9.2 |
| EC-07-38 | Lifted from heavy attacks and heavy slams lasts by combo tier and is not extended by status duration (rule and the `1 + M` vs `M` conflict owned by `03` 8.9); Heavy Blade heavies always Lift survivors; Lifted and Knockdown forced by melee count as statuses for CO and cancel each other (`03` 8.9). | `W:Melee` lines 690, 795-799; `03` OQ-309 |
| EC-07-39 | Slide attacks: stance Slide combo multipliers (the `SlideAttack` field is a cross-check only), Pressure Point and elementals apply, slide-only crit mods gate on `slide`. | `W:Melee` line 47; 9.3 |
| EC-07-40 | Wall attacks (1.67×-3× by class) and aerial combos are data only; no default plan. | `W:Melee` lines 185-250; 9.3 |
| EC-07-41 | Glaives: separate throw, charged throw, bounce (Volatile) and recall (heavy) explosion attacks with their own data; quick throws ≈ 50 % of charged (U30.5); recall explosions have no falloff since U29.5, a 1× headshot multiplier, and are heavies (×M, spend, heavy mods); Volatile explosions are 50 % of the heavy explosion, ×1 combo, no spending, no Killing Blow; CO applies to throws, not explosions; the class 2×/3× and the Stances/data 3 × 500 % heavy are not used. | `W:Glaive` lines 12-58, 121-140; `W:Volatile_Rebound`; `F:melee` §11; 9.4 |
| EC-07-42 | Glaive throws are wind-up-speed scaled, not attack-speed scaled; Power Throw +100 % per consecutive throw (3 stacks) is a weapon-damage bucket (`04` 3.4). | `W:Attack_Speed` line 30; `W:Power_Throw` §Notes |
| EC-07-43 | Gunblade shots use no ammo, have falloff, and their stance Dmg% scales the shot (Decision); gunblade heavies spend combo (U28.0.6 fix). | 9.5; `W:Gunblade` |
| EC-07-44 | Weapons with unique combos override the stance for those combo types (data: Sigma & Octantis, Cobra & Crane, Vitrica aerial; Nepheri neutral; Korumm, Verdilac, Azothane block; Tatsu, Telos Boltace slide; heavies of Argo & Vel, Arum Spinosa, Caustacyst, Edun, Jat Kusar, Keratinos, Quassus, Sampotes, Syam, Tenet Agendus, Tenet Grigori, Zenistar; Sampotes and Tenet Exec slam and heavy slam). | `W:Stance` lines 678-684; E-7.0b |
| EC-07-45 | Stance switching mid-combo is seamless: each combo is an independent loop. | `W:Stance` line 16 |
| EC-07-46 | Stance data quirks: holes, short and long arrays, the Blind Justice bare `Knockdown` (nil → no proc), Crimson Dervish Block = Neutral, non-round multipliers, `StatusMultiplier` ignored. | E-7.0; OQ-714 |
| EC-07-47 | Missing durations: 16 stanceless Neutral loops and 38 of 44 heavy combos; fallbacks E-7.3, E-7.5 are flagged in the output. | 4.5; OQ-701 |
| EC-07-48 | Exalted melee base damage scales with Ability Strength; weapon rank affects only capacity; Garuda Talons do not scale. | `F:melee` §13; `01` E-1.1 |
| EC-07-49 | Exalted weapons keep their own combo counter and their own combo mods (Melee Combo Counter mods enabled on exalted weapons in U38.5); Melee Crescendo stacks only while a channeled exalted weapon is active; the Xoris + exalted combo reset was removed in U38.5; Blood Rush and Weeping Wounds work on Garuda Talons and exalted melee (U29.5 fix). | `W:Exalted_Weapon` lines 251-285; `W:Body_Count` line 30 |
| EC-07-50 | Exalted Blade waves: separate instances with their own rolls, linear falloff to 0 at 70 m, −20 % per enemy (multi-target), add combo and Tennokai; heavy waves: 2 per heavy, weak-point capable, no combo multiplier; normal slam radial: no crit, no combo; Swordsmanship passive +10 % damage and attack speed (`04`). | `W:Exalted_Blade` lines 17-48; E-7.43 |
| EC-07-51 | Hysteria: the first heavy is a slam (Seismic Wave applies), slide does 6 hits, wind-up mods on the regular melee weapon also apply. | `W:Hysteria` lines 40-44, 73 |
| EC-07-52 | Weapon-unique projectile attacks (table 9-C) carry their own data; Incarnon-form, conditional or multi-target ones (Ceramic Dagger, Okina, Hate, Innodem, Destreza Incarnon, Ack & Brunt, Vinquibus, Korumm slam, Telos Boltace Stormpath, Vitrica glass, Zenistar disc aura, Orvius hover) are not in default plans. | 9.7 |
| EC-07-53 | Ability-combo exalted weapons (Shadow Clones, Landslide Fists, Shattered Lash, Whipclaw): ×M_ab (1:1, up to 12×) from recasts within 5 s; no heavies; Blood Rush, Weeping Wounds, Body Count, Power Spike work on that counter. | `W:Exalted_Weapon` lines 352-358; E-7.44; D-701 |
| EC-07-54 | Desert Wind: "melee and wave damage can be increased by the combo counter" (one page) is not applied to light attacks (Decision). | `W:Desert_Wind` line 24; OQ-718 |
| EC-07-55 | Zaws: 2H damage factor per strike (Balla, Ooltha, Sepfahn 1.00; Plague Keewar 0.85; Cyath, Plague Kripath, Rabvee 1.08; Dehtat, Mewan 1.09; Kronsh 1.07; Dokrahm 0.93); Plague strikes split added damage equally over their four types; class from strike and grip hand type. | `W:Zaw` lines 580-631; `M:Modular` lines 384-450; E-7.42; D-703 |
| EC-07-56 | Incarnon melee form: activation by a heavy at ≥ 6× (5×, 3×), Genesis +100 % melee damage for 180 s, native +3 range and +40 attack speed for 90 s; modelled as a selected state. | `W:Melee_Combo` lines 241-251; 9.10 |
| EC-07-57 | Follow-through (n-th enemy FT^(n−1); Volnus 0.6 → 0.4 with a stance; Ironbride 0.7; Magistar, Nami Solo, Dorrclave conditional) and punch-through are multi-target only. | `W:Melee` lines 814-940 |
| EC-07-58 | Finisher class multipliers and hit splits are data (table 9-E); finisher animations use attack-speed mods only, capped 33 % faster; the stealth formula (melee hits only, T-08), the finisher formula, the half elemental/physical rule and the valence half-damage bug are `04`'s. | `W:Finisher` lines 458-566; `W:Attack_Speed` line 30 |
| EC-07-59 | Non-melee status timing: a melee hit deals damage before its procs and before its combo gain (CD-04 extended to combo), so a hit never benefits from its own gain. | Decision; 6.3 |
| EC-07-60 | The schedule's heavy decisions use the expected-path count; MC damage uses the live count (6.9). | 6.9 |
| EC-07-61 | Corrupt Charge regenerates "to full within 2 seconds" after a heavy: consistent with 30 / 40 = 0.75 s. | `W:Corrupt_Charge` line 28 |
| EC-07-62 | Heavy slams have no wind-up; Furax Incarnon heavy-slam HAE; heavy slam Lifted duration by tier. | `W:Melee` lines 686-709 |
| EC-07-63 | Melee projectiles (glaive throws and explosions, Exodia Contagion) use two weapon-damage buckets; Vex Armor, Deathbringer, Amp count in both; Eclipse and Garuda twice (`04` 3.4). | `F:addmul` §12 |
| EC-07-64 | Corufell: the heavy fires a gunblade-like projectile (`Charged Projectile`, CO multiplying). | `W:Gunblade`; `04` catalogue |
| EC-07-65 | The heavy-attack combo multiplier and the class heavy multiplier are separate multipliers (`04` G membership). | `F:addmul` edge 19 |
| EC-07-66 | Stance CO effectiveness differs per input (0.25 Fracturing Wind block to 2.33 Mountain's Edge heavy; Exalted Blade 1.22-1.30; gunblade stances 1.00): imported as data r_in; shared-combo columns apply class-wide. | `W:Condition_Overload_(Mechanic)` §Stances; 7.6 |
| EC-07-67 | Riven melee lines owned here as hooks: Combo Duration (±), Initial Combo (positive only), Additional Combo Count Chance (positive only), Chance to Gain Combo Count (negative only), Heavy Attack Efficiency, spliced Heavy Attack Wind Up Speed, Melee Damage on Heavy Attack, Slam Attack Damage, Critical Chance for Slide Attack, Critical Chance (×2 on heavies). | `F:mods` §8 riven table; E-7.13 |
| EC-07-68 | Owned elsewhere: Melee Duplicate (tier-1 crits, slams trigger, Toxic Lash / Xata bug; `04` E-4.15), Melee Doughty (`02`), Melee Influence, Melee Afflictions, Melee Vortex, Melee Fortification (multi-target or no damage, `04`), Melee Careen ×2.5 vs Frozen (`04`), Exodia arcanes including the Contagion distance bonus (`04`). | `F:melee` §7 |
| EC-07-69 | Weapon-unique heavy slams (Sampotes 12 × 500 %, Tenet Exec 9 × 300 %) and Korumm's `Tactical Combo Slam Attack` are multi-target shockwaves; single target uses the data attack once. | 9.2; 9.7 |

---

## 12. Inspiration differences

Rows of the overview register that this document owns or co-owns, restated, and new rows `D-701` to `D-705`.

| ID | Topic | Inspiration behaviour | Game behaviour (source) | Engine | Toggle |
|---|---|---|---|---|---|
| D-022 | Stealth (04, 07) | melee non-crit ×7, crit `c + 7`, rank ignored (JS L8330-8336) | `1 + SDB + k(h·CDM − 1)` (`W:Stealth`) | `04` E-4.16 on every `melee_hit` instance | User (T-08) + Compat |
| D-062 | Light-attack combo (04, 07) | combo-scaled bonuses at 12× when the option is off (`combo_counter = 11`, JS L8219-8224, L9906-9910) | combo ramps from IC with hits (`W:Melee_Combo`) | expected-path ramp (6.9, E-7.18); T-10 fixes M_b | User (T-10) + Compat |
| D-063 | Combo range | inputs 1-13 for every weapon (`I:feat` quirk 5) | 12×; Venka Prime 13× (bonuses 12×); Dex Nikana 11× (E-7.10) | clamp to M_cap; bonuses to M_bcap | — |
| D-064 | Heavy combo model | spent every `int(0.8·N/T) + heavyHits` hits, never rebuilt, IC restored instantly, IC capped (JS L12151-12181) | gain per stance hit, spend with η ≤ 0.9, IC regenerates at 40/s (`W:Melee_Combo`, `W:Melee`) | E-7.13 to E-7.16 and plans of 8.1 | — |
| D-065 | Melee hit timing | `round(T·hits/cycle)` hits evenly over the window, cycle from the inspiration's own `default_MeleeStances` table (`total` per class), heavy cycle `heavy.total/speed + windUp` from the default table even with a stance, slams 1 hit per second (1/speed with a stance) with `statuses[0]` forced on every slam hit (JS L10137-10216, L12115-12150, L12436-12457); its stance mods carry their own durations and hit lists, which differ from the wiki in places (Blind Justice "Zatōs Creed" 3.0 vs 3.05 s and 6 vs 7 hits, "Heeding Call" 2.5 vs 2.45 s; `F:melee` §2.2) | stance Duration and per-strike lists from `Module:Stances/data`, wind-up and swing separate, slams are data attacks | E-7.3, E-7.30; strikes evenly spaced inside a loop (UNVERIFIED) | — |
| D-094 | Custom crit chance on heavies | added once per non-empty slot and twice per hit (`I:stat` Q7-Q8) | – | once (7.5) | — |
| D-701 | Ability combo scope | `abilityCombo` multiplies every weapon of type "Exalted Weapon" (Exalted Blade, Valkyr Talons …) when set, and the option's stored value (12) applies as soon as the custom stat is > 1 even with the checkbox off (JS L10377-10388; `I:feat` quirk 6) | only Blade Storm, Landslide, Shattered Lash, Whipclaw have an Ability Combo Counter (`W:Exalted_Weapon` lines 352-358) | E-7.44 on those four only | Compat |
| D-702 | Incarnon flat on heavies | the flat is part of the per-type maps (`addDmgToBase`, JS L13193-13205) and the heavy table multiplier `melee_mult` multiplies the whole hit (JS L10586, L10868) | flat ×M but not ×H (`W:Melee` line 681) | E-7.20b | Compat |
| D-703 | Zaw assembly | grip and link damage spread over the strike's types by `addDmgToBase`; no two-handed factor; no Plague equal split (JS L14573-14624, L5581-5593) | `(strike + grip + link) × Multi[hand].Base`; Plague strikes add `(total − base)/4` per type (`W:Zaw` lines 580-631, `M:Modular` lines 384-425) | E-7.42 | — |
| D-704 | Volatile glaive explosions | `glaive_explode`: +0.5 × the throw hit without its combo factor, inside the hit (JS L10370, L11376; mods L2310-2321) | a separate radial explosion at 50 % of the heavy explosion, own rolls, no combo multiplier, no spending, crit doubling always (`W:Volatile_Rebound` lines 28-31, `W:Glaive_(Weapon_Type)` line 45) | data attack `(Charged) Throw Bounce Explosion` (9.4) | — |
| D-705 | Initial Combo cap | IC capped at `20 · combo_counter` (220) (JS L12162-12166) | no documented cap (Melee Crescendo unbounded, `F:melee` §3.4) | uncapped (E-7.15) | Compat |

D-064 and D-065 have no compatibility switch: oracle scenarios must avoid melee heavy and stance timing (overview 4.6),
which in practice excludes every melee scenario from game-profile oracle tests except per-hit damage checks read from
the inspiration's "More details" log (`08`).

---

## 13. Open questions

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-701 | Heavy swing time per class and weapon; does `Duration` exclude the wind-up; when is the first damage frame; stanceless loop durations (overview OQ-021) | T_sw from data or 1.95 s median; Duration excludes wind-up; damage at the end of the wind-up; τ_class fallback | frame-timed recordings (60 fps) of two heavies per class at known attack speed and wind-up; the u/GoldPhos "true attack speed" dataset linked from `W:Attack_Speed` |
| OQ-702 | Does the heavy alternation reset after idle or after a light attack? | never resets | alternate a heavy with a pause of 5 s and with light attacks in between on a Dual Nikana (12× vs 6× damage numbers) |
| OQ-703 | Strike spacing inside a combo; restart of the light loop at input 1 after a heavy | uniform per strike; restart | frame timing of Guiding Light at AS 1.0; combo continuation after a heavy |
| OQ-704 | Combo gain rounding; Additional Combo Count Chance per hit or per point and above 100 %; the riven curse "Chance to Gain Combo Count"; gain of gunblade pellets, thrown hits, waves | half-up per strike; +1 per gaining hit; floor + Bernoulli; curse = probability 1 − x per gain event; 1 per pellet and thrown hit; waves as their entry | count the combo counter after single strikes of 50 %, 125 %, 250 % hits; True Punishment on a 300 % hit (expect 4 or 6); one Redeemer shot |
| OQ-705 | Is the raw combo count capped? | no cap | build past 400 on Xoris, heavy with η 0.5, read the remaining count |
| OQ-706 | Order of percent and flat combo-duration terms | percent on base, then flat | True Punishment + Body Count combo bar timing (14.5 s expected) |
| OQ-707 | Wind-up formula `W0/(1 + ΣWU)`; regeneration during wind-up | yes; yes | frame timing with Killing Blow and with Killing Blow + Swift Momentum; regeneration wait test with IC 140 |
| OQ-708 | Tennokai wind-up value | normal modded wind-up | frame timing of Tennokai heavies with and without Killing Blow |
| OQ-709 | Deterministic Tennokai spacing ceil(1/p) instead of a geometric wait | ceil(1/p) | none needed for game truth; compare EV with a stochastic-window MC variant |
| OQ-710 | Do Slam-typed stance entries hit the struck target (stanceless Hammer Neutral)? | no | stanceless Hammer neutral damage numbers on a single target |
| OQ-711 | Glaive cycle (flight, detonation, catch); slam and heavy slam loop period | data ChargeTime / FireRate; T_slam 1.0 s | timed glaive throw-detonate loops; slam loop timing at minimum height |
| OQ-712 | Heavy crit doubling on heavy slams | doubled | heavy slam crit rate with True Steel at a CC between 0.5 and 1 |
| OQ-713 | Exact internal base attack speed (k/60 or its f32 neighbour) | k/60 in f64 | none needed (< 1e-6) |
| OQ-714 | Meaning of stance `StatusMultiplier` | ignored | status rates on the Tranquil Cleave neutral 3rd input vs the 2nd |
| OQ-715 | `Finisher` and `Disarmed` proc names | crowd control only | proposal to `03` table 4.2 |
| OQ-716 | Shared-combo columns of the stance CO table apply to every stance of the class and to stanceless weapons | yes | CO numbers of a stanceless Dual Nikana heavy (expect 233 %); proposal to `04` 2.12 |
| OQ-717 | Heavy-only bonuses and crit doubling on heavy projectiles (table 9-C) | combo yes, heavy bonuses no | Syam waves with Killing Blow on and off |
| OQ-718 | Desert Wind light and wave damage scaled by combo? | no | Desert Wind hit numbers at 1× and 12× |
| OQ-719 | Zaw slam and heavy slam stats | synthesized (Leap × B, 3 × B, SC 0.10) | arsenal readout of a zaw's slam values |
| OQ-720 | Incarnon melee form: bucket of +100 % Melee Damage, unit of "+40 Attack Speed", activation cost | A; +0.40 ΣAS; form active for the window | arsenal readout in form; Pressure Point interaction test |
| OQ-721 | Exalted Blade finisher multipliers (table 12×/16× vs ability page 400 %/3200 %) | table 9-E | stealth finisher numbers with Exalted Blade |
| OQ-722 | Proposed toggle T-30 "melee plan" (8.1) with its parameters (K, heavy policy, T_slam, glaive throw kind) | scenario parameter | manager decision (overview 6.1) |
| OQ-723 | Heavy multipliers of multi-strike classes (Dagger, Dual Daggers, Dual Swords, Tonfa, Warfan), Sparring, Heavy Scythe, gunblades; κ_w for heavy 2 | class table + stance data (E-7.40); κ_w on both heavies | heavy damage numbers at 1× combo with no mods for Karyst, Dual Ether, Kronen Prime, Gunsen, Kogake, Hirudo, Hespar, Redeemer Prime, Prisma Machete, Tak & Lug (both heavies) |
| OQ-724 | Do Blood Rush, Weeping Wounds and Gladiator read the combo before or after a heavy spends it? | before | heavy crit rate at 12× with Blood Rush, η = 0, vs the two predictions |

---

## 14. Test vectors

Each vector lists everything needed to reproduce it; f64 unless stated; all values checked with `python3`.

**TV-07-01 Attack speed and loop timing (E-7.1 to E-7.3).** Nikana Prime FR_src 1.08 → FR_w = 65/60 =
1.0833333333; Guiding Light D_C = 2.6, N_C = 8, ΣS = 19.0.

| ΣAS | AS | T_loop | strikes/s | ΣS per s |
|---|---|---|---|---|
| 0 | 1.0833333333 | 2.4000000000 | 3.3333333333 | 7.9166666667 |
| 1.45 | 2.6541666667 | 0.9795918367 | 8.1666666667 | 19.3958333333 |

Without snapping (FR 1.08): 2.4074074 s and 0.9826152 s (`F:melee` §1.3 values).

**TV-07-02 Combo ramp of a light loop (E-7.13, E-7.30).** Inputs of TV-07-01, gains [3, 2, 2, 2, 2, 2, 2, 4], IC = 0,
p_acc = 0, D_cb = 5, first strike at t = 0, window [0, 20] closed.

| ΣAS | strikes | c at 20 s | M at 20 s | first time M ≥ 2, 3, 4, 8, 12 (after the strike at) | time-average M on [0, 20] | strike-average M_b(c⁻) |
|---|---|---|---|---|---|---|
| 0 | 67 | 159 | 8 | 2.4, 4.8, 7.2, 17.4, – | 4.550000 | 4.462687 |
| 1.45 | 164 | 389 | 12 | 0.979592, 1.959184, 2.938776, 7.102041, 11.265306 | 8.644898 | 8.591463 |

Time average: M(c) right after each strike, held until the next strike (until 20 s after the last). Blood Rush check
(CC0 0.28, True Steel 1.20, Blood Rush 0.40): strike-average crit chance 1.003821 (ΣAS 0) and 1.466244 (ΣAS 1.45).

**TV-07-03 Heavy damage, crit doubling, E-4 quantization (E-7.20b, 7.5).** Fragor Prime IMP 189, PUN 40.5, SLA 40.5;
H = 6; A = 2.85; no elements; q = (22, 5, 5)/32; F = 0, G = M only, Part = 1. Y = 6237.
M = 8: per type 34,303.5 / 7,796.25 / 7,796.25, total 49,896; M = 12: total 74,844. CC = 0.40 × (1 + 1.20 + 1.20) = 1.36;
code = 320, Q = 2.5006105900 (f32), CDM = Q × 1.9 = 4.7511601210; E[c] = 6.1015777645; E[damage] at M = 8: 304,444.3241,
k = 1: 237,063.8854, k = 2: 424,231.7708; at M = 12: E 456,666.4862. CC without doubling: 0.88.

**TV-07-04 Incarnon flat on a heavy (E-7.20b).** B_ref = 200 (one type), X = 30, A = 1.65, H = 6, M = 4, no CO:
Y_heavy = (6·200 + 30) × 2.65 = 3,259.5; ×M = 13,038.0 (the inspiration's form `(B + X)·H·(1 + A)·M` gives 14,628.0,
D-702). A 300 % light strike of the same weapon: Y = 3 × 230 × 2.65 = 1,828.5.

**TV-07-05 Heavy loop (E-7.16, E-7.30).** c(0) = 220 (T-09 12×), IC = 140, η = 0.5, W = 0.75, T_sw = 1.95, D_cb = 2.5,
ρ_IC = 40, FR_w = 48/60, window 20 s, executions emitted while τ_ex ≤ 20.

| ΣAS | policy | executions (time, c⁻, M) | count | ΣM |
|---|---|---|---|---|
| 0.55 (AS 1.24) | full_regen and back_to_back | (0.75, 220, 12), (3.072581, 140, 8), (5.395161, 140, 8), (7.717742, 140, 8) … last (19.330645, 140, 8) | 9 | 76 |
| 2.0 (AS 2.4) | full_regen | (0.75, 220, 12), (2.3125, 140, 8), (4.0625, 140, 8), (5.8125, 140, 8) … last (19.8125, 140, 8) | 12 | 100 |
| 2.0 (AS 2.4) | back_to_back | (0.75, 220, 12), (2.3125, 140, 8), (3.875, 132.5, 7), (5.4375, 128.75, 7) … last (19.5, 125.0073242, 7) | 13 | 97 |
| 0.55, η = 0.9 | back_to_back | (0.75, 220, 12), (3.072581, 198, 10), (5.395161, 178.2, 9), (7.717742, 160.38, 9), (10.040323, 144.342, 8), (12.362903, 140, 8) … | 9 | 80 |

**TV-07-06 Combo duration and Power Spike (E-7.12, E-7.14).** (D0, Σpct, Σflat) → D_cb: (5, 0, 0) → 5; (5, −0.5, 12) →
14.5; (5, −1.0, 0) → undefined (no gain); (5, −1.0, 10) → 10; (5, 0, 28) → 33; (6, 0, 0) → 6; (5, −1.0, 0.05) → 0.1.
Power Spike δ = 5, D_cb = 5, IC 0, from c = 220 idle: at 4.99 s 220 (12×); 5 s 215 (11×); 100 s 120 (7×); 219.99 s 5;
220 s 0 (44 expiries); with D_cb = 17 the count reaches 0 at 748 s.

**TV-07-07 Zaw assembly (E-7.42).**

| Strike, grip, link | B_zaw | B0_t | FR_src | CC0, SC0 | class |
|---|---|---|---|---|---|
| Balla, Peye (1H, −4), Jai (−4, +0.083) | 216.0 | IMP 10.8, PUN 129.6, SLA 75.6 | 1.166 → 70/60 | 0.18, 0.18 | Dagger |
| Plague Keewar, Plague Bokwin (2H, +7, factor 0.85), Vargeet Ruhang II (+28, CC +0.07, SC −0.04, −0.133) | 289.85 | IMP 83.9625, PUN 52.9625, SLA 86.9625, VIR 65.9625 | 0.717 → 43/60 | 0.25, 0.18 | Staff |
| Plague Keewar, Plague Akwin (1H, −2), Jai (−4) | 300.0 | IMP 86.5, PUN 55.5, SLA 89.5, VIR 68.5 | 1.000 → 60/60 | 0.18, 0.22 | Scythe |

**TV-07-08 Stance strike with physical bonus (E-7.20, `01` E-1.9).** Nikana Prime (IMP 9.9, PUN 9.9, SLA 178.2), A = 1.65,
P_SLA = 0.90.

| Strike | S | S^phys_SLA | r (IMP, PUN, SLA) | q | Y | W per type (non-crit, F = 0) | Σ |
|---|---|---|---|---|---|---|---|
| Guiding Light 1 | 3 | 1.0 | 0.05, 0.05, 3.42 | 2/32, 2/32, 109/32 | 1574.1 | 98.38125, 98.38125, 5361.778125 | 5558.540625 |
| Guiding Light 3 | 2 | 0 | 0.05, 0.05, 1.71 | 2/32, 2/32, 55/32 | 1049.4 | 65.5875, 65.5875, 1803.65625 | 1934.83125 |

**TV-07-09 Gunblade loop (E-7.45).** Redeemer Prime, no stance: stanceless Gunblade Neutral (D = 2.5; inputs [100 %
Ranged], [200 % × 2], [100 % Ranged × 2]); FR 0.917 → 55/60; Ranged Attack 80 Blast × MS 10; Normal Attack IMP 21.2,
PUN 42.4, SLA 148.4 (q = 3/32, 6/32, 22/32, Σ = 31/32); no mods, d = 0. T_loop = 2.7272727 s; per loop: 3 shots × 800 =
2400 and 2 melee strikes × 410.75 = 821.5, total 3221.5 HP (1181.2167 HP/s); combo gain 34 per loop (30 pellets + 2 + 2);
Tennokai events per loop 4 (input 1 shot, input 2, two shots of input 3).

**TV-07-10 Heavy table resolution (E-7.40).** Machete heavy 1: stance entries 125 % × 2, 125 % (Bleed), 125 % (direct sum
500); table 9-A 6× → σ_1 = 1.2, strikes 1.5, 1.5, 1.5 (Bleed), 1.5. Machete heavy 2: [500 % Knockdown], table 5× → σ = 1.
Sparring heavy 1: direct [500 %] (+400 % Slam, excluded) → σ = 2.4 → one strike H = 12.0 with no proc. Sparring heavy 2:
stance 4 strikes vs table breakdown 5 strikes → table: 3.0, 3.0, 3.0, 3.0 (Bleed), 3.0 (Bleed). Hirudo (κ = 6.5/12 =
0.5416667): heavy 1 H = 6.5, heavy 2 five strikes of 1.625.

**TV-07-11 Combo gain rounding (E-7.13).** Dmg % → g: 40 → 0, 50 → 1, 100 → 1, 125 → 1, 150 → 2, 166 → 2, 180 → 2,
185 → 2, 215 → 2, 220 → 2, 225 → 2, 250 → 3, 450 → 5, 555 → 6.

**TV-07-12 Stance normalisation (E-7.0).** Blind Justice Forward Block input 3: Dmg [200, 400, 100], Types [Sweep, "",
Slam], Procs map {1: "", 3: ""} → procs [[], [], []]; strikes 3 (the third excluded from damage as Slam). Hysteria Neutral
input 5: Dmg [300], Procs [Slash, Stagger] → one strike with procs [Slash, Stagger]. Gaia's Tragedy Forward Block input
1: Dmg [200, 100], Hits [2, 1], Types ["", Slam], Procs [Knockback], SlashMultiplier [2] → strikes (200 %, Knockback,
S^phys_SLA 1.0) ×2 and one Slam strike (100 %, no proc, factor 1; no single-target damage). Bullet Dance Neutral input 5:
Dmg [300], Hits [3], Procs [Knockback], Slash/Impact/PunctureMultiplier [1.25, 1] → 3 strikes of 300 % with Knockback
and S^phys = 0.25 on IMP, PUN and SLA (the extra element dropped). Blind Justice Neutral: N_C = 8.

**TV-07-13 Build and spend (E-7.30, build_spend(12)).** TV-07-02 inputs with ΣAS 1.45, Nikana heavy 500 % (Bleed,
Knockdown), η = 0, IC = 0, W = 0.5, T_sw = 1.95 (→ 0.734694 s). The 93rd strike (index 92, t = 11.265306) brings c̄ to 220;
heavy input at 11.387755, execution at 11.887755 with c̄⁻ = 220, M_h = 12, n_anim = 10 (59 light inputs); light restarts
at 12.622449; strikes in [0, 20]: 154 (93 + 61), ΣS = 364; one heavy; c̄ at 20 s = 144.

**TV-07-14 Tennokai plan (E-7.17, E-7.30).** TV-07-02 inputs with ΣAS 1.45 (Δ = 0.122449), Discipline's Merit (n_tk = 4),
Nikana heavy, W = 0.5, T_sw/AS = 0.734694, IC 0. After strikes 0-5 (inputs 1-4) the heavy input is at 0.734694,
execution 1.234694 with c̄ = 13 (M 1), no spending; light restarts at 1.969388. Executions (time, c̄, M): (1.234694, 13, 1),
(3.204082, 26, 2), (5.173469, 39, 2), (7.142857, 52, 3), (9.112245, 65, 4), (11.081633, 78, 4), (13.051020, 91, 5),
(15.020408, 104, 6), (16.989796, 117, 6), (18.959184, 130, 7); 63 light strikes; c̄ at 20 s = 137.

**TV-07-15 Fallback durations (E-7.3, E-7.5).** Stanceless Nikana Neutral (2 strikes): D_C = 2 × 0.4625 = 0.925 s;
stanceless Sword Neutral (4 strikes): 4 × 0.45 = 1.8 s; stanceless Polearm (4 strikes): 0.975 s. Heavy swing fallback
τ_heavy = median(2.2, 1.65, 3.3, 1.0, 2.1, 1.8) = 1.95 s.

**TV-07-16 Exalted Blade wave falloff (E-7.43).** φ_w at d = 0, 2.5, 35, 70, 80 m: 1.0, 0.9642857, 0.5, 0, 0.
