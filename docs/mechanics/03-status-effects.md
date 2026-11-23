Status: draft (spec)

# Damage-calculation spec, part 03: status effects

This document defines what every status effect does once a proc exists: its duration, timers, stack caps
(including enemy-specific caps), replacement, the debuff functions that other stages read, the DoT tick formula E-13
in full, Heat's shared timer and inherit rule, Blast's fuse and detonation, Gas, Electricity, the Magnetic break proc,
status duration (including negative values), and the distinct-status count `n`. It owns the effect half of stage
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

Citation keys are those of `00-overview.md` §0.2. Wiki line numbers refer to the raw copies in
`research/external/cache/wiki/` (file name = page name, `/` → `__`). Pages added to the cache for this document:
`Knockdown.wiki`, `Lifted.wiki`, `Ragdoll.wiki`, `Stun.wiki` (fetched 2026-11-23).

---

## 1. Scope

Copied from `00-overview.md` §7.1: every status type: duration, timers, caps (including enemy-specific caps CD-07 as
rules), replacement; debuff functions (Viral, Magnetic, Corrosive strip, Heat strip ramp, Cold crit damage and Frozen,
Puncture crit chance, Tau, Radiation, Impact, Void, Microwave, Knockdown, Lifted); DoT tick formula E-13 (seed, "+1",
coefficients, elemental scaling, faction twice, status damage, crit, body part, inheritance list); Heat shared timer and
inherit; Blast fuse and detonation; Gas; Electricity; Magnetic break proc; status duration and negative SD;
distinct-status counting (what counts, CD-04 timing).

Explicitly not here: layer routing of ticks (Toxin bypass, overguard, shield gate, armor floor; `05`), the expectation
of `n` and of stack counts in the EV evaluator (`08`), proc counts and proc-type weights (`02`), where Cold and
Puncture enter the crit formulas (`02`), the catalogue of which mods, arcanes and abilities feed SD, A_sd and E^dot
(`04`), the CO term inside the seed (`04`, E-4.7), extra-hit DoTs (`04`, E-14).

Stages: ST-10 (effects), and the status inputs of ST-7 and ST-8. Equations owned: E-13 (restated in full as E-3.4 to
E-3.6) and E-3.1 to E-3.12.

---

## 2. Inputs

