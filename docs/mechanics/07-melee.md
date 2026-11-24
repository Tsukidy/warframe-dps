Status: reviewed (spec)

# Damage-calculation spec, part 07: melee

This document defines how a melee attack turns its stats into a timed sequence of hits with per-hit multipliers,
forced procs and combo state, and every damage rule that exists only for melee. Both evaluators of
`00-overview.md` section 4 consume the same deterministic melee schedule defined here (overview 4.1: "the schedule
never depends on random draws"). It follows the template of overview 0.4.

Related documents: [00-overview.md](00-overview.md) (notation, master formulas E-1 to E-14, register, toggles T-09,
T-10, T-11, T-30, T-31, time grid CD-14), [01-damage-composition.md](01-damage-composition.md) (base vector, stance
physical bonus inside E-1.9, Incarnon flat, base bucket A including the heavy-only members),
[02-crit-status-multishot.md](02-crit-status-multishot.md) (crit chance buckets, the heavy doubling hook, proc rolls
and proc-name mapping), [03-status-effects.md](03-status-effects.md) (Lifted and Knockdown durations, DoT ticks),
[04-conditional-bonuses-and-buffs.md](04-conditional-bonuses-and-buffs.md) (CO family and the stance formula E-4.6,
`G`/`G_F` membership, melee-projectile buckets, effect values, stealth E-4.16, finishers E-4.17, Melee Duplicate
E-4.15), [05-enemies.md](05-enemies.md), [06-ranged-firing-model.md](06-ranged-firing-model.md) (falloff Φ(d), the
shared rules for timing stats under conditional buffs, 5.9), `08-evaluation-and-outputs.md` (how the schedule is
walked), `09-build-rules.md` (stance slot legality, Incarnon tier choice).

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
- [Review notes](#review-notes)

Citation keys are those of overview 0.2. Wiki pages are cited as `W:<Page>` with line numbers of the cached raw copy
in `research/external/cache/wiki/` where given. `M:Stances` and `M:Modular` are the renderer modules
`research/external/cache/wiki-modules/raw/{Stances,Modular}.lua`; `D:Stances` is `Module:Stances/data` and `D:<weapon>`
a record of `Module:Weapons/data/melee` or `/modular`, both as converted by mlua into `research/external/cache/poc/out/`
(module revisions of 2026-10-02 and 2026-10-22). `PE:<weapon>` is a record of the DE Public Export
`research/external/cache/public-export/manifests/ExportWeapons_en.json` (build label 2026.10.28). `R:Stances` is the
revision history of `Module:Stances/data` and three old revisions (2594921 of 2023-08-06, 2765569 of 2026-04-15,
2788398 of 2026-06-24), fetched from the wiki API into `research/external/cache/web/stances-history/`. Counts and
medians quoted from these files were computed with python3 in this session; the scripts are not part of the
repository.

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
falloff Φ(d) (`06`), the evaluation algorithms (`08`), stance and Incarnon legality (`09`). Archmelee weapons
(Archwing melee) and the Necramech exalted Ironbride have build slots in `09` but are out of scope (overview 1.3,
adopted from OQ-737); this document does not schedule them.

**Stages.** No stage of overview 3.1 is owned outright. This document supplies: the melee hit multiplier S_j and the
heavy-attack base vector to ST-1 (section 7.2); the stance physical bonus `stance_phys[t]` to ST-2 (`01` E-1.9); the
melee members of `G` (combo multiplier, Slam Damage, ability combo) to ST-5; the wave falloff factor Φ_w to `G_F`
(ST-4); the instance flags (heavy, slide, Tennokai, slam, radial) that gate buckets in `01`, `02`, `04`; the forced
procs of stance and heavy hits to ST-10 (`02` rolls them); zaw assembled stats to ST-1 and E-4 to E-8; and the schedule
consumed by `08`. S_j and H_j are ST-1 base factors, not `G` members, for every CO behaviour: overview 2.4
(`stance_hit_mult`, `mult_other`), 3.1 (ST-1), 3.1a and 3.3, adopted from OQ-726, and `04` E-4.6 (section 7.2).

**Melee multishot.** Melee strikes have no multishot: MS ≡ 1 for every light, slide, aerial, wall and heavy strike and
for slams (`DS:wiki` §3: "`Multishot` … Absent on melee attacks"; 245 of 248 Normal Attacks lack the field, 3 carry 1).
The only sub-instances j > 0 of a melee hit are gunblade pellets (9.5) and the Count copies of table 9-C
projectile attacks (`Multishot` 6 to 18). Custom multishot (T-26 `p.q`) therefore acts on those attacks only (D-707);
`08` E-8.0 names exactly these two as the melee sub-instances (adopted, OQ-738).

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
| Bucket sums: attack speed ΣAS, wind-up speed ΣWU (Valkyr Talons: also those of the regular melee weapon, E-7.4), combo duration (percent Σpct and flat Σflat), Initial Combo, heavy attack efficiency ΣHAE, additional combo count chance p_acc, Relentless Combination chance p_RC, riven "Chance to Gain Combo Count" curse x_cur, Slam Damage ΣSlam, Tennokai chance modifiers, Volatile Rebound / Volatile Quick Return explosion chance p_vol | – | `04` (values, condition class T-02) | timing stats follow `06` 5.9 |
| External combo gain rates: Tandem Bond, Recompense, Storm of Ukko, Opening Slam, Vexoric Tauron Strike (`W:Melee_Combo` §Other Sources of Combo Count) | – | `04` (values); class (c) user rate, default 0 | added to c̄ as a constant rate (6.9) |
| Expected Slash procs π_i of a melee instance i, given M_b(c̄⁻) (rolled, forced, Hunter Munitions, Internal Bleeding) | π_i | `02` | Relentless Combination only (E-7.13b) |
| Slash tick count N_SLA and first-tick delay δ_SLA | – | `03` (E-13) | Relentless Combination only |
| Target status immunities | – | `05` | through π_i |
| Fixed stance of exalted melee (`stance_fixed`) | – | `09` E-9.2, E-9.8 | E-7.0b |
| T-01 `t_win`, T-09 heavy and start multiplier, T-10 light-attack combo multiplier, T-11 ability combo, T-20 Strength, T-25 zaw grip and link, T-27 distance, T-31 exalted ability rank | – | overview 6.1 | |
| Melee plan of the attack (section 8.1) | – | T-30 (overview 6.1, adopted from OQ-722) | default per attack kind |

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
| `stance_dur` | D_K | s | duration of combo K at attack speed 1.0 (data or fallback, E-7.3) |
| `stance_strikes` | N_K | count | number of strikes of one loop of combo K (Σ Hits over all entries) |
| `stance_loop_time` | T_loop | s | duration of one loop of combo K at the current attack speed, `D_K / AS` (E-7.3) |
| `melee_strike_dt` | Δ | s | time between two strikes of a light loop, `D_K / (AS·N_K)` |
| `melee_gap_max` | gap_max | s | largest cyclic time between consecutive gaining strikes of a loop (E-7.37) |
| `stance_tau_class` | τ_class | s | fallback time per strike at AS 1.0 for a class (E-7.3) |
| `stance_sl_factor` | f_sl | factor | stanceless-to-stance duration factor of the fallback (E-7.3), 1.159091 today |
| `melee_cycle_time` | T_cyc | s | period of the one-strike `cycle_loop` and `data_attack_loop` plans (8.1) |
| `heavy_mult` | H_j | factor | hit multiplier of strike j of a synthesized heavy attack (E-7.40) |
| `heavy_scale` | σ_h | factor | scale that maps the stance heavy entries to the class heavy total (E-7.40) |
| `heavy_strikes` | n_h | count | number of strikes of heavy h (E-7.30) |
| `heavy_index` | h | 1 or 2 | first or second heavy attack of the alternating pair |
| `heavy_combo_mult` | M_h | factor | combo multiplier `M(c⁻)` of one heavy execution, shared by its strikes (E-7.16) |
| `heavy_windup_base`, `heavy_windup_sum`, `heavy_windup` | WU0, ΣWU, WU | s, bonus, s | wind-up (E-7.4) |
| `tennokai_windup`, `tennokai_windup_bonus` | WU_tk, s_tk | s, bonus | wind-up of a Tennokai heavy and its undocumented speed bonus (E-7.4) |
| `heavy_swing` | T_sw | s | heavy swing time at AS 1.0 (data or fallback, E-7.5) |
| `heavy_dmg_time` | t_dmg | s | time from execution to the damage frame at AS 1.0, used only by `heavy_cancel` (8.1, OQ-701) |
| `heavy_swing_fallback` | τ_heavy | s | median per-heavy swing time over the heavy combos with `Duration` (E-7.5) |
| `heavy_weapon_factor` | κ_w | factor | per-weapon heavy multiplier factor (E-7.41), 1 by default |
| `melee_data_scale` | s_j | factor | stance scale of a data attack instance: `Dmg_j/100` for gunblade shots and `scaled` 9-C rows, else 1 (E-7.20c) |
| `combo_tier_size`, `combo_cap`, `combo_bonus_cap` | k_tier, M_cap, M_bcap | count, factor, factor | 20, 12, 12 by default (E-7.10) |
| `combo_mult_bonus` | M_b | factor | combo multiplier read by combo-scaled bonuses (E-7.11) |
| `combo_dur_base`, `combo_dur_raw`, `combo_dur` | D0_cb, D_raw, D_cb | s | combo duration before mods, raw modded value, final value (E-7.12) |
| `combo_dur_pct`, `combo_dur_flat` | Σpct, Σflat | bonus, s | percent and flat combo-duration sums (E-7.12) |
| `combo_initial` | IC | count | Initial Combo (E-7.15) |
| `combo_start_mult` | M_start | factor | start multiplier set by T-09 (E-7.31) |
| `combo_eff_sum`, `combo_eff` | ΣHAE, η | bonus, fraction | heavy attack efficiency sum and its clamp to [0, 0.9] (E-7.16) |
| `combo_regen_rate` | ρ_IC | count/s | Initial Combo regeneration, 40 |
| `combo_gain` | g | count | combo points a hit adds before the additional-chance roll (E-7.13) |
| `combo_acc` | p_acc | fraction | additional combo count chance (additive bucket) |
| `combo_extra` | x_cb | count | extra points of one gaining instance, `floor(p_acc) + Bernoulli(frac(p_acc))` (E-7.13) |
| `combo_curse` | x_cur | fraction | riven "Chance to Gain Combo Count" curse, as a positive fraction (E-7.13) |
| `combo_gain_prob` | p_gain | fraction | probability that a gain event happens, `max(0, 1 − x_cur)` (E-7.13) |
| `combo_rc` | p_RC | fraction | Relentless Combination chance per melee Slash proc and per tick |
| `combo_rc_procs` | π_i | count | expected Slash procs of melee instance i, from `02` (E-7.13b) |
| `combo_decay` | δ_PS | count | Power Spike decay per expiry (20 / 15 / 10 / 5) |
| `combo_expected` | c̄ | count | expected-path combo count stored in the schedule (6.9) |
| `combo_spend_tier` | M_trig | factor | multiplier at which `build_spend` spends (8.1) |
| `melee_horizon` | H_end | s | schedule horizon `max(t_win, t_ttk_max)` (E-7.30) |
| `slam_dmg_sum` | ΣSlam | bonus | Slam Damage bucket; factor `M_slam = 1 + ΣSlam` |
| `slam_period` | T_slam | s | period of `slam_loop` and `heavy_slam_loop` (8.1) |
| `ability_combo` | M_ab | factor | ability combo multiplier of the four ability-combo exalted weapons (E-7.44) |
| `wave_falloff` | Φ_w | factor | Exalted Blade wave falloff, `max(0, 1 − d/70)` (E-7.43) |
| `tennokai_chance`, `tennokai_every` | p_tk, n_tk | fraction, count | Tennokai window chance and the deterministic input spacing (E-7.17) |
| `tennokai_window` | T_tk | s | Tennokai window, 2 s (4 s with Opportunity's Reach) |
| `heavy_animosity_stacks` | n_anim | count | Melee Animosity stacks carried by a heavy (normal inputs since the last connecting heavy, ≤ 10) |
| `glaive_throw_ct` | CT_g | s | modded charge time of a charged glaive or thrown-hammer throw (E-7.6) |
| `glaive_volatile_chance` | p_vol | fraction | Volatile explosion chance per bounce (0.25 per rank, 1.00 at max; E-7.36) |
| `glaive_throw_index` | n_thr | count | index of a throw in the current run of consecutive throws, for Power Throw (`04` 3.4) |
| `zaw_total` | B_zaw | HP | assembled zaw base damage (E-7.42) |
| `melee_plan` | P | enum | `light_loop`, `heavy_loop`, `build_spend`, `tennokai`, `slide_loop`, `slam_loop`, `heavy_slam_loop`, `glaive_cycle`, `cycle_loop`, `data_attack_loop` (8.1) |

Symbols that would clash with overview symbols are renamed here: the extra combo points are x_cb (not the Incarnon
flat X), a combo is K (not the CO sum C), the zaw parts are Z_s, Z_g, Z_l (not S_j or `G`), the horizon is H_end (not
H_j or H_w), the wind-up is WU (not W_t(k)), the stance mod of E-7.0b is St, and the wave falloff is Φ_w (a falloff
like Φ(d), not the type modifier φ_t or the forced procs φ_s).

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
  0. Apply the pipeline's override table (curated fixes of known data errors, each with a data-report line).
     Current entry: High Noon Block input 1 Types ["Slam"] → ["", "Slam"] (Decision, UNVERIFIED, below).
  1. Missing array → defaults above.  Array shorter than n → pad with defaults.
  2. Procs given as a map with numeric-string keys (a Lua table with holes) → list; missing indices = no proc.
  3. Procs longer than n → the extra names are appended to entry n's proc list.
     Any other array longer than n → extra elements dropped.
  4. Proc list of an entry: "" removed; a single string becomes a one-element list.
  5. S^phys_t of an entry = (factor_t − 1) for t ∈ {SLA, IMP, PUN}.
  6. Every entry expands into Hits[e] strikes with identical fields, in entry order; the strike list of a combo is the
     concatenation over inputs. N_K = total strike count (all types, Slam and Ranged included).
```

Data cases handled by E-7.0 (cached module, every input whose arrays differ in length from `Dmg`, recounted this
session): Blind Justice Forward Block input 3 `Procs` with a hole (the unquoted `Knockdown` evaluates to nil; `03` 4.2
maps it to "no proc"); Bullet Dance Neutral input 5 physical arrays of length 2 for 1 entry; Crimson Dervish Neutral
and Block input 3 and Gaia's Tragedy Block input 3 and Forward Block inputs 1-2 short `SlashMultiplier`; Gaia's Tragedy
Forward Block input 1, Rending Crane Neutral input 2, Seismic Palm Neutral input 3 and Twirling Spire Forward Block
input 1 short `Procs`; Hysteria Block input 1, Forward input 4 and Neutral input 5 and Ravenous Wraith Neutral input 3
`Procs` of length 2 for 1 entry (step 3 gives `[Slash, Stagger]` and `[Bleed, Big Stagger]` on that entry); Serene
Storm Heavy input 2 `Procs` `[Lifted]` for 2 entries (Lifted on the 4-hit entry only); Galeforce Dawn and Heavy Scythe
Slide empty `Procs`. Short `Types` arrays: Galeforce Dawn Forward Block input 2 (`["Slam"]` for 4 entries: padding makes
only the 100 % Knockback entry a Slam; the 200 % Knockback, 100 % Bleed and 300 % entries are direct strikes, as in the
wiki renderer, `M:Stances` line 287, which counts `Types[j] = nil` as direct); Vulpine Mask Neutral input 4 and
Forward Block input 2 (`["Thrust"]` for 2 entries: the padded entry is a plain strike, no effect on damage); High Noon
Block input 1 (`["Slam"]` for `Dmg [300, 100]`). Padding would make the 300 % Stagger entry the Slam and the 100 %
Knockback entry a direct strike; in every other High Noon and Bullet Dance pair the Slam is the second entry with
physical factor 1, which is also this input's second entry (factors `[1.25, 1]`), so the override of step 0 sets
`["", "Slam"]` (Decision, UNVERIFIED, OQ-730). Crimson Dervish `Block` is byte-identical to its `Neutral` (`F:melee`
§2.2): kept as data. Dmg values not multiple of 50 exist (40, 125, 166, 180, 185, 215, 220, 225, 555): kept exact.
`StatusMultiplier` (Blind Justice, Decisive Judgement, Tranquil Cleave; values 1.25, 1.5, 2) is stored and not applied
(Decision, UNVERIFIED, OQ-714); `02` EC-02-53 waits on this document.

Proc names map to status types by `03` table 4.2 (with `02` 4.7.2). `Finisher` (Caustacyst heavy projectile; opens
the target to a finisher) and `Disarmed` are crowd control only; `03` table 4.2 now lists both so (OQ-715 settled).
Crowd-control names (Ragdoll, Knockback, Stagger, Big Stagger, Stun, Lifted, Knockdown) stay on the hit record with
the hit's time (8.2). The ragdoll target state that `02` E-2.13/E-2.14 needs (`02` OQ-219) is defined by `03` §8.10
(E-3.13: entered by a forced `Ragdoll` name or a Lifted proc on a Lifted target, 3 s, `03` OQ-326), which overview 7.1
and 7.2 make its owner (adopted from OQ-734); this document supplies the events. On a single target, stance Ragdoll
procs (78 in the stance data) and the slide and heavy Lifted procs are its main sources.

### 4.2 Resolving the combo tables of a weapon

```text
E-7.0b Combo table of combo type K ∈ {Neutral, Forward, Block, Forward Block, Slide, Aerial, Wall, Finisher, Slam,
                                       Heavy, Heavy Slam} for weapon w with equipped stance mod St (or none)
  exalted melee (w.Class = "Exalted Weapon"): St = the weapon's fixed stance, always equipped (09 stance_fixed: the
          stance mod whose M:Mods Type equals w.Family; its D:Stances table has the stance's name: Exalted Blade,
          Primal Fury for Iron Staff, Serene Storm for Desert Wind, Hysteria for Valkyr Talons, Ravenous Wraith for
          Shadow Claws; Diwata's Razorwing and the four pseudo-exalted stances have no D:Stances table)
  U = the UniqueToWeapon table whose key equals w's name, else w's data Family; none otherwise
  if U has K and U.K has no FromWeapon flag:                      use U.K   (weapon-unique override)
  elif St is equipped, St has K and St.K has no FromWeapon flag:   use St.K  (the four stance-specific combos, and
                                                                            shared combos the stance defines itself)
  elif w is exalted melee and St has K:                            use St.K  (FromWeapon accepted: the class
                                                                            "Exalted Weapon" has no stanceless table)
  elif the stanceless table T of w's Class exists and has K:       use T.K
  else: combo K does not exist for w (Ravenous Wraith Finisher and Wall; Bayonet Neutral, Forward, Block, Forward
        Block; every combo of Diwata and of the pseudo-exalted weapons); the plans of 8.1 name the fallback
  Conclave-only stances are never used.  Heavy Slam exists only as a unique override (Sampotes, Tenet Exec).
  Class of a FromWeapon combo taken from an exalted stance (E-7.40 rule 2, table 9-A): the stance's WeaponType
  (Exalted Blade's Heavy is the Sword "Perfect Cut" copy, class Sword).
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
"shared by class" statement for these (Decision). Exalted weapons carry the data `Class` "Exalted Weapon" (21 records)
and always use their fixed stance (`09` E-9.2 `stance_fixed`, E-9.8; `W:Exalted_Weapon` line 94: "associated stance
mod that cannot be removed"). The link from weapon to stance cannot come from the stance's `WeaponType` (Exalted
Blade's is "Sword"), so it goes through `09`'s rule (stance mod Type = weapon Family). Resolution examples: TV-07-21.

Stance switching mid-combo is seamless since U26 (`W:Stance` line 16), so every combo is evaluated as an independent
loop that restarts at its first input (Decision).

### 4.3 Melee weapon fields used

| Field | Use | Notes |
|---|---|---|
| `Normal Attack` | the attack of light, slide, aerial, wall and heavy hits (its damage map, CC0, CDM0, SC0) | heavies and slides have no separate crit or status data (W:Melee §Heavy Attack Damage) |
| `FireRate` of `Normal Attack` | FR_src for E-7.1 when the weapon does not join to PE | |
| `Slam Attack`, `Heavy Slam Attack` | the two slam attacks: damage already 2× and 3× weapon damage, own SC, `ForcedProcs`, `Falloff`, `Range` | Nikana Prime 396 = 2 × 198, 594 = 3 × 198 (`D:Nikana Prime`) |
| `WindUp` | WU0 | present on all 250 records |
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
few tenths of a second" (`M:Stances` line 557). Older module revisions do not fill the gap: revision 2594921
(2023-08-06, `R:Stances`) has the same 11 stanceless Neutral durations as the current one. Fallbacks: E-7.3 (loops) and
E-7.5 (heavy swing); overview OQ-021; the inspiration cross-check is D-708.

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
E-7.3  T_loop(K) = D_K / AS ;   Δ = T_loop / N_K
       D_K = Duration of combo K                                                    (data)
           | Duration of another combo of the same table with an identical strike list (data)
           | N_K · τ_class · f_K                                                     (fallback, UNVERIFIED)
       τ_class = median over every PvE stance-specific combo with Duration of the weapon's class (Neutral, Forward,
                 Block, Forward Block of non-stanceless, non-unique, non-Conclave tables) of Duration / N_combo;
                 classes with no such combo: the median over all classes (0.40 s per strike in the current data)
       f_K     = f_sl for every combo of a stanceless table, 1 otherwise
       f_sl    = median over the stanceless tables whose Neutral has a Duration of Duration / (N_Neutral · τ_class)
                 (1.159091 in the current data: 11 classes, range 0.877-1.466)
```

`W:Attack_Speed` lines 13-16: `Modified Animation Length = Base Attack Animation Length / [Base Weapon Attack Speed ×
(1 + Attack Speed Bonuses)]`; the bonuses are additive with each other (`F:melee` §1.1: Arcane Strike "additive to mods
like Fury", Excalibur passive "additive to mods such as Pressure Point and Fury"). The 0.05 floor mirrors `06` D-603
(Decision; Spoiled Strike −0.20 is the only negative source in the catalogue). Attack speed speeds stance combos and the
heavy **swing**, not the heavy wind-up and not glaive or Wolf Sledge charged throws (`W:Attack_Speed` line 30;
`W:Melee` line 705: "Increasing melee attack speed does not reduce the wind-up time; rather, it reduces the interval
between Heavy Attacks"). Slide and aerial attacks are sped up as well (`F:melee` §1.2, from `W:Attack_Speed`
§Mechanics); wall attacks are included by inference (Decision). Finisher animations use only attack-speed bonuses, not
the weapon's base attack speed, capped at 33 % faster at +50 % (`W:Finisher` line 14: "… and the weapon type influence
the animation speed of the finisher. This caps at a 33% faster animation with +50% attack speed. The weapon's base
attack speed does not affect animation speed"; finisher mode only, 9.11).

Current τ_class values (s per strike at AS 1.0; this session, from the cached module): Assault Saw 0.82875, Dagger
0.355, Claws 0.348571, Dual Daggers 0.30875, Fist 0.340833, Glaive 0.425, Hammer 0.54125, Machete 0.570833, Nikana
0.4625, Polearm 0.24375, Rapier 0.428571, Sparring 0.335833, Staff 0.5125, Sword 0.45, Tonfa 0.257323, Warfan
0.275538; global median 0.40 over 250 combos. The pipeline recomputes them and f_sl from the module (they change when
the wiki adds data). Fallback durations are marked in the result so the UI can flag them.

The factor f_sl exists because stanceless loops are slower per strike than stance loops: for the 11 classes whose
stanceless Neutral has a Duration, Duration / (N · τ_class) is 0.877 (Blade and Whip), 1.000 (Dual Nikanas), 1.031
(Scythe), 1.106 (Sword and Shield), 1.152 (Whip), 1.159 (Two-Handed Nikana), 1.171 (Heavy Scythe), 1.226 (Gunblade),
1.401 (Dual Swords), 1.438 (Heavy Blade), 1.466 (Nunchaku). The inspiration's own stanceless table agrees with those 11
wiki durations within 0.05 s in 10 cases, and for 14 of the 16 classes without a wiki duration it is slower than the
unscaled fallback (geometric-mean ratio 1.51, median 1.63; Claws 2.65, Polearm 2.21, Tonfa 2.07, Nikana 1.91, Hammer
1.85): D-708 records the comparison. The inspiration is a cross-check only, not a data source (CLAUDE.md); with f_sl
the median absolute log deviation from it over those 16 classes falls from 0.49 to 0.35. The fallback remains
UNVERIFIED (OQ-701, overview OQ-021).

Strike timing inside a loop (Decision, UNVERIFIED, OQ-703): the strikes of a light phase that starts at t0 land at
`τ_0 = t0` and `τ_{k+1} = τ_k + D_K / (AS(τ_k) · N_K)` (k = 0, 1, …): strikes are evenly spaced over the loop regardless
of inputs and entries, and the loop repeats without a gap. The wiki gives only the total length per combo. With
attack-speed changes inside a phase (`simulated` deterministic triggers) the interval after strike k uses AS at τ_k,
the evaluation point of `06` 5.9 for the interval after a shot.

### 5.3 Heavy wind-up and swing

```text
E-7.4  WU = WU0 / (1 + ΣWU)                    ΣWU additive (Killing Blow 0.60, Amalgam Organ Shatter 0.60, Melee
                                               Elementalist 0.60, Swift Momentum aura 0.30, Kullervo passive 1.00,
                                               riven splice; values in 04)
       Valkyr Talons:   ΣWU = ΣWU_talons + ΣWU_regular   (wind-up bonuses of the regular melee weapon add)
       Tennokai heavy:  WU_tk = WU0 / (1 + s_tk)          ΣWU excluded; s_tk = 0 by default (UNVERIFIED, OQ-708)
       heavy slams: no wind-up
E-7.5  swing time of one heavy h:   T_sw / AS
       T_sw = Duration of the weapon's Heavy combo / number of heavy attacks in that combo   (data)
            | τ_heavy = median of that quotient over every heavy combo with Duration (1.95 s today)  (fallback)
```

Sources: `W:Melee` lines 703-709 (wind-up "Time a Heavy Attack must charge up before it activates"; "There is no cap
to Wind Up … due to diminishing returns"; "Heavy Slams do not have any wind up"); `W:Swift_Momentum` line 32 states the
formula: "Heavy Attack Wind Up Speed is calculated similarly to Reload speed: Modified Wind Up Speed = Base Wind Up
Speed ÷ (1 + Wind Up Speed Bonus)". The page marks the sentence `{{Cite}}` (citation needed), so E-7.4 stays UNVERIFIED
(OQ-707); the division form is also the one consistent with "no cap" plus diminishing returns (`F:melee` §4.3).
Example: Glaive Prime WU0 1.2 s with Killing Blow → 0.75 s. Tennokai: `W:Melee` line 722 (performing a heavy "during
this flash increases its Wind Up Speed") and line 723 ("The Wind Up Speed of Tennokai attacks is not affected by Wind
Up Speed bonuses from other sources"): WU_tk therefore ignores ΣWU; the Tennokai bonus s_tk has no published value, and
s_tk = 0 makes WU_tk = WU0 an upper bound on the wind-up time. Hysteria: wind-up mods on the regular melee weapon also
apply to Valkyr Talons and stack with those on the talons (`W:Hysteria` lines 40-41: "This stacks with Wind Up Speed
mods on Valkyr Talons itself").

T_sw fallback: the six heavy combos with `Duration` give per-heavy quotients 2.2 (Dual Nikanas), 1.65 (Dual Swords), 3.3
(Edun, one attack), 1.0 (Hysteria), 2.1 (Sampotes), 1.8 (Syam); median 1.95. The Stance page defines Duration as a
length that scales with attack speed, so it excludes the wind-up, which does not (Decision, UNVERIFIED, OQ-701). The
first damage frame of a heavy is taken at the end of the wind-up (Decision, OQ-701). The inspiration's per-heavy times
are shorter (1.2-1.9 s for most classes, median about 1.45 s; Dual Nikanas 2.2 s = the wiki's 4.4 s / 2): D-708.

### 5.4 Throw and charge times

```text
E-7.6  charged glaive or thrown-hammer throw:  CT_g = ChargeTime_attack / (1 + ΣWU)       (data ChargeTime, e.g. 1.192 s)
       a charged throw started inside a Tennokai window: CT_g = ChargeTime_attack / (1 + s_tk)   (E-7.4; UNVERIFIED)
       quick throw: cycle 1 / FireRate_attack (data FireRate of the "Throw" attack; not scaled by AS)
```

Glaive throws are "governed by Heavy Attack Wind Up Speed (1.2 s base since U30.5), not attack speed" (`F:melee` §11;
`W:Attack_Speed` line 30; `W:Glaive` line 127); the Tennokai wind-up "applies to Glaive (Weapon Type) throws as they are
Heavy Attacks" (`W:Melee` line 724). The use of the attack's data `ChargeTime` and `FireRate` as cycle times is a
Decision (UNVERIFIED, OQ-711): `D:Glaive Prime` gives Charged Throw ChargeTime 1.192 and FireRate 0.833 (= 1/1.2),
Throw FireRate 1.25 (the weapon's attack speed).

---

## 6. The combo counter

### 6.1 State

The combo state of one melee weapon (exalted weapons keep their own, 9.8) is `(c, t_x, regen)`: the count c ≥ 0, the
expiry time t_x of the combo timer, and the regeneration segment (start time, start value) while c is below IC after a
heavy. The fresh state at t = 0 is `c = IC` (overview 4.1) unless T-09 sets a start multiplier (8.5); the timer is not
running until the first gain or heavy.

The game's count is an integer. The MC live count is kept integer: after a spend `c⁺ = floor(η·c⁻)` and regeneration
adds whole points (`c⁺ + floor(ρ_IC·(t − t_exec))`, capped at IC) (Decision, UNVERIFIED, OQ-732; `W:Melee_Combo`
line 192 gives only an integer example, 40 % of 160 → 64). The EV expected path keeps c̄ real-valued and does not
round (mean-field; the difference at tier boundaries is a documented EV approximation class for `08`).

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
E-7.13 Gain of one damage instance i on the single target (applied after the instance, EC-07-59):
  stance strike (light, slide, aerial, wall):    g = floor(Dmg_e / 100 + 0.5)
  Slam-typed stance entry (slam_part, 7.1):      g = 0                                      (Decision, UNVERIFIED, OQ-710)
  gunblade shot (one Ranged stance strike):     g = 1 per pellet that hits                 (Decision, UNVERIFIED)
  Rauta pellet:                                  g = 2 per pellet, at most 28 per shot
  Exalted Blade or Desert Wind light wave:       g of its Ranged entry by the stance rule   (Decision, UNVERIFIED)
  thrown glaive or thrown-hammer direct hit:     g = 1                                      (Decision, UNVERIFIED)
  normal slam (radial, per enemy):               g = 1;  Desert Wind normal slam: g = 0     (W:Serene_Storm line 41)
  heavy attack hit, heavy slam, glaive explosion (heavy or Volatile), heavy projectile, heavy gunblade shot,
  heavy wave, extra hit, DoT tick, data_attack_loop instance:                               g = 0
  Melee Duplicate copy:                          g = 0                                      (Decision, UNVERIFIED, OQ-735)
  extra point:  every instance with g > 0 adds x_cb = floor(p_acc) + Bernoulli(frac(p_acc)) more points; gunblade and
                Rauta pellets roll x_cb per pellet that hits
  Relentless Combination: E-7.13b
  riven curse "Chance to Gain Combo Count" −x_cur (x_cur > 0, negative trait only): every gain event (one instance's
                g + x_cb together, or one Relentless Combination point with its x_cb) happens with probability
                p_gain = max(0, 1 − x_cur)                                               (Decision, UNVERIFIED, OQ-704)
                x_cur ≥ 1 (reachable: −84.9 % … −103.8 % on a +3 −1 riven): no gain at all, as for an undefined D_cb
  no gain at all when D_cb is undefined (E-7.12, raw duration ≤ 0); heavies still spend in both cases
  external gain sources (Tandem Bond +6 per companion melee hit and the other sources of section 2): their user rate
                in points per second, class (c), default 0; they add points but do not refresh the timer
                (Decision, UNVERIFIED: W:Weeping_Wounds line 48 reports that Tandem Bond hits leave the timer unchanged)
```

```text
E-7.13b Relentless Combination (p_RC = 0.25 per rank, 1.00 at max rank)
  events:  the application of each Slash proc carried by a melee instance i (at τ_i, after the instance, CD-04) and
           each tick of that proc (τ_i + δ_SLA + m, m = 0 … N_SLA − 1, `03` E-13); every event adds one point with
           chance p_RC, that point rolls x_cb, and the curse factor p_gain applies
  MC:      events from the live procs and ticks of the run (mechanic id `combo_rc`)
  EV path: expected gain per event π_i · p_RC · (1 + p_acc) · p_gain
           π_i = expected Slash procs of instance i as `02` computes them (rolled SC·w_SLA, forced φ_SLA, Hunter
           Munitions, Internal Bleeding), with SC from M_b(c̄⁻_i) of E-7.18 and the target's immunities (`05`);
           target-state terms of SC (the Tau factor of M_sc) at their fresh value 1, enemy status caps ignored
           (Decision, UNVERIFIED, OQ-729)
  order:   the generator is causal: it walks events in time order and queries `02` for π_i at instance i with the
           c̄⁻_i it already holds, so the loop combo → M_b → SC → Slash procs → combo has no cycle; the schedule depends
           on the target only through immunities
```

Sources: `W:Melee_Combo` line 26 ("100 % stance damage multiplier = 1 point … a 300 % damage multiplier adds 3 combo
points per hit. Only successful strikes against an enemy will award combo points"), line 36 (Rauta 2 per pellet, 28
max), lines 38-52 (Additional Combo Count Chance "starts at +0 % and benefits additively"; sources Enduring Strike,
Guardian Derision, Quickening, Relentless Combination, True Punishment, Exodia Triumph, Exodia Valor); `W:Melee` line 63
(slams: "Melee combo is gained for each enemy struck") and line 705 ("connecting with a Heavy Attack does not add to the
combo counter"); `W:Volatile_Rebound` lines 28-29 (explosions add no combo; "only the thrown weapon's direct hits on
enemies count"), which contradicts `W:Relentless_Combination` line 29 ("the combo counter does not interact with
Glaives"); the Volatile pages are explicit and win (Decision). `W:True_Punishment` line 29 (the Redeemer "shoots 10
pellets with each shot, resulting in +10 hit combo"). `W:Relentless_Combination` stats table (+25 % per rank, +100 % at
rank 3) and lines 24-31 (+1 per Slash proc "and additional +1 hits thereafter for every tick of Bleed damage" at max
rank, melee only, exalted weapons, Exalted Blade waves and thrown glaives included; "Benefits from Additional Combo
Count Chance"). `W:Serene_Storm` line 41 ("Each attack adds to the Melee Combo Counter; however, radial damage from slam
attacks does not"); no such statement exists for Exalted Blade, whose normal slam keeps g = 1 (Decision). `W:Extra_Hit`
(extra hits give no melee combo); Melee Duplicate is listed on that page, but its copy is a full hit with its own rolls
(`04` E-4.15), so g = 0 for the copy is a Decision (UNVERIFIED, OQ-735). Riven curse: `W:Riven_Mods` line 292
("Chance to Gain Combo Count … −84.9% — −103.8%" on a +3 −1 riven, "−56.1% — −68.5%" on +2 −1) and line 1020 (the
value is −100v %); disposition is fixed at 1.2 (overview 1.1), so x_cur > 1 is reachable and the clamp is needed.
External combo sources: `W:Melee_Combo` §Other Sources of Combo Count (lines 49-55); Tandem Bond "Companion melee hits
increase your combo by 6" (`W:Companion` line 716).

**Slam-typed entries.** The exclusion of `Slam` entries from single-target damage is sourced (7.1: `M:Stances`
`calcTotalDmgMulti`, lines 276-294). Their exclusion from combo gain and from forced procs is not: the same module's
`calcTotalComboGain` (lines 335-349) counts every entry, Slam ones included, and `calcTotalSlashProcMulti` (lines
313-329) counts a forced Bleed on every entry. The engine still gives Slam entries g = 0 and no procs, because an entry
that does not strike the target cannot award it combo ("Only successful strikes against an enemy will award combo
points", `W:Melee_Combo` line 26) or apply its procs to it (Decision, UNVERIFIED, OQ-710). Alternative recorded: Slam
entries gain round-half-up(Dmg/100) and carry their procs (the module's tabulation). The difference is large for
Slam-heavy loops: the stanceless Hammer Neutral (`[400 Slam]`, `[300 Sweep, 300 Slam]`, `[400 Slam]`) gains 3 points
per loop under the Decision and 14 under the module; Bullet Dance inputs 1-4 lose one 100 % Slam point each.

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
E-7.14 Timer: every gain event at τ and every heavy execution (hit or not) sets t_x = τ + D_cb. A gain event refreshes
       the timer when it adds at least one point (MC) or when its expected gain is > 0 (EV expected path, Decision:
       fractional expected gains of x_cb, Relentless Combination and the curse refresh it; external sources never do,
       E-7.13).
       The timer does not run during a heavy wind-up or a charged glaive throw (t_x is shifted by the charge length).
       At t_x without Power Spike:  if c > IC: c ← IC;  if c ≤ IC: no change (a running regeneration continues)
                                                                                  (Decision, UNVERIFIED, OQ-728)
       At t_x with Power Spike:     c ← max(IC, c − δ_PS);  t_x ← t_x + D_cb  (repeats while c > IC)
                                                                     floor at IC: Decision, UNVERIFIED, OQ-735
       D_cb undefined (E-7.12): no timer and no expiry; a count above IC (T-09 start) is kept until a heavy spends it
                                                                                  (Decision, UNVERIFIED, OQ-728)
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
Decision (IC is "a minimum value of combo points when idle", `W:Melee_Combo` line 135; UNVERIFIED, OQ-735). Expiry while
the count is below IC: the U35.5 fix ("if you had any Heavy Attack Efficiency and your Combo Duration was zero, you would
immediately get all of your Initial Combo back instead of it regenerating", `W:Melee` line 1151) shows that a reset must
not return the spent Initial Combo at once, so an expiry during regeneration leaves the count and the regeneration
unchanged; the alternative (the count snaps to IC at expiry) is recorded in OQ-728 (TV-07-27). The `full_regen` wait of
E-7.30 assumes the same rule. Other combo-duration sources (Body Count, Drifting Contact, Gladiator Rush, Swift
Momentum, Combo Killer, Melee Guidance, Rising Storm, Amalgam Furax Body Count, Primary and Secondary Dexterity, Rauta
in Kullervo's hands; `W:Melee_Combo` lines 104-117) are `04`'s values. Pause of the timer while
holstered (Tenet Livia, Tenet Grigori, Incarnon "Standoff" / "Abiding Hold" perks) and the Combo Killer reset are
outside the single-weapon schedule (EC-07-24).

### 6.5 Initial Combo and regeneration

```text
E-7.15 IC = max(0, Σ sources)                 no upper cap (Decision)
       sources: Corrupt Charge 30, Covert Lethality 16, Galvanized Reflex 20.46 per stack (81.84 at 4 stacks; the
       in-game description rounds it to 20), Ready Steel aura 24, Melee Crescendo 6 per stack, innate Synoid Heliocor
       20, Furax Wraith 20, Fragor Prime 30, Incarnon perks, riven Initial Combo (positive only); values and condition
       classes in 04 (Galvanized Reflex: W:Galvanized_Reflex lines 5-33, 04 table "initial combo +20.46 × 4")
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
         c⁺  = η · c⁻        otherwise,   η = clamp(ΣHAE, 0, 0.9)   (MC live count: floor(η · c⁻), 6.1)
         t_x = t_exec + D_cb (no timer if D_cb is undefined) ;  if c⁺ < IC: start regeneration (E-7.15)
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
       A heavy or heavy slam executed inside a window (τ_ex ≤ t_event + T_tk, t_event = time of the qualifying event
       that opened it) is a Tennokai heavy: wind-up WU_tk (E-7.4), no combo spent (E-7.16), Tennokai-only bonuses
       apply (Master's Edge, Truth's Flame, Condition's Perfection, Dreamer's Wrath; values in 04). A heavy executed
       after the window closed is an ordinary heavy (normal wind-up, spends).
       Glaives: a charged throw started inside a window charges in CT_g of E-7.6 with s_tk; detonating the glaive in
       flight consumes the window (a Tennokai detonation spends no combo); the window timer is paused while a glaive
       flies (0 s at d = 0); in glaive_cycle each throw hit on the target is a qualifying event (Decision, UNVERIFIED)
       Schedule (deterministic, 8.3): n_tk = 4 with Discipline's Merit, else ceil(1 / p_tk) (7, or 5 with Dreamer's Wrath)
```

Sources: `W:Melee` lines 719-742 (line 722: a heavy "during this flash increases its Wind Up Speed and does not consume
Combo Counter"; line 724: "The Wind Up Speed applies to Glaive (Weapon Type) throws as they are Heavy Attacks, though to
consume Tennokai you have to explode a glaive in flight"), `F:melee` §4.2 (Discipline's Merit "every 4 hits instead of
random"; Dreamer's Wrath ×1.5 → 22.5 %; Opportunity's Reach 4.0 s). The window timer is paused while a glaive is in
flight (`W:Melee` patch U37 line 1027). `ceil(1/p)` replaces the geometric waiting time by a fixed spacing so the
schedule stays deterministic (Decision; the MC evaluator uses the same spacing, so its Tennokai heavies are not random;
OQ-709). Opportunity's Reach (T_tk = 4 s) matters only through the window check: a plan whose heavy executes more than
2 s after the opening event needs it.

### 6.8 Combo-scaled bonuses

Blood Rush (`0.40·(M_b − 1)` relative crit chance), Weeping Wounds (`0.40·(M_b − 1)` relative status chance), the
Gladiator set (`0.10·N_glad·(M_b − 1)`), Secondary Outburst (consumes the combo on a weapon swap, external), Melee
Doughty through Weeping Wounds, Lifted duration, Iron Staff range: values and buckets in `02`, `03`, `04`. This document
fixes which M_b an instance uses:

```text
E-7.18 heavy hits, heavy slam, glaive heavy explosion, heavy projectiles, heavy gunblade shots and heavy waves:
             M_b(c⁻) with c⁻ taken at the heavy's execution, before consumption        (Decision, UNVERIFIED, OQ-724)
       every other instance (default row): M_b(c⁻), c⁻ = expected or live count just before the instance (before its
             own gain): light, slide, aerial, wall strikes, light waves, normal slams, gunblade shots (kind `ranged`),
             thrown glaive hits, Volatile explosions, mapped 9-C projectiles of light inputs, `cycle_loop` and
             `data_attack_loop` instances; a Melee Duplicate copy and an extra hit use the c⁻ of their parent
       ability-combo weapons: M_b = min(M_ab, 12) (E-7.44)
       T-10 set to value V:  every instance of the default row uses M_b = min(V, M_bcap) instead of the count
```

Thrown glaive attacks "are affected by the combo counter and so by Blood Rush" (`F:melee` §5). Whether a heavy's
Blood Rush reads the count before or after it is spent is not documented (`F:melee` open 4, `F:addmul` open 12); the
pre-consumption reading matches the inspiration's heavy path (`JS L8218-8224`), so it adds no register row.

### 6.9 Expected path and the Monte Carlo count

The schedule (section 8) stores the **expected-path** count c̄: every random gain enters with its expectation
(`p_gain·(g + p_acc)` per gaining instance, per pellet for gunblade pellets; `π_i·p_RC·(1 + p_acc)·p_gain`
per Relentless Combination event of E-7.13b; external sources at their rate). Plan decisions that depend on the count
(the heavy trigger of `build_spend`, regeneration waits) use c̄, so the schedule is deterministic (overview 4.1). The
EV evaluator uses M(c̄) and M_b(c̄) (mean-field; the gap to the exact expectation over the Poisson-binomial count is a
documented EV approximation class for `08`). The MC evaluator tracks the live (integer, 6.1) count c along the same
schedule with its own draws (mechanic ids `combo_acc`, `combo_rc`, `combo_curse`) and uses M(c) and M_b(c) of the
live count for damage and bonuses; the hit times do not change, so an MC run can spend a heavy at a lower or higher
live count than the expected path assumed.

The combo record (8.2) also lists every **tier increase** of the expected path (time, old and new M), including the
increases during Initial Combo regeneration after a heavy, for effects keyed to them (Tonkkatt: +4 % Heat per
multiplier increase, 15 s, 30 stacks, `W:Tonkkatt` lines 10, 22-33; values in `04`). Effects keyed to the multiplier of
a heavy (Pennant: attack-speed buff of `4 s·(M − 1) + 2 s` on a heavy kill, `W:Pennant` line 25) read M_h from the heavy
record. Rauta gains (2 per pellet) reach the melee counter only when Kullervo's Rauta and the melee weapon are evaluated
together, which the single-weapon scope does not do (overview 1.3); a Rauta schedule from `06` is not merged here.

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
| `ranged` | a `Ranged` stance strike: gunblade shot, wave, or a mapped weapon-unique attack (9.5, 9.7, 9.8); an unmapped `Ranged` entry (no 9-C row, not a gunblade or exalted wave) deals no damage and writes a data-report line | the mapped attack or Normal Attack × S_j | |
| `slam`, `heavy_slam` | the `Slam Attack` and `Heavy Slam Attack` data attacks | data (radial) | 9.2 |
| `throw`, `explosion` | glaive and thrown-weapon data attacks | data | 9.4 |
| `cycle`, `data` | strikes of `cycle_loop` (Normal Attack × 1) and instances of `data_attack_loop` (any data attack) | data | 8.1 |
| `slam_part` | a `Slam`-typed stance entry | none in the single-target metric | below |

`Slam`-typed stance entries are the radial part of a combo slam. They are excluded from single-target damage:
`M:Stances` `calcTotalDmgMulti` (lines 276-294) excludes them because "a combo with a slam attack will only produce one
damage instance on direct hit of a target; the radial slam is not applied on direct hit", and since U35.5 "Slam attacks
during combos deal flat damage amounts and do not scale with Mods" (`W:Melee` line 1104). Their exclusion from forced
procs and combo gain is a Decision, UNVERIFIED, that departs from the same module's `calcTotalComboGain` and
`calcTotalSlashProcMulti`, which count Slam entries (6.3, OQ-710). They still occupy their strike slot in the loop timing
(E-7.3). An input made only of `Slam` entries after E-7.0 therefore deals no modelled damage to the target: the
stanceless Hammer Neutral, Forward, Block and Forward Block inputs 1 and 3; the stanceless Heavy Blade Neutral, Forward,
Block and Forward Block input 3 (500 %, Knockdown); Azothane Block input 1; Primal Fury Block input 3 and Forward input
4. Galeforce Dawn Forward Block input 2 is not such an input: E-7.0 pads its one-element `Types`, so only its first
entry (100 %, Knockback) is a Slam and the 200 % (Knockback), 100 % (Bleed) and 300 % entries are direct strikes. The
Hysteria opening heavy is the one Slam entry modelled as damage (9.8). This follows the wiki rule but is UNVERIFIED
(OQ-710).

**Gunblade `[Ranged, Slam]` pairs.** In gunblade combos a `Slam` entry that directly follows a `Ranged` entry is the AoE
of that shot, not a ground slam: the 2026-06-24 edits that added them to High Noon and the Gunblade finisher are
commented "adding the AoE on shots to High Noon" and "also adding the aoe to the ground finisher" (`R:Stances` revisions
2788396, 2788397); Bullet Dance and the Gunblade heavy already carried them in revision 2765569 (2026-04-15), and
revision 2788398 made the first Gunblade heavy fire two shots, each with its pair. Such an entry is emitted as a
`slam_part` child of the shot at the shot's time and target (epicentre, E-7 radial Part). Decision (UNVERIFIED, OQ-730): like
every combo Slam entry it deals no modelled damage and applies none of its forced procs (Bleed + Stagger, Ragdoll) to
the shot's target, because combo slams deal flat damage the data does not give (`W:Melee` line 1104) and the wiki's own
single-target tabulation excludes them (`M:Stances` line 287); the alternative (the AoE child hits the shot's target
with its procs) is recorded in OQ-730 and kept for the multi-target mode.

### 7.2 Base vector of a melee instance

```text
E-7.20 Instance j of a light-type kind (light, slide, aerial, wall, light waves) with hit multiplier S_j = Dmg_j/100:
         B'_{j,t}  = S_j · (Bw_t + V_t + X_t)          B_ref,j = S_j · B_ref
E-7.20b Strike j of a synthesized heavy attack with heavy multiplier H_j (E-7.40):
         B'_{j,t}  = H_j · (Bw_t + V_t) + X_t           B_ref,j = H_j · B_ref
E-7.20c Data attacks (slams, heavy slams, throws, explosions, Ranged Attack, mapped attacks):
         B'_{j,t}  = their own E-1 base (01) · s_j      s_j = Dmg_j/100 for gunblade shots and mapped attacks marked
                                                        `scaled` in table 9-C, else 1
         slams and heavy slams: the data base is already 2× and 3× the weapon damage; the Incarnon flat enters once,
         B'_t = (data B0_t + V_t) + X_t, not × 2 or × 3; × M_h for heavy slams          (Decision, UNVERIFIED, OQ-725)
```

Everything after ST-1 follows `01` and `04` with B' := B'_j and B_ref := B_ref,j: the composition r_t uses the strike's
stance physical bonus S^phys_{j,t} (`01` E-1.9; composition and quantization per strike when it differs, `01`
EC-01-34); the base scalar is E-11 / `04` E-4.3 with the CO term multiplied by r_in (7.6). Because `S_j` and `H_j` are
inside the base, they are **not** members of G for that instance, whatever the CO behaviour (`adding`, `multiplying`
or `none`); `08` MUST NOT apply S_j or H_j on top of Y (its instance record carries them for display only). For X = 0,
E-7.20 with E-4.3 is exactly `04` E-4.6, so `04` E-4.6 is a consequence of E-7.20.

**Agreement with the overview and `04` (OQ-726, adopted 2026-11-24).** Overview 2.4 (`stance_hit_mult`,
`mult_other`), 3.1 (ST-1), 3.1a (`G` row: "Not members: the stance hit multiplier S_j and the heavy-attack multiplier
H_j, which are ST-1 base factors for every CO behaviour") and 3.3 (`B'_j = S_j·B'`, heavies `H_j·(B' − X) + X`,
`B_ref,j = S_j·B_ref`, heavies `H_j·B_ref`, `Y_j = B'_j·max(0, 1 + A)·G_F + r_in·κ·B_co,j·C` with
`B_co,j = ρ_co·B_ref,j`) state E-7.20 and E-7.20b, and `04` E-4.6 restates them as their consequence; M(c) (M_h),
M_slam and M_ab are `G` members and Φ_w is in G_F (7.4). The overview also asks `08` to treat its per-instance S_j
field as display only. The placement matters numerically only for the Incarnon flat on heavies: with H_j in `G`, X
would be multiplied by H, which `W:Melee` line 681 excludes (D-702).

Sources: `W:Stance` line 88 (`Physical Damage = Base Physical × (1 + Stance Physical Bonus) × (1 + Modded Physical
Bonus) × Combo Damage Multiplier On Hit`), `W:Melee` lines 638-656 (`[Base × (1 + Damage Mods) × (1 + Elemental Mods)] ×
Heavy Attack Multiplier × Combo Multiplier`, Paracesis `222 × 2.2 × 1.9 × 6 × 7 = 38,974.32`), line 681 ("Increased base
damage from Incarnon Genesis is not multiplied by the Heavy Attack Multiplier, but is multiplied by the combo
multiplier"). The light multiplier S_j multiplies the Incarnon flat: Decision, UNVERIFIED (OQ-725). No game source
covers it; the only related statement, line 681, says the heavy multiplier does **not** multiply the flat, which by
analogy argues for the alternative `B'_{j,t} = S_j·(Bw_t + V_t) + X_t`. The Decision matches the inspiration, whose
`addDmgToBase` puts the flat into the per-type map that the stance `melee_mult` then multiplies (`JS L13193-13205`,
L10586), so it needs no register row. With S up to 4×-5×, the two readings differ by up to (S − 1)·X per strike;
TV-07-04 gives both. Because X_t is split proportionally to `Bw_t + V_t` (`01` E-1.3, D-005), E-7.20b leaves the ratios
`B'_{j,t}/B'_j` unchanged, so q_t of a heavy strike equals q_t of a light strike with the same physical bonus. Slams
and heavy slams follow the heavy analogy (E-7.20c: the flat is not multiplied by the slam's 2× or the heavy slam's
"Heavy Attack Multiplier is always 3x weapon damage", `W:Melee` line 686; UNVERIFIED, OQ-725).

### 7.3 Faction, DoT and extra hits

The melee instance is an ordinary direct instance of E-12 for everything after ST-5. Its DoT seeds carry its full
weapon side, so the stance multiplier, heavy multiplier, M(c) and stealth are inherited by its ticks (`W:Damage_over_Time`
lists the melee combo multiplier and stealth among the scaled terms, `F:status`; Reaper Prime heavy Slash example
`200 × 2.2 × 1.3 × 6 × 5 × 1.3 × 0.35 × 6 = 46,846.8`, `W:Melee` lines 660-677; TV-07-17 and TV-07-18 reproduce the
page's two numeric examples). Extra hits (Toxic Lash ×2 on melee, Xata's Whisper) and Melee Duplicate follow `04` §6.

### 7.4 Melee members of G and G_F

| Factor | Applies to | Value | Source |
|---|---|---|---|
| combo M_h = M(c⁻) | heavy strikes, heavy slam, glaive heavy explosion, heavy projectiles of table 9-C (non-CO part only for `no_combo` rows, `04` co_omit) | E-7.10 | `W:Melee` line 557; `W:Glaive` line 58 |
| Slam Damage `M_slam = 1 + ΣSlam` | slam and heavy slam instances (and the Hysteria opening heavy, a slam) | Seismic Wave 2.00, Nira's set 1.50 per piece, riven splice (04) | `W:Melee` lines 177-183 ("multiplicative with other modifiers to melee damage"); Nira's double application fixed U39 |
| ability combo M_ab | Shadow Clones (Blade Storm), Landslide Fists, Shattered Lash, Whipclaw | E-7.44, T-11 | `W:Exalted_Weapon` lines 352-358 |
| Exalted Blade wave falloff Φ_w | Exalted Blade light and heavy waves | E-7.43; member of `G_F` (range falloff) | `W:Exalted_Blade` line 23 |
| stance and heavy multipliers | – | inside the base (E-7.20), not in G, for every CO behaviour | 7.2; overview 2.4, 3.1a |

Not here: Melee Careen, Valkyr Rage, Excalibur passive, Steel Charge, Incarnon transformation "+100 % Melee Damage"
(04: A or G by `04`'s default rule), stealth (`04` E-4.16), finisher damage (`04` E-4.17).

### 7.5 Instance flags that gate other documents

| Flag | Set on | Consumers |
|---|---|---|
| `heavy` | heavy strikes, heavy slams, glaive heavy explosions (standing throws), hitscan heavy gunblade shots (9.5), Exalted Blade heavy waves (9.8), heavy projectiles marked `heavy_bonus` in 9-C | `01` A heavy-only members (Killing Blow, Melee Assimilation, riven "Melee Damage on Heavy Attack": heavy attacks and heavy slams, `W:Melee` line 791); `02` heavy crit doubling; `04` heavy gates (Empowered Blades, Melee Animosity consumption and benefit, Killing Blow wind-up) |
| `tennokai` | heavies executed in a Tennokai window | `04` Tennokai effects; no consumption (E-7.16) |
| `slide` | slide strikes | slide-only crit chance (Maiming Strike family, riven), `02` |
| `slam` | slams, heavy slams | Slam Damage; CO `none` (`04` D2); Melee Duplicate triggers (`04` E-4.15) |
| `radial` | slams, heavy slams, explosions, radial children | `Part = P_body·(1 − ΣBP)·M_son` (overview E-7 radial branch, `02` E-2.10: the body part's P and h, `05` §4.3; T-03 does not apply; never a headshot or weak point, wp_i = hs_i = 0; M_son only from a spot that reaches radial instances, T-32; no melee attack carries `02`'s `radial_parent_part`, which is Zymos only); `02` rad_i' = 0 (no Puncture or Cold proc crit terms) |
| `melee_hit` | every instance of this document except DoT ticks and extra hits | Valkyr Rage hit count, `on_melee_hit` triggers |
| `stealth_ok` | light, slide, aerial and wall strikes ("standard melee attacks", `W:Stealth` line 36); gunblade shots (the first High Noon shot benefits, line 41); light waves (line 41 names Exalted Blade waves only as attacks that do not trigger the cooldown), heavy strikes, heavy shots and waves, slams, heavy slams, throws, explosions, `cycle` and `data` instances: UNVERIFIED (OQ-733) | stealth eligibility (T-08, `04` E-4.16) |
| `melee_contact` | true-melee contact: light, slide, aerial, wall and heavy strikes, slams, heavy slams (not shots, waves, throws, explosions, DoT ticks) | `04` E-4.16: with T-08's invisibility condition, a contact reveals the player and starts the 3 s per-enemy cooldown, so at most one contact instance per 3 s gets the bonus on the single target (`W:Stealth` line 39; UNVERIFIED for slams, OQ-733) |
| `anim_read` | ground (charged and quick) glaive and thrown-weapon throw hits and Volatile explosions | read n_anim without consuming it (bug: "ground throws will benefit from the bonus but will not consume stacks. Same applies to explosions that do not use the heavy attack input", `W:Melee_Animosity` lines 46-47; UNVERIFIED, OQ-740) |
| `input_start` | first strike of each input | Melee Animosity stacking, Tennokai qualifying events |

**Heavy crit doubling** (the hook of `02` 4.2): on `heavy` instances the relative crit chance of the True Steel family
(True Steel, Sacrificial Steel with its set scaling, Galvanized Steel's base +1.10, Amalgam Ripkas True Steel) and of
melee riven Critical Chance enters ΣCC_rel a second time (`W:Melee` line 746: "Melee critical chance from Riven Mods and
the variants of True Steel apply their bonuses twice on Heavy Attacks"; Galvanized Steel heavy = +2.20 after the
U37.0.4 fix, `F:addmul` §2). Heavy slams count as heavy for the doubling (Decision, UNVERIFIED, OQ-712). Glaive
explosions after an airborne throw do not double; Volatile explosions always double (bug, `W:Glaive_(Weapon_Type)`
lines 44-45). Custom crit chance (T-26 p.c) is added once (D-094).

Melee Animosity (`W:Melee_Animosity` §Notes): each light, slide, aerial or wall **input** adds one stack (≤ 10) for the
next heavy regardless of the number of strikes or enemies ("Only provides 1 stack per melee input, regardless of how
many enemies are hit"); slams and heavies add none; the stacks are consumed only by a heavy that connects (always, in
the single-target metric). The schedule stores n_anim on every heavy (8.2) and on every `anim_read` instance, which
benefits from the current stacks without consuming them (§Bugs, lines 46-47; the bug is modelled, UNVERIFIED as a
current behaviour, OQ-740). `04` §5.4 states the same rule (`hit_unit = input`, keyed to `input_start`; aligned
2026-11-24).

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
  waves (9.8): 1.0 when a `04` catalogue row resolves to the wave instance (Exalted Blade family `Light Waves`, row
                            188, `04` OQ-444); otherwise the value of the input that emits the wave (Desert Wind;
                            Decision, UNVERIFIED, OQ-741; `04` E-4.6 agrees)
  slams: unused (CO behaviour `none`, 04 D2; the "Ground Slam Strike" column predates the U35.5 radial slams)
  stanceless stance-specific combos and weapon-unique combos: 1.0
```

The shared-column rule is a Decision (UNVERIFIED, OQ-716): the page lists those columns once per class because the
combos are identical, while its footnotes say "Any blank cells means that those combos are untested or those combos do
not exist for that stance" and "Stanceless combos not included" (cached lines 576-577). `04` E-4.6 follows this rule,
shared columns for stanceless weapons included (a stanceless Dual Daggers heavy uses 0.40, TV-07-29), and has withdrawn
its earlier "blank cells and stanceless combos use 1.0". r_in multiplies only the `adding` CO term (`04` E-4.6,
overview 3.3: `r_in·κ·B_co,j·C`; `multiplying` and `none` take no r_in); per-hit versus per-input application is
`04`'s OQ-401.

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
explosions → `glaive_cycle`. `build_spend` and `tennokai` are selectable. The plan and its parameters are the scenario
toggle T-30 (overview 6.1, adopted from OQ-722). Weapons and attacks without stance timing (E-7.0b gives no combo):

- Normal Attack of a weapon with no light combo (Diwata; the pseudo-exalted Shadow Clones, Landslide Fists, Shattered
  Lash, Whipclaw; Vinquibus without a stance, whose Bayonet table has no Neutral): `cycle_loop`. Every data attack of
  such a weapon (Shattered Lash `Damage` and `Arcing Damage`) is evaluated on its own (CD-12) with the same plan.
  Archmelee weapons are out of scope (overview 1.3).
- A data attack selected directly that no trigger of table 9-C, no stance entry and no plan above produces (Spectral
  Axe, Spectral Dagger, Spectral Blade and their explosions, Dagger Projectile, Glass Explosion, Gunshot and Gunshot
  Explosion, Attacks While Disc Deployed, Disc Impact, Disc Explosion, Disc Aura, Stormpath Slide Attack, Tactical
  Combo Slam Attack): `data_attack_loop`.

| Plan | Sequence | Parameters (default) |
|---|---|---|
| `light_loop(K)` | combo K repeated, strikes per E-7.3 | K (Neutral) |
| `slide_loop` | the shared Slide combo repeated like a light loop | – |
| `heavy_loop` | heavies back to back, alternating h = 1, 2 | start count (T-09), policy `full_regen` or `back_to_back` (`full_regen`), `heavy_cancel` (off) |
| `build_spend(M_trig)` | light loop until M(c̄) ≥ M_trig, then one heavy; the light loop restarts at input 1 | M_trig = M_cap (12) |
| `tennokai` | n_tk qualifying inputs of a light loop, then one Tennokai heavy; repeat | n_tk (E-7.17) |
| `slam_loop` | one normal slam every T_slam | T_slam (1.0 s, UNVERIFIED, OQ-711) |
| `heavy_slam_loop` | one heavy slam every max(T_slam, regeneration wait) | T_slam (1.0 s) |
| `glaive_cycle` | throw (quick or charged), hit, heavy detonation, catch, repeat | throw kind (charged) |
| `cycle_loop` | one 100 % strike of the attack every T_cyc, combo rules of its class (M_ab for the pseudo-exalted weapons) | T_cyc = 1 / AS with E-7.2 on the attack's data FireRate (UNVERIFIED, OQ-727) |
| `data_attack_loop` | one instance of the data attack every T_cyc; g = 0, no M, no spending, no Tennokai or Animosity effect unless a 9-C row says otherwise; CO from `04`'s catalogue | T_cyc = 1 / FireRate of the attack, or user-set (UNVERIFIED, OQ-727) |

`cycle_loop` matches the inspiration's one-hit, 1.0 s tables for these weapons (`ASH_STANCE`, `ATLAS_STANCE`,
`TITANIA_STANCE`, `KHORA_STANCE`, `GARA_STANCE` and `BAYONET_STANCE`: `total: 1, hits: ["1"]`, cycle `total / speed`,
`JS L6545-6579`, L10137-10216), so it needs no register row; no game source gives these cadences, and the
pseudo-exalted weapons are cast-driven in the game.

### 8.2 Output records

| Record | Fields |
|---|---|
| hit | time t; attack id (the data `AttackName`; synthesized waves: `Light Waves`, `Heavy Waves`, E-7.43); kind (7.1); input index u and strike index e of the combo; flags of 7.5; S_j (or H_j, s_j); S^phys_{j,t}; proc names (crowd-control names included); r_in; heavy index h; c̄⁻ and M(c̄⁻), M_b(c̄⁻); g and expected extra gain; n_anim (heavies and `anim_read` instances); throw index n_thr (throws); expected weight (Volatile explosions: p_vol; 1 otherwise); pellet count rule (gunblade shots) |
| heavy | input time, execution time, h, Tennokai flag, c̄⁻, c̄⁺, M_h |
| combo | time and kind of every expected-path change: gain, expiry, Power Spike decay, heavy spend, regeneration start and end, tier increase (old and new M) |

The window rule of `06` 6.3 applies: a hit belongs to the window when `t ≤ t_win + ε` (ε = 1e-9), closed interval,
evaluated on the time grid as `t̂ ≤ t̂_win` (overview CD-14, `08` E-8.0a; 8.3).

### 8.3 Algorithm

```text
E-7.30 Melee schedule generator (times in s, f64, ε = 1e-9); horizon H_end = max(t_win, t_ttk_max)
  state: t (cursor), c̄ (= start count, 8.5), timer t_x (not running), regeneration segment, h = 1, u_since_heavy = 0
  advance(c̄, τ): apply, in time order up to τ, regeneration (E-7.15), expiry and Power Spike decay (E-7.14),
                 pending Relentless Combination events (E-7.13b) and external gains

  LIGHT PHASE (light_loop, slide_loop, cycle_loop, data_attack_loop, and the light parts of build_spend and tennokai),
  starting at t0:
    τ_0 = t0;  τ_{k+1} = τ_k + D_K / (AS(τ_k)·N_K)   (cycle_loop, data_attack_loop: τ_{k+1} = τ_k + T_cyc(τ_k));
    e = k mod N_K;  stop the phase when the plan says so
      if τ_k > H_end + ε: end
      advance(c̄, τ_k);  emit the instance(s) of strike e (kind per 7.1; Ranged strikes expand per 9.5, 9.7, 9.8)
      for every emitted instance i:  c̄ += p_gain·(g_i + p_acc·[g_i > 0])   (per pellet for gunblade shots);
                                     queue its Relentless Combination events (E-7.13b)
      if the strike's expected gain > 0: t_x = τ_k + D_cb (E-7.14);  count Tennokai and Animosity inputs at input starts

  HEAVY STEP at input time τ_in (heavy_loop, build_spend, tennokai):
    τ_ex = τ_in + WU (WU_tk for a heavy whose τ_in + WU_tk lies inside an open Tennokai window, 6.7);
    if τ_ex > H_end + ε: end
    advance(c̄, τ_ex) with the timer frozen on [τ_in, τ_ex];  M_h = M(c̄⁻)
    emit the n_h strikes of heavy h at τ_ex + i·(T_sw/AS)/n_h, i = 0 … n_h − 1, all with M_h and
      n_anim = min(10, u_since_heavy)
    c̄⁺ = c̄⁻ (Tennokai) or η·c̄⁻;  t_x = τ_ex + D_cb;  start regeneration if c̄⁺ < IC;  h ← 3 − h;  u_since_heavy = 0
    next action time τ_next = τ_ex + T_sw/AS        (heavy_cancel on: τ_ex + t_dmg/AS, t_dmg user-set, OQ-701)
      heavy_loop with policy full_regen:  τ_next = max(τ_next, τ_ex + max(0, IC − c̄⁺)/ρ_IC − WU)
  heavy_loop:       τ_in(0) = 0; each next heavy input at τ_next
  build_spend(M_trig): light phase from t0 = 0; end it after the strike that makes M(c̄) ≥ M_trig; heavy input at the
                    time of the next strike slot; after the heavy a new light phase starts at τ_next at input 1
  tennokai:         light phase; end it after the last strike of the input that holds the n_tk-th qualifying event of
                    the phase (t_event = that event's time); heavy input at the next strike slot; the heavy is a
                    Tennokai heavy only if τ_ex ≤ t_event + T_tk (6.7); new light phase at τ_next at input 1
  slam_loop:        slams at 0, T_slam, 2·T_slam, …;  each gains 1 (Desert Wind: 0)
  heavy_slam_loop:  heavy slams at τ_0 = 0, τ_{k+1} = max(τ_k + T_slam, τ_k + max(0, IC − c̄⁺_k)/ρ_IC);
                    each spends per E-7.16 (no wind-up)
  glaive_cycle:     section 9.4
```

**Time model (overview CD-14, `08` E-8.0a).** E-7.30 accumulates times in f64 as written, but every time test it makes
compares grid values `τ̂ = round_half_even(τ·10^6)` (integer µs) of the times and durations involved: the horizon test
(`τ > H_end + ε` reads `τ̂ > Ĥ_end`), the window test of 8.2, combo-timer expiry and Power Spike steps against a
strike or execution time (an expiry at the same grid time as a strike is applied before the strike, as the
`gap_max ≥ D_cb` branch of E-7.37 assumes; E-7.14), the end of a regeneration wait, the Tennokai window
(`τ_ex ≤ t_event + T_tk` reads `τ̂_ex ≤ t̂_event + T̂_tk`), the `full_regen` and `heavy_slam_loop` maxima, the
closed-form test `gap_max < D_cb` of E-7.37 and the time components of its state key. Times that are equal in exact
arithmetic (for example a strike that lands exactly D_cb after the last gain) therefore compare equal in both
evaluators, which read the same grid values from the hit records.

Rationale and status: the heavy cycle (wind-up + swing) and the regeneration wait follow `F:melee` §4.3
(`max(windup + swing / AS, regen_time)`); the decision that the player starts the wind-up so that execution coincides
with full regeneration (`− WU`) relies on regeneration during wind-up (OQ-707) and on expiries not restoring IC during
regeneration (E-7.14, OQ-728). Heavy selection: two wiki pages describe it differently. `W:Melee` line 574: "a first
and second Heavy Attack, repeating in sequence. The second Heavy Attack can also be performed by tapping MMB while
sliding"; `W:Stance` line 52: the heavy combo "Has two different attacks that are dependent on the direction the
Warframe model is facing (left or right)", and line 54: "Clicking the heavy attack button while sliding will typically
start with the second heavy attack animation (exceptions include Dual Swords)". The engine uses strict alternation from
heavy 1 at the plan start, never reset (Decision, UNVERIFIED): a facing-dependent choice cannot be modelled without
player orientation, and the `W:Melee` reading is the only one that gives a sequence. The difference matters for classes
with unequal heavies (Dual Nikanas 12× / 6×, Heavy Scythe 6× / 12×, Blade and Whip 4× / 12×, Sparring 12× / 15×);
OQ-702 holds the test. Light-phase restart at input 1 after a heavy
is a Decision (OQ-703). The `heavy_cancel` option models the `W:Melee` tip (line 954) that the next heavy can be
started "the moment the swing does damage": the period becomes WU + t_dmg/AS instead of WU + T_sw/AS; t_dmg has no
data, so the option is off by default and UNVERIFIED (OQ-701; the combo-retention part of the tip is the bug fixed in
U33.6, EC-07-19).

### 8.4 Melee Duplicate, waves and other inserted instances

Melee Duplicate copies (`04` E-4.15), Exalted Blade waves and mapped projectile instances are emitted at their parent's
time, after the parent, in emission order (CD-04 ordering). Projectile flight time is 0 at the default distance d = 0
(`06` E-6.22 applies at d > 0).

### 8.5 Start state, T-09 and T-10

```text
E-7.31 start count:  c(0) = IC                                         default
                     c(0) = max(IC, k_tier · (M_start − 1))            T-09 on, M_start ∈ [1, M_cap] (default min(12, M_cap))
       the timer is not running at t = 0 (6.1): a start count above IC is kept until the first gain or heavy starts it
       T-09 on: attacks without an explicit heavy flag use the heavy_loop plan; attacks with a data role (slam, heavy
                slam, throw, explosion) keep their own plan (the inspiration's `isHeavy` auto-mode, I:feat §2)
       T-10 = V: E-7.18 for light-type instances; the count still evolves (used by heavies and Lifted)
```

T-09 values above M_cap are clamped (D-063).

### 8.6 Steady-state quantities (`dps_burst`, `dps_sustained`)

```text
E-7.37 light_loop / slide_loop / cycle_loop / data_attack_loop at the steady AS:
         hits per s = N_K / T_loop;  Σ S per s = Σ_e S_e / T_loop   (cycle_loop, data_attack_loop: N_K = 1, T_loop = T_cyc)
         closed form: steady M_b = min(M_bcap, M_cap)  if D_cb is defined, p_gain > 0, at least one strike of the loop
           has expected gain > 0, and gap_max < D_cb, where gap_max = the largest time between consecutive gaining
           strikes of the loop taken cyclically (wrapping from the last gaining strike to the first of the next loop);
           with one gaining strike per loop gap_max = T_loop
         no gain at all (D_cb undefined, p_gain = 0, no gaining strike): steady M_b = M_b(IC)
         otherwise (gap_max ≥ D_cb: the combo expires inside every loop): use the periodic rule below
       periodic rule (heavy-type plans, and light loops outside the closed form): run E-7.30 with the steady stats from
         the fresh state and record the state key at every heavy execution, or at every loop start for plans without
         heavies:
           key = (c̄⁻ rounded to 1e-9, h, phase), phase = (input index u, strike index e of the next light strike,
                 regeneration deficit IC − c̄ rounded to 1e-9 or "none", timer remaining t̂_x − τ̂ (grid, CD-14) or
                 "not running", pending Relentless Combination events relative to τ as grid differences)
         stop at the first key equal to an earlier key (period = time between them), after 1000 heavies or loops, or
         at 3600 s of simulated time, whichever comes first
         period found: dps_burst = damage of the instances in [τ_first, τ_repeat) / (τ_repeat − τ_first)
         no period found: dps_burst = damage landed in the last 60 s of the run / 60 (Decision)
         a plan that never reaches a heavy (build_spend with M_trig out of reach) runs to the time bound as a light loop
       melee has no reload: dps_sustained = dps_burst (overview 4.1)
```

`08` consumes these; with T-10 set, steady M_b = min(V, M_bcap). The closed form needs the gap between gaining strikes,
not Δ: strikes with g = 0 (Slam entries, 40 % strikes, `data_attack_loop` instances) do not refresh the timer, so a
loop can expire while Δ < D_cb (TV-07-31). Test vectors: TV-07-28 (heavy plan), TV-07-31 (light loop).

---

## 9. Weapon families

### 9.1 Heavy attacks: the synthesized attack and its hit table

The heavy attack is not a data attack. The engine synthesizes it from the weapon's Normal Attack (types, CC0, CDM0, SC0)
and a hit table:

```text
E-7.40 Hit list of heavy h ∈ {1, 2} of weapon w
  1. Weapon-unique or stance-defined heavy (the E-7.0b result is a combo without FromWeapon: a UniqueToWeapon table,
     or an exalted stance that defines its own Heavy): its entries as given, H = Dmg/100 (σ_h = 1);
     Ranged entries spawn table 9-C attacks (9.7) or exalted waves (9.8); Slam entries excluded (7.1), except the
     Hysteria opening heavy (9.8).
     Exalted stances: Primal Fury (Iron Staff) "The Way" 500 % Knockdown / 500 % Knockdown; Serene Storm (Desert Wind)
     "Rise From Ashes" heavy 1 = 500 % (Knockdown) + 400 % Slam (Disarmed; excluded with its proc, E-7.0 aligns
     `Procs` per entry), heavy 2 = 4 × 300 % (Lifted) + 300 %; Ravenous Wraith (Shadow Claws) "Death Knocking"
     2 × 400 % (Knockback, Bleed) / 2 × 400 % (Bleed); Hysteria (Valkyr Talons) per 9.8.
  2. Otherwise the class's stanceless Heavy combo, attack h (for Exalted Blade: the stance's FromWeapon copy, class =
     the stance's WeaponType, Sword); direct strikes = non-Slam strikes:
       if table 9-A gives an explicit breakdown for (class, h) whose strike count differs from the direct strikes:
            use the 9-A breakdown (multipliers, Bleed procs; no other procs)
       σ_h = T_{class,h} / Σ_direct (Dmg·Hits/100)       T_{class,h} from table 9-A; σ_h = 1 if the class has no row
       H_j = σ_h · Dmg_j / 100 ;  procs from the stance entry
  3. Weapon factor:  H_j ← κ_w · H_j        (E-7.41; κ_w = 1 unless listed)
  A heavy combo with one attack uses it for h = 1 and h = 2 (Edun).
  Glaives (9.4) and gunblades (9.5) use their own rules. Diwata and the pseudo-exalted weapons have no heavies
  (`W:Melee` lines 563, 730; `W:Exalted_Weapon` line 96): no heavy plan exists for them.
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
       own SC and ForcedProcs (heavy slam: Lifted), radial (Part per overview E-7's radial branch), Falloff from data
       (0.5 / 0.3 at the edge).
       single target at the epicentre: d_aoe = 0, Φ = 1 (06 8.2)
       G factors: M_slam = 1 + ΣSlam (both);  M_h = M(c⁻) and spending (heavy slam only)
       CO: none (04 D2).  Combo gain: slam 1 (Desert Wind 0), heavy slam 0.  Heavy-only A members and heavy crit
       doubling: heavy slam.  Incarnon flat: once, not × 2 or × 3 (E-7.20c, UNVERIFIED, OQ-725).
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
attack once (Decision, UNVERIFIED, OQ-710). Worked numbers: TV-07-19.

### 9.3 Slide, aerial and wall attacks

The slide attack is synthesized from the shared Slide combo (E-7.0b): its strikes are `slide` instances with
`S_j = Dmg_j/100` (Nikana Slide 200 % = Nikana Prime `SlideAttack` 396 = 2 × 198, `D:Nikana Prime`). The `SlideAttack`
field agrees with the class Slide combo for most weapons (Sword: 15 of 16 at 1.0 × B = the 100 % combo; Heavy Blade: 14
of 17 at 2.0 × B = the 200 % combo; Dagger: 9 of 9 at 2.0 × B). Fields that disagree with the combo are reported and
ignored: Sword: Skana 0.625 × B (75/120) vs 100 %; Heavy Blade: Zenistar 0.8725 × B vs 200 %; Staff: Cadus 1.3077 × B vs
100 %; War and War Prime have no field. Slide durations exist for 21 of 28 stanceless and 48 of 63 PvE shared tables;
missing ones use E-7.3. Pressure Point and elemental mods apply (`W:Melee` line 47); "Critical Chance for Slide Attack"
mods and riven lines gate on `slide`. A heavy started while sliding (heavy slide) is typically the second heavy
(`W:Melee` line 574; `W:Stance` line 54: "typically", with Dual Swords named as an exception without saying which heavy
they use); it is a Tennokai qualifying event. No default plan uses heavy slides.
Aerial combos (mostly without Duration) and wall attacks (class multipliers 1.67×-3×, `W:Melee` lines 185-250; zaw
`Multi.Wall`) are data for completeness; no default plan uses them (`F:melee` §9 recommendation).

### 9.4 Glaives and thrown weapons

| Attack (data) | Kind | Rules |
|---|---|---|
| `Throw`, `Charged Throw` | throw (direct, `Thrown`) | own CC, SC, ForcedProcs; CO applies (U28.0.6); combo gain 1; M_b for Blood Rush; Power Throw and the two-bucket rule (`04` 3.4) with throw index n_thr; no heavy flag; `anim_read` (Melee Animosity benefit without consumption, 7.5) |
| `Throw Recall Explosion`, `Charged Throw Recall Explosion` | explosion, heavy | the heavy-input detonation: radial (E-7 radial Part, no falloff since U29.5), × M_h, spends combo, heavy-only bonuses and crit doubling (standing throw), CO `none`, two-bucket (`04`) |
| `Throw Bounce Explosion`, `Charged Throw Bounce Explosion` | explosion, not heavy | Volatile Rebound / Volatile Quick Return explosions (50 % of the heavy explosion): expected weight p_vol, no combo multiplier, no spending, no gain, no forced procs except data ForcedProcs, crit doubling always (bug), `anim_read` |
| `Slam Attack`, `Heavy Slam Attack` | 9.2 | |
| `Normal Attack` | melee swings with the stance loop | |

```text
E-7.36 glaive_cycle (charged throw, default):  cycle n starts at τ_n
         charge on [τ_n, τ_n + CT_g]: a heavy wind-up for the combo timer (paused, E-7.14); release at
         r_n = τ_n + CT_g refreshes the timer like a heavy execution (no spending)
         throw hit at r_n (+ d/v at d > 0): one hit on the single target per throw (bounces reach other enemies only,
           Decision); gain 1 (E-7.13) applied at once; throw index n_thr = n + 1 (consecutive throws, for Power Throw)
         Volatile explosion at the throw hit time, after the throw's gain, expected weight p_vol (EV) or a draw
           Bernoulli(p_vol) (MC, mechanic id `glaive_volatile`); one per throw on the single target
         heavy detonation at the throw hit time, after the throw's gain and the Volatile explosion: M_h = M(c⁻) with c⁻
           including the throw's gain, then spend (E-7.16); a Tennokai detonation spends nothing (6.7)
         τ_{n+1} = r_n + max(0, IC − c̄⁺_n)/ρ_IC            (regeneration wait, no charge-time credit)
       quick-throw variant: release at τ_n, cycle 1 / FireRate_Throw; no timer pause
       ordering at one time: throw hit (damage, gain) → Volatile explosion → detonation (CD-04 order per instance)
```

Sources: `W:Glaive` lines 12-25, 57-58 (heavy attack in flight explodes, "counted as a Heavy Attack and consumes the
combo meter. It benefits from heavy attack mods and the combo multiplier"; "headshot multiplier of 1x"), patch U30.5
(wind-up 0.6 → 1.2 s; quick throws ~50 % of charged, cached lines 121-140), `W:Melee` line 748 ("Glaive type weapons
apply these effects to the explosion of a Glaive Heavy attack only, and not to the weapon's damage from direct hits"),
`W:Volatile_Rebound` lines 28-31 and `W:Volatile_Quick_Return` lines 30-31 (50 % of heavy damage, no combo multiplier, no
spending, no guaranteed procs), `W:Glaive_(Weapon_Type)` lines 44-45 (crit doubling bugs), `F:melee` §11 (class heavy
2×/3× and the Stances/data glaive heavy "Falling Star" 3 × 500 % conflict with the data explosions; the data attacks
win). The flight and catch times are not documented: the cycle uses only the data charge time and fire rate (Decision,
UNVERIFIED, OQ-711). Charged throws "count as heavy attacks" (`W:Melee_Animosity` line 46) and the wind-up governs them
(`W:Melee` line 724), so the charge pauses the combo timer like a wind-up and the release refreshes it like a heavy
execution (`W:Melee_Combo` line 286); only the explosion spends (`W:Glaive` line 58). These two timer rules, the
order "throw gain before detonation" (the player detonates after the glaive has struck), one hit and one Volatile
explosion per throw on a single target, and the run of consecutive throws for Power Throw (every throw of the cycle;
`04` 3.4 owns the stack rule) are Decisions (UNVERIFIED, OQ-711). The Volatile chance is the mod's per-bounce explosion
chance (25 / 50 / 75 / 100 % by rank, `W:Volatile_Rebound` stats table). Melee Animosity: ground throws and Volatile
explosions benefit but do not consume stacks (bug, `W:Melee_Animosity` lines 46-47): flag `anim_read` (7.5); the
throw is not `heavy`, so it never consumes. Wolf Sledge and Sigma & Octantis (shield throw) use this section with their
own throw attacks (`F:melee` §12). Worked numbers: TV-07-20.

### 9.5 Gunblades

```text
E-7.45 A Ranged stance strike of a gunblade fires one shot of the weapon's "Ranged Attack":
         pellets = floor(MS) + Bernoulli(frac(MS)) (02 E-10; MS from the attack's Multishot), each pellet one instance
         base per pellet = E-1 base of Ranged Attack × s_j,  s_j = Dmg_j / 100         (Decision, UNVERIFIED)
         falloff Φ(d) of the Ranged Attack (06 E-6.20), hitscan
         pellets are sequential for the CO count (04 2.3)
         stance physical bonus: the Ranged entry's S^phys_t applies to the shot's IPS types (01 E-1.9), like a strike's
           (Bullet Dance shots ×1.25 on Impact, Puncture and Slash; Decision, UNVERIFIED, OQ-731)
         combo gain 1 per pellet, x_cb per pellet; one Tennokai qualifying event per shot (multishot does not add events)
         a following Slam entry of the same input is the shot's AoE child (slam_part, 7.1)
       Heavy (Full Bore): the stanceless Gunblade Heavy combo as shots: heavy 1 = 2 shots at 400 % (+2 Slam entries,
         the shots' AoE children, excluded), heavy 2 = 1 shot at 400 % (+1 Slam entry); each shot × M_h; σ = 1 (class
         5× not used); g = 0 (a heavy adds no combo); flag `heavy` with heavy-only A members and crit doubling for
         hitscan shots (Redeemer, Redeemer Prime, Sarpa, Vastilok); Stropha's heavy shot is a projectile and keeps the
         9-C default (heavy_bonus = no)                                              (Decision, UNVERIFIED, OQ-717)
       Melee strikes of a gunblade loop are ordinary light strikes.
```

Sources: `F:melee` §12 (shots use no ammo; each shot counts as a Tennokai hit, multishot does not; pellets hit
sequentially for CO; U28.0.6 "Gunblade Heavy Attacks will now expel Melee Combo as intended"), `W:Gunblade` (no ammo,
falloff), `W:Melee` line 728 (gunblade exception for Tennokai), `W:True_Punishment` line 29. Evidence for `s_j`
scaling the shot: `D:<weapon>` `HeavyAttack` of Redeemer Prime, Stropha and Vastilok equals exactly 4 × (Ranged Attack
total × Multishot) (3200, 2800, 2484), i.e. one 400 % heavy shot; the High Noon and Bullet Dance CO ratios are 100 % on
every input. PE gives 5 × the melee base instead (OQ-723). Whether melee-only mods apply to shots: they do (the shot is
an attack of a melee weapon; no source says otherwise; Decision). Heavy-only bonuses on heavy shots: `W:Melee` line 750
excludes only "Non-hitscan projectiles created by Heavy Attacks" from bonuses like Killing Blow, so hitscan heavy shots
keep them (data `ShotType` "Hit-Scan" for Redeemer, Redeemer Prime, Sarpa, Vastilok; "Projectile" for Stropha); the
Decision extends OQ-717 (TV-07-23). The stance physical bonus on shots: the data puts `SlashMultiplier`,
`ImpactMultiplier`, `PunctureMultiplier` on the `Ranged` entries of Bullet Dance, and Sarpa and Vastilok shots are
physical (`D:Sarpa`, `D:Vastilok` Ranged Attack IPS); no page says whether the shot uses them (TV-07-30). The
`[Ranged, Slam]` pairs (`R:Stances` revisions 2765569 and 2788396-2788397, 7.1) and the first heavy's second shot ("First
Gunblade Heavy attack shoots twice", revision 2788398) are current data. Corufell's heavy is a gunblade-like projectile
(table 9-C).

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
| gunblades | Ranged entries (light and heavy) | `Ranged Attack` | 1 shot, scaled; heavy shots: heavy_combo yes, heavy_bonus yes when hitscan (9.5) | 9.5 |
| Exalted Blade, Desert Wind | Ranged entries | none: Normal Attack × S_j (waves, 9.8) | 1, wave | |
| Ack & Brunt, Ceramic Dagger, Okina, Hate, Innodem, Destreza (Incarnon), Vinquibus, Korumm, Telos Boltace, Vitrica, Sigma & Octantis, Orvius | Incarnon form, conditional or multi-target triggers | their data attacks | – | not triggered by any default plan; selectable through `data_attack_loop` (8.1, EC-07-52) |
| unmapped `Ranged` stance entries: Harrowing Spire Block input 1 (300 %, Vinquibus; candidate `Gunshot` + `Gunshot Explosion`), Cobra & Crane and Sigma & Octantis Aerial input 1 (100 %; candidate `Shield Throw` for Sigma & Octantis), Vitrica Aerial input 1 (0 %) | – | none until curated | – | no damage; one data-report line each (7.1; UNVERIFIED, OQ-727) |

The trigger column for Syam, Edun, Zenistar, Caustacyst and the Incarnon rows is the pipeline's curation task; the
values above come from `D:Stances` Ranged entries and the attack names of `D:<weapon>` (this session; the 22 `Ranged`
entries outside gunblade and exalted stances were listed). Vinquibus's `Gunshot` (Puncture 182, Slash 78) equals its
Normal Attack total, which the 300 % Harrowing Spire entry would contradict under `as_data`; the mapping waits for a
source.

### 9.8 Exalted melee weapons

```text
E-7.43 Scope: an exalted weapon name in this section, and wherever else this document names an exalted weapon (e.g.
       E-7.0b, E-7.13, E-7.17, E-7.40, 7.4, 7.5, tables 9-C and 9-E), is a data Family (`D:<weapon>` `Family`) and
       covers its Prime and Umbra variants (01 Table 4.3-A): Exalted Blade = Exalted Blade, Exalted Prime Blade,
       Exalted Umbra Blade (identical `Attacks`); Valkyr Talons, Iron Staff, Desert Wind, Shadow Claws, Diwata
       likewise with their Prime variants.
       Base: m_str = Str·R_r (01 E-1.1 model 1; R_r = s_r/s_3 for the ability rank r of T-31, 1 at the default rank 3)
       for Exalted Blade, Valkyr Talons, Iron Staff, Desert Wind, Shadow Claws, Diwata; the exalted weapon keeps its
       own combo state (6.1) and its own mods; combo tables by E-7.0b (fixed stance), heavies by E-7.40.
       Exalted Blade light waves: every Ranged entry of the Exalted Blade stance (and one wave per slide strike, with
         the slide strike's Dmg, Decision) is one `ranged` instance: E-7.20 with S = Dmg/100, per-type vector of the
         Normal Attack, falloff Φ_w = max(0, 1 − d/70) in G_F, separate crit and status rolls, combo gain (E-7.13),
         Tennokai events; flight time d / 30 m/s at d > 0
       Exalted Blade heavy: the Sword heavy table resolved by E-7.40 rule 2 (blade strikes × M_h) plus 2 waves per heavy,
         each with the per-type base B'_t = H_j·(Bw_t + V_t) + X_t (E-7.20b) and B_ref = H_j·B_ref, Φ_w in G_F,
         Lifted forced, weak points possible (projectile), no M_h, no gain; flags `heavy` (heavy-only A members and
         crit doubling apply) and `stealth_ok` per 7.5; both waves hit the single target (Decision, UNVERIFIED, OQ-736)
       Wave attack ids (the hit record's attack id, 8.2, and the `04` §2.14 join key of wave instances): Exalted Blade
         light and slide waves `Light Waves`, Exalted Blade heavy waves `Heavy Waves`, Desert Wind waves `Light Waves`
         (the wiki attack names of `04` rows 188-189). Their CO behaviour is `04`'s: for Exalted Blade, Exalted Prime
         Blade and Exalted Umbra Blade rows 188 (`Light Waves`: adding, ρ_co = 2, co_ref self) and 189 (`Heavy Waves`:
         multiplying) resolve to these instances (`04` OQ-444; Decision, UNVERIFIED, OQ-741); a wave without a row
         (Desert Wind) takes `04` D6 (adding, ρ_co = 1). The wave's own B_ref of `04` §2.6
         (co_ref self) is B_ref,j of E-7.20 (S_j·B_ref) or E-7.20b (H_j·B_ref)
       r_in of a wave (E-7.21): 1 when a `04` catalogue row resolves to the wave (the row is the wave's own measured
         CO ratio; `04` E-4.6); otherwise the r_in of its input, as for every hit of the input  (Decision,
         UNVERIFIED, OQ-741)
       Exalted Blade normal slam: CC0 = 0 and no M; g = 1 (Decision: no statement excludes it)
       Desert Wind normal slam: CC0 = 0 and no M; g = 0
       Desert Wind waves: Ranged entries of Serene Storm, no falloff (≤ 20 m), line of sight
       Desert Wind heavy: E-7.40 rule 1 with the Serene Storm Heavy (the 400 % Slam entry of heavy 1 excluded; no page
         makes it a slam instance, Decision, UNVERIFIED, OQ-710)
```

Sources: `W:Exalted_Blade` lines 17-40 (100/125/200/250 × Strength; normal, slide and aerial attacks emit waves with
"the same base damage as the attacks that release them"; separate entities; waves add combo and trigger Tennokai; "lose
40 % of their damage per second; Wave damage falls off linearly with distance from 100 % to 0 % between 0 and 70 m";
lose 20 % per enemy hit (multi-target); heavy: 2 waves in a cross, projectiles that hit weak points, do not lose damage
per enemy, "do not scale off combo counter"; slam radial "does not have a critical chance, is not affected by the Melee
Combo Counter"), `W:Serene_Storm` lines 30-50 (waves 20 m, no falloff, add combo, trigger Tennokai; line 37: slam radial
"does not have a critical chance, is not affected by the Melee Combo Counter"; line 41: "radial damage from slam attacks
does not" add combo), `W:Exalted_Weapon` lines 96 and 285 (pseudo-exalted weapons have no heavies; waves gain combo
since U38.5), line 161 (U38.6: "Fixed Heavy Attack mods not being triggered by Exalted Blade's energy blades") and line
291 (U38.5: heavy attacks "release a cross-shaped projectile, dealing Heavy Attack Damage"), `F:melee` §13. The `heavy`
flag of the heavy waves follows lines 161 and 291 (Decision, UNVERIFIED, OQ-736). The two wave statements of
`W:Exalted_Blade` differ (40 %/s at 30 m/s is 1.33 %/m, the linear rule 1.43 %/m); the explicit linear distance rule is
used (Decision). The wave speed 30 m/s is the page's ("fly at a constant speed of 30 m/s", `W:Exalted_Blade` line 19,
after the U38.5 speed increase, `W:Exalted_Weapon` line 289); the data `ShotSpeed` 15 of `D:Exalted Blade` predates it
and is reported as a data warning. The data `CritChance` of the Slam Attacks (0.15 Exalted Blade, 0.35 Desert Wind) is
the weapon's, which both pages override for slam radial damage. `D:Stances` encodes Exalted Blade waves as `Ranged`
entries with their own percentages (e.g. Neutral input 2: 300 % blade, 100 % wave), which overrides the page's "same
base damage as the attacks that release them" for light waves (Decision, UNVERIFIED, OQ-736). `W:Desert_Wind` line 24
says "Melee and wave damage can be increased by the combo counter"; the general rule (normal attacks are not
multiplied) is kept (Decision, UNVERIFIED, OQ-718). Worked numbers: TV-07-22.

Family scope and wave ids: `D:Exalted Blade`, `D:Exalted Prime Blade` and `D:Exalted Umbra Blade` (cached
`Weapons_data_melee.lua` lines 5052-5262) have identical `Attacks` (Normal Attack 37.5/37.5/175, Slam Attack 500,
Heavy Slam Attack 750) and the `Family` "Exalted Blade", which is also how E-7.0b finds their fixed stance; the wiki's
exalted weapon table lists the Prime and Umbra blades as variants of Exalted Blade (`W:Exalted_Weapon` line 50).
`W:Condition_Overload_(Mechanic)` §Exalted / Pseudo-Exalted Weapons (cached lines 760-762, added in revision 2806085 of
2026-09-07 from the WFCD-maintained list, after the U38.5 wave changes) names the two attacks "Light Waves" (250 →
500, Adding) and "Heavy Waves" (250 → 250, Multiplying) for Exalted Umbra Blade only; `04` rows 188-189 copy them with
`no_data_attack`, list all three blades on both rows (`04` OQ-444: the three data entries are identical, so the wave
projectile's CO behaviour is shared; Decision, UNVERIFIED) and leave the instances to this document (`04` §2.14 column
note). The page's own default for unlisted attacks (D6) is the alternative for Exalted Blade and Exalted Prime Blade
(OQ-741 (a), TV-07-22). The r_in rule for waves is a Decision: the stance table (added in
revision 2699081, 2025-10-11) gives one ratio per combo ("Stance multipliers for all hits in that combo", cached line
573) and is applied to the blade strikes and to waves without a row, while row 188 is a separate measurement of the
wave's own CO bonus relative to its own damage; multiplying the two would count the input's correction twice
(alternative r_in·ρ_co, TV-07-22). Row 189's "Attack Damage" 250 equals the Normal Attack total, not the 5 × 250 heavy wave of E-7.43; for a
multiplying row the listed value does not enter Y, and the page does not say whether the 250 is the measured wave or
the weapon's listed base (OQ-736).

Valkyr Talons (Hysteria): the Hysteria heavy combo's first heavy is a Slam entry ("considered a slam, and thus benefits
from Seismic Wave", `W:Hysteria` line 73): it is modelled as a heavy slam-type instance at the epicentre rather than
excluded (Decision, the page is explicit): base E-7.20b with H = 5.0 on the Normal Attack's per-type vector (Puncture
and Slash, not the Blast of the `Heavy Slam Attack` data), × M_h, × M_slam, flags `heavy`, `slam`, `radial` (E-7 radial Part),
CO behaviour `none` (as every slam, `04` D2), forced Lifted, g = 0, spends (Decision, UNVERIFIED, OQ-736). The second
heavy is 5 strikes of 300 % with Slash procs on strikes 1 and 3. Slide: 6 strikes (`W:Hysteria` line 44). Valkyr Rage
passive gains per hit (`04`). Iron Staff gains +0.5 m range per tier (no damage effect). Serene Storm's 400 % heavy Slam
entry has no such statement and stays excluded (E-7.43). Diwata has no heavies (`W:Melee` line 563) and no stance table:
`cycle_loop` (8.1). Shattered Lash, Whipclaw, Landslide Fists and Shadow Clones use E-7.44 and `cycle_loop` and have no
heavies, no blocking, no Tennokai (`W:Melee` line 730). Garuda Talons are a normal claw weapon (no Strength scaling, any
Claw stance).

### 9.9 Zaw assembly

```text
E-7.42 Zaw = strike Z_s + grip Z_g + link Z_l;  hand type τ = Z_g.Type (OneHanded | TwoHanded); class = Z_s.Type[τ]
  B_zaw    = (Σ_t Z_s.Damage_t + Z_g.Damage + Z_l.Damage) · Z_s.Multi[τ].Base
  B0_t     = Z_s.Damage_t · B_zaw / Σ_t Z_s.Damage_t               every strike except
  B0_t     = Z_s.Damage_t + (B_zaw − Σ_t Z_s.Damage_t) / n_types     Plague Keewar, Plague Kripath (n_types = 4)
  CC0 = Z_s.CritChance + Z_l.CritChance ;  CDM0 = Z_s.CritMultiplier ;  SC0 = Z_s.StatusChance + Z_l.StatusChance
  FR_src = Z_g.Speed + Z_l.Speed + Z_s.Speed  (then E-7.1)
  slam ("Leap") Z_s.Multi[τ].Leap = 2 (9.2 synthesis below)
  Slide and wall attacks: the class's shared Slide and Wall combos (9.3), as for every melee weapon; the module's
    Z_s.Multi[τ].Slide and .Wall are cross-checks only, reported when they differ (Decision, UNVERIFIED, OQ-719)
  Heavy, wind-up, combo duration, stance tables: those of `class` (wind-up: table 9-A typical value, Decision,
    UNVERIFIED, OQ-719, because zaw records carry no WindUp in the wiki module or the Public Export; D0_cb = 5)
```

Sources: `W:Zaw` lines 192-200 (grip sets base speed and damage, the distribution keeps the strike's ratio), lines
580-631 ("Final stat values are calculated by adding values of the Strike, Grip, and Link. For Two-Handed Zaws, the
Damage value is further multiplied by a factor … depending on the Strike"), `M:Modular` lines 384-450 (the damage, per-type
and slide formulas, including the Plague-strike split `strike_t + (total − base)/4`) and lines 530-630 (crit, crit
multiplier, status and speed sums), `D:Modular` `Zaw` (strike `Multi` tables, grip and link values). Slide source
conflict: `M:Modular` lines 433-449 compute `SlideDamage = total × Multi[grip type].Slide`, which disagrees with the
class Slide combo for many strikes (Balla 2.142857 on both grips versus the Dagger combo's 200 % and the Staff combo's
100 %; Cyath, Kronsh, Rabvee one-handed 3 versus the Machete combo's 200 %; two-handed Polearm strikes 2.222222 versus
200 %), while it agrees for others (Dokrahm and Plague Keewar one-handed Scythe 2 = 200 %; Rabvee two-handed Hammer 2 =
200 %). The engine uses the class combo, as for every other weapon whose `SlideAttack` field disagrees with its combo
(9.3), and reports the module value (OQ-719). The zaw slam and heavy slam attacks are not in the data: they are
synthesized as `Slam Attack` = Leap × B_zaw Impact with forced Impact and `Heavy Slam Attack` = 3 × B_zaw Blast with
forced Lifted, the strike's CC and CDM, SC 0.10 and the class falloff (Decision, UNVERIFIED, OQ-719). Exodia arcanes (zaw only) are `04`'s; the Exodia Contagion aerial projectile ("2× Zaw
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
instead (`W:Exalted_Blade` line 41; conflict, OQ-721). A finisher mode (T-34) schedules one finisher of the chosen type every
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

**10.3 Initial-combo heavy loop.** Same weapon, IC = 30 (innate) + 30 (Corrupt Charge) + 81.84 (Galvanized Reflex ×4,
20.46 each) = 141.84, η = 0.50, WU = 1.2 / 1.6 = 0.75 s (Killing Blow), Primed Fury → AS = 0.8 × 1.55 = 1.24, T_sw =
1.95 s (fallback) → 1.572581 s, D_cb = 5 × 0.5 = 2.5 s, T-09 start 12× → c(0) = 220. Executions at 0.75 (220, 12×),
3.072581 (141.84, 8×), then every 2.322581 s at 8× (swing-limited: 1.5726 > 70.92/40 − 0.75 = 1.023); 9 heavies in
20 s, ΣM = 76. With ΣAS = 2.0 (AS = 2.4, swing 0.8125 s) the policies differ: `full_regen` waits for regeneration
(period 70.92/40 = 1.773 s: executions 0.75, 2.3125, 4.0855, 5.8585 …, last 18.2695; 11 heavies, ΣM 92), `back_to_back`
fires every 1.5625 s at 7× after the second heavy (13 heavies, ΣM 97). TV-07-05 states the rounded in-game value
IC = 140 as an input (there `full_regen` gives 12 heavies, ΣM 100).

---

## 11. Edge cases

| ID | Rule | Source |
|---|---|---|
| EC-07-01 | Normal (light, slide, aerial, wall) attacks are never multiplied by the combo multiplier; it multiplies heavies, heavy slams, glaive heavy explosions and heavy projectiles, and feeds combo-scaled bonuses. | `W:Melee_Combo` line 13; 6.2, 7.4 |
| EC-07-02 | Venka Prime reaches 13× at 240, but Blood Rush, Weeping Wounds and Gladiator stop at 12× (M_bcap). Dex Nikana: 11 points per tier, cap 11×. | `W:Melee_Combo` lines 93-94; `W:Venka_Prime` line 44; E-7.10, E-7.11 |
| EC-07-03 | A heavy at combo below 20 is allowed and uses M = 1. | `W:Melee` line 705 |
| EC-07-04 | Heavy attacks add no combo; slams add 1 per enemy (Desert Wind slams none); heavy slams add none; blocking (+1 per blocked projectile) is out of scope. | `W:Melee` lines 63, 705; `W:Melee_Combo` line 31; `W:Serene_Storm` line 41; E-7.13 |
| EC-07-05 | Combo gain per stance strike = round-half-up(Dmg/100) (50 % → 1, 40 % → 0, 250 % → 3), per enemy struck; combo-embedded `Slam` entries gain 0 (Decision, UNVERIFIED: `M:Stances` `calcTotalComboGain` counts them); Swirling Tiger "Dancing Hunter" last hit and Homing Fang "Cutting Arches" first hit are documented exceptions without values: they use the rule (UNVERIFIED). | `W:Melee_Combo` lines 26-28; `M:Stances` lines 335-349; OQ-704, OQ-710 |
| EC-07-06 | Additional Combo Count Chance: additive, +1 per gaining hit with chance p (floor + Bernoulli above 1); sources Quickening 0.20, True Punishment 1.00 (with −50 % base duration), Enduring Strike (Lifted targets), Guardian Derision (blocking), Exodia Triumph, Exodia Valor (Lifted, zaws). | `W:Melee_Combo` lines 38-52; `W:Quickening`; `W:True_Punishment`; E-7.13 |
| EC-07-07 | Relentless Combination: +1 (chance) per melee Slash proc and per Bleed tick; works on exalted weapons, Exalted Blade waves and thrown glaives; benefits from Additional Combo Count Chance. | `W:Relentless_Combination` lines 24-31 |
| EC-07-08 | Rauta: 2 combo points per pellet, at most 28 per shot. | `W:Melee_Combo` line 36 |
| EC-07-09 | Gunblades: each shot pellet adds 1 combo; each shot (not each pellet) is a Tennokai event; pellets are sequential for the CO count. | 9.5 |
| EC-07-10 | Volatile glaive explosions add no combo; thrown direct hits do (this beats Relentless Combination's "the combo counter does not interact with Glaives"). | `W:Volatile_Rebound` lines 28-29; 6.3 |
| EC-07-11 | Extra hits add no combo; Melee Duplicate copies add none either (Decision, UNVERIFIED: the copy is a full hit with its own rolls). | `W:Extra_Hit`; `04` E-4.15; E-7.13; OQ-735 |
| EC-07-12 | Combo duration: `D0·(1 + Σpct) + Σflat`; percent penalties act on the base (Corrupt Charge and True Punishment subtract exactly 2.5 s each from 5); minimum 0.1 s; raw ≤ 0 prevents gain and leaves no timer (heavies still spend; U35.1); Body Count +12, Drifting Contact +10, Gladiator Rush +6 (33 s together), Primary/Secondary Dexterity +7.5 each, Swift Momentum +6, Combo Killer +5, Amalgam Furax Body Count +15, Melee Guidance −6 self, Rising Storm (value `04`); riven combo duration ±. | `W:Melee_Combo` lines 96-131; `W:Corrupt_Charge` line 29; `W:Body_Count` line 29; E-7.12 |
| EC-07-13 | Base combo duration exceptions: Guandao Prime 6 s, Pulmonars 9 s, Vitrica 10 s, Xoris infinite. | `W:Melee_Combo` lines 122-131 |
| EC-07-14 | The combo timer is refreshed by every gaining hit and by every heavy (even one that hits nothing) and is paused during the charge of a charged glaive throw and during the wind-up. | `W:Melee_Combo` line 286; `W:Melee` line 705 |
| EC-07-15 | Power Spike: at each expiry the count loses 20/15/10/5 instead of resetting, floored at IC (Decision, UNVERIFIED, OQ-735); 220 → 0 takes 44 expiries (220 s at 5 s, 748 s at 17 s). | `W:Melee_Combo` line 119; `W:Focus/Naramon` lines 23-27 |
| EC-07-16 | Initial Combo is the count after any reset and the minimum when idle; heavies spend it; it regenerates at 40 points/s; Corrupt Charge "rapidly regenerate[s] to full within 2 seconds". Innate: Synoid Heliocor 20, Furax Wraith 20, Fragor Prime 30. No cap (Melee Crescendo is unbounded). | `W:Melee_Combo` line 135; `W:Corrupt_Charge` line 28; D-705 |
| EC-07-17 | Heavy attack efficiency keeps `η·c` (40 % of 160 → 64), capped at 90 %; Furax Incarnon +90 % applies to heavy slams only; Skana Incarnon +80 % with overshields; Corufell / Dual Keres +20 % with their warframe; conditional Incarnon perks per `04`. | `W:Melee_Combo` lines 192-235 |
| EC-07-18 | Heavy with zero combo duration and HAE: Initial Combo regenerates over time, not instantly (U35.5 fix); an expiry while the count is below IC leaves the regeneration running (Decision, UNVERIFIED, OQ-728; TV-07-27). | `W:Melee` line 1151; E-7.14 |
| EC-07-19 | Chaining a second heavy immediately does not keep the combo (bug fixed U33.6; the `W:Melee` tip is outdated). | `W:Melee` lines 954, 1270 |
| EC-07-20 | Tennokai: 15 % per qualifying input (22.5 % with Dreamer's Wrath), every 4th with Discipline's Merit, window 2 s (4 s with Opportunity's Reach), checked against the heavy's execution time; a Tennokai heavy or heavy slam spends no combo; multi-strike inputs count once; gunblade shots, Exalted Blade and Serene Storm waves count; heavies do not; a heavy slide does; glaive throws use the Tennokai wind-up and a detonation in flight consumes the window, which pauses while the glaive flies; pseudo-exalted weapons and Diwata cannot use Tennokai. | `W:Melee` lines 719-742; `F:melee` §4.2; E-7.17 |
| EC-07-21 | Tennokai heavies ignore other wind-up speed bonuses: WU_tk = WU0/(1 + s_tk), the Tennokai bonus s_tk is undocumented (engine: 0, an upper bound on the wind-up). | `W:Melee` lines 722-723; E-7.4; OQ-708 |
| EC-07-22 | Attack speed speeds combos, slides, aerials and the heavy swing (wall attacks by inference), not the wind-up; glaive and Wolf Sledge charged throws use only wind-up speed. | `W:Attack_Speed` line 30; `F:melee` §1.2; `W:Melee` line 705 |
| EC-07-23 | Base attack speeds are k/60 values; the displayed value is rounded (Ripkas: wiki 0.883 vs PE 50/60). | `W:Attack_Speed` line 32; E-7.1 |
| EC-07-24 | Out of the single-weapon schedule: combo pause while holstered (Tenet Livia, Tenet Grigori, Anku, Ack & Brunt, Furax, Okina, Obex Incarnon "Standoff"/"Abiding Hold"), Combo Killer's glaive timer reset on a secondary kill, Secondary Outburst consuming the combo, Melee Crescendo stacks (user input to `04`). | `W:Melee_Combo` lines 120-132 |
| EC-07-25 | Heavy damage `[B·(1 + A)·(1 + Elem)]·H·M`; the Incarnon flat is ×M but not ×H. | `W:Melee` lines 638-681; E-7.20b |
| EC-07-26 | Heavy-only base damage (Killing Blow, Melee Assimilation, riven splice) is additive with Pressure Point and applies to heavies and heavy slams only. | `W:Melee` line 791; `01` Table 4.8-A |
| EC-07-27 | True Steel family and melee riven crit chance count twice on heavies (Galvanized Steel heavy +2.20 after U37.0.4); heavy slams count as heavies (Decision); glaive airborne-throw explosions miss the doubling, Volatile explosions always get it (also Wolf Sledge). | `W:Melee` line 746; `W:Glaive_(Weapon_Type)` lines 44-45; 7.5 |
| EC-07-28 | Melee Animosity: one stack per normal input (not per strike or enemy), max 10, consumed only by a heavy that connects; slams and heavies add none; ground glaive throws and Volatile explosions benefit without consuming (bug; flag `anim_read`). | `W:Melee_Animosity` §Notes, lines 46-47; 7.5; OQ-740 |
| EC-07-29 | Dreamer's Wrath +32 % crit damage applies to Tennokai heavies only (additive with Organ Shatter); Condition's Perfection, Master's Edge, Truth's Flame are Tennokai-gated (values `04`). | `F:melee` §4.1-4.2 |
| EC-07-30 | Heavies roll normal status besides their forced procs; class forced Bleed: Claws, Dual Daggers, Nikanas, Two-Handed Nikanas, Rapiers, Scythes, Tonfas, Warfans, Whips, and per table 9-A. | `W:Melee` line 679; `F:status` §4.3 |
| EC-07-31 | Two heavies alternate (Decision); a heavy slide is typically the second heavy, Dual Swords being an exception (`W:Stance` line 54); `W:Stance` line 52 ties the choice to the model's facing direction. | `W:Melee` line 574; `W:Stance` lines 52-54; 8.3; OQ-702 |
| EC-07-32 | Heavy multiplier sources conflict (class table vs stance data vs `HeavyAttack` vs PE); resolution E-7.40, E-7.41; Machete heavy 1 is rescaled to 150 % strikes; Sparring uses the class table (12× / 15×); weapon factors Hirudo 6.5/12, Coda Hirudo, Korrudo 10.7/12, Tak & Lug 12/5. | 9.1; OQ-723 |
| EC-07-33 | Non-hitscan heavy projectiles (Destreza Incarnon, Syam) often miss heavy-only bonuses unless still in wind-up: table 9-C rows default to `heavy_bonus = no`. | `W:Melee` line 750; OQ-717 |
| EC-07-34 | Stance physical bonuses multiply with physical mods (`(1 + S^phys)(1 + P)`), per strike; composition and quantization per strike. | `W:Stance` lines 87-88; `01` EC-01-34 |
| EC-07-35 | Forced stance procs are guaranteed regardless of status chance and damage types; crowd-control names (Ragdoll, Stagger, Big Stagger, Stun, Impair, Disarmed, Finisher) create no status. | `W:Stance` line 84; `03` 4.2; OQ-715 |
| EC-07-36 | Combo-embedded `Slam` entries deal flat, unmoddable radial damage (U35.5) and are excluded from single-target damage (sourced) and from procs and gain (Decision, UNVERIFIED); Slam-only inputs after E-7.0 (stanceless Hammer inputs 1 and 3, stanceless Heavy Blade input 3, Azothane Block 1, Primal Fury Block 3 and Forward 4) therefore deal no modelled damage; Galeforce Dawn Forward Block input 2 is not Slam-only (padding); the Hysteria opening heavy is the exception (9.8); gunblade `[Ranged, Slam]` pairs are the shot's AoE child, excluded the same way. | `M:Stances` lines 276-294, 313-349; `W:Melee` line 1104; `R:Stances`; OQ-710, OQ-730 |
| EC-07-37 | Slams: 2× (heavy 3×) weapon damage already in the data, one type, radial, falloff to 50 % (70 %), own status chance, forced procs from data `ForcedProcs` (Impact on 205 of 240 slams, Lifted on 232 of 240 heavy slams; Fragor Prime slams force Knockdown), radius 50-150 % by drop height; no CO; Slam Damage ×(1 + ΣSlam) multiplicative with Pressure Point (Nira's double application fixed U39); Arca Titron's Slam Capacitor is base damage. | `W:Melee` lines 63-183, 1092-1104; 9.2 |
| EC-07-38 | Lifted from heavy attacks and heavy slams lasts by combo tier and is not extended by status duration (rule and the `1 + M` vs `M` conflict owned by `03` 8.9); Heavy Blade heavies always Lift survivors; Lifted and Knockdown forced by melee count as statuses for CO and cancel each other (`03` 8.9). | `W:Melee` lines 690, 795-799; `03` OQ-309 |
| EC-07-39 | Slide attacks: stance Slide combo multipliers (the `SlideAttack` field is a cross-check only), Pressure Point and elementals apply, slide-only crit mods gate on `slide`. | `W:Melee` line 47; 9.3 |
| EC-07-40 | Wall attacks (1.67×-3× by class) and aerial combos are data only; no default plan. | `W:Melee` lines 185-250; 9.3 |
| EC-07-41 | Glaives: separate throw, charged throw, bounce (Volatile) and recall (heavy) explosion attacks with their own data; quick throws ≈ 50 % of charged (U30.5); recall explosions have no falloff since U29.5, a 1× headshot multiplier, and are heavies (×M, spend, heavy mods); Volatile explosions are 50 % of the heavy explosion with the mod's chance per bounce (25-100 % by rank; expected weight p_vol), ×1 combo, no spending, no Killing Blow; CO applies to throws, not explosions; the class 2×/3× and the Stances/data 3 × 500 % heavy are not used. | `W:Glaive` lines 12-58, 121-140; `W:Volatile_Rebound`; `F:melee` §11; 9.4 |
| EC-07-42 | Glaive throws are wind-up-speed scaled, not attack-speed scaled; Power Throw +100 % per consecutive throw (3 stacks) is a weapon-damage bucket (`04` 3.4). | `W:Attack_Speed` line 30; `W:Power_Throw` §Notes |
| EC-07-43 | Gunblade shots use no ammo, have falloff, and their stance Dmg% scales the shot (Decision); gunblade heavies spend combo (U28.0.6 fix). | 9.5; `W:Gunblade` |
| EC-07-44 | Weapons with unique combos override the stance for those combo types (data: Sigma & Octantis, Cobra & Crane, Vitrica aerial; Nepheri neutral; Korumm, Verdilac, Azothane block; Tatsu, Telos Boltace slide; heavies of Argo & Vel, Arum Spinosa, Caustacyst, Edun, Jat Kusar, Keratinos, Quassus, Sampotes, Syam, Tenet Agendus, Tenet Grigori, Zenistar; Sampotes and Tenet Exec slam and heavy slam). | `W:Stance` lines 678-684; E-7.0b |
| EC-07-45 | Stance switching mid-combo is seamless: each combo is an independent loop. | `W:Stance` line 16 |
| EC-07-46 | Stance data quirks: holes, short and long arrays, the Blind Justice bare `Knockdown` (nil → no proc), Crimson Dervish Block = Neutral, non-round multipliers, `StatusMultiplier` ignored. | E-7.0; OQ-714 |
| EC-07-47 | Missing durations: 16 stanceless Neutral loops and 38 of 44 heavy combos; fallbacks E-7.3, E-7.5 are flagged in the output. | 4.5; OQ-701 |
| EC-07-48 | Exalted melee base damage scales with Ability Strength; weapon rank affects only capacity; Garuda Talons do not scale. | `F:melee` §13; `01` E-1.1 |
| EC-07-49 | Exalted weapons keep their own combo counter (`W:Body_Count` line 30; that line's statement that Body Count does not affect them is outdated) and, since U38.5, can equip the Melee Combo Counter mods (Blood Rush, Body Count, Gladiator Rush, Weeping Wounds and others); Melee Crescendo stacks only while a channeled exalted weapon is active; the Xoris + exalted combo reset was removed in U38.5; Blood Rush and Weeping Wounds on Garuda Talons were fixed in U29.5. | `W:Exalted_Weapon` lines 220, 251-274, 283; `W:Body_Count` line 30; `W:Blood_Rush` line 87; `W:Weeping_Wounds` line 65 |
| EC-07-50 | Exalted Blade waves: separate instances with their own rolls, linear falloff to 0 at 70 m, −20 % per enemy (multi-target), add combo and Tennokai; heavy waves: 2 per heavy, weak-point capable, no combo multiplier, heavy-attack mods apply; normal slam radial: no crit, no combo multiplier (still gains combo, unlike Desert Wind); Swordsmanship passive +10 % damage and attack speed (`04`); applies to Exalted Prime Blade and Exalted Umbra Blade (same data Family); Exalted Blade family light waves CO adding at 2.0 without r_in, heavy waves multiplying (`04` rows 188-189, measured on Exalted Umbra Blade, `04` OQ-444). | `W:Exalted_Blade` lines 17-48; `W:Exalted_Weapon` lines 161, 291; `W:Condition_Overload_(Mechanic)` lines 760-762; E-7.43 |
| EC-07-51 | Hysteria: the first heavy is a slam (Seismic Wave applies), slide does 6 hits, wind-up mods on the regular melee weapon also apply. | `W:Hysteria` lines 40-44, 73 |
| EC-07-52 | Weapon-unique projectile attacks (table 9-C) carry their own data; Incarnon-form, conditional or multi-target ones (Ceramic Dagger, Okina, Hate, Innodem, Destreza Incarnon, Ack & Brunt, Vinquibus, Korumm slam, Telos Boltace Stormpath, Vitrica glass, Zenistar disc aura, Orvius hover) are not triggered by default plans; selected directly they use `data_attack_loop`; unmapped stance `Ranged` entries deal no damage. | 8.1, 9.7; OQ-727 |
| EC-07-53 | Ability-combo exalted weapons (Shadow Clones, Landslide Fists, Shattered Lash, Whipclaw): ×M_ab (1:1, up to 12×) from recasts within 5 s; no heavies; Blood Rush, Weeping Wounds, Body Count, Power Spike work on that counter. | `W:Exalted_Weapon` lines 352-358; E-7.44; D-701 |
| EC-07-54 | Desert Wind: "melee and wave damage can be increased by the combo counter" (one page) is not applied to light attacks (Decision). | `W:Desert_Wind` line 24; OQ-718 |
| EC-07-55 | Zaws: 2H damage factor per strike (Balla, Ooltha, Sepfahn 1.00; Plague Keewar 0.85; Cyath, Plague Kripath, Rabvee 1.08; Dehtat, Mewan 1.09; Kronsh 1.07; Dokrahm 0.93); Plague strikes split added damage equally over their four types; class from strike and grip hand type; slides from the class Slide combo (the module's `Multi.Slide` is reported only); wind-up from table 9-A (UNVERIFIED). | `W:Zaw` lines 580-631; `M:Modular` lines 384-450; E-7.42; D-703; OQ-719 |
| EC-07-56 | Incarnon melee form: activation by a heavy at ≥ 6× (5×, 3×), Genesis +100 % melee damage for 180 s, native +3 range and +40 attack speed for 90 s; modelled as a selected state. | `W:Melee_Combo` lines 241-251; 9.10 |
| EC-07-57 | Follow-through (n-th enemy FT^(n−1); Volnus 0.6 → 0.4 with a stance; Ironbride 0.7; Magistar, Nami Solo, Dorrclave conditional) and punch-through are multi-target only. | `W:Melee` lines 814-940 |
| EC-07-58 | Finisher class multipliers and hit splits are data (table 9-E); finisher animations use attack-speed bonuses only (not the weapon's base speed), capped 33 % faster at +50 %; the stealth formula (T-08), the finisher formula, the half elemental/physical rule and the valence half-damage bug are `04`'s. | `W:Finisher` lines 14, 458-566 |
| EC-07-59 | Non-melee status timing: a melee hit deals damage before its procs and before its combo gain (CD-04 extended to combo), so a hit never benefits from its own gain. | Decision; 6.3 |
| EC-07-60 | The schedule's heavy decisions use the expected-path count; MC damage uses the live count (6.9). | 6.9 |
| EC-07-61 | Corrupt Charge regenerates "to full within 2 seconds" after a heavy: consistent with 30 / 40 = 0.75 s. | `W:Corrupt_Charge` line 28 |
| EC-07-62 | Heavy slams have no wind-up; Furax Incarnon heavy-slam HAE; heavy slam Lifted duration by tier. | `W:Melee` lines 686-709 |
| EC-07-63 | Melee projectiles (glaive throws and explosions, Exodia Contagion) use two weapon-damage buckets; Vex Armor, Deathbringer, Amp count in both; Eclipse and Garuda twice (`04` 3.4). | `F:addmul` §12 |
| EC-07-64 | Corufell: the heavy fires a gunblade-like projectile (`Charged Projectile`, CO multiplying). | `W:Gunblade`; `04` catalogue |
| EC-07-65 | The heavy-attack combo multiplier and the class heavy multiplier are separate multipliers: M_h in `G`, H_j in the ST-1 base (7.2; overview 2.4, 3.1a and `04` E-4.6 agree, OQ-726 adopted). | `F:addmul` edge 19; 7.2 |
| EC-07-66 | Stance CO effectiveness differs per input (0.25 Fracturing Wind block to 2.33 Mountain's Edge heavy; Exalted Blade 1.22-1.30; gunblade stances 1.00): imported as data r_in; shared-combo columns apply class-wide. | `W:Condition_Overload_(Mechanic)` §Stances; 7.6 |
| EC-07-67 | Riven melee lines owned here as hooks: Combo Duration (±), Initial Combo (positive only), Additional Combo Count Chance (positive only), Chance to Gain Combo Count (negative only; −56.1 % to −103.8 %, gain probability max(0, 1 − x_cur)), Heavy Attack Efficiency, spliced Heavy Attack Wind Up Speed, Melee Damage on Heavy Attack, Slam Attack Damage, Critical Chance for Slide Attack, Critical Chance (×2 on heavies). | `F:mods` §8 riven table; `W:Riven_Mods` lines 292, 1020; E-7.13 |
| EC-07-68 | Owned elsewhere: Melee Duplicate (tier-1 crits, slams trigger, Toxic Lash / Xata bug; `04` E-4.15), Melee Doughty (`02`), Melee Influence, Melee Afflictions, Melee Vortex, Melee Fortification (multi-target or no damage, `04`), Melee Careen ×2.5 vs Frozen (`04`), Exodia arcanes including the Contagion distance bonus (`04`). | `F:melee` §7 |
| EC-07-69 | Weapon-unique heavy slams (Sampotes 12 × 500 %, Tenet Exec 9 × 300 %) and Korumm's `Tactical Combo Slam Attack` are multi-target shockwaves; single target uses the data attack once. | 9.2; 9.7 |
| EC-07-70 | Exalted melee (data Class "Exalted Weapon") always uses its fixed stance; its FromWeapon combos are accepted (Exalted Blade's Heavy is the Sword heavy); Primal Fury, Serene Storm and Ravenous Wraith heavies are their own tables (σ = 1). | `09` E-9.2, E-9.8; `W:Exalted_Weapon` line 94; E-7.0b, E-7.40; TV-07-21 |
| EC-07-71 | Weapons without a light combo (Diwata, the pseudo-exalted weapons, Vinquibus without a stance) use `cycle_loop` at 1/AS; directly selected weapon-unique data attacks use `data_attack_loop` (no combo interaction). | 8.1; OQ-727 |
| EC-07-72 | Riven curse above 100 % (−103.8 % on a +3 −1 riven): no combo gain at all; heavies still spend. | `W:Riven_Mods` line 292; E-7.13; TV-07-25 |
| EC-07-73 | Relentless Combination is generated causally: each instance's expected Slash procs are taken from `02` with the combo count already reached, so the schedule has no circular dependency; it depends on the target only through immunities. | E-7.13b; OQ-729 |
| EC-07-74 | Gunblade heavy shots add no combo; hitscan ones (all gunblades except Stropha) get Killing Blow and the heavy crit doubling. | `W:Melee` line 750; 9.5; OQ-717; TV-07-23 |
| EC-07-75 | High Noon Block input 1 has a one-element `Types` for two entries; the pipeline override makes the 100 % entry the Slam (data-report line). | E-7.0; OQ-730 |
| EC-07-76 | Melee strikes have no multishot (MS ≡ 1); only gunblade pellets and 9-C Count copies are sub-instances. | `DS:wiki` §3; 1; D-707 |
| EC-07-77 | Under invisibility each true-melee contact reveals the player and starts a 3 s per-enemy stealth cooldown; ranged melee attacks (gunblade shots, Exalted Blade waves) do not. | `W:Stealth` lines 39-41; 7.5; OQ-733 |
| EC-07-78 | The MC combo count is an integer (spend `floor(η·c⁻)`, whole regeneration points); the EV expected path is real-valued. | 6.1; OQ-732 |
| EC-07-79 | Combo tier increases (also during Initial Combo regeneration) are output events for Tonkkatt-type effects; Pennant reads M_h of a heavy kill. | `W:Tonkkatt` lines 10, 22-33; `W:Pennant` line 25; 6.9 |

---

## 12. Inspiration differences

Rows of the overview register that this document owns or co-owns, restated, and new rows `D-701` to `D-708`.

| ID | Topic | Inspiration behaviour | Game behaviour (source) | Engine | Toggle |
|---|---|---|---|---|---|
| D-022 | Stealth (04, 07) | melee non-crit ×7, crit `c + 7`, rank ignored (JS L8330-8336) | `1 + SDB + k(h·CDM − 1)` (`W:Stealth`) | `04` E-4.16 on every `stealth_ok` instance (7.5) | User (T-08) + Compat |
| D-062 | Light-attack combo (04, 07) | combo-scaled bonuses at 12× when the option is off (`combo_counter = 11`, JS L8219-8224, L9906-9910) | combo ramps from IC with hits (`W:Melee_Combo`) | expected-path ramp (6.9, E-7.18); T-10 fixes M_b | User (T-10) + Compat |
| D-063 | Combo range | inputs 1-13 for every weapon (`I:feat` quirk 5) | 12×; Venka Prime 13× (bonuses 12×); Dex Nikana 11× (E-7.10) | clamp to M_cap; bonuses to M_bcap | — |
| D-064 | Heavy combo model | spent every `int(0.8·N/T) + heavyHits` hits, never rebuilt, IC restored instantly, IC capped (JS L12151-12181) | gain per stance hit, spend with η ≤ 0.9, IC regenerates at 40/s (`W:Melee_Combo`, `W:Melee`) | E-7.13 to E-7.16 and plans of 8.1 | Compat (E-7.39) |
| D-065 | Melee hit timing | `round(T·hits/cycle)` hits evenly over the window, cycle from the inspiration's own `default_MeleeStances` table (`total` per class), heavy cycle `heavy.total/speed + windUp` from the default table even with a stance, slams 1 hit per second (1/speed with a stance) with `statuses[0]` forced on every slam hit (JS L10137-10216, L12115-12150, L12436-12457); its stance mods carry their own durations and hit lists, which differ from the wiki in places (Blind Justice "Zatōs Creed" 3.0 vs 3.05 s and 6 vs 7 hits, "Heeding Call" 2.5 vs 2.45 s; `F:melee` §2.2) | stance Duration and per-strike lists from `Module:Stances/data`, wind-up and swing separate, slams are data attacks | E-7.3, E-7.30; strikes evenly spaced inside a loop (UNVERIFIED) | Compat (E-7.38) |
| D-094 | Custom crit chance on heavies | added once per non-empty slot and twice per hit (`I:stat` Q7-Q8) | – | once (7.5) | — |
| D-701 | Ability combo scope | `abilityCombo` multiplies every weapon of type "Exalted Weapon" (Exalted Blade, Valkyr Talons …) when set, and the option's stored value (12) applies as soon as the custom stat is > 1 even with the checkbox off (JS L10377-10388; `I:feat` quirk 6) | only Blade Storm, Landslide, Shattered Lash, Whipclaw have an Ability Combo Counter (`W:Exalted_Weapon` lines 352-358) | E-7.44 on those four only | Compat |
| D-702 | Incarnon flat on heavies | the flat is part of the per-type maps (`addDmgToBase`, JS L13193-13205) and the heavy table multiplier `melee_mult` multiplies the whole hit (JS L10586, L10868) | flat ×M but not ×H (`W:Melee` line 681) | E-7.20b | Compat |
| D-703 | Zaw assembly | grip and link damage spread over the strike's types by `addDmgToBase`, scaled by the per-weapon data key `damage_mult` (JS L13193-13196); whether that key carries the two-handed factor is unknown (`I:model` OQ7: the key appears only in code, no sample weapon carries it; UNVERIFIED); no Plague equal split (JS L14573-14624, L5581-5593) | `(strike + grip + link) × Multi[hand].Base`; Plague strikes add `(total − base)/4` per type (`W:Zaw` lines 580-631, `M:Modular` lines 384-425) | E-7.42; a zaw build is proposed for the oracle sample list (`I:oracle`, `08`) to read `damage_mult` | — |
| D-704 | Volatile glaive explosions | `glaive_explode`: +0.5 × the throw hit without its combo factor, inside the hit (JS L10370, L11376; mods L2310-2321) | a separate radial explosion at 50 % of the heavy explosion, own rolls, no combo multiplier, no spending, crit doubling always (`W:Volatile_Rebound` lines 28-31, `W:Glaive_(Weapon_Type)` line 45) | data attack `(Charged) Throw Bounce Explosion` (9.4) | — |
| D-705 | Initial Combo cap | IC capped at `20 · combo_counter` (220) (JS L12162-12166) | no documented cap (Melee Crescendo unbounded, `F:melee` §3.4) | uncapped (E-7.15) | Compat |
| D-706 | Dreamer's Wrath gating | `heavy_crit_mult` (Dreamer's Wrath +0.32, JS L825) is applied to every heavy (`isNeedCalcHeavy`, no Tennokai test) and is added after the multiplication in the displayed crit multiplier but to the relative bucket per hit (JS L8270-8273, L8878-8880; `I:stat` Q9) | +32 % critical damage on Tennokai heavies only, additive with Organ Shatter (`F:melee` §4.1-4.2) | `04` value gated by the `tennokai` flag (7.5) | Compat |
| D-707 | Melee multishot | melee hits are expanded by the attack's multishot (`procMultishotArray`, `I:agg` §4.2, L12458-12525), so a custom multishot stat adds copies of every melee hit | melee strikes have no multishot (`DS:wiki` §3; data field absent or 1 on every Normal Attack) | MS ≡ 1 for melee strikes; pellets only for gunblade shots and 9-C Count attacks (1) | Compat |
| D-708 | Stanceless durations and heavy timing | own `default_MeleeStances` table (`JS L6302-6585`): light `total` per class within 0.05 s of the wiki's stanceless Neutral Duration for 10 of the 11 classes that have one (Dual Nikanas 2.0 vs 2.37 s), and values for the 16 classes the wiki lacks (Claws 1.85, Polearm 2.15, Tonfa 1.6, Nikana 1.77, Hammer 4.0 s …); heavy `total` per class covering both heavies (per heavy 1.2-1.9 s, median about 1.45 s) | no wiki duration for 16 stanceless loops and 38 of 44 heavy combos (`M:Stances` data; revision 2594921 of 2023 has the same gaps, `R:Stances`) | wiki durations; fallback E-7.3 with f_sl (geometric-mean gap to the inspiration 1.51× unscaled, 1.30× with f_sl) and τ_heavy 1.95 s (E-7.5); the inspiration table is an oracle cross-check, not data (CLAUDE.md) | — (Compat path of D-065, OQ-739) |

**Compatibility paths for D-064 and D-065 (OQ-739, adopted 2026-11-24).** The overview register marks both rows
`Compat`, so melee window-DPS scenarios can be oracle-tested with the paths below: D-065 (hit timing) is E-7.38 and
D-064 (heavy combo) is E-7.39. The overview's Toggle column uses the same pairing (D-064 `Compat (07 E-7.39)`, D-065
`Compat (07 E-7.38)`; OQ-739 settled).

```text
E-7.38 Compat melee timing (D-065; getMeleeCountHits10s and createDmgQueue, JS L10137-10216, L12426-12457)
  table entry P = the inspiration's default_MeleeStances entry of the weapon's name, else of its first compatibility
       tag (class); read by the oracle harness from the local bundle at test time, never shipped as data (CLAUDE.md)
  cycle k = P.total / AS (light, no stance mod) | stance mod's total / AS (light, stance mod) |
            P.heavy.total / AS + WU (heavy) | 1 (slam types, hits [1]; 1 / AS with a stance mod)
  hit count N = round(t_win · |hits| / k);  hit times t_i = i · t_win / N, i = 0 … N − 1
  each hit is expanded by procMultishotArray (D-707); stance multipliers and statuses are assigned cyclically from the
  hits list over the expanded hit list (procMeleeComboAttackEffect, JS L12115-12150); slams force statuses[0]
E-7.39 Compat heavy combo (D-064; addTimelineMeleeCombo, JS L12151-12181)
  d = min(max(IC, 0), 20 · combo_counter) (220 with the default combo_counter = 11, D-705);
  g = max(20 · (M_start − 1), d)                         (M_start: the T-09 start multiplier)
  p = int(0.8 · N / t_win) (N = hits after the multishot expansion);
  q = |P.heavy.hits| (1 without a table entry);  k = p + q
  for each hit in order: if k = 0 then (k = p + q; g = max(g · η, d), and g = 0 if g < 20);
                         if g < 20 then g = 0;  comboMult = int(1 + g / 20);  k ← k − 1
```

With these paths and D-706, D-707 also behind `Compat`, melee window DPS can be compared with the oracle (per-hit damage
checks can also be read from the inspiration's "More details" log, `08`).

---

## 13. Open questions

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-701 | Heavy swing time per class and weapon; does `Duration` exclude the wind-up; when is the first damage frame; can the next heavy be started at the damage frame (`heavy_cancel`, `W:Melee` line 954) and after what time t_dmg; stanceless loop durations (overview OQ-021), including the f_sl-scaled fallback and the gap to the inspiration's table (D-708) | T_sw from data or 1.95 s median; Duration excludes wind-up; damage at the end of the wind-up; `heavy_cancel` off; τ_class · f_sl fallback | frame-timed recordings (60 fps) of two heavies per class at known attack speed and wind-up, with and without pressing heavy at the damage frame; stanceless Neutral loops of Claws, Polearm, Tonfa, Nikana and Hammer at AS 1.0; the u/GoldPhos "true attack speed" dataset linked from `W:Attack_Speed` |
| OQ-702 | Which heavy comes next: strict alternation (`W:Melee` line 574), the model's facing direction (`W:Stance` line 52), reset after idle or after light attacks; which heavy a Dual Swords heavy slide uses (`W:Stance` line 54) | strict alternation from heavy 1, never reset | alternate heavies with a pause of 5 s, with light attacks in between and with the camera turned left and right, on a Dual Nikana (12× vs 6× damage numbers); Dual Swords heavy slide damage |
| OQ-703 | Strike spacing inside a combo; restart of the light loop at input 1 after a heavy | uniform per strike; restart | frame timing of Guiding Light at AS 1.0; combo continuation after a heavy |
| OQ-704 | Combo gain rounding; Additional Combo Count Chance per hit or per point and above 100 %; the riven curse "Chance to Gain Combo Count" (probability per gain event, clamp at 0 for curses above 100 %); gain of gunblade pellets, thrown hits, waves; gain from combo-embedded Slam entries | half-up per strike; +1 per gaining hit; floor + Bernoulli; curse = probability max(0, 1 − x_cur) per gain event; 1 per pellet and thrown hit; waves as their entry; Slam entries 0 (OQ-710) | count the combo counter after single strikes of 50 %, 125 %, 250 % hits; True Punishment on a 300 % hit (expect 4 or 6); one Redeemer shot; a −103.8 % curse riven (expect no gain); stanceless Hammer Neutral combo count per loop (3 vs 14) |
| OQ-705 | Is the raw combo count capped? | no cap | build past 400 on Xoris, heavy with η 0.5, read the remaining count |
| OQ-706 | Order of percent and flat combo-duration terms | percent on base, then flat | True Punishment + Body Count combo bar timing (14.5 s expected) |
| OQ-707 | Wind-up formula `WU0/(1 + ΣWU)` (`W:Swift_Momentum` line 32, marked {{Cite}}); regeneration during wind-up | yes; yes | frame timing with Killing Blow and with Killing Blow + Swift Momentum; regeneration wait test with IC 140 |
| OQ-708 | Tennokai wind-up: its own bonus s_tk (`W:Melee` line 722) and the exclusion of other wind-up bonuses (line 723) | WU_tk = WU0/(1 + s_tk), s_tk = 0 | frame timing of Tennokai heavies with and without Killing Blow on one weapon (equal times confirm the exclusion; the ratio to WU0 gives s_tk) |
| OQ-709 | Deterministic Tennokai spacing ceil(1/p) instead of a geometric wait | ceil(1/p) | none needed for game truth; compare EV with a stochastic-window MC variant |
| OQ-710 | Do Slam-typed stance entries hit the struck target (stanceless Hammer Neutral)? Do they award combo and apply their forced procs (the module's `calcTotalComboGain` and `calcTotalSlashProcMulti` count them)? Is the Serene Storm 400 % heavy Slam entry a slam instance like Hysteria's? | no damage, no gain, no procs; Serene Storm entry excluded | stanceless Hammer neutral damage numbers on a single target; combo count after one loop (3 vs 14); Bleed procs from Bullet Dance Neutral input 1 at 0 % weapon status chance (forced procs only) |
| OQ-711 | Glaive cycle (flight, detonation, catch); does the charge pause the combo timer and the release refresh it; one hit and one Volatile explosion per throw on a single target; throw gain before the detonation; slam and heavy slam loop period | data ChargeTime / FireRate; yes; yes; one each; gain first; T_slam 1.0 s | timed glaive throw-detonate loops with the combo bar visible; Volatile explosion count per throw on a lone target; slam loop timing at minimum height |
| OQ-712 | Heavy crit doubling on heavy slams | doubled | heavy slam crit rate with True Steel at a CC between 0.5 and 1 |
| OQ-713 | Exact internal base attack speed (k/60 or its f32 neighbour) | k/60 in f64 | none needed (< 1e-6) |
| OQ-714 | Meaning of stance `StatusMultiplier` | ignored | status rates on the Tranquil Cleave neutral 3rd input vs the 2nd |
| OQ-715 | `Finisher` and `Disarmed` proc names | crowd control only (`03` table 4.2 adopted it) | settled for the spec; in-game: Caustacyst heavy on a target, no status icon expected |
| OQ-716 | Shared-combo columns of the stance CO table apply to every stance of the class and to stanceless weapons | yes (`04` E-4.6 follows it since 2026-11-24) | CO numbers of a stanceless Dual Nikana heavy (expect 233 %) |
| OQ-717 | Heavy-only bonuses and crit doubling on heavy projectiles (table 9-C), hitscan heavy gunblade shots and Exalted Blade heavy waves | 9-C projectiles: combo yes, heavy bonuses no; hitscan gunblade heavy shots and Exalted Blade heavy waves: heavy bonuses yes | Syam waves with Killing Blow on and off; Redeemer Prime heavy shot with Killing Blow on and off (TV-07-23: 334,910.67 vs 125,609.02 expected per heavy 1 at 12× with True Steel) |
| OQ-718 | Desert Wind light and wave damage scaled by combo? | no | Desert Wind hit numbers at 1× and 12× |
| OQ-719 | Zaw slam and heavy slam stats; zaw slide and wall attacks (class combo vs the module's `Multi.Slide` / `Multi.Wall`); zaw wind-up | synthesized (Leap × B, 3 × B, SC 0.10); class combos; table 9-A typical wind-up | arsenal readout of a zaw's slam values; slide damage numbers of a Balla Dagger zaw (200 % vs 214 %); heavy wind-up timing of a zaw |
| OQ-720 | Incarnon melee form: bucket of +100 % Melee Damage, unit of "+40 Attack Speed", activation cost | A; +0.40 ΣAS; form active for the window | arsenal readout in form; Pressure Point interaction test |
| OQ-721 | Exalted Blade finisher multipliers (table 12×/16× vs ability page 400 %/3200 %) | table 9-E | stealth finisher numbers with Exalted Blade |
| OQ-722 | Toggle T-30 "melee plan" (8.1) with its parameters (combo K, M_trig, heavy policy, `heavy_cancel` and t_dmg, T_slam, glaive throw kind, T_cyc) | adopted as overview T-30 (2026-11-24) | settled |
| OQ-723 | Heavy multipliers of multi-strike classes (Dagger, Dual Daggers, Dual Swords, Tonfa, Warfan), Sparring, Heavy Scythe, gunblades; κ_w for heavy 2 | class table + stance data (E-7.40); κ_w on both heavies | heavy damage numbers at 1× combo with no mods for Karyst, Dual Ether, Kronen Prime, Gunsen, Kogake, Hirudo, Hespar, Redeemer Prime, Prisma Machete, Tak & Lug (both heavies) |
| OQ-724 | Do Blood Rush, Weeping Wounds and Gladiator read the combo before or after a heavy spends it? | before | heavy crit rate at 12× with Blood Rush, η = 0, vs the two predictions |
| OQ-725 | Does the light stance multiplier S_j multiply the Incarnon flat X? Is X multiplied by the slam's 2× and the heavy slam's 3×? | yes for S_j; no for slams | per-type damage of a 300 % light strike and of a 100 % strike of an Incarnon Genesis melee weapon with and without the flat evolution (TV-07-04: 1,828.5 vs 1,669.5 for a 30 flat); heavy slam numbers with and without the evolution |
| OQ-726 | S_j and H_j are ST-1 base factors (E-7.20, E-7.20b) for every CO behaviour, not `G` members | adopted: overview 2.4, 3.1, 3.1a, 3.3 and `04` E-4.6 (2026-11-24); `08`'s S_j field is display only (overview request to `08`) | settled for the spec; the only numeric difference (Incarnon flat on heavies) is settled by `W:Melee` line 681 |
| OQ-727 | Cadence and combo rules of weapons and attacks without stance timing: `cycle_loop` (Diwata, pseudo-exalted weapons, Vinquibus without a stance), `data_attack_loop` (weapon-unique data attacks), and the curation of unmapped `Ranged` stance entries (Harrowing Spire Block 1, Cobra & Crane and Sigma & Octantis Aerial 1, Vitrica Aerial 1) | 1 / AS and 1 / FireRate; no combo interaction; no damage until curated | attack-rate timing of Diwata and Whipclaw; Harrowing Spire block-combo damage numbers on a Vinquibus |
| OQ-728 | Combo expiry while the count is below IC (regeneration running): no change, or snap to IC? Timer when D_cb is undefined | no change; no timer | Corrupt Charge (D_cb 2.5 s), IC 220, η 0: read the count 3.0 s after the heavy, 0.5 s after the expiry (TV-07-27: 120 vs 220) |
| OQ-729 | Relentless Combination on the expected path: Slash-proc expectation without target-state SC terms (Tau) and without enemy status caps | as stated (E-7.13b) | combo count over 10 s of a Relentless Combination build on a Tau-afflicted target and on a boss vs the EV path |
| OQ-730 | Gunblade `[Ranged, Slam]` pairs: does the shot's AoE child hit the shot's target and apply its forced procs (Bleed + Stagger, Ragdoll)? Is the High Noon Block input 1 override right? | no; override `["", "Slam"]` | Bleed procs on the shot target from Bullet Dance Neutral input 1 at 0 % weapon status chance; High Noon Block input 1 damage numbers (300 % vs 100 % direct) |
| OQ-731 | Does a `Ranged` entry's stance physical bonus apply to the gunblade shot? | yes | Vastilok shot numbers with Bullet Dance input 1 vs the stanceless Neutral (TV-07-30: 970.3125 vs 776.25 per shot at 125 %) |
| OQ-732 | Rounding of the live combo count after a spend and during regeneration | floor(η·c⁻); whole regeneration points; EV unrounded | heavy at an odd count with η 0.5 (e.g. 141 → 70 or 71), read the counter |
| OQ-733 | Stealth eligibility of heavy strikes, heavy shots and waves, slams, throws and explosions; does a slam count as true-melee contact for the 3 s invisibility cooldown? | light-type strikes and gunblade shots sourced; light waves and the others eligible (UNVERIFIED); slams are contacts | yellow stealth numbers of each instance kind on an unalerted target and under invisibility |
| OQ-734 | `03` keeps the ragdoll state model (E-3.13) fed by this document's crowd-control proc events (`02` OQ-219, `03` OQ-326) | adopted: overview 7.1 and 7.2 make `03` §8.10 the owner | settled (the state's game rules stay `03` OQ-326) |
| OQ-735 | Power Spike decay floor at IC; combo gain of Melee Duplicate copies | floor at IC; no gain | Power Spike decay with IC 60 from 220 (stops at 60 or goes to 0?); combo count with Melee Duplicate at rank max on 100 % strikes |
| OQ-736 | Exalted Blade: light wave percentages from the stance data vs "same base damage" (`W:Exalted_Blade` line 19); heavy waves' `heavy` flag and both waves hitting a single target; Hysteria opening heavy composition (Normal Attack types vs Blast) and flags | stance data; `heavy`, both hit; Normal Attack types, slam flags | Exalted Blade Neutral input 2 wave vs blade numbers; heavy wave numbers with Killing Blow on and off on one target; Hysteria first heavy damage types |
| OQ-737 | Archmelee weapons (Archwing melee) and Ironbride are out of scope (overview 1.3) | adopted (overview 1.3); not scheduled | settled |
| OQ-738 | `08` E-8.0's melee sub-instances are the pellets of gunblade shots and 9-C Count attacks only (MS ≡ 1 for melee strikes) | adopted (`08` E-8.0) | settled |
| OQ-739 | Overview rows D-064 and D-065 `Compat` with the paths of section 12 (D-706 and D-707 are already `Compat` here), so that melee window DPS can be oracle-tested; the oracle harness reads the inspiration's stance table from the local bundle | adopted (overview register: D-064 → E-7.39, heavy combo; D-065 → E-7.38, hit timing, as here) | settled |
| OQ-740 | Melee Animosity: is the ground-throw and Volatile-explosion benefit without consumption still current? (The per-input stacking is adopted by `04` §5.4, `hit_unit = input`.) | modelled (`anim_read`); per input | glaive ground throw crit rate with 10 Animosity stacks, before and after a heavy |
| OQ-741 | Exalted Blade family wave CO: (a) do `04` rows 188-189 (`Light Waves` adding 2.0, `Heavy Waves` multiplying; tested on Exalted Umbra Blade) also hold for Exalted Blade and Exalted Prime Blade, whose data is identical? (b) Does the stance r_in (Exalted Blade 1.22-1.30) also multiply the CO term of a wave that has its own row? (c) `04` E-4.6: "r_in applies to every hit of the input" excludes row-covered waves (E-7.21) | (a) all three (adopted by `04`: the rows list them, OQ-444; Decision, UNVERIFIED, D6 is the alternative); (b) no (r_in = 1); (c) adopted (`04` E-4.6) | Neutral input 1 wave (300 %) with Condition Overload and 2 statuses, Str 1.0, no other mods, at point blank, on Exalted Blade (for (a)) and Exalted Umbra Blade: 3,150 (row, r_in 1), 3,822 (r_in·ρ_co), 2,286 (D6 with r_in 1.28) (TV-07-22, `04` TV-04-46) |

---

## 14. Test vectors

Each vector lists everything needed to reproduce it; f64 unless stated; all values checked with `python3`.

**TV-07-01 Attack speed and loop timing (E-7.1 to E-7.3).** Nikana Prime FR_src 1.08 → FR_w = 65/60 =
1.0833333333; Guiding Light D_K = 2.6, N_K = 8, ΣS = 19.0.

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
H = 6; A = 2.85; no elements; q = (22, 5, 5)/32; F = 0, G = M only, Part = 1 (P_body = 1, no ΣBP, no Sonar spot;
body h = 1). Y = 6237.
M = 8: per type 34,303.5 / 7,796.25 / 7,796.25, total 49,896; M = 12: total 74,844. CC = 0.40 × (1 + 1.20 + 1.20) = 1.36;
code = 320, Q = 2.5006105900 (f32), CDM = Q × 1.9 = 4.7511601210; E[c] = 6.1015777645; E[damage] at M = 8: 304,444.3241,
k = 1: 237,063.8854, k = 2: 424,231.7708; at M = 12: E 456,666.4862. CC without doubling: 0.88.

**TV-07-04 Incarnon flat on a heavy (E-7.20b).** B_ref = 200 (one type), X = 30, A = 1.65, H = 6, M = 4, no CO:
Y_heavy = (6·200 + 30) × 2.65 = 3,259.5; ×M = 13,038.0 (the inspiration's form `(B + X)·H·(1 + A)·M` gives 14,628.0,
D-702). A 300 % light strike of the same weapon: Y = 3 × 230 × 2.65 = 1,828.5 (Decision S_j·X); the alternative of
OQ-725, `S_j·(Bw + V) + X`, gives (3 × 200 + 30) × 2.65 = 1,669.5.

**TV-07-05 Heavy loop (E-7.16, E-7.30).** c(0) = 220 (T-09 12×), IC = 140 (an input: the rounded in-game value of
30 + 30 + 4 × 20; the exact 141.84 is worked in 10.3), η = 0.5, WU = 0.75, T_sw = 1.95, D_cb = 2.5, ρ_IC = 40,
FR_w = 48/60, window 20 s, executions emitted while τ_ex ≤ 20.

| ΣAS | policy | executions (time, c⁻, M) | count | ΣM |
|---|---|---|---|---|
| 0.55 (AS 1.24) | full_regen and back_to_back | (0.75, 220, 12), (3.072581, 140, 8), (5.395161, 140, 8), (7.717742, 140, 8) … last (19.330645, 140, 8) | 9 | 76 |
| 2.0 (AS 2.4) | full_regen | (0.75, 220, 12), (2.3125, 140, 8), (4.0625, 140, 8), (5.8125, 140, 8) … last (19.8125, 140, 8) | 12 | 100 |
| 2.0 (AS 2.4) | back_to_back | (0.75, 220, 12), (2.3125, 140, 8), (3.875, 132.5, 7), (5.4375, 128.75, 7) … last (19.5, 125.0073242, 7) | 13 | 97 |
| 0.55, η = 0.9 | back_to_back | (0.75, 220, 12), (3.072581, 198, 10), (5.395161, 178.2, 9), (7.717742, 160.38, 9), (10.040323, 144.342, 8), (12.362903, 140, 8) … | 9 | 80 |

**TV-07-06 Combo duration and Power Spike (E-7.12, E-7.14).** (D0, Σpct, Σflat) → D_cb: (5, 0, 0) → 5; (5, −0.5, 12) →
14.5; (5, −1.0, 0) → undefined (no gain); (5, −1.0, 10) → 10; (5, 0, 28) → 33; (6, 0, 0) → 6; (5, −1.0, 0.05) → 0.1.
Power Spike δ = 5, D_cb = 5, IC 0, c = 220 after a gain at t = 0 that started the timer (t_x = 5), then idle: at 4.99 s
220 (12×); 5 s 215 (11×); 100 s 120 (7×); 219.99 s 5; 220 s 0 (44 expiries); with D_cb = 17 the count reaches 0 at
748 s. Without that gain (a T-09 start count and no action) the timer never starts and the count stays at 220 (6.1).

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
and S^phys = 0.25 on IMP, PUN and SLA (the extra element dropped). Blind Justice Neutral: N_K = 8. Galeforce Dawn
Forward Block input 2: Dmg [100, 200, 100, 300], Procs [Knockback, Knockback, Bleed, ""], Types [Slam] → padded
[Slam, "", "", ""]: one Slam strike (100 %, Knockback; no single-target damage) and direct strikes of 200 % (Knockback),
100 % (Bleed) and 300 %; direct sum 600 % of the input's 700 %. High Noon Block input 1: Dmg [300, 100], Types [Slam] →
override ["", Slam]: a direct 300 % strike (Stagger, S^phys 0.25 on IPS) and a Slam strike (100 %, Knockback; no
single-target damage); without the override the direct strike would be the 100 % one.

**TV-07-13 Build and spend (E-7.30, build_spend(12)).** TV-07-02 inputs with ΣAS 1.45, Nikana heavy 500 % (Bleed,
Knockdown), η = 0, IC = 0, WU = 0.5, T_sw = 1.95 (→ 0.734694 s). The 93rd strike (index 92, t = 11.265306) brings c̄ to 220;
heavy input at 11.387755, execution at 11.887755 with c̄⁻ = 220, M_h = 12, n_anim = 10 (59 light inputs); light restarts
at 12.622449; strikes in [0, 20]: 154 (93 + 61), ΣS = 364; one heavy; c̄ at 20 s = 144.

**TV-07-14 Tennokai plan (E-7.17, E-7.30).** TV-07-02 inputs with ΣAS 1.45 (Δ = 0.122449), Discipline's Merit (n_tk = 4),
Nikana heavy, WU_tk = WU0 = 0.5 (s_tk = 0), T_sw/AS = 0.734694, IC 0. After strikes 0-5 (inputs 1-4; the 4th qualifying
event is input 4's first strike at 0.489796) the heavy input is at 0.734694, execution 1.234694 ≤ 0.489796 + 2 (inside
the window) with c̄ = 13 (M 1), no spending; light restarts at 1.969388. Executions (time, c̄, M): (1.234694, 13, 1),
(3.204082, 26, 2), (5.173469, 39, 2), (7.142857, 52, 3), (9.112245, 65, 4), (11.081633, 78, 4), (13.051020, 91, 5),
(15.020408, 104, 6), (16.989796, 117, 6), (18.959184, 130, 7); 63 light strikes; c̄ at 20 s = 137.

**TV-07-15 Fallback durations (E-7.3, E-7.5).** f_sl = median of the 11 stanceless ratios = 2.55 / 2.2 = 1.1590909
(Two-Handed Nikana). Stanceless Nikana Neutral (2 strikes): D_K = 2 × 0.4625 × 1.1590909 = 1.0721591 s; stanceless
Sword Neutral (4 strikes): 4 × 0.45 × 1.1590909 = 2.0863636 s; stanceless Polearm (4 strikes): 4 × 0.24375 × 1.1590909 =
1.1301136 s; stanceless Hammer (4 strikes): 2.5094318 s. Gleaming Talon Block (a stance combo, f_K = 1; 4 strikes): 4 × 0.425 = 1.7 s.
Heavy swing fallback τ_heavy = median(2.2, 1.65, 3.3, 1.0, 2.1, 1.8) = 1.95 s.

**TV-07-16 Exalted Blade wave falloff (E-7.43).** Φ_w at d = 0, 2.5, 35, 70, 80 m: 1.0, 0.9642857, 0.5, 0, 0.

**TV-07-17 Paracesis heavy, the wiki's anchor (E-7.20b, `W:Melee` lines 650-656).** Paracesis IMP 48.8, PUN 17.8,
SLA 155.4 (B = 222), Pressure Point 1.20 (the page's value), Fever Strike 0.90 Toxin, H = 6, M = 7, non-crit, F = 0.
Without quantization: 222 × 2.2 × 1.9 × 6 × 7 = **38,974.32** (the page's number). With E-3: r = (0.2198198, 0.0801802,
0.7000000, TOX 0.9), q = (7, 3, 22, 29)/32, Σq = 1.90625; per type IMP 4,487.175, PUN 1,923.075, SLA 14,102.55, TOX
18,589.725; total **39,102.525** (+0.33 %).

**TV-07-18 Reaper Prime heavy Bleed, the wiki's anchor (E-7.20b, overview E-13, `W:Melee` lines 660-677).** B = 200,
Pressure Point 1.20, Smite Grineer F = 0.30, H = 6, M = 5, non-crit: seed = 200 × 6 × 2.2 × 1.3 × 5 = 17,160; tick
without the "+1" = 17,160 × 0.35 × 1.3 = 7,807.8, × 6 ticks = **46,846.8** (the page's number); with the overview's
"+1" (D-025) 7,808.255 per tick, **46,849.53** in total.

**TV-07-19 Slam and heavy slam (E-7.32, E-7.20c).** Fragor Prime `Slam Attack` 540 Impact, `Heavy Slam Attack` 810 Blast
(CC0 0.40, CDM0 2.5), Primed Pressure Point 1.65, Killing Blow 1.20 (heavy slam only), Seismic Wave ΣSlam 2.00
(M_slam = 3), True Steel 1.20, Organ Shatter 0.90, epicentre (Φ = 1), Part = 1 (P_body = 1, no ΣBP, no Sonar spot;
body h = 1), CO none.
Slam: Y = 540 × 2.65 = 1,431; × M_slam = **4,293** non-crit; gain +1; no spending.
Heavy slam at c⁻ = 220 (M_h = 12): Y = 810 × 3.85 = 3,118.5; × 3 × 12 = **112,266** non-crit; CC = 0.40 × (1 + 2 ×
1.20) = 1.36 (doubled, OQ-712); E[c] = 6.1015777645; E[damage] = **684,999.7293**; gain 0; c⁺ = η·220.
Incarnon flat X = 30 on the heavy slam: (810 + 30) × 3.85 × 36 = **116,424** (Decision); the alternative X × 3 gives
124,740 (OQ-725).

**TV-07-20 Glaive cycle (E-7.36, E-7.6).** Glaive Prime `Charged Throw` ChargeTime 1.192, Killing Blow ΣWU 0.60 →
CT_g = 0.745 s; window [0, 20], D_cb 5.
IC 0, η 0: releases (= throw hits = detonations) at 0.745·k, k = 1 … 26 (last 19.37); each detonation sees c⁻ = 1 (the
throw's own gain), M_h = 1; n_thr = 1 … 26.
IC 140, η 0: regeneration wait 3.5 s, τ_{n+1} = r_n + 3.5: releases at 0.745, 4.99, 9.235, 13.48, 17.725 (5 cycles);
each detonation sees c⁻ = 140 + 1 = 141, M_h = 8.
Volatile Rebound rank 1 (p_vol = 0.5): one `Charged Throw Bounce Explosion` (592 Blast, no M, no spending) per throw with
weight 0.5, after the throw hit and before the detonation.

**TV-07-21 Combo-table resolution (E-7.0b, E-7.40).**

| Weapon (Class, Family), stance | Combo type | Table used | Hit list |
|---|---|---|---|
| Keratinos (Claws, Keratinos), Four Riders | Neutral | Four Riders Neutral (stance) | 300, 200, 200 Knockback, 200 Bleed, 400 Knockdown % |
| same | Heavy | Keratinos (unique, rule 1, σ = 1) | h1: 3 × 100, 3 × 100 Bleed, 300 (Bleed, Lifted); h2: 500 (Bleed, Knockdown) |
| same | Slide | Four Riders Slide (defined by the stance) | – |
| Quassus Prime (Warfan, Quassus), none | Heavy | Quassus (unique, Family match) | 2 × 250 Bleed + 12 Ethereal Daggers (9-C) / 1 × 500 Bleed + 6 daggers (Prime: 18 / 9 by Multishot) |
| Telos Boltace (Tonfa, Boltace), none | Slide | Telos Boltace (unique, name match) | 100 Knockback, 100 Lifted (Duration 1.48) |
| same | Heavy | Tonfa stanceless (the unique Heavy is a FromWeapon copy) | 2 × 250 (Bleed, Knockdown) / same |
| Exalted Blade (Exalted Weapon), fixed Exalted Blade | Neutral | Exalted Blade "Cutting Poise" | blade and wave entries (9.8) |
| same | Heavy | Exalted Blade's FromWeapon copy of the Sword Heavy (exalted rule), rule 2 with class Sword (5×, σ = 1) | 500 Lifted / 500 Knockdown, + 2 waves each |
| Desert Wind (Exalted Weapon), fixed Serene Storm | Heavy | Serene Storm "Rise From Ashes" (rule 1) | h1: 500 Knockdown, 400 Slam (Disarmed) excluded; h2: 4 × 300 Lifted, 300 |
| Shadow Claws (Exalted Weapon), fixed Ravenous Wraith | Heavy; Wall | Ravenous Wraith (rule 1); Wall does not exist | 2 × 400 (Knockback, Bleed) / 2 × 400 Bleed; – |
| Iron Staff (Exalted Weapon), fixed Primal Fury | Heavy | Primal Fury "The Way" (rule 1) | 500 Knockdown / 500 Knockdown |
| Diwata (Exalted Weapon), fixed Razorwing | Neutral | none (no D:Stances table) | `cycle_loop` |
| Vinquibus (Melee) (Bayonet), none | Neutral; Heavy | none (Bayonet has no Neutral); Bayonet stanceless Heavy (σ = 1, no 9-A row) | `cycle_loop`; h1: 500 Puncture + 500, h2: 500 Puncture |

**TV-07-22 Exalted Blade waves (E-7.43).** Str 1.0, Normal Attack IMP 37.5, PUN 37.5, SLA 175 (q = 5/32, 5/32, 22/32,
Σ = 1), no mods unless stated. Neutral input 1 (300 % blade, Stagger; 300 % wave): blade per type 117.1875, 117.1875,
515.625 (Σ 750); wave 750 at d = 0, 375 at d = 35 m (Φ_w = 0.5, flight 35/30 = 1.1667 s); gain 3 + 3 = 6. Heavy 1 at
M_h = 12 with Killing Blow (A = 1.20 on every `heavy` instance): blade strike 5 × 250 × 2.2 × 12 = 33,000; each wave 5 ×
250 × 2.2 = 2,750 (no M_h); heavy total 38,500 at d = 0, 35,750 at d = 35 m; gain 0. Normal slam (500 Impact): CC0 = 0,
no M, gain 1 (Desert Wind: gain 0).

CO per wave id (E-7.43, E-7.21, `04` OQ-444, OQ-741): Condition Overload with 2 statuses, C = 1.60; Neutral r_in = 1.28;
the same inputs and the same numbers on Exalted Blade, Exalted Prime Blade and Exalted Umbra Blade (`04` rows 188-189
list all three). Blade strike (D6, adding, r_in 1.28): 750 + 1.28 × 750 × 1.6 = 2,286. Light wave (`Light Waves`, row
188: adding, ρ_co = 2 on B_ref,j = 750, r_in = 1): 750 + 2 × 750 × 1.6 = **3,150** at d = 0, 375 + 2,400 = **2,775** at
d = 35 m (Φ_w is in G_F, which the CO term omits, `04` E-4.3). Heavy wave (`Heavy Waves`, row 189, multiplying) with
Killing Blow: 1,250 × 2.2 × 2.6 = **7,150** at d = 0, **3,575** at d = 35 m. Heavy 1 at M_h = 12 with CO: blade strike
(D6, Heavy column blank, r_in 1) (2,750 + 2,000) × 12 = 57,000; heavy total at d = 0: 57,000 + 2 × 7,150 = **71,300**.
Alternatives (`04` TV-04-46): r_in·ρ_co on the light wave, 3,822; D6 on the waves (the page's default for unlisted
attacks, should OQ-741 (a) settle against the rows for Exalted Blade and Exalted Prime Blade): light wave (r_in 1.28)
2,286 and 1,911 at d = 35 m, heavy wave (r_in 1) 2,750 + 1,250 × 1.6 = 4,750 and 3,375 at d = 35 m, heavy total 66,500.

**TV-07-23 Gunblade heavy shots (E-7.45, 9.5).** Redeemer Prime `Ranged Attack` 80 Blast per pellet, Multishot 10, CC
0.24, CDM0 2.2 (code 282, Q = 2.2036631107), heavy 1 = 2 shots at 400 % (s_j = 4), heavy 2 = 1 shot; Killing Blow
A = 1.20, True Steel 1.20, M_h = 12, d = 0, CO none. Per pellet Y = 320 × 2.2 = 704, × 12 = 8,448 non-crit; CC = 0.24 ×
3.40 = 0.816; E[c] = 1.9821890984; E per pellet 16,745.5335; heavy 1 (20 pellets) **334,910.6701**, heavy 2 (10 pellets)
167,455.3350; gain 0; the two Slam AoE children of heavy 1 deal nothing. Under the 9-C default (no heavy bonus): pellet
3,840 non-crit; True Steel still applies once (CC = 0.24 × 2.20 = 0.528), E[c] = 1.6355341225, heavy 1
**125,609.0206** (OQ-717).

**TV-07-24 Blood Rush read before consumption (E-7.18).** Heavy at c⁻ = 220 (M_b 12), η = 0, CC0 0.28, True Steel 1.20
(doubled), Blood Rush 0.40·(M_b − 1): CC = 0.28 × (1 + 2.40 + 4.40) = **2.184** (Decision); after-consumption reading
(OQ-724): 0.28 × 3.40 = 0.952. The next light strike (c⁻ = 0): 0.28 × 2.20 = 0.616.

**TV-07-25 Expected gains (E-7.13, E-7.13b, 6.9).** A 300 % strike (g = 3), p_acc = 0.20: expected gain 3.2; with the
curse x_cur = 0.561: 0.439 × 3.2 = 1.4048; x_cur = 0.685: 1.008; x_cur = 1.038: **0** (p_gain = max(0, −0.038) = 0;
heavies still spend). Redeemer Prime shot (10 pellets) with p_acc 0.20: 12.0. Relentless Combination: one instance at
t = 0 with π = 0.6, p_RC = 1.00, p_acc = 0.20, SD = 0 (N_SLA = 6, δ_SLA = 1): events at t = 0, 1, …, 6 of 0.72 each,
**5.04** by t = 6; rank 0 (p_RC = 0.25): 0.18 each, 1.26; with x_cur = 0.561: 0.31608 each, 2.21256.

**TV-07-26 Combo tiers (E-7.10, E-7.11).** Default (k_tier 20, M_cap 12): c = 19, 20, 219, 220, 1000 → M = 1, 2, 11, 12,
12. Venka Prime (M_cap 13): c = 219, 220, 239, 240, 400 → 11, 12, 12, 13, 13; M_b = 11, 12, 12, 12, 12. Dex Nikana
(k_tier 11, M_cap 11, M_bcap 11): c = 10, 11, 109, 110, 300 → 1, 2, 10, 11, 11; M_b = M.

**TV-07-27 Expiry during regeneration (E-7.14, OQ-728).** Timing of TV-07-05 with ΣAS 0.55 (AS 1.24, WU 0.75, T_sw
1.95), IC = 220, η = 0, D_cb = 2.5 (Corrupt Charge), c(0) = 220. After the first heavy (execution 0.75, c⁻ = 220,
c⁺ = 0, t_x = 3.25): c(3.0) = 90; at the expiry 3.25, c = 100 and the regeneration continues (Decision): c(3.5) = 110,
c(6.25) = 220. The snap alternative would give c(3.25⁺) = 220. `full_regen`: executions 0.75, 6.25, 11.75, 17.25 (all
c⁻ = 220, M 12; 4 heavies, ΣM 48). `back_to_back`: 0.75 (220, 12), then every 2.322581 s with c⁻ = 92.903226 (M 5);
9 heavies, ΣM 52; each next wind-up starts before t_x and freezes the timer, so no expiry occurs and both rules agree.

**TV-07-28 Steady state of a heavy plan (E-7.37).** TV-07-05 inputs, ΣAS 2.0, `back_to_back`: c̄⁻ of heavy n ≥ 2 is
125 + 15 · 2^−(n−2); at every execution from heavy 2 on the regeneration deficit is IC − c̄⁻ and the timer remaining
is 1.6875 s (1,687,500 µs on the grid). With c̄ rounded to 1e-9, the key of heavy 39 (τ = 60.125, h = 1) is the first to equal an earlier one, heavy 37
(τ = 57.0, h = 1, c̄⁻ → 125.0): period 3.125 s with heavies 37 (h = 1) and 38 (h = 2), both M_h = 7. With TV-07-03's
expected damage per heavy at M = 7 (6,237 × 7 × 6.1015777645 = 266,388.7836): dps_burst = 2 × 266,388.7836 / 3.125 =
**170,488.8215**.

**TV-07-29 Stance CO ratio (E-7.21, `04` E-4.6).** Single-type weapon, B_ref = 100, A = 1.65, CO C = 0.80 × 2 statuses
= 1.60 (`adding`), G_F = 1, X = 0. Gnashing Payara Neutral (r_in = 0.36), a 200 % strike: Y = 100 × 2 × 2.65 + 0.36 ×
2 × 100 × 1.60 = **645.2** (r_in = 1: 850.0). Stanceless Dual Daggers heavy strike H = 2.5 with the class's shared
heavy column (Gnashing Payara row, 0.40): Y = 250 × 2.65 + 0.40 × 250 × 1.60 = **822.5** (`04` E-4.6 agrees); the
alternative of OQ-716, 1.0 for stanceless combos, would give 1,062.5.

**TV-07-30 Gunblade shot with a stance physical bonus (E-7.45, OQ-731).** Vastilok `Ranged Attack` IMP 8.97, PUN
10.35, SLA 49.68 (69 per pellet), Multishot 9, no mods. Bullet Dance Neutral input 1 (125 % Ranged, physical ×1.25):
B' = 86.25 per pellet; r = 0.1625, 0.1875, 0.9 → q = 5/32, 6/32, 29/32 (Σ 1.25); per pellet 13.4765625, 16.171875,
78.1640625 (Σ 107.8125); per shot **970.3125**. Without the physical bonus: q = 4/32, 5/32, 23/32 (Σ 1); 86.25 per
pellet, **776.25** per shot.

**TV-07-31 Light-loop steady state with sparse gains (E-7.37).** Fragor Prime (FR_w 48/60) stanceless Hammer Neutral:
strikes 400 % Slam, 300 % Sweep, 300 % Slam, 400 % Slam; only the Sweep gains (g = 3), so gap_max = T_loop; D_K =
2.5094318 s (TV-07-15); IC 0. ΣAS 0 (AS 0.8): T_loop = 3.1367898 s, Δ = 0.7841974 s. With D_cb = 2.5 (Corrupt Charge)
gap_max ≥ D_cb, so the periodic rule applies: the count is 3 for 2.5 s after each gain and 0 otherwise, the key repeats
from the second loop start (period T_loop), steady M_b = **1** (the old test Δ < D_cb would have given 12). With
D_cb = 5: closed form, steady M_b = 12 (the 74th gain, at 229.770 s, reaches 222). ΣAS 0.6 (AS 1.28): T_loop = 1.9604936 s
< 2.5, closed form, 12.

---

## Review notes

Two verifiers reviewed the draft; every reported issue was checked against the sources and applied, except the parts
noted below (one line each).

- §9.8 wave speed and Desert Wind slam crit "unsourced": rejected; `W:Exalted_Blade` line 19 (30 m/s) and `W:Serene_Storm` line 37 (no slam crit) source them, and the data values are reported as data warnings.
- Register row for the gunblade `[Ranged, Slam]` difference from `M:Stances`: rejected; the register records inspiration differences (overview 5.1), and the choice and its alternative are in 7.1 and OQ-730.
- Zaw slides from the module's `Multi.Slide`: not adopted; the class Slide combo keeps zaws consistent with 9.3, and the module value is reported (OQ-719).
- Adopting `S_j·(Bw + V) + X` for light strikes: not adopted; no source covers light strikes, so the Decision stays UNVERIFIED with the alternative in OQ-725.
- S_j and H_j placement and the D-064 / D-065 Compat switches: applied as proposals only (OQ-726, OQ-739), because the overview, `04` and `08` are outside this document.
- Stanceless fallback "copied from an older module revision": rejected; revision 2594921 (2023) has the same 11 durations, and the rest (D-708, f_sl, OQ-701) is applied.
- Ragdoll state: of the two offered fixes the owner proposal was taken (`03` §8.10, OQ-734); this document emits proc events, not ragdoll intervals.
- Gunblade `[Ranged, Slam]` evidence that the 2026-06-24 edits added the pairs to the Gunblade heavy: corrected; revision 2765569 (2026-04-15) already has them (7.1), the fix itself is applied.
- Re-verification also corrected the Serene Storm heavy procs (Disarmed sits on the excluded Slam entry), the TV-07-23 no-bonus value (True Steel still applies once), OQ-728's test numbers and two line citations.

Follow-up round (2026-11-24), checking this document against the revised overview and `04`:

- S_j and H_j in the ST-1 base (OQ-726): the overview (2.4, 3.1, 3.1a, 3.3) and `04` E-4.6 now state E-7.20 and
  E-7.20b; sections 1, 7.2, 7.4, EC-07-65 and OQ-726 say so instead of "proposal". Base, CO term (`r_in·κ·B_co,j·C`,
  `B_co,j = ρ_co·B_ref,j`) and every test vector agree with overview 3.3; no number changed.
- Toggles: the melee plan is T-30 (inputs, 8.1, OQ-722) with the parameters of 8.1; the exalted rank is T-31 (E-7.43
  now writes `m_str = Str·R_r` as `01` E-1.1); finisher mode is T-34.
- Stance CO ratio for shared combos (OQ-716): `04` E-4.6 follows E-7.21; the "conflict" text of 7.6 and TV-07-29 is
  replaced. Melee Animosity per input: `04` §5.4 uses `hit_unit = input` (7.5, OQ-740).
- Time grid (overview CD-14, `08` E-8.0a; 07 is a listed consumer in overview 7.2): 8.2, 8.3 and E-7.37 state their
  time tests on the µs grid. TV-07-28 rechecked with python3 (period 3.125 s, 170,488.8215).
- Adopted proposals marked as settled: OQ-715 (`03` table 4.2), OQ-734 (overview 7.1, 7.2), OQ-737 (overview 1.3),
  OQ-739 (overview register). Kept against the overview: its Toggle column pairs D-064 with E-7.38 and D-065 with
  E-7.39, but E-7.38 is this document's hit-timing path (D-065) and E-7.39 its heavy-combo path (D-064); section 12
  keeps the correct pairing and the overview should swap the two references.
- Still open for `04` (OQ-741): rows 188-189 for Exalted Blade and Exalted Prime Blade, and the E-4.6 note that r_in
  does not apply to row-covered waves. Still open for `08` (OQ-738): "melee multishot copies" in E-8.0. (Both settled
  in the consistency round below.)

Consistency round (2026-11-24):

- Exalted Blade family waves: `04` now lists Exalted Blade, Exalted Prime Blade and Exalted Umbra Blade on rows 188-189
  (OQ-444) and excludes row-covered waves from the input's r_in (E-4.6). E-7.21, E-7.43, the 9.8 sources, EC-07-50 and
  OQ-741 ((a) and (c) adopted) follow; TV-07-22 gives every blade of the family the row values (3,150 / 2,775, heavy
  wave 7,150 / 3,575, heavy total 71,300) and keeps 2,286 / 1,911 / 4,750 / 3,375 / 66,500 as the D6 alternative, equal
  to `04` TV-04-46 (rechecked with python3). OQ-738 is adopted by `08` E-8.0 (section 1 and the OQ row updated).

Radial Part round (2026-11-24):

- Radial melee instances (slams, heavy slams, glaive explosions, the Hysteria opening heavy, gunblade `slam_part`
  children) no longer use `Part = 1`: the `radial` flag of 7.5, E-7.32, 9.4, 9.8 and 7.1 now cite overview E-7's
  radial branch (`P_body·(1 − ΣBP)·M_son`, the body part's P and h, `02` E-2.10, `05` §4.3), as `06` §9.6 does.
  TV-07-03 and TV-07-19 keep their numbers (rechecked with python3) and state Part = 1 as P_body = 1, no ΣBP, no Sonar
  spot, body h = 1.

Compat pairing round (2026-11-24):

- OQ-739 settled: the overview register now pairs D-064 with `07` E-7.39 and D-065 with `07` E-7.38 (00 D-064, D-065
  and its review notes), as section 12 does; the section 12 paragraph and the OQ-739 row say so. The follow-up round's
  "kept against the overview" note above is resolved.
