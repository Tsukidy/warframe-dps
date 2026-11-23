Status: draft (spec)

# Damage-calculation spec, part 06: ranged firing model

This document defines how a ranged attack turns its stats into a timed sequence of shots, hits, reloads and
time-dependent multipliers. Both evaluators of `00-overview.md` section 4 consume the same deterministic schedule
defined here (overview 4.1: "the schedule never depends on random draws"). It follows the template of overview 0.4.

Related documents: `00-overview.md` (notation, master formulas, register, toggles), `01-damage-composition.md`
(base vector, progenitor, Incarnon flat, kitgun base damage consumer), `02-crit-status-multishot.md` (pellet counts,
beam merge CD-10, body-part rules, Kohm status division), `03-status-effects.md` (DoT, Napalm proc details),
`04-conditional-bonuses-and-buffs.md` (`G`, `G_F` membership, CO ratios of child projectiles, buff catalogue and
trigger model), `05-enemies.md`, `07-melee.md` (melee schedule), `08-evaluation-and-outputs.md` (how the schedule is
walked), `09-build-rules.md` (legality, e.g. Synth Charge magazine requirement, Cannonade trigger requirement).

Contents:

- [1. Scope and stages](#1-scope-and-stages)
- [2. Inputs](#2-inputs)
- [3. Outputs and symbols added](#3-outputs-and-symbols-added)
- [4. Data model: wiki module fields and normalisation](#4-data-model-wiki-module-fields-and-normalisation)
- [5. Modded timing stats](#5-modded-timing-stats)
- [6. The schedule generator](#6-the-schedule-generator)
- [7. Time-dependent multipliers](#7-time-dependent-multipliers)
- [8. Distance: falloff and range](#8-distance-falloff-and-range)
- [9. Parent and child attacks](#9-parent-and-child-attacks)
- [10. Steady-state quantities: EFR, shots per magazine, duty](#10-steady-state-quantities-efr-shots-per-magazine-duty)
- [11. Weapon families](#11-weapon-families)
- [12. Worked examples](#12-worked-examples)
- [13. Edge cases](#13-edge-cases)
- [14. Inspiration differences](#14-inspiration-differences)
- [15. Open questions](#15-open-questions)
- [16. Test vectors](#16-test-vectors)

Citation keys are those of overview 0.2. `F:weapon` = `research/findings/wiki-weapon-mechanics.md`. Wiki pages are
cited as `W:<Page>` with line numbers of the cached raw copy in `research/external/cache/wiki/` where given.
`M:Weapons.lua` and `M:StatObject.lua` are `research/external/cache/wiki-modules/raw/Weapons.lua` and
`.../StatObject.lua` (fetched 2026-11-23). `D:<weapon>` is a record of `Module:Weapons/data/*` as converted in
`research/external/cache/poc/out/` (data, 2026-11).

---

## 1. Scope and stages

**Owns** (overview 7.1, row `06`): the ranged shot-schedule generator used by both evaluators; fire rate (bow ×2,
semi-auto cap, Cannonade lock, floor); burst; charge (including partial charges); beams (ticks, ammo, ramp); spool
(including Kohm pellets); shotguns (pellet timing and falloff only); AoE and falloff (Φ(d), progenitor falloff,
Incarnon-flat exception, radial headshot rule); parent and child attacks; magazine, ammo cost, ammo efficiency,
infinite ammo; reload (standard, per shell, battery, magazine-1, Vectis, Epitaph); first/last-shot schedule; sniper
combo (count, multiplier, decay); kitgun assembly; arch-guns; exalted ranged weapons; companion weapons; Incarnon form
(magazine, switch time); duty ρ per class.

**Not owned**: crit, status and multishot math (`02`; this document only says *when* a shot happens and how many
base pellets a Kohm-family shot has), CO ratios of child projectiles and the bucket of each multiplier (`04`), DoT
math including Nightwatch Napalm tick damage (`03`), melee timing (`07`), how the evaluators walk the schedule (`08`).

**Stages.** No stage of overview 3.1 is owned outright. This document supplies: the falloff factor Φ inside `G_F`
(ST-4); the timing-driven members of `G` (ST-5): beam ramp, sniper combo, first/last-shot flags, Peacemaker ramp;
kitgun base stats to ST-1; the `is_radial` flag that forces `Part = 1` in E-7 (ST-6); and the schedule consumed by
`08`.

---

## 2. Inputs

| Symbol / input | Owner | Use here |
|---|---|---|
| weapon and attack records (section 4) | data pipeline (`docs/data/`) | base timing stats |
| `sum_ms`, `ms`, pellet count rule E-10 | `02` | child counts (9.2), sniper combo count (7.2), Munitions Grit cost |
| `progenitor_frac`, `base_listed[t]`, `incarnon_flat`, `str_base_mult` | `01` | Φ_eff weighting (8.3) |
| buff catalogue: fire-rate, reload, magazine, ammo-efficiency, charge, radius, projectile-speed bonuses and their condition class | `04` | sums of 5.1-5.7 |
| conditions toggle T-02, distance T-27, sniper combo T-12, fight window T-01, TTK horizon `t_ttk_max` | overview 6, 4 | schedule horizon, falloff, sniper combo |

---

## 3. Outputs and symbols added

Overview symbols used with their overview meaning: `fire_rate` FR, `eff_fire_rate` EFR, `charge_time` CT,
`burst_count` BC, `burst_delay` BD, `magazine` Mag, `ammo_cost` cost, `ammo_eff` AE, `shots_per_mag` N_mag, `t_mag`
T_mag, `t_reload` T_rel, `duty` ρ, `distance` d, `falloff(d)` Φ(d), `sniper_combo_mult` M_snp.

**Symbols added** (overview 0.3 requires a domain prefix):

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `fire_rate_base` | FR0 | 1/s | data `FireRate` of the attack (∞ for `math.huge`) |
| `fire_sum` | ΣFR | bonus | fire-rate bucket after the bow weighting (E-6.1) |
| `fire_mult` | ΠM_fr | factor | product of multiplicative fire-rate factors from the `04` catalogue (Peacemaker ramp, 7.5); the spool curve is applied separately (E-6.17) |
| `fire_semi_cap` | FR_cap | 1/s | semi-auto cap, 10 unless overridden |
| `fire_locked` | lock | bool | a Cannonade mod is installed |
| `charge_time_base`, `burst_delay_base` | CT0, BD0 | s | data `ChargeTime`, `BurstDelay` |
| `mag_base`, `mag_sum`, `mag_flat` | Mag0, ΣMag, Mag+ | rounds, bonus, rounds | data `Magazine`, percent bucket, flat additions |
| `ammo_cost_base`, `ammo_cost_eff` | cost0, c | rounds | data `AmmoCost` (default 1), cost per shot after AE |
| `ammo` | a | rounds | live magazine content (may be negative: ammo debt) |
| `reload_base`, `reload_sum` | R0, RS | s, bonus | data `Reload` (attack field first), reload-speed bucket |
| `reload_style` | – | enum | `magazine`, `by_round`, `regenerate` (data `ReloadStyle`, absent = `magazine`) |
| `reload_rech_delay`, `reload_rech_rate` | δ_rech, ρ_rech | s, rounds/s | battery delay (data `ReloadDelayEmpty`, else `ReloadDelay`) and `ReloadRate` |
| `reload_shell_t`, `reload_startend_t` | t_sh, t_se | s | per-shell reload split (hand-authored) |
| `reload_lead` | δ_rel | s | time from the last shot of a magazine to the start of its reload (E-6.10) |
| `t_cycle` | T_cyc | s | magazine cycle: T_mag + T_rel |
| `spool_shots`, `spool_start`, `spool_step` | S_sp, s_sp, Δ_sp | count, fraction, fraction | auto-spool cadence |
| `kohm_pellets` | n_k | count | innate pellets of the k-th shot of a Kohm-family attack |
| `ramp_start`, `ramp_level` | s_rmp, L | fraction | beam ramp start and current level |
| `snp_min`, `snp_reset`, `snp_count` | C_min, D_snp, c_snp | count, s, count | sniper combo minimum, reset time, counter |
| `pm_burst` | b_pm | count | Peacemaker burst index |
| `falloff_start`, `falloff_end`, `falloff_red` | d_s, d_e, r_f | m, m, fraction | data `Falloff` |
| `falloff_prog`, `falloff_eff` | Φ_v, Φ_eff | factor | progenitor falloff, effective falloff factor in `G_F` |
| `aoe_dist` | d_aoe | m | distance of the target from an AoE epicentre (0 for the directly hit target) |
| `inc_charges`, `inc_switch_t` | Mag_inc, T_sw | rounds, s | Incarnon form magazine (`IncarnonCharges`) and switch time |
| `is_radial` | – | bool | damage instance of a radial (AoE) attack: `Part = 1`, no head/weak-point triggers |
| `ramp_eff`, `first_mean`, `snp_mean` | η_rmp, η_1st, η_snp | factor | steady-state means of time-dependent multipliers (10.4) |

Outputs: the schedule (6.1), the per-hit time-dependent factors of section 7, Φ_eff (8.3), EFR, N_mag, T_cyc, ρ and the
steady means of section 10, kitgun assembled stats (11.1).

---

## 4. Data model: wiki module fields and normalisation

### 4.1 Field map

The data source is `Module:Weapons/data/<slot>` (`DS:wiki` §3; counts there). An attack inherits every field it does
not define from its weapon: the wiki builds the attack view with `getStruct2(weapon, attack)`, whose `__index` returns
`attack[i] or weapon[i]` (`M:StatObject.lua` L367-371; used by `M:Weapons.lua` L67-74). The engine MUST use the same
rule (attack field first).

| Wiki field (level) | Engine symbol / rule | Section |
|---|---|---|
| `Trigger` (weapon, attack) | normalised trigger class (4.2) | 4.2, 6 |
| `FireRate` (attack) | FR0; `math.huge` (string `"Infinity"` after conversion) = ∞ (Lanka, `DS:wiki` §3) | 5.1 |
| `ChargeTime` (attack) | CT0; absent = 0 (`M:Weapons.lua` default L155) | 5.3 |
| `BurstCount`, `BurstDelay` (attack) | BC (absent = 1, `M:Weapons.lua` L476), BD0 (absent = 0) | 5.4, 6.4 |
| `BurstReloadDelay` (attack) | reload lead after a magazine burst (UNVERIFIED meaning, OQ-606) | 5.8 |
| `AmmoCost` (attack; typo `Ammocost`) | cost0; absent = 1 for firing attacks (`M:Weapons.lua` L498-500 comment); children ignore it | 5.6 |
| `Magazine`, `Reload` (weapon; `Reload` also attack, 18 attacks) | Mag0, R0 (attack value first) | 5.5, 5.7 |
| `ReloadStyle`, `ReloadDelay`, `ReloadDelayEmpty`, `ReloadRate` (weapon; 1 attack each) | reload style, δ_rech, ρ_rech; non-battery `ReloadDelay = 0` = no reload delay | 5.7, 5.8 |
| `Spool` (weapon) | S_sp | 7.3 |
| `SniperComboMin`, `SniperComboReset` (weapon) | C_min, D_snp | 7.2 |
| `Multishot` (attack; absent = 1) | MS0 (`02`); Kohm-family n_k (7.4) | 7.4, 9.2 |
| `ShotType` (`Hit-Scan`, `Projectile`, `AoE`, `Thrown`, `DoT`; typo `Hitscan`) | radial flag, range limit, flight time | 8, 9 |
| `AttackIndex`, `AttackParentIndex` | parent/child links (9.1) | 9 |
| `Falloff = {StartRange, EndRange, Reduction}` | d_s, d_e, r_f; on AoE attacks the radial falloff (`DS:wiki` §3) | 8 |
| `Range` | hitscan or beam limit, AoE radius (`M:Weapons.lua` L360-363: hitscan default 300 m) | 8.4 |
| `ShotSpeed` | projectile speed for flight time (8.5) | 8.5 |
| `ExplosionDelay`, `EmbedDelay`, `EffectDuration` | child delay and lingering duration (9.3, 9.4) | 9 |
| `IncarnonCharges` | Mag_inc | 6.6 |
| `Class`, `Slot`, `CompatibilityTags` (`BEAM`, `BATTERY`, `POWER_WEAPON`, `SENTINEL_WEAPON`), `Traits`, `AmmoMax` (∞ on robotic) | class rules (bow, exalted, companion, archgun) | 11 |
| `Zoom` (16 weapons) | zoom levels; their stat changes are catalogued in `04` (crit) or 5.1 (fire rate) | 5.1 |
| `PunchThrough`, `Accuracy`, `MinSpread`, `MaxSpread`, `IsSilent` | not used (single target; all pellets hit) | 13 |
| `Modular/data` `KitgunPrimary`/`KitgunSecondary` (`Chamber`, `Grip`, `Loader`) | kitgun assembly (11.1) | 11.1 |

`Modular/data/kitgun` pre-expanded combos MUST NOT be used: they lack fields the parts carry (for example
`Tombfinger-Splat-Tremor` has no `ChargeTime` although the chamber has `ChargeTime[Tremor] = 1.4`; checked on the
cached JSON), and they predate the last parts edit (`DS:wiki` §3).

### 4.2 Trigger normalisation

The trigger of an attack is `attack.Trigger` if present, else `weapon.Trigger` (4.1). The data holds 33 distinct
weapon strings, many compound (`"Auto / Charge"`, `"Semi-Auto / Mag Burst"`) (`DS:wiki` §3). Normalisation:

| Data string | Class | Schedule rule |
|---|---|---|
| `Auto` | AUTO | constant rate (6.4) |
| `Semi-Auto`, `Semi` | SEMI | constant rate with the semi cap (5.2) |
| `Duplex` | SEMI | per-shot fire rate, semi cap (OQ-012 decision: `W:Damage/Calculation` L476 "assume the player is able to achieve the modded Fire Rate") |
| `Burst` | BURST | burst (6.4) |
| `Auto-Burst` | BURST | identical to BURST under continuous fire (`W:Trigger_Type` L17) |
| `Mag Burst` | MAG_BURST | BC = shots the magazine can pay (`W:Damage/Calculation` L499; `W:Trigger_Type` L19) |
| `Held` | HELD | beam ticks (6.4, 7.1) |
| `Charge` | CHARGE | charge (6.4) |
| `Auto-Charge`, `Charged Auto` | CHARGE | tap = full charge; CT from data (0 if absent) (`W:Trigger_Type` L55) |
| `Auto-Spool` | SPOOL | spool (7.3) |
| `Active` | SEMI | the projectile is assumed detonated on impact (Decision: no source gives a detonation delay; `W:Trigger_Type` L21) |
| `DoT` | – | lingering child (9.4) |

Compound strings without an attack override are resolved by: attack has `ChargeTime` → CHARGE; attack has
`BurstCount > 1` or `BurstDelay` → BURST, or MAG_BURST if the weapon string contains `Mag Burst` and `BurstCount` is
absent; otherwise the first component for the attack with the lowest `AttackIndex` among firing attacks and the second
component for the next one. Every compound resolution MUST be written to the data-build report and MAY be overridden by
a hand-authored table (Decision: the wiki has no field linking trigger components to attacks).

### 4.3 Hand-authored tables the data pipeline must supply

Fields the engine needs that no module carries (`DS:wiki` §10, `F:weapon` §4.3, §6, §9.2):

| Table | Content | Source |
|---|---|---|
| spool cadence | `s_sp`, `Δ_sp` per Auto-Spool weapon | weapon pages, table in 7.3 |
| Kohm family | `P_max` (12 or 5), cost rule, pellet spool length | `W:Kohm`, `W:Kohmak` (7.4) |
| beam ramp start | s_rmp per weapon (default 0.2) | `W:Continuous_Weapon` L123-129 (7.1) |
| per-shell reload | t_se, t_sh for `ByRound` weapons | weapon pages (5.7) |
| semi-cap overrides | Komorex ≈ 8 | `W:Komorex` (5.2) |
| bow doubling | effect lines whose card says "(x2 for Bows)" | `Module:Mods/data` descriptions (5.1) |
| fixed charge time | Phantasma | `W:Fire_Rate` (5.3) |
| no-reload weapons | Epitaph, Epitaph Prime | `W:Damage/Calculation` L546 (6.4) |
| exalted ammo mode | infinite or magazine (11.3) | `W:Exalted_Weapon` L8 |
| parent/child overrides, trigger overrides | 4.2, 9.1 | data-build report |
| reload-triggered attacks | Catabolyst family | 9.5 |
| whole-magazine attacks | Tenet Tetra alt, Sepulcrum alt, Tenet Plinx alt | 6.4 |
| sniper combo exclusions | Arca Scisco, Knell (reset only, no minimum) | 7.2 |

---

## 5. Modded timing stats

All bonuses are summed over every source of their bucket (mods, arcanes, buffs, rivens, Incarnon perks, custom stat
T-26) as catalogued by `04`. Conditional sources follow T-02, with the timing rule of 5.9.

### 5.1 Fire rate

```text
E-6.1  Fire-rate bucket
       ΣFR = Σ_sources w_src · b_src
       w_src = 2  if the weapon is a bow (Class = "Bow") and the source line carries "(x2 for Bows)"
                  (fire-rate mod cards, riven "Fire Rate (x2 bows)")
       w_src = 1  otherwise (arcanes, abilities, every non-bow weapon)
       lock (Cannonade installed): ΣFR := 0 and ΠM_fr := 1 for every attack of the weapon
E-6.2  Modded fire rate
       FR = max(0.05, FR0 · (1 + ΣFR) · ΠM_fr)          (FR0 = ∞ gives FR = ∞)
```

Sources: linear scaling `W:Fire_Rate` L5-6, `W:Damage/Calculation` §Modded Stats (`F:weapon` §1.1); floor 0.05
`W:Fire_Rate` L334; bow ×2 `W:Speed_Trigger` (Bow column, "660 % for Bows"), `Module:Mods/data` description lines
5968, 10731 and others (`F:weapon` §1.5); Arcane Acceleration "Does not double its bonus for bows"
(`W:Arcane_Acceleration` L39); lock `W:Fire_Rate` L329 ("preventing any changes ... positive or negative"),
`W:Semi-Rifle_Cannonade` §Notes (U36). Which weapons count as bows: Class `Bow` only; crossbows "behave more similarly
to rifles" (`W:Bow` L3) and get w = 1 (UNVERIFIED, OQ-613). Additivity of non-mod sources with mods: UNVERIFIED per
source (`F:weapon` §1.1); Decision: additive, except factors the `04` catalogue marks multiplicative (the Peacemaker
ramp is "multiplied against other modifiers to fire rate", `W:Peacemaker` L31-32).

### 5.2 Semi-auto cap

```text
E-6.3  FR_trig = min(FR, FR_cap)   for trigger class SEMI (incl. Duplex and Active)
       FR_trig = FR                for every other class
       FR_cap  = 10 (default), per-weapon override (Komorex 8.0)
```

Source: `W:Fire_Rate` L8 ("Semi-Auto weapons are capped at 10 rounds per second"); `W:Komorex` §Notes (≈ 8.0, measured
at 60 FPS). Value UNVERIFIED (overview OQ-012). Decision: the cap applies only to SEMI-class attacks; BURST (bursts
per second) and CHARGE attacks are not capped, since no source states a cap for them. The accessibility auto-fire
option does not change the rate (`W:Trigger_Type` L26).

### 5.3 Charge time

```text
E-6.4  CT = CT0 / max(0.1, 1 + ΣFR)        (ceiling 10 · CT0; lock gives CT = CT0)
       fixed-CT exceptions (Phantasma "not affected by mods"): CT = CT0
```

Source: `W:Fire_Rate` L21-29 (charge time = base / (1 + bonus); "cannot go above 10 times the base charge time (achieved
by having at least −90 % fire rate bonus)"), Phantasma `W:Fire_Rate` charge-time list. Bows use the bow-weighted ΣFR
of E-6.1 (`F:weapon` §1.5 worked example: Paris 0.5 s with Speed Trigger → 0.5 / 2.2 = 0.227 s). ΠM_fr does not
enter CT (Decision: no source; the only multiplicative factors catalogued act on automatic weapons).

### 5.4 Burst delay

```text
E-6.5  BD = BD0 / (1 + max(0, ΣFR))      (lock gives BD = BD0)
```

Source: `W:Fire_Rate` §Mechanics (fire rate "affects both the speed of the burst as well as the time between bursts")
and §Burst L195 ("Burst Delay is not affected by net negative Fire Rate bonuses"); the displayed formula uses the
unmodded delay, so the division is UNVERIFIED numerically (overview D-080, OQ-014).

### 5.5 Magazine

```text
E-6.6  Mag = max(1, floor(Mag0 · (1 + ΣMag) + 0.5 + 1e-9) + Mag+)
       Bows (Class Bow) and Epitaph: Mag = 1, magazine bonuses ignored
       Incarnon form attacks: Mag = Mag_inc (data IncarnonCharges), bonuses ignored
       Kitguns: Mag0 = chamber.Magazine[loader.Magazine]   (11.1)
```

Sources: rounding "to the nearest whole number" (`W:Magazine_Warp`, `W:Slip_Magazine`, `W:Tainted_Mag`,
`W:Burdened_Magazine`, `W:Wildfire`, `W:Depleted_Reload` notes) against "rounded up" (`W:Ammo_Stock` L45-46); every
example satisfies both, so nearest-half-up is chosen (majority, and it agrees with the inspiration's `Math.round`) and
marked UNVERIFIED (OQ-601). Minimum 1: `W:Depleted_Reload` L29. Percent first, flat after: `W:Calculating_Bonuses`
L76-77 (Viper × (1 + Slip Magazine) + Stinging Truth). Incarnon "Increase Base Magazine Capacity by +X" perks are
treated as Mag+ (after the percent product) (Decision, UNVERIFIED: the word "Base" suggests before; OQ-602). Bows and
Epitaph: `W:Ammo` L23, `W:Bow` L12 and U38.5 fix. Incarnon form: `W:Lex_Incarnon_Genesis` L21, `W:Braton_Incarnon_Genesis`
L22. The 1e-9 guards f64 products such as 45 × 1.3 = 58.49999… (Decision).

### 5.6 Ammo cost, ammo efficiency, infinite ammo

```text
E-6.7  AE = min(0.75, ΣAE)                    (no lower bound; Incarnon form: AE = 0)
       c  = cost0 · (1 − AE)                  (per shot; per tick for beams)
       infinite-ammo state active (weapon Magazine = 0, AmmoCost = 0, or an infinite-ammo buff): c = 0
E-6.8  can_fire(a, c) = (c = 0)  or  (a > 1e-9  and  ceil(a − 1e-9) ≥ c − 1e-9)
       after a shot: a := a − c   (a may become negative: ammo debt)
```

Sources: AE formula, debt, HUD ceiling and the "between 0 and 1 rounds up and allows another shot" rule `W:Ammo`
L201-219 (Vectis +25 % worked example, reproduced in TV-06-05); cap 0.75 since Hotfix 44.1.3 (`W:Ammo` L222,
`F:mods` §11 patch table); "Ammo efficiency and infinite ammo buffs do not affect ... transformed Incarnon weapons"
(`W:Ammo` L222); infinite-ammo sources list `W:Ammo` L222-244, Arcane Pistoleer since 44.1.3 (`F:addmul` §5). Beams
cost the data `AmmoCost` per tick (0.5 on every held attack checked, `F:weapon` §4.1); the old "unless they are
Flamethrowers" note (U22.13.3, `W:Continuous_Weapon` L145) is contradicted by current data (Ignis 0.5) and ignored.

The ceiling form of E-6.8 for `c > 1` is UNVERIFIED (OQ-605). It is chosen because it reproduces both the debt rule for
`c ≤ 1` (`W:Ammo` L208) and the Kohmak note "25 shots, with one round left" with `c = 1.67` (TV-06-09): a plain
`a > 0` rule would allow a 26th shot. Kohm: "If it ... doesn't have enough ammo to fire the next shot, it will attempt
to reload even if the magazine still has ammo in it" (`W:Kohm` L29), consistent with E-6.8.

`AmmoCost` of a BURST attack is per shot (Burston: `AmmoCost = 1`, `Magazine = 45`, 15 bursts per magazine,
`F:weapon` §2). For Bubonico "Burst" (`BurstCount = 3`, `AmmoCost = 3`) the per-shot reading gives 3 bursts per 27
rounds; per-burst would give 9 (UNVERIFIED, OQ-605). `M:Weapons.lua` `CalcDamage` (L541-591) treats `ShotsPerMag` as
inputs per magazine and multiplies by `BurstCount`, which overcounts Burston by ×3; it is not used as a reference
(`F:weapon` §9.4).

### 5.7 Reload time

```text
E-6.9  magazine style:   T_rel = R0 / (1 + RS)
       by_round style:   T_rel = (t_se + t_sh · n_load) / (1 + RS)
                         n_load = Mag − max(0, floor(a_left + 1e-9))
       regenerate style: T_rel = δ_rech' + n_regen / ρ_rech
                         δ_rech' = max(min(δ0, 0.5), δ0 / (1 + RS))    (Shedu: δ_rech' = δ0)
                         δ0 = ReloadDelayEmpty if a_left ≤ 0, else ReloadDelay   (fallback: the other one)
                         n_regen = Mag − max(0, a_left)
       guard: 1 + RS is clamped to ≥ 0.05 (Decision; no source covers RS ≤ −0.95)
```

Sources: `W:Reload` L29-35 (Braton 2.0 / 1.3 = 1.54 s; Tainted Mag 2.0 / 0.7), per shell L45-49 (Strun 0.75 + 0.5 × 6 =
3.75 s; Corinth 1.2 + 0.22 × 5 = 2.3 s, `F:weapon` §9.2), battery L51-58 and `W:Battery_Weapon` L1-6 ("Reload Speed
bonuses only affect the recharge delay", "cannot go below 0.5 s"), `ReloadDelayEmpty` fallback `M:Weapons.lua` L378.
The floor is read as "reload-speed bonuses cannot push the delay below 0.5 s; a base below 0.5 s stays" (Decision,
UNVERIFIED: the empty-magazine delays of Imperator Vandal (0.25 s) and Phaedra (0.25 s) would otherwise be raised
unmodded; OQ-608).
Shedu: U30.7 "Fixed Reload Speed Mods affecting the Shedu's ammo regeneration delay" (`W:Battery_Weapon` L68),
contradicting the general rule; kept as a per-weapon exception (UNVERIFIED, OQ-608). Per-shell split: only the data
total `Reload` exists; when the split is unknown, t_se = 0 and t_sh = R0 / Mag0 (Decision, UNVERIFIED). The engine
reloads a full magazine every time (reload interruption is out of scope, `W:Reload` L45).

### 5.8 Reload lead (delay between the last shot and the reload)

```text
E-6.10 δ_rel = 0                     no-delay weapons: Mag = 1 (modded), or data ReloadDelay = 0 on a
                                     non-battery weapon (Vectis, Vectis Prime, Exergis), or Class Bow
       δ_rel = BurstReloadDelay      after a MAG_BURST that emptied the magazine, when the field exists (UNVERIFIED)
       δ_rel = 1 / r_last            otherwise: the reload starts when the next shot's cycle would have begun
                                     (r_last = rate that governs the interval after the last shot, 6.4)
```

Sources: "a short Reload Delay equal to the reciprocal of the weapon's Fire Rate ... cannot be avoided on Battery
Weapons" (`W:Reload` L6); "Weapons that have a Magazine Size of 1 will not have Reload Delay" (`W:Reload` L13, gallery:
bows, Exergis, Epitaph, Vectis); Vectis and Vectis Prime "reload instantly after the last shot" (`W:Fire_Rate` L17;
`W:Damage/Calculation` L545 "one (1) should be subtracted from the denominator"); data `ReloadDelay = 0` on exactly
Exergis, Vectis, Vectis Prime among non-battery weapons (checked on the cached data). `W:Damage/Calculation` applies the
"−1" only to Vectis; the decision "no delay for every magazine-1 weapon" is overview OQ-015's (UNVERIFIED, Tonkor test).
`BurstReloadDelay` exists on 7 attacks (Nagantaka 0.2, Cyanex 0.21, Pandero 0.24, Tenet Diplos 0.1, ...) and is unused
by `M:Weapons.lua` (`F:weapon` §2); its reading here is a Decision (OQ-606).

### 5.9 Timing stats under conditional buffs

Timing stats change the schedule, and overview 4.1 requires a schedule free of random draws. Rule (Decision):

- `all_max` and `none` (T-02): the buff is on (at max stacks) or off for the whole schedule.
- `simulated`: buffs with a deterministic own-event trigger of this attack (on reload, on reload from empty, first
  shot, aiming) are switched on and off by the schedule's own events with their catalogued duration. Buffs with a
  random trigger (on crit, on proc, on headshot kill: Arcane Acceleration, Arcane Velocity, Arcane Tempo, Arcane
  Momentum, Shotgun Vendetta) and external triggers use the class-(c) user uptime input of overview 6.2 as a constant
  fraction of their max value. This keeps the schedule identical in both evaluators.

---

## 6. The schedule generator

### 6.1 Output

For one **firing attack** (an attack that is not a child, 9.1) and a horizon `H = max(t_win, t_ttk_max)` (overview
4.1, 4.2), the generator emits, in time order:

| Record | Fields |
|---|---|
| shot | index i; cycle start τ_i (charge start or fire time); fire time s_i; hit time h_i (8.5); magazine number and index in magazine; ammo before and after; cost c_i; flags `first`, `last` (6.5), `after_reload`; burst number and position; spool index k and rate r_k; Kohm base pellets n_k; Peacemaker burst b_pm; ramp level L_i (7.1) |
| reload | start, end, `from_empty` (a_left ≤ 1e-9 or can_fire false), ammo before |
| sniper decay | time of each −1 decay (7.2), computed from hit times |
| child hits | per shot and child attack: count rule and hit times (9) |

The schedule is deterministic (no random draw is used) and identical for EV and MC (overview 4.1). Quantities that
depend on random pellet counts (sniper combo count, Munitions Grit) are handled in 7.2 and 13.

### 6.2 Algorithm

```text
E-6.11 Schedule generator (all times in s, f64, compared with ε = 1e-9)
  state: τ = 0 (next cycle start), a = Mag (Mag_inc for Incarnon form, ∞ for infinite), k = 0 (spool/Kohm index),
         j = 0 (position inside the current burst), mag_no = 1
  loop:
    c   = cost of the next shot (E-6.7; Kohm: E-6.18)
    s   = τ + CT                      if class CHARGE (CT = 0 when absent)
        = τ                           otherwise
    if s > H + ε: stop
    emit shot(s);  a := a − c
    r   = rate after this shot: FR_trig (constant classes), spool rate r_k (E-6.17), Peacemaker rate (E-6.19)
    k := k + 1
    next cycle start τ' =
        s + BD                        if class BURST/MAG_BURST and j + 1 < BC_current   (j := j + 1)
        s + 1/r                       otherwise (j := 0); 1/r = 0 when r = ∞
    if can_fire(a, cost of the shot after) (E-6.8):  τ := τ'; continue
    -- magazine end
    mark the shot `last`
    reload_start = s + δ_rel          (E-6.10; δ_rel = τ' − s in the "1/r_last" case)
    if reload_start > H + ε: stop
    reload_end = reload_start + T_rel (E-6.9, evaluated with a_left = a)
    emit reload(reload_start, reload_end)
    a := Mag + min(0, a)              (debt carried; positive leftovers are refilled to Mag)
    k := 0; j := 0; mag_no += 1; τ := reload_end      (spool and Kohm pellets reset: 7.3, 7.4)
  BC_current = BC for BURST; for MAG_BURST the number of shots the magazine can pay when the burst starts
```

The shot that ends a magazine is the shot after which `can_fire` is false. The reload of a magazine that is not empty
(Kohm with too little ammo) uses the non-empty variants of E-6.9. Shots fired after the horizon are not generated; a
reload whose start is past the horizon is not emitted (Decision; the inspiration stops a reload that cannot finish
inside the window, `I:agg` §2.3, which only matters for display).

### 6.3 Window boundary

A shot belongs to the window when its fire time satisfies `s_i ≤ t_win + ε` (closed interval, overview 4.1 "landed in
[0, t_win]"). Damage counts toward `dps_window` when its hit time (8.5, 9.3) is ≤ `t_win + ε`. The first shot of a
non-charge attack is at t = 0; of a charge attack at t = CT (fresh state, overview 4.1).

### 6.4 Rules per trigger class

```text
E-6.12 per-class timing (cycle start τ, fire time s)
  AUTO, SEMI (incl. Duplex, Active), HELD:
       s_i = τ_i ;  τ_{i+1} = s_i + 1/FR_trig
  BURST, MAG_BURST (incl. Auto-Burst, instant bursts BD = 0):
       inside a burst  s_{m+1} = s_m + BD ;  after the burst's last shot  τ = s + 1/FR
       EFR_burst = BC / (1/FR + (BC − 1)·BD)
       a magazine that is not a multiple of BC ends with a partial burst of the remaining shots
  CHARGE (incl. Auto-Charge, Charged Auto, bows, Lanka):
       s_i = τ_i + CT ;  τ_{i+1} = s_i + 1/FR      (Lanka: FR = ∞, so τ_{i+1} = s_i)
       every shot pays its charge, including the first shot after a reload
  SPOOL: as AUTO with the rate r_k of E-6.17 (7.3)
  Epitaph (no-reload list): Mag = ∞ for scheduling, shots as SEMI (uncharged) or CHARGE (charged)
  Whole-magazine attacks (cost0 ≥ Mag0, or listed): c = a (the shot spends the whole current magazine), one shot
       per magazine, then reload (E-6.10)
  Reload-triggered attacks: see 9.5
```

Sources: burst formula and "no delay after last shot in burst" `W:Fire_Rate` L170-195, `M:Weapons.lua` L215-218,
`W:Damage/Calculation` L494-499; instant bursts are separate shots (`W:Trigger_Type` list; `F:status` edge 9 "boost later
shots of an Instant Burst", CD-04); partial last burst `W:Burston_Prime` notes via `F:weapon` §2; charge EFR
`W:Damage/Calculation` L483-488 and `W:Fire_Rate` L31-52 (three cases: charge weapons, bows, Lanka); Lanka `FireRate =
math.huge` (`DS:wiki` §3); Epitaph `W:Damage/Calculation` L546 (EFR UNVERIFIED, `F:weapon` §5.3, OQ-614); Tenet Tetra
"Total damage is now based on ammo in mag on fire" (U32, `F:weapon` §8.2). Bows need no special timing rule: with
Mag = 1 and δ_rel = 0, E-6.11 gives the cycle `CT + T_rel` of `W:Fire_Rate` L39-45 (TV-06-04).

Partial charges: the data models quick shots as separate attacks with their own `ChargeTime` and damage (Lanka
"Partially Charged Shot" CT 0.33, Opticor "Quick Shot", Paris "Uncharged Shot") (`F:weapon` §3.3). The engine evaluates
each as its own attack; it does not interpolate damage between charge levels (Decision, UNVERIFIED whether damage is
continuous; `F:weapon` open question 12, here OQ-607). The release threshold of partial-quickshot weapons (Lanka
33 %, Opticor 40 %) is then irrelevant because the data attack already fixes CT. Bow charge scaling is a damage factor
in `G_F` (overview 3.1a); its value is in the data attack damage (charged vs uncharged), so the factor is 1 for every
data attack (Decision).

### 6.5 First-shot and last-shot flags

```text
E-6.13 first(i) = (HUD counter after shot i) = Mag − 1,   HUD counter = max(0, ceil(a_after − 1e-9))
       last(i)  = shot i ends its magazine (E-6.11)
       never set: infinite-ammo states (counter does not move), Incarnon form attacks, robotic (companion) weapons for
       Primed/Charged Chamber; `last` never set on HELD attacks and on whole-magazine attacks
       reload-triggered attacks (9.5) carry `last` = true (Synth Charge applies to the Catabolyst reload grenade)
```

Sources: Primed Chamber "as long as the magazine counter is at Max Magazine − 1 after a shot is fired" (Vulkar with
Energized Munitions: shots 4-7) and "doesn't apply if the mag remains completely full after the shot" (`W:Primed_Chamber`
L39-41); Charged Chamber same, not on sentinel weapons (`W:Charged_Chamber`, `W:Sentinel`); Primed Chamber "Does not work
on the Vulklok" (`W:Primed_Chamber` L55); Synth Charge final shot, no effect on continuous weapons except the Catabolyst
reload grenade, none on Incarnon forms, none on whole-magazine alt-fires (`W:Synth_Charge` L29-60); Vectis Incarnon
fix "benefitting from Primed Chamber on every shot" (`W:Incarnon` L527): Decision no first-shot flag in any Incarnon
form (UNVERIFIED, OQ-612). Values of the bonuses and their bucket are `04`'s; overview 6.2 makes first/last flags follow
the schedule in every T-02 mode.

### 6.6 Incarnon form

```text
E-6.14 Incarnon form attack (data IncarnonCharges > 0):
       Mag = Mag_inc (no magazine bonus, no AE, no infinite ammo), c = cost0 (data AmmoCost, default 1)
       window start: in form with a full form magazine, at t = 0 (switch animation not counted)
       when can_fire is false: the form ends; no reload; the attack's schedule ends
       T_sw = T_rel of the weapon's normal form (E-6.9), used only by OQ-619's proposed option
```

Sources: `W:Braton_Incarnon_Genesis` L18-23 and `W:Lex_Incarnon_Genesis` L18-22 ("Mode switching ... has an animation
equal to the weapon's reload speed", "uses a separate magazine", "not affected by mods", "not affected by Ammo
Efficiency", "Once all ammunition is expended, the Lex reverts to its normal form"); `W:Damage/Calculation` L522-527
("use Max Incarnon Charge. Ammo Cost is 1 per shot"); `W:Ammo` L222; `W:Incarnon` L184 (gauge refilled by weak-point
hits, manual exit wastes charges, exiting refills the normal magazine). Charge gain is out of scope (`F:weapon` §13).
Onos "Incarnon Form Charge Attack" costs 50 form rounds (`D:Onos`), so costs other than 1 are taken from data.
Fresh state "full magazine (or Incarnon charge magazine)" is overview 4.1. Discrepancy D-601.

---

## 7. Time-dependent multipliers

These factors are members of `G` (beam ramp, sniper combo, first/last shot, Peacemaker ramp) or of the fire rate; `04`
owns their bucket placement, this section their values over time.

### 7.1 Beam damage ramp (HELD attacks)

```text
E-6.15 Ramp level L (one per target), start L = s_rmp on a fresh target
       hitting interval of tick i: [h_i, h_i + 1/FR]        (ticks therefore ramp continuously while firing)
       while hitting: L increases at rate (1 − s_rmp)/0.6 per s, capped at 1
       idle time g between the end of one hitting interval and the next tick:
           L := max(s_rmp, L − (1 − s_rmp) · max(0, g − 0.8) / 2.0)
       tick i is multiplied by L at h_i
       from a fresh start: L_k = min(1, s_rmp + (1 − s_rmp) · k / (0.6 · FR)), k = tick index since start
       s_rmp: 0.2 default; Convectrix 0.6 (alt 0.8); Phage 0.7; Embolist 0.3
```

Sources: `W:Continuous_Weapon` L19-20 ("ramps up to 100 % of its damage over 0.6 seconds of hitting a target. 0.8
seconds after the weapon stops hitting a target, the damage decays back to its initial point over 2 seconds. For most
weapons ... 20 %"), U22.14 values L122-129, per target since U22.13.4 (L137). Linear shape, the hitting-interval
convention and the current per-weapon starts are UNVERIFIED (overview D-078, OQ-013); the data modules carry no ramp
field (overview 1.5). The ramp scales the additive CO term too (`F:addmul` §1.6: "beam ramp-up is included"), which
follows from its membership in `G` (overview 3.3). Decision: the ramp also multiplies the AoE child of a HELD attack
(no source; OQ-616). Attacks with ramp: trigger class HELD (Basmu alt-fire "requires a damage ramp-up", `F:weapon`
§4.3; Incarnon held forms such as Furis, Torid, Boar, Onos). Tenet Flux Rifle is `Auto` in data (no ramp, `D:Tenet Flux
Rifle`).

### 7.2 Sniper combo

```text
E-6.16 Applies to attacks of weapons with data SniperComboMin (C_min) (15 weapons); D_snp = SniperComboReset (2 s; Lanka 6 s)
       counter c_snp starts at 0 (fresh state)
       before each shot i (hit time h_i): apply decays: for every m ≥ 1 with h_prev + m·D_snp ≤ h_i: c := max(0, c − 1)
           (h_prev = hit time of the previous counted shot)
       then c := c + p_i       (p_i = projectiles of shot i, all of which hit; radial children and DoT do not count)
       M_snp(c) = 1                     if c < C_min
                = 1.5 + 0.5 · m*        otherwise, m* = max{ m ≥ 0 : C_min · 3^m ≤ c }   (integer arithmetic)
       every damage instance of shot i (all pellets, its children, its DoT seeds) uses M_snp(c after shot i)
       T-12 set: M_snp := T-12 value for every shot
```

Sources: `W:Sniper_Rifle` L14-41 (milestones `Min × 3^(2M − 3)`, `M = 1.5 + 0.5·floor(log3(count/Min))`, each multishot
projectile counts, AoE and DoT do not count, scoped requirement, −1 per miss and −1 when the duration runs out, 2 s and
Lanka 6 s), U22 changes L183-186 ("1 combo count removed at a time when duration runs out"), Lanka "immediate 1.5×
total damage bonus on the first hit if ... a 2nd shot ... lands" (`W:Lanka` L46: the counter includes the shot's own
projectiles), "combo stack for every bullet in your shot regardless of whether that bullet causes damage" (`W:Lanka`
L143-144), max count 2^32 − 1 (`W:Sniper_Rifle` L173, irrelevant), DoT inherits the combo (`F:status` §3.1, `F:addmul` §2, `W:Damage_over_Time`
via overview E-13 seed containing G). Harkonar Scope adds to D_snp (`04` catalogue). Decisions, UNVERIFIED (OQ-607):
the decay repeats every D_snp of idle time; the multiplier also applies to the shot's radial children; the player is
always scoped; Incarnon form attacks inherit the weapon fields (4.1). Weapons with `SniperComboReset` but no
`SniperComboMin` (Arca Scisco, Knell) have no damage multiplier here (their counters are weapon passives, `04`).

**EV evaluation.** p_i = floor(MS) + Bernoulli(frac(MS)) per shot (E-10). The EV evaluator MUST propagate the exact
probability mass function of c over the shot sequence (shift by floor(MS), mix with probability frac(MS), apply each
decay as a shift by −1 clamped at 0). The pellet count of a shot and its M_snp are correlated (more pellets raise the
counter of the same shot), so the expected multiplier per pellet of shot i is the pellet-weighted value

```text
E-6.16a  M̄_snp,i = Σ_c Σ_p P(c_prev = c) · P(p_i = p) · p · M_snp(c + p)  /  MS
         (expected damage of shot i = MS · E[damage per pellet without M_snp] · M̄_snp,i)
         children and DoT seeds of shot i use the same M̄_snp,i (Decision; the child count is p_i for projectiles)
```

which follows from E-6.16 and the independence of the pellet draw from the earlier history (TV-06-08 lists both the
unconditional `E[M_snp]` and `M̄_snp`). MC samples p_i with the shot's multishot draw of `02`.

### 7.3 Auto-spool fire rate

```text
E-6.17 r_k = FR · min(1, s_sp + k · Δ_sp)        k = shots since the last reset (k = 0 is the first shot)
       interval after shot k = 1 / r_k
       reset: at every reload (k := 0) (Decision)
       fallback when Δ_sp is unknown: Δ_sp = (1 − s_sp) / (S_sp − 1)
```

| Weapon | S_sp (data `Spool`) | s_sp | Δ_sp | Source (weapon page § Characteristics via `F:weapon` §6) |
|---|---|---|---|---|
| Gorgon | 9 | 0.20 | 0.10667 | Gorgon |
| Gorgon Wraith | 6 | 0.20 | 0.16 | Gorgon Wraith |
| Prisma Gorgon | 7 | 0.20 | 0.1333 | Prisma Gorgon |
| Soma | 6 | 0.25 | 0.15 | Soma |
| Soma Prime | 4 | 0.25 | 0.30 | Soma Prime |
| Supra | 5 | 0.10 | 0.225 | Supra |
| Supra Vandal | 4 | 0.40 | 0.20 | Supra Vandal |
| Tenora, Tenora Prime | 9 | 0.40 | 0.075 | Tenora |
| Aksomati | 6 | 0.25 | 0.15 | Aksomati |
| Aksomati Prime | 11 | – | fallback | no page values (UNVERIFIED) |
| Secura Dual Cestra | 5 | 0.60 | 0.08 | page says 5 shots but 0.6 + 4·0.08 = 0.92 (UNVERIFIED) |
| Kohmak | 4 | 0.20 | 0.32 | `W:Kohmak` L27 |
| Twin Kohmak | 11 | 0.20 | 0.08 | Twin Kohmak |
| Kohm, Kuva Kohm | 5 | FR0(single)/FR0(full) ≈ 0.2 | fallback 0.2 | data ratio (7.4); no page step |
| Grattler | 4 | 0.40 | 0.20 | Grattler |
| Phaedra | 11 | 0.50 | 0.05 | Phaedra |
| Imperator Vandal | 16 | 0.5333 | 0.0311 | Imperator Vandal |
| Deth Machine Rifle | – | – | – | `Auto-Spool` without `Spool` field: treated as AUTO at full rate (Decision) |

Sources: `F:weapon` §6 table and consistency check; fire-rate bonuses scale the whole curve (`W:Fire_Rate` L5, "also
affects Spool-up Time"); Kuva Grattler "no longer auto-spool" (data trigger `Auto`). UNVERIFIED (overview D-079,
OQ-603): interval after shot k uses r_k rather than r_{k+1}; reset at reload (no source; "burst firing maintains
spool-up" `W:Gorgon` shows that short pauses do not reset it). Full-rate EFR = FR (`W:Damage/Calculation` L477 assumes
fully spooled).

### 7.4 Kohm family (spooling pellets)

```text
E-6.18 Kohm, Kuva Kohm (P_max = 12), Kohmak, Twin Kohmak (P_max = 5) are ONE spooling attack:
       base pellets   n_k  = min(P_max, 1 + k)           (k as in E-6.17, reset at reload)
       base multishot MS0_k = n_k                         (multishot bonuses scale it, `02` E-10)
       listed status  SC0_k = SC0(single) / n_k           (division owned by `02`)
       fire rate      r_k from E-6.17 with FR = data FireRate of "Fully Spooled", s_sp = FR0(single)/FR0(full)
       ammo cost      c_k = max(1, n_k / 3) · (1 − AE)
       damage per pellet: the data per-pellet map (identical in both data attacks)
```

Sources: `W:Kohm` L8, L25-30 ("each consecutive shot adds an additional pellet, up to a maximum of twelve", "At full
spool it shoots 12 pellets and uses 4 ammo per shot", "66 shots ... spending 244 rounds", status "divided between the
pellets"); `W:Kohmak` L21-28, L40-41 ("4 and 5 rounds before reaching optimal fire rate and multishot", "starts at 20 %
... increases by 32 % per shot", "1.6667 (5/3) ammo per shot at maximum spool", "25 shots, with one round left", "its
maximum of 40 shots" unspooled); data `D:Kohm`, `D:Kohmak` (Single Pellet vs Fully Spooled: same per-pellet damage, SC
0.75 → 0.0625 = 0.75/12, 0.69 → 0.138 = 0.69/5, cost 1 → 4 and 1 → 1.67). The cost rule `max(1, n/3)` is inferred: it
reproduces the Kohmak magazine note exactly (25 shots, 0.667 left) and the Kohm note's 66 shots, but gives 243 rounds
spent instead of the page's 244 (`n/3` would give 26 Kohmak shots). UNVERIFIED (OQ-604). The inspiration treats the two
data attacks as separate constant-rate attacks (D-079).

### 7.5 Peacemaker (Regulators)

```text
E-6.19 rate at shot i:   r_i = FR · (0.25 + 0.75 · min(1, s_i / 3.5))     (s_i since the cast; FR from E-6.2)
       interval after shot i = 1 / r_i ; infinite ammo; no reload
       burst index     b_pm(i) = floor(i / 2) + 1
       damage factor   M_pm(i) = 1 + 1.5 · min(b_pm, 20) / 20     (member of G, `04`)
```

Source: `W:Peacemaker` L24-35 ("fires in bursts of 2 shots", "+7.5 % per burst ... up to +150 %", "The first burst
receives ... +7.5 %", fire rate "ramping up from 25 % to 100 % over 3.5 seconds when firing", "multiplied against
other modifiers to fire rate", resets after 2 s without firing). Linear-in-time ramp and the pairing of shots into
bursts are UNVERIFIED (OQ-615). Base damage 50 and the Strength-scaled innate bonus are `01`/`04`.

---

## 8. Distance: falloff and range

### 8.1 Falloff curve

```text
E-6.20 Φ(d) = 1                                    d ≤ d_s
            = 1 − r_f · (d − d_s) / (d_e − d_s)    d_s < d < d_e
            = 1 − r_f                              d ≥ d_e
       attacks without Falloff: Φ = 1
       non-AoE attacks: d = distance (T-27, default 0); d_s, d_e × (1 + ΣPS) (projectile-speed bonuses)
       AoE attacks: d = d_aoe; d_e × (1 + ΣRadius)
```

Sources: `W:Damage_Falloff` L4-24 (200 damage, 10-30 m, 80 % → 120 at 20 m; minimum from 75 % Bronco to 6.25 %
Redeemer), snipers 400-600 m to 50 % (`W:Damage_Falloff` L9; data, e.g. `D:Rubico`), projectile speed modifies falloff
ranges of weapons with falloff (L30-33), AoE: "linear from central impact", radius mods increase it (L36-38;
`W:Area_of_Effect` L18), data semantics `DS:wiki` §3. The proportional scaling of both distances by projectile speed is
UNVERIFIED (OQ-609). Since U27.2.2 radial reductions are per weapon (data) (`W:Area_of_Effect` L370-374).

### 8.2 The single-target AoE distance

The target is the directly hit enemy, so every AoE child produced by a hit on it has `d_aoe = 0` and Φ = 1 (Decision,
from overview CD-11 single target). Radial reduction values matter only for the multi-target mode of overview 1.3; they
are kept in data. Standalone AoE firing attacks (Balefire Charger, Glory, Neutralizer Cryo Grenade) also use d_aoe = 0.

### 8.3 Effective falloff factor with progenitor and Incarnon flat

```text
E-6.21 Φ_v(d)  = 1 (d ≤ d_s);  1 − (d − d_s)/(d_e − d_s) (d_s < d < d_e);  0 (d ≥ d_e);  1 if no Falloff
       Φ_eff(d) = ( Σ_t B0_t · m_str · Φ(d)  +  Σ_t V_t · Φ_v(d)  +  X ) / B'          (B' as E-1)
       Φ_eff enters G_F (ST-4) as the "range falloff" factor; the composition q_t is unchanged
```

Sources: progenitor "uses the weapon's falloff range but always scales linearly from 100 % to 0 %, regardless of what
the weapon's falloff damage scaling is" (Kuva/Tenet/Coda rendered pages, `F:core` §3.2); "Incarnon Genesis Base Damage
increases from evolutions ignore damage falloff" (`W:Damage_Falloff` L92, listed under Bugs); additive CO ignores falloff
(`W:Damage_Falloff` L91; it follows from G_F, overview E-11). Decision, UNVERIFIED (OQ-609): falloff is applied as one
scalar on the total (the three components keep the composition of distance 0); the alternative (per-component
falloff before composition) changes q_t with distance and has no source. Progenitor falloff on a weapon without a data
falloff range: none (Decision).

### 8.4 Range limits

Hitscan attacks (`Hit-Scan`, HELD) deal no damage when d exceeds `Range` (data; hitscan default 300 m, snipers 1000 m,
beams their beam length): `W:Damage_Falloff` L28-29 (U22), `M:Weapons.lua` L360-363. Projectile attacks have no limit
here. Beam-length bonuses (Sinister Reach) extend HELD ranges per the `04` catalogue. A shot out of range is still
scheduled and spends ammo but produces no damage instance (Decision).

### 8.5 Hit time

```text
E-6.22 h_i = s_i + d / v_proj    for ShotType Projectile with ShotSpeed v_proj > 0 and d > 0
       h_i = s_i                 otherwise (hitscan, beams, d = 0)
       v_proj = ShotSpeed · (1 + ΣPS)
```

Decision, UNVERIFIED (OQ-618): straight flight at constant speed (no gravity, no acceleration); no source gives
flight-time rules. With the default d = 0 every hit is at its fire time.

---

## 9. Parent and child attacks

### 9.1 Resolving links

```text
E-6.23 parent(a) = attack with AttackIndex = a.AttackParentIndex                     if the field exists
                 = nearest earlier attack b with b.ShotType ≠ AoE                   if a.ShotType = AoE and a has no AmmoCost
                   (child; flagged in the data-build report)
                 = none (a is a firing attack)                                       otherwise
       hand-authored overrides win over both rules
```

Sources: `AttackParentIndex` links an explosion to its impact and is "used inconsistently" (`DS:wiki` §3, §11; Ack &
Brunt vs Nikana Prime). Checked on the cached data: Opticor "Charged Shot AoE", Braton/Burston "Incarnon Form AoE"/"Radial
Attack", Zylok "Incarnon Form Radial Attack", Javlok and Morgha explosions carry no `AttackParentIndex`; the second rule
links them to the preceding direct attack. Known cases that need overrides: Komorex "3.5x Zoom Radial Attack" (no
`ShotType`), Phantasma "Cluster Bombs Impact" (projectile child of "Plasma Bomb Impact"), Kuva Bramma "Cluster Bomb
Contact" (projectile child of "Charged Shot"), Quanta "Cube (shot by player)". Bubonico "Radial Attack" has both a
parent and an `AmmoCost` (3); the parent link wins and the child's `AmmoCost` is ignored (Decision).

Each child is a separate attack with its own base damage, crit, status and falloff (`F:weapon` §10.1; overview 2.3,
CD-12). Child attacks are never scheduled on their own; their hits are generated by their parent's shots.

### 9.2 Children per parent shot

```text
E-6.24 child instances per parent shot
       parent ShotType Hit-Scan or HELD (hitscan explosions, beam AoE):  1   (not multiplied by multishot)
       parent ShotType Projectile:                                         p_i (one per parent projectile)
       times the child's data Multishot if > 1 (cluster counts), not scaled by multishot bonuses
```

Sources: hitscan explosions are not multiplied by multishot (Opticor bug note, Trumna, and the CO table's "AoE does not
scale off multishot" for Ambassador alt, Braton/Burston Incarnon, Ferrox, Tenet Ferrox, Glaxion Vandal, Opticor Vandal:
`F:weapon` §10.1); beam AoE unaffected by multishot (`W:Multishot` §Continuous Weapons via `F:weapon` §4.2); launchers
create a rocket per multishot ("difficult to see additional rockets generated by Multishot", `W:Ogris`) with an explosion
each (implied, UNVERIFIED); Nightwatch Napalm "Each additional rocket spawned by Multishot will spawn an overlapping copy
of the blaze" (`W:Nightwatch_Napalm`, cached 2026-11-23). The cluster rule is a Decision (UNVERIFIED, OQ-610). In EV,
`p_i` enters linearly (E[p] = MS); in MC each parent projectile spawns its child.

### 9.3 Child timing

`h_child = h_parent + ExplosionDelay` (or `EmbedDelay`), data, default 0 (Lenz bubble 1.3 s, Gorgon Incarnon 1.1 s,
Multron 1.5 s, Stug blobs 1.5 s; list from the cached data). The child counts for `dps_window` only if `h_child ≤
t_win`. Status timing CD-04 applies to the child as a separate instance at its own time.

### 9.4 Lingering effects

Children with `EffectDuration` D_e > 0 (Torid/Pox poison clouds, Mutalist Cernos toxin cloud, Tysis/Stinger DoT,
Cryotra field, Ferrox attraction field) and the Nightwatch Napalm blaze:

```text
E-6.25 ticks at h_spawn + m / FR_child,  m = 0 .. ceil(D_e · FR_child) − 1     (first tick immediate)
       each spawned effect ticks independently on the single target
       Nightwatch Napalm (Ogris, Kuva Ogris; mod on the explosion attack): FR_child = 1, D_e = mod rank duration
       (6 s at max rank): 6 ticks at h, h+1, ..., h+5 per rocket; tick damage and its 3 s Heat procs: `03`
```

Source for Napalm: `W:Nightwatch_Napalm` (cached 2026-11-23): "It ticks 6 times, once per second, with the first tick
occurring immediately", "does not suffer from falloff", "cannot deal critical hits", fixed 68 % Heat chance lasting
3 s, base 150 Heat per tick. Generic lingering effects: first-tick timing, tick rate = data `FireRate` of the child and
independent stacking on one target are UNVERIFIED (OQ-611). Attacks whose `ShotType` is `DoT` without
`EffectDuration`, and Alternox/Simulor singularity attacks, are not scheduled (flagged as unsupported) (Decision).

### 9.5 Reload-triggered attacks

Attacks listed as reload-triggered (Catabolyst and Coda Catabolyst "Partial Reload ..." and "Reload From Empty ...",
`D:Catabolyst`) fire one instance at each reload start of the weapon's primary attack schedule: "Reload From Empty" when
the reload is `from_empty`, "Partial Reload" otherwise; their children follow 9.2-9.3. Under continuous fire every
reload is from empty. Synth Charge applies to the reload grenade (`W:Synth_Charge` L35). Discrepancy D-605.

### 9.6 Radial headshot rule

Every damage instance of an AoE attack (child or standalone) has `is_radial = true`: `Part = 1` in E-7, T-03 does not
apply, it cannot trigger headshot or weak-point conditions and does not count for the sniper combo. The parent
projectile keeps its own body-part rules.

Sources: U32 "Radial damage no longer gains extra headshot damage or triggers headshot conditions ... Explosive
projectiles themselves can still land headshots" (`W:Area_of_Effect` L345-347, L20-21); "AoE damage also cannot
headshot" (`W:Damage/Calculation` L437). Owner of E-7: `02`.

---

## 10. Steady-state quantities: EFR, shots per magazine, duty

`dps_burst` and `dps_sustained` (overview 4.1) need per-attack rates without walking a window.

### 10.1 Effective fire rate (burst rate)

```text
E-6.26 EFR = FR_trig                                   AUTO, SEMI, HELD; SPOOL at full rate
           = BC / (1/FR + (BC − 1) · BD)                BURST (MAG_BURST: BC = N_mag)
           = 1 / (CT + 1/FR)                            CHARGE (Lanka: 1/CT)
           = 1 / (CT + T_rel)                           bows (Class Bow) and exalted bows (11.3): the nock is part of the cycle
           = Peacemaker: FR (full ramp)
```

Sources: `W:Damage/Calculation` L444-499, `W:Fire_Rate` L31-52, L170-195. Bows are the only class whose EFR contains the
reload, as on the wiki; for every other class EFR is the in-magazine rate and the reload enters through ρ.

### 10.2 Shots per magazine and cycle time

```text
E-6.27 N_mag = Mag / c                    (long-run average; ∞ for c = 0)
E-6.28 T_cyc = long-run mean of (reload_end − first cycle start of the magazine), from the schedule E-6.11
       closed forms (no AE debt, constant rate r, integer N):
         AUTO/SEMI/HELD: T_cyc = N/r + T_rel − [no delay]/r
         CHARGE:         T_cyc = N·(CT + 1/FR) + T_rel − [no delay]/FR
         BURST:          T_cyc = floor(N/BC)·P + [rem > 0]·((rem − 1)·BD + 1/FR) + T_rel,  P = (BC − 1)·BD + 1/FR,
                         rem = N − BC·floor(N/BC)
       T_mag = T_cyc − T_rel
```

With ammo debt the number of shots alternates between floor and ceil of N_mag (`W:Ammo` L217-220); the engine MUST then
compute T_cyc from the schedule by running E-6.11 (with H = ∞) until the post-reload ammo repeats within 1e-9 or 10,000
magazines have been simulated, and average shots and time over the simulated magazines.

### 10.3 Duty

```text
E-6.29 ρ = (shots per second over full magazine cycles) / EFR = N_mag / (EFR · T_cyc)
       ρ = 1 for: infinite-ammo attacks, Incarnon form attacks (one activation, no reload), bows, Epitaph,
       Peacemaker
```

For constant-rate attacks with a reload delay, E-6.29 reduces to the overview's `T_mag / (T_mag + T_rel)`; for Vectis
it reduces to `N / (N − 1 + FR·T_rel)` (`W:Damage/Calculation` L539-545). Spool weapons: overview 4.2 fixes "spool
full" for the steady metrics, so `dps_sustained` uses T_cyc with every interval at the full rate (`N/FR + T_rel`); the
spool-up of each magazine enters only `dps_window` through the walked schedule (TV-06-06 gives both values; changing
the steady rule is proposal OQ-620). Battery: T_rel is δ_rech' plus the regeneration time; the
unavoidable 1/FR lead before the recharge is the last interval inside T_mag (TV-06-10).

### 10.4 Steady means of time-dependent multipliers

Overview 4.2 binds the steady metrics (`dps_burst`, `dps_sustained`) to "beam ramp full; spool full" and to "combo,
sniper combo and conditional stacks at their steady values". For each schedule-driven `G` member the steady value is:

| Factor | `dps_burst` | `dps_sustained` |
|---|---|---|
| beam ramp | 1 (overview 4.2) | 1 (overview 4.2); proposal OQ-620: η_rmp = mean of L over the ticks of a steady magazine (magazine 2 onward, TV-06-07) |
| first/last-shot η_1st | `1 + b_first · (first shots per magazine)/N_mag` (and the same for last) | same |
| sniper combo η_snp | pellet-weighted mean of M̄_snp,i (E-6.16a) over the shots of the window schedule [0, t_win] | same |
| Peacemaker | 2.5 | 2.5 |

The first/last mean follows `F:weapon` §16 (`1 + b/N`; Vectis gets the bonus on every shot, `W:Primed_Chamber` L44);
the overview does not fix it (Decision). The sniper combo has no steady value (the counter grows without bound under
continuous hits), so its "steady value" is defined as the window mean (Decision, UNVERIFIED as a metric choice; T-12
replaces it; recorded in OQ-620). If OQ-620 is adopted, proc rates are not scaled by η_rmp (the ramp scales damage,
not status chance).

---

## 11. Weapon families

### 11.1 Kitguns

```text
E-6.30 chamber c, grip g, loader l (Module:Modular/data KitgunPrimary or KitgunSecondary by grip slot):
       base damage   = c.Damage[g]                (per-type map; grips are a vector offset, U28.0.5)
       FR0           = c.FireRate[g]
       CT0           = c.ChargeTime[g]            (Tombfinger primary)
       Range         = c.Range[g]                 (Gaze, Vermisplicer: the grip sets beam range, not fire rate)
       CC0, CDM0, SC0 = c.X + l.X                 (additive offsets)
       Mag0          = c.Magazine[l.Magazine]     (tier name)
       R0            = l.Reload
       trigger, AmmoCost, Multishot, ShotType, Falloff, ForcedProcs = chamber values (per slot)
       Pax Charge (arcane, `04`): reload_style := regenerate with δ0 = R0 (rate from the `04` catalogue)
```

Sources: `F:weapon` §11 (assembly pattern, Catchmoon + Arcroid CC 0.21 + 0.03 = 0.24 checked against the expanded
table), `DS:wiki` §3 (parts structure, 0 mismatches for damage and fire rate over 1200 combos), `W:Kitgun` L26 (beam
grips set range), L144 (Pax Charge: "the base recharge delay is equal to base reload speed"), U28.0.5 grip vector note.
Historical per-chamber loader exceptions (Thunderdrum on Catchmoon, `F:weapon` §11) are absent from current parts data;
the additive model is used (UNVERIFIED, OQ-617). The parts data hold one attack per chamber; whether Sporelacer and
Tombfinger should be split into a direct attack and a radial child is unknown (D-604, OQ-617). Secondary semi kitguns
can equip Cannonade since U37 (`W:Kitgun` L175; legality `09`).

### 11.2 Arch-guns

Archwing (space) and atmospheric (Gravimag) modes are separate data records ("X" and "X (Atmosphere)"); the user picks
one. Archwing entries are batteries (`ReloadStyle = Regenerate`) with falloff; atmospheric entries use a magazine and
reserve and mostly lack falloff (`F:weapon` §12, `W:Archgun`; Corvas Archwing `Regenerate` vs Corvas (Atmosphere)
`Reload = 2`, checked). All rules of sections 5-10 apply unchanged. Arcane and CO restrictions are `04`/`09`'s.

### 11.3 Exalted ranged weapons

| Weapon | Ammo model (Decision unless sourced) | Timing |
|---|---|---|
| Regulators (Peacemaker) | infinite (data `Magazine = 0`, `AmmoCost = 0`) | 7.5 |
| Balefire Charger | infinite; data `AmmoCost = 100` is energy/shield cost | CHARGE: `1/(CT + 1/FR)` (`W:Fire_Rate` general case) |
| Glory, Noctua | infinite (`Magazine = 0`) | data trigger |
| Dex Pixia | infinite (energy weapon; data battery fields without `ReloadStyle` are ignored, UNVERIFIED) | AUTO |
| Lizzie | infinite (its 275 "Backbeat ammo" is refilled by abilities, out of scope; UNVERIFIED) | HELD with ramp |
| Artemis Bow (Prime) | infinite energy (15 per shot, `W:Artemis_Bow` L24); Mag 1 nock cycle kept | bow cycle `CT + T_rel` (UNVERIFIED) |
| Neutralizer (Cyte-09) | magazine 12, reload 2.25 (the one exalted weapon that "uses both", `W:Exalted_Weapon` L8) | SEMI |

"These weapons do not utilize ammunition and instead consume energy (with the exception of Cyte-09's Neutralize which
uses both)" (`W:Exalted_Weapon` L8). Energy is out of scope (overview 1.3), so energy costs never stop the schedule.
Artemis Bow charge speed "is affected by fire rate mods" (`W:Artemis_Bow` L29); whether the bow ×2 applies is
UNVERIFIED (OQ-613; Decision w = 2 since it takes Bow mods, `F:mods` §4). Strength scaling of exalted base damage is
`01` (E-1 `m_str`).

### 11.4 Companion (robotic) weapons

Robotic weapons use their own data records (`Weapons/data/companion`, same schema, `AmmoMax = ∞`) and the rules of
sections 5-10 unchanged: the schedule is the continuous-fire upper bound because AI target acquisition and pauses are
not documented (`F:weapon` §15, overview 1.2 "theoretical upper bound"). Charged Chamber and Primed Chamber give no
first-shot flag on robotic weapons (`W:Charged_Chamber`, `W:Primed_Chamber` L55).

---

## 12. Worked examples

1. **Braton Prime with Speed Trigger** (overview 3.8 step 2): FR = 9.58 × 1.60 = 15.328; magazine 75; T_rel = 2.15. Shots
   at i/15.328; the 75th at 4.82777 s; reload 4.89301 → 7.04301 (δ_rel = 1/15.328); ρ = 4.89301 / 7.04301 = 0.694733,
   identical to the overview. Full numbers in TV-06-01.
2. **Opticor**, +60 % fire rate: CT = 2/1.6 = 1.25 s, FR = 1.6: shots at 1.25, 3.125, 5.0, 6.875, 8.75; reload 9.375 →
   11.375; next charge ends at 12.625. EFR = 1/(1.25 + 0.625) = 0.53333, ρ = 9.375/11.375 = 0.824176 (`F:weapon` §3.2
   gives 0.824). TV-06-03.
3. **Falloff with progenitor and flat**: B0 = 100 Impact with falloff 10-30 m to 20 % (r_f = 0.8), valence 60 % (V = 60),
   Incarnon flat X = 20, d = 20 m: Φ = 0.6, Φ_v = 0.5, Φ_eff = (60 + 30 + 20)/180 = 0.611111. TV-06-11.

---

## 13. Edge cases

Every exception, weapon-specific behaviour and quirk from the findings files that touches the firing model. "Out of
scope" rows name the owner or the reason.

| ID | Rule | Source |
|---|---|---|
| EC-06-01 | Burst delay shrinks with positive ΣFR only; negative ΣFR leaves BD0 (E-6.5). | `W:Fire_Rate` L195; `F:weapon` edge 1 |
| EC-06-02 | Bow ×2 applies only to sources whose card says "(x2 for Bows)"; Arcane Acceleration is not doubled; crossbows not doubled (UNVERIFIED). | `F:weapon` §1.5; `W:Arcane_Acceleration` L39; `W:Bow` L3 |
| EC-06-03 | Semi cap 10/s (Komorex ≈ 8); Duplex treated as Semi per shot (E-6.3). | `W:Fire_Rate` L8; `W:Komorex`; OQ-012 |
| EC-06-04 | Cannonade locks FR, CT, BD and spool curve to base, ignoring negative bonuses (zoom penalty, Critical Delay, Vile Precision); applies to every attack incl. Latron/Vectis Incarnon forms. | `W:Fire_Rate` L329-331; `W:Semi-Rifle_Cannonade` |
| EC-06-05 | FR floor 0.05/s; CT ceiling 10 × CT0. | `W:Fire_Rate` L29, L334 |
| EC-06-06 | Lanka: FR = ∞ in data, so no delay between charged shots (EFR = 1/CT), δ_rel = 0. | `W:Fire_Rate` L52; `DS:wiki` §3 |
| EC-06-07 | Bows: Mag 1 fixed, cycle `CT + T_rel`, no reload delay, rifle reload mods apply. | `W:Bow` L10-12; `W:Reload` L13, L109 |
| EC-06-08 | Epitaph: no reload (draws from reserve), Mag = ∞ for scheduling, EFR UNVERIFIED. | `W:Damage/Calculation` L546 |
| EC-06-09 | Vectis, Vectis Prime, Exergis (data `ReloadDelay = 0`) and every Mag-1 weapon (Tonkor, Knell, Vulcax): no reload delay. | `W:Reload` L13; `W:Fire_Rate` L17; OQ-015 |
| EC-06-10 | Sniper combo counts every projectile of a shot; AoE, DoT and misses-by-AoE do not count; scoped assumed. | `W:Sniper_Rifle` L37-41 |
| EC-06-11 | Lanka combo duration 6 s; Harkonar Scope extends it (`04`). | `W:Sniper_Rifle` L37 |
| EC-06-12 | Lanka with 2 projectiles reaches 1.5× on the first shot (counter includes the shot). | `W:Lanka` L46 |
| EC-06-13 | Lanka and Komorex zoom bonuses are exceptions to additive zoom stacking (crit, `02`); Komorex 3.5× zoom: −75 % fire rate enters ΣFR, +3 m radius irrelevant at d_aoe = 0. | `W:Sniper_Rifle` L12, L60-80 |
| EC-06-14 | Beams: one tick per 1/FR, data AmmoCost (0.5) per tick, multishot merge per CD-10 (`02`). | `W:Continuous_Weapon` L17-18 |
| EC-06-15 | Beam ramp 0.2 → 1 over 0.6 s, decay after 0.8 s idle over 2 s; reloads longer than 0.8 s cost ramp (E-6.15). | `W:Continuous_Weapon` L19-20 |
| EC-06-16 | Ramp starts: Convectrix 60 % (alt 80 %), Phage 70 %, Embolist 30 %. | `W:Continuous_Weapon` L123-129 |
| EC-06-17 | Beam AoE and hitscan explosions are one instance per parent shot (not multishot-scaled); launcher projectiles explode per projectile (E-6.24). | `F:weapon` §4.2, §10.1 |
| EC-06-18 | Tenet Flux Rifle is Auto (no ramp, no beam merge) despite its name; Basmu alt-fire is Held with ramp. | `F:weapon` §4.3; `D:Tenet Flux Rifle` |
| EC-06-19 | Kohm family: pellets spool 1 → P_max, per-pellet status divided by pellets (`02`), cost `max(1, n/3)` (UNVERIFIED), reload when the next shot cannot be paid. | `W:Kohm` L8-30; `W:Kohmak` L21-41 |
| EC-06-20 | Spool: fire-rate bonuses scale the curve; Kuva Grattler has no spool; Deth Machine Rifle has `Auto-Spool` but no `Spool` field (treated as Auto). | `W:Fire_Rate` L5; data |
| EC-06-21 | Secura Dual Cestra page cadence inconsistent (5 shots vs 0.6 + 4·0.08 = 0.92). | `F:weapon` §6 |
| EC-06-22 | Instant bursts (BD = 0: Quartakk, Kuva Quartakk, Quatz, Akjagara, Sepulcrum, Morgha) are BC separate shots at the same time, ordered by index (CD-04). | `F:weapon` §2; `F:status` edge 9 |
| EC-06-23 | Partial last burst: remaining shots fire as a shorter burst (Burston Prime). | `F:weapon` §2 |
| EC-06-24 | Mag Burst (Pandero, Nagantaka, Cyanex, Kuva Kraken, Tenet Detron, Athodai alt): BC = shots the magazine can pay; `BurstReloadDelay` read as the reload lead (UNVERIFIED). | `W:Damage/Calculation` L499; `F:weapon` §2 |
| EC-06-25 | Tenet Diplos "Lock-on Mode" is `Burst` with no `BurstCount` (BC = 1 default; lock count unknown). | data; `M:Weapons.lua` L476 |
| EC-06-26 | AmmoCost per shot on burst attacks (Bubonico 3, Haalvu 1.5): per-shot reading (UNVERIFIED). | 5.6; OQ-605 |
| EC-06-27 | Whole-magazine attacks (Tenet Tetra alt cost 80, Sepulcrum alt, Tenet Plinx alt cost 10 with its own `Reload = 2.5`): one shot per magazine; damage scaling with ammo is a `04` passive; no Synth Charge. | `F:weapon` §8.2; `W:Synth_Charge` L49-60 |
| EC-06-28 | Magazine: nearest rounding (UNVERIFIED vs ceil), minimum 1, percent before flat; bows, Epitaph and Incarnon forms ignore magazine bonuses. | 5.5 |
| EC-06-29 | Ammo efficiency cap 0.75 (Hotfix 44.1.3); infinite-ammo sources separate; neither affects Incarnon forms; ammo debt carried across reloads. | `W:Ammo` L201-222 |
| EC-06-30 | Arcane Pistoleer grants infinite ammo since 44.1.3 (was ammo efficiency). | `F:addmul` §5 |
| EC-06-31 | Per-shell reload (Strun family, Corinth, Felarx, Zarr, Kuva Zarr): magazine bonuses lengthen the reload (Tactical Pump beats Ammo Stock). | `W:Reload` L45-49, L206 |
| EC-06-32 | Battery: reload speed affects only the delay, delay floor 0.5 s, empty vs non-empty delay (Basmu 2/0.2, Shedu 1/0.4, Imperator Vandal 0.25/0.15), Shedu delay unaffected by reload speed (U30.7), the 1/FR lead cannot be skipped, batteries cannot regenerate between shots. | `W:Reload` L6, L51-58; `W:Battery_Weapon` L6, L68-69 |
| EC-06-33 | Cycron "delay halved when the mag is not empty" (U20.4): superseded by data `ReloadDelay` only; unused under continuous fire (always empty). | `W:Battery_Weapon` L75 |
| EC-06-34 | Every reload in the schedule is a full reload; reload interruption, holster reload (Tactical Reload, Synth set, Ferrox/Javlok/Scourge 33 %/s), instant reload (Blood Forge, Charm) and speargun refill while thrown are out of scope (not continuous fire). | `W:Reload` L45, L146-170; `F:weapon` §7, §9.5 |
| EC-06-35 | Reserve ammo is unlimited (AmmoMax ignored; Kuva Bramma holds only 5 arrows); lifetime damage is not an output. | Decision; `W:Damage/Calculation` L771 |
| EC-06-36 | AoE damage has 1× headshot and triggers no headshot conditions since U32; the projectile itself can headshot (9.6). | `W:Area_of_Effect` L20-21, L345-347 |
| EC-06-37 | Direct hit and AoE are separate instances with separate stats; CO not on AoE except the directly hit target of hitscan explosions (`04`); Synth Charge affects AoE. | `W:Damage/Calculation` L395-409; `W:Synth_Charge` L31 |
| EC-06-38 | Some radii cannot be modded (Trumna vs Firestorm): irrelevant at d_aoe = 0. | `F:weapon` §10.1 |
| EC-06-39 | Primary Compression is a separate multiplier on projectile AoE and additive on hitscan AoE: bucket `04`; Napalm blaze ignores it. | `F:addmul` §5; `W:Nightwatch_Napalm` |
| EC-06-40 | Range falloff: shotguns and others per data; snipers 400-600 m to 50 %; hitscan limit 300 m, snipers 1000 m, beams their range. | `W:Damage_Falloff` L9, L28-33 |
| EC-06-41 | Projectile-speed bonuses scale falloff ranges of weapons with falloff; hitscan weapons without falloff ignore them. | `W:Damage_Falloff` L30-33 |
| EC-06-42 | Progenitor bonus falls off linearly 100 % → 0 % over the weapon's falloff range; Incarnon flat base damage ignores falloff (bug); additive CO ignores falloff (bug). | `F:core` §3.2; `W:Damage_Falloff` L91-92 |
| EC-06-43 | Partial charges are separate data attacks (Lanka 33 %, Opticor 40 % minimum, bows); bow quick shots share crit/status with charged shots since U25.7 (data), Epitaph does not. | `F:weapon` §3.3; `W:Bow` L83 |
| EC-06-44 | Autocharge weapons (Javlok, Stahlta/Battacor/Tenora/Tenet Plinx/Ambassador/Aeolak alt, Vadarya Prime) use data CT; Angstrum charge time scales with magazine size (hand-authored per-attack CT, UNVERIFIED). | `W:Trigger_Type` L55-59; `W:Fire_Rate` charge list |
| EC-06-45 | Phantasma charge time is not affected by mods. | `W:Fire_Rate` charge list |
| EC-06-46 | Every charged shot, including the first after a reload, pays its full charge. | `W:Damage/Calculation` L483-488 |
| EC-06-47 | Primed and Charged Chamber: first shot = HUD counter Mag − 1 after the shot (AE shifts which shots qualify); stack additively (`04`); no effect on robotic weapons, Vulklok; Vectis gets it every shot; no flag in Incarnon forms (UNVERIFIED). | `W:Primed_Chamber` L34-68 |
| EC-06-48 | Synth Charge: last shot of a real magazine; equip rule base magazine ≥ 6 with Kulstar exception (`09`); still works if the magazine is reduced below 6; not on beams except the Catabolyst reload grenade, not on Incarnon forms, not on whole-magazine shots. | `W:Synth_Charge` L27-60 |
| EC-06-49 | Torid Incarnon "+3 multishot on last shot in magazine" and "on reload from empty" perks (Ready Retaliation +100 % reload speed): driven by the `last` flag and `from_empty` reloads (values `04`). | `F:weapon` §16; `W:Lex_Incarnon_Genesis` L78-80 |
| EC-06-50 | Primary Plated Round: bonus scaled by fraction reloaded; always a full reload here. | `F:weapon` §9.5 |
| EC-06-51 | Incarnon form: own magazine (IncarnonCharges), no mods, no AE, no infinite ammo, no reload, ends when empty; switch animation = reload time; exiting refills the normal magazine; magazine perks apply only to the normal form; Onos form charged attack costs 50. | 6.6 |
| EC-06-52 | Braton Incarnon "Munitions Grit": multishot projectiles consume ammo, from the form magazine in Incarnon form; the schedule uses the expected cost `c = cost0 · MS / MS0` per shot (Decision, UNVERIFIED). | `W:Braton_Incarnon_Genesis` L63-72 |
| EC-06-53 | Cannonade on Incarnon weapons only if both modes are Semi (Latron, Vectis); Synth Charge no effect on Incarnon forms. | `F:weapon` §13 (legality `09`) |
| EC-06-54 | Exalted weapons: energy instead of ammo except Neutralizer; three Strength-scaling models (`01`); Peacemaker FR and damage ramps (7.5); Artemis Bow nock cycle and charge affected by fire rate. | 11.3 |
| EC-06-55 | Arch-guns: Archwing and atmospheric records differ; Archwing guns are batteries; archguns are neither primary nor secondary for arcanes (`04`, `09`). | 11.2 |
| EC-06-56 | Kitguns: grip damage per chamber as a vector, loader CC/CD/SC additive, magazine tier per chamber, beam grips set range, Tombfinger primary grip sets CT, Pax Charge makes a battery. | 11.1 |
| EC-06-57 | Kitgun pre-expanded table is stale (missing `ChargeTime`); assemble from parts. | 4.1 |
| EC-06-58 | Robotic weapons: AI not modelled (upper bound); unlimited reserve; Burst Laser reload 0.01 s; set-mod counting `09`. | 11.4 |
| EC-06-59 | Compound triggers resolved per attack (4.2); attack fields override weapon fields (getStruct2). | 4.1-4.2 |
| EC-06-60 | Data typos: `Hitscan` (Zylok Incarnon), `Ammocost` (Coda Bassocyst), missing `ShotType` (12 attacks) or `Multishot` (11 attacks, default 1). | `DS:wiki` §3, §11 |
| EC-06-61 | Active-trigger launchers (Penta, Castanas, Talons, Aegrit, Simulor) assumed detonated on impact. | 4.2 |
| EC-06-62 | Delayed explosions (ExplosionDelay, EmbedDelay) land later than their parent (9.3); lingering clouds tick per 9.4 (UNVERIFIED); Napalm blaze 6 ticks from t = 0 per rocket. | 9.3-9.4 |
| EC-06-63 | Reload-triggered attacks (Catabolyst) fire once per reload (9.5). | `W:Synth_Charge` L35 |
| EC-06-64 | Beam chains, punch-through and ricochet extra targets are multi-target (out of scope, overview 1.3); chains use no ammo. | `W:Continuous_Weapon` L27-36 (chains use no ammo: L36) |
| EC-06-65 | Spearguns: multishot does not affect the throw (`02`); throw attacks are scheduled as Semi with their data fire rate. | `W:Multishot` §Spearguns |
| EC-06-66 | Shotgun status per pellet since U27.2 (`02`); all shotguns have falloff (data); pellets all hit (accuracy and spread not modelled; proposed option OQ-619). | `F:weapon` §7 |
| EC-06-67 | Internal Bleeding's "fire rate < 2.5" test uses the attack's modded FR from E-6.2; bows use their hidden fire rate, not the bow cycle (`02` owns the rule). Sources disagree for Cinta: the Internal Bleeding notes give 1.5 (`F:addmul` §8), data `FireRate` 0.667 (1.0 most bows, Kuva Bramma and Nataruk 0.667 in both); the data value is supplied, flagged for `02`. | `F:addmul` §8; `D:Cinta` |
| EC-06-68 | Accessibility auto-fire option does not change fire rate or apply to Duplex/Charge. | `W:Trigger_Type` L26 |
| EC-06-69 | Old pages are stale: `W:Reload` Viper example (reload 1.1 vs data 0.7), Continuous Weapon U22.13.3 "10 % over 0.8 s" (superseded by U22.14). | `F:weapon` §9.4; `W:Continuous_Weapon` L122-145 |
| EC-06-70 | Kuva Ogris/Ogris Nightwatch Napalm blaze: one per rocket, no falloff, no crit, ticks immediately and every second (9.4); damage `03`. | `W:Nightwatch_Napalm` |
| EC-06-71 | Status effect Heat 3 s on Napalm, Cold field, Toxin clouds: proc values `03`. | `03` |
| EC-06-72 | Conditional timing buffs (Arcane Acceleration on crit, Spring-Loaded Chamber on reload) follow 5.9. | 5.9 |
| EC-06-73 | Glory, Balefire Charger, Neutralizer Cryo Grenade and Talons are standalone AoE firing attacks (no parent); radial rules apply. | 9.1, 9.6 |
| EC-06-74 | Dex Pixia carries battery fields without `ReloadStyle`; Lizzie carries a 275 magazine: both treated as infinite (UNVERIFIED). | 11.3 |

---

## 14. Inspiration differences

Rows of the overview register owned (wholly or partly) by `06`, restated with the engine's behaviour, followed by new
rows. Column layout as overview 5.2.

| ID | Topic | Inspiration behaviour | Game behaviour (source) | Engine | Toggle |
|---|---|---|---|---|---|
| D-066 | Sniper combo | fixed multiplier, 1.5 when the option is off (JS L10326-10330) | counter per projectile hit, milestones ×3, −1 decays (`W:Sniper_Rifle` L14-41) | E-6.16 simulated; T-12 fixes M | User (T-12) + Compat |
| D-074 | Nightwatch Napalm (06 part: timing) | credited at the hit, truncated by `min(6, floor(T − i/(N/T)))` from the hit index (JS L10772-10774) | 6 ticks from the explosion, once per second, first immediate, one blaze per rocket (`W:Nightwatch_Napalm`) | E-6.25 tick schedule; damage `03` (note for `03`: the page says the description is outdated and the blaze deals a base 150 Heat per tick, the value the inspiration uses) | — |
| D-075 | Bow cycle | `1/(CT + reload)` per shot plus a magazine reload after every shot: interval `CT + 2·reload` (JS L12290-12293, L12333-12349) | `CT + Reload` (`W:Fire_Rate` L39-45) | E-6.11 with Mag 1, δ_rel = 0 (TV-06-04: 23 vs 13 shots in 20 s) | Compat |
| D-076 | Charge weapons (incl. Balefire Charger by name) | `1/(CT + reload)` per shot, only shot 0 pays the charge, magazine reloads on top (JS L12290-12296, L12351) | `1/(CT + 1/FR)`; Lanka `1/CT`; charge after every reload (`W:Fire_Rate` L31-52) | E-6.12 (TV-06-03) | Compat |
| D-077 | Semi-auto cap | none; `trigger` not read (`I:agg` §2.3) | 10/s, Komorex ≈ 8 (`W:Fire_Rate` L8) | E-6.3, UNVERIFIED value | — |
| D-078 | Beam ramp | not modelled (`I:agg` §2.3) | 20 % → 100 % over 0.6 s, decay after 0.8 s over 2 s (`W:Continuous_Weapon` L19-20) | E-6.15 | Compat |
| D-079 | Spool, Kohm family | no spool code (JS L12251-12406); Kohm/Kohmak "Single Pellet" and "Fully Spooled" are two constant-rate attacks | spool cadence per weapon; Kohm pellets spool (weapon pages) | E-6.17, E-6.18 | Compat |
| D-080 | Burst delay | `BD/(1 + speed)·(1 + speed_mult)`, negative speed also divides (JS L8941-8952) | `BD/(1 + max(0, ΣFR))` (`W:Fire_Rate` L195) | E-6.5 | — |
| D-081 | Shot-count bound | planned count `Pa` truncates: rate below 1/T gives 0 shots; one shot lost per partial cycle (JS L12299-12321) | — | event loop only; a shot at exactly t_win counts in every case (6.3), while the inspiration keeps it only in the loop branch (`I:agg` quirk Q18) (TV-06-12: 6 vs 5 shots; 1 vs 0 for T = 1) | — |
| D-082 | Battery and per-shell reload | battery recharge uses the AE-inflated magazine and `reloadRate` mods divide a rate; no 0.5 s delay floor; per-round reloaders untouched by reload mods (JS L9019-9041, L12278-12287) | `δ/(1 + RS) + Mag/Rate`, floor 0.5 s; per shell `(t_se + t_sh·Mag)/(1 + RS)` (`W:Reload` L45-58) | E-6.9 | — |
| D-083 | Ammo cost and efficiency | 0.5 per shot for every attack of a BEAM-tagged weapon or `isBeam` attack (e.g. Furis Incarnon Form, data AmmoCost 1: 560 instead of 280 ticks); AE up to 0.999999 as a magazine multiplier (JS L12264-12278, L12464-12471) | per-attack `AmmoCost`; AE cap 0.75; debt rule (`W:Ammo` L201-222) | E-6.7, E-6.8 | — |
| D-084 | First/last-shot marks | groups of `perMag` shots by index, also for infinite magazines; Synth Charge only when `perMag > 6` (JS L12182-12196, L12519-12524) | HUD-counter rule for the first shot; last shot of a real magazine; Synth equip rule on base magazine ≥ 6 (`W:Primed_Chamber` L39-41, `W:Synth_Charge` L27) | E-6.13 | — |
| D-085 | Damage falloff | display only (JS L7841-7842) | applied by distance; progenitor 100 % → 0 %; Incarnon flat exempt (`W:Damage_Falloff`) | E-6.20, E-6.21 at T-27 | User (T-27) |
| D-601 | Incarnon form magazine | the form magazine (`incMagazineSize`) is "reloaded" with the weapon's reload time when spent and firing continues (JS L12264-12266, L12333-12349) | the form ends when its ammunition is spent; charges must be re-earned by weak-point hits (`W:Braton_Incarnon_Genesis` L21, `W:Lex_Incarnon_Genesis` L20) | E-6.14: schedule ends | Compat |
| D-602 | Reload lead of no-delay weapons | every reload starts 1/Ca after the last shot, also for Vectis, Vectis Prime, Exergis, Tonkor (Vectis interval 1/FR + R = 1.667 s) (JS L12381, L12333-12344) | no reload delay for Mag-1 weapons and Vectis/Vectis Prime (`W:Reload` L13, `W:Fire_Rate` L17) | E-6.10 (TV-06-05: 21 vs 13 shots) | Compat |
| D-603 | Fire-rate floor, charge-time ceiling, magazine assembly | FR clamped at 0 (JS L8930-8931); no 10× charge-time ceiling (JS L8973-8982); magazine `round((base + baseMag)·(1 + Σ) + add)` without a minimum (JS L9007-9018) | FR ≥ 0.05, CT ≤ 10·CT0 (`W:Fire_Rate` L29, L334); magazine ≥ 1, flat after percent (`W:Depleted_Reload`, `W:Calculating_Bonuses` L76-77) | E-6.2, E-6.4, E-6.6 | — |
| D-604 | Kitgun assembly | loader `crit_mult` +1 for every Sporelacer chamber, both slots (JS L14559-14560); primary Sporelacer split into "Normal Attack" Impact and "Explosion" Toxin (JS L5594-5600) | parts data: Sporelacer CDM0 3 (secondary) and 2 (primary), one attack with both types (`Module:Modular/data`) | wiki parts data (UNVERIFIED which is the game, OQ-617) | — |
| D-605 | Reload-triggered attacks | Catabolyst "Partial Reload" and "Reload From Empty" attacks simulated as 12/s streams with the weapon magazine and 0.5 ammo each (`I:agg` §7.5) | one grenade per reload (`W:Synth_Charge` L35 "reload grenade") | 9.5 | — |

---

## 15. Open questions

Overview questions owned here, restated with the evidence that would settle them, then new ones.

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-012 | Semi-auto cap value (frame-rate dependent?) and Duplex cadence (Tigris) | 10/s; Duplex per-shot FR | frame-timed fire-rate measurement at high FR with the auto-fire accessibility option; Tigris FR 2, Mag 2: 2 shots per 0.5 s or per 1 s |
| OQ-013 | Beam ramp shape, hitting-interval convention, current per-weapon starts | linear, 0.2 default, U22.14 exceptions | damage numbers of the first 10 ticks of Amprex, Phage, Convectrix on a fresh Simulacrum target; damage after reloads of 0.8 s and 2.8 s |
| OQ-014 | Burst delay divided by (1 + ΣFR) | yes, positive ΣFR only | Burston with Speed Trigger: measured cadence 14.91 vs 12.15 shots/s |
| OQ-015 | Mag-1 weapons other than Vectis: reload delay 1/FR? | none | Tonkor cycle time (1.7 s vs 2.02 s) |
| OQ-601 | Magazine rounding: nearest or ceil? | nearest half up | Tigris (Mag 2) with a +20 % magazine bonus: 2 or 3 rounds |
| OQ-602 | Incarnon "+X Base Magazine Capacity": before or after percent bonuses? | after | Furis with Extended Volley (+25) and Slip Magazine (+30 %): 71 or 78 |
| OQ-603 | Spool interval after shot k (r_k or r_{k+1}); spool reset on reload or after idle time | r_k; reset on reload | high-speed capture of Gorgon's first 9 shots and of the first shots after a reload |
| OQ-604 | Kohm-family pellet progression and ammo cost per spool step; Kohm and Kuva Kohm fire-rate step | n = 1 + k; cost `max(1, n/3)`; step 0.2 | Kohm magazine counter after each of the first 12 shots (page: 66 shots, 244 rounds vs 243 here) |
| OQ-605 | Ammo-cost rule when cost > 1 and ammo < cost; AmmoCost per shot or per burst (Bubonico) | ceil rule; per shot | Bubonico burst mode: bursts per 27-round magazine (3 or 9); Kohm reload point |
| OQ-606 | Meaning of `BurstReloadDelay`; Tenet Diplos lock-on burst count | reload lead after a magazine burst; BC = 1 | Pandero alt-fire: time from the last burst shot to the reload animation |
| OQ-607 | Sniper combo: counter includes the current shot; decay repeats every D_snp; multiplier on radial children; scoped assumption; Incarnon form inheritance; partial-charge damage interpolation | yes; yes; yes; scoped; inherits; no interpolation | Rubico with Split Chamber: combo shown on the first shot; idle 6 s then shoot (count −1 or −3); Komorex explosion damage at 2× combo; Lanka partial charges at 50 % vs 33 % charge |
| OQ-608 | Battery delay floor for bases below 0.5 s; Shedu reload-speed exception; debt on batteries | floor only limits mod reductions; Shedu unaffected; debt carried | Imperator Vandal unmodded recharge delay; Shedu delay with Fast Hands; Flux Rifle with ammo efficiency |
| OQ-609 | Falloff: projectile-speed scaling of ranges; scalar vs per-component progenitor and flat falloff; progenitor falloff without data range | proportional; scalar Φ_eff; none | Kuva Hek at 25 m with and without Terminal Velocity, per-type numbers |
| OQ-610 | One explosion per launcher projectile; cluster child counts and multishot | one per projectile; data count, unmodded | Ogris with Split Chamber-type mods on one target: number of explosion damage numbers per shot |
| OQ-611 | Lingering effects: first tick time, tick rate, stacking of overlapping clouds on one target | first tick immediate, data FireRate, independent | Torid poison cloud ticks on a dummy with 1 vs 3 overlapping clouds |
| OQ-612 | Incarnon form: Primed Chamber on form shots; switch time when the form ends | none; not counted | Vectis Incarnon form with Primed Chamber: first form shot damage |
| OQ-613 | Bow ×2: crossbows, Artemis Bow, abilities (Wisp, Gauss, Harrow); bows with ammo efficiency (several shots per nock?); uncharged bow cycle = reload | Class Bow only (+ Artemis Bow); ammo debt rule applies; yes | Zhuge Prime and Artemis Bow charge time with Speed Trigger; Paris with an ammo-efficiency buff |
| OQ-614 | Epitaph EFR; Lizzie and Dex Pixia ammo | `1/(CT + 1/FR)` charged, FR uncharged; infinite | Epitaph shot cadence; Lizzie sustained firing over 40 s |
| OQ-615 | Peacemaker fire-rate ramp shape and burst pairing | linear in time; 2 shots per burst | shot timestamps over the first 4 s of Peacemaker |
| OQ-616 | Beam ramp on the AoE child of a held attack | applies | Glaxion Vandal or Ignis Wraith first-tick radial damage |
| OQ-617 | Kitgun loader offsets universally additive; Sporelacer/Tombfinger direct and radial split; Pax Charge recharge rate | additive; single attack; `04` value | assembled kitgun arsenal stats per loader; Sporelacer damage numbers on hit and burst |
| OQ-618 | Projectile flight time model | straight line at ShotSpeed | only matters for d > 0 near the window end; low priority |
| OQ-619 | Proposed options (manager decides, overview 0.3): (a) Incarnon switch time counted at form start and end (`T_sw` = reload time); (b) pellet hit fraction for shotguns (accuracy); (c) sniper scoped on/off | not added | — (user-facing choices) |
| OQ-620 | Proposal to change overview 4.2 steady metrics: `dps_sustained` should include the per-magazine spool-up (ρ from the walked schedule) and the beam ramp loss after reloads (η_rmp); define the sniper combo "steady value" as the window mean of E-6.16a | overview rule kept (spool full, ramp full); sniper window mean used | manager decision; the game facts behind it are OQ-013 and OQ-603 |

---

## 16. Test vectors

Times in seconds. "Count" = shots with fire time ≤ t_win + 1e-9. Computed with python3 (reference scheduler in the
author's scratchpad implementing E-6.8, E-6.10, E-6.11, E-6.15, E-6.16, E-6.17, E-6.18; f64, repeated addition of
intervals). Tolerances for golden tests: times 1e-6 s, ratios 1e-9 relative. All with d = 0, T-02 irrelevant (no
conditional sources) unless stated.

**TV-06-01 Auto (Braton Prime, Speed Trigger).** FR0 9.58, ΣFR 0.60, Mag 75, c 1, R0 2.15, RS 0, AUTO, t_win 20.
FR = 15.328. Shot 75 (index 74) at 4.827766; reloads [4.893006, 7.043006], [11.936013, 14.086013], [18.979019,
21.129019]; first shot of magazine 2 at 7.043006; last shot ≤ 20 at 18.913779; count 225. EFR = 15.328, T_mag =
4.893006, ρ = 0.694733 (matches overview 3.8).

**TV-06-02 Burst (Burston, Speed Trigger; partial burst).** FR0 5, BD0 0.061, BC 3, ΣFR 0.60, R0 2.0.
(a) Mag 45: FR 8, BD 0.038125, EFR = 14.906832 (unmodded 9.316770); first 7 shots 0, 0.038125, 0.07625, 0.20125,
0.239375, 0.2775, 0.4025; reloads [3.01875, 5.01875], [8.0375, 10.0375], [13.05625, 15.05625]; count 180; T_mag
3.01875, ρ = 0.601494.
(b) Magazine Warp ΣMag 0.30: Mag = floor(58.5 + 0.5) = 59 → 19 full bursts + 2 shots; magazine 1 ends with shots at
3.6225, 3.660625, 3.69875, 3.82375, 3.861875 (last, partial burst of 2); reload [3.986875, 5.986875]; count 208; T_mag
3.986875, ρ = 59 / (14.906832 × 5.986875) = 0.661099.

**TV-06-03 Charge (Opticor).** FR0 1, CT0 2, Mag 5, R0 2.
(a) Unmodded: shots 2, 5, 8, 11, 14, 19; reload [15, 17]; count 6.
(b) ΣFR 0.60: CT 1.25, FR 1.6, EFR 0.533333; shots 1.25, 3.125, 5.0, 6.875, 8.75, 12.625, 14.5, 16.375, 18.25; reload
[9.375, 11.375]; count 9; ρ = 0.824176.
Compat (D-076): unmodded 2, 6, 10, 14, 18 (5 shots); modded 1.25, 4.5, 7.75, 11.0, 14.25, 19.5 (6 shots).

**TV-06-04 Bow (Paris, Speed Trigger ×2).** CT0 0.5, R0 0.65, Mag 1, Class Bow, ΣFR from Speed Trigger 0.60 with w = 2.
CT = 0.5 / 2.2 = 0.227273 (rifle weighting would give 0.3125); cycle 0.877273; EFR = 1.139896; shots 0.227273,
1.104545, 1.981818, ...; count 23; ρ = 1. With Arcane Acceleration (+0.90, w = 1) added: CT = 0.5 / 3.1 = 0.161290.
Compat (D-075): shots 0.227273, 1.754545, 3.281818, 4.809091, ...; count 13.

**TV-06-05 Vectis (no reload delay; ammo debt; first-shot flags).** FR0 1.5, Mag 1, R0 1.0, data `ReloadDelay = 0`, SEMI.
(a) AE 0: shots 0, 1, 2, 3, ... (cycle = R0); count 21 (shot at exactly 20.0 included); every shot has `first`; ρ =
1/(1.5 × 1.0) = 0.666667. Compat (D-602): 0, 1.666667, 3.333333, 5.0, ...; count 13.
(b) AE 0.25 (c = 0.75): shots (time, ammo after, first, last): (0, 0.25, no, no), (0.666667, −0.5, yes, yes),
(1.666667, −0.25, yes, yes), (2.666667, 0.0, yes, yes), (3.666667, 0.25, no, no), (4.333333, −0.5, yes, yes), ...;
reload starts 0.666667, 1.666667, 2.666667, 4.333333, 5.333333, 6.333333; 4 shots per 3 magazines (`W:Ammo` L210-215);
count 23.

**TV-06-06 Spool (Gorgon).** FR0 12.5, s_sp 0.2, Δ_sp 0.10667, Mag 90, R0 4.2. First 10 shots 0, 0.4, 0.660867,
0.854412, 1.008255, 1.135912, 1.245, 1.340236, 1.424741, 1.504741 (sum of the first 8 intervals 1.424741); reload
[7.984741, 12.184741]; count 178; EFR 12.5; steady ρ (overview 4.2 "spool full") = 7.2 / (7.2 + 4.2) = 0.631579;
schedule-based ρ including spool-up (proposal OQ-620) = 90 / (12.5 × 12.184741) = 0.590903.

**TV-06-07 Beam ramp (Amprex).** FR0 12, Mag 100, c 0.5 (N = 200), R0 2.6, s_rmp 0.2, HELD. Ramp of the first 10 ticks
0.2, 0.311111, 0.422222, 0.533333, 0.644444, 0.755556, 0.866667, 0.977778, 1, 1; reload [16.666667, 19.266667]; after
it L = 1 − 0.8 × (2.6 − 0.8)/2 = 0.28; magazine-2 ticks 0.28, 0.391111, 0.502222, 0.613333, 0.724444, 0.835556,
0.946667, 1, 1; mean L magazine 1 = 0.983556, magazine 2 (η_rmp of proposal OQ-620) = 0.986467; ticks ≤ 20: 209 with Σ L =
203.004444; ρ = 0.865052.

**TV-06-08 Sniper combo (Rubico).** FR0 2.67, Mag 5, R0 2.4, C_min 1, D_snp 2.0, SEMI. Shots 0, 0.374532, 0.749064,
1.123596, 1.498127, 4.272659, 4.647191, 5.021723, 5.396255, 5.770787, 8.545318, 8.919850; count 25; reload gap 2.774532 ≥
2 gives one decay per reload.
(a) MS 1: counter after each shot 1, 2, 3, 4, 5, 5, 6, 7, 8, 9, 9, 10; M_snp 1.5, 1.5, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0,
2.0, 2.5, 2.5, 2.5.
(b) MS 1.9 (EV, exact pmf): E[c] 1.9, 3.8, 5.7, 7.6, 9.5, 10.4, 12.3, ...; unconditional E[M_snp] 1.5, 1.995, 2.0,
2.0, 2.459270, 2.492075, 2.499912, 2.5, 2.5, 2.5, 2.5, 2.5; pellet-weighted M̄_snp (E-6.16a) of the first 6 shots 1.5,
1.997368, 2.0, 2.0, 2.466176, 2.493802; η_snp over the 25 window shots = 2.605233 (the unweighted shot mean of
E[M_snp] is 2.604149).
(c) Lanka (C_min 2), first shot with MS 1.9: E[M_snp] = 0.1 × 1 + 0.9 × 1.5 = 1.45.

**TV-06-09 Kohmak (spooling pellets, cost > 1).** Fully Spooled FR0 5, s_sp 0.2, Δ_sp 0.32, P_max 5, Mag 40, R0 2.0.
Magazine 1: 25 shots, 0.666667 rounds left (HUD 1), as `W:Kohmak` L41; base pellets of the first 6 shots 1, 2, 3, 4, 5,
5; times 0, 1.0, 1.384615, 1.622711, 1.822711, 2.022711; reload [6.022711, 8.022711] (not empty: 0.667 < 1.667).
Kohm (P_max 12, Mag 245, FR0 3.67, s_sp 0.199809, Δ_sp 0.200048): 66 shots, 243 rounds spent.

**TV-06-10 Battery (Flux Rifle).** FR0 12, c 0.5, Mag 50, `ReloadDelay` 1, `ReloadRate` 40, RS 0.30. δ_rech' = 0.769231,
T_rel = 0.769231 + 1.25 = 2.019231; reload [8.333333, 10.352564] (lead 1/12); count 200; ρ = 0.804954 (`F:weapon` §9.3:
0.805). Magazine 80, RS 0: T_rel 3.0, ρ = 0.816327.

**TV-06-11 Falloff.** (a) B0 100 (all one type), falloff 10-30 m, r_f 0.8, v 0.6 (V = 60), X 20, d 20: Φ = 0.6, Φ_v =
0.5, Φ_eff = 0.611111. (b) Catchmoon secondary chamber falloff 8-16 m, r_f 0.9375, d 12: Φ = 0.53125. (c) Rubico
400-600 m, r_f 0.5, d 500: Φ = 0.75. (d) Same as (a) with ΣPS 0.5 (ranges 15-45 m), d 20: Φ = 0.866667.
(e) Multi-target reference only: Tonkor explosion 650, radius 7 m, r_f 0.7, d_aoe 3.5: 422.5; at the edge 195
(`F:weapon` §10.2).

**TV-06-12 Window boundary and slow fire (D-081).** FR0 0.3, Mag 5, R0 1, SEMI. t_win 20: shots 0, 3.333333, 6.666667,
10.0, 13.333333, 17.666667 (reload [16.666667, 17.666667]); count 6 (inspiration 5). t_win 1: count 1 (inspiration 0).

**TV-06-13 Modded stats.** (a) SEMI FR0 8.0, ΣFR 0.6: FR 12.8, FR_trig 10. (b) FR0 1.0, ΣFR −0.99: FR 0.05. (c) CT0 1.0,
ΣFR −0.95: CT 10.0. (d) Burston ΣFR −0.20: FR 4.0, BD 0.061. (e) Cannonade on FR0 4.17 with ΣFR 0.6: FR 4.17.
(f) Magazine: 45 × 1.3 → 59; 2 × 1.2 → 2 (ceil would give 3, OQ-601); 2 × 0.4 → 1; 1 × 0.4 → 1 (minimum);
35 × 1.3 + 25 flat → 71 (78 if the flat were added before the percent, OQ-602).

**TV-06-14 Peacemaker.** FR 14.8, no other bonuses. Shots 0, 0.27027, 0.489706, 0.680071, 0.850813, 1.007104, ...; 32
shots before 3.5 s, first full-rate shot index 32 at 3.523650; count in 20 s: 276. M_pm: shots 0-1 1.075, shots 2-3
1.15, shot 38 onward 2.5.

**TV-06-15 Incarnon form.** Lex Incarnon Form FR0 0.67, Mag_inc 20, SEMI: shots k/0.67, k = 0..19; count in 20 s = 14;
last form shot at 28.358209; with t_win 60 the schedule ends after 20 shots (D-601). Furis Incarnon Form (HELD, FR0 12,
Mag_inc 280, data AmmoCost 1): last tick at 23.25 s (inspiration: 560 ticks per form magazine, then refill).

**TV-06-16 Kitgun assembly.** Secondary Catchmoon + Gibber + Arcroid: base Heat 119, Impact 41; FR0 3.17; CC0 0.24; CDM0
2.0; SC0 0.24; Mag0 9 (tier Med); R0 1.5; trigger Semi-Auto; falloff 8-16 m to 6.25 %. Primary Tombfinger + Splat +
Tremor: Impact 69, Radiation 47, FR0 2.13, CT0 1.4, CC0 0.38, CDM0 2.3, SC0 0.16, Mag0 23, R0 1.7, trigger Charge
(the pre-expanded record lacks CT0, EC-06-57).