| Symbol | Owner | Used in |
|---|---|---|
| `base[t]`, `base_total` (B'), `base_ref` (B_ref), Y (E-11) | `01`, `04` | seed (E-3.4) |
| `sum_faction` (F) | `01` | seed, tick |
| `mult_other` (G), `mult_final_omitted` (G_F), `co_dot_mode` | `04` | seed (E-4.7) |
| crit tier k, `cd` (CDM), `headcrit` (h), stealth term SDB, `part` (Part), `sonar_mult` (M_son) | `02`, `04` | c_k, Part_dot |
| proc list of each damage instance (types in roll order, forced procs, Hunter Munitions and Internal Bleeding Slash) | `02` | §6, §7 |
| merged beam count b of a beam tick (CD-10) | `02` | seed |
| SD sources (`sum_status_dur`), status-damage sources (`sum_status_dmg`, `status_dmg_type[s]`), elemental sources with their `dot_scaling` flag, M_dot sources (Mecha Empowered, Vauban passive, custom `vuln_status_damage`) | `04` (catalogue), this document (buckets) | E-3.1, E-3.4 |
| enemy status immunities, per-enemy status cap and per-type overrides, flags `vip`, `no_freeze`, scaled pools OG and SH | `05` | §6, §13 |
| shot and hit schedule, instant-burst grouping, melee combo multiplier M(c) at each hit | `06`, `07` | §7, Lifted |
| T-05 (count mode), T-06 (external Viral), T-26 (`heatInherit`), compat profile flags | `00` §6 | §10.3, §14 |

---

## 3. Outputs and symbols added

Outputs (overview symbols, defined here): `dur_base[s]`, `dur[s]`, `dot_ticks[s]`, `dot_delay[s]`, `dot_coeff[s]`,
`dot_elem[s]`, `sum_status_dmg`, `status_dmg_type[s]`, `sum_status_dur`, `stack_cap[s]`, `stacks[s]`,
`viral_mult(n)`, `mag_mult(n)`, `corr_strip(n)`, `heat_strip(τ)`, `cold_cd(n)`, `punc_cc(n)`, `n_distinct`; the DoT
tick and Blast damage instances (weapon side; the enemy side is E-12 with the tick's type, `05`); the Magnetic break
instances.

**Symbols added**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `dot_seed` | S_p | HP | seed of proc p (the `seed` of E-13), snapshotted at proc creation |
| `dot_mult` | X_p | factor | per-proc tick multiplier (E-3.4) |
| `dot_part` | Part_dot | factor | body-part factor inherited by procs, `Part / M_son` |
| `dot_extra[s]` | M_dot,s | factor | multipliers applied to ticks of s only (second dips, custom status-damage vulnerability) |
| `dot_beam` | b | count | merged beams of the inducing beam tick (CD-10), 1 otherwise |
| `dot_acc` | a | HP | DoT accumulator start value: 1 (game profile, D-025), 0 (compat) |
| `dot_group` | 𝒢 | set | procs whose ticks are consolidated into one tick (§9.5) |
| `dur_eps` | ε | – | 1e-9, rounding guard for floors and inclusive time comparisons (§5) |
| `cap_enemy[s]` | ecap_s | count | enemy cap of s from `05` data (per-type override, else per-enemy global cap, else ∞) |
| `cap_bonus_cor` | β_COR | count | Emerald Archon Shard bonus to the Corrosive cap (Σ over shards: 2, Tauforged 3) |
| `og_present` | – | bool | whether the overguard-dependent caps apply (§6.1) |
| `cold_frozen` | – | bool | target is Frozen (§8.6) |
| `dur_frozen` | D_frz | s | `3·(1 + SD)` of the freezing proc |
| `heat_ign`, `heat_exp` | t_ign, t_exp | s | ignition time and expiry time of the current Heat episode |
| `heat_first` | p₁ | proc | first proc of the Heat episode (inherit source) |
| `heat_step` | k_h | 0..4 | Heat armor-strip step counter |
| `heat_up_int`, `heat_down_int` | h_up, h_dn | s | Heat strip step intervals `0.5·(1 + SD*)`, `1.5·(1 + SD*)` |
| `heat_inh_faction` | F_inh | bonus | faction bonus of an external first Heat source (T-26 companion input, proposed, OQ-315; default 0) |
| `blast_h` | H_p | HP | Blast stack value ("damage H of the hit that applied it") |
| `blast_fuse` | D_BLA | s | `1.5·(1 + SD)` |
| `corr_first` | c₁ | fraction | strip of the first Corrosive stack: 0.26, or 0.50 with Hydroid's passive |
| `mag_break_tick` | T_brk | HP | weapon-side tick of the Magnetic break proc |
| `lif_dur`, `knd_dur` | D_LIF, D_KND | s | Lifted and Knockdown durations |

---

## 4. Status catalogue

### 4.1 Summary table (normative; the engine transcribes this)

SD = `sum_status_dur` of the proc's source (§5). "Indep." = every stack has its own timer. N_s uses ε (§5).

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
| LIF | Lifted | M(c) (1 at no combo) | no | 1 | single; a LIF proc on a Lifted target ends Lifted; removes KND | – | – | – | – | – | – | none on damage | yes |

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
KND `W:Status_Effect` lines 148-153, `W:Knockdown` (no duration given); LIF `W:Lifted` line 5 ("1 second at base,
increasing by 1 second for every Melee Combo tier, up to 12 seconds at 12x"), `W:Melee` line 799 (status duration
does not extend it). Tick tables: `W:Status_Effect` lines 422-447, `W:Damage_over_Time` table. Coefficients: E-13,
`W:Damage/Calculation` line 623.

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
| Ragdoll | CC only | ragdoll state for the Impact rule of `02`; its duration is not documented |
| Stagger, Big Stagger | CC only | `PT_STAGGERED`, `PT_BIG_STAGGER` are separate from Impact (`W:Status_Effect` lines 128-133, 193-197); UNVERIFIED that data editors never use "Stagger" for Impact (OQ-310) |
| Stun | CC only | conflict on whether it counts for CO (OQ-310) |
| Blind, Impair, Disarmed | CC only | Impair is PvP only |

---

## 5. Status duration

The status-duration bucket `sum_status_dur` (SD) is additive over its sources: Lasting Sting family (Lasting Sting,
Continuous Misery, Lingering Torment, Perpetual Agony), Hunter Track, Augur Seeker, riven Status Duration (positive or
negative), Saryn's and Lavos's passives, Harmony, Prismatic Gem (values in `04`) (`W:Status_Effect` lines 692-706,
`W:Damage_over_Time` §Increasing DoT Duration, `F:mods` §7, `F:buffs` §7). A proc snapshots the SD of its source attack
at creation (CD-05); a later change of SD does not change live stacks.

```text
E-3.1  Modded duration and tick count (CD-06)
  D_s   = D0_s · (1 + SD)                       no rounding; every s with "SD scales = yes" in table 4.1
  N_s   = floor(6·(1 + SD) + ε)                 s ∈ {SLA, TOX, ELE, GAS}; ε = 1e-9
  D_BLA = 1.5·(1 + SD)          D_frz = 3·(1 + SD)          h_up = 0.5·(1 + SD*)          h_dn = 1.5·(1 + SD*)
  D_s   = D0_s                                  s ∈ {KND, LIF}; MIC: ∞
```

- `floor` for ticks: `W:Lasting_Sting` §Notes ("Due to status duration always rounding down, there is no difference
  in performance between a 9th rank and 10th rank version": +100 % and +110 % both give 12 ticks). Debuff durations
  are not rounded (Decision of CD-06; UNVERIFIED, OQ-005). The ε guard absorbs f64 summation error of bonuses (for
  example `0.3 + 0.7 = 0.9999999999999999` would otherwise turn 12 ticks into 11); it is part of the rule, not a
  tolerance in tests.
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
SD ≤ −1 (D_s ≤ 0 for every s whose duration scales with SD): the proc is nullified: no stack, no DoT
                   instance, no debuff, not counted in n, no Heat episode, no Frozen. Exception BLA: D_BLA ≤ 0,
                   the stack's expiry hit lands at once (in P4b of its shot, §7.1) and no stack is created.
                   MIC, KND and LIF do not scale with SD and are unaffected.
−1 < SD:           normal rules; a DoT whose N_s = 0 (SD < −5/6) creates an instance that exists (and counts in n)
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
  cap0 = min(base_cap[s], ecap_s)                  base_cap from table 4.1; ecap_s from 05 data:
                                                   per-type override, else per-enemy global cap, else ∞
  if og_present(e) and s = COL:            cap0 = min(cap0, 4)
  if og_present(e) and vip(e) and s = PUN: cap0 = min(cap0, 3)
  if s = PUN and a has the Evensong trait:  cap0 = 5 if (ecap_PUN < 5 or og_present(e)) else 7
  cap_s = cap0 + β_COR   if s = COR (procs of the shard holder's own weapons)
  cap_s = cap0           otherwise
og_present(e): layered pass: OG pool > 0 at that moment; every non-depleting view and the EV evaluator: the enemy
               spawns with OG > 0 (the status state is shared by all views, overview §4.1)
```

Sources and enemy data that `05` supplies (CD-07, D-053):

| Rule | Source |
|---|---|
| Status-capped enemies (Kuva Liches, Sisters, Technocyte Coda, Acolytes, Archons, other bosses in `05` data): global cap 4 for every status, DoT instances included | U27.3 note on `W:Bosses` line 925 ("No Status Effect will exceed a maximum of 4 stacks"); `W:Acolytes` line 17; `W:Archon` (Archon Hunts); application to DoTs UNVERIFIED (OQ-008) |
| Acolytes: Impact cap 3 | `W:Acolytes` line 17 |
| Kuva Liches: Impact override 6 (U27.3), so `min(5, 6) = 5` since U33.6 cut Impact to 5 | `W:Bosses` line 925; `W:Damage/Impact_Damage` |
| Radiation cap 4 on Kuva Liches, Acolytes, enemy Necramechs, Sisters of Parvos, Hounds | `W:Damage/Radiation_Damage` §General |
| Overguard present: Cold cap 4 (so no Frozen) | `W:Damage/Cold_Damage` line 27 (U36) |
| VIP with overguard: Puncture cap 3 | U36 fix "instead of the intended 3" (`F:status` §4.2) |
| Evensong: 7 Puncture stacks per hit, beyond the normal 5; enemies "usually capped at 4 stacks of a status, such as Acolytes and enemies with Overguard" receive 5 | `W:Evensong` lines 17, 42 |
| Emerald Archon Shard: Corrosive cap +2 (Tauforged +3) per shard, additive over shards, also raises a boss's cap of 4; only Corrosion applied by the holder's weapons and abilities exceeds the base cap | `W:Archon_Shard` lines 351-354; `W:Emerald_Archon_Shard` lines 40-43 (multi-shard total UNVERIFIED, OQ-317) |
| Enemies immune to a status (Demolishers: Viral; Manics: Radiation; Alkonost and Necramechs: all) | `05` data (`ProcResists`), `W:Damage/Viral_Damage` list, `F:enemy` §9.2 |

A cap never removes stacks that already exist when it drops (for example an overguard regenerating, not modelled);
it only governs later insertions.

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
Cancelling the replaced Gas instance's ticks follows from the cloud being replaced (Decision; the page says the
instance is replaced, nothing about its pending ticks). For SLA, TOX, ELE and HEA, replacement only occurs on
status-capped enemies (CD-07, OQ-008).

Immunity: a proc of a type the enemy is immune to is discarded at application, including forced procs (the roll
weights are renormalised in `02`; `W:Status_Effect` §Status Immunity Interactions). Immunity to "all" discards every
proc and disables every status-damage term (`F:status` §5; inspiration `Nb`, `JS L10520`). T-06 is ignored on a
Viral-immune enemy.

### 6.3 Stack count n_s

`stacks[s]` (n_s) is the number of live stacks (debuffs), live DoT instances (SLA, TOX, ELE, GAS), procs in the Heat
pool (HEA) or live fuses (BLA). With T-06 on, `n_VIR = min(N_ext + n_VIR,own, cap_VIR)`; own stacks beyond the cap
replace the oldest own stack (overview T-06).

---

## 7. Timing and event order

This section makes CD-04 and CD-05 operational for both evaluators. Event times are f64 values from the schedule
(`06`, `07`) plus the offsets of this document; equality is exact (no tolerance), so equal-time cases are decided only
by the order below.

### 7.1 Processing order at one time τ

```text
P1  State transitions due at τ:
      stack and DoT-instance expiries (t_apply + D_s = τ); Void, Knockdown, Lifted ends;
      Frozen end and its 3-stack re-application (§8.6); Heat strip steps (§10.4).
P2  Scheduled damage due at τ, from procs applied before τ:
      DoT tick groups with m ≥ 1 (SLA, TOX, ELE, GAS, Magnetic-break ELE), Heat pulses, Blast fuse expiries.
      Order among them: creation order of the oldest proc in each group.
P3  Heat episode end if t_exp = τ (after its pulse at τ, if any); schedule the strip ramp-down.
P4  Each shot scheduled at τ, in schedule order (instant-burst shots are separate shots). For a shot S with
    damage instances j = 0 .. J−1 (pellets in order; one merged instance per beam tick, CD-10):
      a. for j = 0 .. J−1: compute the instance's damage against σ_S, the target state after P1-P3 and before any
         proc of S; the CO count n_j (E-3.12) additionally includes types applied by instances 0 .. j−1 of S;
         roll crit and procs (02); apply the damage (05). A layer break here creates a Magnetic break proc (§13).
      b. apply the procs of S: instance order, then the proc-list order of 02; Magnetic break procs of S come
         after the procs of the instance that broke the layer. Insertions, replacements, Frozen start, Heat
         ignition and pool joins, Blast detonations (host hits land here, in stack order) happen in this step.
      c. land the zero-delay ticks created in b (ELE and GAS tick m = 0, Magnetic-break tick m = 0), grouped per
         §9.5, evaluated against the state after b.
Events that step b or c creates at time τ are processed inside that step; nothing created at τ runs before P4.
```

Sources: `W:Damage/Viral_Damage` lines 31-36 and the editor's comment block lines 37-56 (Simulacrum-derived: a
shot's pellets all deal damage, then the shot's procs apply; an instant burst's later shots see earlier Viral; an
Electricity tick created with a Viral proc is amplified at once); CD-04; `F:addmul` §1.2 (CO counts earlier pellets of
the same shot). Extending "no sibling benefit" from Viral to every debuff is CD-04's decision (UNVERIFIED, OQ-028).

### 7.2 When a status starts and stops affecting damage

- **Start.** A stack applied in P4b of shot S at τ affects: the zero-delay ticks of S (P4c); every later shot at τ;
  everything at times > τ. It never affects the instances of S (sibling pellets included) except through the CO count
  n of later instances of S, and never a P2 tick at τ.
- **Stop.** A stack applied at t_a lives on the half-open interval `[t_a, t_a + D_s)`: it is removed in P1 at
  `t_a + D_s`, so damage at exactly `t_a + D_s` does not see it. Heat presence ends in P3 at t_exp.
- **Heat strip.** The step counter changes in P1 at its step times; direct hits and ticks at that time see the new
  value.
- **Snapshots.** Weapon-side terms of a proc (seed, crit tier, body part, faction, elemental and status-damage
  bonuses, SD, M_dot sources) are fixed at proc creation (CD-05). Target-side terms (Viral, Magnetic, armor with its
  strips, type modifier, vulnerability, innate DR) are read when the tick or Blast hit is processed (D-031).

### 7.3 When a DoT tick lands

```text
E-3.6  Tick schedule of a proc p of s applied at t_p (time of its shot)
  SLA, TOX:     τ_m = t_p + 1 + m,  m = 0 .. N_s − 1        (P2)
  ELE, GAS:     τ_0 = t_p in P4c of p's shot;  τ_m = t_p + m, m = 1 .. N_s − 1   (P2)
  HEA:          pulses of the episode (E-3.7), not per proc
  BLA:          one hit at t_p + D_BLA (P2), unless detonated earlier (§11)
  Magnetic break: as ELE, from the break time t_b (§13)
A tick of a removed instance (§6.2) does not land. A tick never lands after the end of the evaluation window for
window metrics; it is reported as pending (overview §4.1).
```

Source: `W:Status_Effect` lines 422-447 and `W:Damage_over_Time` table (Slash, Heat, Toxin first tick at 1 s, six
ticks to 6 s; Electricity and Gas at 0 s to 5 s, "the last tick deals no damage"); CD-06 for N_s. The DoT page's
`floor(rate·(Duration − Delay)) + 1` counts the non-damaging tick at 6 s for Electricity and Gas and is not used
(`F:status` §2.3).

---

## 8. Debuff functions

All functions return the value for the stack count n_s of the state that the consuming damage instance sees (§7.2).

```text
E-3.3  Debuff functions
  viral_mult(n) = 1 if n = 0, else 2 + 0.25·(n − 1)                      on HP damage (incl. armored HP) only
  mag_mult(n)   = 1 if n = 0, else 2 + 0.25·(n − 1)                      on SH and OG damage only
  corr_strip(n) = 0 if n = 0, else min(1, c₁ + 0.06·(n − 1))           c₁ = 0.26 (0.50 with Hydroid's passive)
  heat_strip    = 1 − 0.5^(k_h / 4),  k_h ∈ {0, 1, 2, 3, 4}             (§10.4)
  cold_cd(n)    = 0 if n = 0;  0.10 + 0.05·(min(n, 9) − 1) for n ≥ 1;  1.0 while Frozen
  punc_cc(n)    = 0.05·n                                                 (n ≤ cap_PUN, up to 7 with Evensong)
  tau_sc(n)     = 1 + 0.10·n                                             factor in M_sc (E-8)
```

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
overview's `min(1, 0.20 + 0.06·n)` equals the c₁ = 0.26 case. Hydroid's passive is a target flag set by the `04`
catalogue ("enemies damaged by him"; any Corrosive source counts).

### 8.3 Puncture

Flat (absolute) crit chance on weapon damage the target receives: `punc_cc(n)` enters ΣCC_abs of E-5 for every
non-AoE weapon instance (not radial, not abilities); the DoT of the hit inherits it through the crit tier
(OQ-017). The enemy-damage reduction (40 % + 10 %/stack) has no DPS effect. Source: `W:Damage/Puncture_Damage`
§General ("flat critical chance buff (like Arcane Avenger), but does not apply to Area of Effect damage or Warframe
abilities"); `W:Critical_Hit` §Absolute Increases; U33.6 history (`F:status` §4.2).

### 8.4 Tau

`tau_sc(n)` multiplies the status chance of every damage instance that rolls status on the target, including the
Tau-dealing weapon's own later hits (UNVERIFIED for self-application, OQ-311); it does not add forced procs. Example
(`W:Damage/Tau_Damage`): 210 % SC → 315 % at 5 stacks, 420 % at 10. Placement: E-8 (`02`).

### 8.5 Radiation and Impact

No effect on the player's damage. Radiation's ally-damage bonus (100 % + 50 %/stack, 550 % at 10; on Liches and
similar 100 %/stack taken from allies, cap 4) and Impact's stagger and mercy threshold (+8 %/stack) are not modelled.
Both count in n. Overguarded enemies receive Radiation but ignore it until the overguard is gone (`W:Damage/Radiation_Damage`).
The Internal Bleeding, Hemorrhage and Magnetic Welt triggers belong to the Impact *proc* (`02`); a ragdolled target
receives no status from an Impact roll (`02`, `W:Damage/Impact_Damage`).

### 8.6 Cold and Frozen

```text
E-3.11 Frozen
  Trigger: application of a Cold proc p makes n_COL = 10, cap_COL = 10, target not no_freeze
  At t_f (P4b): remove all Cold stacks; Frozen on [t_f, t_f + D_frz), D_frz = 3·(1 + SD_p)
  While Frozen: cold_cd = 1.0; n_COL reported as 10 (CO count, Secondary Shiver); Cold procs discarded
  At t_f + D_frz (P1): Frozen ends; insert 3 Cold stacks with t_apply = t_f + D_frz and D = 6·(1 + SD_p)
  no_freeze targets (Demolishers): 10 stacks stay stacks; cold_cd(10) = 0.50 (UNVERIFIED, OQ-305)
```

`cold_cd` is an absolute crit-damage bonus added after relative crit-damage mods and before the crit tier (ΣCD_abs of
E-4, `02`), not applied to AoE damage. Frozen "receive[s] double Critical Damage", read as +1.0 by the wiki table.
Sources: `W:Damage/Cold_Damage` lines 18-25 and table (U36: 10 stacks and Freeze; U33.6: crit damage bonus), worked
examples Kunai 3.86× and Paris Prime (TV-03-08); `W:Status_Effect` line 317 (SD lengthens Frozen "and subsequent Cold
status"). Frozen prevents shield recharge (TTK, `05`). Melee Careen's ×2.5 reads the Frozen flag (`04`). The thaw rule
"3 Cold stacks will remain" does not say which timers they carry; fresh stacks are the Decision (UNVERIFIED, OQ-305).

### 8.7 Microwave

Nukor and Kuva Nukor only. Every hit of an attack flagged `applies_microwave` (data, `06`) applies MIC; it lasts for
the rest of the window and counts once in n. The body-part enlargement is not modelled. Source: `W:Microwave`;
`W:Status_Effect` lines 161-165 ("infinite duration and counts as an individual status"). How the game applies it per
hit is not documented (UNVERIFIED).

### 8.8 Void

A 2.5 m projectile-attracting field for 3 s, centred on the body part hit (U29.5). No damage effect in a
single-target model; counts in n. Stacking and SD behaviour are undocumented: Decision one instance, a new proc
restarts its timer, D scaled by SD (UNVERIFIED, OQ-311). Void's ×1.5 against overguard is a damage-type rule (`05`).
The independent `PT_VOID` status (Sentient adaptation reset, `W:Status_Effect` line 205) is not a Void proc and does
not count.

### 8.9 Knockdown and Lifted

- KND lasts D_KND = 3 s (UNVERIFIED: no page states a duration; 3 s is the documented finisher window of a Stun,
  `W:Status_Effect` lines 199-203; OQ-309), not scaled by SD; a new KND proc restarts it; KND removes LIF.
- LIF lasts D_LIF = M(c) seconds, where M(c) is the combo multiplier the inflicting melee attack uses (`07`); 1 s for
  attacks without combo (non-melee sources). Not scaled by SD. A LIF proc on a Lifted target ends Lifted (the enemy is
  ragdolled; no LIF stack results); any non-melee hit on a Lifted target ends it; LIF removes KND. Source: `W:Lifted`
  lines 5-12 (newer, per-mechanic page). `W:Melee` line 690 says "1 second, increased by 1 additional second for each
  Combo Multiplier" (a 4× slam lasting 5 s in its caption): conflict recorded (OQ-309).
- Overguarded enemies ignore the crowd control of KND and LIF; whether the status still counts in n is undocumented:
  Decision it counts (UNVERIFIED, OQ-309). Both count in n and cannot coexist (`W:Condition_Overload_(Mechanic)` lines
  182-185).

---

## 9. DoT ticks (E-13 in full)

### 9.1 Proc snapshot

A proc p of s ∈ {SLA, HEA, TOX, ELE, GAS, BLA} created by damage instance i of attack a stores:

```text
E-3.4  Proc snapshot (restates E-13)
  S_p  = Y_i · (1 + F_i) · G_i · b_i                       seed; Y_i per E-4.7 (CO per co_dot_mode), not quantized
  X_p  = K_s · (1 + E^dot_s) · (1 + F_tick,p) · (1 + A_sd,i + A_sd,s,i) · c_k,i · Part_dot,i · M_dot,s,i
  K_s        = 0.35 SLA; 0.5 HEA, TOX, ELE, GAS; 0.3 BLA
  F_tick,p   = F_i, except HEA (§10.3)
  E^dot_s    = §9.3; HEA: the episode's inherited value (§10.3); SLA, BLA: 0
  c_k,i      = 1 + k·(h·CDM − 1)  or  1 + SDB + k·(h·CDM − 1) for a stealth hit (E-12, T-08), as rolled by instance i
  Part_dot,i = Part_i / M_son,i                             (Sonar and Detect Vulnerability spots not inherited)
  b_i        = merged beam count of a beam tick (CD-10), else 1
  SD_p       = the source's SD (E-3.1)
```

`W:Damage_over_Time` §DoT Damage Scaling lines 167-186; per-type pages (Slash, Heat, Toxin, Electricity, Gas lines
22-36; Blast lines 27-55): `Modded Base Damage = Base × (1 + Base Damage Bonuses) × (1 + Faction)`, tick `= C × Modded
Base × (1 + element bonus) × (1 + Faction) × (1 + Status Damage) × Additional Multipliers (crit, body part)`.
`W:Damage/Calculation` lines 613-617: ticks are not quantized; "Critical DoTs use the effective critical multiplier of
the inducing hit" with crit quantization. Faction double dip: `W:Faction_Damage_Bonus` intro, every DoT page
(`(1+F)²`, +69 % for 0.30, +140.25 % for 0.55).

### 9.2 Inheritance list

| Term of the inducing instance | In the proc? | Source |
|---|---|---|
| base damage incl. progenitor and Incarnon flat (B'), base bucket A | yes (Y) | DoT page line 170; `F:core` §3 |
| CO term | per `co_dot_mode` (E-4.7: seed by default; Tysis and Stinger exclude; Pox recalculates per tick) | `F:addmul` §1.12 |
| G_F members (Eclipse, Furious Javelin, Garuda, ...) | yes, once (in Y) | DoT page line 170; D-041 |
| G members (stance hit, heavy type and combo, sniper combo, Primed/Charged Chamber, Synth Charge, beam ramp, Vauban passive, Navigator, Mecha Empowered, Melee Careen ...) | yes, once (G) | DoT page lines 178-180; `W:Primed_Chamber` ("applies to ... status damage"); `F:addmul` §2 |
| faction F | twice (seed and tick) | §9.1 |
| crit tier as rolled, incl. Cold and Puncture effects on that roll, head crit, stealth term | yes (c_k) | Damage/Calculation line 617; DoT page line 177; OQ-017 |
| body part, weak-point and headshot bonuses | yes (Part) | DoT page line 181 |
| Sonar, Detect Vulnerability spots | no | DoT page line 171; `W:Damage/Slash_Damage` (conflict with `W:Sonar` line 25, OQ-312) |
| merged beam count | yes (b): DoT scales twice with beam multishot | `W:Multishot` §Continuous Weapons; CD-10 |
| per-type composition, quantization, physical and elemental mods | no (except E^dot of the proc's own element) | DoT page lines 172-176; Slash page ("Buzz Kill have no effect") |
| stance physical bonus S^phys | no (not in Y) | E-2; `F:melee` §2.1 |
| finisher-damage mods (Finishing Touch, Savage Silence) | no | `W:Damage/Slash_Damage` §Increasing Slash Proc Damage (D-303) |
| status damage A_sd, A_sd,s | yes | `W:Status_Effect` line 450 |
| target-side terms (Viral, Magnetic, armor, type modifier, vulnerability, innate DR) | no: read at tick time | `W:Damage/Viral_Damage` line 31; `W:Damage_Vulnerability` §Mechanics; CD-05 |

### 9.3 Elemental scaling E^dot_s

```text
E^dot_HEA, E^dot_ELE, E^dot_TOX = Σ of every elemental bonus of that element on the attack (mods, riven element
    stats, arcanes, archon shards, Valence Formation of that element, non-combining additions such as Fireball
    Frenzy or the custom heatAdd), whether or not it was combined into a secondary element on the weapon
E^dot_GAS = Σ of literal Gas bonuses only (Leaded Gas +3.0, Valence Formation set to Gas, riven or spliced Gas)
E^dot_SLA = E^dot_BLA = 0
Excluded everywhere: buffing-field elements (Electric Shield, Conductive Sphere field Electricity, Energy Shell Heat,
    weapon-made fields), Nourish (Viral), bonuses of other elements
```

Sources: `W:Damage_over_Time` lines 172-176 (combined DoTs not buffed by component mods; Toxic Lash's forced Toxin is
buffed by Toxin mods combined into Corrosive); `W:Damage/Gas_Damage` line 44 and example 2 (Leaded Gas ×4);
`W:Damage/Blast_Damage` line 56 and §Notes ("Valence Formation or Thermal Transfer does not increase Blast status
damage"; Heat and Cold mods do not either; OQ-007); `W:Electric_Shield` via `F:buffs` §1.12 and §9 (field Electricity
"does not raise Electricity status damage"); `W:Valence_Formation` via `F:buffs` §8 (+200 % to that element's DoT, not
Blast). Each elemental source in the `04` catalogue carries `dot_scaling`; the default is true except for buffing
fields. Membership of the Violet shard Electricity, spliced Gas and Conductive Sphere's field element is UNVERIFIED
(OQ-316).

### 9.4 Status damage and M_dot

- `sum_status_dmg` A_sd: Elementalist mods, riven Status Damage, Galvanized Elementalist, heavy-attack-only status
  damage (Empowered Blades) on heavy attacks; `status_dmg_type[s]` A_sd,s: Emerald Archon Shard (TOX, also on Toxic
  Lash procs), Conductive Sphere (ELE), Ash (SLA, ability source). All additive in one bucket, multiplicative with
  everything else (`W:Status_Effect` lines 450-470; `W:Archon_Shard` line 337; D-069). Leaded Gas is E^dot_GAS, not
  status damage (D-301).
- M_dot,s (applied to DoT ticks and Blast hits only): Mecha Empowered's second application `(1 + 1.5·n_aura)` while
  the target is marked (D-041); Vauban passive's second ×1.25 when the proc was applied while the target was
  incapacitated; the custom `vuln_status_damage` factor `(1 + x)` (overview §6.3). Values and conditions: `04`.
  Sources: `W:Mecha_Empowered` ("applied each time damage is calculated"), `W:Vauban/Abilities/Passive` example
  (`F:buffs` §1.19).

### 9.5 Tick groups and the "+1" accumulator

```text
E-3.5  Value of one tick group 𝒢 (weapon side)
  𝒢 for SLA, TOX:   one proc
  𝒢 for ELE, GAS:   all live instances of that status whose tick lands at the identical time τ (Magnetic-break
                    instances excluded: each forms its own group, §13)
  𝒢 for HEA:        the episode's pool at the pulse (E-3.7)
  tick_𝒢 = Σ_{p∈𝒢} S_p·X_p  +  a·X_{p₀}            p₀ = the oldest proc in 𝒢 (application order); a = dot_acc
  For |𝒢| = 1 this is E-13: (S_p + a)·X_p.
  BLA and the Magnetic break proc never use a.
Enemy side at the group's processing point: E-12 with the single type s of the tick (SLA as Cinematic: no type
  modifier, no armor; Viral on HP, Magnetic on SH and OG, vulnerability once; layer routing, innate DR, attenuation
  and the armor floor of 1 per 05). The group is one damage instance.
```

Source: `W:Damage/Calculation` lines 619-635 (single editor, added 2026-07/08; UNVERIFIED, D-025, OQ-023): the
accumulator of each tick group starts at 1; Heat consolidates all stacks into one tick; Electricity and Gas
"contributions occurring on the same pulse are consolidated into one damage number", +1 once; Slash and Toxin stacks
tick separately with their own +1. Examples reproduced in TV-03-02 to TV-03-05. f32 arithmetic is not emulated
(overview §2.1, NP-2 row: DoT ticks are f64; differences < 1e-6 relative). Display rounding `floor(x + 0.5)` is display only.

The game stores the crit multiplier inside the seed (the page's "critical seeds of 80.0195389" = 40 × Q(2.0)), so in
the game the "+1" is not multiplied by c_k. E-13 as fixed by the overview places c_k outside the seed. E-3.5 keeps
E-13 for single-proc groups and, for consolidated groups, multiplies the single accumulator by the oldest proc's
multiplier, which reproduces the page's Gas and Heat examples when the oldest proc is the non-critical one
(TV-03-04, TV-03-05). Proposal to move c_k and Part_dot into the seed: OQ-301.

### 9.6 Per-proc totals

The weapon-side total of a non-Heat DoT proc is `N_s·(S_p + a)·X_p` before target-side terms; the landed damage is the
sum of its ticks' E-12 values at their landing times (target state varies). The inspiration credits
`curr_duration × tick` at the hit (D-086, D-036). `08` uses these totals and the tick schedule E-3.6.

---

## 10. Heat

### 10.1 Episode and pool

```text
E-3.7  Heat episode on a target
  Ignition: a Heat proc p applied when no episode is live: t_ign = t_p, pool = {p}, p₁ = p, t_exp = t_p + D_HEA,p.
  Join: a Heat proc p applied while an episode is live (t_ign ≤ t_p, episode not ended):
        pool ← pool ∪ {p} (if |pool| > cap_HEA: remove the oldest proc);  t_exp ← t_p + max_{q ∈ pool} D_HEA,q
  Pulses: at τ_m = t_ign + m, m = 1, 2, ... while τ_m ≤ t_exp + ε (processed in P2):
        tick = Σ_{q ∈ pool, q applied before the pulse} S_q·X_q  +  a·X_{q₀}       (E-3.5; q₀ oldest in the pool)
  End: in P3 at t_exp: pool cleared; strip ramp-down scheduled (§10.4). A Heat proc after the end ignites a new
        episode (new p₁, new pulse grid).
```

Sources: `W:Damage/Heat_Damage` lines 63-66 (one combined tick per second; a new proc adds its damage per second and
refreshes all stacks; duration reset to the longest status duration among the stacks; "stacked up indefinitely"),
line 54-61 table (first tick at 1 s, last at 6 s); `W:Damage_over_Time` ("DoT instance can be refreshed / No limit");
D-027. The pulse grid anchored at ignition, and a joining proc first contributing to the next pulse, are not stated
(Decision, matches the inspiration's TTK model `JS L13690-13703`; UNVERIFIED, OQ-302). A single proc with no refresh
yields `floor(6(1 + SD) + ε)` pulses, as CD-06 requires.

### 10.2 Heat pool on status-capped enemies

The pool holds at most cap_HEA procs (4 on status-capped enemies, ∞ otherwise); the oldest leaves the pool when a new
proc would exceed it, and its contribution stops from the next pulse. The episode's t_exp still follows the join rule.
UNVERIFIED (CD-07 extended to Heat's pool; OQ-008, OQ-304).

### 10.3 Heat inherit

```text
For a proc p in an episode with first proc p₁:
  E^dot_HEA(p) = E^dot_HEA of p₁'s source            (p = p₁: its own)
  F_tick,p     = F of p₁'s source                    (p = p₁: its own)
  S_p keeps p's own Y, own F (once), own G, b;  A_sd, A_sd,HEA, c_k, Part_dot, M_dot and SD: p's own
T-26 heatInherit = v set: an external Heat source is the first source of every episode on the target:
  E^dot_HEA(p) = v and F_tick,p = F_inh (default 0; proposed input, OQ-315) for every own Heat proc.
  The external source's own ticks, presence in n and armor strip are not modelled (inspiration parity).
Compat profile: F_tick,p = F_i (faction squared) and E^dot_HEA = v or own (D-028).
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
  SD* = SD of the proc with the largest D_HEA in the pool when the steps are scheduled
  At ignition t_ign: cancel pending down-steps; schedule up-steps at t_ign + j·h_up, j = 1 .. 4 − k_h(t_ign)
                     (each: k_h ← k_h + 1, in P1)
  At episode end t_exp (P3): cancel pending up-steps; schedule down-steps at t_exp + j·h_dn, j = 1 .. k_h(t_exp)
                     (each: k_h ← k_h − 1, in P1)
  heat_strip = 1 − 0.5^(k_h/4):  0, 0.159104, 0.292893, 0.405396, 0.5
  Joining procs do not reschedule up-steps.
```

Source: `W:Damage/Heat_Damage` §Armor Stripping lines 95-110 (every 0.5 s after the proc current armor × 0.5^(1/4) to
50 % at 2 s; re-proccing does not speed it up; after the proc ends armor returns every 1.5 s over 6 s;
+100 % SD → 1 s steps, −50 % → 0.25 s); `W:Status_Effect` line 325 (ramp down also lengthened by SD); D-026.
Multiplicative with Corrosive and with the additive ability/Corrosive Projection group (E-15, `05`). Re-ignition during
the ramp-down resumes ramping up from the current step and the choice of SD* are Decisions (UNVERIFIED, OQ-303).

---

## 11. Blast

```text
E-3.9  Blast
  Stack value:  H_p = S_p · (1 + F_i) · (1 + A_sd,i + A_sd,BLA,i) · c_k,i · Part_dot,i · M_dot,BLA,i
  Fuse:         D_BLA = 1.5·(1 + SD_p);  stack lives on [t_p, t_p + D_BLA)
  Expiry:       at t_p + D_BLA (P2): one Blast instance of max(1, 0.3·H_p) to the host (E-12, type BLA)
  Detonation:   inserting the stack that makes n_BLA = cap_BLA (10; 4 on status-capped enemies), in P4b:
                host: one Blast instance max(1, 0.3·H_q) per live stack q (the new one included), in stack order;
                other enemies within 5 m, no falloff (multi-target only, not evaluated): one instance 3·Σ_q H_q;
                all stacks cleared
  Death:        the target dying with ≥ 1 live stack (or killed by a hit that applies a Blast proc) detonates
                for other enemies only (multi-target, not evaluated); the host takes nothing more
  SD ≤ −1:      no stack; the host instance max(1, 0.3·H_p) lands at once in P4b
  No "+1", no elemental scaling (E^dot_BLA = 0), status damage applies, faction twice, crit and body part apply
```

Sources: `W:Damage/Blast_Damage` lines 19-66 (U36 rework; edited 2026-11-03: 1.5 s fuse affected by SD; 30 % at expiry;
10 stacks detonate, "10 hits of 30% base damage to the target at once"; 300 % per stack to others in one instance, no
falloff, radius not changed by Firestorm or Fulmination; the host is not dealt the AoE part; killing blows with a
Blast proc trigger the full AoE; faction twice; no Heat, Cold, Valence Formation or Thermal Transfer scaling;
crit, weak points and head crits apply to both parts); `W:Damage/Calculation` lines 641-645 (bosses cap at 4; each
stack records H; expiry `max(1, 0.3 × H)`; detonation `3 × Σ H`). The page's "H of the hit that applied it" is read as
the Blast page's modded-base product (`F:status` open 7; UNVERIFIED, OQ-306). Whether the host's detonation damage is
one instance or one per stack only matters for the armor floor and attenuation: one per stack ("10 hits") is the
Decision (UNVERIFIED, OQ-306). Blast-element bonus scaling conflicts with `W:Damage_over_Time` (OQ-007, D-034).
Xata's Whisper triggering off detonations is an `04` effect (D-071).

---

## 12. Gas and Electricity

**Gas.** Each proc is an instance with its own timer and N_GAS ticks at t_p + 0 .. N − 1 (E-3.6); cap 10 (status-capped
enemies 4) with oldest-replacement (§6.2); same-pulse ticks consolidate (§9.5); E^dot_GAS = literal Gas only (§9.3).
The cloud's radius (3 m, +0.3 m per stack, "6 m at 10 stacks"; the literal arithmetic gives 5.7 m, `F:status` §4.10),
its damage to other enemies, and its persistence after the host dies are multi-target behaviour (not evaluated). The
cloud has its own 1× headshot multiplier: on the host the tick uses the inherited Part_dot and nothing more, so
headshot bonuses count once (OQ-025 decision). Sources: `W:Damage/Gas_Damage` lines 18-60 (U27.3 for persistence).

**Electricity.** Each proc is an instance with its own timer and N_ELE ticks at t_p + 0 .. N − 1; unlimited (4 on
status-capped enemies); same-pulse ticks consolidate; E^dot_ELE includes Electricity mods combined into a secondary
element; the 3 m chain to other enemies is multi-target only; the 3 s stun is crowd control (not SD-scaled; bosses
immune); the chain's own 1× headshot is handled as for Gas (OQ-025). Tick 0 lands in P4c and therefore sees a Viral
proc of the same shot (`W:Damage/Viral_Damage` line 36). Sources: `W:Damage/Electricity_Damage` lines 18-60.

---

## 13. Magnetic break proc

```text
E-3.10 Magnetic break proc (U36)
  Trigger: a damage instance depletes the overguard or the shield (layered pass only, D-054), with
           n = n_MAG ≥ 1 in the state σ_S that the breaking instance saw
  Pool:    P_max = scaled maximum OG or SH of the broken layer (05)
  Source:  the attack that applied the newest live Magnetic stack (for one attack: that attack); its
           F, E^dot_ELE (combined Electricity mods included), A_sd, A_sd,ELE and SD
  T_brk = (0.03 · n · P_max / 6) · (1 + E^dot_ELE) · (1 + F)² · (1 + A_sd + A_sd,ELE)²
  It is an ELE proc of the target (counts for cap_ELE and n) applied in P4b after the breaking instance's procs;
  ticks: N_ELE = floor(6(1 + SD) + ε) at t_b + m (m = 0 in P4c), each one instance of T_brk, enemy side as an
  ELE tick at its time (it hits whatever layer is next: SH after an OG break, else HP)
  No "+1", no crit, no body part, no base-damage bonus, no M_dot; its ticks form their own group
```

Source: `W:Damage/Magnetic_Damage` lines 34-41: "a total of 3% of the target's maximum Shields/Overguard per Magnetic
status effect, to a maximum of 30% at full stacks, over a period of 6 seconds"; determined by the weapon or ability
that applied the Magnetic status; base-damage mods have no effect; status-damage mods apply twice (×3.61 at +90 %);
faction twice; Electricity mods apply even if combined; "Status duration mods are applied like normal and are a final
multiplier to the total damage"; `W:Status_Effect` line 343 (stun unaffected). Equal tick split, the floor tick count
(so `N/6` rather than exactly `1 + SD` as the total's factor), the absence of crit and "+1", "newest stack's source"
and n taken before the breaking instance's procs are Decisions (UNVERIFIED, OQ-308). Example: 1000 shields, 10 stacks,
+90 % status damage: 1083 total, 180.5 per tick (TV-03-11).

---

## 14. Distinct-status count n

```text
E-3.12 Distinct-status count (definition; T-05 chooses the count source, 08 the EV expectation)
  present(s, σ):
    SLA, TOX, ELE, GAS   ≥ 1 live instance (lives on [t_p, t_p + D_s), §7.2)
    HEA                  a live episode (t_ign ≤ τ, ended in P3 at t_exp)
    BLA                  ≥ 1 live fuse
    COL                  n_COL ≥ 1 or Frozen
    IMP PUN RAD MAG VIR COR TAU   ≥ 1 live stack (VIR: also T-06 external stacks)
    VOI KND LIF          live;      MIC   applied earlier in the window
  n(σ) = Σ_s [present(s, σ)]                       s over the 18 StatusTypes; KND and LIF exclusive → n ≤ 17
  For damage instance j of shot S:  n_j = | {s : present(s, σ_S)} ∪ {types of procs of instances 0..j−1 of S} |
     where procs that §5.1 or §6.2 nullifies or discards are not included; instance j's own procs never count
  A DoT seed uses the n_j of its inducing instance (E-4.7); ticks do not recount n except co_dot_mode = per_tick
```

Sources: `W:Condition_Overload_(Mechanic)` lines 162-185 (statuses with a damage-type equivalent including Void and
Tau, plus Lifted, Knockdown, Microwave; Lifted and Knockdown exclusive); `F:addmul` §1.2 (unique types, not stacks; any
source; the applying hit does not benefit; multishot pellets sequential); `W:Galvanized_Aptitude` §Notes example;
CD-04. Frozen is a state of COL. Crowd-control-only names of §4.2 never count; Stun is excluded although
`W:Status_Effect` lines 199-203 say it counts (the per-mechanic CO page wins, overview §1.4; OQ-310). The maximum is 17
with Tau (the Galvanized pages' 16 omit Tau; `F:addmul` §1.2). Secondary Shiver and Shattering Frost read n_COL or Cold
presence, not n (`04`).

---

## 15. Worked example

Heat proc from a critical headshot, checked with `python3`. Inputs: B' = 100, A = 1.65 (Serration), no CO, G_F = G = 1,
b = 1, F = 0.30 (Bane), E^dot_HEA = 0.90 (Hellfire), A_sd = 0.90 (Rifle Elementalist), M_dot = 1; CDM0 = 2.0,
ΣCD_rel = 1.2 → CDM = Q(2.0)·2.2 = 2.0004885197·2.2 = 4.40107474; head hit with P_part = 3, ΣWP = ΣHS = 0, M_son = 1,
head crit h = 2, tier k = 1; SD = 0; first and only Heat proc on the target. Target: Grineer (φ_HEA = 1),
A0 = 1355 armor, no Corrosive, no Viral, health layer, a = 1.

1. `S_p = 100 × 2.65 × 1.30 = 344.5`.
2. `c_1 = 1 + 1 × (2 × 4.40107474 − 1) = 8.80214949`; `Part_dot = 3`.
3. `X_p = 0.5 × 1.9 × 1.3 × 1.9 × 8.80214949 × 3 = 61.96273131`.
4. Weapon-side pulse value `(344.5 + 1) × 61.96273131 = 21,408.123668` (compat a = 0: 21,346.160936).
5. Strip: up-steps at t + 0.5 and t + 1.0 precede the pulse at t + 1 (P1 before P2): k_h = 2, armor
   `1355 × 0.5^(2/4) = 958.129689`, DR = 0.53613329, tick 9,930.515855. Pulses at t + 2 .. t + 6: k_h = 4, armor
   677.5, DR = 0.45083256, tick 11,756.644402 each.
6. Six pulses: **68,713.737867** on armored health. The episode ends at t + 6 (P3); down-steps at t + 7.5, 9, 10.5,
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
| EC-03-12 | DoTs use the crit tier rolled by the inducing hit (yellow, orange, red), with crit quantization | `W:Damage/Calculation` line 617 | §9.1 |
| EC-03-13 | Cold's crit-damage bonus is absolute, before tiers, not on AoE; +1.0 while Frozen; capped at 4 stacks (+0.25) on bosses and overguarded enemies | Cold page lines 18-27 | §8.6, §6.1 |
| EC-03-14 | Frozen: 3 s × (1 + SD); no further Cold stacks or refresh (Primary Frostbite cannot extend it); 3 stacks remain after thaw; no shield recharge | Cold page lines 23-25 | §8.6 |
| EC-03-15 | Demolishers do not freeze at 10 Cold stacks (90 % slow) | `F:enemy` §9.2 (Demolisher page) | §8.6 (`no_freeze`, value UNVERIFIED) |
| EC-03-16 | Puncture's crit chance is flat, after mods, not on AoE or abilities; VIPs with overguard cap at 3 stacks | Puncture page; U36 | §8.3, §6.1 |
| EC-03-17 | Evensong applies 7 Puncture stacks per hit (beyond 5, +35 % crit chance); 5 on enemies capped at 4 and on overguarded enemies | `W:Evensong` lines 17, 42 | §6.1; D-305 |
| EC-03-18 | Magnetic multiplies damage to overguard as well as shields; Viral to health only (armored health included; on overguard it was a bug fixed in U31.6) | Magnetic, Viral, Overguard pages | §8.1 |
| EC-03-19 | Viral and Magnetic multipliers return to 1 when the last stack expires | Viral page | §8.1 (D-029) |
| EC-03-20 | Tau multiplies status chance (×(1 + 0.1n)); forced procs gain nothing | Tau page | §8.4 |
| EC-03-21 | Blast: host takes only 0.3× per stack at expiry or detonation; others 3× per stack in one instance, no falloff, radius not modded; bosses detonate at 4 | Blast page; Damage/Calculation lines 643-645 | §11 |
| EC-03-22 | Blast deals its target damage even with SD below −100 %; its AoE then happens only if the triggering hit kills | `W:Status_Effect` negative table; Blast page line 64 | §5.1, §11 |
| EC-03-23 | A killing blow that applies at least one Blast proc triggers the full AoE even with no prior stacks | Blast page line 65 | §11 (multi-target, not evaluated) |
| EC-03-24 | Xata's Whisper triggers off each Blast detonation (faction a third time, elemental bonuses re-applied) | Blast page §Notes | out: `04` (D-071) |
| EC-03-25 | SD ≤ −100 % nullifies duration effects and DoTs; instant effects happen | `W:Status_Effect` lines 356-420 (outdated 2016 source) | §5.1; OQ-307 |
| EC-03-26 | SD adds DoT ticks rounded down (Lasting Sting rank 9 = rank 10) | `W:Lasting_Sting` §Notes | E-3.1 |
| EC-03-27 | SD extends debuffs, Frozen and the Cold stacks after it, the Blast fuse, the Heat strip ramp and ramp-down, the Magnetic break DoT; not crowd control (stagger, panic, stun, Lifted) | `W:Status_Effect` lines 292-354; Lasting Sting; Lifted | §5 |
| EC-03-28 | More elemental damage does not lengthen procs | `W:Status_Effect` line 231 | §5 |
| EC-03-29 | Sonar and Detect Vulnerability spots boost only the initial hit, not the DoT (the Sonar page says the opposite) | DoT page line 171; Slash page; `W:Sonar` lines 25-26 | §9.2; OQ-312 |
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
| EC-03-41 | Radiation capped at 4 on Liches, Acolytes, enemy Necramechs, Sisters, Hounds; overguarded enemies receive it but ignore it; Manics immune | Radiation page | §6.1, §8.5 |
| EC-03-42 | Corrosive: Emerald shard +2/+3 max stacks per shard, also on status-capped enemies, only for the holder's own Corrosion; 14 stacks strip all armor; Hydroid's passive first stack 50 % | Corrosive page; Archon_Shard lines 351-354; Emerald shard lines 40-43 | §6.1, §8.2; OQ-317 |
| EC-03-43 | Gas cloud radius grows with stacks; host death keeps the cloud ticking on others | Gas page | §12 (multi-target) |
| EC-03-44 | Electricity hits all enemies within 3 m of the target; stun of the primary target only, not SD-scaled, bosses immune | Electricity page | §12 (multi-target) |
| EC-03-45 | Heat inherit: the first source's Heat elemental and faction bonus are used by later procs until Heat ends; abilities and companions without bonuses weaken a later weapon's Heat | Heat page lines 68-90 | §10.3; T-26 |
| EC-03-46 | Overguarded enemies receive status effects and their DoTs, ignoring crowd control; Cold capped at 4 | Overguard page; Cold page | §6.1; routing `05` |
| EC-03-47 | Status immunity to "all" (Objects, Alkonost, Necramechs) disables every proc and status damage | `F:status` §5; Damage/Object | §6.2 |
| EC-03-48 | Viral-status-immune enemies (Ambulas, Deimos Carnis/Rex, Genetrix, Jugulus, Leaping Thrasher, Therid, Saxum/Rex, Demolisher Boiler/Thrasher, Tusk Bolkor, Tusk Firbolg, Techrot Babau) | Viral page | data `05`; §6.2 |
| EC-03-49 | Heat panic and Electricity stun ignore SD; Heat panic re-triggers on overguard break | Heat page §Crowd Control | no damage effect |
| EC-03-50 | Impact 5 stacks cause a Big Stagger; mercy threshold +8 %/stack | Impact page; `W:Status_Effect` line 129 | no damage effect |
| EC-03-51 | Knockdown and Lifted count for CO and cannot coexist; Lifted lasts 1 s per combo tier, is not SD-scaled, ends on a second Lifted proc or a non-melee hit | CO Mechanic lines 182-185; `W:Lifted` | §8.9 |
| EC-03-52 | Microwave (Nukor, Kuva Nukor) lasts forever and counts once | `W:Microwave` | §8.7 |
| EC-03-53 | Void: 3 s field; stacking undocumented; the separate `PT_VOID` status does not count | Void page; `W:Status_Effect` line 205 | §8.8 |
| EC-03-54 | Magnetic break: 3 % of max shield or overguard per stack (30 % at 10) over 6 s; base-damage mods no effect; Electricity mods combined or not apply | Magnetic page | §13 |
| EC-03-55 | Nightwatch Napalm Heat procs last only 3 s base (68 % fixed chance per blaze tick); the blaze is not a status DoT | `W:Nightwatch_Napalm` §Notes | §5 (D0 override); blaze `06` (D-074) |
| EC-03-56 | Bubonico alternate-fire explosion "+9 s pre-mods" status duration | `W:Status_Effect` line 695 | not modelled (no data; OQ-318) |
| EC-03-57 | Primary Debilitate's extra procs are separate instances: faction three times on their Heat, Electricity, Toxin DoTs | `W:Primary_Debilitate` §Notes | D-073 (engine: twice per overview; proposal OQ-313) |
| EC-03-58 | Hunter Munitions and Internal Bleeding Slash procs are normal Slash procs: stacks, n, and E-13 with the hit's crit and headshot | `W:Hunter_Munitions` §Notes | §9; rolls `02`; D-070 |
| EC-03-59 | Melee Afflictions adds stacks of every existing status with averaged proc damage | `W:Melee_Afflictions` §Damage Calculation | out: `04` (trigger and averaged seed); new stacks follow §6 |
| EC-03-60 | Secondary Cryogenic adds Cold stacks (SD of the secondary applies); Archon Vitality and Continuity add Heat or Corrosive procs from abilities | arcane and mod pages | out: `04`, `02`; stacks follow §6 |
| EC-03-61 | Heavy attacks of Claws, Dual Daggers, Nikanas, Rapiers, Scythes, Tonfas, Warfans, Whips force Bleed | Slash page | out: `07` (forced procs) |
| EC-03-62 | Mecha Empowered applies twice on DoT; Vauban passive twice if the proc was applied while incapacitated; target vulnerabilities once, at tick time | `F:buffs` §1.19, §2, §9 | §9.4 (M_dot) |
| EC-03-63 | Electric Shield's field Electricity does not raise Electricity status damage; Valence Formation adds +200 % to its element's DoT (not Blast) | `F:buffs` §1.12, §8, §9 | §9.3 |
| EC-03-64 | Emerald shard's Toxin status damage also scales Toxic Lash's procs; Conductive Sphere gives Electricity status damage | `F:buffs` §5, §1.12 | §9.4 |
| EC-03-65 | Sniper combo counts only direct hits, not AoE or DoT; Relentless Combination gives combo chance on Slash ticks | `F:weapon` §5.4; `F:melee` §3.1 | out: `06`, `07` |
| EC-03-66 | Tysis and Stinger DoTs ignore CO; the Pox cloud recalculates per tick | CO Mechanic catalogue | out: `04` (E-4.7) |
| EC-03-67 | Archons: 50 % reduction to status damage, 20 % to all damage, 4-stack cap | `W:Archon` line 32 | caps §6.1; DR `05` |
| EC-03-68 | DoTs can be ended early by cleanses or invulnerability; DoT "detonation" abilities (Expedite Suffering etc.) and Sickening Pulse duplication | `W:Damage_over_Time` | out of scope (abilities) |
| EC-03-69 | Players: Cold 10 % per stack, max 4, never frozen; overguard gives full status immunity | Cold page; Overguard page | out of scope (player side) |
| EC-03-70 | Viral and Magnetic peak at 4.25× (U27.2 note "up to 4.5x" is outdated) | Viral page; `F:status` §4.13 | §8.1 |
| EC-03-71 | The inspiration's displayed status chance (used by its Melee Duplicate re-roll) assumes 10 Tau stacks for Tau attacks (`JS L8893-8901`; `I:status` Q32); the game reads live stacks | `I:stat` §10; Tau page | engine: live n_TAU (§8.4); duplicate rule `04` |
| EC-03-72 | Inspiration-only weapon `unique` keys `addPunctureStatus` (adds Puncture stacks to the current hit only) and `decr_armor_by_punc` (armor strip per Puncture stack) have no game source and no data in the mod table | `I:queue` §3.1, §10; `I:status` open questions | not modelled; add only with a game source |

---

## 17. Inspiration differences

The overview's register rows owned or co-owned by `03`, restated with the engine behaviour, followed by new rows
(D-301 to D-305) in the same layout. Toggle column as in overview §5.1.

| ID | Topic | Inspiration behaviour | Game behaviour (source) | Engine (game profile) | Toggle |
|---|---|---|---|---|---|
| D-015 | Cold crit damage | `+0.05·n` (+0.5 more at 10) added to the final multiplier after tier and headshot, only on crits, also on AoE; no Frozen state, no 4-stack cap on bosses or overguard except via `maxCountProc` (`JS L10821-10825`; `I:status` §5.4) | `cold_cd` before tiers, not AoE, Frozen +1.0, cap 4 on bosses and overguard (§8.6) | E-3.3, E-3.11, §6.1; compat reproduces the inspiration's term | Compat |
| D-025 | DoT "+1" | none (`getTickStatusDmg`, `JS L9154-9204`) | `(Σ seeds + 1)·C·M` per tick group (§9.5) | a = 1 with E-3.5 | Compat (a = 0) |
| D-026 | Heat armor strip | flat 50 % while any Heat stack exists (`JS L10815`) | 4-step ramp up and down, SD-scaled (§10.4) | E-3.8 | Compat |
| D-027 | Heat stacking | each Heat proc on its own timer in DPS; TTK uses a shared refreshing timer (`JS L11139-11143`, `L13690-13703`) | one episode, pool refreshed by every proc, longest duration (§10.1) | E-3.7 in both evaluators | Compat |
| D-028 | Heat inherit | custom `heatInherit` replaces the Heat elemental bonus; faction always squared (`getTickStatusDmg`, `JS L9154-9204`) | first source's Heat and faction bonus for later procs; inherited faction once (§10.3) | §10.3: own later procs inherit from p₁; with T-26 set, E = v and F_tick = F_inh (default 0) | User (T-26) + Compat |
| D-029 | Viral and Magnetic after expiry | last multiplier sticks for the rest of the queue (`JS L10320-10321`, `L10779-10804`) | 1 without stacks | E-3.3 | Compat |
| D-030 | Own-hit and same-shot timing | a hit sees its own procs and earlier pellets' procs for every debuff (`timelineStatus`, `JS L10093-10136`) | procs apply after the shot's damage; CO counts earlier pellets | §7.1 | Compat |
| D-031 | Target-side terms on ticks | Viral, Magnetic, armor strip frozen at the inducing hit (`JS L10883-11363`; `I:enemy` §9.2) | read when each tick lands | §7.2 | Compat |
| D-032 | Puncture duration | 6 s (`JS L5547`) | 10 s | table 4.1 | Compat |
| D-033 | Impact duration | 1 s (`JS L5545`) | 6 s per stack (UNVERIFIED: the general table's "1 second" is the flinch) | table 4.1 | — |
| D-034 | Blast | unlimited stacks, no detonation, damage `0.3·` × `parseInt(1.5·(1 + SD))` × (1 + Blast element mods), never in TTK (`JS L5571`, `L9177`, `L11291-11361`, `L13669`) | 1.5 s × (1 + SD) fuse, one 0.3·H hit, detonation at 10 (4), no elemental scaling (§11) | E-3.9 | Compat |
| D-035 | Gas | one Gas proc per hit credited `gasStacksOnTarget × tick`; drops the non-crit headshot factor; extra faction in the non-crit path; TTK skips new clouds beyond 10 instead of replacing (`JS L11219-11290`, `L13682-13687`) | own timers, N ticks at t + 0 .., cap 10 with replacement, consolidation (§12) | §12, E-3.5 | — |
| D-036 | Duration rounding | `parseInt(D0·(1 + SD))` for every status, debuffs, Blast "ticks" and Toxic Lash included (`JS L9085-9093`) | ticks floored; debuff rounding unknown | E-3.1 | Compat |
| D-037 | Toxic Lash DoT duration | fixed 7.5 s (7 ticks credited, 8 in TTK) (`JS L5555-5561`, `L13706-13713`) | `floor(6(1 + SD))` Toxin ticks; Saryn's passive +25 % SD | Toxic Lash's forced Toxin procs get SD + 0.25 (with T-16) | — |
| D-041 | Mecha Empowered and Eclipse on DoT | Mecha squared on elemental DoT, once on Slash; Eclipse once on elemental DoT, not on Slash; Mecha unconditional | Mecha twice on every DoT while marked; Eclipse once on every DoT (in Y) | §9.2, §9.4 | — |
| D-053 | Status cap of capped enemies | `maxCountProc` drops same-type procs beyond the cap inside one hit and caps live stacks, never DoT damage; forced procs bypass the per-hit check (`JS L10011-10031`, `L10119-10127`) | 4 stacks of any status (U27.3) | E-3.2 and §6.2 for every status incl. DoT instances and the Heat pool | Compat |
| D-054 | Magnetic break proc | not modelled | forced Electricity proc on SH/OG break (§13) | E-3.10 in layered passes | — |
| D-069 | Per-status status damage | Emerald Toxin etc. added to the product `zb`, outside its other factors (`JS L11003`, `L11084`) | additive with Elementalist in one bucket | `(1 + A_sd + A_sd,s)` | — |
| D-070 | Hunter Munitions proc | DoT credited, never in the timeline (`JS L10761-10766`) | a Slash status | counts for stacks and n | — |
| D-073 | Primary Debilitate faction | faction a third time on its procs (`JS L11459`) | overview: ordinary procs, twice; `W:Primary_Debilitate` §Notes: three times | twice (overview decision); proposal to adopt three times: OQ-313 | — |
| D-074 | Nightwatch Napalm | DoT base 75 overwritten by 150, progenitor scaling, credited at hit, halves armor (`JS L10409-10441`) | blaze 150 Heat per tick, 6 ticks from 0 s, no crit; its Heat procs last 3 s base (`W:Nightwatch_Napalm`) | Heat procs D0 = 3 s (EC-03-55); blaze per `06` | — |
| D-301 | Gas elemental bonus | Leaded Gas stored as `addGas 3` plus `status_damage_gas 3`, added to the status-damage sum: tick × `(1 + A_sd + 3)` (`JS L1824-1830`, `L10462`) | Gas Damage Bonus is its own factor: `0.5 × MB × (1 + 3) × (1 + F) × (1 + A_sd)` (`W:Damage/Gas_Damage` example 2) | E^dot_GAS (§9.3); compat moves literal Gas bonuses into A_sd,GAS | Compat |
| D-302 | Microwave duration | 1 s (`JS L5578`) | infinite (`W:Microwave`) | ∞ (§8.7) | — |
| D-303 | Finisher mods on Slash ticks | attacks flagged `enableFinisherModsWithSlash` multiply Slash ticks by `1 + finisherDmg` (`JS L10471-10473`) | finisher mods do not affect Cinematic Slash ticks (`W:Damage/Slash_Damage`) | never | Compat |
| D-304 | External Viral on capped enemies | with no own Viral stack the option's N is used uncapped (up to 10 on a 4-capped boss); with own stacks capped at `maxCountProc` only above 10 (`JS L10782-10798`) | the enemy cap applies to all Viral stacks | `min(N_ext + n_own, cap_VIR)` (§6.3, T-06) | Compat |
| D-305 | Evensong Puncture cap | 7 forced Puncture procs per hit, but live stacks capped at 5 (`JS L12851-12853`, `L10119-10127`): +25 % crit chance at most | 7 stacks (+35 %), 5 on capped and overguarded enemies (`W:Evensong`) | E-3.2 Evensong rule | Compat |

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
| OQ-301 | Is the "+1" accumulator multiplied by the crit and body-part multipliers? `W:Damage/Calculation` lines 632-633 store crit inside the seed ("critical seeds of 80.0195389"), so the game form is `(Σ S·c_k·Part + 1)·C·M`. **Proposal to the manager:** change E-13 to `seed = Y·(1 + F)·G·c_k·Part_dot`, `tick = (Σ seed + 1)·K_s·(1 + E^dot)·(1 + F)·(1 + A_sd)·M_dot` | E-13 as fixed by the overview; consolidated groups use the oldest proc's multiplier for the accumulator (E-3.5) | single low-damage critical Slash proc (seed 40, CDM 2.0): 28.36 per tick if crit is in the seed, 28.71 if outside |
| OQ-302 | Heat pulse grid: anchored to the first ignition, or restarted by refreshes? Does a joining proc contribute to a pulse less than 1 s after it? | grid anchored at ignition; joins from the next pulse | Heat proc at t = 0 and a second at t = 0.5 with distinct seeds; read the damage numbers' times and values |
| OQ-303 | Heat strip: does re-ignition during the ramp-down continue from the current step? Which SD sets the step interval when sources differ? | continue from the current step; SD of the longest-duration proc | armor readout (enemy info) of a level-locked Simulacrum target, re-igniting 3 s after a Heat episode ends; mixed-SD sources |
| OQ-304 | Does the 4-stack cap of status-capped enemies limit Heat's pool? | yes, oldest leaves | Heat tick values on an Acolyte after 1 to 8 Heat procs within 6 s |
| OQ-305 | After Frozen, do the 3 remaining Cold stacks have fresh timers? What `cold_cd` applies at 10 stacks on enemies that cannot freeze? | fresh timers; 0.50 | time the Cold icons after thaw; crit numbers on a Demolisher with 10 Cold stacks |
| OQ-306 | Blast: is H the modded-base product of the Blast page or the whole hit damage? Is the host's detonation damage one instance per stack or one combined instance? Is `max(1, 0.3·H)` real? | modded-base product; one instance per stack; yes | Blast proc numbers from a weapon with large elemental mods (H differs strongly); detonation on an armored target at low damage (armor floor of 1 per instance) |
| OQ-307 | Electricity and Gas tick count at fractional SD (`floor(6(1+SD))` vs ticks at 0 .. ceil(D) − 1) and at negative SD (2016 table: Tesla Chain still occurs) | floor; nullified at SD ≤ −1 | count Electricity ticks with +55 % SD (9 vs 10) and with a −100 % SD riven |
| OQ-308 | Magnetic break proc: equal split into ticks? total factor `N/6` or exactly `1 + SD`? crit? "+1"? which stack's source? n before or after the breaking hit's own Magnetic proc? | equal split; N/6; no crit; no +1; newest stack's source; before | break a 1000-shield Corpus unit with 10 Magnetic stacks and known Electricity and status-damage mods; read the six tick numbers |
| OQ-309 | Knockdown duration; Lifted `M(c)` (Lifted page) or `1 + M(c)` (Melee page caption); do Knockdown and Lifted count for CO on overguarded enemies? | 3 s; M(c); yes | CO damage numbers immediately and 2, 3, 4 s after a forced Knockdown or Lifted; same on an Eximus with overguard |
| OQ-310 | Does Stun count for CO (Status_Effect page) or not (CO Mechanic page)? Do weapon and stance data use "Stagger" for Impact procs? | no; no | CO damage numbers after a forced Stun; compare Impact-icon presence after a "Stagger" stance hit |
| OQ-311 | Void: stacks and SD scaling; Tau: SD scaling, and whether a Tau weapon's own stacks raise its own status chance | one refreshing instance, SD scales; Tau SD scales and self-applies | status-icon timers with a Lasting Sting family mod; Haalvu proc counts against 0 and 10 own Tau stacks |
| OQ-312 | Are Sonar weak-spot multipliers inherited by DoTs (Sonar page lines 25-26: yes; DoT and Slash pages: no)? | no (overview E-13) | Slash tick numbers from hits on a Sonar spot vs the same body part without Sonar |
| OQ-313 | Primary Debilitate's extra procs: faction twice (overview D-073) or three times (`W:Primary_Debilitate` §Notes)? **Proposal:** adopt three times (game page), which removes D-073 | twice | Toxin tick of a Debilitate-added proc vs an ordinary Toxin proc with Primed Bane |
| OQ-314 | Puncture cap on non-VIP overguarded enemies (Evensong page implies 4); Kuva Lich Impact cap after U33.6 | 5; 5 | Puncture stack count on an Eximus with overguard after many procs; Impact stacks on a Lich |
| OQ-315 | T-26 `heatInherit`: should it carry the external source's faction bonus? **Proposal:** add a companion input `heatInheritFaction` (default 0) to T-26 | F_inh = 0 in the game profile | none needed (UI semantics); game rule is sourced |
| OQ-316 | Which non-mod elemental sources scale their element's DoT: Violet shard Electricity, riven or spliced Gas, custom `heatAdd`, Fireball Frenzy, Conductive Sphere field Electricity? | all yes except buffing-field elements | Electricity tick with and without a Violet shard; Gas tick with a spliced-Gas riven |
| OQ-317 | Do several Emerald shards add their Corrosive cap bonuses (14-stack text) and do they raise a boss's cap of 4 to 6 or 7 per shard? | additive; yes | count Corrosive stacks with two Emerald shards; on an Archon |
| OQ-318 | Bubonico alternate-fire "+9 s pre-mods" status duration: what is it added to? | not modelled | Bubonico alt-fire proc durations in game |

---

## 19. Test vectors

All values computed with `python3` (f64; Q(·) per E-4 in numpy float32). Tolerance for the engine: 1e-9 relative.
"Weapon side" means before E-12's enemy side. a = 1 unless stated; compat values use a = 0.

**TV-03-01 Slash tick (E-13 single proc, overview §3.8 step 6).** Y = 35 × 2.65 = 92.75, F = 0.55, G = b = 1,
A_sd = 0, Part_dot = 1, M_dot = 1, SD = 0. S = 143.7625; X(k=0) = 0.35 × 1.55 = 0.5425. Tick k = 0: **78.533656**
(a = 0: 77.991156). Tick k = 1 with CDM given as 4.4010749: **345.632503** (the overview prints 345.632502). Ticks at t + 1 .. t + 6 (N = 6).
Cinematic: lands on health unchanged by armor and type modifier.

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
ΣCD = 0). X_A = 0.8, X_B = 0.8 × 2.0004885197. Pulse after both joined: A older → **96.815633** (page 96.8156357);
B older → **97.616023**. A alone: (40 + 1) × 0.8 = **32.8** (page Supra Vandal example).

**TV-03-05 Gas same-pulse consolidation (`W:Damage/Calculation` line 632).** Nine Gas procs from one shot, seed 40
each, no bonuses (X = 0.5), six critical with Q(2.0): oldest non-critical → **300.558622** (page 300.558594 in f32);
oldest critical → **301.058867**. Each pulse m = 0 .. 5 has this value (all nine land together).

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
H = 240.25; expiry hit at t + 1.5 = **72.075**; SD = 1: fuse 3.0 s. Tenth stack at once: host **720.75** as ten
instances of 72.075, others (not evaluated) 3 × 2402.5 = **7207.5**. Status-capped enemy: detonation at the 4th
stack, host 4 × 72.075 = **288.3**. With k = 1, CDM = Q(2.0): H = 480.617367, hit **144.185210**.

**TV-03-13 Durations and tick counts (E-3.1).**

| SD | N (SLA/TOX/ELE/GAS) | D_COR | D_BLA | h_up | h_dn | D_frz | D_VIR | D_PUN |
|---|---|---|---|---|---|---|---|---|
| −0.5 | 3 | 4.0 | 0.75 | 0.25 | 0.75 | 1.5 | 3.0 | 5.0 |
| 0 | 6 | 8.0 | 1.5 | 0.5 | 1.5 | 3.0 | 6.0 | 10.0 |
| 0.55 | 9 | 12.4 | 2.325 | 0.775 | 2.325 | 4.65 | 9.3 | 15.5 |
| 1.0 | 12 | 16.0 | 3.0 | 1.0 | 3.0 | 6.0 | 12.0 | 20.0 |
| 1.1 | 12 | 16.8 | 3.15 | 1.05 | 3.15 | 6.3 | 12.6 | 21.0 |
| 0.3 + 0.7 (f64 sum 0.9999999999999999) | 12 | 16.0 | 3.0 | 1.0 | 3.0 | 6.0 | 12.0 | 20.0 |
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
from 9.9: 0. With the enemy flagged `og_present`: cap 4, `cold_cd` stays 0.25 from the 4th proc on, no Frozen.

**TV-03-17 Heat episode pulses (E-3.7).** SD = 0, a = 1, Heat procs at t = 0 (S = 100, X = 0.5) and t = 2.5 (S = 200,
X = 0.5). Episode t_ign = 0, t_exp = 8.5. Pulses t = 1, 2: **50.5**; t = 3 .. 8: **150.5** each; total **1004**.
Strip: up-steps 0.5 .. 2.0; episode end 8.5; down-steps at 10.0, 11.5, 13.0, 14.5: on 1000 initial armor 500 →
594.6036 → 707.1068 → 840.8964 → 1000. Inspiration (D-027, own timers, a = 0): 6 × 50 + 6 × 100 = 900.
