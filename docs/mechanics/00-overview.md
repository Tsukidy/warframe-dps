Status: draft (spec)

# Damage-calculation spec, part 00: overview

This is the master document of the damage-calculation spec. Every topic document (`01` to `09`) and the Rust
engine read it first. It fixes the notation, the bucket names, the master per-hit formulas, the semantics of the
two evaluators, the policy for disagreements between sources, the user toggles that change mechanics, and the
boundaries between the topic documents. It does not hold every detail: where a rule is only named here, the topic
document listed as its owner holds the full definition, and that document must not contradict this one. If a topic
document needs to change a decision made here, it records the proposal under its own open questions and the
manager changes this file.

Contents:

- [0. Conventions of the spec](#0-conventions-of-the-spec)
- [1. Scope and game baseline](#1-scope-and-game-baseline)
- [2. Notation and glossary](#2-notation-and-glossary)
- [3. Master per-hit formulas](#3-master-per-hit-formulas)
- [4. Evaluation model](#4-evaluation-model)
- [5. Discrepancy policy and register](#5-discrepancy-policy-and-register)
- [6. User-facing toggles that change mechanics](#6-user-facing-toggles-that-change-mechanics)
- [7. Topic map](#7-topic-map)
- [8. Open questions that block exact accuracy](#8-open-questions-that-block-exact-accuracy)

---

## 0. Conventions of the spec

### 0.1 Normative words

MUST, MUST NOT, SHOULD and MAY have their RFC 2119 meaning. "Decision" marks a choice this spec makes where sources
are silent or disagree. `UNVERIFIED` marks a rule that no game source confirms (single unconfirmed editor, inference,
or contradiction); the engine still implements the stated decision, and section 8 lists the test that would settle it.

### 0.2 Citation keys

Every formula carries a citation. Short keys:

| Key | File or source |
|---|---|
| `F:core` | `research/findings/wiki-core-damage-formula.md` |
| `F:status` | `research/findings/wiki-damage-types-and-status-effects.md` |
| `F:addmul` | `research/findings/wiki-additive-vs-multiplicative-edge-cases.md` |
| `F:enemy` | `research/findings/wiki-enemy-defenses-and-scaling.md` |
| `F:weapon` | `research/findings/wiki-weapon-mechanics.md` |
| `F:melee` | `research/findings/wiki-melee-mechanics.md` |
| `F:buffs` | `research/findings/wiki-warframe-side-buffs.md` |
| `F:mods` | `research/findings/wiki-mods-arcanes-rivens.md` |
| `I:feat`, `I:model`, `I:stat`, `I:status`, `I:queue`, `I:agg`, `I:enemy`, `I:opt` | `research/findings/inspiration-{feature-inventory, data-model, stat-pipeline, status-and-procs, queue-model, dps-aggregation, enemy-and-ttk, optimizer}.md` |
| `I:oracle` | `research/findings/oracle-feasibility.md` |
| `DS:wfcd`, `DS:wiki`, `DS:pe` | `research/findings/data-source-{wfcd, wiki-modules, public-export-and-connector}.md` |
| `W:<Page>` | wiki.warframe.com page `<Page>` (raw copy in `research/external/cache/wiki/`) |
| `JS L<a>-<b>` | `research/external/inspiration/min.pretty.js`, lines a-b |

### 0.3 Identifier namespaces

All identifiers defined here are stable. Code and topic documents reference them verbatim.

| Kind | Format | Defined in this file | Range reserved for topic document `0n` |
|---|---|---|---|
| Symbol | `lower_snake_case` (section 2) | yes | new symbols MUST be prefixed by a short domain word (`dot_`, `armor_`, `combo_` ...) and listed in a "Symbols added" table of that document |
| Pipeline stage | `ST-1` ... `ST-10` (section 3.1) | yes | none (stages are fixed here) |
| Equation | `E-1` ... | yes | `E-n.k` (document n, local number k) |
| Cross-cutting decision | `CD-01` ... (section 3.7) | yes | none |
| Discrepancy | `D-001` ... `D-099` | yes | `D-n01` ... `D-n99` (for example `D-301` in `03`) |
| Open question | `OQ-001` ... `OQ-099` | yes | `OQ-n01` ... `OQ-n99` |
| User toggle | `T-01` ... | yes | none: a topic document proposes a new toggle in its open questions; the manager adds it here |

### 0.4 Template every topic document follows

1. `Status: draft (spec)` as the first line.
2. Scope (copied from section 7 of this file) and the stages it owns.
3. Inputs: symbols it consumes, with the owning document.
4. Outputs: symbols it defines (and "Symbols added").
5. Rules and formulas, each with a source citation.
6. Worked examples (arithmetic checked with `python3`).
7. Edge cases.
8. Discrepancies: the `D-0xx` rows of this file it owns, plus new `D-n01`... rows in the same column layout.
9. Open questions (`OQ-n01`...), each with the evidence that would settle it.
10. Test vectors for golden tests (inputs and expected outputs).

---

## 1. Scope and game baseline

### 1.1 Target game version

The spec targets **Warframe Hotfix 44.1.4 / 44.1.4.1 ("Iceblade of Narin: The Icebind")**, both released on
**2026-10-27**. The brief's date 2026-10-28 is the date of the live build label, not of the hotfix. Evidence:

| Source | What it says |
|---|---|
| `W:Update_44:_Iceblade_of_Narin` (cache line 211-219) | "Hotfix 44.1.4 + 4.1", `date=October 27, 2026`; newest entry on the page |
| `DS:pe` §TL;DR, §1.7 | live `buildLabel` `2026.10.28.09.54`; newest patch news "Hotfix 44.1.4" (2026-10-27); Public Export index modified 2026-10-28 |
| `DS:wfcd` header | WFCD warframe-items `v1.1276.51` tracks game version 44.1.4 |
| `F:mods` header | newest patch listed on the wiki: Hotfix 44.1.4.1 (2026-10-27) |
| `I:feat` §1 | the inspiration's changelog runs to 2026-11-20 and cites Hotfix 44.1.3 (2026-10-23) for its last game-data sync |

Research snapshot date: 2026-11-23. Patch-sensitive facts this baseline depends on include: 1/32 damage
quantization (U40), post-U36 faction damage-type table, U36 armor formula and 200/2700 clamp, U40 health-proportional
attenuation, riven disposition fixed at 1.2 and ammo efficiency cap 75 % (Hotfix 44.1.3), Arcane Pistoleer infinite
ammo (44.1.3), Arcane Precision on weak-point hit (U44). When the game baseline moves, every register row and
open question that cites a patch number MUST be re-checked before the baseline line above changes.

### 1.2 In scope

- Damage dealt by the player's weapons to **one target enemy**: primary, secondary, melee, arch-gun, exalted
  weapons (Ability Strength scaling), kitguns, zaws, companion robotic weapons (as theoretical upper bound), Incarnon
  forms.
- Everything that changes that damage: mods, arcanes (weapon and warframe), rivens, Incarnon evolutions, progenitor
  (valence) bonus, set bonuses, archon shards, auras, focus passives, warframe ability buffs and target debuffs that
  the user enables, companion and squad buffs the user enables.
- Enemy side: level scaling, Steel Path, Eximus, faction modifiers, overguard, shields, armor, health, status
  effects, special damage reduction.
- Outputs of section 4 and the build optimizer's scoring metrics (the search algorithm itself belongs to
  `docs/architecture/`).

### 1.3 Out of scope

Ability damage itself (abilities only enter as buffs and debuffs), Conclave/PvP, Railjack and Empyrean damage,
damage the player takes, companion and specter AI damage, operator and amp damage, Necramech weapons beyond what
arch-gun rules cover, server-only features of the inspiration (CLAUDE.md). Multi-target effects (AoE radius on other
enemies, punch-through, melee follow-through, Electricity chains, Gas cloud on others, Blast detonation on others)
are out of the default single-target metric; topic documents record their formulas where sources give them so a
later multi-target mode can use them.

### 1.4 Source precedence

Game behaviour wins over the inspiration's behaviour. In order of precedence:

1. In-game measurements recorded in `research/test-cases/` (by the user or by us).
2. DE patch notes (as quoted on wiki patch-history sections).
3. wiki.warframe.com mechanic pages. When two wiki pages disagree, the per-mechanic page beats a summary table, the
   more recently edited page beats the older one, and a page with a worked numeric example beats prose. The loser is
   recorded in the register.
4. wiki Lua data modules and wiki gadgets (as data, and as executable descriptions of a mechanic).
5. Community test spreadsheets cited by the wiki.
6. The inspiration (`min.pretty.js`): source of **feature parity** (section 6 and `I:feat`) and a **test oracle**
   (section 4.6), never of game truth and never of data (CLAUDE.md).

When game sources disagree with each other or are unverified, the spec picks the best-supported behaviour, marks it
`UNVERIFIED`, records the alternative, and adds an open question.

### 1.5 Data limits that bound the model

The engine can only model what the data pipeline can supply (`docs/data/` owns the pipeline). Known gaps that the
topic documents must design around (`DS:wiki` §10, `DS:wfcd` TL;DR, `DS:pe` TL;DR):

| Gap | Consequence for the spec |
|---|---|
| Mod and arcane effects are prose only in every source | effects are a hand-authored table keyed by mod identity; each entry names its bucket (section 2.4) and condition class (T-02) |
| Incarnon evolution perks: wikitext only | hand-authored perk table (owner `09` for selection, the bucket owner for effects) |
| Archon shard effects: absent from all data | hand-authored (`F:buffs` §5) |
| Progenitor element and value: absent | user input (T-24) |
| Enemy special damage reduction: absent | hand-authored table per enemy family (`05`) |
| Heavy-attack swing time: 6 of 44 shared heavy combos have `Duration` | `07` defines a fallback; `OQ-021` |
| Per-weapon beam ramp start values: no data field | constant 0.2 with a hand-authored override list (`06`) |
| Condition Overload behaviour per attack: only on `W:Condition_Overload_(Mechanic)` | hand-imported catalogue (`04`) |
| Body-part multipliers and head-crit capability: prose (`Multis`) | parsed per enemy; `02` and `05` define defaults |

---

## 2. Notation and glossary

### 2.1 Units and numeric conventions

| Rule | Definition |
|---|---|
| U-1 Fractions | Inside the engine every chance, bonus and percentage is a **fraction**: 0.30 means 30 %. Percent appears only at the UI and data-import boundary. Status and crit chances may exceed 1. |
| U-2 Bonus vs multiplier | A *bonus* is an additive fraction summed inside a bucket (`sum_base_dmg = 1.65` for Serration). A *multiplier* is a factor (`1 + sum_base_dmg = 2.65`). Symbols starting with `sum_` are bonuses; symbols ending with `_mult` are factors. |
| U-3 Units | time in seconds, distance in metres, damage and hit points in HP, rates per second, armor in armor points. |
| U-4 Precision | f64 everywhere except the emulation points NP-1 and NP-2 below, where the game's binary32 arithmetic changes results. |
| NP-1 | Base critical multiplier quantization (E-4) in f32 (`W:Critical_Hit` §Quantization, `F:core` §9). |
| NP-2 | Enemy level-scaling curves and their smoothstep blend in f32, evaluated as `f1 + (f2 − f1)·s` (`F:enemy` §7.1). Riven values in f32 (`F:mods` §8.3) belong to `09`. DoT ticks are f64 (Decision: the f32 tick rule of `W:Damage/Calculation` §DoT changes results by < 1e-6 relative). |
| U-5 Display | Damage numbers shown to the user are rounded with `floor(x + 0.5)`; the engine never rounds internally except where a rule says so (quantization, armor `floor`, magazine rounding). |
| U-6 Randomness | Every random draw uses a counter-based generator keyed by `(seed, run, attack, event index, mechanic id)` (section 4.3). |

### 2.2 Index sets and enumerations

Stable enumeration orders; code uses these orders for arrays.

| Set | Members (ID: name) |
|---|---|
| `DamageType` (`t`) | `IMP` Impact, `PUN` Puncture, `SLA` Slash, `HEA` Heat, `COL` Cold, `ELE` Electricity, `TOX` Toxin, `BLA` Blast, `RAD` Radiation, `GAS` Gas, `MAG` Magnetic, `VIR` Viral, `COR` Corrosive, `VOI` Void, `TAU` Tau, `TRU` True, `CIN` Cinematic, `FIN` Finisher |
| Subsets | `IPS = {IMP, PUN, SLA}`; primary elements `PE = {HEA, COL, ELE, TOX}`; secondary elements `SE = {BLA, RAD, GAS, MAG, VIR, COR}` with BLA = HEA+COL, RAD = HEA+ELE, GAS = HEA+TOX, MAG = COL+ELE, VIR = COL+TOX, COR = ELE+TOX (`F:core` §5.2) |
| `StatusType` (`s`) | `IMP`, `PUN`, `SLA`, `HEA`, `COL`, `ELE`, `TOX`, `BLA`, `RAD`, `GAS`, `MAG`, `VIR`, `COR`, `VOI`, `TAU` (same IDs as their damage type), plus `MIC` Microwave, `KND` Knockdown, `LIF` Lifted. "Frozen" is a state of `COL`, not a separate type (`F:status` §1, §6) |
| DoT statuses | `SLA`, `HEA`, `TOX`, `ELE`, `GAS`; `BLA` deals one delayed hit per stack (`F:status` §3) |
| Layers (`ℓ`) | `OG` overguard, `SH` shield, `HP` health. Armor is not a pool; it is a damage reduction on `HP` (`F:enemy` §1) |
| Layer views (`v`) | `health` (health, armor ignored), `armor` (armored health), `shield`, `overguard` (the inspiration's four result panels, `I:feat` §12), plus `layered` (the real sequence OG → SH → HP, used for TTK) |
| Enemy faction (`f`) | Grineer, Kuva Grineer, Corpus, Corpus Amalgam, Infested, Infested Deimos, Orokin (Corrupted), Sentient, Narmer, Murmur, Zariman, Scaldra, Techrot, Anarchs, Stalker, Tenno, Neutral (`F:enemy` §2) |
| Crit tier (`k`) | 0 = no crit, 1 yellow, 2 orange, 3+ red |
| Indices | `i` shot (trigger pull or beam tick or melee hit), `j` pellet within a shot, `a` attack of a weapon |

### 2.3 Damage-instance vocabulary

| Term | Definition |
|---|---|
| Attack | One entry of a weapon's attack list (wiki `Attacks[]`): "Normal Attack", "Charged Shot", "Incarnon Form", an AoE child attack, a melee "Heavy Slam Attack". Each attack has its own base damage, crit, status, timing. |
| Parent and child attack | A radial child (`ShotType = "AoE"`, `AttackParentIndex`) is created by a parent's hit. They are separate attacks (`F:weapon` §10.1). |
| Shot | One trigger event of an attack: one bullet or projectile launch, one beam tick, one melee stance hit. |
| Pellet | One projectile of a shot; a shot has `floor(ms)` or `floor(ms) + 1` pellets. Beams have one merged instance per tick (CD-10). |
| Damage instance | One application of damage to the target, with its own per-type vector, crit tier and body part: a pellet hit (`direct`), a radial hit (`radial`), a DoT tick (`dot_tick`), an extra hit (`extra_hit`), a special instance (Blast fuse, Magnetic break proc). |
| Weapon side / enemy side | Weapon side: everything that depends only on the build and the hit (stages ST-1 to ST-7). Enemy side: everything that depends on the target and its current state (ST-8, ST-9). |
| Proc | One application of a status effect. A proc of a DoT status creates a DoT instance. |
| Stack | One live proc of a stacking status on the target. |

### 2.4 Symbol table

Every symbol has a stable code identifier (left column), a math symbol used in formulas, its unit, and the topic
document that owns its full definition. All bonuses are summed over every source of that bucket (mods, arcanes,
buffs, shards, rivens, custom stats); which source lands in which bucket is defined by the bucket's owner and
catalogued in `04` for non-mod sources.

**Base damage and composition (owner `01`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `base_listed[t]` | B0_t | HP | listed base damage per pellet of the attack, by type (data) |
| `strength` | Str | factor | caster's Ability Strength (1.0 = 100 %); scales exalted base damage and strength-scaled buffs |
| `progenitor_frac` | v | fraction ∈ [0.25, 0.60] | Kuva/Tenet/Coda valence bonus |
| `progenitor_type` | e_v | DamageType | valence element (IMP, HEA, COL, ELE, TOX, MAG, RAD) |
| `progenitor[t]` | V_t | HP | `v · Σ B0` on `e_v`, 0 elsewhere |
| `incarnon_flat` | X | HP | Incarnon "Increase Base Damage by +X" |
| `incarnon_flat[t]` | X_t | HP | split of X over types (`01`, UNVERIFIED, D-005) |
| `str_base_mult` | m_str | factor | Str for exalted weapons whose base damage scales with Strength, else 1 |
| `base[t]`, `base_total` | B'_t, B' | HP | `B0_t·m_str + V_t + X_t`; `B' = Σ B'_t` (E-1) |
| `base_ref` | B_ref | HP | `Σ (B0_t·m_str + V_t)`: base without the Incarnon flat (CO reference, `04`) |
| `sum_base_dmg` | A | bonus | base-damage ("Serration") bucket |
| `sum_phys[t]` | P_t | bonus | physical mods of type t ∈ IPS |
| `stance_phys[t]` | S^phys_t | bonus | stance physical bonus of the current melee hit (`07`) |
| `sum_elem[t]` | E_t | bonus | elemental bonus of final element t after combination (incl. non-combining additions) |
| `ratio[t]` | r_t | dimensionless | per-type composition ratio before quantization (E-2) |
| `ratio_q[t]` | q_t | dimensionless | r_t quantized to the 1/32 grid (E-3) |
| `sum_faction` | F | bonus | faction bucket (bane mods, Roar, Damage Blessing, riven faction, Paracesis) for the target's faction |

**Crit, body part, status chance, multishot (owner `02`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `cc_base`, `cc_base_add` | CC0, CC0+ | fraction | attack base crit chance; flat additions before mods (Incarnon "+X % base", Piercing Navigator) |
| `sum_cc_rel` | ΣCC_rel | bonus | relative crit chance (Point Strike, Blood Rush and Gladiator via combo, weak-point crit chance on weak-point hits, shards, zoom) |
| `sum_cc_abs` | ΣCC_abs | fraction | absolute crit chance added after mods (Arcane Avenger, Cat's Eye, Covenant, Puncture `punc_cc`, flat-crit toggle) |
| `cc_floor` | CC_min | fraction | lower bound (Smeeta Charm 2.0), 0 by default |
| `cc` | CC | fraction | final crit chance (E-5) |
| `cd_base`, `cd_base_add` | CDM0, CDM0+ | factor | attack base crit multiplier; additions before quantization (Incarnon "+X base") |
| `cd_q` | Q(·) | factor | quantized base crit multiplier (E-4) |
| `sum_cd_rel` | ΣCD_rel | bonus | relative crit damage (Vital Sense, shards) |
| `sum_cd_abs` | ΣCD_abs | factor | absolute crit damage added after mods (Arcane Crepuscular, Tenacious Bond, Cold `cold_cd`) |
| `cd_field_mult` | M_cdf | factor | buffing-field crit damage multiplier (Electric Shield ×2), Decision: multiplies CDM before tiers (UNVERIFIED, OQ-016) |
| `cd` | CDM | factor | final crit multiplier (E-4) |
| `crit_tier` | k | integer ≥ 0 | rolled tier (E-6) |
| `tier_raise_chance` | p_vig | fraction | Vigilante set chance to raise a crit by one tier |
| `headcrit` | h | 1 or 2 | 2 on head-crit-capable parts when the hit is on that part |
| `part_mult` | P_part | factor | enemy body-part multiplier of the part hit (1 on body, 3 on most heads) |
| `head_wp_weight` | k_wp | 1.5 or 1.0 | weight of weak-point damage bonuses on head / non-head weak points |
| `sum_wp_dmg` | ΣWP | bonus | weak-point damage bonuses (Acuity, Seek, riven) |
| `sum_hs_mult` | ΣHS | bonus | headshot-multiplier bonuses (Deadhead, Target Acquired, zoom) |
| `sonar_mult` | M_son | factor | Sonar and similar spot multipliers (not inherited by DoT) |
| `part` | Part | factor | `(P_part + k_wp·ΣWP)·(1 + ΣHS)·M_son` on weak-point hits, 1 on body hits (E-7) |
| `sc_base`, `sc_base_add` | SC0, SC0+ | fraction | base status chance; Incarnon flat additions before mods |
| `sum_sc_rel`, `sum_sc_abs` | ΣSC_rel, ΣSC_abs | bonus, fraction | relative and absolute status chance |
| `sc_mult` | M_sc | factor | status-chance multipliers (Tau `1 + 0.1·n_TAU`, custom "vulnerable to status chance") |
| `sc` | SC | fraction per pellet | final status chance (E-8) |
| `proc_weight[s]` | w_s | fraction | proc-type probability (E-9) |
| `forced_procs[s]` | φ_s | count per pellet | forced procs (weapon traits, stance hits via `07`) |
| `p_hunter`, `p_ib` | p_HM, p_IB | fraction | Hunter Munitions 0.30 per crit instance; Internal Bleeding 0.35 (0.70 when fire rate < 2.5) per Impact proc |
| `ms_base`, `sum_ms`, `ms` | MS0, ΣMS, MS | factor | multishot (E-10) |

**Status effects (owner `03`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `sum_status_dmg` | A_sd | bonus | status-damage bucket (Elementalist, riven, Galvanized Elementalist) |
| `status_dmg_type[s]` | A_sd,s | bonus | per-status additions in the same bucket (Emerald shard Toxin, Conductive Sphere Electricity) |
| `sum_status_dur` | SD | bonus | status duration |
| `dot_coeff[s]` | K_s | factor | 0.35 SLA; 0.5 HEA, TOX, ELE, GAS; 0.3 BLA per stack |
| `dot_elem[s]` | E^dot_s | bonus | elemental bonus scaling the DoT of s (`03`: all mods of that element, combined or not, for PE; only literal Gas mods for GAS; none for SLA, BLA) |
| `dur_base[s]`, `dur[s]` | D0_s, D_s | s | base and modded duration |
| `dot_ticks[s]` | N_s | count | ticks per proc |
| `dot_delay[s]` | δ_s | s | time of the first tick after the proc (1 for SLA, HEA, TOX; 0 for ELE, GAS) |
| `stack_cap[s]` | cap_s | count | per-type cap after enemy overrides |
| `stacks[s]` | n_s | count | live stacks on the target |
| `viral_mult(n)`, `mag_mult(n)` | Vir(n), Mag(n) | factor | `2 + 0.25(n − 1)` for n ≥ 1, else 1 |
| `corr_strip(n)` | c_corr(n) | fraction | `min(1, 0.20 + 0.06 n)` for n ≥ 1, else 0 |
| `heat_strip(τ)` | c_heat(τ) | fraction | Heat armor strip as a function of time since ignition (ramp) |
| `cold_cd(n)`, `punc_cc(n)` | – | factor, fraction | `0.10 + 0.05(n − 1)` (n ≤ 9), 1.0 when Frozen; `0.05 n` (n ≤ 5) |
| `n_distinct` | n | count | distinct status types on the target that count for CO-type bonuses |

**Conditional bonuses and buffs (owner `04`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `sum_co` | C | bonus | CO-type bucket: Σ (per-status bonus × n) + per-stack terms (Secondary Shiver per Cold stack) + conditional terms |
| `co_behavior` | – | enum | `adding`, `multiplying`, `none` for this attack (catalogue) |
| `co_ratio` | ρ_co | factor | catalogue ratio `B_co / B_ref` (1.0 by default; 0.5 bows at full charge; > 1 child projectiles) |
| `mult_final_omitted` | G_F | factor | product of final multipliers the Adding-CO recalculation omits (Eclipse, Garuda passive, Parasitic Link, Overdriver, Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, Primary Compression on projectiles, Extinguished Dragon Key, range falloff, bow charge scaling) |
| `mult_other` | G | factor | product of every other independent weapon-side multiplier (stance hit, heavy and combo, sniper combo, Primed/Charged Chamber, Synth Charge, beam ramp, Vauban passive, Navigator, Mecha Empowered, Melee Careen ...) |
| `extra_hit_frac` | p_xh | fraction | extra-hit fraction (Xata 0.26 × Str, Toxic Lash 0.30 × Str, ×2 melee) |
| `stealth_bonus` | SDB | factor | `1 + 0.2 · min(rank, 30)` (7 at rank 30) |
| `vuln` | V | factor | target damage vulnerability `Π(1 + V_mult)·(1 + ΣV_add)` (catalogue in `04`, stage in `05`) |

**Enemies (owner `05`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `level`, `base_level`, `level_q` | L, L0, q | level | `q = L − L0` for health, shield, armor; overguard uses `L − 1` (D-042) |
| `steel_path`, `eximus` | – | bool | modifiers of section 3 of `05` |
| `pool[ℓ]` | OG, SH, HP | HP | scaled pools |
| `armor_init` | A0 | armor | scaled armor, `floor`ed, clamped to [200, 2700] when base armor > 0 |
| `armor_eff` | A_eff | armor | A0 after strips (E-15) |
| `armor_dr(A)` | DR(A) | fraction | `0.9·sqrt(A/2700)` for A ≤ 2700, else `A/(A + 300)` |
| `type_mod[f][t]` | φ_t | factor | faction damage-type modifier (1.5, 1, 0.5); on OG only Void 1.5 |
| `innate_dr[ℓ]`, `status_dr` | – | fraction | enemy flat damage reduction per layer and on status damage |
| `attenuation` | Att(·) | function | per-instance damage attenuation (bosses, `05`) |

**Firing model (owner `06`) and melee (owner `07`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `fire_rate`, `eff_fire_rate` | FR, EFR | 1/s | modded fire rate; achieved attacks per second |
| `charge_time`, `burst_count`, `burst_delay` | CT, BC, BD | s, count, s | charge and burst parameters |
| `magazine`, `ammo_cost`, `ammo_eff` | Mag, cost, AE | rounds, rounds, fraction | magazine, ammo per shot, ammo efficiency (≤ 0.75) |
| `shots_per_mag` | N_mag | shots | `Mag / (cost·(1 − AE))` (average) |
| `t_mag`, `t_reload` | T_mag, T_rel | s | time to empty a magazine; reload time |
| `duty` | ρ | fraction | `T_mag / (T_mag + T_rel)` (class exceptions in `06`) |
| `distance` | d | m | target distance (T-27); `falloff(d)` = Φ(d) |
| `sniper_combo_mult` | M_snp | factor | sniper combo multiplier |
| `attack_speed` | AS | factor | modded melee attack speed |
| `combo_count`, `combo_mult` | c, M(c) | count, factor | melee combo counter and multiplier `clamp(floor(c/20) + 1, 1, 12)` (weapon exceptions in `07`) |
| `stance_hit_mult` | S_j | factor | stance multiplier of hit j of a combo |

**Evaluation (owner `08`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `t_win` | T_win | s | fight window, default 20 (T-01) |
| `t_ttk_max` | T_ttk | s | TTK horizon, default 60 |
| `proc_rate[s]` | λ_s | 1/s | expected procs of s per second |
| `stack_mean[s]` | μ_s | count | mean of live-proc count before capping (λ_s × D_s in steady state) |
| `n_runs`, `seed` | – | count, u64 | Monte Carlo run count (default 1000) and seed |

---

## 3. Master per-hit formulas

### 3.1 Stage pipeline

Every damage instance passes through these stages in this order. Commutative multiplications may be implemented
in any order; non-commutative steps (quantization, the additive CO term, additive stealth term, armor floor of 1,
attenuation) MUST keep the stated position.

| Stage | Name | Owner | Produces |
|---|---|---|---|
| ST-1 | Base vector | `01` (inputs from `06` kitgun, `07` zaw/stance) | `base[t]`, `base_total`, `base_ref` |
| ST-2 | Composition | `01` (stance physical from `07`, Phoenix Talons value from `04`) | `ratio[t]` |
| ST-3 | Quantization | `01` | `ratio_q[t]` |
| ST-4 | Base scalar (base bucket, CO term, omitted final multipliers) | `01` (A), `04` (C, behaviour, G_F), `06` (falloff inside G_F) | scalar Y (E-11) |
| ST-5 | Weapon-side independent multipliers | `01` (F), `04`, `06`, `07` (G) | `(1 + F)·G` |
| ST-6 | Body part | `02` (data from `05`) | `part` |
| ST-7 | Crit | `02` (Cold, Puncture values from `03`) | crit factor `c_k` |
| ST-8 | Enemy side per type | `05` (status multipliers from `03`, vulnerabilities from `04`) | damage per type on a layer |
| ST-9 | Layer routing and depletion | `05` (TTK use in `08`) | pool changes |
| ST-10 | Status application | `02` (rolls), `03` (effects) | procs, stacks, DoT instances |

### 3.1a Buckets

Inside a bucket, entries add; buckets multiply each other unless a formula below says otherwise ("All sources are
multiplicative to each other and to base damage bonuses, unless stated otherwise", `W:Calculating_Bonuses`). The
member lists are the main members only; the owner holds the complete list.

| Bucket (symbol) | Stacking | Main members | Enters at | Owner | Source |
|---|---|---|---|---|---|
| Base damage `sum_base_dmg` (A) | additive, negatives allowed | Serration family, Heavy Caliber, Pressure Point, riven Damage, Cannonade, Vex Armor, Octavia Amp, Deathbringer, Shooting Gallery, Metamorphosis, Rift Torrent, Lethal Levitation, Void Strike, damage arcanes (Fury, Rage, Precision, Merciless, Deadhead, Dexterity, Rise, Awakening, Chargers, Arachne on guns), damage auras, Holster Amp, Vigorous Swap, heavy-only melee damage on heavies, Valkyr Rage, Excalibur passive, Peacemaker innate | ST-4 | `01` | `F:core` §2, `F:addmul` §0, §4-§7 |
| Physical `sum_phys[t]` (P_t) | additive per type | IPS mods, riven IPS | ST-2 (stance physical is a separate factor) | `01` | `F:core` §4, `F:melee` §2.1 |
| Elemental `sum_elem[t]` (E_t) | additive per final element | elemental mods, riven elements and spliced combined elements, Violet shard, Nourish, field elements, Valence Formation, Melee Exposure, Crucible Overflow, Unairu Initiative, weakness-element mods | ST-2 | `01` | `F:core` §5, `F:buffs` §1.22, §5, §8 |
| CO family `sum_co` (C) | additive with each other | Condition Overload, Galvanized Aptitude/Savvy/Shot, Secondary Shiver, Shattering Frost, Cedo, Incarnon per-status and low-health perks | ST-4: added term (`adding`) or factor `(1 + C)` (`multiplying`) | `04` | `F:addmul` §1 |
| CO-omitted final multipliers `mult_final_omitted` (G_F) | each a separate factor | Eclipse, Garuda passive, Parasitic Link, Overdriver, Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, Primary Compression (projectile), Extinguished Dragon Key, range falloff, bow charge scaling | ST-4 | `04` (falloff `06`) | `F:addmul` §1.6 |
| Faction `sum_faction` (F) | additive | bane mods, Roar, Damage Blessing, riven faction, Paracesis Sentient bonus | ST-5; again on DoT ticks and extra hits | `01` | `F:core` §6, `F:addmul` §3 |
| Other multipliers `mult_other` (G) | each a separate factor | stance hit multiplier, heavy-attack type and combo multipliers, ability combo, sniper combo, Primed/Charged Chamber (additive with each other), Synth Charge, beam ramp, Vauban passive, Navigator, Mecha Empowered, Melee Careen, Slam Damage mods, finisher bucket, Braton Incarnon multishot-pellet damage | ST-5 | `04` (`06`, `07` for timing-driven ones) | `F:addmul` §0, §12, `F:weapon` §16 |
| Melee projectiles: universal weapon damage | additive; a second base bucket multiplied with A | Arcane Arachne, Vigorous Swap, Holster Amp, Toxin Ward, Power Throw; Vex Armor, Deathbringer, Amp, Rift Torrent count in both buckets | ST-4 (exception for glaive throws and explosions, Exodia Contagion) | `04` | `F:addmul` §12 |
| Body part (P_part, ΣWP, ΣHS, M_son) | ΣWP additive, ΣHS additive, factors multiply | Acuity, Seek, riven WP; Deadhead, Target Acquired, zoom; Sonar | ST-6 | `02` | `F:core` §10 |
| Crit chance (ΣCC_rel, ΣCC_abs, CC_min) | additive within each | E-5 | ST-7 | `02` | `F:core` §8 |
| Crit damage (ΣCD_rel, ΣCD_abs, M_cdf) | additive within each | E-4 | ST-7 | `02` | `F:core` §9 |
| Multishot (ΣMS) | additive | Split Chamber family, Galvanized multishot, Metronome Opera, Primary Frostbite | E-10 | `02` | `F:core` §7 |
| Status chance (ΣSC_rel, ΣSC_abs, M_sc) | additive within each; M_sc multiplies | status mods, Weeping Wounds via combo, shards; Tau | E-8 | `02` | `F:core` §13 |
| Status damage (A_sd, A_sd,s) | additive | Elementalist mods, riven Status Damage, Galvanized Elementalist, Emerald shard (Toxin), Conductive Sphere (Electricity) | E-13 only | `03` | `F:status` §3.1 |
| Status duration (SD) | additive | Lasting Sting family, Hunter Track, Saryn passive | E-13, CD-06 | `03` | `F:status` §2.3 |
| Enemy side (φ, Vir, Mag, V, DR, innate DR, Att) | φ, Vir, Mag, V and DR multiply; V has an additive subgroup | faction type modifiers; Viral; Magnetic; Molecular Prime and other vulnerabilities; armor; innate DR; attenuation | ST-8 | `05` (vulnerability sources `04`) | `F:enemy`, `F:buffs` §2 |

### 3.2 Weapon-side stats (E-1 to E-10)

```text
E-1  Base vector (ST-1)
     B'_t = B0_t·m_str + V_t + X_t            m_str = Str for exalted weapons whose base scales with Strength, else 1 (01)
     V_t  = v·Σ B0  if t = e_v, else 0         B' = Σ_t B'_t ;  B_ref = Σ_t (B0_t·m_str + V_t)
E-2  Composition (ST-2)
     r_t = (B'_t / B')·(1 + P_t)·(1 + S^phys_t)          t ∈ IPS
     r_t = B'_t / B' + E_t                               t elemental (innate value of t joins; combined elements per 01)
     r_t = B'_t / B'                                     t ∈ {VOI, TAU, TRU, FIN}
E-3  Quantization (ST-3), per type, once per final type
     q_t = sign(r_t)·floor(|r_t|·32 + 0.5) / 32
E-4  Crit multiplier
     code = floor(clamp(CDM0 + CDM0+, 0, 32)·4095/32 + 0.5);  Q = code·32/4095   (f32, NP-1)
     CDM  = (Q·(1 + ΣCD_rel) + ΣCD_abs)·M_cdf
E-5  Crit chance
     CC   = max(CC_min, (CC0 + CC0+)·(1 + ΣCC_rel) + ΣCC_abs)
E-6  Crit tier
     k = floor(CC) + Bernoulli(frac(CC));  if k ≥ 1: k += Bernoulli(p_vig)
     E[k] = CC + p_vig·min(1, CC)
E-7  Body part
     Part = (P_part + k_wp·ΣWP)·(1 + ΣHS)·M_son   on a weak-point hit (k_wp = 1.5 head, 1.0 other weak point)
     Part = 1                                     on a body hit; 1 for radial damage
E-8  Status chance (per pellet)
     SC = ((SC0 + SC0+)·(1 + ΣSC_rel) + ΣSC_abs)·M_sc
E-9  Proc count and type (per pellet)
     procs = floor(SC) + Bernoulli(frac(SC)) + Σ_s φ_s          (UNVERIFIED sampling rule; E[procs] is sourced)
     w_s   = q_s·[s not immune] / Σ_u q_u·[u not immune]
E-10 Multishot
     MS = max(1, MS0·(1 + ΣMS));  Acuity mods: MS = MS0;  pellets = floor(MS) + Bernoulli(frac(MS))
```

Sources: E-1 `F:core` §1, §3.1, §3.2, `F:weapon` §14; E-2 `F:core` §4, §5, `F:melee` §2.1 (stance physical
multiplicative); E-3 `F:core` §14, `W:Damage/Calculation` §Quantization, `W:Damage` patch history (1/32 since
U40); E-4 `F:core` §9, `W:Critical_Hit` §Quantization and §Critical Damage, Cold placement `F:status` §4.5,
`M_cdf` `F:buffs` §1.12 (placement UNVERIFIED); E-5 `F:core` §8, Smeeta floor `F:buffs` §8; E-6 `F:core` §9,
Vigilante `F:mods` §7.2; E-7 `F:core` §10, `W:Enemy_Body_Parts`; E-8 `F:core` §13, Tau `F:status` §4.15; E-9
`F:status` §2.1, §2.2 (quantized weights: Decision, D-024); E-10 `F:core` §7, `W:Multishot` (floor at 1: D-019).

### 3.3 The base scalar Y and the Condition-Overload family (ST-4)

The CO family (Condition Overload, Galvanized Aptitude/Savvy/Shot, Secondary Shiver, Cedo, Incarnon per-status
perks, Shattering Frost) sums into `C` (`F:addmul` §1.1). Its behaviour is a property of each **attack**, taken from
the catalogue owned by `04` (`W:Condition_Overload_(Mechanic)` §Attack Catalog; default `adding` with
`ρ_co = 1`). The scalar Y that multiplies the quantized composition is:

```text
E-11 Base scalar
  none        : Y = B'·(1 + A)·G_F
  multiplying : Y = B'·(1 + A)·(1 + C)·G_F
  adding      : Y = B'·(1 + A)·G_F  +  ρ_co·B_ref·C
```

- `none` applies to radial damage (except the directly hit target of a hitscan explosion, which is `adding` with
  its catalogue ratio), slams, attacks the catalogue lists as "does not apply", and whenever the conditions toggle is
  `none` (T-02).
- In `adding`, the CO term is not multiplied by `G_F` and uses `B_ref` (no Incarnon flat). Everything after ST-4
  (faction, G, body part, crit, enemy side) multiplies Y as a whole, including the CO term. That last point is the
  wiki's framing of every example but is stated on no page: Decision, UNVERIFIED, `OQ-001`.
- `multiplying` uses `B'` including the Incarnon flat (Kunai Incarnon evidence), and `G_F` multiplies the whole.
- C is a scalar. It enters after quantization, so it does not change `q_t`: Decision, `OQ-002`.
- n (the distinct-status count inside C) counts statuses present on the target **before** this damage instance;
  statuses applied by earlier pellets of the same shot count, the instance's own procs do not (`F:addmul` §1.2).
  How n is obtained is T-05; its expectation in the EV evaluator is in section 4.2.
- Melee stance inputs carry a per-input CO effectiveness ratio (`F:addmul` §1.10); `07` owns the data, `04` the
  formula `Y = B'·S·(1 + A)·G_F + r_in·S·ρ_co·B_ref·C` with S applied once (UNVERIFIED whether per hit or per input).

Sources: `F:addmul` §1.4-§1.7, `F:core` §11, `W:Condition_Overload_(Mechanic)` §Math ("confirmed as of 2025-11-03").

### 3.4 Direct hit (E-12)

For one pellet (or merged beam tick, melee hit, radial hit on the target) of attack a, crit tier k, on layer ℓ:

```text
E-12 Direct damage instance
  Weapon side (ST-1 ... ST-7), per type t:
    W_t(k) = q_t · Y · (1 + F) · G · Part · c_k
    c_k    = 1 + k·(h·CDM − 1)                         (no stealth)
    c_k    = 1 + SDB + k·(h·CDM − 1)                   (melee stealth hit, 04)
    beams: W_t is multiplied by the merged beam count (CD-10)
  Enemy side (ST-8), per type t, on layer ℓ, with target state σ just before the instance:
    OG : D_t = W_t · [t = VOI ? 1.5 : 1] · Mag(n_MAG) · V_OG · M_OG
    SH : D_t = W_t · φ_t · Mag(n_MAG) · V_SH                        (TOX routed to HP, 05)
    HP : D_t = W_t · φ_t · Vir(n_VIR) · V_HP · arm_t
         arm_t = 1 − DR(A_eff)  for armor-reduced types when A_eff > 0, else 1   (TRU, FIN, CIN bypass)
         after armor, every type with D_t > 0 is raised to at least 1      (armor floor; not after other DR)
    all layers: D_t ← D_t · (1 − innate_dr[ℓ]);  then D = Att(Σ_t D_t) for enemies with attenuation (05)
  Layer routing (ST-9): OG absorbs everything first; then SH absorbs all types except TOX (bypass exceptions in 05);
    HP takes TOX (while SH is up) and everything once SH is gone. Overflow at a layer break: CD-08.
```

Sources: `F:core` §15, `F:enemy` §1-§6, §9, `F:status` §4.11, §4.13, `F:buffs` §2 (vulnerability, owner `05`),
`F:addmul` §9 (stealth adds to the crit term), `W:Stealth` §Stealth Damage Bonus (formula lines 37-49).

`M_OG` is the overguard-only weapon multiplier (Secondary Fortifier ×8, `F:buffs` §7). `G` contains every weapon-side
multiplier not in `G_F`; `04` owns the membership list and the rule that an unlisted multiplier defaults to `G`.

```text
E-15 Effective armor (05)
  A0    = clamp(floor(A_base · M_armor(q)), 200, 2700) if A_base > 0, else 0
  A_eff = A0 · max(0, 1 − Σ ability_strips − Σ CP·aura_strength − T-07) · (1 − c_heat(τ)) · (1 − c_corr(n_COR))
          (strips act on the clamped value and may take it below 200)
```

Sources: `F:enemy` §3.2-§3.3, `W:Armor` (U32 total-armor strips, U36 clamp), `W:Corrosive_Projection`,
`W:Damage/Heat_Damage`, `W:Damage/Corrosive_Damage`.

### 3.5 Status DoT tick (E-13)

A DoT instance is created by a proc of s ∈ {SLA, HEA, TOX, ELE, GAS} (and a BLA stack) on an inducing instance
with crit tier k and body part Part. Weapon-side terms are snapshotted at proc creation; target-side terms are
evaluated when each tick lands (CD-05).

```text
E-13 DoT tick
  seed      = Y · (1 + F) · G                          (Y with the inducing attack's CO behaviour; not quantized)
  tick_s    = (seed + 1) · K_s · (1 + E^dot_s) · (1 + F) · (1 + A_sd + A_sd,s) · c_k · Part_dot · M_dot,s
  Part_dot  = Part / M_son                             (Sonar-type spots are not inherited)
  M_dot,s   = status-specific multipliers (second dip of Mecha Empowered and of Vauban passive when applicable,
              Magnetic-break proc rules) owned by 03/04
  Enemy side at tick time τ:
    SLA (Cinematic): no φ, no armor;  HP: × Vir(n_VIR(τ)) · V_HP;  SH: × Mag(n_MAG(τ)) · V_SH;  OG: × Mag · V_OG
    HEA, TOX, ELE, GAS, BLA: as E-12 with type s (TOX ticks bypass SH, not OG)
  Ticks: N_s = floor(6·(1 + SD)) at τ = t_proc + δ_s + m, m = 0 .. N_s − 1          (CD-06)
  Beams: the seed of a merged tick is multiplied by the merged beam count, so DoT scales twice with multishot (CD-10)
  HEA: one shared timer per target; all live Heat procs are summed into one tick per second, "+1" added once (03)
  BLA: no "+1"; each stack deals 0.3·H at fuse expiry 1.5·(1 + SD) s, or at detonation (10 stacks, 4 on bosses) (03)
```

Sources: `F:status` §3.1-§3.4 (tick formula, inheritance list, "+1" accumulator UNVERIFIED single editor,
`W:Damage/Calculation` §DoT), §4.4 (Heat), §4.8 (Blast), `W:Damage_over_Time` §DoT Damage Scaling, faction double
dip `W:Faction_Damage_Bonus`. Worked check: `W:Damage/Slash_Damage` example `0.35 × 344.5 × 1.3 × 1.9 = 297.82`
is E-13 without the "+1" (`F:status` §3.2).

### 3.6 Extra hit (E-14)

Xata's Whisper, Toxic Lash, Resupply, Silken Stride, Demonium, Sangodae, Gastroparesis (`F:addmul` §13, `F:buffs`
§1.3, §1.4, §1.22). One extra hit per pellet of a parent direct instance; none if the parent kills the target; no
combo or sniper-combo gain; does not change the parent's elements or proc weights.

```text
E-14 Extra hit
  H_w   = Σ_t W_t(k)                                  (parent weapon-side damage, after quantization, crit, Part)
  XH    = H_w · p_xh · (1 + F) · (Xata ? Part : 1)    one damage instance of type e_xh (VOI for Xata, TOX for Toxic Lash)
  Enemy side: E-12 with the single type e_xh (Xata: Void φ, armor applies)
  Status: Xata rolls VOI with the weapon's SC; Toxic Lash and the others force one proc of e_xh
  Forced DoT of an extra hit:  tick = XH · K_e · (1 + E^dot_all,e) · (1 + F) · (1 + A_sd + A_sd,e)    (no "+1", UNVERIFIED)
    E^dot_all,e = every mod of element e on the weapon, including mods combined into a secondary element
  Melee Duplicate is not an extra hit of this form: it is a full copy of the hit with its own rolls (04)
```

Faction therefore applies twice on extra hits and three times on their DoT; Xata also applies the body part twice.
Sources: `W:Extra_Hit` §Damage Calculation, `W:Xata's_Whisper`, `W:Toxic_Lash` (via `F:addmul` §13, `F:buffs`
§1.3-§1.4).

### 3.7 Cross-cutting decisions

| ID | Decision | Source |
|---|---|---|
| CD-01 | Fractions inside the engine (U-1). | brief |
| CD-02 | Quantization: only the composition ratios `r_t` are quantized, on the 1/32 grid, with the modded base as scale. Base bucket, CO, faction, G, G_F, body part, crit and all enemy-side terms are outside. DoT seeds and extra hits are not quantized again. | `F:core` §14 ("Get Scale → Quantize → Add → Apply all other multipliers"; the page's UpdateMe conflict yields identical numbers for uniform multipliers) |
| CD-03 | Each status roll and each crit roll is independent; crit and status of one pellet are independent. | `F:core` §7, §13 |
| CD-04 | Order inside one damage instance: (1) damage is computed from the target state just before it; (2) crit and procs are rolled; (3) damage is applied; (4) procs are applied. Pellets of one shot are sequential for the CO count n, but do **not** see each other's debuffs (Viral, Magnetic, Corrosive, Heat strip, Cold, Puncture). Shots of an instant burst are separate shots. Electricity and Gas tick 0 lands after step (4), so it sees a Viral proc of the same hit. | `F:status` §4.13 (Viral timing, Simulacrum-tested), `F:addmul` §1.2; extension to the other debuffs is UNVERIFIED (`OQ-028`) |
| CD-05 | Weapon-side terms of a DoT are snapshotted at proc creation; target-side terms (Viral, Magnetic, armor, type modifier, vulnerability) are evaluated at each tick. | `F:status` §3.1, `W:Damage/Viral_Damage` |
| CD-06 | DoT tick count `floor(6·(1 + SD))`; debuff durations `D0·(1 + SD)` without rounding; Heat strip interval `0.5·(1 + SD)`; Blast fuse `1.5·(1 + SD)`; negative SD rules in `03`. | `W:Lasting_Sting` §Notes; others UNVERIFIED (`OQ-005`) |
| CD-07 | Capped statuses (COL, COR, GAS, MAG, RAD, VIR, TAU, IMP, PUN) use independent timers; a new stack beyond the cap replaces the oldest. Enemy-specific caps (bosses and status-capped enemies 4; overguarded enemies 4 COL; VIP overguard 3 PUN) apply to every status, DoT instances included. | `F:status` §1, §2.4; DoT part UNVERIFIED (`OQ-008`) |
| CD-08 | Layer order OG → SH → HP. Overflow of the instance that breaks OG is discarded. Overflow of the instance that breaks SH reaches HP at 5 % during the 0.1 s gate, at 100 % for weak-point hits; status ticks are not gated. | `F:enemy` §4.3, §5; overflow rules UNVERIFIED (`OQ-011`) |
| CD-09 | Attenuation and flat enemy DR apply per damage instance after every modifier. | `F:enemy` §9.1 (U34 note "Modifiers now apply before Damage Attenuation") |
| CD-10 | Non-beam: every pellet is its own damage instance. Beams: the beams of one tick that hit the target merge into one instance with summed damage and summed SC and a single crit roll; forced procs apply once per merged tick. | `F:core` §7, `W:Multishot` §Continuous Weapons |
| CD-11 | Single target by default (section 1.3). | brief |
| CD-12 | Each attack is evaluated on its own (parity, `I:agg` §7.5). Parent and child attacks that hit the same target are also reported summed (section 4.5). "Combine attack effects" groups share one status timeline merged by time. | `I:feat` §10; `F:weapon` §10.1 |
| CD-13 | Expected values: crit, multishot and proc counts enter linearly; target-state functions use the stack-count distribution (section 4.2), never `f(E[n])`. | section 4.2 |

### 3.8 Worked example (checked with python3)

Arithmetic was checked with `python3` (numpy float32 for NP-1 and NP-2); the throwaway script is not part of the
repository. Values are rounded for display; the engine keeps full precision.

**Inputs.**

| Item | Value | Source |
|---|---|---|
| Weapon | Braton Prime, "Normal Attack": IMP 1.75, PUN 12.25, SLA 21 (B' = 35); CC0 0.12; CDM0 2.0; SC0 0.26; FR 9.58; MS0 1; Mag 75; Reload 2.15 s; Auto; Hit-Scan | wiki `Module:Weapons/data/primary` (cached) |
| Mods (max rank) | Serration +1.65 base; Split Chamber +0.90 MS; Point Strike +1.50 CC; Vital Sense +1.20 CD; Infected Clip +0.90 TOX (slot 5); Stormbringer +0.90 ELE (slot 6) → COR 1.80; Primed Bane of Grineer +0.55 faction; Speed Trigger +0.60 FR | wiki `Module:Mods/data` (cached) |
| Target | Lancer (Grineer): base health 100, base armor 100, BaseLevel 1, head 3.0×; level 100; Steel Path on; no shield, no overguard | wiki `Module:Enemies/data/grineer` (cached) |
| Options | body hits; no buffs except Xata's Whisper 0.26 in step 7; conditions irrelevant (no conditional mods) | |

**Step 1, enemy (05).** q = 100 − 1 = 99 ≥ 80, so s = 1.
Grineer health: f1 = 1 + 0.015·99^2.12 = 256.17474, f2 = 1 + 10.7332·99^0.72 = 294.4855; f32 blend gives
294.4855. Steel Path: 100 × 2.5 = 250 base → **HP = 73,621.375**.
Armor: f2 = 1 + 0.4·99^0.75 = 13.554123; floor(100 × 13.554123) = 1355; clamp [200, 2700] → **A0 = 1355**
(Steel Path does not scale armor). DR = 0.9·sqrt(1355/2700) = 0.637574, so `1 − DR = 0.362426`.
(`F:enemy` §7.1-§7.4, §8.1, §3.1.)

**Step 2, weapon stats (02, 06).** CC = 0.12 × 2.50 = **0.30**. CDM: code = floor(2.0 × 4095/32 + 0.5) = 256,
Q = 256 × 32/4095 = 2.0004885 (f32); CDM = 2.0004885 × 2.20 = **4.4010749**. MS = **1.90** (1 pellet with
probability 0.1, 2 with 0.9). SC = **0.26** per pellet. FR = 9.58 × 1.60 = **15.328 /s**; T_mag = 75/15.328 =
4.8930 s; T_rel = 2.15 s; ρ = **0.694733**.

**Step 3, composition and quantization (01).** r = IMP 0.05, PUN 0.35, SLA 0.60, COR 1.80 (Infected Clip and
Stormbringer combine). q = 2/32, 11/32, 19/32, 58/32 = 0.0625, 0.34375, 0.59375, 1.8125; Σq = 2.8125 (Σr = 2.80,
quantization +0.45 %).

**Step 4, scalar and weapon side (01).** No CO: Y = 35 × 2.65 = 92.75. Per type before faction: 5.796875,
31.882813, 55.070313, 168.109375 (Σ 260.859375). × (1 + F) = 1.55 → W(0) = **8.985156, 49.418359, 85.358984,
260.569531; Σ H_w = 404.332031** (G = Part = 1, non-crit).

**Step 5, enemy side on armored health (05), fresh target (state A).** φ(Grineer) = IMP 1.5, COR 1.5, others 1;
× (1 − DR) = 0.362426:

| Type | non-crit (k = 0) | yellow crit (k = 1, × 4.4010749) |
|---|---|---|
| IMP | 4.884688 | 21.497877 |
| PUN | 17.910522 | 78.825548 |
| SLA | 30.936356 | 136.153219 |
| COR | 141.655945 | 623.438422 |
| Σ | **195.387510** | **859.915065** |

E[c] = 1 + 0.30 × 3.4010749 = 2.0203225 → E[damage per pellet] = **394.7458**, per shot × 1.9 = **750.0170**.

**Step 6, Slash DoT from one pellet (03).** seed = 35 × 2.65 × 1.55 = 143.7625; tick = (143.7625 + 1) × 0.35 ×
1.55 = **78.533656** non-crit (77.991156 without the "+1"), **345.632502** at k = 1. Cinematic: no φ, no armor,
lands on health as is. Six ticks at t + 1 ... t + 6 s.

**Step 7, Xata's Whisper 0.26 on the same pellet (04).** XH = 404.332031 × c_k × 0.26 × 1.55 = **162.945809**
(k = 0), **717.136706** (k = 1). Void, φ = 1, armor applies: **59.055875** and **259.909328** on armored health.

**Step 8, proc rates (02).** w = q/Σq: IMP 0.022222, PUN 0.122222, SLA 0.211111, COR 0.644444. Pellets per second
while firing = 1.9 × 15.328 = 29.1232. Procs per second: IMP 0.168267, PUN 0.925471, SLA 1.598540, COR 4.879754.
Under the Poisson stack model of section 4.2: COR mean 4.879754 × 8 = 39.04 live procs, P(n_COR = 10) > 0.99999999;
PUN mean 9.25 (10 s duration), P(n_PUN = 5) = 0.953, E[min(n, 5)] = 4.929.

**Step 9, steady-state target (state B, stacks taken at their caps for illustration).** 10 COR, 5 PUN:
A_eff = 1355 × (1 − 0.80) = 271 (strips may go below the 200 floor), DR = 0.285132; CC = 0.30 + 5 × 0.05 = 0.55;
per type non-crit 9.634807, 35.327626, 61.020445, 279.409406 (Σ **385.392284**); E[c] = 1 + 0.55 × 3.4010749 =
2.8705912; E[damage per pellet] = **1106.3037**.

**Step 10, EV DPS (08).** State A, direct only: burst = 394.7458 × 29.1232 = **11,496.26**; sustained × ρ =
**7,986.83**. State B: direct burst 32,219.10; Slash DoT = 1.598540 procs/s × 6 ticks × 78.533656 × 2.8705912 =
**2,162.23** (Little's law; the inducing hit's crit includes the Puncture bonus, UNVERIFIED `OQ-017`); burst total
**34,381.33**; sustained **23,885.83**. A crude steady-state TTK, HP / sustained = 73,621.375 / 23,885.83 =
**3.08 s**, ignores the stack ramp; the EV window TTK of `08` is longer.

---

## 4. Evaluation model

### 4.1 Common semantics

- **Scenario.** A scenario is (build, attack or attack group, enemy and its options, toggles, buffs, `t_win`). Both
  evaluators read the same resolved stats (stages ST-1 to ST-7) and the same deterministic **schedule generator**
  (`06` ranged, `07` melee), which lists shot times, reload intervals, charge delays, burst spacing, spool, beam ramp
  and melee hit multipliers. The schedule never depends on random draws.
- **Fresh state.** Every window starts with: no statuses on the target, full magazine (or Incarnon charge
  magazine), combo counter at Initial Combo, sniper combo 0, beam ramp at its start value, spool at its start,
  conditional buffs per T-02, Galvanized and arcane stacks per T-02.
- **Layer views.** For the four views `health`, `armor`, `shield`, `overguard`, every damage instance is evaluated
  with that view's layer rules (E-12, E-13) against a target that never depletes. `health` ignores armor ("100 %
  armor stripped", the inspiration's Health panel). The status state is the same in every view. The `layered` view
  depletes OG → SH → HP and is used for TTK only.
- **DoT accounting.** *Landed*: a tick counts at the time it lands; ticks after `t_win` do not count toward window
  metrics and are reported as "pending DoT". *Credited*: the whole DoT of a proc counts at the proc time (the
  inspiration's accounting, `I:agg` §6, `JS L10982-10986`); used only for parity outputs. Steady-state metrics use
  per-proc totals (equal to landed in the long run, Little's law).
- **DPS definitions** (all per attack and per view):

| Output ID | Definition |
|---|---|
| `dps_burst` | Expected damage per second during uninterrupted fire (no reload, unlimited ammo) with every stacking state at its steady-state distribution (section 4.2) and combo, sniper combo and conditional stacks at their steady value. DoT enters as Σ_s λ_s × (expected total damage of one proc of s). Heat: see section 4.2. |
| `dps_sustained` | As `dps_burst` over full magazine cycles: direct damage and proc rates scaled by the duty ρ (`06` defines ρ per weapon class, e.g. bows, battery, per-shell, infinite ammo); stack distributions use the cycle-average proc rate (approximation). For weapons without reload (melee, infinite ammo) it equals `dps_burst`. |
| `dps_window` | Expected damage landed in [0, `t_win`] from the fresh state, using the real schedule (reloads included), divided by `t_win`. Headline number; default optimizer metric. |
| `dps_window_mc` | The same quantity per Monte Carlo run; reported as mean, median and percentiles. |
| `dps_series[b]` | Damage landed in time bin b (default 1 s bins) divided by the bin width. |
| `queue_dps_parity` | The inspiration's "Random queue dps": credited damage of each hit spread over 1-s buckets by hit index (`getDmgPerSecond`, `JS L12787-12801`), median of the `t_win` buckets (`JS L12802-12805`, `L13269-13273`). Parity output only (section 4.5). |

### 4.2 Evaluator (a): deterministic expected value (EV)

Purpose: score many thousands of builds per second per core for the optimizer, and give burst, sustained and window
numbers without noise. `08` owns the full algorithm; these decisions bind it.

**Randomness.**

| Random quantity | EV treatment |
|---|---|
| Crit tier | linear: use `E[c] = 1 + E[k]·(h·CDM − 1)` (exact for any CC; Vigilante via E-6). DoTs and extra hits inherit the inducing crit linearly, so their expectations multiply by the same `E[c]` (crit and procs are independent, CD-03). |
| Multishot | linear: expected pellets `MS` per shot. Effects that depend on pellet order (sequential CO count) use the expected number of earlier pellets. |
| Proc counts | expected procs of s per pellet `SC·w_s + φ_s`; proc rate `λ_s` per second; Hunter Munitions `p_HM·P(crit)` per pellet, Internal Bleeding `p_IB·P(IMP proc)`, combined as `1 − (1 − a)(1 − b)` (`F:addmul` §8). |
| Live stacks | the number of live procs of s is modelled as Poisson with mean μ_s (sum of the expected procs whose duration window covers the time), independent across status types; stacks `n_s = min(N, cap_s)`. Every nonlinear target function uses this distribution: `E[Vir(n)]`, `E[Mag(n)]`, `E[DR(A_eff)]` over the joint distribution of COR stacks and Heat presence, `E[cold_cd(n)]`, `E[punc_cc(n)]`. |
| Distinct statuses | `E[n] = Σ_s P(n_s ≥ 1) + external statuses (T-06)`; with the Poisson model `P(n_s ≥ 1) = 1 − exp(−μ_s)`. Exact by linearity under the independence assumption. CO bonuses are linear in n, so `E[C]` follows. For pellet j of a shot (j = 0 .. MS − 1), add the chance that earlier pellets of the same shot applied a new type. |
| Heat | one shared refreshing timer: Heat is present at time τ iff a Heat proc landed in the last `D_HEA` seconds (`P = 1 − exp(−μ_HEA)`); while present, the pool sums every Heat proc since ignition. The pool has no steady state while Heat keeps being refreshed (`F:status` §4.4), so the Heat DoT contribution to `dps_burst` and `dps_sustained` is its expected mean over [0, `t_win`]. |

**Ramp-up.** Two time modes:

- *Steady* (`dps_burst`, `dps_sustained`, steady-state stack outputs): μ_s = λ_s·D_s; combo, sniper combo and
  conditional stacks at their steady values; beam ramp full; spool full.
- *Window* (`dps_window`, EV TTK): the schedule is walked shot by shot from the fresh state; μ_s(τ) counts expected
  procs of earlier shots whose stack is still alive at τ; combo, sniper combo, spool, beam ramp and `simulated`
  conditional stacks follow their time models (`06`, `07`, `04`); each DoT proc contributes its ticks with
  τ ≤ `t_win`, each tick evaluated with the target-state expectation at τ.

**Outputs.** Per attack and view: `dps_burst`, `dps_sustained`, `dps_window`; steady-state `E[n_s]`,
`P(n_s ≥ 1)`, `P(n_s = cap_s)` for every status the attack can apply; `E[n]` (distinct count); expected damage per
shot and per magazine; expected crit-tier distribution; per-damage-type breakdown (direct per type, DoT per status,
extra hits); approximate TTK.

**Approximate TTK.** Walk the window-mode expected damage per instance through the `layered` view (OG → SH → HP,
Toxin routing, CD-08 overflow, no shield gate) and report the first time the expected cumulative damage reaches the
pools. If the target is not dead at `t_ttk_max`, extrapolate with the remaining expected HP divided by
`dps_sustained` and flag the value as extrapolated. EV TTK is an expectation-path estimate, not the mean of the TTK
distribution; the MC evaluator gives the distribution.

### 4.3 Evaluator (b): event-driven Monte Carlo (MC)

Purpose: game-faithful simulation, distributions, validation of the EV evaluator, TTK.

- **Events.** Shot (from the schedule), pellet hit, radial hit, extra hit, proc application, stack expiry, DoT tick
  (per DoT instance; one shared Heat tick per second), Blast fuse expiry and detonation, Heat strip step, reload
  start and end, combo gain and expiry, Galvanized and arcane stack gain and expiry (`simulated` conditions), sniper
  combo decay, beam ramp, Magnetic break proc. Events with equal time are processed in the order of CD-04, then by
  event creation order.
- **RNG.** Counter-based generator (Philox4x32-10 or ChaCha8; `08` fixes one), keyed by
  `(seed, run, attack, event index, mechanic id)`, so results do not depend on thread count and two builds evaluated
  with the same seed see the same draws per mechanic (common random numbers for the optimizer's validation stage).
  Exact fractions are used; no rounding of chances to whole percent (D-017).
- **Runs.** Default `n_runs = 1000`, `seed = 0`; each run starts from the fresh state. One pass simulates the window
  with a non-depleting target (layer views); a second pass with the same draws simulates the `layered` target with
  depletion, shield gate and overflow (CD-08) until kill or `t_ttk_max`.
- **Reported.** For `dps_window_mc` of every view: mean, standard deviation, 95 % confidence interval of the mean,
  median, p5, p25, p75, p95. `dps_series` (mean and p10/p90 per bin). Per-damage-type and per-source breakdown
  (direct, DoT by status, extra hits, special instances). Status uptime and mean stacks per bin. Crit-tier
  histogram. TTK: kill probability within `t_ttk_max`, median, mean of killed runs, p5, p95; shots and ammo to kill.
  Pending DoT at `t_win`.
- **Fight window.** Default `t_win = 20 s` (the inspiration's `queueLength`, `JS L5496`), range 1-60 s (parity),
  advanced up to 600 s.

### 4.4 Consistency between the evaluators

The EV `dps_window` MUST equal the MC mean `dps_window_mc` within the MC's 99 % confidence interval plus a 2 %
modelling tolerance, on the golden scenarios of `08`. A larger gap is either an engine bug or a documented EV
approximation (Poisson stacks, independence, Heat window mean); `08` lists the expected gap classes. The optimizer
validates its top builds with MC (`I:opt` §12.2 proposal).

### 4.5 Outputs: inspiration parity and our additions

Reproduced from the inspiration (`I:feat` §3, §11-§13; `I:agg` §7.4):

| Inspiration output | Our definition |
|---|---|
| Result panels Armor, Health, Shield, Overguard per attack | layer views `armor`, `health`, `shield`, `overguard` (section 4.1); a panel exists when the enemy has that layer |
| "Random queue dps" | `queue_dps_parity` (one MC run); with "average over cycles": median of per-run values over `n_runs` runs, plus min and max |
| "avg per bullet (without status dmg)" | mean over parent hits of direct damage + extra-hit direct damage (Xata, Toxic Lash hit) + flat-per-status damage |
| "avg per bullet (status dmg)" | mean over parent hits of the credited DoT of the procs that hit applied (Toxic Lash ticks included) |
| "avg per bullet (total)", "median per bullet (total)" | sum of the two means; median over parent hits of direct + credited DoT |
| TTK | MC `layered` TTK (median over runs; one run for a single "random queue") |
| "More details" timeline | per-hit event log of one MC run: time, damage per view, crit multiplier, armor reduction, stance and heavy-combo multipliers, extra-hit damage, statuses applied, one-tick value per status, live stacks per status; reload markers |
| Weapon panel modded stats and per-type damage | resolved stats of ST-1 to ST-7 (`q_t·Y·(1 + F)` per type) |
| Enemy panel (health, shield, armor, overguard, DR) | `05` scaled values |

Added outputs: `dps_burst`, `dps_sustained`, EV `dps_window`, MC distribution of `dps_window_mc`, `dps_series` chart,
per-damage-type and per-source breakdown, steady-state stacks and distinct-status count, status uptime chart,
crit-tier distribution, damage per shot and per magazine, ammo and shots to kill, kill probability, pending DoT,
parent + child summed DPS for attack groups, EV-versus-MC deviation flag.

### 4.6 The inspiration as test oracle

The inspiration runs offline in headless Chromium (`I:oracle`). It is compared with our engine in two ways (`08`
owns the procedure):

1. **Compatibility profile.** An internal configuration `compat = inspiration` switches every register row marked
   `Compat` in section 5 to the inspiration's behaviour. Golden tests compare `queue_dps_parity` (median over seeded
   runs, `Math.random` seeded per `I:oracle`) with the inspiration's averaged output. Such tests are valid only on
   scenarios where no register row marked `—` applies.
2. **Game profile.** With the normal (game) profile, oracle comparisons are valid only on scenarios where no
   register row applies at all (both programs agree by construction); `08` lists the oracle-safe scenario rules.

---

## 5. Discrepancy policy and register

### 5.1 Policy

1. Game behaviour wins (section 1.4). Every known disagreement is a row below; a topic document that finds a new one
   adds a row in its own range (`D-n01`...).
2. The decision states exactly what the engine does. If game sources disagree among themselves or are unverified,
   the decision names the chosen behaviour and is marked `UNVERIFIED`.
3. **Toggle** column: `User` = the behaviour can be changed by a section 6 toggle; `Compat` = an internal flag of the
   compatibility profile (oracle tests only, not shown in the UI); `—` = no switch (oracle tests must avoid it).
4. The owner document restates its rows with the full detail.

### 5.2 Register

Inspiration line references are to `min.pretty.js` as cited in the inspiration findings.

| ID | Topic (owner) | Inspiration behaviour | Game behaviour (source) | Decision | Toggle |
|---|---|---|---|---|---|
| D-001 | Quantization grid (01) | 1/16 of modded base (`setQuantize`, JS L9787-9791) | 1/32 since U40 ("(Undocumented) Damage Quantization changed from 1/16 to 1/32", `W:Damage` patch history; `F:core` §14) | 1/32 | Compat |
| D-002 | What is inside quantization (01) | Eclipse, `multiple`, frozen, Broadhead, Mecha, Madurai, Cyte, CO, Sharpshot inside the rounding (`I:stat` §7, Q5) | only per-type composition is quantized; uniform multipliers outside (`W:Damage/Calculation` §Quantization Summary) | CD-02 | Compat |
| D-003 | Base crit multiplier quantization (02) | none (`I:stat` §9.2) | 32/4095 grid in f32 before mods (`W:Critical_Hit` §Quantization) | E-4 | Compat |
| D-004 | Two innate primary elements (01) | JSON key order, progenitor last (JS L12912-12917; `I:stat` Q6) | Heat > Cold > Electricity > Toxin order (`W:Damage` §Modding; Kuva rendered pages; `F:core` §5.2 rule 5) | HCET | — |
| D-005 | Incarnon flat split (01) | spread proportionally to type split incl. progenitor, ratio from listed total without progenitor (`addDmgToBase`, JS L13193-13205) | split not documented (`F:core` §3.1, open question 1) | X split proportionally to `B0_t + V_t`, summing exactly to X; UNVERIFIED (OQ-030) | — |
| D-006 | Violet shard Electricity position (01) | combined in slot order (shard slots 23-27, before innate elements) (`I:stat` §4.1) | added after modded elements, ordered with innate and progenitor elements by HCET (`W:Violet_Archon_Shard`, `F:buffs` §5) | game order; UNVERIFIED for other non-mod elements (OQ-024) | — |
| D-007 | Phoenix Talons ("Madurai") (01, 04) | ×1.3 on physical types, not on exalted weapons (JS L9254-9256, L9495) | +0.30 × total base (elements included) added as physical, split by physical shares, after physical mods; applies to exalted weapons (`W:Focus/Madurai`, `F:buffs` §6) | game formula; base-bucket scaling UNVERIFIED (OQ-020) | User (T-19) + Compat |
| D-008 | Faction mods vs Murmur, Corpus Amalgam (01) | no Murmur key → NaN → ×1 (Q15); Corpus mods do not hit Corpus Amalgam (`I:model` Q2) | Murmur bane works; Corpus/Grineer/Infested mods apply to Amalgam/Kuva/Deimos (`W:Faction_Damage_Bonus` §Mechanics) | game | — |
| D-009 | Impact-to-Puncture swap (01) | overwrites an existing Puncture value (`swapDmgType`, JS L7342-7350) | conversion adds into the target type (no source; physical conversion mods add) | add; UNVERIFIED | — |
| D-010 | Riven Weak Point Critical Chance (02, 04) | stored under `SMITE`, no effect (JS L14722-14725; `I:stat` Q14) | relative crit chance on weak-point hits (`F:addmul` §11) | game | — |
| D-011 | Riven Weak Point Damage (02, 04) | mapped to the headshot-multiplier bucket `mult_for_head` (JS L14728) | weak-point damage bonus ΣWP (`F:addmul` §10-§11) | game | — |
| D-012 | Custom "Weak Point Increase Damage (as Primary Acuity)" (02) | adds the raw value to the head part `3 + x` (JS L9070-9076) while the Acuity mod itself is stored as 5.25 = 1.5 × 3.5 | ΣWP counts 1.5× on head weak points (`W:Enemy_Body_Parts`) | the custom stat enters ΣWP (head part gains 1.5 × value) | — |
| D-013 | Head-crit (02) | every headshot crit uses `2·CDM` (JS L8319-8328) | only on head-crit parts; no head-crit on Corpus humanoids, most Murmur and Techrot, MOA packs; 1×-headshot weapons never (`W:Critical_Hit`, `F:core` §10) | game; per-enemy part data (`05`) | — |
| D-014 | Absolute crit damage placement (02) | `crit_mult_add` (Crepuscular, Tenacious Bond) and Melee Doughty added after tier scaling: `1 + k(CDM − 1) + abs` (JS L8314-8328; `I:stat` Q10) | added to CDM before tiers: `1 + k(CDM − 1 + abs)` (`W:Arcane_Crepuscular`, `F:core` §9) | game | Compat |
| D-015 | Cold crit damage (02, 03) | +0.05 per stack (+0.5 more at 10) added to the final multiplier after tier and headshot; also on AoE (JS L10821-10825) | +0.10 at 1, +0.05 per stack to +0.50 at 9, Frozen +1.0; absolute, inside CDM before tiers; not on AoE; 4 stacks max on bosses and overguarded units (`W:Damage/Cold_Damage`, `F:status` §4.5) | game | Compat |
| D-016 | Hunter Munitions (02) | needs a crit, then rolls `0.3 × min(CC, 1)` (JS L10761-10766): P = 0.3·CC² below 100 % CC | 30 % per critical damage instance (`W:Hunter_Munitions` lines 2, 38) | game; tier independence UNVERIFIED (OQ-018) | Compat |
| D-017 | Fractional chances (02) | crit, status and multishot fractions rounded to whole percent before rolling (JS L10005, L10012, L10054, L10679) | exact | exact | Compat |
| D-018 | Crit tier on a failed roll (02) | tier from `CC − 99`: one tier too many when frac(CC) ∈ (0.99, 1) (JS L9826) | `floor(CC)` (`F:core` §9) | E-6 | — |
| D-019 | Multishot floor (02) | none for negative riven multishot (JS L8983-9006) | riven curse cannot bring multishot below 1 (`W:Multishot` §Sources) | `MS ≥ 1`; UNVERIFIED for base MS > 1 | — |
| D-020 | Proc type selection (02) | 100-slot mask with `ceil(share)` slots per type; last types lose up to n − 1 points (JS L12239-12246, L10022) | exact proportional weights (`W:Status_Effect` §Damage Distribution) | exact | Compat |
| D-021 | Encumber random status (02, 04) | `round(random × 12)`: Impact and Viral at half weight (JS L10036, L12875) | random status (`F:addmul` arcane list) | uniform over the 13 damage statuses; UNVERIFIED | — |
| D-022 | Stealth damage (04, 07) | melee non-crit ×7; crit `c + 7`; weapon rank ignored (`getStealthDmgBonus`, JS L8330-8336) | `× (1 + SDB)` non-crit = ×8 at rank 30; crit `1 + SDB + k(CDM − 1)`; `SDB = 1 + 0.2·rank`, capped at rank 30 (`W:Stealth` lines 36-49) | game (E-12) | User (T-08) + Compat |
| D-023 | Weak-point crit chance multiplicative (02) | hidden option `o.w` and per-attack `isWPCritMult` multiply CC by `(1 + WP CC)` (JS L8201-8210) | additive in the relative bucket (`W:Primary_Acuity`, `F:addmul` §10) | additive; option kept for link import | User (T-21, hidden) |
| D-024 | Proc weights quantized (02) | weights from unquantized modded damage (JS L10063-10077) | not stated (`F:status` §8 open 2) | weights from `q_t`; UNVERIFIED (OQ-004) | — |
| D-025 | DoT "+1" accumulator (03) | none (JS L9154-9204) | `tick = (Σ seeds + 1)·C·M`, once per merged Heat/Electricity/Gas pulse, per stack for Slash/Toxin (`W:Damage/Calculation` §DoT, single editor 2026) | apply; UNVERIFIED (OQ-023) | Compat |
| D-026 | Heat armor strip (03, 05) | flat 50 % while any Heat stack is live (JS L10815) | ramp: ×0.5^(1/4) every `0.5·(1 + SD)` s to 50 % at 2 s; returns in 4 steps every 1.5 s over 6 s after Heat ends (`W:Damage/Heat_Damage` §Armor Stripping, edited 2026-11-14) | game | Compat |
| D-027 | Heat stacking (03) | each Heat proc expires on its own timer in the DPS numbers; TTK uses a shared refreshing timer (JS L11139-11143, L13690-13703) | one shared timer refreshed by every new proc; all procs pool into one tick per second; duration = longest SD among stacks (`W:Damage/Heat_Damage` §Stack Combining) | game, both evaluators | Compat |
| D-028 | Heat inherit (03) | custom stat `heatInherit` replaces the Heat elemental bonus (JS L9154-9204) | a burning target keeps the first source's Heat elemental and faction bonus for later procs; inherited faction applies once (`W:Damage/Heat_Damage` §Heat Inherit) | external first source via T-26 custom stat; same-weapon later procs keep faction² (UNVERIFIED, OQ-006) | User (T-26) |
| D-029 | Viral and Magnetic after expiry (03) | multiplier keeps its last value after the last stack expires (`I:status` Q6, JS L10320-10321, L10779-10804) | multiplier is 1 without stacks (`W:Damage/Viral_Damage`) | game | Compat |
| D-030 | Own-hit and same-shot timing (03) | a hit sees its own procs and earlier pellets' procs for every debuff (`timelineStatus`, JS L10093-10136) | a hit deals damage before its procs; sibling pellets do not share Viral; CO counts earlier pellets (`W:Damage/Viral_Damage`, `F:addmul` §1.2) | CD-04 | Compat |
| D-031 | Target-side terms on DoT ticks (03) | Viral, Magnetic and armor strip frozen at the inducing hit (`I:enemy` §3.3, §9.2) | evaluated when each tick lands (`W:Damage/Viral_Damage`) | CD-05 | Compat |
| D-032 | Puncture duration (03) | 6 s (JS L5547) | 10 s (`W:Damage/Puncture_Damage`, `W:Status_Effect` tables; `W:Damage` table still says 6 s, outdated) | 10 s | Compat |
| D-033 | Impact duration (03) | 1 s (JS L5545) | 6 s per stack (`W:Damage/Impact_Damage`; `W:Status_Effect` "1 second" is the flinch) | 6 s; UNVERIFIED (conflicting tables) | — |
| D-034 | Blast (03) | unlimited stacks, no detonation; damage `0.3·` scaled by `parseInt(1.5·(1 + SD))` and Blast element mods; ignored by TTK (JS L5571, L9177, L11356-11360, L13669) | 1.5 s fuse per stack (SD lengthens it), one 0.3·H hit at expiry; 10 stacks (4 bosses) or death detonate all; no elemental scaling (`W:Damage/Blast_Damage`, edited 2026-10-27; `W:Damage_over_Time` disagrees) | Blast page; elemental scaling UNVERIFIED (OQ-007) | Compat |
| D-035 | Gas (03) | one Gas proc per hit credited `gasStacks × tick`; drops non-crit headshot factor; extra faction in non-crit path (JS L11219-11290; `I:status` Q22) | each proc: 6 ticks at t + 0 .. t + 5, own timer, cap 10 (`W:Damage/Gas_Damage`) | game | — |
| D-036 | Status duration rounding (03) | `parseInt(D0·(1 + SD))` for every status, incl. debuffs, Blast and Toxic Lash (JS L9085-9093) | DoT ticks rounded down (Lasting Sting note); debuff rounding unknown | CD-06; UNVERIFIED (OQ-005) | Compat |
| D-037 | Toxic Lash DoT duration (03, 04) | fixed 7.5 s (7 ticks credited in DPS, 8 in TTK) (JS L5555-5561, L13706-13713) | Toxin DoT `floor(6·(1 + SD))` ticks; Saryn passive +25 % SD (`W:Toxic_Lash`, `F:buffs` §1.4) | enabling Toxic Lash adds +0.25 SD to its procs (Saryn passive) | — |
| D-038 | Innate DR on DoT (05) | `(1 − innateDR)²` on most ticks, cubed on armor (JS L10885, L9202) | one application per damage instance (`F:enemy` §9) | once | Compat |
| D-039 | DoT on shields (05) | ×3 on the shield layer (`Ob`, JS L10576); no source | no such multiplier | ×1 | Compat |
| D-040 | Armor-layer DoT faction modifier (05) | type modifier applied twice (JS L9198-9202) | once | once | Compat |
| D-041 | Mecha Empowered and Eclipse on DoT (03, 04) | Mecha squared on elemental DoT, once on Slash; Eclipse once on elemental DoT, not on Slash; Mecha unconditional (`I:status` Q13, Q24) | Mecha "applied each time damage is calculated" (twice on every DoT), only vs marked targets; Eclipse is part of the modded hit, once on every DoT (`W:Mecha_Empowered`, `W:Damage_over_Time`) | game (E-13 `M_dot`) | — |
| D-042 | Overguard scaling (05) | f1 below Δ 45, f2 from Δ 45, no smoothstep; Δ = level − BaseLevel (JS L7624-7636) | smoothstep over 45-50; `q = level − 1` per the page text (2026-07 rewrite), `level − BaseLevel` in the wiki gadget (`F:enemy` §7.5) | smoothstep with `q = L − 1`; UNVERIFIED (OQ-009) | Compat |
| D-043 | Eximus scaling (05) | default health/shield curves × step factor on absolute level; armor × up to 1.625 (JS L7637-7686) | faction curves; base health `max(1.1·B, c·(B + 900)·k(d))`, c = 0.25 or 0.375, d = level − BaseLevel (gadget); no armor change (`F:enemy` §8.2) | game (gadget); variable d UNVERIFIED (OQ-010) | Compat |
| D-044 | Faction scaling curves (05) | Techrot health 0.015Δ^2.1 / 13.64Δ^0.72, shield 3.4106; Scaldra, Amalgam, Anarchs on default; Sentient shield f2 2.0; Infested C2 16.0997 (JS L8048-8095) | Techrot 0.02/2.12, 15.0998/0.70, shield 3.5; Scaldra = Grineer, Amalgam = Corpus, Anarchs = Orokin (module); Sentient shield 1.6; Infested 16.0998 (page) vs 16.1 (module) (`F:enemy` §7.2-§7.3) | wiki page values; Anarchs per module; UNVERIFIED where page and module differ | Compat |
| D-045 | Scaling arithmetic (05) | f64 `f1(1 − s) + f2·s`; values rounded to 0.01 then integer; armor `round` (JS L8096-8125) | f32 `f1 + (f2 − f1)·s`; armor `floor` (wiki gadget) (`F:enemy` §7.1, §3.2) | NP-2, armor floor; UNVERIFIED (OQ-026) | Compat |
| D-046 | Faction damage-type table (05) | Sentient +Magnetic; Amalgam lacks Magnetic; Scaldra resists Toxin; Thrax, Void Angel, Wolf of Saturn Six get none; " Infested", " Sentient" match nothing (JS L5503-5518) | U36 table: Sentient Cold/Radiation, resist Corrosive; Amalgam Electricity/Magnetic, resist Blast; Scaldra resists Gas; overrides per enemy (`W:Damage/Overview_Table`, `Module:DamageTypes/data`, `F:enemy` §2) | wiki table + per-enemy `FactionDamageOverride` | — |
| D-047 | Unique damage reduction (05) | disabled for every enemy since 2025-10-18 (`unique = "notset"`, JS L7690) | attenuation (U40 health-proportional MDPI/MDPS), flat DR (Archons 20 % + 50 % status, Juggernaut 90 %), Sentient adaptation, Orphix DPS-based DR (`F:enemy` §9) | model where documented, parameters UNVERIFIED (OQ-022); on by default | User (T-28) |
| D-048 | Shield gate (05) | absent (`I:enemy` §5.3) | 0.1 s at 5 % after a full shield break; weak points and abilities bypass (`W:Shield` §Shield Gating) | MC TTK only (CD-08) | — |
| D-049 | Layer overflow (05) | overflow of a breaking hit discarded on every layer (`applyDamageLayer`, JS L13805-13809) | unclear (`F:enemy` open 10) | CD-08; UNVERIFIED (OQ-011) | Compat |
| D-050 | Toxin bypass exceptions (05) | none (`I:enemy` §5.1) | Treasurer and Sisters' Hounds shields block Toxin (`W:Shield`, `W:Treasurer`) | enemy flag `toxin_blocked` | — |
| D-051 | Armor strip option and Corrosive Projection (05) | option applied to base armor before clamps (skipped on Steel Path except at the clamps); Corrosive Projection multiplies with it (JS L7698-7712, L8113-8116, L10805-10820) | ability strips and Corrosive Projection are additive with each other and use total (clamped) armor; Heat and Corrosive multiply (`W:Corrosive_Projection`, `W:Armor` U32 note, `F:enemy` §3.3) | game (E-15) | User (T-07) + Compat |
| D-052 | Enemy-specific resistance and armor maps (05) | `elemRes` ignored on armor-layer direct hits; armorFull pass uses the Toxin-only map (`I:queue` QM16-QM17) | resistances apply to every instance | consistent application | — |
| D-053 | Status cap of capped enemies (03, 05) | `maxProcStacks` limits same-type procs inside one hit and live stacks, never DoT damage (JS L10011-10031, L10119-10127) | bosses and Liches "no status effect will exceed 4 stacks" (U27.3); per-type caps on current pages | CD-07; UNVERIFIED for DoTs (OQ-008) | Compat |
| D-054 | Magnetic break proc (03) | not modelled (`I:status` §5.2) | on shield or overguard break: forced Electricity proc, `0.03·n_MAG·max pool` over 6 s, status damage twice, faction twice (`W:Damage/Magnetic_Damage`, U36) | model (MC; EV in TTK only) | — |
| D-055 | Steel Path flat base-health bonuses (05) | not modelled (`I:enemy` §2.6) | Juggernaut +1,800 etc. (`Module:Enemies/data` `SteelPathHealthBonus`) | add to base before ×2.5 (gadget order; UNVERIFIED) | — |
| D-056 | CO status count (04) | number of damage types in the attack's modded damage (+1 for external Viral), static (`getCountDifferentStatuses`, JS L12015-12020) | distinct status types on the target from any source, excluding the hit's own (`W:Condition_Overload_(Mechanic)` §What Counts) | expected or simulated count | User (T-05) |
| D-057 | CO additive vs multiplicative (04) | global option or per-attack data flag `isCoMult` (JS L9235, L9393) | per-attack catalogue: behaviour, ratio `B_co/B_ref`, "does not apply" list (`W:Condition_Overload_(Mechanic)`) | catalogue (E-11); default `adding`, ρ_co = 1 | User (T-04) |
| D-058 | Adding-CO omissions (04) | CO term multiplied by Eclipse, `multiple` and the other map multipliers (JS L9389-9401, L9492-9498) | Eclipse, Garuda, Parasitic Link, Overdriver, Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, projectile Primary Compression, Dragon Key, falloff, bow charge omit the CO term (`F:addmul` §1.6) | E-11 with `G_F` | Compat |
| D-059 | CO on AoE and Incarnon flat (04) | off for every `shot_type == "AoE"` and slam; the Incarnon-flat path applies additive CO even with conditions off, on AoE and on slams, and drops Sharpshot (JS L9402-9414; `I:stat` Q2) | hitscan explosions give CO to the directly hit target from the hitscan damage; flat excluded from B_co on Adding attacks (`F:addmul` §1.3, §1.7) | catalogue rules | — |
| D-060 | Secondary Shiver (04) | +4.5 whenever conditions are on, Cold stacks not read (JS L9365) | +0.45 per Cold stack on the target (`W:Secondary_Shiver`) | per stack (expected or simulated) | — |
| D-061 | Conditional stacks over time (04) | Galvanized, arcanes, on-kill mods fixed at max stacks under "trigger all conditions", else ignored (`I:queue` §8) | stacks ramp and decay with their triggers (`F:mods` §6.1) | T-02 modes `all_max`, `none`, `simulated` | User (T-02) |
| D-062 | Light-attack combo (04, 07) | combo-scaled bonuses use 12× when the option is off (`combo_counter = 11`, JS L8219-8224, L9906-9910) | combo ramps from Initial Combo with hits (`W:Melee_Combo`) | option off → simulated ramp | User (T-10) + Compat |
| D-063 | Combo range (07) | combo inputs accept 1-13 for every weapon (`I:feat` quirk 5) | cap 12×; Venka Prime 13×; Dex Nikana 11 per tier, cap 11× (`W:Melee_Combo`) | clamp to the weapon's cap | — |
| D-064 | Heavy-attack combo model (07) | combo spent every `int(0.8·hits/T) + heavyHits` hits, never rebuilt, Initial Combo restored instantly (JS L12158-12181) | combo gained per stance hit, spent by heavies with HAE retention (≤ 0.9), Initial Combo regenerates at 40/s (`W:Melee_Combo`, `W:Melee`) | game | — |
| D-065 | Melee hit timing (07) | hits spread evenly over the window; heavy count from the default stance table even with a stance mod; slams 1/s and forced `statuses[0]` on every slam hit (JS L10137-10216, L12115-12150) | stance combo durations and per-hit lists; per-weapon heavy, wind-up and slam attacks (`Module:Stances/data`, `F:melee` §1-§4) | game data; uniform spacing inside a combo (UNVERIFIED) | — |
| D-066 | Sniper combo (06) | fixed multiplier, 1.5 by default even with the option off (JS L10326-10330) | count rises per hit, `M = 1.5 + 0.5·floor(log3(count/MinCombo))`, decays (`W:Sniper_Rifle`) | simulated unless T-12 fixes it | User (T-12) + Compat |
| D-067 | Spiteful Defilement (04) | doubles the whole crit multiplier when total stacks < 3 (JS L10752-10760) | perk text: +100 % critical damage with fewer than 3 statuses | +1.0 in ΣCD_rel when `n < 3`; bucket UNVERIFIED | — |
| D-068 | Melee Careen and Archon Electricity ability shard (04) | unconditional multipliers (JS L10466, L10389); the shard applies to exalted weapons | Careen ×2.5 only vs Frozen targets; ability-damage shards never apply to exalted weapons (`W:Melee_Careen`, `W:Archon_Shard`) | game | — |
| D-069 | Per-status status-damage bonuses (03) | Emerald Toxin etc. added to the product `zb` (outside `multiple` and other factors) (JS L11003, L11084) | additive with Elementalist inside one bucket (`W:Archon_Shard` line 337) | inside A_sd (E-13) | — |
| D-070 | Hunter Munitions proc as a stack (03) | its Slash DoT is credited but never enters the timeline (JS L10761-10766) | it is a Slash status | counts for stacks and CO | — |
| D-071 | Xata's Whisper (04) | double-counts headshot and vulnerability, ×21 in non-crit branches, innate DR inside the armor argument (JS L11403, L11584, L11763, L13226-13243) | E-14 (`W:Extra_Hit`, `W:Xata's_Whisper`) | E-14; Blast-detonation trigger modelled in MC (UNVERIFIED magnitude) | — |
| D-072 | Toxic Lash damage types (04) | the extra hit inherits the weapon's per-type modifiers (JS L11465-11495) | it is a Toxin instance (`W:Toxic_Lash`) | E-14 | — |
| D-073 | Primary Debilitate faction (03) | faction applied a third time on its procs (JS L11459) | ordinary procs: faction twice | twice | — |
| D-074 | Nightwatch Napalm (03, 06) | DoT base 75 overwritten by 150; progenitor scaling; credited at hit (JS L10409-10441) | wiki mod values | wiki values; UNVERIFIED | — |
| D-075 | Bow cycle (06) | `1/(CT + reload)` per shot plus a magazine reload after every shot: interval `CT + 2·reload` (JS L12290-12293, L12333-12348) | `EFR = 1/(CT + Reload)`, no separate reload (`W:Fire_Rate`, `W:Bow`) | game | Compat |
| D-076 | Charge weapons (06) | `1/(CT + reload)` per shot plus magazine reloads; only shot 0 pays the charge (JS L12296, L12351) | `EFR = 1/(CT + 1/FR)`; Lanka `1/CT`; magazine reload as usual (`W:Fire_Rate`) | game | Compat |
| D-077 | Semi-auto cap (06) | none; no `trigger` field read (`I:agg` §2.3) | 10 shots/s (Komorex ≈ 8) (`W:Fire_Rate` §Mechanics) | cap 10 with per-weapon override; UNVERIFIED value (OQ-012) | — |
| D-078 | Beam damage ramp (06) | not modelled (`I:agg` §2.3) | 20 % → 100 % over 0.6 s, decays after 0.8 s idle over 2 s; per-weapon starts (`W:Continuous_Weapon`) | linear ramp; UNVERIFIED shape and starts (OQ-013) | Compat |
| D-079 | Spool-up (06) | no spool code in the scheduler (JS L12251-12406) | rate rises from a start fraction per shot (Gorgon 9 shots from 20 %) (weapon pages, `F:weapon` §6) | game; reset rule UNVERIFIED | Compat |
| D-080 | Burst delay (06) | divided by `(1 + speed)` and multiplied by `(1 + speed_mult)` (JS L8936-8952) | `BD / (1 + max(0, ΣFR))` (`W:Fire_Rate` §Burst; inspiration changelog 2026-01-28 agrees on scaling) | game; UNVERIFIED numerically (OQ-014) | — |
| D-081 | Shot-count bound (06) | planned count `Pa` truncates: rates below `1/T` give 0 shots (JS L12299-12321) | — | event loop only | — |
| D-082 | Battery and per-shell reload (06) | recharge uses the ammo-efficiency-inflated magazine; `reloadRate` mods divide a rate; reload mods do not touch per-round reloaders (JS L9019-9041, L12278-12287) | battery: `Delay_empty/(1 + RS) + Mag/Rate`, delay ≥ 0.5 s; per shell: `(t_startend + t_shell·Mag)/(1 + RS)` (`W:Reload`, `W:Battery_Weapon`) | game | — |
| D-083 | Ammo cost and efficiency (06) | 0.5 ammo for every attack of a BEAM-tagged weapon; efficiency up to 0.999999 (JS L12267-12273, L12464-12471) | per-attack `AmmoCost`; efficiency capped at 0.75 (Hotfix 44.1.3); infinite-ammo states separate (`W:Ammo`) | game | — |
| D-084 | First- and last-shot marks (06) | groups of `perMag` shots by index, also for infinite magazines (JS L12182-12196) | first shot = counter at Mag − 1 after a shot; last shot of a real magazine (`W:Primed_Chamber`, `W:Synth_Charge`) | game | — |
| D-085 | Damage falloff (06) | display only (JS L7841-7842) | applied by distance; progenitor part falls 100 % → 0 % linearly; Incarnon flat ignores falloff (bug) (`W:Damage_Falloff`) | applied at distance T-27 (default 0 m) | User (T-27) |
| D-086 | DoT accounting in DPS (08) | full DoT credited at the applying hit, never truncated at the window (JS L10982-10986) | ticks land over time | landed for our metrics; credited for parity outputs | Compat |
| D-087 | DPS statistic (08) | median of 1-s buckets filled by hit index, not time (JS L12787-12805) | — | `dps_window` (mean landed damage / T); parity output kept | Compat |
| D-088 | Averaging (08) | runs for 5 s wall-clock; "Avg" shows a median (JS L14273-14485) | — | fixed `n_runs`; mean and median labelled | — |
| D-089 | TTK simulation (08) | base durations (SD ignored), Blast ignored, Toxic Lash 8 ticks, per attack only, frozen target-side terms (JS L13593-13804) | game rules of `03` and `05` | one event model for DPS and TTK | Compat |
| D-090 | Combine attack effects (08) | timelines merged by hit index; merged procs never expire (JS L12540-12549) | — | merge by time, normal expiry | — |
| D-091 | Index-mode status extension (08) | uses `round(N/T)` hits per second; 0 below 0.5 hits/s (JS L10295-10310, L10349) | — | time-based only | — |
| D-092 | Mod capacity and polarity (09) | not modelled (`I:opt` §8) | capacity, drain, polarity, Forma rules (`W:Mod`, `W:Polarity`, `F:mods` §2) | optional constraint modes (09) | User (09) |
| D-093 | Mod incompatibility (09) | substring name test plus asymmetric `uncomp` lists (JS L6998-7007, `I:model` Q18-Q19) | same hidden parent type; wiki `Incompatible` lists, union of both directions (`W:Mod/Compatibility`, `F:mods` §5) | wiki lists | — |
| D-094 | Custom critical chance on heavy attacks (07) | custom crit chance added once per non-empty slot and twice in the per-hit path (`I:stat` Q7-Q8) | — | added once | — |

---

## 6. User-facing toggles that change mechanics

Parity source: the inspiration's "Select additional options" panel, custom-stat popup and simulation controls
(`I:feat` §8, §9, §11; `I:model` §4.5). Every toggle below is part of the scenario, is saved in our build link, and
changes the engine exactly as defined. Values are fractions unless stated (the UI shows percent).

### 6.1 Toggle definitions

| ID | Toggle (inspiration control, link key) | Values, default | Engine definition |
|---|---|---|---|
| T-01 | Fight window ("Continuous firing / attack duration", slider) | 1-60 s (advanced 1-600), 20 | sets `t_win` (section 4.1) |
| T-02 | Conditions ("Trigger all conditions for mods with conditions", `o.m`) | `all_max` (inspiration on, default), `none` (inspiration off), `simulated` (ours) | See 6.2. |
| T-03 | Hit location ("Headshot/Weak Point bonus active", `o.hs`) | `body` (default), `head`, `weak_point` (non-head, ours) | every damage instance that can hit a body part (not radial, not extra hits except Xata, DoT inherits) uses E-7 with the target's head (or first non-head weak-point) part from `05`; weak-point-conditional effects trigger |
| T-04 | CO multiplicative ("damage per status calculated as multiplicative", `o.md`) | off (default) / on | on: every attack whose CO behaviour is `adding` uses `multiplying` with ρ_co = 1; `none` stays `none` |
| T-05 | CO status count ("Manual selection of the total number of statuses", `o.ms`; plus ours) | `expected` (default), `weapon_types` (inspiration rule), `manual(N)`, N ∈ 1..50 | n for CO-type bonuses: `expected` = section 4.2 (EV) or the live target state (MC); `weapon_types` = number of nonzero damage types of the attack after combination, +1 if T-06 is on and the attack has no Viral; `manual` = N |
| T-06 | External Viral (`o.v`) | off / N ∈ 1..10, default off (10 when switched on) | the target permanently holds N Viral stacks from another source; own Viral stacks add, capped at the Viral cap (enemy cap applies); Viral counts as present for n |
| T-07 | External armor strip (`o.d`) | 0-0.99, 0 | an ability-type strip of total armor in E-15 (additive with Corrosive Projection) |
| T-08 | Stealth (melee) (`o.s`) | off / on; weapon rank 0-30 (default 30) | every melee hit uses `c_k = 1 + SDB + k(h·CDM − 1)` with `SDB = 1 + 0.2·min(rank, 30)`; the DoT of such a hit inherits it |
| T-09 | Heavy attack (melee) and start combo (`o.h`) | off / on; start multiplier 1..weapon cap, default 12 | attacks without an explicit heavy flag are evaluated as heavy attacks (`07`); the first heavy uses the start multiplier; consumption and regeneration per `07` |
| T-10 | Light-attack combo multiplier | off (simulated ramp) / M ∈ 1..cap, default off | M fixes `combo_mult` for combo-scaled bonuses (Blood Rush, Weeping Wounds, Gladiator); light attacks are never multiplied by the combo multiplier |
| T-11 | Combo ability multiplier (exalted, `o.ac`) | 1..12, off | multiplies exalted pseudo-weapon damage by the ability combo value (`07`) |
| T-12 | Sniper combo multiplier (`o.sc`) | off (simulated count) / M ∈ 1.5..100 | M fixes `sniper_combo_mult` for sniper-combo attacks |
| T-13 | Roar (`o.r`) | off / value, default 0.30 when on | `sum_faction += value` (value is final, already Strength-scaled) |
| T-14 | Eclipse (`o.z`) | off / value, default 0.30 | factor `1 + value` in `G_F` |
| T-15 | Xata's Whisper (`o.x`) | off / value, default 0.26 | extra hit E-14 with `p_xh = value` |
| T-16 | Toxic Lash (`o.l`) | off / value, default 0.30 | extra hit E-14, `p_xh = value` (×2 for melee), forced Toxin proc, Saryn passive SD (D-037) |
| T-17 | Nourish (`o.n`) | off / value, default 0.45 | `sum_elem[VIR] += value`, non-combining |
| T-18 | Flat crit chance ("~Kullervo, Harrow", `o.j`) | off / value, default 0.50 | `sum_cc_abs += value` |
| T-19 | Phoenix Talons ("Madurai", `o.g`) | off / on | Phoenix Talons per D-007 |
| T-20 | Ability strength (`o.t`) | 0-10, 1.0 | `strength` (Str) for exalted weapons and Strength-scaled weapon effects |
| T-21 | Weak-point crit chance multiplicative (`o.w`, hidden) | off / on | on: weak-point crit chance multiplies `(1 + WP CC)` instead of adding (D-023); kept for link import |
| T-22 | Combine attack effects (popup) | per weapon attack group, off | grouped attacks share one target status timeline, merged by time (CD-12) |
| T-23 | Enemy options: level, Steel Path, Eximus, party size (1-4, squad-scaled enemies only) | per `05` | `05` |
| T-24 | Progenitor element and value (`j`, `jv`) | 7 elements; 0.25-0.60, default 0.60 | `progenitor_type`, `progenitor_frac` |
| T-25 | Kitgun grip and loader, zaw grip and link, Chromatic Blade element, Reactive Storm weakness | per item | inputs to ST-1 (`06`, `07`, `04`) |
| T-26 | Custom stat values (popup, `p.*`) | see 6.3 | each value is added to the bucket in 6.3 |
| T-27 | Target distance (ours) | 0-1000 m, 0 | `distance` d for falloff (`06`) |
| T-28 | Unique damage reduction (ours) | on (default) / off | off: attenuation, flat enemy DR and adaptation are ignored (inspiration behaviour) |
| T-29 | Monte Carlo runs and seed (replaces "Calculate average values over several cycles") | `n_runs` 1-100,000 (1000), seed (0) | section 4.3 |

### 6.2 Conditions toggle T-02

Every conditional effect in the effects catalogue (`04`) declares one condition class:

| Class | Examples | `all_max` | `none` | `simulated` |
|---|---|---|---|---|
| (a) target state | CO-type per status, Secondary Shiver, below-50 %-health perks, Melee Careen (Frozen), Spiteful Defilement | count-based bonuses use n per T-05; threshold conditions are true | CO-type and every other class-(a) bonus off | from the target state (expected in EV, live in MC; health thresholds only in the `layered` pass) |
| (b) own events | on hit, on crit, on headshot, on weak-point hit, on reload, aiming, first/last shot, combo, Galvanized Scope stacks | active at max stacks from t = 0 | inactive | triggered by the attack's own events, with stacks, durations and decay (`04`) |
| (c) external | on kill, on ability cast, while invisible, wall latch, holster, on damaged, Arcane Avenger | active at max stacks from t = 0 | inactive | user uptime and stack inputs per effect, default max |

First/last-shot multipliers (Primed Chamber, Synth Charge) are class (b) but are always driven by the magazine
schedule, also in `all_max` (the inspiration does the same, `I:queue` §3.5).

### 6.3 Custom stats (T-26)

The inspiration's 26 custom stats (`I:feat` §9). Percent inputs are fractions in the engine.

| Inspiration key (link) | Label | Engine bucket |
|---|---|---|
| `base` (`p.a`) | Base Damage | `sum_base_dmg` |
| `base_per_status` (`p.b`) | Base Damage per Status Effect | CO-type per-status bonus in `sum_co` |
| `flat_change_dmg` (`p.z`) | Increase Base Damage by +X | `incarnon_flat` (HP) |
| `abilityCombo` (`p.u`) | Combo Ability Multiplier | T-11 value |
| `crit_chance` (`p.c`) | Critical Chance | `sum_cc_rel` |
| `mult_crit_chance` (`p.d`) | Critical Chance (multiple with others) | final factor `(1 + x)` on CC (no game source; custom only) |
| `flat_crit_chance` (`p.e`) | Critical Chance (absolute bonus after mods) | `sum_cc_abs` |
| `crit_chance_weakp` (`p.f`) | Weak Point Increase Critical Chance | `sum_cc_rel` on weak-point hits |
| `crit_mult` (`p.g`) | Critical Multiplier | `sum_cd_rel` |
| `crit_mult_add` (`p.h`) | Critical Multiplier (absolute bonus after mods) | `sum_cd_abs` (before tiers, D-014) |
| `crit_mult_mult` (`p.i`) | Critical Multiplier (multiple after all sources) | `cd_field_mult` factor `(1 + x)` |
| `status_chance` (`p.j`) | Status Chance | `sum_sc_rel` |
| `status_chance_mult` (`p.k`) | Vulnerable to Status Chance | factor `(1 + x)` in `sc_mult` |
| `absolute_status_chance` (`p.l`) | Status Chance (absolute bonus after mods) | `sum_sc_abs` |
| `status_damage` (`p.m`) | Status Damage (as Elementalist) | `sum_status_dmg` |
| `vuln_status_damage` (`p.o`) | Vulnerability Status Damage (as Burning Hate) | factor `(1 + x)` on DoT ticks (`M_dot`) |
| `speed` (`p.p`) | Fire Rate / Attack Speed | fire-rate or attack-speed bucket |
| `multishot` (`p.q`) | Multishot | `sum_ms` |
| `magazineSize` (`p.r`) | Magazine Size Increase | magazine percent bucket |
| `reloadTime` (`p.s`) | Reload Time Increase | reload-speed bucket |
| `mult_for_head` (`p.t`) | Headshot Increase Damage (as Deadhead) | `sum_hs_mult` |
| `headshot_mult` (`p.v`) | Weak Point Increase Damage (as Primary Acuity) | `sum_wp_dmg` (D-012) |
| `initialCombo` (`p.w`) | Initial Combo | initial combo count |
| `dmgVulnerability` (`p.x`) | Damage Vulnerability | multiplicative entry in `vuln` |
| `heatInherit` (`p.y`) | Heat Inherit | Heat DoT elemental bonus of an external first Heat source (D-028) |
| `heatAdd` (`p.aa`) | Add Heat Damage Instance (like Ember augment) | non-combining `sum_elem[HEA]` |

---

## 7. Topic map

### 7.1 The nine topic documents

The split of the brief is kept. Each scope is exclusive: a mechanic belongs to exactly one document. "Bucket owner"
documents define where a bonus enters the formula; `04` catalogues the sources (values and conditions) of buffs,
arcanes, shards, focus, auras, set bonuses and rivens; `09` decides which of them a build may legally contain.

| Document | Owns (exact scope) | Explicitly not |
|---|---|---|
| `01-damage-composition.md` | ST-1 to ST-4 except the CO term: per-attack base damage per pellet; exalted base scaling by Strength; kitgun and zaw base damage as delivered by `06`/`07`; progenitor bonus (value, fusion range only as input); Incarnon flat damage (split, quantization scale, falloff exception data); damage-type conversion mods; physical mods; elemental mods; element combination order (slots, exilus position, riven bottom-up order, innate last, HCET, non-mod and non-combining sources: Nourish, field elements, Valence Formation, Unairu Impact, Phoenix Talons arithmetic); quantization (E-3, CD-02); base-damage bucket membership; faction bucket (members, which bane applies to which enemy faction and sub-faction) | CO term (`04`), proc weights (`02`), falloff curve (`06`) |
| `02-crit-status-multishot.md` | E-4 to E-10: crit chance (relative, absolute, floor, Piercing Navigator, combo hooks), crit multiplier (quantization, relative, absolute, field multipliers), tiers and Vigilante, headshots, weak points, head-crit, Sonar, body-part rules for radial and 1×-headshot weapons; status chance (relative, absolute, Tau multiplier, per pellet, Kohm division); proc count and type selection, immunity renormalization, ragdoll Impact; forced procs (weapon traits; stance procs as data from `07`); Hunter Munitions, Internal Bleeding, Hemorrhage, Magnetic Welt; multishot (floor, Acuity lock, beam merge CD-10, speargun throw) | status effects themselves (`03`), body-part data per enemy (`05`) |
| `03-status-effects.md` | every status type: duration, timers, caps (incl. enemy-specific caps CD-07 as rules), replacement; debuff functions (Viral, Magnetic, Corrosive strip, Heat strip ramp, Cold crit damage and Frozen, Puncture crit chance, Tau, Radiation, Impact, Void, Microwave, Knockdown, Lifted); DoT tick formula E-13 (seed, "+1", coefficients, elemental scaling, faction twice, status damage, crit, body part, inheritance list); Heat shared timer and inherit; Blast fuse and detonation; Gas; Electricity; Magnetic break proc; status duration and negative SD; distinct-status counting (what counts, CD-04 timing) | layer routing of ticks (`05`), expectation of n in EV (`08`) |
| `04-conditional-bonuses-and-buffs.md` | CO family (sources, catalogue per attack with behaviour, ratio and exclusions, `G_F` membership, timing, sequential pellets, stance ratio formula, CO in DoT seeds); `G` membership and the default rule; condition model (classes of T-02, triggers, stacks, durations, decay); Galvanized and other conditional mods; weapon and warframe arcanes; ability buffs and Helminth values (Strength at the caster); target vulnerabilities and debuff abilities (catalogue; stage math in `05`); auras and Coaction Drift; archon shards; focus; companion and squad buffs; extra hits (E-14) incl. Melee Duplicate; stealth and finishers; riven stat to bucket mapping and the riven value formula; set bonus effects; custom stats (6.3) | legality of builds (`09`), enemy-side formula (`05`) |
| `05-enemies.md` | enemy data model (incl. status immunities and per-enemy status caps as data); faction damage-type table and overrides; level scaling (NP-2), Steel Path (incl. flat bonuses), Eximus, overguard scaling, party scaling; armor (DR, clamp, floor, E-15 strip combination incl. ability strips, Corrosive Projection, flat base-armor strips, Hydroid); per-type minimum 1; shields (Toxin bypass and exceptions, gate, recharge for TTK); overguard (modifiers, caps, Fortifier); health (Viral stage); vulnerability stage; innate DR; unique DR (attenuation, flat DR, adaptation, Orphix) and T-28; layer order and overflow (CD-08); body-part data and head-crit capability supplied to `02` | status values (`03`), TTK procedure (`08`) |
| `06-ranged-firing-model.md` | the ranged schedule generator used by both evaluators; fire rate (bow ×2, semi-auto cap, Cannonade lock, floor); burst; charge (incl. partial charges); beams (ticks, ammo, ramp); spool (incl. Kohm pellets); shotguns; AoE and falloff (Φ(d), progenitor falloff, Incarnon-flat exception, radial headshot rule); parent and child attacks; magazine, ammo cost, efficiency, infinite ammo; reload (standard, per shell, battery, magazine-1, Vectis, Epitaph); first/last-shot schedule; sniper combo (count, multiplier, decay); kitgun assembly; arch-guns; exalted ranged weapons; companion weapons; Incarnon form (magazine, switch time); duty ρ per class | crit/status math (`02`), CO ratios of child projectiles (`04`) |
| `07-melee.md` | the melee schedule generator; attack speed; stance combos (durations, hit multipliers, physical bonuses, forced procs, CO ratio data) and stanceless defaults; combo counter (gain, tiers, caps, duration, decay, Initial Combo, regeneration); heavy attacks (damage, heavy and combo multipliers, Incarnon-flat exception, crit doubling, HAE, wind-up, Tennokai); slams and heavy slams; slide attacks; glaives; gunblades; whips; zaw assembly; exalted melee and ability combo (T-11); melee-only multipliers in `G` | arcane values (`04`), stealth formula (`04`) |
| `08-evaluation-and-outputs.md` | EV algorithm (section 4.2 in full: Poisson stack model, independence, window walk, Heat window mean, sequential-pellet n, Little's law); MC simulator (event set and order, RNG choice and keying, two passes); layer views; TTK (layered, horizon, extrapolation, EV approximation); every output of section 4.5 with exact formulas; attack groups and "combine attack effects"; compatibility profile and the oracle procedure (tolerances, oracle-safe scenarios); EV-versus-MC tolerance; optimizer metric definitions (DPS variants, TTK, max hit), not the search | game mechanics (all other documents) |
| `09-build-rules.md` | slot layout per equipment class; mod compatibility (type tree, tags); incompatibility groups; duplicate rules; capacity, drain, polarity, Forma, Catalyst, overlevelled weapons, aura and stance capacity; exilus; arcane slots (weapon, warframe, arch-gun pair, kitgun and zaw extra slots; duplicates); archon shard slots; Helminth damage-buff restriction; set-bonus counting (active weapon since U38.5); riven legality (classes, trait counts and signs, elements limit, splices, one per weapon, disposition 1.2, value ranges for validation); Incarnon tiers (one perk per tier, variant values); valence fusion range; mod rank scaling | effect values (`04`), bucket math (owners above) |

### 7.2 Ownership of cross-cutting items

| Item | Owner | Consumers |
|---|---|---|
| Faction bucket F | `01` | `03` (second dip), `04` (sources) |
| CO term, `G_F`, `G` | `04` | `01` (Y), `03` (seed) |
| Distinct-status count n: definition | `03` | `04`; expectation `08` |
| Proc weights, proc counts, Hunter Munitions | `02` | `03`, `08` |
| Puncture crit chance and Cold crit damage values | `03` | `02` (E-4, E-5 placement) |
| Heat strip and Corrosive strip values | `03` | `05` (E-15) |
| Viral and Magnetic multipliers | `03` (values) | `05` (stage, layers) |
| Body-part multipliers and head-crit capability | `05` (data) | `02` (E-7) |
| Vulnerability catalogue | `04` | `05` (stage) |
| Falloff Φ(d) | `06` | `01` (`G_F`), `04` |
| Stance data (hits, procs, physical, CO ratios) | `07` | `01`, `02`, `04` |
| Kitgun and zaw stat assembly | `06`, `07` | `01` |
| Incarnon perks | effects: the bucket owner; selection: `09` | |
| Rivens | value and bucket: `04`; legality: `09` | |
| Set bonuses | effects: `04`; counting: `09` | |
| Shot and hit schedules | `06`, `07` | `08` |
| Status immunities and per-enemy status caps (data) | `05` | `02` (weights), `03` (caps rule CD-07) |

---

## 8. Open questions that block exact accuracy

Each question names the evidence that would settle it. In-game tests use the Simulacrum unless stated; record
results in `research/test-cases/` with the build, enemy, level and observed damage numbers.

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-001 | In Adding-CO attacks, do faction, crit, body part and elemental composition multiply the CO term? | yes (E-11) | Braton Prime with Serration + Galvanized Aptitude (2 stacks) vs a Lancer carrying 1-2 statuses applied by another weapon; compare non-crit and crit numbers with Primed Bane on and off against both hypotheses |
| OQ-002 | Is the CO term inside the quantization scale, and does the Incarnon flat enter the scale? | no; yes (B' scale) | Furis Incarnon (+28 flat) with and without Galvanized Shot on a mixed-type configuration where the two hypotheses round differently |
| OQ-003 | Proc count rule for SC > 1: `floor + Bernoulli(frac)`? | yes | single-element weapon at exactly SC 1.5; count stacks added per pellet over 200 shots (expect only 1 or 2) |
| OQ-004 | Are proc weights computed on quantized types? | yes | Nagantaka Prime (Impact share quantizes to 0): 500 hits, any Impact proc disproves |
| OQ-005 | Rounding of fractional status durations (DoT ticks and debuffs) | ticks `floor(6(1 + SD))`; debuffs continuous | Lasting Sting at a rank giving +55 % SD: count Slash ticks (9 expected) and time Corrosive expiry (12.4 s expected) |
| OQ-006 | Same weapon, later Heat procs: faction once (inherit rule) or twice? | twice | Heat-only weapon with Primed Bane vs Grineer: compare the Heat tick before and after the second proc joins |
| OQ-007 | Do Blast-element bonuses scale Blast procs? | no | Blast proc tick with and without a Blast-element source (Valence Formation set to Blast or a riven splice) |
| OQ-008 | Does the 4-stack cap of status-capped enemies limit DoT instances? | yes | Slash procs on an Acolyte or Lich: count simultaneous Bleed numbers above 4 |
| OQ-009 | Overguard level input `L − 1` or `L − BaseLevel`; Steel Path on overguard | `L − 1`; no SP factor | Eximus Heavy Gunner (BaseLevel 8) at level 58: overguard 118,710 vs 105,506; repeat on Steel Path |
| OQ-010 | Eximus base health variable: `L − BaseLevel` or absolute level | `L − BaseLevel` | Eximus Heavy Gunner level 58 health 82,305 (d = 50) vs the absolute-level reading |
| OQ-011 | Overflow at overguard and shield breaks; DoT ticks during the shield gate; DoT on overguard | CD-08 | one-shot tests on a shielded Corpus unit and an Eximus with exact damage numbers |
| OQ-012 | Semi-auto cap value and duplex cadence (Tigris) | 10 /s; per-shot FR | frame-timed fire-rate measurement at high FR with the accessibility auto-fire option |
| OQ-013 | Beam ramp shape and per-weapon start values | linear, 0.2 | damage numbers over the first 0.6 s on a fresh target (Amprex, Phage, Convectrix) |
| OQ-014 | Burst delay division by `(1 + ΣFR)` | yes for positive ΣFR | Burston with Speed Trigger: measured burst cadence vs 3 / (0.125 + 0.076) shots/s |
| OQ-015 | Magazine-1 weapons other than Vectis: reload delay of 1/FR? | no delay | Tonkor cycle time measurement |
| OQ-016 | Head-crit doubling of absolute crit damage; placement of field crit multipliers (Electric Shield) for tiers ≥ 2 | doubles total CDM; field multiplier on CDM | orange crits through Electric Shield; headcrits with Arcane Crepuscular |
| OQ-017 | Do the Puncture crit-chance and Cold crit-damage bonuses carry into DoT seeded by the hit? | yes | Slash ticks from a crit-capped weapon vs targets with 0 and 5 Puncture stacks |
| OQ-018 | Hunter Munitions: one roll per crit instance independent of tier? | yes | count Slash procs from a 0-status weapon at CC 1.5 vs 0.5 |
| OQ-019 | Riven faction stat with Bane: additive? | additive | faction riven + Primed Bane vs the matching faction |
| OQ-020 | Is Phoenix Talons' flat physical scaled by base-damage mods? | yes | Dual Ichor with and without Serration-type mods, numbers per type |
| OQ-021 | Heavy-attack swing times and stanceless neutral loop durations | `07` fallback | frame timing per melee class (data gap in `Module:Stances/data`) |
| OQ-022 | U40 attenuation parameters per boss (MDPI, MDPS model) | documented values only | patch notes or dev statements; damage-number tests on Eidolons, Archons, Hemocyte |
| OQ-023 | DoT "+1" accumulator and f32 tick arithmetic | +1 on, f64 | low-damage Heat proc test: `(40 + 1) × 0.5 × 1.6 = 32.8` vs 32.0 |
| OQ-024 | Combination position of non-mod elements (Violet shard, Nourish, Valence, Crucible Overflow) and of the exilus slot | Violet per HCET with innate; exilus after slot 8 | Tenet weapon with Violet shard, innate element and one mod element: observe the combined result |
| OQ-025 | Electricity and Gas proc headshot behaviour (1× vs Deadhead counted twice) | 1× base, HS bonuses apply once | Electricity proc numbers on headshots with and without Deadhead |
| OQ-026 | Game uses f32 scaling with the documented operation order; armor `floor` | yes | high-level health readout (level 9999) vs the f32 and f64 predictions |
| OQ-027 | Elemental mods scale off the progenitor-inclusive base | yes | Kuva weapon at 60 % valence with one elemental mod: per-type numbers |
| OQ-028 | Do sibling pellets of one shot see each other's Corrosive, Heat, Cold and Puncture procs? | no (Viral analogy) | shotgun first-shot test on a fresh armored target vs single-pellet shots |
| OQ-029 | Complete list of enemies whose shields block Toxin | Treasurer, Hounds | wiki `protoshield` template parameter or in-game checks |
| OQ-030 | How the Incarnon flat splits across damage types | proportional to `B0_t + V_t` | Incarnon weapon with mixed IPS: per-type numbers before and after the evolution |
