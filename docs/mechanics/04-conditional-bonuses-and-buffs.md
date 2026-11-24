Status: reviewed (spec)

# Damage-calculation spec, part 04: conditional bonuses and buffs

This document owns the Condition-Overload family (the CO term of stage ST-4, its per-attack behaviour catalogue,
the CO base and ratio, the multipliers the additive CO form omits, `G_F`), the membership of the "other
multipliers" bucket `G` and its default rule, the condition model behind toggle T-02, the catalogue of every
non-mod-slot damage source (arcanes, ability buffs and Helminth values, target vulnerabilities, auras and Coaction
Drift, archon shards, focus, companion and squad buffs, set bonus effects), the conditional mods, extra hits (E-14)
including Melee Duplicate, stealth and finishers, the riven stat-to-bucket mapping and value formula, and the custom
stats of overview section 6.3. It defines **values, conditions and bucket placement**; the arithmetic of each bucket
belongs to the bucket owner named in [00-overview.md](00-overview.md) section 3.1a.

Related documents: [00-overview.md](00-overview.md) (master; notation, E-1 to E-15, register, toggles),
[01-damage-composition.md](01-damage-composition.md) (base vector, `A`, elements, faction, quantization),
[02-crit-status-multishot.md](02-crit-status-multishot.md) (crit, body part, status chance, proc rolls),
[03-status-effects.md](03-status-effects.md) (status effects, distinct-status count `n`, DoT tick E-13),
[05-enemies.md](05-enemies.md) (enemy side, vulnerability stage, armor strip E-15),
[06-ranged-firing-model.md](06-ranged-firing-model.md) (schedules, charge, falloff, first/last shot),
[07-melee.md](07-melee.md) (stances, combo, heavy attacks, stance CO ratio data),
[08-evaluation-and-outputs.md](08-evaluation-and-outputs.md) (EV and MC algorithms),
[09-build-rules.md](09-build-rules.md) (legality, slots, rank scaling, set counting, riven ranges).

Contents:

- [1. Scope, stages, inputs and outputs](#1-scope-stages-inputs-and-outputs)
- [2. The Condition-Overload family](#2-the-condition-overload-family)
- [3. Multiplier membership: G_F and G](#3-multiplier-membership-g_f-and-g)
- [4. Condition model (T-02)](#4-condition-model-t-02)
- [5. Effects catalogue](#5-effects-catalogue)
- [6. Extra hits (E-14) and Melee Duplicate](#6-extra-hits-e-14-and-melee-duplicate)
- [7. Stealth and finishers](#7-stealth-and-finishers)
- [8. Rivens: stat-to-bucket mapping and value formula](#8-rivens-stat-to-bucket-mapping-and-value-formula)
- [9. Set bonus effects](#9-set-bonus-effects)
- [10. Custom stats (overview 6.3)](#10-custom-stats-overview-63)
- [11. Worked examples](#11-worked-examples)
- [12. Edge cases](#12-edge-cases)
- [13. Inspiration differences](#13-inspiration-differences)
- [14. Open questions](#14-open-questions)
- [15. Test vectors](#15-test-vectors)
- [Review notes](#review-notes)

---

## 1. Scope, stages, inputs and outputs

### 1.1 Scope (overview section 7.1, row `04`)

Owns: CO family (sources, catalogue per attack with behaviour, ratio and exclusions, `G_F` membership, timing,
sequential pellets, stance ratio formula, CO in DoT seeds); `G` membership and the default rule; condition model
(classes of T-02, triggers, stacks, durations, decay); Galvanized and other conditional mods; weapon and warframe
arcanes; ability buffs and Helminth values (Strength at the caster); target vulnerabilities and debuff abilities
(catalogue; stage math in `05`); auras and Coaction Drift; archon shards; focus; companion and squad buffs; extra
hits (E-14) including Melee Duplicate; stealth and finishers; riven stat-to-bucket mapping and the riven value
formula; set bonus effects; custom stats (6.3).

Not: legality of builds (`09`: slots, duplicates, Helminth slot restriction, riven legality and ranges, set counting,
rank scaling), the enemy-side formula (`05`: vulnerability stage, armor strip arithmetic), status-effect values and
the DoT tick formula (`03`), proc rolls and crit arithmetic (`02`), stance data tables (`07`).

Stages: ST-4 (the CO term `C`, `co_behavior`, `co_ratio`, `G_F`), ST-5 (`G` membership; the melee-projectile
weapon-damage bucket), the stealth term inside `c_k` (E-12), E-14. Every other bucket receives **sources** from this
document and applies them with its owner's arithmetic.

### 1.2 Citation keys

As overview section 0.2. Additional keys: `M:Arcane` = wiki `Module:Arcane/data` (cached JSON
`research/external/cache/poc/out/Arcane_data.json`, read 2026-11-23). Pages fetched for this document and added to
`research/external/cache/wiki/`: `W:Kunai_Incarnon_Genesis` and `W:Phenmor` (2026-11-23); in review (2026-11-23):
`W:Prowl`, `W:Crystallize`, `W:Shroud_of_Dynar`, `W:Enraged`, `W:Champion's_Blessing`, `W:Smoke_Shadow`,
`W:Primal_Rage`, `W:Double_Tap`, `W:Deadly_Maneuvers`, `W:Maiming_Strike`, `W:Sea_Snares`, `W:Hakchum`,
`W:Splinter_Storm`, `W:Sentient_Wrath`, `W:Death's_Harvest`, `W:Webbed_Embrace`, `W:Firing_Line`,
`W:Secondary_Surge`, `W:Shadow_Haze`, `W:Ballistic_Bullseye`, `W:Hunter_Synergy`, `W:Merulina_Guardian`,
`W:Reinforced_Bond`, `W:Combo_Fury`, `W:Napalm_Grenades`, `W:Hunter's_Bonesaw`, `W:Dread_Incarnon_Genesis`,
`W:Gotva_Prime`, `W:Sagek_Prime`, `W:Polarize`. `Module:Weapons/data` (cached `Weapons_data_*.json`) is the join key
of the CO catalogue (§2.14).

### 1.3 Inputs

| Symbol | Owner | Use here |
|---|---|---|
| `base[t]`, `base_total` (B'), `base_ref` (B_ref), `incarnon_flat` (X) | `01` | Y (E-4.3); CO base |
| `sum_base_dmg` (A), `sum_faction` (F), `ratio_q[t]` (q_t) | `01` | Y; E-14 |
| `strength` (Str) | T-20 | exalted CO base exceptions, self-cast buffs |
| caster Ability Duration Dur_e | scenario input (proposed toggle, OQ-420) | durations of `dur_scaled` buffs (§4.4) |
| `n_distinct` (n) definition; `stacks[s]` (n_s); `stack_cap[s]` | `03` | C (E-4.1) |
| `proc_weight[s]` (w_s), `sc`, `ms`, `forced_procs[s]` (φ_s), `crit_tier` (k), `cd` (CDM), `part` | `02` | E-4.2, E-14, E-7.1 |
| `combo_mult` M(c), `stance_hit_mult` S_j, stance CO ratio r_in, heavy flags | `07` | E-4.6, conditions of class (b) |
| `falloff(d)` Φ, first/last-shot marks, shot schedule, parent links `parent(a)` (E-6.23) | `06` | G_F, G, condition events, §2.5 defaults |
| data attack names (`AttackName`, `ShotType`, `AttackParentIndex`) | data pipeline (`Module:Weapons/data`) | §2.14 join |
| target health fraction (layered pass), enemy faction, status immunities | `05`, `08` | class (a) conditions |
| toggles T-02 to T-19, T-26 | `00` §6 | everywhere |

### 1.4 Outputs and symbols added

Outputs: `sum_co` (C), `co_behavior`, `co_ratio` (ρ_co), `mult_final_omitted` (G_F), `mult_other` (G),
`extra_hit_frac` (p_xh), `stealth_bonus` (SDB), `vuln` sources (V), and the source lists of every other bucket.

Symbols added (prefixed by domain word as required by overview section 0.3):

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `co_per_status` | b_co | bonus per status | Σ of per-status CO-family bonuses (CO mod, Galvanized, Cedo, Incarnon per-status perks, custom `base_per_status`) |
| `co_per_cold` | b_cold | bonus per Cold stack | Σ of per-Cold-stack bonuses (Secondary Shiver) |
| `co_cond_sum` | C_cond | bonus | Σ of CO-family bonuses gated by a target condition (Kunai, Dread low-health perks, Shattering Frost) |
| `co_ref` | – | enum `self`, `parent`, `uncharged`, `fixed` | what the catalogue's CO base refers to (§2.6) |
| `co_base_abs` | B_co^abs | HP | absolute CO base, not scaled by Str or progenitor (`co_ref = fixed`) |
| `co_base` | B_co | HP | CO base used by the adding form (E-4.3) |
| `co_omit` | – | set | per-attack multipliers moved into G_F for that attack (`combo`) |
| `co_ms_factor` | κ_ms | factor | CO effectiveness on multishot-created projectiles (Tenet Detron 0.65), 1 otherwise |
| `co_dot_mode` | – | enum `seed`, `exclude`, `per_tick` | whether a DoT seeded by this attack carries the CO term (§2.13) |
| `co_tested` | – | bool | false when the attack's behaviour comes from a default of §2.5, not from a catalogue row (UI flag) |
| `co_stance_ratio` | r_in | factor | per stance input CO effectiveness (data from `07`) |
| `co_n_pellet[j]` | n_j | count | distinct statuses seen by pellet j of a shot (E-4.2) |
| `mproj_mode` | – | enum `standard`, `two_bucket`, `single_bucket`, `universal_only` | weapon-damage bucket rule of a melee projectile attack (§3.4) |
| `mproj_melee_dmg` | A_m | bonus | melee-damage bucket on `two_bucket` and `single_bucket` attacks (the members of A that apply to melee) |
| `mproj_weapon_dmg` | A_w | bonus | universal weapon-damage bucket on `two_bucket` attacks |
| `cond_uptime[e]` | u_e | fraction | user uptime of a class (c) or (u) effect (default 1) |
| `cond_stacks[e]` | m_e | count | live stacks of effect e |
| `cond_hits[e]`, `cond_resets[e]` | h_e, b_e | count | hit units and reset-event units since the last reset (Secondary Enervate, E-4.21) |
| `buff_dur[e]` | Dur_e | factor | Ability Duration of the caster of buff e (default 1.0) |
| `buff_str[e]` | Str_e | factor | Ability Strength of the caster of buff e (default: T-20 for self-cast, 1.0 for squad) |
| `xh_type`, `xh_forced` | e_xh | DamageType, bool | type of an extra hit and whether its proc is forced; p_xh,e is the overview's `extra_hit_frac` for source e |
| `dup_chance` | p_dup | fraction | Melee Duplicate chance per tier-1 crit (rank r: 0.25 + 0.15 r) |
| `riven_value` | v | fraction | riven trait value (E-4.18) |
| `riven_coeff`, `riven_layout_mult`, `riven_quality` | b, L, W | –, factor, factor | riven formula terms |
| `vuln_mult[i]`, `vuln_add` | V_mult,i, ΣV_add | bonus | multiplicative and additive-group vulnerability entries (stage in `05`) |
| `fin_class_mult` | M_class | factor | finisher class multiplier (data `07`, E-4.17) |
| `fin_valence_mult` | M_valence | factor | 0.5 for valence weapons (finisher bug), else 1 (E-4.17) |
| `fin_ratio_q[t]` | q'_t | dimensionless | finisher composition with halved elemental and physical bonuses, quantized (E-4.17) |
| `aff_hit` | H_aff | HP | Melee Afflictions averaged seed (E-4.20) |
| `enervate_cc_mean` | CC̄_E | fraction | EV mean Secondary Enervate bonus (E-4.21) |

### 1.5 Value conventions

All values are fractions at **maximum rank** unless a column says otherwise. Per-rank values of mods and arcanes
follow `value(r) = value_max·(r + 1)/(R_max + 1)` (`F:mods` §3, §11), which `09` applies; every effect whose rank
table does not follow it carries `rank_rule` (§4.2) and is listed here (rank r = 0 … 5 for arcanes):

| Effect | Per-rank rule | Source |
|---|---|---|
| Primary / Secondary Merciless reload line | only at rank 5 | `F:mods` §3 |
| Melee Duplicate chance | `0.25 + 0.15·r` | `W:Melee_Duplicate` rank table |
| Primary Compression damage per metre | `0.50 + 0.10·r` (radius factor ×0.2 at every rank) | `W:Primary_Compression` rank table |
| Secondary Fortifier overguard multiplier | `3 + r` (×3 … ×8) | `W:Secondary_Fortifier` rank table |
| Melee Doughty crit multiplier per 10 % Puncture status | `0.5 + 0.1·r` | `W:Melee_Doughty` rank table |
| Secondary Surge cap | `2.00 + 1.00·r` (×3 … ×8); the per-energy rate 0.005 is constant | `W:Secondary_Surge` rank table |
| Secondary Enervate | per-hit 0.10 constant; reset after `r + 1` big crits | `W:Secondary_Enervate` rank table |
| Primary Plated Round | value constant; duration 3 / 4 / 5 / 6 / 8 / 10 s | `W:Primary_Plated_Round` rank table |
| Primary Frostbite, Arcane Bellicose, Melee Afflictions | proportional (`(r + 1)`): Frostbite needs 40 procs at every rank; Bellicose cap `12·(r + 1)` % | rank tables |

"× Str" means the listed value is multiplied by the **caster's** Ability Strength (§4.4). "UNVERIFIED" follows
overview section 0.1.

---

## 2. The Condition-Overload family

### 2.1 Sources

All entries are one internal effect: they add with each other into `C` (`W:Condition_Overload_(Mechanic)`
§Sources, "internally considered the same effect"; `F:addmul` §1.1). Condition class per §4.1.

| Source (slot) | Contribution to C | Stacks, duration | Class | Source |
|---|---|---|---|---|
| Condition Overload (melee mod) | `0.80 · n` | – | (a) | `W:Condition_Overload`; 1.20 → 0.80 in U30.5; exponential before U26 |
| Galvanized Aptitude (rifle) | `0.40 · m · n` | m ≤ 2 stacks on kill, 20 s, lose one stack per timeout | (a) × (c) | `W:Galvanized_Aptitude`, `F:mods` §6.1 |
| Galvanized Savvy (shotgun) | `0.40 · m · n` | m ≤ 2, 20 s | (a) × (c) | `W:Galvanized_Savvy` |
| Galvanized Shot (pistol) | `0.40 · m · n` | m ≤ **3**, **14 s** | (a) × (c) | `W:Galvanized_Shot` |
| Secondary Shiver (secondary arcane; also archgun bottom slot) | `0.45 · n_COL` | per Cold stack on the target, up to the Cold cap | (a) | `W:Secondary_Shiver`; `M:Arcane` |
| Shattering Frost (Koumei decree) | 0.80 / 1.60 / 2.40 by decree level | target "affected by Cold Status" (Decision: any live Cold stack; UNVERIFIED, OQ-402) | (a) | `W:Decree` lines 71-75 ("Deal +80/160/240% damage to enemies affected by Cold Status"; "Has the same behavior as Secondary Shiver"); the CO Mechanic source list (line 39) says "+80% vs Frozen, upgradeable to +240%" (alternative, OQ-402) |
| Cedo, Cedo Prime (weapon innate) | `0.60 · n` | – | (a) | `W:Cedo` §Notes |
| Dual Toxocyst Incarnon, Evo 2 "Carnage Reign" | `0.33 · n` | – | (a) | CO Mechanic §Sources |
| Lato Incarnon, Evo 4 "Carnage Reign" | `0.40 · n` | – | (a) | idem |
| Latron Incarnon, Evo 2 "Swift Punishment" | `0.30 · n` | – | (a) | idem |
| Angstrum / Despair / Zylok / Soma Incarnon "Fatal Affliction" | `0.40 · n` | – | (a) | idem |
| Kunai / Mk1-Kunai Incarnon, Evo 2 "Swift Conclusion" | **2.00** while target health < 50 % (plus flat X = 70 / 80) | – | (a) | `W:Kunai_Incarnon_Genesis` (perk table: "+200 %"; "additive with Galvanized Shot in both forms"). The CO Mechanic source list says +100 %; the per-perk page wins (overview §1.4 rule 3) |
| Dread Incarnon, Evo 2 "Hitman's Opportunity" | 1.00 while target health < 50 % (plus flat X = 70) | – | (a) | CO Mechanic §Sources; `W:Dread_Incarnon_Genesis` (fetched 2026-11-23): "Increase Base Damage by +70", "+100% Damage to enemies below half Health", which "ignores the base damage increase from the same perk, the 2x damage from the charged shot (Primary Fire only) and Galvanized Aptitude's damage bonus" (consistent with an adding CO-family term on the charged shot, ρ_co = 0.5, flat excluded) |
| Custom stat `base_per_status` (T-26) | `x · n` | – | (a) | overview §6.3 |

Not in the family (ordinary bonuses in their own bucket, catalogued in §5): Galvanized Chamber, Diffusion, Hell,
Scope, Crosshairs, Acceleration, Steel, Elementalist, Reflex; Lavos Catalyze (exponential, radial, ability damage)
(`F:addmul` §1.1). Archguns receive only Secondary Shiver and Shattering Frost (`W:Condition_Overload_(Mechanic)`
§Archguns, "as of U43").

### 2.2 The sum C

```text
E-4.1  CO-family sum for one damage instance of attack a
  C      = b_co · n  +  b_cold · n_COL  +  C_cond
  b_co   = Σ per-status entries of §2.1 that apply to a's weapon        (0 on archguns)
  b_cold = 0.45 per equipped Secondary Shiver (rank 5)
  n      = distinct statuses on the target before this instance (definition 03; count source T-05)
  n_COL  = live Cold stacks on the target before this instance, 0 .. stack_cap[COL]; Cold procs of earlier pellets of
           the same shot count (the CO rule of §2.3, not the sibling-debuff rule of CD-04; Decision, UNVERIFIED, OQ-433)
  C_cond = Σ condition-gated entries (Swift Conclusion 2.00, Hitman's Opportunity 1.00, Shattering Frost) whose
           condition holds before this instance
  Shattering Frost reading (Decision, OQ-402): one fixed term s_SF ∈ {0.80, 1.60, 2.40} while n_COL ≥ 1
           (alternatives: only while Frozen; s_SF per Cold stack like Secondary Shiver)
```

Per T-02 (§4.3):

| T-02 | n | n_COL | C_cond |
|---|---|---|---|
| `none` | – | – | C = 0 (every class (a) entry off) |
| `all_max` | per T-05 (`expected`: target state, `weapon_types`, `manual(N)`) | target state (expected in EV, live in MC) | every condition true |
| `simulated` | per T-05 | target state | condition from the target state; health thresholds only in the `layered` pass (§4.3) |

- `n_COL` always comes from the target state (Decision; D-060 "per stack, expected or simulated"). The inspiration
  uses the constant `0.45 × 10 = 4.5` (JS L9365). Register row D-060 has Toggle `—`, so no compatibility switch
  exists and oracle tests must avoid Secondary Shiver; this document proposes changing D-060's Toggle to `Compat`
  with `n_COL = 10` in the compatibility profile (OQ-434). Until the manager changes the register, the engine has no
  such switch.
- Galvanized stack count m follows the condition model (class (c) on-kill, default max under `all_max` and under
  `simulated` with default uptime; §4.3).
- `C` is a scalar per instance; it never changes the quantized composition `q_t` (overview E-11 note, OQ-002).
- The cap on n is the 03 definition (16 or 17 types, `F:addmul` §1.2; OQ-408).

Source: `W:Condition_Overload_(Mechanic)` §Sources, §What Counts As A Status Effect; `W:Secondary_Shiver` §Notes
("Bonus is additive to damage per status type from Galvanized Shot").

### 2.3 Timing and sequential pellets

Rules (CD-04 applied to C; `F:addmul` §1.2):

1. A status applied by an instance does not count for that instance (nor for the DoT it seeds).
2. Pellets of one shot are **sequential**: pellet j sees the statuses applied by pellets 0 .. j − 1 of the same shot
   (`W:Galvanized_Aptitude` §Notes worked example `265 + 345 + 425 = 1035`). Gunblade pellets likewise
   (`W:Condition_Overload` §Notes). Pellets do **not** see each other's debuffs (CD-04).
3. Beams: one merged instance per tick (CD-10); n is read before the tick.
4. Melee Duplicate's copy is a later instance: it sees the original hit's procs (CD-04 order) (Decision, OQ-419).
5. Extra hits do not recompute C: E-14 starts from the parent's finished damage, which already contains its CO term.
6. Before U31.1.6 multishot multiplied the CO bonus per pellet; fixed (`W:Galvanized_Shot` patch history). The
   engine has no per-pellet multiplication.

```text
E-4.2  Expected n for pellet j of a shot (EV evaluator; exact under E-9 and CD-03)
  p_s       = 1 − (1 − w_s)^floor(SC) · (1 − frac(SC)·w_s) · [φ_s = 0] · Π_(forced sources x of s) (1 − P_x)
              probability that one pellet (with its extra hits) applies ≥ 1 proc of s. Forced sources: Hunter
              Munitions, Internal Bleeding, Cryogenic, stance procs (as defined in 02), and the procs of the pellet's
              own extra hits, which land before the next pellet (CD-04 order, §6.2): a forced-proc extra hit of type
              e_xh contributes P_x = 1 to s = e_xh; Xata's Whisper contributes P_x = min(1, SC) to s = VOI
  Encumber  (02 E-2.17, once per shot): its random status is attributed to pellet 0 (Decision), i.e. pellets
              j ≥ 1 see it with probability P_enc/13 per eligible type
  E[n_j]    = E[n_pre] + Σ_(s countable, not immune) P(s absent before the shot) · (1 − (1 − p_s)^j · (1 − e_s)^[j ≥ 1])
              e_s = Encumber's probability of applying s (0 without Encumber)
  E[Σ_j C_j] = Σ_(j < floor(MS)) C(E[n_j]) + frac(MS) · C(E[n_floor(MS)])          (C is linear in n)
  n_COL for pellet j (Secondary Shiver, OQ-433):
  E[n_COL,j] = E[min(cap_COL, N_pre + K_j)],  N_pre ~ the Cold live-proc model of overview 4.2 (Poisson, mean μ_COL),
              K_j ~ Poisson(j·c_COL),  c_COL = SC·w_COL + φ_COL  (expected Cold procs per earlier pellet, incl. forced)
              (the sum of the two Poisson terms is Poisson(μ_COL + j·c_COL); the per-pellet count is approximated
              as Poisson, the same approximation as overview 4.2)
```

`E[n_pre]` and `P(s absent)` come from the stack model of overview section 4.2 (`1 − P(n_s ≥ 1)`). The cap on n is
ignored inside E-4.2 (it only binds above 16 distinct types). `08` implements it; MC needs no formula (it reads the
live state per pellet, extra-hit procs and Encumber included).

### 2.4 The three behaviours (refines overview E-11)

```text
E-4.3  Base scalar Y of one instance of attack a (ST-4)
  none        : Y = B'·(1 + A)·G_F
  multiplying : Y = B'·(1 + A)·(1 + C)·G_F                                 (overview E-11, verbatim)
  adding      : Y = B'·(1 + A)·G_F + B_co·C                                (overview E-11 with B_co for ρ_co·B_ref)
  B_co = B_co^abs                    if co_ref = fixed     (not scaled by Str, progenitor or flat)
       = ρ_co·B_ref                  otherwise             (B_ref includes the progenitor, excludes the Incarnon flat X)
  G_F  = Π of the members of §3.1 that apply to the instance, times every multiplier listed in co_omit(a)

  Proposed amendments of overview E-11 (OQ-428; not implemented until the manager changes E-11 and 01 §4.14):
  multiplying : Y = B'·(1 + A)·(1 + ρ_co·C)·G_F       ρ_co = 1 on every multiplying row except Tenet Agendus waves (0.2)
  adding      : Y = B'·(1 + A)·G_F + B_co·C·κ         κ = co_ms_factor for pellets j ≥ 1 of a shot (Tenet Detron 0.65),
                                                       1 for pellet 0 and for every other attack
```

`B_co^abs` for the `fixed` rows is not an amendment: E-11 names the catalogue ratio, and a fixed base is the
catalogue's statement that the ratio is relative to a constant (Noctua, Artemis Bow, Sporelacer). The two proposed
amendments are needed for rows 114 (κ_ms) and 165-166 (ρ_co = 0.2 on a multiplying attack); until they are accepted
those rows evaluate with E-11 as written (Tenet Agendus waves multiply by `(1 + C)`, Tenet Detron's multishot pellets
get the full CO term), which overstates both (TV-04-34, TV-04-35).

What the adding form omits from the CO term, exhaustively:

| Omitted | How the engine does it | Source |
|---|---|---|
| the base bucket A | CO term is `B_co·C`, no `(1 + A)` | CO Mechanic §Math ("omits +X% Damage in the recalculation") |
| every G_F member (§3.1) | G_F multiplies only the first term | idem, list of "ignored" multipliers |
| the Incarnon flat X | `B_ref` instead of `B'` | idem ("Base Damage increases from Incarnon Genesis Evolutions"); `W:Braton_Incarnon_Genesis` §Known Bugs |
| bow charge scaling | `ρ_co` from the uncharged (or partially charged) reference value (§2.7); the charge itself is in the data attack's damage (`06`), so no separate charge factor exists | CO Mechanic catalogue rows "only applies to base (uncharged) damage" |
| some stance multipliers on multi-hit inputs | r_in (E-4.6) | CO Mechanic §Stances |
| melee combo on `no_combo` attacks | `co_omit = {combo}` | catalogue rows Argo & Vel, Arum Spinosa, Quassus Prime |
| part of the CO on multishot-created projectiles | κ_ms (Tenet Detron 0.65), proposed amendment of E-11 (OQ-428) | catalogue row Tenet Detron |

What the adding form does **not** omit: the elemental and physical composition `q_t` (stated by the melee formula
`Total = Base × [1 + DamageMods + CO × n] × (1 + ElementalMods)`, `W:Condition_Overload` §Notes, and consistent with
the Zhuge Prime quantization note, EC-04-16), faction `(1 + F)`, every `G` member (beam ramp-up included: the page
lists range falloff as omitted "but not Beam ramp-up"), body part, crit and the enemy side. That faction, G, body
part, crit and the enemy side multiply the CO term is the framing of every wiki example but is stated on no page
(Decision, overview OQ-001).

Order of operations and rounding: composition is quantized first (E-3, CD-02), Y is computed in f64 without
rounding, then `W_t = q_t·Y·(1 + F)·G·Part·c_k` (E-12). `C` does not enter the quantization scale (OQ-002).
`(1 + A)` may be negative only through negative mods; the clamp of the base bucket belongs to `01`; the engine clamps
`Y ≥ 0` after adding the CO term (Decision; the inspiration clamps each type at 0, JS L9414).

Multiplying uses `B'` including the Incarnon flat (Kunai Incarnon: "CO-bonus DOES use base damage increase
Evolution", catalogue) and `G_F` multiplies the whole product, so G_F and G are mathematically interchangeable for
`multiplying` and `none` attacks; the distinction matters only for `adding`.

Sources: `W:Condition_Overload_(Mechanic)` §Application Behaviors, §Math (section comment "Mechanics below
confirmed as of 2025-11-03"); `F:addmul` §1.4-§1.7; `W:Longbow_Sharpshot` §Notes (`[(1 + Serration)(1 + Longbow
Sharpshot)] + Galvanized Aptitude`); `W:Eclipse` (adding CO not multiplied).

### 2.5 Per-attack data and the default rule

Every attack of every weapon carries `co_behavior`, `co_ratio` (ρ_co), `co_ref`, `co_base_abs`, `co_omit`,
`co_ms_factor`, `co_dot_mode`, `mproj_mode`. The data pipeline fills them from the catalogue (§2.14); attacks not in
the catalogue receive the first matching default:

| # | Attack kind (data: `ShotType`, parent link `parent(a)` of `06` E-6.23, `07` attack type) | Default | Source |
|---|---|---|---|
| D1 | named in the `Data attack` column of §2.14 (resolution rule there) | the row | catalogue |
| D2 | melee slam, heavy slam, glaive heavy-attack explosion, any radial melee explosion | `none` | `W:Condition_Overload` §Notes; `F:melee` §6 |
| D3 | radial child (`ShotType = AoE`) of a **projectile** parent (rocket explosion, grenade, embedded explosion) | `none` | CO Mechanic §Mechanics ("Projectile Explosion Radius") |
| D4 | radial child of a **hitscan** parent (hitscan explosion) | `adding`, `direct_target_only`, `co_ref = parent`, ρ_co = B_ref(parent)/B_ref(a) (UNVERIFIED for uncatalogued attacks; every catalogued hitscan-AoE row follows it, OQ-429) | CO Mechanic §Mechanics ("Hitscan Explosion ... only on directly-hit target"), catalogue notes "CO-bonus scales off hitscan damage" |
| D4a | beam AoE (no separate data attack today; Glaxion Vandal row 45) | `adding`, ρ_co = 1 relative to the beam, on **every** target in the AoE (no `direct_target_only`) | CO Mechanic §Mechanics lists "Beam AoE (e.g., Glaxion Vandal)" among forms that apply CO, with no direct-target clause; row 45 has none |
| D5 | embedded cloud: the directly embedded target | `adding`, ρ_co = 1; other targets `none` | CO Mechanic §Mechanics (Pox, Torid) |
| D5a | non-radial child projectile (`ShotType = Projectile`) whose `parent(a)` is a projectile attack (cluster bomblets, child bombs) | `adding`, `co_ref = parent`, ρ_co = B_listed(parent)/B_listed(a), `co_tested = false` (UNVERIFIED; OQ-409) | CO Mechanic §Application Behaviors, behaviour #2: "Child projectiles that spawn from a parent projectile (CO-bonus uses parent projectile damage …): Kuva Bramma's Cluster Bomb Contact, Kulstar's Cluster Bombs"; catalogued children follow it (Kulstar 200/75, Zarr 25/15, Kuva Zarr 50/15) except Phantasma (row 63, `multiplying`) |
| D5b | standalone AoE attack with no parent (06 EC-06-73: Glory, Talons, Neutralizer grenade) and data attacks with `ShotType = DoT` (lingering damage such as Tysis "Corrosive DoT", Cryotra "Cold Field") | `none` | CO Mechanic §Mechanics (radial explosion and cloud radius do not apply CO); Tysis "Does not affect DoT"; Cryotra row note "CO does not apply to the AoE" |
| D6 | everything else (hitscan, projectile, beam, melee hit, thrown) | `adding`, ρ_co = 1, `co_tested = false` | CO Mechanic §Primaries: "Anything not listed should be assumed to be Additive with +100% bonus" |

The wiki's observation that projectile attacks *tend* to be `multiplying` is **not** a default: the page instructs
adding for unlisted attacks, and the listed untested weapons (`F:addmul` §1.9 "Not tested yet") stay at D6 with
`co_tested = false` so the UI can flag them (OQ-409). D5a is a stated mechanism, not a guess about one weapon, so it
precedes D6. Archgun attacks use the same rules, but `b_co = 0` (§2.1). Partial and quick shots are separate data
attacks (`06`: Lanka "Partially Charged Shot", Cinta "Quick Shot"); uncatalogued ones take D6.

### 2.6 CO base reference

| `co_ref` | Meaning | ρ_co in the catalogue | Engine |
|---|---|---|---|
| `self` | CO scales off the attack's own base | `B_co / B_listed(a)` | `B_co = ρ_co·B_ref(a)` |
| `uncharged` | CO scales off the uncharged (or partially charged) value of the same attack | as listed (bows 0.5) | as `self` |
| `parent` | CO scales off the parent projectile or hitscan attack (child projectiles incl. D5a and Kuva Bramma row 202, hitscan AoE, Pox cloud, Mutalist Cernos cloud spawned by the charged shot: `parent_variant`) | listed value of the named variant | for a weapon family row ("values listed for Braton Prime"), the pipeline recomputes `ρ_co = B_listed(parent of this variant)/B_listed(a of this variant)` |
| `fixed` | CO scales off a constant | – | `B_co = B_co^abs` (Noctua 200 / 2500 / 200, Artemis Bow 75 per arrow, Sporelacer secondary 57) |

Progenitor: B_ref includes the valence bonus V (fixed for hitscan and melee in U31.1.7, for projectiles in
U31.2/U31.5; "never broken on multiplicative cases", CO Mechanic patch history), so `ρ_co·B_ref` scales with it.
Strength: for exalted weapons whose base scales with Str (`m_str = Str`), B_ref already contains Str; the
`fixed` rows are exactly the documented cases where CO does **not** scale with Str. Incarnon flat: excluded from
`B_ref` (adding), included in `B'` (multiplying).

### 2.7 Bows and charge weapons

- Fully charged bows deal 2× their uncharged damage; adding-CO uses the uncharged value: ρ_co = 0.5 (Cernos family,
  Daikyu Prime, Paris family, Dread charged, Kuva Drakgoon charged) (catalogue).
- Other references the page labels **uncharged** ("CO-bonus only applies to base (uncharged) damage" or "Uses
  uncharged damage value"): Evensong 325/500 = 0.65, Ballistica 0.25, Ballistica Prime 0.5, Rakta Ballistica 0.25.
  References the page labels **partially charged** ("only applies to base (partially charged) damage"): Lanka
  200/525 = 0.381, Miter 200/500 = 0.4. Drakgoon 400/700 = 0.5714 carries no note on the page (catalogue rows 33,
  37, 50, 51, 97, 98, 100).
- Charge level below full: partial and quick shots are separate data attacks with their own damage and charge time
  (`06` §5, "no interpolation", OQ-607), so the engine never scales a charged attack by a charge fraction. Bow charge
  scaling, a G_F member (§3.1), is therefore 1 for every data attack: the charge is already in the data attack's
  damage (`06`). Each catalogued charged attack keeps its uncharged-reference ρ_co; each partial or quick shot
  attack has its own row or a §2.5 default (Decision; whether a partially charged shot's CO base also follows the
  uncharged value is UNVERIFIED, OQ-410).
- Uncharged bow shots are separate attacks at D6 (`adding`, ρ_co = 1; "Paris's Uncharged Shot" is the wiki's example
  of intended behaviour).
- Charged `multiplying` projectiles (Cinta, Nataruk, Velocitus, Corvas Prime) carry the charge in
  their listed damage and multiply C fully. Corvas (atmosphere) charged hitscan "respects Charge damage increase"
  (adding, ρ_co = 1).
- Longbow Sharpshot (+3.00, next shot after a weak-point hit) is a G_F member; on an adding bow the CO term is
  neither multiplied by it nor by the bow's charge (`F:addmul` §1.8 example 8; the combination with ρ_co = 0.5 is
  not shown by `W:Longbow_Sharpshot`, UNVERIFIED).
- Artemis Bow: `fixed`, 75 per arrow, not scaled by Strength, and CO does not apply while Concentrated Arrow is
  equipped (catalogue; fixed in U36 to benefit from Galvanized Aptitude at all).

### 2.8 Child projectiles

Child projectiles that the wiki lists use the parent projectile's damage: Kulstar cluster bombs 75 → 200
(ρ_co = 2.6667; the table's "257 %" is an arithmetic slip, the absolute values are used, OQ-411), Sporelacer
(secondary) bomblet 9 → 57, Zarr child bomb 15 → 25, Kuva Zarr child bomb 15 → 50, Mutalist Cernos charged toxin
cloud 5 → 205 (uncharged arrow). The wiki's prose also names the Kuva Bramma cluster contact (behaviour #2) without a
table row; the general rule D5a (§2.5) covers it and row 202 records it: data "Cluster Bomb Contact" 49 Impact, parent
"Charged Shot" 187 Impact (06 E-6.23 override), ρ_co = 187/49 = 3.8163 (computed from the data, UNVERIFIED in game,
OQ-409; TV-04-31). Phantasma child bombs are `multiplying` at ratio 1 (the documented exception to D5a).

### 2.9 Hitscan AoE (hitscan explosions)

Ambassador alt-fire, Braton-family and Burston-family Incarnon radial attacks, Ferrox, Tenet Ferrox, Opticor,
Opticor Vandal, Trumna, Zylok Incarnon radial, Mausolon main and alt: `adding` with `co_ref = parent`; the radial
instance receives CO **only on the directly hit target** (`direct_target_only`; every one of these rows carries the
page note "Radial hit only receives CO bonus on target directly hit"), scaled from the hitscan damage, and the radial
attack is not multiplied by multishot (`06` owns the instance count). Every other target in the radius gets `none`
(multi-target mode only). Beam AoE (Glaxion Vandal, row 45) is different: the page lists "Beam AoE" among the forms
that apply CO with no direct-target clause and row 45 has no such note, so every target in a beam AoE gets the CO term
(D4a; it matters only in multi-target mode, and the data has no separate beam-AoE attack). Primary Compression on these attacks is **additive with the base
bucket** (§3.1, `W:Primary_Compression` table "Adds"), so it does not touch the CO term either way.

### 2.10 Attacks where CO does nothing

`none` rows: Coda Bassocyst alt-fire, Evensong charged radial, Proboscis Cernos radial and tendrils, Tenet Tetra
alt-fire AoE, Vadarya Prime lightning strikes, Zhuge Prime embedded explosion, Akarius Prime detonation, Azima turret
bullets, Balefire Charger uncharged and charged, Castanas and Sancti Castanas detonations, Kompressa Prime radial,
Sonicor impact and wave, Stug blob impact and Incarnon form, Scyotid alt-fire grab; Latron Incarnon's AoE ("in any
way"); Ferrox embedded alt-fire (prose); Exodia Contagion embed DoT and explosion; Cryotra cloud; Arbucep 228-damage
AoE. Plus the defaults D2 and D3. With T-04 on, `none` stays `none` (overview T-04).

### 2.11 Archguns

Only Secondary Shiver (archgun bottom arcane slot) and Shattering Frost apply. Behaviours: Arbucep direct (all six
projectiles), Kuva Ayanga direct, Corvas Prime uncharged and charged, Grattler, Larkspur Prime alt-fire, Mandonel
uncharged, Velocitus uncharged and charged are `multiplying`; Corvas (atmosphere), Fluctus and Mandonel charged are
`adding` at 1.0; Mausolon radial attacks are hitscan AoE (§2.9). Arcane class restrictions on archguns: §5.4.

### 2.12 Melee: stances, heavy attacks, melee projectiles

```text
E-4.6  Stance hit j of input `in` (adding attacks only)
  Y_j = B'·S_j·(1 + A)·G_F + r_in·S_j·B_co·C          S_j is removed from G for this instance
  multiplying, none: S_j stays in G (no r_in)
```

- r_in is the CO Mechanic §Stances table value of that stance and input (Neutral, Forward, Forward Block, Block,
  Slide, Ground Slam Strike, Ground Finisher, Wall, Air, Heavy), as a fraction; blank cells and stanceless combos use
  1.0. `07` owns the imported table (63 stance rows, Butcher's Revelry … Hysteria; range 0.25 Fracturing Wind block
  to 2.33 Mountain's Edge heavy;
  Exalted Blade 1.22-1.30 on every tested input; High Noon and Bullet Dance 1.00 everywhere).
- r_in applies to every hit of the input (Decision: the table gives one value per input; per-hit structure is not
  published; UNVERIFIED, OQ-401).
- The heavy-attack type multiplier and the combo multiplier M(c) are `G` members (`07`); for attacks with
  `co_omit = {combo}` M(c) moves into G_F.
- CO applies to the thrown glaive (U28.0.6) but not to its radial explosion (`F:melee` §10); gunblade pellets are
  sequential (§2.3).
- Melee projectiles: §3.4.

### 2.13 CO in DoT seeds

```text
E-4.7  Seed of a DoT created by an instance of attack a (input to E-13, owner 03)
  seed = Y·(1 + F)·G                   co_dot_mode = seed (default): Y of E-4.3 with the same behaviour, C and G_F
  seed = Y|_(C = 0)·(1 + F)·G          co_dot_mode = exclude (Tysis "Does not affect DoT"; Stinger embedded DoT)
  per_tick: the attack's damage ticks are direct instances (Pox DoT cloud): each tick is evaluated with E-4.3 and
            the C of the target state at the tick
```

- CO is part of the modded base of procs, including Slash, on melee (`W:Condition_Overload` §Notes) and on guns
  (consequence of the same recalculation; the wiki states it for melee only).
- The seed uses the same Y as the hit, so G_F members stay out of the CO part of the seed on adding attacks
  (Decision; UNVERIFIED, OQ-403; `F:buffs` Q4).
- Radial Blast, Electricity and Gas procs carry the initial target's CO-boosted value to other targets
  (multi-target only).
- An extra hit's DoT uses the extra hit as seed (E-14), which already contains the parent's CO term.
- Lingering damage that the data stores as its own attack is an attack, not a `co_dot_mode` of its parent: Tysis
  "Corrosive DoT" (`ShotType = DoT`, child of "Normal Attack") is `none` by D5b, which is the same result as row
  145's `no_dot_co`; Pox "Poison Cloud" (`ShotType = AoE`, child of "Spore Impact") is row 130, `adding` with
  `co_ref = parent` (ρ_co 2.5) and `per_tick`: each cloud tick on the embedded target is an instance evaluated with
  E-4.3 and the C of the target state at that tick. `co_dot_mode` applies only to status DoTs (E-13) seeded by an
  attack's hits.

### 2.14 Per-attack catalogue (ingestable)

Source of rows 1-201: `W:Condition_Overload_(Mechanic)` §Attack Catalog (cached 2026-11-23, page last edited
2026-11-23; community-tested, spreadsheet references on the page). Rows were extracted mechanically from the cached
wikitext and checked by hand; in review a script compared all 201 rows against the cached table again (behaviour,
values, flags). Row 202 comes from the page's prose (§Application Behaviors, behaviour #2) and the §2.5 rule D5a.
Columns:

- `#` stable row id (catalogue row, not a weapon id); `Class`; `Weapon(s)` and `Attack` exactly as the wiki names
  them (wiki spelling kept for traceability; row 172 corrects the wiki typo "Atomsphere").
- `Data attack`: the exact `AttackName` of wiki `Module:Weapons/data` (cached `Weapons_data_*.json`) that the row
  applies to; this column, not the wiki attack name, is the join key. `=` means the wiki attack name is the data name.
  Several names separated by `;` mean the row applies to each of them. `Weapon: name` gives a name for one variant
  only, or a data weapon name that differs from the wiki name (Catchmoon, Sporelacer and Tombfinger kitguns are the
  data weapons `Catchmoon (Primary)`, `Sporelacer (Primary)`, `Tombfinger (Primary)`; the wiki's grip suffix
  "(Tremor)", "(Brash)" names the grip the page measured, and the row applies to that weapon with any grip,
  Decision). `(parent X)` names the parent the ratio refers to (06 E-6.23 resolves the link). `parent_variant X`
  (rows 27, 28): the row applies to child instances spawned by the parent attack X only; the Mutalist Cernos data has
  one "Toxin Cloud" attack, which is `multiplying` at ratio 1 when spawned by the uncharged shot and `adding` at ratio
  41 when spawned by the charged shot. `–` with flag `no_data_attack`: the data has no such attack (Glaxion Vandal
  beam AoE, Zenith disk bonk, Azima turret bullets, Scyotid grab, Destreza spectral rapiers, Noctua Wordwarden,
  Exalted Umbra Blade waves, Exodia Contagion); the row is kept for an attack another document defines (`07` for
  waves, Exodia Contagion and stance-mapped projectiles, which carry the wiki attack name as their id) and is
  otherwise not modelled.
- Family rows: `A/B/Prime/Vandal/Mk1/Prisma`: the tokens `Mk1`, `Prime`, `Vandal`, `Prisma` expand to `Mk1-A`,
  `A Prime`, `A Vandal`, `Prisma A`; any other token is a full weapon name.
- `Behaviour`: `adding`, `multiplying`, `none`.
- `B listed`, `B_co wiki`: the page's unmodded damage and "CO damage bonus at +100 %" (HP). Incarnon rows show both
  values with and without the flat ("100 or 124").
- `ρ_co` = `B_co wiki / B listed` (first value for Incarnon rows; archgun rows give only the ratio). For Incarnon
  rows ρ_co = 1 relative to B_ref (the flat is excluded by E-4.3). For `fixed` rows the engine ignores ρ_co.
  ρ_co is a ratio and is applied to the data attack's own B_ref. Where the page's `B listed` differs from the data
  attack's total base (pellet or arrow count: Drakgoon ×10, Cernos Prime ×3, Bubonico and Bronco Prime Incarnon ×7;
  heavy-attack multiplier: Stropha 2800 vs 700, Argo & Vel 600 vs 200; data revisions: Braton family, Tenet Agendus
  2880), the ratio is kept unchanged (Decision: the page measured the displayed damage, whose extra factor the engine
  carries in `G` or in the pellet count, so the ratio transfers); the data-build report lists every such row.
- `co_ref`, `B_co^abs`, `Flags`: §2.6; flags: `direct_target_only` (radial; CO only on the directly hit target),
  `flat_excluded` / `flat_included` (documentation of E-4.3), `no_combo` (`co_omit = {combo}`),
  `ms_proj_factor=0.65` (κ_ms), `no_dot_co` / `no_embedded_dot_co` (`co_dot_mode = exclude`), `dot_recalc_per_tick`
  (`per_tick`), `wdmg_mode=…` (`mproj_mode`), `off_with=…` (CO is 0 while that augment is equipped),
  `swift_conclusion_in_C` (§2.1), `child_aoe=none` / `aoe_child=none` (the radial child is `none`),
  `mag_mult_in_G` (Tenet Plinx alt-fire magazine multiplier is a G member and multiplies CO), `no_data_attack`,
  `prose_row` (not in the page's table), `note_*` (no engine effect), `ui_5x_UNVERIFIED`, `table_pct_257_vs_267`
  (OQ-411).

**Resolution rule (data pipeline).** Every row MUST resolve: each expanded variant name must be a data weapon and
each `Data attack` name must be one of its `AttackName`s, unless the row carries `no_data_attack`. An unresolved row
fails the data build; it never falls back to a §2.5 default. Two rows that resolve to the same data attack must agree
on behaviour and ρ_co (rows 165 and 166 do) or carry `parent_variant`. A golden test asserts that all 202 rows
resolve against the pinned data snapshot and that no data attack is matched by two disagreeing rows. Data attacks
that no row names receive the §2.5 defaults, with `co_tested = false`.

Vulcax (row 197): the page measures normal attack plus AoE together (500 + 100 → 1000); the engine puts the whole CO
base on the hitscan normal attack (ρ_co = 2.0 of 500) and `none` on the AoE child, which reproduces the measured total
on the target (Decision). Arbucep (row 170): the six projectiles of the burst are the data attacks "1st Attack" …
"6th Attack"; their children "Nth Attack Radial Attack" (the 228-damage AoE) are `none` by D3. Stinger (row 196) is
`adding` at ratio 1 with its embedded DoT excluded (`co_dot_mode = exclude`).

| # | Class | Weapon(s) | Attack | Data attack | Behaviour | B listed | B_co wiki | ρ_co | co_ref | B_co^abs | Flags |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | Primary | Acceltra | Normal Attack | Rocket Impact | adding | 35 | 26 | 0.7429 | self | – | – |
| 2 | Primary | Aeolak | Normal Attack | = | multiplying | 60 | 60 | 1 | self | – | – |
| 3 | Primary | Aeolak | Alt-fire | Alt-Fire | multiplying | 97 | 97 | 1 | self | – | – |
| 4 | Primary | Alternox | Normal Attack | = | multiplying | 62 | 62 | 1 | self | – | – |
| 5 | Primary | Alternox Prime | Normal Attack | = | multiplying | 75 | 75 | 1 | self | – | – |
| 6 | Primary | Ambassador | Alt-fire Hitscan Radial Attack | Charged AoE (parent Charge) | adding | 800 | 600 | 0.75 | parent | – | direct_target_only |
| 7 | Primary | Arca Plasmor | Normal Attack | = | multiplying | 600 | 600 | 1 | self | – | – |
| 8 | Primary | Tenet Arca Plasmor | Normal Attack | = | multiplying | 760 | 760 | 1 | self | – | – |
| 9 | Primary | Basmu | Main-fire | Auto | multiplying | 39 | 39 | 1 | self | – | – |
| 10 | Primary | Battacor | Main-fire | Normal Attack | multiplying | 66 | 66 | 1 | self | – | – |
| 11 | Primary | Braton/Mk1/Prime/Vandal | Incarnon Form Radial Attack | Incarnon Form AoE (parent Incarnon Form) | adding | 74 | 70 | 0.9459 | parent | – | direct_target_only |
| 12 | Primary | Bubonico | Main-fire | Auto | multiplying | 287 | 287 | 1 | self | – | – |
| 13 | Primary | Bubonico | Alt-fire | Burst | multiplying | 9 | 9 | 1 | self | – | – |
| 14 | Primary | Burston/Burston Prime | Incarnon Form Radial Attack | = | adding | 55 | 13 | 0.2364 | parent | – | direct_target_only |
| 15 | Primary | Buzlok | Normal Attack | = | multiplying | 60 | 60 | 1 | self | – | – |
| 16 | Primary | Buzlok | Alt-fire | Beacon | multiplying | 3 | 3 | 1 | self | – | – |
| 17 | Primary | Catchmoon (Primary) (Tremor) | Normal Attack | Catchmoon (Primary): Normal Attack | multiplying | 216 | 216 | 1 | self | – | – |
| 18 | Primary | Cedo/Cedo Prime | Alt-fire | Alt-Fire Glaive | multiplying | 20 | 20 | 1 | self | – | – |
| 19 | Primary | Cernos | Charged Attack | Charged Shot | adding | 380 | 190 | 0.5 | uncharged | – | – |
| 20 | Primary | Cernos Prime | Charged Attack | Charged Horizontal/Vertical Shot | adding | 552 | 276 | 0.5 | uncharged | – | – |
| 21 | Primary | Coda Bassocyst | Normal Attack | = | multiplying | 808 | 808 | 1 | self | – | – |
| 22 | Primary | Coda Bassocyst | Alt-fire | Alt-Fire | none | 303 | 0 | 0 | self | – | – |
| 23 | Primary | Coda Hema | Normal Attack | = | multiplying | 52 | 52 | 1 | self | – | – |
| 24 | Primary | Rakta Cernos | Charged Attack | Charged Shot | adding | 470 | 235 | 0.5 | uncharged | – | – |
| 25 | Primary | Mutalist Cernos | Uncharged Attack | Uncharged Shot | multiplying | 205 | 205 | 1 | self | – | – |
| 26 | Primary | Mutalist Cernos | Charged Attack | Charged Shot | adding | 410 | 205 | 0.5 | uncharged | – | – |
| 27 | Primary | Mutalist Cernos | Uncharged AoE Toxin Cloud | Toxin Cloud, parent_variant Uncharged Shot | multiplying | 5 | 5 | 1 | self | – | – |
| 28 | Primary | Mutalist Cernos | Charged AoE Toxin Cloud | Toxin Cloud, parent_variant Charged Shot | adding | 5 | 205 | 41 | parent | – | – |
| 29 | Primary | Cinta | Uncharged Attack | Quick Shot | multiplying | 450 | 450 | 1 | self | – | – |
| 30 | Primary | Cinta | Charged Attack | Charged Shot | multiplying | 800 | 800 | 1 | self | – | – |
| 31 | Primary | Cinta | Perfect Shot | = | multiplying | 600 | 600 | 1 | self | – | – |
| 32 | Primary | Daikyu Prime | Charged Attack | Charged Shot | adding | 750 | 375 | 0.5 | uncharged | – | – |
| 33 | Primary | Drakgoon | Charged Attack | Charged Shot | adding | 700 | 400 | 0.5714 | uncharged | – | – |
| 34 | Primary | Kuva Drakgoon | Charged Attack | Charged Shot | adding | 460 | 230 | 0.5 | uncharged | – | – |
| 35 | Primary | Dread | Charged Attack | Charged Shot | adding | 336 | 168 | 0.5 | uncharged | – | – |
| 36 | Primary | Dread | Incarnon Mode | Incarnon Form Charged Shot | multiplying | 400 | 400 | 1 | self | – | – |
| 37 | Primary | Evensong | Charged Attack | Charged Shot | adding | 500 | 325 | 0.65 | uncharged | – | – |
| 38 | Primary | Evensong | Charged Radial Attack | = | none | 150 | 0 | 0 | self | – | – |
| 39 | Primary | Exergis | Normal Attack | = | multiplying | 1620 | 1620 | 1 | self | – | – |
| 40 | Primary | Felarx | Normal Attack | = | multiplying | 760 | 760 | 1 | self | – | – |
| 41 | Primary | Felarx | Incarnon Mode | Incarnon Form | multiplying | 600 | 600 | 1 | self | – | – |
| 42 | Primary | Ferrox | Hitscan AoE Direct | Radial Attack (parent Charged Shot) | adding | 100 | 350 | 3.5 | parent | – | direct_target_only |
| 43 | Primary | Tenet Ferrox | Hitscan AoE Direct | Radial Attack (parent Charged Shot) | adding | 60 | 200 | 3.3333 | parent | – | direct_target_only |
| 44 | Primary | Fulmin | Semi-Mode | Semi-Auto Mode | multiplying | 500 | 500 | 1 | self | – | – |
| 45 | Primary | Glaxion Vandal | Normal Attack Hitscan AoE | – | adding | 29 | 29 | 1 | parent | – | no_data_attack |
| 46 | Primary | Harpak | Alt-fire | Harpoon | multiplying | 100 | 100 | 1 | self | – | – |
| 47 | Primary | Hema | Normal Attack | = | multiplying | 47 | 47 | 1 | self | – | – |
| 48 | Primary | Javlok | Main-fire | Projectile Impact | multiplying | 160 | 160 | 1 | self | – | – |
| 49 | Primary | Latron/Latron Prime | Incarnon Mode | Incarnon Form | multiplying | 50 | 50 | 1 | self | – | child_aoe=none |
| 50 | Primary | Lanka | Charged Attack | Charged Shot | adding | 525 | 200 | 0.381 | uncharged | – | – |
| 51 | Primary | Miter | Charged Attack | Charged Shot | adding | 500 | 200 | 0.4 | uncharged | – | – |
| 52 | Primary | Miter | Incarnon Mode | Incarnon Form | multiplying | 60 | 60 | 1 | self | – | – |
| 53 | Primary | Nataruk | Charged Attack | Charged Shot | multiplying | 900 | 900 | 1 | self | – | – |
| 54 | Primary | Nataruk | Perfect Shot | = | multiplying | 900 | 900 | 1 | self | – | – |
| 55 | Primary | Nunchasa | Alternate Fire | Alt-Fire | multiplying | 1000 | 1000 | 1 | self | – | – |
| 56 | Primary | Opticor | Charged Hitscan Direct Hit Radial Attack | Charged Shot AoE (parent Charged Shot) | adding | 400 | 1000 | 2.5 | parent | – | direct_target_only |
| 57 | Primary | Opticor Vandal | Charged Hitscan Radial Attack | Charged Shot AoE (parent Charged Shot) | adding | 200 | 400 | 2 | parent | – | direct_target_only |
| 58 | Primary | Paracyst | Alt-fire | Infested Harpoon; Infested Harpoon Contact | multiplying | 15 | 15 | 1 | self | – | – |
| 59 | Primary | Paris | Charged Attack | Charged Shot | adding | 320 | 160 | 0.5 | uncharged | – | – |
| 60 | Primary | Paris Prime | Charged Attack | Charged Shot | adding | 360 | 180 | 0.5 | uncharged | – | – |
| 61 | Primary | Mk1-Paris | Charged Attack | Charged Shot | adding | 230 | 115 | 0.5 | uncharged | – | – |
| 62 | Primary | Paris/Paris Prime | Incarnon Mode | Incarnon Form Charged Shot | multiplying | 520 | 520 | 1 | self | – | – |
| 63 | Primary | Phantasma | Alt-fire Child Bomb | Cluster Bombs Impact | multiplying | 3 | 3 | 1 | self | – | – |
| 64 | Primary | Phantasma Prime | Alt-fire Child Bomb | Cluster Bombs Impact | multiplying | 3 | 3 | 1 | self | – | – |
| 65 | Primary | Proboscis Cernos | Radial attack | Charged Shot Explosion | none | 1003 | 0 | 0 | self | – | – |
| 66 | Primary | Proboscis Cernos | Tendrils | Appendages | none | 90 | 0 | 0 | self | – | – |
| 67 | Primary | Quellor | Alt-fire | Alt-Fire | multiplying | 1400 | 1400 | 1 | self | – | – |
| 68 | Primary | Scourge | Throw | Spear Throw Impact | multiplying | 150 | 150 | 1 | self | – | – |
| 69 | Primary | Scourge Prime | Throw | Spear Throw | multiplying | 200 | 200 | 1 | self | – | – |
| 70 | Primary | Shedu | Normal Attack | = | multiplying | 71 | 71 | 1 | self | – | – |
| 71 | Primary | Sporelacer (Primary) (Tremor) | Normal Attack | Sporelacer (Primary): Normal Attack | multiplying | 127 | 127 | 1 | self | – | – |
| 72 | Primary | Stahlta | Main-fire | Auto | multiplying | 52 | 52 | 1 | self | – | – |
| 73 | Primary | Stahlta | Alt-fire | Alt-Fire | multiplying | 600 | 600 | 1 | self | – | – |
| 74 | Primary | Steflos/Steflos Prime | Normal Attack | = | multiplying | 320 | 320 | 1 | self | – | – |
| 75 | Primary | Tenet Envoy | Normal Attack | Grenade Impact | multiplying | 100 | 100 | 1 | self | – | – |
| 76 | Primary | Tenet Tetra | Alt-fire AoE | Grenade AoE | none | 1000 | 0 | 0 | self | – | – |
| 77 | Primary | Tombfinger (Primary) (Brash) | Normal Attack | Tombfinger (Primary): Normal Attack | multiplying | 38 | 38 | 1 | self | – | – |
| 78 | Primary | Torid | Main-fire | Grenade Impact | multiplying | 100 | 100 | 1 | self | – | – |
| 79 | Primary | Torid | Toxin AoE Cloud | Poison Cloud | multiplying | 40 | 40 | 1 | self | – | – |
| 80 | Primary | Trumna | Main-fire Hitscan Radial Attack | Auto AoE (parent Auto) | adding | 50 | 82 | 1.64 | parent | – | direct_target_only |
| 81 | Primary | Trumna | Alt-fire | Grenade Impact | multiplying | 100 | 100 | 1 | self | – | – |
| 82 | Primary | Vadarya Prime | Primary Fire Lightning Strikes | Normal Attack | none | 400 | 0 | 0 | self | – | – |
| 83 | Primary | Zarr | Cannon Child Bomb | Cannon Mode Cluster Bomb Contact | adding | 15 | 25 | 1.6667 | parent | – | – |
| 84 | Primary | Kuva Zarr | Cannon Child Bomb | Cannon Mode Cluster Bomb Contact | adding | 15 | 50 | 3.3333 | parent | – | – |
| 85 | Primary | Zenith | Alt-fire Disk Bonk | – | multiplying | 10 | 10 | 1 | self | – | no_data_attack |
| 86 | Primary | Zhuge Prime | Arrow Direct Hit | Arrow Impact | adding | 50 | 50 | 1 | self | – | note_quantized_48.4375 |
| 87 | Primary | Zhuge Prime | Arrow Embedded Explosion | Arrow Explosion | none | 40 | 0 | 0 | self | – | – |
| 88 | Secondary | Acrid | Normal Attack | = | adding | 43 | 43 | 1 | self | – | – |
| 89 | Secondary | Akarius Prime | Rocket Impact | = | multiplying | 68 | 68 | 1 | self | – | – |
| 90 | Secondary | Akarius Prime | Rocket Detonation | = | none | 509 | 0 | 0 | self | – | – |
| 91 | Secondary | Atomos | Incarnon Mode | Incarnon Form | adding | 100 or 124 (with Evolution II) | 100 | 1 | self | – | flat_excluded |
| 92 | Secondary | Aegrit | Direct Hit | = | multiplying | 9 | 9 | 1 | self | – | – |
| 93 | Secondary | Angstrum/Prisma Angstrum | Incarnon Mode | Incarnon Form | multiplying | 30 | 30 | 1 | self | – | – |
| 94 | Secondary | Azima | Alt-Fire Turret Bullets | – | none | 20 | 0 | 0 | self | – | no_data_attack |
| 95 | Secondary | Balefire Charger | Uncharged Shot | = | none | 500 | 0 | 0 | self | – | – |
| 96 | Secondary | Balefire Charger | Charged Shot | = | none | 1000 | 0 | 0 | self | – | – |
| 97 | Secondary | Ballistica | Charged Shot | = | adding | 100 | 25 | 0.25 | uncharged | – | – |
| 98 | Secondary | Ballistica Prime | Charged Shot | = | adding | 76 | 38 | 0.5 | uncharged | – | – |
| 99 | Secondary | Ballistica Prime | Incarnon Mode | Incarnon Form | multiplying | 833 | 833 | 1 | self | – | – |
| 100 | Secondary | Rakta Ballistica | Charged Shot | = | adding | 300 | 75 | 0.25 | uncharged | – | – |
| 101 | Secondary | Bronco Prime | Incarnon Mode | Incarnon Form | adding | 238 or 448 (with Evolution II Perk 1) | 238 | 1 | self | – | flat_excluded |
| 102 | Secondary | Castanas | Normal Attack | Mid-Flight Detonation; Embedded Detonation | none | 160 | 0 | 0 | self | – | – |
| 103 | Secondary | Sancti Castanas | Mid-Flight Detonation | = | none | 300 | 0 | 0 | self | – | – |
| 104 | Secondary | Sancti Castanas | Embedded Detonation | = | none | 300 | 0 | 0 | self | – | – |
| 105 | Secondary | Catabolyst | Partial Reload Impact | = | multiplying | 11 | 11 | 1 | self | – | – |
| 106 | Secondary | Catabolyst | Reload From Empty Impact | = | multiplying | 11 | 11 | 1 | self | – | – |
| 107 | Secondary | Catchmoon (Secondary) | Normal Attack | = | multiplying | 256 | 256 | 1 | self | – | – |
| 108 | Secondary | Cestra | Normal Attack | = | adding | 26 or 36 (with Evolution II) | 26 | 1 | self | – | flat_excluded |
| 109 | Secondary | Cestra | Incarnon Mode | Incarnon Form | adding | 50 or 60 (with Evolution II) | 50 | 1 | self | – | flat_excluded |
| 110 | Secondary | Cyanex | Projectile Impact | = | multiplying | 50 | 50 | 1 | self | – | – |
| 111 | Secondary | Cyanex | Burst Mode | = | multiplying | 72 | 72 | 1 | self | – | – |
| 112 | Secondary | Despair | Incarnon Mode | Incarnon Form | adding | 60 or 120 (with Evolution II Perk 2) | 60 | 1 | self | – | flat_excluded |
| 113 | Secondary | Mara Detron | Normal Attack | = | adding | 40 | 40 | 1 | self | – | – |
| 114 | Secondary | Tenet Detron | Normal Attack | = | adding | 26 | 26 | 1 | self | – | ms_proj_factor=0.65 |
| 115 | Secondary | Tenet Detron | Burst Shot | = | adding | 26 | 26 | 1 | self | – | – |
| 116 | Secondary | Dual Toxocyst | Incarnon Mode | Incarnon Form | adding | 75 or 135 (with Evolution II Perk 1) | 75 | 1 | self | – | flat_excluded |
| 117 | Secondary | Epitaph | Charged Shot | = | multiplying | 300 | 300 | 1 | self | – | – |
| 118 | Secondary | Epitaph | Uncharged Direct Hit | = | multiplying | 100 | 100 | 1 | self | – | – |
| 119 | Secondary | Furis | Incarnon Mode | Incarnon Form | adding | 100 or 128 (with Evolution II) | 100 | 1 | self | – | flat_excluded |
| 120 | Secondary | Kompressa Prime | Radial Attack | Explosion | none | 184 | 0 | 0 | self | – | – |
| 121 | Secondary | Kulstar | Cluster Bombs | = | adding | 75 | 200 | 2.6667 | parent | – | table_pct_257_vs_267 |
| 122 | Secondary | Kunai | Normal Attack | = | adding | 46 | 46 | 1 | self | – | – |
| 123 | Secondary | Kunai | Incarnon Mode | Incarnon Form | multiplying | 40 | 40 | 1 | self | – | flat_included, swift_conclusion_in_C |
| 124 | Secondary | Mk1-Kunai | Incarnon Mode | Incarnon Form | multiplying | 24 | 24 | 1 | self | – | flat_included |
| 125 | Secondary | Laetum | Normal Attack | = | adding | 160 | 160 | 1 | self | – | note_devouring_attrition_ok |
| 126 | Secondary | Laetum | Incarnon Form | = | adding | 100 | 100 | 1 | self | – | note_devouring_attrition_ok |
| 127 | Secondary | Lato Vandal | Incarnon Mode | Incarnon Form | adding | 152 or 174 (with Evolution II Perk 1) | 152 | 1 | self | – | flat_excluded |
| 128 | Secondary | Lex Prime | Incarnon Mode | Incarnon Form | adding | 1200 or 1220 (with Evolution II) | 1200 | 1 | self | – | flat_excluded |
| 129 | Secondary | Tenet Plinx | Alt Fire Impact | Alt-Fire | multiplying | 1000 | 1000 | 1 | self | – | mag_mult_in_G |
| 130 | Secondary | Pox | DoT Cloud | Poison Cloud (parent Spore Impact) | adding | 20 | 50 | 2.5 | parent | – | dot_recalc_per_tick |
| 131 | Secondary | Seer | Normal Attack | = | multiplying | 101.01 | 101.01 | 1 | self | – | – |
| 132 | Secondary | Kuva Seer | Projectile Impact | = | multiplying | 131 | 131 | 1 | self | – | – |
| 133 | Secondary | Sepulcrum | Primary-Fire | = | multiplying | 48 | 48 | 1 | self | – | – |
| 134 | Secondary | Sepulcrum | Lock-On Mode | = | multiplying | 96 | 96 | 1 | self | – | – |
| 135 | Secondary | Sonicor | Projectile Impact | = | none | 150 | 0 | 0 | self | – | – |
| 136 | Secondary | Sporelacer (Secondary) | Normal Attack | = | adding | 90.5 | 57 | 0.6298 | fixed | 57 | – |
| 137 | Secondary | Sporelacer (Secondary) | Bomblet Impact | = | adding | 9 | 57 | 6.3333 | parent | – | – |
| 138 | Secondary | Stug | Blob Impact | Blob Embed; Blob Explosion | none | 4 | 0 | 0 | self | – | – |
| 139 | Secondary | Stug | Incarnon Form | Incarnon Form Blob Embed; Incarnon Form Blob Explosion; Incarnon Form Bounce Explosion | none | 50 | 0 | 0 | self | – | – |
| 140 | Secondary | Scyotid | Alt-Fire Grab | – | none | 320 | 0 | 0 | self | – | no_data_attack |
| 141 | Secondary | Sonicor | Projectile Wave | Explosion | none | 150 | 0 | 0 | self | – | – |
| 142 | Secondary | Tenet Diplos | Lock-on Mode | = | multiplying | 28 | 28 | 1 | self | – | – |
| 143 | Secondary | Tenet Spirex | Slug Impact | = | multiplying | 120 | 120 | 1 | self | – | – |
| 144 | Secondary | Tombfinger (Secondary) | Normal Attack | = | multiplying | 18 | 18 | 1 | self | – | – |
| 145 | Secondary | Tysis | Normal Attack | = | adding | 49 | 49 | 1 | self | – | no_dot_co |
| 146 | Secondary | Vasto Prime | Incarnon Mode | Incarnon Form | adding | 420 or 564 (with Evolution II Perk 2) | 420 | 1 | self | – | flat_excluded |
| 147 | Secondary | Zakti/Zakti Prime | Normal Attack | Zakti: Dart Impact; Zakti Prime: = | multiplying | 30 | 30 | 1 | self | – | – |
| 148 | Secondary | Zylok/Zylok Prime | Incarnon Form Radial Attack | = | adding | 776 | 700 | 0.9021 | parent | – | direct_target_only |
| 149 | Secondary | Zylok Prime | Incarnon Mode | Incarnon Form | adding | 500 or 530 (with Evolution II) | 500 | 1 | self | – | flat_excluded |
| 150 | Secondary | Zymos | Homing Spore Contact | = | multiplying | 23 | 23 | 1 | self | – | – |
| 151 | Melee | Argo & Vel | Thrown Glaive | Heavy Attack Glaive | adding | 600 | 200 | 0.3333 | self | – | no_combo |
| 152 | Melee | Corufell | Heavy Attack Projectile | Charged Projectile | multiplying | 1200 | 1200 | 1 | self | – | – |
| 153 | Melee | Destreza/Destreza Prime | Spectral Rapiers | – | adding | 765 | 1530 | 2 | self | – | no_data_attack |
| 154 | Melee | Edun | Heavy Attack Projectile | Polearm Throw | adding | 480 | 120 | 0.25 | self | – | – |
| 155 | Melee | Exodia Contagion | Aerial Projectile | – (07 defines the attack) | multiplying | 2x Zaw Base | 2x Zaw Base | 1 | self | – | wdmg_mode=two_bucket, no_data_attack |
| 156 | Melee | Innodem | Aerial Incarnon Projectile | Dagger Projectile | adding | 360 | 720 | 2 | self | – | – |
| 157 | Melee | Nepheri | Neutral Combo Fireball | Fireballs | multiplying | 131 | 131 | 1 | self | – | – |
| 158 | Melee | Arum Spinosa | Heavy Attack 1 Daggers | First Heavy Attack - Toxic Spines | adding | 175 | 35 | 0.2 | self | – | no_combo |
| 159 | Melee | Arum Spinosa | Heavy Attack 2 Daggers | Second Heavy Attack - Toxic Spines | adding | 175 | 18 | 0.1029 | self | – | no_combo |
| 160 | Melee | Quassus Prime | Heavy Attack 1 Daggers | First Heavy Attack - Ethereal Daggers | adding | 190 | 76 | 0.4 | self | – | no_combo |
| 161 | Melee | Quassus Prime | Heavy Attack 2 Daggers | Second Heavy Attack - Ethereal Daggers | adding | 380 | 76 | 0.2 | self | – | no_combo |
| 162 | Melee | Stropha | Heavy Attack Projectile | Ranged Attack | adding | 2800 | 700 | 0.25 | self | – | – |
| 163 | Melee | Syam | Heavy Attack Wave 1 | First Shockwave | multiplying | 500 | 500 | 1 | self | – | – |
| 164 | Melee | Syam | Heavy Attack Wave 2 | Second Shockwave | multiplying | 1500 | 1500 | 1 | self | – | – |
| 165 | Melee | Tenet Agendus | Heavy Attack Wave 1 | Energy Disk (rows 165, 166 agree) | multiplying | 2800 | 560 | 0.2 | self | – | – |
| 166 | Melee | Tenet Agendus | Heavy Attack Wave 2 | Energy Disk (rows 165, 166 agree) | multiplying | 3600 | 720 | 0.2 | self | – | – |
| 167 | Melee | Tenet Grigori | Energy Disk | = | multiplying | 4080 | 4080 | 1 | self | – | wdmg_mode=universal_only |
| 168 | Melee | Verdilac | Energy Wave | = | multiplying | 213 | 213 | 1 | self | – | – |
| 169 | Melee | Zenistar | Heavy Disc | Disc Impact | adding | 75 | 780 | 10.4 | self | – | – |
| 170 | Archgun | Arbucep | Direct Hit | 1st Attack … 6th Attack | multiplying | – | – | 1 | self | – | child_aoe=none |
| 171 | Archgun | Kuva Ayanga | Direct Hit | Grenade Impact | multiplying | – | – | 1 | self | – | – |
| 172 | Archgun | Corvas (Atmosphere) | Normal attack | Corvas (Atmosphere): Uncharged Shot; Charged Shot | adding | – | – | 1 | self | – | – |
| 173 | Archgun | Corvas Prime | Uncharged Attack | Uncharged Shot | multiplying | – | – | 1 | self | – | – |
| 174 | Archgun | Corvas Prime | Charged Attack | Charged Shot | multiplying | – | – | 1 | self | – | – |
| 175 | Archgun | Fluctus | Normal Attack | Normal Attack | adding | – | – | 1 | self | – | – |
| 176 | Archgun | Grattler | Normal attack | Projectile Impact | multiplying | – | – | 1 | self | – | – |
| 177 | Archgun | Larkspur Prime | Alt-fire | Alt-Fire Projectile Impact | multiplying | – | – | 1 | self | – | – |
| 178 | Archgun | Mandonel | Uncharged Attack | Horizontal Spread | multiplying | – | – | 1 | self | – | – |
| 179 | Archgun | Mandonel | Charged attack | Charge | adding | – | – | 1 | self | – | – |
| 180 | Archgun | Mausolon | Main-fire Radial Attack | Auto Radial Attack (parent Auto) | adding | – | – | 2.5 | parent | – | direct_target_only |
| 181 | Archgun | Mausolon | Alt-fire Radial Attack | Charged Shot Explosion (parent Charged Shot Laser) | adding | – | – | 0.33 | parent | – | direct_target_only, ui_5x_UNVERIFIED |
| 182 | Archgun | Velocitus | Uncharged attack | Uncharged Shot | multiplying | – | – | 1 | self | – | – |
| 183 | Archgun | Velocitus | Charged attack | Charged Shot | multiplying | – | – | 1 | self | – | – |
| 184 | Exalted | Noctua | Noctua Primary Fire | Normal Attack | adding | 250 | 200 | 0.8 | fixed | 200 | – |
| 185 | Exalted | Noctua | Noctua Alt Fire | Active Attack | adding | 2750 | 2500 | 0.9091 | fixed | 2500 | – |
| 186 | Exalted | Noctua | Final Verse Wordwarden | – | adding | 125 | 200 | 1.6 | fixed | 200 | no_data_attack |
| 187 | Exalted | Artemis Bow | Uncharged/Charged Attack | Base Uncharged Shot; Base Charged Shot | adding | 1680 | 525 | 0.3125 | fixed | 75 per arrow | off_with=Concentrated Arrow |
| 188 | Exalted | Exalted Umbra Blade | Light Waves | – | adding | 250 | 500 | 2 | self | – | no_data_attack |
| 189 | Exalted | Exalted Umbra Blade | Heavy Waves | – | multiplying | 250 | 250 | 1 | self | – | no_data_attack |
| 190 | Robotic | Burst Laser/Prisma Burst Laser/Burst Laser Prime | Normal Attack | = | multiplying | 25 | 25 | 1 | self | – | – |
| 191 | Robotic | Cryotra | Normal Attack | Direct Hit | multiplying | 10 | 10 | 1 | self | – | child_aoe=none |
| 192 | Robotic | Helstrum | Normal Attack | = | multiplying | 9 | 9 | 1 | self | – | – |
| 193 | Robotic | Prime Laser Rifle | Normal Attack | = | multiplying | 55 | 55 | 1 | self | – | – |
| 194 | Robotic | Laser Rifle | Normal Attack | = | multiplying | 45 | 45 | 1 | self | – | – |
| 195 | Robotic | Multron | Normal Attack | Projectile Impact | multiplying | 10 | 10 | 1 | self | – | – |
| 196 | Robotic | Stinger | Normal Attack | = | adding | 15 | 15 | 1 | self | – | no_embedded_dot_co |
| 197 | Robotic | Vulcax | Normal Attack + AoE | Impact | adding | 500 + 100 | 1000 | 2 | self | – | aoe_child=none |
| 198 | Robotic | Vulklok | Normal Attack | = | multiplying | 175 | 175 | 1 | self | – | – |
| 199 | Amp | Shwaak Prism | Normal Attack | = | multiplying | 3500 | 3500 | 1 | self | – | – |
| 200 | Amp | Phahd Scaffold | Normal Attack | Direct Hit | multiplying | 3000 | 3000 | 1 | self | – | – |
| 201 | Amp | Shraksun Scaffold | Normal Attack | Direct Hit | multiplying | 3000 | 3000 | 1 | self | – | – |
| 202 | Primary | Kuva Bramma | Cluster Bomb Contact | Cluster Bomb Contact (parent Charged Shot, `06` E-6.23 override) | adding | 49 | 187 (parent, computed) | 3.8163 | parent | – | prose_row, co_tested=false |

---

## 3. Multiplier membership: G_F and G

### 3.1 G_F: final multipliers omitted by the adding CO term

Each member is its own factor in `G_F` (E-4.3). On `multiplying` and `none` attacks the same factors multiply the
whole Y. List from `W:Condition_Overload_(Mechanic)` §Math ("Damage multipliers or effects that are ignored with
Additive Stacking CO-like bonuses"), values from the cited pages.

| Member | Factor | Class (§4.1) | Applies to | Source |
|---|---|---|---|---|
| Mirage Eclipse (Solar) | `1 + 2.00·Str_e`; subsumed `1 + 0.30·Str_e`; T-14 enters the final value | (u) | all weapons of the caster; **twice** on `two_bucket` melee projectiles | `W:Eclipse`; `W:Helminth` U35.5 (subsumed 150 % → 30 %) |
| Garuda passive | `1 + min(1.00, 0.05·kills)`, decays 0.01 per 2 s after 5 s without a kill | (u) | weapons and melee; twice on `two_bucket` | `W:Garuda/Abilities/Passive`; `F:buffs` §1.14 |
| Nidus Parasitic Link (damage part) | `1 + 0.25·Str_N` (ranks 0.10 / 0.15 / 0.20 / 0.25) | (u) | Nidus and the linked ally | `W:Parasitic_Link` (example `100 × 1.5 + 100 × 0.8 = 230`) |
| Vauban Overdriver | `1 + 0.25·Str_e` | (u) | buffed ally | `W:Universal_Weapon_Bonuses` §Damage Multipliers |
| Equinox Duality augment, Excalibur Furious Javelin, Mirage Hall of Mirrors damage modifier | user value (not researched; Hall of Mirrors changes clone damage, out of scope) | (u) | – | CO Mechanic §Math; `W:Furious_Javelin` |
| Longbow Sharpshot | `1 + 3.00` on the next shot after a weak-point hit | (b) | bows | `W:Longbow_Sharpshot`; `M:Arcane` |
| Primary Compression, attacks marked "Multiplies" | `1 + dpm·0.8·R_mod·eff_a`, dpm = 0.50 + 0.10·rank (1.00 at rank 5) | (b) aiming | projectile AoE primaries | `W:Primary_Compression` (rank table; Acceltra 4.0 m → +320 %, Primed Firestorm 5.76 m → +460.8 %) |
| Extinguished Dragon Key | 0.25 | (u) | all | CO Mechanic §Math |
| Range falloff | Φ(d) (`06`) | – | attacks with falloff | idem ("but not Beam ramp-up") |
| Bow charge scaling | 1 for every data attack: charged and partial shots are separate data attacks whose damage already holds the charge (`06` §5, OQ-607) | – | charge attacks | idem ("Bow charging and more"); §2.7 |
| `co_omit` of the attack | M(c) for `no_combo` attacks | – | Argo & Vel throw, Arum Spinosa and Quassus Prime heavy daggers | catalogue |

Primary Compression details: `R_mod` is the attack's modded radius (the page's "Max Damage Bonus w/ Primed
Firestorm" column grows with radius mods); `eff_a` is the page's per-attack "Compression Effectiveness" (1.00 for
most; Mausolon main while aiming during alt-fire 4.44); attacks marked "Adds" (Ambassador alt-fire, Battacor
alt-fire, Braton-family and Burston-family Incarnon form, Ferrox and Tenet Ferrox, Mausolon main and alt, Opticor and
Opticor Vandal, Trumna and Trumna Prime) put the same value into **A** instead; attacks marked "Doesn't Work" get
nothing. Ammo efficiency `+0.055` per metre lost is `06`'s. The radius loss is `0.8·R_mod` because aiming sets the
radius to `0.2·R_mod`. The page's effectiveness table is imported as per-attack data (owner of import: data
pipeline; value semantics here).

### 3.2 G: every other weapon-side multiplier

`G` is the product of the factors below; each member is a separate factor (`W:Calculating_Bonuses` §Multiplicative
Stacking; `W:Universal_Weapon_Bonuses` §Damage Multipliers: "All sources are multiplicative to each other and to base
damage bonuses, unless stated otherwise"). On adding attacks G multiplies the CO term too (E-4.3).

| Member | Factor | Class | Value / timing owner | Source |
|---|---|---|---|---|
| Stance hit multiplier S_j | data | – | `07` (adding: E-4.6) | `F:melee` §3 |
| Heavy-attack type multiplier; heavy combo multiplier M(c) | data | – | `07` | `W:Melee` §Heavy Attack Damage |
| Ability combo (exalted, T-11) | data | – | `07` | overview T-11 |
| Sniper combo | `1.5 + 0.5·floor(log3(count/MinCombo))` | – | `06` | `W:Sniper_Rifle` |
| Primed Chamber + Charged Chamber (one factor, additive with each other) | `1 + 1.00·[PC] + 0.40·[CC]` on the first shot of a magazine | (b), schedule-driven also under `all_max` | `06` marks | `W:Primed_Chamber`, `W:Charged_Chamber` (Charged Chamber: no effect on sentinel weapons) |
| Synth Charge | `1 + 2.00` on the final shot of a magazine; requires base magazine ≥ 6; not beams (except Catabolyst reload grenade), not Incarnon forms, not whole-magazine alt-fires | (b), schedule-driven | `06` marks | `W:Synth_Charge`; `F:weapon` §16 |
| Beam ramp-up | 0.2 → 1 | – | `06` | CO Mechanic §Math |
| Lasting Purity | 1.6 while aiming | (b) | – | `F:core` §12 (`W:Calculating_Bonuses`) |
| Vauban passive | 1.25 vs incapacitated targets; also a second factor on DoT procs applied while incapacitated (`M_dot`) | (a) | – | `W:Vauban/Abilities/Passive` (example `… × 1.25 × 1.25`) |
| Ivara Navigator | up to `5·Str_e`, projectiles only (not hitscan), only on projectiles taken over in flight | (u) | – | `W:Universal_Weapon_Bonuses`; `W:Navigator` |
| Mecha Empowered (aura, Mecha set mark) | `1 + 1.50` per aura vs marked target; "applied each time damage is calculated": a second factor on DoT ticks (`M_dot`) | (c) mark uptime | §9 | `W:Mecha_Empowered`; D-041 |
| Melee Careen (melee arcane) | 2.5 vs Frozen targets (melee) | (a) | – | `M:Arcane` ("x2.50 Melee Damage against Frozen enemies"); D-068 |
| Devouring Attrition (Phenmor, Laetum Incarnon) | `1 + 20.00` with probability 0.50 on instances with k = 0 | (b) | MC: Bernoulli; EV: `1 + 0.5·20` on the k = 0 branch | `W:Phenmor` ("Damage bonus is multiplicative to base damage bonuses such as Serration") |
| Braton Incarnon Evo II multishot-pellet damage | `1 + y` on multishot-generated shots only | (b) | perk table | `W:Braton_Incarnon_Genesis` lines 72-75 |
| Torid Incarnon Plentiful Mayhem | `1 + 0.60` on multishot-generated projectiles (normal form) | (b) | – | `F:mods` §9 (`W:Torid_Incarnon_Genesis`) |
| Secondary Surge | `1 + min(cap_r, Σ_casts 0.005·E_after)` on the next shot after one or more ability casts, all pellets of that shot; consumed on firing even on a miss; each cast adds to the stored bonus; separate per weapon; cap_r = 2.00 + 1.00·rank (7.00 at rank 5, ×8); E_after = energy left after each cast | (c) on ability cast; `consume_on = next_shot` | – | `W:Secondary_Surge` §Notes ("+0.5% damage per point of energy that remains after casting", "multiplicative to Hornet Strike", "applies to all additional pellets from multishot", "consumed on firing the weapon, even if it does not hit"); TV-04-24 |
| Peacemaker ramp | `1 + 1.5·min(n_b, 20)/20`, n_b = index of the current **burst** (bursts of 2 shots; the first burst has n_b = 1, so +0.075; the 20th and later bursts ×2.5) | – | `06` (E-6.19 `b_pm`) | `W:Peacemaker` lines 24-35 ("fires in bursts of 2 shots", "+7.5% per burst fired", "(1 + 150% · min(Burst Count, 20)/20)"); TV-04-20 |
| Slam Damage | `1 + Σ` (Seismic Wave 2.00, Nira's set 1.50 per piece, riven splice) on slams | – | `07` | `W:Melee` §Slam Damage |
| Finisher damage | `1 + Σ` finisher bonuses on finishers | – | §7 | `W:Covert_Lethality` |
| Spring-Loaded Broadhead (Daikyu augment) | `1 + 0.40` when the target distance d > 15 m (T-27), once | gate (distance, never assumed) | – | `Module:Mods/data` (cached `Mods_data.json`, MaxRank 5: "Increase damage by +40% if the target is over 15m away"); inspiration `multDAIKYUBroadhead: 0.4` (JS L235-239) agrees on the value; D-404 |
| Tenet Plinx alt-fire magazine multiplier | `1 + x·(Mag_mod − Mag_base)` | – | data | catalogue note "Scales properly with magazine size"; formula from JS L10514-10519 only (UNVERIFIED) |
| Double Tap (Latron augment) | `1 + 0.20·min(20, H − 1)` on every pellet of a shot, H = hits since the last reset **including** this shot's hitting pellets; reset (all stacks) after 2 s without a hit or when a shot misses | (b) on hit, `hit_unit = pellet`, `reset_on = miss` | – | `W:Double_Tap` §Notes (fetched 2026-11-23: "+20% … stacking 20 times up to a maximum of +400%", "applied on hit to all pellets as damage * 20% * (hits − 1)", multishot 3: +40 %, +100 %, +160 %; "Multiplicatively stacks with damage bonuses like Serration and Faction Damage Bonus"); `all_max`: ×5; TV-04-25 |
| Weapon-made buffing field damage penalty (Mutalist Quanta alt-fire orb, Simulor orb or singularity) | `0.66^f` for f fields passed ("−34% Total Damage per field (multiplicative)") | (u) | – | `W:Buffing_Fields` §Sources table |

Not in G: Sonar (`sonar_mult`, `02`), Secondary Fortifier (`M_OG`, overview E-12), vulnerabilities (`V`, `05`),
Arcane Crepuscular and Tenacious Bond (`sum_cd_abs`, `02`).

**Default rule.** A weapon-side source described as a damage *multiplier* that is not in §3.1 joins `G`. A source
described as "+X % (weapon / melee / primary / secondary) damage" with no multiplicative statement joins `A` (the base
bucket of `01`): every page that states the relation for such a buff says "additive with Serration / Pressure Point"
(`W:Universal_Weapon_Bonuses` §Damage list, `W:Vex_Armor`, `W:Elemental_Ward`, `W:Amp_(Ability)`). Unstated items are
marked UNVERIFIED in §5. Vauban passive and Navigator are not on the CO page's omitted list, so they are in `G` and
multiply the adding CO term (UNVERIFIED, OQ-404).

### 3.3 Order inside ST-4 and ST-5

```text
E-4.4  Weapon-side scalar of one instance (refines E-12)
  W_t = q_t · Y · (1 + F) · G · Part · c_k          Y from E-4.3 (G_F inside Y), G from §3.2
```

All factors are commutative except the adding CO term, which must be added inside Y before `(1 + F)`, G, Part and
c_k multiply (overview section 3.1).

### 3.4 Melee projectiles: two weapon-damage buckets

Melee projectiles have a "Melee Damage" bucket `A_m` and a universal "Weapon Damage" bucket `A_w` that multiply each
other; sources that belong to both apply in both; Eclipse and Garuda passive apply twice (multiplicatively); Void
Strike counts once (`W:Exodia_Contagion` §Interactions, `W:Power_Throw` §Notes, `W:Vex_Armor`, `W:Eclipse`;
`F:addmul` §12).

| `mproj_mode` | Attacks | Y (replaces the `(1 + A)` factor of E-4.3) |
|---|---|---|
| `standard` | every gun attack and normal melee hit | `(1 + A)`; universal weapon-damage sources join A only where they apply (Arachne, Vigorous Swap and Holster Amp do not apply to normal melee) |
| `two_bucket` | glaive throws and glaive explosions, Exodia Contagion projectile | `(1 + A_m)·(1 + A_w)`, and `G_F` contains Eclipse² and Garuda² |
| `single_bucket` | Exodia Epidemic, Tenet Agendus, Wolf Sledge, Ceramic Dagger, Ack & Brunt and Hate Incarnon projectiles | `(1 + A_m + A_w)` (Arachne additive) |
| `universal_only` | Tenet Grigori energy disk | `(1 + A_w)` (Pressure Point, Arcane Fury do not apply) |

Assignment of `mproj_mode` (data pipeline; every melee attack must receive exactly one value):

| Rule | Attacks (data `AttackName` of `Module:Weapons/data`, or the `07` attack id) | `mproj_mode` |
|---|---|---|
| M1 | catalogue flag `wdmg_mode=…` (rows 155 Exodia Contagion, 167 Tenet Grigori "Energy Disk") | the flag |
| M2 | weapon class Glaive (`07` class), every non-slam attack whose `ShotType` is `Projectile` or `AoE` (the throw and its explosions: "Thrown", "Heavy Attack Glaive", "… Explosion", "… AoE"); Argo & Vel "Heavy Attack Glaive" and "Heavy Attack Glaive AoE" | `two_bucket` |
| M3 | Exodia Epidemic projectile (`07` attack), Tenet Agendus "Energy Disk", Wolf Sledge throw, Ceramic Dagger throw, Ack & Brunt shield throw, Hate Incarnon projectile (`W:Arcane_Arachne` §Notes via `F:addmul` §5: Arachne "additive" on these) | `single_bucket` |
| M4 | every other melee attack (stance hits, heavies, slams, other melee projectiles such as Corufell "Charged Projectile", Edun "Polearm Throw", Syam shockwaves) and every gun attack | `standard` (Decision for melee projectiles not named in M1-M3: UNVERIFIED, OQ-405) |

The M2 and M3 attack lists are the attacks `F:addmul` §5 and §12 name; the pipeline matches them by weapon name and
`ShotType`, and the data-build report lists every melee projectile that fell to M4.

```text
E-4.5  Adding CO on a two_bucket attack (glaive throw):
  Y = [B'·(1 + A_m)·G_F + B_co·C]·(1 + A_w)           G_F here holds Eclipse² and Garuda² (one of each in each bucket)
```

E-4.5 places the adding CO term in the melee-damage bucket ("Condition Overload when additive" is a Melee Damage
member, `F:addmul` §12) so it is multiplied by `(1 + A_w)` (Decision, UNVERIFIED, OQ-405). Multiplying
`two_bucket` attacks (Exodia Contagion) use `B'·(1 + A_m)·(1 + A_w)·(1 + C)·G_F`.

Bucket membership for melee projectiles: `M` = Pressure Point family, Arcane Fury, Arcane Blade Charger, Steel
Charge, Melee Retaliation, Valkyr Rage passive, Excalibur passive, Kinetic Plating bonus, Void Strike (once); `W` =
Arcane Arachne, Vigorous Swap, Holster Amp, Toxin Elemental Ward, Power Throw, Provoked, Absorb, Metamorphosis,
Shooting Gallery; `M` and `W` = Vex Armor, Deathbringer, Octavia Amp, Rift Torrent, Lethal Levitation. The
placement of Provoked, Absorb, Metamorphosis and Shooting Gallery (listed as universal weapon damage, not tested on
melee projectiles) is UNVERIFIED.

Glaive airborne-throw detonations do not get the heavy-attack crit chance doubling, standing-throw Volatile
explosions do (bug; `02` owns; `F:addmul` §12).

---

## 4. Condition model (T-02)

### 4.1 Classes and triggers

Every effect in §5 declares one class (overview section 6.2, plus class (u) below) and, for classes (b) and (c), a
trigger.

| Class | Trigger ids (examples) | Event source |
|---|---|---|
| (a) target state | `status_count` (n), `cold_stacks`, `target_frozen` (10 Cold stacks or Frozen state, `03`), `target_health_below(f)`, `target_incapacitated`, `target_marked`, `target_affected_by(s)` | target state before the instance |
| (b) own events | `on_hit`, `on_crit`, `on_tier1_crit`, `on_big_crit` (k ≥ 2), `on_weak_point_hit`, `on_headshot`, `on_status(s)`, `on_reload`, `on_reload_from_empty`, `on_fire`, `on_miss`, `while_aiming`, `first_shot`, `last_shot`, `combo(c)`, `on_melee_hit_count`, `next_shot_after(x)` | the attack's own schedule and rolls (`06`, `07`, `02`) |
| (c) external | `on_kill`, `on_weak_point_kill`, `on_melee_kill`, `on_primary_kill`, `on_ability_cast`, `while_invisible`, `on_wall_latch`, `on_holster`, `on_damaged`, `on_roll`, `on_dodge`, `while_overshields`, `on_shield_break`, `in_zone`, `ally_buffs`, `energy_level`, `shield_level`, `armor_level`, `health_level` | user uptime `u_e` and stack count `m_e` |
| (u) user-enabled buff | the scenario's buff list: ability buffs and Helminth values (§5.6), target vulnerabilities and debuff abilities (§5.7), squadmates' auras (§5.8), focus nodes (§5.10), companion, squad and environment buffs (§5.11), weapon-made buffing fields, T-13 … T-19 | the user enabling the entry; user value, stacks `m_e` and uptime `u_e` |

Kills are external in every view: the layer views never kill the target, and kills of other enemies are not
modelled (CD-11). "On kill" effects (Galvanized stacks, Merciless, Deadhead, chargers) are therefore class (c) even
when the kill must be made with the same weapon.

**Class (u) (Decision; addition to overview 6.2 proposed in OQ-431).** An effect that exists only because the user put it in the
scenario's buff list (or switched on T-13 … T-19) is class (u): its activation (the cast, the squadmate's aura, the
companion) is the user's statement, not a condition of the build. It applies with its user inputs (value or caster
Strength, stacks `m_e`, uptime `u_e`; defaults max and 1) in **every** T-02 mode; T-02 governs only effects whose
activation comes from mods, arcanes, set bonuses, Incarnon perks or weapon passives of the build. This matches the
overview's own toggle definitions (T-13 "`sum_faction += value`", T-14 "factor `1 + value` in `G_F`", with no
reference to T-02) and the inspiration, where Roar, Eclipse, Xata's Whisper, Toxic Lash and Nourish are separate
checkboxes applied whenever set (`getFactionMod`, JS L9104-9109: `rhinoBuff.is_set`) while "Trigger all conditions"
gates only `WITH_COND` values (JS L8668-8744; `I:feat` §8). Conditions **inside** a class (u) effect keep their own
handling: target-state conditions (Grave Spirit vs Doomed, Biting Frost vs frozen, Shadow Haze vs Reap-affected,
Firing Line vs Lifted) are class (a) sub-conditions and follow T-02 like any class (a) term; gates (melee only,
headshots, weapon class) apply as gates.

**Triggers are not gates.** A trigger is the event that starts an effect; a gate is a property every instance must
have to use it (§4.3). Under `all_max` every class (b) and (c) trigger is assumed to have fired, even when the
scenario makes the event impossible (Arcane Rage with T-03 = `body`; the inspiration does the same: Arcane Rage and
Arcane Precision are `WITH_COND` values independent of the headshot option, JS L5095-5125). Under `simulated` the
event rates of E-4.9 apply: `on_headshot` has rate (hit units per second) when T-03 = `head` and 0 otherwise;
`on_weak_point_hit` has that rate when T-03 ∈ {`head`, `weak_point`} and 0 otherwise; radial instances, DoT ticks and
extra hits never count as headshots or weak-point hits (overview T-03; E-7 radial Part = 1).

**Hit units.** "On hit" means different things per effect, so every on-hit-type effect carries a `hit_unit`
(§4.2): `pellet` (each pellet instance, the default; Double Tap: "Additional hits caused by … Multishot will allow
each bullet to trigger multiple stacks"), `shot` (all pellets of one shot on one target count once; Secondary
Enervate, whose page says hitscan multishot and shotgun-sidearm pellets count once and projectiles of one shot
register once within a grace period), `instance` (every damage instance including extra hits; Arcane Strike and
other melee "on hit" arcanes: one per melee hit instance). Extra hits count as hits for on-hit effects (`W:Extra_Hit`
§Interactions: "Extra Hits can trigger "on-hit" effects like Secondary Enervate and Double Tap"), each as one more
unit of its effect's `hit_unit` (Decision; whether an extra hit in the same game tick as its parent merges with it
for one-per-tick effects is UNVERIFIED, OQ-436).

### 4.2 Effect state machine

Every effect record of the curated effects data (§5) carries the fields below. Fields marked "data for" are read by
the named document; `04` fills them.

| Field | Values | Meaning, reader |
|---|---|---|
| `id`, `source` | text | effect identity (mod, arcane, ability, perk) and its citation |
| `class` | `–`, (a), (b), (c), (u) | condition class (§4.1); `–` = unconditional |
| `bucket`, `value` | overview symbol; fraction or factor at max rank | where the value enters (§5 intro) |
| `rank_rule` | `prop` (`value·(r + 1)/(R_max + 1)`) or `affine(a, b)` (`a + b·r`) or `table` | per-rank scaling (§1.5) |
| `element`, `combines`, `position` | DamageType; bool; `mod_slot` / `innate_group` / `non_combining` | data for `01` §4.10 (combination order) |
| `dot_scaling` | bool | data for `03`: whether an elemental bonus also raises its element's DoT (`dot_elem`) |
| `universal_weapon_damage` | bool | data for `01` and §3.4: the source is "weapon damage" (bucket W of melee projectiles) |
| `layer_mask`, `type_mask` | subset of {OG, SH, HP}; set of DamageType or `all` | data for `05` (vulnerability entries, §5.7); default `{HP}` and `all` |
| `trigger`, `chance` p | §4.1 trigger id; fraction | independent draw per trigger event (`U-6` key `mechanic id` = effect id) |
| `hit_unit` | `pellet`, `shot`, `instance` | what one "hit" is for on-hit triggers and counters (§4.1) |
| `gain` g, `max_stacks` m, `per_stack` v | count; count (or `none`: unbounded until reset); value | stack arithmetic |
| `duration` D, `refresh` | s; `refresh` / `no_refresh_while_active` | timer |
| `stack_timer` | `shared_lose_one`, `shared_lose_all`, `independent` | expiry rule |
| `decay` | `none`, `linear(D)`, `garuda`, `idle(t_idle, rate)` | value decay (`idle`: after t_idle s without a trigger, lose `rate` per second: Primal Rage 0.01/s after 5 s, Valkyr Rage, Garuda) |
| `consume_on` | `none`, `next_shot`, `next_heavy`, `next_shots(k)` | the buff is used up by the next qualifying use (Secondary Surge, Longbow Sharpshot, Melee Animosity, Molt Vigor next cast, Deadly Maneuvers 2 shots) |
| `reset_on` | `none` or (event, count) | all stacks and counters return to 0 after `count` events of that kind (Secondary Enervate (`on_big_crit`, rank + 1); Double Tap (`on_miss`, 1); Strun Incarnon Blazing Barrel (`on_reload`, 1)) |
| `gate`, `attack_gate` | instance properties | checked at use: heavy, slide, slam, finisher, headshot, weak-point hit, distance, aiming state (§4.3) |
| `weapon_class_scope` | set | primary, secondary, melee, archgun, exalted, robotic, Incarnon form, battery, bow, sniper, shotgun, pistol … (`W:Extra_Hit` applicability, arcane descriptions) |
| `str_scaled`, `dur_scaled` | bool | the value scales with the caster's Strength, the duration with the caster's Duration (§4.4) |
| `caster` | `self`, `ally`, `fixed` | whose Strength and Duration apply (§4.4); `fixed` = not scaled |

```text
E-4.8  State machine (MC; EV uses E-4.9)
  on trigger event at time τ, with probability p:
     if refresh = no_refresh_while_active and stacks > 0: ignore
     stacks ← min(m, stacks + g)                         (m = none: no cap)
     shared_*      : expiry ← τ + D
     independent   : add one timer τ + D per gained stack; at the cap the oldest timer is replaced
  at expiry τ_e:
     shared_lose_one : stacks ← stacks − 1; if stacks > 0: expiry ← τ_e + D
     shared_lose_all : stacks ← 0
     independent     : stacks ← stacks − 1 (that timer only)
  on an event counted by reset_on = (kind, k): counter ← counter + 1; if counter = k: stacks ← 0, counter ← 0
  consume_on: after the qualifying use (the next shot fired, the next heavy, k shots) has read the value:
     stacks ← 0 (Secondary Surge: even if the shot misses)
  value(τ) = v · stacks(τ) · decay(τ)          decay(τ) = 1; linear: 1 − (τ − τ_trigger)/D; garuda (§3.1);
                                               idle: value − rate·max(0, τ − τ_last − t_idle), floored at 0
  an instance at time τ uses value(τ⁻) (state before the instance; CD-04) if its gate holds
```

Effects whose state is driven by the user rather than by events (Secondary Outburst: "per combo multiplier consumed"
on the swap; Molt Augmented kills; Vex Armor charge) are class (c) or (u) with the stack count `m_e` as the user
input; the state machine is not run for them.

Stack timers per family (sources: `F:mods` §6.1, §11; `W:Galvanized_Shot` §Notes; per-arcane notes as cited):

| `stack_timer` | Effects |
|---|---|
| `shared_lose_one` | Galvanized Aptitude, Savvy, Shot, Chamber, Diffusion, Hell, Steel, Elementalist, Reflex ("when the buff times out, one stack is lost and the timer resets"); Primary and Secondary Merciless (4 s, full decay 48 s); Primary and Secondary Deadhead (24 s, full decay 72 s) and Primary and Secondary Dexterity (20 s) (`W:Primary_Deadhead` line 32, `W:Secondary_Deadhead` line 32, `W:Primary_Dexterity` line 31, `W:Secondary_Dexterity` line 31: "When the buff times out, one stack is lost and the buff duration resets"; `W:Primary_Merciless` §Tips: Deadhead "can last for a total of 72 seconds") |
| `independent` | Galvanized Scope, Galvanized Crosshairs (each stack its own 12 s timer) |
| `shared_lose_all` | Primary Frostbite and Primary Blight (12 s; `W:Primary_Frostbite` §Notes: "If the 12 second timer expires without applying a fresh Cold status effect, all stacks are removed … consistent with … Primary Blight and Cascadia Flare"), Cascadia Flare (10 s; `W:Cascadia_Flare` line 55), Arcane Hot Shot (10 s; `W:Arcane_Hot_Shot` line 60: "All stacks of the buff expire simultaneously"); UNVERIFIED for Conjunction Voltage (12 s, by analogy with the other status-triggered arcanes), Primary Crux (10 s), Melee Exposure (25 s shared timer), Split Flights (2 s) |
| single stack, `refresh` | most chance-on-event arcanes (Fury, Rage, Rise, Awakening, Strike, Velocity, Acceleration, Tempo, Avenger …) (`M:Arcane` `IsRefreshable = true`) |
| single stack, `no_refresh_while_active` | Arcane Arachne, Arcane Precision, Melee Influence, Secondary Encumber-type effects with `IsRefreshable = false` (`M:Arcane`; `W:Arcane_Precision` "not refreshed by further hits"); Death's Harvest vulnerability ("cannot be refreshed … has to fully expire") |

### 4.3 Evaluation per T-02 mode

| T-02 | Class (a) | Class (b) | Class (c) | Class (u) |
|---|---|---|---|---|
| `all_max` | count-based terms use n per T-05 and n_COL from the state; every threshold true | `stacks = m` from t = 0 | `stacks = m` from t = 0 | user inputs (default `m_e` max, `u_e` = 1) |
| `none` | 0 | 0 | 0 | user inputs (unchanged) |
| `simulated` | from the target state: expected (EV) or live (MC); health thresholds only in the `layered` pass, false in the non-depleting views (the target stays at full health) | E-4.8 driven by the attack's own events from the fresh state (stacks 0) | `u_e·v·m_e` over the window (default u = 1, m = max) (Decision: external events are not simulated) | user inputs (unchanged) |

Overview 6.2 lists no user-enabled buff among its class (c) examples (on kill, on ability cast, while invisible,
wall latch, holster, on damaged, Arcane Avenger), and its toggles T-13 … T-19 apply their values with no reference to
T-02. This document reads user-enabled buffs as outside T-02's scope (Decision) and gives them the class (u) column
above; the manager is asked to add the class to overview 6.2 (OQ-431).

Gates are never assumed: an effect gated on "heavy attack", "slide attack", "first shot", "weak-point hit" (T-03),
a target distance (T-27) or a weapon class applies only to instances that have that property, in every mode. "While
aiming" and "while invisible" are conditions, not gates: true under `all_max`, user uptime under `simulated`
(Decision). Unconditional parts of a conditional mod (Galvanized Aptitude's +80 % status chance, Merciless +30 %
reload) apply in every mode. First/last-shot factors are schedule-driven in every mode (overview section 6.2).

**Effects without a maximum.** An effect whose stacks have no cap but a reset (`max_stacks = none`, Secondary
Enervate) has no "max stacks" value; it is evaluated as under `simulated` in both `all_max` and `simulated`, and is
off under `none` (Decision, like the schedule-driven first/last-shot factors; the inspiration also tracks Enervate per
hit, JS L10673-10696).

**Timing stats (Decision, aligned with `06` §5.9).** Effects on fire rate, attack speed, reload, charge time, burst
delay, magazine or ammo efficiency change the shot schedule, which overview 4.1 keeps free of random draws. They are
therefore never driven by random events: under `all_max` and `none` they are on at max or off for the whole window;
under `simulated`, effects triggered by deterministic schedule events (on reload, on reload from empty, first and last
shot) follow those events as `06` §5.9 defines, and effects with random or external triggers (Arcane Acceleration,
Velocity, Tempo, Momentum and Strike, Shotgun Vendetta, Primary Crux ammo efficiency, Conjunction Voltage reload,
Berserker Fury, Merulina Guardian, Combo Fury) enter as `u_e × max value` with the class (c) uptime input (default 1)
in both evaluators. An alternative (iterating E-4.9 to a fixed point between the fire rate and the trigger rate) is
recorded in OQ-432; `06` OQ-621 owns the conflict with overview 6.2.

**Nonlinear use of a partial uptime.** A class (c) or (u) value with uptime `u_e < 1` enters linear uses (a bucket
whose contribution to the expected damage is linear) as `u_e·v·m_e`. Where the effect enters a product of two of its
own factors, or any nonlinear function, the EV evaluator uses the **mixture** (on with probability `u_e`, off with
`1 − u_e`, correlated for an instance and the DoT it seeds), never `f(E[x])` (overview CD-13). The case that matters:
Mecha Empowered and Vauban passive, which multiply a DoT twice (seed and tick, `M_dot`): `E = 1 + u·((1 + v)² − 1)`,
not `(1 + u·v)²` (TV-04-41). MC draws on/off intervals: an alternating process with the effect's on-time (Mecha: the
mark duration) and period (the mark interval), random phase per run, so that the uptime is `u_e` and an instance and
its DoT ticks see correlated states.

```text
E-4.9  EV steady state of a class (b) effect under `simulated` (Poisson trigger approximation)
  λ = p · (rate of trigger events)       on_hit: hit units/s of the effect's hit_unit (pellets/s, shots/s, or
                                            instances/s incl. extra hits, §4.1); on_crit: pellets/s·P(k ≥ 1);
                                            on_big_crit: P(k ≥ 2) per unit; on_status(s): λ_s (02);
                                            on_headshot: hit units/s if T-03 = head, else 0;
                                            on_weak_point_hit: hit units/s if T-03 ∈ {head, weak_point}, else 0;
                                            on_reload: reloads/s; radial instances and DoT ticks never count for
                                            on_headshot or on_weak_point_hit
  q = exp(−λ·D)
  single stack (m = 1, refresh):         E[value] = v·(1 − q)
  independent timers:                     E[stacks] = E[min(N, m)],  N ~ Poisson(λ·D)
  shared_lose_one: post-event state k (1..m) is a Markov chain
     k' = min(m, max(0, k − i) + g),   P(i) = q^i·(1 − q)     (i = number of full timeouts in the gap)
     time-average E[stacks] = Σ_k π_k · Σ_(i=0..k−1) (k − i)·(q^i − q^(i+1))
  shared_lose_all: E[stacks] = E[min(m, number of trigger events since the last gap longer than D)]
     (renewal; 08 computes it from the Poisson process, OQ-427)
  reset_on (Secondary Enervate): renewal-reward average over reset cycles, E-4.21 (§5.4)
  window mode: 08 walks the expectation through time with the same chain
```

The m = 1 case of the shared chain reduces to `1 − q`, matching the single-stack formula. EV results of E-4.9 are
approximations whose accuracy `08` checks against MC (overview section 4.4).

### 4.4 Strength at the caster

```text
E-4.10  Strength-scaled buff value
  value_e = listed_e(rank) · Str_e
  Str_e   = T-20 (the build's own Strength) for self-cast buffs; a per-buff caster Strength input (default 1.0)
            for buffs cast by squadmates; Parasitic Link multiplies: Str_new = Str_own·(1 + 0.25·Str_Nidus)
  T-13 … T-17 (Roar, Eclipse, Xata's Whisper, Toxic Lash, Nourish) enter final values (overview §6.1): no further
  Strength factor
  duration_e = listed duration_e · Dur_e   for dur_scaled effects; Dur_e = caster Ability Duration (default 1.0;
            self-cast: a scenario input next to T-20, proposed as a toggle in OQ-420)
```

Decaying buffs (Metamorphosis Day `+0.25·Str` decaying linearly to 0 over `25·Dur_e` s; Wrathful Advance `10·Dur_e` s;
Covenant `12·Dur_e` s; Redline by battery; Primal Rage and Valkyr Rage idle decay): a class (c) effect uses the
**peak** under `all_max` and `u_e·(time average)` under `simulated`; a class (u) buff uses the peak by default and
offers the time average as a user choice. The time average is taken over one cast cycle with a recast at expiry
(linear decay to 0: half the peak; Decision, UNVERIFIED). MC uses the same constant (external events are not
simulated, §4.3).

Strength sources are additive with each other (Intensify, Molt Augmented, Arcane Crepuscular, Crimson shard,
Growing Power, Power Donation …; `F:buffs` §0) except Parasitic Link (multiplicative). The engine does not derive
Str from the build by default: T-20 is the user's total. A "derive Strength from build" option is proposed in
OQ-420 with the strength sources of §5.

---

## 5. Effects catalogue

The curated effects data (overview section 1.5: "a hand-authored table keyed by mod identity") is built from this
section. Every entry has the record fields of §4.2 (`bucket` is an overview symbol, `value` is at max rank).
Buckets use the overview symbols; `A` = `sum_base_dmg`, `CC_rel` = `sum_cc_rel`, `CC_abs` = `sum_cc_abs`,
`CD_rel` = `sum_cd_rel`, `CD_abs` = `sum_cd_abs`, `SC_rel` = `sum_sc_rel`, `MS` = `sum_ms`, `E[t]` =
`sum_elem[t]` (non-combining unless stated), `F` = `sum_faction`, `A_sd` = `sum_status_dmg`, `A_sd,s` =
`status_dmg_type[s]`, `SD` = `sum_status_dur`, `FR`, `AS`, `RL` = fire rate, attack speed, reload speed buckets
(`06`, `07`), `Str` = strength source (§4.4), `V` = vulnerability (§5.7), `MS0+`, `SC0+` = flat base additions
(`02`), `M_dot` = DoT multiplier of E-13 (`03`). Class column: (a), (b), (c), (u) of §4.1; "–" = unconditional.

### 5.1 Unconditional weapon-mod stat lines

Unconditional mod values come from the data (Public Export `levelStats` text, wiki `Module:Mods/data`), not from
this document; this table fixes the bucket of each stat line so the importer can map the text.

| Stat line (examples) | Bucket | Owner of the arithmetic |
|---|---|---|
| "+X % Damage" on guns (Serration, Hornet Strike, Point Blank, Heavy Caliber, riven Damage) | A | `01` |
| "+X % Melee Damage" (Pressure Point family, Spoiled Strike, Sacrificial Pressure) | A (melee) | `01` |
| "+X % Melee Damage on Heavy Attack" (Killing Blow, riven splice) | A, gate: heavy attacks | `01`, `07` |
| "+X % Impact / Puncture / Slash" | `sum_phys[t]` | `01` |
| "+X % Heat / Cold / Electricity / Toxin" | element combination | `01` |
| "+X % Blast / Corrosive / Gas / Magnetic / Radiation / Viral" (riven splices, Biotic Rounds) | E[t] (non-combining) | `01` |
| "Damage to <Faction>" (Bane, Expel, Cleanse, Smite, riven) | F | `01` |
| "+X % Critical Chance" (melee lines "x2 for Heavy Attacks") | CC_rel (doubled on heavies where stated) | `02` |
| "+X % Critical Damage" | CD_rel | `02` |
| "+X % Multishot" | MS | `02` |
| "+X % Status Chance" | SC_rel | `02` |
| "+X % Status Duration" | SD | `03` |
| "+X % Status Damage" (Elementalist family) | A_sd | `03` |
| "+X % Weak Point Damage" (Acuity, Seek) | `sum_wp_dmg` | `02` |
| "+X % Weak Point Critical Chance" | CC_rel on weak-point hits | `02` |
| headshot multiplier lines (Deadhead "+30 % to Headshot Multiplier", Target Acquired) | `sum_hs_mult` | `02` |
| "+X % Fire Rate (x2 for Bows)", "+X % Attack Speed", reload, magazine, ammo efficiency, charge rate | FR, AS, RL, magazine, AE | `06`, `07` |
| "+X % Finisher Damage" | finisher factor | §7 |
| "+X % Slam Attack Damage" | Slam Damage factor in G | `07` |
| "+X % Direct Damage per Status" | C (§2) | here |
| zoom, recoil, accuracy, punch through, projectile speed, range, beam length, noise | no damage effect (zoom-level crit and headshot bonuses of a weapon are class (b) `while_aiming`, §5.12) | – |

### 5.2 Galvanized mods

Source: `F:mods` §6.1 (`W:Galvanized_Mods` and each mod page, `Module:Mods/data`). Stacks: on kill with the weapon
carrying the mod (status kills count), class (c); the stacked bonus applies only to that weapon; same mod on a normal
and an exalted weapon: both benefit (some exalted weapons keep separate stacks).

| Mod (class) | Unconditional part | Stacked part (per stack × max) | Timer | Bucket |
|---|---|---|---|---|
| Galvanized Chamber (rifle) | MS +0.80 | MS +0.30 × 5 (max +2.30) | 20 s, `shared_lose_one` | MS |
| Galvanized Diffusion (pistol) | MS +1.10 | MS +0.30 × 4 (max +2.30) | 20 s | MS |
| Galvanized Hell (shotgun) | MS +1.10 | MS +0.30 × 4 | 20 s | MS |
| Galvanized Aptitude (rifle) | SC_rel +0.80 | C +0.40·n × 2 | 20 s | SC_rel; C |
| Galvanized Savvy (shotgun) | SC_rel +0.80 | C +0.40·n × 2 | 20 s | SC_rel; C |
| Galvanized Shot (pistol) | SC_rel +0.80 | C +0.40·n × 3 | 14 s | SC_rel; C |
| Galvanized Scope (rifle) | on weak-point hit: CC_rel +1.20 while aiming, 12 s (b) | on weak-point kill (since U44.0.2): CC_rel +0.40 × 5, while aiming | 12 s, `independent` | CC_rel |
| Galvanized Crosshairs (pistol) | as Scope | +0.40 × 5 | 12 s, `independent` | CC_rel |
| Galvanized Acceleration (shotgun, exilus) | projectile speed +0.30 | +0.30 × 2 | 10 s | none (no damage) |
| Galvanized Steel (melee) | CC_rel +1.10 (×2 on heavies, heavy total +2.20 since U37.0.4) | CD_rel +0.30 × 4 | 20 s | CC_rel; CD_rel |
| Galvanized Elementalist (melee) | A_sd +0.80 | SC_rel +0.30 × 4 | 20 s | A_sd; SC_rel |
| Galvanized Reflex (melee) | heavy attack efficiency +0.50 | initial combo +20.46 × 4 (+81.84) | 20 s | `07` |

Galvanized Reflex values are the stats table of `W:Galvanized_Reflex` (rank 10: `+50% … +20.46 … +81.84`);
`Module:Mods/data` describes the same stack as "+20 Initial Combo" (rounded). Galvanized Steel and Elementalist stacks gained with archmelee do not apply there (bug, `F:mods` §6.1). Robotic weapons
gain stacks only from the player's kills with a weapon carrying the same mod.

### 5.3 Other conditional and special weapon mods

Values at max rank from `Module:Mods/data` (via `F:mods` §6.2) unless another source is named.

| Mod | Bucket and value | Class, trigger, duration, stacks | Source |
|---|---|---|---|
| Condition Overload (melee) | C +0.80·n | (a) | §2.1 |
| Blood Rush (melee) | CC_rel +0.40·(M(c) − 1) | (b) combo | `W:Blood_Rush` (U26 additive, U30.5 0.60 → 0.40) |
| Weeping Wounds (melee) | SC_rel +0.40·(M(c) − 1) | (b) combo | `W:Weeping_Wounds` |
| Berserker Fury (melee) | AS +0.35 per stack × 2 | (c) on melee kill, 10 s | `F:mods` §6.2 |
| Argon Scope (rifle) / Hydraulic Crosshairs (pistol) / Laser Sight (shotgun) | CC_rel +1.35 / +1.35 / +1.20 while aiming | (b) on weak-point hit, 9 s | idem; `W:Argon_Scope` |
| Bladed Rounds (rifle) / Sharpened Bullets (pistol) / Shrapnel Shot (shotgun) | CD_rel +1.20 / +0.75 / +0.99 while aiming | (c) on kill, 9 s | idem |
| Catalyzer Link / Embedded Catalyzer / Nano-Applicator | SC_rel +0.60 / +0.90 / +0.90 while aiming | (c) on ability cast, 9 s | idem |
| Spring-Loaded Chamber / Pressurized Magazine / Repeater Clip | FR +0.75 / +0.90 / +1.05 while aiming | (b) on reload, 9 s | idem |
| Primed Chamber / Charged Chamber (sniper) | G factor `1 + 1.00` / `1 + 0.40`, additive with each other | (b) first shot of magazine | §3.2 |
| Synth Charge (pistol, set) | G factor `1 + 2.00` | (b) final shot, base magazine ≥ 6 | §3.2 |
| Split Flights (bow, non-AoE) | MS +1.00 per stack × 4 (accuracy −1.80) | (b) on hit, 2 s | `F:mods` §6.2 |
| Spectral Serration (rifle) | A +3.30 | (c) while invisible | idem |
| Motus Setup (shotgun) | CC_rel +1.00, SC_rel +1.00 | (c) after a double / bullet jump landing, 4 s | idem |
| Mark of the Beast (thrown melee) | secondary CC_rel +1.20, SC_rel +1.20 | (c) 6 melee kills in 6 s, 24 s | idem |
| Deadly Efficiency (archgun) | A +1.20 | (b) on reload from empty, 9 s | idem |
| Critical Focus / Marked Target (archgun) | CC_rel +0.60 and CD_rel +0.60 / SC_rel +1.20 while aiming | (c) aiming | idem |
| Proton Snap (melee) | Toxin element +1.00, SC_rel +0.50 | (c) after 2 s wall latch, 20 s | idem |
| Killing Blow, other "Melee Damage on Heavy Attack" | A, gate heavy | – | §5.1 |
| Tennokai mods (Master's Edge +0.60 Tennokai damage, Truth's Flame +1.20, Dreamer's Wrath +0.32 crit damage, Condition's Perfection +1.00 status chance) | Tennokai attacks only | gate | `07`; `F:mods` §6.2 |
| Finishing Touch / Covert Lethality | finisher factor +0.60 / +1.00 (additive: "+160 %") | gate finisher | §7 |
| Seismic Wave; Nira's set | Slam Damage +2.00; +1.50 per piece (U39) | gate slam | `07`; `W:Melee` §Slam Damage |
| Power Throw (glaive) | A_w +1.00 per consecutive buffered throw, 3 stacks | (b) | `W:Power_Throw` §Notes ("multiplicative with Pressure Point, additive with universal weapon damage") |
| Hunter Munitions, Internal Bleeding | forced Slash 0.30 per crit instance; 0.35 (0.70 below 2.5 fire rate) per Impact proc | (b) | `02` owns the rolls; D-016 |
| Empowered Blades (aura) | SC_rel +0.60 and A_sd +0.60 on heavy attacks | gate heavy | §5.8 |
| Double Tap (Latron augment) | G factor `1 + 0.20·min(20, H − 1)` (§3.2) | (b) on hit, `hit_unit = pellet`, 2 s, `reset_on = (on_miss, 1)` | `W:Double_Tap` §Notes |
| Deadly Maneuvers (Magnus augment) | final crit-chance factor `CC ← CC·(1 + 4.00)` after E-5 (custom-stat key `mult_crit_chance`), headshots only, for the next 2 shots after a dodge | (c) `on_dodge`, `consume_on = next_shots(2)`; gate headshot | `W:Deadly_Maneuvers` (fetched 2026-11-23: rank 3 "+400%" headshot critical chance, "multiplicatively applied after buffs"); its example (108.14 % → 648.84 %, a factor 6) disagrees with the rank table (factor 5): OQ-438 |
| Maiming Strike (melee) | CC_rel +1.50 on slide attacks, additive with True Steel and Blood Rush | gate slide | `W:Maiming_Strike` (fetched 2026-11-23; `Module:Mods/data` "+150% Critical Chance for Slide Attack"); D-414 |
| Burning Hate (Hate augment) | `M_dot` factor `(1 + 1.20)` on status procs **created after** a Hate hit damaged a Heat-affected target; persists after Heat expires; procs from any source; additive with Pageflight's status-damage vulnerability | (a) `target_affected_by(HEA)` at the Hate hit, then permanent | `W:Burning_Hate` §Notes (cached; "Status Damage from all sources", "only applies to status procs dealt after the vulnerability is applied"); D-417 |
| Hunter's Bonesaw (Ripkas augment) | finisher factor +1.20 vs prone (knocked-down) enemies | gate finisher (finisher mode, §7.2) | `Module:Mods/data` ("Increase Finisher Damage by 120% against prone enemies") |
| Combo Fury (melee) | secondary RL +1.00 and secondary magazine +1.00 for 12 s | (c) on melee kill; timing stat (§4.3) | `Module:Mods/data` |
| Meticulous Aim (Vulkar augment) | `sum_hs_mult` +1.05 on headshots; bodyshot penalty `part_body_pen` 0.45 on non-headshot hits (arithmetic `02`) | – (gate: headshot / body hit) | `W:Meticulous_Aim` (cached) rank table; `02` ΣHS, ΣBP |
| Piercing Navigator (Ivara augment) | CC_abs +0.50 per enemy already hit by the Navigator-controlled projectile, cap +2.50, not Str-scaled, "applying after mods"; on a single target only hits before the target count (user count, default 0; UNVERIFIED whether the target's own hit counts) | (u) | `W:Piercing_Navigator` (cached) lines 3, 27-29 |
| Napalm Grenades (Penta augment) | lingering burning patch on impact: a lingering AoE child (`06` §9.4), multi-target in effect; no single-target value here | – | `Module:Mods/data` |

### 5.4 Weapon arcanes

Source: `M:Arcane` descriptions (2026-11-21) and pages cited in `F:addmul` §5, `F:melee` §7. Max rank 5 (Zaw,
Kitgun: 3). Arcane-slot rules (primary arcanes on primaries only; on archguns the primary and secondary slots accept
those arcanes, but class-restricted *warframe* arcanes such as Arcane Precision and Acceleration do not apply because
archguns are neither primary nor secondary, `W:Archgun`) belong to `09`; the engine applies an arcane only to the
weapon classes its description names.

| Arcane | Bucket and value | Class, trigger, duration, stacks | Notes and source |
|---|---|---|---|
| Primary / Secondary Merciless | A +0.30 per stack × 12 (+3.60); RL +0.30 (rank 5 only) | (c) on kill, 4 s, `shared_lose_one` | `W:Primary_Merciless` |
| Primary / Secondary Deadhead | A +1.20 per stack × 3 (+3.60); `sum_hs_mult` +0.30 (unconditional); recoil −0.50 | (c) on weak-point kill, 24 s, `shared_lose_one` | `W:Primary_Deadhead` §Notes (HS bonus additive with Target Acquired; line 32 timer) |
| Primary / Secondary Dexterity | A +0.60 per stack × 6 (+3.60); combo duration +7.5 s | (c) on melee kill, 20 s, `shared_lose_one` | `W:Primary_Dexterity` (line 31 timer) |
| Primary Frostbite | CD_rel +0.03 and MS +0.0225 per stack × 40 (+1.20, +0.90) | (b) on Cold status, 12 s, `shared_lose_all`; Cold procs from **any** source add stacks, even while the primary is holstered (external Cold procs, T-06-like inputs and allies, enter as a class (c) proc-rate input); no stack from a Cold proc on a fully Frozen target (10 stacks), which cannot receive one | `W:Primary_Frostbite` §Notes ("does not require the Cold proc to come from the attached primary weapon. It can come from any source"; "fully Frozen enemies … do not have any interaction with Frostbite") |
| Primary Blight | CD_rel +0.036 and MS +0.018 per stack × 40 | (b) on Toxin status from the primary weapon only, 12 s, `shared_lose_all` | `M:Arcane`; `W:Primary_Frostbite` §Notes (Blight "can only be stacked from primary weapon Toxin status effects") |
| Conjunction Voltage (secondary) | MS +0.03 and RL +0.015 per stack × 40 | (b) on Electricity status, 12 s | `M:Arcane` |
| Primary Crux | SC_rel +0.30 and AE +0.06 per stack × 10 | (b) on weak-point hit, 10 s | `M:Arcane` |
| Primary Bulwark | A +0.01 per armor point above 1000, max +5.00 (bucket UNVERIFIED) | (c) `armor_level` | `M:Arcane` |
| Primary Overcharge | MS +0.0035·E_max, cap +3.50 | (c) while energy ≥ 90 % | `M:Arcane` ("35 % of Max Energy as Multishot") |
| Primary Plated Round | A `+0.15·sqrt(5·Mag)·(Mag − left)/Mag` (Mag = modded magazine, left = rounds left before the reload); the value does not depend on rank, the duration does (3 / 4 / 5 / 6 / 8 / 10 s) | (b) on reload, 10 s | `W:Primary_Plated_Round` ("additive to base damage sources such as Serration"; 25 rounds → +167.71 %; rank table); D-401 |
| Primary Compression | §3.1 | (b) aiming | `W:Primary_Compression` |
| Primary Debilitate | 0-damage extra hit with a forced proc of one component when a combined status at 10 stacks is applied again | (a) | `W:Extra_Hit` §Weapon Effects; §6.5 |
| Longbow Sharpshot (bow) | G_F `1 + 3.00` on the next shot | (b) after a weak-point hit | §3.1 |
| Shotgun Vendetta | MS +1.80, RL +0.75 (additivity with mods UNVERIFIED) | (c) on shotgun kill within 5 m, 15 s | `W:Shotgun_Vendetta` |
| Fractalized Reset | RL +2.40 | (c) on ability cast, 5 s | `M:Arcane` |
| Primary Exhilarate, Primary Obstruct, Akimbo Slip Shot | no damage | – | `M:Arcane` |
| Secondary Outburst | CC_rel +0.20 and CD_rel +0.20 per combo multiplier consumed (12× → +2.40 each) | (c) on swap to secondary, 30 s | `W:Secondary_Outburst` |
| Secondary Kinship | CC_rel +0.20 per ally-warframe buff | (c) `ally_buffs` | `W:Secondary_Kinship` |
| Secondary Enervate | CC_abs `+0.10·h` (h = hits since the last reset, before this hit); reset after rank + 1 (6) big crits (k ≥ 2); E-4.21 | (b) on hit, `hit_unit = shot`, `max_stacks = none`, `reset_on = (on_big_crit, 6)`; evaluated as `simulated` also under `all_max` (§4.3) | `W:Secondary_Enervate` §Notes ("Critical chance is an absolute bonus": Lato 10 % + 7 × 10 % = 80 %); 02 ΣCC_abs, EC-02-23; JS L10673-10696 |
| Secondary Encumber | 0.24 chance per proc for one extra random status | (b) | `02` rolls; D-021 |
| Secondary Cryogenic | 3 Cold stacks on Puncture proc | (b) | `03` |
| Secondary Shiver | C +0.45 per Cold stack | (a) | §2.1 |
| Secondary Fortifier | `M_OG` ×8 vs overguard (×3 at rank 0) | – | overview E-12; `W:Secondary_Fortifier` |
| Secondary Surge | G `1 + min(2.00 + 1.00·rank, Σ_casts 0.005·E_after)` on the next shot (§3.2) | (c) on ability cast, `consume_on = next_shot` | `W:Secondary_Surge` §Notes |
| Secondary Irradiate | 180 % of the hit to *other* enemies within 7 m vs 10 Radiation stacks | multi-target only | `M:Arcane` |
| Cascadia Flare | +0.12 per stack, up to +4.80 (bucket A UNVERIFIED) | (b) on Heat status from any source, 10 s, `shared_lose_all` | `M:Arcane`; `W:Cascadia_Flare` lines 34, 55 |
| Cascadia Accuracy | CC_rel +3.00 on weak-point hits | (c) on roll, 4 s | `M:Arcane` |
| Cascadia Overcharge | CC_rel +3.00 | (c) while overshields | `M:Arcane` |
| Cascadia Empowered | +750 flat damage of the status's type per status proc (no crit; scaled by faction, type modifier, Viral or Magnetic; UNVERIFIED, inspiration model JS L10240-10253) | (b) on status | `M:Arcane` |
| Melee Animosity | CC_rel +0.42 per normal melee hit, max +4.20, next heavy only | (b) | `F:melee` §7 |
| Melee Assimilation | A +1.50 on heavy attacks | (c) after a shield break, 20 s; gate heavy | idem |
| Melee Careen | G ×2.5 vs Frozen | (a) | §3.2 |
| Melee Doughty | CD_abs `doughty = min(50, round1(10·(r_PUN/Σ_t r_t)·SC_w·b_r))`, b_r = 0.5 + 0.1·rank (1.0 at rank 5), round1 = round half up to 0.1; r_t the modded unquantized composition (`01`), SC_w the final status chance including combo-scaled bonuses (Weeping Wounds); applies only to the Puncture-bearing part of an attack (U38.6) | – | `W:Melee_Doughty` §Notes (formula, "Buff is rounded to one decimal place", cap +50x; Destreza Prime example +9.7x, TV-04-22); bucket and formula as `02` E-2.5a (`sum_cd_abs`, "Critical Multiplier +Nx" wording; UNVERIFIED, OQ-209) |
| Melee Duplicate | §6.4 | (b) on tier-1 crit | `W:Melee_Duplicate` |
| Melee Exposure | Corrosive E[COR] +0.60 per stack × 4 (+2.40), shared 25 s timer | (c) on ability cast | `F:melee` §7 |
| Melee Retaliation | A +0.30 per 200 current shields, max +4.20 (overshields count half) | (c) `shield_level` | idem |
| Melee Afflictions | +6 stacks (rank + 1) of every status present on a melee-knocked-down, ragdolled or lifted target, with an averaged seed | (b) | §6.6 (E-4.20) |
| Melee Influence, Melee Vortex, Melee Fortification, Melee Crescendo | multi-target, status spread, no damage or combo only | – | idem (`07` for Crescendo) |
| Exodia Contagion | an attack with `mproj_mode = two_bucket`, `multiplying` | – | catalogue row 155 |
| Exodia Force | 200 % weapon damage to enemies within 6 m on status (multi-target; effect on the struck target UNVERIFIED) | (b) 0.50 | `M:Arcane` |
| Exodia Triumph, Valor, Brave, Hunt, Might, Epidemic | combo count, energy, utility | – | `M:Arcane` |
| Pax and Residual (kitgun) arcanes | no weapon damage | – | `M:Arcane` |

**Secondary Enervate.**

```text
E-4.21  Secondary Enervate (rank r; W:Secondary_Enervate §Effect, §Notes)
  state   : h = hit units since the last reset, b = big-crit units since the last reset (both 0 at t = 0)
  hit unit: one per shot on the target (hitscan multishot pellets and the pellets of a shotgun sidearm count once;
            projectiles of one shot register once within the game's grace period), one per AoE explosion, one per
            expiring Blast proc (not detonations at the stack cap or on death), one per extra hit (§4.1, OQ-436);
            at most 30 units per second
  each unit at time τ: every pellet of the unit uses CC = CC_E-5 + 0.10·h  (h before this unit; ΣCC_abs)
            then h ← h + 1; if any pellet of the unit has k ≥ 2: b ← b + 1 (big crits within one game tick count
            once); if b = r + 1: h ← 0, b ← 0
  MC      : exactly as above, with the unit list of the schedule (06) and the rolled tiers (02 E-6)
  EV      : the per-unit expected bonus is the renewal-reward ratio over reset cycles
            CC̄_E = E[Σ_cycle 0.10·h] / E[units per cycle],  with P(unit is big) = 1 − Π_pellets (1 − P(k ≥ 2 | CC))
            computed by recursion over (h, b) (08); the direct-damage EV uses CC̄_E in E[c], which is linear in CC
  all_max : the same as simulated (no maximum exists, §4.3); none: 0
```

Hit-unit evidence: "Hitscan attacks that hit multiple enemies simultaneously (e.g., because of Multishot …) count as
only one hit"; "When used with a Shotgun Sidearm … each pellet does not count as a separate damage instance as the
pellets are tied to Multishot"; projectiles fired "in a delayed fashion" occasionally count more than once "because
the game seems to use a grace period where projectiles from the same shot won't register more times"; "AoE attacks
always count as one hit"; "Damage dealt by expiring Blast procs counts as a hit"; "Multiple big critical hits
registered within 1 game tick only count as 1 stack for the reset counter"; "cannot trigger more than 30 times per
second". The page's AFCC table (average final crit chance, hitscan, no multishot) is the mean of each reset cycle's
average CC, `E[S/L]`; the long-run mean per hit, which is what DPS needs, is `E[S]/E[L]`, a little higher below
CC 2.0 (CC 1.0: 1.531938 vs the page's 1.5245; equal from CC 2.0, where every hit is a big crit). The engine uses
`E[S]/E[L]` (Decision; a 2 × 10^6-hit simulation of the rule reproduces it). TV-04-21 holds both.

### 5.5 Warframe arcanes

| Arcane | Bucket and value | Class, trigger | Notes and source |
|---|---|---|---|
| Arcane Avenger | CC_abs +0.45, all weapons | (c) on damaged 21 %, 12 s | `W:Arcane_Avenger` (`25 % × 2.2 + 60 % + 45 % = 160 %`) |
| Arcane Fury | A +1.80 melee | (b) on crit 60 %, 18 s | `W:Arcane_Fury` |
| Arcane Strike | AS +0.60 melee (also exalted melee) | (b) on hit 15 %, 18 s | `W:Arcane_Strike` |
| Arcane Blade Charger | A +3.00 melee | (c) on primary kill 30 %, 12 s | `F:melee` §7 |
| Arcane Primary Charger | A +3.00 primary | (c) on melee kill 30 %, 12 s | `W:Arcane_Primary_Charger` |
| Arcane Rage | A +1.80 primary | (b) on headshot 15 %, 24 s | `W:Arcane_Rage` |
| Arcane Rise | A +1.50 primary (bucket by analogy with Awakening, UNVERIFIED) | (b) on reload 60 %, 24 s | `M:Arcane` |
| Arcane Awakening | A +1.50 pistols | (b) on reload 60 %, 24 s | `W:Arcane_Awakening` |
| Arcane Precision | A +3.00 secondary | (b) on weak-point hit (since U44; was headshot), 18 s, no refresh | `W:Arcane_Precision` |
| Arcane Arachne | A +1.50 primary and secondary (melee projectiles: §3.4; not normal melee) | (c) on wall latch, 30 s, no refresh | `W:Arcane_Arachne` |
| Arcane Acceleration | FR +0.90 primaries except shotguns, not doubled for bows | (b) on crit 30 %, 9 s | `W:Arcane_Acceleration` |
| Arcane Velocity | FR +1.20 pistols | (b) on crit 90 %, 9 s | `W:Arcane_Velocity` |
| Arcane Tempo | FR +0.90 shotguns | (b) on crit 15 %, 12 s | `W:Arcane_Tempo` |
| Arcane Momentum | RL +1.50 sniper rifles (reload only; timing stat, §4.3) | (b) on crit 60 %, 12 s | `M:Arcane`; D-416 |
| Arcane Pistoleer | infinite ammo 12 s (since Hotfix 44.1.3) | (c) on pistol weak-point kill 60 % | no damage bucket; `06` duty (§5.9 infinite-ammo state); D-415 |
| Arcane Hot Shot | CC_rel +0.06 per stack × 50 (+3.00), all weapon types | (c) per Heat status inflicted by abilities, 10 s | `W:Arcane_Hot_Shot` |
| Arcane Crepuscular | CD_abs +3.0 (`1 + k·(CDM − 1 + 3)`); Str +0.30 | (c) while invisible | `W:Arcane_Crepuscular`; not on Gyre abilities |
| Molt Augmented | Str +0.0024 per kill × 250 (+0.60), kept until death | (c) | `W:Molt_Augmented` |
| Molt Vigor; Arcane Bellicose; Power Ramp; Impetus; Ice Storm; Camisado | Str +0.45 next cast; `0.01·min(72, round(6·MaxHealth/250))` (percent rounded half up, then capped; rank r: `(r + 1)·MaxHealth/250`, cap `12·(r + 1)`); +0.09 × 4; +0.06 per unique ability status; +0.02 × 20; up to +0.60 | (c); Bellicose unconditional given the health input | `M:Arcane`; `W:Arcane_Bellicose` lines 39-44 (`round(Max Health ÷ 250 × Strength Increase)`; "2,980 instead of 3,000"; 2,480 health matches +60 %), TV-04-23 |
| Theorem Demulcent | +0.12 weapon damage per second in a Residual zone × 15 (+1.80), persists 20 s (bucket UNVERIFIED, page empty) | (c) `in_zone` | `M:Arcane` |
| Theorem Contagion | V additive group +2.00 to the orb's damage type, 6 s | (c) | `M:Arcane`; `W:Damage_Vulnerability` §Notes |
| Magus Accelerant / Magus Destruct (operator) | V additive group +0.65 per application, Heat / Puncture only, permanent, no limit | (c) count of applications | `W:Magus_Accelerant` (Ignis Wraith example `35 × 2.65 × 1.9 × (1 + 3 × 0.65)`) |

### 5.6 Warframe ability buffs and Helminth values

Values at the maximum ability rank, `× Str_e` where marked (§4.4). Subsumed = Helminth value. Class (u) unless stated
(the user enables the buff; uptime, stacks and caster are user inputs; §4.1). Target-state sub-conditions ("vs
Doomed", "vs Reap-affected") are class (a). Sources: `F:buffs` §1, §4, §7 and the cited ability pages.

| Buff (frame) | Bucket | Value | Subsumed | Notes | Source |
|---|---|---|---|---|---|
| Roar (Rhino) | F | +0.50 × Str | +0.30 × Str | all weapons and abilities; T-13 enters final; strongest Roar overwrites | `W:Roar` line 15 ("considered Faction Damage Bonus, additive with other sources of Faction Damage"), line 18 example `100 × (1 + 1.65) × (1 + 0.5 + 0.3)`; Strength example `100 × 2.65 × 1.95 = 516.75` from `F:addmul` §3 (constructed there with Intensify 1.3) |
| Eclipse, Solar (Mirage) | G_F | +2.00 × Str | +0.30 × Str (U35.5) | self only (companions and clones with Total Eclipse); twice on `two_bucket` | `W:Eclipse` |
| Xata's Whisper (Xaku) | extra hit VOI | 0.26 × Str | same | §6 | `W:Xata's_Whisper` |
| Toxic Lash (Saryn) | extra hit TOX | 0.30 × Str (×2 melee) | not subsumable | §6; Saryn passive SD +0.25 on its procs (D-037) | `W:Toxic_Lash` |
| Nourish (Grendel) | E[VIR] | +0.75 × Str | +0.45 × Str (U35.5) | already combined, does not scale Toxin or Cold DoT | `W:Nourish` |
| Vex Armor, Fury (Chroma) | A | up to +2.75 × Str (2.75 % per HP lost or 15 % per ranged kill) | not subsumable | both buckets on melee projectiles; several Chromas stack | `W:Vex_Armor` (`100 × (1 + 1.65 + 2.75 × 1.3) = 622.5`) |
| Elemental Ward, Toxin (Chroma) | A | +0.35 × Str for 3 s after holster | same | universal | `W:Elemental_Ward` |
| Amp (Octavia) | A | +0.25 to +2.00 × Str by sound level (user value) | – | only the highest Amp applies | `W:Amp_(Ability)` |
| Metronome Forte / Opera (Octavia) | A (melee) / MS (ranged) | +0.30 × Str each | – | rhythm-gated | `W:Metronome` |
| Shooting Gallery (Mesa) | A | +0.25 × Str | same | ranged and melee | `W:Shooting_Gallery` |
| Symphony of Mercy, Deathbringer (Jade) | A | +1.00 × Str | – | Power of the Seven: Str +0.25 × Str | `W:Symphony_of_Mercy` |
| Metamorphosis, Day (Equinox) | A | +0.25 × Str, decays linearly to 0 over 25 s × Duration; not melee | – | time average = half | `W:Metamorphosis` |
| Rift Torrent (Limbo augment) | A | +0.30 × Str per enemy in Rift Surge | – | user count | `W:Universal_Weapon_Bonuses` |
| Absorb (Nyx) | A (UNVERIFIED) | up to +4.00 after release, 8 s; `sqrt(0.00025·Str·AbsorbedDamage)` unit UNVERIFIED | – | user value | `W:Absorb` |
| Valkyr Rage (passive) | A melee | up to +3.00 (+0.03 per hit, +0.12 per kill, +0.27 per finisher; decays after 5 s idle) | – | additive with Pressure Point and CO | `W:Valkyr/Abilities/Passive` |
| Excalibur Swordsmanship (passive) | A melee +0.10, AS +0.10 | swords, dual swords, nikanas, rapiers, Exalted Blade | – | | `F:buffs` §1.20 |
| Kinetic Plating + Redline (Gauss) | A melee +1.00 above 80 % battery | – | – | not Str-scaled | `W:Kinetic_Plating` |
| Garuda passive, Parasitic Link, Overdriver | G_F | §3.1 | – | | §3.1 |
| Vauban passive, Navigator | G | §3.2 | – | | §3.2 |
| Covenant (Harrow) | CC_abs | `(0.05 + 0.015·absorbed/100)·Str`, ×4 on weak-point hits, cap 0.50 body / 2.00 weak point | not subsumable | allies and companions | `W:Covenant` (`10 % × 2.5 + 200 % = 225 %`) |
| Wrathful Advance (Kullervo) | CC_abs, melee only | +2.00 × Str | +1.00 × Str (also exalted melee) | 10 s × Duration after cast | `W:Wrathful_Advance` |
| Grave Spirit (Dagath) | CD_rel | +0.50 × Str; +1.00 × Str vs Doomed targets (max rank; whether it replaces or adds to the 0.50 is UNVERIFIED, Decision: replaces, OQ-440) | – | self only | `W:Grave_Spirit` line 10 ("35 / 40 / 45 / 50 % critical damage bonus, 70 / 80 / 90 / 100 % Doom critical damage bonus") |
| Prismatic Gem (Citrine) | SC_rel +1.00 × Str; SD +1.00 × Duration | – | – | multiple Citrines stack | `W:Prismatic_Gem` |
| Haste Mote (Wisp) | FR +0.30 × Str; AS +0.20 × Str | – | – | additivity with mods UNVERIFIED | `W:Reservoirs` |
| Warcry (Valkyr) | AS +0.50 × Str | +0.30 × Str | – | | `W:Warcry` |
| Redline (Gauss) | FR 0.15-0.75, AS 0.08-0.40, RL 0.10-0.50 by battery and Duration | – | – | user value (curve UNVERIFIED) | `W:Redline` |
| Electric Shield (Volt) | E[ELE] +0.50 per shield passed (not Str-scaled, non-combining, no Electricity DoT scaling); `cd_field_mult` ×2 (does not stack) | – | – | projectile, hitscan and beam shots of guns and heavy weapons, and projectile-based melee attacks (gunblade shots, warfan heavy projectiles, Syam shockwave, Innodem dagger, Nepheri fireballs); not melee swings, not glaive throws; AoE only if the main projectile passed through; one field source per shot | `W:Buffing_Fields` intro ("Projectile, Hitscan, and Continuous Weapon\|Beam shots from Weapons"); `W:Electric_Shield` lines 14-25 |
| Conductive Sphere (Gyre augment) | E[ELE] +0.75 × Str per field; A_sd,ELE +0.75 × Str; +1 forced Electricity proc per field | – | – | | `F:buffs` §1.12 |
| Energy Shell (Odonata) / Assimilate (Nyx) | E[HEA] +0.50 and ×2 CD / ×2 CD | – | – | fields | idem |
| Valence Formation (Lavos augment) | E[elem] +2.00 non-combining; +2.00 to that element's DoT on all procs | – | – | not on Blast DoT | `W:Valence_Formation` |
| Plunder (Hydroid) | E[COR] +0.10 per enemy and consumed Corrosive stack, cap +2.00, × Str | – | – | combining behaviour UNVERIFIED | `W:Plunder` |
| Sonar (Banshee) | `sonar_mult` ×5 × Str on the highlighted part | – | – | `02`; not inherited by DoT (overview E-13 `Part_dot = Part / M_son`) | value `W:Sonar`; the DoT rule `W:Damage_over_Time` line 171 ("Weakspots generated by Sonar or Detect Vulnerability only increase damage of the initial hit, not the resulting proc damage"); `W:Sonar` line 25 says the opposite ("DoTs … will have their damage calculated on the resulting damage if proc'd from a hit that hits a Sonar weakspot"): OQ-430 |
| Resupply (Cyte-09), Silken Stride (Oraxia), Demonium (Uriel), Sangodae (Narin), Gastroparesis | extra hits | §6.1 | – | | `W:Extra_Hit` |
| Peacemaker (Mesa, exalted) | A +1.50 × Str innate; ramp G | – | – | `50 × (1 + 1.5·min(n_b, 20)/20) × (1 + 1.5·Str + A) × (1 + Elem)`, n_b = burst index (2 shots per burst, §3.2) | `W:Peacemaker` lines 22-35; TV-04-20 |
| Neutralizer (Cyte-09), Lizzie (Temple) | strength multiplier 1 / 1.08 / 1.16 / 1.25 × Str "additive to base damage mods" (form UNVERIFIED) | – | – | `06` exalted | `F:weapon` §14 |
| Fireball Frenzy (Ember augment) | E[HEA] | +1.00 × Str | – | non-combining; `dot_scaling` yes ("does contribute to DoT"); several do not stack (highest Strength wins); caster self or ally | `W:Fireball_Frenzy` lines 14-33; `01` §4.10.3, D-107 |
| Shock Trooper (Volt augment) | E[ELE] | +1.00 × Str | – | non-combining; `dot_scaling` yes (U38 fix "not buffing Status Effect damage (e.g., Electricity ticks)"); highest Strength wins | `W:Shock_Trooper` lines 14-46 |
| Freeze Force (Frost augment) | E[COL] | +1.00 × Str | – | non-combining; Cold has no DoT; highest Strength wins | `W:Freeze_Force` lines 13-31 |
| Thermal Transfer (Gauss augment) | E[HEA] (tap) and E[COL] (hold), each +0.75 × Str; both active: E[BLA] = their sum (+1.50 × Str) instead | +0.75 × Str per field | – | non-combining; `dot_scaling`: Heat yes, Blast no | `W:Thermal_Transfer` lines 11-36; `01` D-107 |
| Venom Dose (Saryn augment) | E[COR] | +1.00 × Str | – | non-combining ("100 Impact, 90 Toxin, and 110 Corrosive"); highest Strength wins | `W:Venom_Dose` lines 14-38 |
| Smite Infusion (Oberon augment) | E[RAD] | +1.00 × Str | – | non-combining; highest Strength wins | `W:Smite_Infusion` lines 13-37 |
| Ripper's Wail (Temple) | E[HEA] | +0.75 × Str, plus +0.05 × Str per Backbeat-timed hit or recast, cap +7.50 (user value; the decay rate "?%/s" is not documented) | – | non-combining ("does not combine with a weapon's existing elements"), "additive with mods"; `dot_scaling` UNVERIFIED (Decision: yes, as the U38 rule for elemental augments; OQ-435) | `W:Ripper's_Wail` (cached) lines 9-21 |
| Prowl (Ivara) | `sum_hs_mult` | +0.40 × Str on headshots while Prowl is active | – | "stacks additively with other headshot bonuses"; breaking invisibility does not remove it | `W:Prowl` (fetched 2026-11-23) lines 11-16, 51 |
| Shadow Haze (Sevagoth augment) | CC_abs | +0.50 × Str vs Reap- or Death's-Harvest-affected targets | – | "applied additively after mods, the same way as Arcane Avenger and Covenant"; Sevagoth's Strength | `W:Shadow_Haze` (fetched) lines 10-33 |
| Ballistic Bullseye (Mesa augment) | CC_abs | up to +0.50 × Str by charge amount (user charge fraction) | – | "applied after mods"; all projectiles of the boosted shot | `W:Ballistic_Bullseye` (fetched) lines 10-29 |
| Crystallize (Citrine) | CC set to 3.00 on hits to a crystal growth (ignores every other crit-chance modifier; Vigilante can still raise the tier); hitscan weapons ×2 crit damage on those hits | – | – | per hit on the growth (user hit fraction) | `W:Crystallize` (fetched) lines 13-24 |
| Shroud of Dynar (Voruna) | melee CD_abs +2.0 × Str, CC_abs +1.00, SC_abs +1.00, forced Slash per melee hit | – | – | 3 s melee phase after invisibility (extended by melee kills) | `W:Shroud_of_Dynar` (fetched) lines 12-22 |
| Enraged (Valkyr augment) | A +3.50 and CC_rel +3.50 | – | – | Hysteria (Valkyr Talons) only; "not affected by mods", "stack additively with mods like Pressure Point and True Steel" | `W:Enraged` (fetched) lines 10-31; D-413 |
| Champion's Blessing (Trinity augment) | CC_rel up to +3.50 primary and secondary | – | – | caster only; "additive with other sources of critical chance bonus" | `W:Champion's_Blessing` (fetched) lines 8-32 |
| Smoke Shadow (Ash augment) | CC_rel +1.50 while invisible from Smoke Screen | – | – | all weapons; "additive to mods like Point Strike"; "not affected by mods"; several Ashes stack to +6.00 | `W:Smoke_Shadow` (fetched) lines 11-35 |
| Primal Rage (Wukong augment) | CC_rel +0.15 × Str per kill, cap +1.50 | – | – | Iron Staff (Primal Fury) only; decays −0.01/s after 5 s without a kill (`decay = idle`) | `W:Primal_Rage` (fetched) lines 9-30 |
| Merulina Guardian (Yareli augment) | secondary FR +2.00 and RL +2.00 for 20 s | – | – | timing stats (§4.3); Semi-Pistol Cannonade blocks the fire rate part | `W:Merulina_Guardian` (fetched) lines 14-42 |
| Weapon-made buffing fields: Mutalist Quanta alt-fire orb / Simulor orb or singularity | E[ELE] +1.00 / +3.66 / +5.00 (Quanta, 1-3 merged orbs) or +1.00 / +3.00 / +4.00 / +5.00 (Simulor, 1-4) per field, non-combining; CC_abs +0.25 per field; CD_rel +0.25; G ×0.66 per field (§3.2) | – | – | one buffing-field source per shot (a shot through a weapon field takes no other field's buffs); user field count | `W:Buffing_Fields` §Sources |
| Weapon-made buffing field: Mandonel charged shot | P_t +1.00 on IPS; Radiation "+100% (multiplicative)" (form UNVERIFIED: Decision, `E[RAD]` +1.00); CC_abs +0.10; `cd_field_mult` ×1.5 per field; `sc_mult` ×2 | – | – | one field source per shot | `W:Buffing_Fields` §Sources |

Helminth slot restriction (subsumed Roar, Eclipse, Xata's Whisper cannot coexist with the host frame's own damage
buff) is legality (`09`).

### 5.7 Target vulnerabilities, debuff abilities and armor strips

Stage arithmetic (overview E-12, owner `05`): `V = Π_i (1 + V_mult,i) · (1 + ΣV_add)` on the layers the source
affects; type-restricted entries multiply only that type's `D_t`; vulnerability is evaluated when each tick lands and
does not double-dip DoT (`W:Damage_Vulnerability` §Mechanics, §Notes). Since U37, Vast Untime, Magnetize, Beguiling
Lantern, Firing Line, Mass Vitrify, Splinter Storm, Blazing Chakram and Molecular Prime also apply to shields and
overguard (patch history on the same page).

Columns: `Group` mult (`V_mult`) or add (`ΣV_add`); `Type mask` (`all` or the damage types it multiplies, data
`type_mask`); `Layer mask` (data `layer_mask`; `{HP}` unless the U37 list above names it, then `{OG, SH, HP}`);
`Class` per §4.1 (every entry is a user-enabled debuff, class (u), with its own conditions as noted). Values at max
ability rank; "× Str" = caster Strength (§4.4).

| Source | Value | Group | Type mask | Layer mask | Condition, scope | Source |
|---|---|---|---|---|---|---|
| Molecular Prime (Nova) | +1.00 (fixed) | mult | all | OG, SH, HP | – | `W:Molecular_Prime` |
| Blazing Chakram (Nezha) | +1.00 × Str (ranks 0.60 / 0.75 / 0.85 / 1.00) | mult | all | OG, SH, HP | – | `W:Blazing_Chakram` |
| Strangledome (Khora) | +1.00 (infobox says 2.00, OQ-417) | mult | all | HP | not Whipclaw | `W:Strangledome` |
| Magnetize (Mag) | factor ×2.0 at Str 1 (ranks ×1.25 / 1.5 / 1.75 / 2.0), V_mult = 1.0; how Strength scales it UNVERIFIED (OQ-417) | mult | all | OG, SH, HP | main target | `W:Magnetize` |
| Rage (Equinox) | +0.50 × Str | mult | all | HP | – | `W:Rest_&_Rage` |
| Razorwing drones (Titania) | +1.00 | mult | all | HP | – | `W:Razorwing` |
| Paralysis (Valkyr) | +0.50 × Str | mult | all | HP | melee instances only | `W:Paralysis` |
| Mass Vitrify (Gara) | +0.50 × Str | mult | all | OG, SH, HP | – | `W:Mass_Vitrify` |
| Splinter Storm (Gara) | +0.35 × Str (ranks 0.15 / 0.20 / 0.25 / 0.35) | mult | all | OG, SH, HP | target wrapped in the storm | `W:Splinter_Storm` (fetched 2026-11-23) line 9 |
| Reap (Sevagoth) | +0.50 × Str | mult | all | HP | no refresh until expired | `W:Reap` |
| Death's Harvest (Sevagoth Shadow) | +0.50 × Str (ranks 0.10 / 0.20 / 0.30 / 0.50) | mult | all | HP | no refresh until expired | `W:Death's_Harvest` (fetched) lines 10-18 |
| The Vast Untime (Xaku) | +0.50 | mult | VOI | OG, SH, HP | – | `W:Xata's_Whisper` |
| Sentient Wrath (Caliban) | +0.35 × Str (ranks 0.15 / 0.20 / 0.25 / 0.35) | mult | all | HP | – | `W:Sentient_Wrath` (fetched) line 10 |
| Webbed Embrace (Oraxia) | +0.50 × Str (ranks 0.35 / 0.40 / 0.45 / 0.50) | mult | all | HP | cocooned target | `W:Webbed_Embrace` (fetched) line 9 |
| Sea Snares (Yareli) | +1.00 × Str (ranks 0.25 / 0.50 / 0.75 / 1.00) | mult | all | HP | snared target; not its own Cold damage | `W:Sea_Snares` (fetched) lines 10, 27 |
| Firing Line (Bonewidow) | ×1.5 (V_mult = 0.50, not Str-scaled) "from all sources while Lifted" | mult | all | OG, SH, HP | target Lifted by Firing Line (class (a) sub-condition) | `W:Firing_Line` (fetched) lines 12-14 |
| Hakchum (Narin) | +1.50 × Str (ranks 1.20 / 1.30 / 1.40 / 1.50) | mult | COL | HP | target inside the ice storm | `W:Hakchum` (fetched) lines 4, 15-18 ("multiplies all Cold damage dealt to enemies from any source") |
| Petrify (Atlas) | +0.50 | add | all | HP | – | `W:Petrify` |
| Containment Wall (Qorvex) | +0.25 | add | all | HP | – | `W:Containment_Wall` |
| Jade's Judgement (passive), Prey of Dynar, Atomi-Barrage | value not researched (user value) | add | all | HP | – | `W:Damage_Vulnerability` §Notes (OQ-417) |
| Magus Accelerant / Magus Destruct | +0.65 per application | add | HEA / PUN | HP | – | §5.5 |
| Theorem Contagion | +2.00 | add | the orb's type | HP | – | §5.5 |
| Lull, Sling Stun, Savage Silence, Radiant Finish | value not researched (user value) | mult (default) | FIN | HP | finisher mode only | `W:Damage_Vulnerability` §Notes (OQ-417) |
| Demonium, Contamination Wave, Beguiling Lantern (weapon damage only), Dual Coda Torxica (own later shots) | value not researched (user value) | mult (default) | all | HP (Beguiling Lantern: OG, SH, HP) | as named | idem (OQ-417) |
| Custom `dmgVulnerability` (T-26) | x | mult | all | HP | – | overview §6.3 |
| Eroding Rapids (Vazarin, Tauron) | +1.00 "status vulnerability" (meaning UNVERIFIED, OQ-415) | – | – | – | – | `W:Focus/Vazarin` |

The groups come from `W:Damage_Vulnerability` §Notes (additive: Atomi-Barrage, Containment Wall, Petrify, Jade's
Judgement, Prey of Dynar, Magus Accelerant, Magus Destruct, Theorem Contagion; every other entry multiplicative); the
type masks from the same section; the layer masks from its U37 patch history. Splinter Storm's own page says its
vulnerability affects "Splinter Storm's damage per second"; the vulnerability page lists it as a general vulnerability
that applies to shields and overguard since U37, so the engine applies it to weapon damage (Decision, UNVERIFIED).
The fetched pages are cached in `research/external/cache/wiki/`.

Not vulnerabilities (they act like faction or weapon factors): Mecha Empowered and Vauban passive (G with a DoT
second dip, §3.2).

Armor strips (values; combination E-15 owned by `05`; all `× Str` of total armor, additive with each other):
Terrify 0.60, Fusion Strike 0.50, Sonic Boom 0.70, Avalanche 0.60, Pulverize 0.50, Regurgitate 0.75, Pillage 0.25
(shields first), Fracturing Crush 0.75, Psychic Bolts 0.80, Reckoning 0.60, Tharros Strike 0.50, Abating Link 0.60,
Seeking Shuriken 0.70 (8 s), Plein Air 0.50, Rakhali's Cavalry 0.35 per hit, Gaze 0.50, Bastille 0.10/s, Ophanim
Eyes 0.10/s, Temporal Erosion 0.10 per hit, Fire Blast 0.50-1.00, Smite (all), Thermal Sunder up to 1.00 by battery;
Unairu Caustic Strike 1.00 (not Str-scaled); Corrosive Projection 0.18 per aura (§5.8). Source: `W:Armor` §Removing
Enemy Armor via `F:buffs` §3. Status strips (Heat, Corrosive, Emerald shard extra stacks) are `03`.

### 5.8 Auras and Coaction Drift

| Aura | Bucket and value | Stacks across squad | Source |
|---|---|---|---|
| Steel Charge | A +0.60 melee (also exalted melee, thrown glaives, Archmelee) | yes, additive | `W:Steel_Charge` |
| Rifle Amp | A +0.27 rifles (rifles, bows, snipers, launchers; not shotguns) | yes | `W:Rifle_Amp` |
| Pistol Amp | A +0.27 secondaries (incl. Glory, Regulators, Dex Pixia, Balefire) | yes | `W:Pistol_Amp` |
| Shotgun Amp | A +0.18 shotguns | yes | `W:Shotgun_Amp` |
| Dead Eye | A +0.525 sniper rifles (not bows, launchers) | yes | `W:Dead_Eye` |
| Holster Amp | A +0.60, class (c) 3 s after holster | yes (per squadmate with it) | `W:Holster_Amp` |
| Worthy Comradery | CC_rel +0.75 on weak-point hits (incl. Sonar spots) | per squadmate | `W:Worthy_Comradery` |
| Corrosive Projection | armor strip 0.18 (U27.2) | yes, additive | `W:Corrosive_Projection` |
| Growing Power | Str +0.25, class (b) 6 s after a weapon status proc | bonus stacks, duration not | `W:Growing_Power` |
| Power Donation | own Str −0.30, allies +0.30 | four → +0.60 each | `W:Power_Donation` |
| Empowered Blades | SC_rel +0.60 and A_sd +0.60 on heavy attacks | – | `W:Empowered_Blades` |
| Mecha Empowered | G +1.50 vs marked target (§3.2, §9) | per aura | `W:Mecha_Empowered` |
| Swift Momentum, Ready Steel | heavy wind-up +0.30, combo duration +6 s; initial combo +24 | – | `07` |

```text
E-4.11  Aura value received by player p from an aura worn by player s
  value = listed · (1 + 0.15·[s has Coaction Drift]) · (1 + 0.15·[p has Coaction Drift] · [aura buffs players])
  own aura on oneself with Coaction Drift: × 1.15² = × 1.3225 (Steel Charge → +0.7935);
  enemy-debuff auras (Corrosive Projection) get only the wearer's aura strength: × 1.15
```

Source: `W:Coaction_Drift` (`F:buffs` §4; fixed for Power Donation in U43).

### 5.9 Archon shards

Source: `W:Archon_Shard` §Shard Buffs, per-colour pages (`F:buffs` §5, `F:addmul` §6). Tauforged = 1.5 × the normal
value. The same bonus on several shards stacks additively.

| Colour | Bonus | Normal (Tauforged) | Bucket |
|---|---|---|---|
| Crimson | melee critical damage | +0.25 (+0.375) | CD_rel (exalted melee too) |
| Crimson | primary status chance | +0.25 (+0.375) | SC_rel (exalted primaries too) |
| Crimson | secondary critical chance | +0.25 (+0.375) | CC_rel (exalted secondaries too) |
| Crimson | Ability Strength / Ability Duration | +0.10 (+0.15) | Str / duration |
| Violet | primary Electricity | E-4.12 | element, **combines** (position per `01`, D-006) |
| Violet | melee critical damage | +0.25 (+0.375), doubled when max Energy > 500 (Arcane Battery counts since U42) | CD_rel |
| Topaz | secondary crit chance per kill on a Heat-affected enemy | +0.01 (+0.015) per kill, max +0.50 (+0.75) per shard; resets on revive | CC_rel, class (c) |
| Emerald | Toxin status damage | +0.30 (+0.45) | A_sd,TOX (weapon and ability Toxin procs, Toxic Lash included) |
| Emerald | maximum Corrosion stacks | +2 (+3) | `03` cap |
| Violet / Topaz / Emerald | ability damage vs Electricity / Radiation / Corrosion-affected | +0.10 (+0.15) | abilities only, **never exalted weapons** (D-068) |
| Azure, Amber | none on weapon damage | – | – |

```text
E-4.12  Violet primary Electricity (per Violet shard carrying it)
  E_ELE += Σ_(violet Electricity shards i) (a_i + b_i·N_cav)
  a_i = 0.30 (0.45 Tauforged), b_i = 0.10 (0.15 Tauforged)
  N_cav = number of equipped Crimson, Azure and Violet shards, the shard itself included
```

Worked values (`W:Violet_Archon_Shard` §Tips lines 36-39, all Tauforged): one Violet alone 0.60; two Violet 1.50; two
Violet + three Crimson `2 × (0.45 + 0.15 × 5) = 2.40`; five Violet `5 × (0.45 + 0.15 × 5) = 6.00`. Derived from
E-4.12 (not a page example): five normal Violet `5 × (0.30 + 0.10 × 5) = 4.00`. Tauforged Violet melee crit
damage: 0.375 (shard table) vs "+37 %" in the U35.0.9 patch text (OQ-423).

### 5.10 Focus

| Node (school) | Bucket and value | Class (§4.1; (u): user-enabled, uptime input for timed nodes) | Source |
|---|---|---|---|
| Phoenix Talons (Madurai, passive) | +0.30 × total base (physical and elemental) added as physical, split by physical shares, after physical mods; warframe weapons incl. exalted; T-19; arithmetic `01` | – | `W:Focus/Madurai`; D-007 |
| Void Strike (Madurai) | A +0.10 per 1 % operator energy consumed (+10.00 at full), 8 s, 40 s cooldown; not abilities, not archguns; once on melee projectiles | (u) | idem (`100 × (1 + 1.65 + 10) = 1265`) |
| Sling Strength; Overwhelming Power (Tauron) | Str +0.40 for 20 s; Str +0.05 per enemy hit, max +0.50, 30 s | (u) | idem |
| Lethal Levitation (Naramon) | A +0.50 per Lifted enemy attacked by the operator, 4 stacks (+2.00), 60 s; melee included; both melee-projectile buckets | (u) | `W:Focus/Naramon` |
| Killer's Rush (Naramon) | CC_rel +0.50 melee, 40 s after an operator finisher | (u) | idem |
| Temporal Shot (Zenurik) | +1.00 headshot damage vs Temporal-Drag-slowed enemies (`sum_hs_mult` UNVERIFIED) | (u) | `W:Focus/Zenurik` |
| Hardened Wellspring; Spectral Pages (Zenurik) | Str +0.20 in a reinforced Wellspring; +0.10 per page (up to 4) | (u) | idem |
| Caustic Strike (Unairu) | armor strip 1.00 in 8 m | (u) | `W:Focus/Unairu` |
| Eroding Rapids (Vazarin, Tauron) | §5.7 | (u) | `W:Focus/Vazarin` |

### 5.11 Companion, squad and environment buffs

| Source | Bucket and value | Class | Source |
|---|---|---|---|
| Adarza Kavat Cat's Eye | CC_abs +0.60, 10 s, 20 s cooldown | (u) | `W:Cat's_Eye` |
| Smeeta Kavat Charm (crit buff) | `cc_floor` CC_min = 2.00, 30 s | (u) | `W:Charm` (floor; `W:Universal_Weapon_Bonuses` says "+200 %", overview decided floor) |
| Tenacious Bond (companion mod) | CD_abs +1.2 while companion CC ≥ 0.50 | (c) | `W:Tenacious_Bond` |
| Vigorous Swap (warframe mod) | A +1.65 after a weapon swap | (c) | `W:Vigorous_Swap` |
| Reinforced Bond (companion mod) | FR +0.60 while the companion has more than 1200 maximum shields or overshields | (c) `shield_level`; timing stat (§4.3) | `W:Reinforced_Bond` (fetched 2026-11-23) lines 12-22 |
| Hunter Synergy (Beast Claws set mod) | companion crit chance +0.30 × the primary's total CC | companion AI damage: out of scope (overview §1.3); listed because `02` names it | `W:Hunter_Synergy` (fetched) lines 11-35 |
| Tek Collateral (set mod) | CD_rel +1.00 inside the Marked Zone | (c) | `W:Universal_Weapon_Bonuses` |
| Biting Frost (Frost augment) | CC_rel +2.00 and CD_rel +2.00 (condition, frozen targets, UNVERIFIED) | (u) with an (a) sub-condition | idem |
| Empowered Quiver (Ivara augment) | CD_rel +1.00 × Str | (u) | idem |
| Cyte-09 passive | CC_rel up to +3.00 on weak-point hits | (u) | idem |
| Provoked | A +1.10 | (u) | idem |
| True Master's Font Damage Blessing | F +0.25 | (u) | idem |
| Crucible Overflow (Alchemy) | E[combined] +1.00 | (u) | idem |
| Unairu Initiative (Perita Rebellion) | +1.00 Impact per order on weapons, scaled like an element (total base; `01`) | (u) | idem |
| Vazarin Initiative (Perita Rebellion) | SC_rel +0.25 per order | (u) | idem |
| Harmony heavy-attack kill | SD +0.30, 30 s | (u) | idem |
| Ash passive; Lavos passive; Saryn passive | SD +0.50 (Slash only); +1.00 × Duration; +0.25 | – | idem |
| Decrees (Duviri, Descendia; Shattering Frost, Cloak and Dagger, Critical Frost, Salted Wound, Wyrmling's Aid), Iceblade of Narin blessing | mode-specific; only Shattering Frost is catalogued (§2.1); the others are out of scope | – | `W:Extra_Hit`, `W:Universal_Weapon_Bonuses` |
| Hunter set bonus | +0.25 companion damage per piece vs Slash-affected | companion only (out of scope) | `W:Hunter_Set` |

### 5.12 Weapon passives and Incarnon perks owned here

| Item | Bucket and value | Class | Source |
|---|---|---|---|
| CO-type Incarnon perks, Cedo | C (§2.1) | (a) | §2.1 |
| Kunai Incarnon Swift Conclusion | C +2.00 below 50 % health; flat X = 70 (Kunai) / 80 (Mk1-Kunai) | (a) | `W:Kunai_Incarnon_Genesis` |
| Phenmor and Laetum Devouring Attrition | G, 0.50 chance of ×21 on non-crit instances | (b) | `W:Phenmor` |
| Laetum Overwhelming Attrition | A +12.00 on instances that neither crit nor apply a status (bucket UNVERIFIED; Xata's Whisper's Void proc does not count, `W:Extra_Hit` §Bugs) | (b) | `W:Extra_Hit`; inspiration `base_uncrit_unstatus` |
| Phenmor Spiteful Defilement | +1.00 crit damage vs targets with fewer than 3 distinct statuses; bucket per D-067 (`sum_cd_rel`) until the register is amended; the per-perk page puts it in `sum_cd_abs`, and overview §1.4 rule 3 makes that the game behaviour, so the amendment is required (OQ-406) | (a) | `W:Phenmor` ("a +1x flat bonus to critical damage, calculated after percentage modifiers, similarly to Tenacious Bond"; counts distinct types) |
| Phenmor Lingering Judgement | `sum_hs_mult` +0.50 for 8 s after 2 headshots within 2 s | (b) | `W:Phenmor` |
| Braton Incarnon multishot-pellet damage; Torid Plentiful Mayhem | G (§3.2) | (b) | §3.2 |
| Torid Incarnon last-shot multishot | MS +3 on the last shot of a magazine | (b) | `F:weapon` §16 |
| Weapon zoom levels | the zoom level's crit chance, crit damage or headshot bonus, class (b) `while_aiming`; zoom level selectable (default highest under `all_max`) | (b) | D-406 |
| Prelude of Might-type perks ("+X base crit damage when CC < 40 %") | `cd_base_add`, `02` | (a) | `F:mods` §9 |
| Dread Incarnon Hitman's Opportunity | flat X = +70 (charged shot and Incarnon form); C +1.00 below 50 % health (§2.1) | –; (a) | `W:Dread_Incarnon_Genesis` (fetched 2026-11-23) lines 46-52 |
| Strun Incarnon Blazing Barrel | `MS0+` +0.05 per stack, 5 stacks ("adds 0.25 pellets to the base Multishot of both forms when fully stacked") | (b) `on_fire`, `max_stacks` 5, `reset_on = (on_reload, 1)` | `W:Strun_Incarnon_Genesis` lines 61-72; `02` MS0+ |
| Gotva Prime passive | after a hit that applies ≥ 1 status: 0.15 chance that the next hit's CC is **set** to 3.00 (ignores every other crit-chance modifier; Vigilante can still raise the tier); several statuses in one hit do not raise the chance | (b) `on_status(any)`, `consume_on = next_shot` | `W:Gotva_Prime` (fetched 2026-11-23) lines 10-15 (15 % confirmed by DE) |
| Sagek Prime passive | after a critical hit: 0.06 chance that the next shot's SC is **set** to 5.00 (5 procs; ignores every other status-chance modifier; Encumber triggers once); the chance persists through misses | (b) `on_crit`, `consume_on = next_shot` | `W:Sagek_Prime` (fetched) lines 7-12 |

### 5.13 Values other documents delegate to this catalogue

| Item (requesting document) | Value, bucket | Class, gate | Source |
|---|---|---|---|
| Paracesis innate Sentient bonus (`01` §4.12) | F +0.07 / 0.13 / 0.20 / 0.27 / 0.33 / 0.40 / 0.47 / 0.53 / 0.60 / 0.60 at weapon ranks 31-40, vs Sentient only; additive with Roar and the Sacrificial set | – (rank input) | `W:Paracesis` lines 15-40 |
| Sacrificial Steel and Sacrificial Pressure, Sentient part (`01` §4.12) | F vs Sentient per the mods' rank tables (Pressure rank 5: +0.75 alone; the set column ×1.25); the pages' "×1.xx" columns do not follow `1 + v` and stay UNVERIFIED (OQ-415) | – | `W:Sacrificial_Pressure`, `W:Sacrificial_Steel`, `W:Sacrificial_Set` ("Increases the effects of both mods by 25%") |
| Summoner's Wrath aura (`01` §4.12) | F +0.75 for companion and summon damage only ("additive to Faction Damage Bonuses"); in this engine only companion robotic weapons (UNVERIFIED, `01` OQ-114) | – | `W:Summoner's_Wrath` lines 12-29 |
| Conditional Incarnon flat perks (`01` lines 73, 229) | X with a per-perk attack filter (normal form, Incarnon form, charged shot) and a condition class: unconditional perks `–`; "+X while Overshields" (c) `while_overshields`; Lex Prime "+80 for 8 s on Shield Break" (c) `on_shield_break`; Dread Hitman's Opportunity +70 (§5.12) | per perk | perk tables (`F:mods` §9, `DS:wiki` §3); tier choice `09` |
| Shattering Impact (`05` §9.4) | `rank + 1` base-armor points (6 at rank 5) per melee hit whose quantized Impact is > 0 (proc not required); gate: melee hits; not Archons or Commandeered Primes | – | `W:Shattering_Impact` lines 2-35 |
| Amalgam Argonak Metal Auger (`05` §9.4) | 6 base-armor points per dagger damage instance, status ticks of dagger procs included; additive with Shattering Impact | – | `W:Amalgam_Argonak_Metal_Auger` lines 9-35 |
| Polarize (`05` E-5.10) | 400 × Str shield then armor points per pulse (`05` P_pts) | (u) | `W:Armor` L438-440 via `05`; `W:Polarize` (fetched) line 23 ("Shields must be completely drained … before a new Polarize cast can deplete armor") |
| Sharpened Claws (`05` §9.6) | armor strip +1.20 (rank 3) of total armor, additive with Corrosive Projection, once per target; gate: not Kuva Liches | (u) companion | `W:Sharpened_Claws` lines 12-30 |
| Hydroid passive (`03` corr_strip c₁, `05` E-5.12) | scenario flag `armor_hydroid`: first Corrosive stack strips 0.50 instead of 0.26 | (u) | `05` E-5.12; flag proposed in `05` OQ-523 |
| Archon Continuity, Archon Vitality (`03` EC-03-60, `02` D-204) | add Corrosive (Toxin abilities) or extra Heat procs to **ability** procs and exalted-weapon instances only; never to ordinary weapon instances | – | `W:Archon_Continuity`, `W:Archon_Vitality`; `02` D-204 |
| Strun Incarnon stack condition (`02` line 155) | §5.12 | (b) | §5.12 |
| Gotva Prime `crit_after_status`, Sagek Prime passive (`02` line 1007) | §5.12 | (b) | §5.12 |

---

## 6. Extra hits (E-14) and Melee Duplicate

### 6.1 Sources

The extra-hit fraction is the overview's `extra_hit_frac` p_xh; `p_xh,e` is the value of source e. The applicability
columns copy the `W:Extra_Hit` §Sources table (columns "Applies to Primary and Secondary Weapons?", "… Incarnon
Forms?", "… Melee Weapons?", "… Archguns?"); they are the effect's `weapon_class_scope` gate (§4.2) in E-4.13: an
attack outside the scope never receives the extra hit.

| Source | p_xh,e (listed, max rank) | Type, proc | Multipliers of p_xh,e | Primary / secondary | Incarnon forms | Melee | Archguns | Other limits | Source |
|---|---|---|---|---|---|---|---|---|---|
| Xata's Whisper (Xaku; also subsumed) | 0.26 × Str | VOI, rolled with the weapon's modded status chance | – | yes | yes | yes | yes | – | `W:Extra_Hit` table; `W:Xata's_Whisper` |
| Toxic Lash (Saryn) | 0.30 × Str | TOX, forced | ×2 for melee weapons | yes | yes | yes | yes | – | idem; `W:Toxic_Lash` |
| Resupply (Cyte-09) | 0.25 × Str | chosen element, forced | ×2 for sniper rifles | yes | no | n/a | no | not battery weapons | idem |
| Silken Stride (Oraxia) | 0.40 × Str | TOX, forced | ×2 while wall latching | yes | no | n/a | no | – | idem |
| Demonium rune (Uriel passive) | 0.30 × Str | HEA, forced | – | yes | no | no | no | – | idem |
| Gastroparesis (augment) | 0.15 per enemy consumed | GAS, forced | – | yes | no | no | no | – | idem |
| Sangodae (Narin passive) | 0.50 | COL, forced | – | yes | no | no | no | also her abilities (out of scope) | idem |
| Reconifex Active Reload | 0.25 | HEA, forced | – | Reconifex only | – | – | – | – | `W:Extra_Hit` §Weapon Effects |
| Primary Debilitate | 0 (no damage) | one component of the re-applied combined status, forced | – | primaries | – | – | – | – | idem (§6.5) |
| Melee Duplicate | special (§6.4) | full copy | – | – | – | melee | – | – | idem |

"n/a" (Resupply, Silken Stride on melee) is the wiki's entry; the engine treats it as "no". Exalted weapons are not a
column of the wiki table: Xata's Whisper and Toxic Lash apply to them as "all equipped weapons" (Decision); the
others do not (UNVERIFIED). Strength is the caster's (§4.4). T-15 and T-16 enter final values of p_xh (Toxic Lash's
×2 for melee is applied by the engine, overview T-16).

### 6.2 Damage

```text
E-4.13  Extra hit of source e on a parent instance (refines E-14; W:Extra_Hit §Damage Calculation)
  H_w   = Σ_t W_t(k)                         parent weapon side (E-12): q_t, Y with its CO term, (1 + F), G, Part, c_k
  XH_e  = H_w · p_xh,e · (1 + F) · (Part if e = Xata's Whisper, else 1)
  one damage instance of type e_xh; enemy side E-12 with that single type (Xata: Void type modifier, armor applies)
  stealth: H_w already contains the stealth c_k of the parent (E-12)
  gate: the parent attack is inside the source's weapon_class_scope (§6.1)
```

Rules (`W:Extra_Hit` §Interactions, §Notes, §Bugs; `F:addmul` §13; `F:buffs` §1.3-§1.4):

- Which parent instances trigger. Overview E-14 says "one extra hit per pellet of a parent direct instance"; the engine
  follows it (current decision: per pellet, per merged beam tick (CD-10, Decision), radial instances do not trigger).
  The game sources say more: `W:Toxic_Lash` §Tips: "Weapon mods and features that cause additional hits will create
  additional instances of Toxic Lash Damage. This includes the radial damage of slam attacks, explode on death mods
  such as Acid Shells, and mods like Concealed Explosives and Nightwatch Napalm … especially useful for Arca Titron,
  Tenet Exec, (Kuva) Sobek, and (Kuva) Ogris", and `W:Extra_Hit` §Interactions: "Most non-standard weapon hits will
  trigger an Extra Hit … The damage calculation will use the damage dealt by the non-standard hit as the Weapon Hit
  Damage". So radial instances on the target (rocket and grenade explosions, slam radial damage) and the hits of
  Acid Shells, Concealed Explosives and Nightwatch Napalm do trigger, each with its own damage as H_w. This document
  proposes amending E-14 to "one extra hit per pellet of a parent direct or radial instance on the target" (OQ-412,
  D-412); until the manager amends E-14, radial instances do not trigger. DoT ticks never trigger. Bursting Mass's
  absorbed damage and Pathocyst's maggots never trigger (`W:Extra_Hit`).
- No extra hit if the parent kills the target (`layered` pass only).
- Extra hits give no melee or sniper combo; they do not change the parent's composition or proc weights; their
  kills count for the weapon; they **do** count as hits for on-hit triggers (Secondary Enervate, Double Tap), so
  `on_hit` events of §4.1 include extra hits, counted in each effect's `hit_unit` (§4.1).
- Xata's Whisper is the only extra hit without the forced 1× weak-point multiplier; `W:Sonar` claims extra hits
  receive Sonar again (contradiction; engine follows `W:Extra_Hit`, OQ-412). Xata's Whisper cannot crit by itself
  (the crit is inside H_w; listed as a bug).
- Toxic Lash and Xata's Whisper are reduced by ability-damage reduction (Ancient Disruptor auras): enemy-side, out of
  scope for the default enemy (`05`).
- Toxic Lash grants no Incarnon charge; for Torid and Angstrum no extra hit grants charge (`06` Incarnon uptime).
- Xata's Whisper is also triggered by Blast detonations (bug, only Xata). MC creates one extra hit per detonation
  from the weapon-side detonation damage `H_det` of `03` (which already holds faction twice):

  ```text
  E-4.13a  XH_det = H_det · p_xh,Xata · (1 + F) · (1 + E_all)
           E_all  = Σ of every elemental mod bonus on the weapon, combined into a secondary element or not
                    (non-mod elemental additions: UNVERIFIED, Decision: excluded)
  ```

  Source: `W:Xata's_Whisper` §Tips line 48: a 100-damage gun with Thermite Rounds, Rime Rounds, Stormbringer and
  Primed Bane: detonation `0.3 × 100 × (1 + 0.55)^2 = 72.075`, then Xata's Whisper deals
  `0.26 × 72.075 × (1 + 0.55) × (1 + 0.6 + 0.6 + 0.9) = 90.04` ("the Faction Damage Bonus is applied yet again, and
  the Elemental Damage Bonus is applied even though Blast detonations don't scale off Elemental Damage Bonuses").
  The factor is the weapon's whole elemental bonus (3.1 here, Electricity included), not the bonus of the mods that
  form Blast (TV-04-19). Faction thus applies three times. EV omits it unless a Blast detonation rate is available
  from `03` (D-071).

```text
E-4.14  Status and DoT of an extra hit
  Xata's Whisper: VOI proc with probability SC of the parent pellet (no forced proc)
  forced sources: one proc of e_xh per extra hit
  tick = XH_e · K_e · (1 + E^dot_all,e) · (1 + F) · (1 + A_sd + A_sd,e)          (overview E-14; no "+1", OQ-413)
     E^dot_all,e = every elemental bonus of type e on the weapon, including mods combined into a secondary element;
                   an already-combined bonus (Nourish Viral) is not a matching bonus for Toxin (W:Extra_Hit §Notes)
  Toxic Lash procs: ticks floor(6·(1 + SD + 0.25)) (Saryn passive, D-037)
```

Faction therefore applies twice on the extra hit and three times on its DoT (`W:Extra_Hit` §Notes, "triple-dip").
Worked check (`W:Extra_Hit` §Notes): 100 base, Infected Clip +0.90, Toxic Lash 0.30 → `XH = 100 × 1.9 × 0.3 = 57`,
tick `57 × 1.9 × 0.5 = 54.15` (TV-04-11).

### 6.3 Expected value

EV per parent pellet in scope (§6.1): `E[XH_e] = E[H_w] · p_xh,e · (1 + F)` (· E[Part] factor for Xata); the extra hit's enemy side
uses the target-state expectations of overview section 4.2 at the parent's time. Forced-proc extra hits add `1` proc
of `e_xh` per parent pellet to `λ_(e_xh)`; Xata adds `SC` Void procs per pellet.

### 6.4 Melee Duplicate

```text
E-4.15  Melee Duplicate (melee arcane; W:Melee_Duplicate, W:Extra_Hit §Weapon Effects)
  trigger : a parent melee instance with crit tier k = 1 (base, yellow); chance p_dup = 0.25 + 0.15·rank (1.00 at rank 5)
  effect  : one new direct instance of the same attack right after the parent (CD-04 order):
            same B', A, composition, G, Part; own crit roll (E-6: may be k = 0, 1, 2 …), own status roll (E-8, E-9),
            the weapon's forced procs repeat (not Seeking Talons'); C and the target state are read after the
            parent's procs; faction applies once (it is a full hit, not E-14); it is not an extra hit of E-14
  scope   : melee hits including heavy attacks and slams (the wiki's bug note about slam instances implies slams
            trigger it); Exodia Contagion direct hit and explosion (not the embed DoT); not if the parent kills
  no recursion: a duplicate does not trigger Melee Duplicate (Decision, UNVERIFIED, OQ-419)
  EV      : instances per parent = 1 + p_dup · P(k = 1), P(k = 1) from E-6 (02)
```

Tier-1-only triggering is also implied by the page's optimal-crit formula `CC = (3·CM − 4)/(2·CM − 2)`, which holds
only if orange and red crits do not trigger (`F:melee` §7 derivation).

Bug (game behaviour, implemented): with Toxic Lash or Xata's Whisper active, a tier-1 crit duplicates the **extra
hit** of the ability in the earlier ability slot instead of the melee hit, and no duplicate of the melee hit occurs
(`W:Melee_Duplicate` §Bugs; `W:Extra_Hit` §Bugs). The engine takes the slot order from a scenario input (default:
the extra-hit ability listed first in the user's buff list; OQ-419).

### 6.5 Primary Debilitate

When a combined status with 10 live stacks is applied again by the weapon, a 0-damage extra hit forces one proc of
one of its two components (each with probability 1/2, Decision as inspiration JS L10621-10639). Because the proc comes
from an extra hit, its Heat, Toxin and Electricity DoTs triple-dip faction (`W:Extra_Hit` §Weapon Effects). The seed
of such a proc is not defined by the wiki (a 0-damage extra hit cannot be its seed); `03` owns the tick and the
register row D-073 (twice) conflicts with this statement: proposal in OQ-414.

### 6.6 Melee Afflictions

`03` hands this arcane's trigger and seed to this document (EC-03-59); the added stacks then behave as ordinary
procs of `03`.

```text
E-4.20  Melee Afflictions (melee arcane, rank r; W:Melee_Afflictions §Effect, §Damage Calculation, §Trigger)
  trigger  : a melee instance of the equipped weapon inflicts Knockdown, Ragdoll or Lifted on a target that carries
             at least one status, and the target's state allows it (page table: never on Overguard; on a Cold-frozen
             target only Ragdoll; otherwise all three); class (b) on_status(KND | ragdoll | LIF)
  effect   : for every status type s present on the target, add N = r + 1 (6 at rank 5) stacks of s
             (caps, timers and replacement per 03; non-damaging statuses add only stacks)
  seed     : MBD_p   = B'_src · (1 + A_src) · (1 + F_src) · (1 + A_sd,src) · c_k,p · Part_p   per live proc p of s
                       (the proc's source weapon terms; faction once, "the double dip is not counted")
             avg_s   = Σ_(p of s) MBD_p / (number of live procs of s)
             H_aff   = (Σ_(s ∈ {SLA, HEA, TOX, ELE, GAS} present) avg_s) · (1 + F_melee)
  new DoT  : tick_s  = H_aff · K_s · (1 + F_melee) · (1 + A_sd,melee + A_sd,melee,s) · (1 + E^dot_melee,s)
             E^dot only for TOX and ELE; HEA takes the melee weapon's elemental and faction bonus only if the melee
             weapon was the first Heat source (Heat inherit, 03); BLA: the new stacks use H_aff as their H (0.3·H, 03)
  EV       : λ_aff = rate of triggering melee instances (07 forced KND/LIF procs, ragdolling heavies) × P(target has
             ≥ 1 status); per trigger, expected new stacks of s = N·P(n_s ≥ 1); avg_s from the expected proc seeds of
             this weapon's own procs (single source in the default scenario)
```

Sources: `W:Melee_Afflictions` (cached) §Damage Calculation steps 1-4 (Proc MBD "Weapon Base Damage × (1 + Weapon
Base Damage Bonus) × (1 + Weapon Faction Damage Bonus) × (1 + Weapon Status Damage Bonus) × (Additional
Multipliers)", additional multipliers = "modded critical multiplier on Critical Hit and multipliers on Enemy Body
Parts"; Affliction hit "× (1 + Melee Faction Bonuses)"; tick "× Status Effect Multiplier × (1 + Melee Faction Damage
Bonus) × (1 + Melee Status Damage Bonus) × (1 + Elemental on Melee of Status Effect (where applicable))", "Only Toxin
and Electricity statuses can benefit from Elemental Damage Bonuses"), §Trigger table. Whether `B'_src·(1 + A_src)`
includes the CO term and the `G` members of the source hit is not stated (Decision: the CO term and `G` are not
included, following the page's formula literally; UNVERIFIED, OQ-441). The page carries an UpdateMe banner. In the
default single-source scenario faction therefore applies three times to an added tick (seed, Affliction hit, tick).
TV-04-26.

---

## 7. Stealth and finishers

### 7.1 Stealth damage bonus on melee hits (T-08)

```text
E-4.16  Stealth (normal melee attacks; overview E-12)
  SDB = 1 + 0.2·min(rank, 30)                       (W:Stealth §Stealth Damage Bonus, lines 37-49)
  c_k = 1 + SDB + k·(h·CDM − 1)                     non-crit k = 0: 1 + SDB = 8 at rank 30
  DoT seeded by such a hit inherits c_k (E-13; W:Damage_over_Time §DoT Damage Scaling)
```

- Conditions (class (c) under `simulated`, true under `all_max` only when T-08 is on): unalerted, blinded, Silenced,
  finisher-stunned targets or an invisible player; 3 s cooldown after being revealed (`W:Stealth`). T-08 is the
  switch; when T-08 is off no stealth term exists.
- Ranged weapons: no stealth bonus. Exodia Contagion's projectile is affected (`W:Exodia_Contagion` §Interactions).
- Rank-40 weapons: `W:Finisher` (lines 102-106) gives `SDB = 1 + 0.15·rank`, the same 7 at max rank; its valence
  example (Kuva Shildeg rank 18, `1 + 3.7`) uses it. The overview formula differs for rank-40 weapons below rank 40;
  proposed amendment in OQ-407. The Simulacrum does not apply SDB unless enemy AI is enabled (`W:Finisher` line 115).

### 7.2 Finishers (optional mode)

Finishers are not part of the default DPS metric (they need crowd control; `F:melee` §9 recommendation). No overview
toggle selects them; this document proposes one ("finisher mode": off / stealth / back / front / universal, OQ-437).
For that mode (`07` owns its schedule and the class multipliers M_class of `W:Finisher` §All Finishers):

```text
E-4.17  Finisher instance (stealth, back, front, universal)
  W_fin = Σ_t q'_t · Y · (1 + F) · G · (1 + Σ finisher) · M_class · (1 + SDB + k·(h·CDM − 1)) · M_valence
  q'_t  : composition with every elemental E_t and physical P_t bonus halved ("Elemental and Physical damage mods
          only apply half of their bonus before being converted", W:Finisher; UNVERIFIED form), computed by
          E-2 with the halved bonuses and quantized by E-3 on the 1/32 grid (Decision, UNVERIFIED, OQ-437)
  damage type FIN (true; ignores armor); ground finishers deal Finisher damage and do not proc status
  Σ finisher: Finishing Touch 0.60, Covert Lethality 1.00 (additive: "+160 %"), riven Finisher Damage
  M_valence = 0.5 for weapons with a valence (progenitor) bonus (bug, W:Finisher §Bugs line 622), else 1
  crit term: the Stealth page form (Decision); the Finisher page writes (1 + SDB + Critical Multiplier) (OQ-407)
```

Finisher-only vulnerabilities (Lull, Sling Stun, Savage Silence, Radiant Finish) are type-restricted entries of §5.7.
Mercy kills (`residual health + 1`) are executions, out of scope.

---

## 8. Rivens: stat-to-bucket mapping and value formula

### 8.1 Trait to bucket

Rivens are ordinary stat lines of the corresponding mod family and stack in the same bucket (`F:mods` §8.5,
`W:Riven_Mods`, `W:Faction_Damage_Bonus` §Other Sources).

| Trait | Bucket | Notes |
|---|---|---|
| Damage (guns) / Melee Damage | A | negative values possible (OQ-424) |
| Multishot | MS | cannot bring MS below 1 (D-019, `02`); Acuity locks MS |
| Critical Chance | CC_rel | melee: ×2 on heavy attacks (`02`) |
| Critical Chance for Slide Attack | CC_rel on slide attacks | `07` |
| Critical Damage | CD_rel | |
| Status Chance / Status Duration | SC_rel / SD | |
| Heat, Cold, Electricity, Toxin (positive only) | element mods; two elements on one riven: bottom-up order (`01`) | |
| Impact, Puncture, Slash | `sum_phys[t]` | roll only if the weapon has > 25 % of that type ("usually", `09`) |
| Damage to Corpus / Grineer / Infested; splices Orokin, Scaldra, Techrot | F (`+v`; displayed `×(1 + v)`) | Grineer covers Kuva Grineer, Infested covers Infested Deimos (`01` family rule); additivity with Bane: overview OQ-019 |
| Status Damage (splice) | A_sd | |
| Blast, Corrosive, Gas, Magnetic, Radiation, Viral (splices) | E[t] non-combining | |
| Weak Point Damage (splice) | `sum_wp_dmg` | D-011 |
| Weak Point Critical Chance (splice) | CC_rel on weak-point hits | D-010 |
| Finisher Damage | finisher factor | §7.2 |
| Melee Damage on Heavy Attack (splice) | A, gate heavy | |
| Slam Attack Damage (splice) | Slam Damage factor (G) | |
| Fire Rate / Attack Speed | FR (×2 for bows) / AS | `06`, `07` |
| Magazine, Reload, Ammo Maximum, Ammo Efficiency (splice), Reload While Holstered (splice), Projectile Speed, Punch Through, Recoil, Zoom, Range, Combo Duration, Initial Combo, Additional Combo Count Chance, Chance to Gain Combo Count, Heavy Attack Efficiency, Heavy Attack Wind Up (splice), Parry Angle (splice) | non-damage stats (`06`, `07`) or none | |

### 8.2 Value formula

```text
E-4.18  Riven trait value (W:Riven_Mods §Attribute Value Formula; F:mods §8.3)
  v = b × 10 × (r + 1) × D × L × W        evaluated left to right in binary32 (each product rounded to f32)
  W = 0.9 + 0.2·u,  u ∈ [0, 1]            (W computed in f32: 0.9f + 0.2f·u)
  D = 1.2                                 (constant since Hotfix 44.1.3, 2026-10-22; dispositions retired)
  r = riven rank 0 … 8
  L (per trait) by layout:   2 positive, 0 negative: +0.99
                             2 positive, 1 negative: +1.2375 each positive, −0.495 the negative
                             3 positive, 0 negative: +0.75
                             3 positive, 1 negative: +0.9375 each positive, −0.75 the negative
  displayed: percentage traits 100·v %; Ammo Efficiency and Chance to Gain Combo Count −100·v %;
             faction traits ×(1 + v)
  splice: the spliced trait keeps the larger u of the two consumed traits (even a negative's), is always positive and
          keeps the riven's layout; its L is taken from the layout like any positive trait (UNVERIFIED, OQ-424)
```

Engine use: riven values in a build are user input as displayed on the card; E-4.18 serves range validation and riven
generation (ranges and legality: `09`). The input conversion depends on how the trait is displayed:

| Trait kind | Card shows | Engine value v |
|---|---|---|
| percentage traits (Damage, Multishot, Critical Chance and Damage, Status, elements, IPS, Fire Rate, Magazine, Reload, Zoom, Projectile Speed, Finisher, Slam, Heavy Attack Efficiency, splices …) | `±x %` (0.1 % steps) | `±x/100` |
| faction traits (Damage to Corpus / Grineer / Infested, splices Orokin / Scaldra / Techrot) | `×y` (for example ×1.53, ×0.85) | `y − 1` (×1.535 → 0.535; ×0.85 → −0.15) |
| Punch Through | `+x m` | x (metres, raw) |
| Range (melee) | `+x m` | x (metres, raw) |
| Combo Duration | `+x s` | x (seconds, raw) |
| Initial Combo | `+x` | x (count, raw) |
| Ammo Efficiency, Chance to Gain Combo Count | `+x %` (the display is `−100·v %` of a negative formula value, E-4.18) | formula value `v = −x/100`; the bucket (ammo efficiency, combo chance) receives the displayed bonus `+x/100` |
| Weapon Recoil | `±x %` | `±x/100` (non-damage) |

The same per-trait conversion is the inspiration's (`showRiven`, JS L7273-7340: combo duration, initial combo, range
and punch through raw; faction rows `v − 1`; everything else `v/100`; `I:stat` §1). TV-04-27. Worked check: Rifle Critical
Chance, 2+1, rank 8: `u = 0 → 2.0046701`, `u = 1 → 2.4501524` (wiki range "200.5 % — 245.0 %"; TV-04-13).

### 8.3 Coefficients b

Ordinary traits (`W:Riven_Mods` §Base Values; `F:mods` §8.4). "–" = cannot roll. Kitgun rivens use the Pistol column
even on primary kitguns; Zaw uses Melee; companion (robotic) rivens use the column of the associated class.

| Trait | Rifle | Shotgun | Pistol / Kitgun | Archgun | Melee / Zaw |
|---|---|---|---|---|---|
| Damage / Melee Damage | 0.0183333 | 0.0183 | 0.0244 | 0.0111 | 0.0183 |
| Critical Chance | 0.016666 | 0.01 | 0.016666 | 0.0111 | 0.02 |
| Critical Damage | 0.013333 | 0.01 | 0.01 | 0.0089 | 0.01 |
| Multishot | 0.01 | 0.0133 | 0.0133 | 0.0067 | – |
| Fire Rate / Attack Speed | 0.00667 | 0.01 | 0.0083 | 0.00667 | 0.0061 |
| Status Chance | 0.01 | 0.01 | 0.01 | 0.0067 | 0.01 |
| Status Duration | 0.01111 | 0.01111 | 0.01111 | 0.01111 | 0.01111 |
| Heat / Cold / Electricity / Toxin | 0.01 | 0.01 | 0.01 | 0.0133 | 0.01 |
| Impact / Slash | 0.013333 | 0.013333 | 0.013333 | 0.01 | 0.0133 |
| Puncture | 0.01333 | 0.01333 | 0.01333 | 0.01 | 0.0133 |
| Damage vs Corpus / Grineer / Infested | 0.005 | 0.005 | 0.005 | 0.005 | 0.005 |
| Magazine Capacity | 0.005555 | 0.005555 | 0.005555 | 0.0067 | – |
| Reload Speed | 0.005555 | 0.005555 | 0.005555 | 0.0111 | – |
| Ammo Maximum | 0.00555 | 0.01 | 0.01 | 0.0111 | – |
| Projectile Speed | 0.01 | 0.01 | 0.01 | – | – |
| Punch Through (m) | 0.03 | 0.03 | 0.03 | 0.03 | – |
| Weapon Recoil | −0.01 | −0.01 | −0.01 | −0.01 | – |
| Zoom | 0.006666 | 0.004666 | 0.0089 | 0.006666 | – |
| Range (m) | – | – | – | – | 0.02158 |
| Combo Duration (s) | – | – | – | – | 0.09 |
| Initial Combo | – | – | – | – | 0.27224 |
| Additional Combo Count Chance | – | – | – | – | 0.00653 |
| Chance to Gain Combo Count | – | – | – | – | −0.01165 |
| Critical Chance for Slide Attack | – | – | – | – | 0.013334 |
| Finisher Damage | – | – | – | – | 0.0133 |
| Heavy Attack Efficiency | – | – | – | – | 0.00816 |

Spliced traits (always positive): Blast, Corrosive, Gas, Magnetic, Radiation, Viral 0.01 (all classes); Damage vs
Orokin / Scaldra / Techrot 0.005; Status Damage 0.01; Weak Point Damage 0.025 and Weak Point Critical Chance 0.0275
(ranged only); Ammo Efficiency −0.005 (ranged); Reload While Holstered 0.01 (rifle, shotgun, pistol); melee only:
Heavy Attack Wind Up Speed 0.0133, Melee Damage On Heavy Attack 0.0133, Slam Attack Damage 0.0133, Parry Angle 0.009.
Splice recipes are `09`'s (`F:mods` §8.4).

---

## 9. Set bonus effects

Counting (number of different set mods on the warframe, the **active** weapon and the companion since U38.5; a mod on
both the player's weapon and the robotic weapon counts once; rank does not matter) is `09`'s (`F:mods` §7.1). U41
fixed Gladiator, Vigilante, Carnis, Mecha, Nira's, Saxum, Synth and Tek bonuses "not applying": pre-U41 data may
encode broken behaviour.

| Set | Effect by number of pieces N | Bucket | Source |
|---|---|---|---|
| Vigilante | crit tier raise chance `p_vig = 0.05·N` (N = 1 … 6) on critical hits of primary weapons | `tier_raise_chance` (`02`, E-6) | `W:Vigilante_Set`; PE `ExportModSet` |
| Gladiator | melee CC_rel `+0.10·N·(M(c) − 1)` | CC_rel, class (b) combo | `W:Gladiator_Set` (`110 = 10 × 11` at 12×) |
| Sacrificial (2 pieces) | both mods' values ×1.25 with 2 pieces (Steel CC 2.20 → 2.75 (×2 on heavies), Pressure A 1.10 → 1.375); Sentient bonus column non-linear (OQ-415) | CC_rel, A | `W:Sacrificial_Set`; PE `modSetValues [0.25, 0.5]` |
| Nira's | Slam Damage `+1.50·N` (U39; was 1.00) | Slam factor (G) | `W:Nira's_Set` |
| Mecha | mark every 60 / 45 / 30 / 15 s for 3 / 6 / 9 / 12 s (N = 1 … 4); enables Mecha Empowered (+1.50 vs marked, §3.2); needs a Kubrow or Predasite; the Mecha Empowered aura itself does **not** count towards N (bug: "This mod does not provide a set bonus, despite what the arsenal UI shows"; Decision: the engine follows the bug, OQ-439) | class (c): default uptime `duration/interval` (0.05 / 0.133 / 0.30 / 0.80) (UNVERIFIED); nonlinear DoT use per §4.3 (mixture) | `W:Mecha_Set`; `W:Mecha_Empowered` §Bugs |
| Tek | Marked Zone radius 3 / 6 / 9 / 12 m, cooldown 60 / 45 / 30 / 15 s; Tek Collateral +1.00 CD inside | CD_rel, class (c) | `W:Tek_Set` |
| Strain | Strain Infection +0.20 CD per Cyst; Cysts 2 / 4 / 6 / 8 → up to +1.60 (derived, UNVERIFIED) | CD_rel, class (c) | `W:Strain_Set` |
| Synth | reload while holstered | none | `W:Synth_Set` |
| Augur, Hunter, Carnis, Jugulus, Saxum, Umbral | no weapon-damage set effect (Hunter: companion only; the pieces' own stats are ordinary mods; Umbral: warframe stats) | – | `F:mods` §7.2 |

```text
E-4.19  Gladiator (melee instance with combo multiplier M(c))
  ΣCC_rel += 0.10 · N_glad · (M(c) − 1)          additive with Blood Rush and True Steel
```

---

## 10. Custom stats (overview 6.3)

Every custom stat is unconditional (class "–") and enters its bucket exactly once per instance, with the gate shown.

| Key | Bucket, formula position | Gate | Notes |
|---|---|---|---|
| `base` | A | – | negative values allowed (D-405) |
| `base_per_status` | b_co (E-4.1) | – | follows the attack's CO behaviour; 0 when T-02 = `none` |
| `flat_change_dmg` | X (`incarnon_flat`, E-1) | – | excluded from B_co on adding attacks (E-4.3) |
| `abilityCombo` | T-11 value | exalted | `07` |
| `crit_chance` / `flat_crit_chance` | CC_rel / CC_abs (E-5) | – | |
| `mult_crit_chance` | `CC ← CC·(1 + x)` after E-5 | – | game source: Deadly Maneuvers (§5.3; "multiplicatively applied after buffs"); overview 6.3's "no game source; custom only" needs amending (OQ-438) |
| `crit_chance_weakp` | CC_rel | weak-point hits | |
| `crit_mult` / `crit_mult_add` / `crit_mult_mult` | CD_rel / CD_abs / `cd_field_mult` (E-4) | – | |
| `status_chance` / `absolute_status_chance` / `status_chance_mult` | SC_rel / SC_abs / `sc_mult` (E-8) | – | |
| `status_damage` | A_sd | DoT only | |
| `vuln_status_damage` | factor `(1 + x)` in `M_dot,s` (E-13) | DoT only | |
| `speed`, `multishot`, `magazineSize`, `reloadTime` | FR or AS, MS, magazine, reload | – | |
| `mult_for_head` | `sum_hs_mult` | headshots | |
| `headshot_mult` | `sum_wp_dmg` (D-012) | weak-point hits | |
| `initialCombo` | initial combo count (`07`) | melee | |
| `dmgVulnerability` | `V_mult` entry (E-12, stage `05`) | – | not a weapon-side factor, so it does not double-dip DoT |
| `heatInherit` | Heat DoT elemental bonus of an external first Heat source (D-028, `03`) | Heat DoT | |
| `heatAdd` | `E[HEA]` non-combining (`01`) | – | |

---

## 11. Worked examples

All arithmetic checked with `python3` (scripts in the session scratchpad, not in the repository). Section 15 holds
the full test vectors; two short examples here show the two forms.

**Adding vs multiplying (`F:addmul` §1.8 example 1, 2).** B' = B_ref = 100, Serration A = 1.65, Galvanized
Aptitude 2 stacks with n = 3 → C = 0.8 × 3 = 2.4. Adding: `100 × 2.65 + 100 × 2.4 = 505`. Multiplying:
`100 × 2.65 × 3.4 = 901`. With Eclipse at Str 1 (G_F = 3): adding `100 × 2.65 × 3 + 240 = 1035` (the CO part is not
tripled); multiplying `901 × 3 = 2703`.

**Incarnon flat on an adding attack (Furis Incarnon, catalogue row 119).** B_ref = 100, X = 28 (B' = 128), Hornet
Strike A = 2.2, Galvanized Shot 3 stacks with n = 3 → C = 3.6: `128 × 3.2 + 100 × 3.6 = 769.6`; a naive
`128 × (1 + 2.2 + 3.6) = 870.4` overstates it by 13.1 %.

---

## 12. Edge cases

Every exception and quirk from the findings files that falls in this document's scope, plus out-of-scope items with
their owner. "Out of scope" rows are listed so that no research item is silently dropped.

| ID | Rule | Source |
|---|---|---|
| EC-04-01 | CO behaviour is per **attack**, not per weapon: Cedo primary fire adding vs alt-fire multiplying; Trumna main radial adding vs alt-fire multiplying; Kunai normal adding vs Incarnon multiplying; Exalted Umbra Blade light waves adding at 2.0 vs heavy waves multiplying. | `F:addmul` edge 1; catalogue; `W:Cedo` |
| EC-04-02 | Adding CO omits A and every G_F member; beam ramp-up is **not** omitted (G). | CO Mechanic §Math |
| EC-04-03 | Incarnon flat X is excluded from the CO base on adding attacks (Atomos, Bronco Prime, Cestra both attacks, Despair, Dual Toxocyst, Furis, Lato Vandal, Lex Prime, Vasto Prime, Zylok Prime, Braton line) and included on multiplying Incarnon attacks (Kunai, Mk1-Kunai). | catalogue; `W:Braton_Incarnon_Genesis` §Known Bugs |
| EC-04-04 | Bows and charge weapons use a reference value: uncharged (ρ_co 0.5 bows and Kuva Drakgoon, Evensong 0.65, Ballistica 0.25, Ballistica Prime 0.5, Rakta Ballistica 0.25), partially charged (Lanka 0.381, Miter 0.4), or unlabelled (Drakgoon 0.5714); Corvas (atmosphere) respects charge. Partial and quick shots are separate data attacks. | catalogue; §2.7 |
| EC-04-05 | Child projectiles use the parent's damage: Kulstar 2.6667, Sporelacer (secondary) bomblet 6.3333, Zarr 1.6667, Kuva Zarr 3.3333, Mutalist Cernos charged cloud 41.0 (uncharged arrow 205), Kuva Bramma cluster contact 3.8163 (row 202, rule D5a); Phantasma child bombs are the exception (`multiplying`). | catalogue; CO Mechanic §Application Behaviors |
| EC-04-06 | Hitscan AoE: CO only on the directly hit target, scaled from the hitscan damage, the radial attack not multiplied by multishot. Beam AoE (Glaxion Vandal) gives CO to every target in the AoE. | catalogue; `F:weapon` §10; §2.9 |
| EC-04-07 | CO does nothing on: Balefire Charger, Stug (incl. Incarnon), Ferrox embedded alt-fire, Azima turret, Sonicor, Scyotid grab, Coda Bassocyst alt, Proboscis Cernos radial and tendrils, Evensong charged radial, Tenet Tetra alt AoE, Vadarya Prime strikes, Zhuge Prime embedded explosion, Akarius Prime detonation, Castanas and Sancti Castanas detonations, Kompressa Prime radial, Latron Incarnon AoE. | §2.10 |
| EC-04-08 | Archguns get CO only from Secondary Shiver and Shattering Frost (as of U43). | CO Mechanic §Archguns |
| EC-04-09 | Noctua (200, 2500, Wordwarden 200) and Artemis Bow (75 per arrow) CO bases ignore Ability Strength; Artemis Bow gets no CO while Concentrated Arrow is equipped (and got none at all before U36). | catalogue; CO patch history |
| EC-04-10 | Tenet Grigori energy disk: melee base sources (Pressure Point, Arcane Fury) do not apply; universal weapon damage does and is multiplied by CO (multiplying). | catalogue |
| EC-04-11 | Tenet Detron normal attack: CO only 65 % effective on multishot-created projectiles (κ_ms; proposed amendment of E-11, OQ-428; until accepted the engine gives every pellet the full term). | catalogue; TV-04-35 |
| EC-04-12 | A status counts only for later instances; pellets of one shot and gunblade pellets are sequential for n; sibling pellets do not share debuffs. | §2.3; CD-04 |
| EC-04-13 | Secondary Shiver counts Cold **stacks** (0.45 each, 4.50 at 10), additive with Galvanized Shot; earlier pellets' Cold procs count (OQ-433); Shattering Frost (`W:Decree`: "affected by Cold Status", "same behavior as Secondary Shiver") vs the CO page's "vs Frozen": reading UNVERIFIED. | `W:Secondary_Shiver`; `W:Decree`; OQ-402 |
| EC-04-14 | Stance CO effectiveness per input from 0.25 (Fracturing Wind block) to 2.33 (Mountain's Edge heavy); Exalted Blade 1.22-1.30 on every input; gunblade stances 1.00. | CO Mechanic §Stances; E-4.6 |
| EC-04-15 | CO is in the DoT seed (melee "including proc damage like Slash"); Tysis DoT excludes it; Stinger embedded DoT excludes it; Pox cloud recalculates CO each tick. | §2.13 |
| EC-04-16 | Zhuge Prime direct hit: "real" value 48.4375 "for both the base damage and aptitude damage" because of quantization; consistent with E-4.4 (q_t multiplies the CO term). | catalogue row 86 |
| EC-04-17 | Kulstar row: 75 → 200 is 267 %, the table prints 257 %; absolute values used. | OQ-411 |
| EC-04-18 | Kunai Swift Conclusion is +2.00 (perk page) not +1.00 (source list); it is a CO-type term (additive with Hornet Strike on the normal attack, multiplicative on the Incarnon form, additive with Galvanized Shot in both). | `W:Kunai_Incarnon_Genesis` |
| EC-04-19 | Sporelacer (secondary) normal attack uses 57 "for any configuration" (`fixed`). | catalogue |
| EC-04-20 | Mausolon alt-fire radial: "Actual damage is 5x higher than UI displays" (data note, UNVERIFIED). | catalogue; OQ-425 |
| EC-04-21 | Tenet Plinx alt-fire impact scales with magazine size (G) and the CO term with it. | catalogue |
| EC-04-22 | Argo & Vel thrown glaive (0.333), Arum Spinosa (0.2 / 0.1) and Quassus Prime (0.4 / 0.2) heavy daggers: CO part also ignores the combo multiplier (`co_omit = {combo}`). | catalogue |
| EC-04-23 | Odd melee ratios: Destreza spectral rapiers 2.0, Innodem aerial Incarnon projectile 2.0, Zenistar heavy disc 10.4, Edun and Stropha heavy projectiles 0.25. | catalogue |
| EC-04-24 | Tenet Agendus heavy waves (data "Energy Disk") are **multiplying at ratio 0.2**: `(1 + ρ_co·C)` is a proposed amendment of E-11 (OQ-428); until accepted the engine uses `(1 + C)`. | catalogue; E-4.3; TV-04-34 |
| EC-04-25 | Vulcax: measured 500 + 100 → 1000; ρ_co 2.0 on the hitscan attack, AoE child `none` (Decision). | §2.14; OQ-425 |
| EC-04-26 | Progenitor bonus is in the CO base (hitscan and melee fixed U31.1.7, projectiles U31.2 / U31.5; never broken on multiplying attacks). | CO patch history |
| EC-04-27 | Before U31.1 many projectiles used a CO base of 10; before U31.1.6 multishot multiplied CO per pellet: pre-U31.1 data is invalid. | CO patch history |
| EC-04-28 | Maximum distinct statuses 16 (Galvanized pages, no Tau) vs 17 (CO pages); Lifted and Knockdown exclusive; Microwave counts. | `03`; OQ-408 |
| EC-04-29 | Untested attacks (all Kuva primaries, most Tenet and Coda weapons, several melee) default to adding 1.0 (D6); non-radial child projectiles of a projectile parent default to the parent's base (D5a, Kuva Bramma row 202). | §2.5; OQ-409 |
| EC-04-30 | CO does not apply to slams, heavy slams, glaive heavy-attack explosions; it applies to the thrown glaive (U28.0.6). | `W:Condition_Overload`; `F:melee` §10 |
| EC-04-31 | Melee projectiles: two buckets (A_m, A_w); Vex Armor, Deathbringer, Octavia Amp, Rift Torrent, Lethal Levitation count in both; Eclipse and Garuda passive apply twice; Void Strike once; single-bucket and universal-only exceptions. | §3.4 |
| EC-04-32 | Primary Compression is G_F on projectile AoE but additive with the base bucket on hitscan-AoE attacks (Ambassador, Battacor alt-fire, Braton and Burston Incarnon, Ferrox, Tenet Ferrox, Mausolon, Opticor, Opticor Vandal, Trumna, Trumna Prime); per-attack effectiveness (Mausolon 4.44 when aiming during alt-fire). | `W:Primary_Compression` |
| EC-04-33 | Longbow Sharpshot on an adding bow does not multiply the CO term: `[(1 + Serration)(1 + 3)] + CO`. | `W:Longbow_Sharpshot` |
| EC-04-34 | Eclipse is additive-equivalent only against adding CO; Parasitic Link damage is G_F and its Strength bonus is the only multiplicative Strength source. | `W:Eclipse`; `W:Parasitic_Link` |
| EC-04-35 | Vauban passive and Mecha Empowered act like faction on DoT (second factor when applied while the condition holds); Vauban passive and Navigator are in G (not on the omitted list). | §3.2; OQ-404 |
| EC-04-36 | Vulnerabilities multiply each other except the additive group (Petrify, Containment Wall, Jade's Judgement, Prey of Dynar, Atomi-Barrage, Magus Accelerant, Magus Destruct, Theorem Contagion); type-restricted ones (Vast Untime Void, Hakchum Cold, Lull / Sling Stun / Savage Silence / Radiant Finish Finisher, Magus Heat / Puncture); no DoT double dip; since U37 several apply to shields and overguard. | `W:Damage_Vulnerability` |
| EC-04-37 | Roar is faction (additive with Bane and Damage Blessing); the strongest Roar overwrites; Strength is the caster's. | `W:Roar`; §4.4 |
| EC-04-38 | Helminth values: Roar 0.30, Eclipse 0.30 (U35.5, was 1.50), Nourish 0.45 (U35.5), Wrathful Advance 1.00 (also exalted melee), Warcry 0.30, Xata's Whisper unchanged; slot restriction is `09`'s. | `W:Helminth`; §5.6 |
| EC-04-39 | Xata's Whisper double-dips body part and faction (`W:Xata's_Whisper` line 25: "double dips on faction damage, and body part weaknesses"); other extra hits have a forced 1× weak-point multiplier; `W:Sonar` claims Sonar applies again to extra hits (contradiction). `F:buffs` §1.3's head example (465.8) applies the body part only once and contradicts its own text; the engine follows E-14 (931.5, TV-04-11). | E-4.13; OQ-412 |
| EC-04-40 | Xata's Whisper is triggered by Blast detonations (bug), with triple faction and elemental bonuses. | §6.2; D-071 |
| EC-04-41 | Xata's Whisper and Toxic Lash are reduced by ability-damage reduction (Ancient Disruptor): out of scope (`05`, enemy auras). | `F:buffs` §1.3 |
| EC-04-42 | Toxic Lash ×2 on melee; Resupply ×2 on sniper rifles, not battery weapons, not Incarnon forms, not archguns; Silken Stride ×2 while wall latching, not Incarnon forms, not archguns; Demonium, Gastroparesis and Sangodae primary and secondary normal forms only; only Xata's Whisper and Toxic Lash reach melee, Incarnon forms and archguns. | §6.1; `W:Extra_Hit` §Sources |
| EC-04-43 | Toxic Lash ticks are multiplied by every Toxin mod of the weapon, also when combined into a secondary; Nourish Viral does not count. | E-4.14 |
| EC-04-44 | Extra hits: one per multishot pellet; Acid Shells, Concealed Explosives and Nightwatch Napalm hits and the radial damage of slams and explosions trigger them in game (proposal to amend E-14, OQ-412; until then the engine triggers on direct instances only); Bursting Mass and Pathocyst maggots do not; a lethal parent prevents them. | `W:Extra_Hit` §Interactions; `W:Toxic_Lash` §Tips; D-412 |
| EC-04-45 | Extra hits trigger on-hit effects (Secondary Enervate, Double Tap; each counts as one unit of the effect's `hit_unit`), give no melee or sniper combo; Toxic Lash gives no Incarnon charge. | `W:Extra_Hit` §Interactions, §Notes; §4.1 |
| EC-04-46 | Melee Duplicate: tier-1 crits only, chance 0.25-1.00 by rank, own crit and status rolls, faction once, forced weapon procs repeat (not Seeking Talons), slams and Exodia Contagion (not its embed DoT) trigger it; with Toxic Lash or Xata's Whisper it duplicates the extra hit instead. | E-4.15 |
| EC-04-47 | Primary Debilitate procs come from a 0-damage extra hit and triple-dip faction (conflicts with D-073). | §6.5; OQ-414 |
| EC-04-48 | Reconifex Active Reload is a 25 % Heat extra hit with a forced Heat proc. | `W:Extra_Hit` |
| EC-04-49 | Nourish Viral, field elements (Electric Shield, Conductive Sphere, Energy Shell) and Valence Formation never combine; Violet shard Electricity combines (combination position `01`, D-006). | §5.6, §5.9 |
| EC-04-50 | Electric Shield Electricity does not raise Electricity DoT; Conductive Sphere adds Electricity status damage; Valence Formation adds +2.00 to its element's DoT (not Blast). | §5.6 |
| EC-04-51 | Electric Shield crit factor does not stack across shields, its Electricity does (+0.50 per shield); projectile, hitscan and beam shots and projectile-based melee attacks benefit; a shot takes buffs from one field source only; melee swings and glaive throws get nothing; AoE only through its main projectile. | `W:Buffing_Fields`; `W:Electric_Shield` |
| EC-04-52 | Covenant: ×4 on weak points, caps 0.50 body / 2.00 weak point; Sonar spots count as weak points. | `W:Covenant` |
| EC-04-53 | Wrathful Advance is melee only (subsumed version also exalted melee). | `W:Wrathful_Advance` |
| EC-04-54 | Smeeta Charm sets a crit chance floor of 2.00; Cat's Eye and Avenger add. | `W:Charm`; OQ-416 |
| EC-04-55 | Metamorphosis Day decays linearly to 0 and does not affect melee; Shooting Gallery affects melee too. | §5.6 |
| EC-04-56 | Void Strike (+10.00 at full energy) is in A: real gain `(1 + A + 10)/(1 + A)` (≈ ×4.77 with Serration); 8 s per 40 s; not archguns. | `W:Focus/Madurai` |
| EC-04-57 | Violet shard Electricity per-shard term counts the shard itself; Tauforged per-shard 0.15; Violet melee crit doubling needs max Energy > 500 (501+), Arcane Battery counts since U42. | E-4.12 |
| EC-04-58 | Archon ability-damage bonuses need the status before the hit and never apply to exalted weapons. | D-068 |
| EC-04-59 | Emerald shard Corrosion stacks (+2 / +3) also raise 4-stack boss caps: `03`. | `W:Archon_Shard` |
| EC-04-60 | Coaction Drift: own aura ×1.3225 on oneself, ×1.15 on allies; enemy-debuff auras only ×1.15. | E-4.11 |
| EC-04-61 | Arcane Acceleration is not doubled for bows and excludes shotguns; Arcane Precision triggers on weak points since U44 and does not refresh. | §5.5 |
| EC-04-62 | Arcane Arachne: A on guns, A_w on two-bucket melee projectiles, additive on single-bucket projectiles, nothing on normal melee; Holster Amp and Vigorous Swap also not on normal melee. | §3.4, §5.5 |
| EC-04-63 | Arcane Crepuscular and Tenacious Bond are absolute crit damage: their relative gain falls as CDM rises. | §5.5, §5.11 |
| EC-04-64 | Buckets not documented: Absorb, Theorem Demulcent, Plunder (combining), Haste Mote, Cascadia Flare, Primary Bulwark, Temporal Shot, Eroding Rapids, Shotgun Vendetta additivity. (Secondary Surge, Secondary Enervate and Melee Doughty are now sourced: §3.2, E-4.21, §5.4.) | OQ-415 |
| EC-04-65 | Arcane Pistoleer grants infinite ammo (Hotfix 44.1.3): no damage bucket; `06` duty. | §5.5 |
| EC-04-66 | Phoenix Talons uses total base (elements included) and adds only physical: value here, arithmetic `01` (D-007). | §5.10 |
| EC-04-67 | Peacemaker: +1.5 × Str innate in A, ramp ×2.5 separate (G), counted in bursts of 2 shots (20 bursts = 40 shots to the maximum; the first burst already +0.075). | `W:Peacemaker` lines 24-35 |
| EC-04-68 | Stealth is additive with the crit term (`1 + SDB + k(CDM − 1)`); rank-40 weapons use `1 + 0.15·rank`; no SDB in the Simulacrum without enemy AI. | §7.1; OQ-407 |
| EC-04-69 | Finishers: FIN damage, half elemental and physical bonuses, valence weapons deal half (bug), crit formula conflict. | §7.2; OQ-407 |
| EC-04-70 | Faction applies twice to DoT and three times to extra-hit DoT. | E-4.14 |
| EC-04-71 | Combo-scaled bonuses (Blood Rush, Weeping Wounds, Gladiator) use `(M(c) − 1)` in the relative bucket (multiplicative before U26); whether heavies use the pre-consumption combo is `07`/`02` (`F:addmul` open 12). | §5.3, §9 |
| EC-04-72 | Out of scope (`02`): True Steel family and melee riven crit chance doubled on heavy attacks; Galvanized Steel heavy +2.20 (U37.0.4); Internal Bleeding doubled below 2.5 fire rate; Hunter Munitions roll per crit instance. | `F:addmul` edges 19, 25 |
| EC-04-73 | Rivens: dispositions retired in Hotfix 44.1.3; splice keeps the larger u; Damage can be negative despite the footnote; kitgun rivens use the Pistol column; robotic rivens the associated class. | §8 |
| EC-04-74 | Galvanized: Scope and Crosshairs stacks expire individually, the others lose one stack per timeout; Galvanized Shot 3 × 14 s vs Aptitude and Savvy 2 × 20 s; Galvanized Aptitude ignores the Braton Incarnon flat. | §5.2 |
| EC-04-75 | Primed Chamber and Charged Chamber stack additively (Primed Chamber is not a Primed variant); Charged Chamber does nothing on sentinel weapons; Synth Charge needs base magazine ≥ 6, excludes beams (except Catabolyst reload grenade), Incarnon forms and whole-magazine alt-fires. | §3.2 |
| EC-04-76 | Set bonuses: Gladiator per piece × (M(c) − 1); Nira's 1.50 per piece since U39; U41 fixed several sets; Sacrificial ×1.25 with both mods; counting is `09`'s. | §9 |
| EC-04-77 | Primary and Secondary Merciless reload line exists only at rank 5 (rank scaling `09`). | `F:mods` §3 |
| EC-04-78 | Torid Plentiful Mayhem: +0.60 damage on multishot projectiles in normal form; in Incarnon form it raises multishot bonuses by 60 % instead (`02`). | `F:mods` §9 |
| EC-04-79 | Archguns are neither primary nor secondary: class-restricted warframe arcanes (Arcane Precision, Acceleration) do not apply; universal ones (Avenger, Hot Shot) do. | `W:Archgun` |
| EC-04-80 | Spiteful Defilement counts distinct status types (not stacks) and is a flat +1.0 crit damage. | `W:Phenmor`; D-067; OQ-406 |
| EC-04-81 | Devouring Attrition is multiplicative with Serration and CO multiplies properly with it (Laetum rows); Overwhelming Attrition still applies when every weapon hit crits if Xata's Whisper is the only status source (bug). | `W:Phenmor`; `W:Extra_Hit` §Bugs |
| EC-04-82 | Kill-triggered effects are class (c): the layer views never kill the target. | §4.1 |
| EC-04-83 | Melee Careen ×2.5 only vs Frozen targets (Frozen targets also take +1.0 crit damage from any source, `03`). | `W:Melee_Careen` |
| EC-04-84 | Mecha Empowered needs a Kubrow or Predasite to mark; value per aura; twice on DoT. | §3.2, §9 |
| EC-04-85 | Exodia Contagion: stealth bonus applies; Melee Duplicate triggers on direct hit and explosion. | `W:Exodia_Contagion`; `W:Melee_Duplicate` |
| EC-04-86 | Secondary Shiver has reduced effectiveness on Ballistica charged shots and Incarnon secondaries and is multiplicative on Cyanex, Catchmoon, Epitaph, Seer series, Sepulcrum, Tenet Spirex: both follow from the catalogue. | `W:Secondary_Shiver` §Notes |
| EC-04-87 | Primary Plated Round scales with the rounds reloaded (partial reloads linear) and with sqrt of the magazine; it scales with the base magazine after an Incarnon transformation, not with Incarnon charges. | `W:Primary_Plated_Round`; D-401 |
| EC-04-88 | Decrees and mode blessings (Iceblade of Narin blessing, Cloak and Dagger, Critical Frost, Salted Wound, Wyrmling's Aid) are out of scope; only Shattering Frost is catalogued. | §5.11 |
| EC-04-89 | Melee Influence (spreads elemental procs incl. CO and crit; faction twice on spread hits, thrice on spread DoTs), Secondary Irradiate, Exodia Force: multi-target, out of the default metric. Melee Afflictions is single-target (stacks on the struck target): §6.6. | §5.4 |
| EC-04-90 | Health-threshold conditions (Kunai, Dread) are false in the non-depleting views under `simulated` and evaluated from the health fraction in the `layered` pass. | §4.3 |
| EC-04-91 | Ability Strength floor: `W:Power_Donation` implies a 10 % minimum ("can be used with Overextended to achieve 10 %"), UNVERIFIED; the engine does not clamp T-20 or Str_e. | `F:buffs` §0, Q12 |
| EC-04-92 | Exodia Contagion projectile: +0 / 1.00 / 2.00 / 3.00 damage after 30 m plus `(combo − 1)/4`; bucket UNVERIFIED (OQ-415); the attack itself is `07`'s. | `F:addmul` §2; `W:Exodia_Contagion` |
| EC-04-93 | Buff dynamics the user value must reflect: a Vex Armor recast with more Strength raises the cap only if not capped; several Chromas' Fury stack but Elemental Wards do not; only the highest Octavia Amp applies; several Citrines' Prismatic Gems stack; a stronger Roar overwrites a weaker one. | `F:buffs` §1.1, §1.6, §1.7, §1.10, §1.21 |
| EC-04-94 | Weapon-made buffing fields (Mutalist Quanta alt-fire, Simulor orbs, Mandonel charged shot) are catalogued as user-entered class (u) field buffs (§5.6) so that `02`'s bucket lists have values; a shot takes one field source only. | `W:Buffing_Fields` |
| EC-04-95 | Secondary Fortifier ×8 vs overguard (×3 at rank 0) is `M_OG`; the inspiration multiplies by Σ, not 1 + Σ, which is identical for a single arcane. | `W:Secondary_Fortifier`; JS L10407-10408 |
| EC-04-96 | The CO catalogue joins the weapon data by the exact data attack name, not the wiki attack name (only 65 of 201 wiki names are data names); an unresolved row fails the data build. | §2.14 |
| EC-04-97 | Secondary Enervate: the wiki's AFCC table is the mean of per-cycle averages; the long-run mean per hit is slightly higher below CC 2.0 (1.531938 vs 1.5245 at CC 1.0). | E-4.21 |
| EC-04-98 | Xata's Whisper on a Blast detonation is scaled by the weapon's whole elemental bonus (Electricity included in the page example). | E-4.13a |
| EC-04-99 | Arbucep's six burst projectiles are `multiplying`; only the 228-damage AoE gets no CO. Stinger is `adding` with its embedded DoT excluded. | rows 170, 196 |

---

## 13. Inspiration differences

Register rows of overview section 5.2 owned (fully or jointly) by this document, restated with the engine behaviour,
plus new rows `D-401` … in the same column layout. Cross-references: Archon Continuity and Archon Vitality are `02`'s D-204 (§5.13); Double Tap
and the other `multiple` keys are D-403.

| ID | Topic (owner) | Inspiration behaviour | Game behaviour (source) | Decision | Toggle |
|---|---|---|---|---|---|
| D-007 | Phoenix Talons (01, 04) | ×1.3 on physical types, not on exalted (JS L9254-9256, L9495) | +0.30 × total base as physical, split by physical shares, after physical mods, also exalted (`W:Focus/Madurai`) | game; value §5.10, arithmetic `01` | User (T-19) + Compat |
| D-010 | Riven Weak Point Critical Chance (02, 04) | stored under `SMITE`, no effect (JS L14722-14725) | relative crit chance on weak-point hits | §8.1 | — |
| D-011 | Riven Weak Point Damage (02, 04) | `mult_for_head` (JS L14728) | `sum_wp_dmg` | §8.1 | — |
| D-021 | Encumber (02, 04) | `round(random × 12)` (JS L10036, L12875) | random status | value 0.24 per proc (§5.4); uniform draw in `02` | — |
| D-022 | Stealth (04, 07) | non-crit ×7, crit `c + 7`, rank ignored (JS L8330-8336) | `1 + SDB + k(CDM − 1)`, SDB by rank (`W:Stealth`) | E-4.16 | User (T-08) + Compat |
| D-037 | Toxic Lash DoT duration (03, 04) | fixed 7.5 s (JS L5555-5561) | `floor(6·(1 + SD))` ticks, Saryn passive +0.25 SD | E-4.14 | — |
| D-041 | Mecha Empowered and Eclipse on DoT (03, 04) | Mecha unconditional, squared on elemental DoT; Eclipse once on elemental DoT, not Slash (JS L9157, L9253, L10444) | Mecha twice on every DoT vs marked targets; Eclipse once via the seed (`W:Mecha_Empowered`, `W:Damage_over_Time`) | §3.2; E-4.7 | — |
| D-056 | CO status count (04) | number of damage types of the attack, +1 external Viral (JS L12015-12020) | distinct statuses on the target, any source, excluding the instance's own | E-4.1, E-4.2 | User (T-05) |
| D-057 | CO adding vs multiplying (04) | global option or `isCoMult` (JS L9235, L9393) | per-attack catalogue | §2.14, E-4.3 | User (T-04) |
| D-058 | Adding-CO omissions (04) | CO term multiplied by Eclipse, `multiple` and other map multipliers (JS L9389-9401, L9492-9498) | G_F omitted from the CO term | E-4.3, §3.1 | Compat |
| D-059 | CO on AoE and Incarnon flat (04) | off on every AoE and slam; the flat path applies additive CO even with conditions off, on AoE and slams, and drops Sharpshot (JS L9402-9414) | hitscan AoE gives CO to the direct target; flat excluded from B_co on adding attacks only | §2.5, §2.9, E-4.3 | — |
| D-060 | Secondary Shiver (04) | +4.5 whenever conditions are on (JS L9365) | +0.45 per Cold stack | E-4.1 (n_COL from the state) | — (proposal: `Compat` with n_COL = 10, OQ-434) |
| D-061 | Conditional stacks (04) | `WITH_COND` max values when "trigger all conditions" is on, else ignored (JS L8668-8744; Merciless `WITH_COND 3.6`, L4594-4600; Avenger L5153-5158) | stacks ramp and decay with triggers | §4 (T-02 modes) | User (T-02) |
| D-062 | Light-attack combo (04, 07) | combo-scaled bonuses use 12× when the option is off (JS L8219-8224) | combo ramps from Initial Combo | simulated ramp (`07`); bonus formulas §5.3, E-4.19 | User (T-10) + Compat |
| D-067 | Spiteful Defilement (04) | whole crit multiplier ×2 when total stacks < 3 (JS L10752-10760) | +1.0 flat crit damage after percentage modifiers when distinct statuses < 3 (`W:Phenmor`, fetched 2026-11-23) | current decision: `sum_cd_rel` +1.0 when n < 3; proposal `sum_cd_abs` (OQ-406) | — |
| D-068 | Melee Careen and Electricity ability shard (04) | unconditional (JS L10466, L10389) | Careen only vs Frozen; ability shards never on exalted weapons | §3.2, §5.9 | — |
| D-071 | Xata's Whisper (04) | double-counted headshot and vulnerability, ×21 in non-crit branches, innate DR inside the armor term (JS L11403, L11584, L11763, L13226-13243) | E-14 | E-4.13; Blast-detonation trigger in MC (UNVERIFIED magnitude) | — |
| D-072 | Toxic Lash damage types (04) | extra hit inherits the weapon's per-type modifiers (JS L11465-11495); other-boss branch reads an undefined value (NaN, I:queue QM13) | a Toxin instance | E-4.13 | — |
| D-401 | Primary Plated Round (04) | `A += round2(0.33·sqrt(modded magazine))` (`parseFloat((0.33 * Math.sqrt(magazineSize)).toFixed(2))`) whenever equipped with conditions on; rounds reloaded ignored (JS L8446-8447, L10235-10239) | `0.15·sqrt(5·Mag)·(Mag − left)/Mag` for 10 s after a reload (`W:Primary_Plated_Round`; 25 rounds → +1.6771) | game, class (b) `on_reload` | Compat |
| D-402 | Melee Duplicate (04) | 100 % on tier-1 crits of non-slam attacks; clone rolls status with the displayed SC; its procs do not reach later timeline entries (JS L10697-10741; I:queue QM23, I:status Q32) | chance by rank; slams trigger; procs persist; Toxic Lash / Xata bug (`W:Melee_Duplicate`, `W:Extra_Hit`) | E-4.15 | — |
| D-403 | Shared `multiple` bucket (04) | Double Tap (`WITH_COND multiple: 4`, i.e. the 20-stack maximum on every hit, JS L229-233), Synth Charge, Secondary Surge, Power Throw … summed into one `(1 + Σ multiple)` on direct and status damage (JS L9366, L10465, L10498-10503) | independent factors; Double Tap ramps per hit and resets (`W:Double_Tap`); Power Throw in A_w (`W:Calculating_Bonuses`, `W:Power_Throw`) | §3.2, §3.4 | Compat |
| D-404 | Spring-Loaded Broadhead (04) | applied twice (per-type maps and per-hit factor, JS L9369, L9513, L10406, L10875), with no distance test (`WITH_COND`, JS L235-239) | once, only when the target is over 15 m away (`Module:Mods/data`) | once, gated on T-27 > 15 m | Compat |
| D-405 | Custom Base Damage (04) | negative custom `base` ignored (JS L8444-8445) | – (custom input) | negative honoured | — |
| D-406 | Zoom-level bonuses (04) | only the last zoom level, only with "trigger all conditions" (JS L9095-9103) | each zoom level has its own bonus while aiming | zoom level selectable, class (b) `while_aiming`, default highest under `all_max` | — |
| D-407 | Incarnon-form tables (04) | `incForm`/`normForm` take `WITH_COND` values even with conditions off (JS L8723-8724) | conditional perks depend on their condition | §4 | — |
| D-408 | Riven faction traits (04, 09) | rows Corpus, Grineer, Infested, Scaldra, Techrot only (JS L14717-14721) | Corpus, Grineer, Infested; splices Orokin, Scaldra, Techrot (`W:Riven_Mods`) | wiki list | — |
| D-409 | Primary Debilitate faction (03, 04) | faction applied a third time on its procs (JS L11459) | procs from a 0-damage extra hit triple-dip faction (`W:Extra_Hit` §Weapon Effects) | proposal: amend D-073 to "thrice" (OQ-414); until then D-073 | — |
| D-410 | Overwhelming Attrition (04) | `base_uncrit_unstatus` +12 on every hit, crits and status hits included (JS L11401, L11575) | only instances that neither crit nor apply a status | gated (§5.12) | — |
| D-411 | Reconifex Active Reload (04, 06) | weapon unique `reconifexHeat`: extra Heat added to status damage, armor halved, re-counted (JS L11964-11984; I:queue QM19) | 25 % Heat extra hit with a forced Heat proc (`W:Extra_Hit` §Weapon Effects) | E-4.13 | — |
| D-412 | Extra hits on radial instances (04) | no shot-type test around the extra-hit code (JS L11400-11500): every evaluated hit of any attack, AoE attacks included, gets Xata's Whisper and Toxic Lash | radial damage of slams, explode-on-death mods, Concealed Explosives and Nightwatch Napalm create extra hits (`W:Toxic_Lash` §Tips; `W:Extra_Hit` §Interactions); overview E-14 says "parent direct instance" | E-14 as written (direct only) until amended; proposal: direct and radial instances on the target (OQ-412) | — |
| D-413 | Enraged (04) | tagged primary, secondary, melee and arch-gun; `base: 2, crit_chance: 2` on every weapon (JS L4966-4970) | Hysteria (Valkyr Talons) only, +3.50 damage and crit chance at rank 3 (`W:Enraged`) | game (§5.6) | — |
| D-414 | Maiming Strike (04) | `crit_chance_slide: 1.2` (JS L2343-2348) | +1.50 slide crit chance (`W:Maiming_Strike`; `Module:Mods/data`) | game (§5.3) | — |
| D-415 | Arcane Pistoleer (04, 06) | `WITH_COND ammoEff: 1` ("+102% Ammo Efficiency", JS L5080-5087) | infinite ammo for 12 s since Hotfix 44.1.3 (overview §1.1) | game: `06` infinite-ammo state | — |
| D-416 | Arcane Momentum (04) | `WITH_COND reloadTime: 1.5, charge_time: 1.5` (JS L5143-5150) | reload speed only (`M:Arcane`) | game (reload only) | — |
| D-417 | Burning Hate (04) | `vuln_status_damage: 1.2` unconditional (JS L205-209) | only for procs created after a Hate hit on a Heat-affected target (`W:Burning_Hate`) | gated (§5.3) | — |
| D-418 | Deadly Maneuvers (04) | `WITH_COND mult_crit_chance: 4` on every shot (JS L600-603) | headshots of the next 2 shots after a dodge (`W:Deadly_Maneuvers`) | gated, class (c) (§5.3) | — |

---

## 14. Open questions

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-401 | Stance CO ratio r_in: per hit or averaged over the input's hits? | same r_in on every hit of the input | per-hit damage numbers of a multi-hit input (Gnashing Payara neutral) with CO and 2 external statuses vs without |
| OQ-402 | Shattering Frost: one fixed term while any Cold stack is present (`W:Decree` "affected by Cold Status"), only vs Frozen (CO Mechanic line 39 "+80% vs Frozen"), or per Cold stack (`W:Decree` "same behavior as Secondary Shiver")? Values per level | any Cold stack, fixed term; 0.80 / 1.60 / 2.40 (`W:Decree`) | Koumei archgun hits on targets with 1, 5 and 10 (Frozen) Cold stacks |
| OQ-403 | Are G_F members (Eclipse, falloff …) also left out of the CO part of the DoT seed on adding attacks? | yes (seed uses Y) | Slash tick of an adding hitscan with Galvanized Aptitude and Eclipse vs the two predictions |
| OQ-404 | Are Vauban passive and Navigator omitted from the adding CO term like Overdriver? | no (G) | adding attack with CO and 2 statuses on an incapacitated target, Vauban passive on/off |
| OQ-405 | Adding CO on two-bucket melee projectiles: is the CO term multiplied by (1 + A_w)? Bucket of Provoked, Absorb, Metamorphosis, Shooting Gallery on melee projectiles | yes; W | glaive throw with CO, Arachne on/off, 2 statuses |
| OQ-406 | Spiteful Defilement bucket: `sum_cd_rel` (D-067) or `sum_cd_abs` (`W:Phenmor`: "a +1x flat bonus to critical damage, calculated after percentage modifiers, similarly to Tenacious Bond")? | D-067 (`sum_cd_rel`) until amended. **Required register amendment** (overview §1.4 rule 3: the per-perk page is the game source): change D-067's decision to `sum_cd_abs`; `02` then moves the entry from ΣCD_rel to ΣCD_abs | Phenmor crit numbers with Vital Sense on/off against a target with 0 statuses |
| OQ-407 | Stealth on rank-40 weapons (`1 + 0.15·rank`, `W:Finisher`, Kuva Shildeg example) vs overview `1 + 0.2·min(rank, 30)`; finisher crit term (`1 + SDB + CDM` vs `1 + SDB + k(CDM − 1)`); the "half elemental and physical" rule; scope of the valence ×0.5 bug | overview formula (proposal: rank-40 form); Stealth page crit form; halve E_t and P_t; ×0.5 for valence weapons | stealth hits with a rank-20 Kuva melee; stealth finisher crit numbers; finisher numbers with one elemental mod |
| OQ-408 | Maximum distinct statuses for CO: 16 or 17? | per `03` | target carrying 17 types incl. Tau and Microwave, CO hit numbers |
| OQ-409 | Behaviour of untested attacks (Kuva primaries, most Tenet and Coda weapons, several melee); the D5a default for child projectiles (Kuva Bramma cluster contact ratio 187/49) | D6 adding 1.0; D5a parent ratio; `co_tested = false` | catalogue updates; per-attack tests with Galvanized Aptitude and 2 external statuses; Kuva Bramma cluster-contact numbers |
| OQ-410 | Partial and quick shots (separate data attacks, `06` OQ-607): does their CO base follow the uncharged reference like the charged shot, or their own damage (D6)? | own row or D6 (ratio 1 of their own damage) | Lanka "Partially Charged Shot" and Paris "Uncharged Shot" with Galvanized Aptitude and 2 external statuses |
| OQ-411 | Kulstar cluster ratio: 200/75 (2.667) or the printed 257 %? | 2.667 | Kulstar cluster numbers with Galvanized Shot |
| OQ-412 | (1) Extra hits: Sonar applied again (`W:Sonar`) or forced 1× (`W:Extra_Hit`)? (2) **Proposed amendment of overview E-14**: "one extra hit per pellet of a parent direct **or radial** instance on the target" (`W:Toxic_Lash` §Tips: slam radial damage, explode-on-death mods, Concealed Explosives, Nightwatch Napalm, "especially useful for … Ogris"; `W:Extra_Hit` §Interactions); D-412. (3) One extra hit per merged beam tick? | (1) forced 1× except Xata; (2) direct instances only until the manager amends E-14; (3) one per merged tick | Toxic Lash numbers on an Ogris explosion and a slam; Toxic Lash on a Sonar spot; beam with multishot |
| OQ-413 | Does the DoT "+1" accumulator apply to extra-hit procs (Toxic Lash tick 39 vs 39.5)? | no | low-damage Toxic Lash tick |
| OQ-414 | Primary Debilitate: faction twice (D-073) or thrice (`W:Extra_Hit`), and what seeds its procs? | D-073 until the manager amends it; seed per `03` | Debilitate Heat tick with Primed Bane on/off |
| OQ-415 | Undocumented buckets or forms: Absorb (unit of `sqrt(0.025 % · Str · dmg)`), Theorem Demulcent, Plunder (combining), Haste Mote and Redline (additive with mods), Cascadia Flare, Primary Bulwark, Temporal Shot, Eroding Rapids ("status vulnerability"), Shotgun Vendetta (additive with mods), Strain Infection total, Sacrificial Sentient column, Mandonel field Radiation "(multiplicative)" | values and buckets as in §5 (A for "+X % damage") | per-item in-game tests or page updates |
| OQ-416 | Smeeta Charm: floor 2.00 (`W:Charm`) or +2.00 absolute (`W:Universal_Weapon_Bonuses`)? | floor (overview `cc_floor`) | crit numbers of a 250 % CC weapon during Charm |
| OQ-417 | Vulnerability values still not researched (Jade's Judgement, Prey of Dynar, Atomi-Barrage, Lull, Sling Stun, Savage Silence, Radiant Finish, Demonium, Contamination Wave, Beguiling Lantern, Dual Coda Torxica); Strangledome 1.00 vs 2.00; Magnetize Strength scaling; Splinter Storm on weapon damage | user value; 1.00; factor 2.0 at Str 1; yes | ability pages; in-game numbers |
| OQ-418 | Octavia Amp value by sound level; Redline battery curve; Garuda passive kill rate | user inputs | page updates |
| OQ-419 | Melee Duplicate: can a duplicate trigger another? Does the duplicate see the parent's procs (CD-04)? Default ability priority when Toxic Lash and Xata's Whisper are both active | no; yes; first in the buff list | high-CC melee damage logs; Saryn with subsumed Xata |
| OQ-420 | Proposed toggles: "derive Strength from build" (sum of §5 strength sources, Parasitic Link multiplicative), per-buff caster Strength, and the self-cast Ability Duration Dur_e (§4.4) | T-20 is the total; caster Str per buff = 1.0 for squad buffs; Dur_e = 1.0 | manager decision (overview §6) |
| OQ-421 | Proposed toggle: Covenant with weak-point ×4 and the 0.50 / 2.00 caps (T-18 has no weak-point split) | T-18 flat value | manager decision |
| OQ-422 | Mecha Empowered: per-aura stacking and mark uptime from the Mecha set count | `1 + 1.50` per aura; uptime duration/interval | timing of marks in game; damage numbers vs marked targets |
| OQ-423 | Tauforged Violet melee crit damage 0.375 or 0.37; Tauforged Topaz 0.015 per kill | 0.375; 0.015 | in-game arsenal readout |
| OQ-424 | Riven: does a spliced trait use the layout L? Can Damage roll negative? Is W computed in f32 before the product? | yes; yes; yes | riven range readouts from the in-game Riven Splicer and trait locking |
| OQ-425 | Vulcax split of the CO base between attack and AoE; Mausolon alt-fire "5× UI" note | ρ_co 2.0 on the hitscan attack; ratio 0.33 | per-instance damage numbers |
| OQ-426 | Primary Compression: per-attack effectiveness data, modded radius used, "Doesn't Work" rows | page table imported; modded radius | arsenal radius vs damage numbers with Primed Firestorm |
| OQ-427 | Accuracy of E-4.9, and the stack-timer kind of Conjunction Voltage, Primary Crux, Melee Exposure and Split Flights (`shared_lose_all`, UNVERIFIED) | as §4.2 | MC vs EV comparison (`08`); arcane timer observations |
| OQ-428 | **Proposed amendments of overview E-11** (and of `01` §4.14, which restates it): (a) `multiplying: Y = B'(1 + A)(1 + ρ_co·C)G_F` for multiplying rows with ρ_co ≠ 1 (Tenet Agendus waves 0.2); (b) `adding: … + B_co·C·κ` with κ = co_ms_factor on multishot-created pellets (Tenet Detron 0.65) | E-11 as written until the manager amends it (TV-04-34, TV-04-35 give both forms) | Tenet Agendus heavy wave numbers with CO on 1 and 2 statuses; Tenet Detron pellet numbers |
| OQ-429 | Default for uncatalogued hitscan explosions: adding with the parent hitscan base | yes | any uncatalogued hitscan-explosion weapon with Galvanized Aptitude |
| OQ-430 | Sonar on DoT: `W:Damage_over_Time` line 171 ("only increase damage of the initial hit, not the resulting proc damage") vs `W:Sonar` line 25 ("DoTs … will have their damage calculated on the resulting damage if proc'd from a hit that hits a Sonar weakspot"); `F:buffs` §1.16 sides with `W:Sonar`. Owner of the rule: `03` / overview E-13 | overview E-13 (`Part_dot = Part / M_son`, not inherited) | Slash tick from a hit on a Sonar spot vs on the same part without Sonar |
| OQ-431 | Add class (u) "user-enabled buff" to overview 6.2: ability buffs, vulnerabilities, squadmates' auras, focus, companion and squad buffs enabled in the scenario apply in every T-02 mode with their user inputs (§4.1) | this document applies it (Decision) | manager decision; matches overview T-13 … T-19 and the inspiration (JS L9104-9109, L8668-8744) |
| OQ-432 | Timing-stat effects with random triggers under `simulated`: constant user uptime (`06` §5.9, default 1) or the E-4.9 steady state iterated to a fixed point with the fire rate (at most 20 iterations, relative tolerance 1e-6)? | user uptime, aligned with `06` (`06` OQ-621 owns the conflict with overview 6.2) | manager decision; `08` MC-vs-EV comparison of Arcane Velocity on a pistol |
| OQ-433 | Do earlier pellets' Cold procs count for a later pellet's Secondary Shiver term (CO rule) or not (sibling-debuff rule of CD-04)? | they count (E-4.1, E-4.2) | Shiver shotgun pellet numbers on a fresh target |
| OQ-434 | Change register row D-060's Toggle from `—` to `Compat` (compatibility profile n_COL = 10), so oracle tests may use Secondary Shiver | proposal | manager decision |
| OQ-435 | Ripper's Wail: decay rate of the Heat bonus (page: "?%/s") and whether the bonus raises Heat DoT | user value; raises DoT (U38 rule for elemental augments) | timed Heat readouts after the last Backbeat; Heat tick with and without the buff |
| OQ-436 | Secondary Enervate hit units: does an extra hit in the same game tick as its parent count as a separate hit? Do projectiles of one shot always register once on a single target? | separate hit; once per shot | Enervate stack counter with Xata's Whisper on and off; shotgun-sidearm projectile weapon |
| OQ-437 | Proposed toggle "finisher mode" (off / stealth / back / front / universal); is q'_t quantized on the 1/32 grid with halved bonuses? | proposal; quantized | manager decision; finisher numbers with one elemental mod |
| OQ-438 | Deadly Maneuvers: factor ×5 (rank table "+400%", inspiration `mult_crit_chance: 4`) or ×6 (page example 108.14 % → 648.84 %)? Amend overview 6.3, which calls `mult_crit_chance` "no game source; custom only" | ×5; amendment proposed | Magnus headshot crit numbers after a dodge |
| OQ-439 | Does the Mecha Empowered aura count towards the Mecha set count? | no (page bug note) | arsenal set count vs mark interval in mission |
| OQ-440 | Grave Spirit vs Doomed targets: does +1.00 × Str replace or add to the +0.50 × Str? | replaces | Dagath crit numbers on a Doomed and a non-Doomed target |
| OQ-441 | Melee Afflictions seed: does the Proc MBD include the source hit's CO term and `G` members? Is the trigger table (Overguard, frozen) current? | no; yes | Afflictions tick numbers with Condition Overload on and off |

---

## 15. Test vectors

Computed with `python3` (f64; numpy f32 where marked). Unless stated: G = 1, F = 0, Part = 1, k = 0, T-02
`all_max`, T-05 `manual(n)` with the n given, no Incarnon flat. Outputs are Y (E-4.3) or the stated quantity.

**TV-04-01: the three behaviours.** B' = B_ref = 100, A = 1.65, b_co = 0.8, n = 3 (C = 2.4), G_F = 1, ρ_co = 1.
Expected Y: adding **505.0**, multiplying **901.0**, none **265.0**.

**TV-04-02: G_F (Eclipse).** As TV-04-01 with Eclipse 2.0 at Str 1 (G_F = 3). Expected: adding **1035.0**,
multiplying **2703.0**.

**TV-04-03: Incarnon flat.** B_ref = 100, X = 28 (B' = 128), A = 2.2, b_co = 1.2 (Galvanized Shot, 3 stacks),
n = 3 (C = 3.6). Expected: adding **769.6**; multiplying (flat included) **1884.16**.

**TV-04-04: bow.** Paris "Charged Shot": B' = B_ref = 320, ρ_co = 0.5, A = 1.65, b_co = 0.8, n = 2 (C = 1.6).
Expected adding **1104.0** (naive `320 × 4.25 = 1360`). Paris "Uncharged Shot" (separate data attack, D6):
B' = 160, ρ_co = 1: **680.0**.

**TV-04-05: child projectile.** Kuva Zarr child bomb: B' = B_ref = 15, ρ_co = 50/15, A = 1.65, C = 1.6. Expected
adding **119.75**.

**TV-04-06: hitscan AoE.** Opticor charged radial: B' = B_ref = 400, ρ_co = 2.5, A = 1.65, C = 1.6. Expected:
directly hit target **2660.0**; any other target (`none`) **1060.0**.

**TV-04-07: sequential pellets (MC rule).** B' = B_ref = 100, A = 1.65, b_co = 0.8 (Aptitude, 2 stacks), one shot of
3 pellets, fresh target, every pellet applies one new status type. Expected Y per pellet: **265.0, 345.0, 425.0**;
sum **1035.0** (`W:Galvanized_Aptitude` §Notes).

**TV-04-08: sequential pellets (EV, E-4.2).** Fresh target (E[n_pre] = 0, nothing present), SC = 0.5, two status
types with w = (0.25, 0.75), no forced procs, MS = 2.6, b_co = 0.8, B' = B_ref = 100, A = 1.65. Expected:
p_s = (0.125, 0.375); E[n_0] = 0, E[n_1] = **0.5**, E[n_2] = **0.84375**; E[Σ_j C_j] = **0.805**; E[Σ_j Y_j] =
**769.5** (`265 + 305 + 0.6 × 332.5`).

**TV-04-09: full pipeline with CO (extends overview section 3.8).** Braton Prime normal attack (IMP 1.75, PUN 12.25,
SLA 21; B' = B_ref = 35), Serration A = 1.65, Galvanized Aptitude 2 stacks (b_co = 0.8), T-05 `manual(2)` (C = 1.6),
Infected Clip + Stormbringer (q = 2/32, 11/32, 19/32, 58/32 for IMP, PUN, SLA, COR), Primed Bane F = 0.55, Point
Strike and Vital Sense (CDM = Q(2.0)·2.2 = 4.4010747, f32 quantization), target Lancer level 100 Steel Path
(A0 = 1355, 1 − DR = 0.362426, φ IMP 1.5, COR 1.5). Adding (default):
- Y = 35 × 2.65 + 35 × 1.6 = **148.75**
- W per type (k = 0): IMP **14.410156**, PUN **79.255859**, SLA **136.896484**, COR **417.894531**; H_w = **648.457031**;
  k = 1: **2853.907862**
- armored health, k = 0: IMP 7.833933, PUN 28.724422, SLA 49.614910, COR 227.184063, sum **313.357328**; k = 1 sum
  **1379.109021**
- Slash DoT seed (E-4.7) **230.5625**; tick with "+1" (E-13) **125.622656** (k = 0), **552.874700** (k = 1)
- Xata's Whisper 0.26 on the k = 0 pellet: XH **261.328184**, on armored health **94.712252**
- T-04 on (multiplying): Y = **241.15**, H_w = **1051.263281**, armored sum **508.007527**

**TV-04-10: stance ratio (E-4.6).** B' = B_ref = 200, S_j = 1.5, A = 1.65, b_co = 0.8, n = 4 (C = 3.2). Expected Y:
r_in = 0.36 (Gnashing Payara neutral) **1140.6**; r_in = 1 **1755.0**; r_in = 2.33 (Mountain's Edge heavy)
**3031.8**.

**TV-04-11: extra hits (E-4.13, E-4.14).**
- Toxic Lash 0.30 on a gun hit of 100 base with Infected Clip (E_TOX = 0.9, F = 0): XH = **57.0**; Toxin tick
  `57 × 0.5 × 1.9` = **54.15** (`W:Extra_Hit` §Notes).
- Toxic Lash on melee (p = 0.60), H_w = 1000, F = 0.3: XH = **780.0**; tick (E = 0, A_sd = 0) `780 × 0.5 × 1.3` =
  **507.0**.
- Xata's Whisper 0.26, parent 100 base, A = 1.65, k = 1 with CDM 2.0, F = 0.3: body H_w = 689, XH = **232.882**;
  head (Part = 2): H_w = 1378, XH = `1378 × 0.26 × 1.3 × 2` = **931.528**.

**TV-04-12: stealth (E-4.16).** CDM = 3.0, h = 1. Rank 30: SDB = 7, c_0 = **8**, c_1 = **10**, c_2 = **12**;
rank 15: SDB = 4, c_0 = 5, c_1 = 7; rank 0: SDB = 1, c_0 = 2. Proposed rank-40 form (OQ-407) at rank 30:
SDB = **5.5**, at rank 40: **7.0**.

**TV-04-13: riven value (E-4.18, f32).** Rifle Critical Chance (b = 0.016666), rank 8, layout 2+1 (L = 1.2375):
u = 0 → **2.0046701**, u = 0.5 → **2.2274113**, u = 1 → **2.4501524**. Rank 0, u = 0.5 → **0.24749011**. Faction
(b = 0.005), rank 8, 2+0 (L = 0.99): u = 0.5 → v = **0.5346** (×1.5346), u = 0 → 0.48114002, u = 1 → 0.58806.
Rifle Damage negative (b = 0.0183333, L = −0.495), u = 0.5 → **−0.98009825**.

**TV-04-14: condition state machine (E-4.8, E-4.9).**
- Galvanized Shot, 3 stacks at t = 0, no further kills (`shared_lose_one`, D = 14 s): stacks at t = 0, 13.9 → 3
  (b_co 1.2); t = 14, 27.9 → 2 (0.8); t = 28, 41.9 → 1 (0.4); t = 42 → **0**.
- Galvanized Scope stacks gained at t = 0, 1, 2, 3, 4 (`independent`, 12 s): at t = 12.5 → **4** stacks; with the
  weak-point-hit part active, CC_rel = 1.20 + 4 × 0.40 = **2.80**.
- Single stack, refresh: λ = 0.075 /s (Arcane Rage: 0.5 headshots/s × 0.15), D = 24 s → E[value]/v =
  `1 − e^(−1.8)` = **0.834701**.
- `shared_lose_one`, m = 3, g = 1, λ·D = 0.5: stationary post-event π = (0.426882, 0.276927, 0.296191) for k = 1, 2,
  3; time-average E[stacks] = **0.985851** (three Monte Carlo runs of 3 × 10^6 Poisson events each gave 0.98408 to 0.98647).
- `independent`, m = 5, λ·D = 3.0: E[min(N, 5)] = **2.865379**.

**TV-04-15: Primary Compression (§3.1) on Acceltra (catalogue row 1, adding, ρ_co = 26/35).** R_mod = 4.0 m, rank 5:
bonus = 0.8 × 4.0 × 1.00 = **3.20** (with Primed Firestorm R_mod = 5.76: **4.608**). B' = 35, B_co = 26, A = 1.65,
C = 1.6, G_F = 1 + 3.2: Y = `35 × 2.65 × 4.2 + 26 × 1.6` = **431.15** (without Compression **134.35**).

**TV-04-16: Primary Plated Round (§5.4).** Mag 25, empty reload: A += **1.677051**; Mag 40 empty: **2.121320**;
Mag 80 with 40 left: **1.5**; Mag 120 with 80 left: **1.224745** (`W:Primary_Plated_Round` values 167.71 %, 212 %,
150 %, 122 %). Inspiration (D-401, `round2(0.33·sqrt(Mag))`, independent of the rounds reloaded): Mag 25 → 1.65,
Mag 40 → **2.09**, Mag 80 → **2.95**, Mag 120 → **3.61**.

**TV-04-17: Secondary Shiver + Galvanized Shot.** Pistol B' = B_ref = 100, Hornet Strike A = 2.2, 10 Cold stacks,
Galvanized Shot 3 stacks, n = 4: C = 0.45 × 10 + 1.2 × 4 = **9.3**; adding Y = **1250.0**; multiplying (Cyanex-type)
**3296.0**.

**TV-04-18: auras, shards, melee projectiles.**
- Coaction Drift: own Steel Charge on oneself 0.60 × 1.15² = **0.7935**; on an ally without Coaction Drift **0.69**;
  Corrosive Projection worn with Coaction Drift **0.207**.
- Violet Electricity (E-4.12): two Tauforged Violet + three Crimson **2.40**; one Tauforged Violet alone **0.60**;
  two Tauforged Violet **1.50**; five Tauforged Violet **6.00** (all four are `W:Violet_Archon_Shard` §Tips examples);
  five normal Violet **4.00** (derived from E-4.12).
- Two-bucket melee projectile (Exodia Contagion, multiplying, C = 0): zaw base 200 × 2, A_m = 1.2 (Pressure Point) +
  2.75 (Vex Armor), A_w = 1.5 (Arachne) + 2.75 (Vex Armor): Y = `400 × 4.95 × 5.25` = **10395.0** (`F:addmul` §12).

**TV-04-19: Xata's Whisper on a Blast detonation (E-4.13a; `W:Xata's_Whisper` line 48).** Gun 100 base, Thermite
Rounds 0.6, Rime Rounds 0.6, Stormbringer 0.9, Primed Bane F = 0.55, p_xh = 0.26. Hit `100 × 3.1 × 1.55` = **480.5**;
Xata on the hit `0.26 × 480.5 × 1.55` = **193.6415**; detonation H_det = `0.3 × 100 × 1.55²` = **72.075**; Xata on the
detonation `0.26 × 72.075 × 1.55 × 3.1` = **90.043298** (the superseded Blast-only factor 1 + 1.2 gave 63.901695).

**TV-04-20: Peacemaker (§3.2; `W:Peacemaker` example).** Rank 3, Intensify (Str 1.3), Hornet Strike A = 2.2,
Convulsion E = 0.9, 8th burst: `50 × (1 + 1.5 × 8/20) × (1 + 1.5 × 1.3 + 2.2) × 1.9` = **782.8**; burst ≥ 20:
**1223.125**. Ramp factor of the first burst **1.075**; shot index i = 2 (0-based, third shot) is burst 2: **1.15**.

**TV-04-21: Secondary Enervate (E-4.21).** Rank 5, hitscan, one pellet per shot, no Vigilante, no extra hits,
x = modded CC before Enervate. Long-run mean CC per hit `E[S]/E[L]` (engine), mean units per reset cycle `E[L]`, and
the wiki AFCC (`E[S/L]`):

| x | engine `E[S]/E[L]` | `E[L]` | `E[S/L]` | wiki AFCC |
|---|---|---|---|---|
| 0.5 | **1.279708** | 16.491691 | 1.274585 | 1.2745 |
| 1.0 | **1.531938** | 11.491691 | 1.524585 | 1.5245 |
| 1.5 | **1.831333** | 7.5 | 1.825000 | 1.8250 |
| 2.0 | **2.250000** | 6.0 | 2.250000 | 2.2500 |

**TV-04-22: Melee Doughty (§5.4; `W:Melee_Doughty` example).** Destreza Prime (IMP 20.4, PUN 119, SLA 30.6), Auger
Strike +1.2 Puncture, Drifting Contact +0.4 SC, Weeping Wounds at 12× (+4.4 SC): share `261.8/312.8` = 0.836957,
SC = `0.2 × 5.8` = 1.16; `10 × 0.836957 × 1.16 × 1.0` = 9.708696 → **+9.7** (CD_abs).

**TV-04-23: Arcane Bellicose (§5.5).** Max health 2480 → **+0.60**; 2479 → **+0.59** (round(59.496) = 59); 2980 →
**+0.72**; 3500 → **+0.72** (cap).

**TV-04-24: Secondary Surge (§3.2).** 300 energy left after each cast, rank 5: one cast **×2.5**; three casts **×5.5**;
five casts **×8.0** (capped at +7.00). Rank 0, three casts: **×3.0** (cap +2.00). The factor applies to every pellet
of the next shot and is then consumed.

**TV-04-25: Double Tap (§3.2).** Multishot 3, every pellet hits: shot 1 (H = 3) **×1.4**, shot 2 (H = 6) **×2.0**,
shot 3 (H = 9) **×2.6** (the page's +40 %, +100 %, +160 %); H ≥ 21 and `all_max`: **×5.0**.

**TV-04-26: Melee Afflictions (E-4.20).** Single-source melee weapon, B' = 100, A = 1.65, F = 0.30, A_sd = 0,
non-crit body procs; SLA and TOX present; E_TOX = 0.9 on the melee weapon; rank 5. MBD per proc **344.5**;
H_aff = `(344.5 + 344.5) × 1.3` = **895.7**; new Slash tick `895.7 × 0.35 × 1.3` = **407.5435**; new Toxin tick
`895.7 × 0.5 × 1.3 × 1.9` = **1106.1895**; 6 new stacks of each.

**TV-04-27: riven input conversion (§8.2).** Card ×1.535 Grineer → v = **0.535**; ×0.85 Corpus → **−0.15**; +150.3 %
Critical Chance → **1.503**; +2.7 m Punch Through → **2.7**; +21 Initial Combo → **21**; +33.1 % Ammo Efficiency →
formula value **−0.331**, ammo-efficiency bucket **+0.331**. E-4.18 in f32 for the riven side: a spliced Corrosive
(b = 0.01) on a 2+1 rifle riven, rank 8, kept u = 0.7 → v = **1.3899601** (card +139.0 %); a negative faction trait
(b = 0.005, L = −0.495), rank 8, u = 0.5 → v = **−0.26730001** (card ×0.73).

**TV-04-28: Melee Duplicate EV (E-4.15).** Instances per parent `1 + p_dup·P(k = 1)`, p_vig = 0: rank 5, CC 0.6 →
**1.6**; rank 5, CC 1.4 (P(k = 1) = 0.6) → **1.6**; rank 5, CC 2.5 → **1.0**; rank 0, CC 0.6 → **1.15**.

**TV-04-29: `fixed` CO base with Strength (row 184).** Noctua primary fire, Str 2.0 (B' = 250 × 2 = 500), A = 0,
C = 2.4: adding with B_co^abs = 200: **980.0** (a Strength-scaled base `0.8 × 500` would give 1460.0).

**TV-04-30: `co_omit = {combo}` (row 151).** Argo & Vel "Heavy Attack Glaive": B' = 200, ρ_co = 1/3, A = 1.2,
A_w = 0, M(c) = 12, C = 1.6: `200 × 2.2 × 12 + (200/3) × 1.6` = **5386.666667** (with M(c) in G instead: 6560.0).

**TV-04-31: child projectile default D5a (row 202).** Kuva Bramma "Cluster Bomb Contact": B_ref = 49, ρ_co = 187/49
= 3.816327, A = 1.65, C = 1.6: `49 × 2.65 + 187 × 1.6` = **429.05** (D6 would give 208.25).

**TV-04-32: `co_dot_mode` (E-4.7).** Tysis normal attack: B' = 49, A = 2.2, C = 1.6: hit Y = **235.2**; status-DoT
seed (exclude, F = 0, G = 1) = **156.8**. Pox "Poison Cloud" tick (row 130, `per_tick`): B' = 20, B_co = 2.5 × 20 =
50, A = 2.2, C at the tick 1.6: Y = **144.0**.

**TV-04-33: adding CO on a `two_bucket` attack (E-4.5).** B' = B_co = 200, A_m = 1.2, A_w = 1.5, G_F = 1, C = 1.6:
`(200 × 2.2 + 200 × 1.6) × 2.5` = **1900.0**.

**TV-04-34: multiplying row with ρ_co = 0.2 (OQ-428).** Tenet Agendus "Energy Disk": B' = 2880, A = 1.2, C = 1.6.
E-11 as written (current): **16473.6**; proposed `(1 + ρ_co·C)`: **8363.52**.

**TV-04-35: κ_ms (OQ-428).** Tenet Detron normal attack, 2 pellets, B' = B_ref = 26, A = 1.2, C = 1.6. E-11 as
written (current): 98.8 per pellet, shot **197.6**; proposed κ = 0.65 on pellet 1: 98.8 + 84.24 = **183.04**.

**TV-04-36: Kunai Incarnon, multiplying with flat and Swift Conclusion (rows 123, §2.1).** B_ref = 40, X = 70
(B' = 110), Hornet Strike A = 2.2, Galvanized Shot 3 stacks with n = 3 (3.6), Swift Conclusion 2.00: target below
50 % health `110 × 3.2 × 6.6` = **2323.2**; above 50 % `110 × 3.2 × 4.6` = **1619.2**.

**TV-04-37: one scenario under the three T-02 modes.** Braton Prime normal attack (B' = 35), Serration A = 1.65,
Galvanized Aptitude (2 stacks, class (c)), T-05 `manual(2)`, Arcane Rage (A +1.80, class (b) on headshot), Roar 0.30
(T-13, class (u)), T-03 `body`; output `Y·(1 + F)`:
- `all_max`: C = 1.6, Arcane Rage on (triggers are not gates): `(35 × 4.45 + 35 × 1.6) × 1.3` = **275.275**
- `none`: C = 0, Arcane Rage off, Roar on: `35 × 2.65 × 1.3` = **120.575**
- `simulated` (EV): Galvanized at default uptime 1 and 2 stacks, Arcane Rage rate 0 (T-03 = body), Roar on:
  `(92.75 + 56) × 1.3` = **193.375**

**TV-04-38: Devouring Attrition EV (§3.2).** CC = 0.3, CDM = 2.0, k ≤ 1: `E[c·DA] = 0.7 × (1 + 0.5 × 20) + 0.3 × 2.0`
= **8.3** (without the perk 1.3).

**TV-04-39: Parasitic Link (E-4.10, §3.1).** Own Strength 1.3, Nidus Strength 1.0: Str_new = `1.3 × 1.25` =
**1.625**; damage factor in G_F **1.25**.

**TV-04-40: extra hit on a merged beam tick (§6.2, CD-10).** Two beams merged per tick, 100 weapon-side each (H_w =
200), Toxic Lash 0.30, F = 0: one extra hit of **60.0** and **1** forced Toxin proc per tick (not two hits of 30 and
two procs).

**TV-04-41: partial uptime in a nonlinear use (§4.3).** Mecha Empowered v = 1.50, uptime u = 0.30: direct factor
**1.45**; DoT seed × tick factor (mixture) `1 + 0.3 × (2.5² − 1)` = **2.575** (the forbidden `(1.45)²` = 2.1025).

**TV-04-42: finisher instance (E-4.17).** Illustrative M_class = 4 (not a data value): Slash-only base 100, A = 1.65,
Heat mod +0.9 (halved 0.45 → q'_HEA = 14/32 = **0.4375**, q'_SLA = 1), Finishing Touch + Covert Lethality 1.60,
stealth rank 30 (SDB = 7), k = 0, F = 0, G = 1, M_valence = 1: `(1 + 0.4375) × 265 × 2.6 × 4 × 8` = **31694.0** (FIN).

**TV-04-43: Spring-Loaded Broadhead gate (§3.2).** T-27 distance 20 m: G factor **1.40**; 10 m: **1.00** in every T-02
mode (a gate is never assumed).

---

## Review notes

Two verifiers reviewed the draft (2026-11-23); every issue was checked against the cached sources. Issues applied
are reflected above; the notes below record the issues, or parts of issues, that were not applied as proposed.

- Glaxion Vandal (row 45) `direct_target_only`: rejected; CO Mechanic §Mechanics lists "Beam AoE (e.g., Glaxion Vandal)" among forms that apply CO with no direct-target clause and row 45 has no such note (D4a); the `mproj_mode` part of that issue was applied (§3.4).
- Secondary Enervate "projectiles count once per pellet": rejected; the page's "count as multiple hits" concerns several enemies, while for one target projectiles of one shot register once (grace period), so `hit_unit = shot` (the bucket, reset and EV parts were applied, E-4.21).
- Firing Line "5x Damage Vulnerability": corrected to the page's value, "1.5x Damage Vulnerability from all sources while Lifted" (`W:Firing_Line` line 14).
- Timing-stat effects under `simulated`: the fixed-point option was not adopted; `06` §5.9 already fixes constant user uptime for random triggers, and 04 follows it to stay consistent (alternative in OQ-432).
- Deadly Maneuvers "correct overview 6.3": applied in this document only; only the manager edits `00` (OQ-438), and the page's own example (×6) disagrees with its rank table (×5).
- Melee Doughty "Puncture share after quantization": not applied; the page's example uses unquantized modded damage (`119 × 2.2 / 312.8`) and `02` E-2.5a does the same (quantized share stays an OQ-209 alternative).
- Extra hits on radial instances: the engine decision stays overview E-14 (direct instances) until the manager amends it; the game evidence is recorded as a proposal (OQ-412, D-412), as the issue's second option allows.
- κ_ms and ρ_co in `multiplying`: E-4.3 now states overview E-11 verbatim and keeps both as proposals (OQ-428), so rows 114 and 165-166 evaluate with E-11 until accepted.
- Melee Duplicate's Toxic Lash / Xata's Whisper bug path has no separate numeric vector: it duplicates an E-4.13 extra hit, whose arithmetic TV-04-11 covers.
