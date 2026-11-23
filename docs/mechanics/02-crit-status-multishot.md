Status: draft (spec)

# Damage-calculation spec, part 02: crit, status chance and multishot

This document defines, for one damage instance, everything random or location-dependent that sits between the
weapon-side scalar and the enemy side: how many pellets a shot fires, the crit chance, the crit multiplier, the rolled
crit tier and its damage factor, the body-part factor, the status chance, the number and types of procs (rolled and
forced), and the expectations of all of these for the expected-value (EV) evaluator. It refines equations E-4 to E-10
of [00-overview.md](00-overview.md) and never contradicts them; where the research found that the overview needs a
correction, the proposal is in section 8 (open questions) and this document states which behaviour the engine uses
until the manager decides.

Related documents: [00-overview.md](00-overview.md) (notation, E-1 to E-15, CD-xx, D-xxx, T-xx),
[01-damage-composition.md](01-damage-composition.md) (composition `q_t`, `r_t`), [03-status-effects.md](03-status-effects.md)
(what a proc does, stacks, caps, DoT, `punc_cc`, `cold_cd`, Tau stack values), [04-conditional-bonuses-and-buffs.md](04-conditional-bonuses-and-buffs.md)
(catalogue of every buff, arcane, shard and riven value and its condition), [05-enemies.md](05-enemies.md) (body-part
data, head-crit capability, status immunities), [06-ranged-firing-model.md](06-ranged-firing-model.md) (shot schedule,
fire rate, beams, spool, parent and child attacks), [07-melee.md](07-melee.md) (combo multiplier, heavy attacks,
stance hit data incl. forced procs), [08-evaluation-and-outputs.md](08-evaluation-and-outputs.md) (EV algorithm, MC
simulator, RNG keying), [09-build-rules.md](09-build-rules.md) (which mods and set pieces a build may contain).

Contents:

- [1. Scope and stages](#1-scope-and-stages)
- [2. Inputs](#2-inputs)
- [3. Outputs and symbols added](#3-outputs-and-symbols-added)
- [4. Rules and formulas](#4-rules-and-formulas)
- [5. Worked example](#5-worked-example)
- [6. Edge cases](#6-edge-cases)
- [7. Inspiration differences](#7-inspiration-differences)
- [8. Open questions](#8-open-questions)
- [9. Test vectors](#9-test-vectors)

Citation keys are those of 00 §0.2. Cached wiki pages fetched for this document and added to
`research/external/cache/wiki/`: `Hemorrhage`, `Magnetic_Welt`, `Piercing_Navigator`, `Secondary_Enervate`,
`Seeking_Talons`, `Hard_Engage`, `Meticulous_Aim`, `Target_Acquired`, `Archon_Continuity`, `Archon_Vitality`,
`Secondary_Cryogenic`, `Primary_Debilitate`, `Strun_Incarnon_Genesis`, `Boar_Incarnon_Genesis`. Revision dates quoted
below come from the MediaWiki revisions API (2026-11-23).

---

## 1. Scope and stages

Scope (from 00 §7.1, row `02-crit-status-multishot.md`): E-4 to E-10: crit chance (relative, absolute, floor,
Piercing Navigator, combo hooks), crit multiplier (quantization, relative, absolute, field multipliers), tiers and
Vigilante, headshots, weak points, head-crit, Sonar, body-part rules for radial and 1×-headshot weapons; status chance
(relative, absolute, Tau multiplier, per pellet, Kohm division); proc count and type selection, immunity
renormalization, ragdoll Impact; forced procs (weapon traits; stance procs as data from `07`); Hunter Munitions,
Internal Bleeding, Hemorrhage, Magnetic Welt; multishot (floor, Acuity lock, beam merge CD-10, speargun throw).
Not: status effects themselves (`03`), body-part data per enemy (`05`).

Stages owned: ST-6 (body part), ST-7 (crit), and the rolling half of ST-10 (which procs an instance carries; `03`
applies them). Multishot (E-10) feeds the instance count of every stage.

Boundary rules used in this document:

- Values of buffs, arcanes, shards, rivens and their conditions are `04`'s. This document defines only the **bucket**
  each one enters and any hit-context rule (weak point, radial, heavy) that changes the bucket.
- The value of a stack-dependent bonus (`punc_cc(n)`, `cold_cd(n)`, Tau `0.1·n`) and the stack count rules (caps,
  timers, CD-04 timing) are `03`'s. This document defines where the value enters and how its expectation is taken.
- Body-part multipliers, head-crit capability, crit-promotion flags and status immunities are data supplied by `05`.
- Which attacks are beams, children, spool states or throws, and their fire rate, are `06`'s; combo multiplier,
  heavy flag and stance-hit forced procs are `07`'s.

---

## 2. Inputs

| Symbol (00 §2.4 or added) | Meaning used here | Owner |
|---|---|---|
| `ratio_q[t]` (q_t), `ratio[t]` (r_t) | quantized and unquantized composition of the attack | `01` |
| `cc_base`, `cd_base`, `sc_base`, `ms_base` | attack data `CritChance`, `CritMultiplier`, `StatusChance`, `Multishot` (per pellet; kitgun and zaw values already assembled) | data (`06`, `07` assemble modular weapons) |
| `combo_mult` (M) | melee combo multiplier of the current hit | `07` |
| heavy flag, stance hit forced procs, stance `StatusMultiplier` | per melee hit | `07` |
| `fire_rate` (FR) | modded fire rate of the attack (Internal Bleeding threshold) | `06` |
| beam flag, spool step, `ms_applies` flag, radial flag (`ShotType = "AoE"`) | per attack | `06` |
| `stacks[s]` (n_PUN, n_COL, n_TAU) before the instance | target state, CD-04 | `03` (EV distribution: `08`) |
| `punc_cc(n)`, `cold_cd(n)` | stack value functions | `03` |
| part data: `part_mult` (P_part), weak-point flag, headshot flag, head-crit flag and factor, crit-promotion flag | per enemy part | `05` |
| status immunity set `Imm` (incl. "all") | per enemy | `05` |
| ragdoll state | target state | `03` (see OQ-219) |
| every bonus value (ΣCC_rel entries, ΣCD_abs entries, Covenant raw value, Sonar multiplier and node count ...) | catalogue with conditions (T-02) | `04` |
| `strength` (Str) | for Strength-scaled values | `04` / T-20 |
| T-03 hit location, T-18 flat crit, T-21 multiplicative weak-point crit chance, T-26 custom stats | toggles | 00 §6 |

---

## 3. Outputs and symbols added

Outputs per damage instance (consumed by E-12, E-13, E-14 and by `03`, `04`, `08`): pellet count `n_pel` (or merged
beam count `m`), `cc`, `cd`, `crit_tier` k (MC) or its distribution (EV), crit factor `c_k`, `headcrit` h, `part`,
`sc`, the list of procs with their source class, and the EV moments of section 4.9.

Symbols added (prefixed per 00 §0.3):

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `ms_base_add` | MS0+ | count | flat additions to base multishot before mods (Strun Incarnon "Blazing Barrel" +0.05 per stack; inspiration hook `flat_multishot`) |
| `ms_lock` | – | bool | build contains an Acuity mod (Primary Acuity, Pistol Acuity) |
| `ms_applies` | – | bool | multishot modifies this attack (false: speargun throw, hitscan explosions, beam AoE; data from `06`) |
| `ms_pellets` | n_pel | count | pellets of one shot (non-beam) |
| `ms_beam_count` | m | count | beams merged into one tick (beam attacks; `03` writes it b) |
| `cc_wp` | ΣCC_WP | bonus | the weak-point-only part of the relative crit chance (Acuity, Worthy Comradery, Cyte-09 passive, riven Weak Point Critical Chance, Incarnon "on weak point hits", custom p.f) |
| `cc_mult` | M_cc | factor | custom "Critical Chance (multiple with others)" (p.d), 1 by default; no game source |
| `cc_roll` | CC_r | fraction | `max(0, CC)`: the value the tier is rolled from |
| `cd_base_cond` | CDM0c | factor | conditional base crit multiplier addition that depends on the hit's CC (Braton Incarnon "Prelude of Might") |
| `crit_promo` | π_p | count | crit tiers added by the hit part (`05` field `tier_add`, 0 by default; 1 on Critical Promotion parts) |
| `crit_pk[k]` | P_k | probability | crit tier distribution of the instance |
| `crit_mean_k`, `crit_mean_c`, `crit_mean_c_crit` | E[k], E[c], C1 | – | moments of section 4.4 |
| `part_wp`, `part_hs` | – | bool | the instance hits a weak point (incl. Sonar spot) / a headshot part |
| `part_hs_unique` | M_hsu | factor | unique headshot multiplier multiplicative with ΣHS (Cernos Prime 1.5) |
| `part_body_pen` | ΣBP | fraction | bodyshot damage penalties on non-headshot hits (Meticulous Aim 0.45) |
| `sc_weapon` | SC_w | fraction | weapon-side status chance per pellet (before target status-chance vulnerability) |
| `sc_vuln` | M_sc,t | factor | target-side status-chance vulnerability (Tau stacks, custom p.k, Pageflight) |
| `sc_roll` | SC_r | fraction | `max(0, SC)` (merged beams: `m·SC_r`) |
| `proc_gen(z)` | G(z) | – | probability generating function of the rolled proc count (E-2.12) |
| `proc_any[s]` | A_s | probability | P(instance carries ≥ 1 proc of s) |
| `proc_mean[s]` | E[N_s] | count | expected procs of s per instance, all sources |
| `proc_enc` | P_enc | probability | Secondary Encumber proc per shot |
| `p_welt` | p_W | fraction | Magnetic Welt chance per instance (0.35, ×2 when FR < 2.5) |
| `p_talons` | p_ST | fraction | Seeking Talons forced-Slash chance on a marked target (0.75 × Str) |
| `cc_cov` | v_cov | fraction | Covenant crit chance before the body and weak-point caps (value from `04`) |
| `cd_doughty` | doughty | factor | Melee Doughty absolute crit multiplier (E-2.5a) |
| `part_sonar_mult`, `part_sonar_nodes` | s_son, ν | factor, count | Sonar multiplier per node and number of highlighted nodes on the hit part |
| `proc_enc_chance` | c_enc | fraction | Encumber chance per pellet that delivers a status (0.24 at max rank) |
| `proc_pi_ib`, `proc_pi_welt` | π_IB, π_W | probability | P(instance has an Impact status and no own Slash / Magnetic) |

---

## 4. Rules and formulas

Numeric conventions: U-1 to U-6 of 00 §2.1. All chances are fractions; nothing is rounded to whole percent (D-017).
`floor`, `frac(x) = x − floor(x)`. f32 is used only in E-2.5 (NP-1).

### 4.1 Multishot (E-10)

```text
E-2.1  Modded multishot of an attack
  if not ms_applies:                MS = 1                                      (speargun throw, hitscan explosion, beam AoE)
  elif ms_lock:                     MS = MS0                                    (Acuity: base value, every bonus and penalty ignored)
  else:                             MS = max(1, (MS0 + MS0+)·(1 + ΣMS))
```

- ΣMS sums every multishot bonus (mods incl. Galvanized Chamber/Diffusion/Hell, riven multishot of either sign,
  Vigilante Armaments, Metronome Opera, Primary Frostbite/Blight, Conjunction Voltage, Primary Overcharge, Split Flights,
  Shotgun Vendetta, Braton Incarnon Evolution II multishot, custom p.q). Members and values: `04`. Additivity of
  Shotgun Vendetta with mods is UNVERIFIED (`F:addmul` §5); it is put in ΣMS.
  Source: `F:core` §7, `W:Multishot` §Mechanics, §Calculating Multishot (Lex +180 % → 2.8; Hek 7 × 2.2 = 15.4).
- Floor at 1: a riven multishot curse "can no longer go below 1" (`W:Multishot` §Sources, D-019). The floor is applied
  to every weapon, also with MS0 > 1 (UNVERIFIED for MS0 > 1, the page's only example is MS0 = 1).
- Acuity lock: since U38 an Acuity mod "prevent[s] the selected weapon's multishot value from being modified by any
  means (even penalties from Riven Mods)"; since U44 only Primary Acuity and Pistol Acuity exist (`W:Multishot`
  §Locking Multishot, `W:Primary_Acuity` §Notes). MS0+ is ignored too (Decision: "any means").
- MS0+: Strun Incarnon "Blazing Barrel: On Firing: +0.05 Base Multishot. Stacks up to 5x", "adds 0.25 pellets to the
  base Multishot of both forms when fully stacked" (`W:Strun_Incarnon_Genesis`). The stack count is a condition (`04`).

```text
E-2.2  Pellets of one shot (non-beam), per roll and expectation
  n = floor(MS), f = frac(MS)
  n_pel = n + Bernoulli(f)                              (one draw per shot; mechanic id "ms_pellets")
  E[n_pel] = MS ;  Var[n_pel] = f·(1 − f) ;  E[n_pel²] = MS² + f·(1 − f)
```

Each pellet is its own damage instance with its own crit roll and its own status roll (CD-03, CD-10; `W:Multishot`
in-game description "Each damage instance has its own chance to apply Critical Hits or Status Effects"). All pellets
are assumed to hit the target (`W:Multishot`: "the in-game UI will show the sum of all the projectiles' damage");
spread and accuracy are not modelled (EC-02-11).

```text
E-2.3  Beams (continuous weapons, CD-10)
  m = floor(MS) + Bernoulli(frac(MS))                   merged beam count of one tick (mechanic id "ms_pellets")
  one damage instance per tick:  W_t ← m·W_t ;  SC_tick = m·SC_r ;  one crit roll with the CC of one beam
  forced procs (weapon traits, Hunter Munitions, Internal Bleeding, Seeking Talons) once per tick, not × m
  DoT seeds of the tick carry the factor m (E-13, CD-10)
  E[m] = MS ;  E[m²] = MS² + f·(1 − f)
```

Source: `W:Multishot` §Continuous Weapons ("additional beams that hit the same target instead merge into a singular
damage tick ... damage **and** Status Chance equal to the sum ... Critical Chance ... of a single beam ... Forced status
effects ... are applied after the damage instances are merged"), `F:weapon` §4.2, `F:core` §7. Innate-multishot beams
(Quanta, 2 beams) merge the same way on one target (same page; see OQ-213 for its contradictory Quanta example).
Status is rolled per tick since U22.15 (`F:weapon` §4.1).

Attacks with `ms_applies = false` (data supplied by `06`):

- Speargun thrown component: "Multishot has no effect on the thrown component of Spearguns" (`W:Multishot`
  §Spearguns).
- Spherical blast of continuous weapons (Ignis, Ignis Wraith, Glaxion Vandal, Gaze primary, Embolist, Catabolyst,
  Cortege) (`W:Multishot` §Continuous Weapons).
- Hitscan explosions: Opticor, Opticor Vandal, Trumna, Ferrox, Tenet Ferrox, Ambassador alt, Braton and Burston
  Incarnon radial, Glaxion Vandal (`F:weapon` §10.1, `F:addmul` §1 catalogue "AoE does not scale off multishot").
- Projectile launchers create one projectile per pellet; each projectile's child explosion is its own instance
  (implied by `F:weapon` §10.1, UNVERIFIED, OQ-212). `06` owns the parent/child instance count.

Multishot-generated pellets: a few perks act only on "shots generated by multishot" (Braton Incarnon Evolution II
"+Y Damage", `F:core` §3.1; Torid "Plentiful Mayhem" +60 % damage, `F:mods` §9). For such a perk the generated pellets
of a shot are `n_pel − 1` (the first pellet is the original), expectation `MS − 1`. For MS0 > 1 it is UNVERIFIED
whether the generated pellets are `n_pel − 1` or `n_pel − MS0` (OQ-212); no such perk exists on an innate-multishot
weapon in the research. The damage multiplier itself is in `G` (`04`).

### 4.2 Crit chance (E-5)

```text
E-2.4  Crit chance of damage instance i
  CC_b   = CC0 + CC0+
  T-21 off:  CC_raw = ( CC_b·(1 + ΣCC_rel + wp_i·ΣCC_WP) + ΣCC_abs,i )·M_cc
  T-21 on:   CC_raw = ( CC_b·(1 + ΣCC_rel)·(1 + wp_i·ΣCC_WP) + ΣCC_abs,i )·M_cc            (D-023, link import only)
  CC     = max(CC_min, CC_raw)
  CC_r   = max(0, CC)                                       (CC ≤ 0: no hit can crit)
  ΣCC_abs,i = ΣCC_abs,static + rad_i'·punc_cc(n_PUN) + cov_i
  wp_i   = 1 if instance i is a direct (non-radial) hit on a weak point (head weak point, non-head weak point or
           Sonar spot), else 0;   rad_i' = 1 for non-radial instances, 0 for radial instances
  cov_i  = min(v_cov, 0.5) if wp_i = 0 ;  min(4·v_cov, 2.0) if wp_i = 1          (v_cov: Covenant value from 04)
```

ΣCC_rel in E-2.4 is the general relative bucket; ΣCC_WP is kept separate only because T-21 needs it. Without T-21 the
two are one additive bucket, as in 00 E-5.

Sources: `F:core` §8; `W:Critical_Hit` §Relative Increases, §Absolute Increases ("applied after relative bonuses",
Braton `12 % × (1 + 150 %) + 45 %`); `W:Calculating_Bonuses` §Flat Value and Percentage Point Bonuses lines 92-102; `W:Enemy_Body_Parts` §Weak Point
Hits ("weak point critical chance bonuses are applied to the shot, additive with bonuses like Point Strike");
`W:Covenant` lines 17-20 (×4 on weak points, caps 50 % body and 200 % weak point, "flat value applied after mods");
`W:Charm` lines 34, 92, 105-106 (floor); `W:Critical_Hit` §Critical Tiers table ("x ≤ 0 %: no hit can crit");
`JS L8194-8267` (inspiration form, D-023).

Bucket membership (values and conditions in `04`; the list is the research's complete list of named sources):

| Bucket | Members |
|---|---|
| CC0+ (before mods) | Incarnon "Increase Base Critical Chance by +X %" (`F:weapon` §13: `CC = (CC_base + X)·(1 + ΣCC_mods)`); inspiration hooks `add_crit_chance` and the cross-stat Incarnon perk "Increase Base Critical Chance by +x % of current Status Chance" (OQ-220) |
| ΣCC_rel | Point Strike family, Critical Delay, riven Critical Chance, Argon Scope, Galvanized Scope/Crosshairs, Hydraulic Crosshairs, Laser Sight, Motus Setup, Mark of the Beast, Blood Rush `0.4·(M − 1)`, Gladiator set `0.1·pieces·(M − 1)`, Melee Animosity (heavy), Secondary Outburst, Secondary Kinship, Arcane Hot Shot, Crimson and Topaz shards, Killer's Rush, sniper zoom crit chance (all snipers except Lanka), weapon traits such as AX-52 (headshot while aiming), heavy-attack doubling of True Steel family and melee riven (`07`, D-094), custom p.c |
| ΣCC_WP (weak-point hits only) | Primary/Pistol Acuity, Worthy Comradery, Cyte-09 passive, riven Weak Point Critical Chance (D-010), Incarnon "Critical Chance on Weakpoint Hits", custom p.f |
| ΣCC_abs | Arcane Avenger, Cat's Eye, Covenant (`cov_i`), Wrathful Advance (melee), Puncture `punc_cc(n_PUN)` (non-radial only), Secondary Enervate, Shadow Haze, Piercing Navigator (since U40, EC-02-16), Lanka zoom, buffing fields with flat crit chance (Mutalist Quanta and Simulor +0.25 per field, Mandonel +0.10), Ballistic Bullseye, Concentrated Arrow, Crystallize, Shroud of Dynar, Hunter Synergy, Laetum −0.10 (negative), T-18, custom p.e, inspiration hook `critFlatChancePerStatuses·n` |
| CC_min | Smeeta Charm crit buff 2.0 (floor, not an addition) |
| M_cc | custom p.d only |

Sources for the table: `W:Critical_Hit` source galleries; `F:addmul` §2, §5, §6, §10; `F:buffs` §1.8, §1.9, §1.12,
§8, §11; `F:melee` §4.1, §5, §7; `W:Lanka` line 43 (flat zoom bonus after mods); `W:Piercing_Navigator` §Notes and U40
patch note; `W:Secondary_Enervate` §Notes ("Critical chance is an absolute bonus").

Context rules (each instance computes its own CC):

- Weak-point terms (ΣCC_WP, Covenant ×4) apply only when `wp_i = 1`. Radial instances never trigger weak-point or
  headshot conditions (`W:Enemy_Body_Parts` §Notes, U32 `W:Area_of_Effect`), so `wp_i = 0` for them.
- `punc_cc` uses the Puncture stacks on the target just before the instance (CD-04) and is excluded for radial
  instances and abilities (`F:status` §4.2, `W:Critical_Hit` absolute list).
- Melee combo hooks use the combo multiplier M of the hit supplied by `07` (whether a heavy uses M before or after the
  consumption is `07`'s, `F:addmul` open 12). `(M − 1)` form since U26 (`F:addmul` §2).
- Heavy attacks add the doubled True Steel family / melee riven entries to ΣCC_rel (`07`, `F:melee` §4.1).
- The CC that conditions on CC (Prelude of Might, E-2.5) and the CC that drives "on crit" triggers are the final CC of
  the instance.

### 4.3 Crit multiplier (E-4)

```text
E-2.5  Crit multiplier of damage instance i
  x32   = f32( f32(CDM0) + f32(CDM0+) + f32(cond_i·CDM0c) )      f32 additions, in this order
  xc    = min(max(x32, 0), 32)
  code  = floor( f32( f32(xc × 127.96875) + 0.5 ) )               integer 0 ... 4095 (127.96875 = 4095/32, exact in f32)
  Q     = f32( code × 32 / 4095 )                                 the binary32 value nearest to the exact quotient
  CDM   = ( Q·(1 + ΣCD_rel) + ΣCD_abs,i )·M_cdf                   f64 from here on
  ΣCD_abs,i = ΣCD_abs,static + rad_i'·cold_cd(n_COL) + doughty
  cond_i = 1 if the conditional base addition's condition holds for instance i (Prelude of Might: CC of instance i
           below the perk threshold), else 0
```

Sources: `W:Critical_Hit` §Quantization ("This applies before mods, but after certain effects that are additive to the
base value ... clamped to the range from 0 to 32 ... The calculation uses 32-bit floating-point arithmetic"; Incarnon
"Critical Parallel" example `Code(1.6 + 0.4) = 256`), §Critical Damage (relative, absolute "applied after relative
bonuses"); `F:core` §9; `F:status` §4.5 (Cold absolute, before tiers, not AoE); `F:buffs` §1.12 and `W:Electric_Shield`
line 15 ("multiplicative to all other sources of critical damage"); `W:Buffing_Fields` table; `JS L8268-8329`
(inspiration form, D-014, D-206).

Rounding detail: the wiki gives `Code × 32/4095` in f32 without an operation order. Computing `code × (32/4095)` with a
pre-rounded f32 constant differs from the correctly rounded quotient by one ulp for some codes (1.5, 1.6, 1.9, 3.0,
3.4 in the test of section 9). The engine uses the correctly rounded quotient (Decision; the difference is
< 1.2·10⁻⁷ relative and is UNVERIFIED either way, OQ-222). In Rust: `((code as f32) * 32.0) / 4095.0`, the product being exact.

Bucket membership:

| Bucket | Members |
|---|---|
| CDM0 | attack data `CritMultiplier`; kitgun and zaw values are the assembled sums (`06`, `07`) and are quantized as a whole (Decision, EC-02-24) |
| CDM0+ | Incarnon "Increase Base Critical Damage Multiplier by +X" ("added as base critical damage and will be affected by mods", `W:Braton_Incarnon_Genesis` notes); inspiration `flat_critmult` |
| CDM0c | Braton Incarnon "Prelude of Might": "With Critical Chance Below 50 %: Increase Base Critical Damage Multiplier by +X" (X = 3.4 / 3.3 / 3.0 / 3.0 by variant); "Condition is affected by ... Covenant, Puncture status, Galvanized Scope and Primary Acuity" (`W:Braton_Incarnon_Genesis` lines 117-126; U34 fix "always applying instead of on its intended trigger"). Threshold conflict 40 % vs 50 %: OQ-210 |
| ΣCD_rel | Vital Sense family, riven Critical Damage, Organ Shatter, Primed Target Cracker, Hammer Shot, Bladed Rounds, Sharpened Bullets, Shrapnel Shot, Gladiator Might, Galvanized Steel, Secondary Outburst, Primary Frostbite, Primary Blight, Crimson and Violet shards, Grave Spirit, Dreamer's Wrath (Tennokai heavy), Tek Collateral, Strain Infection, Spiteful Defilement +1.0 when fewer than 3 statuses (D-067), sniper zoom crit damage (Rubico etc.), Mutalist Quanta and Simulor fields +0.25 ("additive"), custom p.g |
| ΣCD_abs | Arcane Crepuscular +3.0, Tenacious Bond +1.2, Shroud of Dynar, Cold `cold_cd(n_COL)` (non-radial, incl. Frozen +1.0), Melee Doughty (`doughty`, bucket UNVERIFIED, OQ-209), Knell "Death Knell" +0.5 per stack (written "0.5x" like the absolute sources; bucket UNVERIFIED), custom p.h |
| M_cdf | Electric Shield ×2, Energy Shell ×2, Assimilate ×2, Mandonel field ×1.5 per field, custom p.i as `(1 + x)`; one buffing-field source per projectile, melee swings excluded (`04`); placement for tiers ≥ 2 UNVERIFIED (OQ-016) |

Melee Doughty (`W:Melee_Doughty` §Notes; U38, U38.6 fix "applying the crit damage bonus to the portion of the
weapon that does not have any Puncture damage"):

```text
E-2.5a  doughty = min(50, round1( (r_PUN / Σ_t r_t) × (100·SC_w) × 0.1 × b_rank ))      b_rank = 1.0 at max rank
        round1(x) = floor(10·x + 0.5) / 10 ;  r_t: modded unquantized composition (01) ;  SC_w: E-2.11
```

The page's example (Destreza Prime, Auger Strike, Drifting Contact, Weeping Wounds at 12×) gives +9.7 (TV-02-14). The
share uses unquantized modded damage per the example's numbers (`119 × 2.2 / 312.8`); quantized shares are
UNVERIFIED alternatives (OQ-209).

### 4.4 Crit tier and crit factor (E-6)

```text
E-2.6  Tier of damage instance i (Monte Carlo)
  n = floor(CC_r), f = frac(CC_r)
  k0 = n + Bernoulli(f)                                      mechanic id "crit_roll"
  k1 = k0 + Bernoulli(p_vig)   if k0 ≥ 1 and p_vig > 0       mechanic id "crit_vig"
  k1 = k0                      otherwise
  k  = k1 + π_p                                              Critical Promotion part (π_p = 1): +1 tier, a non-crit
                                                             becomes tier 1
```

- `floor + Bernoulli(frac)` is the sourced rule (U13.5 patch note on `W:Critical_Hit`: "a guaranteed critical damage
  value ... a secondary red crit chance based on the remaining percent"; tier table of `W:Critical_Hit` §Critical
  Tiers). Tiers are unbounded (red "!", "!!", "!!!" since U34 are display only; display overflow past ~255 is a display
  bug, `W:Critical_Hit` §Bugs: ignored).
- Vigilante: `p_vig = 0.05 × (Vigilante set pieces counted per 09)`, max 0.30, only for attacks of **primary weapons**,
  only on crits ("if the weapon deals a yellow critical, there is a 30 % chance that it will deal an orange critical",
  `W:Vigilante_Set` §Set Bonus, §Notes; introduced U22, fixed "not applying" in U41). Turning a non-crit into a crit is
  excluded by the page wording (`F:mods` open 9).
- Critical Promotion: "Some weak points, like Rogue Culverin's arm canisters and Scaldra backpacks, will increase the
  critical tier of incoming hits. For example, a non-crit would become a tier 1 yellow crit" (`W:Enemy_Body_Parts`
  §Critical Promotion; Techrot parts marked ¹; `Multis` strings such as "Head: +1 Crit Tier", `DS:wiki`; `05` field
  `tier_add`). Radial instances use the body part's values (`05` §4.3), whose `tier_add` is 0 in all known data.
  Order Vigilante → promotion is a Decision (UNVERIFIED, OQ-205): Vigilante reads the rolled tier k0.
- Every "on critical hit" rule downstream (Hunter Munitions, Arcane Fury and other on-crit triggers, Melee Duplicate
  "on base critical hits" = k = 1, Secondary Enervate "big critical hit" = k ≥ 2) reads the final k.

```text
E-2.7  Tier distribution and moments (EV)
  CC_r < 1 (n = 0):  P(0) = 1 − CC_r ; P(1) = CC_r·(1 − p_vig) ; P(2) = CC_r·p_vig
  CC_r ≥ 1:          P(n) = (1 − f)(1 − p_vig) ; P(n + 1) = (1 − f)·p_vig + f·(1 − p_vig) ; P(n + 2) = f·p_vig
  promotion shifts every tier by π_p
  E[k]       = CC_r + p_vig·min(1, CC_r) + π_p                      (00 E-6 with clamp and promotion)
  P(k ≥ 1)   = 1 if π_p ≥ 1, else min(1, CC_r)
  P(k ≥ 2)   = from the distribution above (Secondary Enervate reset, 04)
```

Crit factor of the instance (E-12 notation):

```text
E-2.8  c_k = 1 + k·(h·CDM − 1)                         (no stealth)
       c_k = 1 + SDB + k·(h·CDM − 1)                   (melee stealth hit, SDB and T-08 owned by 04)
       h   = `headcrit` of the part hit (05 §4.3 data: 2 on head-crit heads, 1 by default, other values per enemy);
             radial instances use the body part's h; 1×-headshot weapons use h = 1 on weak-point parts;
             a Sonar spot never changes h
  EV:  E[c]   = c_0 + E[k]·(h·CDM − 1)                   with c_0 = 1 (or 1 + SDB)
       C1     = E[c·1{k ≥ 1}] = E[c] − P(k = 0)·c_0      (crit-conditional mass, for Hunter Munitions, E-2.16)
```

- Head-crit: "Applies a 2x multiplier to the weapon's base critical multiplier"; formula `Headshot Multi × (1 + Critical
  Tier × (2 × Total Critical Damage Multiplier − 1))` (`W:Critical_Hit` §Critical Headshots, `W:Enemy_Body_Parts`
  §Critical Headshots). h is 2 on normal head-crit parts; other factors exist as part data: Juggernaut maw and belly 4
  ("Bodyshot, but applies critical headshot bonus"), Ropalolyst head 0.2, Eidolon, Lephantis and Hemocyte parts 2
  (`F:enemy` §9 table; `05` §4.3), `Multis` "1.5x Critical Damage". The overview defines h ∈ {1, 2}; the
  generalisation to the data value is proposed in OQ-203. Whether h also doubles ΣCD_abs and M_cdf: Decision "doubles the total CDM",
  UNVERIFIED (OQ-016, `F:core` open 5).
- E[c] is exact for any CC because c_k is linear in k (`W:Critical_Hit` §Average Damage: `1 + CC × (CDM − 1)`;
  `F:core` §9). Check: CC 1.2, CDM 4.401074743 → 0.8 × 4.401074743 + 0.2 × 7.802149487 = 5.081289692 (TV-02-02).
- DoT and extra hits of the instance use its rolled k (MC) and E[c] (EV) except where a proc exists only on crits
  (Hunter Munitions): those use C1 (E-2.16).

```text
E-2.9  Target-state averaging (EV, CD-13)
  CC_i depends on n_PUN (punc_cc), CDM_i on n_COL (cold_cd) and, through CDM0c, on CC_i.
  E[c] = Σ_{nP} P(n_PUN = nP) · ( c_0 + E[k | CC_i(nP)] · ( h·E_{nC}[ CDM_i(nP, nC) ] − 1 ) )
  E_{nC}[CDM] = ( Q(nP)·(1 + ΣCD_rel) + ΣCD_abs,static + doughty + E[cold_cd(n_COL)] )·M_cdf      (linear in cold_cd)
  P(k ≥ 1) = Σ_{nP} P(nP)·min(1, CC_r(nP))   (π_p = 0)
```

E[k | CC] is piecewise linear (the `min(1, CC_r)` of Vigilante, the CC_min floor, the 0 clamp, the Prelude threshold),
so E[k] MUST be averaged over the stack distribution, never evaluated at E[n] (TV-02-13 shows a 0.6 % error at
μ_PUN = 2). n_PUN and n_COL are independent under the Poisson model of 00 §4.2.

### 4.5 Body part (E-7)

Hit-location model: T-03 selects for every instance that can hit a part (direct hits; not radial; extra hits per E-14)
the target's body part, head part or first non-head weak point, as designated by `05` §4.3 (`head` = first part with
`is_head`; `weak_point` = first part with `is_weak_point` and not `is_head`, falling back to `body` if none). Part
record fields used here (`05` §4.3): `part_mult` (P_p), `is_weak_point` (wp_p), `is_head` (hs_p), `headcrit` (h),
`tier_add` (π_p), `status_ok`. A Sonar buff (`04` catalogue: multiplier `s_son = 2 / 3 / 4 / 5 × Str` by rank, number
of highlighted nodes `ν` on the part) marks whether the hit part carries Sonar.

```text
E-2.10  Part factor of damage instance i
  radial instance:                Part = P_body·M_son,body       (05 §4.3: radial uses the body part's P and h;
                                                                  M_son,body = s_son^ν if the scenario puts a Sonar spot
                                                                  on the torso, else 1: UNVERIFIED, W:Sonar tip, OQ-224)
  direct hit on part p:
    wp   = wp_p or (Sonar on p)                                   Sonar spots are weak points
    hs   = hs_p                                                   Sonar does not change the headshot flag
    M_son = s_son^ν if Sonar on p, else 1                         nodes on one part multiply
    A    = 1                                                      1×-headshot weapon and wp
         = P_p + (hs ? 1.5 : 1.0)·ΣWP                             wp
         = P_p                                                    otherwise
    B    = (1 + ΣHS)·M_hsu                                        hs
         = 1 − ΣBP                                                not hs
    Part = A·B·M_son
```

Sources: `F:core` §10; `W:Enemy_Body_Parts` §Body Part Types, §Weak Point Hits (`3 + 1.5 × (3.5 + 0.75) = 9.375`,
non-head `1 + 3.5 + 0.75 = 5.25`), §Headshots (`× (1 + 0.3 + 0.4) = 15.9375`), §Critical Headshots, §Notes (1×-headshot
list, Sonar formulas `40 × (1.0 + 3.5 + 0.75) × 5.0 = 1,050` and `40 × (3.0 + 1.5 × (3.5 + 0.75)) × 5.0 = 1,875`, radial
1×, "Weakpoint damage multiplier and headshot multiplier effects are ... multiplicative with Sonar's damage mult, the
Sniper Combo multiplier, and the unique Cernos Prime 50 % damage bonus"), §Patch History (U32 3× heads and radial rule;
U44 Weak Point Standardization and non-headshot weak points); `W:Primary_Acuity` §Notes (1.5× vs 1.0×, Deadhead
multiplicative `1.3 × (3 + 1.5 × 3.5) = 10.725`); `W:Primary_Deadhead` lines 31-41; `W:Sonar` info (U44 rework, revisions
2026-09-24 to 2026-10-29: spots are weak points, keep the part's original multiplier and headshot criteria, no
head-crit on Sonar spots, nodes multiply, torso spots raise AoE damage); `W:Meticulous_Aim` §Notes (bodyshot penalty
"multiplicative to weapon overall damage", "anything but a headshot, including ... MOA backpacks").

Bucket membership:

| Symbol | Members |
|---|---|
| P_p | `05` part data: heads usually 3.0 (since U32; Murmur and Techrot about 1.5; some heads 1.0 or 1.875), non-head weak points (MOA and Bursa rears, Severed Warden arms, Lumbering Fragment, Scaldra Harbinger balloon, Amalgam Heqet backpack), bodies 1 or below (armor plates 0.5, Jordas Golem body 0) |
| ΣWP | Primary/Pistol Acuity +3.5, Seek +0.75, Cascadia Accuracy, riven Weak Point Damage (D-011; spliced trait functional since Hotfix 44.1.1), custom p.v (D-012) |
| ΣHS (headshot parts only) | Primary/Secondary Deadhead +0.3, Target Acquired +0.6, Amalgam Daikyu Target Acquired +0.75, Meticulous Aim +1.05, Prowl, sniper zoom headshot bonuses (Snipetron, Sporothrix, Vectis, Vectis Prime, Vulkar), Synapse and Coda Synapse +0.2 and Kuva Chakkhurr +0.5 (data `ExtraHeadshotDmg`), Laetum +0.5, Latron Incarnon +1.0, Phenmor +0.5, Epitaph with Sevagoth +0.2, custom p.t |
| M_hsu | Cernos Prime 1.5 (data `ExtraHeadshotDmg = 0.5`, hand-tagged as unique: "Unique +50 % Damage Multiplier bonus on headshots, multiplicative with other Headshot Multiplier bonuses", `W:Enemy_Body_Parts` weapon table; `W:Primary_Deadhead` line 34) |
| ΣBP | Meticulous Aim 0.45 |
| M_son | Sonar (`04` value), `s_son^ν` |

1×-headshot weapons (flag per attack): data `ExtraHeadshotDmg = −2` (Sirocco, Catchmoon secondary, Afentis Prime,
Alternox, Amprex, Fulmin and Fulmin Prime semi-auto, Nataruk perfect shot, Telos Boltor; `Module:Weapons/data`) plus the
hand list of `W:Enemy_Body_Parts` §Notes (Arca Plasmor and Tenet, Alternox primary, Argo & Vel heavy glaive, Athodai
alt-fire, Catchmoon both, Corufell shots, Cyanex primary, Dread Incarnon, Fulmin semi, Ignis and Wraith, Lex Incarnon,
Lizzie primary and waves, Nataruk charged and perfect, Nepheri fireballs, Paris Incarnon, Steflos, Stropha shots, Tenet
Agendus disks; Electricity and Gas procs and Resupply, Silken Stride, Demonium, Sangodae, Gastroparesis extra hits are
handled by `03` and `04`). For them: ΣWP ignored and P replaced by 1 on weak points; ΣHS still applies ("still affected
by Deadhead's unique multiplier"); h = 1 ("If a weapon defaults to a 1x headshot multiplier it will not benefit from
the critical bonus, this is true even if the headshot multiplier is raised", `W:Critical_Hit`). Applying the rule to
non-head weak points is a Decision (UNVERIFIED, OQ-211).

E-2.10 widens the body branch of 00 E-7 (`Part = 1`) to `P_body·(1 − ΣBP)·M_son` (OQ-223); with P_body = 1 and no
Meticulous Aim or Sonar both agree.

Inheritance of Part by DoT (E-13 `Part_dot = Part / M_son`) and extra hits (E-14) is defined by `03` and `04`. The
post-U44 Sonar page contradicts E-13 for Sonar (OQ-202).

### 4.6 Status chance (E-8)

```text
E-2.11  Status chance of damage instance i (per pellet)
  SC_w   = ( (SC0 + SC0+)·(1 + ΣSC_rel) + ΣSC_abs )·M_sc,w            weapon side
  M_sc,t = (1 + 0.1·n_TAU)·Π_v (1 + v)                                target side: Tau stacks before the instance;
                                                                      v: custom p.k, Pageflight (04)
  SC     = SC_w·M_sc,t ;   M_sc = M_sc,w·M_sc,t (00 E-8)
  SC_r   = max(0, SC) ;    beams: SC_tick = m·SC_r
  attacks flagged sc_locked (inspiration `noIncrStatus`): SC = SC0 (hook, no game source)
```

Sources: `F:core` §13; `F:status` §2.1, §4.15; `W:Status_Effect` §Status Chance, §Multishot ("the status chance on a
listed weapon in the Arsenal is the probability that each pellet will individually proc"), §Average Procs, §Trivia
(per-pellet since U27.2; before U27.2 `1 − (1 − SC)^(1/pellets)`); `W:Damage/Tau_Damage` lines 13-17 ("10 % per stack
for up to 10 stacks ... multiplicative to the status chance of the source", 210 % → 315 % at 5 stacks, 420 % at 10;
"Forced Procs ... will not create additional procs due to this vulnerability"; U37); `JS L9888-9977`.

Bucket membership:

| Bucket | Members |
|---|---|
| SC0 | attack data `StatusChance` per pellet; Kohm family: the data's per-pellet value of the spool state (below) |
| SC0+ | Incarnon "Increase Base Status Chance by +X %" per projectile (Furis "Elemental Balance", Strun and Boar perks with separate Incarnon-form values; `W:Boar_Incarnon_Genesis`: "Now it applies to each projectile instead of being divided by the Multishot count"); cross-stat Incarnon perk "Increase Base Status Chance by +x % of current Critical Chance" (hook, OQ-220) |
| ΣSC_rel | status chance mods incl. dual-stat elemental mods, Galvanized Aptitude/Savvy/Shot +0.8, Galvanized Elementalist, riven Status Chance, Weeping Wounds `0.4·(M − 1)`, Empowered Blades (heavy only), Condition's Perfection (Tennokai), Crimson shard (primary), Prismatic Gem, Vazarin Initiative, Primary Crux, Catalyzer Link family, Knell Death Knell (bucket UNVERIFIED), weapon trait `ifMaxStatSlash` (D-207), custom p.j |
| ΣSC_abs | Entropy Burst (`W:Calculating_Bonuses` overview), Phenmor perks (`absolute_status_chance` in the inspiration's sample data, UNVERIFIED), custom p.l |
| M_sc,w | Mandonel field ×2 ("+100 % Status Chance (multiplicative)", `W:Buffing_Fields`) |
| M_sc,t | Tau `1 + 0.1·n_TAU` (cap from `03`), custom p.k `(1 + x)`, Dante Pageflight status chance vulnerability (combination UNVERIFIED, OQ-207) |

Rules:

- Per pellet: SC is the chance of each pellet; shotguns are not special since U27.2.
- Tau uses the Tau stacks on the target before the instance (CD-04); the instance's own Tau proc does not raise its own
  SC. Whether sibling pellets of one shot see each other's Tau procs follows CD-04 (no; UNVERIFIED like OQ-028).
- Kohm family: "The listed status chance is for the first shot only, and is divided between the pellets the longer it
  is fired" (`W:Kohm` line 30; `F:weapon` §6). Data: Kohm "Single Pellet" SC 0.75, "Fully Spooled" `Multishot = 12`,
  SC 0.0625 = 0.75/12 (`Module:Weapons/data/primary`). Intermediate spool states (`06`): `SC0 = SC0,first / n_spool`.
  Multishot mods multiply the pellets without a further division (Decision, UNVERIFIED, OQ-208).
- Merged beams sum SC (E-2.3): a tick can exceed 100 % although one beam does not.
- Negative SC (kitgun loader offsets such as −0.08) cannot roll: SC_r = 0.

### 4.7 Procs of an instance (E-9)

#### 4.7.1 Rolled procs

```text
E-2.12  Count (MC) and generating function (EV)
  n = floor(SC_r), f = frac(SC_r)          (beams: SC_tick in place of SC_r)
  N = n + Bernoulli(f)                     mechanic id "sc_count"
  each of the N procs draws its type independently from w (mechanic id "sc_type"); duplicates allowed
  G(z) = E[z^N] = (1 − f)·z^n + f·z^(n+1)
  beams: G_beam(z) = Σ_m P(m)·G_{m·SC_r}(z)              (m distribution of E-2.3)
  E[N] = SC_r   (beams: E[m]·SC_r)
```

The expected count `SC` per pellet is sourced (`W:Status_Effect` §Average Procs: "Multishot × (Number of Forced Procs +
Status Chance per Projectile)"); the sampling rule `floor + Bernoulli(frac)` is not (UNVERIFIED, OQ-003). Each proc's
type is "independently drawn, so it is possible to apply the same status several times in one hit" (`W:Status_Effect`
§Status Chance).

```text
E-2.13  Type weights
  q⁺_t = max(0, q_t) for damage types t that have a status (IMP ... TAU of 00 §2.2); TRU, FIN, CIN excluded
  w_s  = q⁺_s·[s ∉ Imm] / Σ_u q⁺_u·[u ∉ Imm]          (s = t)
  if the denominator is 0 (every type immune, or "all" immune): no rolled proc is delivered
  ragdolled target: a draw of IMP delivers nothing; weights are NOT renormalized; the draw still counts as an
  Impact roll for Internal Bleeding and Hemorrhage (E-2.15)
  part with status_ok = false (05: Juggernaut parts other than maw and belly): the instance delivers no proc at
  all (rolled, forced, triggered); it still rolls crit
```

Sources: `W:Status_Effect` §Damage Distribution (`Proc Type Chance = Damage / Total Damage`, 20/5/10/25/50 example),
§Status Immunity Interactions ("modified by enemy status immunities ... excluded from proc type chance calculations";
"not altered by enemy resistances or weaknesses"), §Trivia (physical 4× weight before U27.2, obsolete);
`W:Damage/Impact_Damage` lines 25-26 (ragdoll: "Any status proc that rolls the Impact damage type will instead inflict
no status. Hemorrhage and Internal Bleeding can still trigger"); `F:status` §2.2. Weights use the quantized composition
q_t (Decision D-024, UNVERIFIED, OQ-004); combined elements have replaced their components in q_t (`01`), so component
statuses cannot roll. Excluding non-status types from the denominator is a Decision (UNVERIFIED, OQ-206). Full status
immunity (Eidolons, Ropalolyst, Tusk Thumpers, Profit-Taker Orb, Exploiter Orb ..., `W:Status_Effect` §Status Immune
Enemies) is `Imm = all` in `05` data.

#### 4.7.2 Forced procs

A forced proc is "guaranteed to occur regardless of the status chance and damage distribution", "independent from
normally occurring procs which can also take place alongside", "not the same as having 100 % status chance"
(`W:Status_Effect` §Forced Procs). Per pellet (`W:Status_Effect` §Average Procs), per merged beam tick (E-2.3). Tau adds
none. A forced proc of an immune type is dropped. φ_s counts the forced procs of s on the instance:

```text
E-2.14  φ_s = Σ_sources (number of listed procs of s on the instance)·[s ∉ Imm]      (no SC, no Tau, no weights)
        beams: φ_s per merged tick, not × m
```

Sources of φ:

| Source | Rule | Source of the rule |
|---|---|---|
| Weapon data `ForcedProcs` (716 attacks) | each listed proc once per instance | `DS:wiki` (Module:Weapons/data); `F:weapon` §10.1 |
| Conditional weapon traits (Catchmoon within 10 m, Arca Plasmor within 14 m, Fulmin within 9 m, "direct hit", "fully charged") | apply only when the condition holds; distance via T-27 (`06`); the data does not encode the condition, so a hand-authored condition table is needed | `W:Internal_Bleeding` tips, `W:Hemorrhage` tips, `W:Magnetic_Welt` tips |
| Evensong | 7 Puncture per direct hit and per explosion, "bypassing the normal limit of 5"; 5 on enemies capped at 4 (cap rule `03`) | `W:Evensong` lines 17, 42-44 |
| Cedo alt-fire explosion | 2 forced procs that SC cannot increase: first drawn with the normal weights over the modded elements and innate Blast, second uniform over HEA, COL, ELE, TOX (mechanic id "cedo_type") | `W:Cedo` lines 42-46 (D-202) |
| Nukor, Kuva Nukor | Microwave (MIC) on every hit (application rule UNVERIFIED, no data field) | `W:Microwave` (OQ-215) |
| Stance hits (07 data `Procs`) | each listed name once per hit | `07`; `F:melee` §2.1 |
| Buffs (04): Conductive Sphere +1 ELE per field; Toxic Lash forced TOX on its extra hit (E-14) | as catalogued | `F:buffs` §1.12, §1.4 |

Proc-name mapping (weapon and stance data): Impact, Knockback → IMP; Puncture, Weakened → PUN; Slash, Bleed → SLA;
Heat → HEA; Cold → COL; Electricity, TeslaChain → ELE; Toxin → TOX; Blast, Detonate → BLA; Radiation → RAD; Gas → GAS;
Magnetic → MAG; Viral → VIR; Corrosive → COR; Void → VOI; Tau → TAU; Microwave → MIC; Knockdown → KND; Lifted → LIF.
Ragdoll, Stagger, Big Stagger (BigStagger), Blind, Stun, Impair, Disarmed and empty strings are crowd-control-only:
no StatusType, ignored by the damage engine except that Ragdoll sets the ragdoll state (OQ-219) and Stun's CO counting
is `03`'s (OQ-216). Sources: `W:Status_Effect` §General and §Independent from Damage; `Module:Stances/data` and
`Module:Weapons/data` proc-name counts (this session).

#### 4.7.3 Impact-triggered procs: Internal Bleeding, Hemorrhage, Magnetic Welt

```text
E-2.15  Per instance (mechanic ids "ib", "welt")
  I      = the instance carries ≥ 1 Impact status: a rolled IMP draw (also when the ragdoll rule discards it) or a
           non-immune forced IMP
  p_IB   = b_rank·(FR < 2.5 ? 2 : 1)            b_rank = 0.35 at max rank; FR = modded fire rate of the attack (06),
                                                strictly below 2.5; bows use their hidden fire rate
  IB / Hemorrhage:  x_IB = Bernoulli(p_IB)  if I and not S_r       → one SLA (shared with Hunter Munitions, E-2.16)
  Magnetic Welt:    x_W  = Bernoulli(p_W)   if I and not M_r       → one MAG   (p_W like p_IB)
  S_r, M_r = the instance already carries a rolled or forced (weapon trait, stance) SLA / MAG proc
  at most one trigger per instance, regardless of the number of Impact procs
```

Sources: `F:addmul` §8; `W:Internal_Bleeding` (rifle, U29.10) and `W:Hemorrhage` (pistol, U29.10) §Notes: "chance is
doubled when the weapon's fire rate is below 2.5", "If the weapon's fire rate is exactly 2.5 it won't count", "Proccing
impact more than once ... will not allow this mod to proc more than once", "Fire Rate buffs ... can negatively impact
this mod", bow hidden fire rates (1.0; Cinta 1.5; Kuva Bramma and Nataruk 0.667), "Cannot produce multiple procs in a
single instance of damage alongside any other Slash sources, such as a weapon's innate Slash, Hunter Munitions, or the
debuff from Seeking Talons"; `W:Magnetic_Welt` (shotgun, U38), same notes for Magnetic; `W:Damage/Impact_Damage` line 26;
`JS L9853-9878`. The suppression by an already rolled SLA / MAG (`not S_r`) follows the three mod pages and contradicts
nothing on them, but conflicts in spirit with the Hunter Munitions page (E-2.16): UNVERIFIED, OQ-204. Which fire rate
counts for charge and spool weapons: FR, not EFR (Decision, OQ-214).

#### 4.7.4 Hunter Munitions and the forced-Slash group

```text
E-2.16  Per instance (mechanic ids "hm", "st")
  x_HM = Bernoulli(p_HM) if k ≥ 1                 p_HM = 0.30 at max rank; primary weapons (incl. Artemis Bow since U33)
  forced SLA added if x_HM or x_IB                (at most one Slash from the pair; independent draws)
  Seeking Talons (target marked, 04):  x_ST = Bernoulli(p_ST) if no SLA on the instance from any source above
  Hunter Munitions is NOT suppressed by a rolled SLA (it stacks with a status-chance Slash)
```

Sources: `W:Hunter_Munitions` (U22.3) §Notes: "30 % trigger chance is independent ... not affected by the weapon's
Status Chance, or damage type distribution", "Cannot produce multiple procs in a single instance of damage alongside
forced Slash from sources such as Internal Bleeding or ... Seeking Talons, but can stack with Slash statuses applied
using a weapon's innate status chance", "Procs of Hunter Munitions and Internal Bleeding are drawn independently, and
if both proc at the same time, only 1 slash proc is applied", worked probabilities 54.5 % (fast) and 79 % (slow), "On
direct hit, AoE weapons can potentially trigger Hunter Munitions twice" (direct and radial instances each roll);
`W:Seeking_Talons` info ("will not receive a forced slash proc from the ability on the same attack that naturally
triggered a slash proc through status chance, or from other sources of forced Slash statuses such as Hunter Munitions
and Internal Bleeding"); `F:addmul` §8; `W:Multishot` §Continuous Weapons (once per beam tick). One roll per crit
instance regardless of tier: the page says "upon a Critical Hit" and gives no tier dependence (UNVERIFIED, OQ-018).

EV forms (independence of crit and status, CD-03):

```text
  P_c     = P(k ≥ 1)                                        (E-2.7)
  π_IB    = P(I ∧ ¬S_r)  = G(1 − w_SLA) − G(1 − w_IMP − w_SLA)     no forced IMP, no forced SLA
          = G(1 − w_SLA)                                            ≥ 1 forced IMP, no forced SLA
          = 0                                                       ≥ 1 forced SLA
  E[X_SLA]   = 1 − (1 − p_HM·P_c)·(1 − p_IB·π_IB)                    expected forced Slash from HM/IB
  E[c·X_SLA] = p_IB·π_IB·E[c] + p_HM·C1·(1 − p_IB·π_IB)               crit-weighted (for the Slash DoT seed)
  E[x_ST]    = p_ST·(1 − p_HM·P_c)·(G(1 − w_SLA) − p_IB·π_IB)         (no forced SLA)
  E[c·x_ST]  = p_ST·(G(1 − w_SLA) − p_IB·π_IB)·(E[c] − p_HM·C1)
  Magnetic Welt: E[x_W] = p_W·π_W,  π_W = G(1 − w_MAG) − G(1 − w_IMP − w_MAG)  (variants as for π_IB)
  ragdoll: unchanged (an IMP draw still counts as I)
  beams: evaluate every line above conditionally on the merged count m (SC_tick = m·SC_r, G = G_{m·SC_r}), then
         average: proc counts E[X] = Σ_m P(m)·X(m); DoT masses (seed × m) E[m·X] = Σ_m P(m)·m·X(m)
```

The wiki's worked check (crit and Impact certain): fast `1 − 0.7 × 0.65 = 0.545`, slow `1 − 0.7 × 0.3 = 0.79`
(TV-02-10).

#### 4.7.5 Secondary Encumber and other proc-triggered procs

```text
E-2.17  Secondary Encumber (per shot; mechanic ids "enc", "enc_type")
  a      = c_enc × π_any                c_enc = 0.24 at max rank (04); π_any = P(pellet delivers ≥ 1 status)
  π_any  = 1 if the pellet has ≥ 1 non-immune forced proc, else 1 − G(δ)
           δ = w_IMP if the target is ragdolled, else 0 (G(δ) = P(every rolled proc is discarded); 0 if all types immune)
  pellets of the shot are tried in order; the first success adds one proc uniform over
  {IMP, PUN, SLA, HEA, COL, ELE, TOX, BLA, COR, MAG, VIR, GAS, RAD} (dropped if immune); at most one per shot
  P_enc  = 1 − E[(1 − a)^(n_pel)] = 1 − ( (1 − f)·(1 − a)^n + f·(1 − a)^(n + 1) )     n, f of MS (E-2.2)
  beams: one try per tick with π_any of the merged tick
```

Source: `W:Secondary_Encumber` §Notes ("only apply a max of 1 additional status effect per instant of time ... more
multishot does allow more chances of an Encumber proc until one succeeds", average per shot `1 − (1 − 24 % ×
min(statusChance, 1))^pelletCount`, "Forced procs ... can trigger Encumber", "Random proc is uniformly chosen from the
following and is independent from the weapon's damage distribution" with the 13-entry list, exalted weapons since
U38.5). This settles the uniform choice of D-021 (no longer UNVERIFIED). The Encumber proc is appended after E-2.15 and
E-2.16 and triggers nothing in them (Decision, UNVERIFIED, OQ-218).

Other proc-triggered procs (catalogue and values `04`, effects `03`; this document only fixes their trigger
probabilities, which use `A_s = P(instance carries ≥ 1 proc of s)` of E-2.18):

- Secondary Cryogenic: on a Puncture status, Cold stacks (3 at max rank) on the target and enemies in 15 m
  (`W:Secondary_Cryogenic`). Decision: one trigger per instance with ≥ 1 PUN (UNVERIFIED).
- Primary Debilitate: when a combined-element status is at ≥ 10 stacks, a chance (100 % at R5) for an extra status,
  "only activate once per damage instance" (`W:Primary_Debilitate`). Faction three times on its DoT per that page;
  D-073 decided "twice" (OQ-217).
- Archon Continuity and Archon Vitality act on **ability** procs and exalted weapons only (`W:Archon_Continuity`,
  `W:Archon_Vitality`); they never add procs to ordinary weapon instances (D-204).

#### 4.7.6 Expected procs per instance (EV)

```text
E-2.18
  A_s (rolled only)       = 1 − G(1 − w_s)                     (ragdoll: A_IMP delivered = 0)
  E[N_s] (rolled)         = SC_r·w_s ·[delivered]              beams: E[m]·SC_r·w_s
  E[N_s] (all sources)    = rolled + φ_s + [s = SLA]·(E[X_SLA] + E[x_ST]) + [s = MAG]·E[x_W]
                            + P_enc/13·[s ∈ ENC13, s ∉ Imm] (attributed once per shot) + proc-triggered arcanes (04)
  A_s (all sources)       = 1 if φ_s ≥ 1; otherwise 1 − P(no source fires), computed from the same independent pieces
  per second: λ_s = Σ over instances per second (08)
```

`A_s` per instance feeds `03` and `08` (distinct-status presence, CO count for later pellets, Secondary Cryogenic,
Encumber).

### 4.8 Order of operations for one shot

1. Shot from the schedule (`06`, `07`). Draw `n_pel` (E-2.2) or `m` (E-2.3).
2. For each pellet j = 0 ... n_pel − 1 in order (CD-04: each pellet reads the target state as it was before the
   shot for debuffs, and the CO count including earlier pellets, `04`):
   1. Hit context: part (T-03, `05`), radial flag, combo M and heavy flag (`07`), active conditions (T-02).
   2. CC (E-2.4) → CC_r.
   3. CDM (E-2.5): conditional base addition from CC, quantization in f32, then mods, Cold, Doughty, field.
   4. Tier k (E-2.6): crit roll, Vigilante, promotion.
   5. Part and h (E-2.10, E-2.8); crit factor c_k.
   6. SC (E-2.11) with Tau stacks before the shot.
   7. Rolled procs: N, then N type draws; immunity and ragdoll filtering (E-2.12, E-2.13).
   8. Forced procs (weapon traits, stance, buffs) (E-2.14).
   9. Impact-triggered procs (E-2.15), then Hunter Munitions and Seeking Talons (E-2.16).
   10. Encumber, once per shot (E-2.17); other proc-triggered arcanes (`04`).
   11. Hand over (k, c_k, h, Part, proc list) to E-12/E-13/E-14; damage is applied before the procs (CD-04, `03`).
3. Beams: steps 2.1-2.11 once per tick with the merged instance.

The draws listed with mechanic ids are the only random draws of this document; `08` fixes the keying `(seed, run,
attack, event index, mechanic id)` (U-6). Draw order within one instance: "crit_roll", "crit_vig", "sc_count",
"sc_type" (one per proc), "cedo_type", "ib", "welt", "hm", "st", then per shot "enc", "enc_type".

### 4.9 Summary of what the EV evaluator uses

| Quantity | EV form | Where |
|---|---|---|
| pellets per shot | `MS`; `E[n_pel²] = MS² + f(1 − f)` | E-2.2 |
| beam merged count | `E[m] = MS`, `E[m²] = MS² + f(1 − f)` | E-2.3 |
| crit factor | `E[c] = c_0 + E[k]·(h·CDM − 1)`, averaged over n_PUN and n_COL | E-2.8, E-2.9 |
| crit-only proc mass | `C1 = E[c] − P(k = 0)·c_0` | E-2.8 |
| status chance | `E[SC] = SC_w·E[M_sc,t]` (linear in n_TAU) | E-2.11 |
| procs of s per instance | `SC_r·w_s + φ_s + ...` | E-2.18 |
| any proc of s per instance | `1 − G(1 − w_s)` and E-2.18 | E-2.12 |
| DoT mass of rolled procs | non-beam `E[c]·SC_r·w_s`; beam `E[c]·SC·w_s·E[m²]` | E-2.3, CD-10 |
| DoT mass of forced procs | non-beam `E[c]·φ_s`; beam `E[c]·φ_s·E[m]` | E-2.14 |
| DoT mass of HM/IB/ST Slash | E-2.16 crit-weighted forms (× E[m] on beams) | E-2.16 |
| Encumber per shot | `P_enc` | E-2.17 |

Nonlinear functions of a target-state variable (G of an SC that depends on n_TAU, E[k] of a CC that depends on n_PUN)
are averaged over that variable's distribution (CD-13). The steady-state Tau stack mean depends on the Tau proc rate,
which depends on E[SC]: `08` solves this fixed point by iteration (monotone in μ_TAU; Decision).

---

## 5. Worked example

Arithmetic checked with `python3` (numpy float32 for E-2.5). Weapon as in 00 §3.8: Braton Prime "Normal Attack"
(CC0 0.12, CDM0 2.0, SC0 0.26, MS0 1, composition q = IMP 2/32, PUN 11/32, SLA 19/32, COR 58/32), Point Strike
(ΣCC_rel 1.5), Vital Sense (ΣCD_rel 1.2), Split Chamber (ΣMS 0.9), Primary Acuity (ΣWP 3.5, ΣCC_WP 3.5, ms_lock), target
Grineer Lancer (head P = 3, head-crit h = 2), no Puncture, Cold or Tau stacks, T-03 `head` vs `body`.

| Step | Body hit | Head hit |
|---|---|---|
| MS (E-2.1) | Acuity lock: MS = MS0 = 1 (Split Chamber ignored) | same |
| CC (E-2.4) | 0.12 × (1 + 1.5) = 0.30 | 0.12 × (1 + 1.5 + 3.5) = 0.72 |
| CDM (E-2.5) | code 256, Q = 2.000488519668579, CDM = 4.401074743270875 | same |
| h | 1 | 2 |
| Part (E-2.10) | 1 | 3 + 1.5 × 3.5 = 8.25 |
| E[c] (E-2.8) | 1 + 0.30 × 3.401074743 = 2.020322423 | 1 + 0.72 × 7.802149487 = 6.617547630 |
| Part × E[c] per pellet | 2.020322423 | 54.594767950 |
| SC, E[procs] | 0.26; COR 0.167556, SLA 0.054889, PUN 0.031778, IMP 0.005778 | same |

Without Acuity the body shot would have MS 1.9 and `1.9 × 2.020322423 = 3.838612604` per shot; the head shot with Acuity
gives 54.59 per shot, a factor 14.2.

---

## 6. Edge cases

Each item: rule, then source. "→ nn" means the rule is owned by document nn and is listed here only because the
research put it near this topic.

Multishot

- **EC-02-01** Fractional multishot fires one extra pellet with probability `frac(MS)`; expected pellets = MS exactly.
  `W:Multishot` §Calculating Multishot; `F:core` §7.
- **EC-02-02** Multishot never below 1 after a riven curse (D-019); applied also for MS0 > 1 (UNVERIFIED).
  `W:Multishot` §Sources.
- **EC-02-03** Acuity mods set MS to base, ignoring every bonus and penalty (riven curse, MS0+); since U44 only Primary
  and Pistol Acuity. `W:Multishot` §Locking Multishot (U38); `W:Primary_Acuity` §Notes.
- **EC-02-04** Beams: one merged instance per tick, damage and SC summed over the m beams, one crit roll with one
  beam's CC, forced procs (incl. Hunter Munitions, Internal Bleeding, Seeking Talons) once per tick, DoT scales twice
  with multishot (EV uses E[m²]); also innate-multishot beams (Quanta). E-2.3; `W:Multishot` §Continuous Weapons.
- **EC-02-05** Spherical AoE of beams (Ignis, Ignis Wraith, Glaxion Vandal, Gaze primary, Embolist, Catabolyst, Cortege)
  is not multiplied by multishot. `W:Multishot`.
- **EC-02-06** Hitscan explosions (Opticor, Opticor Vandal, Trumna, Ferrox, Tenet Ferrox, Ambassador alt, Braton and
  Burston Incarnon radial) are not multiplied by multishot; launcher projectiles each explode (UNVERIFIED). `F:weapon`
  §10.1; instance counts → 06.
- **EC-02-07** Speargun throw: multishot has no effect. `W:Multishot` §Spearguns.
- **EC-02-08** Kohm, Kuva Kohm, Kohmak, Twin Kohmak: per-pellet SC is the first-shot SC divided by the spool pellet
  count (data already divided for the fully spooled attack). `W:Kohm` line 30; `F:weapon` §6; spool steps → 06.
- **EC-02-09** Strun Incarnon "Blazing Barrel": +0.05 base multishot per stack, 5 stacks, lost on reload (MS0+).
  `W:Strun_Incarnon_Genesis`.
- **EC-02-10** Perks on "multishot-generated" pellets (Braton Incarnon Evolution II, Torid Plentiful Mayhem) act on
  `n_pel − 1` pellets (UNVERIFIED for MS0 > 1). `F:core` §3.1; `F:mods` §9. Torid Plentiful Mayhem in Incarnon form
  "raises all multishot bonuses by 60 %" (value semantics → 04).
- **EC-02-11** Spread and accuracy are not modelled: every pellet hits the single target. `W:Multishot` §Mechanics;
  a "pellets hitting" input is not in scope of the overview toggles.
- **EC-02-12** Every pellet rolls crit and status independently, also pellets of one shot. CD-03; `W:Multishot`.

Crit chance

- **EC-02-13** Relative bonuses multiply base CC; absolute bonuses are added after; negative absolute entries (Laetum
  −10 points) are allowed; CC ≤ 0 means no crit. `W:Critical_Hit`; `W:Calculating_Bonuses` overview (`F:core` §8).
- **EC-02-14** Smeeta Charm sets CC to 2.0 if lower; higher CC is unaffected; with Vigilante red crits are possible.
  `W:Charm` lines 34, 92-93, 105-106 (patch note fixing Charm lowering higher CC). `W:Universal_Weapon_Bonuses` "+200 %"
  loses (`F:buffs` Q5).
- **EC-02-15** Covenant: flat after mods; ×4 on weak-point hits; caps 0.5 body and 2.0 weak point; Sonar spots count as
  weak points. `W:Covenant` lines 17-20; `W:Enemy_Body_Parts` U44 (Covenant converted to weak points).
- **EC-02-16** Piercing Navigator: +0.5 per hit up to +2.5, **absolute after mods** since U40 ("Critical Chance is now
  additive"; Boltor `10 % × (1 + 150 %) + 250 %`), not affected by Strength. `W:Piercing_Navigator` §Notes and U40;
  `W:Critical_Hit` absolute list (revision 2026-09-02 "changed piercing navigator from percent to flat").
  `W:Calculating_Bonuses` §Exceptions ("applied before other bonuses") and 00 §2.4 (`cc_base_add`) are outdated (OQ-201).
- **EC-02-17** Lanka zoom crit chance is flat after mods (+0.20 / 0.30 / 0.50) and applies if scoped when the shot hits;
  other sniper zoom bonuses are relative. `W:Lanka` lines 43-44; `F:weapon` §5.4.
- **EC-02-18** Weak-point crit chance applies only on weak-point hits (head, non-head weak point, Sonar spot), never on
  radial instances; additive with Point Strike; multiplicative only with the hidden T-21. `W:Primary_Acuity`;
  `W:Enemy_Body_Parts`; D-023, D-209.
- **EC-02-19** Puncture crit chance: absolute, from the stacks before the instance, not on radial instances or
  abilities; Evensong's 7 stacks give +0.35 (value and cap → 03). `F:status` §4.2; `W:Evensong` line 17.
- **EC-02-20** Combo hooks: Blood Rush `0.4·(M − 1)`, Gladiator `0.1·pieces·(M − 1)`, Weeping Wounds `0.4·(M − 1)` in the
  relative buckets (additive since U26); True Steel family and melee riven CC doubled on heavies; pre- or post-
  consumption combo on heavies → 07; glaive airborne-throw explosions miss the heavy ×2 while Volatile explosions
  always get it (bugs, → 07). `F:addmul` §2; `F:melee` §4.1, §5, §11.
- **EC-02-21** Incarnon base additions ("Increase Base Critical Chance / Status Chance / Critical Damage Multiplier by
  +X") enter before mods; the CDM one before quantization; status bonuses per projectile; some apply to one form only
  (perk table → 09/04). The inspiration's note that Strun and Boar Incarnon-form status bonuses scale with the base
  pellet count (changelog 2023-09-22, `F:core` open 12) is the wiki's separate form value (Boar +12 % per projectile,
  +96 % for the Incarnon form) and needs no rule. `F:weapon` §13; `W:Critical_Hit` §Quantization; `W:Boar_Incarnon_Genesis`.
- **EC-02-22** Prelude of Might's condition is evaluated against the final CC of the instance, including Puncture,
  Covenant, Galvanized Scope and Acuity. `W:Braton_Incarnon_Genesis` lines 117-126. Threshold 50 % (page) vs 40 %
  (`F:mods` §9): OQ-210.
- **EC-02-23** Secondary Enervate: +0.10 absolute per hit, reset after 6 big crits (k ≥ 2); AoE and simultaneous
  hitscan multi-hits count as one hit; max 30 triggers per second (stack model → 04; P(k ≥ 2) from E-2.7).
  `W:Secondary_Enervate` §Notes.
- **EC-02-24** Kitgun and zaw: CC0, CDM0, SC0 are the assembled values (chamber or strike plus loader or link offsets;
  `06`, `07`); the assembled CDM0 is quantized as a whole (Decision). `F:weapon` §11.

Crit multiplier

- **EC-02-25** Base CDM is quantized on a 32/4095 grid in f32 before mods (2.0 → 2.000488519668579; 2.3 → 2.297435999;
  values above 32 clamp to 32); incremental base additions are quantized together with the base. E-2.5;
  `W:Critical_Hit` §Quantization. One-ulp order ambiguity: section 4.3.
- **EC-02-26** Cold crit damage is absolute, before tiers, not on radial instances; Frozen gives +1.0; 4-stack cap on
  bosses and overguarded units (values → 03). `F:status` §4.5; `W:Critical_Hit` absolute list.
- **EC-02-27** Arcane Crepuscular (+3.0 while invisible, not on Gyre abilities) and Tenacious Bond (+1.2 when the
  companion's CC ≥ 50 %) are absolute: their relative gain falls as CDM rises. `F:buffs` §8, §11 item 34.
- **EC-02-28** Buffing fields: Electric Shield, Energy Shell, Assimilate double crit damage multiplicatively and do not
  stack across shields; Mandonel ×1.5 per field; Mutalist Quanta and Simulor +0.25 additive; a projectile takes one
  field source only; melee swings get nothing. `W:Electric_Shield`; `W:Buffing_Fields`; `F:buffs` §1.12, §11 items 13-14.
- **EC-02-29** Melee Doughty: `round1(Puncture share × modded SC % × 0.1)`, cap +50, only for the Puncture-bearing part
  of an attack (U38.6), SC includes Weeping Wounds (combo-dependent). E-2.5a; `W:Melee_Doughty`.
- **EC-02-30** Spiteful Defilement +1.0 crit damage with fewer than 3 statuses (D-067; condition → 04).
- **EC-02-31** A crit can lower damage when `h·CDM < 1` (Ropalolyst head 0.2; CDM0 1.0 abilities); no clamp is applied.
  `W:Enemy_Body_Parts` §Critical Headshots.

Crit tiers

- **EC-02-32** Tiers are unbounded; colours and "!" marks are display only (U21, U34); the overflow of numbers past
  ~255× is a display bug. `W:Critical_Hit` §Critical Tiers, §Bugs, §Patch History.
- **EC-02-33** Vigilante: +1 tier with 5 % per set piece (max 30 %), primaries only, crits only. `W:Vigilante_Set`;
  `W:Critical_Hit` §Critical Tiers.
- **EC-02-34** Critical Promotion parts (Rogue Culverin canisters, Scaldra backpacks, Techrot parts marked ¹, `Multis`
  "+1 Crit Tier"): +1 tier, a non-crit becomes tier 1; order with Vigilante UNVERIFIED (OQ-205).
  `W:Enemy_Body_Parts` §Critical Promotion.

Body part

- **EC-02-35** Head multiplier 3× since U32 for Grineer, Corpus, Infested, Sentient, Anarchs, Narmer, Corrupted;
  Murmur and Techrot about 1.5×; older pages saying 2× are outdated. `W:Enemy_Body_Parts` §Weak Point Hits, U32;
  `F:core` edge case 25. Data → 05.
- **EC-02-36** Weak-point damage bonuses count 1.5× on head weak points and 1.0× on non-head weak points (MOA rears
  except Anti, Disc, Fusion; Bursa rear panels; Severed Warden arms; Lumbering Fragment; Scaldra Harbinger balloon;
  Amalgam Heqet backpack; U44). `W:Primary_Acuity`; `W:Enemy_Body_Parts` footnote 2.
- **EC-02-37** Headshot-only effects (ΣHS) do not apply on non-head weak points. `W:Enemy_Body_Parts` ("Effects that
  specify headshots do not apply against non-headshot weak points").
- **EC-02-38** Head-crit only on parts with the head-crit flag: none on Corpus humanoids and ground units (even without
  helmets), most Murmur, much of Techrot, MOA "fanny pack"; Jordas Golem engines 1× but head-crit; Juggernaut maw and
  belly 10× with crit factor 4 (bodyshot locations); Ropalolyst head 0.2. `W:Critical_Hit` §Critical Headshots;
  `W:Enemy_Body_Parts`. Data → 05; generalised h: OQ-203.
- **EC-02-39** Sonar (U44): spots are weak points (Acuity, Covenant, Worthy Comradery, Incarnon charge) with the
  part's original multiplier and headshot flag; multiplicative with the part multiplier; several nodes on one part
  multiply; no head-crit on Sonar spots; no natural head multiplier on other parts; a torso spot raises AoE damage aimed
  at the centre; DoT inheritance disputed (OQ-202, `03` OQ-312); extra hits get Sonar again per `W:Sonar` vs forced 1×
  per `W:Extra_Hit` (→ 04 OQ-412, `F:buffs` Q3). `W:Sonar` info; `W:Enemy_Body_Parts` §Notes.
- **EC-02-40** Radial damage: the body part's P and h (1 and 1 for most enemies; Eidolon body 2× with 2× crit damage,
  `W:Eidolon_Teralyst` L77 via `05` §4.3), a Sonar torso spot, no headshot or weak-point conditions, no weak-point crit
  chance, no Puncture crit chance, no Cold crit damage, no CO (→ 04); the projectile itself can headshot.
  `W:Enemy_Body_Parts` §Notes; `W:Area_of_Effect` U32; `F:status` §4.2, §4.5.
- **EC-02-41** 1×-headshot weapons (list in 4.5): P = 1 and ΣWP ignored on weak points, ΣHS still applies, no head-crit
  even when ΣHS raises the multiplier; Deadhead makes them 1.3×. `W:Enemy_Body_Parts` §Notes; `W:Critical_Hit`;
  `W:Primary_Deadhead` line 40.
- **EC-02-42** `ExtraHeadshotDmg`: −2 marks a 1×-headshot attack; +0.2 (Synapse, Coda Synapse) and +0.5 (Kuva
  Chakkhurr) enter ΣHS; +0.5 on Cernos Prime is the unique multiplicative M_hsu. `Module:Weapons/data`;
  `W:Enemy_Body_Parts` weapon tables; `DS:wiki` (semantics undocumented, OQ-221).
- **EC-02-43** Meticulous Aim: −45 % on every non-headshot hit (incl. MOA backpacks), +105 % headshot multiplier
  additive with zoom and Target Acquired; applied twice to Slash, Heat, Toxin, Blast DoT (→ 03). `W:Meticulous_Aim`.
- **EC-02-44** Electricity and Gas procs have their own 1× headshot multiplier, raised by ΣHS (→ 03, OQ-025).
  `W:Enemy_Body_Parts` §Notes; `W:Primary_Deadhead` line 41.
- **EC-02-45** Extra hits other than Xata's Whisper use a forced 1× weak-point multiplier (→ 04, E-14). `W:Extra_Hit`.
- **EC-02-46** `W:Damage/Calculation` §Weak Point Hit (`AvgShot × (WP mult + WP bonus)`) and §DoT
  (`WP mult × (1 + WP bonus)`) contradict `W:Enemy_Body_Parts`; the engine follows `W:Enemy_Body_Parts` (per-mechanic,
  newer, worked examples). `F:core` open 6.
- **EC-02-47** Body parts below 1 (armor plates 0.5×, Deimos Charger carapace 0.05×) and 0× bodies (Jordas Golem,
  Lephantis, Condrix) are ordinary P values; Acuity on a Sonar-highlighted 0.05× part gives `5 × (0.05 + 3.5)`.
  `W:Enemy_Body_Parts` tables; `W:Primary_Acuity` §Tips.

Status chance

- **EC-02-48** SC is per pellet since U27.2 (the old shotgun formula `1 − (1 − SC)^(1/n)` is obsolete).
  `W:Status_Effect` §Trivia, §Multishot.
- **EC-02-49** Tau: `SC × (1 + 0.1·n)`, max ×2 at 10 stacks (cap → 03), multiplicative; applies to merged beam SC;
  does not add forced procs; an instance's own Tau proc does not raise its own SC. `W:Damage/Tau_Damage`; CD-04.
- **EC-02-50** Field status multiplier (Mandonel ×2) multiplies SC; the inspiration hooks `noIncrStatus` and
  `set_status_chance` exist without a game source (hooks only). `W:Buffing_Fields`; `I:stat` §10.
- **EC-02-51** Negative SC (kitgun loader offsets) rolls as 0. `DS:wiki` (Splat loader StatusChance −0.08).
- **EC-02-52** Heavy-only and Tennokai-only status bonuses (Empowered Blades, Condition's Perfection) apply only to those
  hits (flags → 07). `F:buffs` §4; `F:melee` §4.2.
- **EC-02-53** Melee stance `StatusMultiplier` (6 uses in `Module:Stances/data`) has no documented meaning; ignored
  until `07` defines it. `F:melee` open 11.

Procs

- **EC-02-54** SC above 1: `floor(SC)` guaranteed procs plus one with `frac(SC)`; types drawn independently, duplicates
  possible. `W:Status_Effect` §Status Chance; OQ-003.
- **EC-02-55** Weights from the modded composition after element combination; combined elements remove their
  components' procs; resistances and armor do not change weights; physical 4× weighting ended in U27.2. E-2.13.
- **EC-02-56** Status immunity removes the type and renormalizes; forced procs of an immune type are dropped; "all"
  immunity gives no statuses at all. `W:Status_Effect` §Status Immunity Interactions, §Status Immune Enemies.
- **EC-02-57** Ragdolled target: an Impact draw inflicts nothing (no renormalization); Internal Bleeding and Hemorrhage
  still trigger. `W:Damage/Impact_Damage` lines 25-26. State model: OQ-219.
- **EC-02-58** Damage types without a status (True, Finisher, Cinematic) are not in the weights (Decision, OQ-206).
- **EC-02-59** Forced procs are independent of SC, coexist with rolled procs, count per pellet and per beam tick, are
  not multiplied by Tau; crowd-control-only names carry no StatusType. `W:Status_Effect` §Forced Procs, §Average Procs;
  `W:Damage/Tau_Damage` line 17.
- **EC-02-60** Conditional weapon-trait forced procs (distance, direct hit, full charge) need a hand-authored condition
  table; the wiki data lists them unconditionally. `W:Internal_Bleeding` tips list.
- **EC-02-61** Evensong: 7 forced Puncture per direct hit and per explosion (5 on 4-capped enemies). `W:Evensong`.
- **EC-02-62** Cedo explosion: 2 forced procs, first weighted over modded elements and innate Blast, second uniform
  over the four primaries; SC cannot add more. `W:Cedo` (D-202).
- **EC-02-63** Nukor and Kuva Nukor apply Microwave (counts for CO, → 03); no data field (OQ-215). `W:Microwave`.
- **EC-02-64** Conductive Sphere: +1 forced Electricity proc per field (→ 04). `W:Buffing_Fields`.
- **EC-02-65** Hunter Munitions: 30 % per critical instance on primary weapons (Artemis Bow since U33), independent of
  SC and weights, tier-independent (OQ-018), direct and radial instances each roll, once per beam tick. E-2.16.
- **EC-02-66** Internal Bleeding and Hemorrhage: 35 % per instance with ≥ 1 Impact status, doubled when the modded fire
  rate is strictly below 2.5 (fire-rate buffs count; bows use hidden fire rates 1.0, Cinta 1.5, Kuva Bramma and Nataruk
  0.667); at most once per instance. E-2.15.
- **EC-02-67** Magnetic Welt (shotguns, U38): same as Internal Bleeding with Magnetic. `W:Magnetic_Welt`.
- **EC-02-68** Hunter Munitions and Internal Bleeding draw independently but add at most one Slash together (54.5 % /
  79 % examples). `W:Hunter_Munitions`; `W:Internal_Bleeding`.
- **EC-02-69** Internal Bleeding, Hemorrhage and Magnetic Welt add nothing when the instance already carries a rolled
  or weapon-forced proc of their type; Hunter Munitions stacks with a rolled Slash (UNVERIFIED, OQ-204). E-2.15, E-2.16.
- **EC-02-70** Seeking Talons' forced Slash is suppressed by any Slash on the instance (rolled, HM, IB). `W:Seeking_Talons`.
- **EC-02-71** Secondary Encumber: per shot at most one extra proc, uniform over 13 statuses, pellets tried until one
  succeeds, forced procs can trigger it, exalted weapons since U38.5. E-2.17; `W:Secondary_Encumber`.
- **EC-02-72** Archon Continuity and Archon Vitality affect ability procs and exalted weapons only. `W:Archon_Continuity`;
  `W:Archon_Vitality` (D-204).
- **EC-02-73** Secondary Cryogenic and Primary Debilitate are proc-triggered arcanes: trigger probabilities from
  E-2.18, effects → 04 and 03; Debilitate fires once per instance (beam ticks therefore gain less). `W:Primary_Debilitate`.
- **EC-02-74** Melee Duplicate triggers on tier-1 crits only (→ 04; P(k = 1) from E-2.7). `F:melee` §7.
- **EC-02-75** Heavy attacks roll normal status besides their class forced procs (Bleed on Claws, Nikanas, Rapiers,
  Scythes ...). `F:melee` §4.1; `F:status` §4.3. Data → 07.
- **EC-02-76** Per-hit and live stack caps of status-capped enemies (bosses 4, overguarded units 4 Cold, VIP overguard 3
  Puncture) are applied by 03 to the proc list produced here (CD-07, D-053). Parts with `status_ok = false` (Juggernaut
  outside maw and belly: "status only through weak points") deliver no procs (E-2.13). `05` §4.3-§4.4; `F:enemy` §9.
- **EC-02-77** Quick shots and partial charges are separate attacks with their own CC, CDM, SC (Lanka partial charge:
  reduced crit chance; bow quick shots share crit and status with charged shots since U25.7, Epitaph does not); data → 06.
  `F:weapon` §3.3.
- **EC-02-78** Attacks that cannot crit (Exalted Blade's slam radial) carry CC0 = 0 in data (→ 07); the inspiration
  flag `disableBodyCrit` (no crit unless headshot, JS L10693) has no identified game counterpart and is kept as a hook.
  `F:melee` §13.
- **EC-02-79** Weapon traits and arcanes that change crit or procs but whose game rule was not researched are hooks
  for `04`'s catalogue: Gotva Prime `crit_after_status` (CC override after a status), Sagek Prime passive (next hit
  rolls at 500 % SC), Melee Afflictions (+6 stacks of every present status on knockdown, ragdoll or lift), Cascadia
  Empowered (flat damage per elemental status). `I:queue` §3.1-§3.4; `F:melee` §7.

Explicitly out of scope (with reason):

- Multiple enemies hit by one attack each roll separately (single target, 00 §1.3).
- Hits on Objects (Nullifier bubbles, Tornado), non-body hitboxes (Symbilyst shield, Aerolyst canisters, Efervon Tank
  growths): not an enemy damage instance of the target model.
- Enemy critical hits (removed in U35.0.9), ability crits, Breach Surge sparks, Zenurik Temporal Shot (amp only),
  Hard Engage (companion melee): ability, enemy or companion damage (00 §1.3).
- Inspiration's acolyte "metric" crit replacement (`I:enemy` §5): dead code (unique DR disabled, D-047).

---

## 7. Inspiration differences

The rows D-0xx are the overview's (00 §5.2) restated with the detail the engine needs; D-2xx rows are new proposals
in this document's range, in the register's column layout. "Compat" gives the emulation used by the compatibility
profile (00 §4.6).

| ID | Topic | Inspiration behaviour | Game behaviour (source) | Engine (game profile) | Toggle; compat emulation |
|---|---|---|---|---|---|
| D-003 | Base CDM quantization | none: `(crit_mult + flat_critmult)·(1 + Σ)` (JS L8268-8329; `I:stat` §9.2) | 32/4095 grid in f32 before mods (`W:Critical_Hit` §Quantization) | E-2.5 | Compat: `Q = x` |
| D-010 | Riven Weak Point Critical Chance | stored under `SMITE`, no effect (JS L14722-14725) | relative crit chance on weak-point hits (`F:addmul` §11) | ΣCC_WP | — |
| D-011 | Riven Weak Point Damage | mapped to `mult_for_head` (headshot-multiplier bucket) (JS L14728) | ΣWP (`F:addmul` §10-§11; spliced trait fixed in 44.1.1) | ΣWP | — |
| D-012 | Custom "Weak Point Increase Damage" (p.v) | raw value added to the head part `3 + x` while Acuity is stored as 5.25 = 1.5 × 3.5 (JS L9070-9076) | ΣWP counts 1.5× on head weak points | ΣWP (head gains 1.5 × value) | — |
| D-013 | Head-crit | every headshot crit uses `2·CDM` (JS L8319-8328) | only head-crit parts (`W:Critical_Hit`, `W:Enemy_Body_Parts`) | h from part data | — (see also D-211) |
| D-014 | Absolute crit damage placement | `crit_mult_add` and Melee Doughty (`incr_CM_by_punc_status`: `min(puncShare × SC% × 0.1, 50)`, unrounded) added after tiers: `1 + k(CM − 1) + abs` (JS L8293-8328) | before tiers, inside CDM (`W:Arcane_Crepuscular`, `W:Critical_Hit`); Doughty rounded to 0.1 (`W:Melee_Doughty`) | E-2.5, E-2.5a | Compat: `c_k = 1 + k(h·CM' − 1) + abs + doughty_unrounded` |
| D-015 | Cold crit damage (02 placement, 03 values) | +0.05 per stack (+0.5 more at 10) added to the final multiplier after tier and headshot; also on AoE (JS L10821-10825) | absolute in CDM before tiers; not on radial (`F:status` §4.5) | E-2.5 | Compat: `c ← c + 0.05·n_COL + 0.5·[n_COL = 10]` when c > 1 |
| D-016 | Hunter Munitions | needs a crit, then rolls `0.3 × min(CC, 1)` (JS L10761-10766): P = 0.3·CC² below CC 1 | 30 % per critical instance (`W:Hunter_Munitions` lines 2, 38) | E-2.16 | Compat: `P = 0.3·min(CC,1)·[k ≥ 1 or CC ≥ 1]`, skipped when IB fired |
| D-017 | Fractional chances | fractions of CC, SC, MS and the beam count rounded to whole percent before rolling (JS L10005, L10012, L10054, L10679) | exact | exact | Compat: use `round(100·frac)/100` |
| D-018 | Tier on a failed roll | tier from `CC − 99` (percent units): one tier too many when `frac(CC) ∈ (0.99, 1)` (JS L9826, L8314-8322) | `floor(CC)` | E-2.6 | — |
| D-019 | Multishot floor | none (JS L8983-9006) | ≥ 1 (`W:Multishot` §Sources) | E-2.1 | — |
| D-020 | Proc type selection | 100-slot mask, `ceil(100·w)` slots per type in damage-key order; the last types lose up to n − 1 points; if every type is immune the placeholder `splice` removes `1 + floor(SC)` entries (JS L12239-12246, L10020-10032) | exact proportional weights (`W:Status_Effect`) | E-2.13 | Compat: P(type) = slots of that type inside indices 0-99 / 100 |
| D-021 | Secondary Encumber | per pellet ("hit object") at most one, 24 % per placeholder until one succeeds; also one extra roll per weapon forced proc; type `x[round(random·12)]` gives Impact and Viral half weight (JS L9980-10041, L12863-12876) | per shot ("instant") at most one; uniform over 13 statuses (`W:Secondary_Encumber`; now sourced) | E-2.17 | — |
| D-023 | Weak-point crit chance multiplicative | hidden option `o.w` / attack flag `isWPCritMult`: `(1 + WP CC)` factor (JS L8201-8210) | additive (`W:Primary_Acuity`, `W:Enemy_Body_Parts`) | additive; T-21 keeps the inspiration form for link import | User (T-21, hidden) |
| D-024 | Proc weights on quantized types | weights from unquantized modded damage (JS L10063-10077) | not stated | `q_t` (Decision, OQ-004) | — |
| D-201 | Prelude of Might condition (`flat_critmult_below_cchance`) | compared against the tier input, i.e. `CC − 99` after a failed roll above 100 % (so CC 1.2 qualifies as 0.21); `reduce` keeps only the last mod's value (JS L8278-8286, L8868-8876) | final CC of the hit below the threshold (`W:Braton_Incarnon_Genesis` lines 119-126) | E-2.5 with the instance CC | Compat: compare `ccArg` |
| D-202 | Cedo explosion forced procs | one uniform base element plus one uniformly random damage key of the attack (JS L12854-12863) | first proc weighted over modded elements and innate Blast, second uniform over HCET primaries (`W:Cedo` lines 42-46) | 4.7.2 | — |
| D-203 | Tau status-chance vulnerability state | counts only the same attack's own Tau procs, capped at 10 regardless of the enemy cap; the displayed SC assumes 10 stacks and is reused by Melee Duplicate re-rolls and `ifMaxStatSlash` (JS L9987-9996, L8893-8901, L10721, L9459-9467) | every Tau stack on the target (any source), capped per `03`, read before the instance (`W:Damage/Tau_Damage`, CD-04) | E-2.11 with the target state | — |
| D-204 | Archon Continuity / Vitality | add one Corrosive per hit with Toxin, one extra Heat per hit with Heat, for every weapon (JS L10029-10030, L10042-10043) | ability procs and exalted weapons only (`W:Archon_Continuity`, `W:Archon_Vitality`) | only on exalted-weapon instances | — |
| D-205 | Impact-triggered procs vs an already rolled proc of the same type | IB/Hemorrhage/Welt add their proc regardless; Hunter Munitions is skipped when IB fired (JS L9853-9878, L10761) | IB/Hemorrhage/Welt: no extra proc alongside a weapon's own Slash/Magnetic; HM stacks with a rolled Slash; HM and IB give one Slash together (mod pages) | E-2.15, E-2.16 (UNVERIFIED, OQ-204) | — |
| D-206 | Custom "Critical Multiplier (multiple after all sources)" (p.i) | multiplies the whole per-hit crit value, including the headshot part and the absolute terms: `val·(1 + x)` (JS L8293-8328) | custom stat, no game source; 00 §6.3 maps it to M_cdf (before tiers) | M_cdf | Compat: `c_k ← c_k·(1 + x)` (head part included) |
| D-207 | Weapon trait `ifMaxStatSlash` (status chance when Slash is the largest physical type) | stored in a global that is never reset and applied to attack index 0 only (JS L9459, L10001; `I:stat` Q16) | per attack (trait semantics from weapon data, no wiki source) | ΣSC_rel of the attack whose largest physical type is Slash; no carry-over | — |
| D-208 | Vigilante on non-primary weapons | `double_crit` 0.05 summed over all equipped pieces, including the Warframe pieces, for any weapon class (JS L9829, L10682; data L911, L995, L1111, L2181, L5040, L5047) | "Critical Hits from Primary Weapons" only (`W:Vigilante_Set`) | p_vig = 0 for non-primary attacks | — |
| D-209 | Weak-point crit chance on radial attacks | `crit_chance_weakp` and `headCritChance` added whenever the headshot option is on, also for AoE attacks (JS L8195-8210; no AoE test) | radial damage cannot trigger weak-point conditions (`W:Enemy_Body_Parts` §Notes, U32) | wp_i = 0 for radial instances | — |
| D-210 | Headshot bonuses on attacks flagged `no_headshot_mult` | the flag removes both the head multiplier and the Deadhead-type factor `ta` (JS L10358-10367) | 1×-headshot weapons keep ΣHS (`W:Enemy_Body_Parts` §Notes); radial attacks get neither | 1×-headshot: A = 1, B = 1 + ΣHS; radial: body part values (E-2.10) | — |
| D-211 | Head multiplier per enemy | fixed `headShotMultInitial = 3` for every enemy (JS L5456, L9070-9076) | per part: 3.0 most heads, ~1.5 Murmur and Techrot, 1.0 or 1.875 on some heads, crit-promotion parts (`W:Enemy_Body_Parts`) | P_p from `05` | Compat: P = 3, h = 2 on every headshot (with D-013) |
| D-212 | Status-chance multipliers | one additive factor `1 + Σ status_chance_mult + custom + 0.1·n_TAU` (JS L9891-9895) | Tau "multiplicative to the status chance of the source" (`W:Damage/Tau_Damage`); 00 §6.3: custom p.k is a factor `(1 + x)` | product of factors (E-2.11) | Compat: additive form |

Rows of other owners that touch E-4 to E-10: D-094 (`07`, custom crit chance on heavy attacks counted per slot) feeds
ΣCC_rel; D-067 (`04`, Spiteful Defilement) enters ΣCD_rel; D-070 (`03`, Hunter Munitions proc as a stack) consumes the
procs of E-2.16; D-030 (`03`, own-hit timing) also covers the inspiration's same-shot Tau counting.

---

## 8. Open questions

Questions of 00 §8 owned here, with this document's decision: OQ-003 (proc count sampling: `floor + Bernoulli`),
OQ-004 (weights from `q_t`), OQ-016 (head-crit doubles total CDM; field multiplier on CDM), OQ-018 (one Hunter
Munitions roll per crit instance), OQ-017 (Puncture and Cold carry into the DoT through the inducing hit's k and CDM,
consumed by `03`).

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-201 | Piercing Navigator: 00 §2.4 lists it under `cc_base_add` (before mods, from `W:Calculating_Bonuses`); `W:Piercing_Navigator` (U40 "Critical Chance is now additive") and `W:Critical_Hit` (2026-09-02) list it as absolute after mods | ΣCC_abs; proposal: the manager removes Piercing Navigator from `cc_base_add` in 00 §2.4 | Boltor (CC0 0.10) with Point Strike under Navigator after 5 hits: HUD buff icon (shows final CC) 2.75 if absolute vs (0.10 + 2.5) × 2.5 = 6.5 if a base addition |
| OQ-202 | Does a DoT inherit the Sonar multiplier? `W:Sonar` (revision 2026-09-25, after the U44 rework: "All status DoTs are affected by Sonar now") vs `W:Damage_over_Time` and `W:Damage/Slash_Damage` (statement from 2023-03) | this document keeps M_son separate so `03` can choose (`03` tracks it as OQ-312 and keeps E-13); recommendation to `03`/manager: change E-13 to `Part_dot = Part` (newer, per-mechanic source) | Slash tick numbers from a hit on a Sonar spot vs the same hit off the spot, same crit tier |
| OQ-203 | Head-crit factor beyond {1, 2}: Juggernaut 4, Ropalolyst 0.2, Eidolon/Lephantis/Hemocyte 2 on non-head weak points, `Multis` "1.5x Critical Damage" | h = data value; proposal: 00 §2.4 `headcrit` becomes "part crit factor, 1 by default" | in-game crit numbers on Juggernaut maw and Ropalolyst head |
| OQ-204 | Internal Bleeding / Hemorrhage / Magnetic Welt when the instance already has a rolled Slash / Magnetic | suppressed (mod pages), while Hunter Munitions stacks (its page) | single-pellet Impact + Slash weapon with Internal Bleeding: count Slash procs per hit on hits that rolled both Impact and Slash (2 vs 1) |
| OQ-205 | Order of Vigilante and Critical Promotion; does a promoted non-crit trigger "on crit" effects and Vigilante? | Vigilante on the rolled k0; promotion after; "on crit" reads the final k | 0 % CC weapon with Hunter Munitions on Scaldra backpacks: Slash procs present or not |
| OQ-206 | Do True, Finisher and Cinematic damage dilute proc weights? | excluded from the denominator | Basmu (True) proc distribution over 500 hits |
| OQ-207 | How do Tau, custom "vulnerable to status chance" and Pageflight combine? | separate factors (00 §6.3) | Tau weapon at 5 stacks with Pageflight active: proc counts per hit at a known SC |
| OQ-208 | Kohm: is the per-pellet division by the spool pellet count or by the modded pellet count? | spool count (data), multishot mods add pellets without further division | Kohm at full spool with and without Split Chamber equivalent: procs per shot |
| OQ-209 | Melee Doughty: absolute or relative bucket; quantized or unquantized share; rounding of ties | absolute; unquantized share; half up | Destreza Prime crit numbers with and without Doughty at a fixed combo |
| OQ-210 | Prelude of Might threshold: 50 % (`W:Braton_Incarnon_Genesis`) or 40 % (`F:mods` §9) | 50 % (per-weapon page, with the U34 fix note) | Braton Incarnon at 45 % final CC: arsenal CDM with and without the perk |
| OQ-211 | 1×-headshot weapons on non-head weak points; weak-point crit chance on 1×-headshot weapons | P → 1 on every weak point; weak-point CC still applies | Arca Plasmor on a MOA rear with Acuity-like bonuses (Seek) |
| OQ-212 | Each launcher pellet spawns its own explosion? Generated-pellet count for perks on MS0 > 1 weapons | yes; `n_pel − 1` | Ogris with Split Chamber: explosion numbers per shot; no MS0 > 1 perk known |
| OQ-213 | Quanta example in `W:Multishot` says instances "always remain 2 per tick" while the same section says beams on one target merge | merge (CD-10) | Quanta damage numbers on one target with and without Split Chamber |
| OQ-214 | Internal Bleeding threshold for charge and spool weapons: FR or effective fire rate; spool state | modded FR (06 `fire_rate`), full spool | Opticor (FR 1) with Internal Bleeding: Slash rate per Impact proc |
| OQ-215 | Microwave application on Nukor: every hit, per beam tick, or rolled? | forced on every instance (hand-authored) | CO-type damage increase on the first vs second tick of Nukor on a fresh target |
| OQ-216 | Stun: counted by CO (`W:Status_Effect` §Independent says yes; `F:addmul` §1.2 list says no); no StatusType exists | crowd-control-only here; `03` decides CO counting (proposal: STN in 00 §2.2 if counted) | Galvanized Aptitude damage on a target stunned by a forced Stun proc |
| OQ-217 | Primary Debilitate: `W:Primary_Debilitate` says faction applies three times on its Heat/Electricity/Toxin DoT; D-073 decided twice | trigger rule here; faction count → `03` (recommend revisiting D-073) | Debilitate Heat tick vs ordinary Heat tick with Primed Bane |
| OQ-218 | Does an Encumber Impact trigger Internal Bleeding; is an immune Encumber type re-rolled? | no; dropped | Secondary Encumber + Hemorrhage on a 0-Impact pistol: any Slash procs |
| OQ-219 | Ragdoll state: duration and which forced Ragdoll procs (29 attacks, 78 stance hits) keep the target ragdolled | not modelled: the target is never ragdolled unless `03` defines a state model | Impact proc counts from a forced-Ragdoll heavy followed by fast hits |
| OQ-220 | Cross-stat Incarnon perks ("Base Critical Chance by +x % of current Status Chance", "Base Status Chance by +x % of current Critical Chance", inspiration changelog 2024.10.08): which perks, which order | hooks: base addition computed from the other stat's final value without its own cross term, one pass (inspiration order) | wiki perk text for the weapons carrying them |
| OQ-221 | `ExtraHeadshotDmg` semantics (0.2, 0.5, −2) | −2 → 1× flag; positive → ΣHS except Cernos Prime (unique factor) | Synapse headshot numbers with Deadhead: 3 × (1 + 0.2 + 0.3) vs 3 × 1.2 × 1.3 |
| OQ-222 | Exact f32 decode order of the CDM quantization (one ulp) | correctly rounded quotient | in-game crit numbers at CDM0 1.5 with large damage (difference 1.2·10⁻⁷ relative; practically unobservable) |
| OQ-223 | Overview E-7 says `Part = 1` on body hits; Meticulous Aim, body parts ≠ 1 and Sonar body spots need `P_body·(1 − ΣBP)·M_son` | E-2.10 (radial and body hits use the body part's P and h, as `05` §4.3 and its OQ-524 request); proposal: the manager widens E-7's body branch | Vulkar with Meticulous Aim body numbers (example on `W:Meticulous_Aim`: 596.25 → 327.93) |
| OQ-224 | Does radial damage take the Sonar multiplier of a torso spot (`W:Sonar`: "A sonar spot on an enemy's torso will increase the damage of many AoE damage sources")? | yes, `M_son,centre` when the scenario puts the spot on the torso | Tonkor explosion numbers on a target with a torso Sonar spot vs a head spot |

The 00 §3.8 worked example prints CDM = 4.4010749; the exact value of E-2.5 is 4.401074743270875 (a display slip in the
7th significant digit; no rule changes).

---

## 9. Test vectors

Tolerances: integers and f32 values (Q) must match bit for bit; f64 results within 1e-9 relative. Every value below was
computed with `python3` (numpy float32 for Q). Unless stated, M_cc = M_cdf = 1, no conditions, p_vig = 0, π_p = 0,
no stealth (c_0 = 1).

**TV-02-01 Crit multiplier quantization (E-2.5)**

| Input (CDM0 + CDM0+, ΣCD_rel, ΣCD_abs, M_cdf) | code | Q (f32, exact decimal) | CDM |
|---|---|---|---|
| 2.0, 1.2, 0, 1 | 256 | 2.000488519668579 | 4.401074743270875 |
| 1.6 + 0.4 (f32 sum = 2.0), 1.2, 0, 1 | 256 | 2.000488519668579 | 4.401074743270875 |
| 2.3, 0, 0, 1 (Kohm) | 294 | 2.297435998916626 | 2.297435998916626 |
| 1.5, 0, 0, 1 | 192 | 1.5003663301467896 | 1.5003663301467896 |
| 1.6, 0, 0, 1 | 205 | 1.6019536256790161 | 1.6019536256790161 |
| 40.0 (clamped), 0, 0, 1 | 4095 | 32.0 | 32.0 |
| 0.0, 0, 0, 1 | 0 | 0.0 | 0.0 |
| 1.5, 0, 3.0 (Crepuscular), 1 | 192 | 1.5003663301467896 | 4.5003663301467896; c_1, c_2, c_3 = 4.5003663301467896, 8.000732660293579, 11.501098990440369 |
| 2.0, 1.2, 0, 2 (Electric Shield) | 256 | 2.000488519668579 | 8.80214948654175 |

**TV-02-02 Crit tiers above 100 % (E-2.4, E-2.6 to E-2.8)**

Input: CC0 0.30, ΣCC_rel 1.5, ΣCC_abs 0.45 (Arcane Avenger), CDM from row 1 of TV-02-01, h = 1.
Expected: CC = 1.2; P(k = 1) = 0.8, P(k = 2) = 0.2 (f64: 0.19999999999999996); c_1 = 4.401074743270875,
c_2 = 7.802149486541749; E[k] = 1.2; E[c] = 5.08128969192505 (wiki 5.08129). Second input CC0 0.12 → CC = 0.75.

**TV-02-03 Vigilante and the Smeeta floor (E-2.7)**

| CC | p_vig | Distribution | E[k] |
|---|---|---|---|
| 1.5 | 0.3 | P1 0.35, P2 0.50, P3 0.15 | 1.8 |
| 0.4 | 0.3 | P0 0.60, P1 0.28, P2 0.12 | 0.52 (E[c] at CDM 3.0: 2.04) |
| 0.75 raised to 2.0 by CC_min = 2.0 | 0.3 | P2 0.70, P3 0.30 | 2.3 |
| 2.3 with CC_min = 2.0 (unchanged) | 0 | P2 0.70, P3 0.30 | 2.3 |

**TV-02-04 Critical Promotion part (E-2.6)**: CC 0.3, p_vig 0, π_p = 1 → P(k = 1) = 0.7, P(k = 2) = 0.3, E[k] = 1.3,
P(k ≥ 1) = 1.

**TV-02-05 Body part (E-2.10) and head-crit**

| Case | Inputs | Part |
|---|---|---|
| head weak point | P 3, ΣWP 4.25 (Acuity 3.5 + Seek 0.75), ΣHS 0.7 (Deadhead 0.3 + Prowl 0.4) | 15.9375 |
| non-head weak point | P 3, ΣWP 4.25 | 7.25 |
| Sonar on a 1× limb | P 1, ΣWP 4.25, s_son 5, ν 1; × 40 base | 26.25 (× 40 = 1050) |
| Sonar on the head | P 3, ΣWP 4.25, s_son 5; × 40 | 46.875 (× 40 = 1875) |
| Acuity + Deadhead head | P 3, ΣWP 3.5, ΣHS 0.3 | 10.725 |
| 1×-headshot weapon, head | P 3, ΣWP 3.5, ΣHS 0.3 | 1.3 (h = 1) |
| Cernos Prime head | P 3, ΣHS 0.3, M_hsu 1.5 | 5.85 |
| Meticulous Aim body | P 1, ΣBP 0.45 | 0.55 |
| Meticulous Aim head | P 3, ΣHS 2.35 (zoom 0.7 + MA 1.05 + TA 0.6) | 10.05 |
| radial | body part P_body 1, h 1 (Eidolon body: P 2, h 2) | 1 (Eidolon body: 2, with h = 2 in c_k) |

Head-crit: CDM 4.401074743270875, h = 2, k = 1 → c_1 = 8.80214948654175; with Part 8.25 (Acuity only) the hit factor is
72.61773326396943 (`F:core` §10: 72.618); E[c] on that head at CC 1.2 = 10.362579383850099. Juggernaut maw (P 10,
h 4, CDM0 2.0 quantized, no mods, k = 1): Part × c = 10 × (1 + (4 × 2.000488519668579 − 1)) = 80.01954078674316.

**TV-02-06 Crit chance assembly (E-2.4)**

| Case | Inputs | CC |
|---|---|---|
| Covenant body | CC0 0.10, ΣCC_rel 1.5, v_cov ≥ 0.5, Avenger 0.45, wp 0 | 1.2 |
| Covenant weak point | same, wp 1 | 2.7 |
| Covenant uncapped | v_cov 0.2: body adds 0.2, weak point adds 0.8 | – |
| Acuity weak point | CC0 0.12, ΣCC_rel 1.5, ΣCC_WP 3.5, wp 1 | 0.72 (body 0.30; T-21 on: 1.35) |
| Puncture 5 stacks | CC0 0.30, ΣCC_rel 1.5, punc_cc 0.25 | 1.0 (radial: 0.75) |
| Lanka max zoom | CC0 0.25, ΣCC_rel 1.5, ΣCC_abs 0.5 | 1.125 |
| melee combo | CC0 0.25, True Steel 1.2, Blood Rush 0.4 × 11, Gladiator 3 pieces 0.1 × 3 × 11, Avenger 0.45, M = 12 | 2.925 |
| Piercing Navigator | CC0 0.10, ΣCC_rel 1.5, ΣCC_abs 2.5 | 2.75 |
| negative absolute entry (Laetum-type −10 points) | CC0 0.05 (illustrative), ΣCC_rel 0, ΣCC_abs −0.10 | CC −0.05, CC_r 0 (no crit) |

**TV-02-07 Status chance (E-2.11)**

| Case | Inputs | SC |
|---|---|---|
| Tau 5 stacks | SC_w 2.1, n_TAU 5 | 3.15 |
| Tau 10 stacks | SC_w 2.1, n_TAU 10 | 4.2 |
| Weeping Wounds | SC0 0.2, ΣSC_rel 0.6 + 0.4 × 3 (M = 4) | 0.56 |
| Kohm full spool | SC0 0.0625, ΣSC_rel 1.7; MS 12 × (1 + 1.2) = 26.4 | 0.16875 per pellet; E[procs per shot] 4.455 |
| Incarnon base addition | SC0 0.12, SC0+ 0.28, ΣSC_rel 0.9 | 0.76 |

**TV-02-08 Proc counts and types (E-2.12, E-2.13, E-2.18)**

Weights from 00 §3.8 (q = 2/32, 11/32, 19/32, 58/32): w = IMP 0.022222, PUN 0.122222, SLA 0.211111, COR 0.644444.

| SC | P(N = n), P(N = n + 1) | E[N_s] IMP, PUN, SLA, COR | A_s = 1 − G(1 − w_s) IMP, PUN, SLA, COR |
|---|---|---|---|
| 2.5 | P(2) 0.5, P(3) 0.5 | 0.055556, 0.305556, 0.527778, 1.611111 | 0.054573388, 0.276591907, 0.443346365, 0.914315501 |
| 0.26 | P(0) 0.74, P(1) 0.26 | 0.005778, 0.031778, 0.054889, 0.167556 | equal to E[N_s] (at most one proc) |

**TV-02-09 Immunity and ragdoll (E-2.13)**

Damage 20 IMP, 5 PUN, 10 SLA, 25 HEA, 50 COR (`W:Status_Effect`): w = 0.181818, 0.045455, 0.090909, 0.227273, 0.454545.
Corrosive-immune target: w = 0.333333, 0.083333, 0.166667, 0.416667, 0. Ragdolled target (no immunity), SC 1.0: delivered
procs per instance 0.818182 (IMP draws dropped, no renormalization); P(Impact roll) for Internal Bleeding stays 0.181818.

**TV-02-10 Hunter Munitions and Internal Bleeding (E-2.15, E-2.16)**

- Wiki check, crit and Impact certain: fast weapon (p_IB 0.35) 0.545; slow weapon (p_IB 0.70) 0.79.
- EV per pellet: CC 0.5, CDM 3.0, h 1 (E[c] = 2.0, P(k = 0) = 0.5, C1 = 1.5); SC 0.3, w_IMP 0.4, w_SLA 0.1; p_HM 0.3,
  p_IB 0.35 (FR ≥ 2.5); no forced procs. π_IB = G(0.9) − G(0.5) = 0.12; E[X_SLA] = 0.1857; E[c·X_SLA] = 0.5151.
  Same weapon with FR 2.0 (p_IB 0.70): E[X_SLA] = 0.2214. Hunter Munitions alone: game 0.15, inspiration (D-016) 0.075.
- SC 1.5, w_IMP 0.4, w_SLA 0.1: G(0.9) = 0.855, G(0.5) = 0.375, π_IB = 0.48.

**TV-02-11 Multishot and beams (E-2.1 to E-2.3)**

| Case | Inputs | Output |
|---|---|---|
| Lex | MS0 1, ΣMS 1.8 | MS 2.8; P(2) 0.2, P(3) 0.8; E[n_pel²] 8.0 |
| Hek + Hell's Chamber | MS0 7, ΣMS 1.2 | MS 15.4; P(15) 0.6, P(16) 0.4 |
| riven curse | MS0 1, ΣMS −0.6 | 1 (floor); with Split Chamber (ΣMS 0.3): 1.3 |
| Acuity | MS0 1, ΣMS 0.9, ms_lock | 1 |
| base addition | MS0 1, MS0+ 0.25, ΣMS 1.2 | 2.75 |
| speargun throw | MS0 1, ΣMS 0.9, ms_applies false | 1 |
| Amprex beam | MS 2.5, SC 0.4 | m 2 or 3 (0.5 each); SC_tick 0.8 / 1.2; E[m] 2.5; E[m²] 6.5; E[procs per tick] 1.0; E[m·procs] 2.6 |

**TV-02-12 Secondary Encumber (E-2.17)**

MS 1.9, SC 0.5, no forced procs, c_enc 0.24: a = 0.12, P_enc = 1 − (0.1 × 0.88 + 0.9 × 0.88²) = 0.21504; per type
0.0165415385. Integer-pellet check (wiki formula, 2 pellets): 0.2256. MS 1, SC 2.0: 0.24. MS 7, SC 0.107: 0.166489026.

**TV-02-13 Target-state averaging of crit (E-2.9, CD-13)**

CC without Puncture 0.9, punc_cc(n) = 0.05·n with n = min(N, 5), N ~ Poisson(2); p_vig 0.3; CDM0 2.0 quantized, no mods,
h 1. P(n) = 0.1353352832, 0.2706705665, 0.2706705665, 0.1804470443, 0.0902235222, 0.0526530173 (n = 0 ... 5);
E[n] = 1.97751200771565. E[k] = 1.290755483391586 (distribution) vs 1.2985382805015173 if evaluated at E[n];
E[c] = 2.291386042832549 vs 2.2991726419919454.

**TV-02-14 Cold placement and Melee Doughty (E-2.5, E-2.5a)**

- Kunai: CDM0 1.6 (code 205, Q 1.6019536256790161), Primed Target Cracker 1.1, 9 Cold stacks (cold_cd 0.5): CDM =
  3.864102613925934; tier 2 factor 6.728205227851868 (wiki without quantization: 3.86 and 6.72).
- Paris (CDM0 2.0) with cold_cd 0.5: tier 1 2.500488519668579, tier 2 4.000977039337158 (wiki 2.5 / 4.0).
- Melee Doughty: share 119 × 2.2 / 312.8 = 0.8369565217391305; SC_w = 0.2 × (1 + 0.4 + 0.4 × 11) = 1.16;
  raw 9.708695652173915 → doughty 9.7 (wiki +9.7).

**TV-02-15 Conditional base crit multiplier (E-2.5, D-201)**

CDM0 2.0, Prelude of Might X 3.4, threshold 0.5. Instance CC 0.45: x32 = 5.400000095367432, code 691,
Q = 5.399755954742432. Instance CC 0.55: code 256, Q = 2.000488519668579. Compat emulation at CC 1.2 after a failed
roll: ccArg 0.21 < 0.5, so the inspiration adds 3.4 (game profile does not).

**TV-02-16 Worked example of section 5**: body E[c] 2.020322423, head E[c] 6.617547630, head Part × E[c]
54.594767950, body per shot without Acuity 3.838612604.
