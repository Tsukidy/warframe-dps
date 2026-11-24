Status: reviewed (spec)

# Damage-calculation spec, part 08: evaluation and outputs

This document turns the per-hit formulas of `01` to `07` into numbers over time. It specifies the two evaluators of
overview section 4 in full: (a) the deterministic expected-value (EV) evaluator, including the Poisson stack model, its
independence assumptions, the window walk, the Heat model, the sequential-pellet distinct count and Little's law; and
(b) the event-driven Monte Carlo (MC) simulator, including its event set and order, the random number generator and its
keying, and its two passes. It defines the layer views, time to kill (TTK), every output of overview section 4.5 with an
exact formula, attack groups and "combine attack effects", the compatibility profile and the oracle procedure, the
EV-versus-MC tolerance, and the metrics the optimizer scores (not the search).

Related documents: [00-overview.md](00-overview.md) (notation, evaluation decisions this document implements, register),
[02-crit-status-multishot.md](02-crit-status-multishot.md) (crit tiers, proc counts and their EV forms, mechanic ids),
[03-status-effects.md](03-status-effects.md) (durations, caps, event order §7, DoT ticks, Heat, Blast),
[04-conditional-bonuses-and-buffs.md](04-conditional-bonuses-and-buffs.md) (CO family, condition model, extra hits),
[05-enemies.md](05-enemies.md) (enemy side, layer routing E-5.25, views §18), [06-ranged-firing-model.md](06-ranged-firing-model.md)
(ranged schedule, steady means), `07-melee.md` (melee schedule, combo), `01-damage-composition.md`, `09-build-rules.md`.

Contents:

- [1. Scope](#1-scope)
- [2. Inputs](#2-inputs)
- [3. Outputs and symbols added](#3-outputs-and-symbols-added)
- [4. Common model](#4-common-model)
- [5. Evaluator (a): expected value](#5-evaluator-a-expected-value)
- [6. Evaluator (b): Monte Carlo](#6-evaluator-b-monte-carlo)
- [7. Time to kill](#7-time-to-kill)
- [8. Outputs](#8-outputs)
- [9. Attack groups and combine attack effects](#9-attack-groups-and-combine-attack-effects)
- [10. Optimizer metrics](#10-optimizer-metrics)
- [11. Compatibility profile and oracle procedure](#11-compatibility-profile-and-oracle-procedure)
- [12. EV versus MC: tolerance and gap classes](#12-ev-versus-mc-tolerance-and-gap-classes)
- [13. Worked example](#13-worked-example)
- [14. Edge cases](#14-edge-cases)
- [15. Inspiration differences](#15-inspiration-differences)
- [16. Open questions](#16-open-questions)
- [17. Test vectors](#17-test-vectors)
- [18. Review notes](#18-review-notes)

---

## 1. Scope

**Owns** (overview 7.1, row `08`): the EV algorithm (overview section 4.2 in full: Poisson stack model, independence,
window walk, Heat window mean, sequential-pellet n, Little's law); the MC simulator (event set and order, RNG choice and
keying, two passes); layer views; TTK (layered, horizon, extrapolation, EV approximation); every output of overview
section 4.5 with exact formulas; attack groups and "combine attack effects" (T-22); the compatibility profile and the
oracle procedure (tolerances, oracle-safe scenarios); the EV-versus-MC tolerance; optimizer metric definitions (DPS
variants, TTK, max hit).

**Not owned**: every game mechanic. Per-hit damage (E-1 to E-15) is owned by `01` to `05`; schedules by `06` and `07`;
proc counts and their per-instance expectations by `02`; durations, caps, the event order inside one time point and DoT
values by `03`; condition semantics by `04`; layer routing E-5.25 by `05`. The optimizer's search algorithm belongs to
`docs/architecture/`. Where this document restates a rule of another document it cites it and adds only the evaluation
procedure.

**Stages.** No stage formula of overview 3.1 is owned here. This document drives ST-9 (layer depletion for TTK) and
decides at which time point, and with which target-state distribution, every stage is evaluated.

---

## 2. Inputs

| Input | Owner | Use |
|---|---|---|
| Resolved weapon-side stats per attack and instance kind (ST-1 to ST-7: `ratio_q[t]`, Y, F, G, G_F, CC, CDM, SC, w_s, φ_s, MS) as functions of the target state and condition state | `01`, `02`, `04` | per-instance damage and procs |
| Per-instance proc expectations `E[N_s]`, crit-weighted proc mass, `A_s` (any proc of s), `G(z)`, `P(k)` | `02` E-2.7, E-2.12, E-2.18, 4.9 | proc rates, presence, DoT mass |
| Durations D_s, tick counts N_s, first-tick delays δ_s, caps cap_s, replacement, event order P1-P4, tick groups and "+1", Heat episode E-3.7, strip E-3.8, Blast E-3.9, Magnetic break E-3.10, distinct count E-3.12 | `03` | stack model, DoT accounting, MC events |
| CO family, sequential-pellet rule E-4.2, condition classes and state machine E-4.8, EV steady state E-4.9, effect fields (§4.2), partial uptimes (§4.3), Secondary Enervate E-4.21, extra hits E-4.13, E-4.13a, E-4.14, Melee Duplicate E-4.15 | `04` | CO term, conditional stacks |
| Enemy pools, views (§18), effective armor E-5.10 with flat strips E-5.11 and Hydroid E-5.12, routing E-5.25, shield gate, overflow, attenuation, adaptation, per-type minimum of 1 | `05` | views, TTK |
| Schedule records (shots, reloads, hit times, flags, children, lingering ticks), EFR, N_mag, T_cyc, ρ, steady means of time-dependent multipliers; melee loops and periodic plans (`07` E-7.37) | `06` §6, §9, §10; `07` (melee) | instance stream, steady metrics |
| Toggles T-01 to T-29, `t_win`, `t_ttk_max`, `n_runs`, `seed` | overview 6, 2.4 | evaluation settings |

---

## 3. Outputs and symbols added

Overview symbols used with their overview meaning: `t_win`, `t_ttk_max`, `proc_rate[s]` (λ_s), `stack_mean[s]` (μ_s),
`n_runs`, `seed`, `stacks[s]` (n_s), `n_distinct` (n).

**Symbols added** (overview 0.3: domain prefix):

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `ev_event` | e | – | proc event: the set of damage instances whose procs are applied in one P4b step (one shot, 4.2) |
| `ev_eta[s][e]` | η_{s,e} | count | expected procs of s applied by event e (all its instances; 02 E-2.18) |
| `ev_m[s][e]` | m_{s,e} | count | crit-weighted proc count of s of event e: E[Σ over procs q of s of c_q] (the "DoT mass" of `02` 4.9) |
| `ev_pi[s][e]` | π_{s,e} | fraction | `1 − exp(−η_{s,e})`: probability that e applies ≥ 1 proc of s under the Poisson model |
| `ev_heat_v[e]` | v_e | HP | expected weapon-side Heat pool value added by e: Σ over its Heat procs of S_q·X_q (E-3.5 notation) |
| `ev_ign[e]`, `ev_end[e]` | ι_e, ω_e | fraction | probability that e ignites a Heat episode; that the episode whose last proc is e ends at t_e + D_HEA |
| `ev_noheat(A)` | Z(A) | fraction | probability that no event of the set A applies a Heat proc (E-8.5) |
| `ev_ign_rate` | r_ign | 1/s | steady ignition rate of Heat episodes (E-8.7) |
| `ev_kh[x](τ)` | P_kh(x; τ) | fraction | probability that the Heat strip counter k_h equals x ∈ {0..4} at τ |
| `ev_surv[s](ι, u)` | R_s(ι, u) | count | expected number of the DoT instances of s applied by instance ι that are not replaced by time u, per expected proc (caps, `03` §6.2; E-8.11) |
| `ev_nj` | n̄_j | count | expected distinct-status count seen by instance j of a shot (E-8.10) |
| `ev_n_ext` | n_ext | count | external status types the scenario declares present for the whole window and that no status group of E-8.9 counts; 0 in every current scenario (T-06's Viral enters through `present_VIR ≡ 1`, not here) |
| `ev_kbar(n_P)` | k̄(n_P) | count | E[k \| CC(n_P)], the expected crit tier at Puncture count n_P (`02` E-2.7, E-2.9) |
| `ev_period` | T_per | s | length of one steady period of a unit (E-8.13) |
| `rng_mech(name)` | – | u32 | FNV-1a 32-bit hash of a mechanic id string (E-8.16) |
| `mc_anchor` | (a, i, j) | – | attack index, shot index, sub-instance index of a random draw (E-8.16) |
| `mc_pass` | V, L, L0 | – | view pass, layered pass, layered pass with armor ignored (6.4) |
| `mc_time` | τ̂ | µs (integer) | event time on the integer grid of 4.3 (E-8.0a), used by both evaluators |
| `out_bin` | b | s | width of `dps_series` bins, default 1 |
| `ttk_ev`, `ttk_kappa` | T_EV, κ | s, fraction | EV TTK (event time) and overkill margin at T_EV (7.2) |
| `ttk_extrap` | – | bool | EV TTK extrapolated beyond `t_ttk_max` |
| `par_hit` | – | record | per parent hit parity record (8.8) |
| `grp` | 𝒢 | set | attack group (section 9) |
| `out_dot_landed`, `out_dot_pending`, `out_dot_credited` | – | HP | DoT accounting quantities (4.6) |

Local names (not symbols): `qnt(p)` is the sample quantile of E-8.19; `n_max` is the cap on the distinct-status count
of `03` E-3.12 (16 or 17 types).

**Output IDs added** (the output-ID namespace of overview 4.1, `dps_*`, which has no domain prefix):

| Output ID | Unit | Defined in |
|---|---|---|
| `dmg_per_shot[v]`, `dmg_per_mag[v]` | HP | 8.5 |
| `dps_series[v][k]` | HP/s | 8.1 |
| `out_dot_pending[v]` | HP | 8.7 (the symbol `out_dot_pending` of 4.6, per view) |
| `uptime[s][k]`, `stacks_mean[s][k]`, `crit_tier_dist[x]` | fraction, count, fraction | 8.3, 8.4 |
| `ttk_mc` (P_kill, median, mean of killed runs, qnt(0.05), qnt(0.95)), `shots_to_kill`, `ammo_to_kill`, `shield_regen_possible` | –, s, count, rounds, bool | 7.1, 8.6 |
| `ttk_ev` (T_EV), `ttk_kappa` (κ), `ttk_extrap` | s, fraction, bool | 7.2, 8.6 |
| `dev[v]` and its flag | HP/s, bool | 8.11 |
| `queue_dps_parity`, `avg_direct`, `avg_status`, `avg_total`, `median_total`, `ttk_parity` | HP/s, HP, s | 8.8 (overview 4.5 parity rows) |
| `hit_infight[v]`, `hit_fresh[v]`, `ammo_window`, `ttk_noarmor` | HP, rounds, s | section 10 |

Outputs: every quantity of section 8; the optimizer metrics of section 10.

---

## 4. Common model

### 4.1 Scenario and evaluation unit

A **scenario** is (build, enemy and its options, toggles T-01 to T-29, buffs, `t_win`, `t_ttk_max`, `n_runs`, `seed`)
(overview 4.1). An **evaluation unit** is either one attack a of the weapon, or one attack group 𝒢 with T-22 on
(section 9). Both evaluators read the same resolved stats and the same schedules. The default units are every attack of
the weapon on its own (CD-12, `I:agg` §7.5). Every per-unit quantity of this document (instances, proc events, `D_per`,
`hit_fresh`, breakdowns) ranges over **the instances of the unit's own attacks only**: with T-22 off a parent unit does
not contain its children's instances, and a child unit contains only the child's instances (section 9).

Hit model (normative for both evaluators): every pellet, merged beam tick, melee hit, child instance and extra hit of
the schedule hits the single target (`W:Damage/Calculation` §Average Shot/Hit, "if all projectiles hit the target").
No accuracy, spread or miss model exists; a per-attack hit fraction is proposed in OQ-808.

Data structures (normative content; field names are illustrative):

```text
Scenario       { build, enemy, toggles, t_win, t_ttk_max, n_runs, seed, out_bin, profile ∈ {game, compat} }
Unit           { attacks: [attack index], firing attacks, combined: bool }
Instance       { anchor (a, i, j), time τ, kind ∈ {direct, radial, extra_hit, duplicate, lingering_tick},
                 parent anchor (derived kinds), flags (first, last, heavy, weak_point, is_radial, ...),
                 schedule multipliers (stance S_j, combo M(c), sniper M_snp, ramp L_i, spool, first/last) }
ProcEvent e    { time t_e, order key (t_e, firing attack, shot i, child attack), instances [Instance] }  (one P4b step)
```

### 4.2 The instance stream

For each attack a of the unit the schedule generator (`06` E-6.11 ranged, `07` melee) emits shots i with fire time s_i
and hit time h_i, for the horizon `H = max(t_win, t_ttk_max)` (`06` 6.1). Damage instances and their anchors:

```text
E-8.0  Instances and anchors (attack index a = position in the weapon's attack list; shot index i from 0)
  non-beam ranged shot i of a:   pellets j = 0 .. n_pel − 1 at h_i                         anchor (a, i, j)
  beam tick i (HELD):            one merged instance j = 0 at h_i (CD-10)                   anchor (a, i, 0)
  melee hit i (07):              j = 0, melee multishot copies j = 1, 2 ...                 anchor (a, i, j)
  child attack c of shot i:      p = parent projectile index (0 for hitscan or HELD parents), q = copy index of
                                 the child's data multishot K_c (06 E-6.24), at h_i + delay (06 9.3)
                                                                                            anchor (c, i, p·K_c + q)
  lingering tick m of child c:   at h_spawn + m/FR_c (06 E-6.25)        anchor (c, i, ((p·K_c + q) << 8) | m),
                                 m < 256 and p·K_c + q < 255 (so j ≤ 0xFEFF; a lingering attack has ticks only, so
                                 its anchors never meet the child-instance anchors of the same attack)
  reload-triggered attack r:     shot index = number of the triggering reload (06 9.5)      anchor (r, i_reload, j)
  extra hit (E-14), Melee Duplicate (E-4.15): derived instances; they use their parent's anchor and prefixed
                                 mechanic ids (E-8.16)
  Xata's Whisper detonation extra hit (04 E-4.13a): derived from the Blast host instance that detonates; it uses the
                                 anchor of the instance whose proc created the detonating stack, mechanic prefix "xd/"
  range: every j of the rules above is < 0xFF00; j ≥ 0xFFFD and i = 0xFFFFF are sentinels (E-8.16); a schedule
         that needs i ≥ 0xFFFFF or j ≥ 0xFF00 is an error
```

A **proc event** e is the set of instances whose procs are applied in one P4b step of `03` §7.1: all instances of one
shot that have the same time. A child instance at the same time as its parent's pellets belongs to the parent's shot
and is processed after all pellets, in (p, q) order; a child instance at a later time forms its own event (Decision,
UNVERIFIED, OQ-803). In a combined group, events at the same time are ordered by (firing attack index, shot index,
child attack index).

A **Melee Duplicate copy** (`04` E-4.15) is an instance of its parent's shot, placed right after the parent and its
extra hits (`03` §7.1 P4a, which owns the order; `07` 8.4). As every later instance of a shot it counts the parent's
procs in its CO count n (E-8.10) but reads the debuffs of σ_S, the state before the shot (CD-04). It exists with
probability `p_dup·P(parent k = 1)`; its own procs join the event's η and m. With Toxic Lash or Xata's Whisper active
the copy is replaced by a copy of the extra hit of the earlier ability slot (`04` 6.4 bug, game behaviour), placed
right after that extra hit; the duplicated extra hit repeats the copied extra hit's value (Decision, OQ-811; TV-08-28).
`04` E-4.15 says the copy reads "C and the target state ... after the parent's procs"; for debuffs that contradicts
`03` §7.1, and this document follows the owner of the order (OQ-811).

### 4.3 Event order and tie-breaking

Both evaluators use the processing order of `03` §7.1 (P1 state transitions, P2 scheduled damage, P3 Heat episode end,
P4 shots with a/b/c/d) at every time point.

**Time model (Decision).** `06` E-6.11 builds fire times as cumulative f64 sums (`τ_{i+1} = s_i + 1/r`), so a time that
is equal in exact arithmetic can land a few ulp away (at 10 shots/s, the 61st shot is at 5.999999999999995 instead of
6, which would let a stack applied at t = 0 with D = 6 cover one shot too many and make results jump under tiny fire-rate
changes). Both evaluators therefore hold every event time on an integer grid of 1 µs:

```text
E-8.0a Time grid
  τ̂ = round_half_even(τ·10^6)  (i64)       applied once to every schedule time of 06/07 (fire, hit, reload, child,
                                            lingering tick) when the record is read
  offsets and durations of 03 (δ_s + m, D_s, h_up, h_dn, D_BLA, D_frz) and of 04 (effect durations) are converted the
                                            same way and added as integers: t̂_tick = t̂_p + round((δ_s + m)·10^6)
  every equality, ordering, expiry and window test is an exact integer comparison; floor((τ − t)/h) is the integer
  quotient of the grid values; "τ ≤ t_win + ε" (ε = 1e-9) is τ̂ ≤ t̂_win
```

The accumulated rounding of the `06` sums is below 2e-10 s over the longest schedule (600 s at up to 30 shots/s,
checked against exact rational arithmetic), more than three orders of magnitude below the half step, while every real
time difference of the schedules (burst delays, beam ticks, Heat steps) is at least milliseconds. Times that are equal
in exact arithmetic therefore get the same grid value unless their exact value lies within that rounding of a
half-microsecond boundary (a period whose length in µs has a fractional part within 2e-4 of 0.5); this residual case is
accepted (Decision). This realises `03` §7's "equality is exact" for times that are equal in exact arithmetic and the
intent of `06` E-6.11's ε = 1e-9 at once (OQ-807 asks `03` and `06` to restate their time rules in these terms). The
other ε = 1e-9 tests of this document (E-8.6, E-8.8) are grid comparisons.

**Expiry never cancels a tick.** A DoT instance removed in P1 at its expiry time keeps its tick scheduled at that time,
which lands in P2 (`03` E-3.6, §7.3: "Every scheduled tick lands, including a tick at the instance's own expiry
time"); only replacement or cap removal (`03` §6.2) and DoT detonation (`03` §9.7) cancel ticks. With SD = 0 the sixth
Slash or Toxin tick lands at `t_p + 6`, the expiry time (TV-08-18).

Additions owned here:

- Inside P2, tick groups are ordered by the creation sequence number of their oldest proc (`03` §7.1); a tick group of
  ELE or GAS consolidates instances whose ticks land at the identical grid time (`03` E-3.5).
- Inside P4, shots are processed in the order key of 4.2. Condition-stack gains (`04` E-4.8) triggered by the
  instances of a shot are applied in its P4b, after its procs, so sibling instances do not see them (CD-04 reading of
  `04` E-4.8 "value(τ⁻)").
- MC heap key: `(τ̂, phase, sub_key, seq)` with phase P1 = 1, P2 = 2, P3 = 3, P4 = 4; sub_key = the 4.2 order key
  (firing attack index, shot index, child attack index) in P4, the creation sequence number of the oldest proc of the
  tick group in P2, 0 in P1 and P3; seq a per-run counter incremented on every event creation (last tie-break). Shots
  inserted lazily from several firing attacks therefore follow 4.2 whatever their creation order. An event created while
  processing (τ, P) for the same time τ and a phase ≤ P is processed inside that step (`03` §7.1: "Events that step b
  or c creates at time τ are processed inside that step"); one created for a later phase of τ, or a later time, waits
  for its key.

### 4.4 Window, horizon and fresh state

```text
E-8.1a Window membership
  damage counts toward window metrics iff its landing time τ satisfies τ ≤ t_win + ε            (ε = 1e-9)
  instances: τ = hit time (06 8.5, 9.3); DoT ticks and Heat pulses: their landing time; Blast hits: fuse expiry or
  detonation time
  a shot belongs to the window iff s_i ≤ t_win + ε (06 6.3); its instances count only by their own hit time
  an instance is processed (damage, rolls, procs) only if its hit time is ≤ t_win + ε in the view pass and ≤
  t_ttk_max + ε in the layered passes; an instance of a window shot that lands later does not exist there, so its
  procs do not exist either (EC-08-02); the same rule decides which instances are parity hits (8.8)
  pending DoT (4.6) comes only from procs applied at or before t_win + ε: the EV continuation after t_win adds no proc
  and no event, and target-side terms keep evolving by expiry only
```

Every evaluation starts from the fresh state of overview 4.1 (no statuses; full magazine or Incarnon form magazine;
combo at Initial Combo; sniper combo 0; ramp and spool at their start; conditional stacks per T-02). `t_win` default 20 s
(T-01, `JS L5496`), `t_ttk_max` default 60 s (overview 2.4).

### 4.5 Layer views and the shared status timeline

```text
E-8.1b Views (05 §18)
  overguard : exists iff OG_max > 0;  instance damage Σ_t D^OG_t                         (05 E-5.17)
  shield    : exists iff SH_max > 0;  Σ_{t ∉ B_SH} D^SH_t   (bypass types excluded)
  armor     : exists iff A0 > 0;      Σ_t D^HP_t with armor (E-15) and the per-type minimum of 1
  health    : always;                 Σ_t D^HP_t with arm_t = 1 and no minimum of 1 ("100 % armor stripped")
  layered   : the real depletion OG → SH → HP (E-5.25); TTK only (section 7)
```

The four non-depleting views see one shared status timeline (overview 4.1): the procs, stacks and conditions of a run
(MC) or their distributions (EV) are the same in every view; a view only changes the enemy-side layer rules of each
instance. Caps that depend on overguard (Cold 4, VIP Puncture 3) follow `03` E-3.2, the owner of caps, which is
normative for the engine here and which `05` §18 restates: in the shared view timeline and in the EV evaluator
`og_present` is true for an enemy that spawns with OG_max > 0 (the caps apply for the whole window, in every view); in
the layered passes it is the live state (OG > 0 at the moment of application). The rule is a Decision of `03`
(UNVERIFIED); `03` OQ-321 proposes to the manager a per-view exception (`overguard` view with the caps, the other views
without), recorded here as OQ-804. Unique damage reduction in views: attenuation and M_dps apply per instance;
adaptation and per-hit caps only in the layered pass (`05` 15.4).

Health-threshold conditions (class (a) of T-02, e.g. Kunai Incarnon Swift Conclusion, Dread Incarnon Hitman's
Opportunity) follow T-02 (overview 6.2; `04` §2.2, 4.3):

| T-02 | Non-depleting views (and every steady metric) | Layered passes (TTK) |
|---|---|---|
| `all_max` (default) | predicate = 1 ("every threshold true") | predicate = 1 |
| `none` | predicate = 0 | predicate = 0 |
| `simulated` | predicate = 0 (the target stays at full health) | from the live (MC) or expected (EV) health before the instance |

### 4.6 DoT accounting

| Quantity | Definition |
|---|---|
| landed (`out_dot_landed`) | each tick, Heat pulse or Blast hit counts at its landing time (window test E-8.1a); used by every metric of this spec |
| pending (`out_dot_pending`) | damage landed after `t_win` by procs applied at or before `t_win` (the view pass continues until no DoT, pulse or fuse remains, 6.4; no proc after t_win, E-8.1a) |
| credited (`out_dot_credited`, game profile) | for each proc, the sum of the landed values of its own ticks, whenever they land (window ignored); a Heat pulse or consolidated ELE/GAS group is split over its procs in proportion to their weapon-side terms `S_q·X_q`, the "+1" term `a·X_{q₀}` going to the oldest proc q₀; a Blast hit to its stack |
| credited (compat profile) | the inspiration's formula: per proc `curr_duration × one-tick value` evaluated with the target state at the inducing hit (`JS L10982-10986`, `I:queue` §4, §6.7; D-086) |

Credited values are used only by parity outputs (8.8) and by the hit metrics of section 10 (overview 4.1, D-086).

---

## 5. Evaluator (a): expected value

The EV evaluator computes expectations of the quantities that the MC simulator samples, under the model assumptions of
5.1. Within those assumptions every formula below is exact; every departure from the game is listed as an assumption
and as a gap class (section 12).

### 5.1 Model assumptions

| ID | Assumption | Status |
|---|---|---|
| A-1 | Random draws of different instances, shots and mechanics are independent; crit and status of one instance are independent (CD-03). The target state an instance sees is therefore independent of that instance's own draws. | game rule (`F:core` §7, §13) |
| A-2 | Live-proc counts of different status types are independent of each other at every time. | approximation: one pellet's proc types are multinomial (slightly negative correlation); crit-triggered procs correlate with crit (gap G-2) |
| A-3 | The number of live procs of s at τ, before capping, is Poisson with mean μ_s(τ) (overview 4.2). Per event, the count of s procs is treated as Poisson(η_{s,e}), so its presence probability is π_{s,e} = 1 − exp(−η_{s,e}). | approximation: exact for Poisson arrivals; deterministic forced procs and sparse Bernoulli procs deviate (gap G-1, OQ-801) |
| A-4 | Stack replacement (`03` §6.2) with one duration per status: the live set is the newest `min(N, cap_s)` procs, so `n_s = min(N_s, cap_s)`. | exact given equal D within one status and unit (proof: removals by expiry and by replacement both take the oldest); fails on capped targets when one status has procs of different durations in one unit (Toxic Lash Toxin procs carry +0.25 SD, D-037; Nightwatch Napalm Heat): gap G-14 |
| A-5 | The crit tier and Part of an inducing instance depend on the target state at t_e only through PUN and COL (`02` E-2.9), which no tick-time enemy factor reads, so they are independent of the tick-time state (A-1, A-2: different statuses). The CO term of a DoT seed reads presence indicators at t_e of statuses that the tick reads again at u (Viral, Corrosive and Heat through ARM, Magnetic): this same-status, two-time correlation is not covered by A-1 or A-2 and is computed exactly by E-8.11 (two-time group expectations, E-8.9 step 4b). Heat pulses versus the Heat strip are conditioned exactly by E-8.8. | exact for DoT ticks (E-8.11); for Heat pulses the seed-versus-pulse correlation of other statuses is not computed (A-11) |
| A-6 | Conditional stacks (`04` class (b)) are independent of the target statuses. | approximation (gap G-6) |
| A-7 | Nonlinear enemy-side functions that are not products of single-status terms (armor DR, minimum of 1, attenuation) are evaluated by enumerating the states they read, with every other factor at its conditional expectation (E-8.9). | exact for DR; approximation for the minimum of 1 and attenuation (gap G-5) |
| A-8 | Frozen: on targets that can freeze, `n_COL = 10` is read as Frozen (`cold_cd = 1.0`, `03` E-3.11); the thaw and the discarding of Cold procs while Frozen are not modelled. | approximation (gap G-3) |
| A-9 | Blast: each stack deals its host hit at fuse expiry; detonation timing and the clearing of stacks at detonation are ignored. Host damage per stack is unchanged by detonation (`03` E-3.9: one 0.3·H instance per stack either way). Xata's Whisper detonation extra hits (`04` E-4.13a) are not produced in EV. | approximation of timing (gap G-4); detonation extra hits omitted (gap G-15) |
| A-10 | Steady mode (5.9): stationarity with the **time-average** live count μ_s = λ_s·D_s (overview 4.2), which every consumer reads; rates that depend on the target state or on conditional stacks are solved as a joint fixed point (E-8.13). | definition of the steady metrics; a periodic schedule's consumers see the count at their arrival time, not the time average (gap G-13, OQ-806) |
| A-11 | Heat pulses (E-8.8): the CO term of each pooled proc's seed reads statuses at its proc time that the pulse reads again at its landing time; the pool value and the pulse-time enemy factor are multiplied as expectations (Heat's own presence is conditioned exactly, E-8.8). | approximation (gap G-12) |
| A-12 | Class (c) and (u) effects with uptime `u_e < 1` used nonlinearly are an on/off mixture (`04` 4.3) whose state is the same for an instance and every tick of the DoT it seeds (perfect correlation); MC draws an alternating on/off process (E-8.18). | approximation of the correlation in EV (gap G-17) |

### 5.2 Per-event expectations

For every proc event e (4.2), from the target-state distribution at t_e (5.3 to 5.5) and the condition state at t_e:

```text
E-8.2  Per-event quantities (sum over the instances of e, each from 02 E-2.18 and 4.9)
  η_{s,e}   = Σ_{instances} w_ι·E[N_s]                (rolled + forced + IB/HM/Seeking Talons + Welt + Encumber (02
                                                       e_s,i) + Debilitate + proc-triggered arcanes + the procs of the
                                                       instance's extra hits; merged beam ticks per 02 E-2.3)
  m_{s,e}   = Σ_{instances} w_ι·crit-weighted mass of s  (rolled: E[c]·E[N_s,rolled]; forced: E[c]·φ_s; HM-type:
                                                       C1 = E[c·1{k ≥ 1}] of 02 E-2.16)
  π_{s,e}   = 1 − exp(−η_{s,e})                       (A-3)
  v_e       = Σ_{Heat procs} E[S·X]                   = Σ_{instances ι of e} w_ι·S_ι·X_base,HEA·m_{HEA,ι}
                                                        (S, X of 03 E-3.5; X_base: X without c_k)
  w_ι       = existence probability of instance ι: 1, f = frac(MS) for pellet n of a random pellet count (04 E-4.2),
              p_dup·P(parent k = 1) for a Melee Duplicate copy (4.2), the T-03 setting for headshot-only children
  procs of an SD ≤ −1 source are nullified (03 §5.1): η_{s,e} = 0 for them (Blast: the host hit lands in P4b)
```

Sources: `W:Status_Effect` lines 491-496 ("Average Number of Procs Per Shot = Multishot × (Number of Forced Procs +
Status Chance per Projectile)", "per Second = per Shot × Fire Rate"); `02` 4.9.

### 5.3 Live-count distributions

```text
E-8.3  Capped Poisson distribution (A-3, A-4)
  N ~ Poisson(μ);  n = min(N, cap)
  P(n = k) = e^{−μ} μ^k / k!            k < cap
  P(n = cap) = 1 − Σ_{k<cap} P(n = k)                       (cap = ∞: no truncation; evaluate up to the
                                                             smallest K with P(N > K) < 1e-15)
  E[f(n)] = Σ_k P(n = k)·f(k)                               for every nonlinear f (Vir, Mag, corr_strip, cold_cd,
                                                             punc_cc, tau_sc, conditional predicates)
  presence P(n ≥ 1) = 1 − e^{−μ}
```

```text
E-8.4  Window mean μ_s(τ) for a consumer at time τ (direct instance in P4, tick in P2)
  μ_s(τ) = Σ_{events e before τ, t_e + D_{s,e} > τ} η_{s,e}          (grid times, E-8.0a)
  "before τ": t_e < τ, or t_e = τ and e precedes the consumer in the order of 4.3 (an instant-burst shot sees
  earlier shots at the same time; no instance sees its own event; a P2 tick at τ sees no event at τ)
  D_{s,e}: the modded duration of e's source (03 E-3.1); the stack lives on [t_e, t_e + D) (03 §7.2)
  external Viral (T-06): n_VIR = min(N_ext + N_own, cap_VIR), N_own ~ Poisson(μ_VIR); present_VIR ≡ 1
```

Special statuses (rules of `03`, distributions here):

| Status | EV distribution |
|---|---|
| IMP, PUN, RAD, MAG, VIR, COR, TAU, GAS (count) | E-8.3 with cap_s (E-3.2; overguard-dependent caps per 4.5) |
| SLA, TOX, ELE (count, for n and presence) | E-8.3 with cap_s (∞ on ordinary targets) |
| COL | E-8.3 with cap 10 (overguard-dependent cap per 4.5); n = 10 on a freezable target is read as Frozen (A-8) |
| BLA | presence `1 − e^{−μ_BLA}`, μ over the fuse duration D_BLA (A-9) |
| VOI | presence `1 − e^{−μ_VOI}` (single instance refreshed by every proc: present iff ≥ 1 proc in the last D) |
| MIC | presence `1 − exp(−Σ_{e before τ} η_{MIC,e})` (permanent) |
| KND, LIF | combined presence of the exclusive pair `1 − exp(−μ_KND − μ_LIF)`, each μ with its own duration; Lifted-only effects use `(μ_LIF/(μ_KND + μ_LIF))` of it (Decision, gap G-9) |
| HEA | presence and strip counter: 5.4; pool: E-8.8 |

### 5.4 Heat: presence, strip counter and pool

Heat has one episode per target whose pool grows with every proc while the episode is refreshed (`03` E-3.7), so it has
no steady state (`F:status` §4.4; overview 4.2). Presence and the strip counter are computed from per-event presence
probabilities π_e := π_{HEA,e} under A-1 and A-3. All Heat procs of a unit are assumed to have the duration D = the
largest D_HEA among its Heat sources, and the strip intervals h_up = 0.5·(1 + SD*), h_dn = 1.5·(1 + SD*) use the SD* of
that source (`03` E-3.8; Decision for mixed sources such as Nightwatch Napalm 3 s procs, gap G-8).

```text
E-8.5  Presence, ignition and end probabilities (events in the order of 4.3; "<" is that order)
  Z(A)   = Π_{e ∈ A} (1 − π_e)                                    no Heat proc in the event set A
  live for a consumer at τ in P4:   P_H(τ) = 1 − Z({e < τ : t_e > τ − D})          (= 1 − exp(−μ_HEA(τ)))
  live for a pulse at τ in P2:      the episode has t_exp ≥ τ (03 E-3.7: pulse at t_exp is processed before P3)
  ignition at e:   ι_e = π_e · Z({f < e : t_f > t_e − D})
  end after e:     ω_e = π_e · Z({f > e : t_f < t_e + D})        (the episode whose last proc is e ends at
                                                                  e_e = t_e + D, in P3 of that time)
```

```text
E-8.6  Strip-counter distribution P_kh(x; τ) for a consumer at τ (after P1 steps at τ), x ∈ {0, 1, 2, 3, 4}
  (all floors are integer quotients on the time grid, E-8.0a)
  k0 distribution of an ignition e (residual of the previous episode's ramp-down, 03 E-3.8):
     for each earlier event f with e_f = t_f + D ≤ t_e and t_e − e_f < 4·h_dn:
        P(ign at e, previous end at e_f) = ω_f · Z({g < e : t_g ≥ e_f}) · π_e
        with k0 = 4 − floor((t_e − e_f)/h_dn)
     P(ign at e, k0 = 0) = ι_e − Σ_{k0 > 0} P(ign at e, k0)
  live part (young episodes; an episode is live for at least D ≥ 4·h_up after ignition):
     for each ignition event e < τ with τ − t_e < 4·h_up and each k0:
        x = min(4, k0 + floor((τ − t_e)/h_up))              add P(ign at e, k0) to P_kh(x; τ)
     P_kh(4; τ) += P_H(τ) − Σ (young contributions above)    (every other live episode is at 4)
  dead part:
     for each event f with e_f ≤ τ and τ − e_f < 4·h_dn:
        x = 4 − floor((τ − e_f)/h_dn)                       add ω_f · Z({g < τ : t_g ≥ e_f}) to P_kh(x; τ)
     P_kh(0; τ) += (1 − P_H(τ)) − Σ (dead contributions above)
  heat_strip(x) = 1 − 0.5^(x/4)                             (03 E-3.3)
  conditioning (E-8.9 step 4b, E-8.11): P_kh(x; τ | no Heat proc in the event set W) is E-8.6 with π_f = 0 for f ∈ W
```

E-8.6 uses that every episode ends with k_h = 4 (an episode lasts at least D = 6(1 + SD*) > 4·h_up = 2(1 + SD*) and
the counter only rises while live) and that a ramp-down starts at the episode end (`03` E-3.8). The live and dead parts
are kept separately: the pair (x, live) is the ARM group state of E-8.9 (Heat presence counts in n, x sets the strip).
The formulas were checked against brute-force enumeration and simulation of E-3.8 (TV-08-07). For targets immune to
armor reduction (`05` 9.6), the strip factor is 1 in every state; for Heat-immune targets no Heat proc exists.

```text
E-8.7  Steady strip-counter distribution (5.9; continuous-time stationary Poisson stream of rate λ = λ_HEA)
  P(dead) = e^{−λD};  ignition rate r_ign = λ·e^{−λD}
  k0 / ramp-down weights:  w_j = e^{−λ j h_dn} − e^{−λ (j+1) h_dn}  (j = 0..3 → value 4 − j),  w_0' = e^{−4 λ h_dn} (value 0)
  dead:  P(dead, x = 4 − j) = e^{−λD}·w_j (j = 0..3);   P(dead, 0) = e^{−λD}·e^{−4 λ h_dn}
  live:  P(live, y) = r_ign·h_up·Σ_{c ≤ y} W(c)  for y < 4,   W(c) = weight of k0 = c (W(4 − j) = w_j, W(0) = w_0')
         P(live, 4) = (1 − e^{−λD}) − Σ_{y<4} P(live, y)
```

```text
E-8.8  Heat pool damage, window (per-ignition decomposition; exact under A-1, A-3 and A-11)
  For each event k with ι_k > 0:
    forward state list L = [(t_l, P_l, W_l)]   start: [(t_k, 1, v_k^ign/π_k)]     (conditioned on Heat at k)
    pulses g = t_k + m, m = 1, 2, ... while g ≤ (window or horizon end) + ε:
      first process every event j > k with t_j < g, in order:
          drop entries with t_l + D ≤ t_j                       (episode ended in P3 before P4 of t_j)
          P_live = Σ P_l,  W_live = Σ W_l
          every entry: P_l ← (1 − π_j)·P_l,  W_l ← (1 − π_j)·W_l
          append (t_j, π_j·P_live, π_j·W_live + v_j^live·P_live)   (a proc of j joins only a live episode)
      live entries at the pulse: t_l + D ≥ g (grid comparison)
      weapon-side pulse value  V_k(g) = Σ_live W_l + a·X̄_k·Σ_live P_l           (a = dot_acc, 03 E-3.5)
      X̄_k = X_base·E[c](t_k)                                  (the igniter is the oldest proc; 03 E-3.5)
      stop when no entry is live
    v_k^ign, v_j^live: v of E-8.2 with the Heat presence indicator of the seeds' CO count (E-8.10) fixed at
      present_HEA = 0 for the igniting event (no Heat was live before it) and 1 for a joining event (the episode
      is live); the earlier-sibling Heat term of E-8.10 is unchanged (exact: given ignition or a live episode, the
      Heat presence before e is known)
    pulse damage on view v: V_k(g) · E[enemy factor of a Heat instance at g | episode ignited at k]
      armor views: k_h = min(4, k0 + floor((g − t_k)/h_up)) with the k0 distribution of E-8.6, joint with
      n_COR (E-8.9); other factors at time g (Viral, Magnetic, vulnerability)
  E[Heat landed in [0, T]] = Σ_k ι_k · Σ_{g ≤ T} E[pulse damage]
  Status-capped targets (cap_HEA = 4): Σ_live W_l is replaced by (W/N̄)·E[min(Np, cap)], with N̄ the analogous
  expected-count sum (v replaced by η) and Np ~ 1 + Poisson(N̄/P − 1) (Decision, gap G-7)
```

The decomposition is exact because, given an ignition at k, the future events are independent of the past (A-1), and
the pulse grid `t_k + m` is fixed by k (`03` E-3.7; grid anchored at ignition, OQ-302). Checked against brute-force
enumeration (TV-08-06). The seeds' CO terms that read statuses other than Heat (Viral, Corrosive) are multiplied with
the pulse-time factors as independent expectations (A-11, gap G-12: −0.07 % with b_co = 0.8 and −0.10 % with b_co =
1.6 in the measured scenario). An
implementation MAY skip ignitions with ι_k < 1e-15 and MAY compute E-8.8 by any reordering that gives the same value
within 1e-12 relative.

### 5.5 The EV kernel: expected damage of one instance

```text
E-8.9  Expectation of one damage instance on view v at time τ
  1. Write the instance's damage (E-12, E-13, E-14, E-3.9 as applicable) as a sum of terms, each a product of
     (i) deterministic factors, (ii) functions of a single status group, (iii) the crit factor.
     Status groups: ARM = (n_COR, k_h, Heat live) [armor views only], VIR, MAG, PUN, COL (with Frozen), TAU,
     and one group per other status type (presence indicators for the CO count), one per conditional effect of
     class (b) (state of E-8.15), one per class (c)/(u) effect with u_e < 1 used nonlinearly (on/off, A-12).
  2. The CO term is affine in single-group functions (E-3.12: n = n_ext + Σ_s 1{present_s}; Secondary Shiver:
     n_COL; threshold predicates on single groups: indicators): Y = Y0 + ρ_co·B_ref·Σ_s g_s (adding) or
     Y0·(1 + Σ_s g_s) (multiplying). Health-threshold predicates are constants per 4.5.
  3. Crit factor (02 E-2.9, which this step implements): c(n_P, n_C) = c_0 + k̄(n_P)·(h·CDM(n_P, n_C) − 1) with
        CDM(n_P, n_C) = a(n_P) + b_C·cold_cd(n_C)     a(n_P) = (Q(n_P)·(1 + ΣCD_rel) + ΣCD_abs,static + doughty)·M_cdf,
                                                     Q(n_P) the Prelude-of-Might quantized base (02 E-2.5: depends on
                                                     CC(n_P) through θ_P), b_C = rad'·M_cdf
     so c = (c_0 − k̄(n_P)) + h·k̄(n_P)·a(n_P) + h·b_C·k̄(n_P)·cold_cd(n_C): a sum of products of one PUN-group
     function and at most one COL-group function. Never E[k̄]·E[CDM] (TV-08-25(a): +3.9 % error).
  4. E[term] = product over groups of E[product of that term's factors of the group]          (A-2)
        e.g. E[g_VIR·Vir] = E[Vir] − P(n_VIR = 0)·Vir(0) when g_VIR = 1{n_VIR ≥ 1}
     4a. Predicates on the distinct count n itself (Spiteful Defilement "+1.0 ΣCD_abs while n < 3", 02 §4.3,
        D-067; the cap n_max of 03 E-3.12 when more than n_max types can be present) couple every group. Let R be
        the groups the term also reads through other factors (PUN, COL, VIR, MAG, ARM, ...). Enumerate the presence
        vector of R (2^|R| cases; each case's group factors use the group distribution conditioned on presence or
        absence); for each case the remaining count N_rest = Σ over the other countable groups of their indicators
        (sibling-extended per E-8.10) is Poisson-binomial with the groups' presence probabilities; E[f(n)] =
        Σ P(N_rest = r)·f(n_R + r + n_ext). Exact under A-2 (TV-08-25(b)).
     4b. Two-time group expectations. When a factor reads group g at time t (a DoT seed's CO indicator at t_e) and
        another reads g at u > t (the tick's Viral, Magnetic or ARM factor), they are correlated through the procs
        alive at both times. With W_g(t) = the procs that make g present for the seed (events before t within D,
        and for instance j the earlier instances of its shot, E-8.10):
           E[1{present_g(t)}·f_g(u)] = E[f_g(u)] − P(no proc in W_g(t))·E[f_g(u) | no proc in W_g(t)]
        where the conditional expectation is the ordinary one with η_{g,f} = 0 (π_{g,f} = 0 for Heat, E-8.6) for
        every proc f in W_g(t). Exact under A-1, A-3 (TV-08-19).
     4c. Class (c)/(u) effects with u_e < 1 that enter a nonlinear use, or a product of two of their own factors
        (Mecha Empowered and Vauban passive on a DoT: seed and tick, M_dot), are one on/off group:
        E = u_e·E[term | on] + (1 − u_e)·E[term | off], with the same state for the instance and every tick of the
        DoT it seeds (04 4.3: E = 1 + u·((1 + v)² − 1), not (1 + u·v)²; TV-04-41; A-12). Linear uses keep u_e·v·m_e.
  5. ARM (05 E-5.10 in full; 05 §19):
        E[1 − DR(A_eff)] = Σ_{n_COR} Σ_x P(n_COR)·P_kh(x; τ)·(1 − DR(A_eff(n_COR, x, s_flat(τ))))
        A_eff = A0                                                     if armor_strip_immune = all (05 9.6)
        A_eff = A_tot·(1 − heat_strip(x))·(1 − c_corr(n_COR))         otherwise
        A_tot = max(0, A0·f_flat·max(0, 1 − Σ_add) − P_pts)
        f_flat = max(0, B_A − s_flat(τ))/B_A (1 if flat_only immune); s_flat(τ) = expected count-weighted points of
           the earlier qualifying hits (05 E-5.11, §19: Shattering Impact, Metal Auger; Melee Duplicate copies and
           Metal Auger ticks enter with their existence probabilities; the randomness of s_flat is not enumerated,
           gap G-16)
        Σ_add = ability strips·Str + Σ_CP 0.18·aura_str + T-07 + companion % strips (Sharpened Claws) (05 E-5.10;
           0 for ability_debuff_immune = all); P_pts = Polarize 400·Str (05 9.3)
        c_corr = corr_strip(n) of 03, or c_corr_H of 05 E-5.12 when the Hydroid flag is set; 0 if Corrosive-immune
        heat_strip(x) = 1 − 0.5^(x/4); x from the strip counter, which outlives the episode (03 E-3.8; 05 E-5.10)
        (Corrosive and Heat together: TV-08-21)
  6. Minimum of 1 per type (05; after armor) and attenuation Att(·) (05): if the minimum can bind (the smallest
     per-type value over the enumerated states is < 1) or the target has attenuation, enumerate the crit tier k
     (marginal over n_PUN) × ARM states (× n_VIR or n_MAG for attenuation), apply the function per state with every
     other factor at its expectation, and sum with the state probabilities (A-7)
```

Sources: CD-13 (overview: "target-state functions use the stack-count distribution, never f(E[n])"); `05` §19
("EV MUST take expectations over the stack and crit distributions (per crit tier and per stack state)"; "The minimum of
1 is applied per crit tier"); `02` E-2.9 ("E[c] MUST be averaged over the stack distribution"); `04` 4.3 (partial
uptimes). Step 4 applied to the CO term is the exact form of "CO bonuses are linear in n, so E[C] follows" (overview
4.2) when C multiplies a factor that reads a counted status; using `E[C]·E[rest]` instead is wrong by about 2 % in
TV-08-09.

### 5.6 Distinct-status count and sequential pellets

```text
E-8.10 Expected CO count for instance j of a shot (refines overview 4.2; 04 E-4.2)
  n_j = n_ext + Σ_s [ 1{present_s(σ_S)} + (1 − 1{present_s(σ_S)})·a_{s,j} ]          (T-05 = expected)
  a_{s,j} = 1 − Π_{j' < j} (1 − w_{j'}·A_{s,j'})   A_{s,j'} = P(instance j' applies ≥ 1 delivered proc of s), all
            sources of 02 E-2.18: rolled, forced, Impact-triggered, Hunter Munitions, Encumber (02's per-instance
            e_s,i), Debilitate, and the forced procs of j′'s own extra hits (04 E-4.2: they land before j)
  instances j' < j: the pellets, merged tick, extra hits, Melee Duplicate copy and zero-delay children before j in the
  order of 4.2 (03 §7.1 P4a); w_{j'} their existence probabilities (E-8.2)
  random pellet count (n_pel = n or n + 1, f = frac(MS)): instance j = n exists with probability f; its terms are
  weighted by f (04 E-4.2)
  each indicator is a single-group function (E-8.9 step 2): E[1{present_s}·f_s] uses the group distribution;
  the sibling term pairs with f_s(0) (siblings' procs do not change debuffs, CD-04)
  cap: when more than n_max (03 E-3.12) types are countable, E[min(n_j, n_max)] uses the Poisson-binomial
  distribution of n_j (E-8.9 step 4a); below that the cap cannot bind (04 E-4.2 delegates the cap here)
  T-05 = weapon_types or manual(N): n_j is that deterministic number for every instance (overview T-05); no group
  enumeration enters the CO term
  a DoT seed uses the n_j of its inducing instance (03 E-3.12)
```

Encumber attribution: this document follows `02` E-2.17/E-2.18 (the owner of per-instance proc expectations,
overview 7.2): the event's single Encumber proc lands on instance i with probability e_i, and later instances count it
through A_{s,i}. `04` E-4.2 attributes it to pellet 0 (Decision there); the two differ only when the first instance
cannot deliver a status, and the alignment is asked in OQ-810.

### 5.7 DoT: per-proc expected landed damage

```text
E-8.11 DoT procs of s ∈ {SLA, TOX, ELE, GAS} created by event e (weapon side per proc: S·X_base·c, 03 E-3.5)
  survival under caps (A-4, 03 §6.2: the oldest is replaced, ties by application order): a proc is still live for a
  tick at u (P2) iff fewer than cap_s procs of s were applied after it and before u, counting the later procs of its
  own instance, the procs of the later instances of e (order of 4.2) and the procs of later events:
     X_ι(u) ~ Poisson( Σ_{ι' after ι in e} η_{s,ι'} + Σ_{f: e < f, t_f < u} η_{s,f} )
     R_s(ι, u) = (1/η_{s,ι})·Σ_{r ≥ 0} P(N_ι ≥ r + 1)·P(X_ι(u) ≤ cap_s − 1 − r)       N_ι ~ Poisson(η_{s,ι})
                 (expected surviving procs of ι per expected proc; = 1 when cap_s = ∞; Σ over the instances of e of
                 η_{s,ι}·R_s(ι, u) with no later event equals E[min(Poisson(η_{s,e}), cap_s)], TV-08-24)
  expected tick of e's procs at u = Σ_{instances ι of e} E[ (S_ι + [s ∈ {SLA, TOX}]·a)·X_base,s·m_{s,ι}
                                    · F_s(u) ]·R_s(ι, u)
     S_ι = the DoT seed of instance ι (03 E-3.4, with ι's CO count n_ι at t_e); X_base,s = X of 03 E-3.5 without c_k;
     m_{s,ι} = crit-weighted proc mass of ι (E-8.2); F_s(u) = the enemy factor of an s instance at u (Viral, Magnetic,
     ARM, vulnerability per view)
     the expectation is taken per E-8.9: S_ι is affine in the presence indicators at t_e (step 2); the pairs it forms
     with m_{s,ι} (Puncture, Cold counted in n: same time, step 4) and with F_s(u) (Viral, Magnetic, Corrosive and
     Heat in ARM: two times, step 4b) are expanded exactly; never E[S_ι]·E[F_s(u)] (TV-08-19: −0.74 % on the first
     tick of a CO + Viral build)
  ticks at u = t_e + δ_s + m, m = 0 .. N_s − 1 (03 E-3.6); ELE and GAS tick 0 at P4c of e; every one of the N_s ticks
     lands, including the tick at the instance's own expiry time t_e + D_s (03 §7.3); only replacement removes ticks
  ELE, GAS "+1" (one per tick group, 03 E-3.5): for each distinct landing point (u, phase), add
     a·X̄·(1 − exp(−Σ_{e ticking at (u, phase)} η_{s,e}·R̄_s(e, u))) · E[F_s(u)]
     R̄_s(e, u) = Σ_ι η_{s,ι}·R_s(ι, u) / η_{s,e};  X̄ = Σ η_{s,e}·X_base,s·E[c](t_e) / Σ η_{s,e} over those e (Decision)
  BLA (A-9): one host hit max(1, 0.3·H) per stack at t_e + D_BLA, expected value with H per 03 E-3.9; the floor
     max(1, ·) is applied per crit tier (E-8.9 step 6)
  extra-hit DoT (E-14): forced procs of e_xh with the E-14 tick, no "+1"; Toxic Lash procs carry their own duration
     (+0.25 SD, D-037), which E-8.4 honours per source; their replacement order against weapon Toxin procs of another
     duration is the A-4 exception (gap G-14)
```

**Little's law.** In steady state (5.9) the expected number of live instances of s equals the arrival rate times the
expected lifetime, so the DoT damage rate of s equals `λ_s × (expected landed damage of one proc)`; with every tick at
the steady target state this is `λ_s · Σ_m E[tick_m] · R_s(m)` with `R_s(m) = P(Poisson(λ_s·(δ_s + m)) ≤ cap_s − 1)`
(TV-08-10; with several procs per instance, R_s(m) takes the within-event form above with the steady later-event
mean λ_s·(δ_s + m)). This is the wiki's "Avg Total Avg DoT = Modded Status Chance × Total Avg DoT × (1 + CC × (CDM −
1))" (`W:Damage/Calculation` lines 739-745) generalised to tick-time target states and caps. The seed's CO term and the
tick-time state are expanded as in E-8.11 with the steady distributions (the presence window W_g and the tick-time
window overlap by `max(0, D_g − (δ_s + m))` seconds of the Poisson stream). Heat is excluded (5.4).

### 5.8 Window walk

```text
E-8.12 EV window walk for a unit, horizon T_end (t_win for window metrics, t_ttk_max for TTK)
  inputs: schedule records up to T_end; evaluation views
  build the proc-event list E (4.2) and the consumer list: every instance (at its hit time, P4), every DoT tick point
    and ELE/GAS group point (P2, P4c), every Blast fuse expiry (P2)
  for each event e in order (4.3):
     state at t_e: μ_s(t_e) (E-8.4) from η of earlier events, Heat P_H and P_kh (E-8.5, E-8.6), condition-effect
        state distributions (E-8.15), expected flat strip s_flat(t_e) (E-8.9 step 5), schedule multipliers of the
        instances (06, 07)
     compute per instance, in the order of 4.2, with its existence probability w_ι (E-8.2): expected direct damage per
        view (E-8.9), extra hits (E-4.13), the Melee Duplicate copy as a later instance of the shot (4.2; its CO count
        includes the parent's procs, E-8.10; with Toxic Lash or Xata's Whisper the copy is the extra hit of the earlier
        ability slot, 04 6.4), and the per-event quantities η, m, v (E-8.2)
     register DoT procs of e (E-8.11), Blast stacks, condition triggers (E-8.15) and flat-strip hits
  evaluate every registered tick and Blast hit at its landing point with the state at that point (E-8.4 excludes
     events at or after the point in the order of 4.3)
  Heat: E-8.8 over all events
  landed[v] = Σ direct + extra hits + duplicate copies + ticks + Heat pulses + Blast hits with landing time ≤ T_end + ε
  dps_window[v] = landed[v](T_end = t_win) / t_win
  not produced in EV: Xata's Whisper detonation extra hits (04 E-4.13a; gap G-15)
```

The walk is causal: the expectations of event e depend only on earlier events, so rates that depend on the target state
(Tau status chance, Puncture crit chance feeding Hunter Munitions, conditions) need no fixed point in window mode.
TV-08-20 is the realistic check: the Braton Prime build of overview 3.8 over a 20 s window with two reloads.

### 5.9 Steady mode: `dps_burst`, `dps_sustained`

```text
E-8.13 Steady state of a unit
  steady period: the repeating pattern of the unit's schedule at the steady state
     constant-rate ranged attacks: one shot, T_per = 1/EFR;  BURST: one burst of BC shots, T_per = BC/EFR
     melee light_loop / slide_loop: one loop of N_C hits, T_per = T_loop (07 E-7.37)
     heavy-type melee plans: the period that 07 E-7.37 finds (expected-path state repeats), T_per its length
     child unit with T-22 off (section 9): the parent's period, with n_child (06 E-6.24) instances per parent shot
  per-period quantities at the steady state: η_s,per (procs of s per period), m_s,per, direct damage D_per[v]: Σ over
     the unit's instances of one period (pellets with E[n_pel] = MS, merged ticks, melee hits with their own stance
     multipliers S_j, combo multipliers, forced procs and CO ratios, extra hits, Melee Duplicate copies with w_ι)
  rate λ_s = r_per·η_s,per;  r_per = 1/T_per (burst) or ρ/T_per (sustained)       (06 E-6.26, E-6.29; 07 E-7.37)
  μ_s = λ_s·D_s                                                   (time average, overview 4.2; A-10, gap G-13)
  joint fixed point over x = (μ_s of every status, E[state] of every class (b) effect whose trigger or value feeds a
     rate or a damage term, e.g. Primary Frostbite multishot from Cold procs):
     x⁽⁰⁾ = (0, fresh-state effect states);  x⁽ⁿ⁺¹⁾ = F(x⁽ⁿ⁾): rates from the per-period quantities at x⁽ⁿ⁾, effect
     states from E-8.15b at those rates; stop when every component satisfies |x⁽ⁿ⁺¹⁾ − x⁽ⁿ⁾| ≤ 1e-12·max(1, |x⁽ⁿ⁺¹⁾|)
     or n = 200 (warning `ev.steady.no_convergence`); the iteration is monotone when every coupling raises rates (Tau
     raises SC, Puncture raises crit and Hunter Munitions, Frostbite raises multishot; 02 4.9); for any other coupling
     the update is damped, x ← (x + F(x))/2, from n = 50 on (Decision)
  Heat presence and strip: E-8.7 with λ = λ_HEA; Heat pool: window mean (E-8.14)
  multi-pellet shots: the CO count per pellet j by E-8.10 with the steady presence probabilities
  schedule-driven factors: steady means of 06 §10.4 (beam ramp, first/last shot, sniper combo, Peacemaker), combo
     (07 E-7.37) and conditions at their steady values (04 E-4.9, E-8.15b)
  permanent flat armor strips (05 E-5.11) have no stationary value: the ARM factor of every steady quantity is its mean
     over the shots in [0, t_win] of the uninterrupted (burst) or real (sustained) schedule with s_flat at its expected
     value before each shot and every other state at the steady state (Decision, like the Heat window mean)
```

```text
E-8.14 Steady metrics per view v
  dps_burst[v]     = D_per[v]/T_per + Σ_{s ∈ {SLA, TOX, ELE, GAS, BLA}} λ_s·L_s[v] + H_burst[v]
       D_per[v]   = expected direct damage of the unit's instances in one steady period (E-8.13); for a one-shot
                    period D_per/T_per = EFR·D_shot with D_shot the expected direct damage of one shot
       L_s[v]     = expected landed damage of one proc of s at the steady state (E-8.11 with R_s(m) of 5.7; BLA: the
                    host hit of A-9); ELE and GAS add their group "+1" at the steady group-presence probability
       H_burst[v] = (EV Heat landed in [0, t_win] on the uninterrupted schedule) / t_win, with every other state at
                    the steady state and per-shot Heat values at the steady state (overview 4.2 "Heat window mean")
       uninterrupted schedule: 06 E-6.11 with Mag = ∞ (no reload), from t = 0; melee: the 07 schedule
  dps_sustained[v] = ρ·D_per[v]/T_per + Σ_s λ_s·L_s[v] + H_sust[v]   with λ_s, μ_s at r_per = ρ/T_per (cycle-average
                     rate, overview 4.1), D_per and L_s at that steady state, 06 sustained means,
       H_sust[v]  = (EV Heat landed in [0, t_win] on the real schedule 06 E-6.11) / t_win
  attacks without reload (melee, infinite-ammo states while active, an Incarnon form within one activation, ρ = 1,
     06 E-6.29): dps_sustained = dps_burst by construction
  gauge-gated attacks (06 6.7, E-6.26: no EFR): no steady metrics; dps_burst, dps_sustained, dmg_per_shot,
     dmg_per_mag and hit_infight are reported as "not available"
  child unit (T-22 off): T_per, ρ of the parent's firing attack; D_per holds only the child's own instances, so a group
     sum (section 9) counts each instance once
```

Sources: overview 4.1 (definitions), 4.2 (steady mode); `W:Damage/Calculation` lines 444-459 ("Average Burst DPS =
Average Shot × Effective Fire Rate", ramp at max), 531-560 ("Average Sustained DPS = Average Burst DPS × Proportion of
Time Spent Shooting") and §Melee DPS ("Average DPS = Avg Hit × Modded Attack Speed / Base Combo Length", the per-period
average for melee); `06` §10; `07` E-7.37 ("dps_burst = damage over the last full period / period length"). The wiki's
formulas exclude status damage; adding DoT by Little's law is the Decision of overview 4.1. TV-08-20 compares
`dps_burst` with the late-window average of a long uninterrupted MC run (−0.16 %, gaps G-1 and G-13 of opposite sign);
TV-08-17 shows the arrival-time effect on a slow weapon (G-13 alone +6.45 %, net +4.72 % against the exact mean).

### 5.10 Conditional stacks in EV

Owner of the condition model: `04` (E-4.8 state machine, §4.2 fields, E-4.9 EV steady state, E-4.21 Enervate). Window
mode (`04` E-4.9: "window mode: 08 walks the expectation through time with the same chain"):

```text
E-8.15 Window walk of one class (b) effect (T-02 simulated, and effects without maximum in all_max; A-6)
  trigger units: the units of the effect's trigger and hit_unit (04 §4.1: pellets, shots, or instances including
     extra hits; reloads; schedule events), in the order of 4.3, each with success probability
     p_i = chance × P(trigger condition | unit): on_hit 1; on_crit P(k ≥ 1); on_tier1_crit P(k = 1); on_big_crit
     P(some pellet of the unit has k ≥ 2); on_status(s) A_s (delivered procs, 03 §6.4); on_headshot and
     on_weak_point_hit per T-03; schedule events (reload, first/last shot) 1; all at the expected target state (A-6)
  refresh, single stack, gain g:      P(active at τ) = 1 − Π_{i before τ, t_i > τ − D} (1 − p_i);  stacks = min(m, g)
  independent timers, gain g, cap m:  N(τ) = successes among the units in (τ − D, τ): Poisson-binomial({p_i});
                                      stacks = min(m, g·N(τ))   (exact under A-6; the cap keeps the newest, A-4 argument)
  every other combination of the 04 §4.2 fields (shared_lose_one, shared_lose_all, no_refresh_while_active,
     consume_on, reset_on, decay): exact forward chain over σ = (k stacks, t_l last refresh time, c uses since the
     last activation, r counted reset events), start (0, −∞, 0, 0) or the T-02 start state; between units, expiry
     per 04 E-4.8:  shared_lose_one: k_eff(τ) = max(0, k − floor((τ − t_l)/D));  shared_lose_all: k_eff = k·[τ − t_l < D]
     at trigger unit i, with probability p_i:
        refresh = no_refresh_while_active and k_eff(t_i) > 0: no change
        otherwise: k ← min(m, k_eff(t_i) + g), t_l ← t_i            (m = none: no cap)
     at a unit of the reset_on kind, with its probability (Enervate on_big_crit, Double Tap on_miss = 0 under the hit
        model of 4.1, Strun on_reload = 1): r ← r + 1; if r = count: k ← 0, r ← 0
     at a consume_on use (next_shot, next_heavy, next_shots(k): schedule events, deterministic): the use reads
        value(τ⁻); then c ← c + 1, and k ← 0, c ← 0 when c reaches the use count
     states with k_eff = 0 and zero counters merge into the empty state
  E[value(τ)] = v·E[k_eff(τ)·decay(τ)]; effects that feed a nonlinear stage use the state distribution (E-8.9 step 1)
  Secondary Enervate (04 E-4.21): forward distribution over (h, b) along its hit units: a unit in state (h, b) uses
     CC = CC_E-5 + 0.10·h, and P(big | h) = 1 − Π_pellets (1 − P(k ≥ 2 | CC(h))) averaged over n_PUN at the unit; then
     h ← h + 1, b ← b + [big], (h, b) ← (0, 0) when b = r + 1; the unit's damage uses Σ P(h, b)·E[c | CC(h)]
     (E[c] is not linear in CC with Vigilante, 02 E-2.7)
```

```text
E-8.15b Steady state of one class (b) effect (rate λ of successful triggers; Poisson approximation of 04 E-4.9)
  single stack, independent timers (E[min(m, g·N)], N ~ Poisson(λD)), shared_lose_one: 04 E-4.9
  shared_lose_all (04 E-4.9 delegates it here, OQ-427): with p = 1 − e^{−λD}, the effect is active with probability p
     and, given active, the triggers since the last gap longer than D number K with P(K = j) = p^{j−1}(1 − p):
        E[stacks] = p·Σ_{j ≥ 1} p^{j−1}(1 − p)·min(m, g·j)        g = 1: p·(1 − p^m)/(1 − p)          (TV-08-26)
  no_refresh_while_active, single stack: active periods of D alternate with Exp(λ) waits: P(active) = λD/(1 + λD)
  consume_on with uses at the steady period's use times: at a use, the stacks are those gained since the previous use
     and still alive: single stack P(active at use) = 1 − e^{−λ·min(T_u, D)} (T_u = time since the previous use);
     other stack timers: the E-8.15 chain over one steady period, iterated until its state at the period start repeats
     within 1e-12
  reset_on and Secondary Enervate: renewal-reward over reset cycles: forward reach probabilities of the states from
     the reset state until the remaining mass is < 1e-15; Enervate's per-unit mean CC = CC_E-5 + 0.10·E[Σ_cycle h] /
     E[units per cycle] (04 E-4.21; reproduces TV-04-21: 1.279708, 1.531938, 1.831333)
```

`all_max` and `none` need no walk (point masses, `04` 4.3), except effects without a maximum (Secondary Enervate),
which E-8.15 walks under `all_max` as under `simulated` (`04` 4.3). Class (c) and (u) effects enter linear uses as
`u_e·v·m_e`; a nonlinear use, or a product of two of their own factors, uses the on/off mixture of E-8.9 step 4c with
one state per instance and its DoT (A-12).

### 5.11 Complexity notes

E-8.4 is a sliding-window sum per status (O(1) amortised per consumer). E-8.6 needs the events within `4·h_up` and
`4·h_dn + D` of τ. E-8.8 is O(K·M·L) for K ignitions, M events per episode and L live entries (L ≤ events within D).
E-8.9 step 4b re-evaluates a group's distribution once per conditioning window; step 4a costs 2^|R| times a
Poisson-binomial convolution over at most 17 types.
Implementations MAY use prefix products in log space for Z(·) (with a separate count of factors equal to 0) and any
algebraically equivalent order of summation; results MUST agree with the direct formulas within 1e-12 relative
(the test vectors use 1e-9).

---

## 6. Evaluator (b): Monte Carlo

### 6.1 Random number generator and keying

```text
E-8.16 Counter-based RNG (overview U-6, 4.3)
  generator:  Philox4x32-10 (Salmon et al., Random123), multipliers 0xD2511F53, 0xCD9E8D57, Weyl constants
              0x9E3779B9, 0xBB67AE85; round: (hi0, lo0) = mulhilo(0xD2511F53, c0), (hi1, lo1) = mulhilo(0xCD9E8D57, c2),
              c ← (hi1 ⊕ c1 ⊕ k0, lo1, hi0 ⊕ c3 ⊕ k1, lo0); the key is bumped by the Weyl constants before rounds 2..10
  key:        (k0, k1) = (seed mod 2^32, floor(seed / 2^32))                       seed: u64 (T-29, default 0)
  counter:    c0 = run (u32)
              c1 = rng_mech(name) = FNV-1a 32-bit of the mechanic id's UTF-8 bytes (offset 0x811C9DC5, prime 0x01000193)
              c2 = (a << 20) | i                     a < 4096 attack index, i < 2^20 shot index of the anchor
              c3 = (j << 16) | b                     j < 65536 sub-instance index, b < 65536 block index
  draw:       the k-th draw (k = 0, 1, 2 ...) of a given (run, anchor, mechanic) in processing order uses block b = k >> 1;
              half 0 (k even): x_lo = x0, x_hi = x1;  half 1 (k odd): x_lo = x2, x_hi = x3;
              m = ((x_hi << 32) | x_lo) >> 11, i.e. u = floor((x1·2^32 + x0)/2^11)·2^−53 ∈ [0, 1) for half 0
  anchors (E-8.0 for instances):
     per-instance draws (02 4.8: "crit_roll" ... "st"; "enc" once per instance tried, "enc_type" on the instance that
              succeeded, 02 E-2.17; "deb", "deb_type", "deb_comp", 02 E-2.17b): the instance's anchor
     shot-level draws ("ms_pellets", 02 E-2.2, E-2.3): j = 0xFFFF
     reload-level draws: a = the firing attack whose magazine reloads, i = reload number, j = 0xFFFE
     per-run draws (the random phase of a class (c)/(u) on/off process, E-8.18): a = 0, i = 0xFFFFF, j = 0xFFFD,
              mechanic "eff/<effect id>/phase"
     conditional effects ("eff/<effect id>", trigger draws of 04 E-4.8 with p < 1): the anchor of the triggering
              unit (the instance; for a shot unit its j = 0 instance; for a reload unit the reload anchor)
     melee draws (07: "combo_acc", "combo_rc", "combo_curse", "glaive_volatile"): the anchor of the hit or tick that
              rolls them (a Volatile explosion: the anchor of its glaive throw hit)
  derived instances: extra hits use the parent anchor and the mechanic id "xh/<source>/<name>" (source = catalogue id
              of 04, e.g. "xh/xata/sc_count"); Melee Duplicate: "dup/<name>"; Xata's Whisper detonation extra hits
              (04 E-4.13a): "xd/<name>" on the anchor of the instance whose proc created the detonating stack
  an index outside its range is an error (no wrap-around); i = 0xFFFFF and j ≥ 0xFFFD are reserved for the sentinel
              anchors above (E-8.0 keeps every instance anchor below them)
```

```text
E-8.17 Mapping draws to outcomes
  Bernoulli(p):         success iff u < p          (p ≤ 0 never, p ≥ 1 always; exact fractions, D-017)
  floor + Bernoulli(f): n + [u < f]                (pellets, crit tier, proc count: 02 E-2.2, E-2.6, E-2.12)
  categorical(w):       the first s in the enumeration order of overview 2.2 with u < Σ_{t ≤ s} w_t; if rounding
                        leaves u ≥ Σ w, the last s with w_s > 0
  uniform over a list:  index floor(u·L)
```

The draw order is the one of `02` 4.8: per shot "ms_pellets"; per instance "crit_roll", "crit_vig", "sc_count",
"sc_type" (one per proc), "cedo_type", "ib", "welt", "hm", "st"; then per proc event "enc" (one draw per instance tried,
keyed by that instance's anchor, until one succeeds) and "enc_type" (keyed by the anchor of the instance that
succeeded); then per instance "deb", "deb_type", "deb_comp" (TV-08-23). `07` adds "combo_acc", "combo_rc",
"combo_curse" and "glaive_volatile"; `04` adds the
"eff/<id>" trigger draws; the derived prefixes "xh/", "dup/", "xd/" repeat the ids of their instance. Ordinals count
per (run, anchor, mechanic), so the order matters only within one mechanic. Two builds evaluated with the same seed
see the same draw for the same (run, anchor, mechanic, ordinal): common random numbers (overview 4.3). FNV-1a values of
the `02` and `07` ids are listed in TV-08-02; the registry of mechanic ids is the union of these lists, the effect ids
of `04` and the derived prefixed ids, and an engine unit test MUST assert that every registered mechanic id hashes to
a distinct value. Sources: Philox
constants and known-answer outputs reproduced from Random123 (`kat_vectors`) by an independent implementation
(TV-08-01); FNV-1a reference values ("" → 0x811C9DC5, "a" → 0xE40C292C, "foobar" → 0xBF9CF968) checked.

### 6.2 Run state

```text
Target (one per pass):
  per capped status s: live entries ordered by application seq: (t_apply, t_expire, source attack, seq)
  DoT instances (SLA, TOX, ELE, GAS): (seq, s, S_p, X_p incl. c_k and Part_dot, N, next tick m, cancelled flag)
  Heat episode: t_ign, pool [(seq, S_q·X_q, D_q, SD_q)], t_exp, next pulse; strip counter k_h, pending up/down steps
  Blast fuses: (seq, H_p, expiry);  Frozen: (until);  VOI, KND, LIF, MIC single states;  T-06 external Viral
  pools (layered passes): OG, SH, HP, gate_until, adaptation state, health-threshold flags
  flat armor strip s_flat (05 E-5.11, permanent)
Attacker: schedule cursor per firing attack; combo counter and timers (07); sniper combo (06); condition-effect
  states (04 E-4.8: stacks, timers, use and reset counters); on/off state and phase of each class (c)/(u) effect with
  u_e < 1 used nonlinearly (A-12); magazine state is read from the schedule records
Accumulators: per view: landed in window, pending, per bin, per source and type; crit-tier histogram; uptime per bin;
  parity hit records (when requested); TTK and shots/ammo to kill (layered passes)
Event heap: key (τ̂, phase, sub_key, seq) (4.3)
```

### 6.3 Event processing

```text
E-8.18 One pass of one run (r)
  start: draw the phase φ ~ U[0, T_period) of every class (c)/(u) on/off process (per-run anchor, E-8.16); the
     effect is on during [φ + k·T_period, φ + k·T_period + u_e·T_period) (04 4.3: on-time and period of the effect,
     e.g. the Mecha mark duration and interval)
  events: shot (from the schedule, inserted lazily), stack/DoT expiry (removes presence only; never cancels a tick,
     4.3), Frozen end, Heat strip step, condition expiry or decay step, combo expiry (07), sniper decay (06), DoT tick
     (one per instance and tick; ELE/GAS grouped at processing time), Heat pulse, Heat episode end, Blast fuse expiry,
     Magnetic-break tick (layered only), on/off switch of a class (c)/(u) process (P1)
  loop: pop the smallest key; process per 03 §7.1:
     P1 transitions; P2 damage (ticks, pulses, fuses), each tick group one instance (03 E-3.5), routed per pass; a
        Blast host hit that detonates creates Xata's Whisper's detonation extra hit right after it (04 E-4.13a; also in
        P4b detonations); P3 Heat end;
     P4 shot S: (a) for each instance of S in order: damage against σ_S (CO count with earlier instances of S), draws
        per E-8.16/E-8.17, extra hits after their parent; on a parent melee instance with k = 1 and a successful
        "dup/..." Bernoulli(p_dup): the Melee Duplicate copy as the next instance (4.2), or, with Toxic Lash or Xata's
        Whisper active, a copy of the extra hit of the earlier ability slot right after that extra hit (04 6.4);
        (b) apply all procs of S (insertion, replacement, Frozen, Heat join/ignition, Blast detonation),
        Magnetic-break procs after the procs of the breaking instance, condition gains, flat-strip hits; (c) zero-delay
        ticks; (d) DoT detonation (03 §9.7)
     every damage instance is evaluated once per view of the pass (view pass) or routed once (layered pass)
  stop: view pass: no shot with s_i > t_win and no instance with hit time > t_win is processed (E-8.1a); continue P1-P3
        until no tick, pulse or fuse remains (pending, credited); layered passes: at the kill (HP ≤ 0) or when no
        event ≤ t_ttk_max remains
```

### 6.4 Passes

| Pass | Target | Horizon | Draws |
|---|---|---|---|
| V (views) | non-depleting; every instance evaluated for each existing view of E-8.1b | shots with s_i ≤ t_win; DoT continuation for pending | keys of E-8.16 |
| L (layered) | depletion OG → SH → HP, shield gate, overflow, Magnetic break, adaptation, health thresholds (05 E-5.25) | until kill or `t_ttk_max` | the same keys (same draws for the same anchors) |
| L0 (parity only) | as L with `arm_t = 1` (armor ignored, the inspiration's Health-panel TTK, D-802) | as L | the same keys |

Pass L0 runs only when parity outputs are requested and A0 > 0. Pass L stops creating extra hits for a parent that
kills (`04` 6.2) and processes no instance after the kill (remaining pellets of the killing shot hit nothing).

### 6.5 Aggregation across runs

```text
E-8.19 Statistics over runs r = 0 .. R − 1 (R = n_runs), values x_r reduced in run order
  mean      x̄ = (Σ_r x_r)/R
  sd        s = sqrt(Σ_r (x_r − x̄)²/(R − 1))  (R ≥ 2; R = 1: s undefined, reported as none)
  SE        s/√R;  CI95 = x̄ ± 1.959963984540054·SE;  CI99 = x̄ ± 2.5758293035489004·SE
  quantile  qnt(p) (Hyndman-Fan type 7): sort ascending x_(0) ≤ ... ≤ x_(R−1); g = (R − 1)·p; l = floor(g);
            qnt = x_(l) + (g − l)·(x_(l+1) − x_(l)) (qnt = x_(l) when g = l or l = R − 1);
            a term involving +∞ gives +∞ (TTK of runs that did not kill)
  median = qnt(0.5) (equals the inspiration's "mean of the two middle values", JS L13269-13273, which also gives
            Infinity when one of them is infinite)
  median CI99 (distribution-free, oracle 11.3): ranks l = floor(R/2 − 1.28791465·√R), u = ceil(1 + R/2 + 1.28791465·√R),
            clipped to [1, R], interval [x_(l), x_(u)] (1-based)
```

Decisions (conventions, not game rules): sample sd with R − 1; normal-approximation CIs of the mean; quantile type 7
of Hyndman and Fan (1996), the default of R and numpy; the median CI from the normal approximation of the binomial
distribution of order statistics (1.28791465 = z_0.995/2).

Runs are independent and MAY be computed in parallel; per-run results are stored by run index and reduced in run
order, so results are bit-identical for any thread count (overview 4.3).

---

## 7. Time to kill

### 7.1 MC TTK

```text
E-8.20 TTK of run r (pass L): the time τ of the damage instance (direct, tick, pulse, Blast hit, extra hit, break
       tick) after whose routing HP ≤ 0; +∞ if no kill by t_ttk_max
  outputs over runs: kill probability P_kill = #{finite}/R; median qnt(0.5), which is +∞ ("not killed") whenever it
     involves an infinite order statistic: R odd and more than half of the runs not killed, or R even and at least
     half of them; mean over killed runs; qnt(0.05), qnt(0.95)
  shots to kill: number of shots whose P4 was executed up to and including the kill (the killing shot included;
     shots at the same time after the killing shot are not processed; a P2 tick that kills at the fire time of a shot
     kills before that shot's P4, so that shot is not counted); ammo to kill: Σ of their cost c_i (06 6.1); not
     killed: none
  flag shield_regen_possible from 05 10.5
```

### 7.2 EV TTK

```text
E-8.21 EV TTK (expected path; overview 4.2)
  run the window walk E-8.12 with T_end = t_ttk_max on the layered view: pools OG, SH, HP (05) as deterministic numbers
  each consumer in order deals its expected damage per layer: E[D^OG], E[D^SH] (non-bypass types), E[D^HP] split
     into bypass and non-bypass parts, routed by 05 E-5.25 with g = g' = 1 (no shield gate) and fractional overflow;
     OG overflow discarded (CD-08); a break emits the expected Magnetic-break proc with E[n_MAG] (03 E-3.10 is linear
     in n); overguard-dependent caps follow the expected path (og_present = expected OG > 0); health-threshold
     predicates per 4.5 (simulated: from the expected HP before the consumer); attenuation and M_dps per consumer on
     its expected damage; adaptation (05 E-5.23) is driven by the expected per-type health damage of the path, and
     per-hit caps apply to the expected value of each consumer (Decision: the expected path is one deterministic run
     of the 05 rules with expected inputs)
  T_EV = time of the first consumer after which HP ≤ 0
  κ = (Σ_{consumers c at T_EV} ΔHP_c − HP_before) / HP_0 ≥ 0      overkill margin at T_EV: ΔHP_c is the expected HP
     damage of each consumer at time T_EV, routed as if the kill did not stop the walk (all pellets of the killing
     shot, its extra hits and ticks at that time); HP_before = expected HP before the first of them; HP_0 = HP pool
  shots_to_kill_EV = #{shots whose P4 is at or before the killing consumer};  ammo_to_kill_EV = Σ their c_i
  not killed by t_ttk_max (ttk_extrap = true): extrapolate phase by phase from the remaining pools with the sustained
     per-layer rates of the steady state (E-8.14 on the view of each layer; per-type split of the steady breakdown):
        d_OG = dps_sustained[overguard];  d_SH = dps_sustained[shield] (non-bypass types);
        d_BP = the bypass-type part (B_SH, 05 E-5.15) of dps_sustained[A] and d_HP = dps_sustained[A],
              A = armor if A0 > 0 else health
        T_OG = OG_rem/d_OG;  T_SH = SH_rem/d_SH;  HP_1 = HP_rem − d_BP·T_SH  (Toxin keeps draining health while the
              shield is up; OG absorbs every type, so nothing drains during T_OG)
        T_EV = t_ttk_max + T_OG + T_SH + max(0, HP_1)/d_HP               if HP_1 > 0
        T_EV = t_ttk_max + T_OG + HP_rem/d_BP                            if HP_1 ≤ 0 (killed through the shield)
     a phase with zero remaining pool takes 0 s; a positive pool whose rate is 0 gives T_EV = +∞ ("never") (TV-08-22)
     no steady metric (gauge-gated attacks, 06 E-6.26): the rates are the mean per-layer DPS of the walk over
        [t_ttk_max − min(t_ttk_max, 10 s), t_ttk_max] (Decision); if that is 0 as well, T_EV = +∞
```

T_EV is an expectation-path estimate, not the mean of the TTK distribution (overview 4.2); with no shield gate it can
be shorter than every MC sample (TV-08-13). The MC distribution is the reported TTK (overview 4.5).

### 7.3 TTK horizon

`t_ttk_max` (default 60 s) bounds the schedule and the layered pass; the inspiration's TTK is bounded by its DPS window
and has no extrapolation (D-804). Shields never regenerate (`05` 10.5).

---

## 8. Outputs

Per evaluation unit and per view unless stated. "EV" and "MC" mark the evaluator; both are computed on request.

### 8.1 DPS

| Output | Definition |
|---|---|
| `dps_burst[v]` (EV) | E-8.14 |
| `dps_sustained[v]` (EV) | E-8.14 |
| `dps_window[v]` (EV) | E-8.12: expected landed damage in [0, t_win] / t_win (headline, default optimizer metric) |
| `dps_window_mc[v]` (MC) | per run x_r = landed_r[v]/t_win (pass V); mean, sd, CI95, median, qnt(0.05), qnt(0.25), qnt(0.75), qnt(0.95) (E-8.19) |
| `dps_series[v][k]` | landed damage with landing time in bin k divided by the bin's length within [0, t_win], k = 0 .. K − 1, K = ceil(t_win/b); bin of a landing time τ ≤ t_win + ε: k = min(K − 1, floor(τ̂/b̂)) on the time grid (E-8.0a), so damage at exactly t_win (and within ε of it) goes to the last bin; the last bin's length is t_win − (K − 1)·b. EV: expected value; MC: mean, qnt(0.10), qnt(0.90) over runs (TV-08-27) |

### 8.2 Breakdown

Landed damage in the window split by source: direct per damage type t (the post-enemy-side D_t; under attenuation the
attenuated sum is attributed in proportion to the pre-attenuation D_t), DoT per status s, Heat pulses, Blast hits,
extra hits per source, Melee Duplicate, lingering child ticks. EV: expectations; MC: means over runs. The parts sum to
the view's landed damage. Magnetic-break damage exists only in the layered passes (D-054) and is reported in the TTK
breakdown of pass L (MC mean over runs of the damage routed up to the kill or `t_ttk_max`, split by the same sources
plus "Magnetic break"; EV: the same split along the expected path of E-8.21).

### 8.3 Stacks, distinct count, uptime

EV steady (5.9): `E[n_s] = Σ_k k·P(n_s = k)`, `P(n_s ≥ 1)`, `P(n_s = cap_s)` for every status the unit can apply
(E-8.3; Heat: P(live) = 1 − e^{−λD} and the strip distribution E-8.7); `E[n] = n_ext + Σ_s P(present_s)`.
Window curves (`uptime[s][k]` = mean of `1{n_s ≥ 1}`, `stacks_mean[s][k]` = mean of n_s, over bin k of `dps_series`):
MC: the time average over the bin, averaged over runs; EV: the time average over the bin of the E-8.3 quantities
from μ_s(τ) (E-8.4), which is piecewise constant between events, so the bin average is the duration-weighted sum over
the event intervals inside the bin. Both are bin averages, so EV and MC are comparable.

### 8.4 Crit-tier distribution

`crit_tier_dist[x]`: EV steady: `P(k = x)` at the steady state, averaged over n_PUN (`02` E-2.7, E-2.9); EV window: the
mean of `P(k = x)` over the direct instances of the window, weighted by their existence probabilities w_ι (E-8.2) and
averaged over n_PUN at each instance's time. MC: histogram of the tiers of direct instances in the window (pass V),
normalised. The EV window distribution is the counterpart of the MC histogram.

### 8.5 Damage per shot and per magazine

`dmg_per_shot[v] = dps_burst[v]/EFR` for ranged attacks (expected damage, DoT included, of one shot at the steady
state); melee: the mean per hit over the steady period of E-8.13, `dps_burst[v]·T_per/N_hits` with N_hits the parent
hits of the period (stance hits differ, `07` E-7.37); `dmg_per_mag[v] = dmg_per_shot[v]·N_mag` (`06` E-6.27; none for
N_mag = ∞). Not available for gauge-gated attacks (E-8.14).

### 8.6 TTK outputs

MC (E-8.20): P_kill, median, mean of killed runs, qnt(0.05), qnt(0.95), shots and ammo to kill (median over killed
runs), `shield_regen_possible`, the TTK breakdown of 8.2. EV (E-8.21): T_EV, κ, `ttk_extrap`, shots and ammo to kill.

### 8.7 Pending DoT

`out_dot_pending[v]`: expected (EV) or mean (MC) damage landed after t_win by procs applied at or before t_win (4.6).

### 8.8 Parity outputs (the inspiration's result rows)

Computed per attack, per view of E-8.1b, from MC runs, for integer t_win only (the inspiration's buckets assume an
integer window, `JS L12787-12801`). A **parent hit** h is one direct instance of the attack (pellet, merged beam tick,
melee hit; a Melee Duplicate is its own hit; children are their own attack's hits), in schedule order. Membership:
the instances the view pass processes, i.e. hit time ≤ t_win + ε (E-8.1a); a pellet fired before t_win that lands
later is not a parity hit.

```text
E-8.22 Per run r, per view v
  direct_h   = direct damage of h + extra-hit direct damage (Xata, Toxic Lash hit, other E-14) + flat-per-status
               damage (04 catalogue, e.g. Cascadia Empowered)
  status_h   = credited DoT (4.6) of the procs h applied, extra-hit DoT included
  total_h    = direct_h + status_h
  buckets    N = #hits; slot = t_win/N; bucket[s] = Σ_p total_p·|[p·slot, (p+1)·slot) ∩ [s, s+1)|/slot, s = 0 .. t_win − 1
             (by hit index, not time; N = 0: all buckets 0)                                     (JS L12787-12801)
  queue_dps_parity_r = median(bucket)                                                          (JS L12802-12805)
  avg_direct_r = mean_h direct_h;  avg_status_r = mean_h status_h;  avg_total_r = avg_direct_r + avg_status_r
  median_total_r = median_h total_h                                                             (JS L12217-12238)
  ttk_parity_r: Health, Shield, Overguard rows: pass L0 TTK (armor ignored); Armor row: pass L TTK (D-802)
Reported ("average over cycles" replacement, D-088): for each quantity, the median over runs; for queue_dps_parity
  also min and max over runs; TTK median with +∞ for runs that did not kill ("not killed"). One run (R = 1) gives the
  inspiration's single "random queue".
```

In the compat profile the same definitions apply with compat per-hit values (D-801 for the per-bullet averages,
D-086 credited accounting).

### 8.9 "More details" timeline

One MC run (run index chosen by the user, default 0), one view: for every parent hit in time order: hit number (1-based
index of the parent hit) and time, damage on the view, crit tier and crit factor c_k, effective armor and DR at the
hit, stance and combo multipliers, Xata's Whisper damage, Toxic Lash direct hit and its one-tick value (separately),
flat-per-status damage (Cascadia Empowered and other `04` flat-per-status sources), statuses applied (with Primary
Debilitate's added component procs marked, and their tick value), the one-tick weapon-side value of each DoT proc
applied, live stacks of every status labelled "before the hit (game order)": the state σ the hit's damage used, which
excludes the hit's own procs (CD-04; the inspiration's popup counts them, D-030); reload records numbered "↻ #n" with
start and end, and Heat episode boundaries, interleaved by time (overview 4.5; `I:feat` §13).

### 8.10 Weapon and enemy panels

Weapon panel per attack: resolved stats of ST-1 to ST-7 on the fresh target (no statuses), conditions per T-02 at t = 0,
CO count per T-05 at the fresh state: per-type `q_t·Y·(1 + F)`, total, CC, CDM, SC, MS, FR, EFR, magazine, reload, ammo
cost, charge, burst values, falloff. Enemy panel: `05` scaled pools, armor and DR, innate DR, immunities, caps.

### 8.11 EV-versus-MC deviation flag

When both evaluators ran: `dev[v] = dps_window[v] − mean(dps_window_mc[v])`; flag iff E-8.24 fails; the flag lists the
gap classes active in the scenario (section 12).

### 8.12 Display rounding

Numbers are rounded only for display with `floor(x + 0.5)` (overview U-5); times with three decimals; percentages
from fractions. Engine values are never rounded.

---

## 9. Attack groups and combine attack effects

```text
E-8.23 Groups
  family(a): a firing attack (06 E-6.23: no parent; a reload-triggered attack of 06 9.5 counts as firing) with all
             its descendants (children, lingering children)
  group 𝒢 = one family with ≥ 2 attacks (Acceltra Prime impact + explosion, Lenz charged shot + blasts, Catabolyst
             reload grenade impact + explosion; the inspiration's `comb` lists are exactly such families,
             research/external/inspiration/samples/weapon-*.json; I:feat §10)
  T-22 off (default): every attack is its own evaluation unit (own status timeline, own TTK); each unit's quantities
             range over its own instances only (4.1), so group outputs are sums that count every instance once:
             dps_*[𝒢][v] = Σ_{a ∈ 𝒢} dps_*[a][v]  (EV and per-run MC sums); no group TTK
  T-22 on:   the group is one evaluation unit: all member instances on one target state merged by time with normal
             expiry (CD-12); per-attack outputs are the members' shares of the unit's landed damage; group outputs
             are the unit totals; group TTK from passes L/L0 of the unit
  child unit (T-22 off): its instances are the child instances of the parent's shots (06 E-6.24, 9.3); EFR, ρ and the
             steady period are the parent's (E-8.13, E-8.14)
```

Alternative firing attacks (normal fire and Incarnon form, light and heavy melee, zoom modes) are never grouped: they
are not fired together. A child is never evaluated without its parent's schedule: when T-22 is off it is evaluated on
the parent's shot times with only its own procs on the target. This is a Decision from the game rule that children are
created by parent hits (overview 2.3; `F:weapon` §10.1; `06` 9.2-9.3), not parity: the inspiration gives every attack,
children included, its own shot schedule from that attack's own stats (`I:agg` §7.5, §2), which D-810 records.

With T-22 off, the group sum of a parent and its children ignores the debuffs one member's procs put on the others
(Viral, Corrosive and Heat strip, the CO count) although in game they hit one target (CD-12), and no TTK exists for the
real kill by the whole family. OQ-802 asks the manager to make the family one unit by default in the game profile.

---

## 10. Optimizer metrics

The optimizer scores builds with the EV evaluator and confirms finalists with MC (overview 4.4; search in
`docs/architecture/`). Every metric is evaluated for one unit (attack or combined group, or a group sum) and one view.
The view `auto` is armor when A0 > 0, else health (the inspiration's default, `I:opt` §2).

| Metric | Definition | Better |
|---|---|---|
| `dps_window[v]` (default) | E-8.12 | higher |
| `dps_burst[v]`, `dps_sustained[v]` | E-8.14 | higher |
| `ttk` | E-8.21 on the layered view; ordering E-8.26 | lower |
| `ttk_noarmor` | E-8.21 on the layered view with `arm_t = 1` (the EV counterpart of pass L0): the inspiration's TTK for the defense rows health, shield and overguard, where "the layer only toggles whether health is computed through armor" (`I:opt` §2; D-802); ordering E-8.26 | lower |
| `hit_infight[v]` | expected damage of one parent hit at the steady state: direct + extra hits + flat-per-status + credited DoT of its procs, `Σ_{s ∈ {SLA, TOX, ELE, GAS, BLA}} η_{s,hit}·L_s[v] + η_{HEA,hit}·L_HEA[v]` with `L_HEA[v] = H_burst[v]/λ_HEA` (the steady Heat credit per proc: every pulse is credited to its procs, 4.6, so the credit rate equals the Heat damage rate); the EV counterpart of "avg per bullet (total)" | higher |
| `hit_fresh[v]` | expected damage of the first shot on the fresh target: all its instances at the first fire time (pellets, zero-delay children, extra hits) + the credited DoT of their procs: every tick, pulse and Blast hit at its own landing time with the target state produced by that shot's procs alone (no later shot), with normal expiry (a Viral proc of the shot stops amplifying its ticks after D_VIR; ticks at the expiry time land, 4.3); Heat: the episode ignited by that shot alone | higher |
| `ammo_window` | Σ c_i over shots with s_i ≤ t_win (`06`) | tie-break, lower |
| `ammo_to_kill` | E-8.21 | tie-break, lower |
| weighted objective | `Σ_u w_u·M_u` over units u (e.g. normal attack and heavy attack, or the members of a family) with user weights w_u ≥ 0 and one DPS-type metric M (all members the same metric and view); for TTK only the combined unit of T-22 on is allowed (a sum of per-attack TTKs means nothing) | higher |

TTK metrics (`ttk`, `ttk_noarmor`, `ammo_to_kill`) are defined only for a single attack or a T-22-on group; for a
group sum (T-22 off) the optimizer refuses them (no group TTK exists, section 9).

```text
E-8.26 TTK ordering of two builds A, B
  band(X) = 0 if killed by t_ttk_max, 1 if extrapolated, 2 if never
  A better than B iff band(A) < band(B), or equal bands and:
     band 0: T_EV(A) < T_EV(B), or T_EV equal and κ(A) > κ(B)
     band 1: T_EV(A) < T_EV(B)
     band 2: larger fraction of the total pool (OG + SH + HP) removed by t_ttk_max on the expected path of E-8.21
             (the stuck layer has rate 0 by definition of the band), then higher dps_sustained[auto]
```

The search window equals the scenario's t_win; the optimizer never changes the scenario it was given (contrast `I:opt`
Q10, `I:feat` quirk 14). MC confirmation of a finalist: `dps_window_mc` mean, CI95 and the deviation flag of 8.11 with
the same seed for every finalist (common random numbers, E-8.16). When the metric is `dps_window` and the flag fires
for any finalist, the optimizer reports the flag with its gap classes and re-ranks the finalists by the MC mean
(Decision: MC is the game-faithful evaluator, overview 4.3); EV ranks stay visible next to the MC ranks. For the other
metrics the flag is reported and the ranking is unchanged.

---

## 11. Compatibility profile and oracle procedure

### 11.1 Compat flags owned by this document

The compatibility profile (overview 4.6, 5.1) switches every register row whose Toggle is `Compat`. It is an **MC-only**
profile (Decision): several Compat rows are path-dependent and have no expectation form in the algorithm of section 5
(D-017 whole-percent chances, D-027 per-proc Heat timers, D-029 Viral and Magnetic keeping their last value, D-030
own-hit timing, D-031 frozen tick terms, D-036 truncated durations, D-086 credited accounting), so the EV evaluator
returns "not available" for a compat scenario. The resolved stats of ST-1 to ST-7 are shared, so the deterministic
panels (OC1 of 11.3) exist in both profiles. Oracle comparisons use the MC outputs. Rows owned here:

| Row | Compat behaviour |
|---|---|
| D-086 | `dps_window` and every credited quantity use the inspiration's credited accounting: each proc's full DoT counts at its hit with the target state at the hit; Nightwatch Napalm credited for `min(6, floor(t_win − i·t_win/N))` s with i the hit index (`JS L10772-10774`, `I:agg` Q8) |
| D-087 | the headline DPS shown is `queue_dps_parity` (median of index buckets) |
| D-089 | TTK per the inspiration (`JS L13593-13804`, `I:enemy` §9.2): horizon t_win (D-804), per attack, base durations and tick counts (SD ignored), Blast and Magnetic break ignored, DoT tick values frozen from the source hit's record for the layer current at the tick, Heat sources summed per pulse on a shared 6 s timer (refreshed by every Heat hit) with the first pulse 1 s after the first proc, overflow discarded on every layer (D-049); ties per D-803 |
| D-801 | the per-hit quantities of E-8.22 take the inspiration's values (`JS L12197-12238`, `L11998-12010`; `I:agg` §6, §7.4, Q10): `direct_h` = curr_dmg + xataTotal + dmgOnStatusEff + Toxic Lash direct hit; `status_h` = Σ_s one-tick value × curr_duration × (count of s in the hit) + Toxic Lash tick × its duration; the per-hit total that fills the `queue_dps_parity` buckets and gives `median_total` is the inspiration's `totalDmg` (its own credited status sum, Xata's Whisper with the headshot factor `hc` twice, the Toxic Lash hit and ticks, Debilitate, napalm, Reconifex Heat); `avg_total` stays `avg_direct + avg_status`, which excludes napalm, Reconifex Heat and Debilitate |
| D-802 to D-804 | as listed in section 15 |

The shield gate (D-048) and every row marked `—` have no compat switch; scenarios that touch them are not oracle-safe.
Inspiration TTK behaviours that belong to `—` rows are therefore not switched either: Gas instances beyond 10 skipped
(D-035), Toxic Lash's 8 ticks and its doubled direct hit and ticks (D-037, D-805), and the Toxin-only `armorFull` map of
the Armor-row TTK on shielded targets (D-052; the former D-806). The averaging statistic (D-088) needs no switch: parity
outputs report the median over runs in every profile (8.8).

### 11.2 Oracle harness

The inspiration runs offline in headless Chromium (`I:oracle`). Procedure:

1. Export the scenario to an inspiration link (architecture); refuse scenarios that the link cannot express.
2. Options that the link does not carry (EC-08-24: queue length = t_win, combine-attack groups, heavy start counter,
   light-combo option, stance combo, party size) are set after the page has loaded, through the page's own controls
   with DOM input events, and every value is read back from the page before the run; a scenario whose options cannot
   be set and read back is refused. Until the harness implements a control, scenarios keep that option at the page's
   default (t_win = 20 s, no combined groups, default stance and combo options, party size 1).
3. Weapon data: the harness serves `getWeapon.php` from `research/external/inspiration/samples/` (`I:oracle`). A weapon
   that is not cached is fetched once from the live site by a manually run script (one request at a time, a few
   seconds apart), stored with the fetch date next to the samples (gitignored, never committed) and used only as the
   inspiration's input in oracle runs, never as engine data (CLAUDE.md: the bundle is a test oracle, not a data source).
4. For r = 0 .. R − 1 (R ≥ 200), load the page with `Math.random` replaced before any page script runs by
   mulberry32 seeded with r: `a = (a + 0x6D2B79F5) | 0; t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^
   (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296` (Decision: fixed generator so recordings
   are reproducible).
5. Read per attack and panel: "Random queue dps", the three "avg per bullet" values, "median per bullet (total)", TTK;
   and the weapon and enemy panels. Record the bundle hash, the weapon file's fetch date and the run date.

### 11.3 Comparison classes and tolerances

```text
E-8.25 Oracle comparisons (compat profile unless stated)
  OC1 deterministic panels; the inspiration field and our counterpart:
     per-type damage and Total of the attack panel (`typeDamageVal`, JS L9470-9486: the I:stat §6 value dmg[type]
        after base mods, CO at the static count of D-056 and physical mods, times the cyte factor; before faction,
        quantization and crit; toFixed(1), Total toFixed(2)) ↔ our compat r_t·Y (E-2, E-11) with n from T-05
        weapon_types, before quantization, (1 + F) and G
     CC, CDM (display value, I:stat §9.1), SC, MS, fire rate or attack speed (setAdditionalMods, JS L8843-9094) ↔ our
        E-5, E-4, E-8, E-10 and modded FR or AS at the fresh state with conditions per T-02 (8.10)
     enemy pools, armor, DR (enemy panel) ↔ 05 scaled values
     match: ours rounded half away from zero to the number of decimals of the displayed string; pass iff equal to
        the displayed value or one unit of its last decimal away (the page rounds binary values with toFixed)
  OC2 statistical: X = inspiration per-run values (R runs), Y = our compat MC per-run values (n_runs ≥ 1000, seed 0),
     each Y truncated or rounded the way the page displays it (parseInt for "Random queue dps", round for the
     per-bullet values, toFixed(3) for TTK); medians m_X, m_Y with distribution-free CI99 [X_(l), X_(u)],
     [Y_(l), Y_(u)] (E-8.19); widening w_X = max(0.01·m_X, q), w_Y = max(0.01·m_Y, q), q = the display quantum
     (1 for the DPS and per-bullet values, 0.001 s for TTK);
     pass iff [X_(l) − w_X, X_(u) + w_X] ∩ [Y_(l) − w_Y, Y_(u) + w_Y] ≠ ∅
     quantities: queue_dps_parity, avg per bullet (total), median per bullet (total), TTK median
     TTK with runs not killed (+∞): both medians +∞: pass; exactly one +∞: fail; otherwise the rule above, an interval
        end at +∞ stays +∞ (the widening uses the finite median)
  OC3 game profile: OC2 with our game-profile MC, only on scenarios that touch no register row
  validity: OC1, OC2 only when no touched row has Toggle "—"; OC3 only when no row is touched
```

The 1 % widening absorbs float noise in the comparison of statistics; the display quantum covers the page's integer
display of small values (an average of 3.4 shown as 3). Neither is a modelling tolerance: compat reproduces the
mechanics.

### 11.4 Touched rows and oracle-safe scenarios

A register row is **touched** by a scenario when the scenario contains the row's trigger feature: the code path that
implements the row's Decision runs at least once on an input that the row's two behaviours can treat differently.
Detection is by feature presence, not by comparing the two behaviours' values: every code path that implements a
register row records the row ID in the trace when it runs (conservative: a run whose two values happen to agree still
touches the row). Predicates for the overview rows whose feature is not a single named mechanic:

| Row | Touched when |
|---|---|
| D-001, D-002 | an attack has ≥ 2 damage types after combination; D-002 also when a multiplier the inspiration puts inside its rounding (`I:stat` §7: Eclipse, `multiple`, frozen, Broadhead, Mecha, Madurai, Cyte, CO, Sharpshot) is ≠ 1 |
| D-003 | some instance can crit (CC > 0) |
| D-017 | a crit, status or multishot chance that is not a whole percent |
| D-018 | frac(CC) ∈ (0.99, 1) for some instance |
| D-020, D-024 | an instance can roll a proc and has ≥ 2 proc types with nonzero weight |
| D-030, D-031, D-036 | a proc is applied (D-031: a DoT tick lands; D-036: a status whose modded duration is not a whole number of seconds) |
| D-053 | a proc on an enemy with a per-enemy status cap |
| D-081 | t_win·EFR < 1, or a reload shorter than 1/EFR inside the window (`I:agg` §2.2, Q3) |
| D-084 | a first- or last-shot effect is equipped (Primed Chamber, Charged Chamber, Synth Charge) |
| D-085 | distance (T-27) > 0 |
| D-086, D-087 | a landed-damage metric (`dps_window`, `dps_series`) is compared with the inspiration; the parity outputs of E-8.22 use the inspiration's accounting and statistic in both profiles |
| D-088 | never (the harness reads seeded single runs, 11.2) |
| D-089, D-802, D-803 | a TTK is compared (D-802: the Armor row needs A0 > 0 to differ; D-803: a DoT tick or Heat pulse exists) |
| D-804 | a TTK is compared and either program kills after t_win or not at all |
| D-801 | a per-bullet output is compared and a flat-per-status (Cascadia Empowered), Xata's Whisper, Toxic Lash, napalm, Reconifex or Debilitate source is present |
| D-805 | Toxic Lash on a target with shields and an attack with Toxin |
| D-090 | T-22 on |
| D-091 | Debilitate or Toxic Lash present |
| D-810 | the unit is a child attack |
| every other row | the mechanic named in its Topic and Inspiration columns is present (e.g. D-007: Phoenix Talons on; D-047: an enemy with unique DR; D-075: a bow) |

Rows of the topic documents (`D-n01` ...) follow the same rule; each owner names the trigger in its row. Static guide
(fallback) for choosing oracle-safe scenarios in the game profile (each item avoids the rows named):

| Requirement | Rows avoided |
|---|---|
| status chance 0 and no forced, impact-triggered or extra-hit procs | D-015, D-016, D-020, D-021, D-024 to D-037, D-053, D-054, D-069, D-070, D-073, D-091 |
| crit chance 0 (D-003 touches every crit) | D-003, D-013, D-014, D-017, D-018, D-023, D-067 |
| composition exact on both grids (one damage type, or ratios that are multiples of 1/16) | D-001, D-002 |
| integer multishot | D-017 |
| AUTO hitscan attack, not bow, charge, burst, beam, spool or battery; `t_win·EFR` not an integer while the magazine lasts the window (`I:agg` Q18) | D-075 to D-083 |
| fire rate ≥ 1/t_win and reload ≥ 1/EFR (`I:agg` Q3) | D-081 |
| no first- or last-shot effect (Primed Chamber, Charged Chamber, Synth Charge) | D-084 |
| distance 0 (T-27) | D-085 |
| enemy at its base level, neutral faction, Steel Path off, no armor, no shield, no overguard, no unique DR | D-042 to D-052, D-055 |
| no progenitor element, Incarnon flat, Phoenix Talons, Violet shard, conversion or faction mods, riven weak-point stats | D-004 to D-012 |
| no buffs, no conditional mods, T-22 off, no child attack | D-022, D-056 to D-068, D-090, D-810 |
| per-bullet outputs only without flat-per-status sources (Cascadia Empowered) and the other D-801 sources | D-801 |
| TTK compared only when both programs kill within t_win | D-804 |
| only `queue_dps_parity` and the per-bullet outputs compared, never `dps_window` | D-086, D-087 |

The trace, not this table, decides validity.

---

## 12. EV versus MC: tolerance and gap classes

```text
E-8.24 Consistency test (overview 4.4) on dps_window per view, MC with n_runs ≥ 1000
  pass iff |dps_window − x̄| ≤ 2.5758293035489004·SE + 0.02·|x̄|
```

Golden scenarios are TV-08-03, TV-08-05, TV-08-06 and TV-08-20 of section 17 plus every scenario in `research/test-cases/`
with an EV and an MC expectation. They MUST pass E-8.24 unless they belong to a listed gap class, in which case the
expected sign and size of the gap are part of the test. Gap classes (the EV approximations of 5.1):

| Class | Cause | Direction and size |
|---|---|---|
| G-1 | Poisson count for Bernoulli or deterministic proc arrivals (A-3) | EV low when few events fall in a duration (presence understated): −2.32 % in TV-08-05 (at most 5 earlier events in the Viral duration, Bernoulli 0.5 per event); −0.73 % in TV-08-06(a) (21 events, Heat presence 0.3 per event against the Poisson 0.259); forced procs on slow weapons worst |
| G-2 | Multinomial type correlation within a pellet; crit-triggered procs (A-2) | not yet measured; expected small (UNVERIFIED) |
| G-3 | Frozen thaw and discarded Cold procs (A-8) | EV high on Frozen-dependent bonuses (Melee Careen, cold_cd 1.0) |
| G-4 | Blast detonation timing and clearing (A-9) | window edge only; presence of BLA overstated near detonation |
| G-5 | Minimum of 1 and attenuation at conditional expectations (A-7) | only when per-type damage is near 1 or on attenuated bosses |
| G-6 | Conditional stacks independent of statuses; Poisson triggers (A-6, E-8.15) | `simulated` T-02 only |
| G-7 | Capped Heat pool (E-8.8) | status-capped targets with Heat |
| G-8 | Mixed Heat durations in one unit (5.4) | Nightwatch Napalm with other Heat sources |
| G-9 | Knockdown and Lifted exclusivity (5.3) | CO count only |
| G-10 | EV TTK is an expectation path without shield gate (7.2) | no tolerance; TTK is compared only as MC median vs oracle |
| G-11 | Melee combo count on the expected path: EV uses M(c̄) and M_b(c̄) of the mean count, MC the live count (`07` 6.9, mechanic ids `combo_acc`, `combo_rc`) | melee with random combo gains only; size not yet measured (UNVERIFIED) |
| G-12 | Heat pulses: the CO term of each pooled proc's seed and the pulse-time enemy factor read the same statuses (Viral, Corrosive) at two times and are multiplied as expectations (A-11) | EV low; −0.07 % (b_co = 0.8) and −0.10 % (b_co = 1.6) measured by simulation (2 shots/s, η_HEA 0.3, η_VIR 0.5 per shot, D = 6, t_win = 20, Adding CO counting Viral and Heat, Y0 = 265; 200,000 coupled and decoupled runs, SE 0.007 % and 0.011 %) |
| G-13 | Steady mode uses the time-average live count μ_s = λ_s·D_s, while a periodic schedule's consumers see the count at their arrival time (A-10, OQ-806) | EV steady metrics high; μ overstated by up to one event's procs, about 1/(D·r) relative for r events per second: +6.45 % in TV-08-17 (1 shot/s, D = 6); below 1 % for the Braton of TV-08-20 (29 pellets/s), where it nets −0.16 % with G-1 |
| G-14 | One status with procs of different durations in one unit on a capped target (A-4: Toxic Lash Toxin procs carry +0.25 SD, D-037; Nightwatch Napalm Heat) | capped targets only; direction and size not yet measured (UNVERIFIED) |
| G-15 | Xata's Whisper detonation extra hits are not produced in EV (A-9; `04` E-4.13a) | EV low by their damage; Xata's Whisper with Blast procs only |
| G-16 | The flat armor strip s_flat (E-8.9 step 5) enters at its expectation where it is random (Melee Duplicate copies, Metal Auger ticks with existence probabilities) | EV low (1 − DR is convex in A_eff, which is linear in s_flat); small |
| G-17 | Class (c)/(u) on/off effects: EV keeps the state of the inducing instance for every tick of its DoT (perfect correlation, `04` 4.3), MC switches at the process boundaries (A-12) | EV high on products such as the Mecha Empowered and Vauban second dip when the on-time is short against the DoT duration; size not yet measured |

The steady metrics (`dps_burst`, `dps_sustained`) have no MC counterpart and are not tested by E-8.24; they are checked
against the late-window average of a long uninterrupted MC run, with the expected gap of G-1 and G-13 (TV-08-17,
TV-08-20).

---

## 13. Worked example

Steady state of the overview's example (00 §3.8: Braton Prime with Serration, Split Chamber, Point Strike, Vital Sense,
Infected Clip, Stormbringer, Primed Bane of Grineer, Speed Trigger; Lancer level 100, Steel Path, body hits; checked with
python3, numpy float32 for Q):

1. Pellets per second 1.9 × 15.328 = 29.1232; proc rates (00 §3.8 step 8) IMP 0.168267, PUN 0.925471, SLA 1.598540,
   COR 4.879754 /s; μ = λ·D with D = 6, 10, 6, 8 s: IMP 1.009604, PUN 9.254706, SLA 9.591241, COR 39.038032.
2. PUN (cap 5): E[min(N, 5)] = 4.929175 → E[CC] = 0.30 + 0.05 × 4.929175 = 0.546459; E[c] = 1 + 0.546459 × 3.401075 =
   2.858547 (CC < 1, linear, E-8.9 step 3).
3. COR (cap 10): P(n = 10) = 0.99999999; E[1 − DR] = 0.714868 (A_eff = 271 in every state with weight).
4. Direct per pellet (armor view): Σ_t W_t·φ_t = 539.109375 (IMP 8.985156 × 1.5, PUN 49.418359, SLA 85.358984,
   COR 260.569531 × 1.5; non-crit, before armor) × 0.714868 × 2.858547 = 1101.662009; × 29.1232 = 32,083.923.
   Health view: 539.109375 × 2.858547 = 1541.069560 per pellet; × 29.1232 = 44,880.877.
5. Slash DoT (Cinematic, no armor, no cap): λ_SLA × 6 × 78.533656 × E[c] = 2,153.158 (Little's law, 5.7).
6. `dps_burst[armor]` = **34,237.081**; health view 44,880.877 + 2,153.158 = **47,034.035**. The overview's crude
   state-B figure (34,381.33) took the caps as certain; the Poisson expectation lowers E[c] from 2.870591 to 2.858547.
7. Sustained (ρ = 0.694733): λ and μ scale by ρ (PUN μ 6.429546 → E[min] 4.592625, E[c] 2.801315; COR still capped):
   `dps_sustained[armor]` = 21,843.390 + 1,465.920 = **23,309.310**; health **32,021.863**.
8. `dmg_per_shot[armor]` = 34,237.081/15.328 = 2,233.630; `dmg_per_mag` = × 75 = 167,522.25.

---

## 14. Edge cases

"→ nn" marks an item owned by document nn and listed here only because the research put it near this topic.

| ID | Rule | Source | Where |
|---|---|---|---|
| EC-08-01 | A shot exactly at t_win counts (closed window); the inspiration includes it in its loop branch but excludes it in the no-reload branch (`floor(T·Ca)`) | `I:agg` Q18, `JS L12311-12321`, `L12353` | E-8.1a; D-081 (→ 06); oracle-safe rule 11.4 |
| EC-08-02 | A projectile fired before t_win that lands after it does not count; its procs do not exist in the view pass | `06` 6.3, 8.5 | E-8.1a, 6.3 |
| EC-08-03 | A DoT tick, Heat pulse or Blast hit landing exactly at t_win counts; later ones are pending | overview 4.1 | E-8.1a, 4.6 |
| EC-08-04 | No instance lands in the window (for example charge time > t_win): every window metric is 0; parity buckets all 0, median 0. The inspiration also returns 0 for fire rates below 1/t_win because its loop bound `Pa = floor(t_win·rate)` is 0 although a shot at t = 0 exists; ours keeps that shot | `I:agg` §2.2, Q3; `JS L12299-12321` | E-8.12, E-8.22; D-081 (→ 06) |
| EC-08-05 | Every DoT is credited in full at its hit in the inspiration, also past the window | `I:agg` §6, `I:queue` QM24, `JS L10982-10986` | 4.6; D-086 |
| EC-08-06 | Napalm (Kuva Ogris "Rocket Explosion") is the only inspiration DoT truncated at the window, by hit index time | `I:agg` Q8, `JS L10772-10774` | compat D-086 (11.1); blaze → 06, 03 |
| EC-08-07 | The inspiration spreads per-hit totals over 1-s buckets by hit index, ignoring timestamps (reload gaps and bursts vanish) and shows the median | `I:agg` §7.2-7.3, Q15; `I:feat` quirk 4 | E-8.22; D-087 |
| EC-08-08 | "Average over cycles" runs for 5 s wall clock (CPU-dependent sample count); its "Avg" tip is a median; per-bullet direct/status values keep the last run | `I:agg` §9, Q17; `I:enemy` quirk 21; `I:feat` quirk 3 | 8.8; D-088 |
| EC-08-09 | `getStatusDamageValue` multiplies one tick by `curr_duration \|\| 1` and by the count of that status in the hit; Gas is credited once per hit as `gasStacks × tick` | `JS L12197-12216`; `I:status` Q22 | D-801, D-035 (→ 03) |
| EC-08-10 | Per-bullet averages exclude napalm, Reconifex heat and Debilitate that `totalDmg` includes; Xata gets the unique-DR factor twice in `totalDmg` | `I:agg` Q10, `JS L12217-12232`, `L11998-12010` | D-801 |
| EC-08-11 | "bullet" means one hit entry: pellet, merged beam tick or melee hit, not a trigger pull | `I:agg` §7.4 | 8.8 parent hit |
| EC-08-12 | All attacks of a weapon are simulated and reported separately; nothing sums them; TTK is per attack | `I:agg` §7.5; `I:enemy` quirk 19; `I:feat` §2 | CD-12, section 9 |
| EC-08-13 | Combined timelines pair attacks by hit index and every combined proc lasts to the window end (no `time` field) | `I:agg` §8, Q4; `JS L12540-12549` | section 9; D-090 |
| EC-08-14 | Index-mode status extension (Debilitate, Toxic Lash) uses `round(N/T)` hits per second; below 0.5 hits/s the procs never reach the timeline | `I:agg` §5, Q9; `JS L10295-10310`, `L10349` | time-based only; D-091 |
| EC-08-15 | Health panel = damage with 100 % armor stripped, not damage to an unarmored enemy; its TTK ignores armor; Health, Shield and Overguard rows share one TTK | `I:feat` §12 and quirk 17; `I:agg` §7.4; `JS L12694-12715` | E-8.1b; D-802 |
| EC-08-16 | With shields and a Toxin attack the inspiration's health and armor maps hold only Toxin; its armor TTK keeps that map after the shield breaks | `I:enemy` quirk 14; `I:queue` QM16 | D-504 (→ 05); D-052 (→ 05, absorbs the former D-806) |
| EC-08-17 | Inspiration TTK: hits win ties against ticks, Heat pulses before other ticks; base durations; Blast never; Toxic Lash 8 ticks; Gas instances beyond 10 skipped; no extrapolation | `I:enemy` §9.2, quirks 16, 17; `JS L13593-13804` | D-089, D-803, D-804 |
| EC-08-18 | Inspiration TTK applies the Toxic Lash direct hit and every Toxic Lash tick to health twice (shield record, which carries the ×3 shield factor, and health record), in the shield phase and in the health phase, on shielded targets with Toxin attacks | `I:enemy` quirk 18; `JS L13735-13741`, `L13757-13759`, `L13787-13793`; `I:queue` §6.2 | D-805 |
| EC-08-19 | Target-side terms of DoT ticks frozen at the inducing hit in DPS and TTK | `I:enemy` quirk 20; `I:queue` §4 | D-031 (→ 03), D-089 |
| EC-08-20 | TTK window = DPS window; "not killed" instead of a time; the optimizer extrapolates EHP/DPS with EHP excluding armor | `I:enemy` §7, open question 9; `JS L16019-16051` | 7.2 extrapolation; D-804, D-807 |
| EC-08-21 | The optimizer's `hit_fresh` adds the status sum on top of `totalDmg` (DoT counted twice); `hit_fresh` has no UI radio | `I:agg` Q10; `I:opt` Q14; `JS L15726-15754` | section 10; D-808 |
| EC-08-22 | The optimizer's fast estimate uses `a/101` chances, one accumulator per percent shared by all mechanics, and a golden-ratio `Math.random` | `I:opt` §4.3, Q1-Q3 | not reproduced; EV + Philox (D-809) |
| EC-08-23 | The optimizer's search window (`round(100/speed)` clamped 5-20 s) differs from the honest-choice window (the user's t_win); the global window may stay changed after a run | `I:opt` §2, Q10; `I:feat` quirk 14 | section 10: one t_win |
| EC-08-24 | Queue length, combine-attack groups, heavy start counter, light-combo option, stance combo and party size are not in the inspiration's build link | `I:feat` §15, quirk 13 | all are scenario fields (4.1) |
| EC-08-25 | Every inspiration recalculation draws a new random queue; ours is reproducible from `seed` (default 0); a "new random queue" action increments the seed | `I:feat` §11; overview T-29 | E-8.16 |
| EC-08-26 | Fractional chances are rounded to whole percent before rolling in the inspiration | `I:agg` §11; `JS L10005`, `L10012`, `L10054`, `L10679` | D-017 (→ 02); E-8.17 exact |
| EC-08-27 | The wiki's DPS formulas assume full beam ramp and spool and ignore status damage; ours add DoT by Little's law and use 06's steady means | `W:Damage/Calculation` L444-459, L531-560 | E-8.14; ramp, spool → 06 |
| EC-08-28 | The wiki's "Normal Shot" (floor tier) and "Critical Shot" (ceil tier) are bounds, not expectations; the average shot is linear in CC | `W:Damage/Calculation` L356-392 | crit-tier distribution 8.4 |
| EC-08-29 | An infinite-ammo state has no reload only while it is active (Arcane Pistoleer: 12 s of infinite ammo, 60 % on a pistol weak-point kill, since U44.1.3); an Incarnon form has a finite charge magazine and reverts when it is empty, so ρ = 1 holds within one activation (`06` E-6.29); dps_sustained = dps_burst only for an attack whose whole schedule is in such a state | `F:addmul` (Arcane Pistoleer row, item 26); `F:weapon` §8.3, §13 | E-8.14; ρ → 06 |
| EC-08-30 | Shield gate: only the breaking hit and hits in the next 0.1 s are reduced; EV has no gate (TTK only) | `F:enemy` §4.3 engine note; `W:Shield` L341 | E-8.21; gate → 05 |
| EC-08-31 | Shields do not regenerate in either evaluator; MC flags runs where regeneration could have started | `05` 10.5; `F:enemy` §4.5 | 7.1 |
| EC-08-32 | Kill inside a shot: later pellets, its extra hits and later shots at the same time are not processed; shots to kill count the killing shot | `04` 6.2 | 6.4, E-8.20 |
| EC-08-33 | One-shot kill (TTK at the first fire time, possibly t = 0): T_EV = that time, κ ranks overkill | `I:opt` §4.5 (one-shot band) | E-8.26 |
| EC-08-34 | No damage can reach health (all immune, zero damage): not killed, T_EV = +∞ ("never") | – | E-8.21 |
| EC-08-35 | Median TTK = +∞ ("not killed") whenever qnt(0.5) involves an infinite order statistic: more than half of the runs not killed (R odd) or at least half (R even); the mean over killed runs is still reported | `JS L13269-13273`, `L14319-14345` (null → ∞) | E-8.19, E-8.20 |
| EC-08-36 | A view exists only when its layer exists; `auto` = armor if A0 > 0 else health | overview 4.5; `I:opt` §2 | E-8.1b, section 10 |
| EC-08-37 | Overguard-dependent caps (Cold 4, VIP Puncture 3) in the shared view timeline: on for the whole window when the enemy spawns with overguard | `03` E-3.2, OQ-321; `05` §18, OQ-529 | 4.5; OQ-804 |
| EC-08-38 | A Viral-immune target ignores T-06; immunity to "all" removes every proc and status term: EV skips all statuses | `03` §6.2 | E-8.4 |
| EC-08-39 | Procs nullified by SD ≤ −1 add nothing; a Blast proc then hits at once in P4b | `03` §5.1 | E-8.2 |
| EC-08-40 | Deterministic forced procs (stance, weapon traits, Toxic Lash) are certain, but the Poisson model gives presence 1 − e^{−1} per event | overview 4.2 | A-3, G-1, OQ-801 |
| EC-08-41 | Heat has no steady state: its pool keeps growing while refreshed, so steady metrics use the window mean | `F:status` §4.4; overview 4.2 | E-8.14 |
| EC-08-42 | Heat pulses read the strip of their own episode: the age of the episode is known given its ignition | `03` E-3.8 | E-8.8 |
| EC-08-43 | ELE and GAS ticks landing at the same time consolidate with one "+1"; tick 0 groups only within its shot (P4c) | `03` E-3.5, §7.1 | E-8.11 |
| EC-08-44 | A capped DoT instance that is replaced loses its remaining ticks; survival enters per tick | `03` §6.2 | E-8.11 |
| EC-08-45 | Tau raises the status chance of later procs: steady fixed point; window causal | `02` 4.9; `03` §8.4 | E-8.13, E-8.12 |
| EC-08-46 | CO count multiplies factors that read counted statuses (Viral, Corrosive, Puncture, Cold, Heat): expectation by status group, not E[C]·E[rest] | overview 4.2; `F:addmul` §1.2 | E-8.9 step 4; TV-08-09 |
| EC-08-47 | Sibling pellets add to each other's CO count but not to their debuffs | CD-04; `04` E-4.2 | E-8.10 |
| EC-08-48 | Child instance at the same time as its parent's pellets | – (no source) | 4.2; OQ-803 |
| EC-08-49 | Melee Duplicate and extra hits on the killing instance are not created | `04` 6.2, E-4.15 | 6.4 |
| EC-08-50 | Magnetic break procs exist only in layered passes; EV uses E[n_MAG] at the break | D-054; `03` E-3.10 | E-8.21 |
| EC-08-51 | Non-integer t_win (advanced range): parity outputs are not reported; the last `dps_series` bin is shorter | `JS L12787-12801` (integer buckets) | E-8.22, 8.1 |
| EC-08-52 | n_runs = 1: sd and CIs undefined (reported as none); the median is the single value; E-8.24 not evaluated | – | E-8.19 |
| EC-08-53 | Ammo to kill and ammo in window with cost 0 (infinite ammo): 0 | `I:opt` §4.2 (ammoSchedule absent for infinite) | E-8.20, section 10 |
| EC-08-54 | The optimizer's `prefer` bias, status-duration-only exclusion and regex exclusions are search rules | `I:opt` Q8, Q9, §3.3 | out of scope (`docs/architecture/`) |
| EC-08-55 | Bow double reload, charge cadence, burst delay scaling, battery recharge, beam ammo, first/last marks by index | `I:agg` Q1, Q2, Q5-Q7, Q16 | → 06 (D-075, D-076, D-080, D-082, D-083, D-084) |
| EC-08-56 | Slam forced statuses, 12× light combo default, decay-only heavy combo, uniform melee timing | `I:agg` Q11-Q14 | → 07, 04 (D-062 to D-065) |
| EC-08-57 | The inspiration's layer passes of one attack share their crit and proc rolls through a memo keyed `hitId:label:index`; ours share them by construction (one view pass, same keys in the layered passes) | `I:agg` §7.1; `I:queue` §1 (`JS L10313-10318`) | 4.5, 6.4 |
| EC-08-58 | The inspiration creates Armor and Shield rows from base values but fills them only when the scaled value is > 0; our armor view exists when A0 > 0 even if strips bring A_eff to 0 | `I:agg` §7.4 (`JS L9143-9146`) | E-8.1b |
| EC-08-59 | `dps_window` depends on the window's phase against the magazine cycle (Furis: 144 shots in 20 s = 7.2/s against the long-run 7.14/s); it reads as burst DPS when t_win is shorter than a magazine | `I:agg` §10 | 8.1; `dps_sustained` for the long-run value |
| EC-08-60 | In the layered pass Toxin bypasses shields (direct and ticks) but not overguard; health can reach 0 with shields up | `I:enemy` §9.2; `05` E-5.25 | E-8.21, 6.4; routing → 05 |

---

## 15. Inspiration differences

Rows of the overview register owned by `08`, then new rows. Toggle as overview 5.1.

| ID | Topic | Inspiration behaviour | Game / spec behaviour (source) | Engine (game profile) | Toggle |
|---|---|---|---|---|---|
| D-086 | DoT accounting | every proc's full DoT credited at the applying hit, never truncated (`JS L10982-10986`) | ticks land over time (`03` E-3.6) | landed for every metric; credited only for parity outputs and hit metrics (4.6); compat 11.1 | Compat |
| D-087 | DPS statistic | median of 1-s buckets filled by hit index (`JS L12787-12805`) | – (statistic choice) | `dps_window` = mean landed damage / t_win; `queue_dps_parity` kept (E-8.22) | Compat |
| D-088 | Averaging | 5 s wall-clock loop; "Avg" is a median (`JS L14273-14485`) | – | fixed n_runs; mean and median both shown and labelled (E-8.19, 8.8) | — |
| D-089 | TTK simulation | 11.1 list (`JS L13593-13804`; `I:enemy` §9.2) | `03` and `05` rules, SD honoured, Blast and Magnetic break included | one event model (E-8.18) for DPS and TTK | Compat |
| D-090 | Combine attack effects | timelines merged by hit index; merged procs never expire (`JS L12540-12549`) | – | merge by time with normal expiry (section 9) | — |
| D-091 | Index-mode extension | `round(N/T)` hits per second; 0 below 0.5 hits/s (`JS L10295-10310`, `L10349`) | – | time-based only | — |
| D-801 | Per-bullet averages | "avg per bullet" direct = `curr_dmg + xataTotal + dmgOnStatusEff + toxicLash.direct_hit`, status = Σ one-tick × `curr_duration` × count of the status in the hit (+ Toxic Lash ticks); `totalDmg` also holds napalm, Reconifex heat, Debilitate and Xata × `hc` twice, so avg total ≠ mean totalDmg (`JS L12197-12238`, `L11998-12010`; `I:agg` Q10) | – | one consistent definition (E-8.22): total = direct + credited status | Compat |
| D-802 | TTK rows | Health, Shield and Overguard rows show one TTK computed without armor; Armor row with armor; per attack (`JS L12694-12715`) | a real enemy has armor | headline TTK = pass L (armor applied); parity rows reproduce both (pass L0, L) | Compat |
| D-803 | TTK tie order | at equal times a hit is processed before DoT ticks and Heat pulses; Heat pulses before other ticks (`JS L13719-13800`) | `03` §7.1: P2 ticks and pulses before P4 shots | 4.3 | Compat |
| D-804 | TTK horizon | TTK bounded by the DPS window; "not killed" (`JS L12697-12715`; `I:enemy` open 9) | – | `t_ttk_max` (60 s) and EV extrapolation (7.2) | Compat |
| D-805 | Toxic Lash in TTK | when the attack has Toxin and the enemy has shields, the Toxic Lash direct hit and every Toxic Lash tick are applied to health twice, once from the shield record (whose values carry the ×3 shield factor `Ob`, D-039) and once from the health record, in the shield phase and in the health phase (`JS L13735-13741`, `L13757-13759`, `L13787-13793`; `I:enemy` quirk 18; `I:queue` §6.2) | one extra hit per parent with its own Toxin procs (`04` E-4.13, E-4.14) | once | — (every Toxic Lash scenario also touches D-037) |
| D-806 | (retired) | the Toxin-only `armorFull` map of the Armor-row TTK on shielded targets with Toxin attacks (`JS L11365`; `I:queue` QM16) is the second half of overview D-052 (owner `05`, Toggle —); the Toxin-only DPS panels are `05` D-504 (Compat) | – | the ID stays reserved; the behaviour is handled under D-052 (OQ-809) | — |
| D-807 | Optimizer TTK score | EHP = health + shield + overguard (no armor); not killed: −(T + EHP/DPS); one-shot: 0.001 + ln(1 + hit/EHP); sub-ms tie-break (`JS L16019-16051`; `I:opt` Q7) | – | E-8.21, E-8.26 | — |
| D-808 | Optimizer hit metrics | `hit_fresh` = Σ (totalDmg + status sum) of the earliest hits (DoT twice); `hit_infight` = rounded avg per bullet of the last run (`JS L15726-15797`) | – | section 10 (DoT once, expectations) | — |
| D-809 | Optimizer fast estimate | stratified `isProc` with `a/(c + 1)`, one accumulator per percent for all mechanics, golden-ratio `Math.random` (`JS L15688-15725`; `I:opt` Q1-Q3) | – | EV for search; Philox MC with common random numbers for confirmation | — |
| D-810 | Child attack schedule | every attack, children included, gets its own queue from its own stats; Catabolyst's reload attacks are continuous 12/s streams with the weapon's magazine (`I:agg` §2, §7.5) | children are created by their parent's hits (overview 2.3; `F:weapon` §10.1; `06` 9.2-9.3) | a child unit runs on its parent's shot times (section 9) | — |

Rows of other owners used by the procedures here: D-017 (exact chances, `02`), D-030 and D-031 (event order, tick-time
target state, `03`), D-048 and D-049 (gate, overflow, `05`), D-054 (Magnetic break, `03`), D-081 (shot-count bound and
the window boundary, `06`), D-052 and D-504 (Toxin-only maps, `05`).

---

## 16. Open questions

Overview questions these algorithms depend on, with the current decisions: OQ-011 (overflow and gate: CD-08 in E-8.21
and MC), OQ-302 (Heat pulse grid anchored at ignition: E-8.8), OQ-008 and OQ-304 (caps on DoT and the Heat pool:
E-8.11, E-8.8), OQ-028 (siblings do not share debuffs: E-8.10).

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-801 | **Proposal to the manager** (changes overview 4.2): model the live count of s as (deterministic forced procs in the duration window) + Poisson(rolled mean), and per-event presence as the exact `P(e applies ≥ 1 proc of s)` from `02` E-2.18, instead of a pure Poisson | pure Poisson (A-3) | EV-versus-MC gaps on the golden corpus with and without the change; TV-08-05 (−2.3 %) and forced-proc melee scenarios |
| OQ-802 | **Proposal**: default T-22 on for families (parent and children hit the same target in game) in the game profile | off (overview 6.1) | none needed for the game fact; UI decision by the manager |
| OQ-803 | Is a child instance (explosion) that lands at the same time as its parent's pellets part of the same shot (it does not see the pellets' procs) or a later instance (it sees them)? | same shot, after the pellets (4.2) | Simulacrum: hitscan weapon with a radial child (Opticor Vandal or Ferrox), impact applying a forced Viral proc on a fresh target: compare the radial number with and without the Viral source |
| OQ-804 | Overguard-dependent caps in the shared view timeline and EV: the spawn state of `03` E-3.2 (restated by `05` §18), or `03` OQ-321's per-view exception (`overguard` view with the caps, the other views without, which needs a second status timeline)? Manager decision | `03` E-3.2: caps on for the whole window when the enemy spawns with overguard; live state in the layered passes (4.5) | the game rule is sourced (Cold 4 while overguard is present, `W:Overguard`, U36); `05` OQ-529's test (Cold stack count on an Eximus before and after its overguard breaks) confirms the layered rule; the view rule is a modelling choice |
| OQ-805 | Mixed Heat durations in one pool (Nightwatch Napalm 3 s procs with 6 s procs): does each proc keep its own duration inside the episode for the expiry (`03` E-3.7 uses the pool maximum)? | EV: one D = maximum (5.4) | Ogris with Napalm and a Heat mod: time the Heat icon after the last weapon Heat proc |
| OQ-806 | **Proposal to the manager** (changes overview 2.4 `stack_mean` and 4.2 "Steady: μ_s = λ_s·D_s"): define the steady μ of each consumer class as the long-run mean it sees at its arrival (a shot of a constant-rate schedule: η·#{k ≥ 1 : k·T_per < D}; a tick: the same count at its offset), or compute the steady metrics as the per-period average of the window walk started from a converged state | time average λ_s·D_s (A-10, gap G-13) | none needed for the arithmetic (TV-08-17: +6.45 % on a 1 shot/s weapon); the late-window MC average of slow weapons (bows, snipers, launchers) against `dps_burst` |
| OQ-807 | Request to `03` and `06`: restate their time comparisons (`03` §7 "equality is exact", `06` E-6.11 ε = 1e-9) on the microsecond grid of E-8.0a, so the three documents use one time model | E-8.0a binds both evaluators | consistency only; no game evidence needed |
| OQ-808 | **Proposal** (new toggle): a per-attack hit fraction (accuracy, spread at the distance T-27) for shotguns and AoE children; today every instance hits the target (4.1) | all hit | `W:Damage/Calculation` §Average Shot/Hit assumes all projectiles hit; per-weapon spread data has no data source (`docs/data/`); manager decision |
| OQ-809 | The former D-806 (Toxin-only `armorFull` map of the Armor-row TTK) is the second half of overview D-052 (Toggle —). Request to the manager: restate D-052 to name both halves (armor-layer `elemRes` skip and the TTK map), and decide whether either half gets a Compat switch | D-806 retired; D-052 covers it with Toggle —, so shielded Toxin scenarios are not oracle-safe for the Armor-row TTK | none (register bookkeeping) |
| OQ-810 | Encumber attribution: `02` E-2.17/E-2.18 put the event's single Encumber proc on instance i with probability e_i (later instances count it), `04` E-4.2 on pellet 0. Request to align `04` | `02` (owner of per-instance proc expectations, 5.6) | Simulacrum: a secondary shotgun (Kuva Brakk) with Secondary Encumber and an Adding CO mod on a fresh target: in shots where Encumber fires, which pellets' numbers carry the extra CO step |
| OQ-811 | Melee Duplicate: `04` E-4.15 says the copy reads "C and the target state ... after the parent's procs"; `03` §7.1 applies a shot's procs in P4b after all its instances, so under CD-04 the copy counts the parent's procs in n but reads the debuffs of σ_S. Also: does a duplicated extra hit (the Toxic Lash or Xata's Whisper bug, `04` 6.4) repeat the extra hit's value? | CO count with the parent's procs, debuffs of σ_S (4.2); the duplicated extra hit repeats the copied extra hit's value (TV-08-28) | Simulacrum: melee weapon with Melee Duplicate at rank 5 and a forced Viral proc (stance hit or Toxin/Cold build) on a fresh target: is the duplicate's number Viral-amplified? With Toxic Lash: compare the two Toxin numbers of a tier-1 crit |

---

## 17. Test vectors

All values computed with python3 (f64; Philox and FNV-1a by an independent implementation checked against the published
known answers). Engine tolerance 1e-9 relative unless stated. "MC" vectors that give a mean from seed 0 are exact
outputs of the reference implementation of E-8.16 to E-8.19 and MUST be reproduced to 1e-9; "MC within CI" vectors give
an analytic or high-precision reference that the engine's MC mean MUST approach within 2.5758·SE.

**TV-08-01 Philox4x32-10 known answers (E-8.16).** Counter (c0, c1, c2, c3), key (k0, k1) → output words:
(0, 0, 0, 0), (0, 0) → (0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8);
(0xffffffff × 4), (0xffffffff, 0xffffffff) → (0x408f276d, 0x41c83b0e, 0xa20bc7c6, 0x6d5451fd);
(0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344), (0xa4093822, 0x299f31d0) → (0xd16cfe09, 0x94fdcceb, 0x5001e420,
0x24126ea1).

**TV-08-02 Keyed draws (E-8.16, E-8.17).** `rng_mech`: ms_pellets 0xf351e291, crit_roll 0xa4bc7899, crit_vig 0x2f2fe9e4,
sc_count 0xe545931b, sc_type 0x95eb190c, cedo_type 0xe304b3a9, ib 0x3d387452, welt 0xecb57171, hm 0x643af04e,
st 0x4b522242, enc 0x658e6dcb, enc_type 0x6e512494, deb 0xc1597840, deb_type 0xa6c40bc5, deb_comp 0xc8e04db2
(`02`); combo_acc 0x4feff83f, combo_rc 0xa570f7a1, combo_curse 0x151631cc, glaive_volatile 0xd5e66d86 (`07`); all
distinct.
(a) seed 0, run 3, anchor (a = 1, i = 5, j = 2), "crit_roll", ordinal 0: counter (0x3, 0xa4bc7899, 0x100005, 0x20000),
key (0, 0), output (0x9309936e, 0x97b774da, 0xf1618d73, 0x9d71bd6b), u = **0.5926430734280415**; ordinal 1 (half 1 of the
same block): u = **0.6150167835743386**. With CC = 1.3: tier = 1 + [0.5926 < 0.3] = **1**.
(b) seed 0xDEADBEEF12345678, run 0, shot-level draw (a = 0, i = 0, j = 0xFFFF), "ms_pellets", ordinal 0: counter
(0x0, 0xf351e291, 0x0, 0xffff0000), key (0x12345678, 0xdeadbeef), output (0x5ca4ea64, 0xe2d9566b, 0xa0a1bbbc,
0x3d345ce6), u = **0.8861288082231514**; with MS = 1.9 the shot has 1 + [u < 0.9] = **2** pellets.

**TV-08-03 Crit-only scenario: EV, MC reference, MC within CI.** One attack (index 0), SEMI 2 shots/s, shots at
t = 0, 0.5, ..., 10 (21 shots, t_win = 10), MS = 1.9, per-pellet weapon damage 100 of one type, CC = 1.3, CDM = 2.5 (final,
no quantization step in the test), SC = 0, no Vigilante, health view, neutral target, no armor. Draws: per shot
"ms_pellets" (j = 0xFFFF), per pellet j "crit_roll".
EV: E[c] = 1 + 1.3 × 1.5 = 2.95; `dps_window` = 21 × 1.9 × 295 / 10 = **1177.05**. Analytic per-run sd = **59.414202847**
(Var per shot = 1.9 × 4725 + 0.09 × 295²), SE(1000) = 1.878842.
MC (seed 0, n_runs 1000): first runs 1115.0, 1125.0, 1235.0; mean **1174.89**; sd **57.968942532**; median **1175.0**;
qnt(0.05) **1085.0**; qnt(0.95) **1270.0**; tier histogram over all runs: tier 1 **27,930**, tier 2 **11,916**.
E-8.24: |1177.05 − 1174.89| = 2.16 ≤ 2.5758 × 1.8331 + 0.02 × 1174.89 → pass (and within 1.15 SE of EV).

**TV-08-04 Capped Poisson expectations (E-8.3).** μ = 9.254705777777776, cap 5: E[n] = **4.929175484660303**.
Viral (cap 10): μ = 1.5: E[Vir] = **1.957652222708901**, P(n ≥ 1) = **0.7768698398515702**; μ = 4.5: E[Vir] =
**2.864110608302472**. Corrosive, μ = 7.3: E[corr_strip] = **0.6224531932124506**. μ = 39.038031644444445, cap 10:
P(n = 10) = 0.9999999916927463; with A0 = 1355, E[1 − DR] = **0.7148684510001223**.

**TV-08-05 Viral window walk, gap class G-1 (E-8.4, E-8.12).** SEMI 1 shot/s, shots at t = 0 .. 10 (11 shots,
t_win = 10), one pellet, pure Viral composition, weapon damage 100, CC 0, P(Viral proc) = 0.5 per shot (η = 0.5), D_VIR = 6,
cap 10, health view, neutral. Earlier shots inside (t − 6, t): k = 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5. EV E[Vir] per shot
(Poisson μ = 0.5k): 1, 1.420102005, 1.724090416, 1.957652223, 2.148496059, 2.313416973 (× 6).
EV landed **2213.0842542892**, `dps_window` **221.30842543**. Exact model (Binomial): landed **2265.625**, dps
**226.5625**, per-run sd of dps **25.079220186**; the engine's MC mean MUST lie within 2.5758·SE of 226.5625 (reference
MC, 1000 runs, other generator: 227.895, sd 24.505). Gap EV − exact = **−2.32 %** (class G-1). The deviation flag of 8.11
is a random event here: E-8.24 fails iff x̄ > (EV + 2.5758·SE)/0.98 or x̄ < (EV − 2.5758·SE)/1.02, so with the exact mean
and sd it is on with probability **0.045** at n_runs = 1000, **0.623** at 10,000 and > 0.999999 at 100,000; a test
asserts the EV-versus-exact gap, not the flag state.

**TV-08-06 Heat pool (E-8.8).** (a) Shots at t = 0, 0.5, ..., 10 (21 events), η_HEA = 0.3 per event (Poisson π =
0.2591817793), S·X = 50 per proc (v = 15 per event), X̄ = 0.5, a = 1, D = 6, health view, neutral, no other status,
t_win = 10: EV Heat landed **1562.696453310**, dps **156.269645331** (first ignitions, with Σ_g V_k(g) the pulse total
given ignition at k: t = 0 ι 0.2591817793, Σ_g 2034.2420035; t = 0.5 ι 0.1920065846, Σ_g 1712.2516943; t = 1.0 ι
0.1422419764, Σ_g 1712.2516943). With the per-event presence set to the Bernoulli
value π = 0.3 (algorithm check, v = 15): **1574.190683**, which a simulation of `03` E-3.7 reproduces (200,000 runs:
1574.862 ± 1.434); the engine's MC mean MUST lie within CI of 1574.190683. EV-versus-exact gap −0.73 % (E-8.24 passes).
(b) Algorithm check with presence probabilities given directly (bypassing A-3): events at t = 0, 0.7, 1.9, 2.2, 4.0, 8.5,
9.1, 12.0, 13.3 with π = 0.3, 0.5, 0.2, 0.9, 0.4, 0.35, 0.6, 0.25, 0.5, v = 50·π, X̄ = 0.5, a = 1, D = 6, T = 16:
E-8.8 = brute-force enumeration of all 512 outcomes = **1492.343171**.

**TV-08-07 Heat strip counter, window (E-8.6).** D = 6, h_up = 0.5, h_dn = 1.5. (a) Events at t = 0, 7.0, 7.5 with
η = 1.0, 0.5, 0.5 (π = 0.6321205588, 0.3934693403, 0.3934693403), consumer at τ = 8.25: P_kh(0..4) = **0.1353352832,
0.0877948769, 0.1447492810, 0.2325441579, 0.3995764009**; P_H = 0.6321205588 (equal to brute-force enumeration of E-3.8).
(b) Events at t = 0.25·i, i = 0..39, η = 0.08 each: at τ = 3.3: **0.382892886, 0.066436078, 0.077963460, 0.091490968,
0.381216608**; τ = 7.9: **0.090717953, 0.015740551, 0.018471708, 0.036788015, 0.838281773**; τ = 9.99: **0.047834889,
0.008299873, 0.027390465, 0.052824568, 0.863650205** (simulation of E-3.8 with 200,000 samples agrees within 0.002).

**TV-08-08 Heat strip counter, steady (E-8.7).** λ = 0.6 /s, D = 6, h_up = 0.5, h_dn = 1.5: dead part (x = 0..4)
0.0007465858, 0.0010897190, 0.0026802762, 0.0065924156, 0.0162147259; live part 0.0002239757, 0.0005508914,
0.0013549743, 0.0033326990, 0.9672137371; total **0.0009705616, 0.0016406104, 0.0040352504, 0.0099251146,
0.9834284630**; E[heat_strip] = **0.4971807621** (a 3·10^6 s simulation gives 0.001017, 0.001744, 0.004137, 0.009781,
0.983321).

**TV-08-09 CO count with a correlated factor (E-8.9 step 4, E-8.10).** Adding CO, B' = B_ref = 100, A = 1.65 (Y0 = 265),
b_co = 0.8 per status, ρ_co = 1, single damage type, F = 0, G = 1, CC = 0, health view. Before the shot: μ_VIR = 1.5,
μ_SLA = 0.7, no other status; shot of 2 pellets, each applies Toxin with A_TOX = 0.3. Pellet 0: E[n_0] = 1.2802845361,
exact E[D] = **736.3804759409** (naive E[C]·E[Vir]: 719.2859884353, −2.32 %); pellet 1: E[n_1] = 1.5802845361, exact
**783.3641292859** (naive 766.2696417803). Shot total **1519.7446052267**. Fractional pellet count (E-8.10): with
MS = 1.5 instead of 2, pellet 1 exists with probability f = 0.5 and keeps its conditional value: shot total 736.3804759409 +
0.5 × 783.3641292859 = **1128.0625405838**.

**TV-08-10 DoT survival under a cap, Little's law (E-8.11).** Slash on a status-capped target (cap 4), steady λ = 2 /s,
δ = 1, N = 6, weapon-and-enemy tick value 100: R(m) for m = 0..5 = **0.857123460, 0.433470120, 0.151203883, 0.042380112,
0.010336051, 0.002291791**; expected ticks per proc **1.4968054175**; DoT DPS **299.3610835** (uncapped: 1200).

**TV-08-11 Steady metrics (E-8.13, E-8.14; section 13).** Inputs of 00 §3.8 (Braton Prime build, Lancer level 100 Steel
Path, body hits, no conditions): `dps_burst[armor]` **34,237.0813035**, `dps_burst[health]` **47,034.0352817**,
`dps_sustained[armor]` **23,309.3102941**, `dps_sustained[health]` **32,021.8628948**; components: E[c] burst
2.8585471353, sustained 2.8013154326; E[1 − DR] burst 0.7148684510, sustained 0.7148655281; Slash DoT burst
2,153.1582862, sustained 1,465.9201367; `dmg_per_shot[armor]` 2,233.6300433; `dmg_per_mag[armor]` 167,522.2532464.

**TV-08-12 DoT accounting at the window edge (4.6, E-8.1a).** t_win = 20, health view, neutral, SD = 0, no other
status. (a) One Slash proc applied at t = 17 with tick value 100 (weapon side incl. "+1", enemy side ×1): ticks at
18, 19, 20, 21, 22, 23; landed in window **300** (`dps_window` contribution 15), pending **300**, credited **600**
(compat D-086 contribution 30). (b) One Electricity proc at t = 17, tick 100: ticks at 17 (P4c), 18, 19, 20, 21, 22;
landed **400**, pending **200**.

**TV-08-13 TTK, EV versus MC with the shield gate (E-8.20, E-8.21).** Target SH 100, HP 250, no armor, no overguard,
neutral; AUTO 1 shot/s from t = 0; each shot one instance of 40 non-bypass + 20 Toxin, no crit, no status, body hits.
MC (deterministic): t = 0: SH 60, HP 230; t = 1: SH 20, HP 210; t = 2: breaks the shield (a = 0.5, gate 0.05 on the
other 20) HP 210 − 21 = 189; then −60 per shot: 129, 69, 9, −51 → **TTK 6.0**, shots to kill **7**. EV (no gate):
HP 230, 210, 170, 110, 50, −10 → **T_EV = 5.0**, κ = (60 − 50)/250 = **0.04**, shots to kill **6**. With T-03 `head` on a
target whose head has P_part = 1 and no weak-point or headshot bonus (each shot still deals 40 + 20; weak-point hits
bypass the gate, factor 1): MC t = 2: HP 210 − 20 − 20 = 170, then 110, 50, −10 → **5.0** like EV. With a typical head
(P_part = 3, every shot 120 + 60): t = 0: the shield absorbs 100, the overflow 20 passes at factor 1, HP 250 − 60 − 20 =
170; t = 1: HP −10 → MC and EV **1.0**, 2 shots. Without Toxin (60 non-bypass, HP 300, body): MC 6.0 (HP 299 after the
break), EV 6.0 with κ = (60 − 40)/300 = **0.0666667**.

**TV-08-14 Parity buckets and median (E-8.22).** t_win = 5, per-hit totals [10, 20, 30, 40, 50, 60, 70]: buckets
**18, 36, 56, 76, 94**, median **56**. Totals [300, 0, 600]: buckets **180, 120, 0, 240, 360**, median **180**. t_win = 20,
45 hits of 5: every bucket **11.25**, median 11.25.

**TV-08-15 Statistics (E-8.19).** x = [100, 120, 90, 300, 110, 105, 95, 130, 0, 115]: mean **116.5**, sd
**73.825996626**, SE 23.345829987, CI95 **[70.743014036, 162.256985964]**, CI99 **[56.365127004, 176.634872996]**,
qnt(0.05) **40.5**, qnt(0.10) **81.0**, qnt(0.25) **96.25**, median **107.5**, qnt(0.75) **118.75**, qnt(0.90) **147.0**,
qnt(0.95) **223.5**. TTK runs [2.0, 2.5, ∞, 3.0, 2.25, ∞, 2.75, 4.0]: P_kill **0.75**, median **2.875**, mean of killed
**2.75**, qnt(0.05) **2.0875**, qnt(0.95) **∞**; with four runs not killed out of 8 ([2.0, ∞, ∞, 3.0, 2.25, ∞, ∞, 4.0]) the
median is **∞** (g = 3.5 interpolates between 4.0 and ∞). Median CI99 ranks: R = 200 → (81, 120); R = 1000 → (459, 542).
R = 1 (x = [42.0]): mean 42.0, sd, SE and CIs none, median and every qnt(p) 42.0 (EC-08-52).

**TV-08-16 Oracle comparison (E-8.25).** Inspiration R = 200, median 18,492, CI99 [x_(81), x_(120)] = [18,350, 18,620];
ours R = 1000, median 18,745, CI99 [y_(459), y_(542)] = [18,700, 18,790]. Widened: [18,165.08, 18,804.92] and
[18,512.55, 18,977.45] → intersect → **pass**. With ours [18,900, 18,990] (median 18,945): [18,710.55, 19,179.45]
intersects [18,165.08, 18,804.92] → pass; with ours [19,000, 19,100] (median 19,050): [18,809.5, 19,290.5] → **fail**.

**TV-08-17 Steady mode at arrival times, gap class G-13 (E-8.13, E-8.14).** The weapon of TV-08-05 fired without end
(SEMI 1 shot/s, no reload, one pellet, Viral with probability 0.5 per shot (η = 0.5), D_VIR = 6, cap 10, weapon damage
100, health view, neutral). EV `dps_burst` = 100·E[Vir] at μ = λD = 3.0: **246.2563675003**. A shot sees 5 earlier
events (the one at τ − 6 expired in P1): the Poisson value at that arrival mean 2.5 is 231.3416973248 (G-13 alone
+6.45 %); the exact long-run mean (Binomial(5, 0.5) live stacks) is **235.15625**, which the late-window average of a
long uninterrupted MC run MUST approach within its CI. Gap of the steady metric **+4.72 %** (G-13 +6.45 %, G-1 −1.62 %).

**TV-08-18 A tick at the instance's expiry time (4.3, E-8.11).** One Slash proc at t = 0, SD = 0 (D_SLA = 6, N = 6,
δ = 1), tick value 100, t_win = 20, health view: ticks at t = 1, 2, 3, 4, 5, 6; the instance's presence ends in P1 at
t = 6 and its sixth tick lands in P2 at t = 6: landed **600**, pending 0. A shot at t = 6 does not count Slash in its CO
count; a shot at t = 5.999999 (grid 5,999,999 µs) does.

**TV-08-19 DoT seed CO term against a tick-time Viral factor (E-8.9 step 4b, E-8.11).** SEMI 1 shot/s, shots at
t = 0, 1, 2, ...; every shot applies Viral with η_VIR = 0.5 (Poisson), D_VIR = 6, cap 10. The shot at t_e = 5 (five
earlier shots in the Viral window) seeds a Slash proc whose seed is S = Y0 + b·1{present_VIR(t_e)}, Y0 = 265, b = 80
(Adding CO 0.8 × B_ref 100; Viral is the only counted status); its ticks at u = 6, 7, ..., 11 read Vir(n_VIR(u)) with
n_VIR(u) counting the shots in (u − 6, u), the shot at t_e included. E[S·Vir(u)] per tick: **788.8033300791,
786.8070990141, 785.2733453701, 784.0201120863, 782.9371094454, 782.9371094454**; the product of expectations
E[S]·E[Vir(u)] is 782.9371094454 at every tick (−0.74 %, −0.49 %, −0.30 %, −0.14 %, 0, 0; −0.28 % over the six ticks).

**TV-08-20 Realistic window and steady check (E-8.12, E-8.14).** The Braton Prime build of overview 3.8 (section 13),
Lancer level 100, Steel Path, body hits, t_win = 20. Schedule (`06` E-6.11, δ_rel = 1/FR): 225 shots in three
magazines: shots 0-74 at k/15.328 s (last 4.827766), reload 4.893006-7.043006, shots 75-149 from 7.043006 (last
11.870772), reload 11.936013-14.086013, shots 150-224 from 14.086013 to 18.913779; the third reload starts at 18.979019
and changes nothing in the window. EV: `dps_window[armor]` **22,445.7525988**, `dps_window[health]`
**31,809.8248810** (direct 21,236.7513303 and 30,600.8236125; Slash DoT landed in the window 1,209.0012685).
MC within CI (reference simulation with another generator, 200,000 runs): armor 22,466.39 (SE 2.64, per-run sd
1,181.03), health 31,823.89 (SE 3.63, sd 1,622.81): EV − MC = −0.09 % and −0.04 % (G-1 on the Puncture and Corrosive
presence); E-8.24 passes at n_runs = 1000 (bound ≈ 2.5758 × 37.35 + 0.02 × 22,466 ≈ 545). Steady check: the average over
(20, 40] of an uninterrupted (no reload) MC run, 200,000 runs: armor 34,292.40 (SE 2.32), health 47,110.87 (SE 3.08),
against `dps_burst` 34,237.08 and 47,034.04 of TV-08-11: −0.16 % in both views (G-1 low, G-13 high).

**TV-08-21 Corrosive and Heat together on the armor view (E-8.9 step 5, E-8.6).** Consumer at τ = 8.25 with the Heat
events of TV-08-07(a) (P_kh = 0.1353352832, 0.0877948769, 0.1447492810, 0.2325441579, 0.3995764009) and n_COR capped
Poisson (μ = 0.8, cap 10), A0 = 1000, no other strip: E[1 − DR(A_eff)] = **0.5997781264**; DR at the expected strips
(E[heat_strip] = 0.3504253373, E[c_corr] = 0.1581342071) gives 0.5949612283 (−0.80 %).

**TV-08-22 EV TTK extrapolation (E-8.21).** Health view (A0 = 0), no overguard, t_ttk_max = 60. At 60 s the expected
path leaves SH_rem = 600 and HP_rem = 2900; steady rates d_SH = 200 (non-bypass), d_BP = 50 (Toxin), d_HP = 250. T_SH =
3, HP_1 = 2900 − 50 × 3 = 2750, **T_EV = 60 + 3 + 2750/250 = 74.0**, `ttk_extrap` true, band 1 of E-8.26. With HP_rem =
100: HP_1 = −50, **T_EV = 60 + 100/50 = 62.0** (killed through the shield). With d_BP = d_HP = 0: **+∞**, band 2.

**TV-08-23 MC status draws and Encumber keying (E-8.16, E-8.17; `02` 4.8, E-2.17).** One attack (a = 0), shot i = 0,
exactly 2 pellets (MS = 2), CC 0, SC 1.4 per pellet (N = 1 + [u < 0.4], draw "sc_count"), weights IMP 0.2, PUN 0.3,
SLA 0.5 (draw "sc_type" per proc, categorical in overview 2.2 order), Secondary Encumber c_enc = 0.24 with π_any = 1;
seed 0. "enc" is drawn on pellet 0, then on pellet 1 if pellet 0 failed (anchors (0, 0, 0) and (0, 0, 1)); "enc_type" on
the anchor of the pellet that succeeded, ENC13 index floor(13·u).
run 0: pellet 0 u 0.1318198436 → 2 procs: 0.5712010750 SLA, 0.0533004239 IMP; pellet 1 u 0.5338028245 → 1 proc:
0.7150411073 SLA; enc 0.5035961784, 0.9697036634: no Encumber proc.
run 4: pellet 0 u 0.7135643146 → 0.8439932872 SLA; pellet 1 u 0.7554267169 → 0.7584019750 SLA; enc 0.8859910998 (fail),
0.1501950664 (pellet 1 succeeds); enc_type 0.2850568968 → index 3, **HEA** on pellet 1.
run 6: pellet 0 u 0.3759630850 → 2 procs: 0.3516695404 PUN, 0.3481029323 PUN; pellet 1 u 0.5029198950 → 0.8872380694 SLA;
enc 0.0428846312 (pellet 0 succeeds; pellet 1 is not tried); enc_type 0.3523313144 → index 4, **COL** on pellet 0.
Runs 0-999: rolled procs SLA **1,405**, PUN **834**, IMP **567**; Encumber procs **436** (expected 1000 × (1 − 0.76²) =
422.4).

**TV-08-24 DoT survival inside one event (E-8.11).** Cap 4; one event with three instances ι = 0, 1, 2 in order, each
η_{s,ι} = 1. No later event: η·R_s per instance **0.7557836979, 0.9292077599, 0.9956512304**, sum **2.6806426883** =
E[min(Poisson(3), 4)]. A later event of mean 1.0 before u: **0.5378900525, 0.7557836979, 0.9292077599**, sum
**2.2228815103** = E[max(0, min(N_e, 4 − N_f))] (N_e ~ Poisson(3), N_f ~ Poisson(1)). Ignoring the order inside the
event gives 3.0 and 2.9430355294.

**TV-08-25 Crit factor with coupled groups (E-8.9 steps 3, 4a).** c_0 = 1, h = 1, ΣCD_rel = 0, CC(n_P) = 0.30 +
0.05·n_PUN with n_PUN capped Poisson (3.0, cap 5); n_COL capped Poisson (1.5, cap 10), cold_cd in ΣCD_abs (Frozen 1.0 at
10); quantization of the base multiplier omitted for the test.
(a) Prelude of Might (`02` E-2.5): CDM0 = 2.0, +3.0 to the base while CC < 0.50: E[c] = **2.2666772997**; the form
1 + E[CC]·(E[CDM] − 1) with E[CC] = 0.4432689722, E[CDM] = 4.0555409708 gives 2.3544265056 (+3.87 %).
(b) Spiteful Defilement (`02` §4.3: +1.0 in ΣCD_abs while n < 3), CDM0 = 2.0, n = presence count of VIR (capped Poisson
1.5, cap 10), SLA (presence 1 − e^{−0.7}), PUN and COL; term E[c·Vir(n_VIR)] = **3.0838353460**; treating 1{n < 3} as an
independent factor (P(n < 3) = 0.2455568254) gives 3.1372958356 (+1.73 %).

**TV-08-26 Steady shared_lose_all (E-8.15b).** Successful triggers at λ = 0.3 /s, D = 12, m = 5, g = 1: p = 1 −
e^{−3.6} = 0.9726762776, E[stacks] = p(1 − p^5)/(1 − p) = **4.6047732160** (simulation of the shared timer, 5 × 2·10^7
s: 4.60442 ± 0.00045). λ = 0.1, D = 10, m = 3: **1.2842774175** (simulation 1.28406 ± 0.00038).

**TV-08-27 `dps_series` with a partial last bin (8.1).** t_win = 2.5, b = 1: K = 3 bins of length 1, 1, 0.5. Damage
100 at t = 0; 100 at t = 1.0; 50 at 2.2; 100 at 2.5 (= t_win); 100 at 2.5000000004 (grid 2,500,000 µs, inside); 100 at
2.5000006 (grid 2,500,001 µs, outside: not in the window). Series **100, 100, 500** (last bin (50 + 100 + 100)/0.5);
`dps_window` = 450/2.5 = **180**.

**TV-08-28 Melee Duplicate in EV (4.2, E-8.10, E-8.12).** One melee hit on a fresh target, health view, neutral, one
damage type, B' = B_ref = 100, A = 0, Adding CO b_co = 0.5 per status (ρ_co = 1), F = 0, G = 1, CC = 0.6 (k ∈ {0, 1}),
CDM = 2.0, SC 0 with one forced Slash proc per hit, Melee Duplicate rank 5 (p_dup = 1.0); direct damage only. Parent:
n = 0, Y = 100, E[c] = 1.6: **160**. Copy: weight w = p_dup·P(k = 1) = 0.6; it counts the parent's forced Slash proc
(n = 1, Y = 150) and rolls its own crit (E[c] = 1.6): 0.6 × 240 = **144**. Expected direct damage **304**; scaling the
parent by 1 + p_dup·P(k = 1) gives 256 (−15.8 %). With Toxic Lash active (p_xh = 0.30 × 2 = 0.60, Toxin): the parent's
extra hit E[XH] = 0.6 × 100 × 1.6 = 96; a tier-1 crit duplicates that extra hit instead of the melee hit (`04` 6.4 bug),
value 0.6 × 100 × 2.0 = 120 (OQ-811), weight 0.6: 72; no melee copy: total **328**.

---

## 18. Review notes

Issues of the two verifier reviews that were not applied as proposed, and why (every other issue was applied after
checking its evidence):

- Steady arrival-time count (G-13): evidence holds, but overview 2.4 and 4.2 fix the steady μ_s = λ_s·D_s, so the
  definition is kept and the bias is recorded as A-10, gap class G-13, TV-08-17 and the proposal OQ-806.
- T-22 default (family as one unit): evidence holds, but T-22's default is an overview decision; section 9 states the
  consequence and OQ-802 stays a proposal to the manager.
- Overguard caps in views: resolved by following `03` E-3.2 (spawn state, restated by `05` §18), not by keeping the
  earlier "overguard gone" rule; the per-view exception stays with the manager (OQ-804, `03` OQ-321).
- c_heat alignment request to `05`: no longer needed; `05` E-5.10 now keeps k_h after the episode, as `03` E-3.8 does.
- D-806 versus D-052: D-806 is retired rather than kept as a Compat row; the register update is asked in OQ-809.
- Toxic Lash TTK double count (D-805): applied with one correction to the evidence: in the shield phase the direct hit
  comes from the shield record and the Toxin-only health record, not the healthFull record (`JS L13735-13741`).
- Test-vector coverage: the minimum set and more were added (TV-08-17 to TV-08-28); still without a vector: the steady
  fixed point with Tau or Hunter Munitions, T-22 on versus off, overguard TTK with overflow and the expected Magnetic
  break, ELE/GAS "+1" groups in EV, Blast in EV, beam merged ticks in the walk, melee steady metrics, E-8.9 step 6.
