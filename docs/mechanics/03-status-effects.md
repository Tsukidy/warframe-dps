Status: reviewed (spec)

# Damage-calculation spec, part 03: status effects

This document defines what every status effect does once a proc exists: its duration, timers, stack caps
(including enemy-specific caps), replacement, the debuff functions that other stages read, the DoT tick formula E-13
in full, Heat's shared timer and inherit rule, Blast's fuse and detonation, Gas, Electricity, the Magnetic break proc,
DoT detonation (Harmony), status duration (including negative values), the ragdoll target state, and the
distinct-status count `n`. It owns the effect half of stage
ST-10 (`00-overview.md` §3.1) and the values that ST-7 (Cold, Puncture) and ST-8 (Viral, Magnetic, armor strips) read.

Related documents: [00-overview.md](00-overview.md) (notation, E-13, CD-04 to CD-07, register),
[01-damage-composition.md](01-damage-composition.md) (Y, faction bucket F),
[02-crit-status-multishot.md](02-crit-status-multishot.md) (proc rolls, crit tier, body part, where Cold and Puncture
enter E-4/E-5), [04-conditional-bonuses-and-buffs.md](04-conditional-bonuses-and-buffs.md) (catalogue of status
duration, status damage and elemental sources; CO in DoT seeds E-4.7; extra hits E-14),
[05-enemies.md](05-enemies.md) (layer routing of ticks, armor E-15, immunities and caps as data),
[06-ranged-firing-model.md](06-ranged-firing-model.md) and [07-melee.md](07-melee.md) (schedules, combo),
[08-evaluation-and-outputs.md](08-evaluation-and-outputs.md) (EV expectation of everything defined here).

Contents:

- [1. Scope](#1-scope)
- [2. Inputs](#2-inputs)
- [3. Outputs and symbols added](#3-outputs-and-symbols-added)
- [4. Status catalogue](#4-status-catalogue)
- [5. Status duration](#5-status-duration)
- [6. Stacks, caps and replacement](#6-stacks-caps-and-replacement)
- [7. Timing and event order](#7-timing-and-event-order)
- [8. Debuff functions](#8-debuff-functions)
- [9. DoT ticks (E-13 in full)](#9-dot-ticks-e-13-in-full)
- [10. Heat](#10-heat)
- [11. Blast](#11-blast)
- [12. Gas and Electricity](#12-gas-and-electricity)
- [13. Magnetic break proc](#13-magnetic-break-proc)
- [14. Distinct-status count n](#14-distinct-status-count-n)
- [15. Worked example](#15-worked-example)
- [16. Edge cases](#16-edge-cases)
- [17. Inspiration differences](#17-inspiration-differences)
- [18. Open questions](#18-open-questions)
- [19. Test vectors](#19-test-vectors)
- [20. Review notes](#20-review-notes)

Citation keys are those of `00-overview.md` §0.2. Wiki line numbers refer to the raw copies in
`research/external/cache/wiki/` (file name = page name, `/` → `__`). Pages added to the cache for this document:
`Knockdown.wiki`, `Lifted.wiki`, `Ragdoll.wiki`, `Stun.wiki` (fetched 2026-11-23).

---

## 1. Scope

Copied from `00-overview.md` §7.1: every status type: duration, timers, caps (including enemy-specific caps CD-07 as
rules), replacement; debuff functions (Viral, Magnetic, Corrosive strip, Heat strip ramp, Cold crit damage and Frozen,
Puncture crit chance, Tau, Radiation, Impact, Void, Microwave, Knockdown, Lifted); DoT tick formula E-13 (seed, "+1",
coefficients, elemental scaling, faction twice, status damage, crit, body part, inheritance list); Heat shared timer and
inherit; Blast fuse and detonation; Gas; Electricity; Magnetic break proc; status duration (including per-type SD_s)
and negative SD; distinct-status counting (what counts, CD-04 timing); ragdoll target state RAG (duration,
Impact-discard state; fed by `07`'s crowd-control proc events, used by `02`'s Impact rule). Overview §7.2 also makes
this document the owner of the stack state of external statuses (T-06, T-43; §6.3).

Explicitly not here: layer routing of ticks (Toxin bypass, overguard, shield gate, armor floor; `05`), the expectation
of `n` and of stack counts in the EV evaluator (`08`), proc counts and proc-type weights (`02`), where Cold and
Puncture enter the crit formulas (`02`), the catalogue of which mods, arcanes and abilities feed SD, A_sd and E^dot
(`04`), the CO term inside the seed (`04`, E-4.7), extra-hit DoTs (`04`, E-14).

Stages: ST-10 (effects), and the status inputs of ST-7 and ST-8. Equations owned: E-13 (restated in full as E-3.4 to
E-3.6) and E-3.1 to E-3.14.

The ragdoll target state RAG (§8.10) is this document's per overview §7.1 and §7.2 (settled from OQ-326 and `07`
OQ-734); `02` E-2.13 and E-2.13a read it.

---

## 2. Inputs

| Symbol | Owner | Used in |
|---|---|---|
| `base[t]`, `base_total` (B'), `base_ref` (B_ref), Y (E-11) | `01`, `04` | seed (E-3.4) |
| `sum_faction` (F) | `01` | seed, tick |
| `mult_other` (G), `mult_final_omitted` (G_F), `co_dot_mode` | `04` | seed (E-4.7) |
| crit tier k, `cd` (CDM), `headcrit` (h), stealth term SDB, `part` (Part, which contains `sonar_mult` M_son, `02` E-2.10) | `02`, `04` | c_k, Part_dot |
| proc list of each damage instance (types in roll order, forced procs, Hunter Munitions and Internal Bleeding Slash) | `02` | §6, §7 |
| merged beam count b of a beam tick (CD-10) | `02` | seed |
| SD sources (`sum_status_dur`, per-type `status_dur_type[s]`), status-damage sources (`sum_status_dmg`, `status_dmg_type[s]`), elemental sources with their `dot_scaling` flag, status-damage vulnerabilities (Burning Hate, Pageflight, custom `vuln_status_damage`), tick-time conditions (Mecha Empowered mark, Vauban incapacitation), `M_OG` (Secondary Fortifier) | `04` (catalogue and conditions), this document (buckets) | E-3.1, E-3.4, §9.4 |
| enemy status immunities, per-enemy status cap and per-type overrides, flags `vip`, `no_freeze`, `cc_immune` (crowd-control and ragdoll immunity), scaled pools OG and SH | `05` | §6, §8.10, §13 |
| shot and hit schedule, instant-burst grouping, melee combo multiplier M(c) at each hit (before the hit's own consumption), the `dot_detonate` trait of an attack (Harmony heavy attacks) | `06`, `07` | §7, §8.9, §9.7 |
| proc-name normalisation of weapon and stance data, the special rule `microwave` | `02` §4.7 | §4.2, §8.7 |
| T-05 (count mode), T-06 (external Viral), T-43 (external statuses), T-26 (`heatInherit`, `heatInheritFaction`), compat profile flags | `00` §6 | §6.3, §10.3, §14 |
| layer view or pass being evaluated (for `og_present`) | `00` §4.1, `08` | §6.1, §8.10 |

---

## 3. Outputs and symbols added

Outputs (overview symbols, defined here): `dur_base[s]`, `dur[s]`, `dot_ticks[s]`, `dot_delay[s]`, `dot_coeff[s]`,
`dot_elem[s]`, `sum_status_dmg`, `status_dmg_type[s]`, `sum_status_dur`, `stack_cap[s]`, `stacks[s]`,
`viral_mult(n)`, `mag_mult(n)`, `corr_strip(n)`, `heat_strip(k_h)` (on the step counter k_h, as overview §2.4 now
writes it), `cold_cd(n)`, `punc_cc(n)`, `n_distinct`; the DoT tick and Blast
damage instances (weapon side; the enemy side is E-12 with the tick's type, `05`); the Magnetic break instances; the
DoT detonation instances (§9.7).

**Symbols added**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `dot_seed` | S_p | HP | seed of proc p (the `seed` of E-13, crit factor c_k and Part_dot included), snapshotted at proc creation |
| `dot_mult` | X_p | factor | per-proc tick multiplier (E-3.4); holds no crit or body-part factor |
| `dot_part` | Part_dot | factor | body-part factor inherited by procs, `Part` (Sonar and Detect Vulnerability spots included, D-095) |
| `dot_extra[s]` | M_dot,s | factor | multipliers applied to status instances of s only: `V_sd,p · M_tick(τ)` (§9.4) |
| `dot_vuln_sd` | V_sd,p | factor | status-damage vulnerability of the target when proc p is created, snapshotted (§9.4) |
| `dot_tick_mult` | M_tick(τ) | factor | target-condition second dips read when a status instance lands (Mecha Empowered, Vauban passive; §9.4) |
| `dot_og_mult` | M_OG,p | factor | the source attack's `M_OG`, applied by `05` when a status instance of p lands on overguard (§9.4) |
| `dot_tick_time` | τ_m | s | landing time of tick m of an instance, `t_p + (δ_s + m)` (E-3.6) |
| `dot_remaining` | N_rem | count | ticks or pulses of an instance not yet landed at a DoT detonation (§9.7) |
| `status_dur_type[s]` | SD_s | bonus | per-type status duration (Ash passive: SLA +0.50); overview §2.4 symbol (adopted from OQ-323) |
| `status_sd_eff` | SD_p | bonus | effective status duration of proc p of s: `SD + SD_s` of its source (E-3.1) |
| `cap_base[s]` | – | count | per-type base cap of table 4.1 |
| `heat_pool_cap` | cap_HEA | count | maximum procs in the Heat pool: ecap_HEA (4 on status-capped enemies), else ∞ (§10.2) |
| `heat_sd_star` | SD* | bonus | SD_p of the pool's proc with the largest D_HEA, evaluated before the pool is cleared (E-3.8) |
| `dot_faction_tick` | F_tick,p | bonus | faction bonus of the tick-level factor of p (F_i, or the Heat episode's first source, §10.3) |
| `tau_sc(n)` | – | factor | `1 + 0.10·n`, Tau's status-chance factor in M_sc (E-3.3) |
| `mag_break_time` | t_b | s | time of the damage instance that breaks the layer (§13) |
| `blast_hit` | H_det | HP | weapon-side value of one Blast host instance `max(1, 0.3·H_p)` (E-3.9; read by `04` E-4.13a) |
| `cc_ragdoll` | RAG | bool | ragdoll target state (§8.10); not a StatusType |
| `cc_ragdoll_dur` | D_RAG | s | duration of the ragdoll state (§8.10) |
| `dot_beam` | b | count | merged beams of the inducing beam tick (CD-10), 1 otherwise |
| `dot_acc` | a | HP | DoT accumulator start value: 1 (game profile, D-025), 0 (compat) |
| `dot_group` | 𝒢 | set | procs whose ticks are consolidated into one tick (§9.5) |
| `dur_eps` | ε | – | 1e-9, rounding guard for the tick-count floor N_s (§5); not a time tolerance (times use the grid of §7) |
| `mc_time` | τ̂ | µs (integer) | event time on the integer microsecond grid (overview CD-14, `08` E-8.0a); every time test of this document compares grid values (§7) |
| `cap_enemy[s]` | ecap_s | count | enemy cap of s from `05` data (per-type override, else per-enemy global cap, else ∞) |
| `cap_bonus_cor` | β_COR | count | Emerald Archon Shard bonus to the Corrosive cap (Σ over shards: 2, Tauforged 3) |
| `og_present` | – | bool | whether the overguard-dependent caps and the overguard ragdoll immunity apply; per layer view or pass (§6.1, overview §4.1) |
| `ext_stacks[s]` | N_ext,s | count | external stacks of s held for the whole window (T-43; T-06 for VIR) (§6.3) |
| `cold_frozen` | – | bool | target is Frozen (§8.6) |
| `dur_frozen` | D_frz | s | `3·(1 + SD)` of the freezing proc |
| `heat_ign`, `heat_exp` | t_ign, t_exp | s | ignition time and expiry time of the current Heat episode |
| `heat_first` | p₁ | proc | first proc of the Heat episode (inherit source) |
| `heat_step` | k_h | 0..4 | Heat armor-strip step counter |
| `heat_up_int`, `heat_down_int` | h_up, h_dn | s | Heat strip step intervals `0.5·(1 + SD*)`, `1.5·(1 + SD*)` |
| `heat_inh_faction` | F_inh | bonus | faction bonus of an external first Heat source (T-26 custom stat `heatInheritFaction`, overview §6.3; default 0; adopted from OQ-315) |
| `blast_h` | H_p | HP | Blast stack value ("damage H of the hit that applied it") |
| `blast_fuse` | D_BLA | s | `1.5·(1 + SD)` |
| `corr_first` | c₁ | fraction | strip of the first Corrosive stack: 0.26 (the Hydroid variant 0.50 is `05` E-5.12) |
| `mag_break_tick` | T_brk | HP | weapon-side tick of the Magnetic break proc |
| `lif_dur`, `knd_dur` | D_LIF, D_KND | s | Lifted and Knockdown durations |

---

## 4. Status catalogue

### 4.1 Summary table (normative; the engine transcribes this)

SD = the effective status duration SD_p = `sum_status_dur` + `status_dur_type[s]` of the proc's source (§5).
"Indep." = every stack has its own timer. N_s uses ε (§5).

| s | Proc | D0 (s) | SD scales | Base cap | Stacking and replacement | First tick δ (s) | Interval | Ticks N | K_s | E^dot_s | Tick type | Debuff function (consumer) | Counts in n |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| IMP | Knockback | 6 | yes | 5 | indep.; at cap the oldest is replaced | – | – | – | – | – | – | none on damage (the *proc* triggers Internal Bleeding, Hemorrhage, Magnetic Welt in `02`) | yes |
| PUN | Weakened | 10 | yes | 5 | indep.; oldest replaced even if it has more time left | – | – | – | – | – | – | `punc_cc(n) = 0.05·n` into ΣCC_abs of non-AoE weapon instances (`02`) | yes |
| SLA | Bleed | 6 | yes | ∞ | indep. DoT instances | 1 | 1 | `floor(6(1+SD)+ε)` | 0.35 | none | Cinematic: no type modifier, no armor (`05`) | – | yes |
| HEA | Ignite | 6 | yes | ∞ (procs in the pool) | one episode per target; every proc joins the pool and refreshes it (§10) | 1 after ignition | 1 | episode-driven (E-3.7); one proc alone: `floor(6(1+SD)+ε)` | 0.5 | Heat bonuses of the episode's first source (inherit) | Heat | `heat_strip`: armor strip ramp (E-15, `05`) | yes |
| COL | Freeze | 6; Frozen 3 | yes (both) | 10; the 10th freezes | indep.; at cap the oldest is replaced; 10th stack → Frozen (§8.6) | – | – | – | – | – | – | `cold_cd(n)` into ΣCD_abs of non-AoE weapon instances (`02`) | yes |
| ELE | Tesla Chain | 6 | yes | ∞ | indep.; same-pulse ticks consolidated (§9.5) | 0 | 1 | `floor(6(1+SD)+ε)` | 0.5 | every Electricity bonus, combined or not | Electricity | – (stun is crowd control only) | yes |
| TOX | Poison | 6 | yes | ∞ | indep. | 1 | 1 | `floor(6(1+SD)+ε)` | 0.5 | every Toxin bonus, combined or not | Toxin (bypasses SH, not OG; `05`) | – | yes |
| BLA | Detonate | 1.5 (fuse) | yes | 10 = detonation threshold | indep. fuses; reaching the cap detonates all (§11) | fuse | – | 1 hit per stack | 0.3 | none (OQ-007) | Blast; no "+1" | – | yes |
| RAD | Confusion | 12 | yes | 10 | indep.; oldest replaced | – | – | – | – | – | – | none on the player's damage | yes |
| GAS | Gas Cloud | 6 | yes | 10 | indep.; oldest replaced, its remaining ticks cancelled; same-pulse ticks consolidated | 0 | 1 | `floor(6(1+SD)+ε)` | 0.5 | literal Gas bonuses only | Gas | – | yes |
| MAG | Disrupt | 6 | yes | 10 | indep.; oldest replaced | – | – | – (break proc §13) | – | – | – | `mag_mult(n)` on SH and OG damage (`05`) | yes |
| VIR | Virus | 6 | yes | 10 | indep.; oldest replaced | – | – | – | – | – | – | `viral_mult(n)` on HP damage incl. armored HP (`05`) | yes |
| COR | Corrosion | 8 | yes | 10 + β_COR | indep.; oldest replaced | – | – | – | – | – | – | `corr_strip(n)` armor strip (E-15, `05`) | yes |
| VOI | Bullet Attractor | 3 | yes (UNVERIFIED) | 1 (UNVERIFIED) | single; a new proc restarts the timer | – | – | – | – | – | – | none on damage | yes |
| TAU | Status Vulnerability | 8 | yes | 10 | indep.; oldest replaced | – | – | – | – | – | – | factor `1 + 0.10·n` in M_sc of every status roll on the target (`02`, E-8); not forced procs | yes |
| MIC | Microwave | ∞ | – | 1 | permanent once applied | – | – | – | – | – | – | none modelled (enlarges a body part) | yes |
| KND | Knockdown | 3 (UNVERIFIED) | no | 1 | single; re-proc restarts; removes LIF | – | – | – | – | – | – | none on damage | yes |
| LIF | Lifted | M(c) at the hit, before consumption (§8.9) | no | 1 | single; a LIF proc on a Lifted target ends Lifted and ragdolls (§8.10); removes KND | – | – | – | – | – | – | none on damage | yes |

Sources per row (`F:status` §1 and §4 restate each page):
IMP `W:Damage/Impact_Damage` §General (6 s per stack, 5 stacks since U33.6; D-033); PUN `W:Damage/Puncture_Damage`
§General ("replace the oldest stack, even if the oldest stack has a longer remaining duration"; 10 s; D-032); SLA
`W:Damage/Slash_Damage` §General, `W:Damage_over_Time` table; HEA `W:Damage/Heat_Damage` lines 15-120 (edited
2026-11-14); COL `W:Damage/Cold_Damage` lines 17-25 (U36); ELE `W:Damage/Electricity_Damage` §General; TOX
`W:Damage/Toxin_Damage`; BLA `W:Damage/Blast_Damage` lines 19-66 (U36 rework, edited 2026-11-03), `W:Damage/Calculation`
lines 641-645; RAD `W:Damage/Radiation_Damage` §General; GAS `W:Damage/Gas_Damage` lines 18-60; MAG
`W:Damage/Magnetic_Damage` lines 20-40 (U36); VIR `W:Damage/Viral_Damage` lines 18-36; COR `W:Damage/Corrosive_Damage`
§General (U36), `W:Emerald_Archon_Shard` lines 40-43; VOI `W:Damage/Void_Damage` §Status Effects (U29.5; stacking not
documented); TAU `W:Damage/Tau_Damage` §Status Effects (U37); MIC `W:Microwave`, `W:Status_Effect` lines 161-165;
KND `W:Status_Effect` lines 149-153, `W:Knockdown` (no duration given); LIF `W:Lifted` line 6 ("1 second at base,
increasing by 1 second for every Melee Combo tier, up to 12 seconds at 12x"), `W:Melee` line 799 ("The higher the combo
multiplier at the time of a heavy attack or heavy slam attack, the longer"; status duration does not extend it).
Tick tables: `W:Status_Effect` lines 422-448, `W:Damage_over_Time` table. Coefficients: E-13, `W:Damage/Calculation`
line 623.

### 4.2 Proc names in game data

Forced-proc names appear in `Module:Weapons/data` (`ForcedProcs`) and `Module:Stances/data` (`Procs`) (counts from
the cached modules, 2026-11-23). The engine maps them as follows; names mapped to "CC only" create no status, are
never counted in n and have no damage effect.

| Data name | StatusType | Note |
|---|---|---|
| Impact, Knockback | IMP | `PT_KNOCKBACK` (`W:Status_Effect` line 30) |
| Puncture, Weakened | PUN | |
| Slash, Bleed | SLA | |
| Heat, Cold, Electricity, Toxin, Blast, Corrosive, Gas, Magnetic, Radiation, Viral | same name | |
| Tesla Chain | ELE | |
| Detonate | BLA | |
| Lifted | LIF | |
| Knockdown | KND | the unquoted Lua identifier in Blind Justice evaluates to nil (`F:melee` §2.2): no proc |
| Ragdoll | CC only | not a StatusType; enters the ragdoll target state RAG (§8.10) for `02`'s Impact rule |
| Stagger, Big Stagger | CC only | `PT_BIG_STAGGER` is separate from Impact (`W:Status_Effect` lines 128-132). Counter-evidence: `W:Damage/Impact_Damage` line 22 names Impact's own status "Stagger", and the status-duration table row "Stagger, 6 seconds" (`W:Status_Effect` line 303) is the Impact row. "Stagger" is still mapped to CC only because both data modules name Impact procs otherwise: weapon `ForcedProcs` use "Impact" about 350 times and "Stagger" 7 times, one attack (a spear-throw explosion, `Weapons_data_primary.lua` line 5633) lists `{ "Impact", "Stagger" }` side by side; stance `Procs` use "Knockback" (239, counting nested proc groups) for Impact and "Stagger" (48) separately. UNVERIFIED (OQ-310) |
| Stun | CC only | conflict on whether it counts for CO (OQ-310) |
| Blind, Impair, Disarmed | CC only | Impair is PvP only |
| Finisher | CC only | Caustacyst heavy attack, ranged part (`Stances_data.lua` lines 1666, 1673: `{ "Knockback", "Finisher" }`); opens the target to a finisher, not a damage-type status and not in the CO list (`W:Condition_Overload_(Mechanic)` lines 162-185); `07` §4.1, `07` OQ-715 |
| Microwave | MIC | no weapon in the data lists it; Nukor and Kuva Nukor get it from `02`'s hand-table rule `microwave` (§8.7) |

Names are looked up after `02`'s normalisation (`02` §4.7: spaces removed, so "Tesla Chain" → TeslaChain, "Big
Stagger" → BigStagger; empty strings in stance `Procs` are CC only); an unknown name after normalisation is a
data-build error (`02`). This table and `02`'s mapping MUST agree; `02` owns the normalisation.

---

## 5. Status duration

The status-duration bucket `sum_status_dur` (SD) is additive over its sources: Lasting Sting family (Lasting Sting,
Continuous Misery, Lingering Torment, Perpetual Agony), Hunter Track, Augur Seeker, riven Status Duration (positive or
negative), Saryn's and Lavos's passives, Harmony's Harmonic Resonance, Prismatic Gem (values in `04`)
(`W:Status_Effect` lines 692-706, `W:Damage_over_Time` §Increasing DoT Duration, `F:mods` §7, `F:buffs` §7). A proc
snapshots the SD of its source attack at creation (CD-05); a later change of SD does not change live stacks.

- **Per-type status duration** `status_dur_type[s]` (SD_s) holds bonuses that lengthen one status only. Ash's passive:
  "Slash procs inflicted by Ash's weapons and abilities deal 25% more damage and last 50% longer ... Bleed duration
  bonus stacks additively with status duration mods" (`W:Ash/Abilities` lines 3-6): SD_SLA += 0.50 and A_sd,SLA += 0.25
  for every proc of the build when Ash is the frame (`04` catalogue). SD_s is additive with SD, so a proc of s uses
  SD_p = SD + SD_s everywhere this document writes SD for a proc. Overview §2.4 (`status_dur_type[s]`), §3.1a and
  CD-06 now carry it (adopted from OQ-323).
- **Saryn's passive** ("Status Effects inflicted by Saryn's weapons and abilities last 25% longer", additive with
  Lasting Sting; `W:Saryn/Abilities/Passive` lines 1-2) is one SD source of +0.25 for every proc of the build when Saryn
  is the frame. T-16 (Toxic Lash) implies that Saryn is the frame (Toxic Lash is not subsumable, `04`), so T-16 turns the
  passive on; with the passive also selected in the `04` catalogue it is still counted once. Toxic Lash procs and the
  weapon's own procs therefore all get SD + 0.25, as overview D-037 and T-16 now state (adopted from OQ-333).

```text
E-3.1  Modded duration and tick count (CD-06); SD below means SD_p = SD + SD_s of the proc's source
  D_s   = D0_s · (1 + SD_p)                     no rounding but the 1 µs grid (§7); every s with "SD scales = yes"
  N_s   = floor(6·(1 + SD_p) + ε)               s ∈ {SLA, TOX, ELE, GAS}; ε = 1e-9
  D_BLA = 1.5·(1 + SD_p)        D_frz = 3·(1 + SD_p)        h_up = 0.5·(1 + SD*)          h_dn = 1.5·(1 + SD*)
  D_s   = D0_s                                  s ∈ {KND, LIF}; MIC: ∞;  D_LIF per §8.9
```

- `floor` for ticks: `W:Lasting_Sting` §Notes ("Due to status duration always rounding down, there is no difference
  in performance between a 9th rank and 10th rank version": +100 % and +110 % both give 12 ticks). Debuff durations
  are not rounded beyond the 1 µs time grid of §7 (Decision of CD-06; UNVERIFIED, OQ-005). The ε guard absorbs f64
  summation error of bonuses: for example +0.60, +0.95 and −0.55 summed left to right give `0.9999999999999998` and
  `6·(1 + SD) = 11.999999999999998`, which `floor` alone turns into 11 ticks instead of 12 (TV-03-13); summed in another
  order (`0.6 + (−0.55) + 0.95`) the same bonuses give 12 without the guard. The engine does not fix a summation
  order for SD sources and relies on ε, which covers any order (the error of a sum of a few bonuses is far below
  1e-9); ε is part of the rule, not a tolerance in tests and not a time tolerance (durations and times use the grid
  of §7).
- SD lengthens the Heat strip ramp **and** the ramp back (`W:Status_Effect` line 325: "Ramp up and ramp down times for
  armor strip lengthened"; `W:Damage/Heat_Damage` §Armor Stripping example +100 % → 1 s steps), Frozen and the Cold
  stacks after it (line 317), the Blast fuse (`W:Damage/Blast_Damage` line 64), the Magnetic break proc (`W:Status_Effect`
  line 343; §13).
- SD does not extend crowd control: Impact stagger, Heat panic, Electricity stun, Lifted (`W:Lasting_Sting` §Notes,
  `W:Lifted`); those have no damage effect. More elemental damage does not lengthen procs (`W:Status_Effect` line 231).
- Whether SD scales Void and Tau is undocumented; both are treated like the other debuffs (UNVERIFIED, OQ-311).
- Weapon-specific base durations: Heat procs created by Nightwatch Napalm ticks have D0 = 3 s (`W:Nightwatch_Napalm`
  §Notes); every other weapon proc uses table 4.1. Bubonico's alternate-fire explosion is listed as "+9 s pre-mods"
  status duration (`W:Status_Effect` line 695) with no data field: not modelled (EC-03-56).

### 5.1 Negative and very low status duration

```text
SD_p ≤ −1 (D_s ≤ 0 for every s whose duration scales with SD): the proc is nullified: no stack, no DoT
                   instance, no debuff, not counted in n, no Heat episode, no Frozen. Exception BLA: D_BLA ≤ 0,
                   the stack's expiry hit lands at once (in P4b of its shot, §7.1) and no stack is created.
                   MIC, KND and LIF do not scale with SD and are unaffected.
−1 < SD_p:         normal rules; a DoT whose N_s = 0 (SD_p < −5/6) creates an instance that exists (and counts in n)
                   for D_s seconds but deals no tick.
```

Source: `W:Status_Effect` §Negative Status Duration (lines 356-420): "all proc effects which have a duration or deal
damage over time [are] nullified ... Blast: effect expiry damage to target"; `W:Damage/Blast_Damage` line 64 ("will
always deal damage even if total Status Duration is below 0%"). The wiki marks the table outdated (2016 forum post).
The table also says Electricity's Tesla Chain "occurs normally" at negative duration; this spec nullifies it like every
other DoT (Decision; UNVERIFIED, OQ-307). Heat panic and other crowd control have no damage effect.

---

## 6. Stacks, caps and replacement

### 6.1 Cap resolution

```text
E-3.2  Cap of status s for a proc of attack a on target e at the moment of application
  cap0 = min(cap_base[s], ecap_s)                  cap_base from table 4.1; ecap_s from 05 data:
                                                   per-type override, else per-enemy global cap, else ∞
  if og_present(e) and s = COL:               cap0 = min(cap0, 4)
  if og_present(e) and vip(e) and s = PUN:    cap0 = min(cap0, 3)
  if s = PUN and a has the Evensong trait:     cap0 = 5 if (ecap_PUN < 5 or og_present(e)) else 7
  cap_s = cap0 + β_COR   if s = COR (procs of the shard holder's own weapons)
  cap_s = cap0           otherwise
og_present(e), per view or pass (overview §4.1):
  layered pass:                       OG pool > 0 at the moment of application (live state)
  `overguard` view (MC and EV):       true when the enemy spawns with OG_max > 0
  `health`, `armor`, `shield` views:  false (they model damage as if the overguard were gone, `05` §18)
```

The per-view rule is overview §4.1 (adopted from OQ-321, `05` OQ-529, `08` OQ-804): the status state is the same in
every view except `og_present`, so on an enemy with overguard the `overguard` view and the other three views run two
status timelines that differ only in the overguard-dependent caps (Cold 4, hence no Frozen; VIP Puncture 3) and in
the overguard ragdoll immunity of E-3.13; the EV evaluator evaluates each view with its own `og_present`. The game rule
is sourced ("Bosses, as well as enemies with active Overguard, can receive a maximum of 4 Cold stacks",
`W:Damage/Cold_Damage` line 27, U36); the split by view is a modelling Decision (UNVERIFIED, OQ-321; TV-03-28).
The overview's review notes name `05` §18 and `08` §4.5 as following this rule.

Sources and enemy data that `05` supplies (CD-07, D-053):

| Rule | Source |
|---|---|
| Status-capped enemies: global cap 4 for every status, DoT instances included. Members with a source: Kuva Liches (U27.3 note on `W:Bosses` line 925, "No Status Effect will exceed a maximum of 4 stacks", stated for Kuva Liches only), Acolytes (`W:Acolytes` line 17), Archon Boreal in Archon Hunts (`W:Archon_Boreal` line 43, "takes up to a maximum of 4 Status Effects of any type"; stated for Boreal only, applied to all three Archons by `05`). Sisters of Parvos and Technocyte Coda: no cached page states a global cap (`W:Damage/Radiation_Damage` line 27 caps only Radiation for Sisters); membership UNVERIFIED (OQ-320) | as listed; application to DoTs UNVERIFIED (OQ-008) |
| Other bosses (`05` flag `boss`, not in a status-capped group): Cold 4 (so no Frozen) and Blast 4 (detonation at the 4th stack) only; every other status, Heat's pool and DoT instances keep their normal caps | `W:Damage/Cold_Damage` lines 27, 364; `W:Damage/Calculation` lines 643-645 ("bosses cap at 4"); `05` §4.4, CD-07 (UNVERIFIED, `05` OQ-531) |
| Acolytes: Impact cap 3 | `W:Acolytes` line 17 |
| Kuva Liches: Impact override 6 (U27.3), so `min(5, 6) = 5` since U33.6 cut Impact to 5 | `W:Bosses` line 925; `W:Damage/Impact_Damage` |
| Radiation cap 4 on Kuva Liches, Acolytes, enemy Necramechs, Sisters of Parvos, Hounds | `W:Damage/Radiation_Damage` line 27 |
| Overguard present: Cold cap 4 (so no Frozen) | `W:Damage/Cold_Damage` line 27 (U36) |
| VIP with overguard: Puncture cap 3 | U36 fix "instead of the intended 3" (`F:status` §4.2) |
| Evensong: 7 Puncture stacks per hit, beyond the normal 5; enemies "usually capped at 4 stacks of a status, such as Acolytes and enemies with Overguard" receive 5 | `W:Evensong` lines 17, 42 |
| Emerald Archon Shard: Corrosive cap +2 (Tauforged +3) per shard, additive over shards, also raises the cap of 4 of status-capped enemies ("Increase is applied to Status Capped enemies"); only Corrosion applied by the holder's weapons and abilities exceeds the base cap | `W:Archon_Shard` lines 351-354; `W:Emerald_Archon_Shard` lines 40-43 (multi-shard total UNVERIFIED, OQ-317) |
| Enemies immune to a status (Demolishers: Viral; Manics: Radiation; Alkonost and the "Full Immunity" list: all) | `05` §4.4 data (`ProcResists`), `W:Damage/Viral_Damage` list, `F:enemy` §9.2 |
| Enemy Necramechs: not immune; Radiation cap 4. `W:Necramech` line 102 ("Necramechs are immune to Knockdown, Stagger, and Status Effects") describes player Necramechs and contradicts the Radiation page's cap for enemy Necramechs | `05` §4.4 Decision (UNVERIFIED, `05` OQ-520); this document no longer lists them as immune |

A cap never removes stacks that already exist when it drops (for example when the overguard returns in the layered
pass); it only governs later insertions (Decision, UNVERIFIED, OQ-331). The views of §6.1 never switch caps within a
timeline.

### 6.2 Insertion and replacement

```text
On application of a proc of s with D_s > 0 (§5.1) to a target not immune to s:
  COL while Frozen:                          discard the proc (§8.6)
  BLA:                                       insert the stack; if n_BLA = cap_BLA now, detonate (§11)
  COL, n_COL = 10 after insertion, cap 10,
     target can freeze (not no_freeze):      start Frozen (§8.6)
  VOI, MIC, KND, LIF:                        single-instance rules of §8.7 to §8.9
  HEA:                                       join or ignite the episode (§10); if |pool| > cap_HEA, the oldest
                                             proc leaves the pool
  otherwise:  if n_s = cap_s before insertion: remove the oldest live stack (smallest application time; ties by
                                             application order); then insert. A removed DoT instance (SLA, TOX,
                                             ELE, GAS) loses all its ticks that have not yet landed.
```

Replacement rule: `W:Damage/Puncture_Damage` §General, `W:Damage/Gas_Damage` line 44 ("Any instances after the 10th
will replace the oldest instance"), the same sentence on the Viral, Magnetic, Corrosive, Radiation and Tau pages; CD-07.
Cancelling the replaced Gas instance's ticks follows from the cloud being replaced (Decision, UNVERIFIED, OQ-331: the
page says the instance is replaced, nothing about its pending ticks; TV-03-22). For SLA, TOX, ELE and HEA,
replacement only occurs on status-capped enemies (CD-07, OQ-008). Replacement (and DoT detonation, §9.7) are the only
events that cancel scheduled ticks; natural expiry never does (§7.3).

Immunity: a proc of a type the enemy is immune to is discarded at application, including forced procs (the roll
weights are renormalised in `02`; `W:Status_Effect` §Status Immunity Interactions). Immunity to "all" discards every
proc and disables every status-damage term (`F:status` §5; inspiration `Nb`, `JS L10520`). External stacks (T-43, T-06) of a
type the enemy is immune to are ignored.

### 6.3 Stack count n_s

`stacks[s]` (n_s) is the number of live stacks (debuffs), live DoT instances (SLA, TOX, ELE, GAS), procs in the Heat
pool (HEA) or live fuses (BLA), plus the external stacks of overview T-43 (T-06 is T-43 restricted to Viral).

**External statuses (T-43, T-06).** For each s with T-43 set to N_ext,s (1 .. cap_s; COL 1 .. 9), the target holds
N_ext,s stacks of s for the whole window, from t = 0: they never expire, are never replaced, deal no damage and fire
none of the build's on-proc triggers (§6.4), and they count in n (E-3.12) and in every stack-dependent function of §8
(Vir, Mag, c_corr, cold_cd, punc_cc, tau_sc) and in n_COL. Own stacks use the free slots: `n_s = N_ext,s + n_s,own`
with `n_s,own ≤ cap_s − N_ext,s`, and the insertion rule of §6.2 replaces the oldest **own** stack when the own part
is full. The cap is resolved per E-3.2 at each insertion; if it falls below N_ext,s, the external stacks stay and no
own stack is inserted. Rules for the statuses whose stacks do more than count (Decisions, UNVERIFIED, OQ-336):

- COL: an own Cold proc that makes n_COL = 10 (cap 10) starts Frozen (§8.6); Frozen removes the own stacks only; at
  thaw `min(3, cap_COL − N_ext,COL)` own stacks are inserted (no new Frozen from the thaw insertion itself).
- BLA: an own Blast proc that makes n_BLA = cap_BLA detonates (§11) with host hits for the own stacks only; the
  external stacks stay.
- HEA: Heat is present for the whole window; the strip counter is k_h = 4 throughout (overview T-43); own Heat procs
  join one episode that the external source keeps live until the window ends (every re-application by that source
  refreshes the pool, `W:Damage/Heat_Damage` lines 63-66), so own procs stay in the pool until then (cap_HEA still
  applies); the pulse grid is anchored at t = 0; the episode's first source is the external one (§10.3: E^dot_HEA = v
  of `heatInherit`, F_tick = F_inh, each 0 when unset).
- SLA, TOX, ELE, GAS: external instances only count; own instances tick normally.

With T-06 alone this reduces to `n_VIR = min(N_ext + n_VIR,own, cap_VIR)` with own stacks beyond the cap replacing the
oldest own stack (overview T-06).

### 6.4 Delivered procs and on-proc triggers

A proc is **delivered** when its application creates or refreshes something on the target: a stack, a DoT instance, a
Heat pool entry, a Blast fuse, a single-instance status, or (Blast at SD_p ≤ −1) its immediate host hit. On-proc
triggers fire only for delivered procs, in the order of §7.1 P4b: status-triggered arcanes and mods of `04` (Primary
Frostbite, Primary Blight, Conjunction Voltage, ...), and every other "on status" condition of `04`. Per discard case:

| Case | Delivered? | Source |
|---|---|---|
| inserted normally, or inserted after replacing the oldest stack (§6.2) | yes | §6.2 |
| Cold proc on a Frozen target | no: "Frozen enemies cannot receive additional Cold stacks", so no Primary Frostbite stack or refresh | `W:Damage/Cold_Damage` line 24; `04` (Primary Frostbite row) |
| Impact roll or forced Impact on a ragdolled target (RAG, §8.10) | no Impact status; Internal Bleeding and Hemorrhage still trigger, and Magnetic Welt by `02`'s Decision | `W:Damage/Impact_Damage` lines 25-26; `02` E-2.15 |
| proc of a type the target is immune to | no | Decision, UNVERIFIED (OQ-329) |
| proc nullified by SD_p ≤ −1 (§5.1) | no (Blast: yes, its host hit lands) | Decision, UNVERIFIED (OQ-329) |
| Heat proc that joins the pool and pushes the oldest proc out (§10.2) | yes | §10.2 |

The Impact-triggered procs are `02`'s; this table only fixes which events `04` and `02` may treat as a status having
been applied.

---

## 7. Timing and event order

This section makes CD-04 and CD-05 operational for both evaluators.

**Time model (overview CD-14, `08` E-8.0a).** Every event time is held on the integer microsecond grid, and every
equality, ordering, expiry and window test of this document is an exact integer comparison of grid values; equal-time
cases are decided only by the order of §7.1:

```text
Event times on the grid (restates 08 E-8.0a for the times of this document)
  τ̂ = round_half_even(τ·10^6)  (i64)     schedule times of 06 and 07 (shot, hit, child), converted once when read
  D̂ = round_half_even(D·10^6)            every duration of this document, computed in f64 first: D_s (E-3.1),
                                         D_HEA, D_BLA, D_frz, D_KND, D_LIF, D_RAG, h_up, h_dn
  derived times are integer sums:  tick   t̂_p + (δ_s + m)·10^6        (an exact integer)
                                   expiry t̂_a + D̂_s                   (stacks, instances, fuses, Frozen, KND, LIF, RAG)
                                   Heat   t̂_exp = t̂_p + max_q D̂_HEA,q;  pulses t̂_ign + m·10^6;
                                          up-steps t̂_ign + j·ĥ_up;  down-steps t̂_exp + j·ĥ_dn  (E-3.8)
  "τ ≤ t + ε" for a time anywhere in this document reads τ̂ ≤ t̂; the ε of E-3.1 guards the tick-count floor only
```

The grid removes the f64 drift of cumulative schedule sums: at 10 shots/s the 61st fire time summed in f64 is
5.999999999999995, which a raw f64 test would place before the expiry at 6 of a stack applied at t = 0; its grid value
is 6,000,000, equal to the expiry, so the stack is gone (TV-03-18(c)). It also absorbs the f64 error of durations:
`6·(1 + SD)` = 11.999999999999998 for the SD sum of TV-03-13 has D̂ = 12,000,000. Times that are equal in exact
arithmetic get the same grid value except within the accumulated rounding of a half-microsecond boundary, a residual
case `08` §4.3 accepts. Heat steps use the rounded intervals ĥ_up and ĥ_dn (step j at `t̂ + j·ĥ`), so that `08`'s
integer quotient `floor((τ̂ − t̂)/ĥ)` (E-8.6) counts exactly the steps that MC has taken. Because expiry never cancels
a tick (§7.3), the outcome of a tick that coincides with its own instance's expiry does not depend on whether the two
grid values coincide.

### 7.1 Processing order at one time τ

```text
P1  State transitions due at τ:
      stack and DoT-instance expiries (t̂_apply + D̂_s = τ̂): the stack or instance stops being present (n_s, n, debuff
      value); an expiring DoT instance keeps the tick scheduled at τ, which still lands in P2 (§7.3);
      Void, Knockdown, Lifted, ragdoll ends; Frozen end and its 3-stack re-application (§8.6); Heat strip steps (§10.4).
P2  Scheduled damage due at τ, from procs applied before τ:
      DoT tick groups with m ≥ 1 (SLA, TOX, ELE, GAS, Magnetic-break ELE), Heat pulses, Blast fuse expiries.
      Order among them: creation order of the oldest proc in each group.
      After each group: if it broke OG or SH (layered pass), the Magnetic break proc is created and applied at once and
      its m = 0 tick lands before the next group (§13); an extra hit triggered by a Blast host instance (Xata's
      Whisper, `04` E-4.13a) lands right after that instance.
P3  Heat episode end if t̂_exp = τ̂ (after its pulse at τ, if any); schedule the strip ramp-down.
P4  Each shot scheduled at τ, in schedule order (instant-burst shots are separate shots). For a shot S with
    damage instances j = 0 .. J−1:
      a. instance order: pellets in order (one merged instance per beam tick, CD-10); each pellet's derived instances
         right after it: its extra hits (E-14, `04` E-4.13) and a Melee Duplicate copy (`04`); then radial child
         instances of S that hit the target at τ (the explosion of a projectile that hit it, `06`), after all pellets
         (Decision, UNVERIFIED, OQ-334), each followed by its own extra hits (overview E-14: radial instances on the
         target trigger extra hits; DoT ticks never do). Derived and child instances are instances of S: they see
         σ_S, their procs count for the CO-family reads (n_j, n_COL,j) of later instances, and their own reads count
         the procs of every instance before them.
         For j = 0 .. J−1: compute the instance's damage against σ_S, the target state after P1-P3 and before any
         proc of S; the CO-family reads of 04 E-4.1 (the count n_j and the Cold stack count n_COL,j of Secondary
         Shiver and Shattering Frost) are taken from σ_S^{<j} (E-3.12), which adds the procs of instances
         0 .. j−1 of S; every debuff (cold_cd, punc_cc, Viral, ...) reads σ_S;
         roll crit and procs (02); apply the damage (05). A layer break here creates a Magnetic break proc (§13).
      b. apply the procs of S: instance order, then the proc-list order of 02; Magnetic break procs of S come
         after the procs of the instance that broke the layer. Insertions, replacements, Frozen start, Heat
         ignition and pool joins, Blast detonations (host hits land here, in stack order) happen in this step; a
         layer break by a detonation host hit creates its break proc right after that hit; an extra hit triggered
         by a host hit lands right after it.
      c. land the zero-delay ticks created in b (ELE and GAS tick m = 0, Magnetic-break tick m = 0), grouped per
         §9.5, evaluated against the state after b. A layer break by one of these groups creates its break proc right
         after the group, and that proc's m = 0 tick lands next, still in c.
      d. DoT detonation, if an instance of S carries the `dot_detonate` trait (Harmony heavy attacks, §9.7).
Events that steps b to d create at time τ are processed inside that step; nothing created at τ runs before P4.
```

Sources: `W:Damage/Viral_Damage` lines 31-36 and the editor's comment block lines 37-56 (Simulacrum-derived: a
shot's pellets all deal damage, then the shot's procs apply; an instant burst's later shots see earlier Viral; an
Electricity tick created with a Viral proc is amplified at once); CD-04; `F:addmul` §1.2 (CO counts earlier pellets of
the same shot). Extending "no sibling benefit" from Viral to every debuff is CD-04's decision (UNVERIFIED, OQ-028).
Secondary Shiver's per-stack term is a CO-family bonus, not a debuff: its n_COL follows the CO rule, so earlier
pellets' Cold procs count (`04` E-4.1, Decision, UNVERIFIED, `04` OQ-433), while `cold_cd` of the same pellet does
not see them (TV-03-39).
`08` E-8.18 processes extra hits after their parent and Melee Duplicate as the next instance, as above. Placing radial
children after the pellets of their shot, and the timing of break procs and extra hits created inside P2, P4b and P4c,
are Decisions (UNVERIFIED, OQ-334, OQ-308).

### 7.2 When a status starts and stops affecting damage

- **Start.** A stack applied in P4b of shot S at τ affects: the zero-delay ticks of S (P4c); every later shot at τ;
  everything at times > τ. It never affects the instances of S (sibling pellets included) except through the
  CO-family reads (n and n_COL, E-3.12) of later instances of S, and never a P2 tick at τ.
- **Stop.** A stack applied at t_a is present on the half-open interval `[t_a, t_a + D_s)`, on the grid
  `[t̂_a, t̂_a + D̂_s)`: it is removed in P1 at `t̂_a + D̂_s`, so a direct hit, a debuff read and a CO count whose grid
  time equals `t̂_a + D̂_s` do not see it. For a DoT instance
  this ends its presence only; its own tick scheduled at `t_a + D_s` (SLA and TOX whenever `6·(1 + SD_p)` is an
  integer, SD_p = 0 included) still lands in P2 (§7.3). Heat presence ends in P3 at t_exp.
- **Heat strip.** The step counter changes in P1 at its step times; direct hits and ticks at that time see the new
  value. The counter is a target state that outlives the Heat episode (§10.4).
- **Snapshots.** Weapon-side terms of a proc (seed, crit tier, body part, faction, elemental and status-damage
  bonuses, SD_p, M_OG,p) are fixed at proc creation (CD-05). One target-side term is also fixed at creation: the
  status-damage vulnerability V_sd,p ("only applies to status procs dealt after the vulnerability is applied",
  `W:Burning_Hate` line 32; §9.4). Target-side terms (Viral, Magnetic, armor with its strips, type modifier,
  vulnerability, innate DR) and the target-condition second dips M_tick (Mecha Empowered mark, Vauban incapacitation;
  §9.4) are read when the tick or Blast hit is processed (D-031).

### 7.3 When a DoT tick lands

```text
E-3.6  Tick schedule of a proc p of s applied at t_p (time of its shot); fixed when p is created
  SLA, TOX:     τ_m = t_p + (1 + m),  m = 0 .. N_s − 1        (P2)
  ELE, GAS:     τ_0 = t_p in P4c of p's shot;  τ_m = t_p + m, m = 1 .. N_s − 1   (P2)
  HEA:          pulses of the episode (E-3.7), not per proc
  BLA:          one hit at t_p + D_BLA (P2), unless detonated earlier (§11)
  Magnetic break: as ELE, from the break time t_b (§13)
  Times are grid values (§7): t̂_m = t̂_p + (δ_s + m)·10^6, an exact integer, and the expiry is t̂_p + D̂_s; the last
  SLA or TOX tick equals the expiry exactly whenever 6·(1 + SD_p) is an integer (SD_p = 0 included), also when the
  f64 product carries summation error (D̂ rounds it to the integer).
Every scheduled tick lands, including a tick at the instance's own expiry time (P1 removes the instance's presence
before P2; its tick still lands). Only replacement or cap removal (§6.2) and DoT detonation (§9.7) cancel ticks that
have not landed; expiry never does. A tick never lands after the end of the evaluation window for window metrics; it
is reported as pending (overview §4.1).
```

Source: `W:Status_Effect` lines 422-448 and `W:Damage_over_Time` table (Slash, Heat, Toxin first tick at 1 s and a
damaging tick at 6 s, six ticks; Electricity and Gas at 0 s to 5 s, "the last tick deals no damage" at 6 s);
`W:Damage/Slash_Damage` lines 44-50 (tick at 6 s ✓, none at 7 s); CD-06 for N_s. The DoT page's
`floor(rate·(Duration − Delay)) + 1` counts the non-damaging tick at 6 s for Electricity and Gas and is not used
(`F:status` §2.3). TV-03-14 and TV-03-18 cover the tick at the expiry time.

---

## 8. Debuff functions

All functions return the value for the stack count n_s of the state that the consuming damage instance sees (§7.2).

```text
E-3.3  Debuff functions
  viral_mult(n) = 1 if n = 0, else 2 + 0.25·(n − 1)                      on HP damage (incl. armored HP) only
  mag_mult(n)   = 1 if n = 0, else 2 + 0.25·(n − 1)                      on SH and OG damage only
  corr_strip(n) = 0 if n = 0, else min(1, c₁ + 0.06·(n − 1))           c₁ = 0.26 (Hydroid variant: 05 E-5.12)
  heat_strip    = 1 − 0.5^(k_h / 4),  k_h ∈ {0, 1, 2, 3, 4}             (§10.4)
  cold_cd(n)    = 0 if n = 0;  0.10 + 0.05·(min(n, 9) − 1) for n ≥ 1;  1.0 while Frozen
  punc_cc(n)    = 0.05·n                                                 (n ≤ cap_PUN, up to 7 with Evensong)
  tau_sc(n)     = 1 + 0.10·n                                             factor in M_sc (E-8)
```

- `heat_strip` is a function of the target's step counter k_h (E-3.8), not of Heat presence. The strip that E-15 reads
  is `c_heat = heat_strip(k_h)` for the current k_h **with or without live Heat**, including during the ramp-down after
  the episode has ended ("After the proc ends, the target will regain armor every 1.5 seconds for 6 seconds",
  `W:Damage/Heat_Damage` line 98). Consumers MUST NOT gate it on Heat presence; only Heat immunity gates it (an immune
  target never has k_h > 0). `05` E-5.10 (c_heat = 0 only when Heat-immune or k_h = 0) and `08` (its ARM term with
  `P_kh(x; τ)`) follow this, and so does the overview symbol `heat_strip(k_h)` (§2.4, E-15; adopted from OQ-322,
  TV-03-19).
- `punc_cc(n)` reaches n = 7 (0.35) with Evensong (§6.1); the overview row now reads n ≤ cap_PUN (adopted from
  OQ-335).

### 8.1 Viral and Magnetic

`viral_mult` multiplies every damage instance to health (direct, radial, DoT ticks, Blast hits, extra hits, break
procs), also when armor reduces it; never shields or overguard. `mag_mult` multiplies every instance to shields and
overguard. Both are 1 without a live stack (D-029). n ≤ 10 (4.25 at 10; the U27.2 note "up to 4.5x" is superseded by the
formula and in-game text, `F:status` §4.13). Magnetic also stops shield regeneration (TTK only, `05`). Sources:
`W:Damage/Viral_Damage` lines 18-31 (Viral on overguard was a bug fixed in U31.6); `W:Damage/Magnetic_Damage`
lines 20-30; `W:Overguard` (Magnetic on overguard since U36). Stage placement: E-12, `05`.

### 8.2 Corrosive

`corr_strip` reduces armor multiplicatively with the Heat strip and with the additive group of ability strips and
Corrosive Projection (E-15, `05`). With β_COR the strip reaches 0.92 (12 stacks), 0.98 (13) and 1.0 (14); with
Hydroid's passive 1.0 at 10 stacks (`W:Damage/Corrosive_Damage` §General, U36; `W:Archon_Shard` line 351). The
overview's `min(1, 0.20 + 0.06·n)` equals the c₁ = 0.26 case. The Hydroid variant (first stack 0.50) is `05` E-5.12,
selected by `05`'s scenario flag `armor_hydroid` (proposed toggle, `05` OQ-523); this document only supplies n_COR and
the cap.

### 8.3 Puncture

Flat (absolute) crit chance on weapon damage the target receives: `punc_cc(n)` enters ΣCC_abs of E-5 for every
non-AoE weapon instance (not radial, not abilities); the DoT of the hit inherits it through the crit tier
(OQ-017). The enemy-damage reduction (40 % + 10 %/stack) has no DPS effect. Source: `W:Damage/Puncture_Damage`
§General ("flat critical chance buff (like Arcane Avenger), but does not apply to Area of Effect damage or Warframe
abilities"); `W:Critical_Hit` §Absolute Increases; U33.6 history (`F:status` §4.2).

### 8.4 Tau

`tau_sc(n)` multiplies the status chance of every damage instance that rolls status on the target, including the
Tau-dealing weapon's own later hits; it does not add forced procs. Example (`W:Damage/Tau_Damage`): 210 % SC → 315 % at
5 stacks, 420 % at 10. Placement: E-8 (`02`). Self-application is a Decision (UNVERIFIED, OQ-311) that follows the
in-game description ("Tau increases Status Chance of all subsequent attacks", `W:Damage/Tau_Damage` line 8) and the
inspiration (a Tau attack counts its own earlier Tau procs, `I:status` §3.3 step 1, `JS L9989-9996`). Two wiki sentences
read against it: "It increases Status Chance received from all other status sources by 10% per stack"
(`W:Damage/Tau_Damage` line 13) and "increases the Status Chance received from all other status effects"
(`W:Status_Effect` line 226).

### 8.5 Radiation and Impact

No effect on the player's damage. Radiation's ally-damage bonus (100 % + 50 %/stack, 550 % at 10; on Liches and
similar +100 % (+50 %/stack) taken from allies, 250 % at the 4-stack cap, `W:Damage/Radiation_Damage` line 27) and
Impact's stagger and mercy threshold (+8 %/stack) are not modelled. Both count in n. Overguarded enemies receive
Radiation but ignore it until the overguard is gone (`W:Damage/Radiation_Damage` line 27).
The Internal Bleeding, Hemorrhage and Magnetic Welt triggers belong to the Impact *proc* (`02`); a ragdolled target
receives no status from an Impact roll (`02`, `W:Damage/Impact_Damage`).

### 8.6 Cold and Frozen

```text
E-3.11 Frozen
  Trigger: application of a Cold proc p makes n_COL = 10, cap_COL = 10, target not no_freeze
  At t_f (P4b): remove all Cold stacks (Decision, UNVERIFIED); Frozen on [t_f, t_f + D_frz), D_frz = 3·(1 + SD_p)
  While Frozen: cold_cd = 1.0; n_COL reported as 10 (CO count, Secondary Shiver) (Decision, UNVERIFIED);
                Cold procs discarded (not delivered, §6.4)
  At t_f + D_frz (P1): Frozen ends; insert 3 Cold stacks with t_apply = t_f + D_frz and D = 6·(1 + SD_p)
                (fresh timers: Decision, UNVERIFIED)
  no_freeze targets (Demolishers): 10 stacks stay stacks; cold_cd(10) = 0.50 (UNVERIFIED)
  All four UNVERIFIED points: OQ-305
```

`cold_cd` is an absolute crit-damage bonus added after relative crit-damage mods and before the crit tier (ΣCD_abs of
E-4, `02`), not applied to AoE damage. Frozen "receive[s] double Critical Damage", read as +1.0 by the wiki table.
Sources: `W:Damage/Cold_Damage` lines 18-25 and table (U36: 10 stacks and Freeze; U33.6: crit damage bonus), worked
examples Kunai 3.86× and Paris Prime (TV-03-08); `W:Status_Effect` line 317 (SD lengthens Frozen "and subsequent Cold
status"). Frozen prevents shield recharge (TTK, `05`). Melee Careen's ×2.5 reads the Frozen flag (`04`).

The Cold page states only that the 10th stack freezes for 3 s, that "Frozen enemies cannot receive additional Cold
stacks" and that "After the Frozen effects wears off, 3 Cold stacks will remain" (lines 23-24). Everything else in
E-3.11 is a Decision (UNVERIFIED, OQ-305):

| Point | Decision | Alternatives |
|---|---|---|
| stacks at freeze | all 10 removed; the state is "Frozen" | (b) the 10 stacks keep their own timers under Frozen, and the 3 left at thaw are whichever have time left |
| n_COL while Frozen (Secondary Shiver +0.45 per stack, `04`) | 10, so Secondary Shiver gives +4.50 | (a) 0 (no stacks); (b) the live count of the kept stacks |
| timers of the 3 thaw stacks | fresh, D = 6·(1 + SD_p) from the thaw | the timers of 3 kept stacks (alternative b above) |
| cold_cd at 10 stacks without freezing | 0.50 (the table stops at 9) | 0.55 by extending +0.05 per stack |

COL counts as present for n while Frozen under every alternative (E-3.12). TV-03-16 and TV-03-27 encode the
Decisions.

### 8.7 Microwave

Nukor and Kuva Nukor only. Every hit of an attack with `02`'s hand-table special rule `microwave` (`02` §4.7, OQ-215;
no data module has a Microwave field) applies MIC; it lasts for the rest of the window and counts once in n. The
body-part enlargement is not modelled. Source: `W:Microwave`; `W:Status_Effect` lines 161-165 ("infinite duration and
counts as an individual status"). How the game applies it per hit is not documented (UNVERIFIED).

### 8.8 Void

A 2.5 m projectile-attracting field for 3 s, centred on the body part hit (U29.5). No damage effect in a
single-target model; counts in n. Stacking and SD behaviour are undocumented: Decision one instance, a new proc
restarts its timer, D scaled by SD (UNVERIFIED, OQ-311). Void's ×1.5 against overguard is a damage-type rule (`05`).
The independent `PT_VOID` status (Sentient adaptation reset, `W:Status_Effect` line 205) is not a Void proc and does
not count.

### 8.9 Knockdown and Lifted

- KND lasts D_KND = 3 s (UNVERIFIED: no page states a duration; 3 s is the documented finisher window of a Stun,
  `W:Status_Effect` lines 199-203; OQ-309), not scaled by SD; a new KND proc restarts it; KND removes LIF.
- LIF lasts `D_LIF = M(c)` seconds, where M(c) is the combo multiplier (`combo_mult`, `07`) of the counter at the
  moment of the inflicting hit, **before** any combo consumption by that hit (heavy attacks and heavy slams consume
  combo, `07`), whatever the attack type: light attacks use it here although they never use M(c) for damage (T-10).
  1 s for non-melee sources. Not scaled by SD. Sources: `W:Lifted` line 6 ("1 second at base, increasing by 1 second
  for every Melee Combo tier, up to 12 seconds at 12x combo"); `W:Melee` line 799 ("The higher the combo multiplier at
  the time of a heavy attack or heavy slam attack, the longer enemies can stay suspended"). "Before consumption" is a
  Decision read from "at the time of" (UNVERIFIED, OQ-309).
- A LIF proc on a Lifted target ends Lifted and ragdolls the enemy (§8.10); no LIF stack results. "Any non-melee
  attack on an enemy in a Lifted state will also prematurely end it" (`W:Lifted` lines 10-11): every non-melee direct,
  radial or extra-hit instance ends Lifted in P4b of its shot. DoT ticks, Heat pulses, Blast host hits and Magnetic
  break ticks are not attacks and do not end it (Decision, UNVERIFIED, OQ-309). LIF removes KND.
- Conflict: `W:Melee` line 690 says "1 second, increased by 1 additional second for each Combo Multiplier", and its
  caption at line 684 shows a 4× heavy slam lifting for 5 s, i.e. `1 + M(c)`; the per-mechanic `W:Lifted` page (and
  overview §1.4) gives `M(c)`. Recorded in OQ-309.
- Overguarded enemies ignore the crowd control of KND and LIF; whether the status still counts in n is undocumented:
  Decision it counts (UNVERIFIED, OQ-309). Both count in n and cannot coexist (`W:Condition_Overload_(Mechanic)` lines
  182-185; `W:Status_Effect` lines 153, 160).

### 8.10 Ragdoll state RAG

RAG is a target state, not a StatusType: it is never counted in n_s or n and deals no damage. `02` reads it for its
Impact rule ("Enemies that are currently ragdolled cannot be inflicted with Impact statuses", `W:Damage/Impact_Damage`
lines 25-26; `02` E-2.13, E-2.14), from the state before each shot (CD-04). The rule is active in both evaluators: MC
reads RAG from the timeline of this section; the EV evaluator mixes each proc event's results over RAG = 0 and
RAG = 1 with the probability ρ_RAG that `08` supplies (`02` E-2.13a), never an expected δ_rag.

```text
E-3.13 Ragdoll state
  Entered in P4b of the shot (after that shot's Impact procs, which were rolled against σ_S):
    - by a forced proc named "Ragdoll" in weapon or stance data (§4.2; about 33 weapon ForcedProcs and 86 stance Procs)
    - by a LIF proc on a Lifted target (§8.9)
  Not entered on targets immune to ragdoll: targets with the `05` field `cc_immune` (bosses, Acolytes, the
    Demolisher family; `05` §4.1, EC-05-93; `W:Ragdoll` line 4, `W:Demolisher` lines 68-69) and units with
    overguard (og_present of the view or pass, E-3.2: live OG in the layered pass, true in the `overguard` view,
    false in the other views); ρ_RAG is therefore per view (02 E-2.13a)
  Duration D_RAG = 3 s, not scaled by SD; a new entry restarts it          (Decision, UNVERIFIED, OQ-326)
  Ends in P1 at t_RAG + D_RAG
  Interactions: RAG does not remove existing IMP stacks (they still count in n) and does not end KND (Decision,
    UNVERIFIED, OQ-326); `W:Status_Effect` lines 171-176 say Ragdoll "cannot be active with Stagger, Big Stagger, or
    Knockback", which this model reads as crowd-control animations only
```

No page gives a ragdoll duration (`W:Ragdoll` lines 1-6; `W:Status_Effect` lines 171-176); 3 s reuses the Knockdown
stand-in of §8.9. Ownership is settled: overview §7.1 and §7.2 make this document the owner, `07` emits the
crowd-control proc events (`07` §4.1, OQ-734), `02` reads the state (OQ-219); the duration and interactions stay open
(OQ-326).

---

## 9. DoT ticks (E-13 in full)

### 9.1 Proc snapshot

A proc p of s ∈ {SLA, HEA, TOX, ELE, GAS, BLA} created by damage instance i of attack a stores:

```text
E-3.4  Proc snapshot (restates E-13)
  S_p  = Y_i · (1 + F_i) · G_i · b_i · c_k,i · Part_dot,i    seed; Y_i per E-4.7 (CO per co_dot_mode), not quantized
  X_p  = K_s · (1 + E^dot_s) · (1 + F_tick,p) · (1 + A_sd,i + A_sd,s,i) · V_sd,p
  K_s        = 0.35 SLA; 0.5 HEA, TOX, ELE, GAS; 0.3 BLA
  F_tick,p   = F_i, except HEA (§10.3)
  E^dot_s    = §9.3; HEA: the episode's inherited value (§10.3); SLA, BLA: 0
  c_k,i      = 1 + k·(h·CDM − 1)  or  1 + SDB + k·(h·CDM − 1) for a stealth hit (E-12, T-08), as rolled by instance i
  Part_dot,i = Part_i                                       (E-7 / 02 E-2.10, M_son included: Sonar spots are
                                                             inherited, D-095; Detect Vulnerability by analogy, OQ-312)
  b_i        = merged beam count of a beam tick (CD-10), else 1
  V_sd,p     = status-damage vulnerability of the target when p is created (§9.4), snapshotted
  M_OG,p     = M_OG of attack a (Secondary Fortifier); stored with p, applied only on overguard (§9.4)
  SD_p       = SD + SD_s of the source (E-3.1)
At tick time τ the group value (E-3.5) is multiplied by M_tick(τ) (§9.4); the overview's M_dot,s = V_sd,p · M_tick(τ).
```

`W:Damage_over_Time` §DoT Damage Scaling lines 167-186; per-type pages (Slash, Heat, Toxin, Electricity, Gas lines
22-36; Blast lines 27-55): `Modded Base Damage = Base × (1 + Base Damage Bonuses) × (1 + Faction)`, tick `= C × Modded
Base × (1 + element bonus) × (1 + Faction) × (1 + Status Damage) × Additional Multipliers (crit, body part)`.
`W:Damage/Calculation` lines 613-617: ticks are not quantized; "Critical DoTs use the effective critical multiplier of
the inducing hit" with crit quantization. Faction double dip: `W:Faction_Damage_Bonus` intro, every DoT page
(`(1+F)²`, +69 % for 0.30, +140.25 % for 0.55). c_k and Part_dot are factors of the seed, so the "+1" of E-3.5 is
never multiplied by them: `W:Damage/Calculation` lines 632-635 store the crit multiplier in the seed ("critical seeds
of 80.0195389" = 40 × Q(2.0); "(80.0195389 + 1) × 0.35 = 28.3568382"), and overview E-13 adopted that structure
(D-025, OQ-301). Part_dot follows by analogy (no page gives a headshot example with the "+1"; UNVERIFIED, OQ-301).
Sonar inheritance: `W:Sonar` lines 25-26 ("DoTs ... will have their damage calculated on the resulting damage if
proc'd from a hit that hits a Sonar weakspot"; the U44 rework revision), which overview D-095 prefers over the
pre-rework `W:Damage_over_Time` line 171 and `W:Damage/Slash_Damage` line 60; Detect Vulnerability spots were made
"Weak Points instead of weak spots so it matches Banshee's updated Sonar" (`W:Detect_Vulnerability` lines 50-52), which
supports the analogy without stating DoT behaviour (UNVERIFIED, OQ-312).

### 9.2 Inheritance list

| Term of the inducing instance | In the proc? | Source |
|---|---|---|
| base damage incl. progenitor and Incarnon flat (B'), base bucket A | yes (Y) | DoT page line 170; `F:core` §3 |
| CO term | per `co_dot_mode` (E-4.7: seed by default; Tysis and Stinger exclude; Pox recalculates per tick) | `F:addmul` §1.12 |
| G_F damage buffs (Eclipse, Furious Javelin, Garuda, ...) | yes, once (in Y) | DoT page line 170; D-041 |
| G_F members range falloff Φ(d) and bow charge scaling | yes, once (in Y), by the same rule | no source mentions status damage (`W:Damage_Falloff` is silent); Decision, UNVERIFIED (OQ-330) |
| stance hit multiplier S_j and heavy-attack multiplier H_j (ST-1 base factors, not G; overview §2.4, `07` E-7.20, E-7.20b) | yes, once (in Y) | overview §2.4 `mult_other`; `F:addmul` §2 |
| G members (melee combo M(c), sniper combo, Primed/Charged Chamber, Synth Charge, beam ramp, Navigator, Melee Careen ...) | yes, once (G) | DoT page line 179 (combo counters); `W:Primed_Chamber` ("applies to ... status damage"); `F:addmul` §2 |
| G members that test a target condition (Mecha Empowered mark, Vauban incapacitation) | first dip once in G as the inducing hit saw it; second dip M_tick at each tick (§9.4) | `W:Mecha_Empowered` line 32; `W:Vauban/Abilities/Passive` lines 14-17 |
| faction F | twice (seed and tick) | §9.1 |
| crit tier as rolled, incl. Cold and Puncture effects on that roll, head crit, stealth term | yes (c_k, in the seed) | Damage/Calculation lines 617, 632-635; DoT page line 177; OQ-017 |
| body part, weak-point and headshot bonuses | yes (Part_dot = Part, in the seed) | DoT page line 180; seed placement OQ-301 |
| overguard multiplier M_OG (Secondary Fortifier) | yes, on overguard only (M_OG,p) | §9.4 |
| Sonar, Detect Vulnerability spots | yes (M_son is part of Part) | `W:Sonar` lines 25-26 (D-095; the DoT page line 171 and `W:Damage/Slash_Damage` line 60 say no, pre-rework text); Detect Vulnerability by analogy, UNVERIFIED (OQ-312) |
| merged beam count | yes (b): DoT scales twice with beam multishot | `W:Multishot` §Continuous Weapons; CD-10 |
| per-type composition, quantization, physical and elemental mods | no (except E^dot of the proc's own element) | DoT page lines 172-176; Slash page ("Buzz Kill have no effect") |
| stance physical bonus S^phys | no (not in Y) | E-2; `F:melee` §2.1 |
| finisher-damage mods (Finishing Touch, Savage Silence) | no | `W:Damage/Slash_Damage` §Increasing Slash Proc Damage (D-303) |
| status damage A_sd, A_sd,s | yes | `W:Status_Effect` line 450 |
| status-damage vulnerability of the target (Burning Hate, Pageflight, custom) | yes, as live when p is created (V_sd,p) | `W:Burning_Hate` lines 28-34 |
| target-side terms (Viral, Magnetic, armor, type modifier, vulnerability, innate DR) | no: read at tick time | `W:Damage/Viral_Damage` line 31; `W:Damage_Vulnerability` §Mechanics; CD-05 |

### 9.3 Elemental scaling E^dot_s

```text
E^dot_HEA, E^dot_ELE, E^dot_TOX = Σ of every elemental bonus of that element on the attack (mods, riven element
    stats, arcanes, archon shards, Valence Formation of that element, non-combining additions such as Fireball
    Frenzy or the custom heatAdd), whether or not it was combined into a secondary element on the weapon
E^dot_GAS = Σ of literal Gas bonuses only (Leaded Gas +3.0, Valence Formation set to Gas, riven or spliced Gas)
E^dot_SLA = E^dot_BLA = 0
Excluded everywhere: buffing-field elements (Electric Shield, Conductive Sphere field Electricity, Energy Shell Heat,
    weapon-made fields), Nourish (Viral), Sentient Incision's weakness element, bonuses of other elements; base
    damage is not a bonus and never counts (innate elements, Chromatic Blade, Reactive Storm's converted Impact;
    01 E-1.15)
Computed as elem_dot_sum[s] (01 E-1.15) over the contributions whose dot_scaling flag is true
```

Sources: `W:Damage_over_Time` lines 172-176 (combined DoTs not buffed by component mods; Toxic Lash's forced Toxin is
buffed by Toxin mods combined into Corrosive); `W:Damage/Gas_Damage` line 44 and example 2 (Leaded Gas ×4);
`W:Damage/Blast_Damage` line 56 and §Notes ("Valence Formation or Thermal Transfer does not increase Blast status
damage"; Heat and Cold mods do not either; OQ-007); `W:Electric_Shield` via `F:buffs` §1.12 and §9 (field Electricity
"does not raise Electricity status damage"); `W:Valence_Formation` via `F:buffs` §8 (+200 % to that element's DoT, not
Blast); `W:Sentient_Incision` line 46 ("Elemental damage from this augment does not scale status damage of the
corresponding element", listed under Bugs, so a later fix would flip it). This section is the rule (overview §7.2,
`dot_elem`); each source's `dot_scaling` flag is the rule applied to that source and is data: `04` sets it for the
sources it catalogues (default true except buffing fields), and `01` sets it for the others (weapon-mod and riven
element stats, spliced and secondary-element mods, Sentient Incision, Reactive Storm, `heatAdd`; `01` §4.10.5 and
Table 4.10-B). A flag that disagrees with this section is a data bug. Membership of the Violet shard Electricity,
spliced Gas and Conductive Sphere's field element is UNVERIFIED (OQ-316).

### 9.4 Status damage, status-damage vulnerability and M_dot

- `sum_status_dmg` A_sd: Elementalist mods, riven Status Damage, Galvanized Elementalist, heavy-attack-only status
  damage (Empowered Blades) on heavy attacks; `status_dmg_type[s]` A_sd,s: Emerald Archon Shard (TOX, also on Toxic
  Lash procs), Conductive Sphere (ELE), Ash's passive (SLA +0.25 on every proc of the build when Ash is the frame,
  `W:Ash/Abilities` lines 3-5; its +0.50 duration is SD_SLA, §5). All additive in one bucket, multiplicative with
  everything else (`W:Status_Effect` lines 450-470; `W:Archon_Shard` line 337; D-069). Leaded Gas is E^dot_GAS, not
  status damage (D-301).
- **Status-damage vulnerability** `V_sd,p = 1 + Σ v_j` over the status-damage vulnerabilities live on the target when
  p is created, snapshotted in X_p. Members (additive with each other; values and trigger conditions in `04`): Burning
  Hate (+1.20 at rank 5; "Status Damage from all sources", "only applies to status procs dealt after the vulnerability
  is applied", "additive with Dante's Final Verse - Pageflight's Status Damage increase", "multiplicative with Melee
  Elementalist and Faction Damage Bonus", `W:Burning_Hate` lines 28-34), Pageflight's status-damage increase, and the
  custom stat `vuln_status_damage` (overview §6.3, "as Burning Hate"; a constant, present from t = 0). Snapshotting a
  target-side term at creation is an exception to CD-05, sourced by the Burning Hate page. Putting the custom stat in
  the same additive group follows the inspiration, where Burning Hate is itself the stat `vuln_status_damage: 1.2`
  (`JS L205-209`) and the mod and custom values are summed into one factor, `md = getStatValFromMods(
  "vuln_status_damage") + getCustomStat("vuln_status_damage")`, applied as `(1 + md)` (`JS L10467-10469`,
  `L10498-10502`); no game source exists for a custom stat (UNVERIFIED, OQ-324). Overview §6.3 and `04` §10 define
  the stat the same way (an entry of V_sd, additive with Burning Hate and Pageflight; TV-03-37).
- **Target-condition second dips** `M_tick(τ)`, read when each tick, pulse or Blast hit lands, as a factor of the whole
  group value (E-3.5):

  ```text
  M_tick(τ) = (1 + 1.5·n_aura)  if Mecha Empowered is active and the target is marked at τ, else 1
            × 1.25              if Vauban's passive is active and the target is incapacitated at τ, else 1
  ```

  The first dip of each is part of G of the inducing hit and so is in the seed (§9.2). Model: each "damage
  calculation" tests the condition once, the hit that creates the proc (seed) and each tick (M_tick). It reproduces
  the sources: Mecha "a multiplicative bonus applied each time damage is calculated against a marked enemy ... applied
  twice" on status damage (`W:Mecha_Empowered` lines 32-33); Vauban "applies a second time on status damage, if the
  damaging status effect is applied *while* an enemy is incapacitated" (×1.25 twice), "applied *before* the enemy hit
  is incapacitated, the damage bonus will apply only once" (seed ×1, tick ×1.25), and "the damage bonus for damaging
  status effects on an enemy is lost should the enemy no longer be incapacitated" (tick ×1; this model keeps the
  seed's ×1.25) (`W:Vauban/Abilities/Passive` lines 13-17). The alternative reading of "lost" drops the seed's ×1.25 as
  well, i.e. ×1 in total once incapacitation ends; the exact structure is UNVERIFIED (OQ-319, TV-03-32). Incapacitation
  means the crowd-control animation of an Electricity stun, Heat panic, Frozen, or the listed abilities, not the mere
  status (`W:Vauban/Abilities/Passive` lines 3-11); the predicates "marked" and "incapacitated" are `04` condition-class
  (a) states (Frozen comes from §8.6; stun and panic windows are `04`'s). `F:buffs` §1.19 omitted the "lost" sentence;
  `04` is notified through OQ-319.
- **Overguard multiplier** M_OG,p: the M_OG of the proc's source attack (Secondary Fortifier ×8, overview E-12), stored
  with the proc and applied by `05` (E-5.17 applies M_OG to every type, Cinematic included) when a tick, pulse or Blast
  hit of p lands on overguard; in a Heat group each proc's term `S_q·X_q` carries its own M_OG,q and the accumulator
  term carries M_OG,q₀. Source: `W:Secondary_Fortifier` lines 37-39 ("DoTs will also steal Overguard"; "Extra damage to
  Overguard is not inheritable and is dynamically applied", read as: applied at damage time, never passed on by Heat
  inherit). That status instances get M_OG at all is a Decision (UNVERIFIED, OQ-327); overview E-13 now applies
  M_OG,p on overguard to every type, Slash ticks included ("OG: × Mag · V_OG · M_OG,p · R_OG").
- The Magnetic break proc takes neither V_sd nor M_tick (Decision, UNVERIFIED, OQ-308); it never lands on overguard.

### 9.5 Tick groups and the "+1" accumulator

```text
E-3.5  Value of one tick group 𝒢 (weapon side), formed within one processing step (P2 or P4c)
  𝒢 for SLA, TOX:   one proc
  𝒢 for ELE, GAS:   all live instances of that status whose tick lands at the identical grid time τ̂ in the same step
                    (Magnetic-break instances excluded: each forms its own group, §13)      (Decision, UNVERIFIED, OQ-328)
  𝒢 for HEA:        the episode's pool at the pulse (E-3.7)
  tick_𝒢 = ( Σ_{p∈𝒢} S_p·X_p  +  a·X_{p₀} ) · M_tick(τ)      p₀ = the oldest proc in 𝒢 (application order); a = dot_acc
  For |𝒢| = 1 this is E-13: (S_p + a)·X_p·M_tick(τ). S_p holds c_k and Part_dot (E-3.4), so the accumulator is never
  multiplied by a crit or body-part factor.
  BLA and the Magnetic break proc never use a. Procs of extra hits (E-14, `04` E-4.14; Primary Debilitate, §9.8) take
  no "+1" (overview E-14, UNVERIFIED, `04` OQ-413): a group whose oldest proc p₀ is such a proc uses a = 0
  (Decision, UNVERIFIED).
Enemy side at the group's processing point: E-12 with the single type s of the tick (SLA as Cinematic: no type
  modifier, no armor; Viral on HP, Magnetic on SH and OG, vulnerability once; layer routing, innate DR, attenuation
  and the armor floor of 1 per 05). The group is one damage instance.
```

Source: `W:Damage/Calculation` lines 619-635 (single editor, added 2026-07/08; UNVERIFIED, D-025, OQ-023): the
accumulator of each tick group starts at 1; Heat consolidates all stacks into one tick; Electricity and Gas
"contributions occurring on the same pulse are consolidated into one damage number", +1 once; Slash and Toxin stacks
tick separately with their own +1. Examples reproduced in TV-03-02 to TV-03-05. f32 arithmetic is not emulated
(overview §2.1, NP-2 row: DoT ticks are f64; differences < 1e-6 relative). Display rounding `floor(x + 0.5)` is display
only. Reading "the same pulse" as an identical landing time (equal grid value, §7) is a Decision: the page could
also mean a shared per-target pulse clock, as Heat has, which would merge two Electricity procs applied 0.5 s apart
into one number per second. Excluding Magnetic-break instances from ELE groups is also a Decision. Both change the
"+1" count, the armor floor of 1 and attenuation (UNVERIFIED, OQ-328).

**Crit inside the seed (D-025, OQ-301).** The game stores the crit multiplier inside the seed: the page's Gas example
uses "critical seeds of 80.0195389" (= 40 × Q(2.0), line 632), and its Slash example answers the single-proc case
directly: "separate Slash stacks with seeds of 40 and 80.0195389 deal (40 + 1) × 0.35 = 14.35 and
(80.0195389 + 1) × 0.35 = 28.3568382" (line 635). Overview E-13 now has this structure (c_k and Part_dot in the seed,
adopted from OQ-301), and so do E-3.4 and E-3.5: the "+1" is added once per group and multiplied only by the
tick-level factors of the oldest proc. Because X_p holds no crit or body-part factor, the page's consolidated Gas and
Heat examples come out the same whichever proc is oldest (TV-03-04, TV-03-05). The members of a group can still differ
in X through A_sd, A_sd,s and V_sd (and, in ELE and GAS groups fed by several attacks, E^dot_s and F_tick); taking the
oldest proc's X for the accumulator then is a Decision (UNVERIFIED, OQ-301). The superseded structure ("+1"
multiplied by c_k) gave (40 + 1) × 0.35 × 2.0004885 = 28.7070 instead of 28.3568; TV-03-01 gives both values.

### 9.6 Per-proc totals

The weapon-side total of a SLA or TOX proc is `N_s·(S_p + a)·X_p` before target-side terms (and before M_tick). For
ELE and GAS the "+1" is paid once per tick group (E-3.5), not per proc: a group of g procs landing together (for
example the pellets of one multishot shot, TV-03-05) carries one accumulator, so the per-proc total is
`N_s·(S_p + a/g)·X_p` when the group members share X (in general the group's accumulator `a·X_{p₀}` is shared by its
g members). `08` either evaluates groups as groups or uses the expected group size g; charging a full a to every ELE or
GAS proc overcounts. The landed damage is the sum of the tick groups' E-12 values at their landing times (target state
varies). The inspiration credits `curr_duration × tick` at the hit (D-086, D-036). `08` uses these totals and the tick
schedule E-3.6. S_p is linear in c_k, so its expectation takes E[c] (C1 for procs that exist only on crits, overview
§4.2) while the "+1" carries no crit weight: the expected tick of a Slash proc in overview §3.8 step 10 is
`(143.7625·E[c] + 1) × 0.35 × 1.55`.

### 9.7 DoT detonation (Harmony)

"Enemies hit by Heavy Attacks while debuffed with active Slash, Heat, Toxin, Electricity, and Gas Status Effects will
instantly receive the full damage from all damage ticks of the procs; status effects are removed after inflicting
damage. Effect also applies to procs generated by the Heavy Attack itself" (`W:Harmony` lines 11-12; line 29:
"detonates status procs immediately after it does damage"); `W:Damage_over_Time` lines 186-191: DoT effects "apply all
of their damage ticks in a single damage instance" under Expedite Suffering, Final Verse (Tragedy), Divine Retribution
and the Harmony heavy attack. The abilities are out of scope (EC-03-68); the weapon is in scope.

```text
E-3.14 DoT detonation, in P4d of a shot S with an instance carrying the trait dot_detonate (07 hand-authored data)
  For each live DoT instance p of SLA, TOX, ELE, GAS (Magnetic-break instances included), in creation order:
    N_rem = number of its scheduled ticks not yet landed (ticks at τ already landed in P2 or P4c)
    one damage instance of type s with weapon-side value  N_rem · (S_p + a)·X_p · M_tick(τ)   (break proc: N_rem · T_brk)
    enemy side at τ (E-12 with type s; Viral, Magnetic, armor read now); then p is removed (its ticks cancelled)
  Heat episode (if live): N_rem = pulses with τ̂_m ≤ t̂_exp (grid, §7) not yet landed;
    one HEA instance of value N_rem · (Σ_{q∈pool} S_q·X_q + a·X_{q₀}) · M_tick(τ); then the episode ends at τ as in P3
    (pool cleared, down-steps from the current k_h scheduled with SD* of the pool before clearing)
  n_s and n drop accordingly; Blast fuses are not DoTs and stay
```

Decisions (UNVERIFIED, OQ-325): one damage instance per DoT instance (per Heat pool) rather than one for all statuses;
the "+1" counted once per remaining tick (the instance delivers the sum of the ticks it replaces); remaining Heat pulses
valued with the current pool; zero-delay ticks of the heavy's own ELE and GAS procs land in P4c before the detonation.
Faction, crit, body part and status damage stay as snapshotted. TV-03-35.

### 9.8 Procs of extra hits and Primary Debilitate

A forced proc of an extra hit (overview E-14, `04` E-4.14: Toxic Lash, Resupply, Demonium, ...) has the seed
`S_p = XH` (which already holds the parent's crit, body part and faction once, plus the extra hit's own faction) and
the X_p of E-3.4 with E^dot_s = `elem_dot_sum[s]`; it takes no "+1" (E-3.5). Faction thus enters three times.

Primary Debilitate's proc comes from a 0-damage extra hit (`04` §6.5), so `S = XH` would be 0, and no page gives its
seed; its faction count is three, as overview D-073 now states (`W:Primary_Debilitate` §Notes line 31: "Faction Damage
Bonuses ... multiply the Damage over Time effect of Heat, Electricity, and Toxin status three separate times";
`W:Extra_Hit` lines 138-139). Decision (UNVERIFIED, OQ-313): the proc is seeded like an ordinary proc of its type from
the inducing instance i with one more faction factor, `S_p = S_i·(1 + F_i)` with S_i the E-3.4 seed of i (c_k,i and
Part_dot,i included), X_p per E-3.4 for its own type s, and no "+1". This is the inspiration's structure (the
ordinary tick of that type from the same hit times a third faction factor, `JS L11432-11459`). TV-03-40.

---

## 10. Heat

### 10.1 Episode and pool

```text
E-3.7  Heat episode on a target
  Ignition: a Heat proc p applied when no episode is live: t_ign = t_p, pool = {p}, p₁ = p, t_exp = t_p + D_HEA,p.
  Join: a Heat proc p applied while an episode is live (t_ign ≤ t_p, episode not ended):
        pool ← pool ∪ {p} (if |pool| > cap_HEA: remove the oldest proc);  t_exp ← t_p + max_{q ∈ pool} D_HEA,q
  Pulses: at τ_m = t_ign + m, m = 1, 2, ... while τ̂_m ≤ t̂_exp (grid values, §7; processed in P2):
        tick = ( Σ_{q ∈ pool, q applied before the pulse} S_q·X_q  +  a·X_{q₀} ) · M_tick(τ_m)   (E-3.5; q₀ oldest)
  End: in P3 at t_exp: SD* is read from the pool, then the pool is cleared and the strip ramp-down is scheduled
        (§10.4). A Heat proc after the end ignites a new episode (new p₁, new pulse grid). D_HEA,q = 6·(1 + SD_q).
```

Sources: `W:Damage/Heat_Damage` lines 63-66 (one combined tick per second; a new proc adds its damage per second and
refreshes all stacks; duration reset to the longest status duration among the stacks; "stacked up indefinitely"),
line 54-61 table (first tick at 1 s, last at 6 s); `W:Damage_over_Time` ("DoT instance can be refreshed / No limit");
D-027. The pulse grid anchored at ignition, and a joining proc first contributing to the next pulse, are not stated
(Decision, matches the inspiration's TTK model `JS L13690-13703`; UNVERIFIED, OQ-302). A single proc with no refresh
yields `floor(6(1 + SD) + ε)` pulses, as CD-06 requires.

Heat procs forced by extra hits (Uriel's Demonium, Resupply set to Heat, Reconifex; overview E-14, `04` E-4.14) are
Heat procs like any other: they join or ignite the episode, with seed `S = XH` (E-14). Their tick-level factors follow
the inherit rule of §10.3, which overrides E-14's own tick-level elemental factor `(1 + E^dot_all,HEA)` and its
tick-level faction factor `(1 + F)` with the episode's first source's values; when such a proc is the first source
(p₁), its own E-14 values are the inherited ones. "ALL sources of Heat status can benefit from Heat inherit"
(`W:Damage/Heat_Damage` line 88). Which rule wins is a Decision (UNVERIFIED, OQ-332). The extra-hit proc's "+1"
follows the group rule of E-3.5 (the pool has one accumulator, from its oldest proc, and none while that proc is an
extra-hit proc).

### 10.2 Heat pool on status-capped enemies

The pool holds at most cap_HEA procs (4 on status-capped enemies, ∞ otherwise); the oldest leaves the pool when a new
proc would exceed it, and its contribution stops from the next pulse. The episode's t_exp still follows the join rule.
UNVERIFIED (CD-07 extended to Heat's pool; OQ-008, OQ-304).

### 10.3 Heat inherit

```text
For a proc p in an episode with first proc p₁:
  E^dot_HEA(p) = E^dot_HEA of p₁'s source            (p = p₁: its own)
  F_tick,p     = F of p₁'s source                    (p = p₁: its own)
  S_p keeps p's own Y, own F (once), own G, b, c_k and Part_dot;  A_sd, A_sd,HEA, V_sd, M_OG and SD_p: p's own
T-26 heatInherit = v set: an external Heat source is the first source of every episode on the target:
  E^dot_HEA(p) = v and F_tick,p = F_inh (T-26 custom stat heatInheritFaction, default 0; overview §6.3, adopted from
  OQ-315) for every own Heat proc. The external source's own ticks, presence in n and armor strip are not modelled
  (inspiration parity), except when T-43 also sets external Heat (§6.3).
Compat flag (overview D-028 Toggle "User (T-26) + Compat"): F_tick,p = F_i (faction squared) and E^dot_HEA = v or
  own, ignoring p₁ for own procs.
```

Source: `W:Damage/Heat_Damage` §Heat Inherit lines 68-90 (community term, rewritten 2026-11-14): the first stack's Heat
elemental and faction bonuses overwrite the new source's; base damage and status damage are not inherited; "While
Faction Damage is inherited, it is only applied once to subsequent procs that inherit it"; "If both the initial proc
and a subsequent proc have a faction damage bonus, each bonus will be applied once". The model "p₁'s faction replaces
the tick-level faction factor, own faction stays in the seed" reproduces both worked examples on the page (696/s and
236.5/s, TV-03-06). For one weapon with fixed bonuses, later procs therefore keep `(1+F)²` (OQ-006 decision).

### 10.4 Heat armor strip

```text
E-3.8  Heat strip step counter k_h (target state; 0 at window start)
  SD* = SD_p of the proc with the largest D_HEA in the pool when the steps are scheduled; at the episode end it is
        read before P3 clears the pool
  At ignition t_ign: cancel pending down-steps; schedule up-steps at t_ign + j·h_up, j = 1 .. 4 − k_h(t_ign)
                     (each: k_h ← k_h + 1, in P1)
  At episode end t_exp (P3): cancel pending up-steps; schedule down-steps at t_exp + j·h_dn, j = 1 .. k_h(t_exp)
                     (each: k_h ← k_h − 1, in P1)
  heat_strip = 1 − 0.5^(k_h/4):  0, 0.159104, 0.292893, 0.405396, 0.5
  Joining procs do not reschedule up-steps.
  c_heat (E-15) = heat_strip(k_h) for the current k_h, with or without live Heat (§8, normative); Heat-immune targets
  never ignite, so k_h stays 0. External Heat (T-43, §6.3): k_h = 4 for the whole window.
```

Source: `W:Damage/Heat_Damage` §Armor Stripping lines 95-110 (every 0.5 s after the proc current armor × 0.5^(1/4) to
50 % at 2 s; re-proccing does not speed it up; after the proc ends armor returns every 1.5 s over 6 s;
+100 % SD → 1 s steps, −50 % → 0.25 s); `W:Status_Effect` line 325 (ramp down also lengthened by SD); D-026.
Multiplicative with Corrosive and with the additive ability/Corrosive Projection group (E-15, `05`). Re-ignition during
the ramp-down resumes ramping up from the current step and the choice of SD* are Decisions (UNVERIFIED, OQ-303).
During the ramp-down no Heat is live but k_h > 0, and the strip still applies (`05` E-5.10 agrees; TV-03-19).

---

## 11. Blast

```text
E-3.9  Blast
  Stack value:  H_p = S_p · (1 + F_i) · (1 + A_sd,i + A_sd,BLA,i) · V_sd,p      (S_p of E-3.4, c_k and Part_dot inside)
  Host hit:     H_det = max(1, 0.3·H_p) on the weapon side, then × M_tick(τ), then E-12 with type BLA
                (the max(1, ·) is sourced for natural expiry only; applying it on the weapon side, before armor, and
                to detonation host hits is a Decision, UNVERIFIED, OQ-306)
  Fuse:         D_BLA = 1.5·(1 + SD_p);  stack lives on [t_p, t_p + D_BLA)
  Expiry:       at t_p + D_BLA (P2): one Blast instance H_det to the host
  Detonation:   inserting the stack that makes n_BLA = cap_BLA (10; 4 on status-capped enemies and bosses, §6.1), in P4b:
                host: one Blast instance H_det,q per live stack q (the new one included), in stack order
                  (Decision per the Blast page, UNVERIFIED, D-306; Calculation-page reading: nothing to the host)
                other enemies within 5 m, no falloff (multi-target only, not evaluated): one instance 3·Σ_q H_q;
                all stacks cleared
  Death:        the target dying with ≥ 1 live stack (or killed by a hit that applies a Blast proc) detonates
                for other enemies only (multi-target, not evaluated); the host takes nothing more
  SD_p ≤ −1:    no stack; the host instance H_det lands at once in P4b
  No "+1", no elemental scaling (E^dot_BLA = 0), status damage applies, faction twice, crit and body part apply
```

Sources: `W:Damage/Blast_Damage` lines 19-66 (U36 rework; edited 2026-11-03: 1.5 s fuse affected by SD; 30 % at expiry;
10 stacks detonate, "10 hits of 30% base damage to the target at once" (line 21); 300 % per stack to others in one
instance, no falloff, radius not changed by Firestorm or Fulmination; "The detonating enemy only receives the single
target damage and is not dealt this AoE damage" (line 23); "This radial damage does not affect the initial target,
which will only take the normal expiration damage of all procs" (line 59); killing blows with a Blast proc trigger the
full AoE; faction twice; no Heat, Cold, Valence Formation or Thermal Transfer scaling; crit, weak points and head
crits apply to both parts); `W:Damage/Calculation` lines 641-645 (edited 2026-11-04; bosses cap at 4; each stack
records H; "if a stack expires on its own (1.5 s) instead of being consumed by a detonation, it deals a single instance
of max(1, 0.3 × H)"; detonation: "the currently active stacks are not settled individually but are combined into a
single 5-meter radial Blast centered on the enemy, dealing 3 × (sum of the active stacks' H)"). The page's "H of the
hit that applied it" is read as the Blast page's modded-base product (`F:status` open 7; UNVERIFIED, OQ-306).

**Conflict: host damage at detonation (D-306).** The two pages disagree on what the host takes when the stacks reach
the cap. Blast page: ten (or four) hits of 0.3·H each, and the radial skips the host. Calculation page: the stacks are
consumed into the radial and "not settled individually", so the host takes no per-stack hits (and, if the radial is
read as including its centre, would take 3·ΣH instead). Overview §1.4 gives no clear winner: the Blast page is the
per-mechanic page and states the host rule three times (lines 21, 23, 59); the Calculation page is the more recent
edit (2026-11-04 against 2026-11-03, `F:status` header) and the more formal description; neither has a numeric
detonation example. Decision: the Blast page (per-mechanic page, explicit and repeated), UNVERIFIED. The test that
settles it: on a non-boss target, count the host's Blast damage numbers when the 10th stack lands (ten 0.3·H numbers
versus none, or one 3·ΣH number) (OQ-306, TV-03-12, TV-03-23). Whether the host's detonation damage is one instance
or one per stack only matters for the armor floor and attenuation: one per stack ("10 hits") is part of the same
Decision. Blast-element bonus scaling conflicts with `W:Damage_over_Time` (OQ-007, D-034). Xata's Whisper triggering
off Blast host hits is an `04` effect (D-071, `04` E-4.13a reads H_det); its extra hit lands right after the host
instance that triggered it (§7.1).

---

## 12. Gas and Electricity

**Gas.** Each proc is an instance with its own timer and N_GAS ticks at t_p + 0 .. N − 1 (E-3.6); cap 10 (status-capped
enemies 4) with oldest-replacement (§6.2); same-pulse ticks consolidate (§9.5); E^dot_GAS = literal Gas only (§9.3).
The cloud's radius (3 m, +0.3 m per stack, "6 m at 10 stacks"; the literal arithmetic gives 5.7 m, `F:status` §4.10),
its damage to other enemies, and its persistence after the host dies are multi-target behaviour (not evaluated). The
cloud has its own 1× headshot multiplier: on the host the tick uses the inherited Part_dot and nothing more, so
headshot bonuses and a Sonar spot count once (OQ-025 decision; OQ-312). Sources: `W:Damage/Gas_Damage` lines 18-60 (U27.3 for persistence).

**Electricity.** Each proc is an instance with its own timer and N_ELE ticks at t_p + 0 .. N − 1; unlimited (4 on
status-capped enemies); same-pulse ticks consolidate; E^dot_ELE includes Electricity mods combined into a secondary
element; the 3 m chain to other enemies is multi-target only; the 3 s stun is crowd control (not SD-scaled; bosses
immune); the chain's own 1× headshot is handled as for Gas (OQ-025). Tick 0 lands in P4c and therefore sees a Viral
proc of the same shot (`W:Damage/Viral_Damage` line 36). Sources: `W:Damage/Electricity_Damage` lines 18-60.

---

## 13. Magnetic break proc

```text
E-3.10 Magnetic break proc (U36)
  Trigger: any damage instance that depletes the overguard or the shield (layered pass only, D-054): a shot
           instance (P4a), a P2 tick group, Heat pulse or Blast expiry hit, a Blast detonation host hit (P4b), a
           zero-delay tick group (P4c), or a tick of an earlier break proc; with n = n_MAG ≥ 1 in the state that
           the breaking instance saw (σ_S for a shot instance; the state at its processing point otherwise)
  Pool:    P_max = scaled maximum OG or SH of the broken layer (05)
  Source:  the attack that applied the newest live Magnetic stack in that state (for one attack: that attack);
           its F, E^dot_ELE (combined Electricity mods included), A_sd, A_sd,ELE and SD_p of ELE
  T_brk = (0.03 · n · P_max / 6) · (1 + E^dot_ELE) · (1 + F)² · (1 + A_sd + A_sd,ELE)²
  It is an ELE proc of the target (counts for cap_ELE and n), created at the break time t_b and applied:
    breaking shot instance (P4a):        in P4b, after that instance's procs; tick m = 0 in P4c
    breaking group or hit in P2:          right after that group, inside P2; tick m = 0 lands before the next group
    breaking host hit (P4b) or group (P4c): right after it; tick m = 0 in P4c
  ticks: N_ELE = floor(6(1 + SD_p) + ε) at t_b + m, m ≥ 1 in P2, each one instance of T_brk, enemy side as an
  ELE tick at its time (it hits whatever layer is next: SH after an OG break, else HP)
  Chain: a break proc created by an OG break can break the shield with its own ticks; that creates a second break
         proc for the shield (with n_MAG as its tick saw it). At most one break proc per layer break
  No "+1", no crit, no body part, no base-damage bonus, no V_sd or M_tick; its ticks form their own group
```

Source: `W:Damage/Magnetic_Damage` lines 34-41: "a total of 3% of the target's maximum Shields/Overguard per Magnetic
status effect, to a maximum of 30% at full stacks, over a period of 6 seconds"; determined by the weapon or ability
that applied the Magnetic status; base-damage mods have no effect; status-damage mods apply twice (×3.61 at +90 %);
faction twice; Electricity mods apply even if combined; "Status duration mods are applied like normal and are a final
multiplier to the total damage"; `W:Status_Effect` line 343 (stun unaffected). Equal tick split, the floor tick count
(so `N/6` rather than exactly `1 + SD` as the total's factor), the absence of crit and "+1", "newest stack's source"
and n taken before the breaking instance's procs are Decisions (UNVERIFIED, OQ-308), as are the timing of break procs
created outside P4a and the chain rule (the source says only "on shield or overguard break", D-054). Example: 1000
shields, 10 stacks, +90 % status damage: 1083 total, 180.5 per tick (TV-03-11); P4c timing and the exclusion of the
breaking instance's own Magnetic proc: TV-03-24; source from the newest stack: TV-03-34.

---

## 14. Distinct-status count n

```text
E-3.12 Distinct-status count (definition; T-05 chooses the count source, 08 the EV expectation)
  present(s, σ):
    SLA, TOX, ELE, GAS   ≥ 1 live instance (lives on [t_p, t_p + D_s), §7.2)
    HEA                  a live episode (t_ign ≤ τ, ended in P3 at t_exp)
    BLA                  ≥ 1 live fuse
    COL                  n_COL ≥ 1 or Frozen
    IMP PUN RAD MAG VIR COR TAU   ≥ 1 live stack
    every s              also present while N_ext,s ≥ 1 (T-43; T-06 for VIR; §6.3)
    VOI KND LIF          live;      MIC   applied earlier in the window
  n(σ) = Σ_s [present(s, σ)]                       s over the 18 StatusTypes; KND and LIF exclusive → n ≤ 17
  For damage instance j of shot S:  n_j = n(σ_S^{<j}),   n_COL,j = n_COL(σ_S^{<j})   (Frozen: reported 10, E-3.11)
     σ_S^{<j} = σ_S with the procs of instances 0 .. j−1 of S applied for the CO-family reads only, in §7.1 P4b order
     and with the stack and presence rules of this document: insertion, replacement, discards (§5.1, §6.2, §6.4: a
     proc that is not delivered adds nothing), Frozen start, Blast detonation (clears BLA), and the mutual removal of
     KND and LIF (§8.9); no debuff takes effect on damage. Instance j's own procs never count.
     Readers of σ_S^{<j}: the CO-family sum C of 04 E-4.1 only (n_j; n_COL,j for Secondary Shiver and the Shattering
     Frost predicate n_COL ≥ 1; 04 OQ-433). Every debuff function (§8) reads σ_S.
  A DoT seed uses the n_j and n_COL,j of its inducing instance (E-4.7); ticks do not recount them except
     co_dot_mode = per_tick
```

Sources: `W:Condition_Overload_(Mechanic)` lines 162-185 (statuses with a damage-type equivalent including Void and
Tau, plus Lifted, Knockdown, Microwave; Lifted and Knockdown exclusive); `F:addmul` §1.2 (unique types, not stacks; any
source; the applying hit does not benefit; multishot pellets sequential); `W:Galvanized_Aptitude` §Notes example;
CD-04. Frozen is a state of COL. Crowd-control-only names of §4.2 never count; Stun is excluded although
`W:Status_Effect` lines 199-203 say it counts (the per-mechanic CO page wins, overview §1.4; OQ-310). The maximum is 17
with Tau (the Galvanized pages' 16 omit Tau; `F:addmul` §1.2). Secondary Shiver and Shattering Frost read n_COL or Cold
presence, not n (`04`), from the same σ_S^{<j} as n: `04` E-4.1 counts earlier pellets' Cold procs for them (the CO
rule of `F:addmul` §1.2 applied to the stack count; Decision, UNVERIFIED, `04` OQ-433; TV-03-39). The counting rule
is `04` E-4.2 (n_COL,j = min(cap_COL, N_pre + K_j), with N_pre the live Cold count before the shot and K_j the Cold
procs delivered by the instances before j); its EV expectation is `08` E-8.10, the sole owner (overview §7.2).

---

## 15. Worked example

Heat proc from a critical headshot, checked with `python3`. Inputs: B' = 100, A = 1.65 (Serration), no CO, G_F = G = 1,
b = 1, F = 0.30 (Bane), E^dot_HEA = 0.90 (Hellfire), A_sd = 0.90 (Rifle Elementalist), M_dot = 1; CDM0 = 2.0,
ΣCD_rel = 1.2 → CDM = Q(2.0)·2.2 = 2.0004885197·2.2 = 4.40107474; head hit with P_part = 3, ΣWP = ΣHS = 0, M_son = 1,
head crit h = 2, tier k = 1; SD = 0; first and only Heat proc on the target. Target: Grineer (φ_HEA = 1),
A0 = 1355 armor, no Corrosive, no Viral, health layer, a = 1.

1. `c_1 = 1 + 1 × (2 × 4.40107474 − 1) = 8.80214949`; `Part_dot = Part = 3`.
2. Seed `S_p = 100 × 2.65 × 1.30 × 8.80214949 × 3 = 344.5 × 26.40644846 = 9,097.021494`.
3. `X_p = 0.5 × 1.9 × 1.3 × 1.9 = 2.3465`.
4. Weapon-side pulse value `(9,097.021494 + 1) × 2.3465 = 21,348.507436` (compat a = 0: 21,346.160936; the superseded
   structure with c_k and Part_dot outside the seed gave 21,408.123668).
5. Strip: up-steps at t + 0.5 and t + 1.0 precede the pulse at t + 1 (P1 before P2): k_h = 2, armor
   `1355 × 0.5^(2/4) = 958.129689`, DR = 0.53613329, tick 9,902.861870. Pulses at t + 2 .. t + 6: k_h = 4, armor
   677.5, DR = 0.45083256, tick 11,723.905109 each.
6. Six pulses: **68,522.387417** on armored health. The episode ends at t + 6 (P3); down-steps at t + 7.5, 9, 10.5,
   12.

---

## 16. Edge cases

The "Where" column gives the section that specifies the rule, or ("out") the owner when the item is outside this
document.

| ID | Rule | Source | Where |
|---|---|---|---|
| EC-03-01 | Faction applies twice to every DoT and Blast hit; once to Heat procs that inherit a faction (tick-level factor replaced); twice on the Magnetic break proc | `F:status` §7 item 1; `W:Damage/Heat_Damage` §Heat Inherit; `W:Damage/Magnetic_Damage` | §9.1, §10.3, §13 |
| EC-03-02 | Status damage applies twice on the Magnetic break proc (×3.61 for +90 %), once elsewhere | `W:Damage/Magnetic_Damage` line 37 | §13 |
| EC-03-03 | Slash ticks are Cinematic: no faction type modifier, no armor, but they damage shields and overguard, and Viral and Magnetic apply | `W:Damage/Slash_Damage`; `W:Damage/Cinematic_Damage` | §9.5; routing `05` |
| EC-03-04 | Toxin ticks bypass shields (not Treasurer and Hound shields) but not overguard | `F:status` §4.7; `F:enemy` §4.2 | out: `05` (D-050) |
| EC-03-05 | Heat merges all procs into one tick per second, refreshes the whole pool, takes the longest duration, inherits the first source's Heat and faction bonus | `W:Damage/Heat_Damage` lines 63-90 | §10 |
| EC-03-06 | Heat strip ramps in 4 steps (0.5 s each × (1 + SD)), returns in 4 steps (1.5 s × (1 + SD)), is not sped up by more procs, multiplies with Corrosive and the ability/CP group | Heat page §Armor Stripping; `W:Status_Effect` line 325 | §10.4; E-15 `05` |
| EC-03-07 | Gas and Blast ignore component-element mods; Gas scales with literal Gas only; Blast with no elemental bonus | `W:Damage_over_Time` lines 172-176; Blast page line 56 | §9.3 |
| EC-03-08 | A forced primary-element proc (Toxic Lash) is scaled by that element's mods even when combined on the weapon | `W:Damage_over_Time` line 176 | §9.3; extra-hit DoT E-14 `04` |
| EC-03-09 | Electricity and Gas tick at once; their tick 0 sees a Viral proc of the same shot; Slash, Heat, Toxin start 1 s later | `W:Damage/Viral_Damage` line 36; tick tables | §7.1 P4c, E-3.6 |
| EC-03-10 | A hit is not boosted by its own Viral proc or by sibling pellets' procs; later shots of an instant burst are | Viral page lines 33-35 | §7.1, §7.2 |
| EC-03-11 | DoT ticks read Viral (and every target-side term) when they land; a Bleed of 35 deals 70 while one Viral stack lives | Viral page line 31 | §7.2; TV-03-14 |
| EC-03-12 | DoTs use the crit tier rolled by the inducing hit (yellow, orange, red), with crit quantization; the crit multiplier is part of the seed, so the "+1" is not scaled by it | `W:Damage/Calculation` lines 617, 632-635 | §9.1, §9.5 |
| EC-03-13 | Cold's crit-damage bonus is absolute, before tiers, not on AoE; +1.0 while Frozen; capped at 4 stacks (+0.25) on bosses and overguarded enemies (overguard: per view, §6.1) | Cold page lines 18-27 | §8.6, §6.1 |
| EC-03-14 | Frozen: 3 s × (1 + SD); no further Cold stacks or refresh (Primary Frostbite cannot extend it); 3 stacks remain after thaw; no shield recharge | Cold page lines 23-25 | §8.6 |
| EC-03-15 | Demolishers do not freeze at 10 Cold stacks (90 % slow) | `F:enemy` §9.2 (Demolisher page) | §8.6 (`no_freeze`, `05`; value UNVERIFIED) |
| EC-03-16 | Puncture's crit chance is flat, after mods, not on AoE or abilities; VIPs with overguard cap at 3 stacks | Puncture page; U36 | §8.3, §6.1 |
| EC-03-17 | Evensong applies 7 Puncture stacks per hit (beyond 5, +35 % crit chance); 5 on enemies capped at 4 and on overguarded enemies | `W:Evensong` lines 17, 42 | §6.1; D-305 |
| EC-03-18 | Magnetic multiplies damage to overguard as well as shields; Viral to health only (armored health included; on overguard it was a bug fixed in U31.6) | Magnetic, Viral, Overguard pages | §8.1 |
| EC-03-19 | Viral and Magnetic multipliers return to 1 when the last stack expires | Viral page | §8.1 (D-029) |
| EC-03-20 | Tau multiplies status chance (×(1 + 0.1n)); forced procs gain nothing | Tau page | §8.4 |
| EC-03-21 | Blast: host takes 0.3× per stack at expiry and (Blast page) at detonation; the Calculation page says detonated stacks are not settled individually (no host hits); others 3× per stack in one instance, no falloff, radius not modded; bosses detonate at 4 | Blast page lines 21, 23, 59; Damage/Calculation lines 644-645 | §11 (D-306, OQ-306) |
| EC-03-22 | Blast deals its target damage even with SD below −100 %; its AoE then happens only if the triggering hit kills | `W:Status_Effect` negative table; Blast page line 64 | §5.1, §11 |
| EC-03-23 | A killing blow that applies at least one Blast proc triggers the full AoE even with no prior stacks | Blast page line 65 | §11 (multi-target, not evaluated) |
| EC-03-24 | Xata's Whisper triggers off each Blast detonation (faction a third time, elemental bonuses re-applied) | Blast page §Notes | out: `04` (D-071) |
| EC-03-25 | SD ≤ −100 % nullifies duration effects and DoTs; instant effects happen | `W:Status_Effect` lines 356-420 (outdated 2016 source) | §5.1; OQ-307 |
| EC-03-26 | SD adds DoT ticks rounded down (Lasting Sting rank 9 = rank 10) | `W:Lasting_Sting` §Notes | E-3.1 |
| EC-03-27 | SD extends debuffs, Frozen and the Cold stacks after it, the Blast fuse, the Heat strip ramp and ramp-down, the Magnetic break DoT; not crowd control (stagger, panic, stun, Lifted) | `W:Status_Effect` lines 292-354; Lasting Sting; Lifted | §5 |
| EC-03-28 | More elemental damage does not lengthen procs | `W:Status_Effect` line 231 | §5 |
| EC-03-29 | Sonar spots boost the DoT of a proc from a hit on the spot (U44 rework; the DoT and Slash pages keep the older "initial hit only" text); Detect Vulnerability by analogy; the AoE parts of Electricity, Gas and Blast double-dip Sonar on other enemies (multi-target, not evaluated) | `W:Sonar` lines 25-26 (D-095); DoT page line 171; Slash page line 60; `W:Detect_Vulnerability` lines 50-52 | §9.1, §9.2; OQ-312 |
| EC-03-30 | Body-part and headshot multipliers of the inducing hit carry into the DoT; the Electricity chain and Gas cloud have their own 1× headshot | DoT page; Electricity line 48, Gas line 46 | §9.1, §12; OQ-025 |
| EC-03-31 | Beams: merged multishot beams sum damage and status chance on one tick, so DoT scales twice with multishot; forced procs once per tick | `W:Multishot` §Continuous Weapons | §9.1 (b); merging `02` |
| EC-03-32 | Stealth damage bonus, melee combo and sniper combo multipliers carry into the DoT | DoT page lines 177-180 | §9.2 |
| EC-03-33 | Finisher-damage mods do not affect Slash ticks | Slash page | §9.2; D-303 |
| EC-03-34 | "+1" accumulator per tick group, f32 intermediates; Heat, and same-pulse Electricity and Gas, add it once per group | Damage/Calculation lines 619-635 | §9.5 (f32 not emulated) |
| EC-03-35 | Only 10 Slash and Toxin tick numbers are displayed; all stacks deal damage | Slash page; U27.2 | display only; no cap |
| EC-03-36 | Combined elements remove their components' procs; innate secondary elements never combine | `F:status` §7 item 24 | out: `01`, `02` |
| EC-03-37 | Two innate primaries on Kuva/Tenet/Coda weapons combine by HCET, which changes which procs exist | `F:status` §7 item 25 | out: `01` |
| EC-03-38 | A ragdolled target turns an Impact roll into no proc (no renormalisation); immunity renormalises | Impact page; `W:Status_Effect` §Status Immunity Interactions | out: `02`; immunity at application §6.2 |
| EC-03-39 | Enemy-inflicted Bleed is 10 %, not 35 % | Slash page §Enemy Behavior | not applicable (do not reuse the constant) |
| EC-03-40 | Status-capped enemies: at most 4 stacks of any status (Acolytes 3 Impact; Liches Impact 6 in U27.3, now 5); the cap includes DoT instances | Bosses U27.3; Acolytes; CD-07 | §6.1; OQ-008 |
| EC-03-41 | Radiation capped at 4 on Liches, Acolytes, enemy Necramechs, Sisters, Hounds; overguarded enemies receive it but ignore it; Manics immune | Radiation page line 27 | §6.1, §8.5; Necramech immunity conflict `05` OQ-520 |
| EC-03-42 | Corrosive: Emerald shard +2/+3 max stacks per shard, also on status-capped enemies, only for the holder's own Corrosion; 14 stacks strip all armor; Hydroid's passive first stack 50 % | Corrosive page; Archon_Shard lines 351-354; Emerald shard lines 40-43 | §6.1, §8.2; OQ-317; Hydroid `05` E-5.12 |
| EC-03-43 | Gas cloud radius grows with stacks; host death keeps the cloud ticking on others | Gas page | §12 (multi-target) |
| EC-03-44 | Electricity hits all enemies within 3 m of the target; stun of the primary target only, not SD-scaled, bosses immune | Electricity page | §12 (multi-target) |
| EC-03-45 | Heat inherit: the first source's Heat elemental and faction bonus are used by later procs until Heat ends; abilities and companions without bonuses weaken a later weapon's Heat | Heat page lines 68-90 | §10.3; T-26 |
| EC-03-46 | Overguarded enemies receive status effects and their DoTs, ignoring crowd control; Cold capped at 4 | Overguard page; Cold page | §6.1; routing `05` |
| EC-03-47 | Status immunity to "all" (Objects, Alkonost, the `05` "Full Immunity" list) disables every proc and status damage; enemy Necramechs are not immune (`05` §4.4) | `F:status` §5; Damage/Object; `05` OQ-520 | §6.2 |
| EC-03-48 | Viral-status-immune enemies (Ambulas, Deimos Carnis/Rex, Genetrix, Jugulus, Leaping Thrasher, Therid, Saxum/Rex, Demolisher Boiler/Thrasher, Tusk Bolkor, Tusk Firbolg, Techrot Babau) | Viral page | data `05`; §6.2 |
| EC-03-49 | Heat panic and Electricity stun ignore SD; Heat panic re-triggers on overguard break | Heat page §Crowd Control | no damage effect |
| EC-03-50 | Impact 5 stacks cause a Big Stagger; mercy threshold +8 %/stack | Impact page; `W:Status_Effect` line 131 | no damage effect |
| EC-03-51 | Knockdown and Lifted count for CO and cannot coexist; Lifted lasts 1 s per combo tier (taken before the hit's consumption), is not SD-scaled, ends on a second Lifted proc (which ragdolls) or a non-melee hit | CO Mechanic lines 182-185; `W:Lifted` lines 6-11 | §8.9, §8.10 |
| EC-03-52 | Microwave (Nukor, Kuva Nukor) lasts forever and counts once | `W:Microwave` | §8.7 |
| EC-03-53 | Void: 3 s field; stacking undocumented; the separate `PT_VOID` status does not count | Void page; `W:Status_Effect` line 205 | §8.8 |
| EC-03-54 | Magnetic break: 3 % of max shield or overguard per stack (30 % at 10) over 6 s; base-damage mods no effect; Electricity mods combined or not apply | Magnetic page | §13 |
| EC-03-55 | Nightwatch Napalm Heat procs last only 3 s base (68 % fixed chance per blaze tick); the blaze is not a status DoT | `W:Nightwatch_Napalm` §Notes | §5 (D0 override); blaze `06` (D-074) |
| EC-03-56 | Bubonico alternate-fire explosion "+9 s pre-mods" status duration | `W:Status_Effect` line 695 | not modelled (no data; OQ-318) |
| EC-03-57 | Primary Debilitate's extra procs are separate instances: faction three times on their Heat, Electricity, Toxin DoTs | `W:Primary_Debilitate` §Notes line 31 | §9.8; D-073 (three times); seed OQ-313 |
| EC-03-58 | Hunter Munitions and Internal Bleeding Slash procs are normal Slash procs: stacks, n, and E-13 with the hit's crit and headshot | `W:Hunter_Munitions` §Notes | §9; rolls `02`; D-070 |
| EC-03-59 | Melee Afflictions adds stacks of every existing status with averaged proc damage | `W:Melee_Afflictions` §Damage Calculation | out: `04` (trigger and averaged seed); new stacks follow §6 |
| EC-03-60 | Secondary Cryogenic adds Cold stacks (SD of the secondary applies); Archon Vitality and Continuity add Heat or Corrosive procs from abilities | arcane and mod pages | out: `04`, `02`; stacks follow §6 |
| EC-03-61 | Heavy attacks of Claws, Dual Daggers, Nikanas, Two-Handed Nikanas, Rapiers, Scythes, Tonfas, Warfans and Whips force Bleed on the entire heavy attack; Daggers and Machetes only on parts of their heavy attacks | `W:Damage/Slash_Damage` line 73 | out: `07` (forced procs, full list) |
| EC-03-62 | Mecha Empowered applies twice on DoT, the second time only while the target is marked when the tick lands; Vauban passive applies on the tick only while the target is incapacitated at that tick, a second time (in the seed) if the proc was applied while incapacitated; target vulnerabilities once, at tick time | `W:Mecha_Empowered` lines 32-33; `W:Vauban/Abilities/Passive` lines 13-17; `F:buffs` §2, §9 | §9.4 (M_tick, OQ-319) |
| EC-03-63 | Electric Shield's field Electricity does not raise Electricity status damage; Valence Formation adds +200 % to its element's DoT (not Blast) | `F:buffs` §1.12, §8, §9 | §9.3 |
| EC-03-64 | Emerald shard's Toxin status damage also scales Toxic Lash's procs; Conductive Sphere gives Electricity status damage | `F:buffs` §5, §1.12 | §9.4 |
| EC-03-65 | Sniper combo counts only direct hits, not AoE or DoT; Relentless Combination gives combo chance on Slash ticks | `F:weapon` §5.4; `F:melee` §3.1 | out: `06`, `07` |
| EC-03-66 | Tysis and Stinger DoTs ignore CO; the Pox cloud recalculates per tick | CO Mechanic catalogue | out: `04` (E-4.7) |
| EC-03-67 | Archons: 50 % reduction to status damage, 20 % to all damage (`W:Archon` line 32); 4-stack cap of any status, stated for Archon Boreal (`W:Archon_Boreal` line 43) | `W:Archon` line 32; `W:Archon_Boreal` line 43 | caps §6.1; DR `05` |
| EC-03-68 | DoTs can be ended early by cleanses or invulnerability; DoT "detonation" abilities (Expedite Suffering, Final Verse Tragedy, Divine Retribution) and Sickening Pulse duplication. The Harmony heavy attack's DoT detonation is a weapon effect and is in scope | `W:Damage_over_Time` lines 186-191; `W:Harmony` lines 11-12 | abilities out of scope; Harmony §9.7 |
| EC-03-69 | Players: Cold 10 % per stack, max 4, never frozen; overguard gives full status immunity | Cold page; Overguard page | out of scope (player side) |
| EC-03-70 | Viral and Magnetic peak at 4.25× (U27.2 note "up to 4.5x" is outdated) | Viral page; `F:status` §4.13 | §8.1 |
| EC-03-71 | The inspiration's displayed status chance (used by its Melee Duplicate re-roll) assumes 10 Tau stacks for Tau attacks (`JS L8893-8901`; `I:status` Q32); the game reads live stacks | `I:stat` §10; Tau page | engine: live n_TAU (§8.4); duplicate rule `04` |
| EC-03-72 | Inspiration-only weapon `unique` keys `addPunctureStatus` (adds Puncture stacks to the current hit only) and `decr_armor_by_punc` (armor strip per Puncture stack) have no game source and no data in the mod table | `I:queue` §3.1, §10; `I:status` open questions | not modelled; add only with a game source |
| EC-03-73 | The last Slash or Toxin tick lands at the instance's expiry time (6 s at SD 0); expiry ends the instance's presence but never cancels its tick | `W:Status_Effect` tick table; Slash page lines 44-50 | §7.2, E-3.6 |
| EC-03-74 | Heat armor strip persists through the 6 s ramp-down after the episode ends, with no Heat live | Heat page line 98 | §8, E-3.8 (OQ-322) |
| EC-03-75 | Ash's passive: Slash procs +25 % status damage and +50 % duration, additive with SD mods (9 ticks at SD 0) | `W:Ash/Abilities` lines 3-6 | §5 (SD_s), §9.4 |
| EC-03-76 | Burning Hate: status-damage vulnerability for procs created after the Hate hit, from any source, persisting after Heat ends; additive with Pageflight | `W:Burning_Hate` lines 28-34 | §9.4 (V_sd) |
| EC-03-77 | Harmony heavy attacks deal all remaining DoT ticks at once (its own procs included) and remove the statuses | `W:Harmony` lines 11-12, 29 | §9.7 |
| EC-03-78 | Ragdolled enemies receive no Impact status; bosses, Acolytes and overguarded units cannot be ragdolled; a second Lifted proc ragdolls | `W:Ragdoll` lines 4-5; `W:Lifted` line 10 | §8.10 |
| EC-03-79 | Secondary Fortifier's overguard bonus also applies to DoTs on overguard ("dynamically applied", not inheritable) | `W:Secondary_Fortifier` lines 37-39 | §9.4 (M_OG,p, OQ-327) |

---

## 17. Inspiration differences

The overview's register rows owned or co-owned by `03`, restated with the engine behaviour, followed by new rows
(D-301 to D-306) in the same layout. Toggle column as in overview §5.1.

| ID | Topic | Inspiration behaviour | Game behaviour (source) | Engine (game profile) | Toggle |
|---|---|---|---|---|---|
| D-015 | Cold crit damage | `+0.05·n` (+0.5 more at 10) added to the final multiplier after tier and headshot, only on crits, also on AoE; no Frozen state, no 4-stack cap on bosses or overguard except via `maxCountProc` (`JS L10821-10825`; `I:status` §5.4) | `cold_cd` before tiers, not AoE, Frozen +1.0, cap 4 on bosses and overguard (§8.6) | E-3.3, E-3.11, §6.1; compat reproduces the inspiration's term | Compat |
| D-025 | DoT "+1" | none (`getTickStatusDmg`, `JS L9154-9204`) | `(Σ seeds + 1)·C·M` per tick group, with the crit multiplier inside the seeds, so the "+1" is not multiplied by c_k (`W:Damage/Calculation` lines 619-635; line 635: `(80.0195389 + 1) × 0.35 = 28.3568382` for a critical seed) (§9.5) | a = 1 with E-3.5; c_k and Part_dot inside the seed, as overview E-13 now fixes it (adopted from OQ-301; Part_dot placement by analogy, UNVERIFIED) | Compat (a = 0) |
| D-026 | Heat armor strip | flat 50 % while any Heat stack exists (`JS L10815`) | 4-step ramp up and down, SD-scaled (§10.4) | E-3.8 | Compat |
| D-027 | Heat stacking | each Heat proc on its own timer in DPS; TTK uses a shared refreshing timer (`JS L11139-11143`, `L13690-13703`) | one episode, pool refreshed by every proc, longest duration (§10.1) | E-3.7 in both evaluators | Compat |
| D-028 | Heat inherit | custom `heatInherit` replaces the Heat elemental bonus; faction always squared (`getTickStatusDmg`, `JS L9154-9204`) | first source's Heat and faction bonus for later procs; inherited faction once (§10.3) | §10.3: own later procs inherit from p₁; with T-26 set, E = v and F_tick = F_inh (`heatInheritFaction`, default 0) | User (T-26) + Compat (§10.3 flag: F_tick = F_i, E = v or own) |
| D-029 | Viral and Magnetic after expiry | last multiplier sticks for the rest of the queue (`JS L10320-10321`, `L10779-10804`) | 1 without stacks | E-3.3 | Compat |
| D-030 | Own-hit and same-shot timing | a hit sees its own procs and earlier pellets' procs for every debuff (`timelineStatus`, `JS L10093-10136`) | procs apply after the shot's damage; CO counts earlier pellets | §7.1 | Compat |
| D-031 | Target-side terms on ticks | Viral, Magnetic, armor strip frozen at the inducing hit (`JS L10883-11363`; `I:enemy` §9.2) | read when each tick lands | §7.2 | Compat |
| D-032 | Puncture duration | 6 s (`JS L5547`) | 10 s | table 4.1 | Compat |
| D-033 | Impact duration | 1 s (`JS L5545`) | 6 s per stack (UNVERIFIED: the general table's "1 second" is the flinch) | table 4.1 | — |
| D-034 | Blast | unlimited stacks, no detonation, damage `0.3·` × `parseInt(1.5·(1 + SD))` × (1 + Blast element mods), never in TTK (`JS L5571`, `L9177`, `L11291-11361`, `L13669`) | 1.5 s × (1 + SD) fuse, one 0.3·H hit, detonation at 10 (4), no elemental scaling (§11) | E-3.9 | Compat |
| D-035 | Gas | one Gas proc per hit credited `gasStacksOnTarget × tick`; drops the non-crit headshot factor; extra faction in the non-crit path; TTK skips new clouds beyond 10 instead of replacing (`JS L11219-11290`, `L13676-13681`) | own timers, N ticks at t + 0 .., cap 10 with replacement, consolidation (§12) | §12, E-3.5 | — |
| D-036 | Duration rounding | `parseInt(D0·(1 + SD))` for every status, debuffs, Blast "ticks" and Toxic Lash included (`JS L9085-9093`) | ticks floored; debuff rounding unknown | E-3.1 | Compat |
| D-037 | Toxic Lash DoT duration | fixed 7.5 s (7 ticks credited, 8 in TTK) (`JS L5557-5563`, `L13706-13713`) | `floor(6(1 + SD))` Toxin ticks; Saryn's passive +25 % SD on every status of Saryn's weapons (`W:Saryn/Abilities/Passive` line 1) | Saryn's passive is one SD source (+0.25) for every proc of the build when Saryn is the frame; T-16 implies it, counted once (§5), as overview D-037 now states (adopted from OQ-333) | — |
| D-041 | Mecha Empowered and Eclipse on DoT | Mecha squared on elemental DoT, once on Slash; Eclipse once on elemental DoT, not on Slash; Mecha unconditional | Mecha "applied each time damage is calculated against a marked enemy", twice on every DoT; Eclipse once on every DoT (in Y) | §9.2, §9.4: first dip in G when the hit saw a mark, second dip M_tick while the target is marked at the tick (OQ-319) | — |
| D-053 | Status cap of capped enemies | `maxCountProc` drops same-type procs beyond the cap inside one hit and caps live stacks, never DoT damage; forced procs bypass the per-hit check (`JS L10011-10031`, `L10119-10127`) | 4 stacks of any status (U27.3) | E-3.2 and §6.2 for every status incl. DoT instances and the Heat pool | Compat |
| D-054 | Magnetic break proc | not modelled | forced Electricity proc on SH/OG break (§13) | E-3.10 in layered passes | — |
| D-069 | Per-status status damage | Emerald Toxin etc. added to the product `zb`, outside its other factors (`JS L11003`, `L11084`) | additive with Elementalist in one bucket | `(1 + A_sd + A_sd,s)` | — |
| D-070 | Hunter Munitions proc | DoT credited, never in the timeline (`JS L10761-10766`) | a Slash status | counts for stacks and n | — |
| D-073 | Primary Debilitate faction | faction a third time on its procs (`JS L11459`) | three times (`W:Primary_Debilitate` §Notes line 31; `W:Extra_Hit` lines 138-139) | three times, as overview D-073 now states; seed `S_i·(1 + F_i)`, no "+1" (§9.8; seed UNVERIFIED, OQ-313) | — |
| D-074 | Nightwatch Napalm | DoT base 75 overwritten by 150, progenitor scaling, credited at hit, halves armor (`JS L10409-10441`) | blaze 150 Heat per tick, 6 ticks from 0 s, no crit; its Heat procs last 3 s base (`W:Nightwatch_Napalm`) | Heat procs D0 = 3 s (EC-03-55); blaze per `06` | — |
| D-095 | Sonar on DoT | no Sonar input (no Sonar code in the bundle) | Sonar spots carry into the DoT since the U44 rework (`W:Sonar` lines 25-26, revision 2813083); the DoT and Slash pages keep the older "initial hit only" text | `Part_dot = Part` (E-3.4); Detect Vulnerability by analogy, UNVERIFIED (OQ-312) | — |
| D-301 | Gas elemental bonus | Leaded Gas stored as `addGas 3` plus `status_damage_gas 3`, added to the status-damage sum: tick × `(1 + A_sd + 3)` (`JS L1824-1830`, `L10462`) | Gas Damage Bonus is its own factor: `0.5 × MB × (1 + 3) × (1 + F) × (1 + A_sd)` (`W:Damage/Gas_Damage` example 2) | E^dot_GAS (§9.3); compat moves literal Gas bonuses into A_sd,GAS | Compat |
| D-302 | Microwave duration | 1 s (`JS L5578`) | infinite (`W:Microwave`) | ∞ (§8.7) | — |
| D-303 | Finisher mods on Slash ticks | attacks flagged `enableFinisherModsWithSlash` multiply Slash ticks by `1 + finisherDmg` (`JS L10471-10473`) | finisher mods do not affect Cinematic Slash ticks (`W:Damage/Slash_Damage`) | never | Compat |
| D-304 | External Viral on capped enemies | with no own Viral stack the option's N is used uncapped (up to 10 on a 4-capped boss); with own stacks capped at `maxCountProc` only above 10 (`JS L10782-10798`) | the enemy cap applies to all Viral stacks | `min(N_ext + n_own, cap_VIR)` (§6.3, T-06) | Compat |
| D-305 | Evensong Puncture cap | 7 forced Puncture procs per hit, but live stacks capped at 5 (`JS L12851-12853`, `L10119-10127`): +25 % crit chance at most | 7 stacks (+35 %), 5 on capped and overguarded enemies (`W:Evensong`) | E-3.2 Evensong rule | Compat |
| D-306 | Blast host damage at detonation | no detonation at all (D-034) | the wiki disagrees with itself: `W:Damage/Blast_Damage` lines 21, 23, 59 (edited 2026-11-03): the host takes "10 hits of 30% base damage" and is not hit by the radial; `W:Damage/Calculation` lines 644-645 (edited 2026-11-04): detonated stacks "are not settled individually but are combined into a single 5-meter radial", and `max(1, 0.3 × H)` is for a stack that "expires on its own" | Blast page: one `H_det` per live stack to the host (§11); UNVERIFIED (OQ-306); the alternative gives the host nothing at detonation | — |

Rows of other owners that touch status damage, applied as their owners define: D-038 (innate DR once), D-039 (no ×3 on
shields), D-040 (type modifier once on armor) — `05`; D-086 (landed vs credited DoT), D-089 (one event model for DPS
and TTK, SD honoured, Blast included) — `08`; D-071 (Xata on Blast detonation) — `04`.

---

## 18. Open questions

Overview questions this document depends on, with its current decisions: OQ-005 (tick and debuff rounding: E-3.1),
OQ-006 (Heat faction on later procs: §10.3), OQ-007 (Blast elemental scaling: none), OQ-008 (status cap on DoTs: yes,
§6), OQ-017 (Cold and Puncture into DoT: yes, through c_k), OQ-023 ("+1": on), OQ-025 (Electricity and Gas headshot:
§12), OQ-028 (sibling pellets and debuffs: no, §7.1).

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-301 | (Crit part settled 2026-11-24: overview E-13 adopted `seed = Y·(1 + F)·G·b·c_k·Part_dot`, as `W:Damage/Calculation` lines 632-635 require.) Remaining: (1) is the body-part factor also inside the seed (by analogy with crit; no page has a headshot example with the "+1")? (2) when the members of a consolidated group differ in X (A_sd, A_sd,s, V_sd; E^dot_s and F_tick in ELE and GAS groups of several attacks), which X multiplies the one accumulator? | (1) yes; (2) the oldest proc's (E-3.5) | (1) a low-damage Slash proc from a non-crit headshot (seed 40, Part 3): 42.35 per tick (Part in the seed) vs 43.05 (outside); (2) two Electricity procs landing together from weapons with different status-damage mods, read the merged number |
| OQ-302 | Heat pulse grid: anchored to the first ignition, or restarted by refreshes? Does a joining proc contribute to a pulse less than 1 s after it? | grid anchored at ignition; joins from the next pulse | Heat proc at t = 0 and a second at t = 0.5 with distinct seeds; read the damage numbers' times and values |
| OQ-303 | Heat strip: does re-ignition during the ramp-down continue from the current step? Which SD sets the step interval when sources differ? | continue from the current step; SD of the longest-duration proc | armor readout (enemy info) of a level-locked Simulacrum target, re-igniting 3 s after a Heat episode ends; mixed-SD sources |
| OQ-304 | Does the 4-stack cap of status-capped enemies limit Heat's pool? | yes, oldest leaves | Heat tick values on an Acolyte after 1 to 8 Heat procs within 6 s |
| OQ-305 | Frozen (E-3.11): are all 10 Cold stacks removed at the freeze? What n_COL do CO-type bonuses and Secondary Shiver see while Frozen (10, 0, or the live count of kept stacks)? Do the 3 stacks left after thaw have fresh timers or the timers of 3 kept stacks? What `cold_cd` applies at 10 stacks on enemies that cannot freeze? | removed; 10; fresh; 0.50 | Secondary Shiver damage numbers on a Frozen target (+4.50 vs 0); the Cold icon count during and after Frozen, and the expiry times of the 3 thaw stacks relative to the original procs; crit numbers on a Demolisher with 10 Cold stacks |
| OQ-306 | Blast: (1) host damage at detonation: one 0.3·H hit per stack (Blast page) or nothing, the stacks being consumed into the radial (Calculation page; D-306)? (2) Is H the modded-base product of the Blast page or the whole hit damage? (3) Is the host's detonation damage one instance or one per stack? (4) Is `max(1, 0.3·H)` real (sourced only for natural expiry), and is it applied on the weapon side before armor, and to detonation hits? | (1) one hit per stack; (2) modded-base product; (3) one per stack; (4) yes, weapon side, also at detonation | (1) on a non-boss target, count the host's Blast damage numbers when the 10th stack lands: ten 0.3·H numbers, none, or one 3·ΣH number; (2) Blast proc numbers from a weapon with large elemental mods (H differs strongly); (4) natural expiry and detonation on a heavily armored target at very low damage (a 1 before armor shows as < 1 after) |
| OQ-307 | Electricity and Gas tick count at fractional SD (`floor(6(1+SD))` vs ticks at 0 .. ceil(D) − 1) and at negative SD (2016 table: Tesla Chain still occurs) | floor; nullified at SD ≤ −1 | count Electricity ticks with +55 % SD (9 vs 10) and with a −100 % SD riven |
| OQ-308 | Magnetic break proc: equal split into ticks? total factor `N/6` or exactly `1 + SD`? crit? "+1"? V_sd and M_tick? which stack's source? n before or after the breaking hit's own Magnetic proc? When a P2 tick, a Heat pulse or a Blast hit breaks the layer, when is the proc applied and when does its first tick land? Can a break proc's ticks trigger a second break proc (OG break, then shield break)? | equal split; N/6; no crit; no +1; neither; newest stack's source; before; right after the breaking group, first tick at once; yes | break a 1000-shield Corpus unit with 10 Magnetic stacks and known Electricity and status-damage mods; read the six tick numbers; break a shield with a Slash tick and read the first Electricity number's time; break an Eximus's overguard with a break proc pending against a shielded target |
| OQ-309 | Knockdown duration; Lifted `M(c)` (`W:Lifted` line 6) or `1 + M(c)` (`W:Melee` line 690 and the 4× / 5 s caption at line 684); M(c) taken before or after the hit's combo consumption; do DoT ticks and Blast hits end Lifted ("any non-melee attack" does); do Knockdown and Lifted count for CO on overguarded enemies? | 3 s; M(c) before consumption; no; yes | CO damage numbers immediately and 2, 3, 4 s after a forced Knockdown or Lifted; Lifted duration after a heavy slam at a known combo tier; a Slash tick on a Lifted target; the same on an Eximus with overguard |
| OQ-310 | Does Stun count for CO (Status_Effect page) or not (CO Mechanic page)? Do weapon and stance data use "Stagger" for Impact procs? Evidence for: `W:Damage/Impact_Damage` line 22 ("The status effect of Impact damage is Stagger") and the status-duration table row "Stagger, 6 seconds" (`W:Status_Effect` line 303). Evidence against: weapon data name Impact procs "Impact" (about 350 entries) against 7 "Stagger", and one attack lists `{ "Impact", "Stagger" }` together; stance data use "Knockback" (239) for Impact and "Stagger" (48) separately | no; no ("Stagger" is CC only) | CO damage numbers after a forced Stun; Impact-icon presence and a CO count after a weapon attack whose data lists only "Stagger" (Strun Wraith or Vectis Prime normal attack, Opticor Vandal alt-fire; `Weapons_data_primary.lua` lines 11866, 13962, 8546) and after a "Stagger" stance hit |
| OQ-311 | Void: stacks and SD scaling; Tau: SD scaling, and whether a Tau weapon's own stacks raise its own status chance. For self-application: the in-game text "all subsequent attacks" (`W:Damage/Tau_Damage` line 8) and the inspiration (`JS L9989-9996`); against: "from all other status sources" (`W:Damage/Tau_Damage` line 13) and "from all other status effects" (`W:Status_Effect` line 226) | one refreshing instance, SD scales; Tau SD scales and self-applies (follows the inspiration and the in-game text) | status-icon timers with a Lasting Sting family mod; Haalvu proc counts against 0 and 10 own Tau stacks |
| OQ-312 | (Sonar settled 2026-11-24: overview D-095 adopts `W:Sonar` lines 25-26, DoTs inherit Sonar spots.) Remaining: do Detect Vulnerability spots carry into the DoT too (by analogy: its spots became weak points "so it matches Banshee's updated Sonar", `W:Detect_Vulnerability` lines 50-52)? Does the host's own Electricity or Gas tick take a second Sonar dip (`W:Sonar` line 26 describes the double dip for AoE DoTs on hit body parts)? | yes; no (the host tick uses Part_dot only, §12) | Slash tick numbers from a hit on a Helios-marked weak point vs the same part unmarked; Electricity tick numbers on the host after a hit on a Sonar spot |
| OQ-313 | (Faction count settled 2026-11-24: overview D-073 says three times.) Remaining: what seeds a Primary Debilitate proc, whose extra hit deals 0 damage? | `S_p = S_i·(1 + F_i)` from the inducing instance, X_p of its own type, no "+1" (§9.8) | Toxin tick of a Debilitate-added proc vs an ordinary Toxin proc of the same hit, with and without Primed Bane (ratio `(1 + F)` expected; TV-03-40) |
| OQ-314 | Puncture cap on non-VIP overguarded enemies (Evensong page implies 4); Kuva Lich Impact cap after U33.6 | 5; 5 | Puncture stack count on an Eximus with overguard after many procs; Impact stacks on a Lich |
| OQ-315 | (Settled 2026-11-24: overview §6.3 added `heatInheritFaction`, F_inh, default 0, and D-028's compat flag.) T-26 `heatInherit`: should it carry the external source's faction bonus? | F_tick,p = F_inh with T-26 set (§10.3) | none needed (UI semantics); the game rule is sourced (`W:Damage/Heat_Damage` lines 84-86) |
| OQ-316 | Which non-mod elemental sources scale their element's DoT: Violet shard Electricity, riven or spliced Gas, custom `heatAdd`, Fireball Frenzy, Conductive Sphere field Electricity? | all yes except buffing-field elements | Electricity tick with and without a Violet shard; Gas tick with a spliced-Gas riven |
| OQ-317 | Do several Emerald shards add their Corrosive cap bonuses (14-stack text) and do they raise a status-capped enemy's cap of 4 to 6 or 7 per shard? | additive; yes | count Corrosive stacks with two Emerald shards; on an Archon |
| OQ-318 | Bubonico alternate-fire "+9 s pre-mods" status duration: what is it added to? | not modelled | Bubonico alt-fire proc durations in game |
| OQ-319 | Vauban passive and Mecha Empowered on status damage: does each damage calculation (the proc-creating hit and each tick) test the condition, so that a proc applied while incapacitated keeps the seed's ×1.25 after incapacitation ends (this model), or is the whole bonus lost then (the alternative reading of `W:Vauban/Abilities/Passive` line 17)? Same for the Mecha mark. `04` owns the "marked" and "incapacitated" predicates and is notified here (`F:buffs` §1.19 dropped the "lost" sentence) | per-calculation: seed dip from the hit, tick dip M_tick at each tick | Slash ticks on a Vauban-stunned enemy before, during and after the stun, for procs applied before and during it (ratios 1, 1.25, 1.5625); the same with a Mecha mark that expires |
| OQ-320 | Do Sisters of Parvos and Technocyte Coda have a global cap of 4 for every status (no cached page says so; the U27.3 note covers Kuva Liches only)? | yes (as `05` data) | Slash and Viral stack counts on a Sister and on a Technocyte Coda after many procs |
| OQ-321 | (Modelling settled 2026-11-24: overview §4.1 adopted the per-view split.) Overguard-dependent caps (Cold 4, no Frozen, VIP Puncture 3) and the overguard ragdoll immunity in the non-depleting views and EV | `overguard` view `og_present = true` (spawn state), `health`, `armor`, `shield` views `false`, each view its own timeline in MC and EV; live OG in the layered pass (E-3.2) | the game rule itself is sourced (Cold 4 while overguard is present, `W:Damage/Cold_Damage` line 27, U36); `05` OQ-529's test (Cold stack count on an Eximus before and after its overguard breaks) confirms the layered rule |
| OQ-322 | (Symbol settled 2026-11-24: overview §2.4 and E-15 now write `heat_strip(k_h)`.) Heat strip after the episode: the strip persists through the 6 s ramp-down with no Heat live (`W:Damage/Heat_Damage` line 98); `05` E-5.10 is 0 only for Heat-immune targets or k_h = 0, as §8 requires | no presence gate (§8, E-3.8) | armor readout of a Simulacrum target during the 6 s after a Heat proc ends |
| OQ-323 | (Settled 2026-11-24: overview §2.4, §3.1a and CD-06 carry `status_dur_type[s]`.) Per-type status duration: Ash's passive (+50 % Slash duration, additive with SD) needs a per-type bucket, with `D_s = D0_s·(1 + SD + SD_s)` and `N_s = floor(6(1 + SD + SD_s) + ε)` | SD_s defined here (§5) | Slash tick count with Ash at SD 0 (9 expected) |
| OQ-324 | Status-damage vulnerability V_sd: are Burning Hate, Pageflight and the custom `vuln_status_damage` one additive group (Burning Hate and Pageflight are, per `W:Burning_Hate` line 34; the inspiration sums Burning Hate and the custom stat into one `(1 + md)`, `JS L10467-10469`, `L10498-10502`)? Does V_sd apply to the Magnetic break proc? Overview §6.3 and `04` §10 put the custom stat in V_sd. | one additive group, snapshotted at creation; not on the break proc | Slash tick with Burning Hate and Pageflight together vs each alone |
| OQ-325 | Harmony DoT detonation (E-3.14): one damage instance per DoT instance or one per status or one in total? "+1" once per remaining tick or once per instance? Remaining Heat pulses valued with the current pool? Do the heavy's own ELE and GAS tick 0 land before the detonation? Are Magnetic-break instances detonated? | per instance; per remaining tick; current pool; yes; yes | Harmony heavy on a target with one Slash and one Heat proc at known values: count and read the detonation numbers |
| OQ-326 | Ragdoll state (§8.10): duration, and whether it removes Impact stacks or Knockdown (ownership settled 2026-11-24: overview §7.1 and §7.2 make `03` the owner, `07` OQ-734) | 3 s; no | time from a Ragdoll stance hit to the end of the ragdoll; Impact procs on the target during that time |
| OQ-327 | Does M_OG (Secondary Fortifier) multiply DoT ticks and Blast hits that land on overguard? (Overview E-13 now applies M_OG,p on every type, Slash included, as `05` E-5.17 does; only the game question remains.) | yes, the proc's own M_OG,p, not inherited by Heat procs | Slash tick numbers on an Eximus's overguard with and without Secondary Fortifier on the secondary that applied the proc |
| OQ-328 | Electricity and Gas consolidation: "same pulse" = identical landing time (this model) or a shared per-target pulse clock as Heat has? Are Magnetic-break instances consolidated with ordinary Electricity ticks? | identical time; not consolidated | two Electricity procs 0.5 s apart: one damage number per second or two; a break proc on a target with live Electricity procs |
| OQ-329 | Do procs discarded for immunity or nullified by SD ≤ −1 fire on-proc triggers (status-triggered arcanes and mods)? | no (not delivered, §6.4) | Primary Blight stacks from Toxin procs with a −100 % SD riven |
| OQ-330 | Do range falloff Φ(d) and bow charge scaling (G_F members) carry into DoT seeds? | yes, through Y | Slash tick from a shot inside vs beyond the falloff range; from a partially charged bow shot |
| OQ-331 | Does Gas replacement at the cap cancel the replaced cloud's pending ticks? Does a cap that drops (overguard returning) remove existing stacks? | yes; no | Gas tick numbers after an 11th proc within 6 s (TV-03-22: 62 vs 66 ticks) |
| OQ-332 | Heat procs from extra hits (E-14): does Heat inherit override E-14's tick-level elemental and faction factors when the proc joins an episode? | yes (inherit wins) | Demonium Heat ticks on a target first ignited by a weapon with Heat mods vs a fresh target |
| OQ-333 | (Settled 2026-11-24: overview D-037 and T-16 adopted it.) Saryn's passive applies to every status of Saryn's weapons, not only to Toxic Lash procs | +0.25 SD on every proc of the build when Saryn is the frame; T-16 implies it (§5) | none needed (sourced by `W:Saryn/Abilities/Passive` line 1) |
| OQ-334 | Event order of derived instances: radial child instances of the same shot after all its pellets, as instances of S (n_j, no sibling debuffs)? | yes | shotgun with an explosive child attack on a fresh target: CO numbers of the explosion vs the pellets |
| OQ-335 | (Settled 2026-11-24: the overview adopted (1) to (5), and §3.8 step 6 now prints 343.787408, the k = 1 tick with c_k in the seed.) Overview edits proposed by this document: (1) `punc_cc(n)` range n ≤ 7 (Evensong), not n ≤ 5; (2) D-028 Toggle column "User (T-26) + Compat" with the compat flag of §10.3; (3) E-13's M_dot split into V_sd (snapshotted) and M_tick (tick time), and its Slash OG path (OQ-327); (4) `heat_strip(k_h)` (OQ-322); (5) `status_dur_type[s]` (OQ-323); (6) overview §3.8 step 6 prints 345.632502 for the k = 1 Slash tick, which matches neither the rounded CDM 4.4010749 (345.632503) nor the full-precision CDM (345.632491, TV-03-01) | adopted | none |
| OQ-336 | External statuses (T-43, §6.3): with external Cold, does Frozen remove the external stacks and how many own stacks remain after thaw? With external Blast, do external stacks count toward the detonation threshold and stay afterwards? With external Heat, do own Heat procs stay in the pool for the whole window (the external source keeps refreshing it)? | Frozen removes own stacks only, `min(3, cap_COL − N_ext,COL)` own thaw stacks; external Blast counts and stays; own Heat procs stay until the window ends, k_h = 4 | model choices for a user input (overview T-43 calls its per-status rules Decisions); a game check needs a second player or an ability applying the status continuously: Heat tick values of one own proc 7 s after it was applied while a teammate keeps the target burning |

---

## 19. Test vectors

All values computed with `python3` (f64; Q(·) per E-4 in numpy float32). Tolerance for the engine: 1e-9 relative.
"Weapon side" means before E-12's enemy side. a = 1 unless stated; compat values use a = 0.

**TV-03-01 Slash tick (E-13 single proc, overview §3.8 step 6).** Y = 35 × 2.65 = 92.75, F = 0.55, G = b = 1,
A_sd = 0, Part_dot = 1, M_dot = 1, SD = 0. Seed at k = 0: S = 143.7625; X = 0.35 × 1.55 = 0.5425. Tick k = 0:
**78.533656** (a = 0: 77.991156). Tick k = 1, CDM computed by E-4 at full precision, `Q(2.0)·2.2 =
2.000488519668579 × 2.2 = 4.401074743270875`, seed `143.7625 × 4.401074743270875 = 632.709508`: `(632.709508 + 1) ×
0.5425` =
**343.787408** (E-13 and E-3.4: c_k inside the seed, "+1" not multiplied by it; overview §3.8 step 6 prints the same).
For reference only: the superseded structure (c_k outside the seed) gave 345.632491 (345.632503 with CDM injected as
the rounded 4.4010749). Ticks at t + 1 .. t + 6 (N = 6). Cinematic: lands on health unchanged by armor and type
modifier.

**TV-03-02 Toxin tick with faction (`W:Damage/Calculation` line 628).** Y = 40, F = 0.55, E^dot_TOX = 2.25, A_sd = 0,
k = 0: S = 62, tick = (62 + 1) × 0.5 × 3.25 × 1.55 = **158.68125**.

**TV-03-03 Per-type wiki examples (no "+1" on the pages).** Modded base S = 100 × 2.65 × 1.3 = 344.5, F = 0.3,
A_sd = 0.9:
- Heat, E^dot = 0.9: a = 0 **808.369250** (page 808.37); a = 1 **810.715750**.
- Slash: a = 0 **297.820250** (page 297.82); a = 1 **298.684750**.
- Gas (Vesper 77): S = 180 × 3.2 × 1.3 = 748.8, F = 0.3, A_sd = 0.9, E^dot_GAS = 3.0 (Leaded Gas): a = 0
  **3699.072** (page 3699.07); a = 1 **3704.012**. With Scorch + Pathogen Rounds instead (E^dot_GAS = 0): a = 0
  **924.768**. Inspiration form (D-301) `0.5 × 748.8 × 1.3 × (1 + 0.9 + 3)` = 2384.928.

**TV-03-04 Heat pool consolidation (`W:Damage/Calculation` line 633).** Two Heat procs in one episode, both seed 40
(Y = 40, F = 0), E^dot_HEA = 0.6, A_sd = 0; proc A k = 0, proc B k = 1 with CDM0 = 2.0 (Q = 2.0004885197,
ΣCD = 0). S_A = 40, S_B = 40 × 2.0004885197 = 80.019541 (crit in the seed), X_A = X_B = 0.8. Pulse after both
joined: (40 + 80.019541 + 1) × 0.8 = **96.815633** (page 96.8156357), whichever proc is older (the superseded
structure gave 97.616023 with B older). A alone: (40 + 1) × 0.8 = **32.8** (page Supra Vandal example).

**TV-03-05 Gas same-pulse consolidation (`W:Damage/Calculation` line 632).** Nine Gas procs from one shot, seed 40
each, no bonuses (X = 0.5), six critical with Q(2.0) (critical seeds 80.019541): (3 × 40 + 6 × 80.019541 + 1) × 0.5 =
**300.558622** (page 300.558594 in f32), whichever proc is oldest (the superseded structure gave 301.058867 with a
critical proc oldest). Each pulse m = 0 .. 5 has this value (all nine land together).

**TV-03-06 Heat inherit (`W:Damage/Heat_Damage` §Heat Inherit example).** Panthera: Y = 100, F = 0.55, E^dot = 0.9;
Silva & Aegis Prime: Y = 318, F = 0, E^dot = 0; A_sd = 0, k = 0, both procs live.
- Panthera first: Panthera 0.5 × 155 × 1.9 × 1.55 = 228.2375/s; Silva inherits E = 0.9, F_tick = 0.55:
  0.5 × 318 × 1.9 × 1.55 = 468.255/s; a = 0 total **696.4925** (page 696); a = 1 **697.965**.
- Silva first: Silva 159/s; Panthera inherits E = 0, F_tick = 0: 0.5 × 155 = 77.5/s; a = 0 total **236.5**
  (page 237 with rounding); a = 1 **237.0**.

**TV-03-07 Heat strip ramp (E-3.8).** Initial armor 500, SD = 0, episode from t = 0 to t_exp = 6: armor at
[0, 0.5) 500; [0.5, 1) **420.448208**; [1, 1.5) **353.553391**; [1.5, 2) **297.301779**; [2, 7.5) **250.0**; then
down-steps at 7.5, 9, 10.5, 12: 297.301779, 353.553391, 420.448208, 500. SD = 1.0: up-steps at 1, 2, 3, 4; down
intervals 3 s. Full Heat with 10 Corrosive on 1000 armor: **100.0**. On capped armor 2700: full Heat 1350 (DR
0.636396), + 10 Corrosive 270 (DR 0.284605, damage multiplier 0.715395).

**TV-03-08 Cold crit damage (E-3.3 into E-4, `W:Damage/Cold_Damage` examples).** Kunai CDM0 = 1.6 (Q = 1.6019536257),
Primed Target Cracker ΣCD_rel = 1.1, 9 Cold stacks: CDM = **3.8641026** (page, unquantized: 3.86); orange crit
`1 + 2 × 2.8641026` = **6.7282052** (page 6.72). Paris Prime CDM0 = 2.0, 9 stacks, CC 1.125: CDM = **2.5004885**,
tier 1 2.5004885 (87.5 %), tier 2 **4.0009770** (12.5 %), E[c] = **2.6880496**; Frozen: CDM = **3.0004885**.
`cold_cd(n)` for n = 1 .. 9: 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50.

**TV-03-09 Debuff functions.** `corr_strip`: n = 1 0.26, 2 0.32, 5 0.50, 10 0.80, 12 0.92, 13 0.98, 14 1.0;
Hydroid c₁ = 0.50: n = 1 0.50, 9 0.98, 10 1.0. `viral_mult` = `mag_mult`: n = 0 1, 1 2.0, 4 2.75, 10 4.25.
`heat_strip`: k = 0..4: 0, 0.159104, 0.292893, 0.405396, 0.5.

**TV-03-10 Puncture and Tau.** CC0 = 0.30, ΣCC_rel = 1.5 (Point Strike), 5 Puncture stacks: CC = 0.75 + 0.25 =
**1.00**; Evensong 7 stacks: **1.10**. SC = 2.10 with 5 Tau stacks: **3.15**; 10 stacks: **4.20**.

**TV-03-11 Magnetic break proc (E-3.10).** Shield break, P_max = 1000, n_MAG = 10, A_sd = 0.9, F = 0, E^dot_ELE = 0,
SD = 0: T_brk = **180.5** per tick, ticks at t_b + 0 .. 5, total **1083** (page). SD = 1.0: 12 ticks, total 2166.
Variant E^dot_ELE = 0.9, F = 0.3: T_brk = **579.5855**, total 3477.513.

**TV-03-12 Blast (E-3.9, `W:Damage/Blast_Damage` Xata example).** Y = 100, F = 0.55, A_sd = 0, k = 0, Part_dot = 1:
H = 240.25; expiry hit at t + 1.5 = **72.075**; SD = 1: fuse 3.0 s. Ten stacks applied within one fuse (10th at
once): host under the Decision (Blast page) **720.75** as ten instances of 72.075; under the Calculation-page reading
(D-306) **0** to the host; others (not evaluated) one instance 3 × 2402.5 = **7207.5** in both readings.
Status-capped enemy or boss: detonation at the 4th stack, host 4 × 72.075 = **288.3** (alternative 0), others
3 × 961 = 2883. With k = 1, CDM = Q(2.0): S = 310.075721 (crit in the seed), H = 480.617367, hit **144.185210**.

**TV-03-13 Durations and tick counts (E-3.1).**

| SD | N (SLA/TOX/ELE/GAS) | D_COR | D_BLA | h_up | h_dn | D_frz | D_VIR | D_PUN |
|---|---|---|---|---|---|---|---|---|
| −0.5 | 3 | 4.0 | 0.75 | 0.25 | 0.75 | 1.5 | 3.0 | 5.0 |
| 0 | 6 | 8.0 | 1.5 | 0.5 | 1.5 | 3.0 | 6.0 | 10.0 |
| 0.55 | 9 | 12.4 | 2.325 | 0.775 | 2.325 | 4.65 | 9.3 | 15.5 |
| 1.0 | 12 | 16.0 | 3.0 | 1.0 | 3.0 | 6.0 | 12.0 | 20.0 |
| 1.1 | 12 | 16.8 | 3.15 | 1.05 | 3.15 | 6.3 | 12.6 | 21.0 |
| +0.60, +0.95, −0.55 summed left to right (f64 0.9999999999999998; `6·(1 + SD)` = 11.999999999999998, 11 without ε) | 12 | 16.0 (f64 15.999999999999998) | 3.0 | 1.0 | 3.0 | 6.0 | 12.0 | 20.0 |
| −0.9 | 0 (instances exist 0.6 s) | 0.8 | 0.15 | 0.05 | 0.15 | 0.3 | 0.6 | 1.0 |
| −1.0 | nullified | nullified | immediate hit | – | – | – | nullified | nullified |

**TV-03-14 Event order and Viral timing (§7).** Neutral faction, no armor, no shield, health view, SD = 0, every
weapon-side value given directly. Shot A at t = 0: direct 100, one Slash proc with weapon-side tick 35. Shot B at
t = 2.0: direct 100, procs Viral then Electricity (weapon-side tick 50). Shot C at t = 8.0: direct 100. Expected:
A 100; Slash ticks t = 1: 35, t = 2: **35** (P2 before B's procs), t = 3..6: 70 each (Slash total **350**); B direct
**100** (own Viral not applied yet); Electricity ticks t = 2 (P4c, sees Viral): 100, t = 3..7: 100 each (total
**600**); C direct **100** (Viral removed in P1 at 8.0). Total **1250**. Counts: n_A = 0, n_B = 1 (SLA), n_C = 0
(Slash ended at 6.0; Viral and Electricity ended at 8.0).

**TV-03-15 Cap replacement on a status-capped enemy (§6.2).** cap_SLA = 4. Slash procs at t = 0.0, 0.1, 0.2, 0.3, 0.4,
weapon-side tick 10 each, SD = 0: the proc at 0.4 replaces the one at 0.0 before any of its ticks; ticks land at
1.1 .. 6.1, 1.2 .. 6.2, 1.3 .. 6.3, 1.4 .. 6.4; total **240** (uncapped enemy: 300). n_SLA never exceeds 4.

**TV-03-16 Frozen (E-3.11).** Normal enemy, no overguard, SD = 0; Cold procs at t = 0.0, 0.1, ..., 0.9 (one per shot).
`cold_cd` seen by a hit at t = 0.85: 0.50 (9 stacks); the shot at 0.9 itself: 0.50; any hit in (0.9, 3.9): **1.0**;
a Cold proc at 2.0: discarded; at 3.9 (P1) three stacks inserted, expiring at 9.9: hits in [3.9, 9.9) see **0.20**;
from 9.9: 0. In the layered pass of an enemy with overguard (OG > 0 throughout, so `og_present`): cap 4, `cold_cd`
stays 0.25 from the 4th proc on, no Frozen; the same holds in its `overguard` view (og_present = true), while its
`health`, `armor` and `shield` views (og_present = false) freeze at 0.9 exactly as the normal enemy above (§6.1,
TV-03-28).

**TV-03-17 Heat episode pulses (E-3.7).** SD = 0, a = 1, Heat procs at t = 0 (S = 100, X = 0.5) and t = 2.5 (S = 200,
X = 0.5). Episode t_ign = 0, t_exp = 8.5. Pulses t = 1, 2: **50.5**; t = 3 .. 8: **150.5** each; total **1004**.
Strip: up-steps 0.5 .. 2.0; episode end 8.5; down-steps at 10.0, 11.5, 13.0, 14.5: on 1000 initial armor 500 →
594.6036 → 707.1068 → 840.8964 → 1000. Inspiration (D-027, own timers, a = 0): 6 × 50 + 6 × 100 = 900.

**TV-03-18 Tick at the instance's expiry (E-3.6, §7.2).** Weapon-side tick value 10 per tick (a included), neutral
target, health view. (a) Slash proc at t_p = 0.0, SD = 0: expiry `0 + 6·(1 + 0) = 6.0`; ticks at 1.0 .. 6.0, the last
one in P2 at the expiry time after P1 removed the instance: 6 ticks, **60** (a reading where expiry cancels the tick
gives 50). A shot at t = 6.0 sees n_SLA = 0. (b) Toxin proc at t_p = 0.3, SD = 1.0: D = 12.0, expiry
`300,000 + 12,000,000 = 12,300,000` µs, last tick `300,000 + (1 + 11)·10^6 = 12,300,000` µs, the same grid value;
12 ticks, **120**. (c) Grid against f64 drift (§7 time model): a Viral proc applied by the shot at t = 0 of a
10 shots/s schedule (SD = 0, D̂ = 6,000,000); the 61st fire time summed in f64 (`t ← t + 1/10`, 60 times) is
5.999999999999995, grid **6,000,000** = the expiry, so that shot sees n_VIR = **0** (direct 100 → 100 on health; a
raw f64 comparison would see the stack and give 200).

**TV-03-19 Heat strip ramp-down without live Heat, and re-ignition (E-3.8, §8).** A0 = 1000, no other strip, SD = 0.
Heat proc at t = 0 (alone): k_h = 1, 2, 3, 4 at 0.5, 1.0, 1.5, 2.0; episode ends at 6.0 (P3) with k_h = 4; down-steps
scheduled at 7.5, 9.0, 10.5, 12.0. Armor: [6.0, 7.5) **500.0** (no Heat live; a presence gate would give 1000);
[7.5, 9.0) **594.603558**; [9.0, 9.5) **707.106781**. Second Heat proc at t = 9.5 (k_h = 2): pending down-steps 10.5
and 12.0 cancelled; up-steps at 10.0 and 10.5; [9.5, 10.0) 707.106781; [10.0, 10.5) 594.603558;
[10.5, 15.5) 500.0; the new episode ends at 15.5; down-steps at 17.0, 18.5, 20.0, 21.5.

**TV-03-20 Heat pool with mixed SD (E-3.7, E-3.8).** a = 1. Proc A at t = 0 (SD_p = 0, S = 100, X = 0.5); proc B at
t = 1.5 (SD_p = 1.0, D_HEA = 12, S = 200, X = 0.5). t_exp after the join = 1.5 + max(6, 12) = **13.5**. Pulses at
t = 1: 50.5; t = 2 .. 13 (12 pulses): (100 + 200 + 1) × 0.5 = 150.5; total **1856.5**. Strip: up-steps scheduled at
ignition with SD* = 0 (0.5, 1.0, 1.5, 2.0); at 13.5 SD* is read before the pool is cleared: 1.0, h_dn = 3.0,
down-steps at 16.5, 19.5, 22.5, 25.5.

**TV-03-21 Heat pool cap on a status-capped enemy (§10.2).** cap_HEA = 4, a = 1, SD = 0, X = 0.5. Heat procs at
t = 0.0, 0.2, 0.4, 0.6, 0.8 with S = 100, 200, 300, 400, 500; the 5th pushes the first out. t_exp = 6.8; pulses at
1 .. 6, each (200 + 300 + 400 + 500 + 1) × 0.5 = **700.5**; total **4203.0** (uncapped enemy: 750.5 per pulse, 4503.0).

**TV-03-22 Gas replacement at cap 10 (§6.2).** Normal enemy, SD = 0, a = 1. Eleven Gas procs, one per shot, at
t = 0.0, 0.1, ..., 1.0, each S = 40, X = 0.5: every tick (40 + 1) × 0.5 = 20.5. Proc 1 (t = 0.0) lands tick 0 (P4c at
0.0) and tick 1 (P2 at 1.0, before shot 11's procs in P4b); the 11th proc replaces it at 1.0 and its ticks at
2.0 .. 5.0 are cancelled. Total 62 ticks: **1271.0** (no cancellation: 66 ticks, 1353.0). The inspiration's TTK
model skips the 11th cloud instead (D-035).

**TV-03-23 Several Blast procs crossing the cap in one shot (§11).** Normal enemy, SD = 0, every H = 100 (H_det = 30).
Eight stacks applied at t = 0.0 .. 0.7 (one per shot). Shot at t = 1.0 with three pellets, each applying one Blast
proc. P4b: 9th stack inserted; 10th inserted → n_BLA = 10 → detonation: host 10 instances of **30** in stack order
(Decision; the Calculation-page reading gives none), radial 3 × 1000 = 3000 to others (not evaluated), stacks cleared;
the 11th proc then becomes a new stack (n_BLA = 1) that expires at 2.5 with **30**. Host total from the 11 procs:
**330** (alternative reading: 30).

**TV-03-24 Magnetic break proc from a shot (E-3.10, layered pass).** Corpus target, SH max 1000 with 50 left, n_MAG = 4
before shot S at t = 2.0 (one source attack: F = 0, E^dot_ELE = 0, A_sd = 0, SD = 0). Pellet 0 breaks the shield and
also applies a Magnetic proc. Break proc with n = 4 (σ_S; the pellet's own Magnetic proc is excluded, n_MAG = 5 only
after P4b): `T_brk = 0.03 × 4 × 1000 / 6` = **20.0** per tick (n = 5 would give 25.0); applied in P4b after the
pellet's Magnetic proc; tick m = 0 lands on HP in P4c at 2.0; ticks m = 1 .. 5 at 3.0 .. 7.0 in P2; total **120.0**.
It counts as an ELE instance (n_ELE + 1) on [2.0, 8.0).

**TV-03-25 Negative status duration end to end (§5.1).** (a) SD = −1.0 on every proc. Shot at t = 0 with two pellets:
pellet 0 applies SLA and BLA (H = 100). SLA is nullified: no instance, no tick, not delivered, not in n; BLA: no stack,
its host hit **30** lands at once in P4b. Pellet 1's CO count n_1 = **0**. (b) SD = −0.9: the Slash instance lives on
[0, 0.6) (f64 `6 × 0.09999999999999998 = 0.5999999999999999`, grid D̂ = 600,000 µs) with
N = floor(0.5999999999999999 + ε) = **0** ticks; a shot at 0.5 counts SLA in n, a shot at 0.6 (grid 600,000) does not;
the Blast fuse is 0.15 s (D̂ = 150,000 µs), so the host hit 30 lands in P2 at grid 150,000.

**TV-03-26 Sequential-pellet CO count with exclusivity and discards (E-3.12).** Target immune to Radiation; σ_S holds
Lifted and 2 Viral stacks: n(σ_S) = 2. Shot S, three pellets: pellet 0 applies a forced KND and a COR proc; pellet 1
applies a forced RAD (discarded: immune, not delivered). n_0 = **2**; n_1 = |{VIR, KND, COR}| = **3** (KND removes
LIF; the plain union would give 4); n_2 = **3** (the discarded RAD adds nothing; a union including it would give 5).

**TV-03-27 Frozen with SD ≠ 0, and a target that cannot freeze (E-3.11).** (a) SD = 0.5 on every Cold proc; procs at
t = 0.0, 0.1, ..., 0.9 (one per shot). Frozen at 0.9 for D_frz = 3 × 1.5 = **4.5**, ending at **5.4** (P1); the three
thaw stacks have D = 9.0 and expire at **14.4**. `cold_cd`: shot at 0.9 sees 0.50; hits in (0.9, 5.4) **1.0**;
[5.4, 14.4) **0.20**; from 14.4, 0. A Cold proc at 3.0 is discarded (no Primary Frostbite stack). Secondary Shiver
during [0.9, 5.4): n_COL = 10, +4.50 (Decision, OQ-305). (b) `no_freeze` (Demolisher), SD = 0.5: no Frozen;
10 stacks, `cold_cd` **0.50** for hits in (0.9, 9.0); the stacks expire at 9.0, 9.1, ..., 9.9: [9.0, 9.1) 9 stacks
0.50, [9.1, 9.2) 8 stacks 0.45.

**TV-03-28 Cap resolution (E-3.2).** Base caps from table 4.1, enemy caps from `05`.

| Target | s | Attack | og_present | cap_s |
|---|---|---|---|---|
| normal enemy | SLA | any | false | ∞ |
| normal enemy | COL | any | false | 10 |
| Acolyte (all 4, IMP 3) | IMP | any | false | **3** |
| Acolyte | SLA | any | false | **4** |
| Kuva Lich (all 4, IMP 6, RAD 4) | IMP | any | false | **5** (min(5, 6)) |
| Kuva Lich | RAD | any | false | **4** |
| Eximus with OG, layered pass, OG > 0 | COL | any | true | **4** |
| the same Eximus, `overguard` view (MC and EV; spawns with OG) | COL | any | true | **4** (no Frozen) |
| the same Eximus, `health`, `armor` and `shield` views (MC and EV) | COL | any | false | **10** |
| the same Eximus, layered pass after its OG broke | COL | any | false | **10** |
| VIP with OG (`vip`), layered, OG > 0 | PUN | any | true | **3** |
| normal enemy | PUN | Evensong | false | **7** |
| Eximus with OG, layered, OG > 0 | PUN | Evensong | true | **5** |
| Acolyte | PUN | Evensong | false | **5** |
| status-capped enemy (all 4) | COR | shard holder's weapon, one Emerald (β = 2) | false | **6** |
| status-capped enemy (all 4) | COR | holder's weapon, one Tauforged Emerald (β = 3) | false | **7** |
| normal enemy | COR | holder's weapon, two Emerald (β = 4) | false | **14** |
| status-capped enemy (all 4) | COR | a weapon of another player | false | **4** |
| boss (`boss`, not status-capped, e.g. Vay Hek) | VIR | any | false | **10** |
| the same boss | COL | any | false | **4** (no Frozen) |
| the same boss | BLA | any | false | **4** (detonation at the 4th stack) |

Layered pass of an Eximus whose overguard breaks at t = 0.55, Cold procs at t = 0.0, 0.1, ..., 0.9: procs 1-4 insert
(n = 4), procs 5 and 6 replace the oldest (n = 4); from proc 7 the cap is 10, so n_COL = 8 at 0.9 and no Frozen; a hit
at 0.95 sees `cold_cd` = **0.45**.

**TV-03-29 Beam seed with b > 1 (E-3.4).** Beam tick of b = 2 merged beams, per-beam Y = 20, F = 0, G = 1, ELE proc,
E^dot_ELE = 1.5, A_sd = 0, k = 0: S = 20 × 2 = 40, tick (40 + 1) × 0.5 × 2.5 = **51.25** (the page's Electricity
example, `W:Damage/Calculation` line 627, with seed 40).

**TV-03-30 Stealth term in c_k (E-3.4, T-08).** Melee stealth hit, weapon rank 30: SDB = 1 + 0.2 × 30 = 7. Slash proc,
Y·(1 + F)·G = 100, F = 0, A_sd = 0, Part_dot = 1: k = 0: c = 8, seed 800, tick (800 + 1) × 0.35 = **280.35**;
k = 1, CDM = Q(2.0): c = 9.000488520, seed 900.048852, tick **315.367098** (the superseded structure, "+1" scaled by
c, gave 282.8 and 318.167269).

**TV-03-31 Sonar inheritance (Part_dot = Part, D-095).** Head hit on a Sonar spot: P_part = 3, ΣWP = ΣHS = 0,
M_son = 3.5: Part = Part_dot = 10.5. Slash with Y·(1 + F)·G = 100, k = 0: seed 1050, tick (1050 + 1) × 0.35 =
**367.85** (the former exclusion, Part_dot = 3: 105.35).

**TV-03-32 Mecha Empowered and Vauban second dips (§9.4).** Slash, F = 0, A_sd = 0, k = 0, a = 1.
- Mecha, one aura (2.5): Y = 40 on a marked target, S = 40 × 2.5 = 100; tick while marked (100 + 1) × 0.35 × 2.5 =
  **88.375** (a = 0: 87.5 = 40 × 0.35 × 6.25, the page's 6.25×); after the mark ends: **35.35**.
- Vauban: Y = 100, proc applied while incapacitated, S = 125; tick while still incapacitated (125 + 1) × 0.35 × 1.25 =
  **55.125** (a = 0: 54.6875 = the page's `0.35 × 100 × 1.25 × 1.25`); after incapacitation ends: **44.1** (alternative
  reading, OQ-319: 35.35); proc applied before incapacitation (S = 100), tick during it: **44.1875**.

**TV-03-33 Heat inherit with T-26 set (§10.3).** heatInherit v = 0.9; the weapon's own E^dot_HEA = 0, Y = 100,
F = 0.3, A_sd = 0, k = 0, one proc: S = 130, E^dot = 0.9. Game profile, `heatInheritFaction` unset (F_inh = 0):
tick (130 + 1) × 0.5 × 1.9 × 1.0 = **124.45**; F_inh = 0.3: **161.785**. Compat flag (F_tick = F_i = 0.3, a = 0):
130 × 0.5 × 1.9 × 1.3 = **160.55**.

**TV-03-34 Magnetic break source from different attacks (E-3.10).** Magnetic stacks from attack A (F = 0,
E^dot_ELE = 0, A_sd = 0.9) at t = 0 and attack B (F = 0.3, E^dot_ELE = 0.6, A_sd = 0) at t = 0.5. Shield break at
t = 1.0, n = 2, P_max = 1000, SD = 0: source B (newest): T_brk = 10 × 1.6 × 1.3² = **27.04** per tick, total 162.24
(A as source would give 36.1).

**TV-03-35 Harmony DoT detonation (E-3.14).** Neutral target, health view, SD = 0, a = 1. At t = 0 a Slash proc
(S = 100, X = 0.35) and a Heat proc (S = 100, X = 0.5, alone in its episode). Slash ticks at 1, 2: 35.35 each; Heat
pulses at 1, 2: 50.5 each. Harmony heavy at t = 2.5 (no procs of its own): Slash N_rem = 4, one instance
4 × 35.35 = **141.4**; Heat N_rem = 4 (pulses 3 .. 6 ≤ t_exp = 6), one instance 4 × 50.5 = **202.0**; both removed;
the episode ends at 2.5 with k_h = 4, down-steps at 4.0, 5.5, 7.0, 8.5. Totals unchanged (212.1 and 303.0), landed
earlier.

**TV-03-36 Ash's passive (§5, §9.4; `W:Ash/Abilities` line 5).** SD = 0, SD_SLA = 0.5, A_sd,SLA = 0.25. Y = 100,
F = 0.3 (Bane of Grineer), k = 0: S = 130; N = floor(6 × 1.5 + ε) = **9**, D_SLA = 9.0; tick a = 0: 130 × 0.35 × 1.3 ×
1.25 = **73.9375** (page 73.94); a = 1: **74.50625**; nine ticks 670.55625.

**TV-03-37 Status-damage vulnerability (§9.4).** Slash S = 100, F = 0, A_sd = 0, k = 0, a = 1. Proc created while
Burning Hate (+1.20) is on the target: V_sd = 2.2, tick (100 + 1) × 0.35 × 2.2 = **77.77**; with the custom
`vuln_status_damage` 0.30 as well: V_sd = 2.5, **88.375**; a proc created before the Hate hit: **35.35**, also after
the vulnerability starts.

**TV-03-38 M_OG on a status instance (§9.4).** Secondary Fortifier ×8 on the secondary that applied a Slash proc of
weapon-side tick 35.35; Eximus target, no Magnetic, no vulnerability. A tick landing on overguard: 35.35 × 8 =
**282.8**; the same tick on health: 35.35. A Heat proc of another weapon that inherits from this secondary's first Heat
proc gets no M_OG (not inheritable).

**TV-03-39 Sequential pellets: n_COL for Secondary Shiver against cold_cd (E-3.12, §7.1 P4a; `04` E-4.1, OQ-433).**
Normal enemy (can freeze, no overguard), SD = 0, one Secondary Shiver (b_cold = 0.45). σ_S holds 8 Cold stacks and no
other status. Shot S with three pellets, each applying exactly one Cold proc. CO-family reads (σ_S^{<j}): n_COL,0 =
**8**, n_COL,1 = **9**, n_COL,2 = **10** (pellets 0 and 1 make 10 stacks, Frozen starts in σ_S^{<2}, n_COL reported
10); Shiver terms b_cold·n_COL = **3.60**, **4.05**, **4.50**; n_j = 1 for all three (COL present throughout).
Debuff read (σ_S): `cold_cd(8)` = **0.45** for all three pellets (no pellet sees Frozen's 1.0). P4b: pellet 0's proc
makes 9 stacks, pellet 1's makes 10 → Frozen (stacks removed), pellet 2's is discarded (not delivered). On a fresh
target the same shot gives Shiver terms 0, 0.45, 0.90 and `cold_cd` 0 for every pellet.

**TV-03-40 Primary Debilitate proc seed (§9.8, OQ-313).** Inducing instance: Y = 100, F = 0.3 (Primed Bane), G = b = 1,
Part = 1, k = 0, E^dot_TOX = 0, A_sd = 0, SD = 0, a = 1; it applies an ordinary Toxin proc and Debilitate adds a Toxin
proc. Ordinary: S = 130, tick (130 + 1) × 0.5 × 1.3 = **85.15** (six ticks 510.9). Debilitate: S = 130 × 1.3 =
**169**, no "+1", tick 169 × 0.5 × 1.3 = **109.85** (six ticks 659.1; faction three times). With k = 1, CDM = Q(2.0):
S_i = 260.063508, Debilitate S = 338.082560, tick **219.753664** (ordinary: 169.691280).

---

## 20. Review notes

Two reviews of the draft raised 46 issues; each was checked against the cited sources (and against the current
versions of `05` and `08`, which changed during the review) before any change. All were applied except the flag
renames noted below; qualifications:

- Vauban passive and Mecha Empowered (§9.4): applied as a tick-time second dip, but with the "each damage calculation
  tests the condition" structure (the seed keeps the inducing hit's first dip after incapacitation ends) rather than
  the suggested "both dips lost" structure; the suggested reading is recorded as the alternative in OQ-319 and
  TV-03-32. Both readings fix the two errors reported (×1.5625 kept after the stun; ×1 for procs applied before it).
- `og_present` in the views (§6.1): the reported contradiction with `05` no longer exists: `05` §18 has since been
  revised to defer to E-3.2 and restates this document's rule (`05` OQ-529). The rule is kept; the remaining conflict
  with `08` §4.5 (`08` OQ-804) is recorded, the trade-off stated and a per-view exception proposed (OQ-321), and a test
  vector added (TV-03-28).
- Flag names (issue on notation): rejected for the flags. `05` has since renamed its fields to `vip` and `no_freeze`,
  "named as `03` consumes it" (`05` data model), so this document keeps `vip` and `no_freeze`. The other notation
  items (missing symbols, `punc_cc` range, D-028 toggle column) were applied.
- `05` E-5.10's Heat presence gate (issue on the Heat strip): `05` has since removed it; the normative sentence (§8)
  and the overview symbol proposal (OQ-322) were still added.
- M_OG (§9.4): snapshotted per proc as M_OG,p and applied on overguard only, not folded into X_p as suggested, because
  X_p applies on every layer.
- The leading-tab ForcedProcs entry "\tBlast" named in one review is Lua whitespace before the string
  (`Weapons_data_misc.lua` line 367, `SlamForcedProcs = {<tab>"Blast" }`), not part of the name; no normalisation rule
  is needed for it.
- Proc-name counts differ slightly from one review's (for example "Impact" about 350 vs 344, "Ragdoll" 33 vs 31): the
  counts here include every `Weapons_data_*.lua` category module; the conclusions do not depend on the exact numbers.

Third review (three critics on the whole spec, five issues on this document), each checked against the sources and
the current versions of `00`, `04`, `05`, `07` and `08`:

- Time model (§7, E-3.6, E-3.7, E-3.14, TV-03-18, TV-03-25): applied. Overview CD-14 already adopts `08` E-8.0a; this
  document now compares grid values only, and ε is confined to the tick-count floor. TV-03-18(c) added.
- `vuln_status_damage` placement (§9.4): rejected for this document. The inspiration sums Burning Hate (itself the
  stat `vuln_status_damage: 1.2`) and the custom stat into one `(1 + md)` (`JS L205-209`, `L10467-10469`,
  `L10498-10502`), so the additive V_sd group stays; overview §6.3 and `04` §10 should be amended instead (OQ-324).
- Sequential-pellet n_COL (§7.1 P4a, §7.2, E-3.12, §14): applied; σ_S^{<j} now serves every CO-family read (n and
  n_COL) as `04` E-4.1 requires (OQ-433); `08` E-8.9/E-8.10 must add the K_j term of `04` E-4.2. TV-03-39 added.
- Stale OQ-804 remark (§6.1, OQ-321): applied; `08` §4.5 now follows E-3.2.
- Proc name `Finisher` (§4.2): applied as CC only (`07` OQ-715); stance counts re-measured including nested proc
  groups (Knockback 239, Ragdoll 86).

Follow-up round (2026-11-24), applying what the revised `00`, `01` and `02` asked of this document; each item was
checked against the cited sources before it was applied, and none was rejected:

- Crit and body part in the seed (`00` E-13, D-025; `W:Damage/Calculation` lines 632-635): E-3.4, E-3.5, E-3.9 (H_p
  unchanged in value), §9.5, §9.6 (EV of the seed), §10.3, §15 (68,522.387417, was 68,713.737867), TV-03-01 (k = 1:
  343.787408), TV-03-04 and TV-03-05 (order no longer matters), TV-03-12, TV-03-30 (280.35, 315.367098), all
  recomputed with `python3`. OQ-301 keeps only Part_dot's placement and the accumulator of mixed groups.
- Sonar inherited (`00` D-095; `W:Sonar` lines 25-26): Part_dot = Part, §9.2, EC-03-29, TV-03-31 (367.85), a D-095
  row in §17; Detect Vulnerability by analogy with the patch-note quote of `W:Detect_Vulnerability` lines 50-52
  (OQ-312).
- Per-view `og_present` (`00` §4.1): E-3.2, §6.1, E-3.13 (ragdoll immunity per view, as `02` E-2.13a reads it),
  TV-03-16, TV-03-28, OQ-321.
- M_OG on Slash ticks (`00` E-13), Saryn's passive (`00` D-037, T-16), extra hits of radial instances (`00` E-14,
  §7.1 P4a), boss caps (`00` CD-07, `05` §4.4: bosses Cold 4 and Blast 4 only; §6.1, E-3.9, TV-03-12, TV-03-28).
- `01` §4.10.5: §9.3 excludes Sentient Incision (`W:Sentient_Incision` line 46, a Bugs entry) and states that `04`
  and `01` supply the `dot_scaling` flags; base damage (Reactive Storm's converted Impact) never counts, as `01` E-1.15
  already says.
- `02` E-2.13a: §8.10 no longer calls the ragdoll rule dormant; the EV mixing over RAG is `02`'s with ρ_RAG from `08`.
- Found by the scan of `00` as finalised: the adopted proposals OQ-315 (F_inh), OQ-322, OQ-323, OQ-333, OQ-335 and
  the ragdoll ownership (OQ-326) are now cited as adopted (TV-03-33 recomputed: 124.45 with F_inh = 0); the
  `vuln_status_damage` sentence of §9.4 now names only `04` §10 as stale; D-073 is three times, so §9.8 and TV-03-40
  give the Debilitate seed Decision (OQ-313); extra-hit procs take no "+1" (E-3.5, as overview E-14 says); stance S_j
  and heavy H_j moved from G to Y in §9.2 (overview §2.4); T-43 external statuses, whose stack state overview §7.2
  gives to this document, are defined in §6.3 with their Decisions in OQ-336.

Tail round (2026-11-24), three cross-document issues, each checked against the current `00`, `04`, `05` and the cached
wiki pages; all applied, none rejected:

- §14: the EV of n_COL,j is now cited as `08` E-8.10, its sole owner (overview §3.3, §7.2); `04` E-4.2 is cited for
  the counting rule only, as it states no EV form.
- §9.4 and OQ-324: `04` §10 now puts `vuln_status_damage` in V_sd (additive with Burning Hate and Pageflight), so the
  "must follow" sentence and the OQ-324 request are removed; the game questions of OQ-324 stay.
- E-3.13 and the §2 inputs row: ragdoll immunity reads the `05` field `cc_immune` (bosses, Acolytes, the Demolisher
  family; `05` §4.1, EC-05-93; `W:Ragdoll` line 4, `W:Demolisher` lines 68-69, both checked in the cache) plus
  `og_present`.
